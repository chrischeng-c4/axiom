//! Borrowed, conservative pending-change cost estimates for committed records.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalyzerKind {
    WhitespaceLower,
    Jieba,
    Ngram,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextUpperBound {
    /// Repetitions are possible distinct terms. No token set is needed.
    pub terms: usize,
    pub total_utf8_bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormalizeError {
    Overflow,
    /// Caller must retain the record and obtain schema/live coverage. Never discard.
    ContextMissing,
}

/// This allocates no normalized tokens.  It mirrors the public analyzer's
/// lowercasing walk and counts scalar windows with checked arithmetic.
pub fn text_upper_bound(
    input: &str,
    analyzer: AnalyzerKind,
    ngram_min: usize,
    ngram_max: usize,
) -> Result<TextUpperBound, NormalizeError> {
    match analyzer {
        AnalyzerKind::Ngram => {
            ngram_upper_bound(lowered_non_whitespace_scalars(input)?, ngram_min, ngram_max)
        }
        AnalyzerKind::WhitespaceLower => whitespace_lower_upper_bound(input),
        AnalyzerKind::Jieba => jieba_upper_bound(input),
    }
}

/// Exactly count valid windows after the public n-gram normalizer removes
/// whitespace and expands `char::to_lowercase`.  A scalar emits at most four
/// UTF-8 bytes, so this is a byte upper bound without a token Vec or a window
/// allocation.
pub fn ngram_upper_bound(
    scalars: usize,
    ngram_min: usize,
    ngram_max: usize,
) -> Result<TextUpperBound, NormalizeError> {
    if ngram_min == 0 || ngram_min > ngram_max {
        return Ok(TextUpperBound::default());
    }
    let mut terms = 0usize;
    let mut bytes = 0usize;
    for width in ngram_min..=ngram_max {
        let Some(windows) = scalars.checked_sub(width).and_then(|n| n.checked_add(1)) else {
            continue;
        };
        terms = add(terms, windows)?;
        bytes = add(bytes, mul(mul(windows, width)?, 4)?)?;
    }
    Ok(TextUpperBound {
        terms,
        total_utf8_bytes: bytes,
    })
}

fn lowered_non_whitespace_scalars(input: &str) -> Result<usize, NormalizeError> {
    let mut count = 0usize;
    for character in input.chars().filter(|character| !character.is_whitespace()) {
        for _ in character.to_lowercase() {
            count = add(count, 1)?;
        }
    }
    Ok(count)
}

fn lowered_utf8_bytes(input: &str) -> Result<usize, NormalizeError> {
    let mut bytes = 0usize;
    for character in input.chars() {
        for lowered in character.to_lowercase() {
            bytes = add(bytes, lowered.len_utf8())?;
        }
    }
    Ok(bytes)
}

/// Match `index_text::for_whitespace_lower_cow` without creating output
/// strings.  Punctuation-only words produce no token.
fn whitespace_lower_upper_bound(mut text: &str) -> Result<TextUpperBound, NormalizeError> {
    let mut terms = 0usize;
    let mut bytes = 0usize;
    while !text.is_empty() {
        let trimmed = text.trim_start();
        if trimmed.is_empty() {
            break;
        }
        text = trimmed;
        let end = text.find(char::is_whitespace).unwrap_or(text.len());
        let raw = &text[..end];
        text = &text[end..];
        let token = raw.trim_matches(|character: char| !character.is_alphanumeric());
        if !token.is_empty() {
            terms = add(terms, 1)?;
            bytes = add(bytes, lowered_utf8_bytes(token)?)?;
        }
    }
    Ok(TextUpperBound {
        terms,
        total_utf8_bytes: bytes,
    })
}

/// With the `jieba` feature, cuts are non-overlapping lowered substrings.  The
/// fallback emits CJK bigrams, where an input scalar can occur in two output
/// windows.  Count every non-whitespace lowered scalar as a possible term and
/// charge at most two four-byte appearances.  This is conservative for both
/// feature modes without constructing the fallback's `Vec<char>`.
fn jieba_upper_bound(input: &str) -> Result<TextUpperBound, NormalizeError> {
    let scalars = lowered_non_whitespace_scalars(input)?;
    Ok(TextUpperBound {
        terms: scalars,
        total_utf8_bytes: mul(scalars, 8)?,
    })
}
fn add(left: usize, right: usize) -> Result<usize, NormalizeError> {
    left.checked_add(right).ok_or(NormalizeError::Overflow)
}
fn mul(left: usize, right: usize) -> Result<usize, NormalizeError> {
    left.checked_mul(right).ok_or(NormalizeError::Overflow)
}

/// Borrowed record normalizer used by the Engine admission seam.
use crate::change_memory_cost::{
    estimate_change, Change, Cost, CostError, FieldCost, VectorBackendCost,
};
use crate::log_entry::RaftLogEntry;
use crate::types::{Analyzer, FieldSpec, FieldType, FieldValue, VectorBackend};

/// One fixed table, reserved before exact pricing starts. The table never owns
/// input or heap tokens. All other analyzers keep their allocation-free bound.
pub(crate) const DEFAULT_NGRAM_COST_WORKSPACE_BYTES: usize =
    std::mem::size_of::<NgramDistinctTable>();
const DEFAULT_NGRAM_DISTINCT_CAP: usize = 256;

#[repr(C)]
#[derive(Clone, Copy)]
struct NgramSlot {
    occupied: bool,
    hash: u64,
    len: u8,
    bytes: [u8; 12],
}

struct NgramDistinctTable {
    slots: [NgramSlot; DEFAULT_NGRAM_DISTINCT_CAP],
    full: bool,
}

impl NgramDistinctTable {
    fn new() -> Self {
        Self {
            slots: [NgramSlot {
                occupied: false,
                hash: 0,
                len: 0,
                bytes: [0; 12],
            }; DEFAULT_NGRAM_DISTINCT_CAP],
            full: false,
        }
    }

    fn add(&mut self, token: &str) {
        let hash = token
            .as_bytes()
            .iter()
            .fold(14695981039346656037_u64, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(1099511628211)
            });
        self.add_with_hash(token, hash);
    }

    fn add_with_hash(&mut self, token: &str, hash: u64) {
        if self.full {
            return;
        }
        let bytes = token.as_bytes();
        if bytes.len() > 12 {
            self.full = true;
            return;
        }
        let mut index = (hash as usize) % DEFAULT_NGRAM_DISTINCT_CAP;
        for _ in 0..DEFAULT_NGRAM_DISTINCT_CAP {
            let slot = &mut self.slots[index];
            if !slot.occupied {
                slot.occupied = true;
                slot.hash = hash;
                slot.len = bytes.len() as u8;
                slot.bytes[..bytes.len()].copy_from_slice(bytes);
                return;
            }
            if slot.hash == hash
                && usize::from(slot.len) == bytes.len()
                && slot.bytes[..bytes.len()] == *bytes
            {
                return;
            }
            index = (index + 1) % DEFAULT_NGRAM_DISTINCT_CAP;
        }
        self.full = true;
    }

    fn bound(&mut self, input: &str) -> Result<TextUpperBound, NormalizeError> {
        // Each Text cell creates its own TokenSet during apply. Do not carry a
        // previous field, document, or discarded validation prefix into its cost.
        for slot in &mut self.slots {
            slot.occupied = false;
        }
        self.full = false;
        let conservative = text_upper_bound(
            input,
            AnalyzerKind::Ngram,
            crate::tokenize::DEFAULT_NGRAM_MIN,
            crate::tokenize::DEFAULT_NGRAM_MAX,
        )?;
        let streamed = crate::ngram_stream::stream_default_ngrams(input, |token| {
            self.add(token);
            if self.full {
                Err(())
            } else {
                Ok(())
            }
        });
        match streamed {
            Ok(_) => (),
            Err(crate::ngram_stream::NgramStreamError::Callback(())) => return Ok(conservative),
            Err(crate::ngram_stream::NgramStreamError::TokenCountOverflow) => {
                return Err(NormalizeError::Overflow);
            }
        }
        let mut bound = TextUpperBound::default();
        for slot in self.slots.iter().filter(|slot| slot.occupied) {
            bound.terms = add(bound.terms, 1)?;
            bound.total_utf8_bytes = add(bound.total_utf8_bytes, usize::from(slot.len))?;
        }
        Ok(bound)
    }
}

/// Borrowed collection view while Engine holds the same state write lock
/// that will make the following mutation. No request, schema, coverage, or
/// posting-list clone is allowed.
pub trait CostContext {
    fn collection_exists(&self, collection_id: &str) -> bool;
    fn index_cell_is_stale(
        &self,
        collection_id: &str,
        external_id: &str,
        field: &str,
        version: Option<u64>,
    ) -> bool;
    fn field_spec<'a>(&'a self, collection_id: &str, field: &str) -> Option<&'a FieldSpec>;
    fn known_external_id(&self, collection_id: &str, external_id: &str) -> bool;
    fn visit_coverage(&self, collection_id: &str, external_id: &str, visit: &mut dyn FnMut(&str));
    fn request_is_deduplicated(&self, collection_id: &str, request_id: Option<&str>) -> bool;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RecordCost {
    pub active: usize,
    pub frozen: usize,
    pub prepublish: usize,
}

/// An explicit retain signal. A committed entry that cannot be normalized
/// or reserved stays queued for wait/spill; it is never discarded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordEstimate {
    Ready(RecordCost),
    Retain { cause: NormalizeError },
}

impl RecordCost {
    fn add_cost(&mut self, cost: Cost) -> Result<(), NormalizeError> {
        self.active = self
            .active
            .checked_add(cost.active)
            .ok_or(NormalizeError::Overflow)?;
        self.frozen = self
            .frozen
            .checked_add(cost.frozen)
            .ok_or(NormalizeError::Overflow)?;
        self.prepublish = self
            .prepublish
            .checked_add(cost.prepublish)
            .ok_or(NormalizeError::Overflow)?;
        Ok(())
    }
    fn add_change(&mut self, change: &Change) -> Result<(), NormalizeError> {
        self.add_cost(
            estimate_change(change).map_err(|CostError::Overflow| NormalizeError::Overflow)?,
        )
    }
    fn add_request_id(&mut self, request_id: Option<&str>) -> Result<(), NormalizeError> {
        let Some(request_id) = request_id else {
            return Ok(());
        };
        // seen_requests is VecDeque<(String, Instant)> and is not part of
        // FrozenCheckpoint. This is one String allocation bound plus its
        // queue tuple and deque slot; it is active-only.
        let string = request_id
            .len()
            .checked_add(16)
            .and_then(usize::checked_next_power_of_two)
            .ok_or(NormalizeError::Overflow)?;
        self.active = self
            .active
            .checked_add(string)
            .and_then(|n| n.checked_add(32))
            .ok_or(NormalizeError::Overflow)?;
        Ok(())
    }
}

pub fn estimate_record_or_retain(entry: &RaftLogEntry, ctx: &impl CostContext) -> RecordEstimate {
    match estimate_record(entry, ctx) {
        Ok(cost) => RecordEstimate::Ready(cost),
        Err(cause) => RecordEstimate::Retain { cause },
    }
}

/// Estimate one committed entry before its first state mutation. Validation
/// precedes this call; ContextMissing means retain the entry and normalize
/// under the owning Engine lock after validation.
pub fn estimate_record(
    entry: &RaftLogEntry,
    ctx: &impl CostContext,
) -> Result<RecordCost, NormalizeError> {
    estimate_record_with_text_representation(entry, ctx, TextRepresentation::Normalized, None)
}

/// Estimate a record whose valid Text cells will be prepared as immutable
/// on-disk rows before apply.
///
/// The caller must separately reserve raw transport, staging workspace, row
/// handles, and reader metadata. This estimate alone never authorizes apply.
pub(crate) fn estimate_record_prepared_text(
    entry: &RaftLogEntry,
    ctx: &impl CostContext,
) -> Result<RecordCost, NormalizeError> {
    estimate_record_with_text_representation(entry, ctx, TextRepresentation::PreparedRow, None)
}

/// Select potential Ngram work without allocating a token table. This may
/// over-select invalid or stale cells; the shared cost walker decides their
/// actual semantics later. Non-Text records need no new workspace reservation.
pub(crate) fn may_need_default_ngram_workspace(
    entry: &RaftLogEntry,
    ctx: &impl CostContext,
) -> bool {
    let is_ngram = |collection: &str, field: &str, value: &FieldValue| {
        matches!(value, FieldValue::String(_))
            && ctx.field_spec(collection, field).is_some_and(|spec| {
                spec.field_type == FieldType::Text && spec.analyzer == Some(Analyzer::Ngram)
            })
    };
    match entry {
        RaftLogEntry::Index { collection_id, req } => req
            .items
            .iter()
            .any(|item| is_ngram(collection_id, &item.field, &item.value)),
        RaftLogEntry::ReplaceDocs { collection_id, req } => req.docs.iter().any(|doc| {
            doc.fields
                .iter()
                .any(|(field, value)| is_ngram(collection_id, field, value))
        }),
        _ => false,
    }
}

/// The caller owns DEFAULT_NGRAM_COST_WORKSPACE_BYTES before entering this
/// function. The fixed table is gone before it returns the final record price.
pub(crate) fn estimate_record_exact_default_ngram(
    entry: &RaftLogEntry,
    ctx: &impl CostContext,
) -> RecordEstimate {
    let mut table = NgramDistinctTable::new();
    match estimate_record_with_text_representation(
        entry,
        ctx,
        TextRepresentation::Normalized,
        Some(&mut table),
    ) {
        Ok(cost) => RecordEstimate::Ready(cost),
        Err(cause) => RecordEstimate::Retain { cause },
    }
}

#[derive(Clone, Copy)]
enum TextRepresentation {
    Normalized,
    PreparedRow,
}

fn estimate_record_with_text_representation(
    entry: &RaftLogEntry,
    ctx: &impl CostContext,
    text_representation: TextRepresentation,
    mut ngram_table: Option<&mut NgramDistinctTable>,
) -> Result<RecordCost, NormalizeError> {
    match entry {
        RaftLogEntry::CreateCollection { collection_id, req } => {
            let bytes = collection_id
                .len()
                .checked_add(req.fields.iter().try_fold(0usize, |n, (name, _)| {
                    n.checked_add(name.len() + 64)
                        .ok_or(NormalizeError::Overflow)
                })?)
                .ok_or(NormalizeError::Overflow)?;
            one(Change::Schema {
                metadata_bytes: bytes,
            })
        }
        RaftLogEntry::Index { collection_id, req } => {
            // The storage path rejects a missing collection before it interns
            // an ID or records request metadata.
            if !ctx.collection_exists(collection_id) {
                return Ok(RecordCost::default());
            }
            if ctx.request_is_deduplicated(collection_id, req.request_id.as_deref()) {
                return Ok(RecordCost::default());
            }
            let mut out = RecordCost::default();
            for item in &req.items {
                if ctx.index_cell_is_stale(
                    collection_id,
                    &item.external_id,
                    &item.field,
                    item.version,
                ) {
                    continue;
                }
                let Some(spec) = ctx.field_spec(collection_id, &item.field) else {
                    charge_index_error_prefix(&mut out, ctx, collection_id, item, false)?;
                    out.add_request_id(req.request_id.as_deref())?;
                    return Ok(out);
                };
                let field = match field_cost(
                    spec,
                    &item.value,
                    text_representation,
                    ngram_table.as_deref_mut(),
                ) {
                    Ok(field) => field,
                    Err(FieldCostError::Invalid) => {
                        charge_index_error_prefix(&mut out, ctx, collection_id, item, true)?;
                        out.add_request_id(req.request_id.as_deref())?;
                        return Ok(out);
                    }
                    Err(FieldCostError::Overflow) => return Err(NormalizeError::Overflow),
                };
                out.add_change(&Change::Index {
                    external_id_bytes: item.external_id.len(),
                    new_document: !ctx.known_external_id(collection_id, &item.external_id),
                    field,
                    volatile_metadata_bytes: item.version.map(|_| 96).unwrap_or(0),
                })?;
                out.add_change(&Change::Direct {
                    metadata_bytes: field_metadata_bytes(&item.field)?,
                })?;
            }
            out.add_request_id(req.request_id.as_deref())?;
            Ok(out)
        }
        RaftLogEntry::ReplaceDocs { collection_id, req } => {
            // replace_docs rejects a missing collection before it touches any
            // document. A malformed document is per-item, so earlier valid
            // document replacements remain applied and must stay charged.
            if !ctx.collection_exists(collection_id) {
                return Ok(RecordCost::default());
            }
            let mut out = RecordCost::default();
            for doc in &req.docs {
                let mut fields = Vec::with_capacity(doc.fields.len());
                let mut sidecars: usize = doc.version.map(|_| 96).unwrap_or(0);
                let mut invalid = false;
                for (name, value) in &doc.fields {
                    let Some(spec) = ctx.field_spec(collection_id, name) else {
                        invalid = true;
                        break;
                    };
                    if matches!(spec.field_type, FieldType::Text | FieldType::Vector) {
                        sidecars = sidecars
                            .checked_add(name.len() + 96)
                            .ok_or(NormalizeError::Overflow)?;
                    }
                    match field_cost(spec, value, text_representation, ngram_table.as_deref_mut()) {
                        Ok(field) => fields.push(field),
                        // replace_one_doc validates its complete field map
                        // before it deletes or writes this document. The
                        // accumulated earlier documents are still a real
                        // prefix, but this document is not.
                        Err(FieldCostError::Invalid) => {
                            invalid = true;
                            break;
                        }
                        Err(FieldCostError::Overflow) => return Err(NormalizeError::Overflow),
                    }
                }
                if invalid {
                    charge_replace_invalid_prefix(&mut out, ctx, collection_id, doc)?;
                    continue;
                }
                out.add_change(&Change::Replace {
                    external_id_bytes: doc.external_id.len(),
                    new_document: !ctx.known_external_id(collection_id, &doc.external_id),
                    fields,
                    volatile_metadata_bytes: sidecars,
                })?;
                for name in doc.fields.keys() {
                    out.add_change(&Change::Direct {
                        metadata_bytes: field_metadata_bytes(name)?,
                    })?;
                }
                // Omitted declared-and-present fields become dirty tombstones.
                let mut omitted = 0usize;
                ctx.visit_coverage(collection_id, &doc.external_id, &mut |name| {
                    if !doc.fields.contains_key(name) {
                        omitted = omitted.saturating_add(1);
                    }
                });
                if omitted != 0 {
                    out.add_change(&Change::Unindex {
                        external_id_bytes: doc.external_id.len(),
                        field_count: omitted,
                    })?;
                }
            }
            Ok(out)
        }
        RaftLogEntry::UnindexDocs { collection_id, req } => {
            let mut out = RecordCost::default();
            for external_id in &req.external_ids {
                let mut fields = 0usize;
                ctx.visit_coverage(collection_id, external_id, &mut |_| {
                    fields = fields.saturating_add(1);
                });
                if fields != 0 {
                    out.add_change(&Change::Unindex {
                        external_id_bytes: external_id.len(),
                        field_count: fields,
                    })?;
                }
            }
            Ok(out)
        }
        RaftLogEntry::Delete {
            collection_id,
            external_id,
            field,
        } => {
            let mut fields = 0usize;
            if ctx.known_external_id(collection_id, external_id) {
                match field {
                    Some(_) => fields = 1, // current delete marks this field dirty
                    None => ctx.visit_coverage(collection_id, external_id, &mut |_| {
                        fields = fields.saturating_add(1);
                    }),
                }
            }
            if fields == 0 {
                Ok(RecordCost::default())
            } else {
                one(Change::Unindex {
                    external_id_bytes: external_id.len(),
                    field_count: fields,
                })
            }
        }
        // Retired old state is already charged by the reclaimer owner.
        // No negative capacity credit is created here.
        RaftLogEntry::TruncateDocs { .. } | RaftLogEntry::DropCollection { .. } => {
            Ok(RecordCost::default())
        }
        RaftLogEntry::AddField {
            collection_id,
            field_name,
            spec: _,
        }
        | RaftLogEntry::DropField {
            collection_id,
            field_name,
        } => one(Change::Schema {
            metadata_bytes: collection_id.len() + field_name.len() + 64,
        }),
    }
}

/// The index path interns the group external ID before field lookup and may
/// drop an existing field before its value validation fails. Charge that
/// concrete prefix without guessing a field payload that never became live.
fn charge_index_error_prefix(
    out: &mut RecordCost,
    ctx: &impl CostContext,
    collection_id: &str,
    item: &crate::types::IndexItem,
    declared_field: bool,
) -> Result<(), NormalizeError> {
    if !ctx.known_external_id(collection_id, &item.external_id) {
        out.add_change(&Change::Direct {
            metadata_bytes: item
                .external_id
                .len()
                .checked_add(item.field.len())
                .and_then(|bytes| bytes.checked_add(128))
                .ok_or(NormalizeError::Overflow)?,
        })?;
        return Ok(());
    }
    if declared_field {
        // apply_value errors call mark_field_dirty even when coverage did not
        // yet contain this declared field.
        out.add_change(&Change::Unindex {
            external_id_bytes: item.external_id.len(),
            field_count: 1,
        })?;
        out.add_change(&Change::Direct {
            metadata_bytes: field_metadata_bytes(&item.field)?,
        })?;
    }
    Ok(())
}

fn charge_replace_invalid_prefix(
    out: &mut RecordCost,
    ctx: &impl CostContext,
    collection_id: &str,
    doc: &crate::types::ReplaceDocItem,
) -> Result<(), NormalizeError> {
    if !ctx.known_external_id(collection_id, &doc.external_id) {
        out.add_change(&Change::Direct {
            metadata_bytes: doc
                .external_id
                .len()
                .checked_add(128)
                .ok_or(NormalizeError::Overflow)?,
        })?;
    }
    Ok(())
}

fn field_metadata_bytes(field: &str) -> Result<usize, NormalizeError> {
    field.len().checked_add(96).ok_or(NormalizeError::Overflow)
}

fn one(change: Change) -> Result<RecordCost, NormalizeError> {
    let mut out = RecordCost::default();
    out.add_change(&change)?;
    Ok(out)
}

enum FieldCostError {
    Overflow,
    Invalid,
}

fn field_cost(
    spec: &FieldSpec,
    value: &FieldValue,
    text_representation: TextRepresentation,
    ngram_table: Option<&mut NgramDistinctTable>,
) -> Result<FieldCost, FieldCostError> {
    match (&spec.field_type, value) {
        (FieldType::Keyword, FieldValue::String(value)) => Ok(FieldCost::Keyword {
            value_bytes: value.len(),
        }),
        (FieldType::Number, FieldValue::Number(_)) => Ok(FieldCost::Number),
        (FieldType::Set, FieldValue::StringList(values)) => Ok(FieldCost::Set {
            members: values.len(),
            member_bytes: values.iter().try_fold(0usize, |n, value| {
                n.checked_add(value.len()).ok_or(FieldCostError::Overflow)
            })?,
        }),
        (FieldType::Hash, FieldValue::String(_)) => Ok(FieldCost::Hash),
        (FieldType::Text, FieldValue::String(value)) => {
            if matches!(text_representation, TextRepresentation::PreparedRow) {
                return Ok(FieldCost::Text {
                    distinct_terms: 0,
                    total_term_bytes: 0,
                });
            }
            let analyzer = match spec.analyzer.unwrap_or(Analyzer::WhitespaceLower) {
                Analyzer::WhitespaceLower => AnalyzerKind::WhitespaceLower,
                Analyzer::Jieba => AnalyzerKind::Jieba,
                Analyzer::Ngram => AnalyzerKind::Ngram,
            };
            let TextUpperBound {
                terms,
                total_utf8_bytes,
            } = match (analyzer, ngram_table) {
                (AnalyzerKind::Ngram, Some(table)) => table.bound(value),
                _ => text_upper_bound(
                    value,
                    analyzer,
                    crate::tokenize::DEFAULT_NGRAM_MIN,
                    crate::tokenize::DEFAULT_NGRAM_MAX,
                ),
            }
            .map_err(|_| FieldCostError::Overflow)?;
            Ok(FieldCost::Text {
                distinct_terms: terms,
                total_term_bytes: total_utf8_bytes,
            })
        }
        (FieldType::Vector, FieldValue::Vector(value)) => {
            let dim = usize::try_from(spec.dim.ok_or(FieldCostError::Invalid)?)
                .map_err(|_| FieldCostError::Overflow)?;
            if value.len() != dim {
                return Err(FieldCostError::Invalid);
            }
            let backend = match spec.backend.unwrap_or(VectorBackend::HnswCpu) {
                VectorBackend::FlatCpu => VectorBackendCost::Flat,
                VectorBackend::HnswCpu => VectorBackendCost::Hnsw,
            };
            Ok(FieldCost::Vector {
                dim,
                backend,
                quantized_sq: spec.quantize.is_some(),
            })
        }
        _ => Err(FieldCostError::Invalid),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    use crate::types::{IndexItem, IndexRequest, ReplaceDocItem, ReplaceDocsRequest};

    #[derive(Default)]
    struct TestContext {
        fields: BTreeMap<(String, String), FieldSpec>,
        known: BTreeSet<(String, String)>,
        coverage: BTreeMap<(String, String), BTreeSet<String>>,
        stale: BTreeSet<(String, String, String, u64)>,
    }

    impl TestContext {
        fn keyword(&mut self, collection: &str, field: &str) {
            self.fields.insert(
                (collection.to_owned(), field.to_owned()),
                FieldSpec {
                    field_type: FieldType::Keyword,
                    analyzer: None,
                    multi: None,
                    dim: None,
                    metric: None,
                    backend: None,
                    quantize: None,
                },
            );
        }

        fn text(&mut self, collection: &str, field: &str) {
            self.fields.insert(
                (collection.to_owned(), field.to_owned()),
                FieldSpec {
                    field_type: FieldType::Text,
                    analyzer: Some(Analyzer::WhitespaceLower),
                    multi: None,
                    dim: None,
                    metric: None,
                    backend: None,
                    quantize: None,
                },
            );
        }
    }

    impl CostContext for TestContext {
        fn collection_exists(&self, collection: &str) -> bool {
            self.fields.keys().any(|(known, _)| known == collection)
        }

        fn index_cell_is_stale(
            &self,
            collection: &str,
            external_id: &str,
            field: &str,
            version: Option<u64>,
        ) -> bool {
            version.is_some_and(|version| {
                self.stale.contains(&(
                    collection.to_owned(),
                    external_id.to_owned(),
                    field.to_owned(),
                    version,
                ))
            })
        }

        fn field_spec<'a>(&'a self, collection: &str, field: &str) -> Option<&'a FieldSpec> {
            self.fields.get(&(collection.to_owned(), field.to_owned()))
        }

        fn known_external_id(&self, collection: &str, external_id: &str) -> bool {
            self.known
                .contains(&(collection.to_owned(), external_id.to_owned()))
        }

        fn visit_coverage(&self, collection: &str, external_id: &str, visit: &mut dyn FnMut(&str)) {
            if let Some(fields) = self
                .coverage
                .get(&(collection.to_owned(), external_id.to_owned()))
            {
                for field in fields {
                    visit(field);
                }
            }
        }

        fn request_is_deduplicated(&self, _collection: &str, _request_id: Option<&str>) -> bool {
            false
        }
    }

    fn ready(estimate: RecordEstimate) -> RecordCost {
        match estimate {
            RecordEstimate::Ready(cost) => cost,
            RecordEstimate::Retain { cause } => {
                panic!("must not retain ordinary apply error: {cause:?}")
            }
        }
    }

    #[test]
    fn ngram_bound_uses_windows_without_allocating_tokens() {
        let bound = ngram_upper_bound(4, 2, 3).unwrap();
        assert_eq!(bound.terms, 5);
        assert_eq!(bound.total_utf8_bytes, (3 * 2 + 2 * 3) * 4);
    }

    #[test]
    fn text_bound_counts_repetitions_and_unicode_lower_expansion() {
        let bound = text_upper_bound("İİ", AnalyzerKind::WhitespaceLower, 2, 3).unwrap();
        assert_eq!(bound.terms, 1);
        assert_eq!(bound.total_utf8_bytes, 6);
    }

    #[test]
    fn invalid_ngram_range_is_empty_and_large_inputs_overflow() {
        assert_eq!(
            ngram_upper_bound(9, 3, 2).unwrap(),
            TextUpperBound::default()
        );
        assert_eq!(
            ngram_upper_bound(usize::MAX, 1, 1),
            Err(NormalizeError::Overflow)
        );
    }

    #[test]
    fn mixed_index_prefix_is_charged_when_a_later_item_is_type_invalid() {
        let mut ctx = TestContext::default();
        ctx.keyword("c", "tag");
        let entry = RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items: vec![
                    IndexItem {
                        external_id: "doc".into(),
                        field: "tag".into(),
                        value: FieldValue::String("kept".into()),
                        version: None,
                    },
                    IndexItem {
                        external_id: "doc".into(),
                        field: "tag".into(),
                        value: FieldValue::Number(7.0),
                        version: None,
                    },
                ],
                request_id: Some("idempotency-key".into()),
            },
        };

        let cost = ready(estimate_record_or_retain(&entry, &ctx));
        assert!(
            cost.active > 0 || cost.frozen > 0,
            "the earlier indexed field and request metadata survive the later validation error"
        );
    }

    #[test]
    fn replacement_deletion_prefix_is_charged_when_a_later_doc_is_invalid() {
        let mut ctx = TestContext::default();
        ctx.keyword("c", "tag");
        ctx.known.insert(("c".into(), "old".into()));
        ctx.coverage
            .insert(("c".into(), "old".into()), BTreeSet::from(["tag".into()]));
        let entry = RaftLogEntry::ReplaceDocs {
            collection_id: "c".into(),
            req: ReplaceDocsRequest {
                docs: vec![
                    ReplaceDocItem {
                        external_id: "old".into(),
                        version: None,
                        fields: BTreeMap::new(),
                    },
                    ReplaceDocItem {
                        external_id: "later".into(),
                        version: None,
                        fields: BTreeMap::from([(
                            "missing".into(),
                            FieldValue::String("invalid".into()),
                        )]),
                    },
                ],
            },
        };

        let cost = ready(estimate_record_or_retain(&entry, &ctx));
        assert!(
            cost.active > 0 || cost.frozen > 0,
            "the first replacement deletes tag before the second document reports its error"
        );
    }

    #[test]
    fn invalid_replacement_doc_does_not_hide_a_later_valid_document_cost() {
        let mut ctx = TestContext::default();
        ctx.keyword("c", "tag");
        let entry = RaftLogEntry::ReplaceDocs {
            collection_id: "c".into(),
            req: ReplaceDocsRequest {
                docs: vec![
                    ReplaceDocItem {
                        external_id: "bad".into(),
                        version: None,
                        fields: BTreeMap::from([(
                            "missing".into(),
                            FieldValue::String("invalid".into()),
                        )]),
                    },
                    ReplaceDocItem {
                        external_id: "later".into(),
                        version: None,
                        fields: BTreeMap::from([("tag".into(), FieldValue::String("kept".into()))]),
                    },
                ],
            },
        };

        let cost = ready(estimate_record_or_retain(&entry, &ctx));
        assert!(
            cost.active > 0 || cost.frozen > 0,
            "replace_docs continues after an item error and applies the later document"
        );
    }

    #[test]
    fn invalid_new_replacement_document_charges_its_interned_id_metadata() {
        let mut ctx = TestContext::default();
        ctx.keyword("c", "tag");
        let entry = RaftLogEntry::ReplaceDocs {
            collection_id: "c".into(),
            req: ReplaceDocsRequest {
                docs: vec![ReplaceDocItem {
                    external_id: "new-invalid".into(),
                    version: None,
                    fields: BTreeMap::from([(
                        "missing".into(),
                        FieldValue::String("invalid".into()),
                    )]),
                }],
            },
        };

        let cost = ready(estimate_record_or_retain(&entry, &ctx));
        assert!(
            cost.active > 0 || cost.frozen > 0,
            "replace_one_doc interns the ID before it validates fields"
        );
    }

    #[test]
    fn invalid_existing_index_value_charges_dirty_declared_field_without_coverage() {
        let mut ctx = TestContext::default();
        ctx.keyword("c", "tag");
        ctx.known.insert(("c".into(), "existing".into()));
        let entry = RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: "existing".into(),
                    field: "tag".into(),
                    value: FieldValue::Number(7.0),
                    version: None,
                }],
                request_id: None,
            },
        };

        let cost = ready(estimate_record_or_retain(&entry, &ctx));
        assert!(
            cost.active > 0 || cost.frozen > 0,
            "apply errors mark the declared field dirty even when it had no prior coverage"
        );
    }

    #[test]
    fn stale_invalid_index_item_does_not_hide_later_valid_work() {
        let mut ctx = TestContext::default();
        ctx.keyword("c", "tag");
        ctx.known.insert(("c".into(), "existing".into()));
        ctx.stale
            .insert(("c".into(), "existing".into(), "tag".into(), 7));
        let later = IndexItem {
            external_id: "existing".into(),
            field: "tag".into(),
            value: FieldValue::String("kept".into()),
            version: Some(8),
        };
        let stale_then_later = RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items: vec![
                    IndexItem {
                        external_id: "existing".into(),
                        field: "tag".into(),
                        value: FieldValue::Number(7.0),
                        version: Some(7),
                    },
                    later.clone(),
                ],
                request_id: None,
            },
        };
        let later_only = RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items: vec![later],
                request_id: None,
            },
        };
        assert_eq!(
            ready(estimate_record_or_retain(&stale_then_later, &ctx)),
            ready(estimate_record_or_retain(&later_only, &ctx)),
            "storage skips stale cells before value validation and continues the batch"
        );
    }

    #[test]
    fn missing_collection_is_a_noop_cost_not_indefinite_context_retention() {
        let ctx = TestContext::default();
        let entry = RaftLogEntry::Index {
            collection_id: "missing".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: "doc".into(),
                    field: "tag".into(),
                    value: FieldValue::String("value".into()),
                    version: None,
                }],
                request_id: None,
            },
        };
        assert_eq!(
            ready(estimate_record_or_retain(&entry, &ctx)),
            RecordCost::default()
        );
    }

    #[test]
    fn text_record_cost_uses_the_borrowed_upper_bound_without_normalized_tokens() {
        let mut ctx = TestContext::default();
        ctx.text("c", "body");
        let entry = RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: "doc".into(),
                    field: "body".into(),
                    value: FieldValue::String("İstanbul rust".into()),
                    version: None,
                }],
                request_id: None,
            },
        };
        let cost = ready(estimate_record_or_retain(&entry, &ctx));
        assert!(cost.active > 0 || cost.frozen > 0);
    }

    fn total_cost(cost: RecordCost) -> usize {
        cost.active + cost.frozen + cost.prepublish
    }

    fn cost_delta(larger: RecordCost, smaller: RecordCost) -> RecordCost {
        RecordCost {
            active: larger.active - smaller.active,
            frozen: larger.frozen - smaller.frozen,
            prepublish: larger.prepublish - smaller.prepublish,
        }
    }

    #[test]
    fn prepared_jieba_removes_only_the_normalized_term_map_charge() {
        let mut ctx = TestContext::default();
        ctx.text("c", "body");
        ctx.fields
            .get_mut(&(String::from("c"), String::from("body")))
            .unwrap()
            .analyzer = Some(Analyzer::Jieba);
        let entry = RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                request_id: None,
                items: vec![IndexItem {
                    external_id: "doc".into(),
                    field: "body".into(),
                    value: FieldValue::String("搜尋引擎 ΣΟΣ".into()),
                    version: None,
                }],
            },
        };
        assert!(
            total_cost(estimate_record_prepared_text(&entry, &ctx).unwrap())
                < total_cost(estimate_record(&entry, &ctx).unwrap())
        );
    }

    #[test]
    fn prepared_text_removes_only_the_normalized_term_map_charge() {
        let mut ctx = TestContext::default();
        ctx.text("c", "body");
        ctx.keyword("c", "tag");
        let body = "distinct-term ".repeat(4_096);
        let text_only = RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items: vec![IndexItem {
                    external_id: "doc".into(),
                    field: "body".into(),
                    value: FieldValue::String(body.clone()),
                    version: Some(1),
                }],
                request_id: Some("text-request".into()),
            },
        };
        let mixed_prefix_error = RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items: vec![
                    IndexItem {
                        external_id: "doc".into(),
                        field: "body".into(),
                        value: FieldValue::String(body),
                        version: Some(1),
                    },
                    IndexItem {
                        external_id: "doc".into(),
                        field: "tag".into(),
                        value: FieldValue::String("kept".into()),
                        version: Some(2),
                    },
                    IndexItem {
                        external_id: "doc".into(),
                        field: "tag".into(),
                        value: FieldValue::Number(7.0),
                        version: Some(3),
                    },
                ],
                request_id: Some("mixed-request".into()),
            },
        };

        let normal_text = estimate_record(&text_only, &ctx).unwrap();
        let prepared_text = estimate_record_prepared_text(&text_only, &ctx).unwrap();
        assert!(
            total_cost(prepared_text) < total_cost(normal_text),
            "prepared Text must not reserve an in-RAM normalized term map"
        );
        assert!(
            total_cost(prepared_text) > 0,
            "Text row, ID, version, and request metadata stay charged"
        );
        assert_eq!(
            estimate_record(&text_only, &ctx).unwrap(),
            normal_text,
            "the ordinary estimator remains unchanged"
        );

        let normal_mixed = estimate_record(&mixed_prefix_error, &ctx).unwrap();
        let prepared_mixed = estimate_record_prepared_text(&mixed_prefix_error, &ctx).unwrap();
        assert_eq!(
            cost_delta(normal_mixed, prepared_mixed),
            cost_delta(normal_text, prepared_text),
            "keyword, ID/version/request metadata, and later partial-error work stay charged"
        );
    }
    fn exact_oracle(input: &str) -> TextUpperBound {
        let mut terms = BTreeSet::new();
        crate::ngram_stream::stream_default_ngrams(input, |token| {
            terms.insert(token.to_owned());
            Ok::<_, ()>(())
        })
        .unwrap();
        TextUpperBound {
            terms: terms.len(),
            total_utf8_bytes: terms.iter().map(String::len).sum(),
        }
    }

    #[test]
    fn exact_default_ngram_matches_repeated_and_unicode_terms() {
        let mut table = NgramDistinctTable::new();
        for input in [
            "durable search token ".repeat(16),
            "İİ 中文🦀 A\t中 İ".repeat(8),
            String::new(),
        ] {
            assert_eq!(table.bound(&input).unwrap(), exact_oracle(&input));
        }
    }

    #[test]
    fn exact_default_ngram_compares_full_bytes_on_hash_collisions() {
        let mut table = NgramDistinctTable::new();
        for token in ["ab", "ac", "中a", "中b", "ab", "中a"] {
            table.add_with_hash(token, 7);
        }
        let actual: BTreeSet<Vec<u8>> = table
            .slots
            .iter()
            .filter(|s| s.occupied)
            .map(|s| s.bytes[..usize::from(s.len)].to_vec())
            .collect();
        let expected = ["ab", "ac", "中a", "中b"]
            .into_iter()
            .map(|s| s.as_bytes().to_vec())
            .collect();
        assert_eq!(actual, expected);
        assert!(!table.full);
    }

    #[test]
    fn exact_default_ngram_overflow_falls_back_and_next_cell_resets() {
        let input: String = (0x4e00..0x4e00 + 300)
            .map(|n| char::from_u32(n).unwrap())
            .collect();
        assert!(exact_oracle(&input).terms > DEFAULT_NGRAM_DISTINCT_CAP);
        let mut table = NgramDistinctTable::new();
        assert_eq!(
            table.bound(&input).unwrap(),
            text_upper_bound(&input, AnalyzerKind::Ngram, 2, 3).unwrap()
        );
        assert!(
            table.full,
            "real distinct terms must exhaust the fixed table"
        );
        assert_eq!(table.bound("aaaaa").unwrap(), exact_oracle("aaaaa"));
        assert!(
            !table.full,
            "a previous large cell must not poison a later small one"
        );
    }

    fn ngram_context() -> TestContext {
        let mut ctx = TestContext::default();
        for field in ["a", "b"] {
            ctx.text("c", field);
            ctx.fields
                .get_mut(&("c".into(), field.into()))
                .unwrap()
                .analyzer = Some(Analyzer::Ngram);
        }
        ctx
    }

    #[test]
    fn exact_default_ngram_prices_each_field_and_document_separately() {
        let ctx = ngram_context();
        let items = vec![
            IndexItem {
                external_id: "one".into(),
                field: "a".into(),
                value: FieldValue::String("aaaaaa".into()),
                version: None,
            },
            IndexItem {
                external_id: "two".into(),
                field: "b".into(),
                value: FieldValue::String("中文中文".into()),
                version: None,
            },
        ];
        let entry = |items| RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                items,
                request_id: None,
            },
        };
        let separate: usize = items
            .iter()
            .cloned()
            .map(|item| {
                total_cost(ready(estimate_record_exact_default_ngram(
                    &entry(vec![item]),
                    &ctx,
                )))
            })
            .sum();
        assert_eq!(
            total_cost(ready(estimate_record_exact_default_ngram(
                &entry(items),
                &ctx
            ))),
            separate
        );
    }

    #[test]
    fn exact_default_ngram_invalid_replace_does_not_pollute_later_doc() {
        let ctx = ngram_context();
        let large: String = (0x4e00..0x4e00 + 300)
            .map(|n| char::from_u32(n).unwrap())
            .collect();
        let bad = ReplaceDocItem {
            external_id: "bad".into(),
            version: None,
            fields: BTreeMap::from([
                ("a".into(), FieldValue::String(large)),
                ("z-missing".into(), FieldValue::String("bad".into())),
            ]),
        };
        let good = ReplaceDocItem {
            external_id: "good".into(),
            version: None,
            fields: BTreeMap::from([("b".into(), FieldValue::String("aaaaaa".into()))]),
        };
        let entry = |docs| RaftLogEntry::ReplaceDocs {
            collection_id: "c".into(),
            req: ReplaceDocsRequest { docs },
        };
        let bad_cost = total_cost(ready(estimate_record_exact_default_ngram(
            &entry(vec![bad.clone()]),
            &ctx,
        )));
        let good_cost = total_cost(ready(estimate_record_exact_default_ngram(
            &entry(vec![good.clone()]),
            &ctx,
        )));
        let combined = total_cost(ready(estimate_record_exact_default_ngram(
            &entry(vec![bad, good]),
            &ctx,
        )));
        assert_eq!(combined, bad_cost + good_cost);
        assert!(good_cost > 0);
    }
}
