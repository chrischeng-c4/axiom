//! Plan one committed `docs:replace` command without retaining its values.
//!
//! The caller writes a private fast-Index wire image whose flattened fields
//! match `ReplaceDocDescriptor` ranges.  This module only owns bounded
//! identifiers, action metadata, and the final replacement metadata ledger.
//! It never decodes a `String`, vector, or set into an owned request value.

use super::committed_index_plan::{
    PlanStamp, PlanView, PlannedCell, RequestOutcome, ScalarAction, ScalarPlan,
};
use crate::storage::{SortableF64, StorageError};
use crate::types::{FieldType, MAX_BATCH_REPLACE_SIZE};
use crate::wal::fast_index_scanner::{FastIndexScanner, FastIndexValue};
use anyhow::{bail, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

/// One document in request order. `fields_start..fields_end` names a sorted,
/// unique range in the flattened source scanner. Empty documents use an empty
/// range. The adapter must prove the range fields belong to `external_id`.
pub(super) use crate::wal::borrowed_replace_spool::BorrowedReplaceSpoolDoc as ReplaceDocDescriptor;

/// The read-only replacement facts. All methods must remain allocation-free.
/// `existing_unchanged` compares a source ordinal with an existing live cell.
/// `old_fields` is sorted by the adapter before it reaches this planner.
pub(super) trait ReplacePlanView: PlanView {
    fn doc_version(&self, id: u32) -> Option<u64>;
    fn old_fields(&self, id: u32) -> Option<&[String]>;
    /// Exact allocation-free price for cloning this coverage snapshot. The
    /// planner asks before it allocates any ID, action, or descriptor map.
    fn old_fields_bound(&self, id: u32) -> OldFieldsBound;
    fn existing_unchanged(
        &self,
        id: u32,
        field: &str,
        kind: FieldType,
        ordinal: usize,
        checksum: Option<u64>,
    ) -> bool;
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct OldFieldsBound {
    pub(super) count: usize,
    pub(super) copied_bytes: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ReplaceItemOutcome {
    Ok {
        fields_written: u32,
        fields_skipped: u32,
    },
    Dropped {
        current_version: u64,
    },
    Error {
        error: ReplaceItemError,
    },
}

/// Retain only bounded error metadata. Hash parsing is rendered from its
/// source ordinal after the state read lock has been released.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ReplaceItemError {
    UnknownField {
        field: String,
    },
    TypeMismatch {
        field: String,
        expected: FieldType,
        got: &'static str,
    },
    InvalidNumber(String),
    InvalidVectorDimension {
        field: String,
        expected: u32,
        got: usize,
    },
    InvalidHash {
        ordinal: usize,
    },
}

impl ReplaceItemError {
    pub(super) fn render(
        self,
        collection: &str,
        scanner: &FastIndexScanner<'_>,
    ) -> RenderedReplaceItemOutcome {
        let error: anyhow::Error = match self {
            Self::UnknownField { field } => StorageError::UnknownField {
                collection: collection.to_owned(),
                field,
            }
            .into(),
            Self::TypeMismatch {
                field,
                expected,
                got,
            } => StorageError::TypeMismatch {
                field,
                expected,
                got,
            }
            .into(),
            Self::InvalidNumber(message) => StorageError::InvalidNumber(message).into(),
            Self::InvalidVectorDimension {
                field,
                expected,
                got,
            } => anyhow::anyhow!(
                "vector field `{field}` declared dim={expected} but got vector of length {got}"
            ),
            Self::InvalidHash { ordinal } => {
                let item = scanner
                    .items()
                    .nth(ordinal)
                    .expect("validated replacement hash ordinal");
                let FastIndexValue::String(value) = item.value else {
                    unreachable!("validated replacement Hash source")
                };
                super::parse_hash(value).expect_err("same immutable replacement Hash source")
            }
        };
        let code = if matches!(
            error.downcast_ref::<StorageError>(),
            Some(StorageError::UnknownField { .. })
        ) {
            "unknown_field"
        } else {
            "type_mismatch"
        };
        RenderedReplaceItemOutcome::Error {
            code,
            message: error.to_string(),
        }
    }
}

/// Convert `ReplaceItemOutcome::Error` through `ReplaceItemError::render`
/// only after planning releases its state-read adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum RenderedReplaceItemOutcome {
    Ok {
        fields_written: u32,
        fields_skipped: u32,
    },
    Dropped {
        current_version: u64,
    },
    Error {
        code: &'static str,
        message: String,
    },
}

/// Final document metadata. The common adapter applies actions in source
/// order, then installs this ledger while it holds the same apply lease.
#[derive(Clone, Debug)]
pub(super) struct ReplaceDocFinal {
    pub(super) id: u32,
    pub(super) fields: Vec<String>,
    pub(super) version: Option<u64>,
}

/// Text/Vector checksums are replacement-path state. `None` removes a stale
/// checksum after an omitted field or an empty replacement.
#[derive(Clone, Debug)]
pub(super) struct ReplaceChecksumFinal {
    pub(super) cell: PlannedCell,
    pub(super) checksum: Option<u64>,
}

#[derive(Debug)]
pub(super) struct ReplacePlan {
    pub(super) scalar: ScalarPlan,
    pub(super) outcomes: Vec<ReplaceItemOutcome>,
    pub(super) final_docs: Vec<ReplaceDocFinal>,
    pub(super) final_checksums: Vec<ReplaceChecksumFinal>,
    pub(super) fields_written: u64,
    pub(super) fields_skipped: u64,
    pub(super) reservation: usize,
}

/// Replacement has a document limit, not the generic Index field-item limit.
/// This bound covers all copied names and every metadata map/vector before
/// this module allocates. Source value bytes remain in the scanner wire image.
pub(super) fn metadata_bound(
    scanner: &FastIndexScanner<'_>,
    docs: &[ReplaceDocDescriptor<'_>],
    view: &impl ReplacePlanView,
) -> Result<usize> {
    if docs.len() > MAX_BATCH_REPLACE_SIZE {
        return Err(StorageError::BulkLimit {
            got: docs.len(),
            max: MAX_BATCH_REPLACE_SIZE,
        }
        .into());
    }
    let fields = scanner.cost().item_count;
    validate_descriptors(scanner, docs)?;
    let mut copied = 0usize;
    let mut old = OldFieldsBound::default();
    for doc in docs {
        copied = copied
            .checked_add(
                doc.external_id
                    .len()
                    .checked_mul(2)
                    .ok_or_else(|| anyhow::anyhow!("replace id bound overflow"))?,
            )
            .ok_or_else(|| anyhow::anyhow!("replace metadata overflow"))?;
        let id = view.id(doc.external_id);
        if let Some(id) = id {
            let bound = view.old_fields_bound(id);
            old.count = old
                .count
                .checked_add(bound.count)
                .ok_or_else(|| anyhow::anyhow!("replace old coverage count overflow"))?;
            old.copied_bytes = old
                .copied_bytes
                .checked_add(bound.copied_bytes)
                .ok_or_else(|| anyhow::anyhow!("replace old coverage bytes overflow"))?;
        }
    }
    for item in scanner.items() {
        copied = copied
            .checked_add(
                item.field
                    .len()
                    .checked_mul(6)
                    .ok_or_else(|| anyhow::anyhow!("replace field bound overflow"))?,
            )
            .ok_or_else(|| anyhow::anyhow!("replace metadata overflow"))?;
    }
    // Source fields can coexist in item/validated/supplied/action/winner/
    // checksum/final-doc ledgers. Old coverage can coexist with its omitted
    // action/checksum rows and an adapter snapshot, so price it separately.
    const NODE: usize = 256;
    let per_field = std::mem::size_of::<ScalarAction>()
        .checked_add(std::mem::size_of::<PlannedCell>() * 5)
        .and_then(|n| n.checked_add(NODE * 16))
        .and_then(|n| n.checked_add(128))
        .ok_or_else(|| anyhow::anyhow!("replace field metadata overflow"))?;
    copied
        .checked_add(
            old.copied_bytes
                .checked_mul(12)
                .ok_or_else(|| anyhow::anyhow!("replace old copy overflow"))?,
        )
        .and_then(|n| n.checked_add(fields.checked_mul(per_field)?))
        .and_then(|n| {
            n.checked_add(old.count.checked_mul(
                std::mem::size_of::<ScalarAction>()
                    + std::mem::size_of::<PlannedCell>() * 5
                    + NODE * 14,
            )?)
        })
        .and_then(|n| n.checked_add(docs.len().checked_mul(NODE * 6 + 256)?))
        .and_then(|n| n.checked_add(4096))
        .ok_or_else(|| anyhow::anyhow!("replace metadata overflow"))
}

/// The private spool may be malformed even though its original CBOR scan was
/// valid. Refuse overlapping, gapped, reordered, or non-canonical descriptor
/// rows before admission and before stale items can skip this proof.
fn validate_descriptors(
    scanner: &FastIndexScanner<'_>,
    docs: &[ReplaceDocDescriptor<'_>],
) -> Result<()> {
    let fields = scanner.cost().item_count;
    let mut expected = 0usize;
    for doc in docs {
        if doc.fields_start != expected
            || doc.fields_start > doc.fields_end
            || doc.fields_end > fields
        {
            bail!("invalid non-canonical replacement descriptor range");
        }
        let mut prior = None;
        for (_, item) in scanner
            .items()
            .enumerate()
            .skip(doc.fields_start)
            .take(doc.fields_end - doc.fields_start)
        {
            if item.external_id != doc.external_id {
                bail!("replacement descriptor source id mismatch");
            }
            if prior.is_some_and(|old: &str| old >= item.field) {
                bail!("replacement descriptor fields are not sorted and unique");
            }
            prior = Some(item.field);
        }
        expected = doc.fields_end;
    }
    if expected != fields {
        bail!("replacement descriptor does not cover source fields");
    }
    Ok(())
}

/// Precomputed Hash parses and Text/Vector checksums by flattened ordinal.
/// Missing Hash parse means an item error; missing Text/Vector checksum is a
/// programmer error in the root prepass because equality must not hash under
/// a state/apply lease.
pub(super) type ParsedValues = BTreeMap<usize, ParsedValue>;

#[derive(Clone, Copy, Debug)]
pub(super) struct ParsedValue {
    pub(super) hash: Option<u64>,
    pub(super) checksum: Option<u64>,
}

pub(super) fn plan<V: ReplacePlanView>(
    scanner: &FastIndexScanner<'_>,
    docs: &[ReplaceDocDescriptor<'_>],
    parsed: &ParsedValues,
    view: &V,
    now: Instant,
    mut reserve_total: impl FnMut(usize) -> Result<()>,
) -> Result<ReplacePlan> {
    let reservation = metadata_bound(scanner, docs, view)?;
    reserve_total(reservation)?;
    if !view.is_live() {
        bail!(StorageError::Gone(scanner.collection_id().to_owned()));
    }
    let stamp = PlanStamp {
        engine_epoch: view.engine_epoch(),
        collection_generation: view.collection_generation(),
        schema_version: view.schema_version(),
        data_version: view.data_version(),
        revision: view.revision(),
        interner_watermark: view.interner_len(),
    };
    let mut ids = BTreeMap::<String, u32>::new();
    let mut new_external_ids = Vec::new();
    // Preserve owned replace behavior: intern all document IDs before any
    // stale or schema check. This is also why there is no request-id outcome.
    for doc in docs {
        planned_id(
            view,
            &mut ids,
            &mut new_external_ids,
            stamp.interner_watermark,
            doc.external_id,
        )?;
    }

    let mut actions = Vec::new();
    let mut winners = BTreeMap::new();
    let mut bytes_by_field = BTreeMap::<String, u64>::new();
    let mut outcomes = Vec::with_capacity(docs.len());
    let mut final_docs = BTreeMap::<u32, ReplaceDocFinal>::new();
    let mut final_checksums = BTreeMap::<PlannedCell, Option<u64>>::new();
    let mut virtual_versions = BTreeMap::<u32, Option<u64>>::new();
    let mut source_winners = BTreeMap::<PlannedCell, (usize, FieldType, Option<u64>)>::new();
    let mut written = 0u64;
    let mut skipped = 0u64;

    for doc in docs {
        let id = *ids.get(doc.external_id).expect("all document ids planned");
        let stored = virtual_versions
            .get(&id)
            .copied()
            .flatten()
            .or_else(|| view.doc_version(id));
        if doc
            .version
            .is_some_and(|incoming| stored.is_some_and(|old| old >= incoming))
        {
            outcomes.push(ReplaceItemOutcome::Dropped {
                current_version: stored.expect("stale has stored version"),
            });
            continue;
        }
        let items: Vec<_> = scanner
            .items()
            .enumerate()
            .skip(doc.fields_start)
            .take(doc.fields_end - doc.fields_start)
            .collect();
        // The adapter's descriptor ranges must cover exactly this document.
        if items
            .iter()
            .any(|(_, item)| item.external_id != doc.external_id)
        {
            bail!("replace descriptor does not match flattened source");
        }
        let mut supplied = BTreeSet::new();
        let mut validated = Vec::with_capacity(items.len());
        let mut error: Option<ReplaceItemError> = None;
        for (ordinal, item) in &items {
            if !supplied.insert(item.field.to_owned()) {
                bail!(
                    "replacement descriptor has duplicate sorted field `{}`",
                    item.field
                );
            }
            let Some(kind) = view.field_type(item.field) else {
                error = Some(ReplaceItemError::UnknownField {
                    field: item.field.to_owned(),
                });
                break;
            };
            match validate_borrowed(
                *ordinal,
                kind,
                &item.value,
                item.field,
                doc.external_id.len(),
                view.vector_dimension(item.field),
                parsed.get(ordinal).copied(),
            ) {
                Ok((bytes, checksum)) => {
                    validated.push((*ordinal, item.field, kind, bytes, checksum))
                }
                Err(ValidationError::Domain(domain)) => {
                    error = Some(domain);
                    break;
                }
                Err(ValidationError::Preparation(message)) => {
                    bail!("replacement preparation required: {message}")
                }
            }
        }
        if let Some(error) = error {
            outcomes.push(ReplaceItemOutcome::Error { error });
            continue;
        }
        let old: BTreeSet<String> = final_docs
            .get(&id)
            .map(|d| d.fields.iter().cloned().collect())
            .or_else(|| view.old_fields(id).map(|f| f.iter().cloned().collect()))
            .unwrap_or_default();
        // Omitted scalar actions carry `usize::MAX`: the common Vector drop
        // uses `cell.id` for its external id and must never index this sentinel.
        for field in old.difference(&supplied) {
            let kind = view
                .field_type(field)
                .expect("old coverage field has schema");
            let cell = PlannedCell {
                id,
                field: field.clone(),
            };
            actions.push(ScalarAction::Drop {
                cell: cell.clone(),
                ordinal: usize::MAX,
                kind,
            });
            winners.remove(&cell);
            source_winners.remove(&cell);
            final_checksums.insert(cell, None);
        }
        let mut item_written = 0u32;
        let mut item_skipped = 0u32;
        for (ordinal, field, kind, bytes, checksum) in validated {
            let cell = PlannedCell {
                id,
                field: field.to_owned(),
            };
            let unchanged = match source_winners.get(&cell) {
                Some((prior, prior_kind, prior_checksum)) => {
                    *prior_kind == kind
                        && same_source(
                            scanner,
                            *prior,
                            ordinal,
                            kind,
                            *prior_checksum,
                            checksum,
                            parsed,
                        )
                }
                None if old.contains(field) => {
                    view.existing_unchanged(id, field, kind, ordinal, checksum)
                }
                None => false,
            };
            if unchanged {
                item_skipped = item_skipped
                    .checked_add(1)
                    .ok_or_else(|| anyhow::anyhow!("replace item fields_skipped overflow"))?;
                continue;
            }
            if old.contains(field) || source_winners.contains_key(&cell) {
                actions.push(ScalarAction::Drop {
                    cell: cell.clone(),
                    ordinal,
                    kind,
                });
            }
            actions.push(ScalarAction::Apply {
                ordinal,
                cell: cell.clone(),
                kind,
                bytes,
            });
            winners.insert(cell.clone(), ordinal);
            let total = bytes_by_field.entry(field.to_owned()).or_insert(0);
            *total = total
                .checked_add(bytes)
                .ok_or_else(|| anyhow::anyhow!("replace field bytes overflow"))?;
            source_winners.insert(cell.clone(), (ordinal, kind, checksum));
            final_checksums.insert(
                cell,
                if matches!(kind, FieldType::Text | FieldType::Vector) {
                    checksum
                } else {
                    None
                },
            );
            item_written = item_written
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("replace item fields_written overflow"))?;
        }
        if supplied.is_empty() {
            final_checksums.retain(|cell, _| cell.id != id);
        }
        let effective_version = doc.version.or(stored);
        final_docs.insert(
            id,
            ReplaceDocFinal {
                id,
                fields: supplied.into_iter().collect(),
                version: effective_version,
            },
        );
        virtual_versions.insert(id, effective_version);
        written = written
            .checked_add(u64::from(item_written))
            .ok_or_else(|| anyhow::anyhow!("replace fields_written overflow"))?;
        skipped = skipped
            .checked_add(u64::from(item_skipped))
            .ok_or_else(|| anyhow::anyhow!("replace fields_skipped overflow"))?;
        outcomes.push(ReplaceItemOutcome::Ok {
            fields_written: item_written,
            fields_skipped: item_skipped,
        });
    }
    let applied = u32::try_from(
        actions
            .iter()
            .filter(|a| matches!(a, ScalarAction::Apply { .. }))
            .count(),
    )
    .map_err(|_| anyhow::anyhow!("replace apply count exceeds u32"))?;
    Ok(ReplacePlan {
        scalar: ScalarPlan {
            stamp,
            request: RequestOutcome::None,
            new_external_ids,
            actions,
            winners,
            bytes_by_field,
            applied,
            source_items: scanner.cost().item_count,
            business_error: None,
            reservation,
        },
        outcomes,
        final_docs: final_docs.into_values().collect(),
        final_checksums: final_checksums
            .into_iter()
            .map(|(cell, checksum)| ReplaceChecksumFinal { cell, checksum })
            .collect(),
        fields_written: written,
        fields_skipped: skipped,
        reservation,
    })
}

fn planned_id<V: PlanView>(
    view: &V,
    ids: &mut BTreeMap<String, u32>,
    new_ids: &mut Vec<String>,
    watermark: usize,
    eid: &str,
) -> Result<u32> {
    if let Some(id) = ids.get(eid) {
        return Ok(*id);
    }
    let id = match view.id(eid) {
        Some(id) => id,
        None => {
            let id = u32::try_from(
                watermark
                    .checked_add(new_ids.len())
                    .ok_or_else(|| anyhow::anyhow!("interner watermark overflow"))?,
            )
            .map_err(|_| anyhow::anyhow!("interner id exceeds u32"))?;
            new_ids.push(eid.to_owned());
            id
        }
    };
    ids.insert(eid.to_owned(), id);
    Ok(id)
}

enum ValidationError {
    Domain(ReplaceItemError),
    Preparation(&'static str),
}

fn validate_borrowed(
    ordinal: usize,
    kind: FieldType,
    value: &FastIndexValue<'_>,
    field: &str,
    eid_len: usize,
    dimension: Option<u32>,
    parsed: Option<ParsedValue>,
) -> std::result::Result<(u64, Option<u64>), ValidationError> {
    match (kind, value) {
        (FieldType::Text, FastIndexValue::String(_)) => parsed
            .and_then(|p| p.checksum)
            .map(|checksum| (0, Some(checksum)))
            .ok_or(ValidationError::Preparation(
                "missing precomputed Text checksum",
            )),
        (FieldType::Keyword, FastIndexValue::String(value)) => {
            let bytes = value
                .len()
                .checked_add(eid_len)
                .ok_or(ValidationError::Preparation("Keyword byte count overflow"))?;
            Ok((
                u64::try_from(bytes)
                    .map_err(|_| ValidationError::Preparation("Keyword byte count exceeds u64"))?,
                None,
            ))
        }
        (FieldType::Number, FastIndexValue::Number(value)) => SortableF64::new(*value)
            .map_err(|e| ValidationError::Domain(ReplaceItemError::InvalidNumber(e.to_string())))
            .and_then(|_| {
                u64::try_from(eid_len)
                    .map_err(|_| ValidationError::Preparation("Number id length exceeds u64"))
            })
            .and_then(|eid| {
                eid.checked_add(8)
                    .ok_or(ValidationError::Preparation("Number byte count overflow"))
            })
            .map(|bytes| (bytes, None)),
        (FieldType::Set, FastIndexValue::StringList(values)) => {
            Ok((set_bytes(*values, eid_len)?, None))
        }
        (FieldType::Hash, FastIndexValue::String(_)) if parsed.and_then(|p| p.hash).is_some() => {
            Ok((12, None))
        }
        (FieldType::Hash, FastIndexValue::String(_)) => {
            Err(ValidationError::Domain(ReplaceItemError::InvalidHash {
                ordinal,
            }))
        }
        (FieldType::Vector, FastIndexValue::Vector { len, .. })
            if u32::try_from(*len).ok() == dimension =>
        {
            let bytes =
                u64::try_from(*len)
                    .map_err(|_| ValidationError::Preparation("Vector length exceeds u64"))?
                    .checked_mul(std::mem::size_of::<f32>() as u64)
                    .ok_or(ValidationError::Preparation("Vector byte count overflow"))?
                    .checked_add(u64::try_from(eid_len).map_err(|_| {
                        ValidationError::Preparation("Vector id length exceeds u64")
                    })?)
                    .ok_or(ValidationError::Preparation("Vector byte count overflow"))?;
            parsed
                .and_then(|p| p.checksum)
                .map(|checksum| (bytes, Some(checksum)))
                .ok_or(ValidationError::Preparation(
                    "missing precomputed Vector checksum",
                ))
        }
        (FieldType::Vector, FastIndexValue::Vector { len, .. }) => Err(ValidationError::Domain(
            ReplaceItemError::InvalidVectorDimension {
                field: field.to_owned(),
                expected: dimension.unwrap_or(0),
                got: *len,
            },
        )),
        (FieldType::Set, FastIndexValue::String(_)) => {
            Err(ValidationError::Domain(ReplaceItemError::TypeMismatch {
                field: field.to_owned(),
                expected: FieldType::Set,
                got: "string (expected array of strings)",
            }))
        }
        (expected, got) => Err(ValidationError::Domain(ReplaceItemError::TypeMismatch {
            field: field.to_owned(),
            expected,
            got: fast_kind(got),
        })),
    }
}

fn same_source(
    scanner: &FastIndexScanner<'_>,
    left: usize,
    right: usize,
    kind: FieldType,
    left_checksum: Option<u64>,
    right_checksum: Option<u64>,
    parsed: &ParsedValues,
) -> bool {
    if matches!(kind, FieldType::Text | FieldType::Vector) {
        return left_checksum.is_some() && left_checksum == right_checksum;
    }
    let a = scanner.items().nth(left).expect("planned ordinal");
    let b = scanner.items().nth(right).expect("planned ordinal");
    match (kind, a.value, b.value) {
        (FieldType::Keyword, FastIndexValue::String(a), FastIndexValue::String(b)) => a == b,
        (FieldType::Hash, FastIndexValue::String(_), FastIndexValue::String(_)) => {
            parsed.get(&left).and_then(|p| p.hash).is_some()
                && parsed.get(&left).and_then(|p| p.hash) == parsed.get(&right).and_then(|p| p.hash)
        }
        (FieldType::Number, FastIndexValue::Number(a), FastIndexValue::Number(b)) => {
            SortableF64::new(a).ok() == SortableF64::new(b).ok()
        }
        (FieldType::Set, FastIndexValue::StringList(a), FastIndexValue::StringList(b)) => {
            same_set(a, b)
        }
        _ => false,
    }
}

/// Match `apply_value`: duplicate set members have one logical live value.
/// Repeated lexical-min scans avoid materializing a giant source array.
fn set_bytes(
    values: crate::wal::fast_index_scanner::FastStringList<'_>,
    eid_len: usize,
) -> std::result::Result<u64, ValidationError> {
    let mut total = 0u64;
    let mut previous = None;
    while let Some(next) = next_distinct(values, previous) {
        let bytes = next
            .len()
            .checked_add(eid_len)
            .ok_or(ValidationError::Preparation(
                "Set member byte count overflow",
            ))?;
        total =
            total
                .checked_add(u64::try_from(bytes).map_err(|_| {
                    ValidationError::Preparation("Set member byte count exceeds u64")
                })?)
                .ok_or(ValidationError::Preparation("Set byte count overflow"))?;
        previous = Some(next);
    }
    Ok(total)
}

fn same_set(
    a: crate::wal::fast_index_scanner::FastStringList<'_>,
    b: crate::wal::fast_index_scanner::FastStringList<'_>,
) -> bool {
    let mut left = None;
    let mut right = None;
    loop {
        let next_left = next_distinct(a, left);
        let next_right = next_distinct(b, right);
        if next_left != next_right {
            return false;
        }
        let Some(value) = next_left else {
            return true;
        };
        left = Some(value);
        right = next_right;
    }
}

fn next_distinct<'a>(
    values: crate::wal::fast_index_scanner::FastStringList<'a>,
    previous: Option<&'a str>,
) -> Option<&'a str> {
    values
        .values()
        .filter(|value| previous.is_none_or(|old| *value > old))
        .min()
}

fn fast_kind(value: &FastIndexValue<'_>) -> &'static str {
    match value {
        FastIndexValue::String(_) => "string",
        FastIndexValue::Number(_) => "number",
        FastIndexValue::Vector { .. } => "f32[]",
        FastIndexValue::StringList(_) => "string[]",
    }
}

#[cfg(test)]
#[path = "committed_replace_plan_tests.rs"]
mod tests;
