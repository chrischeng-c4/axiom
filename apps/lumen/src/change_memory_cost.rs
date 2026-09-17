//! Scratch proposal for Lumen #4246: admission-owned change-memory cost.
//!
//! This file is deliberately self-contained.  The controller can move this
//! into `apps/lumen/src/` after choosing the admission owner.

/// `FastHashMap` / `HashMap` bucket, key/value slots, and control-byte slack.
/// The stored Rust values are smaller, but 64 B stays above the 64-bit current
/// layouts after a table rounds its capacity.
pub const MAP_ENTRY_BYTES: usize = 64;
/// One `BTreeMap<String, u64>` dirty row: node, `String`, revision, and links.
pub const DIRTY_ID_ENTRY_BYTES: usize = 96;
/// `Postings` keeps two `Vec<u32>` streams.  Capacity rounding is charged by
/// `allocation`, so 16 B covers their two values and vector bookkeeping.
pub const POSTING_ROW_BYTES: usize = 16;
const ALLOC_HEADER_BYTES: usize = 16;
const ROARING_MEMBER_BYTES: usize = 32;
const BTREE_MEMBER_BYTES: usize = 64;
const DENSE_DOC_SLOT_BYTES: usize = 8;
/// `BTreeMap<String, Postings>` stores up to eleven `(String, Postings)` keys
/// per node. On 64-bit Rust the payload is `11 * (24 + 48) = 792` bytes, its
/// twelve child pointers add 96 bytes, and headers/alignment leave roughly
/// 920 bytes. A non-root node may hold only five keys, so 192 bytes per term
/// stays above the amortized node cost without assuming a 64-byte map entry.
const TEXT_BTREE_TERM_BYTES: usize = 192;
/// The first ordered dictionary node is allocated before its per-key capacity
/// amortizes. This also leaves explicit slack for root-node layout changes.
const TEXT_BTREE_FIRST_NODE_SLACK_BYTES: usize = 1024;
/// A changed Text document lives in sparse `delta_docs: HashMap<u32,
/// (u32, TokenSet)>`. This covers the map bucket, SmallVec's eight inline
/// Strings, and the first rounded hash allocation after the row grows past it.
/// It replaces a dense slot and never scales with the stable document ID.
const SPARSE_TEXT_DOCUMENT_BUCKET_BYTES: usize = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VectorBackendCost {
    Flat,
    Hnsw,
}

/// Input is normalized after schema lookup and text analysis.  It deliberately
/// carries term counts and bytes, not the encoded WAL size and not a cloned
/// postings expansion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldCost {
    Keyword {
        value_bytes: usize,
    },
    Number,
    Set {
        members: usize,
        /// Total UTF-8 bytes over all members, not bytes per member.
        member_bytes: usize,
    },
    Hash,
    Text {
        distinct_terms: usize,
        total_term_bytes: usize,
    },
    Vector {
        dim: usize,
        backend: VectorBackendCost,
        quantized_sq: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    Index {
        external_id_bytes: usize,
        new_document: bool,
        field: FieldCost,
        /// `cell_versions` entries for explicit item versions.
        volatile_metadata_bytes: usize,
    },
    Replace {
        external_id_bytes: usize,
        new_document: bool,
        fields: Vec<FieldCost>,
        /// `doc_versions` plus Text/Vector `field_checksums`. These maps are
        /// intentionally in-memory only, so they have no frozen clone charge.
        volatile_metadata_bytes: usize,
    },
    /// A complete unindex of one document. `field_count` is the current
    /// coverage count, obtained from `eid_fields`; it is not caller supplied.
    Unindex {
        external_id_bytes: usize,
        field_count: usize,
    },
    /// `docs:truncate` retains the old collection in the reclaimer.  The owner
    /// must pass its measured live owned bytes, including its interner.
    Truncate { retired_live_bytes: usize },
    /// Create/drop-field and create/drop-collection metadata after validation.
    Schema { metadata_bytes: usize },
    /// Direct engine and raft apply metadata, e.g. request-id or log routing.
    Direct { metadata_bytes: usize },
    /// Reshard accumulator key/value metadata. Payload buffering is charged by
    /// its owning reshard code separately.
    Reshard { metadata_bytes: usize },
}

impl Change {
    pub fn index_new_document(external_id_bytes: usize, field: FieldCost) -> Self {
        Self::Index {
            external_id_bytes,
            new_document: true,
            field,
            volatile_metadata_bytes: 0,
        }
    }
    pub fn index_existing_document(external_id_bytes: usize, field: FieldCost) -> Self {
        Self::Index {
            external_id_bytes,
            new_document: false,
            field,
            volatile_metadata_bytes: 0,
        }
    }
    pub fn unindex(external_id_bytes: usize, field_count: usize) -> Self {
        Self::Unindex {
            external_id_bytes,
            field_count,
        }
    }
    pub fn truncate() -> Self {
        Self::Truncate {
            retired_live_bytes: 0,
        }
    }
    pub fn schema(metadata_bytes: usize) -> Self {
        Self::Schema { metadata_bytes }
    }
    pub fn reshard(metadata_bytes: usize) -> Self {
        Self::Reshard { metadata_bytes }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cost {
    /// New mutable index, maps, and dirty bookkeeping owned after apply.
    pub active: usize,
    /// One detached checkpoint representation.  A first base captures every
    /// current row; an incremental checkpoint captures each dirty row.
    pub frozen: usize,
    /// Capture-time references and dirty-ID table space that coexist until
    /// durable publication; it is separate so admission can expose it.
    pub prepublish: usize,
}

impl Cost {
    pub fn total(self) -> usize {
        self.active
            .saturating_add(self.frozen)
            .saturating_add(self.prepublish)
    }
    fn checked_total(self) -> Result<usize, CostError> {
        add(add(self.active, self.frozen)?, self.prepublish)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CostError {
    Overflow,
}

/// Estimate the *additional owned resident memory* caused by one accepted
/// change.  Admission must add this to an owner-maintained reservation for
/// already accepted uncheckpointed rows.  It must seed that reservation from
/// every mutable row that existed before #4246, because first checkpoint base
/// capture clones those rows too.
pub fn estimate_change(change: &Change) -> Result<Cost, CostError> {
    let cost = match change {
        Change::Index {
            external_id_bytes,
            new_document,
            field,
            volatile_metadata_bytes,
        } => add_volatile(
            index_cost(
                *external_id_bytes,
                *new_document,
                std::slice::from_ref(field),
            )?,
            *volatile_metadata_bytes,
        )?,
        Change::Replace {
            external_id_bytes,
            new_document,
            fields,
            volatile_metadata_bytes,
        } => add_volatile(
            index_cost(*external_id_bytes, *new_document, fields)?,
            *volatile_metadata_bytes,
        )?,
        Change::Unindex {
            external_id_bytes,
            field_count,
        } => {
            // Removing a sealed value creates a field dirty record and may make
            // a tombstone bitmap.  It never credits old capacity before the
            // owning collection proves that Rust released it.
            let dirty = mul(*field_count, dirty_id(*external_id_bytes)?)?;
            Cost {
                active: dirty,
                frozen: dirty,
                prepublish: dirty,
            }
        }
        Change::Truncate { retired_live_bytes } => Cost {
            // The active collection is replaced, but `RetiredGeneration` keeps
            // the old collection until the asynchronous reclaimer drains it.
            active: *retired_live_bytes,
            frozen: 0,
            prepublish: MAP_ENTRY_BYTES,
        },
        Change::Schema { metadata_bytes }
        | Change::Direct { metadata_bytes }
        | Change::Reshard { metadata_bytes } => {
            let metadata = allocation(*metadata_bytes)?;
            Cost {
                active: metadata,
                frozen: metadata,
                prepublish: metadata,
            }
        }
    };
    // Reject before a wrapped counter can turn an impossible record into a
    // small reservation. `checked_total` also validates truncate inputs.
    cost.checked_total()?;
    Ok(cost)
}

fn index_cost(
    external_id_bytes: usize,
    new_document: bool,
    fields: &[FieldCost],
) -> Result<Cost, CostError> {
    let mut active = 0;
    let mut frozen = 0;
    let mut prepublish = 0;
    if new_document {
        // Interner `to_eid`, hash lookup, and `eid_fields` coverage entry.
        let id = add(
            add(string_bytes(external_id_bytes)?, MAP_ENTRY_BYTES)?,
            MAP_ENTRY_BYTES,
        )?;
        active = add(active, id)?;
        frozen = add(frozen, id)?; // base capture clones eids + coverage
    }
    for field in fields {
        let (field_active, field_frozen, field_prepublish) = field_cost(external_id_bytes, field)?;
        active = add(active, field_active)?;
        frozen = add(frozen, field_frozen)?;
        prepublish = add(prepublish, field_prepublish)?;
        // `field_dirty` is live state, cloned into `CheckpointCapture`, and
        // remains present until durable publication acknowledges it.
        let dirty = dirty_id(external_id_bytes)?;
        active = add(active, dirty)?;
        frozen = add(frozen, dirty)?;
        prepublish = add(prepublish, dirty)?;
    }
    Ok(Cost {
        active,
        frozen,
        prepublish,
    })
}

fn field_cost(
    external_id_bytes: usize,
    field: &FieldCost,
) -> Result<(usize, usize, usize), CostError> {
    let active = match field {
        FieldCost::Keyword { value_bytes } => add(
            add(string_bytes(*value_bytes)?, MAP_ENTRY_BYTES)?,
            add(ROARING_MEMBER_BYTES, DENSE_DOC_SLOT_BYTES)?,
        )?,
        FieldCost::Number => add(
            MAP_ENTRY_BYTES,
            add(ROARING_MEMBER_BYTES, DENSE_DOC_SLOT_BYTES)?,
        )?,
        FieldCost::Set {
            members,
            member_bytes,
        } => {
            // `elements` owns one string + bitmap membership per member and
            // `forward` owns another string in its per-document BTreeSet.
            // `member_bytes` already sums all member lengths. Charge it once
            // for each owned String copy, and charge only metadata per member.
            let strings = mul(2, string_collection_bytes(*member_bytes, *members)?)?;
            let entries = mul(
                *members,
                add(
                    add(MAP_ENTRY_BYTES, ROARING_MEMBER_BYTES)?,
                    BTREE_MEMBER_BYTES,
                )?,
            )?;
            add(strings, entries)?
        }
        FieldCost::Hash => add(MAP_ENTRY_BYTES, add(ROARING_MEMBER_BYTES, 8)?)?,
        FieldCost::Text {
            distinct_terms,
            total_term_bytes,
        } => {
            // `tokens` is an ordered BTreeMap and `distinct` stores a second
            // token copy for delete/reseal. Both string payload copies remain
            // charged; the sparse changed-document bucket is independent of a
            // high stable ID and replaces the former dense Vec slot.
            let strings = mul(
                2,
                string_collection_bytes(*total_term_bytes, *distinct_terms)?,
            )?;
            let ordered_dictionary = if *distinct_terms == 0 {
                0
            } else {
                add(
                    TEXT_BTREE_FIRST_NODE_SLACK_BYTES,
                    mul(*distinct_terms, TEXT_BTREE_TERM_BYTES)?,
                )?
            };
            let term = add(strings, ordered_dictionary)?;
            add(
                term,
                add(
                    mul(*distinct_terms, POSTING_ROW_BYTES)?,
                    SPARSE_TEXT_DOCUMENT_BUCKET_BYTES,
                )?,
            )?
        }
        FieldCost::Vector {
            dim,
            backend: _,
            quantized_sq,
        } => {
            let stored = mul(*dim, if *quantized_sq { 1 } else { 4 })?;
            // The 256 MiB change budget owns pending VectorStore payloads and
            // checkpoint rows. HNSW graph and query-side flat materialization
            // belong to separate RSS policy and are intentionally excluded.
            add(
                stored,
                add(string_bytes(external_id_bytes)?, MAP_ENTRY_BYTES)?,
            )?
        }
    };
    let frozen = match field {
        FieldCost::Vector { dim, .. } => {
            add(add(mul(*dim, 4)?, string_bytes(external_id_bytes)?)?, 48)?
        }
        _ => active,
    };
    let prepublish = match field {
        FieldCost::Vector { .. } => 16, // `Vec<Option<&[f32]>>` capture row
        _ => 0,
    };
    Ok((active, frozen, prepublish))
}

fn add_volatile(mut cost: Cost, bytes: usize) -> Result<Cost, CostError> {
    // `cell_versions`, `doc_versions`, and `field_checksums` do not appear in
    // FrozenField::capture. They still coexist with every live change.
    cost.active = add(cost.active, allocation(bytes)?)?;
    Ok(cost)
}

fn dirty_id(external_id_bytes: usize) -> Result<usize, CostError> {
    add(DIRTY_ID_ENTRY_BYTES, string_bytes(external_id_bytes)?)
}

fn string_bytes(bytes: usize) -> Result<usize, CostError> {
    allocation(bytes)
}

/// Upper bound for `count` independently allocated strings with combined UTF-8
/// bytes. Every allocation is below twice `(length + header)`, so this stays
/// linear in bytes and count without needing every term/member value.
fn string_collection_bytes(bytes: usize, count: usize) -> Result<usize, CostError> {
    add(mul(2, bytes)?, mul(2 * ALLOC_HEADER_BYTES, count)?)
}

/// A practical capacity upper bound for allocations made from one change.
/// Rust permits an allocator to return at least the request; rounding a
/// request plus header to the next power of two also covers ordinary Vec/String
/// growth without selecting an arbitrary multi-megabyte constant.
fn allocation(bytes: usize) -> Result<usize, CostError> {
    if bytes == 0 {
        return Ok(0);
    }
    add(bytes, ALLOC_HEADER_BYTES)?
        .checked_next_power_of_two()
        .ok_or(CostError::Overflow)
}
fn add(a: usize, b: usize) -> Result<usize, CostError> {
    a.checked_add(b).ok_or(CostError::Overflow)
}
fn mul(a: usize, b: usize) -> Result<usize, CostError> {
    a.checked_mul(b).ok_or(CostError::Overflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_cost_charges_terms_postings_ids_dirty_and_one_capture_clone() {
        let change = Change::index_new_document(
            12,
            FieldCost::Text {
                distinct_terms: 3,
                total_term_bytes: 15,
            },
        );
        let cost = estimate_change(&change).unwrap();
        assert!(cost.active > 12, "text needs more than its external id");
        assert_eq!(cost.frozen, cost.active);
        assert!(cost.prepublish >= DIRTY_ID_ENTRY_BYTES);
        assert_eq!(cost.total(), cost.active + cost.frozen + cost.prepublish);
    }

    #[test]
    fn ngram_is_charged_by_normalized_terms_not_original_wire_bytes() {
        let one_wire_byte = Change::index_existing_document(
            2,
            FieldCost::Text {
                distinct_terms: 5,
                total_term_bytes: 15,
            },
        );
        let cost = estimate_change(&one_wire_byte).unwrap();
        assert!(cost.active >= 5 * POSTING_ROW_BYTES);
        assert!(cost.active > 15);
    }

    #[test]
    fn hnsw_vector_charges_payload_ids_maps_and_a_checkpoint_row_without_wal_bytes() {
        let change = Change::index_new_document(
            9,
            FieldCost::Vector {
                dim: 768,
                backend: VectorBackendCost::Hnsw,
                quantized_sq: false,
            },
        );
        let cost = estimate_change(&change).unwrap();
        assert!(cost.active >= 768 * 4);
        assert!(
            cost.active > 768 * 4,
            "pending payload has a VectorStore map"
        );
        assert!(cost.frozen >= 768 * 4 + 9);
    }

    #[test]
    fn delete_and_schema_changes_charge_their_dirty_or_metadata_work() {
        assert!(estimate_change(&Change::unindex(24, 4)).unwrap().total() > 0);
        assert!(estimate_change(&Change::schema(40)).unwrap().total() >= 40);
        assert!(estimate_change(&Change::reshard(128)).unwrap().total() >= 128);
        assert!(estimate_change(&Change::truncate()).unwrap().total() >= MAP_ENTRY_BYTES);
    }

    #[test]
    fn set_total_bytes_scale_once_per_owned_copy_not_per_member() {
        let small = estimate_change(&Change::index_existing_document(
            8,
            FieldCost::Set {
                members: 10,
                member_bytes: 100,
            },
        ))
        .unwrap();
        let large = estimate_change(&Change::index_existing_document(
            8,
            FieldCost::Set {
                members: 10,
                member_bytes: 200,
            },
        ))
        .unwrap();
        // 2 copies * (2 * 100 B collection allocation bound) = 400 B.
        assert_eq!(large.active - small.active, 400);
        assert_eq!(large.frozen - small.frozen, 400);
    }

    #[test]
    fn text_charges_allocation_headers_per_distinct_term() {
        let one = estimate_change(&Change::index_existing_document(
            8,
            FieldCost::Text {
                distinct_terms: 1,
                total_term_bytes: 100,
            },
        ))
        .unwrap();
        let ten = estimate_change(&Change::index_existing_document(
            8,
            FieldCost::Text {
                distinct_terms: 10,
                total_term_bytes: 100,
            },
        ))
        .unwrap();
        assert_eq!(
            ten.active - one.active,
            9 * (2 * 2 * ALLOC_HEADER_BYTES + TEXT_BTREE_TERM_BYTES + POSTING_ROW_BYTES)
        );
    }

    #[test]
    fn text_dictionary_charges_first_btree_node_slack_and_sparse_document_bucket() {
        let zero = estimate_change(&Change::index_existing_document(
            8,
            FieldCost::Text {
                distinct_terms: 0,
                total_term_bytes: 0,
            },
        ))
        .unwrap();
        let one = estimate_change(&Change::index_existing_document(
            8,
            FieldCost::Text {
                distinct_terms: 1,
                total_term_bytes: 1,
            },
        ))
        .unwrap();
        let many = estimate_change(&Change::index_existing_document(
            8,
            FieldCost::Text {
                distinct_terms: 12,
                total_term_bytes: 12,
            },
        ))
        .unwrap();
        assert!(one.active > zero.active);
        assert!(many.active > one.active);
        assert!(zero.active >= SPARSE_TEXT_DOCUMENT_BUCKET_BYTES);
        assert!(
            one.active - zero.active >= TEXT_BTREE_FIRST_NODE_SLACK_BYTES + TEXT_BTREE_TERM_BYTES,
            "the first ordered dictionary node and one sparse row must be reserved"
        );
    }

    #[test]
    fn text_sparse_row_estimate_has_no_dense_prefix_component() {
        let changed_row = || {
            estimate_change(&Change::index_existing_document(
                8,
                FieldCost::Text {
                    distinct_terms: 1,
                    total_term_bytes: 4,
                },
            ))
            .unwrap()
        };
        let row_zero = changed_row();
        // The estimator has no document ordinal input. A far sparse stable ID
        // therefore keeps this same per-row reservation; storage owns the
        // separate no-dense-prefix proof for `delta_docs`.
        let row_large_sparse_id = changed_row();
        assert_eq!(row_large_sparse_id, row_zero);
    }

    #[test]
    fn text_zero_terms_is_finite_and_high_term_inputs_fail_checked() {
        let zero = estimate_change(&Change::index_existing_document(
            8,
            FieldCost::Text {
                distinct_terms: 0,
                total_term_bytes: 0,
            },
        ))
        .unwrap();
        assert!(
            zero.total() > 0,
            "dirty tracking remains owned for empty text"
        );
        let bytes_overflow = Change::index_existing_document(
            8,
            FieldCost::Text {
                distinct_terms: 1,
                total_term_bytes: usize::MAX,
            },
        );
        let terms_overflow = Change::index_existing_document(
            8,
            FieldCost::Text {
                distinct_terms: usize::MAX,
                total_term_bytes: 0,
            },
        );
        assert_eq!(estimate_change(&bytes_overflow), Err(CostError::Overflow));
        assert_eq!(estimate_change(&terms_overflow), Err(CostError::Overflow));
    }

    #[test]
    fn hnsw_graph_is_outside_the_change_budget() {
        let flat = estimate_change(&Change::index_existing_document(
            8,
            FieldCost::Vector {
                dim: 768,
                backend: VectorBackendCost::Flat,
                quantized_sq: false,
            },
        ))
        .unwrap();
        let hnsw = estimate_change(&Change::index_existing_document(
            8,
            FieldCost::Vector {
                dim: 768,
                backend: VectorBackendCost::Hnsw,
                quantized_sq: false,
            },
        ))
        .unwrap();
        assert_eq!(hnsw, flat);
    }

    #[test]
    fn explicit_versions_and_replace_checksums_stay_active_but_are_not_checkpoint_clones() {
        let plain = Change::index_existing_document(8, FieldCost::Number);
        let versioned = Change::Index {
            external_id_bytes: 8,
            new_document: false,
            field: FieldCost::Number,
            volatile_metadata_bytes: 40,
        };
        let plain = estimate_change(&plain).unwrap();
        let versioned = estimate_change(&versioned).unwrap();
        assert!(versioned.active > plain.active);
        assert_eq!(versioned.frozen, plain.frozen);
        assert_eq!(versioned.prepublish, plain.prepublish);
    }

    #[test]
    fn checked_overflow_rejects_an_unrepresentable_reservation() {
        let change = Change::index_new_document(usize::MAX, FieldCost::Keyword { value_bytes: 1 });
        assert_eq!(estimate_change(&change), Err(CostError::Overflow));
    }
}
