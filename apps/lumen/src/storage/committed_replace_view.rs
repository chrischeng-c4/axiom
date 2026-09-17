//! Allocation-free live-state adapter for committed `docs:replace` planning.
//!
//! This child module lends only state already owned by `Collection` and source
//! rows already lent by `FastIndexScanner`. Segment reads may use mmap, but it
//! performs no writes and creates no owned value containers.

use super::committed_index_plan::PlanView;
use super::committed_replace_plan::{OldFieldsBound, ParsedValues, ReplacePlanView};
use super::*;
use crate::types::FieldType;
use crate::wal::fast_index_scanner::{FastIndexScanner, FastIndexValue, FastStringList};
use std::time::Instant;

pub(super) struct View<'state, 'scan, 'wire> {
    pub(super) engine: &'state Engine,
    pub(super) coll: &'state Collection,
    pub(super) scanner: &'scan FastIndexScanner<'wire>,
    pub(super) parsed: &'state ParsedValues,
    pub(super) now: Instant,
}

impl PlanView for View<'_, '_, '_> {
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

impl ReplacePlanView for View<'_, '_, '_> {
    fn doc_version(&self, id: u32) -> Option<u64> {
        self.coll.doc_versions.get(&id).copied()
    }
    fn old_fields(&self, id: u32) -> Option<&[String]> {
        self.coll
            .eid_fields
            .get(&id)
            .map(|coverage| coverage.names.as_slice())
    }
    fn old_fields_bound(&self, id: u32) -> OldFieldsBound {
        let Some(fields) = self.old_fields(id) else {
            return OldFieldsBound::default();
        };
        let copied_bytes = fields
            .iter()
            .try_fold(0usize, |total, field| total.checked_add(field.len()))
            .unwrap_or(usize::MAX);
        OldFieldsBound {
            count: fields.len(),
            copied_bytes,
        }
    }
    fn existing_unchanged(
        &self,
        id: u32,
        field: &str,
        kind: FieldType,
        ordinal: usize,
        checksum: Option<u64>,
    ) -> bool {
        let Some(index) = self.coll.fields.get(field) else {
            return false;
        };
        let Some(item) = self.scanner.items().nth(ordinal) else {
            return false;
        };
        match (kind, index, item.value) {
            (FieldType::Keyword, FieldIndex::Keyword(keyword), FastIndexValue::String(want)) => {
                keyword_equal(keyword, id, want)
            }
            (FieldType::Number, FieldIndex::Number(number), FastIndexValue::Number(want)) => {
                SortableF64::new(want).is_ok_and(|want| number.number_at(id) == Some(want))
            }
            (FieldType::Set, FieldIndex::Set(set), FastIndexValue::StringList(want)) => {
                set_equal(set, id, want)
            }
            (FieldType::Hash, FieldIndex::Hash(hash), FastIndexValue::String(_)) => self
                .parsed
                .get(&ordinal)
                .and_then(|value| value.hash)
                .is_some_and(|want| hash.hash_at(id) == Some(want)),
            (FieldType::Text, FieldIndex::Text { .. }, FastIndexValue::String(_))
            | (FieldType::Vector, FieldIndex::Vector { .. }, FastIndexValue::Vector { .. }) => {
                checksum.is_some_and(|want| {
                    self.coll
                        .field_checksums
                        .get(&id)
                        .and_then(|fields| fields.get(field))
                        .is_some_and(|stored| *stored == want)
                })
            }
            _ => false,
        }
    }
}

fn keyword_equal(index: &KeywordIndex, id: u32, want: &str) -> bool {
    if let Some(value) = index
        .dense_forward
        .get(id as usize)
        .and_then(Option::as_ref)
    {
        return value == want;
    }
    if let Some(value) = index.forward.get(&id) {
        return value == want;
    }
    if index.tombstones.contains(id) {
        return false;
    }
    index
        .segment
        .as_ref()
        .filter(|segment| id < segment.n_docs())
        .and_then(|segment| segment.keyword_at_cow(id))
        .is_some_and(|value| value.as_ref() == want)
}

fn set_equal(index: &SetIndex, id: u32, want: FastStringList<'_>) -> bool {
    if let Some(live) = index.forward.get(&id) {
        return equal_source_to_iter(want, live.iter().map(String::as_str));
    }
    if index.tombstones.contains(id) {
        return false;
    }
    let Some(segment) = index
        .segment
        .as_ref()
        .filter(|segment| id < segment.n_docs())
    else {
        return false;
    };
    let Some((present, count)) = segment.set_row_member_count(id) else {
        return false;
    };
    if !present {
        return false;
    }
    let mut member = 0u32;
    let mut prior = None;
    loop {
        let next = next_source_member(want, prior);
        let stored = if member < count {
            let value = segment.set_member_at_cow(id, member);
            member += 1;
            value
        } else {
            None
        };
        match (next, stored) {
            (None, None) => return true,
            (Some(source), Some(stored)) if source == stored.as_ref() => prior = Some(source),
            _ => return false,
        }
    }
}

fn equal_source_to_iter<'source, 'live>(
    want: FastStringList<'source>,
    mut stored: impl Iterator<Item = &'live str>,
) -> bool {
    let mut prior = None;
    loop {
        let source = next_source_member(want, prior);
        let live = stored.next();
        match (source, live) {
            (None, None) => return true,
            (Some(source), Some(live)) if source == live => prior = Some(source),
            _ => return false,
        }
    }
}

/// `apply_value` makes set members unique. Repeated lexical-min scans are
/// fixed-memory and compare that logical set without creating a `BTreeSet`.
fn next_source_member<'a>(values: FastStringList<'a>, prior: Option<&'a str>) -> Option<&'a str> {
    values
        .values()
        .filter(|value| prior.is_none_or(|old| *value > old))
        .min()
}

#[cfg(test)]
#[path = "committed_replace_view_tests.rs"]
mod tests;
