//! Apply a retained fast-Index record for scalar and Text fields without
//! owning its field values.
//!
//! Planning, admission, file IO and row-map construction precede apply. The
//! apply interval only rechecks the cut, installs prepared readers and changes
//! small metadata. The caller advances its durable watermark in that interval.

use super::committed_index_plan::{
    self as plan, PlanView, PlannedCell, RequestOutcome, ScalarAction,
};
use super::committed_replace_apply::{ReplacementInput, ReplacementLedger};
use super::committed_replace_plan as replace_plan;
use super::*;
use crate::capture_barrier::ApplyLease;
use crate::composed_segment::ComposedSegmentReader;
use crate::wal::fast_index_scanner::FastIndexScanner;

#[cfg(test)]
thread_local! {
    static BEFORE_ATTACH: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = Default::default();
    static TEXT_WORKSPACE_RETRIES: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(super) fn reset_text_workspace_retries_for_test() {
    TEXT_WORKSPACE_RETRIES.with(|retries| retries.set(0));
}

#[cfg(test)]
pub(super) fn text_workspace_retries_for_test() -> u32 {
    TEXT_WORKSPACE_RETRIES.with(std::cell::Cell::get)
}

struct View<'a> {
    engine: &'a Engine,
    coll: &'a Collection,
    now: Instant,
}

impl PlanView for View<'_> {
    fn engine_epoch(&self) -> u64 {
        self.engine.capture_barrier.epoch()
    }
    fn collection_generation(&self) -> u64 {
        self.coll.collection_generation
    }
    fn schema_version(&self) -> u32 {
        self.coll.version
    }
    fn data_version(&self) -> u64 {
        self.coll.data_version
    }
    fn revision(&self) -> u64 {
        self.engine.capture_barrier.apply_revision()
    }
    fn interner_len(&self) -> usize {
        self.coll.interner.to_eid.len()
    }
    fn is_live(&self) -> bool {
        self.coll.deleted_at.is_none()
    }
    fn field_type(&self, field: &str) -> Option<FieldType> {
        self.coll.fields.get(field).map(FieldIndex::field_type)
    }
    fn vector_dimension(&self, field: &str) -> Option<u32> {
        match self.coll.fields.get(field)? {
            FieldIndex::Vector { spec, .. } => Some(spec.dim),
            _ => None,
        }
    }
    fn id(&self, external_id: &str) -> Option<u32> {
        self.coll.interner.id(external_id)
    }
    fn has_cell(&self, id: u32, field: &str) -> bool {
        self.coll
            .eid_fields
            .get(&id)
            .is_some_and(|fields| fields.contains(field))
    }
    fn cell_version(&self, id: u32, field: &str) -> Option<u64> {
        self.coll.cell_versions.get(&id)?.get(field).copied()
    }
    fn request_deadline(&self, request_id: &str) -> Option<Instant> {
        self.coll.seen_requests.iter().find_map(|(key, at)| {
            let deadline = *at + IDEMPOTENCY_TTL;
            (key == request_id && self.now <= deadline).then_some(deadline)
        })
    }
}

struct FieldPlan {
    kind: FieldType,
    before: Option<Arc<ComposedSegmentReader>>,
    after: Option<Arc<ComposedSegmentReader>>,
    winners: Vec<(u32, usize)>,
    final_bytes: u64,
}

struct Prepared {
    plan: plan::ScalarPlan,
    business_error: Option<anyhow::Error>,
    replacement: Option<ReplacementLedger>,
    fields: BTreeMap<String, FieldPlan>,
    /// Hash has an eight-byte canonical value, regardless of wire string size.
    hash_rows: BTreeMap<PlannedCell, Option<u64>>,
    vector_codebooks: BTreeMap<String, Option<ScalarCodebook>>,
    /// Every source action remains alive through the ordered backend updates.
    vector_rows: BTreeMap<usize, VectorRows>,
    /// Replaced journal payloads may own files. Release them outside apply.
    old_vector_rows: Vec<crate::change_journal::Row<CheckpointValue>>,
    // The per-cell journal points into the same reader that queries use.
    rows: BTreeMap<PlannedCell, Option<Arc<CheckpointValue>>>,
    /// Final Text values use staged rows, not a scalar composed segment.
    text_rows: BTreeMap<PlannedCell, Option<Arc<staged_text_row::StagedTextRow>>>,
    text_actions: BTreeMap<usize, String>,
    text_prepared: Option<text_preparation::PreparedTextRows>,
    /// Old staged file owners must outlive the state write and apply lease.
    old_text_rows: Vec<Arc<staged_text_row::StagedTextRow>>,
    text_placeholder: FieldValue,
    retained: usize,
}

struct VectorRows {
    raw: Arc<staged_vector_row::StagedVectorRow>,
    canonical: Arc<staged_vector_row::StagedVectorRow>,
}

fn composed(index: &FieldIndex) -> Option<&Arc<ComposedSegmentReader>> {
    match index {
        FieldIndex::Keyword(k) => k.segment.as_ref(),
        FieldIndex::Number(n) => n.segment.as_ref(),
        FieldIndex::Set(s) => s.segment.as_ref(),
        _ => None,
    }
}

fn scalar_bytes(index: &FieldIndex) -> u64 {
    match index {
        FieldIndex::Keyword(k) => k.bytes,
        FieldIndex::Number(n) => n.bytes,
        FieldIndex::Set(s) => s.bytes,
        _ => unreachable!("scalar plan validated field kind"),
    }
}

/// Current logical byte weight. Raw staged dictionaries lend values directly;
/// no String or set of all members is constructed for an immutable row.
fn cell_bytes(index: &FieldIndex, id: u32, eid_len: usize) -> u64 {
    match index {
        FieldIndex::Keyword(k) => {
            if let Some(value) = k
                .dense_forward
                .get(id as usize)
                .and_then(Option::as_ref)
                .or_else(|| k.forward.get(&id))
            {
                return (value.len() + eid_len) as u64;
            }
            if k.tombstones.contains(id) {
                return 0;
            }
            k.segment
                .as_ref()
                .and_then(|s| s.keyword_at_cow(id))
                .map_or(0, |value| (value.len() + eid_len) as u64)
        }
        FieldIndex::Number(n) => n.number_at(id).map_or(0, |_| (8 + eid_len) as u64),
        FieldIndex::Set(s) => {
            if let Some(values) = s.forward.get(&id) {
                return values
                    .iter()
                    .map(|value| (value.len() + eid_len) as u64)
                    .sum();
            }
            if s.tombstones.contains(id) {
                return 0;
            }
            let Some(view) = s.segment.as_ref() else {
                return 0;
            };
            let Some((true, count)) = view.set_row_member_count(id) else {
                return 0;
            };
            (0..count)
                .map(|member| {
                    view.set_member_at_cow(id, member)
                        .map_or(0, |value| (value.len() + eid_len) as u64)
                })
                .sum()
        }
        _ => unreachable!("scalar plan validated field kind"),
    }
}

fn require(reserved: &mut RecordReservation, bytes: usize) -> Result<()> {
    if reserved.bytes() < bytes {
        reserved
            .wait_grow_to(bytes)
            .map_err(RecordAdmissionError::Capacity)?;
    }
    Ok(())
}

impl Engine {
    /// `false` is an internal routing result for an unsupported command. A real
    /// preparation failure returns an error and never invokes `complete`.
    pub(crate) fn try_apply_committed_scalar(
        &self,
        scanner: &FastIndexScanner<'_>,
        sequence: u64,
        complete: impl FnOnce(&ApplyLease<'_>, Result<ApplyOutcome>),
    ) -> Result<bool> {
        self.try_apply_committed_scalar_with_capacity_owner(
            scanner,
            sequence,
            &mut || bail!("committed layer capacity needs a caller-owned maintainer"),
            complete,
        )
    }

    pub(crate) fn try_apply_committed_scalar_with_capacity_owner(
        &self,
        scanner: &FastIndexScanner<'_>,
        sequence: u64,
        ensure_owner: &mut dyn FnMut() -> Result<()>,
        complete: impl FnOnce(&ApplyLease<'_>, Result<ApplyOutcome>),
    ) -> Result<bool> {
        self.try_apply_committed_fields_with_capacity_owner(
            scanner,
            None,
            sequence,
            ensure_owner,
            complete,
        )
    }

    pub(super) fn try_apply_committed_fields_with_capacity_owner(
        &self,
        scanner: &FastIndexScanner<'_>,
        replacement: Option<&ReplacementInput<'_>>,
        sequence: u64,
        ensure_owner: &mut dyn FnMut() -> Result<()>,
        complete: impl FnOnce(&ApplyLease<'_>, Result<ApplyOutcome>),
    ) -> Result<bool> {
        let collection = scanner.collection_id();
        let (item_count, item_limit) = replacement
            .map_or((scanner.cost().item_count, MAX_INDEX_ITEMS), |input| {
                (input.docs.len(), MAX_BATCH_REPLACE_SIZE)
            });
        // These business errors need no value decoder or preparation. Recheck
        // them inside apply so restore cannot turn a stale refusal into a cut.
        {
            let apply = self.capture_barrier.apply();
            let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
            let error = match state.collections.get(collection) {
                None => Some(StorageError::CollectionNotFound(collection.to_owned())),
                Some(_) if item_count > item_limit => Some(StorageError::BulkLimit {
                    got: item_count,
                    max: item_limit,
                }),
                Some(coll) if coll.deleted_at.is_some() => {
                    Some(StorageError::Gone(collection.to_owned()))
                }
                _ => None,
            };
            if let Some(error) = error {
                drop(state);
                complete(&apply, Err(error.into()));
                return Ok(true);
            }
        }
        let metadata = match replacement {
            None => plan::metadata_bound(scanner)?,
            Some(input) => {
                let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
                match state.collections.get(collection) {
                    Some(coll) => replace_plan::metadata_bound(
                        scanner,
                        input.docs,
                        &super::committed_replace_view::View {
                            engine: self,
                            coll,
                            scanner,
                            parsed: &input.parsed,
                            now: Instant::now(),
                        },
                    )?,
                    None => 0,
                }
            }
        };
        // Price the additional adapter maps, row handles and snapshot Arc
        // vectors before constructing them. Value bytes stay in the source.
        // Scalar-only records keep this exact existing floor. Text metadata
        // is added only after the ledger has proved a Text action exists.
        let mut floor = metadata
            .checked_mul(3)
            .ok_or(RecordAdmissionError::Overflow)?;
        let text_metadata = if replacement.is_some() {
            text_preparation::borrowed_field_metadata_bound(scanner)?
        } else {
            text_preparation::borrowed_text_metadata_bound(scanner)?
        };
        let request = self.record_ram_request_from_bound(floor, 0);
        let mut reservation = match self.try_reserve_record_ram(&request) {
            Ok(reservation) => reservation,
            Err(RecordAdmissionError::Capacity(crate::change_budget::AdmissionError::Full {
                ..
            })) => self.wait_reserve_record_ram(&request)?,
            Err(error) => return Err(error.into()),
        };
        let mut hashes = plan::ParsedHashes::new();
        if let Some(input) = replacement {
            hashes.extend(
                input
                    .parsed
                    .iter()
                    .map(|(ordinal, value)| (*ordinal, value.hash)),
            );
        }
        loop {
            // Only the schema/ordinal selection holds a state lock. Parsing a
            // valid giant leading-zero string must never hold state or apply.
            let hash_ordinals: BTreeSet<_> = {
                let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
                scanner
                    .items()
                    .enumerate()
                    .filter_map(|(ordinal, item)| {
                        (!hashes.contains_key(&ordinal)
                            && matches!(
                                item.value,
                                crate::wal::fast_index_scanner::FastIndexValue::String(_)
                            )
                            && state
                                .collections
                                .get(collection)
                                .and_then(|coll| coll.fields.get(item.field))
                                .is_some_and(|index| index.field_type() == FieldType::Hash))
                        .then_some(ordinal)
                    })
                    .collect()
            };
            for (ordinal, item) in scanner.items().enumerate() {
                if hash_ordinals.contains(&ordinal) {
                    let crate::wal::fast_index_scanner::FastIndexValue::String(value) = item.value
                    else {
                        unreachable!("selected Hash source is a String")
                    };
                    hashes.insert(ordinal, parse_hash_number(value).ok());
                }
            }
            drop(hash_ordinals);
            let mut prepared = {
                let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
                let Some(coll) = state.collections.get(collection) else {
                    drop(state);
                    // Re-enter through the small business-error path.
                    drop(reservation);
                    return self.try_apply_committed_fields_with_capacity_owner(
                        scanner,
                        replacement,
                        sequence,
                        ensure_owner,
                        complete,
                    );
                };
                if coll.deleted_at.is_some() {
                    drop(state);
                    drop(reservation);
                    return self.try_apply_committed_fields_with_capacity_owner(
                        scanner,
                        replacement,
                        sequence,
                        ensure_owner,
                        complete,
                    );
                }
                let now = Instant::now();
                let view = View {
                    engine: self,
                    coll,
                    now,
                };
                let mut replacement_ledger = None;
                let plan = if let Some(input) = replacement {
                    let replace_view = super::committed_replace_view::View {
                        engine: self,
                        coll,
                        scanner,
                        parsed: &input.parsed,
                        now,
                    };
                    let required =
                        replace_plan::metadata_bound(scanner, input.docs, &replace_view)?
                            .checked_mul(3)
                            .ok_or(RecordAdmissionError::Overflow)?;
                    if reservation.bytes() < required {
                        drop(state);
                        require(&mut reservation, required)?;
                        floor = floor.max(required);
                        continue;
                    }
                    floor = floor.max(required);
                    let replacement_plan = replace_plan::plan(
                        scanner,
                        input.docs,
                        &input.parsed,
                        &replace_view,
                        now,
                        |bytes| {
                            anyhow::ensure!(
                                bytes <= reservation.bytes(),
                                "replacement metadata was not reserved"
                            );
                            Ok(())
                        },
                    )?;
                    replacement_ledger = Some(ReplacementLedger {
                        outcomes: replacement_plan.outcomes,
                        results: Vec::new(),
                        final_docs: replacement_plan.final_docs,
                        final_checksums: replacement_plan.final_checksums,
                        fields_written: replacement_plan.fields_written,
                        fields_skipped: replacement_plan.fields_skipped,
                    });
                    replacement_plan.scalar
                } else {
                    match plan::plan_with_hashes(scanner, &view, &hashes, now, |bytes| {
                        anyhow::ensure!(
                            bytes <= reservation.bytes(),
                            "borrowed planner metadata was not reserved"
                        );
                        Ok(())
                    })? {
                        plan::PlanResult::Unsupported { .. } => return Ok(false),
                        plan::PlanResult::NeedsHash => continue,
                        plan::PlanResult::Planned(plan) => plan,
                    }
                };
                let blocked = plan.winners.keys().find_map(|cell| {
                    let view = coll.fields.get(&cell.field).and_then(composed)?;
                    (view.incremental_layer_count() >= self.layer_maintenance.append_limit()).then(
                        || {
                            if view.has_private_layers() {
                                crate::segment_capacity::Work::Checkpoint
                            } else {
                                crate::segment_capacity::Work::Merge
                            }
                        },
                    )
                });
                if let Some(work) = blocked {
                    drop(plan);
                    drop(state);
                    if self.layer_maintenance.owner().is_none() {
                        ensure_owner()?;
                    }
                    if let Some(owner) = self.layer_maintenance.owner() {
                        owner.wait_for(work)?;
                    }
                    continue;
                }
                // A duplicate, stale item or failed replacement has no reader
                // append. Price the Arc-vector copy once per winning field.
                let mut view_bytes = 0usize;
                let mut seen_fields = BTreeSet::new();
                for cell in plan.winners.keys() {
                    if !seen_fields.insert(cell.field.as_str()) {
                        continue;
                    }
                    if let Some(view) = coll.fields.get(&cell.field).and_then(composed) {
                        view_bytes = view_bytes
                            .checked_add(
                                view.private_append_metadata_bound()
                                    .ok_or(RecordAdmissionError::Overflow)?,
                            )
                            .ok_or(RecordAdmissionError::Overflow)?;
                    }
                }
                let required = floor
                    .checked_add(view_bytes)
                    .and_then(|bytes| {
                        plan.has_text()
                            .then(|| text_metadata)
                            .map_or(Some(bytes), |text| bytes.checked_add(text))
                    })
                    .ok_or(RecordAdmissionError::Overflow)?;
                if reservation.bytes() < required {
                    drop(seen_fields);
                    drop(plan);
                    drop(state);
                    require(&mut reservation, required)?;
                    continue;
                }
                let mut fields = BTreeMap::new();
                let mut sizes = BTreeMap::new();
                let mut rows = BTreeMap::new();
                let mut text_actions = BTreeMap::new();
                let mut text_rows = BTreeMap::new();
                let mut old_text_rows = Vec::new();
                let mut hash_rows = BTreeMap::new();
                let mut vector_codebooks = BTreeMap::new();
                for action in &plan.actions {
                    let (cell, next) = match action {
                        ScalarAction::Apply {
                            cell,
                            kind: FieldType::Vector,
                            ..
                        } => {
                            if !vector_codebooks.contains_key(&cell.field) {
                                let FieldIndex::Vector { idx, .. } = &coll.fields[&cell.field]
                                else {
                                    unreachable!("matched Vector schema")
                                };
                                vector_codebooks.insert(
                                    cell.field.clone(),
                                    idx.checkpoint_codebook_for_preparation()?,
                                );
                            }
                            continue;
                        }
                        ScalarAction::Drop {
                            kind: FieldType::Vector,
                            ..
                        } => continue,
                        ScalarAction::Apply {
                            cell,
                            kind: FieldType::Hash,
                            ordinal,
                            ..
                        } => {
                            hash_rows.insert(
                                cell.clone(),
                                Some(hashes[ordinal].expect("planned valid Hash")),
                            );
                            continue;
                        }
                        ScalarAction::Drop {
                            cell,
                            kind: FieldType::Hash,
                            ..
                        } => {
                            hash_rows.insert(cell.clone(), None);
                            continue;
                        }
                        ScalarAction::Apply {
                            cell,
                            kind: FieldType::Text,
                            ordinal,
                            ..
                        } => {
                            text_actions.insert(*ordinal, cell.field.clone());
                            if let Some(FieldIndex::Text { idx, .. }) = coll.fields.get(&cell.field)
                            {
                                if let Some(row) = idx.staged_rows.get(&cell.id) {
                                    old_text_rows.push(row.clone());
                                }
                            }
                            continue;
                        }
                        ScalarAction::Drop {
                            cell,
                            kind: FieldType::Text,
                            ..
                        } => {
                            if let Some(FieldIndex::Text { idx, .. }) = coll.fields.get(&cell.field)
                            {
                                if let Some(row) = idx.staged_rows.get(&cell.id) {
                                    old_text_rows.push(row.clone());
                                }
                            }
                            text_rows.insert(cell.clone(), None);
                            continue;
                        }
                        ScalarAction::Apply { cell, bytes, .. } => (cell, *bytes),
                        ScalarAction::Drop { cell, .. } => (cell, 0),
                    };
                    let index = &coll.fields[&cell.field];
                    let field = fields
                        .entry(cell.field.clone())
                        .or_insert_with(|| FieldPlan {
                            kind: index.field_type(),
                            before: composed(index).cloned(),
                            after: None,
                            winners: Vec::new(),
                            final_bytes: scalar_bytes(index),
                        });
                    let old = sizes.entry(cell.clone()).or_insert_with(|| {
                        if view.has_cell(cell.id, &cell.field) {
                            cell_bytes(index, cell.id, coll.interner.resolve(cell.id).len())
                        } else {
                            0
                        }
                    });
                    field.final_bytes = field.final_bytes.saturating_sub(*old).saturating_add(next);
                    *old = next;
                    rows.insert(cell.clone(), None);
                }
                for (cell, ordinal) in &plan.winners {
                    if matches!(
                        coll.fields[&cell.field].field_type(),
                        FieldType::Text | FieldType::Hash | FieldType::Vector
                    ) {
                        continue;
                    }
                    fields
                        .get_mut(&cell.field)
                        .expect("winner has action")
                        .winners
                        .push((cell.id, *ordinal));
                }
                let old_vector_rows = Vec::with_capacity(
                    plan.actions
                        .iter()
                        .filter(|action| {
                            matches!(
                                action,
                                ScalarAction::Apply {
                                    kind: FieldType::Vector,
                                    ..
                                } | ScalarAction::Drop {
                                    kind: FieldType::Vector,
                                    ..
                                }
                            )
                        })
                        .count(),
                );
                Prepared {
                    plan,
                    business_error: None,
                    replacement: replacement_ledger,
                    fields,
                    hash_rows,
                    vector_codebooks,
                    vector_rows: BTreeMap::new(),
                    old_vector_rows,
                    rows,
                    text_rows,
                    text_actions,
                    text_prepared: None,
                    old_text_rows,
                    text_placeholder: FieldValue::String(String::new()),
                    retained: required,
                }
            };

            // Error response formatting can include the caller's original Hash
            // string. It is response data, never a retained checkpoint value,
            // and it must finish before state/apply is acquired.
            prepared.business_error = prepared
                .plan
                .business_error
                .take()
                .map(|error| error.into_error(collection, scanner));

            if let Some(ledger) = &mut prepared.replacement {
                ledger.render(collection, scanner);
            }

            if !prepared.text_actions.is_empty() {
                let rows = match self.prepare_borrowed_text_rows_for_actions(
                    scanner,
                    &prepared.text_actions,
                    prepared.retained,
                    &mut reservation,
                ) {
                    Ok(rows) => rows,
                    Err(error)
                        if error
                            .downcast_ref::<text_preparation::RequiredBorrowedTextWorkspace>()
                            .is_some() =>
                    {
                        let required = error
                            .downcast_ref::<text_preparation::RequiredBorrowedTextWorkspace>()
                            .expect("matched borrowed Text workspace")
                            .required_bytes;
                        drop(prepared);
                        #[cfg(test)]
                        TEXT_WORKSPACE_RETRIES
                            .with(|retries| retries.set(retries.get().saturating_add(1)));
                        require(&mut reservation, required)?;
                        continue;
                    }
                    Err(error)
                        if error
                            .downcast_ref::<text_preparation::StaleBorrowedTextPreparation>()
                            .is_some() =>
                    {
                        drop(prepared);
                        continue;
                    }
                    Err(error) => return Err(error),
                };
                prepared.retained = prepared
                    .retained
                    .checked_add(rows.retained_reader_bytes())
                    .ok_or(RecordAdmissionError::Overflow)?;
                let planned_text: Vec<_> = prepared
                    .plan
                    .text_actions()
                    .map(|(cell, ordinal)| (cell.clone(), ordinal))
                    .collect();
                for (cell, ordinal) in planned_text {
                    let row = rows
                        .get(ordinal, &cell.field)
                        .expect("accepted Text action has a staged row");
                    let external_id = scanner
                        .items()
                        .nth(ordinal)
                        .expect("validated Text action ordinal")
                        .external_id;
                    prepared
                        .plan
                        .set_text_bytes(ordinal, row.indexed_bytes(external_id))?;
                    if prepared.plan.winners.get(&cell) == Some(&ordinal) {
                        prepared.text_rows.insert(cell, Some(row.clone()));
                    }
                }
                // Keep every staged row alive through attach. Non-winner rows
                // drop only after the apply lease, along with displaced rows.
                prepared.text_prepared = Some(rows);
            }
            // The source and canonical SQ checkpoint view use private aligned
            // mmap files. Only a bounded decode chunk and reader metadata are
            // admitted. No input or checkpoint Vec is built under apply.
            for action in &prepared.plan.actions {
                let ScalarAction::Apply {
                    ordinal,
                    cell,
                    kind: FieldType::Vector,
                    ..
                } = action
                else {
                    continue;
                };
                let item = scanner
                    .items()
                    .nth(*ordinal)
                    .expect("validated Vector ordinal");
                let crate::wal::fast_index_scanner::FastIndexValue::Vector { values, len } =
                    item.value
                else {
                    unreachable!("planned Vector wire value")
                };
                let retained = prepared.retained;
                let raw = Arc::new(staged_vector_row::StagedVectorRow::stage(
                    values,
                    u32::try_from(len).expect("planned Vector dimension fits u32"),
                    |workspace| {
                        require(
                            &mut reservation,
                            retained
                                .checked_add(workspace)
                                .ok_or(RecordAdmissionError::Overflow)?,
                        )
                    },
                )?);
                prepared.retained = prepared
                    .retained
                    .checked_add(staged_vector_row::StagedVectorRow::retained_metadata_bound())
                    .ok_or(RecordAdmissionError::Overflow)?;
                let codebook = prepared
                    .vector_codebooks
                    .get_mut(&cell.field)
                    .expect("Vector preparation snapshot");
                let canonical = match codebook {
                    Some(snapshot) => {
                        let retained = prepared.retained;
                        let (row, widened) = raw.stage_sq_canonical(*snapshot, |workspace| {
                            require(
                                &mut reservation,
                                retained
                                    .checked_add(workspace)
                                    .ok_or(RecordAdmissionError::Overflow)?,
                            )
                        })?;
                        *snapshot = widened;
                        prepared.retained = prepared
                            .retained
                            .checked_add(
                                staged_vector_row::StagedVectorRow::retained_metadata_bound(),
                            )
                            .ok_or(RecordAdmissionError::Overflow)?;
                        Arc::new(row)
                    }
                    None => raw.clone(),
                };
                prepared
                    .vector_rows
                    .insert(*ordinal, VectorRows { raw, canonical });
            }
            let parent = std::env::temp_dir();
            for (name, field) in &mut prepared.fields {
                if field.winners.is_empty() {
                    continue;
                }
                let winning: Vec<_> = field.winners.iter().map(|(_, ordinal)| *ordinal).collect();
                let retained = prepared.retained;
                let file = committed_scalar_files::prepare_validated_fields(
                    scanner,
                    name,
                    &winning,
                    field.kind,
                    sequence,
                    &parent,
                    |peak| {
                        require(
                            &mut reservation,
                            retained
                                .checked_add(peak)
                                .ok_or(RecordAdmissionError::Overflow)?,
                        )
                    },
                )?;
                prepared.retained = prepared
                    .retained
                    .checked_add(file.retained_bytes)
                    .ok_or(RecordAdmissionError::Overflow)?;
                let base = match &field.before {
                    Some(base) => base.clone(),
                    None => {
                        let retained = prepared.retained;
                        let empty = committed_scalar_files::prepare_validated_fields(
                            scanner,
                            name,
                            &[],
                            field.kind,
                            sequence,
                            &parent,
                            |peak| {
                                require(
                                    &mut reservation,
                                    retained
                                        .checked_add(peak)
                                        .ok_or(RecordAdmissionError::Overflow)?,
                                )
                            },
                        )?;
                        prepared.retained = prepared
                            .retained
                            .checked_add(empty.retained_bytes)
                            .ok_or(RecordAdmissionError::Overflow)?;
                        Arc::new(ComposedSegmentReader::from_private_empty_base(
                            empty.reader,
                        )?)
                    }
                };
                let ids = field.winners.iter().map(|(id, _)| *id).collect();
                field.after = Some(Arc::new(base.with_private_delta(file.reader.clone(), ids)?));
                for (row, (id, _)) in field.winners.iter().enumerate() {
                    *prepared
                        .rows
                        .get_mut(&PlannedCell {
                            id: *id,
                            field: name.clone(),
                        })
                        .expect("winner journal row") =
                        Some(Arc::new(CheckpointValue::StagedScalar {
                            reader: file.reader.clone(),
                            row: row as u32,
                        }));
                }
            }
            require(&mut reservation, prepared.retained)?;
            reservation
                .finish_borrowed_preparation(prepared.retained)
                .map_err(RecordAdmissionError::Capacity)?;
            #[cfg(test)]
            if let Some(hook) = BEFORE_ATTACH.with(|hook| hook.borrow_mut().take()) {
                hook();
            }
            let apply = self.capture_barrier.apply();
            let mut telemetry = self.metrics.committed_apply_telemetry();
            let state_write_wait_started = Instant::now();
            let state_write = self.state.write();
            telemetry.record_state_write_lock_wait(state_write_wait_started.elapsed());
            let mut state = state_write.map_err(|_| anyhow!("state poisoned"))?;
            let matches = state.collections.get(collection).is_some_and(|coll| {
                prepared.plan.matches(
                    &View {
                        engine: self,
                        coll,
                        now: Instant::now(),
                    },
                    Instant::now(),
                )
            });
            let capacity_changed = prepared.fields.values().any(|field| {
                !field.winners.is_empty()
                    && field.before.as_ref().is_some_and(|view| {
                        view.incremental_layer_count() >= self.layer_maintenance.append_limit()
                    })
            });
            if !matches || capacity_changed {
                drop(state);
                drop(apply);
                drop(prepared); // private file owners and old views drop outside apply
                reservation
                    .shrink_borrowed_to(floor)
                    .map_err(RecordAdmissionError::Capacity)?;
                continue;
            }
            let charge = self.retain_borrowed_reservation(reservation)?;
            let coll = state
                .collections
                .get_mut(collection)
                .expect("matched collection");
            if replacement.is_none() {
                coll.gc_requests();
            }
            if let RequestOutcome::Register(id) = &prepared.plan.request {
                coll.seen_requests.push_back((id.clone(), Instant::now()));
            }
            let duplicate = matches!(prepared.plan.request, RequestOutcome::Duplicate { .. });
            if !duplicate && item_count != 0 {
                coll.clear_search_cache();
            }
            if prepared.plan.has_text() || (replacement.is_some() && item_count != 0) {
                coll.clear_text_rank_caches();
            }
            for eid in &prepared.plan.new_external_ids {
                coll.interner.intern(eid);
            }
            if prepared
                .fields
                .values()
                .any(|f| matches!(f.kind, FieldType::Keyword | FieldType::Number))
            {
                coll.clear_number_filter_caches();
            }
            for (name, field) in &mut prepared.fields {
                let index = coll.fields.get_mut(name).expect("matched scalar field");
                if let Some(after) = field.after.take() {
                    match index {
                        FieldIndex::Keyword(k) => k.segment = Some(after),
                        FieldIndex::Number(n) => n.segment = Some(after),
                        FieldIndex::Set(s) => s.segment = Some(after),
                        _ => unreachable!("matched scalar kind"),
                    }
                }
                match index {
                    FieldIndex::Keyword(k) => k.bytes = field.final_bytes,
                    FieldIndex::Number(n) => n.bytes = field.final_bytes,
                    FieldIndex::Set(s) => s.bytes = field.final_bytes,
                    _ => unreachable!("matched scalar kind"),
                }
            }
            for (cell, row) in &prepared.text_rows {
                let eid = coll.interner.resolve(cell.id).to_owned();
                let index = coll
                    .fields
                    .get_mut(&cell.field)
                    .expect("matched Text row field");
                // `drop_eid` maintains Text doc_count, total_doc_len and
                // indexed-byte counters. Keep old row Arcs in Prepared until
                // after state and the apply lease are released.
                index.drop_eid(cell.id, &eid);
                if let Some(row) = row {
                    apply_prepared_value(
                        index,
                        cell.id,
                        &eid,
                        &prepared.text_placeholder,
                        &cell.field,
                        Some(row),
                    )?;
                }
                coll.next_field_dirty_revision = coll.next_field_dirty_revision.saturating_add(1);
                coll.field_dirty
                    .entry(cell.field.clone())
                    .or_default()
                    .insert(eid.clone(), coll.next_field_dirty_revision);
                let value = row
                    .as_ref()
                    .map(|row| Arc::new(CheckpointValue::StagedText(row.clone())));
                coll.change_journal.record_charged(
                    cell.field.clone(),
                    eid,
                    coll.next_field_dirty_revision,
                    value,
                    Some(charge.clone()),
                );
            }
            for (cell, value) in &prepared.hash_rows {
                let eid = coll.interner.resolve(cell.id).to_owned();
                let index = coll
                    .fields
                    .get_mut(&cell.field)
                    .expect("matched Hash field");
                index.drop_eid(cell.id, &eid);
                if let Some(value) = value {
                    let FieldIndex::Hash(hash) = index else {
                        unreachable!("matched Hash kind")
                    };
                    hash.forward.insert(cell.id, *value);
                    hash.bytes += 12;
                }
                coll.next_field_dirty_revision = coll.next_field_dirty_revision.saturating_add(1);
                coll.field_dirty
                    .entry(cell.field.clone())
                    .or_default()
                    .insert(eid.clone(), coll.next_field_dirty_revision);
                coll.change_journal.record_charged(
                    cell.field.clone(),
                    eid,
                    coll.next_field_dirty_revision,
                    value.map(|value| Arc::new(CheckpointValue::Hash(value))),
                    Some(charge.clone()),
                );
            }
            // Apply each Vector action in source order. Even a superseded row
            // can widen SQ's codebook or change HNSW's online graph.
            for action in &prepared.plan.actions {
                let (cell, _ordinal, row) = match action {
                    ScalarAction::Apply {
                        cell,
                        ordinal,
                        kind: FieldType::Vector,
                        ..
                    } => (cell, *ordinal, Some(&prepared.vector_rows[ordinal])),
                    ScalarAction::Drop {
                        cell,
                        ordinal,
                        kind: FieldType::Vector,
                    } => (cell, *ordinal, None),
                    _ => continue,
                };
                // Omitted replacement fields have no source ordinal.
                let eid = coll.interner.resolve(cell.id);
                let index = coll
                    .fields
                    .get_mut(&cell.field)
                    .expect("matched Vector field");
                index.drop_eid(cell.id, eid);
                if let Some(row) = row {
                    let FieldIndex::Vector {
                        idx, bytes, spec, ..
                    } = index
                    else {
                        unreachable!("matched Vector kind")
                    };
                    let add = if matches!(spec.backend, crate::types::VectorBackend::HnswCpu) {
                        let hnsw_add_started = Instant::now();
                        let add = idx.add(eid, row.raw.as_f32_slice());
                        telemetry.record_hnsw_add(hnsw_add_started.elapsed());
                        add
                    } else {
                        idx.add(eid, row.raw.as_f32_slice())
                    };
                    if let Err(error) = add {
                        // A backend fault after a validated prepare is not a
                        // business rejection. The partial apply must never be
                        // published or acknowledged as a successful cut.
                        apply.mark_uncertain();
                        return Err(error);
                    }
                    *bytes += (row.raw.dim() * 4 + eid.len()) as u64;
                }
                coll.next_field_dirty_revision = coll.next_field_dirty_revision.saturating_add(1);
                coll.field_dirty
                    .entry(cell.field.clone())
                    .or_default()
                    .insert(eid.to_owned(), coll.next_field_dirty_revision);
                let value =
                    row.map(|row| Arc::new(CheckpointValue::StagedVector(row.canonical.clone())));
                if let Some(old) = coll.change_journal.replace_charged(
                    cell.field.clone(),
                    eid.to_owned(),
                    coll.next_field_dirty_revision,
                    value,
                    Some(charge.clone()),
                ) {
                    prepared.old_vector_rows.push(old);
                }
            }
            // Preserve versions and coverage for every successful prefix item.
            // Failed replacements intentionally retain their previous coverage.
            if replacement.is_none() {
                let mut items = scanner.items().enumerate().peekable();
                for action in &prepared.plan.actions {
                    if let ScalarAction::Apply { cell, ordinal, .. } = action {
                        while items.peek().is_some_and(|(i, _)| i < ordinal) {
                            items.next();
                        }
                        let (_, item) = items.next().expect("validated action ordinal");
                        if let Some(version) = item.version {
                            coll.cell_versions
                                .entry(cell.id)
                                .or_default()
                                .insert(cell.field.clone(), version);
                        }
                        let coverage = coll.eid_fields.entry(cell.id).or_default();
                        if !coverage.contains(&cell.field) {
                            coverage.insert_absent(cell.field.clone());
                        }
                    }
                }
            }
            if let (Some(input), Some(ledger)) = (replacement, &prepared.replacement) {
                // Any accepted empty document clears all checksum history,
                // including when a later duplicate adds fields in this batch.
                for (doc, result) in input.docs.iter().zip(&ledger.results) {
                    if doc.fields_start == doc.fields_end
                        && matches!(result, ReplaceDocResult::Ok { .. })
                    {
                        let id = coll
                            .interner
                            .id(doc.external_id)
                            .expect("planned replacement id");
                        coll.field_checksums.remove(&id);
                    }
                }
                for doc in &ledger.final_docs {
                    if doc.fields.is_empty() {
                        coll.eid_fields.remove(&doc.id);
                    } else {
                        coll.eid_fields.insert(
                            doc.id,
                            FieldCoverage {
                                names: doc.fields.clone(),
                            },
                        );
                    }
                    if let Some(version) = doc.version {
                        coll.doc_versions.insert(doc.id, version);
                    }
                }
                for row in &ledger.final_checksums {
                    match row.checksum {
                        Some(value) => {
                            coll.field_checksums
                                .entry(row.cell.id)
                                .or_default()
                                .insert(row.cell.field.clone(), value);
                        }
                        None => {
                            if let Some(fields) = coll.field_checksums.get_mut(&row.cell.id) {
                                fields.remove(&row.cell.field);
                            }
                        }
                    }
                }
            }
            for (cell, value) in &prepared.rows {
                let index = coll
                    .fields
                    .get_mut(&cell.field)
                    .expect("matched scalar row field");
                retire_live_delta_overlay(index, cell.id);
                if value.is_none() {
                    match index {
                        FieldIndex::Keyword(k) => {
                            k.tombstones.insert(cell.id);
                        }
                        FieldIndex::Number(n) => {
                            n.tombstones.insert(cell.id);
                        }
                        FieldIndex::Set(s) => {
                            s.tombstones.insert(cell.id);
                        }
                        _ => unreachable!("matched scalar kind"),
                    }
                }
                let eid = coll.interner.resolve(cell.id).to_owned();
                coll.next_field_dirty_revision = coll.next_field_dirty_revision.saturating_add(1);
                coll.field_dirty
                    .entry(cell.field.clone())
                    .or_default()
                    .insert(eid.clone(), coll.next_field_dirty_revision);
                coll.change_journal.record_charged(
                    cell.field.clone(),
                    eid,
                    coll.next_field_dirty_revision,
                    value.clone(),
                    Some(charge.clone()),
                );
            }
            let outcome = if let Some(ledger) = &mut prepared.replacement {
                if ledger
                    .results
                    .iter()
                    .any(|result| matches!(result, ReplaceDocResult::Ok { .. }))
                {
                    coll.last_indexed_at = Some(std::time::SystemTime::now());
                }
                self.metrics.incr_index(
                    ledger.fields_written,
                    prepared.plan.bytes_by_field.values().sum(),
                );
                self.metrics.incr_replace_skipped(ledger.fields_skipped);
                Ok(ApplyOutcome::Replaced(ReplaceDocsResponse {
                    results: std::mem::take(&mut ledger.results),
                }))
            } else {
                match prepared.business_error.take() {
                    Some(error) => Err(error),
                    None => {
                        if prepared.plan.applied > 0 {
                            coll.last_indexed_at = Some(std::time::SystemTime::now());
                        }
                        let bytes = std::mem::take(&mut prepared.plan.bytes_by_field);
                        self.metrics
                            .incr_index(prepared.plan.applied as u64, bytes.values().sum());
                        Ok(ApplyOutcome::Indexed(IndexResponse {
                            indexed: prepared.plan.applied,
                            bytes_written: bytes,
                            shard_lag_ms: 0,
                        }))
                    }
                }
            };
            self.publish_storage_bytes(&state);
            drop(state);
            complete(&apply, outcome);
            drop(apply);
            drop(prepared);
            return Ok(true);
        }
    }
}

#[cfg(test)]
#[path = "committed_index_apply_tests.rs"]
mod tests;
