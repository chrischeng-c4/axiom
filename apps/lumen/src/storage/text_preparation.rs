//! Record-owned Text preparation runs before the apply lease. Row keys use
//! the original item ordinal, so duplicate cells retain arrival-order meaning.

use super::*;
use crate::log_entry::RaftLogEntry;
use crate::types::{IndexItem, IndexRequest, MAX_INDEX_BATCH_SIZE};
use crate::wal::fast_index_scanner::{FastIndexScanner, FastIndexValue};
use anyhow::bail;

pub(super) const TEXT_SCRATCH_BYTES: usize = 8 * 1024 * 1024;

#[derive(Default)]
pub(super) struct PreparedTextRows {
    epoch: u64,
    collection: Option<(String, u64, u32)>,
    rows: BTreeMap<usize, BTreeMap<String, Arc<staged_text_row::StagedTextRow>>>,
    /// Reader metadata stays live through apply and the retained pending
    /// charge.  It is distinct from the bounded staging workspace, which the
    /// caller releases before it retains the private Index entry.
    retained_reader_bytes: usize,
    /// Extra workspace above `TEXT_SCRATCH_BYTES` accepted while staging.
    /// This is temporary even though it was charged to the reservation during
    /// preparation, so the caller can shrink it before apply.
    scratch_growth_bytes: usize,
}
impl PreparedTextRows {
    pub(super) fn get(
        &self,
        ordinal: usize,
        field: &str,
    ) -> Option<&Arc<staged_text_row::StagedTextRow>> {
        self.rows.get(&ordinal)?.get(field)
    }
    pub(super) fn matches(&self, engine: &Engine) -> bool {
        if self.epoch != engine.capture_barrier.epoch() {
            return false;
        }
        let Some((name, generation, version)) = &self.collection else {
            return true;
        };
        engine.state.read().ok().is_some_and(|state| {
            state.collections.get(name).is_some_and(|coll| {
                coll.collection_generation == *generation && coll.version == *version
            })
        })
    }
    pub(super) fn retained_reader_bytes(&self) -> usize {
        self.retained_reader_bytes
    }
    pub(super) fn scratch_growth_bytes(&self) -> usize {
        self.scratch_growth_bytes
    }
}

fn collection_id(entry: &RaftLogEntry) -> Option<&str> {
    match entry {
        RaftLogEntry::Index { collection_id, .. }
        | RaftLogEntry::ReplaceDocs { collection_id, .. } => Some(collection_id),
        _ => None,
    }
}
fn visit_text_values(entry: &RaftLogEntry, mut visit: impl FnMut(usize, &str, &str)) {
    match entry {
        RaftLogEntry::Index { req, .. } => {
            for (ordinal, item) in req.items.iter().enumerate() {
                if let FieldValue::String(value) = &item.value {
                    visit(ordinal, &item.field, value);
                }
            }
        }
        RaftLogEntry::ReplaceDocs { req, .. } => {
            for (ordinal, doc) in req.docs.iter().enumerate() {
                for (field, value) in &doc.fields {
                    if let FieldValue::String(value) = value {
                        visit(ordinal, field, value);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Bound the owned shell for a borrowed fast-Index Text command.  The source
/// value bytes stay in the scanner; only identifiers, empty value shells, row
/// map nodes, and the small analyzer lookup may allocate here.
///
/// Callers reserve this plus [`TEXT_SCRATCH_BYTES`] before calling
/// `prepare_borrowed_text_rows`.  The method repeats the assertion before it
/// allocates, so a new caller cannot accidentally build metadata while it
/// holds a state lock without first pricing it.
pub(super) fn borrowed_text_metadata_bound(scanner: &FastIndexScanner<'_>) -> Result<usize> {
    anyhow::ensure!(
        scanner.cost().item_count <= MAX_INDEX_BATCH_SIZE,
        "borrowed Text preparation exceeds Index item limit"
    );
    borrowed_field_metadata_bound(scanner)
}

/// Price borrowed Text metadata after a command-aware replacement planner has
/// validated its document and flattened-field limits. It keeps the same
/// per-item arithmetic and overflow checks as the Index entry point.
pub(super) fn borrowed_field_metadata_bound(scanner: &FastIndexScanner<'_>) -> Result<usize> {
    // BTreeMap has no reserve API.  Price three independent maps (analyzers,
    // outer rows, and inner rows) at a deliberately conservative node bound.
    const BTREE_NODE_BOUND: usize = 256;
    const MAP_POPULATIONS: usize = 3;
    const ITEM_SHELL_BOUND: usize = std::mem::size_of::<IndexItem>() + 128;
    let mut bytes = scanner
        .collection_id()
        .len()
        .checked_add(scanner.request_id().map_or(0, str::len))
        .and_then(|n| n.checked_add(std::mem::size_of::<IndexRequest>()))
        .ok_or_else(|| anyhow!("borrowed Text metadata reservation overflow"))?;
    for item in scanner.items() {
        bytes = bytes
            .checked_add(item.external_id.len())
            .and_then(|n| n.checked_add(item.field.len().checked_mul(4)?))
            .and_then(|n| n.checked_add(ITEM_SHELL_BOUND))
            .and_then(|n| n.checked_add(BTREE_NODE_BOUND.checked_mul(MAP_POPULATIONS)?))
            .ok_or_else(|| anyhow!("borrowed Text metadata reservation overflow"))?;
    }
    Ok(bytes)
}

/// The largest temporary normalized token produced before the row writer can
/// return `RequiredTextRowWorkspace`. Whitespace splitting and punctuation
/// trimming match `index_text::for_whitespace_lower_cow`; no output string is
/// built. Pricing only the largest token preserves valid large inputs made of
/// many small words.
fn lowercase_token_workspace_bound(input: &str) -> Result<usize> {
    let mut largest = 0usize;
    for raw in input.split(char::is_whitespace) {
        let token = raw.trim_matches(|character: char| !character.is_alphanumeric());
        if token.is_empty() {
            continue;
        }
        // `to_lowercase` can expand Unicode. Three source bytes plus a small
        // String allocation allowance is the existing owned-preparation rule,
        // applied to one simultaneous token rather than the whole input.
        let bound = token
            .len()
            .checked_mul(3)
            .and_then(|bytes| bytes.checked_add(64))
            .ok_or_else(|| anyhow!("borrowed Text lowercase workspace overflow"))?;
        largest = largest.max(bound);
    }
    Ok(largest)
}

/// Long whitespace tokens stream to private files. Only the shorter tokens
/// use an owned lowercase buffer; its bound stays fixed as the row grows.
fn bounded_whitespace_workspace(input: &str) -> Result<usize> {
    let mut largest = 0usize;
    for raw in input.split_whitespace() {
        let token = raw.trim_matches(|c: char| !c.is_alphanumeric());
        if token.is_empty() {
            continue;
        }
        let source = token
            .len()
            .min(super::large_text_row::LARGE_TOKEN_SOURCE_BYTES);
        let bound = source
            .checked_mul(3)
            .and_then(|n| n.checked_add(64))
            .ok_or_else(|| anyhow!("Text lowercase workspace overflow"))?;
        largest = largest.max(bound);
    }
    Ok(largest)
}

/// The feature-off Jieba stream emits CJK bigrams from stack buffers and sends
/// only each non-CJK run to whitespace_lower. Do not price a CJK run as one
/// lowercase String.
#[cfg(not(feature = "jieba"))]
fn fallback_jieba_token_workspace_bound(input: &str) -> Result<usize> {
    let text = input.trim();
    let mut largest = 0usize;
    let mut non_cjk_start = 0usize;
    let mut in_cjk = false;
    for (offset, character) in text.char_indices() {
        if crate::jieba_fallback_stream::is_cjk_char(character) {
            if !in_cjk {
                largest = largest.max(lowercase_token_workspace_bound(
                    &text[non_cjk_start..offset],
                )?);
                in_cjk = true;
            }
        } else if in_cjk {
            in_cjk = false;
            non_cjk_start = offset;
        }
    }
    if !in_cjk {
        largest = largest.max(lowercase_token_workspace_bound(&text[non_cjk_start..])?);
    }
    Ok(largest)
}

/// Dictionary Jieba lowercases each raw word without whitespace-token
/// punctuation trimming. jieba-rs splits at whitespace, so one emitted word is
/// contained in one non-whitespace input span.
#[cfg(feature = "jieba")]
fn dictionary_jieba_token_workspace_bound(input: &str) -> Result<usize> {
    let mut largest = 0usize;
    for span in input.split(char::is_whitespace) {
        if span.is_empty() {
            continue;
        }
        let bound = span
            .len()
            .checked_mul(3)
            .and_then(|bytes| bytes.checked_add(64))
            .ok_or_else(|| anyhow!("borrowed Text lowercase workspace overflow"))?;
        largest = largest.max(bound);
    }
    Ok(largest)
}

fn borrowed_text_pre_stage_workspace_bound(
    scanner: &FastIndexScanner<'_>,
    mut analyzer_for: impl FnMut(&str) -> Option<Analyzer>,
    mut selected: impl FnMut(usize) -> bool,
) -> Result<usize> {
    let mut lowercase = 0usize;
    #[cfg(feature = "jieba")]
    let mut dictionary_route = false;
    for (ordinal, item) in scanner.items().enumerate() {
        if !selected(ordinal) {
            continue;
        }
        let FastIndexValue::String(input) = item.value else {
            continue;
        };
        let Some(analyzer) = analyzer_for(item.field) else {
            continue;
        };
        match analyzer {
            Analyzer::WhitespaceLower => {
                lowercase = lowercase.max(bounded_whitespace_workspace(input)?);
            }
            #[cfg(not(feature = "jieba"))]
            Analyzer::Jieba => {
                lowercase = lowercase.max(fallback_jieba_token_workspace_bound(input)?);
            }
            #[cfg(feature = "jieba")]
            Analyzer::Jieba => {
                lowercase = lowercase.max(dictionary_jieba_token_workspace_bound(input)?);
                dictionary_route = true;
            }
            Analyzer::Ngram => {}
        }
    }
    #[cfg(feature = "jieba")]
    {
        if dictionary_route {
            lowercase = lowercase
                .checked_add(super::jieba_disk_route::ROUTE_CACHE_BYTES)
                .ok_or_else(|| anyhow!("borrowed Text pre-stage workspace overflow"))?;
        }
    }
    Ok(lowercase)
}

#[derive(Debug)]
pub(super) struct RequiredBorrowedTextWorkspace {
    pub(super) required_bytes: usize,
}

impl std::fmt::Display for RequiredBorrowedTextWorkspace {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "borrowed Text pre-stage workspace requires {} bytes",
            self.required_bytes
        )
    }
}
impl std::error::Error for RequiredBorrowedTextWorkspace {}

/// The plan captured a Text field, then a restore or schema change replaced
/// it before staging started. This is a retry signal, never a client error.
#[derive(Debug)]
pub(super) struct StaleBorrowedTextPreparation;
impl std::fmt::Display for StaleBorrowedTextPreparation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("borrowed Text preparation became stale")
    }
}
impl std::error::Error for StaleBorrowedTextPreparation {}

impl Engine {
    /// Convert a validated borrowed fast Index command into a private, small
    /// apply entry and staged Text rows.  This entry is an apply-only adapter:
    /// it must never become an AOF source because Text values are represented
    /// by empty strings paired with their staged rows.
    ///
    /// `None` is an internal fallback signal.  An existing non-Text field
    /// needs the owned path.  Unknown fields intentionally remain in the
    /// private entry so live Index validation retains its valid-prefix order.
    pub(super) fn prepare_borrowed_text_rows(
        &self,
        scanner: &FastIndexScanner<'_>,
        reservation: &mut record_admission::RecordReservation,
    ) -> Result<Option<(RaftLogEntry, PreparedTextRows)>> {
        let metadata = borrowed_text_metadata_bound(scanner)?;
        let initial_required = metadata
            .checked_add(TEXT_SCRATCH_BYTES)
            .ok_or_else(|| anyhow!("borrowed Text preparation reservation overflow"))?;
        anyhow::ensure!(
            reservation.bytes() >= initial_required,
            "borrowed Text metadata and scratch were not reserved"
        );

        // Capture only the referenced Text analyzer metadata.  Do not clone
        // the schema, and do not grow staging workspace while this lock lives.
        let (mut prepared, analyzers) = {
            let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
            // A missing collection has no stable Text analyzer.  Keep the raw
            // source path so a concurrent create cannot make an empty private
            // placeholder value observable at apply.
            let Some(coll) = state.collections.get(scanner.collection_id()) else {
                return Ok(None);
            };
            let mut analyzers = BTreeMap::new();
            for item in scanner.items() {
                match coll.fields.get(item.field) {
                    Some(FieldIndex::Text { analyzer, .. }) => {
                        analyzers.insert(item.field.to_owned(), *analyzer);
                    }
                    Some(_) => return Ok(None),
                    None => {}
                }
            }
            (
                PreparedTextRows {
                    epoch: self.capture_barrier.epoch(),
                    collection: Some((
                        scanner.collection_id().to_owned(),
                        coll.collection_generation,
                        coll.version,
                    )),
                    rows: BTreeMap::new(),
                    retained_reader_bytes: 0,
                    scratch_growth_bytes: 0,
                },
                analyzers,
            )
        };

        let pre_stage_workspace = borrowed_text_pre_stage_workspace_bound(
            scanner,
            |field| analyzers.get(field).copied(),
            |_| true,
        )?;
        let required = metadata
            .checked_add(TEXT_SCRATCH_BYTES)
            .and_then(|bytes| bytes.checked_add(pre_stage_workspace))
            .ok_or_else(|| anyhow!("borrowed Text preparation reservation overflow"))?;
        if reservation.bytes() < required {
            return Err(RequiredBorrowedTextWorkspace {
                required_bytes: required,
            }
            .into());
        }

        let mut items = Vec::with_capacity(scanner.cost().item_count);
        for (ordinal, item) in scanner.items().enumerate() {
            let value = match item.value {
                // The staged row is the value for a declared Text field.  The
                // empty String is only an internal placeholder for the normal
                // apply function; it must never be persisted or encoded.
                FastIndexValue::String(input) => {
                    if let Some(analyzer) = analyzers.get(item.field).copied() {
                        let (row, reader_bytes, scratch_growth) =
                            self.stage_text_row(input, analyzer, reservation)?;
                        prepared.retained_reader_bytes = prepared
                            .retained_reader_bytes
                            .checked_add(reader_bytes)
                            .ok_or_else(|| anyhow!("borrowed Text reader metadata overflow"))?;
                        prepared.scratch_growth_bytes = prepared
                            .scratch_growth_bytes
                            .checked_add(scratch_growth)
                            .ok_or_else(|| anyhow!("borrowed Text workspace overflow"))?;
                        prepared
                            .rows
                            .entry(ordinal)
                            .or_default()
                            .insert(item.field.to_owned(), Arc::new(row));
                        self.metrics.text_row_stage_rows_total.incr();
                        self.metrics
                            .text_row_stage_input_bytes_total
                            .add(input.len() as u64);
                    }
                    FieldValue::String(String::new())
                }
                // Keep the original type for a Text mismatch.  In particular,
                // do not turn a vector or string list into a String: live
                // validation must still report the same type mismatch.
                FastIndexValue::Number(value) => FieldValue::Number(value),
                FastIndexValue::Vector { .. } => FieldValue::Vector(Vec::new()),
                FastIndexValue::StringList(_) => FieldValue::StringList(Vec::new()),
            };
            items.push(IndexItem {
                external_id: item.external_id.to_owned(),
                field: item.field.to_owned(),
                value,
                version: item.version,
            });
        }
        Ok(Some((
            RaftLogEntry::Index {
                collection_id: scanner.collection_id().to_owned(),
                req: IndexRequest {
                    items,
                    request_id: scanner.request_id().map(str::to_owned),
                },
            },
            prepared,
        )))
    }

    /// Stage only the Text actions accepted by the unified borrowed planner.
    /// The caller supplies original ordinals from that single ledger, so an
    /// invalid later item cannot cause staging beyond the live valid prefix.
    pub(super) fn prepare_borrowed_text_rows_for_actions(
        &self,
        scanner: &FastIndexScanner<'_>,
        actions: &BTreeMap<usize, String>,
        live_baseline: usize,
        reservation: &mut record_admission::RecordReservation,
    ) -> Result<PreparedTextRows> {
        // `live_baseline` already prices the planner, scalar adapter and this
        // method's action/analyzer/row metadata. Scratch is additive: it must
        // never reuse bytes whose owners are still live.
        let initial_required = live_baseline
            .checked_add(TEXT_SCRATCH_BYTES)
            .ok_or_else(|| anyhow!("borrowed Text preparation reservation overflow"))?;
        if reservation.bytes() < initial_required {
            return Err(RequiredBorrowedTextWorkspace {
                required_bytes: initial_required,
            }
            .into());
        }
        let (mut prepared, analyzers) = {
            let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
            let Some(coll) = state.collections.get(scanner.collection_id()) else {
                return Err(StaleBorrowedTextPreparation.into());
            };
            let mut analyzers = BTreeMap::new();
            for field in actions.values() {
                let Some(FieldIndex::Text { analyzer, .. }) = coll.fields.get(field) else {
                    return Err(StaleBorrowedTextPreparation.into());
                };
                analyzers.insert(field.clone(), *analyzer);
            }
            (
                PreparedTextRows {
                    epoch: self.capture_barrier.epoch(),
                    collection: Some((
                        scanner.collection_id().to_owned(),
                        coll.collection_generation,
                        coll.version,
                    )),
                    rows: BTreeMap::new(),
                    retained_reader_bytes: 0,
                    scratch_growth_bytes: 0,
                },
                analyzers,
            )
        };
        let pre_stage_workspace = borrowed_text_pre_stage_workspace_bound(
            scanner,
            |field| analyzers.get(field).copied(),
            |ordinal| actions.contains_key(&ordinal),
        )?;
        let required = live_baseline
            .checked_add(TEXT_SCRATCH_BYTES)
            .and_then(|bytes| bytes.checked_add(pre_stage_workspace))
            .ok_or_else(|| anyhow!("borrowed Text preparation reservation overflow"))?;
        if reservation.bytes() < required {
            return Err(RequiredBorrowedTextWorkspace {
                required_bytes: required,
            }
            .into());
        }
        for (ordinal, item) in scanner.items().enumerate() {
            let Some(field) = actions.get(&ordinal) else {
                continue;
            };
            debug_assert_eq!(field, item.field);
            let FastIndexValue::String(input) = item.value else {
                bail!("planned Text action lost its string value")
            };
            let analyzer = analyzers[field];
            let (row, reader_bytes, scratch_growth) =
                self.stage_text_row(input, analyzer, reservation)?;
            prepared.retained_reader_bytes = prepared
                .retained_reader_bytes
                .checked_add(reader_bytes)
                .ok_or_else(|| anyhow!("borrowed Text reader metadata overflow"))?;
            prepared.scratch_growth_bytes = prepared
                .scratch_growth_bytes
                .checked_add(scratch_growth)
                .ok_or_else(|| anyhow!("borrowed Text workspace overflow"))?;
            prepared
                .rows
                .entry(ordinal)
                .or_default()
                .insert(field.clone(), Arc::new(row));
            self.metrics.text_row_stage_rows_total.incr();
            self.metrics
                .text_row_stage_input_bytes_total
                .add(input.len() as u64);
        }
        Ok(prepared)
    }

    pub(super) fn prepared_text_bound(
        &self,
        entry: &RaftLogEntry,
    ) -> Result<usize, record_admission::RecordAdmissionError> {
        use record_admission::RecordAdmissionError as Error;
        let state = self
            .state
            .read()
            .map_err(|_| Error::Preparation("state poisoned".into()))?;
        let cost = crate::change_record_cost::estimate_record_prepared_text(
            entry,
            &EngineCostContext { state: &state },
        )
        .map_err(Error::NeedsPreparation)?;
        let mut metadata = Some(0usize);
        let mut normalized_token = 0usize;
        #[cfg(feature = "jieba")]
        let mut dictionary_route_bytes = 0usize;
        #[cfg(not(feature = "jieba"))]
        let dictionary_route_bytes = 0usize;
        let coll = collection_id(entry).and_then(|name| state.collections.get(name));
        visit_text_values(entry, |_, field, input| {
            if let Some(FieldIndex::Text { analyzer, .. }) =
                coll.and_then(|coll| coll.fields.get(field))
            {
                #[cfg(feature = "jieba")]
                if *analyzer == Analyzer::Jieba {
                    dictionary_route_bytes = super::jieba_disk_route::ROUTE_CACHE_BYTES;
                }
                metadata = metadata.and_then(|n| {
                    field
                        .len()
                        .checked_mul(4)
                        .and_then(|name| n.checked_add(name))
                        .and_then(|n| n.checked_add(4096))
                });
                match analyzer {
                    Analyzer::WhitespaceLower => {
                        normalized_token = normalized_token
                            .max(bounded_whitespace_workspace(input).unwrap_or(usize::MAX));
                    }
                    Analyzer::Jieba => {
                        normalized_token =
                            normalized_token.max(input.len().saturating_mul(3).saturating_add(64));
                    }
                    Analyzer::Ngram => {}
                }
            }
        });
        cost.active
            .checked_add(cost.frozen)
            .and_then(|n| n.checked_add(cost.prepublish))
            .and_then(|n| n.checked_add(metadata?))
            .and_then(|n| n.checked_add(TEXT_SCRATCH_BYTES))
            .and_then(|n| n.checked_add(normalized_token))
            .and_then(|n| n.checked_add(dictionary_route_bytes))
            .ok_or(Error::Overflow)
    }

    pub(super) fn prepare_text_rows(
        &self,
        entry: &RaftLogEntry,
        reservation: &mut record_admission::RecordReservation,
    ) -> Result<PreparedTextRows> {
        let (mut prepared, analyzers) = {
            let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
            let coll = collection_id(entry)
                .and_then(|name| state.collections.get(name).map(|coll| (name, coll)));
            let mut analyzers = BTreeMap::new();
            if let Some((_, coll)) = coll {
                for (name, field) in &coll.fields {
                    if let FieldIndex::Text { analyzer, .. } = field {
                        analyzers.insert(name.clone(), *analyzer);
                    }
                }
            }
            (
                PreparedTextRows {
                    epoch: self.capture_barrier.epoch(),
                    collection: coll.map(|(name, coll)| {
                        (name.to_owned(), coll.collection_generation, coll.version)
                    }),
                    rows: BTreeMap::new(),
                    retained_reader_bytes: 0,
                    scratch_growth_bytes: 0,
                },
                analyzers,
            )
        };
        let mut failure = None;
        visit_text_values(entry, |ordinal, field, input| {
            if failure.is_some() {
                return;
            }
            let Some(analyzer) = analyzers.get(field).copied() else {
                return;
            };
            let result = self.stage_text_row(input, analyzer, reservation);
            match result {
                Ok((row, reader_bytes, scratch_growth)) => {
                    prepared.retained_reader_bytes =
                        match prepared.retained_reader_bytes.checked_add(reader_bytes) {
                            Some(bytes) => bytes,
                            None => {
                                failure = Some(anyhow!("owned Text reader metadata overflow"));
                                return;
                            }
                        };
                    prepared.scratch_growth_bytes =
                        match prepared.scratch_growth_bytes.checked_add(scratch_growth) {
                            Some(bytes) => bytes,
                            None => {
                                failure = Some(anyhow!("owned Text workspace overflow"));
                                return;
                            }
                        };
                    self.metrics.text_row_stage_rows_total.incr();
                    self.metrics
                        .text_row_stage_input_bytes_total
                        .add(row.input_bytes() as u64);
                    prepared
                        .rows
                        .entry(ordinal)
                        .or_default()
                        .insert(field.to_owned(), Arc::new(row));
                }
                Err(error) => failure = Some(error),
            }
        });
        if let Some(error) = failure {
            return Err(error);
        }
        Ok(prepared)
    }

    /// Stage one borrowed or owned Text value after the state lock is gone.
    /// Reader metadata and expanded workspace are charged to the record before
    /// their allocation becomes observable to apply.
    fn stage_text_row(
        &self,
        input: &str,
        analyzer: Analyzer,
        reservation: &mut record_admission::RecordReservation,
    ) -> Result<(staged_text_row::StagedTextRow, usize, usize)> {
        let mut scratch = TEXT_SCRATCH_BYTES;
        let mut scratch_growth = 0usize;
        loop {
            let mut retained_reader_bytes = 0usize;
            let mut transient_bytes = 0usize;
            let result = staged_text_row::StagedTextRow::stage_charged(
                input,
                analyzer,
                scratch,
                |allocation| {
                    use staged_text_row::StageAllocation;
                    let (StageAllocation::Reader(bytes) | StageAllocation::Workspace(bytes)) =
                        allocation;
                    self.request_pending_checkpoint();
                    reservation
                        .grow_preparation(bytes)
                        .map_err(record_admission::RecordAdmissionError::Capacity)?;
                    match allocation {
                        StageAllocation::Reader(_) => {
                            retained_reader_bytes = retained_reader_bytes
                                .checked_add(bytes)
                                .ok_or_else(|| anyhow!("Text reader metadata overflow"))?
                        }
                        StageAllocation::Workspace(_) => {
                            transient_bytes = transient_bytes
                                .checked_add(bytes)
                                .ok_or_else(|| anyhow!("Text transient workspace overflow"))?
                        }
                    }
                    Ok(())
                },
            );
            // stage_charged has dropped its private helper mappings and all
            // temporary allocations. A failed final reader has no live owner.
            let released = transient_bytes
                .checked_add(if result.is_err() {
                    retained_reader_bytes
                } else {
                    0
                })
                .ok_or_else(|| anyhow!("Text released workspace overflow"))?;
            reservation
                .release_preparation_workspace(released)
                .map_err(record_admission::RecordAdmissionError::Capacity)?;
            match result {
                Err(error)
                    if error
                        .downcast_ref::<crate::segment::text_row_stage::RequiredTextRowWorkspace>()
                        .is_some() =>
                {
                    let required = error
                        .downcast_ref::<crate::segment::text_row_stage::RequiredTextRowWorkspace>()
                        .expect("matched required Text workspace")
                        .required_bytes;
                    let delta = required.saturating_sub(scratch);
                    reservation
                        .grow_preparation(delta)
                        .map_err(record_admission::RecordAdmissionError::Capacity)?;
                    scratch_growth = scratch_growth
                        .checked_add(delta)
                        .ok_or_else(|| anyhow!("Text workspace overflow"))?;
                    scratch = required;
                }
                Ok(row) => return Ok((row, retained_reader_bytes, scratch_growth)),
                Err(error) => return Err(error),
            }
        }
    }
}

#[cfg(test)]
mod borrowed_tests {
    use super::*;
    use crate::types::{CreateCollectionRequest, FieldSpec, FieldType};
    use crate::wal::WalRecord;

    fn spec(field_type: FieldType, analyzer: Option<Analyzer>) -> FieldSpec {
        FieldSpec {
            field_type,
            analyzer,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    fn engine(fields: &[(&str, FieldType)]) -> Engine {
        let mut schema = BTreeMap::new();
        for (name, kind) in fields {
            schema.insert(
                (*name).to_owned(),
                spec(
                    *kind,
                    (*kind == FieldType::Text).then_some(Analyzer::WhitespaceLower),
                ),
            );
        }
        let engine = Engine::new();
        engine
            .create_collection("docs", CreateCollectionRequest { fields: schema })
            .unwrap();
        engine
    }

    fn record_reservation(
        engine: &Engine,
        scanner: &FastIndexScanner<'_>,
    ) -> record_admission::RecordReservation {
        let bytes = borrowed_text_metadata_bound(scanner)
            .unwrap()
            .checked_mul(2)
            .and_then(|metadata| metadata.checked_add(TEXT_SCRATCH_BYTES))
            .unwrap();
        engine
            .wait_reserve_record_ram(&engine.record_ram_request_from_bound(bytes, 0))
            .unwrap()
    }

    fn encode(items: Vec<IndexItem>, request_id: Option<&str>) -> Vec<u8> {
        WalRecord::new(RaftLogEntry::Index {
            collection_id: "docs".into(),
            req: IndexRequest {
                items,
                request_id: request_id.map(str::to_owned),
            },
        })
        .encode()
        .unwrap()
    }

    #[test]
    fn borrowed_text_rows_match_owned_rows_and_keep_duplicate_ordinals_metadata() {
        let entry = RaftLogEntry::Index {
            collection_id: "docs".into(),
            req: IndexRequest {
                request_id: Some("request-7".into()),
                items: vec![
                    IndexItem {
                        external_id: "same".into(),
                        field: "body".into(),
                        value: FieldValue::String("alpha beta alpha".into()),
                        version: Some(4),
                    },
                    IndexItem {
                        external_id: "same".into(),
                        field: "body".into(),
                        value: FieldValue::String("beta gamma".into()),
                        version: Some(5),
                    },
                ],
            },
        };
        let bytes = WalRecord::new(entry.clone()).encode().unwrap();
        let scanner = FastIndexScanner::parse(&bytes).unwrap();
        let engine = engine(&[("body", FieldType::Text)]);

        let mut owned_reservation = engine
            .wait_reserve_record_ram(
                &engine
                    .record_ram_request_from_bound(engine.prepared_text_bound(&entry).unwrap(), 0),
            )
            .unwrap();
        let owned = engine
            .prepare_text_rows(&entry, &mut owned_reservation)
            .unwrap();
        let mut borrowed_reservation = record_reservation(&engine, &scanner);
        let Some((RaftLogEntry::Index { req, .. }, borrowed)) = engine
            .prepare_borrowed_text_rows(&scanner, &mut borrowed_reservation)
            .unwrap()
        else {
            panic!("Text-only command must use the borrowed preparation path")
        };

        assert_eq!(req.request_id.as_deref(), Some("request-7"));
        assert_eq!(req.items.len(), 2);
        assert_eq!(req.items[0].external_id, "same");
        assert_eq!(req.items[0].version, Some(4));
        assert_eq!(req.items[1].version, Some(5));
        assert!(matches!(&req.items[0].value, FieldValue::String(value) if value.is_empty()));
        assert_eq!(
            borrowed.retained_reader_bytes(),
            owned.retained_reader_bytes(),
        );
        assert_eq!(
            borrowed.scratch_growth_bytes(),
            owned.scratch_growth_bytes(),
        );
        for ordinal in 0..2 {
            let owned = owned.get(ordinal, "body").unwrap();
            let borrowed = borrowed.get(ordinal, "body").unwrap();
            assert_eq!(borrowed.doc_len(), owned.doc_len());
            assert_eq!(borrowed.indexed_bytes("same"), owned.indexed_bytes("same"));
            assert_eq!(
                borrowed.reader().text_dictionary_stats(),
                owned.reader().text_dictionary_stats(),
            );
        }
    }

    #[test]
    fn borrowed_text_wrong_value_type_keeps_the_type_and_has_no_staged_row() {
        let bytes = encode(
            vec![IndexItem {
                external_id: "one".into(),
                field: "body".into(),
                value: FieldValue::Vector(vec![1.0, 2.0]),
                version: None,
            }],
            None,
        );
        let scanner = FastIndexScanner::parse(&bytes).unwrap();
        let engine = engine(&[("body", FieldType::Text)]);
        let mut reservation = record_reservation(&engine, &scanner);
        let Some((RaftLogEntry::Index { req, .. }, rows)) = engine
            .prepare_borrowed_text_rows(&scanner, &mut reservation)
            .unwrap()
        else {
            panic!("a Text field with a wrong value type still needs live validation")
        };
        assert!(matches!(&req.items[0].value, FieldValue::Vector(values) if values.is_empty()));
        assert!(rows.get(0, "body").is_none());
    }

    #[test]
    fn borrowed_text_leaves_unknown_fields_for_live_valid_prefix_handling() {
        let bytes = encode(
            vec![
                IndexItem {
                    external_id: "one".into(),
                    field: "body".into(),
                    value: FieldValue::String("alpha".into()),
                    version: None,
                },
                IndexItem {
                    external_id: "one".into(),
                    field: "later".into(),
                    value: FieldValue::String("not validated here".into()),
                    version: None,
                },
            ],
            None,
        );
        let scanner = FastIndexScanner::parse(&bytes).unwrap();
        let engine = engine(&[("body", FieldType::Text)]);
        let mut reservation = record_reservation(&engine, &scanner);
        let Some((RaftLogEntry::Index { req, .. }, rows)) = engine
            .prepare_borrowed_text_rows(&scanner, &mut reservation)
            .unwrap()
        else {
            panic!("unknown field must remain for live validation")
        };
        assert_eq!(req.items[1].field, "later");
        assert!(matches!(&req.items[1].value, FieldValue::String(value) if value.is_empty()));
        assert!(rows.get(0, "body").is_some());
        assert!(rows.get(1, "later").is_none());
    }

    #[test]
    fn borrowed_text_falls_back_when_an_existing_field_is_not_text() {
        let bytes = encode(
            vec![IndexItem {
                external_id: "one".into(),
                field: "tag".into(),
                value: FieldValue::String("value".into()),
                version: None,
            }],
            None,
        );
        let scanner = FastIndexScanner::parse(&bytes).unwrap();
        let engine = engine(&[("tag", FieldType::Keyword)]);
        let mut reservation = record_reservation(&engine, &scanner);
        assert!(engine
            .prepare_borrowed_text_rows(&scanner, &mut reservation)
            .unwrap()
            .is_none());
    }

    #[test]
    fn validated_replace_field_metadata_allows_32_by_33_flattened_fields() {
        const REPLACE_DOCUMENTS: usize = 32;
        const FIELDS_PER_DOCUMENT: usize = 33;
        let count = REPLACE_DOCUMENTS * FIELDS_PER_DOCUMENT;
        let bytes = encode(
            (0..count)
                .map(|ordinal| IndexItem {
                    external_id: format!("doc-{}", ordinal / FIELDS_PER_DOCUMENT),
                    field: format!("field-{}", ordinal % FIELDS_PER_DOCUMENT),
                    value: FieldValue::String("small".into()),
                    version: None,
                })
                .collect(),
            None,
        );
        let scanner = FastIndexScanner::parse(&bytes).unwrap();
        assert!(borrowed_text_metadata_bound(&scanner).is_err());
        assert!(borrowed_field_metadata_bound(&scanner).unwrap() > 0);
    }

    // Append inside `#[cfg(test)] mod borrowed_tests` in
    // apps/lumen/src/storage/text_preparation.rs.
    // This adds an analyzer-selecting helper; existing test helpers stay unchanged.

    fn engine_with_analyzer(analyzer: Analyzer) -> Engine {
        let mut schema = BTreeMap::new();
        schema.insert("body".to_owned(), spec(FieldType::Text, Some(analyzer)));
        let engine = Engine::new();
        engine
            .create_collection("docs", CreateCollectionRequest { fields: schema })
            .unwrap();
        engine
    }

    fn borrowed_pre_stage_reservation(
        engine: &Engine,
        scanner: &FastIndexScanner<'_>,
    ) -> record_admission::RecordReservation {
        // This is the current committed Text admission: two metadata populations
        // plus the fixed row-writer workspace, with no normalization or route
        // workspace. The target guard must reject it before stage creation.
        let metadata = borrowed_text_metadata_bound(scanner)
            .unwrap()
            .checked_mul(2)
            .unwrap();
        let bytes = metadata.checked_add(TEXT_SCRATCH_BYTES).unwrap();
        engine
            .wait_reserve_record_ram(&engine.record_ram_request_from_bound(bytes, 0))
            .unwrap()
    }

    #[test]
    fn borrowed_unicode_single_token_requires_pre_stage_workspace_before_staging() {
        // One token only. Do not turn this into many small tokens: the required
        // transient allocation is the largest normalized token, not whole input.
        let input = "İ".repeat(3_000_000);
        let bytes = encode(
            vec![IndexItem {
                external_id: "one".into(),
                field: "body".into(),
                value: FieldValue::String(input),
                version: None,
            }],
            None,
        );
        let scanner = FastIndexScanner::parse(&bytes).unwrap();
        let engine = engine_with_analyzer(Analyzer::WhitespaceLower);
        let mut reservation = borrowed_pre_stage_reservation(&engine, &scanner);

        let before_stage = staged_text_row::last_stage_directory_for_test();
        let result = engine.prepare_borrowed_text_rows(&scanner, &mut reservation);
        assert_eq!(
            staged_text_row::last_stage_directory_for_test(),
            before_stage,
            "the pre-stage guard must refuse before StageDirectory::create"
        );
        let error = result
            .err()
            .expect("pre-stage lowercase workspace must be reserved before staging starts");
        assert!(
            error
                .to_string()
                .contains("borrowed Text pre-stage workspace"),
            "the refusal must be the pre-stage reservation guard: {error:#}"
        );
    }

    #[cfg(feature = "jieba")]
    #[test]
    fn borrowed_dictionary_jieba_requires_route_cache_before_staging() {
        let bytes = encode(
            vec![IndexItem {
                external_id: "one".into(),
                field: "body".into(),
                value: FieldValue::String("南京市长江大桥".into()),
                version: None,
            }],
            None,
        );
        let scanner = FastIndexScanner::parse(&bytes).unwrap();
        let engine = engine_with_analyzer(Analyzer::Jieba);
        let mut reservation = borrowed_pre_stage_reservation(&engine, &scanner);

        let before_stage = staged_text_row::last_stage_directory_for_test();
        let result = engine.prepare_borrowed_text_rows(&scanner, &mut reservation);
        assert_eq!(
            staged_text_row::last_stage_directory_for_test(),
            before_stage,
            "the pre-stage guard must refuse before StageDirectory::create or DiskRoute::create"
        );
        let error = result
            .err()
            .expect("the two-page Jieba route cache must be reserved before staging starts");
        assert!(
            error
                .to_string()
                .contains("borrowed Text pre-stage workspace"),
            "the refusal must happen before DiskRoute allocation: {error:#}"
        );
    }

    // Append inside `borrowed_tests` after the implementation candidate.
    #[test]
    fn lowercase_pre_stage_bound_is_limited_to_the_largest_token() {
        let input = "İ ".repeat(4096);
        let bound = lowercase_token_workspace_bound(&input).unwrap();
        assert!(
            bound < input.len(),
            "many small words must not price whole input"
        );
        assert!(
            bound >= "İ".len() * 3,
            "one Unicode-lowercase token remains priced"
        );
    }

    // Append inside `#[cfg(test)] mod borrowed_tests` in
    // apps/lumen/src/storage/text_preparation.rs.
    //
    // The expected charge uses only the reservation baseline and the stage receipt.
    // It deliberately does not duplicate any large-row helper workspace formula.
    #[test]
    fn repeated_large_rows_release_transient_stage_charge_before_the_next_row() {
        let engine = engine_with_analyzer(Analyzer::WhitespaceLower);
        let baseline = TEXT_SCRATCH_BYTES;
        let mut reservation = engine
            .wait_reserve_record_ram(&engine.record_ram_request_from_bound(baseline, 0))
            .unwrap();
        let starting_bytes = reservation.bytes();
        let input = "İ".repeat(65_537); // one source token, above 64 KiB

        for ordinal in 0..3 {
            let (row, reader_bytes, scratch_growth) = engine
                .stage_text_row(&input, Analyzer::WhitespaceLower, &mut reservation)
                .unwrap();
            let during_stage = starting_bytes
                .checked_add(reader_bytes)
                .and_then(|bytes| bytes.checked_add(scratch_growth))
                .unwrap();
            assert_eq!(
                reservation.bytes(),
                during_stage,
                "row {ordinal} retains only its reader receipt and any explicit scratch retry"
            );
            assert_eq!(row.doc_len(), 1);

            // The caller owns the staged reader until it is no longer needed. Once
            // it drops, only the receipt-reported charges may remain to release.
            drop(row);
            reservation
                .release_preparation_workspace(reader_bytes + scratch_growth)
                .unwrap();
            assert_eq!(
                reservation.bytes(),
                starting_bytes,
                "row {ordinal} must not carry private helper workspace into row {}",
                ordinal + 1
            );
        }
    }
}
