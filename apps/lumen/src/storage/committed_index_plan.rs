//! Plan the supported borrowed fast-Index fields for one retained command.
//!
//! The plan keeps Keyword, Number, Set, Text, Hash, and Vector in one action ledger.
//! Text bytes are filled after its borrowed rows are staged outside state and
//! apply locks. `storage.rs` supplies the read-only `PlanView` adapter while
//! holding its state read lock; attachment remains in the committed adapter.
//!
//! A plan owns identifiers and small metadata only.  Every `ordinal` points
//! back into `FastIndexScanner`, so no field value is decoded or copied.

use crate::storage::{SortableF64, StorageError, MAX_INDEX_ITEMS};
use crate::types::FieldType;
use crate::wal::fast_index_scanner::{FastIndexScanner, FastIndexValue};
use anyhow::{bail, ensure, Result};
use std::collections::BTreeMap;
use std::time::Instant;

/// State read by the planner.  The storage adapter must not allocate or mutate
/// while implementing this trait.  `revision` comes from the capture barrier,
/// not from a new `Collection` field.
pub(super) trait PlanView {
    fn engine_epoch(&self) -> u64;
    fn collection_generation(&self) -> u64;
    fn schema_version(&self) -> u32;
    fn data_version(&self) -> u64;
    fn revision(&self) -> u64;
    fn interner_len(&self) -> usize;
    fn is_live(&self) -> bool;
    fn field_type(&self, field: &str) -> Option<FieldType>;
    /// Vector fields expose their declared dimension without lending values.
    fn vector_dimension(&self, field: &str) -> Option<u32>;
    fn id(&self, external_id: &str) -> Option<u32>;
    fn has_cell(&self, id: u32, field: &str) -> bool;
    fn cell_version(&self, id: u32, field: &str) -> Option<u64>;
    /// `Some(deadline)` means this request is currently deduplicated.  The
    /// adapter scans only; it treats an expired retained entry as absent.  It
    /// must not call the mutating `gc_requests` from planner or `matches`.
    fn request_deadline(&self, request_id: &str) -> Option<Instant>;
}

/// `revision` is a capture-barrier apply-generation counter.  It increments
/// once when an outer apply lease releases, after all its state changes are
/// visible.  The initial snapshot reads it while holding the state read lock;
/// attachment acquires an apply lease, reads it with the state write lock, and
/// calls `matches` before it publishes.  This covers request-id-only applies
/// without adding a counter update at every collection mutation site.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PlanStamp {
    pub(super) engine_epoch: u64,
    pub(super) collection_generation: u64,
    pub(super) schema_version: u32,
    pub(super) data_version: u64,
    pub(super) revision: u64,
    pub(super) interner_watermark: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct PlannedCell {
    pub(super) id: u32,
    pub(super) field: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum RequestOutcome {
    None,
    /// Attach must insert this key before it applies items, as index_collection
    /// does today.  The actual insertion uses attach-time `Instant::now()`.
    Register(String),
    /// A duplicate result is valid only until this precise expiry deadline.
    Duplicate {
        request_id: String,
        deadline: Instant,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum BusinessError {
    UnknownField {
        field: String,
    },
    TypeMismatch {
        field: String,
        expected: FieldType,
        got: &'static str,
    },
    InvalidNumber(String),
    /// Keep the owned apply path's vector dimension wording without decoding
    /// the retained vector into a `Vec<f32>`.
    InvalidVectorDimension {
        field: String,
        expected: u32,
        got: usize,
    },
    /// The scalar parse failed outside state/apply. Defer the existing error
    /// response formatting until the planner has released its state lock.
    InvalidHash {
        ordinal: usize,
    },
}

impl BusinessError {
    pub(super) fn into_error(
        self,
        collection: &str,
        scanner: &FastIndexScanner<'_>,
    ) -> anyhow::Error {
        (match self {
            Self::UnknownField { field } => StorageError::UnknownField {
                collection: collection.to_owned(),
                field,
            },
            Self::TypeMismatch {
                field,
                expected,
                got,
            } => StorageError::TypeMismatch {
                field,
                expected,
                got,
            },
            Self::InvalidNumber(message) => StorageError::InvalidNumber(message),
            Self::InvalidVectorDimension {
                field,
                expected,
                got,
            } => {
                return anyhow::anyhow!(
                    "vector field `{field}` declared dim={expected} but got vector of length {got}"
                );
            }
            Self::InvalidHash { ordinal } => {
                let item = scanner
                    .items()
                    .nth(ordinal)
                    .expect("validated Hash ordinal");
                let FastIndexValue::String(value) = item.value else {
                    unreachable!("invalid Hash parse has a String source")
                };
                return super::parse_hash(value).expect_err("same immutable Hash source");
            }
        })
        .into()
    }
}

/// `Drop` is retained even when the next action returns an error.  The attach
/// path journals it before reconstructing `business_error`, matching the live
/// `drop_eid`-then-`apply_value` order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ScalarAction {
    Apply {
        ordinal: usize,
        cell: PlannedCell,
        kind: FieldType,
        bytes: u64,
    },
    Drop {
        cell: PlannedCell,
        ordinal: usize,
        kind: FieldType,
    },
}

#[derive(Debug)]
pub(super) struct ScalarPlan {
    pub(super) stamp: PlanStamp,
    pub(super) request: RequestOutcome,
    /// New IDs are allocated at `stamp.interner_watermark + offset`, in this
    /// exact order.  The vector has at most one entry per source item.
    pub(super) new_external_ids: Vec<String>,
    pub(super) actions: Vec<ScalarAction>,
    /// Final supported-field winner per cell. Scalar winners are passed to
    /// `committed_scalar_files::prepare`; Text and Vector winners select
    /// staged rows. Earlier valid repeated writes remain in `actions` so
    /// metrics and indexed counts retain current behavior.
    pub(super) winners: BTreeMap<PlannedCell, usize>,
    pub(super) bytes_by_field: BTreeMap<String, u64>,
    pub(super) applied: u32,
    pub(super) source_items: usize,
    pub(super) business_error: Option<BusinessError>,
    pub(super) reservation: usize,
}

#[derive(Debug)]
pub(super) enum PlanResult {
    Planned(ScalarPlan),
    /// A schema change exposed a Hash field after the caller's parse snapshot.
    /// The caller retries and parses it outside state and apply locks.
    NeedsHash,
    /// Internal routing signal only.  It must fall back to the owned current
    /// path, never become a new HTTP rejection.
    Unsupported {
        ordinal: usize,
        field: String,
        kind: FieldType,
    },
}

impl ScalarPlan {
    pub(super) fn has_text(&self) -> bool {
        self.actions.iter().any(|action| {
            matches!(
                action,
                ScalarAction::Apply {
                    kind: FieldType::Text,
                    ..
                } | ScalarAction::Drop {
                    kind: FieldType::Text,
                    ..
                }
            )
        })
    }

    pub(super) fn text_actions(&self) -> impl Iterator<Item = (&PlannedCell, usize)> {
        self.actions.iter().filter_map(|action| match action {
            ScalarAction::Apply {
                cell,
                ordinal,
                kind: FieldType::Text,
                ..
            } => Some((cell, *ordinal)),
            _ => None,
        })
    }

    /// Text staging happens after planning, so add its exact indexed bytes to
    /// the original action rather than constructing a second response ledger.
    pub(super) fn set_text_bytes(&mut self, ordinal: usize, bytes: u64) -> Result<()> {
        let action = self
            .actions
            .iter_mut()
            .find(|action| matches!(action, ScalarAction::Apply { ordinal: current, kind: FieldType::Text, .. } if *current == ordinal))
            .ok_or_else(|| anyhow::anyhow!("missing planned Text action"))?;
        let ScalarAction::Apply {
            cell,
            bytes: stored,
            ..
        } = action
        else {
            unreachable!("matched Text action")
        };
        *stored = bytes;
        let total = self.bytes_by_field.entry(cell.field.clone()).or_insert(0);
        *total = total
            .checked_add(bytes)
            .ok_or_else(|| anyhow::anyhow!("Text indexed bytes overflow"))?;
        Ok(())
    }

    /// This is a read-only recheck.  Attachment calls it again after acquiring
    /// `capture_barrier.apply()` and the state write lock.
    pub(super) fn matches<V: PlanView>(&self, view: &V, now: Instant) -> bool {
        let s = self.stamp;
        if !view.is_live()
            || view.engine_epoch() != s.engine_epoch
            || view.collection_generation() != s.collection_generation
            || view.schema_version() != s.schema_version
            || view.data_version() != s.data_version
            || view.revision() != s.revision
            || view.interner_len() != s.interner_watermark
        {
            return false;
        }
        match &self.request {
            RequestOutcome::None => true,
            RequestOutcome::Register(id) => view.request_deadline(id).is_none(),
            RequestOutcome::Duplicate {
                request_id,
                deadline,
            } => now <= *deadline && view.request_deadline(request_id) == Some(*deadline),
        }
    }
}

/// Reserve before the first `String`, `Vec`, or `BTreeMap` allocation in this
/// module.  Values remain borrowed, so their wire spans are deliberately not
/// part of this budget.  This prices every copied identifier, copied field,
/// request key, action, and bounded map node, even if it later becomes stale.
/// Allocation-free upper bound.  Engine admission calls this before it takes a
/// state lock, then lets `plan` use only a no-wait assertion callback.
pub(super) fn metadata_bound(scanner: &FastIndexScanner<'_>) -> Result<usize> {
    let c = scanner.cost();
    ensure!(
        c.item_count <= MAX_INDEX_ITEMS,
        "committed scalar plan exceeds Index item limit"
    );
    // BTreeMap has no reserve API.  Price five worst-case node populations
    // (IDs, cells, versions, winners, field bytes) at a deliberately larger
    // per-node bound, plus the action Vec and copied string capacities.
    const BTREE_NODE_BOUND: usize = 256;
    const MAP_NODE_POPULATIONS: usize = 5;
    let mut copied = scanner.request_id().map_or(0, str::len);
    for item in scanner.items() {
        copied = copied
            .checked_add(
                item.external_id
                    .len()
                    .checked_mul(2)
                    .ok_or_else(|| anyhow::anyhow!("external id reservation overflow"))?,
            )
            .and_then(|n| n.checked_add(item.field.len().checked_mul(4)?))
            .ok_or_else(|| anyhow::anyhow!("identifier metadata reservation overflow"))?;
    }
    let per_item = std::mem::size_of::<ScalarAction>()
        .checked_add(
            std::mem::size_of::<PlannedCell>()
                .checked_mul(3)
                .ok_or_else(|| anyhow::anyhow!("cell metadata overflow"))?,
        )
        .and_then(|n| n.checked_add(BTREE_NODE_BOUND.checked_mul(MAP_NODE_POPULATIONS)?))
        .and_then(|n| n.checked_add(128))
        .ok_or_else(|| anyhow::anyhow!("planner metadata reservation overflow"))?;
    copied
        .checked_add(
            c.item_count
                .checked_mul(per_item)
                .ok_or_else(|| anyhow::anyhow!("planner item reservation overflow"))?,
        )
        .and_then(|n| n.checked_add(4096))
        .ok_or_else(|| anyhow::anyhow!("planner reservation overflow"))
}

/// `None` is an invalid numeric value. Missing keys have not been parsed yet.
pub(super) type ParsedHashes = BTreeMap<usize, Option<u64>>;

#[cfg(test)]
pub(super) fn plan<V: PlanView>(
    scanner: &FastIndexScanner<'_>,
    view: &V,
    now: Instant,
    mut reserve_total: impl FnMut(usize) -> Result<()>,
) -> Result<PlanResult> {
    reserve_total(metadata_bound(scanner)?)?;
    let hashes = scanner
        .items()
        .enumerate()
        .filter_map(|(ordinal, item)| {
            if view.field_type(item.field) != Some(FieldType::Hash) {
                return None;
            }
            let FastIndexValue::String(value) = item.value else {
                return None;
            };
            Some((ordinal, super::parse_hash_number(value).ok()))
        })
        .collect();
    plan_with_hashes(scanner, view, &hashes, now, reserve_total)
}

pub(super) fn plan_with_hashes<V: PlanView>(
    scanner: &FastIndexScanner<'_>,
    view: &V,
    hashes: &ParsedHashes,
    now: Instant,
    mut reserve_total: impl FnMut(usize) -> Result<()>,
) -> Result<PlanResult> {
    let reservation = metadata_bound(scanner)?;
    reserve_total(reservation)?;
    if !view.is_live() {
        bail!(StorageError::Gone(
            "collection disappeared during planning".into()
        ));
    }
    let request = match scanner.request_id() {
        Some(id) => match view.request_deadline(id) {
            Some(deadline) if now <= deadline => RequestOutcome::Duplicate {
                request_id: id.to_owned(),
                deadline,
            },
            _ => RequestOutcome::Register(id.to_owned()),
        },
        None => RequestOutcome::None,
    };
    let stamp = PlanStamp {
        engine_epoch: view.engine_epoch(),
        collection_generation: view.collection_generation(),
        schema_version: view.schema_version(),
        data_version: view.data_version(),
        revision: view.revision(),
        interner_watermark: view.interner_len(),
    };
    if matches!(request, RequestOutcome::Duplicate { .. }) {
        return Ok(PlanResult::Planned(ScalarPlan {
            stamp,
            request,
            new_external_ids: vec![],
            actions: vec![],
            winners: BTreeMap::new(),
            bytes_by_field: BTreeMap::new(),
            applied: 0,
            source_items: scanner.cost().item_count,
            business_error: None,
            reservation,
        }));
    }

    let mut ids: BTreeMap<String, u32> = BTreeMap::new();
    let mut new_external_ids = Vec::new();
    let mut cells: BTreeMap<PlannedCell, bool> = BTreeMap::new();
    let mut versions: BTreeMap<PlannedCell, u64> = BTreeMap::new();
    let mut actions = Vec::new();
    let mut winners = BTreeMap::new();
    let mut bytes_by_field = BTreeMap::new();
    let mut applied = 0u32;

    for (ordinal, item) in scanner.items().enumerate() {
        let id = planned_id(
            view,
            &mut ids,
            &mut new_external_ids,
            stamp.interner_watermark,
            item.external_id,
        )?;
        let cell = PlannedCell {
            id,
            field: item.field.to_owned(),
        };
        let stored = versions
            .get(&cell)
            .copied()
            .or_else(|| view.cell_version(id, item.field));
        if item
            .version
            .is_some_and(|v| stored.is_some_and(|old| old >= v))
        {
            continue;
        }
        let Some(kind) = view.field_type(item.field) else {
            return Ok(PlanResult::Planned(finish(
                stamp,
                request,
                new_external_ids,
                actions,
                winners,
                bytes_by_field,
                applied,
                scanner.cost().item_count,
                reservation,
                Some(BusinessError::UnknownField {
                    field: item.field.to_owned(),
                }),
            )));
        };
        if !matches!(
            kind,
            FieldType::Keyword
                | FieldType::Number
                | FieldType::Set
                | FieldType::Text
                | FieldType::Hash
                | FieldType::Vector
        ) {
            return Ok(PlanResult::Unsupported {
                ordinal,
                field: item.field.to_owned(),
                kind,
            });
        }
        let cost = if kind == FieldType::Hash && matches!(item.value, FastIndexValue::String(_)) {
            let Some(value) = hashes.get(&ordinal) else {
                return Ok(PlanResult::NeedsHash);
            };
            value
                .map(|_| 12)
                .ok_or(BusinessError::InvalidHash { ordinal })
        } else {
            borrowed_apply_cost(
                kind,
                &item.value,
                item.external_id,
                item.field,
                (kind == FieldType::Vector).then(|| {
                    view.vector_dimension(item.field)
                        .expect("vector field exposes a declared dimension")
                }),
            )
        };
        match cost {
            Ok(bytes) => {
                cells.insert(cell.clone(), true);
                if let Some(v) = item.version {
                    versions.insert(cell.clone(), v);
                }
                winners.insert(cell.clone(), ordinal);
                *bytes_by_field.entry(cell.field.clone()).or_insert(0) += bytes;
                actions.push(ScalarAction::Apply {
                    ordinal,
                    cell,
                    kind,
                    bytes,
                });
                applied = applied.checked_add(1).expect("Index item count is bounded");
            }
            Err(error) => {
                // The live path marks the dirty cell even if no old cell exists.
                // `Drop` is therefore unconditional; attach performs its current
                // no-op drop when `cells` says it was absent, then journals it.
                let existed = cells
                    .get(&cell)
                    .copied()
                    .unwrap_or_else(|| view.has_cell(id, item.field));
                if existed {
                    cells.insert(cell.clone(), false);
                }
                winners.remove(&cell);
                actions.push(ScalarAction::Drop {
                    cell,
                    ordinal,
                    kind,
                });
                return Ok(PlanResult::Planned(finish(
                    stamp,
                    request,
                    new_external_ids,
                    actions,
                    winners,
                    bytes_by_field,
                    applied,
                    scanner.cost().item_count,
                    reservation,
                    Some(error),
                )));
            }
        }
    }
    Ok(PlanResult::Planned(finish(
        stamp,
        request,
        new_external_ids,
        actions,
        winners,
        bytes_by_field,
        applied,
        scanner.cost().item_count,
        reservation,
        None,
    )))
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
            let next = watermark
                .checked_add(new_ids.len())
                .ok_or_else(|| anyhow::anyhow!("interner watermark overflow"))?;
            let id = u32::try_from(next).map_err(|_| anyhow::anyhow!("interner id exceeds u32"))?;
            new_ids.push(eid.to_owned());
            id
        }
    };
    ids.insert(eid.to_owned(), id);
    Ok(id)
}

fn finish(
    stamp: PlanStamp,
    request: RequestOutcome,
    new_external_ids: Vec<String>,
    actions: Vec<ScalarAction>,
    winners: BTreeMap<PlannedCell, usize>,
    bytes_by_field: BTreeMap<String, u64>,
    applied: u32,
    source_items: usize,
    reservation: usize,
    business_error: Option<BusinessError>,
) -> ScalarPlan {
    ScalarPlan {
        stamp,
        request,
        new_external_ids,
        actions,
        winners,
        bytes_by_field,
        applied,
        source_items,
        business_error,
        reservation,
    }
}

/// Keep this body in lockstep with the scalar arms of `apply_value`.  It uses
/// borrowed terms.  `Set` scans the next lexical distinct member on each pass,
/// as `committed_scalar_files::Selected::set_row` does, so it owns no member
/// set while planning.
fn borrowed_apply_cost(
    kind: FieldType,
    value: &FastIndexValue<'_>,
    eid: &str,
    field: &str,
    vector_dimension: Option<u32>,
) -> std::result::Result<u64, BusinessError> {
    match (kind, value) {
        // Text is staged after the planner has captured its small metadata.
        // Keep the ordered action now; exact row bytes are written into it
        // before attachment.
        (FieldType::Text, FastIndexValue::String(_)) => Ok(0),
        (FieldType::Keyword, FastIndexValue::String(value)) => Ok((value.len() + eid.len()) as u64),
        (FieldType::Number, FastIndexValue::Number(value)) => SortableF64::new(*value)
            .map(|_| (8 + eid.len()) as u64)
            .map_err(|e| BusinessError::InvalidNumber(e.to_string())),
        (FieldType::Set, FastIndexValue::StringList(values)) => set_bytes(*values, eid.len()),
        (FieldType::Vector, FastIndexValue::Vector { len, .. }) => {
            let expected = vector_dimension.expect("vector dimension supplied for vector action");
            if *len != expected as usize {
                return Err(BusinessError::InvalidVectorDimension {
                    field: field.to_owned(),
                    expected,
                    got: *len,
                });
            }
            Ok((expected as u64) * std::mem::size_of::<f32>() as u64 + eid.len() as u64)
        }
        (FieldType::Set, FastIndexValue::String(_)) => Err(BusinessError::TypeMismatch {
            field: field.to_owned(),
            expected: FieldType::Set,
            got: "string (expected array of strings)",
        }),
        (expected, value) => Err(BusinessError::TypeMismatch {
            field: field.to_owned(),
            expected,
            got: fast_kind(value),
        }),
    }
}

fn set_bytes(
    values: crate::wal::fast_index_scanner::FastStringList<'_>,
    eid_len: usize,
) -> std::result::Result<u64, BusinessError> {
    let mut total = 0u64;
    let mut previous = None;
    loop {
        let next = values
            .values()
            .filter(|value| previous.is_none_or(|old| *value > old))
            .min();
        let Some(value) = next else {
            return Ok(total);
        };
        total = total
            .checked_add((value.len() + eid_len) as u64)
            .expect("Index wire length fits usize and u64");
        previous = Some(value);
    }
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
#[path = "committed_index_plan_tests.rs"]
mod tests;
