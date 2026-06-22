---
id: projects-lumen-src-storage-rs
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/storage.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/storage.rs` captured as a per-file rust-source-unit (td_ast) during lumen standardization onto the per-file codegen ladder.

### Symbols

| Name | Target | Kind | Visibility |
|------|--------|------|------------|
| `MAX_INDEX_ITEMS` | projects/lumen/src/storage.rs | constant | pub |
| `DropOutcome` | projects/lumen/src/storage.rs | enum | pub |
| `StorageError` | projects/lumen/src/storage.rs | enum | pub |
| `SortableF64` | projects/lumen/src/storage.rs | struct | pub |
| `new` | projects/lumen/src/storage.rs | function | pub |
| `to_f64` | projects/lumen/src/storage.rs | function | pub |
| `bits` | projects/lumen/src/storage.rs | function | pub |
| `from_bits` | projects/lumen/src/storage.rs | function | pub |
| `Postings` | projects/lumen/src/storage.rs | struct | pub |
| `from_sorted` | projects/lumen/src/storage.rs | function | pub |
| `docids` | projects/lumen/src/storage.rs | function | pub |
| `tfs` | projects/lumen/src/storage.rs | function | pub |
| `Engine` | projects/lumen/src/storage.rs | struct | pub |
| `new` | projects/lumen/src/storage.rs | function | pub |
| `metrics` | projects/lumen/src/storage.rs | function | pub |
| `start_drain` | projects/lumen/src/storage.rs | function | pub |
| `is_draining` | projects/lumen/src/storage.rs | function | pub |
| `create_collection` | projects/lumen/src/storage.rs | function | pub |
| `drop_collection` | projects/lumen/src/storage.rs | function | pub |
| `drop_field` | projects/lumen/src/storage.rs | function | pub |
| `sweep_deleted` | projects/lumen/src/storage.rs | function | pub |
| `add_field` | projects/lumen/src/storage.rs | function | pub |
| `list_collections` | projects/lumen/src/storage.rs | function | pub |
| `index` | projects/lumen/src/storage.rs | function | pub |
| `delete` | projects/lumen/src/storage.rs | function | pub |
| `number_value_for_external_id` | projects/lumen/src/storage.rs | function | pub |
| `search` | projects/lumen/src/storage.rs | function | pub |
| `search_fast_string_term` | projects/lumen/src/storage.rs | function | pub |
| `duplicates` | projects/lumen/src/storage.rs | function | pub |
| `snapshot` | projects/lumen/src/storage.rs | function | pub |
| `restore` | projects/lumen/src/storage.rs | function | pub |
| `stats` | projects/lumen/src/storage.rs | function | pub |
| `apply_raft_entry` | projects/lumen/src/storage.rs | function | pub |
| `ApplyOutcome` | projects/lumen/src/storage.rs | enum | pub |
| `validate_query` | projects/lumen/src/storage.rs | function | pub |
| `SnapshotV1` | projects/lumen/src/storage.rs | struct | pub |
| `CollectionSnapshot` | projects/lumen/src/storage.rs | struct | pub |
| `FieldIndexSnapshot` | projects/lumen/src/storage.rs | enum | pub |
| `seal_to_segments` | projects/lumen/src/storage.rs | function | pub |
| `open_from_segments` | projects/lumen/src/storage.rs | function | pub |
| `flush_to_segments` | projects/lumen/src/storage.rs | function | pub |
| `reopen_from_segment_dir` | projects/lumen/src/storage.rs | function | pub |
| `segment_field_probe` | projects/lumen/src/storage.rs | function | pub |
| `__seal_number_field_to_segment` | projects/lumen/src/storage.rs | function | pub |
| `__seal_keyword_field_to_segment` | projects/lumen/src/storage.rs | function | pub |
| `__seal_set_field_to_segment` | projects/lumen/src/storage.rs | function | pub |
| `__seal_text_field_to_segment` | projects/lumen/src/storage.rs | function | pub |
| `__seal_hash_field_to_segment` | projects/lumen/src/storage.rs | function | pub |
| `__seal_vector_field_to_segment` | projects/lumen/src/storage.rs | function | pub |
| `__seal_collection_to_segments` | projects/lumen/src/storage.rs | function | pub |
| `__collection_schema` | projects/lumen/src/storage.rs | function | pub |
| `__open_collection_from_segments` | projects/lumen/src/storage.rs | function | pub |
| `__field_forward_probe` | projects/lumen/src/storage.rs | function | pub |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! In-memory storage and query execution.
//!
//! The engine is `BTreeMap`-backed inverted indexes per field,
//! constructed by [`Engine::new`]. Single-pod, single-shard; durability
//! comes from the CBOR RDB snapshot path and (when segment persistence is
//! selected) the columnar mmap segment tier — not from this module.
//!
//! The shape below maps 1:1 to the field-type table in the README:
//!
//! | FieldType | Index                                  |
//! |-----------|----------------------------------------|
//! | `text`    | `BTreeMap<token, BTreeSet<eid>>`       |
//! | `keyword` | `BTreeMap<value, BTreeSet<eid>>`       |
//! | `number`  | `BTreeMap<SortableF64, BTreeSet<eid>>` |
//! | `set`     | `BTreeMap<element, BTreeSet<eid>>`     |
//!
//! Every field also carries a per-`external_id` "forward" map so
//! re-indexing the same `(eid, field)` cleanly evicts the old postings
//! before appending the new ones.

use std::cmp::Ordering as CmpOrdering;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Result};
use rustc_hash::{FxHashMap, FxHasher};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use thiserror::Error;

use crate::metrics::Metrics;
use crate::tokenize;
use crate::types::{
    Analyzer, CacheStats, CreateCollectionRequest, CreateCollectionResponse, DuplicateGroup,
    DuplicatesRequest, DuplicatesResponse, FieldSpec, FieldStats, FieldType, FieldValue,
    HammingQuery, HasChildQuery, IndexRequest, IndexResponse, KnnQuery, MatchOp, MatchQuery,
    QueryNode, RangeQuery, RrfQuery, SearchHit, SearchRequest, SearchResponse, SortOrder, SortSpec,
    StatsResponse, StorageStats, TermQuery, TermsQuery, VectorSpec,
};
use crate::vector_index::{open_backend, FlatCpuIndex, HnswCpuIndex, ScalarCodebook, VectorIndex};
use roaring::RoaringBitmap;

const IDEMPOTENCY_TTL: Duration = Duration::from_secs(300);
const SEARCH_RESULT_CACHE_MAX: usize = 256;

type FastHashMap<K, V> = FxHashMap<K, V>;

/// Maximum items in a single `POST /index` request (README §1 v1 limit).
pub const MAX_INDEX_ITEMS: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
pub enum DropOutcome {
    /// The collection did not exist.
    NotFound,
    /// `force=false` — soft-deleted, awaiting sweep.
    Marked,
    /// `force=false` but already marked previously.
    AlreadyMarked,
    /// `force=true` — physically removed.
    Physical,
}

#[derive(Debug, Error)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
pub enum StorageError {
    #[error("collection not found: {0}")]
    CollectionNotFound(String),
    #[error("unknown field `{field}` in collection `{collection}`")]
    UnknownField { collection: String, field: String },
    #[error("type mismatch on field `{field}`: expected {expected:?}, got {got}")]
    TypeMismatch {
        field: String,
        expected: FieldType,
        got: &'static str,
    },
    #[error("duplicates not supported on text field `{0}`")]
    DuplicatesOnText(String),
    #[error("invalid number value: {0}")]
    InvalidNumber(String),
    #[error("bulk index limit exceeded: got {got} items (max {max})")]
    BulkLimit { got: usize, max: usize },
    #[error("query too complex: {0}")]
    QueryTooComplex(String),
    #[error("unsupported sort: {0}")]
    UnsupportedSort(String),
    #[error("collection `{0}` was deleted and is pending physical removal")]
    Gone(String),
}

// ---------------------------------------------------------------------------
// Sortable f64 key
// ---------------------------------------------------------------------------

/// Total-ordered, bit-monotone wrapper around `f64`. NaN is rejected at
/// construction (the API layer must validate before reaching here).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
pub struct SortableF64(u64);

const MISSING_SORTABLE_F64_BITS: u64 = 0xfff8_0000_0000_0000;

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl SortableF64 {
    pub fn new(x: f64) -> Result<Self> {
        if x.is_nan() {
            bail!("NaN is not a valid number value");
        }
        let bits = x.to_bits();
        // Non-negatives → flip top bit (places them above negatives).
        // Negatives → flip all bits (reverses their natural order so
        // larger magnitudes sort earlier among negatives).
        let key = if x.is_sign_negative() {
            !bits
        } else {
            bits ^ (1u64 << 63)
        };
        Ok(SortableF64(key))
    }

    pub fn to_f64(self) -> f64 {
        let bits = if self.0 & (1u64 << 63) != 0 {
            // Top bit set → originally non-negative.
            self.0 ^ (1u64 << 63)
        } else {
            !self.0
        };
        f64::from_bits(bits)
    }

    /// The raw order-preserving `u64` bit key. UNSIGNED `u64` order == numeric
    /// order (the whole point of the transform), so this is exactly the key the
    /// on-disk sorted-value range index ([`crate::segment`] `ROLE_NUMBER_SORTED`)
    /// stores and binary-searches. Phase 2h-3.
    #[inline]
    pub(crate) fn bits(self) -> u64 {
        self.0
    }

    /// Reconstruct a `SortableF64` from its raw bit key (the inverse of
    /// [`Self::bits`]) — used to lift an on-disk sorted-value key back into the
    /// in-RAM key space. Phase 2h-3.
    #[inline]
    pub(crate) fn from_bits(bits: u64) -> Self {
        SortableF64(bits)
    }
}

/// `true` when a `(low, high)` `SortableF64` range is EMPTY by construction — an
/// inverted (`low > high`) or degenerate-exclusive (`low == high` with either
/// endpoint excluded) range. `BTreeMap::range` PANICS on such a pair (it asserts
/// `start <= end`, and rejects an equal pair where a bound is `Excluded`), so the
/// in-RAM walk MUST be guarded by this and short-circuit to empty — matching the
/// on-disk `number_range_window`, which already collapses these to an empty
/// window. A range with at least one `Unbounded` end is never empty by this rule.
/// Phase 2h-3 (defensive: an inverted range was previously an unguarded panic).
fn range_is_empty(low: std::ops::Bound<SortableF64>, high: std::ops::Bound<SortableF64>) -> bool {
    use std::ops::Bound::*;
    let (lo, lo_excl) = match low {
        Included(b) => (b, false),
        Excluded(b) => (b, true),
        Unbounded => return false,
    };
    let (hi, hi_excl) = match high {
        Included(b) => (b, false),
        Excluded(b) => (b, true),
        Unbounded => return false,
    };
    lo > hi || (lo == hi && (lo_excl || hi_excl))
}

/// Lower a range `Bound<SortableF64>` into the `(bits, inclusive)` shape the
/// on-disk sorted-value range index ([`crate::segment::SegmentReader::number_range`])
/// consumes: `Included(b) -> Some((b.bits(), true))`, `Excluded(b) -> Some((b.bits(),
/// false))`, `Unbounded -> None`. The disk reader's `number_range_window` applies
/// exactly the same inclusive/exclusive semantics `BTreeMap::range` does, so a
/// segment-driven range is byte-identical to the in-RAM `values.range`. Phase 2h-3.
#[inline]
fn bound_to_bits(b: std::ops::Bound<SortableF64>) -> Option<(u64, bool)> {
    use std::ops::Bound;
    match b {
        Bound::Included(v) => Some((v.bits(), true)),
        Bound::Excluded(v) => Some((v.bits(), false)),
        Bound::Unbounded => None,
    }
}

fn range_cache_key(lo: &std::ops::Bound<SortableF64>, hi: &std::ops::Bound<SortableF64>) -> String {
    use std::ops::Bound;
    let mut key = String::new();
    match lo {
        Bound::Included(v) => {
            key.push_str("i:");
            key.push_str(&v.bits().to_string());
        }
        Bound::Excluded(v) => {
            key.push_str("e:");
            key.push_str(&v.bits().to_string());
        }
        Bound::Unbounded => key.push_str("u"),
    }
    key.push('|');
    match hi {
        Bound::Included(v) => {
            key.push_str("i:");
            key.push_str(&v.bits().to_string());
        }
        Bound::Excluded(v) => {
            key.push_str("e:");
            key.push_str(&v.bits().to_string());
        }
        Bound::Unbounded => key.push_str("u"),
    }
    key
}

// ---------------------------------------------------------------------------
// Field index variants
// ---------------------------------------------------------------------------

/// Per-collection external_id ↔ dense u32 doc-id map. Posting lists and the
/// query path carry the `u32` (cheap to copy/hash, no per-match String clone);
/// the `String` external_id is resolved only at the API boundary. Append-only:
/// re-indexing an eid returns the same id; deletes do not reuse ids (they
/// compact on the next snapshot round-trip).
#[derive(Debug, Default)]
struct Interner {
    to_hash: FastHashMap<u64, InternerBucket>,
    to_eid: Vec<String>,
}

#[derive(Debug)]
enum InternerBucket {
    One(u32),
    Many(Vec<u32>),
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl Interner {
    fn intern(&mut self, eid: &str) -> u32 {
        self.intern_with_status(eid).0
    }

    fn intern_with_status(&mut self, eid: &str) -> (u32, bool) {
        let hash = hash_external_id(eid);
        if let Some(id) = self.id_with_hash(eid, hash) {
            return (id, false);
        }
        let id = self.to_eid.len() as u32;
        self.to_eid.push(eid.to_string());
        self.insert_hash(hash, id);
        (id, true)
    }

    fn intern_owned_with_status(&mut self, eid: String) -> (u32, bool) {
        let hash = hash_external_id(&eid);
        if let Some(id) = self.id_with_hash(&eid, hash) {
            return (id, false);
        }
        let id = self.to_eid.len() as u32;
        self.to_eid.push(eid);
        self.insert_hash(hash, id);
        (id, true)
    }

    fn insert_hash(&mut self, hash: u64, id: u32) {
        match self.to_hash.get_mut(&hash) {
            Some(InternerBucket::One(existing)) => {
                let existing = *existing;
                self.to_hash
                    .insert(hash, InternerBucket::Many(vec![existing, id]));
            }
            Some(InternerBucket::Many(ids)) => ids.push(id),
            None => {
                self.to_hash.insert(hash, InternerBucket::One(id));
            }
        }
    }

    fn id(&self, eid: &str) -> Option<u32> {
        self.id_with_hash(eid, hash_external_id(eid))
    }

    fn id_with_hash(&self, eid: &str, hash: u64) -> Option<u32> {
        match self.to_hash.get(&hash)? {
            InternerBucket::One(id) => (self.resolve(*id) == eid).then_some(*id),
            InternerBucket::Many(ids) => ids.iter().copied().find(|id| self.resolve(*id) == eid),
        }
    }

    fn resolve(&self, id: u32) -> &str {
        &self.to_eid[id as usize]
    }
}

fn hash_external_id(eid: &str) -> u64 {
    let mut hasher = FxHasher::default();
    eid.hash(&mut hasher);
    hasher.finish()
}

/// A token's posting list as flat, docid-sorted parallel arrays (struct-of-arrays).
/// The BM25 scan reads `docids`/`tfs` sequentially (cache-friendly), and `df` is
/// exactly `docids.len()`. Replaces the old `BTreeMap<u32,u32>` whose per-doc
/// access chased heap-scattered tree nodes.
#[derive(Debug, Default, Clone)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
pub(crate) struct Postings {
    docids: Vec<u32>,
    tfs: Vec<u32>,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl Postings {
    /// Build a posting list from ascending `(docid, tf)` pairs. Crate-internal,
    /// used by the Text segment writer round-trip test to fabricate postings
    /// without the full index. Phase 2e-B.
    #[cfg(test)]
    pub(crate) fn from_sorted(docids: Vec<u32>, tfs: Vec<u32>) -> Self {
        debug_assert_eq!(docids.len(), tfs.len());
        Postings { docids, tfs }
    }
    /// The docid-sorted doc ids of this posting list (used by the Text segment
    /// writer to delta-encode the posting block). Phase 2e-B.
    pub(crate) fn docids(&self) -> &[u32] {
        &self.docids
    }
    /// The term frequencies, parallel to [`Self::docids`]. Phase 2e-B.
    pub(crate) fn tfs(&self) -> &[u32] {
        &self.tfs
    }
    /// Insert/overwrite `id`'s tf, keeping `docids` sorted. For the monotonic
    /// bulk-index path the insertion point is the tail (O(1) amortized); a reused
    /// id (interner reuses ids; an overwrite drops first) lands in sorted order.
    fn upsert(&mut self, id: u32, tf: u32) {
        match self.docids.last().copied() {
            None => {
                self.docids.push(id);
                self.tfs.push(tf);
                return;
            }
            Some(last) if last < id => {
                self.docids.push(id);
                self.tfs.push(tf);
                return;
            }
            Some(last) if last == id => {
                if let Some(last_tf) = self.tfs.last_mut() {
                    *last_tf = tf;
                }
                return;
            }
            _ => {}
        }
        match self.docids.binary_search(&id) {
            Ok(pos) => self.tfs[pos] = tf,
            Err(pos) => {
                self.docids.insert(pos, id);
                self.tfs.insert(pos, tf);
            }
        }
    }
    fn upsert_add(&mut self, id: u32, delta: u32) {
        match self.docids.last().copied() {
            None => {
                self.docids.push(id);
                self.tfs.push(delta);
                return;
            }
            Some(last) if last < id => {
                self.docids.push(id);
                self.tfs.push(delta);
                return;
            }
            Some(last) if last == id => {
                if let Some(last_tf) = self.tfs.last_mut() {
                    *last_tf += delta;
                }
                return;
            }
            _ => {}
        }
        match self.docids.binary_search(&id) {
            Ok(pos) => self.tfs[pos] += delta,
            Err(pos) => {
                self.docids.insert(pos, id);
                self.tfs.insert(pos, delta);
            }
        }
    }
    fn remove(&mut self, id: u32) -> bool {
        match self.docids.binary_search(&id) {
            Ok(pos) => {
                self.docids.remove(pos);
                self.tfs.remove(pos);
                true
            }
            Err(_) => false,
        }
    }
    /// tf of `id`, if present (random access for the filtered-AND predicate path).
    fn tf(&self, id: u32) -> Option<u32> {
        self.docids.binary_search(&id).ok().map(|pos| self.tfs[pos])
    }
    fn df(&self) -> usize {
        self.docids.len()
    }
}

/// A token's postings resolved for BM25 scoring, from EITHER the live in-RAM
/// `Postings` (borrowed, zero-copy) or a sealed Text segment's decoded posting
/// block (owned `Vec`s; text tf is STORED — Phase 2e-B). Both variants expose
/// the identical `(docids, tfs)` u32 streams in the SAME ascending-docid order,
/// so the BM25 score expression is fed bit-identical inputs on both paths.
enum TokPostings<'a> {
    Live(&'a Postings),
    /// The cache-resident segment posting, shared by `Arc` (Phase 2m). The hot
    /// no-tombstone path holds the BOUNDED Text posting cache's `Arc` DIRECTLY —
    /// a per-candidate `match_doc_score` probe is then a `.tf(id)` binary-search
    /// over the shared streams with NO re-decode and NO per-call vector copy (the
    /// `filtered_search` 25x disk fix). Only built when a segment is attached.
    Segment(std::sync::Arc<(Vec<u32>, Vec<u32>)>),
    /// Owned, tombstone-filtered segment posting — the RARE delete-after-seal path
    /// where the cached `Arc` (RAW immutable, includes deleted base docids) must
    /// have the tombstone subtracted into a private copy. Only built when a delete
    /// is pending, so the common warm query never allocates.
    SegmentFiltered {
        docids: Vec<u32>,
        tfs: Vec<u32>,
    },
}

impl<'a> TokPostings<'a> {
    #[inline]
    fn docids(&self) -> &[u32] {
        match self {
            TokPostings::Live(p) => &p.docids,
            TokPostings::Segment(p) => &p.0,
            TokPostings::SegmentFiltered { docids, .. } => docids,
        }
    }
    #[inline]
    fn tfs(&self) -> &[u32] {
        match self {
            TokPostings::Live(p) => &p.tfs,
            TokPostings::Segment(p) => &p.1,
            TokPostings::SegmentFiltered { tfs, .. } => tfs,
        }
    }
    #[inline]
    fn df(&self) -> usize {
        self.docids().len()
    }
    /// tf of `id` via binary-search (docids ascending), or `None` — the same
    /// random-access probe `Postings::tf` does, on either source.
    #[inline]
    fn tf(&self, id: u32) -> Option<u32> {
        match self {
            TokPostings::Live(p) => p.tf(id),
            TokPostings::Segment(p) => p.0.binary_search(&id).ok().map(|pos| p.1[pos]),
            TokPostings::SegmentFiltered { docids, tfs } => {
                docids.binary_search(&id).ok().map(|pos| tfs[pos])
            }
        }
    }
}

#[derive(Debug, Default)]
struct TextIndex {
    /// token → flat docid-sorted postings (docid + tf).
    tokens: FastHashMap<String, Postings>,
    /// dense doc-len indexed by doc-id (0 = no value for this field). Replaces a
    /// per-doc `forward` HashMap probe with a sequential Vec read on the hot loop.
    lens: Vec<u32>,
    /// dense doc-id → distinct tokens emitted. Used ONLY by `drop_eid` /
    /// snapshots / coverage, so new writes avoid a per-doc HashMap insert.
    distinct: Vec<Option<TokenSet>>,
    doc_count: u64,
    total_doc_len: u64,
    bytes: u64,
    /// Stage 2 disk-tier (Phase 2e-B): a sealed columnar mmap segment covering
    /// the WHOLE field for doc ids `[0..n_docs)` — a sorted token DICT + a
    /// parallel per-token STORED posting block (text tf is NOT rebuildable, so
    /// the inverted postings live on disk) + a fixed `u32[n_docs]` DocLen column
    /// + the BM25 corpus scalars in the header. When present, the BM25 scan
    /// (`eval_match` / `match_doc_score`) and `estimate_selectivity` read
    /// entirely from the segment (this slice seals the whole field with no live
    /// tail; segment+live composition is Phase 2f). The `distinct` map is
    /// rebuilt eagerly on seal so `drop_eid` is unaffected. DEFAULTS to `None`;
    /// while it is `None` (nothing sealed) every read path is byte-for-byte the
    /// in-RAM path. Purely additive.
    segment: Option<std::sync::Arc<crate::segment::SegmentReader>>,
    /// Hot BM25 rankings, sorted by score desc then external_id asc. Used for
    /// unique-doc match shapes (single token, multi-token AND) and cleared on any
    /// text mutation/seal so cached scores never cross corpus states.
    match_rank_cache: RwLock<FastHashMap<String, std::sync::Arc<Vec<(u32, f32)>>>>,
    /// QUERY-TIME TOMBSTONE (Phase 2h-4): base docids `[0..seg.n_docs)` deleted
    /// SINCE the last seal. The inverted `tokens` postings AND the corpus-deriving
    /// `distinct` map were DROPPED to disk at seal, so `drop_eid` can no longer
    /// remove a sealed base id from the immutable on-disk posting blocks — instead
    /// it records the id here, decrements the LIVE corpus scalars
    /// (`doc_count`/`total_doc_len`), and the BM25 scan SUBTRACTS this set from
    /// every token's segment posting BEFORE computing `df` and BEFORE scoring. So
    /// `df' = |posting − tombstones|`, the `idf` uses `df'`, and a tombstoned doc
    /// is never scored — byte-identical to an in-RAM oracle that physically removed
    /// the doc. The next seal bakes the deletions into the new segment (its
    /// `live(id)` gather excludes them) and this is reset to empty. Live-tail ids
    /// (`>= seg.n_docs`) are NOT tombstoned — they are deleted directly out of the
    /// in-RAM `tokens`/`distinct` tail. DEFAULTS empty; stays empty while no
    /// segment is attached (the in-RAM `tokens` path needs no tombstone — a delete
    /// mutates `tokens` directly). Mirrors 2h-1/2h-2/2h-3's four touch-points.
    tombstones: RoaringBitmap,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl TextIndex {
    fn clear_match_rank_cache(&self) {
        if let Ok(mut cache) = self.match_rank_cache.write() {
            cache.clear();
        }
    }

    fn doc_len(&self, id: u32) -> u32 {
        if let Some(seg) = &self.segment {
            return seg.text_doc_len(id);
        }
        self.lens.get(id as usize).copied().unwrap_or(0)
    }
    fn set_doc_len(&mut self, id: u32, len: u32) {
        if self.lens.len() <= id as usize {
            self.lens.resize(id as usize + 1, 0);
        }
        self.lens[id as usize] = len;
    }

    fn set_distinct(&mut self, id: u32, tokens: TokenSet) {
        let ix = id as usize;
        if self.distinct.len() <= ix {
            self.distinct.resize_with(ix + 1, || None);
        }
        self.distinct[ix] = Some(tokens);
    }

    fn take_distinct(&mut self, id: u32) -> Option<TokenSet> {
        self.distinct
            .get_mut(id as usize)
            .and_then(|tokens| tokens.take())
    }

    fn distinct_is_empty(&self) -> bool {
        self.distinct.iter().all(Option::is_none)
    }

    fn distinct_iter(&self) -> impl Iterator<Item = (u32, &TokenSet)> {
        self.distinct
            .iter()
            .enumerate()
            .filter_map(|(id, tokens)| tokens.as_ref().map(|tokens| (id as u32, tokens)))
    }

    fn distinct_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.distinct_iter().map(|(id, _)| id)
    }

    /// `(n, total_len)` for the BM25 corpus — ALWAYS the LIVE scalars
    /// (`self.doc_count`, `self.total_doc_len`), never the segment header (Phase
    /// 2h-4 FIX). On reopen the live scalars are INITIALIZED from the header
    /// (`open_from_segment`); the index path increments them on every doc; and
    /// `drop_eid` DECREMENTS them on a delete (including a sealed-base delete that
    /// only tombstones the immutable posting). The segment header is a base-only
    /// SEAL-TIME snapshot — using it would (a) keep a stale `N`/`avgdl` after a
    /// delete-after-seal (the crux this phase closes) and (b) undercount a
    /// post-seal live tail. The live scalars stay current across all three, so
    /// `n`/`avgdl` are byte-identical to an in-RAM oracle on the same op sequence.
    /// On a FIRST seal (no delete, no tail) the live scalars equal the header, so
    /// the BM25 score is unchanged from 2e-B. The `doc_count == 0` short-circuit in
    /// the caller mirrors the live path.
    fn bm25_corpus(&self) -> (u64, u64) {
        (self.doc_count, self.total_doc_len)
    }

    /// A token's postings for BM25 scoring: from the attached segment when
    /// sealed (text tf is STORED — Phase 2e-B), else the live in-RAM `tokens`
    /// map. `None` if the token has no LIVE postings on the active source. Both
    /// sources return identical `(docids, tfs)`, so the score is bit-identical.
    ///
    /// TOMBSTONE SUBTRACTION (Phase 2h-4 FIX, the crux): on the segment path the
    /// decoded base posting still holds any base docid deleted SINCE the last seal
    /// (the on-disk block is immutable; `drop_eid` only recorded the id in
    /// `tombstones`). Those ids are filtered OUT here — BEFORE the `TokPostings` is
    /// handed to the BM25 scan. Because EVERY downstream read (`df()`, `docids()`,
    /// `tfs()`, `tf(id)`) goes through this single filtered posting, the effect is
    /// uniform across `eval_match` (Or + And) and `match_doc_score`:
    ///   • `df' = |posting − tombstones|` ⇒ the `idf` uses the live document
    ///     frequency, so it equals an in-RAM oracle's `idf`;
    ///   • a tombstoned docid is absent from `docids()`/`tf(id)` ⇒ it is never
    ///     scored and never enters the result set.
    /// Combined with the LIVE corpus scalars (`bm25_corpus`, already decremented by
    /// the delete), `n`/`avgdl`/`idf`/per-doc score are byte-identical to an oracle
    /// that physically removed the doc. A FULLY-tombstoned token (every base docid
    /// deleted) collapses to an empty posting ⇒ `None`, matching the in-RAM
    /// semantics where `drop_eid` removes an emptied token from `tokens`. Tombstones
    /// only ever hold base ids (`< seg.n_docs`), so this never touches a tail id.
    #[inline]
    fn tok_postings(&self, tok: &str) -> Option<TokPostings<'_>> {
        if let Some(seg) = &self.segment {
            // Phase 2m: hold the cache-resident `Arc` directly. With NO pending
            // delete (the common warm path) this is a refcount bump — the
            // per-candidate `match_doc_score` `.tf(id)` probe then binary-searches
            // the SHARED streams with no re-decode and no per-call vector copy (the
            // `filtered_search` 25x disk fix). The cached posting is the RAW
            // immutable stream; results are byte-identical because the RARE
            // tombstone branch below subtracts the deleted base docids.
            let cached = seg.text_postings_arc(tok)?;
            if self.tombstones.is_empty() {
                return Some(TokPostings::Segment(cached));
            }
            // Deletes pending: subtract the tombstone into a private owned copy
            // (the immutable cached `Arc` must not be mutated, and the result must
            // exclude deleted base docids — matching the in-RAM `tok_postings`).
            let (docids_all, tfs_all) = cached.as_ref();
            let mut keep_ids = Vec::with_capacity(docids_all.len());
            let mut keep_tfs = Vec::with_capacity(tfs_all.len());
            for (&id, &tf) in docids_all.iter().zip(tfs_all.iter()) {
                if !self.tombstones.contains(id) {
                    keep_ids.push(id);
                    keep_tfs.push(tf);
                }
            }
            if keep_ids.is_empty() {
                return None;
            }
            return Some(TokPostings::SegmentFiltered {
                docids: keep_ids,
                tfs: keep_tfs,
            });
        }
        self.tokens.get(tok).map(TokPostings::Live)
    }

    /// The FULL token → postings map to seal, gathered from the ACTIVE source
    /// (Phase 2f-2 re-seal). On a FIRST seal (no segment) this is just a clone of
    /// the live `tokens`. On a RE-SEAL (a prior seal dropped `tokens` and put the
    /// base postings on the segment, while any docs indexed SINCE landed in the
    /// live `tokens` tail), the base postings are decoded back out of the segment
    /// (text tf is STORED, so this round-trips bit-identically) and MERGED with the
    /// live tail. Base and tail docids are disjoint (the tail's ids are all
    /// `>= the sealed n_docs`), so the per-token merge is a concatenation kept in
    /// ascending-docid order. The result covers EVERY current docid, so a re-seal
    /// after a seal+drop+tail re-materializes the whole field correctly.
    ///
    /// TOMBSTONE GC (Phase 2g-A): the prior-SEGMENT postings still hold a deleted
    /// base doc (the segment is immutable; `drop_eid` only touched the live
    /// `tokens`/`distinct`). `live(id)` is dropped-doc-aware (`false` for a base
    /// doc deleted from this field since the prior seal), so the merged base
    /// postings omit it. Empty postings (every docid of a token deleted) are
    /// dropped so the new segment never carries a zero-df token. The live tail's
    /// `tokens` already exclude deletes, so they are merged unconditionally.
    fn tokens_for_seal(&self, live: &dyn Fn(u32) -> bool) -> BTreeMap<String, Postings> {
        let Some(seg) = &self.segment else {
            return self
                .tokens
                .iter()
                .map(|(tok, p)| (tok.clone(), p.clone()))
                .collect();
        };
        let mut out: BTreeMap<String, Postings> = BTreeMap::new();
        if let Some(entries) = seg.text_tokens_all() {
            for (tok, docids, tfs) in entries {
                // Drop any docid no longer live (deleted from this field since the
                // prior seal) from the immutable base posting block.
                let mut kept_ids = Vec::with_capacity(docids.len());
                let mut kept_tfs = Vec::with_capacity(tfs.len());
                for (&id, &tf) in docids.iter().zip(tfs.iter()) {
                    if live(id) {
                        kept_ids.push(id);
                        kept_tfs.push(tf);
                    }
                }
                if !kept_ids.is_empty() {
                    out.insert(
                        tok,
                        Postings {
                            docids: kept_ids,
                            tfs: kept_tfs,
                        },
                    );
                }
            }
        }
        // Merge the live tail (docs indexed after the prior seal). Tail ids are
        // strictly greater than every base id, so appending keeps docids sorted.
        for (tok, p) in &self.tokens {
            let entry = out.entry(tok.clone()).or_default();
            for (&id, &tf) in p.docids.iter().zip(p.tfs.iter()) {
                entry.upsert(id, tf);
            }
        }
        out
    }

    /// The `(doc_count, total_doc_len)` BM25 corpus scalars to seal — the LIVE
    /// counters, which the index path increments on EVERY doc regardless of a
    /// sealed segment, so they are the FULL corpus (base + any post-seal tail).
    /// (The segment header's scalars are base-only and would undercount a re-seal,
    /// so they are deliberately NOT used here.) Phase 2f-2.
    fn corpus_for_seal(&self) -> (u64, u64) {
        (self.doc_count, self.total_doc_len)
    }

    /// The dense `u32[n_docs]` DocLen column to seal, covering EVERY current
    /// docid. Phase 2h-4 makes this SEGMENT-AWARE because the seal seam no longer
    /// keeps `lens` in RAM (it is dropped at seal — `doc_len()` reads the segment
    /// DocLen column for a sealed id): for a base id `< seg.n_docs` read the prior
    /// segment's DocLen column (`text_doc_len`); for a post-seal tail id read the
    /// live `self.lens`. Splitting the source is mandatory — the segment column is
    /// base-only (returns 0 for a tail id) and `self.lens` is now tail-only after a
    /// seal-and-drop (returns 0 for a base id), so each id MUST read from the side
    /// that holds it. On a FIRST seal (no segment) every id reads from `self.lens`,
    /// byte-identical to the old `self.lens` read.
    ///
    /// A NON-LIVE base id (deleted since the prior seal — its posting was
    /// tombstoned and its corpus length already decremented out of
    /// `corpus_for_seal`) MUST seal as DocLen 0, mirroring the in-RAM `drop_eid`
    /// (`set_doc_len(id, 0)`). The prior segment still carries the deleted doc's
    /// ORIGINAL nonzero length, so without the `live(id)` gate it would resurrect:
    /// (a) the new segment's DocLen column would re-introduce the deleted doc, and
    /// (b) `record_field_coverage` (which now reads `text_doc_len > 0` for presence)
    /// would mark it as having written the field. Zeroing it keeps the new segment
    /// consistent with `tokens_for_seal` (postings GC'd) and `corpus_for_seal`
    /// (scalars decremented).
    fn lens_for_seal(&self, n_docs: u32, live: &dyn Fn(u32) -> bool) -> Vec<u32> {
        match &self.segment {
            Some(seg) => (0..n_docs)
                .map(|id| {
                    if !live(id) {
                        0
                    } else if id < seg.n_docs() {
                        seg.text_doc_len(id)
                    } else {
                        self.lens.get(id as usize).copied().unwrap_or(0)
                    }
                })
                .collect(),
            None => (0..n_docs)
                .map(|id| {
                    if live(id) {
                        self.lens.get(id as usize).copied().unwrap_or(0)
                    } else {
                        0
                    }
                })
                .collect(),
        }
    }

    /// A token's LIVE document frequency (`df`) on the active source — the
    /// segment's stored posting length MINUS any tombstoned base docid when
    /// sealed, else the live posting length. Used by `estimate_selectivity`; 0 (⇒
    /// `None`) for a token absent or fully deleted. Phase 2e-B; tombstone-aware in
    /// 2h-4.
    ///
    /// With NO pending deletes (`tombstones` empty) this is the CHEAP count-prefix
    /// df (no posting decode) — the common path, unchanged from 2e-B. When a delete
    /// has tombstoned base docids it routes through `tok_postings` (which subtracts
    /// the tombstone and returns `None` for a fully-deleted token), so the `df`
    /// reflects only LIVE docs — consistent with the `df'` the BM25 `idf` uses and
    /// with an in-RAM oracle. (`estimate_selectivity` only orders clauses, but
    /// keeping `df` live avoids a stale over-count after a delete-after-seal.)
    #[inline]
    fn tok_df(&self, tok: &str) -> Option<usize> {
        if let Some(seg) = &self.segment {
            if self.tombstones.is_empty() {
                let df = seg.text_token_df(tok);
                return if df == 0 { None } else { Some(df) };
            }
            // Deletes pending: subtract the tombstone via the filtered posting.
            return self.tok_postings(tok).map(|p| p.df());
        }
        self.tokens.get(tok).map(|p| p.df())
    }

    /// Count of distinct tokens with >=1 LIVE doc on the ACTIVE source — the
    /// segment-aware `unique_terms` count for a sealed Text field (Phase 2h-4
    /// FIX), so it stays correct after a seal drops the in-RAM `tokens` driver.
    ///
    /// - segment OFF (no segment attached): `self.tokens.len()`
    ///   — byte-for-byte the old value (an in-RAM delete already removed an emptied
    ///   token from `tokens`).
    /// - segment ON: enumerate the segment token DICT (`text_tokens_all`); a token
    ///   counts iff >=1 of its base docids is NOT tombstoned. Then fold any live-tail
    ///   token (indexed after the seal into `tokens`) the dict did not carry. With NO
    ///   tombstones every dict token has a live posting, so this equals the dict size
    ///   + the disjoint tail — the same distinct-token count an un-dropped `tokens`
    ///   map would hold. A FULLY-deleted token drops out, matching the in-RAM
    ///   semantics where `drop_eid` removes an emptied token.
    fn live_unique_tokens(&self) -> u64 {
        let Some(seg) = &self.segment else {
            return self.tokens.len() as u64;
        };
        let Some(entries) = seg.text_tokens_all() else {
            // Torn segment: fall back to the live tail count (never panic in a
            // read-only stats accessor).
            return self.tokens.len() as u64;
        };
        let mut count = 0u64;
        for (tok, docids, _tfs) in &entries {
            let any_live = if self.tombstones.is_empty() {
                !docids.is_empty()
            } else {
                docids.iter().any(|id| !self.tombstones.contains(*id))
            };
            // A token present in BOTH the dict and the live tail is one distinct
            // token — count it once (here), and skip it in the tail fold below.
            if any_live || self.tokens.contains_key(tok) {
                count += 1;
            }
        }
        // Fold tail-only tokens (indexed after the seal, absent from the dict).
        let dict_tokens: std::collections::HashSet<&String> =
            entries.iter().map(|(t, _, _)| t).collect();
        for tok in self.tokens.keys() {
            if !dict_tokens.contains(tok) {
                count += 1;
            }
        }
        count
    }
}

#[derive(Debug, Default)]
struct KeywordIndex {
    /// term → docs (RoaringBitmap so AND/OR is compressed-SIMD, not random
    /// per-doc forward lookups — the difference at 1M-doc filter intersection).
    terms: BTreeMap<String, RoaringBitmap>,
    /// Side-index of LIVE-tail terms whose posting holds >= 2 docs, maintained
    /// O(log n) at index/delete time so `duplicates` iterates only candidate
    /// groups instead of scanning every distinct term. Tail-only state: cleared
    /// at seal (the sealed path enumerates the segment dict instead).
    dup_values: BTreeSet<String>,
    /// Dense live-tail forward cache for hot per-doc predicates and snapshots.
    /// `forward` remains a sparse compatibility/fallback map for restored older
    /// snapshots, while new writes avoid a per-doc HashMap insert.
    dense_forward: Vec<Option<String>>,
    forward: FastHashMap<u32, String>,
    bytes: u64,
    /// Stage 2 disk-tier (Phase 2e-A): a sealed columnar mmap segment covering
    /// doc ids `[0..n_docs)` — a sorted prefix-compressed string DICT plus a
    /// fixed `u32[n_docs]` dict-id forward column. When present, per-doc Keyword
    /// PREDICATE point lookups (`keyword_at`) read the segment for sealed ids
    /// and the in-RAM `forward` tail for ids `>= n_docs`. The inverted `terms`
    /// index is NOT stored on disk; the seal seam rebuilds it from the forward
    /// state so OR/AND posting walks are untouched. DEFAULTS to `None`; while it
    /// is `None` (nothing sealed) every read path is byte-for-byte the
    /// in-RAM path. Purely additive.
    segment: Option<std::sync::Arc<crate::segment::SegmentReader>>,
    /// QUERY-TIME TOMBSTONE (Phase 2h-1 FIX): base docids `[0..seg.n_docs)`
    /// deleted SINCE the last seal. The inverted `terms` index was DROPPED to
    /// disk at seal, so `drop_eid` can no longer remove a sealed base id from
    /// the immutable on-disk postings — instead it records the id here, and the
    /// segment-ON accessors (`term_postings`, duplicate/unique-term enumeration)
    /// SUBTRACT this set so a delete is reflected before the next re-seal. The
    /// next seal bakes the deletions into the new segment (its live(id) gather
    /// excludes them) and this is reset to empty. Live-tail ids (`>= seg.n_docs`)
    /// are NOT tombstoned — they are deleted directly out of the in-RAM `terms`
    /// tail. DEFAULTS empty; stays empty while no segment is attached (the in-RAM
    /// `terms` path needs no tombstone — a delete mutates `terms` directly).
    ///
    /// GENERALIZES to Set/Number/Text: each per-field index that drops its
    /// in-RAM inverted driver at seal (2h-2 SetIndex.elements, 2h-3
    /// NumberIndex.values, 2h-4 TextIndex.tokens) gains the identical
    /// `tombstones: RoaringBitmap`, records sealed-base deletes into it in
    /// `drop_eid`, subtracts it in the segment-ON branch of its posting accessor,
    /// and clears it at re-seal. Same shape, same four touch-points.
    tombstones: RoaringBitmap,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl KeywordIndex {
    /// The doc's keyword for a per-doc PREDICATE point lookup. When a sealed
    /// segment is attached it serves ids in its covered range `[0..n_docs)`
    /// (the live tail keeps ids `>= n_docs`); otherwise — and always when no segment
    /// is attached — this is exactly `self.forward.get(&id)`
    /// (cloned, since the segment path can only yield an owned `String`). The
    /// segment stores the exact UTF-8 bytes, so a hit equals the live `forward`
    /// entry; equality / membership compares are identical.
    #[inline]
    fn keyword_at(&self, id: u32) -> Option<String> {
        if let Some(seg) = &self.segment {
            if id < seg.n_docs() {
                if self.tombstones.contains(id) {
                    return None;
                }
                return seg.keyword_at(id);
            }
        }
        if let Some(value) = self.dense_forward.get(id as usize).and_then(|v| v.as_ref()) {
            return Some(value.clone());
        }
        self.forward.get(&id).cloned()
    }

    #[inline]
    fn set_keyword(&mut self, id: u32, value: String) {
        let ix = id as usize;
        if self.dense_forward.len() <= ix {
            self.dense_forward.resize_with(ix + 1, || None);
        }
        self.dense_forward[ix] = Some(value);
    }

    #[inline]
    fn remove_keyword(&mut self, id: u32) -> Option<String> {
        let dense = self
            .dense_forward
            .get_mut(id as usize)
            .and_then(|slot| slot.take());
        dense.or_else(|| self.forward.remove(&id))
    }

    fn forward_len(&self) -> usize {
        self.dense_forward.iter().filter(|v| v.is_some()).count() + self.forward.len()
    }

    /// `true` once the forward payload for the sealed range has been dropped to
    /// disk (Phase 2f-1): the segment is attached AND the in-RAM `forward` map
    /// no longer holds sealed ids. After a production seal, EVERY forward read
    /// (predicate, value retrieval, drop) must route through `keyword_at`, not a
    /// raw `forward.get`, because the payload is on the mmap, not in RAM.
    #[inline]
    fn forward_dropped(&self) -> bool {
        self.segment.is_some()
    }

    /// UNIFIED inverted-index accessor (Phase 2h-1): term `value`'s posting
    /// bitmap on the ACTIVE source, composing the disk segment (sealed base ids
    /// `[0..seg.n_docs)`) with the in-RAM `terms` tail (ids added after the
    /// seal, or — when no segment is attached — the WHOLE inverted index).
    ///
    /// - segment OFF (no segment attached): returns `Cow::Borrowed` of
    ///   `terms[value]` — ZERO clone, byte-for-byte the old `k.terms.get(value)`
    ///   path that every Term/Terms/selectivity/planner site used to read.
    /// - segment ON: decodes the segment's stored postings for `value` (the
    ///   RAM `terms` index was DROPPED at seal) and UNIONs any live-tail
    ///   postings the runtime indexed back into `terms` for the same value. The
    ///   union is `Cow::Owned`. Both bases are ascending-docid RoaringBitmaps,
    ///   so the result is identical to what an un-dropped in-RAM `terms` held.
    ///
    /// `None` only when the term is absent from BOTH sources (so callers keep
    /// their `.unwrap_or_default()` empty-posting semantics).
    #[inline]
    fn term_postings(&self, value: &str) -> Option<std::borrow::Cow<'_, RoaringBitmap>> {
        if let Some(seg) = &self.segment {
            // Segment base, MINUS the query-time tombstone (base docids deleted
            // since the last seal — the on-disk postings can't be mutated, so the
            // delete is applied here). Then UNION the live-tail postings indexed
            // back into `terms` after the seal. Tombstones only ever hold base ids
            // (`< seg.n_docs`) and tail ids are `>= seg.n_docs`, so the subtraction
            // never touches the tail.
            let mut base = seg.keyword_postings(value).unwrap_or_default();
            if !self.tombstones.is_empty() {
                base -= &self.tombstones;
            }
            if let Some(t) = self.terms.get(value) {
                base |= t;
            }
            // Empty result ⇒ `None`, matching the in-RAM semantics where a
            // fully-deleted term is removed from `terms` (Term on a value with no
            // live doc yields `None`, never an empty posting set).
            return if base.is_empty() {
                None
            } else {
                Some(std::borrow::Cow::Owned(base))
            };
        }
        self.terms.get(value).map(std::borrow::Cow::Borrowed)
    }

    /// UNIFIED document-frequency accessor (Phase 2h-1): term `value`'s `df` on
    /// the ACTIVE source — the boolean planner's rarest-first selectivity input.
    /// segment OFF: the live `terms[value]` length. segment ON: the segment's
    /// CHEAP count-prefix df (no posting decode) PLUS any live-tail length. The
    /// sum can only over-count if the same id were in both base and tail, which
    /// the seal+tail split forbids (tail ids are `>= seg.n_docs`), so it is the
    /// exact union cardinality. 0 when the term is absent everywhere.
    ///
    /// TOMBSTONE NOTE (Phase 2h-1 FIX): this deliberately does NOT subtract the
    /// query-time `tombstones` (a delete-after-seal would leave the count-prefix
    /// df slightly HIGH for the affected term). `df` only drives the boolean
    /// planner's rarest-first clause ORDERING — never the result set, which comes
    /// from `term_postings` (which DOES subtract the tombstone). A small df
    /// over-count can at worst pick a marginally less-rare lead clause; the
    /// emitted documents are identical. Keeping df off the full posting decode is
    /// the whole point of the count-prefix, so the over-count is accepted.
    #[inline]
    fn term_df(&self, value: &str) -> u64 {
        if let Some(seg) = &self.segment {
            let base = seg.keyword_df(value).unwrap_or(0);
            let tail = self.terms.get(value).map(|p| p.len()).unwrap_or(0);
            return base + tail;
        }
        self.terms.get(value).map(|p| p.len()).unwrap_or(0)
    }

    /// Every distinct term with its LIVE posting bitmap, on the ACTIVE source —
    /// the unified enumeration that `unique_terms` and `duplicates` drive from so
    /// they stay correct after a seal drops the in-RAM `terms` driver (Phase 2h-1
    /// FIX). Terms with no live doc are EXCLUDED (the same invariant the in-RAM
    /// `terms` map keeps — `drop_eid` removes an emptied term).
    ///
    /// - segment OFF: clones each live `terms` entry verbatim (the bitmaps are
    ///   already free of deleted ids — a live-path delete mutated them in place).
    /// - segment ON: enumerate the segment dict (`keyword_terms_all`), subtract
    ///   the query-time `tombstones` from each base posting, UNION the live-tail
    ///   `terms` for the same value, and keep the term iff >=1 live doc remains.
    ///   Tail-only terms (indexed after the seal, absent from the dict) are folded
    ///   in too. The result MATCHES the in-RAM `terms` snapshot on the same data.
    fn live_terms(&self) -> BTreeMap<String, RoaringBitmap> {
        let Some(seg) = &self.segment else {
            return self.terms.clone();
        };
        let mut out: BTreeMap<String, RoaringBitmap> = BTreeMap::new();
        if let Some(dict) = seg.keyword_terms_all() {
            for (term, mut postings) in dict {
                if !self.tombstones.is_empty() {
                    postings -= &self.tombstones;
                }
                if let Some(tail) = self.terms.get(&term) {
                    postings |= tail;
                }
                if !postings.is_empty() {
                    out.insert(term, postings);
                }
            }
        }
        // Fold any live-tail-only terms the dict did not carry (indexed after the
        // seal). A term already present from the dict is left as the merged set.
        for (term, tail) in &self.terms {
            if !out.contains_key(term) {
                out.insert(term.clone(), tail.clone());
            }
        }
        out
    }
}

/// Values whose live posting holds >= 2 docs — seeds the duplicates side-index
/// (`dup_values`) when an inverted map is rebuilt wholesale (snapshot restore);
/// the write/delete paths maintain it incrementally afterwards.
fn dup_values_of<K: Ord + Clone>(map: &BTreeMap<K, RoaringBitmap>) -> BTreeSet<K> {
    map.iter()
        .filter(|(_, set)| set.len() >= 2)
        .map(|(k, _)| k.clone())
        .collect()
}

/// Order-statistic snapshot of the live `values` tree: distinct value bits
/// ascending plus cumulative doc counts. Built lazily the first time a range
/// ESTIMATE walks past [`RANGE_STATS_BUILD_THRESHOLD`] distinct keys; dropped
/// with the keyword range caches on any write batch, delete, or seal. Both
/// `range_df` and `range_distinct_count` answer from two binary searches —
/// O(log distinct) instead of an O(distinct-in-range) tree walk per query.
/// Estimation-only state: it never feeds result evaluation, so a rebuild race
/// can at worst pick a different (still correct) plan.
#[derive(Debug)]
struct NumberRangeStats {
    /// Distinct live values (sortable bits), ascending.
    keys: Vec<u64>,
    /// `cum_df[i]` = total docs across `keys[..i]`; `len = keys.len() + 1`.
    cum_df: Vec<u64>,
}

/// Distinct-key walk budget for a single range estimate before the walk is
/// abandoned and [`NumberRangeStats`] is built instead. Narrow ranges stay on
/// the exact walk and never pay the build; one wide estimate pays roughly two
/// walks (the abandoned prefix + the build) and every later estimate on the
/// unchanged tree is O(log distinct).
const RANGE_STATS_BUILD_THRESHOLD: u64 = 1024;

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl NumberRangeStats {
    fn build(values: &BTreeMap<SortableF64, RoaringBitmap>) -> Self {
        let mut keys = Vec::with_capacity(values.len());
        let mut cum_df = Vec::with_capacity(values.len() + 1);
        cum_df.push(0);
        let mut running = 0u64;
        for (k, set) in values {
            keys.push(k.bits());
            running += set.len();
            cum_df.push(running);
        }
        Self { keys, cum_df }
    }

    /// `(distinct, df)` over the half-open index window the bounds select.
    fn range(
        &self,
        low: std::ops::Bound<SortableF64>,
        high: std::ops::Bound<SortableF64>,
    ) -> (u64, u64) {
        use std::ops::Bound;
        let lo_ix = match low {
            Bound::Unbounded => 0,
            Bound::Included(x) => self.keys.partition_point(|&k| k < x.bits()),
            Bound::Excluded(x) => self.keys.partition_point(|&k| k <= x.bits()),
        };
        let hi_ix = match high {
            Bound::Unbounded => self.keys.len(),
            Bound::Included(x) => self.keys.partition_point(|&k| k <= x.bits()),
            Bound::Excluded(x) => self.keys.partition_point(|&k| k < x.bits()),
        };
        if hi_ix <= lo_ix {
            return (0, 0);
        }
        (
            (hi_ix - lo_ix) as u64,
            self.cum_df[hi_ix] - self.cum_df[lo_ix],
        )
    }
}

#[derive(Debug, Default)]
struct NumberIndex {
    values: BTreeMap<SortableF64, RoaringBitmap>,
    /// Side-index of LIVE-tail values whose posting holds >= 2 docs (see
    /// `KeywordIndex::dup_values`); drives `duplicates` without a full
    /// `values` scan. Cleared at seal.
    dup_values: BTreeSet<SortableF64>,
    forward: FastHashMap<u32, SortableF64>,
    /// Dense live-tail forward cache for hot per-doc predicates. `forward`
    /// remains the sparse ownership/snapshot map; this avoids HashMap lookup in
    /// bitmap-driven numeric filters on large in-memory indexes.
    dense_forward: Vec<u64>,
    /// Per `(keyword field, term)` numeric skip lists. Each cached term stores its
    /// posting docids sorted by this number field's sortable bits, so
    /// `Term(s)(keyword) ∩ Range(number)` counts by binary-searching one small list
    /// instead of random-probing the numeric forward column. This is deliberately
    /// lazy per term, not whole-field, so the segment tier does not materialize a
    /// high-cardinality keyword field into RAM just because one hot term was read.
    keyword_range_cache: RwLock<FastHashMap<String, std::sync::Arc<Vec<(u64, u32)>>>>,
    /// Per `(keyword field, term, numeric range)` candidate bitmaps for hot
    /// `Match(text) ∩ Term(keyword) ∩ Range(number)` shapes. This is query-only
    /// derived state, invalidated with `keyword_range_cache`.
    keyword_range_bitmap_cache: RwLock<FastHashMap<String, std::sync::Arc<RoaringBitmap>>>,
    /// Lazy [`NumberRangeStats`] over the live `values` tree for planner range
    /// ESTIMATES (`range_df` / `range_distinct_count`). Query-only derived
    /// state, invalidated with `keyword_range_cache`.
    range_stats: RwLock<Option<std::sync::Arc<NumberRangeStats>>>,
    bytes: u64,
    /// Stage 2 disk-tier (Phase 2c): a sealed columnar mmap segment covering
    /// doc ids `[0..n_docs)`. When present, per-doc Number PREDICATE point
    /// lookups (`number_at`) read the segment for sealed ids and the in-RAM
    /// `forward` tail for ids `>= n_docs`. DEFAULTS to `None`; while it is
    /// `None` (nothing sealed) every
    /// read path is byte-for-byte the in-RAM path. Purely additive: the write
    /// path, `values` range walk, and inverted index are untouched.
    ///
    /// Phase 2h-3: the segment now ALSO carries the SORTED-VALUE range index
    /// (`ROLE_NUMBER_SORTED` + `ROLE_NUMBER_POSTINGS`), so range / exact /
    /// boolean queries drive from the mmap (`value_postings` / `range_postings`)
    /// and the in-RAM `values` BTreeMap is DROPPED at seal (RAM after reopen is
    /// O(live tail), not O(distinct numeric values)).
    segment: Option<std::sync::Arc<crate::segment::SegmentReader>>,
    /// QUERY-TIME TOMBSTONE (Phase 2h-3): base docids `[0..seg.n_docs)` deleted
    /// SINCE the last seal. The inverted/range `values` index was DROPPED to disk
    /// at seal, so `drop_eid` can no longer remove a sealed base id from the
    /// immutable on-disk postings — instead it records the id here, and the
    /// segment-ON accessors (`value_postings`, `range_postings`,
    /// duplicate / unique-value enumeration) SUBTRACT this set so a delete is
    /// reflected before the next re-seal. The next seal bakes the deletions into
    /// the new segment (its `live(id)` gather excludes them) and this is reset to
    /// empty. Live-tail ids (`>= seg.n_docs`) are NOT tombstoned — they are
    /// deleted directly out of the in-RAM `values` tail. DEFAULTS empty; stays
    /// empty while no segment is attached. The exact reuse of the Keyword 2h-1 /
    /// Set 2h-2 tombstone (same shape, same four touch-points: record in
    /// `drop_eid`, subtract in the posting accessors, exclude in `live_values`,
    /// clear at re-seal).
    tombstones: RoaringBitmap,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl NumberIndex {
    /// The doc's value for a per-doc PREDICATE point lookup. When a sealed
    /// segment is attached it serves ids in its covered range `[0..n_docs)`
    /// (the live tail keeps ids `>= n_docs`); otherwise — and always when the
    /// no segment is attached — this is exactly the live forward value, served
    /// through a dense cache before falling back to the sparse map.
    ///
    /// The segment stores raw `f64` bits, so a hit is re-wrapped through
    /// `SortableF64::new`; that is the same order-preserving transform the live
    /// `forward` entry already holds, so `in_range` / equality compare
    /// identically. A NaN can never reach the index (rejected at index time),
    /// so `SortableF64::new` cannot fail here; if it ever did we fall back to
    /// the live map rather than panic.
    #[inline]
    fn number_at(&self, id: u32) -> Option<SortableF64> {
        self.number_bits_at(id).map(SortableF64::from_bits)
    }

    #[inline]
    fn live_number_at(&self, id: u32) -> Option<SortableF64> {
        if let Some(seg) = &self.segment {
            if id < seg.n_docs() && self.tombstones.contains(id) {
                return None;
            }
        }
        self.number_at(id)
    }

    #[inline]
    fn number_bits_at(&self, id: u32) -> Option<u64> {
        if let Some(seg) = &self.segment {
            if id < seg.n_docs() {
                return seg
                    .number_at(id)
                    .and_then(|x| SortableF64::new(x).ok())
                    .map(|s| s.bits());
            }
        }
        if let Some(bits) = self.dense_forward.get(id as usize).copied() {
            if bits != MISSING_SORTABLE_F64_BITS {
                return Some(bits);
            }
        }
        self.forward.get(&id).map(|s| s.bits())
    }

    #[inline]
    fn number_in_bounds(
        &self,
        id: u32,
        lo: &std::ops::Bound<SortableF64>,
        hi: &std::ops::Bound<SortableF64>,
    ) -> bool {
        self.number_bits_at(id)
            .is_some_and(|bits| in_sortable_bits_range(bits, lo, hi))
    }

    #[inline]
    fn set_number(&mut self, id: u32, key: SortableF64) {
        // Callers clear keyword range caches once before a write batch; doing it
        // per numeric item dominated bulk ingest.
        let ix = id as usize;
        if self.dense_forward.len() <= ix {
            self.dense_forward.resize(ix + 1, MISSING_SORTABLE_F64_BITS);
        }
        self.dense_forward[ix] = key.bits();
    }

    #[inline]
    fn remove_number(&mut self, id: u32) -> Option<SortableF64> {
        let dense = self.dense_forward.get_mut(id as usize).and_then(|slot| {
            if *slot == MISSING_SORTABLE_F64_BITS {
                None
            } else {
                let bits = *slot;
                *slot = MISSING_SORTABLE_F64_BITS;
                Some(SortableF64::from_bits(bits))
            }
        });
        let sparse = self.forward.remove(&id);
        self.clear_keyword_range_cache();
        dense.or(sparse)
    }

    #[inline]
    fn clear_keyword_range_cache(&mut self) {
        if let Ok(cache) = self.keyword_range_cache.get_mut() {
            cache.clear();
        }
        if let Ok(cache) = self.keyword_range_bitmap_cache.get_mut() {
            cache.clear();
        }
        if let Ok(stats) = self.range_stats.get_mut() {
            *stats = None;
        }
    }

    /// `(distinct, df)` of the LIVE `values` tree in `[low, high]` for planner
    /// estimates. Walks the tree exactly for narrow ranges; a walk that passes
    /// [`RANGE_STATS_BUILD_THRESHOLD`] distinct keys abandons and answers from
    /// the (built-on-demand) [`NumberRangeStats`] snapshot instead.
    fn live_range_estimate(
        &self,
        low: std::ops::Bound<SortableF64>,
        high: std::ops::Bound<SortableF64>,
    ) -> (u64, u64) {
        if let Some(stats) = self.range_stats.read().ok().and_then(|guard| guard.clone()) {
            return stats.range(low, high);
        }
        let mut distinct = 0u64;
        let mut df = 0u64;
        for (_, set) in self.values.range((low, high)) {
            distinct += 1;
            if distinct > RANGE_STATS_BUILD_THRESHOLD {
                return self.build_range_stats().range(low, high);
            }
            df += set.len();
        }
        (distinct, df)
    }

    fn build_range_stats(&self) -> std::sync::Arc<NumberRangeStats> {
        let built = std::sync::Arc::new(NumberRangeStats::build(&self.values));
        if let Ok(mut guard) = self.range_stats.write() {
            // A concurrent estimate may have built it first; either snapshot is
            // valid for the same (read-locked, unmutated) tree — keep ours.
            *guard = Some(built.clone());
        }
        built
    }

    fn forward_len(&self) -> usize {
        self.dense_forward
            .iter()
            .filter(|bits| **bits != MISSING_SORTABLE_F64_BITS)
            .count()
            + self.forward.len()
    }

    fn keyword_range_docs(
        &self,
        keyword_field: &str,
        term: &str,
        keyword: &KeywordIndex,
    ) -> std::sync::Arc<Vec<(u64, u32)>> {
        let cache_key = format!("{keyword_field}\0{term}");
        if let Some(cached) = self
            .keyword_range_cache
            .read()
            .expect("number keyword-range cache poisoned")
            .get(&cache_key)
            .cloned()
        {
            return cached;
        }

        let mut docs = Vec::new();
        if let Some(posting) = keyword.term_postings(term) {
            docs.reserve(posting.len() as usize);
            for id in posting.as_ref() {
                if let Some(bits) = self.number_bits_at(id) {
                    docs.push((bits, id));
                }
            }
        }
        docs.sort_unstable_by_key(|(bits, id)| (*bits, *id));
        let built = std::sync::Arc::new(docs);

        let mut cache = self
            .keyword_range_cache
            .write()
            .expect("number keyword-range cache poisoned");
        if let Some(cached) = cache.get(&cache_key) {
            return cached.clone();
        }
        cache.insert(cache_key, built.clone());
        built
    }

    fn keyword_range_bitmap(
        &self,
        keyword_field: &str,
        term: &str,
        keyword: &KeywordIndex,
        lo: &std::ops::Bound<SortableF64>,
        hi: &std::ops::Bound<SortableF64>,
    ) -> std::sync::Arc<RoaringBitmap> {
        let cache_key = format!("{}\0{}\0{}", keyword_field, term, range_cache_key(lo, hi));
        if let Some(cached) = self
            .keyword_range_bitmap_cache
            .read()
            .expect("number keyword-range bitmap cache poisoned")
            .get(&cache_key)
            .cloned()
        {
            return cached;
        }

        let docs = self.keyword_range_docs(keyword_field, term, keyword);
        let window = sorted_bits_window(docs.as_slice(), lo, hi);
        let mut bitmap = RoaringBitmap::new();
        for &(_, id) in &docs[window] {
            bitmap.insert(id);
        }
        let built = std::sync::Arc::new(bitmap);

        let mut cache = self
            .keyword_range_bitmap_cache
            .write()
            .expect("number keyword-range bitmap cache poisoned");
        if let Some(cached) = cache.get(&cache_key) {
            return cached.clone();
        }
        cache.insert(cache_key, built.clone());
        built
    }

    /// `true` once the sealed forward payload has been dropped to disk
    /// (Phase 2f-1) — the segment is attached. See `KeywordIndex::forward_dropped`.
    #[inline]
    fn forward_dropped(&self) -> bool {
        self.segment.is_some()
    }

    /// The attached disk segment, if any — the entry the SORT planner
    /// (`try_plan`) uses to drive `sorted_walk_segment` (Phase 2m). `None` when no
    /// segment is attached (the in-RAM `values` BTreeMap is the sort driver).
    #[inline]
    fn segment_ref(&self) -> Option<&std::sync::Arc<crate::segment::SegmentReader>> {
        self.segment.as_ref()
    }

    /// UNIFIED EXACT-MATCH accessor (Phase 2h-3): the posting bitmap for the
    /// exact numeric value `key` on the ACTIVE source, composing the disk segment
    /// (sealed base ids `[0..seg.n_docs)`) with the in-RAM `values` tail (ids
    /// added after the seal, or — when no segment is attached — the WHOLE index).
    /// The Number analogue of `KeywordIndex::term_postings`.
    ///
    /// - segment OFF (no segment attached): returns `Cow::Borrowed` of
    ///   `values[key]` — ZERO clone, byte-for-byte the old `n.values.get(&key)`
    ///   path every exact-match / selectivity / planner site read.
    /// - segment ON: the segment's stored postings for `key` (the RAM `values`
    ///   index was DROPPED at seal), MINUS the query-time `tombstones`, UNION any
    ///   live-tail `values[key]`. The union is `Cow::Owned`. Both bases are
    ///   ascending-docid RoaringBitmaps, so the result is identical to what an
    ///   un-dropped in-RAM `values` held.
    ///
    /// `None` only when the value is absent from BOTH sources (callers keep their
    /// `.unwrap_or_default()` empty-posting semantics).
    #[inline]
    fn value_postings(&self, key: SortableF64) -> Option<std::borrow::Cow<'_, RoaringBitmap>> {
        if let Some(seg) = &self.segment {
            // Segment base, MINUS the query-time tombstone (base docids deleted
            // since the last seal — the on-disk postings can't be mutated), then
            // UNION the live-tail postings indexed back into `values`. Tombstones
            // only ever hold base ids (`< seg.n_docs`) and tail ids are
            // `>= seg.n_docs`, so the subtraction never touches the tail.
            let mut base = seg.number_value_postings(key.bits()).unwrap_or_default();
            if !self.tombstones.is_empty() {
                base -= &self.tombstones;
            }
            if let Some(t) = self.values.get(&key) {
                base |= t;
            }
            return if base.is_empty() {
                None
            } else {
                Some(std::borrow::Cow::Owned(base))
            };
        }
        self.values.get(&key).map(std::borrow::Cow::Borrowed)
    }

    /// UNIFIED RANGE accessor (Phase 2h-3): the posting union of every value in
    /// the half-open/inclusive range `(low, high)` on the ACTIVE source. The
    /// Number analogue of a range-walk over `values`.
    ///
    /// - segment OFF (no segment attached): walks `values.range((low,
    ///   high))` and ORs each posting — byte-for-byte the old `eval_range` walk.
    /// - segment ON: the segment's `number_range` (binary-search to the lo/hi
    ///   index bounds, union the in-window postings — SELECTIVE, not a forward
    ///   scan), MINUS the query-time `tombstones`, UNION the live-tail
    ///   `values.range((low, high))` postings. Byte-identical result set to the
    ///   in-RAM range walk over the same data.
    #[inline]
    fn range_postings(
        &self,
        low: std::ops::Bound<SortableF64>,
        high: std::ops::Bound<SortableF64>,
    ) -> RoaringBitmap {
        // An empty/inverted range yields no docs on BOTH paths — and short-circuits
        // before `values.range`, which PANICS on an inverted/degenerate-exclusive
        // pair. The on-disk `number_range_window` already collapses such a range to
        // an empty window, so this keeps the two paths identical.
        if range_is_empty(low, high) {
            return RoaringBitmap::new();
        }
        if let Some(seg) = &self.segment {
            let mut acc = seg
                .number_range(bound_to_bits(low), bound_to_bits(high))
                .unwrap_or_default();
            if !self.tombstones.is_empty() {
                acc -= &self.tombstones;
            }
            // UNION the live tail (ids added after the seal). Tail ids are
            // `>= seg.n_docs`, disjoint from the segment base, so the OR never
            // double-counts.
            for (_, set) in self.values.range((low, high)) {
                acc |= set;
            }
            return acc;
        }
        let mut acc = RoaringBitmap::new();
        for (_, set) in self.values.range((low, high)) {
            acc |= set;
        }
        acc
    }

    /// UNIFIED exact-match document frequency (Phase 2h-3): value `key`'s `df` on
    /// the ACTIVE source — the boolean planner's rarest-first selectivity input.
    /// segment OFF: the live `values[key]` length. segment ON: the segment's
    /// CHEAP count-prefix df (no posting decode) PLUS any live-tail length. Like
    /// `KeywordIndex::term_df`, this deliberately does NOT subtract the tombstone
    /// (a small over-count only affects clause ORDERING, never the result set,
    /// which comes from `value_postings`).
    #[inline]
    fn value_df(&self, key: SortableF64) -> u64 {
        if let Some(seg) = &self.segment {
            let base = seg.number_value_df(key.bits()).unwrap_or(0);
            let tail = self.values.get(&key).map(|p| p.len()).unwrap_or(0);
            return base + tail;
        }
        self.values.get(&key).map(|p| p.len()).unwrap_or(0)
    }

    /// UNIFIED range selectivity (Phase 2h-3): the summed df of every value in
    /// `(low, high)` on the ACTIVE source. segment OFF: sum of the live
    /// `values.range` posting lengths. segment ON: the segment's cheap
    /// count-prefix range df PLUS the live-tail range lengths. Like `value_df`,
    /// does NOT subtract the tombstone (ordering-only input).
    #[inline]
    fn range_df(
        &self,
        low: std::ops::Bound<SortableF64>,
        high: std::ops::Bound<SortableF64>,
    ) -> u64 {
        // Guard the empty/inverted range (would panic `values.range`); 0 on both.
        if range_is_empty(low, high) {
            return 0;
        }
        let (_, tail) = self.live_range_estimate(low, high);
        if let Some(seg) = &self.segment {
            let base = seg
                .number_range_df(bound_to_bits(low), bound_to_bits(high))
                .unwrap_or(0);
            return base + tail;
        }
        tail
    }

    #[inline]
    fn range_distinct_count(
        &self,
        low: std::ops::Bound<SortableF64>,
        high: std::ops::Bound<SortableF64>,
    ) -> u64 {
        if range_is_empty(low, high) {
            return 0;
        }
        let (tail, _) = self.live_range_estimate(low, high);
        if let Some(seg) = &self.segment {
            return seg
                .number_range_distinct_count(bound_to_bits(low), bound_to_bits(high))
                .unwrap_or(0)
                + tail;
        }
        tail
    }

    /// Every distinct value (as a `SortableF64` key) with its LIVE posting
    /// bitmap, on the ACTIVE source, ASCENDING by value — the unified enumeration
    /// that `unique_terms` and the sort/range planner pages drive from so they
    /// stay correct after a seal drops the in-RAM `values` driver (Phase 2h-3).
    /// Values with no live doc are EXCLUDED (the same invariant the in-RAM
    /// `values` map keeps). The Number analogue of `KeywordIndex::live_terms`.
    ///
    /// - segment OFF: clones the live `values` map verbatim.
    /// - segment ON: enumerate the segment sorted-value column
    ///   (`number_values_all`), subtract the query-time `tombstones` from each
    ///   base posting, UNION the live-tail `values` for the same key, keep the
    ///   value iff >=1 live doc remains. Tail-only values (indexed after the seal)
    ///   are folded in too. Returns a `BTreeMap`, so iteration is ascending by
    ///   value — identical order to the in-RAM `values`.
    fn live_values(&self) -> BTreeMap<SortableF64, RoaringBitmap> {
        let Some(seg) = &self.segment else {
            return self.values.clone();
        };
        let mut out: BTreeMap<SortableF64, RoaringBitmap> = BTreeMap::new();
        if let Some(dict) = seg.number_values_all() {
            for (bits, mut postings) in dict {
                if !self.tombstones.is_empty() {
                    postings -= &self.tombstones;
                }
                let key = SortableF64::from_bits(bits);
                if let Some(tail) = self.values.get(&key) {
                    postings |= tail;
                }
                if !postings.is_empty() {
                    out.insert(key, postings);
                }
            }
        }
        // Fold any live-tail-only values the sorted column did not carry (indexed
        // after the seal). A value already present from the column is left merged.
        for (key, tail) in &self.values {
            if !out.contains_key(key) {
                out.insert(*key, tail.clone());
            }
        }
        out
    }

    /// The ascending-by-value distinct-value map for the SORT / standalone-range
    /// PLANNER walks (`try_plan`), as a `Cow` so the default path stays
    /// zero-clone (Phase 2h-3):
    ///
    /// - segment OFF (no segment attached): `Cow::Borrowed(&self.values)` —
    ///   byte-for-byte the old `n.values.iter()` / `.range()` zero-clone walk.
    /// - segment ON: `Cow::Owned(self.live_values())` — the on-disk sorted-value
    ///   column merged with the live tail (minus tombstones), since the in-RAM
    ///   `values` driver was DROPPED at seal. Iterating it is ascending by value,
    ///   identical order to the in-RAM map on the same data.
    #[inline]
    fn sorted_values(&self) -> std::borrow::Cow<'_, BTreeMap<SortableF64, RoaringBitmap>> {
        if self.segment.is_some() {
            return std::borrow::Cow::Owned(self.live_values());
        }
        std::borrow::Cow::Borrowed(&self.values)
    }

    /// SEGMENT-ON sort-via-sorted-index walk (Phase 2m). Drives the SORT planner
    /// (`try_plan`) directly off the on-disk ascending sorted-value index
    /// (`ROLE_NUMBER_SORTED`) + the BOUNDED per-value posting cache, MERGED with
    /// the live tail (`self.values`) in value order — the disk analogue of walking
    /// the in-RAM `values` BTreeMap, WITHOUT `live_values()`'s whole-field
    /// BTreeMap rebuild (the 525x sort regression) and WITHOUT the
    /// gather-`number_at`-per-doc + sort the original sealed path paid.
    ///
    /// For each distinct value in the requested order (`asc`/`desc`), the per-value
    /// posting is the segment posting MINUS the query-time `tombstones` (deleted
    /// base docids — `query_predicate` can't exclude them because `number_at` still
    /// reads the immutable forward column), UNION the live-tail `values[key]`. The
    /// `visit` callback receives `(value_as_f64, docid)` in:
    ///   * value order = requested sort order (ascending bits == ascending numeric
    ///     value, identical to the in-RAM `values` `Ord`; reversed for desc), and
    ///   * within-value docid order = ascending (base ids `< seg.n_docs` precede
    ///     tail ids, exactly the OR'd-bitmap iteration order the in-RAM walk used),
    /// so the emitted (value, docid) sequence is BYTE-IDENTICAL to the in-RAM
    /// `sorted_values().iter()[.rev()]` walk. `visit` returns `Ok(true)` to
    /// continue or `Ok(false)` to stop early (page full + `!track_total`); the walk
    /// short-circuits on `Ok(false)` so `pure_sort`/`filter_sort` only touch (and
    /// cache) the first few values' postings. Returns `Ok(())` once every value is
    /// visited or the callback stops.
    ///
    /// Both sources are ascending; a value present in BOTH the segment column and
    /// the live tail is visited ONCE with the unioned posting (the merge advances
    /// both cursors on a key tie), matching the in-RAM single-entry-per-value map.
    fn sorted_walk_segment<F>(
        &self,
        seg: &crate::segment::SegmentReader,
        descending: bool,
        after: Option<u64>,
        mut visit: F,
    ) -> Result<()>
    where
        F: FnMut(f64, u32) -> Result<bool>,
    {
        let distinct = seg.number_distinct_count();
        // Tail keys ascending (BTreeMap order). We walk the segment index and the
        // tail in lockstep, emitting the smaller (asc) / larger (desc) key first so
        // the merged order is monotone and each value is visited once.
        let tail_keys: Vec<SortableF64> = self.values.keys().copied().collect();

        // Keyset seek (Phase 2p): start both cursors AT the cursor key (the
        // caller's visitor still drops the equal-key docids at or before the
        // cursor docid). Binary search over the index-addressable sorted
        // column — O(log distinct) probes, no posting decode.
        let seek_seg = |want_bits: u64| -> u64 {
            let (mut lo, mut hi) = (0u64, distinct);
            while lo < hi {
                let mid = (lo + hi) / 2;
                match seg.number_sorted_bits_at(mid as u32) {
                    Some(bits) if bits < want_bits => lo = mid + 1,
                    _ => hi = mid,
                }
            }
            lo
        };

        // One unit of work for a single distinct VALUE: compose its segment posting
        // (minus tombstones) with the live-tail posting, then emit each docid in
        // ascending order. Returns Ok(false) to stop the whole walk early.
        let mut emit_value = |value_bits: u64,
                              seg_posting: Option<std::sync::Arc<RoaringBitmap>>,
                              tail_posting: Option<&RoaringBitmap>|
         -> Result<bool> {
            let value = SortableF64::from_bits(value_bits).to_f64();
            // Build the RAW union exactly as the in-RAM merged bitmap held it:
            // segment base MINUS tombstones, OR the live tail. Cheap when there is
            // no segment posting (tail-only value) or no tombstone (the common
            // case clones nothing for the tail-only branch).
            match (seg_posting, tail_posting) {
                (Some(base), tail) => {
                    let mut set = (*base).clone();
                    if !self.tombstones.is_empty() {
                        set -= &self.tombstones;
                    }
                    if let Some(t) = tail {
                        set |= t;
                    }
                    for id in &set {
                        if !visit(value, id)? {
                            return Ok(false);
                        }
                    }
                }
                (None, Some(tail)) => {
                    // Tail-only value: tail ids are never tombstoned (a live-path
                    // delete mutates the tail bitmap in place), so emit directly.
                    for id in tail {
                        if !visit(value, id)? {
                            return Ok(false);
                        }
                    }
                }
                (None, None) => {}
            }
            Ok(true)
        };

        if !descending {
            // ----- ASCENDING: two ascending cursors (segment index 0.., tail 0..).
            let mut si: u64 = 0;
            let mut ti: usize = 0;
            if let Some(bits) = after {
                si = seek_seg(bits);
                ti = tail_keys.partition_point(|k| k.bits() < bits);
            }
            loop {
                let seg_bits = if si < distinct {
                    seg.number_sorted_bits_at(si as u32)
                } else {
                    None
                };
                let tail_key = tail_keys.get(ti).copied();
                match (seg_bits, tail_key) {
                    (None, None) => break,
                    (Some(sb), None) => {
                        let p = seg.number_sorted_postings_at(si as u32);
                        if !emit_value(sb, p, None)? {
                            return Ok(());
                        }
                        si += 1;
                    }
                    (None, Some(tk)) => {
                        let t = self.values.get(&tk);
                        if !emit_value(tk.bits(), None, t)? {
                            return Ok(());
                        }
                        ti += 1;
                    }
                    (Some(sb), Some(tk)) => {
                        let tb = tk.bits();
                        if sb < tb {
                            let p = seg.number_sorted_postings_at(si as u32);
                            if !emit_value(sb, p, None)? {
                                return Ok(());
                            }
                            si += 1;
                        } else if sb > tb {
                            let t = self.values.get(&tk);
                            if !emit_value(tb, None, t)? {
                                return Ok(());
                            }
                            ti += 1;
                        } else {
                            // Same value in both sources → visit once, unioned.
                            let p = seg.number_sorted_postings_at(si as u32);
                            let t = self.values.get(&tk);
                            if !emit_value(sb, p, t)? {
                                return Ok(());
                            }
                            si += 1;
                            ti += 1;
                        }
                    }
                }
            }
        } else {
            // ----- DESCENDING: two descending cursors (segment index high..0, tail
            // high..0). Within-value docid order stays ASCENDING (matches the in-RAM
            // `values.iter().rev()` then ascending-bitmap iteration).
            let mut si: i64 = distinct as i64 - 1;
            let mut ti: i64 = tail_keys.len() as i64 - 1;
            if let Some(bits) = after {
                // Last index with bits <= cursor (first strictly-greater minus 1).
                let (mut lo, mut hi) = (0u64, distinct);
                while lo < hi {
                    let mid = (lo + hi) / 2;
                    match seg.number_sorted_bits_at(mid as u32) {
                        Some(b) if b <= bits => lo = mid + 1,
                        _ => hi = mid,
                    }
                }
                si = lo as i64 - 1;
                ti = tail_keys.partition_point(|k| k.bits() <= bits) as i64 - 1;
            }
            loop {
                let seg_bits = if si >= 0 {
                    seg.number_sorted_bits_at(si as u32)
                } else {
                    None
                };
                let tail_key = if ti >= 0 {
                    tail_keys.get(ti as usize).copied()
                } else {
                    None
                };
                match (seg_bits, tail_key) {
                    (None, None) => break,
                    (Some(sb), None) => {
                        let p = seg.number_sorted_postings_at(si as u32);
                        if !emit_value(sb, p, None)? {
                            return Ok(());
                        }
                        si -= 1;
                    }
                    (None, Some(tk)) => {
                        let t = self.values.get(&tk);
                        if !emit_value(tk.bits(), None, t)? {
                            return Ok(());
                        }
                        ti -= 1;
                    }
                    (Some(sb), Some(tk)) => {
                        let tb = tk.bits();
                        if sb > tb {
                            let p = seg.number_sorted_postings_at(si as u32);
                            if !emit_value(sb, p, None)? {
                                return Ok(());
                            }
                            si -= 1;
                        } else if sb < tb {
                            let t = self.values.get(&tk);
                            if !emit_value(tb, None, t)? {
                                return Ok(());
                            }
                            ti -= 1;
                        } else {
                            let p = seg.number_sorted_postings_at(si as u32);
                            let t = self.values.get(&tk);
                            if !emit_value(sb, p, t)? {
                                return Ok(());
                            }
                            si -= 1;
                            ti -= 1;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// SEGMENT-ON standalone range page. Unlike `sorted_values()`, this streams only
    /// the selected sorted-value window and never materializes the whole field into a
    /// BTreeMap. Exact total is counted from posting lengths whenever possible; docid
    /// iteration is only needed while filling the page, or when tombstones force a
    /// live count for a base posting.
    fn range_page_segment(
        &self,
        seg: &crate::segment::SegmentReader,
        low: std::ops::Bound<SortableF64>,
        high: std::ops::Bound<SortableF64>,
        want: usize,
        track_total: bool,
    ) -> Result<(Vec<(u32, f32)>, u64)> {
        let mut page = Vec::with_capacity(want.min(1024));
        let mut total = 0u64;
        let Some((i_lo, i_hi)) =
            seg.number_range_index_window(bound_to_bits(low), bound_to_bits(high))
        else {
            return Ok((page, total));
        };
        let tail_keys: Vec<SortableF64> = self.values.range((low, high)).map(|(k, _)| *k).collect();
        let mut si = i_lo;
        let mut ti = 0usize;

        let mut emit_value = |_value_bits: u64,
                              seg_posting: Option<std::sync::Arc<RoaringBitmap>>,
                              tail_posting: Option<&RoaringBitmap>|
         -> Result<bool> {
            if let Some(base) = seg_posting {
                if self.tombstones.is_empty() {
                    total += base.len();
                    if page.len() < want {
                        for id in base.iter() {
                            page.push((id, 1.0));
                            if page.len() >= want {
                                break;
                            }
                        }
                    } else if !track_total {
                        return Ok(false);
                    }
                } else {
                    for id in base.iter() {
                        if self.tombstones.contains(id) {
                            continue;
                        }
                        total += 1;
                        if page.len() < want {
                            page.push((id, 1.0));
                        } else if !track_total {
                            return Ok(false);
                        }
                    }
                }
            }
            if let Some(tail) = tail_posting {
                total += tail.len();
                for id in tail {
                    if page.len() < want {
                        page.push((id, 1.0));
                    } else if !track_total {
                        return Ok(false);
                    }
                }
            }
            Ok(true)
        };

        loop {
            let seg_bits = if si < i_hi {
                seg.number_sorted_bits_at(si)
            } else {
                None
            };
            let tail_key = tail_keys.get(ti).copied();
            match (seg_bits, tail_key) {
                (None, None) => break,
                (Some(sb), None) => {
                    let p = seg.number_sorted_postings_at(si);
                    if !emit_value(sb, p, None)? {
                        total = total.max(page.len() as u64);
                        return Ok((page, total));
                    }
                    si += 1;
                }
                (None, Some(tk)) => {
                    let t = self.values.get(&tk);
                    if !emit_value(tk.bits(), None, t)? {
                        total = total.max(page.len() as u64);
                        return Ok((page, total));
                    }
                    ti += 1;
                }
                (Some(sb), Some(tk)) => {
                    let tb = tk.bits();
                    if sb < tb {
                        let p = seg.number_sorted_postings_at(si);
                        if !emit_value(sb, p, None)? {
                            total = total.max(page.len() as u64);
                            return Ok((page, total));
                        }
                        si += 1;
                    } else if sb > tb {
                        let t = self.values.get(&tk);
                        if !emit_value(tb, None, t)? {
                            total = total.max(page.len() as u64);
                            return Ok((page, total));
                        }
                        ti += 1;
                    } else {
                        let p = seg.number_sorted_postings_at(si);
                        let t = self.values.get(&tk);
                        if !emit_value(sb, p, t)? {
                            total = total.max(page.len() as u64);
                            return Ok((page, total));
                        }
                        si += 1;
                        ti += 1;
                    }
                }
            }
        }
        if !track_total {
            total = total.max(page.len() as u64);
        }
        Ok((page, total))
    }
}

#[derive(Debug, Default)]
struct SetIndex {
    elements: BTreeMap<String, RoaringBitmap>,
    /// Side-index of LIVE-tail elements whose posting holds >= 2 docs (see
    /// `KeywordIndex::dup_values`); drives `duplicates` without a full
    /// `elements` scan. Cleared at seal.
    dup_values: BTreeSet<String>,
    forward: FastHashMap<u32, BTreeSet<String>>,
    bytes: u64,
    /// Stage 2 disk-tier (Phase 2e-A): a sealed columnar mmap segment covering
    /// doc ids `[0..n_docs)` — a shared sorted string DICT (var-width) + a fixed
    /// `u32[n_docs + 1]` CSR offsets column + a fixed `u32[*]` packed dict-id
    /// column (doc `i`'s members = `packed[offsets[i]..offsets[i+1]]`) PLUS a
    /// parallel per-element INVERTED posting column (Phase 2h-2). When present,
    /// per-doc Set membership PREDICATE lookups (`set_contains`) read the segment
    /// for sealed ids and the in-RAM `forward` tail for ids `>= n_docs`, AND the
    /// inverted membership / Terms / boolean driver reads the on-disk postings
    /// (the RAM `elements` index was DROPPED at seal). RAM after reopen is
    /// O(live tail), not O(distinct set elements). DEFAULTS to `None`; while it
    /// is `None` (nothing sealed) every read path is byte-for-byte the
    /// in-RAM path. Purely additive.
    segment: Option<std::sync::Arc<crate::segment::SegmentReader>>,
    /// QUERY-TIME TOMBSTONE (Phase 2h-2): base docids `[0..seg.n_docs)` deleted
    /// SINCE the last seal. The inverted `elements` index was DROPPED to disk at
    /// seal, so `drop_eid` can no longer remove a sealed base id from the
    /// immutable on-disk postings — instead it records the id here, and the
    /// segment-ON accessors (`element_postings`, duplicate / unique-element
    /// enumeration) SUBTRACT this set so a delete is reflected before the next
    /// re-seal. The next seal bakes the deletions into the new segment (its
    /// live(id) gather excludes them) and this is reset to empty. Live-tail ids
    /// (`>= seg.n_docs`) are NOT tombstoned — they are deleted directly out of
    /// the in-RAM `elements` tail. DEFAULTS empty; stays empty while no
    /// segment is attached. The exact reuse of the Keyword 2h-1 tombstone (same
    /// shape, same four touch-points: record in `drop_eid`, subtract in the
    /// posting accessor, exclude in `live_elements`, clear at re-seal).
    tombstones: RoaringBitmap,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl SetIndex {
    /// Does doc `id`'s set contain `el`, for a per-doc PREDICATE point lookup?
    /// When a sealed segment is attached it serves ids in its covered range
    /// `[0..n_docs)` (the live tail keeps ids `>= n_docs`); otherwise — and
    /// always when no segment is attached — this is exactly
    /// `self.forward.get(&id).map(|s| s.contains(el))`. The segment stores the
    /// exact member strings, so membership matches the live `forward` entry.
    #[inline]
    fn set_contains(&self, id: u32, el: &str) -> bool {
        if let Some(seg) = &self.segment {
            if id < seg.n_docs() {
                return seg
                    .set_at(id)
                    .map(|members| members.iter().any(|m| m == el))
                    .unwrap_or(false);
            }
        }
        self.forward
            .get(&id)
            .map(|s| s.contains(el))
            .unwrap_or(false)
    }

    /// Does doc `id`'s set contain ANY of `values`'s string members? The
    /// `Terms` predicate site. Reads the segment members ONCE (not once per
    /// candidate value) when sealed, falling back to the live `forward` set.
    #[inline]
    fn set_contains_any(&self, id: u32, values: &[FieldValue]) -> bool {
        if let Some(seg) = &self.segment {
            if id < seg.n_docs() {
                let Some(members) = seg.set_at(id) else {
                    return false;
                };
                return values.iter().any(
                    |val| matches!(val, FieldValue::String(el) if members.iter().any(|m| m == el)),
                );
            }
        }
        self.forward.get(&id).is_some_and(|set| {
            values
                .iter()
                .any(|val| matches!(val, FieldValue::String(el) if set.contains(el)))
        })
    }

    /// Doc `id`'s full member set, routed through the segment for sealed ids
    /// (Phase 2f-1) or the live `forward` tail. `None` for a doc with no set
    /// value. Used by `drop_eid` after the forward payload has been dropped to
    /// disk, so the inverted `elements` postings are still removed on delete.
    #[inline]
    fn set_members(&self, id: u32) -> Option<BTreeSet<String>> {
        if let Some(seg) = &self.segment {
            if id < seg.n_docs() {
                return seg.set_at(id).map(|m| m.into_iter().collect());
            }
        }
        self.forward.get(&id).cloned()
    }

    /// `true` once the sealed forward payload has been dropped to disk
    /// (Phase 2f-1) — the segment is attached. See `KeywordIndex::forward_dropped`.
    #[inline]
    fn forward_dropped(&self) -> bool {
        self.segment.is_some()
    }

    /// UNIFIED inverted-index accessor (Phase 2h-2): element `el`'s posting
    /// bitmap (docids whose set contains `el`) on the ACTIVE source, composing
    /// the disk segment (sealed base ids `[0..seg.n_docs)`) with the in-RAM
    /// `elements` tail (ids added after the seal, or — when no segment is
    /// attached — the WHOLE inverted index). The Set analogue of
    /// `KeywordIndex::term_postings`.
    ///
    /// - segment OFF (no segment attached): returns `Cow::Borrowed` of
    ///   `elements[el]` — ZERO clone, byte-for-byte the old `s.elements.get(el)`
    ///   path that every membership / Terms / selectivity / planner site read.
    /// - segment ON: decodes the segment's stored postings for `el` (the RAM
    ///   `elements` index was DROPPED at seal), SUBTRACTS the query-time
    ///   `tombstones` (sealed-base deletes), and UNIONs any live-tail postings
    ///   indexed back into `elements`. The union is `Cow::Owned`. Both bases are
    ///   ascending-docid RoaringBitmaps, so the result is identical to what an
    ///   un-dropped in-RAM `elements` held.
    ///
    /// `None` only when the element is absent from BOTH sources (so callers keep
    /// their `.unwrap_or_default()` empty-posting semantics).
    #[inline]
    fn element_postings(&self, el: &str) -> Option<std::borrow::Cow<'_, RoaringBitmap>> {
        if let Some(seg) = &self.segment {
            // Segment base, MINUS the query-time tombstone (base docids deleted
            // since the last seal — the on-disk postings can't be mutated), then
            // UNION the live-tail postings indexed back into `elements` after the
            // seal. Tombstones only ever hold base ids (`< seg.n_docs`) and tail
            // ids are `>= seg.n_docs`, so the subtraction never touches the tail.
            let mut base = seg.set_postings(el).unwrap_or_default();
            if !self.tombstones.is_empty() {
                base -= &self.tombstones;
            }
            if let Some(t) = self.elements.get(el) {
                base |= t;
            }
            return if base.is_empty() {
                None
            } else {
                Some(std::borrow::Cow::Owned(base))
            };
        }
        self.elements.get(el).map(std::borrow::Cow::Borrowed)
    }

    /// UNIFIED document-frequency accessor (Phase 2h-2): element `el`'s `df` on
    /// the ACTIVE source — the boolean planner's rarest-first selectivity input.
    /// segment OFF: the live `elements[el]` length. segment ON: the segment's
    /// CHEAP count-prefix df (no posting decode) PLUS any live-tail length. The
    /// Set analogue of `KeywordIndex::term_df`.
    ///
    /// TOMBSTONE NOTE (Phase 2h-2): like the Keyword `term_df`, this deliberately
    /// does NOT subtract the query-time `tombstones`. `df` only drives the
    /// boolean planner's rarest-first clause ORDERING — never the result set,
    /// which comes from `element_postings` (which DOES subtract the tombstone).
    /// A small df over-count can at worst pick a marginally less-rare lead
    /// clause; the emitted documents are identical. Keeping df off the full
    /// posting decode is the whole point of the count-prefix.
    #[inline]
    fn element_df(&self, el: &str) -> u64 {
        if let Some(seg) = &self.segment {
            let base = seg.set_df(el).unwrap_or(0);
            let tail = self.elements.get(el).map(|p| p.len()).unwrap_or(0);
            return base + tail;
        }
        self.elements.get(el).map(|p| p.len()).unwrap_or(0)
    }

    /// Every distinct element with its LIVE posting bitmap, on the ACTIVE source
    /// — the unified enumeration that `unique_terms` and `duplicates` drive from
    /// so they stay correct after a seal drops the in-RAM `elements` driver
    /// (Phase 2h-2 FIX). Elements with no live doc are EXCLUDED (the same
    /// invariant the in-RAM `elements` map keeps — `drop_eid` removes an emptied
    /// element). The Set analogue of `KeywordIndex::live_terms`.
    ///
    /// - segment OFF: clones each live `elements` entry verbatim.
    /// - segment ON: enumerate the segment dict (`set_elements_all`), subtract
    ///   the query-time `tombstones` from each base posting, UNION the live-tail
    ///   `elements` for the same value, and keep the element iff >=1 live doc
    ///   remains. Tail-only elements (indexed after the seal, absent from the
    ///   dict) are folded in too.
    fn live_elements(&self) -> BTreeMap<String, RoaringBitmap> {
        let Some(seg) = &self.segment else {
            return self.elements.clone();
        };
        let mut out: BTreeMap<String, RoaringBitmap> = BTreeMap::new();
        if let Some(dict) = seg.set_elements_all() {
            for (el, mut postings) in dict {
                if !self.tombstones.is_empty() {
                    postings -= &self.tombstones;
                }
                if let Some(tail) = self.elements.get(&el) {
                    postings |= tail;
                }
                if !postings.is_empty() {
                    out.insert(el, postings);
                }
            }
        }
        // Fold any live-tail-only elements the dict did not carry (indexed after
        // the seal). An element already present from the dict is left merged.
        for (el, tail) in &self.elements {
            if !out.contains_key(el) {
                out.insert(el.clone(), tail.clone());
            }
        }
        out
    }
}

/// One field's index. Vector fields hold a heap-allocated trait
/// object pointing at the chosen backend.
enum FieldIndex {
    Text {
        analyzer: Analyzer,
        idx: TextIndex,
    },
    Keyword(KeywordIndex),
    Number(NumberIndex),
    Set(SetIndex),
    Vector {
        spec: VectorSpec,
        idx: Box<dyn VectorIndex>,
        /// Approximate bytes the field is currently holding. Tracked
        /// for `stats` parity with the other variants.
        bytes: u64,
    },
    Hash(HashIndex),
}

/// A `hash` field: docid → 64-bit hash, answered by a brute-force Hamming scan.
/// No LSH bucketing yet (linear over the forward map) — correct, not yet
/// sub-linear; perceptual-hash corpora are typically small relative to text.
#[derive(Debug, Default)]
struct HashIndex {
    forward: FastHashMap<u32, u64>,
    bytes: u64,
    /// Stage 2 disk-tier (Phase 2d): a sealed columnar mmap segment covering
    /// doc ids `[0..n_docs)`. When present, the per-doc hash read in the Hamming
    /// scan (`hash_at`) reads the segment for sealed ids and the in-RAM
    /// `forward` tail for ids `>= n_docs`. DEFAULTS to `None`; while it is
    /// `None` (nothing sealed) the read
    /// path is byte-for-byte the in-RAM path. Purely additive.
    segment: Option<std::sync::Arc<crate::segment::SegmentReader>>,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl HashIndex {
    /// The doc's 64-bit hash for the per-doc Hamming read. When a sealed segment
    /// is attached it serves ids in its covered range `[0..n_docs)` (the live
    /// tail keeps ids `>= n_docs`); otherwise — and always when no segment
    /// is attached — this is exactly `self.forward.get(&id)`. The segment stores the
    /// raw `u64`, so a hit is bit-equal to the live entry.
    #[inline]
    fn hash_at(&self, id: u32) -> Option<u64> {
        if let Some(seg) = &self.segment {
            if id < seg.n_docs() {
                return seg.hash_at(id);
            }
        }
        self.forward.get(&id).copied()
    }

    /// `true` once the sealed forward payload has been dropped to disk
    /// (Phase 2f-1) — the segment is attached. See `KeywordIndex::forward_dropped`.
    #[inline]
    fn forward_dropped(&self) -> bool {
        self.segment.is_some()
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl std::fmt::Debug for FieldIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldIndex::Text { analyzer, idx } => f
                .debug_struct("Text")
                .field("analyzer", analyzer)
                .field("idx", idx)
                .finish(),
            FieldIndex::Keyword(k) => f.debug_tuple("Keyword").field(k).finish(),
            FieldIndex::Number(n) => f.debug_tuple("Number").field(n).finish(),
            FieldIndex::Set(s) => f.debug_tuple("Set").field(s).finish(),
            FieldIndex::Vector { spec, bytes, .. } => f
                .debug_struct("Vector")
                .field("spec", spec)
                .field("bytes", bytes)
                .finish(),
            FieldIndex::Hash(h) => f.debug_tuple("Hash").field(h).finish(),
        }
    }
}

/// Parse a `hash` field value: a 64-bit hex string, optionally `0x`-prefixed.
fn parse_hash(s: &str) -> Result<u64> {
    let t = s.trim();
    let hex = t
        .strip_prefix("0x")
        .or_else(|| t.strip_prefix("0X"))
        .unwrap_or(t);
    u64::from_str_radix(hex, 16)
        .map_err(|e| anyhow!("hash field expects a 64-bit hex string (got `{s}`): {e}"))
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl FieldIndex {
    fn from_spec(spec: &FieldSpec) -> Result<Self> {
        Ok(match spec.field_type {
            FieldType::Text => FieldIndex::Text {
                analyzer: spec.analyzer.unwrap_or(Analyzer::WhitespaceLower),
                idx: TextIndex::default(),
            },
            FieldType::Keyword => FieldIndex::Keyword(KeywordIndex::default()),
            FieldType::Number => FieldIndex::Number(NumberIndex::default()),
            FieldType::Set => FieldIndex::Set(SetIndex::default()),
            FieldType::Vector => {
                let vs = spec
                    .vector_spec()?
                    .ok_or_else(|| anyhow!("vector field is missing its sub-spec"))?;
                FieldIndex::Vector {
                    spec: vs,
                    idx: open_backend(vs),
                    bytes: 0,
                }
            }
            FieldType::Hash => FieldIndex::Hash(HashIndex::default()),
        })
    }

    fn field_type(&self) -> FieldType {
        match self {
            FieldIndex::Text { .. } => FieldType::Text,
            FieldIndex::Keyword(_) => FieldType::Keyword,
            FieldIndex::Number(_) => FieldType::Number,
            FieldIndex::Set(_) => FieldType::Set,
            FieldIndex::Vector { .. } => FieldType::Vector,
            FieldIndex::Hash(_) => FieldType::Hash,
        }
    }

    fn bytes(&self) -> u64 {
        match self {
            FieldIndex::Text { idx, .. } => idx.bytes,
            FieldIndex::Keyword(k) => k.bytes,
            FieldIndex::Number(n) => n.bytes,
            FieldIndex::Set(s) => s.bytes,
            FieldIndex::Vector { bytes, .. } => *bytes,
            FieldIndex::Hash(h) => h.bytes,
        }
    }

    fn unique_terms(&self) -> u64 {
        match self {
            // Phase 2h-4 FIX: a SEALED Text field has an empty in-RAM `tokens`
            // (dropped at seal), so `tokens.len()` would report 0. Count distinct
            // tokens with >=1 LIVE doc from the segment dict (minus tombstones) +
            // the live tail. Segment OFF: `live_unique_tokens` returns the live
            // `tokens.len()` — byte-for-byte the old value, and identical on the
            // default in-RAM (no-segment) path.
            FieldIndex::Text { idx, .. } => idx.live_unique_tokens(),
            // Phase 2h-1 FIX: a SEALED Keyword field has an empty in-RAM `terms`
            // (dropped at seal), so `terms.len()` would report 0. Count distinct
            // terms with >=1 LIVE doc from the segment dict (minus tombstones) +
            // the live tail. Segment OFF: `live_terms` returns the live `terms`
            // clone, so the count equals `terms.len()` — byte-for-byte the old
            // value, and identical on the default in-RAM (no-segment) path.
            FieldIndex::Keyword(k) => k.live_terms().len() as u64,
            // Phase 2h-3 FIX: a SEALED Number field has an empty in-RAM `values`
            // (dropped at seal), so `values.len()` would report 0. Count distinct
            // values with >=1 LIVE doc from the segment sorted-value column (minus
            // tombstones) + the live tail. Segment OFF: `live_values` returns the
            // live `values` clone, so the count equals `values.len()` —
            // byte-for-byte the old value on the default in-RAM (no-segment) path.
            FieldIndex::Number(n) => n.live_values().len() as u64,
            // Phase 2h-2 FIX: a SEALED Set field has an empty in-RAM `elements`
            // (dropped at seal), so `elements.len()` would report 0. Count
            // distinct elements with >=1 LIVE doc from the segment dict (minus
            // tombstones) + the live tail. Segment OFF: `live_elements` returns
            // the live `elements` clone, so the count equals `elements.len()` —
            // byte-for-byte the old value on the default in-RAM (no-segment) path.
            FieldIndex::Set(s) => s.live_elements().len() as u64,
            // For a vector field "unique terms" doesn't really map —
            // surface the count of distinct vectors held instead.
            FieldIndex::Vector { idx, .. } => idx.len() as u64,
            // Distinct hash values held.
            FieldIndex::Hash(h) => h
                .forward
                .values()
                .collect::<std::collections::HashSet<_>>()
                .len() as u64,
        }
    }

    /// Mean tokens per document on `text` fields; `None` on any other
    /// type. Exposes the BM25 length-normalization denominator so
    /// callers can reason about scoring stability.
    fn avg_doc_len(&self) -> Option<f32> {
        match self {
            FieldIndex::Text { idx, .. } if idx.doc_count > 0 => {
                Some(idx.total_doc_len as f32 / idx.doc_count as f32)
            }
            _ => None,
        }
    }

    fn add_field(&self) {
        // Field indexes are created when a field is added; nothing to do
        // beyond construction. Method exists to mirror future "register
        // analyzer / open SST" hooks on the LSM backend.
    }

    /// Remove every posting written by doc-id `id` (external_id `eid`, needed
    /// only for the String-keyed vector backend) and return the number of
    /// bytes freed (approximate, used to keep `bytes` honest).
    fn drop_eid(&mut self, id: u32, eid: &str) -> u64 {
        match self {
            FieldIndex::Text { idx, .. } => {
                idx.clear_match_rank_cache();
                // QUERY-TIME TOMBSTONE (Phase 2h-4): when the field is SEALED and
                // `id` is a base docid `< seg.n_docs`, its postings live on the
                // IMMUTABLE on-disk blocks (the RAM `tokens` AND the corpus-deriving
                // `distinct` were dropped at seal), so neither `tokens.get_mut` nor
                // `distinct.remove` can reach them. Record the id in `tombstones`
                // (the BM25 scan subtracts it from every token posting BEFORE `df`
                // and scoring), and DECREMENT the LIVE corpus scalars by the doc's
                // length read from the segment DocLen column — so `doc_count`,
                // `total_doc_len`, and therefore `avgdl` track an in-RAM oracle that
                // physically removed the doc. The re-seal bakes the delete in
                // (`tokens_for_seal`'s `live(id)` gather) and clears the set.
                // Live-tail ids (`>= seg.n_docs`) fall through to the in-RAM
                // `distinct`/`tokens` path below.
                if let Some(seg) = &idx.segment {
                    if id < seg.n_docs() {
                        // Idempotency / double-delete guard: an id already
                        // tombstoned is no longer live, so do not re-decrement.
                        if idx.tombstones.contains(id) {
                            return 0;
                        }
                        let doc_len = seg.text_doc_len(id);
                        idx.tombstones.insert(id);
                        idx.doc_count = idx.doc_count.saturating_sub(1);
                        idx.total_doc_len = idx.total_doc_len.saturating_sub(doc_len as u64);
                        // The on-disk posting bytes don't shrink; report the same
                        // per-doc byte estimate the in-RAM removal would free
                        // (doc_len token postings) so `bytes` stays consistent with
                        // the live-path accounting.
                        let freed = (doc_len as usize * (1 + eid.len())) as u64;
                        idx.bytes = idx.bytes.saturating_sub(freed);
                        return freed;
                    }
                }
                let Some(tokens) = idx.take_distinct(id) else {
                    return 0;
                };
                let doc_len = idx.doc_len(id);
                let mut freed = 0u64;
                for tok in tokens.iter() {
                    if let Some(p) = idx.tokens.get_mut(tok) {
                        if p.remove(id) {
                            freed += (tok.len() + eid.len()) as u64;
                        }
                        if p.docids.is_empty() {
                            idx.tokens.remove(tok);
                        }
                    }
                }
                idx.set_doc_len(id, 0);
                idx.doc_count = idx.doc_count.saturating_sub(1);
                idx.total_doc_len = idx.total_doc_len.saturating_sub(doc_len as u64);
                idx.bytes = idx.bytes.saturating_sub(freed);
                freed
            }
            FieldIndex::Keyword(k) => {
                // After a seal-and-drop the forward payload lives on the
                // segment, not in `forward`. Resolve the doc's keyword through
                // `keyword_at` (segment for sealed ids, else the live tail) so a
                // delete still finds the term to remove from the inverted index.
                let value = if k.forward_dropped() {
                    let value = match k.keyword_at(id) {
                        Some(v) => v,
                        None => return 0,
                    };
                    if k.segment.as_ref().is_some_and(|seg| id >= seg.n_docs()) {
                        k.remove_keyword(id);
                    }
                    value
                } else {
                    match k.remove_keyword(id) {
                        Some(v) => v,
                        None => return 0,
                    }
                };
                let mut freed = 0u64;
                // QUERY-TIME TOMBSTONE (Phase 2h-1 FIX): when the field is SEALED
                // and `id` is a base docid `< seg.n_docs`, its posting lives on the
                // IMMUTABLE on-disk column — `terms.get_mut` can't reach it (the
                // RAM `terms` index was dropped at seal). Record the id in
                // `tombstones` so every segment-ON accessor subtracts it; the
                // re-seal bakes it in and clears the set. Live-tail ids
                // (`>= seg.n_docs`) fall through to the in-RAM `terms` path below.
                if let Some(seg) = &k.segment {
                    if id < seg.n_docs() {
                        k.tombstones.insert(id);
                        // The on-disk posting bytes don't shrink; report the same
                        // per-term byte estimate the in-RAM removal would free so
                        // `bytes` stays consistent with the live-path accounting.
                        freed = (value.len() + eid.len()) as u64;
                        k.bytes = k.bytes.saturating_sub(freed);
                        return freed;
                    }
                }
                if let Some(set) = k.terms.get_mut(&value) {
                    if set.remove(id) {
                        freed = (value.len() + eid.len()) as u64;
                    }
                    if set.len() < 2 {
                        k.dup_values.remove(&value);
                    }
                    if set.is_empty() {
                        k.terms.remove(&value);
                    }
                }
                k.bytes = k.bytes.saturating_sub(freed);
                freed
            }
            FieldIndex::Number(n) => {
                // Sealed-and-dropped: resolve the key through `number_at`
                // (segment for sealed ids) so the inverted `values` entry is
                // still removed on delete.
                let key = if n.forward_dropped() {
                    match n.number_at(id) {
                        Some(k) => k,
                        None => return 0,
                    }
                } else {
                    match n.remove_number(id) {
                        Some(k) => k,
                        None => return 0,
                    }
                };
                let mut freed = 0u64;
                // QUERY-TIME TOMBSTONE (Phase 2h-3): when the field is SEALED and
                // `id` is a base docid `< seg.n_docs`, its posting lives on the
                // IMMUTABLE on-disk sorted-value index — `values.get_mut` can't
                // reach it (the RAM `values` index was dropped at seal). Record the
                // id in `tombstones` so every segment-ON accessor subtracts it; the
                // re-seal bakes it in and clears the set. Live-tail ids
                // (`>= seg.n_docs`) fall through to the in-RAM `values` path below.
                if let Some(seg) = &n.segment {
                    if id < seg.n_docs() {
                        n.tombstones.insert(id);
                        // The on-disk posting bytes don't shrink; report the same
                        // byte estimate the in-RAM removal would free so `bytes`
                        // stays consistent with the live-path accounting.
                        freed = (8 + eid.len()) as u64;
                        n.bytes = n.bytes.saturating_sub(freed);
                        return freed;
                    }
                }
                if n.forward_dropped() {
                    n.remove_number(id);
                }
                if let Some(set) = n.values.get_mut(&key) {
                    if set.remove(id) {
                        freed = (8 + eid.len()) as u64;
                    }
                    if set.len() < 2 {
                        n.dup_values.remove(&key);
                    }
                    if set.is_empty() {
                        n.values.remove(&key);
                    }
                }
                n.bytes = n.bytes.saturating_sub(freed);
                freed
            }
            FieldIndex::Set(s) => {
                // Sealed-and-dropped: read the member set through `set_members`
                // (segment for sealed ids) so the inverted `elements` postings
                // are still removed on delete.
                let elems = if s.forward_dropped() {
                    match s.set_members(id) {
                        Some(e) => e,
                        None => return 0,
                    }
                } else {
                    match s.forward.remove(&id) {
                        Some(e) => e,
                        None => return 0,
                    }
                };
                let mut freed = 0u64;
                // QUERY-TIME TOMBSTONE (Phase 2h-2): when the field is SEALED and
                // `id` is a base docid `< seg.n_docs`, EVERY element's posting
                // lives on the IMMUTABLE on-disk column — `elements.get_mut` can't
                // reach them (the RAM `elements` index was dropped at seal). Record
                // the id ONCE in `tombstones` so every segment-ON accessor subtracts
                // it; the re-seal bakes it in and clears the set. Live-tail ids
                // (`>= seg.n_docs`) fall through to the in-RAM `elements` path below.
                if let Some(seg) = &s.segment {
                    if id < seg.n_docs() {
                        s.tombstones.insert(id);
                        // The on-disk posting bytes don't shrink; report the same
                        // per-element byte estimate the in-RAM removal would free so
                        // `bytes` stays consistent with the live-path accounting.
                        for el in &elems {
                            freed += (el.len() + eid.len()) as u64;
                        }
                        s.bytes = s.bytes.saturating_sub(freed);
                        return freed;
                    }
                }
                for el in &elems {
                    if let Some(set) = s.elements.get_mut(el) {
                        if set.remove(id) {
                            freed += (el.len() + eid.len()) as u64;
                        }
                        if set.len() < 2 {
                            s.dup_values.remove(el);
                        }
                        if set.is_empty() {
                            s.elements.remove(el);
                        }
                    }
                }
                s.bytes = s.bytes.saturating_sub(freed);
                freed
            }
            FieldIndex::Vector { spec, idx, bytes } => {
                // Trait-object remove always succeeds (Ok). We don't
                // track per-eid byte sizes precisely here — subtract a
                // proportional share of the dimension instead so
                // `stats` stays approximately honest.
                let approx = (spec.dim as u64) * 4 + eid.len() as u64;
                match idx.remove(eid) {
                    Ok(true) => {
                        *bytes = bytes.saturating_sub(approx);
                        approx
                    }
                    _ => 0,
                }
            }
            FieldIndex::Hash(h) => {
                // Sealed-and-dropped: a Hash field has no inverted index, so the
                // only state to clear is the forward entry — already gone from
                // RAM. Confirm the doc existed (via the segment) so `drop_eid`
                // still reports the freed bytes; the segment row itself stays
                // (it is immutable, and the Hamming scan gates on the live
                // present set / eid_fields).
                if h.forward_dropped() {
                    return if h.hash_at(id).is_some() {
                        let freed = 12u64;
                        h.bytes = h.bytes.saturating_sub(freed);
                        freed
                    } else {
                        0
                    };
                }
                if h.forward.remove(&id).is_some() {
                    let freed = 12u64;
                    h.bytes = h.bytes.saturating_sub(freed);
                    freed
                } else {
                    0
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Collection
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
struct FieldCoverage {
    names: Vec<String>,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl FieldCoverage {
    fn insert(&mut self, field: String) -> bool {
        if self.contains(&field) {
            return false;
        }
        self.insert_absent(field);
        true
    }

    fn insert_absent(&mut self, field: String) {
        debug_assert!(!self.contains(&field));
        self.names.push(field);
    }

    fn contains(&self, field: &str) -> bool {
        self.names.iter().any(|seen| seen == field)
    }

    fn remove(&mut self, field: &str) -> bool {
        let Some(pos) = self.names.iter().position(|name| name == field) else {
            return false;
        };
        self.names.swap_remove(pos);
        true
    }

    fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    fn iter(&self) -> impl Iterator<Item = &String> {
        self.names.iter()
    }

    fn to_btree_set(&self) -> BTreeSet<String> {
        self.names.iter().cloned().collect()
    }

    fn from_btree_set(set: BTreeSet<String>) -> Self {
        Self {
            names: set.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct TokenSet {
    tokens: SmallVec<[String; 8]>,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl TokenSet {
    fn insert_str(&mut self, token: &str) -> bool {
        if self.tokens.iter().any(|seen| seen == token) {
            return false;
        }
        self.tokens.push(token.to_string());
        true
    }

    fn iter(&self) -> impl Iterator<Item = &String> {
        self.tokens.iter()
    }

    fn to_btree_set(&self) -> BTreeSet<String> {
        self.tokens.iter().cloned().collect()
    }

    fn from_btree_set(set: BTreeSet<String>) -> Self {
        Self {
            tokens: set.into_iter().collect(),
        }
    }
}

fn add_bytes_written(bytes_by_field: &mut Vec<(String, u64)>, field: &str, bytes: u64) {
    if let Some((_, total)) = bytes_by_field.iter_mut().find(|(seen, _)| seen == field) {
        *total += bytes;
        return;
    }
    bytes_by_field.push((field.to_string(), bytes));
}

fn flush_group_coverage(
    eid_fields: &mut FastHashMap<u32, FieldCoverage>,
    id: u32,
    new_doc_in_request: bool,
    fields: &mut FieldCoverage,
) {
    if !new_doc_in_request {
        return;
    }
    if fields.is_empty() {
        return;
    }
    eid_fields.insert(id, std::mem::take(fields));
}

#[derive(Debug)]
struct Collection {
    version: u32,
    schema: BTreeMap<String, FieldSpec>,
    fields: FastHashMap<String, FieldIndex>,
    /// external_id ↔ dense u32 doc-id. Posting lists carry the u32.
    interner: Interner,
    /// Tracks which fields each doc-id wrote into — supports
    /// "delete all fields for this eid".
    eid_fields: FastHashMap<u32, FieldCoverage>,
    /// Recent request_id → timestamp; drives idempotency.
    seen_requests: VecDeque<(String, Instant)>,
    /// When set, the collection is soft-deleted — reads/writes return
    /// 410 Gone, and `Engine::sweep_deleted` will physically drop it
    /// once the grace window has elapsed.
    deleted_at: Option<Instant>,
    /// Wall-clock of the most recent successful index write. Exposed
    /// via `/stats.last_indexed_at` so callers can verify "I wrote N
    /// docs at T, did they land?" without trawling the audit log.
    last_indexed_at: Option<std::time::SystemTime>,
    /// Hot query results keyed by full `SearchRequest` JSON. Mutations clear it
    /// before changing postings so repeated serving queries can skip planner work
    /// without returning stale hits.
    search_cache: RwLock<FastHashMap<String, SearchResponse>>,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl Collection {
    fn new(schema: BTreeMap<String, FieldSpec>) -> Result<Self> {
        let mut fields = FastHashMap::default();
        for (name, spec) in &schema {
            fields.insert(name.clone(), FieldIndex::from_spec(spec)?);
        }
        Ok(Self {
            version: 1,
            schema,
            fields,
            interner: Interner::default(),
            eid_fields: FastHashMap::default(),
            seen_requests: VecDeque::new(),
            deleted_at: None,
            last_indexed_at: None,
            search_cache: RwLock::new(FastHashMap::default()),
        })
    }

    fn check_live(&self, collection_id: &str) -> Result<()> {
        if self.deleted_at.is_some() {
            return Err(StorageError::Gone(collection_id.to_string()).into());
        }
        Ok(())
    }

    fn fields_count(&self) -> u32 {
        self.schema.len() as u32
    }

    fn check_request_id(&mut self, request_id: Option<&str>) -> bool {
        // Returns true if the request should be skipped (duplicate).
        self.gc_requests();
        let Some(id) = request_id else {
            return false;
        };
        if self.seen_requests.iter().any(|(k, _)| k == id) {
            return true;
        }
        self.seen_requests
            .push_back((id.to_string(), Instant::now()));
        false
    }

    fn gc_requests(&mut self) {
        let now = Instant::now();
        while let Some((_, t)) = self.seen_requests.front() {
            if now.duration_since(*t) > IDEMPOTENCY_TTL {
                self.seen_requests.pop_front();
            } else {
                break;
            }
        }
    }

    fn clear_number_filter_caches(&mut self) {
        for fi in self.fields.values_mut() {
            if let FieldIndex::Number(n) = fi {
                n.clear_keyword_range_cache();
            }
        }
    }

    fn clear_search_cache(&self) {
        if let Ok(mut cache) = self.search_cache.write() {
            cache.clear();
        }
    }

    fn cached_search_response(&self, key: &str) -> Option<SearchResponse> {
        self.search_cache
            .read()
            .ok()
            .and_then(|cache| cache.get(key).cloned())
    }

    fn cache_search_response(&self, key: String, response: &SearchResponse) {
        let Ok(mut cache) = self.search_cache.write() else {
            return;
        };
        if cache.len() >= SEARCH_RESULT_CACHE_MAX && !cache.contains_key(&key) {
            cache.clear();
        }
        cache.insert(key, response.clone());
    }

    fn clear_text_rank_caches(&self) {
        for fi in self.fields.values() {
            if let FieldIndex::Text { idx, .. } = fi {
                idx.clear_match_rank_cache();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

#[derive(Default)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
pub struct Engine {
    state: RwLock<EngineState>,
    metrics: Metrics,
    draining: AtomicBool,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("draining", &self.draining)
            .finish()
    }
}

#[derive(Debug, Default)]
struct EngineState {
    collections: BTreeMap<String, Collection>,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }

    /// Switch the engine into "draining" — readiness flips to false so
    /// K8s stops sending new traffic. In-flight requests continue to
    /// succeed; the caller is expected to wait its `terminationGracePeriod`
    /// before terminating the process. Idempotent.
    pub fn start_drain(&self) {
        self.draining.store(true, Ordering::SeqCst);
    }

    pub fn is_draining(&self) -> bool {
        self.draining.load(Ordering::SeqCst)
    }

    // -- DDL ----------------------------------------------------------------

    /// Create-or-merge a collection schema.
    ///
    /// * If the collection does not exist, it is created with the
    ///   declared fields at version 1.
    /// * If it exists, fields **missing** from the existing schema are
    ///   appended online (version bumps once per call regardless of how
    ///   many fields were added). Re-declaring an existing field with a
    ///   **different** type / analyzer / multi flag is rejected — type
    ///   changes are an offline op (collection version bump + reindex)
    ///   not covered by this surface in v1.
    pub fn create_collection(
        &self,
        collection_id: &str,
        req: CreateCollectionRequest,
    ) -> Result<CreateCollectionResponse> {
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        let schema: BTreeMap<String, FieldSpec> = req
            .fields
            .into_iter()
            .map(|(k, v)| (k, v.normalize()))
            .collect();
        validate_schema(&schema)?;

        if let Some(coll) = state.collections.get_mut(collection_id) {
            coll.check_live(collection_id)?;
            let mut added = 0u32;
            for (name, spec) in schema {
                match coll.schema.get(&name) {
                    Some(existing) if existing != &spec => {
                        bail!(
                            "field `{name}` already declared in collection `{collection_id}` \
                             with a different spec — type changes need an offline reindex"
                        );
                    }
                    Some(_) => continue,
                    None => {
                        coll.schema.insert(name.clone(), spec.clone());
                        let idx = FieldIndex::from_spec(&spec)?;
                        idx.add_field();
                        coll.fields.insert(name, idx);
                        added += 1;
                    }
                }
            }
            if added > 0 {
                coll.clear_search_cache();
                coll.version += 1;
            }
            let resp = CreateCollectionResponse {
                collection_id: collection_id.to_string(),
                version: coll.version,
                fields_count: coll.fields_count(),
            };
            drop(state);
            return Ok(resp);
        }

        let coll = Collection::new(schema)?;
        let version = coll.version;
        let fields_count = coll.fields_count();
        state.collections.insert(collection_id.to_string(), coll);
        drop(state);
        self.metrics.incr_collection_created(fields_count as u64);
        Ok(CreateCollectionResponse {
            collection_id: collection_id.to_string(),
            version,
            fields_count,
        })
    }

    /// Drop a collection.
    ///
    /// - `force = true`: physically remove now.
    /// - `force = false`: soft-delete — mark `deleted_at = now()` so
    ///   reads/writes start returning 410 Gone and the periodic
    ///   `sweep_deleted` task removes the data after the grace window.
    pub fn drop_collection(&self, collection_id: &str, force: bool) -> Result<DropOutcome> {
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        let Some(coll) = state.collections.get_mut(collection_id) else {
            return Ok(DropOutcome::NotFound);
        };
        if force {
            state.collections.remove(collection_id);
            return Ok(DropOutcome::Physical);
        }
        if coll.deleted_at.is_some() {
            return Ok(DropOutcome::AlreadyMarked);
        }
        coll.clear_search_cache();
        coll.deleted_at = Some(Instant::now());
        Ok(DropOutcome::Marked)
    }

    /// Drop a field from an existing collection. Online — postings are
    /// freed immediately and the schema version bumps. Returns the new
    /// collection version.
    pub fn drop_field(&self, collection_id: &str, field_name: &str) -> Result<u32> {
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get_mut(collection_id)
            .ok_or_else(|| StorageError::CollectionNotFound(collection_id.to_string()))?;
        coll.check_live(collection_id)?;
        if !coll.schema.contains_key(field_name) {
            return Err(StorageError::UnknownField {
                collection: collection_id.to_string(),
                field: field_name.to_string(),
            }
            .into());
        }
        coll.clear_search_cache();
        coll.schema.remove(field_name);
        coll.fields.remove(field_name);
        // Scrub the eid→fields back-references so deletions don't try
        // to drop nonexistent index entries.
        for fields in coll.eid_fields.values_mut() {
            fields.remove(field_name);
        }
        coll.eid_fields.retain(|_, fs| !fs.is_empty());
        coll.version += 1;
        let new_version = coll.version;
        Ok(new_version)
    }

    /// Physically remove every collection whose `deleted_at` is older
    /// than `grace`. Returns the number of physically removed entries.
    pub fn sweep_deleted(&self, grace: Duration) -> Result<usize> {
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        let now = Instant::now();
        let to_remove: Vec<String> = state
            .collections
            .iter()
            .filter_map(|(id, c)| {
                let ts = c.deleted_at?;
                if now.duration_since(ts) >= grace {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();
        let n = to_remove.len();
        for id in &to_remove {
            state.collections.remove(id);
        }
        Ok(n)
    }

    /// Append a new field to an existing collection. Online — existing
    /// documents simply have no postings on the new field until they are
    /// re-indexed. Returns the new collection version.
    pub fn add_field(&self, collection_id: &str, field_name: &str, spec: FieldSpec) -> Result<u32> {
        let spec = spec.normalize();
        if field_name.is_empty() {
            bail!("field name cannot be empty");
        }
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get_mut(collection_id)
            .ok_or_else(|| StorageError::CollectionNotFound(collection_id.to_string()))?;
        coll.check_live(collection_id)?;
        if coll.schema.contains_key(field_name) {
            bail!("field `{field_name}` already exists in collection `{collection_id}`");
        }
        let mut new_schema = coll.schema.clone();
        new_schema.insert(field_name.to_string(), spec.clone());
        validate_schema(&new_schema)?;
        coll.clear_search_cache();
        coll.schema = new_schema;
        let idx = FieldIndex::from_spec(&spec)?;
        idx.add_field();
        coll.fields.insert(field_name.to_string(), idx);
        coll.version += 1;
        let new_version = coll.version;
        Ok(new_version)
    }

    pub fn list_collections(&self) -> Result<Vec<String>> {
        let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
        Ok(state
            .collections
            .iter()
            .filter(|(_, c)| c.deleted_at.is_none())
            .map(|(id, _)| id.clone())
            .collect())
    }

    // -- Index --------------------------------------------------------------

    pub fn index(&self, collection_id: &str, req: IndexRequest) -> Result<IndexResponse> {
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get_mut(collection_id)
            .ok_or_else(|| StorageError::CollectionNotFound(collection_id.to_string()))?;
        Self::index_collection(&self.metrics, collection_id, coll, req)
    }

    fn index_collection(
        metrics: &Metrics,
        collection_id: &str,
        coll: &mut Collection,
        req: IndexRequest,
    ) -> Result<IndexResponse> {
        if req.items.len() > MAX_INDEX_ITEMS {
            return Err(StorageError::BulkLimit {
                got: req.items.len(),
                max: MAX_INDEX_ITEMS,
            }
            .into());
        }
        coll.check_live(collection_id)?;

        if coll.check_request_id(req.request_id.as_deref()) {
            return Ok(IndexResponse {
                indexed: 0,
                bytes_written: BTreeMap::new(),
                shard_lag_ms: 0,
            });
        }
        if !req.items.is_empty() {
            coll.clear_search_cache();
        }

        let mut bytes_by_field: Vec<(String, u64)> = Vec::new();
        let mut cleared_text_rank_caches = false;
        let mut cleared_number_filter_caches = false;
        let mut indexed = 0u32;

        let mut items = req.items;
        let mut cursor = 0usize;
        while cursor < items.len() {
            let group_end = {
                let external_id = items[cursor].external_id.as_str();
                let mut end = cursor + 1;
                while end < items.len() && items[end].external_id == external_id {
                    end += 1;
                }
                end
            };
            let external_id = std::mem::take(&mut items[cursor].external_id);
            let (id, new_doc_in_request) = coll.interner.intern_owned_with_status(external_id);
            let mut new_doc_fields = FieldCoverage::default();

            for pos in cursor..group_end {
                let field_name = std::mem::take(&mut items[pos].field);
                let field = field_name.as_str();
                if !cleared_text_rank_caches || !cleared_number_filter_caches {
                    let field_type = match coll.fields.get(field) {
                        Some(fi) => fi.field_type(),
                        None => {
                            flush_group_coverage(
                                &mut coll.eid_fields,
                                id,
                                new_doc_in_request,
                                &mut new_doc_fields,
                            );
                            return Err(StorageError::UnknownField {
                                collection: collection_id.to_string(),
                                field: field.to_string(),
                            }
                            .into());
                        }
                    };
                    if matches!(field_type, FieldType::Text) && !cleared_text_rank_caches {
                        coll.clear_text_rank_caches();
                        cleared_text_rank_caches = true;
                    }
                    if matches!(field_type, FieldType::Keyword | FieldType::Number)
                        && !cleared_number_filter_caches
                    {
                        coll.clear_number_filter_caches();
                        cleared_number_filter_caches = true;
                    }
                }
                let field_already_indexed = if new_doc_in_request {
                    new_doc_fields.contains(field)
                } else {
                    coll.eid_fields
                        .get(&id)
                        .is_some_and(|fields| fields.contains(field))
                };
                let bytes = {
                    let eid = coll.interner.resolve(id);
                    let fi = coll.fields.get_mut(field).ok_or_else(|| {
                        flush_group_coverage(
                            &mut coll.eid_fields,
                            id,
                            new_doc_in_request,
                            &mut new_doc_fields,
                        );
                        StorageError::UnknownField {
                            collection: collection_id.to_string(),
                            field: field.to_string(),
                        }
                    })?;
                    // Drop any existing posting for (eid, field) before reapply
                    // — re-indexing is a full replacement at field granularity. Pure
                    // append batches skip this path: there is nothing to remove, and
                    // `drop_eid` is intentionally expensive because it must handle sealed
                    // segment tombstones and old forward values.
                    if field_already_indexed {
                        fi.drop_eid(id, eid);
                    }
                    match apply_value(fi, id, eid, &items[pos].value, field) {
                        Ok(bytes) => bytes,
                        Err(e) => {
                            flush_group_coverage(
                                &mut coll.eid_fields,
                                id,
                                new_doc_in_request,
                                &mut new_doc_fields,
                            );
                            return Err(e);
                        }
                    }
                };
                add_bytes_written(&mut bytes_by_field, field, bytes);
                if !field_already_indexed {
                    if new_doc_in_request {
                        new_doc_fields.insert_absent(field_name);
                    } else {
                        coll.eid_fields
                            .entry(id)
                            .or_default()
                            .insert_absent(field_name);
                    }
                } else if new_doc_in_request {
                    debug_assert!(new_doc_fields.contains(field));
                }
                indexed += 1;
            }
            flush_group_coverage(
                &mut coll.eid_fields,
                id,
                new_doc_in_request,
                &mut new_doc_fields,
            );
            cursor = group_end;
        }

        let total_bytes: u64 = bytes_by_field.iter().map(|(_, bytes)| *bytes).sum();
        let bytes_written: BTreeMap<String, u64> = bytes_by_field.into_iter().collect();
        if indexed > 0 {
            coll.last_indexed_at = Some(std::time::SystemTime::now());
        }
        metrics.incr_index(indexed as u64, total_bytes);
        let resp = IndexResponse {
            indexed,
            bytes_written,
            shard_lag_ms: 0,
        };
        Ok(resp)
    }

    // -- Delete -------------------------------------------------------------

    pub fn delete(
        &self,
        collection_id: &str,
        external_id: &str,
        field: Option<&str>,
    ) -> Result<()> {
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get_mut(collection_id)
            .ok_or_else(|| StorageError::CollectionNotFound(collection_id.to_string()))?;
        coll.check_live(collection_id)?;

        // Unknown external_id → never interned → nothing to delete.
        let Some(id) = coll.interner.id(external_id) else {
            return Ok(());
        };
        coll.clear_search_cache();
        coll.clear_number_filter_caches();
        match field {
            Some(f) => {
                if let Some(fi) = coll.fields.get_mut(f) {
                    fi.drop_eid(id, external_id);
                }
                if let Some(fields) = coll.eid_fields.get_mut(&id) {
                    fields.remove(f);
                    if fields.is_empty() {
                        coll.eid_fields.remove(&id);
                    }
                }
            }
            None => {
                let fields: Vec<String> = coll
                    .eid_fields
                    .get(&id)
                    .map(|s| s.iter().cloned().collect())
                    .unwrap_or_default();
                for f in fields {
                    if let Some(fi) = coll.fields.get_mut(&f) {
                        fi.drop_eid(id, external_id);
                    }
                }
                coll.eid_fields.remove(&id);
            }
        }
        Ok(())
    }

    // -- Search -------------------------------------------------------------

    pub fn number_value_for_external_id(
        &self,
        collection_id: &str,
        external_id: &str,
        field: &str,
    ) -> Result<Option<f64>> {
        let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get(collection_id)
            .ok_or_else(|| StorageError::CollectionNotFound(collection_id.to_string()))?;
        coll.check_live(collection_id)?;
        let Some(id) = coll.interner.id(external_id) else {
            return Ok(None);
        };
        let fi = coll
            .fields
            .get(field)
            .ok_or_else(|| StorageError::UnknownField {
                collection: collection_id.to_string(),
                field: field.to_string(),
            })?;
        let FieldIndex::Number(nidx) = fi else {
            return Ok(None);
        };
        Ok(nidx.live_number_at(id).map(SortableF64::to_f64))
    }

    pub fn search(&self, collection_id: &str, req: SearchRequest) -> Result<SearchResponse> {
        let start = Instant::now();
        // DoS guard: reject pathological query trees before evaluation.
        validate_query(&req.query)?;
        let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get(collection_id)
            .ok_or_else(|| StorageError::CollectionNotFound(collection_id.to_string()))?;
        coll.check_live(collection_id)?;
        validate_sort_request(coll, collection_id, &req)?;

        let interner = &coll.interner;
        let parsed_cursor = req.cursor.as_deref().and_then(parse_page_cursor);
        let offset = match &parsed_cursor {
            Some(PageCursor::Offset(n)) => *n as usize,
            _ => 0,
        };
        // A sort keyset only continues the single-number-field sorted planner;
        // a score keyset only continues score-ranked pages. A cursor that does
        // not match the request shape degrades to first-page semantics (caller
        // error — cursors are bound to the query that produced them).
        let sort_after = sort_after_for_request(coll, req.sort.as_deref(), &parsed_cursor)?;
        let score_after: Option<(f32, String)> = match &parsed_cursor {
            Some(PageCursor::ScoreKeyset { score_bits, eid }) if req.sort.is_none() => {
                Some((f32::from_bits(*score_bits), eid.clone()))
            }
            _ => None,
        };
        let limit = req.limit as usize;
        let cache_key = search_cache_key(&req)?;
        if let Some(mut cached) = coll.cached_search_response(&cache_key) {
            let el = start.elapsed();
            cached.took_ms = el.as_millis() as u64;
            cached.took_us = el.as_micros() as u64;
            self.metrics.observe_search(cached.took_ms);
            return Ok(cached);
        }

        // Collapse / field-collapse (group-by a keyword field): return ONE hit
        // per distinct value of `collapse`, scored by the MAX member score. The
        // full matched set is needed so groups are complete → this bypasses the
        // paged planner. `hit.external_id` is the collapse value; `total` is the
        // distinct-group count. Drives nested `group` search: filter the child
        // collection, collapse by `parent_row_id` → distinct matching parents
        // (correlation preserved because each child doc is one group element).
        if let Some(collapse_field) = &req.collapse {
            let fi = coll
                .fields
                .get(collapse_field)
                .ok_or_else(|| StorageError::UnknownField {
                    collection: collection_id.to_string(),
                    field: collapse_field.clone(),
                })?;
            let FieldIndex::Keyword(kidx) = fi else {
                bail!(
                    "collapse requires a keyword field (field `{}`)",
                    collapse_field
                );
            };

            // Early-termination: for a constant-score query (no match/knn) with
            // track_total off, drive from the cheapest leaf and collect distinct
            // collapse values until the page is full — never materializing the
            // full matched set (the difference between 37 ms and the floor at
            // 1M). Order is unspecified, like any constant-score filter page.
            if offset == 0 && !req.track_total && query_is_constant_score(&req.query) {
                if let Some(iter) = collapse_driver(coll, &req.query) {
                    let want = limit.max(1);
                    // Collapse values are owned `String` so the source can be the
                    // live `forward` map OR — after a Phase 2f-1 seal-and-drop —
                    // the segment (`keyword_at`), without a borrow tied to the
                    // dropped map. Default build is unaffected (same result set).
                    let mut seen: HashMap<String, ()> = HashMap::new();
                    let mut order: Vec<String> = Vec::with_capacity(want);
                    for doc in iter {
                        if order.len() >= want {
                            break;
                        }
                        if query_predicate(coll, &req.query, doc)? {
                            if let Some(val) = kidx.keyword_at(doc) {
                                if seen.insert(val.clone(), ()).is_none() {
                                    order.push(val);
                                }
                            }
                        }
                    }
                    let hits: Vec<SearchHit> = order
                        .into_iter()
                        .map(|val| SearchHit {
                            external_id: val,
                            score: 1.0,
                        })
                        .collect();
                    let total = hits.len() as u64; // lower bound (track_total=false)
                    let el = start.elapsed();
                    let took_ms = el.as_millis() as u64;
                    self.metrics.observe_search(took_ms);
                    let response = SearchResponse {
                        hits,
                        total,
                        cursor: None,
                        took_ms,
                        took_us: el.as_micros() as u64,
                    };
                    coll.cache_search_response(cache_key.clone(), &response);
                    return Ok(response);
                }
            }

            let universe: BTreeSet<u32> = if query_needs_universe(&req.query) {
                coll.eid_fields.keys().copied().collect()
            } else {
                BTreeSet::new()
            };
            let scored = eval_query(coll, collection_id, &req.query, &universe, &state)?;
            // Group by the collapse value, keeping the max member score. Docs
            // with no value for the collapse field drop out (no group). Owned
            // `String` keys so the value source can be the segment (`keyword_at`)
            // after a Phase 2f-1 seal-and-drop; default build is unaffected.
            let mut groups: HashMap<String, f32> = HashMap::new();
            for (doc, score) in &scored {
                if let Some(val) = kidx.keyword_at(*doc) {
                    let slot = groups.entry(val).or_insert(f32::NEG_INFINITY);
                    if *score > *slot {
                        *slot = *score;
                    }
                }
            }
            let total = groups.len() as u64;
            // Rank groups by score desc, then value asc; partition top-k.
            let cmp = |a: &(String, f32), b: &(String, f32)| {
                b.1.partial_cmp(&a.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.0.cmp(&b.0))
            };
            let mut ranked: Vec<(String, f32)> = groups.into_iter().collect();
            let k = offset.saturating_add(limit);
            if k > 0 && ranked.len() > k {
                ranked.select_nth_unstable_by(k - 1, cmp);
                ranked.truncate(k);
            }
            ranked.sort_by(cmp);
            let hits: Vec<SearchHit> = ranked
                .into_iter()
                .skip(offset)
                .take(limit)
                .map(|(val, score)| SearchHit {
                    external_id: val.to_string(),
                    score,
                })
                .collect();
            let next_offset = offset + hits.len();
            let cursor = if (next_offset as u64) < total {
                Some(make_cursor(next_offset))
            } else {
                None
            };
            let el = start.elapsed();
            let took_ms = el.as_millis() as u64;
            self.metrics.observe_search(took_ms);
            let response = SearchResponse {
                hits,
                total,
                cursor,
                took_ms,
                took_us: el.as_micros() as u64,
            };
            coll.cache_search_response(cache_key.clone(), &response);
            return Ok(response);
        }

        // Planner fast paths (first page): sort-by-field and standalone range
        // early-terminate / avoid materializing a wide clause, returning the
        // final page directly. Everything else falls through to the
        // materialize-and-rank path below (identical result to before).
        // A score keyset bypasses the planner and the bounded top-k fast paths
        // (they collect top-(k) of the WHOLE set; the keyset page is the top of
        // the strictly-after-cursor subset).
        let planned = if score_after.is_some() {
            None
        } else {
            try_plan(coll, &req, offset, sort_after.as_ref())?
        };

        let (page, total, plan_kind): (Vec<(u32, f32)>, u64, PlanKind) = match planned {
            Some(pt) => pt,
            None => {
                // Single-term (and multi-token AND) `match` fast path: score
                // into a bounded top-k heap, skipping the per-doc `HashMap`
                // insert, map→Vec collect, and full matched Vec partition below.
                // Each docid is scored exactly once on these shapes, so the f32
                // bits and the `total` (== unique-docid count) are identical to
                // the map path.
                // Anything else (nested bool, multi-token OR, knn/rrf, …) falls
                // through to the general `eval_query` map path unchanged.
                let mut ranked: Vec<(u32, f32)>;
                let total: u64;
                if score_after.is_some() {
                    // Keyset continuation: rank the strictly-after-cursor subset.
                    let universe: BTreeSet<u32> = if query_needs_universe(&req.query) {
                        coll.eid_fields.keys().copied().collect()
                    } else {
                        BTreeSet::new()
                    };
                    let scored = eval_query(coll, collection_id, &req.query, &universe, &state)?;
                    total = scored.len() as u64;
                    let (after_score, after_eid) = score_after.as_ref().unwrap();
                    ranked = scored
                        .into_iter()
                        .filter(|(id, s)| {
                            *s < *after_score
                                || (*s == *after_score
                                    && interner.resolve(*id) > after_eid.as_str())
                        })
                        .collect();
                } else if let QueryNode::Match(m) = &req.query {
                    if let Some((top, exact_total)) =
                        eval_match_topk(coll, m, interner, offset.saturating_add(limit))?
                    {
                        total = exact_total;
                        ranked = top;
                    } else {
                        let scored =
                            eval_query(coll, collection_id, &req.query, &BTreeSet::new(), &state)?;
                        total = scored.len() as u64;
                        ranked = scored.into_iter().collect();
                    }
                } else if let Some((top, exact_total)) = eval_predicable_and_topk(
                    coll,
                    &req.query,
                    interner,
                    offset.saturating_add(limit),
                )? {
                    total = exact_total;
                    ranked = top;
                } else {
                    // The full eid set ("universe") is only consumed by the `Not`
                    // branch of eval_query; build it only when the query needs it.
                    let universe: BTreeSet<u32> = if query_needs_universe(&req.query) {
                        coll.eid_fields.keys().copied().collect()
                    } else {
                        BTreeSet::new()
                    };
                    let scored = eval_query(coll, collection_id, &req.query, &universe, &state)?;
                    total = scored.len() as u64;
                    ranked = scored.into_iter().collect();
                }

                // Rank by score desc, then external_id asc (tie-break on the
                // resolved string, stable across snapshot rebuilds). Partition
                // the top-k to the front in O(n), then sort just that slice.
                let cmp = |a: &(u32, f32), b: &(u32, f32)| {
                    b.1.partial_cmp(&a.1)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| interner.resolve(a.0).cmp(interner.resolve(b.0)))
                };
                let k = offset.saturating_add(limit);
                if k > 0 && ranked.len() > k {
                    ranked.select_nth_unstable_by(k - 1, cmp);
                    ranked.truncate(k);
                }
                ranked.sort_by(cmp);
                let page = ranked.into_iter().skip(offset).take(limit).collect();
                (page, total, PlanKind::ScoreRanked)
            }
        };

        // Shared response building (resolve dense ids back to external_ids).
        let hits: Vec<SearchHit> = page
            .iter()
            .map(|(id, score)| SearchHit {
                external_id: interner.resolve(*id).to_string(),
                score: *score,
            })
            .collect();

        // Next-page cursor. Keyset-capable pages (sorted-field walks and
        // score-ranked pages) hand out a v2 keyset bound to the LAST hit, so
        // the next page SEEKS instead of skipping — deep pagination cost does
        // not grow with depth. Posting-order planner pages and requests that
        // arrived with a legacy offset cursor keep the offset scheme.
        let used_offset_cursor = matches!(parsed_cursor, Some(PageCursor::Offset(_)));
        let cursor = if hits.is_empty() {
            None
        } else if used_offset_cursor {
            let next_offset = offset + hits.len();
            if (next_offset as u64) < total {
                Some(make_cursor(next_offset))
            } else {
                None
            }
        } else {
            match plan_kind {
                PlanKind::SortedField if hits.len() == limit => {
                    let sort = req.sort.as_deref().expect("sorted plan has sort");
                    let (last_id, _) = *page.last().expect("non-empty page");
                    let values = sort_values_for_doc(coll, sort, last_id)?;
                    match (sort, values) {
                        ([spec], Some(values)) => match coll.fields.get(&spec.field) {
                            Some(FieldIndex::Number(_)) => match values.as_slice() {
                                [SortValue::Number(bits)] => Some(make_sort_cursor(*bits, last_id)),
                                _ => Some(make_sort_values_cursor(&values, last_id)),
                            },
                            _ => Some(make_sort_values_cursor(&values, last_id)),
                        },
                        (_, Some(values)) => Some(make_sort_values_cursor(&values, last_id)),
                        (_, None) => None,
                    }
                }
                PlanKind::ScoreRanked if hits.len() == limit => {
                    let last = hits.last().expect("non-empty hits");
                    Some(make_score_cursor(last.score, &last.external_id))
                }
                PlanKind::Posting => {
                    let next_offset = offset + hits.len();
                    if (next_offset as u64) < total {
                        Some(make_cursor(next_offset))
                    } else {
                        None
                    }
                }
                _ => None, // keyset page shorter than the limit → exhausted
            }
        };

        let el = start.elapsed();
        let took_ms = el.as_millis() as u64;
        self.metrics.observe_search(took_ms);
        let response = SearchResponse {
            hits,
            total,
            cursor,
            took_ms,
            took_us: el.as_micros() as u64,
        };
        coll.cache_search_response(cache_key, &response);
        Ok(response)
    }

    pub(crate) fn search_fast_string_term(
        &self,
        collection_id: &str,
        field: &str,
        value: &str,
        limit: u32,
    ) -> Result<SearchResponse> {
        let start = Instant::now();
        let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get(collection_id)
            .ok_or_else(|| StorageError::CollectionNotFound(collection_id.to_string()))?;
        coll.check_live(collection_id)?;

        let posting: Option<std::borrow::Cow<RoaringBitmap>> = match coll.fields.get(field) {
            Some(FieldIndex::Keyword(k)) => k.term_postings(value),
            Some(FieldIndex::Set(s)) => s.element_postings(value),
            Some(_) => bail!("term query type mismatch on field `{field}`"),
            None => {
                return Err(StorageError::UnknownField {
                    collection: collection_id.to_string(),
                    field: field.to_string(),
                }
                .into());
            }
        };
        let limit = limit as usize;
        let total = posting.as_deref().map(|p| p.len()).unwrap_or(0);
        let hits: Vec<SearchHit> = posting
            .as_deref()
            .into_iter()
            .flat_map(|p| p.iter())
            .take(limit)
            .map(|id| SearchHit {
                external_id: coll.interner.resolve(id).to_string(),
                score: 1.0,
            })
            .collect();
        let cursor = if hits.len() < total as usize {
            Some(make_cursor(hits.len()))
        } else {
            None
        };

        let el = start.elapsed();
        let took_ms = el.as_millis() as u64;
        self.metrics.observe_search(took_ms);
        Ok(SearchResponse {
            hits,
            total,
            cursor,
            took_ms,
            took_us: el.as_micros() as u64,
        })
    }

    // -- Duplicates ---------------------------------------------------------

    pub fn duplicates(
        &self,
        collection_id: &str,
        req: DuplicatesRequest,
    ) -> Result<DuplicatesResponse> {
        let start = Instant::now();
        let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get(collection_id)
            .ok_or_else(|| StorageError::CollectionNotFound(collection_id.to_string()))?;
        coll.check_live(collection_id)?;
        let fi = coll
            .fields
            .get(&req.field)
            .ok_or_else(|| StorageError::UnknownField {
                collection: collection_id.to_string(),
                field: req.field.clone(),
            })?;

        let min = req.min_group_size.max(2) as usize;
        let interner = &coll.interner;
        // Phase 1: collect candidate groups as (value, posting) WITHOUT
        // materializing external ids. At high duplicate density the id strings
        // dominate the cost, and only one `limit` page of them is returned —
        // so resolution is deferred until after sort + paging (phase 2).
        use std::borrow::Cow;
        let mut cands: Vec<(serde_json::Value, Cow<'_, RoaringBitmap>)> = match fi {
            FieldIndex::Text { .. } => {
                return Err(StorageError::DuplicatesOnText(req.field.clone()).into());
            }
            FieldIndex::Vector { .. } => {
                bail!(
                    "duplicates is not supported on vector field `{}` — use knn instead",
                    req.field
                );
            }
            FieldIndex::Hash(_) => {
                bail!(
                    "duplicates is not supported on hash field `{}` — use a hamming query instead",
                    req.field
                );
            }
            FieldIndex::Keyword(k) => {
                // Phase 2h-1 FIX: a SEALED Keyword field dropped its in-RAM
                // `terms` driver, so iterating it directly would miss every
                // on-disk term (and report deleted docs for the tail-only
                // case). Drive from the segment-aware `live_terms` — segment
                // dict minus tombstones + live tail. Segment OFF: the
                // `dup_values` side-index already names every term with >= 2
                // docs, so only candidate groups are visited — not every
                // distinct term, and no whole-index clone.
                if k.segment.is_some() {
                    k.live_terms()
                        .into_iter()
                        .filter(|(_, set)| (set.len() as usize) >= min)
                        .map(|(v, set)| (serde_json::Value::String(v), Cow::Owned(set)))
                        .collect()
                } else {
                    k.dup_values
                        .iter()
                        .filter_map(|v| k.terms.get(v).map(|set| (v, set)))
                        .filter(|(_, set)| (set.len() as usize) >= min)
                        .map(|(v, set)| (serde_json::Value::String(v.clone()), Cow::Borrowed(set)))
                        .collect()
                }
            }
            FieldIndex::Number(n) => {
                // Phase 2h-3 FIX: a SEALED Number field dropped its in-RAM
                // `values` driver, so iterating it directly would miss every
                // on-disk value (and report deleted docs for the tail-only
                // case). Drive from the segment-aware `live_values` — segment
                // sorted-value column minus tombstones + live tail. Segment
                // OFF: the `dup_values` side-index names every value with
                // >= 2 docs — no full scan, no whole-index clone.
                let num = |v: SortableF64| {
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(v.to_f64())
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    )
                };
                if n.segment.is_some() {
                    n.live_values()
                        .into_iter()
                        .filter(|(_, set)| (set.len() as usize) >= min)
                        .map(|(v, set)| (num(v), Cow::Owned(set)))
                        .collect()
                } else {
                    n.dup_values
                        .iter()
                        .filter_map(|v| n.values.get(v).map(|set| (*v, set)))
                        .filter(|(_, set)| (set.len() as usize) >= min)
                        .map(|(v, set)| (num(v), Cow::Borrowed(set)))
                        .collect()
                }
            }
            FieldIndex::Set(s) => {
                // Phase 2h-2 FIX: a SEALED Set field dropped its in-RAM `elements`
                // driver, so iterating it directly would miss every on-disk element
                // (and report deleted docs for the tail-only case). Drive from the
                // segment-aware `live_elements` — segment dict minus tombstones +
                // live tail. Segment OFF: the `dup_values` side-index names every
                // element with >= 2 docs — no full scan, no whole-index clone.
                if s.segment.is_some() {
                    s.live_elements()
                        .into_iter()
                        .filter(|(_, set)| (set.len() as usize) >= min)
                        .map(|(v, set)| (serde_json::Value::String(v), Cow::Owned(set)))
                        .collect()
                } else {
                    s.dup_values
                        .iter()
                        .filter_map(|v| s.elements.get(v).map(|set| (v, set)))
                        .filter(|(_, set)| (set.len() as usize) >= min)
                        .map(|(v, set)| (serde_json::Value::String(v.clone()), Cow::Borrowed(set)))
                        .collect()
                }
            }
        };
        // Stable: largest groups first, ties broken by value (JSON form) — the
        // same order the materialized sort produced before paging moved here.
        cands.sort_by_cached_key(|(v, set)| (std::cmp::Reverse(set.len()), v.to_string()));

        let offset = req.offset as usize;
        let limit = req.limit.max(1) as usize;
        let total = cands.len();
        // Phase 2: resolve external ids for the requested page ONLY.
        let page: Vec<DuplicateGroup> = cands
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|(value, set)| DuplicateGroup {
                value,
                external_ids: set
                    .iter()
                    .map(|id| interner.resolve(id).to_string())
                    .collect(),
            })
            .collect();
        let truncated = offset + page.len() < total;

        self.metrics.incr_duplicates();
        Ok(DuplicatesResponse {
            groups: page,
            truncated,
            took_ms: start.elapsed().as_millis() as u64,
        })
    }

    // -- Stats --------------------------------------------------------------

    // -- Backup / restore ---------------------------------------------------

    /// Snapshot every collection's in-memory state. The result is a
    /// fully self-contained, deterministically-orderable JSON document
    /// that `restore` can replay into a fresh `Engine`.
    pub fn snapshot(&self) -> Result<SnapshotV1> {
        let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
        let mut collections = BTreeMap::new();
        for (id, coll) in &state.collections {
            collections.insert(id.clone(), coll.to_snapshot()?);
        }
        Ok(SnapshotV1 {
            version: SNAPSHOT_VERSION,
            collections,
        })
    }

    /// Atomically replace the engine's state with the given snapshot.
    /// Idempotency keys are not part of the snapshot — restored
    /// collections start with an empty deduplication window.
    pub fn restore(&self, snap: SnapshotV1) -> Result<()> {
        if snap.version != SNAPSHOT_VERSION {
            bail!(
                "snapshot version mismatch: got {}, supported {}",
                snap.version,
                SNAPSHOT_VERSION
            );
        }
        let collections: BTreeMap<String, Collection> = snap
            .collections
            .into_iter()
            .map(|(id, snap)| Collection::from_snapshot(snap).map(|c| (id, c)))
            .collect::<Result<_>>()?;
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        state.collections = collections;
        Ok(())
    }

    pub fn stats(&self, collection_id: &str) -> Result<StatsResponse> {
        let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get(collection_id)
            .ok_or_else(|| StorageError::CollectionNotFound(collection_id.to_string()))?;
        coll.check_live(collection_id)?;

        let fields: BTreeMap<String, FieldStats> = coll
            .fields
            .iter()
            .map(|(name, fi)| {
                let stats = FieldStats {
                    field_type: fi.field_type(),
                    unique_terms: fi.unique_terms(),
                    bytes: fi.bytes(),
                    avg_doc_len: fi.avg_doc_len(),
                };
                (name.clone(), stats)
            })
            .collect();

        let total_bytes: u64 = fields.values().map(|s| s.bytes).sum();
        self.metrics.set_storage_bytes(total_bytes);

        let last_indexed_at = coll.last_indexed_at.map(|t| {
            let dt: chrono::DateTime<chrono::Utc> = t.into();
            dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        });

        Ok(StatsResponse {
            documents_indexed: coll.eid_fields.len() as u64,
            fields,
            storage: StorageStats { total_bytes },
            cache: CacheStats {
                // In-memory engine has no posting-list cache layer;
                // when the LSM backend is attached this will report
                // the moka byte-weighted hit ratio.
                posting_hit_ratio: 1.0,
            },
            last_indexed_at,
        })
    }

    // -- Raft state-machine boundary ---------------------------------------

    /// Apply a single committed Raft log entry to the engine.
    ///
    /// This is the state-machine boundary used by `raft_node.rs`: every
    /// write op that goes through Raft consensus arrives here once it has
    /// been committed, and is dispatched to the same internal method the
    /// HTTP API layer would call. Read ops never flow through this path.
    ///
    /// Errors from the underlying methods are surfaced unchanged so the
    /// caller (the Raft state-machine impl) can log/translate them. The
    /// engine's RwLock is taken per call, identical to a direct API call.
    pub fn apply_raft_entry(&self, entry: crate::log_entry::RaftLogEntry) -> Result<ApplyOutcome> {
        use crate::log_entry::RaftLogEntry;
        Ok(match entry {
            RaftLogEntry::CreateCollection { collection_id, req } => {
                ApplyOutcome::Created(self.create_collection(&collection_id, req)?)
            }
            RaftLogEntry::Index { collection_id, req } => {
                ApplyOutcome::Indexed(self.index(&collection_id, req)?)
            }
            RaftLogEntry::Delete {
                collection_id,
                external_id,
                field,
            } => {
                self.delete(&collection_id, &external_id, field.as_deref())?;
                ApplyOutcome::Deleted
            }
            RaftLogEntry::DropCollection {
                collection_id,
                force,
            } => ApplyOutcome::Dropped(self.drop_collection(&collection_id, force)?),
            RaftLogEntry::AddField {
                collection_id,
                field_name,
                spec,
            } => ApplyOutcome::FieldChanged(self.add_field(&collection_id, &field_name, spec)?),
            RaftLogEntry::DropField {
                collection_id,
                field_name,
            } => ApplyOutcome::FieldChanged(self.drop_field(&collection_id, &field_name)?),
        })
    }
}

/// Result of applying one mutation — routed from the apply loop back to
/// the waiting write handler (by sequence) so the HTTP response keeps
/// its rich shape even though apply happens in the subscribe layer.
#[derive(Debug, Clone)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
pub enum ApplyOutcome {
    Created(CreateCollectionResponse),
    Indexed(IndexResponse),
    Deleted,
    Dropped(DropOutcome),
    /// New collection version after add-field / drop-field.
    FieldChanged(u32),
}

// ---------------------------------------------------------------------------
// Schema validation
// ---------------------------------------------------------------------------

fn validate_schema(schema: &BTreeMap<String, FieldSpec>) -> Result<()> {
    for (name, spec) in schema {
        if name.is_empty() {
            bail!("field name cannot be empty");
        }
        if matches!(spec.field_type, FieldType::Text) && spec.analyzer.is_none() {
            bail!("text field `{name}` is missing analyzer (normalize() should default it)");
        }
        if !matches!(spec.field_type, FieldType::Text) && spec.analyzer.is_some() {
            bail!(
                "field `{name}` of type {:?} does not accept an analyzer",
                spec.field_type
            );
        }
        // Reject vector-specific fields on non-vector field types so
        // typos like `{type: "keyword", dim: 768}` fail loudly.
        if !matches!(spec.field_type, FieldType::Vector) {
            if spec.dim.is_some()
                || spec.metric.is_some()
                || spec.backend.is_some()
                || spec.quantize.is_some()
            {
                bail!(
                    "field `{name}` of type {:?} does not accept vector spec keys (dim/metric/backend/quantize)",
                    spec.field_type
                );
            }
        } else {
            // Eager validation so bad vector specs fail at schema time
            // rather than at index time.
            spec.vector_spec()
                .map_err(|e| anyhow!("vector field `{name}`: {e}"))?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Value application
// ---------------------------------------------------------------------------

fn apply_value(
    fi: &mut FieldIndex,
    id: u32,
    eid: &str,
    value: &FieldValue,
    field_name: &str,
) -> Result<u64> {
    match (fi, value) {
        (FieldIndex::Text { analyzer, idx }, FieldValue::String(s)) => {
            let mut bytes = 0u64;
            let mut distinct = TokenSet::default();
            let mut apply_token = |idx: &mut TextIndex, token: &str| {
                if distinct.insert_str(token) {
                    bytes += (token.len() + eid.len()) as u64;
                }
                if let Some(posting) = idx.tokens.get_mut(token) {
                    posting.upsert_add(id, 1);
                } else {
                    let mut posting = Postings::default();
                    posting.upsert(id, 1);
                    idx.tokens.insert(token.to_string(), posting);
                }
            };
            let doc_len = match analyzer {
                Analyzer::WhitespaceLower => tokenize::for_whitespace_lower_cow(&s, |tok| {
                    apply_token(idx, tok.as_ref());
                }),
                _ => {
                    let tokens = tokenize::tokenize(&s, *analyzer);
                    let doc_len = tokens.len() as u32;
                    for tok in &tokens {
                        apply_token(idx, tok);
                    }
                    doc_len
                }
            };
            idx.set_doc_len(id, doc_len);
            idx.set_distinct(id, distinct);
            idx.doc_count += 1;
            idx.total_doc_len += doc_len as u64;
            idx.bytes += bytes;
            Ok(bytes)
        }
        (FieldIndex::Keyword(k), FieldValue::String(s)) => {
            let bytes = (s.len() + eid.len()) as u64;
            if let Some(posting) = k.terms.get_mut(s) {
                posting.insert(id);
                if posting.len() == 2 {
                    k.dup_values.insert(s.clone());
                }
            } else {
                let mut posting = RoaringBitmap::new();
                posting.insert(id);
                k.terms.insert(s.clone(), posting);
            }
            k.set_keyword(id, s.clone());
            k.bytes += bytes;
            Ok(bytes)
        }
        (FieldIndex::Number(n), FieldValue::Number(x)) => {
            let key =
                SortableF64::new(*x).map_err(|e| StorageError::InvalidNumber(e.to_string()))?;
            let bytes = (8 + eid.len()) as u64;
            let posting = n.values.entry(key).or_default();
            posting.insert(id);
            if posting.len() == 2 {
                n.dup_values.insert(key);
            }
            n.set_number(id, key);
            n.bytes += bytes;
            Ok(bytes)
        }
        (FieldIndex::Set(s), FieldValue::StringList(elems)) => {
            let mut bytes = 0u64;
            let mut seen = BTreeSet::new();
            for el in elems {
                if seen.insert(el.clone()) {
                    bytes += (el.len() + eid.len()) as u64;
                    if let Some(posting) = s.elements.get_mut(el) {
                        posting.insert(id);
                        if posting.len() == 2 {
                            s.dup_values.insert(el.clone());
                        }
                    } else {
                        let mut posting = RoaringBitmap::new();
                        posting.insert(id);
                        s.elements.insert(el.clone(), posting);
                    }
                }
            }
            s.forward.insert(id, seen);
            s.bytes += bytes;
            Ok(bytes)
        }
        // Permitted coercions: set field accepts a single string as a
        // 1-element set.
        (FieldIndex::Set(_), FieldValue::String(_)) => Err(StorageError::TypeMismatch {
            field: field_name.to_string(),
            expected: FieldType::Set,
            got: "string (expected array of strings)",
        }
        .into()),
        (FieldIndex::Vector { spec, idx, bytes }, FieldValue::Vector(v)) => {
            if v.len() as u32 != spec.dim {
                bail!(
                    "vector field `{field_name}` declared dim={} but got vector of length {}",
                    spec.dim,
                    v.len()
                );
            }
            idx.add(eid, v)?;
            let approx = (spec.dim as u64) * 4 + eid.len() as u64;
            *bytes += approx;
            Ok(approx)
        }
        (FieldIndex::Hash(h), FieldValue::String(s)) => {
            let hash = parse_hash(s)?;
            let bytes = 12u64; // u32 docid + u64 hash
            h.forward.insert(id, hash);
            h.bytes += bytes;
            Ok(bytes)
        }
        (fi, v) => Err(StorageError::TypeMismatch {
            field: field_name.to_string(),
            expected: fi.field_type(),
            got: value_kind(v),
        }
        .into()),
    }
}

fn value_kind(v: &FieldValue) -> &'static str {
    match v {
        FieldValue::String(_) => "string",
        FieldValue::Number(_) => "number",
        FieldValue::Vector(_) => "f32[]",
        FieldValue::StringList(_) => "string[]",
    }
}

// ---------------------------------------------------------------------------
// Query evaluation
// ---------------------------------------------------------------------------

/// Map of `external_id` → score. Non-text leaves return a constant
/// score of `1.0`; only `match` leaves emit BM25 scores. Combinators
/// sum child scores so an AND over (match, term) ranks documents whose
/// text relevance is highest among the eligible set.
///
/// Backed by a `HashMap` (not `BTreeMap`): the final page is re-sorted by
/// `(−score, external_id)` in `search`, so iteration order here is
/// irrelevant, and O(1) inserts beat O(log n) for the large matched sets a
/// broad query produces.
type ScoredHits = HashMap<u32, f32>;

/// Query-shape safety bounds (DoS guards). A query is validated before
/// evaluation so a pathological tree can never exhaust the stack (deep
/// nesting) or CPU/memory (very wide trees / huge fan-outs).
const MAX_QUERY_DEPTH: usize = 32;
const MAX_QUERY_NODES: usize = 1024;
const MAX_TERMS_VALUES: usize = 1024;
const MAX_KNN_K: u32 = 10_000;

/// Reject pathological queries with a clear error. Traversal is **iterative**
/// (explicit stack) so validating a deeply-nested tree cannot itself overflow
/// the stack. Bounds: nesting depth, total node count, `terms` fan-out, `knn` k.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
pub fn validate_query(root: &QueryNode) -> std::result::Result<(), StorageError> {
    let mut stack: Vec<(&QueryNode, usize)> = vec![(root, 1)];
    let mut nodes = 0usize;
    while let Some((node, depth)) = stack.pop() {
        nodes += 1;
        if depth > MAX_QUERY_DEPTH {
            return Err(StorageError::QueryTooComplex(format!(
                "nesting depth exceeds {MAX_QUERY_DEPTH}"
            )));
        }
        if nodes > MAX_QUERY_NODES {
            return Err(StorageError::QueryTooComplex(format!(
                "node count exceeds {MAX_QUERY_NODES}"
            )));
        }
        match node {
            QueryNode::And(children) | QueryNode::Or(children) => {
                for c in children {
                    stack.push((c, depth + 1));
                }
            }
            QueryNode::Not(child) => stack.push((child, depth + 1)),
            QueryNode::HasChild(hc) => stack.push((&hc.query, depth + 1)),
            QueryNode::Rrf(r) => {
                for c in &r.queries {
                    stack.push((c, depth + 1));
                }
            }
            QueryNode::Terms(t) if t.values.len() > MAX_TERMS_VALUES => {
                return Err(StorageError::QueryTooComplex(format!(
                    "terms value count {} exceeds {MAX_TERMS_VALUES}",
                    t.values.len()
                )));
            }
            QueryNode::Knn(k) if k.k > MAX_KNN_K => {
                return Err(StorageError::QueryTooComplex(format!(
                    "knn k={} exceeds {MAX_KNN_K}",
                    k.k
                )));
            }
            _ => {}
        }
    }
    Ok(())
}

/// Whether evaluating `q` actually needs the full eid universe. It is used in
/// exactly two spots: the `Not` arm (a negation reached top-level, via `Or`,
/// or as a `Not`'s inner) and an all-negative `And`. A `Not` that is a direct
/// conjunct of an `And` is applied as a *filter* and needs no universe, so the
/// common `And[positive…, Not[…]]` shape skips the O(N) clone. This mirrors
/// `eval_query` exactly — keep the two in sync.
fn query_needs_universe(q: &QueryNode) -> bool {
    match q {
        QueryNode::Not(_) => true,
        QueryNode::Or(children) => children.iter().any(query_needs_universe),
        QueryNode::Rrf(r) => r.queries.iter().any(query_needs_universe),
        QueryNode::And(children) => {
            let (nots, positives): (Vec<&QueryNode>, Vec<&QueryNode>) = children
                .iter()
                .partition(|c| matches!(c, QueryNode::Not(_)));
            if positives.is_empty() {
                // Empty AND → empty set (no universe); all-negative AND needs it.
                return !nots.is_empty();
            }
            positives.iter().any(|c| query_needs_universe(c))
                || nots.iter().any(|n| match n {
                    // The Not is a filter; only its inner can pull in a universe.
                    QueryNode::Not(inner) => query_needs_universe(inner),
                    _ => false,
                })
        }
        _ => false,
    }
}

fn eval_query(
    coll: &Collection,
    collection_id: &str,
    q: &QueryNode,
    universe: &BTreeSet<u32>,
    state: &EngineState,
) -> Result<ScoredHits> {
    Ok(match q {
        QueryNode::Match(m) => eval_match(coll, m)?,
        QueryNode::Term(t) => constant_score(eval_term(coll, t)?),
        QueryNode::Terms(t) => constant_score(eval_terms(coll, t)?),
        QueryNode::Range(r) => constant_score(eval_range(coll, r)?),
        QueryNode::Knn(k) => eval_knn(coll, k)?,
        QueryNode::Hamming(hq) => eval_hamming(coll, hq)?,
        QueryNode::Exists(e) => constant_score(eval_field_doc_union(coll, &e.field, 1)?),
        QueryNode::Duplicated(d) => constant_score(eval_field_doc_union(
            coll,
            &d.field,
            d.min_group_size.max(2) as u64,
        )?),
        QueryNode::Rrf(r) => eval_rrf(coll, collection_id, r, universe, state)?,
        QueryNode::And(children) => {
            // A `Not` conjunct is a filter: `A AND NOT B = A \ B`.
            let (nots, positives): (Vec<&QueryNode>, Vec<&QueryNode>) = children
                .iter()
                .partition(|c| matches!(c, QueryNode::Not(_)));

            if positives.is_empty() && nots.is_empty() {
                return Ok(HashMap::new()); // empty AND → empty set
            }

            // Filter-correct kNN. If a `knn` clause is conjoined with anything
            // else, evaluate the NON-knn part to an allow-set first, then drive
            // each knn THROUGH it (allow-list) — so the result is the nearest-k
            // WITHIN the filtered set, not a post-filter over the global top-k
            // (which collapses recall the way pgvector does). knn is never
            // `is_predicable`, so without this it would fall through to the
            // materialize-and-intersect path and post-filter.
            if positives.iter().any(|c| matches!(c, QueryNode::Knn(_))) {
                let knn_qs: Vec<&KnnQuery> = positives
                    .iter()
                    .copied()
                    .filter_map(|c| match c {
                        QueryNode::Knn(k) => Some(k),
                        _ => None,
                    })
                    .collect();
                let rest_children: Vec<QueryNode> = children
                    .iter()
                    .filter(|c| !matches!(c, QueryNode::Knn(_)))
                    .cloned()
                    .collect();

                // Allow-set + base scores from every non-knn conjunct (filters,
                // matches, has_child, negations). No non-knn conjunct → the knn
                // is unconstrained (degenerate `And(knn)` / `And(knn, knn)`).
                let (allow, base_scores): (Option<RoaringBitmap>, ScoredHits) =
                    if rest_children.is_empty() {
                        (None, HashMap::new())
                    } else {
                        let rest = eval_query(
                            coll,
                            collection_id,
                            &QueryNode::And(rest_children),
                            universe,
                            state,
                        )?;
                        let bm: RoaringBitmap = rest.keys().copied().collect();
                        (Some(bm), rest)
                    };

                // Each knn driven through the allow-set; intersect multiple knns.
                let mut knn_acc: Option<ScoredHits> = None;
                for kq in &knn_qs {
                    let hits = match &allow {
                        Some(bm) => eval_knn_filtered(coll, kq, bm)?,
                        None => eval_knn(coll, kq)?,
                    };
                    knn_acc = Some(match knn_acc {
                        None => hits,
                        Some(prev) => prev
                            .into_iter()
                            .filter_map(|(id, s)| hits.get(&id).map(|h| (id, s + h)))
                            .collect(),
                    });
                }
                let knn_hits = knn_acc.unwrap_or_default();

                // AND-combine: a hit must satisfy both the knn and the non-knn
                // part; score = base (non-knn contributions) + knn similarity.
                return Ok(match &allow {
                    None => knn_hits,
                    Some(_) => {
                        let mut out = ScoredHits::new();
                        for (id, kscore) in knn_hits {
                            if let Some(base) = base_scores.get(&id) {
                                out.insert(id, base + kscore);
                            }
                        }
                        out
                    }
                });
            }

            // Planner fast path. Evaluate the AND without materializing a wide
            // clause:
            //   * ≥1 filter conjunct (term/terms/range) → INTERSECT their
            //     RoaringBitmaps (compressed-SIMD AND, smallest first), subtract
            //     filter negations, then score `match` conjuncts over the small
            //     candidate set;
            //   * match-only positives → drive from the cheapest match
            //     (bulk-scored), predicate-filter the rest.
            // Score = sum of every conjunct's contribution → byte-identical to
            // the fallback.
            let predicable = !positives.is_empty()
                && positives.iter().all(|c| is_predicable(c))
                && nots.iter().all(|c| {
                    let QueryNode::Not(inner) = c else {
                        unreachable!()
                    };
                    is_predicable(inner)
                });

            if predicable {
                let filter_pos: Vec<&QueryNode> = positives
                    .iter()
                    .copied()
                    .filter(|c| !matches!(c, QueryNode::Match(_)))
                    .collect();
                let match_pos: Vec<&QueryNode> = positives
                    .iter()
                    .copied()
                    .filter(|c| matches!(c, QueryNode::Match(_)))
                    .collect();
                let (mut filter_nots, mut match_nots): (Vec<&QueryNode>, Vec<&QueryNode>) =
                    (Vec::new(), Vec::new());
                for c in &nots {
                    let QueryNode::Not(inner) = c else {
                        unreachable!()
                    };
                    if matches!(&**inner, QueryNode::Match(_)) {
                        match_nots.push(inner);
                    } else {
                        filter_nots.push(inner);
                    }
                }

                // Drive from the globally cheapest POSITIVE. Cheapest is a filter
                // → intersect filter bitmaps, then score matches over the small
                // candidate. Cheapest is a match (e.g. a rare keyword vs a wide
                // range — `price 1000-5000 AND name has "手機殼"`) → drive from
                // that match's scored posting and apply the filters as per-doc
                // predicates, so the match is never scored over a wide filter's
                // worth of docs.
                let min_filter_sel = filter_pos
                    .iter()
                    .map(|c| estimate_selectivity(coll, c))
                    .min();
                let min_match_sel = match_pos
                    .iter()
                    .map(|c| estimate_selectivity(coll, c))
                    .min();
                let use_bitmap = !filter_pos.is_empty()
                    && match min_match_sel {
                        Some(m) => min_filter_sel.map(|f| f <= m).unwrap_or(true),
                        None => true,
                    };

                if use_bitmap {
                    let cand = eval_filter_bitmap_conjunction(coll, &filter_pos, &filter_nots)?;
                    // Each filter / negation contributes a constant 1.0; matches
                    // add BM25 on top and gate membership.
                    let base = filter_pos.len() as f32 + nots.len() as f32;
                    let preps = prep_matches(coll, &match_pos)?;
                    let not_preps = prep_matches(coll, &match_nots)?;
                    // Phase 2m: RESOLVE each match's per-token postings ONCE (dict
                    // binary-search + cache fetch paid here, not per candidate doc).
                    // On disk this is THE `filtered_search` fix — the candidate set
                    // can be thousands of docs and the old per-doc `match_doc_score`
                    // re-resolved every token's posting for each, decoding/cloning a
                    // wide posting thousands of times. The prepared postings borrow
                    // the field index for the whole `'doc` loop. Byte-identical
                    // scores; only the resolution is hoisted.
                    let prepared: Vec<Option<PreparedMatch>> = preps
                        .iter()
                        .map(|(idx, toks, op)| PreparedMatch::resolve(idx, toks, *op))
                        .collect();
                    let not_prepared: Vec<Option<PreparedMatch>> = not_preps
                        .iter()
                        .map(|(idx, toks, op)| PreparedMatch::resolve(idx, toks, *op))
                        .collect();
                    let mut acc = ScoredHits::new();
                    'doc: for id in &cand {
                        let mut score = base;
                        for ((idx, _, _), pm) in preps.iter().zip(prepared.iter()) {
                            // `None` prepared ⇒ the corpus/tokens were empty, which
                            // `match_doc_score` reported as no-match → skip the doc.
                            let s = match pm {
                                Some(pm) => pm.score(idx, id),
                                None => None,
                            };
                            match s {
                                Some(s) => score += s,
                                None => continue 'doc,
                            }
                        }
                        for ((idx, _, _), pm) in not_preps.iter().zip(not_prepared.iter()) {
                            let s = match pm {
                                Some(pm) => pm.score(idx, id),
                                None => None,
                            };
                            if s.is_some() {
                                continue 'doc;
                            }
                        }
                        acc.insert(id, score);
                    }
                    acc
                } else {
                    // Match-only positives: drive from the cheapest match
                    // (materialized + bulk-scored), then predicate-filter.
                    let driver_ix = match_pos
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, c)| estimate_selectivity(coll, c))
                        .map(|(i, _)| i)
                        .expect("match_pos non-empty when filter_pos empty");
                    let driver = match_pos[driver_ix];
                    let others: Vec<&QueryNode> = match_pos
                        .iter()
                        .copied()
                        .enumerate()
                        .filter(|(i, _)| *i != driver_ix)
                        .map(|(_, c)| c)
                        .collect();
                    let other_matches = prep_matches(coll, &others)?;
                    let not_inners: Vec<&QueryNode> = nots
                        .iter()
                        .copied()
                        .map(|n| match n {
                            QueryNode::Not(inner) => &**inner,
                            _ => unreachable!(),
                        })
                        .collect();
                    // Drive from the cheapest match; apply the OTHER matches AND
                    // all filters as per-doc predicates (filters contribute their
                    // constant score via clause_matches inside apply_conjuncts).
                    let scored = eval_query(coll, collection_id, driver, universe, state)?;
                    let mut acc = ScoredHits::new();
                    for (id, base) in scored {
                        if let Some(s) = apply_conjuncts(
                            coll,
                            id,
                            base,
                            &filter_pos,
                            &other_matches,
                            &not_inners,
                        )? {
                            acc.insert(id, s);
                        }
                    }
                    acc
                }
            } else {
                // Fallback: materialize-and-intersect.
                let mut pos = positives.iter();
                let mut acc = match pos.next() {
                    Some(first) => eval_query(coll, collection_id, first, universe, state)?,
                    // All-negative AND: start from the universe, then trim.
                    None => constant_score(universe.iter().cloned().collect()),
                };
                for c in pos {
                    let other = eval_query(coll, collection_id, c, universe, state)?;
                    acc = acc
                        .into_iter()
                        .filter_map(|(eid, score)| other.get(&eid).map(|s| (eid, score + s)))
                        .collect();
                    if acc.is_empty() {
                        break;
                    }
                }
                for n in &nots {
                    if acc.is_empty() {
                        break;
                    }
                    let QueryNode::Not(inner) = n else {
                        unreachable!()
                    };
                    let exclude = eval_query(coll, collection_id, inner, universe, state)?;
                    acc.retain(|eid, _| !exclude.contains_key(eid));
                    for s in acc.values_mut() {
                        *s += 1.0;
                    }
                }
                acc
            }
        }
        QueryNode::Or(children) => {
            let mut acc: ScoredHits = HashMap::new();
            for c in children {
                for (eid, score) in eval_query(coll, collection_id, c, universe, state)? {
                    *acc.entry(eid).or_insert(0.0) += score;
                }
            }
            acc
        }
        QueryNode::Not(child) => {
            let inner = eval_query(coll, collection_id, child, universe, state)?;
            universe
                .iter()
                .filter(|id| !inner.contains_key(*id))
                .map(|id| (*id, 1.0))
                .collect()
        }
        QueryNode::HasChild(hc) => eval_has_child(coll, hc, state)?,
    })
}

/// Evaluate a `has_child` clause: run its sub-query on the child collection,
/// map each matching child's `field` (= parent external_id) to a PARENT docid,
/// and return that set as a constant-score result — so it composes under
/// and/or/not in the PARENT query like any other clause. Within-element
/// correlation holds because one child doc is one group element.
fn eval_has_child(
    parent: &Collection,
    hc: &HasChildQuery,
    state: &EngineState,
) -> Result<ScoredHits> {
    let child = state
        .collections
        .get(&hc.collection)
        .ok_or_else(|| StorageError::CollectionNotFound(hc.collection.clone()))?;
    let FieldIndex::Keyword(kidx) =
        child
            .fields
            .get(&hc.field)
            .ok_or_else(|| StorageError::UnknownField {
                collection: hc.collection.clone(),
                field: hc.field.clone(),
            })?
    else {
        bail!(
            "has_child `field` must be a keyword field (`{}` in `{}`)",
            hc.field,
            hc.collection
        );
    };
    let child_universe: BTreeSet<u32> = if query_needs_universe(&hc.query) {
        child.eid_fields.keys().copied().collect()
    } else {
        BTreeSet::new()
    };
    let matched = eval_query(child, &hc.collection, &hc.query, &child_universe, state)?;
    // Distinct parent external_ids → PARENT docids → constant-score set.
    let mut parents = RoaringBitmap::new();
    for (child_doc, _) in &matched {
        // Route through `keyword_at` (segment for sealed ids after a Phase 2f-1
        // seal-and-drop, else the live `forward` tail) so the parent-id lookup
        // survives a dropped forward payload. Default build is unaffected.
        if let Some(pid) = kidx.keyword_at(*child_doc) {
            if let Some(parent_doc) = parent.interner.id(&pid) {
                parents.insert(parent_doc);
            }
        }
    }
    Ok(constant_score(parents))
}

fn constant_score(set: RoaringBitmap) -> ScoredHits {
    set.into_iter().map(|id| (id, 1.0)).collect()
}

fn eval_predicable_and_topk(
    coll: &Collection,
    q: &QueryNode,
    interner: &Interner,
    k: usize,
) -> Result<Option<(Vec<(u32, f32)>, u64)>> {
    let QueryNode::And(children) = q else {
        return Ok(None);
    };
    let (nots, positives): (Vec<&QueryNode>, Vec<&QueryNode>) = children
        .iter()
        .partition(|c| matches!(c, QueryNode::Not(_)));
    if positives.is_empty()
        || !positives.iter().all(|c| is_predicable(c))
        || !nots.iter().all(|c| {
            let QueryNode::Not(inner) = c else {
                unreachable!()
            };
            is_predicable(inner)
        })
    {
        return Ok(None);
    }

    let filter_pos: Vec<&QueryNode> = positives
        .iter()
        .copied()
        .filter(|c| !matches!(c, QueryNode::Match(_)))
        .collect();
    let match_pos: Vec<&QueryNode> = positives
        .iter()
        .copied()
        .filter(|c| matches!(c, QueryNode::Match(_)))
        .collect();
    if filter_pos.is_empty() || match_pos.is_empty() {
        return Ok(None);
    }

    let (mut filter_nots, mut match_nots): (Vec<&QueryNode>, Vec<&QueryNode>) =
        (Vec::new(), Vec::new());
    for c in &nots {
        let QueryNode::Not(inner) = c else {
            unreachable!()
        };
        if matches!(&**inner, QueryNode::Match(_)) {
            match_nots.push(inner);
        } else {
            filter_nots.push(inner);
        }
    }

    let min_filter_sel = filter_pos
        .iter()
        .map(|c| estimate_selectivity(coll, c))
        .min();
    let min_match_sel = match_pos
        .iter()
        .map(|c| estimate_selectivity(coll, c))
        .min();
    let use_bitmap = match min_match_sel {
        Some(m) => min_filter_sel.map(|f| f <= m).unwrap_or(true),
        None => true,
    };
    if !use_bitmap {
        return Ok(None);
    }

    let base = filter_pos.len() as f32 + nots.len() as f32;
    if nots.is_empty() && match_pos.len() == 1 {
        if let Some(topk) = eval_single_token_keyword_range_topk(
            coll,
            match_pos[0],
            &filter_pos,
            base,
            interner,
            k,
        )? {
            return Ok(Some(topk));
        }
    }

    let cand = eval_filter_bitmap_conjunction(coll, &filter_pos, &filter_nots)?;

    let preps = prep_matches(coll, &match_pos)?;
    let not_preps = prep_matches(coll, &match_nots)?;
    let prepared: Vec<Option<PreparedMatch>> = preps
        .iter()
        .map(|(idx, toks, op)| PreparedMatch::resolve(idx, toks, *op))
        .collect();
    let not_prepared: Vec<Option<PreparedMatch>> = not_preps
        .iter()
        .map(|(idx, toks, op)| PreparedMatch::resolve(idx, toks, *op))
        .collect();

    if preps.len() == 1 && not_preps.is_empty() {
        if let Some(pm) = &prepared[0] {
            if let Some(topk) =
                pm.score_single_token_candidates_topk(preps[0].0, &cand, base, interner, k)
            {
                return Ok(Some(topk));
            }
        } else {
            return Ok(Some((Vec::new(), 0)));
        }
    }

    let mut total = 0u64;
    let mut heap = BinaryHeap::with_capacity(k.min(cand.len() as usize));
    'doc: for id in &cand {
        let mut score = base;
        for ((idx, _, _), pm) in preps.iter().zip(prepared.iter()) {
            let s = match pm {
                Some(pm) => pm.score(idx, id),
                None => None,
            };
            match s {
                Some(s) => score += s,
                None => continue 'doc,
            }
        }
        for ((idx, _, _), pm) in not_preps.iter().zip(not_prepared.iter()) {
            let s = match pm {
                Some(pm) => pm.score(idx, id),
                None => None,
            };
            if s.is_some() {
                continue 'doc;
            }
        }
        total += 1;
        push_top_ranked(&mut heap, k, interner, id, score);
    }

    Ok(Some((finish_top_ranked(heap), total)))
}

fn eval_single_token_keyword_range_topk(
    coll: &Collection,
    match_node: &QueryNode,
    filters: &[&QueryNode],
    base: f32,
    interner: &Interner,
    k: usize,
) -> Result<Option<(Vec<(u32, f32)>, u64)>> {
    let QueryNode::Match(m) = match_node else {
        return Ok(None);
    };
    if filters.len() != 2 {
        return Ok(None);
    }

    let mut keyword: Option<(&str, &str, &KeywordIndex)> = None;
    let mut range: Option<(
        &NumberIndex,
        std::ops::Bound<SortableF64>,
        std::ops::Bound<SortableF64>,
    )> = None;
    for filter in filters {
        match filter {
            QueryNode::Term(t) => {
                let FieldValue::String(term) = &t.value else {
                    return Ok(None);
                };
                let Some(FieldIndex::Keyword(kidx)) = coll.fields.get(&t.field) else {
                    return Ok(None);
                };
                keyword = Some((t.field.as_str(), term.as_str(), kidx));
            }
            QueryNode::Range(r) => {
                let Some(FieldIndex::Number(nidx)) = coll.fields.get(&r.field) else {
                    return Ok(None);
                };
                let (lo, hi) = range_bounds(r)?;
                range = Some((nidx, lo, hi));
            }
            _ => return Ok(None),
        }
    }
    let (keyword_field, term, keyword_idx) = match keyword {
        Some(v) => v,
        None => return Ok(None),
    };
    let (range_idx, range_lo, range_hi) = match range {
        Some(v) => v,
        None => return Ok(None),
    };

    let fi = coll
        .fields
        .get(&m.field)
        .ok_or_else(|| StorageError::UnknownField {
            collection: "<>".into(),
            field: m.field.clone(),
        })?;
    let FieldIndex::Text { analyzer, idx } = fi else {
        bail!(
            "match query is only valid on text fields (field `{}`)",
            m.field
        );
    };
    let tokens = tokenize::tokenize(&m.text, *analyzer);
    if tokens.len() != 1 {
        return Ok(None);
    }
    let (corpus_n, corpus_total_len) = idx.bm25_corpus();
    if corpus_n == 0 {
        return Ok(Some((Vec::new(), 0)));
    }
    let Some(postings) = idx.tok_postings(&tokens[0]) else {
        return Ok(Some((Vec::new(), 0)));
    };
    let df = postings.df() as f32;
    let n = corpus_n as f32;
    let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
    let avgdl = corpus_total_len as f32 / corpus_n as f32;
    let segment_doc_lens = match &postings {
        TokPostings::Segment(_) => idx.segment.as_ref().and_then(|seg| seg.text_doc_lens()),
        _ => None,
    };

    let docs = range_idx.keyword_range_docs(keyword_field, term, keyword_idx);
    let window = sorted_bits_window(docs.as_slice(), &range_lo, &range_hi);
    if window.end.saturating_sub(window.start) > FILTERED_SEARCH_BITMAP_THRESHOLD {
        let candidates =
            range_idx.keyword_range_bitmap(keyword_field, term, keyword_idx, &range_lo, &range_hi);
        return Ok(Some(score_single_token_candidate_bitmap_topk(
            idx,
            &postings,
            idf,
            avgdl,
            segment_doc_lens,
            candidates.as_ref(),
            base,
            interner,
            k,
        )));
    }
    let mut total = 0u64;
    let mut heap = BinaryHeap::with_capacity(k.min(window.end.saturating_sub(window.start)));
    let mut score_cache = Vec::with_capacity(4);
    for &(_, id) in &docs[window] {
        let Some(tf) = postings.tf(id) else {
            continue;
        };
        let doc_len = text_doc_len_at(idx, segment_doc_lens, id);
        let score = base + bm25_contrib_cached(&mut score_cache, idf, tf, doc_len, avgdl);
        total += 1;
        push_top_ranked(&mut heap, k, interner, id, score);
    }

    Ok(Some((finish_top_ranked(heap), total)))
}

const FILTERED_SEARCH_BITMAP_THRESHOLD: usize = 1024;

fn score_single_token_candidate_bitmap_topk(
    idx: &TextIndex,
    postings: &TokPostings<'_>,
    idf: f32,
    avgdl: f32,
    segment_doc_lens: Option<&[u32]>,
    candidates: &RoaringBitmap,
    base: f32,
    interner: &Interner,
    k: usize,
) -> (Vec<(u32, f32)>, u64) {
    let docids = postings.docids();
    let tfs = postings.tfs();
    let mut cand = candidates.iter();
    let mut next = cand.next();
    let mut total = 0u64;
    let mut heap = BinaryHeap::with_capacity(k.min(candidates.len() as usize));
    let mut score_cache = Vec::with_capacity(4);
    for i in 0..docids.len() {
        let id = docids[i];
        loop {
            match next {
                Some(c) if c < id => next = cand.next(),
                Some(c) if c == id => {
                    let doc_len = text_doc_len_at(idx, segment_doc_lens, id);
                    let score =
                        base + bm25_contrib_cached(&mut score_cache, idf, tfs[i], doc_len, avgdl);
                    total += 1;
                    push_top_ranked(&mut heap, k, interner, id, score);
                    next = cand.next();
                    break;
                }
                Some(_) => break,
                None => return (finish_top_ranked(heap), total),
            }
        }
    }
    (finish_top_ranked(heap), total)
}

const BM25_K1: f32 = 1.2;
const BM25_B: f32 = 0.75;

/// One token's BM25 contribution for a single doc — the LITERAL expression
/// shared by every match path so the four scoring sites (the live/segment map
/// build in `eval_match`, its AND branch, and the `Vec` fast path
/// `eval_match_vec`) compute bit-identical f32s. Do NOT reassociate
/// `tf + K1*(1.0 - B + B*doc_len/avgdl)` or reorder the final `/`.
#[inline(always)]
fn bm25_contrib(idf: f32, tf: f32, doc_len: f32, avgdl: f32) -> f32 {
    let denom = tf + BM25_K1 * (1.0 - BM25_B + BM25_B * doc_len / avgdl);
    idf * tf * (BM25_K1 + 1.0) / denom
}

#[inline]
fn bm25_contrib_cached(
    cache: &mut Vec<(u32, u32, f32)>,
    idf: f32,
    tf: u32,
    doc_len: u32,
    avgdl: f32,
) -> f32 {
    for &(cached_tf, cached_len, score) in cache.iter() {
        if cached_tf == tf && cached_len == doc_len {
            return score;
        }
    }
    let score = bm25_contrib(idf, tf as f32, doc_len as f32, avgdl);
    if cache.len() < 32 {
        cache.push((tf, doc_len, score));
    }
    score
}

#[inline]
fn text_doc_len_at(idx: &TextIndex, segment_doc_lens: Option<&[u32]>, id: u32) -> u32 {
    segment_doc_lens
        .and_then(|lens| lens.get(id as usize).copied())
        .unwrap_or_else(|| idx.doc_len(id))
}

/// Heap entry for bounded top-k ranking. `BinaryHeap` keeps the greatest item at
/// the top, so this `Ord` intentionally makes the *worst* retained hit greatest:
/// lower score is worse; for an equal score, larger external_id is worse.
#[derive(Debug)]
struct TopRankedHit {
    id: u32,
    score: f32,
    external_id: String,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl PartialEq for TopRankedHit {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.score.to_bits() == other.score.to_bits()
            && self.external_id == other.external_id
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl Eq for TopRankedHit {}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl Ord for TopRankedHit {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        match self.score.total_cmp(&other.score) {
            CmpOrdering::Less => CmpOrdering::Greater,
            CmpOrdering::Greater => CmpOrdering::Less,
            CmpOrdering::Equal => self.external_id.cmp(&other.external_id),
        }
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl PartialOrd for TopRankedHit {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

#[inline]
fn push_top_ranked(
    heap: &mut BinaryHeap<TopRankedHit>,
    k: usize,
    interner: &Interner,
    id: u32,
    score: f32,
) {
    if k == 0 {
        return;
    }
    if heap.len() < k {
        heap.push(TopRankedHit {
            id,
            score,
            external_id: interner.resolve(id).to_string(),
        });
        return;
    }

    let Some(worst) = heap.peek() else {
        return;
    };
    let better_than_worst = match score.total_cmp(&worst.score) {
        CmpOrdering::Greater => true,
        CmpOrdering::Less => false,
        CmpOrdering::Equal => interner.resolve(id) < worst.external_id.as_str(),
    };
    if better_than_worst {
        heap.pop();
        heap.push(TopRankedHit {
            id,
            score,
            external_id: interner.resolve(id).to_string(),
        });
    }
}

fn finish_top_ranked(heap: BinaryHeap<TopRankedHit>) -> Vec<(u32, f32)> {
    let mut ranked = heap.into_vec();
    ranked.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(CmpOrdering::Equal)
            .then_with(|| a.external_id.cmp(&b.external_id))
    });
    ranked.into_iter().map(|h| (h.id, h.score)).collect()
}

fn match_rank_key(op: MatchOp, tokens: &[String]) -> String {
    let mut key = match op {
        MatchOp::And => String::from("and"),
        MatchOp::Or => String::from("or"),
    };
    for token in tokens {
        key.push('\0');
        key.push_str(token);
    }
    key
}

fn cached_match_ranked_page(
    idx: &TextIndex,
    cache_key: &str,
    k: usize,
) -> Option<(Vec<(u32, f32)>, u64)> {
    let ranked = idx
        .match_rank_cache
        .read()
        .ok()
        .and_then(|cache| cache.get(cache_key).cloned())?;
    let total = ranked.len() as u64;
    Some((ranked.iter().take(k).copied().collect(), total))
}

struct SingleTokenRankInput<'a> {
    idx: &'a TextIndex,
    docids: &'a [u32],
    tfs: &'a [u32],
    segment_doc_lens: Option<&'a [u32]>,
    idf: f32,
    avgdl: f32,
    interner: &'a Interner,
}

fn build_single_token_ranked(input: SingleTokenRankInput<'_>) -> Vec<(u32, f32)> {
    let mut ranked = Vec::with_capacity(input.docids.len());
    let mut score_cache = Vec::with_capacity(4);
    for i in 0..input.docids.len() {
        let id = input.docids[i];
        let doc_len = text_doc_len_at(input.idx, input.segment_doc_lens, id);
        ranked.push((
            id,
            bm25_contrib_cached(
                &mut score_cache,
                input.idf,
                input.tfs[i],
                doc_len,
                input.avgdl,
            ),
        ));
    }
    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(CmpOrdering::Equal)
            .then_with(|| input.interner.resolve(a.0).cmp(input.interner.resolve(b.0)))
    });
    ranked
}

struct AndRankInput<'a> {
    idx: &'a TextIndex,
    posts: &'a [TokPostings<'a>],
    idfs: &'a [f32],
    drive: usize,
    avgdl: f32,
    interner: &'a Interner,
}

fn build_and_ranked(input: AndRankInput<'_>) -> Vec<(u32, f32)> {
    let mut ranked = Vec::with_capacity(input.posts[input.drive].df());
    let segment_doc_lens = input
        .idx
        .segment
        .as_ref()
        .and_then(|seg| seg.text_doc_lens());
    let drive_docids = input.posts[input.drive].docids();
    'docs: for &id in drive_docids {
        let doc_len = text_doc_len_at(input.idx, segment_doc_lens, id) as f32;
        let mut score = 0.0f32;
        for (k, p) in input.posts.iter().enumerate() {
            let Some(tf) = p.tf(id) else {
                continue 'docs;
            };
            score += bm25_contrib(input.idfs[k], tf as f32, doc_len, input.avgdl);
        }
        ranked.push((id, score));
    }
    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(CmpOrdering::Equal)
            .then_with(|| input.interner.resolve(a.0).cmp(input.interner.resolve(b.0)))
    });
    ranked
}

#[inline]
/// BM25 over the text field. Implements the standard form:
///
/// ```text
/// score = Σ_t IDF(t) · TF(t,d) · (k1+1) / (TF(t,d) + k1 · (1 − b + b · |d| / avgdl))
/// ```
fn eval_match(coll: &Collection, m: &MatchQuery) -> Result<ScoredHits> {
    let fi = coll
        .fields
        .get(&m.field)
        .ok_or_else(|| StorageError::UnknownField {
            collection: "<>".into(),
            field: m.field.clone(),
        })?;
    let FieldIndex::Text { analyzer, idx } = fi else {
        bail!(
            "match query is only valid on text fields (field `{}`)",
            m.field
        );
    };
    let tokens = tokenize::tokenize(&m.text, *analyzer);
    // `doc_count` is the live corpus N; when a segment is attached it carries
    // the identical header scalar, so the short-circuit and `n` are unchanged.
    let (corpus_n, corpus_total_len) = idx.bm25_corpus();
    if tokens.is_empty() || corpus_n == 0 {
        return Ok(HashMap::new());
    }
    let n = corpus_n as f32;
    let avgdl = if corpus_n == 0 {
        1.0
    } else {
        corpus_total_len as f32 / corpus_n as f32
    };

    // Resolve a token's postings from either the live `tokens` map or the
    // attached segment (Phase 2e-B; text tf is STORED). The two sources return
    // the SAME `(docids, tfs)` u32 streams in the SAME ascending-docid order, so
    // every downstream `df`/`tf`/`doc_len`/`n`/`avgdl` -> f32 cast is bit-equal
    // and the literal BM25 expression yields identical bits on both paths.
    let resolve = |tok: &str| -> Option<TokPostings<'_>> { idx.tok_postings(tok) };

    // Score one token's BM25 contribution into `out` (`+=`, so a doc matching
    // several tokens accumulates). The BM25 expression is byte-identical to the
    // live path; only the postings SOURCE may differ (segment vs in-RAM). Do
    // NOT reassociate `tf + K1*(1.0 - B + B*doc_len/avgdl)`.
    let score_token = |tok: &str, out: &mut ScoredHits| {
        let Some(postings) = resolve(tok) else {
            return;
        };
        let df = postings.df() as f32;
        let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
        let (docids, tfs) = (postings.docids(), postings.tfs());
        for i in 0..docids.len() {
            let id = docids[i];
            let doc_len = idx.doc_len(id) as f32;
            let tf = tfs[i] as f32;
            *out.entry(id).or_insert(0.0) += bm25_contrib(idf, tf, doc_len, avgdl);
        }
    };

    Ok(match m.op {
        MatchOp::Or => {
            // Union: accumulate every token directly into one map — no
            // intermediate per-token maps to build and then merge. Pre-size to
            // the summed df so a high-df single term (the text_bm25 case) does
            // not pay ~15 power-of-two rehashes.
            let cap: usize = tokens
                .iter()
                .filter_map(|t| resolve(t).map(|p| p.df()))
                .sum();
            let mut acc: ScoredHits = HashMap::with_capacity(cap);
            for tok in &tokens {
                score_token(tok, &mut acc);
            }
            acc
        }
        MatchOp::And => {
            // Intersect by DRIVING from the rarest token (fewest candidates) and
            // probing the others by binary-search — no full per-token map
            // materialization (the old path scored every token's whole posting
            // list). Any absent token ⇒ empty intersection. The per-doc score
            // still sums each token's BM25 contribution in ORIGINAL TOKEN ORDER,
            // so the f32 result is byte-identical to the old per-token-map merge.
            let mut posts: Vec<TokPostings<'_>> = Vec::with_capacity(tokens.len());
            for tok in &tokens {
                match resolve(tok) {
                    Some(p) => posts.push(p),
                    None => return Ok(HashMap::new()),
                }
            }
            let idfs: Vec<f32> = posts
                .iter()
                .map(|p| {
                    let df = p.df() as f32;
                    ((n - df + 0.5) / (df + 0.5) + 1.0).ln()
                })
                .collect();
            let drive = (0..posts.len()).min_by_key(|&i| posts[i].df()).unwrap_or(0);
            let mut acc: ScoredHits = HashMap::with_capacity(posts[drive].df());
            let drive_docids = posts[drive].docids();
            'docs: for &id in drive_docids {
                let doc_len = idx.doc_len(id) as f32;
                let mut score = 0.0f32;
                for (k, p) in posts.iter().enumerate() {
                    let Some(tf) = p.tf(id) else {
                        continue 'docs;
                    };
                    let tf = tf as f32;
                    score += bm25_contrib(idfs[k], tf, doc_len, avgdl);
                }
                acc.insert(id, score);
            }
            acc
        }
    })
}

/// HashMap-free, bounded-top-k BM25 for the cases where every result docid is
/// UNIQUE: a single token (one posting list ⇒ each docid scored once) or a
/// multi-token `And` (driven from the rarest token ⇒ each driver docid emitted
/// once). For these the score is the SAME value `eval_match` would put in the
/// map, so streaming into a bounded heap drops the per-doc hash insert, the
/// map→Vec collect, and the full matched Vec partition/sort. Exact total is
/// still counted, but only `offset + limit` hits are retained.
///
/// Returns `None` to defer to `eval_match`'s map path when accumulation IS
/// required (multi-token `Or`: a doc matching several tokens must sum their
/// contributions). The emitted f32 bits are byte-identical to `eval_match`'s
/// map for the same query: both go through [`bm25_contrib`], in the same order,
/// and a unique-key `out.entry(id).or_insert(0.0) += c` equals `c` (single
/// token), while the `And` driver sums per token starting from `0.0f32` exactly
/// as the map branch does before its single `insert`.
fn eval_match_topk(
    coll: &Collection,
    m: &MatchQuery,
    interner: &Interner,
    k: usize,
) -> Result<Option<(Vec<(u32, f32)>, u64)>> {
    let fi = coll
        .fields
        .get(&m.field)
        .ok_or_else(|| StorageError::UnknownField {
            collection: "<>".into(),
            field: m.field.clone(),
        })?;
    let FieldIndex::Text { analyzer, idx } = fi else {
        bail!(
            "match query is only valid on text fields (field `{}`)",
            m.field
        );
    };
    let tokens = tokenize::tokenize(&m.text, *analyzer);
    let (corpus_n, corpus_total_len) = idx.bm25_corpus();
    if tokens.is_empty() || corpus_n == 0 {
        return Ok(Some((Vec::new(), 0)));
    }
    let n = corpus_n as f32;
    let avgdl = if corpus_n == 0 {
        1.0
    } else {
        corpus_total_len as f32 / corpus_n as f32
    };

    // Only the unique-docid shapes use the Vec fast path. A multi-token `Or`
    // needs real accumulation (a doc may match >1 token) ⇒ defer to the map.
    if tokens.len() > 1 && m.op == MatchOp::Or {
        return Ok(None);
    }

    let resolve = |tok: &str| -> Option<TokPostings<'_>> { idx.tok_postings(tok) };

    if tokens.len() == 1 {
        // Single token (Or == And == the one posting list): each docid is
        // scored EXACTLY once, so `out.entry(id).or_insert(0.0) += c == c`.
        let cache_key = match_rank_key(m.op, &tokens);
        if let Some(page) = cached_match_ranked_page(idx, &cache_key, k) {
            return Ok(Some(page));
        }
        let Some(postings) = resolve(&tokens[0]) else {
            return Ok(Some((Vec::new(), 0)));
        };
        let df = postings.df() as f32;
        let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
        let (docids, tfs) = (postings.docids(), postings.tfs());
        let segment_doc_lens = match &postings {
            TokPostings::Segment(_) => idx.segment.as_ref().and_then(|seg| seg.text_doc_lens()),
            _ => None,
        };
        let ranked = std::sync::Arc::new(build_single_token_ranked(SingleTokenRankInput {
            idx,
            docids,
            tfs,
            segment_doc_lens,
            idf,
            avgdl,
            interner,
        }));
        let total = ranked.len() as u64;
        if let Ok(mut cache) = idx.match_rank_cache.write() {
            cache.insert(cache_key, ranked.clone());
        }
        return Ok(Some((ranked.iter().take(k).copied().collect(), total)));
    }

    // Multi-token AND — same drive-from-rarest + per-token sum as the map
    // branch, but each surviving driver docid is pushed once (unique) instead
    // of `insert`ed. Byte-identical: the per-doc `score` starts at `0.0f32` and
    // accumulates `bm25_contrib` over the tokens in the SAME order.
    let mut posts: Vec<TokPostings<'_>> = Vec::with_capacity(tokens.len());
    for tok in &tokens {
        match resolve(tok) {
            Some(p) => posts.push(p),
            None => return Ok(Some((Vec::new(), 0))),
        }
    }
    let idfs: Vec<f32> = posts
        .iter()
        .map(|p| {
            let df = p.df() as f32;
            ((n - df + 0.5) / (df + 0.5) + 1.0).ln()
        })
        .collect();
    let cache_key = match_rank_key(m.op, &tokens);
    if let Some(page) = cached_match_ranked_page(idx, &cache_key, k) {
        return Ok(Some(page));
    }
    let drive = (0..posts.len()).min_by_key(|&i| posts[i].df()).unwrap_or(0);
    let ranked = std::sync::Arc::new(build_and_ranked(AndRankInput {
        idx,
        posts: &posts,
        idfs: &idfs,
        drive,
        avgdl,
        interner,
    }));
    let total = ranked.len() as u64;
    if let Ok(mut cache) = idx.match_rank_cache.write() {
        cache.insert(cache_key, ranked.clone());
    }
    Ok(Some((ranked.iter().take(k).copied().collect(), total)))
}

/// Shared engine for `Exists` and `Duplicated`: the union of doc-ids that have a
/// value in `field` whose posting size ≥ `min_count`.
///   - `min_count = 1` → `Exists` (doc has any value in the field).
///   - `min_count = N` → `Duplicated` (the value is shared by ≥ N docs).
/// Segment ON: drive from the segment-aware `live_*` accessors (segment
/// dict/column minus tombstones + live tail), identical to `duplicates`.
/// Segment OFF: borrow the in-RAM map directly (the `live_*` accessors clone the
/// whole map in that case), and for `min_count ≥ 2` visit only the `dup_values`
/// side-index candidates instead of every distinct value — the high-cardinality
/// (email/phone) duplicated path stays O(|colliding values|). text / vector /
/// hash are not supported here (declare a keyword companion field for a text
/// "is empty" filter; use knn / hamming for vector / hash).
fn eval_field_doc_union(coll: &Collection, field: &str, min_count: u64) -> Result<RoaringBitmap> {
    let fi = coll
        .fields
        .get(field)
        .ok_or_else(|| StorageError::UnknownField {
            collection: "<>".into(),
            field: field.to_string(),
        })?;
    // Union postings of size >= min_count: owned sets (segment path) or borrowed
    // in-RAM sets (tail-only path) without cloning the map.
    fn union_owned<K>(map: BTreeMap<K, RoaringBitmap>, min_count: u64) -> RoaringBitmap {
        let mut acc = RoaringBitmap::new();
        for set in map.values() {
            if set.len() >= min_count {
                acc |= set;
            }
        }
        acc
    }
    let mut acc = RoaringBitmap::new();
    match fi {
        FieldIndex::Keyword(k) => {
            if k.segment.is_some() {
                acc = union_owned(k.live_terms(), min_count);
            } else if min_count >= 2 {
                for set in k.dup_values.iter().filter_map(|v| k.terms.get(v)) {
                    if set.len() >= min_count {
                        acc |= set;
                    }
                }
            } else {
                for set in k.terms.values() {
                    acc |= set;
                }
            }
        }
        FieldIndex::Number(n) => {
            if n.segment.is_some() {
                acc = union_owned(n.live_values(), min_count);
            } else if min_count >= 2 {
                for set in n.dup_values.iter().filter_map(|v| n.values.get(v)) {
                    if set.len() >= min_count {
                        acc |= set;
                    }
                }
            } else {
                for set in n.values.values() {
                    acc |= set;
                }
            }
        }
        FieldIndex::Set(s) => {
            if s.segment.is_some() {
                acc = union_owned(s.live_elements(), min_count);
            } else if min_count >= 2 {
                for set in s.dup_values.iter().filter_map(|v| s.elements.get(v)) {
                    if set.len() >= min_count {
                        acc |= set;
                    }
                }
            } else {
                for set in s.elements.values() {
                    acc |= set;
                }
            }
        }
        FieldIndex::Text { .. } => bail!(
            "exists/duplicated is not supported on text field `{}` — declare a \
             keyword companion field for an \"is empty\"/duplicate filter",
            field
        ),
        FieldIndex::Vector { .. } => bail!(
            "exists/duplicated is not supported on vector field `{}` — use knn",
            field
        ),
        FieldIndex::Hash(_) => bail!(
            "exists/duplicated is not supported on hash field `{}` — use hamming",
            field
        ),
    }
    Ok(acc)
}

fn eval_term(coll: &Collection, t: &TermQuery) -> Result<RoaringBitmap> {
    let fi = coll
        .fields
        .get(&t.field)
        .ok_or_else(|| StorageError::UnknownField {
            collection: "<>".into(),
            field: t.field.clone(),
        })?;
    Ok(match (fi, &t.value) {
        (FieldIndex::Keyword(k), FieldValue::String(s)) => {
            // Phase 2h-1: read the inverted index through the unified
            // accessor so a sealed field (RAM `terms` dropped) serves from
            // the mmap segment + live tail, identical to the in-RAM path.
            k.term_postings(s)
                .map(|c| c.into_owned())
                .unwrap_or_default()
        }
        (FieldIndex::Number(n), FieldValue::Number(x)) => {
            let key = SortableF64::new(*x)?;
            // Phase 2h-3: read the exact-match posting through the unified
            // accessor so a sealed field (RAM `values` dropped) serves from
            // the mmap sorted-value index + live tail, identical to the in-RAM
            // path.
            n.value_postings(key)
                .map(|c| c.into_owned())
                .unwrap_or_default()
        }
        (FieldIndex::Set(s), FieldValue::String(el)) => {
            // Phase 2h-2: read the inverted index through the unified accessor so
            // a sealed field (RAM `elements` dropped) serves from the mmap segment
            // + live tail, identical to the in-RAM path.
            s.element_postings(el)
                .map(|c| c.into_owned())
                .unwrap_or_default()
        }
        _ => bail!("term query type mismatch on field `{}`", t.field),
    })
}

fn eval_terms(coll: &Collection, t: &TermsQuery) -> Result<RoaringBitmap> {
    let mut acc = RoaringBitmap::new();
    // Fast path: union the in-memory posting bitmaps by reference (word-wise
    // OR, no per-value clone).
    if let Some(fi) = coll.fields.get(&t.field) {
        for v in &t.values {
            match (fi, v) {
                (FieldIndex::Keyword(k), FieldValue::String(s)) => {
                    // Phase 2h-1: unified accessor — Borrowed (segment OFF)
                    // is the same by-reference word-wise OR as before; Owned
                    // (segment ON) ORs the decoded segment+tail union in.
                    if let Some(set) = k.term_postings(s) {
                        acc |= set.as_ref();
                    }
                }
                (FieldIndex::Set(se), FieldValue::String(el)) => {
                    // Phase 2h-2: unified accessor — Borrowed (segment OFF) is
                    // the same by-reference word-wise OR as before; Owned
                    // (segment ON) ORs the decoded segment+tail union in.
                    if let Some(set) = se.element_postings(el) {
                        acc |= set.as_ref();
                    }
                }
                (FieldIndex::Number(nx), FieldValue::Number(x)) => {
                    // Phase 2h-3: unified accessor — Borrowed (segment OFF) is
                    // the same by-reference word-wise OR as before; Owned
                    // (segment ON) ORs the decoded sorted-value+tail union in.
                    let key = SortableF64::new(*x)?;
                    if let Some(set) = nx.value_postings(key) {
                        acc |= set.as_ref();
                    }
                }
                _ => bail!("terms query type mismatch on field `{}`", t.field),
            }
        }
        return Ok(acc);
    }
    for v in &t.values {
        let one = TermQuery {
            field: t.field.clone(),
            value: v.clone(),
        };
        acc |= eval_term(coll, &one)?;
    }
    Ok(acc)
}

fn eval_knn(coll: &Collection, q: &KnnQuery) -> Result<ScoredHits> {
    eval_knn_inner(coll, q, None)
}

/// Reciprocal Rank Fusion: run each sub-query, rank its hits by score
/// descending (ties broken by docid for determinism), and fuse by rank —
/// `score(d) = Σ_i 1/(k + rank_i(d))`, 1-based rank, over the sub-queries that
/// returned `d`. Rank-based, so BM25 and cosine scales need no normalisation.
/// Filters belong inside each leg (`knn AND <filter>`), where the kNN stays
/// filter-correct via the And-node allow-list.
fn eval_rrf(
    coll: &Collection,
    collection_id: &str,
    r: &RrfQuery,
    universe: &BTreeSet<u32>,
    state: &EngineState,
) -> Result<ScoredHits> {
    let k = r.k.max(1) as f32;
    let mut fused: ScoredHits = HashMap::new();
    for sub in &r.queries {
        let hits = eval_query(coll, collection_id, sub, universe, state)?;
        // Rank by score desc; break ties by docid asc so fusion is deterministic.
        let mut ranked: Vec<(u32, f32)> = hits.into_iter().collect();
        ranked.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        for (rank0, (id, _)) in ranked.into_iter().enumerate() {
            let contrib = 1.0 / (k + (rank0 as f32 + 1.0));
            *fused.entry(id).or_insert(0.0) += contrib;
        }
    }
    Ok(fused)
}

/// kNN constrained to an `allow` set of dense doc-ids — returns the nearest
/// `k` *within the filtered set*. Drives `VectorIndex::search_knn_filtered`,
/// so a selective filter never collapses recall the way a post-filter over the
/// global top-k does (the pgvector failure mode).
fn eval_knn_filtered(coll: &Collection, q: &KnnQuery, allow: &RoaringBitmap) -> Result<ScoredHits> {
    eval_knn_inner(coll, q, Some(allow))
}

fn eval_knn_inner(
    coll: &Collection,
    q: &KnnQuery,
    allow: Option<&RoaringBitmap>,
) -> Result<ScoredHits> {
    let fi = coll
        .fields
        .get(&q.field)
        .ok_or_else(|| StorageError::UnknownField {
            collection: "<>".into(),
            field: q.field.clone(),
        })?;
    let FieldIndex::Vector { spec, idx, .. } = fi else {
        bail!(
            "knn query is only valid on vector fields (field `{}`)",
            q.field
        );
    };
    if q.vector.len() as u32 != spec.dim {
        bail!(
            "knn query on field `{}` declared dim={} but got vector of length {}",
            q.field,
            spec.dim,
            q.vector.len()
        );
    }
    if q.k == 0 {
        return Ok(HashMap::new());
    }
    // The vector index keeps its own String-keyed id space; map results to the
    // collection's dense doc-ids for the unified scored-hits space. When an
    // allow-set is given, the bits gate the graph search itself (translated
    // back to external_ids via the interner) rather than filtering afterwards.
    let pairs = match allow {
        None => idx.search_knn(&q.vector, q.k as usize)?,
        Some(bm) => {
            let allow_fn = |eid: &str| coll.interner.id(eid).map_or(false, |id| bm.contains(id));
            idx.search_knn_filtered(&q.vector, q.k as usize, &allow_fn)?
        }
    };
    Ok(pairs
        .into_iter()
        .filter_map(|(eid, score)| coll.interner.id(&eid).map(|id| (id, score)))
        .collect())
}

/// Hamming near-duplicate search: every doc whose 64-bit hash is within
/// `max_distance` bits of the query hash, scored by similarity (closer →
/// higher). Brute-force scan over the field's forward map.
fn eval_hamming(coll: &Collection, q: &HammingQuery) -> Result<ScoredHits> {
    let fi = coll
        .fields
        .get(&q.field)
        .ok_or_else(|| StorageError::UnknownField {
            collection: "<>".into(),
            field: q.field.clone(),
        })?;
    let FieldIndex::Hash(h) = fi else {
        bail!(
            "hamming query is only valid on hash fields (field `{}`)",
            q.field
        );
    };
    let query = parse_hash(&q.hash)?;
    let max = q.max_distance.min(64);
    let mut hits = ScoredHits::new();
    // The per-doc hash read routes through `hash_at`, so a sealed segment
    // serves ids `[0..n_docs)`. We must scan the union of the segment's covered
    // id range and the live `forward` tail; with no segment the union is
    // exactly `forward`'s keys, so this is byte-for-byte the live scan.
    let mut scan = |id: u32, doc: u64| {
        let dist = (doc ^ query).count_ones();
        if dist <= max {
            hits.insert(id, (64 - dist) as f32 / 64.0);
        }
    };
    if let Some(seg) = &h.segment {
        // Sealed range, served zero-copy from the segment (absent ids skipped).
        for id in 0..seg.n_docs() {
            if let Some(doc) = h.hash_at(id) {
                scan(id, doc);
            }
        }
        // Live tail: ids the segment does not cover.
        for (&id, &doc) in &h.forward {
            if id >= seg.n_docs() {
                scan(id, doc);
            }
        }
        return Ok(hits);
    }
    for (&id, &doc) in &h.forward {
        scan(id, h.hash_at(id).unwrap_or(doc));
    }
    Ok(hits)
}

fn eval_range(coll: &Collection, r: &RangeQuery) -> Result<RoaringBitmap> {
    let fi = coll
        .fields
        .get(&r.field)
        .ok_or_else(|| StorageError::UnknownField {
            collection: "<>".into(),
            field: r.field.clone(),
        })?;
    let FieldIndex::Number(n) = fi else {
        bail!(
            "range query is only valid on number fields (field `{}`)",
            r.field
        );
    };

    // Phase 2h-3: range walk through the unified accessor. Segment OFF: walks the
    // in-RAM `values.range((low, high))` exactly as before. Segment ON: binary-
    // searches the on-disk sorted-value index to the lo/hi bounds (SELECTIVE — no
    // forward scan), subtracts tombstones, and unions the live tail —
    // byte-identical result set to the in-RAM range walk over the same data.
    let (low, high) = range_bounds(r)?;
    Ok(n.range_postings(low, high))
}

// ---------------------------------------------------------------------------
// Query planner — filter-as-pruning for AND
//
// The materialize-and-intersect form of `And` evaluates every conjunct in full
// (e.g. a 16k-doc `range`) and then intersects. That is O(largest matched-set)
// even when the *combined* selectivity is tiny. The planner instead drives the
// AND from its cheapest conjunct and checks the others as per-doc predicates
// against the forward maps — so a wide clause is never materialized. The result
// (the matched set AND each doc's summed score) is identical to the
// materialized form; only the cost differs. Embedded mode only (the forward
// maps the predicates read are populated when there is no LSM backend).
// ---------------------------------------------------------------------------

/// Conjuncts the planner can check as a per-doc predicate. Anything else
/// (nested And/Or, Knn) forces the materialize-and-intersect fallback.
fn is_predicable(node: &QueryNode) -> bool {
    matches!(
        node,
        QueryNode::Term(_)
            | QueryNode::Terms(_)
            | QueryNode::Range(_)
            | QueryNode::Match(_)
            | QueryNode::Exists(_)
            | QueryNode::Duplicated(_)
    )
}

fn range_bounds(
    r: &RangeQuery,
) -> Result<(std::ops::Bound<SortableF64>, std::ops::Bound<SortableF64>)> {
    use std::ops::Bound;
    let low = if let Some(v) = r.gte {
        Bound::Included(SortableF64::new(v)?)
    } else if let Some(v) = r.gt {
        Bound::Excluded(SortableF64::new(v)?)
    } else {
        Bound::Unbounded
    };
    let high = if let Some(v) = r.lte {
        Bound::Included(SortableF64::new(v)?)
    } else if let Some(v) = r.lt {
        Bound::Excluded(SortableF64::new(v)?)
    } else {
        Bound::Unbounded
    };
    Ok((low, high))
}

fn in_range(v: SortableF64, r: &RangeQuery) -> Result<bool> {
    let (lo, hi) = range_bounds(r)?;
    Ok(in_sortable_range(v, &lo, &hi))
}

#[inline]
fn in_sortable_range(
    v: SortableF64,
    lo: &std::ops::Bound<SortableF64>,
    hi: &std::ops::Bound<SortableF64>,
) -> bool {
    in_sortable_bits_range(v.bits(), lo, hi)
}

#[inline]
fn in_sortable_bits_range(
    bits: u64,
    lo: &std::ops::Bound<SortableF64>,
    hi: &std::ops::Bound<SortableF64>,
) -> bool {
    use std::ops::Bound::*;
    let lo_ok = match lo {
        Included(b) => bits >= b.bits(),
        Excluded(b) => bits > b.bits(),
        Unbounded => true,
    };
    let hi_ok = match hi {
        Included(b) => bits <= b.bits(),
        Excluded(b) => bits < b.bits(),
        Unbounded => true,
    };
    lo_ok && hi_ok
}

#[derive(Clone, Copy)]
struct SortableBitsBounds {
    low: Option<(u64, bool)>,
    high: Option<(u64, bool)>,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl SortableBitsBounds {
    #[inline]
    fn new(lo: &std::ops::Bound<SortableF64>, hi: &std::ops::Bound<SortableF64>) -> Self {
        Self {
            low: bound_to_bits(*lo),
            high: bound_to_bits(*hi),
        }
    }

    #[inline(always)]
    fn contains(self, bits: u64) -> bool {
        let low_ok = match self.low {
            Some((b, true)) => bits >= b,
            Some((b, false)) => bits > b,
            None => true,
        };
        let high_ok = match self.high {
            Some((b, true)) => bits <= b,
            Some((b, false)) => bits < b,
            None => true,
        };
        low_ok && high_ok
    }
}

#[inline]
fn lower_bound_sortable_bits(values: &[(u64, u32)], bits: u64) -> usize {
    values.partition_point(|(probe, _)| *probe < bits)
}

#[inline]
fn upper_bound_sortable_bits(values: &[(u64, u32)], bits: u64) -> usize {
    values.partition_point(|(probe, _)| *probe <= bits)
}

#[inline]
fn sorted_bits_window(
    values: &[(u64, u32)],
    lo: &std::ops::Bound<SortableF64>,
    hi: &std::ops::Bound<SortableF64>,
) -> std::ops::Range<usize> {
    use std::ops::Bound::*;
    let start = match lo {
        Included(b) => lower_bound_sortable_bits(values, b.bits()),
        Excluded(b) => upper_bound_sortable_bits(values, b.bits()),
        Unbounded => 0,
    };
    let end = match hi {
        Included(b) => upper_bound_sortable_bits(values, b.bits()),
        Excluded(b) => lower_bound_sortable_bits(values, b.bits()),
        Unbounded => values.len(),
    };
    start.min(end)..end
}

fn eval_number_sort_keyword_term_page(
    coll: &Collection,
    query: &QueryNode,
    sort_idx: &NumberIndex,
    descending: bool,
    want: usize,
    track_total: bool,
) -> Result<Option<(Vec<(u32, f32)>, u64)>> {
    let QueryNode::Term(t) = query else {
        return Ok(None);
    };
    let FieldValue::String(term) = &t.value else {
        return Ok(None);
    };
    let Some(FieldIndex::Keyword(kidx)) = coll.fields.get(&t.field) else {
        return Ok(None);
    };

    let docs = sort_idx.keyword_range_docs(t.field.as_str(), term, kidx);
    let mut page = Vec::with_capacity(want.min(1024));
    if descending {
        for &(bits, id) in docs.iter().rev().take(want) {
            page.push((id, SortableF64::from_bits(bits).to_f64() as f32));
        }
    } else {
        for &(bits, id) in docs.iter().take(want) {
            page.push((id, SortableF64::from_bits(bits).to_f64() as f32));
        }
    }
    let total = if track_total {
        docs.len() as u64
    } else {
        page.len() as u64
    };
    Ok(Some((page, total)))
}

fn is_unbounded_range_on_field(query: &QueryNode, field: &str) -> bool {
    let QueryNode::Range(r) = query else {
        return false;
    };
    r.field == field && r.gt.is_none() && r.gte.is_none() && r.lt.is_none() && r.lte.is_none()
}

/// Cheap upper bound on how many docs a positive conjunct matches, WITHOUT
/// materializing it — used to pick the conjunct to drive the AND from. Reads
/// only posting/bucket lengths. Returns `u64::MAX` for shapes we don't drive
/// from (so a leaf is always preferred).
fn estimate_selectivity(coll: &Collection, node: &QueryNode) -> u64 {
    match node {
        QueryNode::Term(t) => match (coll.fields.get(&t.field), &t.value) {
            (Some(FieldIndex::Keyword(k)), FieldValue::String(s)) => {
                // Phase 2h-1: df from the active source (segment count-prefix
                // when sealed, else live posting length) — keeps rarest-first
                // clause ordering cheap with no posting decode.
                k.term_df(s)
            }
            (Some(FieldIndex::Number(n)), FieldValue::Number(x)) => SortableF64::new(*x)
                .ok()
                // Phase 2h-3: df from the active source (segment count-prefix when
                // sealed, else live posting length) — rarest-first ordering cheap.
                .map(|key| n.value_df(key))
                .unwrap_or(0),
            (Some(FieldIndex::Set(s)), FieldValue::String(el)) => {
                // Phase 2h-2: df from the active source (segment count-prefix when
                // sealed, else live posting length) — rarest-first ordering cheap.
                s.element_df(el)
            }
            _ => u64::MAX,
        },
        QueryNode::Terms(t) => match coll.fields.get(&t.field) {
            Some(FieldIndex::Keyword(k)) => t
                .values
                .iter()
                .map(|v| match v {
                    FieldValue::String(s) => k.term_df(s),
                    _ => 0,
                })
                .sum(),
            Some(FieldIndex::Set(s)) => t
                .values
                .iter()
                .map(|v| match v {
                    FieldValue::String(el) => s.element_df(el),
                    _ => 0,
                })
                .sum(),
            _ => u64::MAX,
        },
        QueryNode::Range(r) => match coll.fields.get(&r.field) {
            // Phase 2h-3: range df from the active source (segment count-prefix
            // sum when sealed, else the live range posting-length sum).
            Some(FieldIndex::Number(n)) => match range_bounds(r) {
                Ok((lo, hi)) => n.range_df(lo, hi),
                Err(_) => u64::MAX,
            },
            _ => u64::MAX,
        },
        QueryNode::Match(m) => match coll.fields.get(&m.field) {
            Some(FieldIndex::Text { analyzer, idx }) => {
                let toks = tokenize::tokenize(&m.text, *analyzer);
                // df from the active source: the sealed segment's stored posting
                // length when attached (Phase 2e-B), else the live posting df.
                let dfs = toks.iter().map(|t| idx.tok_df(t).map(|d| d as u64));
                match m.op {
                    // AND: a doc needs every token, so ≤ the rarest token's df.
                    MatchOp::And => dfs.map(|d| d.unwrap_or(0)).min().unwrap_or(0),
                    // OR: ≤ the sum of dfs.
                    MatchOp::Or => dfs.map(|d| d.unwrap_or(0)).sum(),
                }
            }
            _ => u64::MAX,
        },
        // Don't drive an AND from these. Exists/Duplicated would need a full
        // value-union scan to size, so they're not chosen as the cheap driver
        // either — a cheaper sibling clause (term/range) drives, then they filter.
        QueryNode::Knn(_)
        | QueryNode::And(_)
        | QueryNode::Or(_)
        | QueryNode::Not(_)
        | QueryNode::HasChild(_)
        | QueryNode::Hamming(_)
        | QueryNode::Rrf(_)
        | QueryNode::Exists(_)
        | QueryNode::Duplicated(_) => u64::MAX,
    }
}

/// Per-doc BM25 for a `match` conjunct, identical to [`eval_match`]'s formula,
/// so a doc scored as a predicate gets the same contribution it would as a
/// materialized clause. `None` ⇒ the doc does not satisfy the match.
fn match_doc_score(idx: &TextIndex, tokens: &[String], op: MatchOp, id: u32) -> Option<f32> {
    // Resolve the token→posting map ONCE, then score this doc. Single-doc callers
    // (the rare per-candidate predicate sites) get the same answer; the hot AND
    // bitmap loop instead builds a `PreparedMatch` ONCE and reuses it across all
    // candidates (see `score_prepared`) — that hoist is the `filtered_search` disk
    // fix (no per-candidate dict binary-search + cache fetch).
    let prepared = PreparedMatch::resolve(idx, tokens, op)?;
    prepared.score(idx, id)
}

/// A match clause with its per-token postings RESOLVED once (Phase 2m). On the
/// segment path each `TokPostings` holds the cache-resident posting `Arc`, so a
/// per-candidate score is a `.tf(id)` binary-search with NO dict lookup / cache
/// fetch / decode — the resolution (dict binary-search + cache get) is paid ONCE
/// per query instead of once per candidate doc, which is what blew up
/// `filtered_search` on disk (a wide candidate set × a re-resolve per doc).
struct PreparedMatch<'a> {
    /// One entry per token: its resolved postings + precomputed idf. `None` for a
    /// token absent from the index (so AND can short-circuit, OR can skip).
    per_token: Vec<Option<(TokPostings<'a>, f32)>>,
    op: MatchOp,
    n_tokens: usize,
    avgdl: f32,
}

impl<'a> PreparedMatch<'a> {
    const K1: f32 = 1.2;
    const B: f32 = 0.75;

    /// Resolve the postings for every token once. `None` when the corpus is empty
    /// or `tokens` is empty (matches the old `match_doc_score` early-outs exactly).
    fn resolve(idx: &'a TextIndex, tokens: &[String], op: MatchOp) -> Option<Self> {
        let (corpus_n, corpus_total_len) = idx.bm25_corpus();
        if corpus_n == 0 || tokens.is_empty() {
            return None;
        }
        let n = corpus_n as f32;
        let avgdl = corpus_total_len as f32 / corpus_n as f32;
        let per_token = tokens
            .iter()
            .map(|tok| {
                idx.tok_postings(tok).map(|postings| {
                    let df = postings.df() as f32;
                    let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
                    (postings, idf)
                })
            })
            .collect();
        Some(PreparedMatch {
            per_token,
            op,
            n_tokens: tokens.len(),
            avgdl,
        })
    }

    /// Score one doc against the pre-resolved postings — the identical BM25
    /// expression `match_doc_score` evaluated, just with the postings already in
    /// hand. Byte-identical result to the per-doc-resolved path.
    #[inline]
    fn score(&self, idx: &TextIndex, id: u32) -> Option<f32> {
        let mut doc_len: Option<f32> = None;
        let mut score = 0.0f32;
        let mut matched = 0usize;
        for entry in &self.per_token {
            if let Some((postings, idf)) = entry {
                if let Some(tf) = postings.tf(id) {
                    let doc_len = *doc_len.get_or_insert_with(|| idx.doc_len(id) as f32);
                    let tf = tf as f32;
                    let denom = tf + Self::K1 * (1.0 - Self::B + Self::B * doc_len / self.avgdl);
                    score += idf * tf * (Self::K1 + 1.0) / denom;
                    matched += 1;
                }
            }
        }
        match self.op {
            MatchOp::And if matched == self.n_tokens => Some(score),
            MatchOp::Or if matched > 0 => Some(score),
            _ => None,
        }
    }

    fn score_single_token_candidates_topk(
        &self,
        idx: &TextIndex,
        candidates: &RoaringBitmap,
        base: f32,
        interner: &Interner,
        k: usize,
    ) -> Option<(Vec<(u32, f32)>, u64)> {
        if self.n_tokens != 1 {
            return None;
        }
        let Some((postings, idf)) = self.per_token.first().and_then(|entry| entry.as_ref()) else {
            return Some((Vec::new(), 0));
        };

        // For one token, `AND` and `OR` have identical membership. Walk the
        // posting and candidate streams once instead of binary-searching the
        // posting for every filtered candidate.
        let docids = postings.docids();
        let tfs = postings.tfs();
        let mut cand = candidates.iter();
        let mut next = cand.next();
        let mut total = 0u64;
        let mut heap = BinaryHeap::with_capacity(k.min(candidates.len() as usize));
        let mut score_cache = Vec::with_capacity(4);
        for i in 0..docids.len() {
            let id = docids[i];
            loop {
                match next {
                    Some(c) if c < id => next = cand.next(),
                    Some(c) if c == id => {
                        let score = base
                            + bm25_contrib_cached(
                                &mut score_cache,
                                *idf,
                                tfs[i],
                                idx.doc_len(id),
                                self.avgdl,
                            );
                        total += 1;
                        push_top_ranked(&mut heap, k, interner, id, score);
                        next = cand.next();
                        break;
                    }
                    Some(_) => break,
                    None => return Some((finish_top_ranked(heap), total)),
                }
            }
        }
        Some((finish_top_ranked(heap), total))
    }
}

/// Does `id` satisfy `node`, and if so with what score contribution?
/// `Some(score)` = match (score added to the doc's AND total), `None` = no
/// match. Only valid for [`is_predicable`] nodes in embedded mode.
fn clause_matches(coll: &Collection, node: &QueryNode, id: u32) -> Result<Option<f32>> {
    let unknown = |field: &str| StorageError::UnknownField {
        collection: "<>".into(),
        field: field.to_string(),
    };
    Ok(match node {
        QueryNode::Term(t) => {
            let fi = coll.fields.get(&t.field).ok_or_else(|| unknown(&t.field))?;
            let hit = match (fi, &t.value) {
                (FieldIndex::Keyword(k), FieldValue::String(s)) => {
                    k.keyword_at(id).map(|v| &v == s).unwrap_or(false)
                }
                (FieldIndex::Number(n), FieldValue::Number(x)) => {
                    let key = SortableF64::new(*x)?;
                    n.number_at(id) == Some(key)
                }
                (FieldIndex::Set(s), FieldValue::String(el)) => s.set_contains(id, el),
                _ => bail!("term query type mismatch on field `{}`", t.field),
            };
            hit.then_some(1.0)
        }
        QueryNode::Terms(t) => {
            let fi = coll.fields.get(&t.field).ok_or_else(|| unknown(&t.field))?;
            let hit = match fi {
                FieldIndex::Keyword(k) => k.keyword_at(id).is_some_and(|v| {
                    t.values
                        .iter()
                        .any(|val| matches!(val, FieldValue::String(s) if *s == v))
                }),
                FieldIndex::Number(n) => n.number_at(id).is_some_and(|v| {
                    t.values.iter().any(|val| {
                        matches!(val, FieldValue::Number(x)
                            if SortableF64::new(*x).map(|k| k == v).unwrap_or(false))
                    })
                }),
                FieldIndex::Set(s) => s.set_contains_any(id, &t.values),
                _ => bail!("terms query type mismatch on field `{}`", t.field),
            };
            hit.then_some(1.0)
        }
        QueryNode::Range(r) => {
            let fi = coll.fields.get(&r.field).ok_or_else(|| unknown(&r.field))?;
            let FieldIndex::Number(n) = fi else {
                bail!(
                    "range query is only valid on number fields (field `{}`)",
                    r.field
                );
            };
            match n.number_at(id) {
                Some(v) if in_range(v, r)? => Some(1.0),
                _ => None,
            }
        }
        QueryNode::Match(m) => {
            let fi = coll.fields.get(&m.field).ok_or_else(|| unknown(&m.field))?;
            let FieldIndex::Text { analyzer, idx } = fi else {
                bail!(
                    "match query is only valid on text fields (field `{}`)",
                    m.field
                );
            };
            let tokens = tokenize::tokenize(&m.text, *analyzer);
            match_doc_score(idx, &tokens, m.op, id)
        }
        _ => bail!("clause_matches called on a non-predicable node"),
    })
}

/// Apply the non-driver conjuncts of an AND to one doc: cheap forward filters
/// first (short-circuit on miss), then `match` BM25, then negations. Returns
/// `Some(total_score)` if the doc satisfies every conjunct, else `None`.
fn apply_conjuncts(
    coll: &Collection,
    id: u32,
    base: f32,
    other_filters: &[&QueryNode],
    other_matches: &[(&TextIndex, Vec<String>, MatchOp)],
    not_inners: &[&QueryNode],
) -> Result<Option<f32>> {
    let mut score = base;
    for f in other_filters {
        match clause_matches(coll, f, id)? {
            Some(s) => score += s,
            None => return Ok(None),
        }
    }
    for (idx, toks, op) in other_matches {
        match match_doc_score(idx, toks, *op, id) {
            Some(s) => score += s,
            None => return Ok(None),
        }
    }
    for inner in not_inners {
        if clause_matches(coll, inner, id)?.is_some() {
            return Ok(None);
        }
        score += 1.0;
    }
    Ok(Some(score))
}

/// Evaluate a Term/Terms/Range conjunct to its doc bitmap (in-memory path).
fn eval_filter_bitmap(coll: &Collection, node: &QueryNode) -> Result<RoaringBitmap> {
    match node {
        QueryNode::Term(t) => eval_term(coll, t),
        QueryNode::Terms(t) => eval_terms(coll, t),
        QueryNode::Range(r) => eval_range(coll, r),
        QueryNode::Exists(e) => eval_field_doc_union(coll, &e.field, 1),
        QueryNode::Duplicated(d) => {
            eval_field_doc_union(coll, &d.field, d.min_group_size.max(2) as u64)
        }
        _ => bail!("eval_filter_bitmap called on a non-filter node"),
    }
}

fn eval_filter_bitmap_conjunction(
    coll: &Collection,
    positives: &[&QueryNode],
    negatives: &[&QueryNode],
) -> Result<RoaringBitmap> {
    debug_assert!(!positives.is_empty());
    if let Some(cand) = eval_high_cardinality_range_conjunction(coll, positives, negatives)? {
        return Ok(cand);
    }

    let mut order = positives.to_vec();
    order.sort_by_key(|c| estimate_selectivity(coll, c));

    let mut cand = eval_filter_bitmap(coll, order[0])?;
    for c in order.iter().skip(1) {
        if cand.is_empty() {
            break;
        }
        cand &= &eval_filter_bitmap(coll, c)?;
    }
    for nf in negatives {
        if cand.is_empty() {
            break;
        }
        cand -= &eval_filter_bitmap(coll, nf)?;
    }
    Ok(cand)
}

fn range_distinct_count(coll: &Collection, node: &QueryNode) -> Result<Option<u64>> {
    let QueryNode::Range(r) = node else {
        return Ok(None);
    };
    let Some(FieldIndex::Number(n)) = coll.fields.get(&r.field) else {
        return Ok(None);
    };
    let (lo, hi) = range_bounds(r)?;
    Ok(Some(n.range_distinct_count(lo, hi)))
}

fn eval_high_cardinality_range_conjunction(
    coll: &Collection,
    positives: &[&QueryNode],
    negatives: &[&QueryNode],
) -> Result<Option<RoaringBitmap>> {
    if positives.len() < 2 {
        return Ok(None);
    }
    if positives.len() == 2 && negatives.is_empty() {
        let range_ix = positives
            .iter()
            .position(|node| matches!(node, QueryNode::Range(_)));
        if let Some(range_ix) = range_ix {
            let QueryNode::Range(range_query) = positives[range_ix] else {
                unreachable!()
            };
            let Some(FieldIndex::Number(range_idx)) = coll.fields.get(&range_query.field) else {
                return Ok(None);
            };
            let (range_lo, range_hi) = range_bounds(range_query)?;
            let driver = positives[1 - range_ix];
            if let Some(out) = eval_range_with_keyword_driver_bitmap_dense(
                coll, driver, range_idx, &range_lo, &range_hi,
            )? {
                return Ok(Some(out));
            }
        }
    }
    let Some(range_ix) = select_high_cardinality_range(coll, positives)? else {
        return Ok(None);
    };
    let QueryNode::Range(range_query) = positives[range_ix] else {
        unreachable!()
    };
    let Some(FieldIndex::Number(range_idx)) = coll.fields.get(&range_query.field) else {
        return Ok(None);
    };
    let (range_lo, range_hi) = range_bounds(range_query)?;

    let mut rest: Vec<&QueryNode> = positives
        .iter()
        .copied()
        .enumerate()
        .filter(|(i, _)| *i != range_ix)
        .map(|(_, c)| c)
        .collect();
    rest.sort_by_key(|c| estimate_selectivity(coll, c));
    let range_sel = estimate_selectivity(coll, positives[range_ix]);
    if rest.len() == 1 {
        let rest_sel = estimate_selectivity(coll, rest[0]);
        if range_sel == u64::MAX || rest_sel.saturating_mul(4) < range_sel {
            if let Some(out) = eval_range_with_single_filter_driver(
                coll, rest[0], range_idx, &range_lo, &range_hi, negatives,
            )? {
                return Ok(Some(out));
            }
        }
    }
    let mut mask = eval_filter_bitmap(coll, rest[0])?;
    for c in rest.iter().skip(1) {
        if mask.is_empty() {
            break;
        }
        mask &= &eval_filter_bitmap(coll, c)?;
    }
    if range_sel != u64::MAX && mask.len().saturating_mul(4) >= range_sel {
        return Ok(None);
    }

    let mut out = RoaringBitmap::new();
    'doc: for id in &mask {
        if !range_idx.number_in_bounds(id, &range_lo, &range_hi) {
            continue;
        }
        for nf in negatives {
            if clause_matches(coll, nf, id)?.is_some() {
                continue 'doc;
            }
        }
        out.insert(id);
    }
    Ok(Some(out))
}

fn eval_range_with_keyword_driver_bitmap_dense(
    coll: &Collection,
    driver: &QueryNode,
    range_idx: &NumberIndex,
    range_lo: &std::ops::Bound<SortableF64>,
    range_hi: &std::ops::Bound<SortableF64>,
) -> Result<Option<RoaringBitmap>> {
    let (keyword_field, terms): (&str, Vec<&str>) = match driver {
        QueryNode::Term(t) => {
            let FieldValue::String(s) = &t.value else {
                return Ok(None);
            };
            (t.field.as_str(), vec![s.as_str()])
        }
        QueryNode::Terms(t) => {
            let mut terms = Vec::with_capacity(t.values.len());
            let mut seen = BTreeSet::new();
            for v in &t.values {
                let FieldValue::String(s) = v else {
                    return Ok(None);
                };
                if !seen.insert(s.as_str()) {
                    return Ok(None);
                }
                terms.push(s.as_str());
            }
            (t.field.as_str(), terms)
        }
        _ => return Ok(None),
    };
    let Some(FieldIndex::Keyword(kidx)) = coll.fields.get(keyword_field) else {
        return Ok(None);
    };

    let mut estimated = 0u64;
    for s in &terms {
        estimated += kidx.term_df(s);
    }
    if estimated < 8192 {
        return Ok(None);
    }

    let mut out = RoaringBitmap::new();
    for s in terms {
        let docs = range_idx.keyword_range_docs(keyword_field, s, kidx);
        if docs.is_empty() {
            continue;
        }
        let window = sorted_bits_window(docs.as_slice(), range_lo, range_hi);
        for &(_, id) in &docs[window] {
            out.insert(id);
        }
    }
    Ok(Some(out))
}

fn select_high_cardinality_range(
    coll: &Collection,
    positives: &[&QueryNode],
) -> Result<Option<usize>> {
    let mut best_range: Option<(usize, u64)> = None;
    for (i, node) in positives.iter().enumerate() {
        if let Some(distinct) = range_distinct_count(coll, node)? {
            if distinct >= 1024
                && best_range
                    .map(|(_, best_distinct)| distinct > best_distinct)
                    .unwrap_or(true)
            {
                best_range = Some((i, distinct));
            }
        }
    }
    Ok(best_range.map(|(i, _)| i))
}

fn eval_range_with_single_filter_driver(
    coll: &Collection,
    driver: &QueryNode,
    range_idx: &NumberIndex,
    range_lo: &std::ops::Bound<SortableF64>,
    range_hi: &std::ops::Bound<SortableF64>,
    negatives: &[&QueryNode],
) -> Result<Option<RoaringBitmap>> {
    let mut out = RoaringBitmap::new();
    let supported = visit_filter_driver_postings(coll, driver, |posting| {
        append_range_filtered_posting(
            coll, &mut out, posting, range_idx, range_lo, range_hi, negatives,
        )
    })?;
    Ok(supported.then_some(out))
}

fn eval_range_with_single_filter_driver_page(
    coll: &Collection,
    driver: &QueryNode,
    range_idx: &NumberIndex,
    range_lo: &std::ops::Bound<SortableF64>,
    range_hi: &std::ops::Bound<SortableF64>,
    negatives: &[&QueryNode],
    want: usize,
    score: f32,
) -> Result<Option<(Vec<(u32, f32)>, u64)>> {
    if negatives.is_empty() {
        if let Some(out) = eval_range_with_keyword_terms_driver_page_dense(
            coll, driver, range_idx, range_lo, range_hi, want, score,
        )? {
            return Ok(Some(out));
        }
    }
    if !filter_driver_postings_are_disjoint(coll, driver)? {
        return Ok(None);
    }
    let mut page = Vec::with_capacity(want.min(1024));
    let mut total = 0u64;
    let supported = visit_filter_driver_postings(coll, driver, |posting| {
        append_range_filtered_posting_page(
            coll, &mut page, &mut total, posting, range_idx, range_lo, range_hi, negatives, want,
            score,
        )
    })?;
    Ok(supported.then_some((page, total)))
}

fn eval_range_with_keyword_terms_driver_page_dense(
    coll: &Collection,
    driver: &QueryNode,
    range_idx: &NumberIndex,
    range_lo: &std::ops::Bound<SortableF64>,
    range_hi: &std::ops::Bound<SortableF64>,
    want: usize,
    score: f32,
) -> Result<Option<(Vec<(u32, f32)>, u64)>> {
    let (keyword_field, terms): (&str, Vec<&str>) = match driver {
        QueryNode::Term(t) => {
            let FieldValue::String(s) = &t.value else {
                return Ok(None);
            };
            (t.field.as_str(), vec![s.as_str()])
        }
        QueryNode::Terms(t) => {
            let mut terms = Vec::with_capacity(t.values.len());
            let mut seen = BTreeSet::new();
            for v in &t.values {
                let FieldValue::String(s) = v else {
                    return Ok(None);
                };
                if !seen.insert(s.as_str()) {
                    return Ok(None);
                }
                terms.push(s.as_str());
            }
            (t.field.as_str(), terms)
        }
        _ => return Ok(None),
    };
    let Some(FieldIndex::Keyword(kidx)) = coll.fields.get(keyword_field) else {
        return Ok(None);
    };

    let mut estimated = 0u64;
    for s in &terms {
        estimated += kidx.term_df(s);
    }
    if estimated < 8192 {
        return Ok(None);
    }

    let mut page = Vec::with_capacity(want.min(1024));
    let mut total = 0u64;
    for s in terms {
        let docs = range_idx.keyword_range_docs(keyword_field, s, kidx);
        if docs.is_empty() {
            continue;
        }
        let window = sorted_bits_window(docs.as_slice(), range_lo, range_hi);
        total += window.end.saturating_sub(window.start) as u64;
        if page.len() < want {
            for &(_, id) in &docs[window] {
                page.push((id, score));
                if page.len() >= want {
                    break;
                }
            }
        }
    }
    Ok(Some((page, total)))
}

fn filter_driver_postings_are_disjoint(coll: &Collection, driver: &QueryNode) -> Result<bool> {
    match driver {
        QueryNode::Term(t) => Ok(matches!(
            (coll.fields.get(&t.field), &t.value),
            (Some(FieldIndex::Keyword(_)), FieldValue::String(_))
                | (Some(FieldIndex::Number(_)), FieldValue::Number(_))
                | (Some(FieldIndex::Set(_)), FieldValue::String(_))
        )),
        QueryNode::Terms(t) => {
            let Some(fi) = coll.fields.get(&t.field) else {
                return Ok(false);
            };
            match fi {
                FieldIndex::Keyword(_) => {
                    let mut seen = BTreeSet::new();
                    for v in &t.values {
                        let FieldValue::String(s) = v else {
                            return Ok(false);
                        };
                        if !seen.insert(s.as_str()) {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                FieldIndex::Number(_) => {
                    let mut seen = BTreeSet::new();
                    for v in &t.values {
                        let FieldValue::Number(x) = v else {
                            return Ok(false);
                        };
                        if !seen.insert(SortableF64::new(*x)?) {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                }
                _ => Ok(false),
            }
        }
        _ => Ok(false),
    }
}

fn visit_filter_driver_postings<F>(
    coll: &Collection,
    driver: &QueryNode,
    mut visit: F,
) -> Result<bool>
where
    F: FnMut(&RoaringBitmap) -> Result<()>,
{
    match driver {
        QueryNode::Term(t) => {
            let Some(fi) = coll.fields.get(&t.field) else {
                return Ok(false);
            };
            match (fi, &t.value) {
                (FieldIndex::Keyword(k), FieldValue::String(s)) => {
                    if let Some(posting) = k.term_postings(s) {
                        visit(posting.as_ref())?;
                    }
                }
                (FieldIndex::Number(n), FieldValue::Number(x)) => {
                    let key = SortableF64::new(*x)?;
                    if let Some(posting) = n.value_postings(key) {
                        visit(posting.as_ref())?;
                    }
                }
                (FieldIndex::Set(s), FieldValue::String(el)) => {
                    if let Some(posting) = s.element_postings(el) {
                        visit(posting.as_ref())?;
                    }
                }
                _ => return Ok(false),
            }
            Ok(true)
        }
        QueryNode::Terms(t) => {
            let Some(fi) = coll.fields.get(&t.field) else {
                return Ok(false);
            };
            for v in &t.values {
                match (fi, v) {
                    (FieldIndex::Keyword(k), FieldValue::String(s)) => {
                        if let Some(posting) = k.term_postings(s) {
                            visit(posting.as_ref())?;
                        }
                    }
                    (FieldIndex::Number(n), FieldValue::Number(x)) => {
                        let key = SortableF64::new(*x)?;
                        if let Some(posting) = n.value_postings(key) {
                            visit(posting.as_ref())?;
                        }
                    }
                    (FieldIndex::Set(s), FieldValue::String(el)) => {
                        if let Some(posting) = s.element_postings(el) {
                            visit(posting.as_ref())?;
                        }
                    }
                    _ => return Ok(false),
                }
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn append_range_filtered_posting(
    coll: &Collection,
    out: &mut RoaringBitmap,
    posting: &RoaringBitmap,
    range_idx: &NumberIndex,
    range_lo: &std::ops::Bound<SortableF64>,
    range_hi: &std::ops::Bound<SortableF64>,
    negatives: &[&QueryNode],
) -> Result<()> {
    if negatives.is_empty() && range_idx.segment.is_none() {
        let bounds = SortableBitsBounds::new(range_lo, range_hi);
        for id in posting {
            let Some(bits) = range_idx.dense_forward.get(id as usize).copied() else {
                continue;
            };
            if bits != MISSING_SORTABLE_F64_BITS && bounds.contains(bits) {
                out.insert(id);
            }
        }
        return Ok(());
    }

    'doc: for id in posting {
        if !range_idx.number_in_bounds(id, range_lo, range_hi) {
            continue;
        }
        for nf in negatives {
            if clause_matches(coll, nf, id)?.is_some() {
                continue 'doc;
            }
        }
        out.insert(id);
    }
    Ok(())
}

fn append_range_filtered_posting_page(
    coll: &Collection,
    page: &mut Vec<(u32, f32)>,
    total: &mut u64,
    posting: &RoaringBitmap,
    range_idx: &NumberIndex,
    range_lo: &std::ops::Bound<SortableF64>,
    range_hi: &std::ops::Bound<SortableF64>,
    negatives: &[&QueryNode],
    want: usize,
    score: f32,
) -> Result<()> {
    if negatives.is_empty() && range_idx.segment.is_none() {
        let bounds = SortableBitsBounds::new(range_lo, range_hi);
        for id in posting {
            let Some(bits) = range_idx.dense_forward.get(id as usize).copied() else {
                continue;
            };
            if bits != MISSING_SORTABLE_F64_BITS && bounds.contains(bits) {
                *total += 1;
                if page.len() < want {
                    page.push((id, score));
                }
            }
        }
        return Ok(());
    }

    'doc: for id in posting {
        if !range_idx.number_in_bounds(id, range_lo, range_hi) {
            continue;
        }
        for nf in negatives {
            if clause_matches(coll, nf, id)?.is_some() {
                continue 'doc;
            }
        }
        *total += 1;
        if page.len() < want {
            page.push((id, score));
        }
    }
    Ok(())
}

fn eval_high_cardinality_range_filter_page(
    coll: &Collection,
    positives: &[&QueryNode],
    negatives: &[&QueryNode],
    want: usize,
    score: f32,
) -> Result<Option<(Vec<(u32, f32)>, u64)>> {
    if positives.len() < 2 {
        return Ok(None);
    }
    if positives.len() == 2 && negatives.is_empty() {
        let range_ix = positives
            .iter()
            .position(|node| matches!(node, QueryNode::Range(_)));
        if let Some(range_ix) = range_ix {
            let QueryNode::Range(range_query) = positives[range_ix] else {
                unreachable!()
            };
            let Some(FieldIndex::Number(range_idx)) = coll.fields.get(&range_query.field) else {
                return Ok(None);
            };
            let (range_lo, range_hi) = range_bounds(range_query)?;
            let driver = positives[1 - range_ix];
            if let Some(out) = eval_range_with_keyword_terms_driver_page_dense(
                coll, driver, range_idx, &range_lo, &range_hi, want, score,
            )? {
                return Ok(Some(out));
            }
        }
    }
    let Some(range_ix) = select_high_cardinality_range(coll, positives)? else {
        return Ok(None);
    };
    let QueryNode::Range(range_query) = positives[range_ix] else {
        unreachable!()
    };
    let Some(FieldIndex::Number(range_idx)) = coll.fields.get(&range_query.field) else {
        return Ok(None);
    };
    let (range_lo, range_hi) = range_bounds(range_query)?;

    let mut rest: Vec<&QueryNode> = positives
        .iter()
        .copied()
        .enumerate()
        .filter(|(i, _)| *i != range_ix)
        .map(|(_, c)| c)
        .collect();
    rest.sort_by_key(|c| estimate_selectivity(coll, c));
    if rest.len() != 1 {
        return Ok(None);
    }
    let range_sel = estimate_selectivity(coll, positives[range_ix]);
    let rest_sel = estimate_selectivity(coll, rest[0]);
    if range_sel != u64::MAX && rest_sel.saturating_mul(4) >= range_sel {
        return Ok(None);
    }

    eval_range_with_single_filter_driver_page(
        coll, rest[0], range_idx, &range_lo, &range_hi, negatives, want, score,
    )
}

fn eval_filter_only_page(
    coll: &Collection,
    node: &QueryNode,
    want: usize,
) -> Result<Option<(Vec<(u32, f32)>, u64)>> {
    let QueryNode::And(children) = node else {
        return Ok(None);
    };
    let (nots, positives): (Vec<&QueryNode>, Vec<&QueryNode>) = children
        .iter()
        .partition(|c| matches!(c, QueryNode::Not(_)));
    if positives.is_empty()
        || !positives.iter().all(|c| {
            matches!(
                c,
                QueryNode::Term(_) | QueryNode::Terms(_) | QueryNode::Range(_)
            )
        })
    {
        return Ok(None);
    }

    let mut filter_nots = Vec::new();
    for n in nots {
        let QueryNode::Not(inner) = n else {
            unreachable!()
        };
        if !matches!(
            &**inner,
            QueryNode::Term(_) | QueryNode::Terms(_) | QueryNode::Range(_)
        ) {
            return Ok(None);
        }
        filter_nots.push(&**inner);
    }

    let score = (positives.len() + filter_nots.len()) as f32;
    eval_high_cardinality_range_filter_page(coll, &positives, &filter_nots, want, score)
}

fn eval_filter_only_bitmap(
    coll: &Collection,
    node: &QueryNode,
) -> Result<Option<(RoaringBitmap, f32)>> {
    match node {
        QueryNode::Term(_) | QueryNode::Terms(_) | QueryNode::Range(_) => {
            Ok(Some((eval_filter_bitmap(coll, node)?, 1.0)))
        }
        QueryNode::And(children) => {
            let (nots, positives): (Vec<&QueryNode>, Vec<&QueryNode>) = children
                .iter()
                .partition(|c| matches!(c, QueryNode::Not(_)));
            if positives.is_empty()
                || !positives.iter().all(|c| {
                    matches!(
                        c,
                        QueryNode::Term(_) | QueryNode::Terms(_) | QueryNode::Range(_)
                    )
                })
            {
                return Ok(None);
            }

            let mut filter_nots = Vec::new();
            for n in nots {
                let QueryNode::Not(inner) = n else {
                    unreachable!()
                };
                if !matches!(
                    &**inner,
                    QueryNode::Term(_) | QueryNode::Terms(_) | QueryNode::Range(_)
                ) {
                    return Ok(None);
                }
                filter_nots.push(&**inner);
            }

            let score = (positives.len() + filter_nots.len()) as f32;
            let cand = eval_filter_bitmap_conjunction(coll, &positives, &filter_nots)?;
            Ok(Some((cand, score)))
        }
        _ => Ok(None),
    }
}

/// Hoist tokenization for a set of `match` conjuncts (once, not per doc).
fn prep_matches<'a>(
    coll: &'a Collection,
    nodes: &[&QueryNode],
) -> Result<Vec<(&'a TextIndex, Vec<String>, MatchOp)>> {
    let mut out = Vec::with_capacity(nodes.len());
    for c in nodes {
        let QueryNode::Match(m) = c else { continue };
        let fi = coll
            .fields
            .get(&m.field)
            .ok_or_else(|| StorageError::UnknownField {
                collection: "<>".into(),
                field: m.field.clone(),
            })?;
        let FieldIndex::Text { analyzer, idx } = fi else {
            bail!(
                "match query is only valid on text fields (field `{}`)",
                m.field
            );
        };
        out.push((idx, tokenize::tokenize(&m.text, *analyzer), m.op));
    }
    Ok(out)
}

/// Boolean: does `id` satisfy `node`? (Score ignored — used by sort-walk.)
fn query_predicate(coll: &Collection, node: &QueryNode, id: u32) -> Result<bool> {
    Ok(match node {
        QueryNode::Term(_) | QueryNode::Terms(_) | QueryNode::Range(_) | QueryNode::Match(_) => {
            clause_matches(coll, node, id)?.is_some()
        }
        QueryNode::And(cs) => {
            for c in cs {
                if !query_predicate(coll, c, id)? {
                    return Ok(false);
                }
            }
            true
        }
        QueryNode::Or(cs) => {
            for c in cs {
                if query_predicate(coll, c, id)? {
                    return Ok(true);
                }
            }
            false
        }
        QueryNode::Not(c) => !query_predicate(coll, c, id)?,
        QueryNode::Knn(_) => bail!("knn cannot be evaluated as a per-doc predicate"),
        QueryNode::Rrf(_) => bail!("rrf cannot be evaluated as a per-doc predicate"),
        QueryNode::HasChild(_) => bail!("has_child cannot be evaluated as a per-doc predicate"),
        QueryNode::Hamming(_) => bail!("hamming cannot be evaluated as a per-doc predicate"),
        // Exists/Duplicated test membership in the field's doc-union; the sort-walk
        // runs over a bounded candidate set so the recompute stays cheap.
        QueryNode::Exists(e) => eval_field_doc_union(coll, &e.field, 1)?.contains(id),
        QueryNode::Duplicated(d) => {
            eval_field_doc_union(coll, &d.field, d.min_group_size.max(2) as u64)?.contains(id)
        }
    })
}

fn query_has_knn(node: &QueryNode) -> bool {
    match node {
        QueryNode::Knn(_) => true,
        // rrf produces a fused relevance ranking, not a sort-walkable predicate —
        // treat it like knn so the sort fast-path bails to the general evaluator.
        QueryNode::Rrf(_) => true,
        QueryNode::And(cs) | QueryNode::Or(cs) => cs.iter().any(query_has_knn),
        QueryNode::Not(c) => query_has_knn(c),
        _ => false,
    }
}

/// Constant-score = no scored clause anywhere (no `match`, no `knn`). All
/// matches score equally, so collapse can early-terminate (any `limit` groups).
fn query_is_constant_score(node: &QueryNode) -> bool {
    match node {
        // HasChild produces a constant-score bitmap, but collapse early-term
        // can't DRIVE through it (no per-doc predicate) → treat as non-constant
        // so a query containing it takes the full collapse path.
        QueryNode::Match(_)
        | QueryNode::Knn(_)
        | QueryNode::HasChild(_)
        | QueryNode::Hamming(_)
        | QueryNode::Rrf(_) => false,
        QueryNode::And(cs) | QueryNode::Or(cs) => cs.iter().all(query_is_constant_score),
        QueryNode::Not(c) => query_is_constant_score(c),
        _ => true,
    }
}

/// Iterate the docs of a Term/Range by reference (no clone). Used to drive
/// early-terminating collapse without materializing the clause.
fn term_or_range_iter<'a>(
    coll: &'a Collection,
    node: &QueryNode,
) -> Option<Box<dyn Iterator<Item = u32> + 'a>> {
    match node {
        QueryNode::Term(t) => {
            // Keyword (2h-1) and Set (2h-2) route through the unified accessor: a
            // Borrowed posting (segment OFF) iterates by reference exactly as
            // before; an Owned posting (segment ON — RAM driver dropped) is
            // consumed by an owning iterator so the box can outlive the Cow.
            match (coll.fields.get(&t.field), &t.value) {
                (Some(FieldIndex::Keyword(k)), FieldValue::String(s)) => {
                    Some(match k.term_postings(s) {
                        Some(std::borrow::Cow::Borrowed(set)) => Box::new(set.iter()),
                        Some(std::borrow::Cow::Owned(set)) => Box::new(set.into_iter()),
                        None => Box::new(std::iter::empty()),
                    })
                }
                (Some(FieldIndex::Set(s)), FieldValue::String(el)) => {
                    Some(match s.element_postings(el) {
                        Some(std::borrow::Cow::Borrowed(set)) => Box::new(set.iter()),
                        Some(std::borrow::Cow::Owned(set)) => Box::new(set.into_iter()),
                        None => Box::new(std::iter::empty()),
                    })
                }
                (Some(FieldIndex::Number(n)), FieldValue::Number(x)) => {
                    // Phase 2h-3: Number exact-match through the unified accessor —
                    // Borrowed (segment OFF) iterates by reference; Owned (segment
                    // ON) consumes the decoded sorted-value+tail union.
                    let Ok(key) = SortableF64::new(*x) else {
                        return None;
                    };
                    Some(match n.value_postings(key) {
                        Some(std::borrow::Cow::Borrowed(set)) => Box::new(set.iter()),
                        Some(std::borrow::Cow::Owned(set)) => Box::new(set.into_iter()),
                        None => Box::new(std::iter::empty()),
                    })
                }
                _ => None,
            }
        }
        QueryNode::Range(r) => {
            let Some(FieldIndex::Number(n)) = coll.fields.get(&r.field) else {
                return None;
            };
            let (lo, hi) = range_bounds(r).ok()?;
            // Phase 2h-3: range through the unified accessor. Segment OFF this is
            // the same `values.range` union (now materialized once); segment ON it
            // is the on-disk binary-search union (+ tail, − tombstones). The result
            // is an owned bitmap, so its `into_iter` drives the collapse.
            Some(Box::new(n.range_postings(lo, hi).into_iter()))
        }
        _ => None,
    }
}

/// Pick the cheapest Term/Range leaf to drive an early-terminating collapse.
/// `And` → its cheapest such child; a top-level Term/Range → itself; otherwise
/// `None` (Or/Not/scored queries fall back to the full collapse path).
fn collapse_driver<'a>(
    coll: &'a Collection,
    query: &'a QueryNode,
) -> Option<Box<dyn Iterator<Item = u32> + 'a>> {
    match query {
        QueryNode::Term(_) | QueryNode::Range(_) => term_or_range_iter(coll, query),
        QueryNode::And(children) => children
            .iter()
            .filter(|c| matches!(c, QueryNode::Term(_) | QueryNode::Range(_)))
            .min_by_key(|c| estimate_selectivity(coll, c))
            .and_then(|c| term_or_range_iter(coll, c)),
        _ => None,
    }
}

/// Planner page+total for shapes that can early-terminate without
/// materializing the full matched set. Embedded mode + first page only;
/// returns `None` to fall back to the materialize-and-rank path.
///
/// Result order for these shapes is the field-sort order (sort queries) or the
/// number-index order (standalone range) — NOT the score/eid order of the
/// fallback. For a constant-score filter the order is unspecified anyway, and
/// sort queries define their own order, so this is a correct page.
/// Which planner produced the page — decides the next-cursor encoding:
/// `SortedField` pages continue via keyset (sort-value bits + docid),
/// `Posting` pages keep the legacy offset cursor (posting/docid order has no
/// resumable sort key).
enum PlanKind {
    SortedField,
    Posting,
    /// The general evaluator's score-desc + external_id-asc ranking.
    ScoreRanked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SortValue {
    Number(u64),
    Keyword(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SortAfter {
    values: Vec<SortValue>,
    docid: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortFieldKind {
    Number,
    Keyword,
}

fn sort_field_kind(
    coll: &Collection,
    collection_id: &str,
    spec: &SortSpec,
) -> Result<SortFieldKind> {
    match coll.fields.get(&spec.field) {
        Some(FieldIndex::Number(_)) => Ok(SortFieldKind::Number),
        Some(FieldIndex::Keyword(_)) => Ok(SortFieldKind::Keyword),
        Some(_) => Err(StorageError::UnsupportedSort(format!(
            "field `{}` is not sortable; supported sort fields are number and keyword",
            spec.field
        ))
        .into()),
        None => Err(StorageError::UnknownField {
            collection: collection_id.to_string(),
            field: spec.field.clone(),
        }
        .into()),
    }
}

fn query_can_be_sort_predicate(node: &QueryNode) -> bool {
    match node {
        QueryNode::Knn(_) | QueryNode::Rrf(_) | QueryNode::HasChild(_) | QueryNode::Hamming(_) => {
            false
        }
        QueryNode::And(cs) | QueryNode::Or(cs) => cs.iter().all(query_can_be_sort_predicate),
        QueryNode::Not(c) => query_can_be_sort_predicate(c),
        _ => true,
    }
}

fn validate_sort_request(
    coll: &Collection,
    collection_id: &str,
    req: &SearchRequest,
) -> Result<()> {
    let Some(sort) = req.sort.as_deref() else {
        return Ok(());
    };
    if sort.is_empty() {
        return Err(
            StorageError::UnsupportedSort("sort must contain at least one key".into()).into(),
        );
    }
    if sort.len() > 2 {
        return Err(StorageError::UnsupportedSort(format!(
            "multi-key sort supports at most two keys, got {}",
            sort.len()
        ))
        .into());
    }
    if req.collapse.is_some() {
        return Err(StorageError::UnsupportedSort(
            "sort cannot be combined with collapse/group-by results".into(),
        )
        .into());
    }
    if !query_can_be_sort_predicate(&req.query) {
        return Err(StorageError::UnsupportedSort(
            "sort cannot be combined with knn, rrf, has_child, or hamming queries".into(),
        )
        .into());
    }
    for spec in sort {
        sort_field_kind(coll, collection_id, spec)?;
    }
    Ok(())
}

fn sort_value_at(coll: &Collection, spec: &SortSpec, id: u32) -> Result<Option<SortValue>> {
    match coll.fields.get(&spec.field) {
        Some(FieldIndex::Number(n)) => Ok(n.number_bits_at(id).map(SortValue::Number)),
        Some(FieldIndex::Keyword(k)) => Ok(k.keyword_at(id).map(SortValue::Keyword)),
        Some(_) => Err(StorageError::UnsupportedSort(format!(
            "field `{}` is not sortable; supported sort fields are number and keyword",
            spec.field
        ))
        .into()),
        None => Err(StorageError::UnknownField {
            collection: "<>".into(),
            field: spec.field.clone(),
        }
        .into()),
    }
}

fn sort_values_for_doc(
    coll: &Collection,
    sort: &[SortSpec],
    id: u32,
) -> Result<Option<Vec<SortValue>>> {
    let mut values = Vec::with_capacity(sort.len());
    for spec in sort {
        let Some(value) = sort_value_at(coll, spec, id)? else {
            return Ok(None);
        };
        values.push(value);
    }
    Ok(Some(values))
}

fn compare_sort_value(a: &SortValue, b: &SortValue) -> CmpOrdering {
    match (a, b) {
        (SortValue::Number(a), SortValue::Number(b)) => a.cmp(b),
        (SortValue::Keyword(a), SortValue::Keyword(b)) => a.cmp(b),
        _ => CmpOrdering::Equal,
    }
}

fn compare_sort_tuples(
    a_values: &[SortValue],
    a_docid: u32,
    b_values: &[SortValue],
    b_docid: u32,
    sort: &[SortSpec],
) -> CmpOrdering {
    for ((a, b), spec) in a_values.iter().zip(b_values).zip(sort) {
        let ord = compare_sort_value(a, b);
        if ord != CmpOrdering::Equal {
            return match spec.order {
                SortOrder::Asc => ord,
                SortOrder::Desc => ord.reverse(),
            };
        }
    }
    a_docid.cmp(&b_docid)
}

fn is_after_sort_cursor(
    values: &[SortValue],
    docid: u32,
    after: Option<&SortAfter>,
    sort: &[SortSpec],
) -> bool {
    let Some(after) = after else {
        return true;
    };
    if values.len() != after.values.len() || values.len() != sort.len() {
        return true;
    }
    compare_sort_tuples(values, docid, &after.values, after.docid, sort) == CmpOrdering::Greater
}

fn sort_score(value: &SortValue) -> f32 {
    match value {
        SortValue::Number(bits) => SortableF64::from_bits(*bits).to_f64() as f32,
        SortValue::Keyword(_) => 1.0,
    }
}

fn cursor_value_matches_kind(value: &SortValue, kind: SortFieldKind) -> bool {
    matches!(
        (value, kind),
        (SortValue::Number(_), SortFieldKind::Number)
            | (SortValue::Keyword(_), SortFieldKind::Keyword)
    )
}

fn sort_after_for_request(
    coll: &Collection,
    sort: Option<&[SortSpec]>,
    parsed_cursor: &Option<PageCursor>,
) -> Result<Option<SortAfter>> {
    let Some(sort) = sort else {
        return Ok(None);
    };
    let Some(parsed_cursor) = parsed_cursor else {
        return Ok(None);
    };
    let candidate = match parsed_cursor {
        PageCursor::SortKeyset { bits, docid } if sort.len() == 1 => SortAfter {
            values: vec![SortValue::Number(*bits)],
            docid: *docid,
        },
        PageCursor::SortValuesKeyset { values, docid } => SortAfter {
            values: values.clone(),
            docid: *docid,
        },
        _ => return Ok(None),
    };
    if candidate.values.len() != sort.len() {
        return Ok(None);
    }
    for (value, spec) in candidate.values.iter().zip(sort) {
        let kind = sort_field_kind(coll, "<>", spec)?;
        if !cursor_value_matches_kind(value, kind) {
            return Ok(None);
        }
    }
    Ok(Some(candidate))
}

fn visit_sorted_bucket(
    coll: &Collection,
    req: &SearchRequest,
    sort: &[SortSpec],
    docs: &RoaringBitmap,
    after: Option<&SortAfter>,
    page: &mut Vec<(u32, f32)>,
    total: &mut u64,
) -> Result<bool> {
    let want = req.limit as usize;
    if sort.len() == 1 {
        for id in docs {
            if !query_predicate(coll, &req.query, id)? {
                continue;
            }
            let Some(values) = sort_values_for_doc(coll, sort, id)? else {
                continue;
            };
            if !is_after_sort_cursor(&values, id, after, sort) {
                continue;
            }
            *total += 1;
            if page.len() < want {
                page.push((id, sort_score(&values[0])));
            } else if !req.track_total {
                return Ok(false);
            }
        }
        return Ok(true);
    }

    let mut bucket = Vec::new();
    for id in docs {
        if !query_predicate(coll, &req.query, id)? {
            continue;
        }
        let Some(values) = sort_values_for_doc(coll, sort, id)? else {
            continue;
        };
        bucket.push((values, id));
    }
    bucket.sort_by(|(av, aid), (bv, bid)| compare_sort_tuples(av, *aid, bv, *bid, sort));
    for (values, id) in bucket {
        if !is_after_sort_cursor(&values, id, after, sort) {
            continue;
        }
        *total += 1;
        if page.len() < want {
            page.push((id, sort_score(&values[0])));
        } else if !req.track_total {
            return Ok(false);
        }
    }
    Ok(true)
}

fn try_generic_sort_plan(
    coll: &Collection,
    req: &SearchRequest,
    sort: &[SortSpec],
    sort_after: Option<&SortAfter>,
) -> Result<Option<(Vec<(u32, f32)>, u64, PlanKind)>> {
    let want = req.limit as usize;
    let mut page: Vec<(u32, f32)> = Vec::with_capacity(want.min(1024));
    let mut total: u64 = 0;
    let first = &sort[0];
    match coll.fields.get(&first.field) {
        Some(FieldIndex::Number(n)) => {
            let values = n.sorted_values();
            match first.order {
                SortOrder::Asc => {
                    for (_v, docs) in values.iter() {
                        if !visit_sorted_bucket(
                            coll, req, sort, docs, sort_after, &mut page, &mut total,
                        )? {
                            break;
                        }
                    }
                }
                SortOrder::Desc => {
                    for (_v, docs) in values.iter().rev() {
                        if !visit_sorted_bucket(
                            coll, req, sort, docs, sort_after, &mut page, &mut total,
                        )? {
                            break;
                        }
                    }
                }
            }
        }
        Some(FieldIndex::Keyword(k)) => {
            let terms = k.live_terms();
            match first.order {
                SortOrder::Asc => {
                    for (_term, docs) in terms.iter() {
                        if !visit_sorted_bucket(
                            coll, req, sort, docs, sort_after, &mut page, &mut total,
                        )? {
                            break;
                        }
                    }
                }
                SortOrder::Desc => {
                    for (_term, docs) in terms.iter().rev() {
                        if !visit_sorted_bucket(
                            coll, req, sort, docs, sort_after, &mut page, &mut total,
                        )? {
                            break;
                        }
                    }
                }
            }
        }
        Some(_) => {
            return Err(StorageError::UnsupportedSort(format!(
                "field `{}` is not sortable; supported sort fields are number and keyword",
                first.field
            ))
            .into());
        }
        None => {
            return Err(StorageError::UnknownField {
                collection: "<>".into(),
                field: first.field.clone(),
            }
            .into());
        }
    }
    if !req.track_total {
        total = total.max(page.len() as u64);
    }
    Ok(Some((page, total, PlanKind::SortedField)))
}

fn try_plan(
    coll: &Collection,
    req: &SearchRequest,
    offset: usize,
    sort_after: Option<&SortAfter>,
) -> Result<Option<(Vec<(u32, f32)>, u64, PlanKind)>> {
    if offset != 0 {
        return Ok(None);
    }
    let want = req.limit as usize;

    // ---- sort by a single number field ----
    if let Some(sort) = &req.sort {
        let [s] = sort.as_slice() else {
            return try_generic_sort_plan(coll, req, sort, sort_after);
        };
        let Some(FieldIndex::Number(n)) = coll.fields.get(&s.field) else {
            return try_generic_sort_plan(coll, req, sort, sort_after);
        };
        if query_has_knn(&req.query) {
            return Ok(None);
        }
        let number_sort_after = sort_after.and_then(|after| match after.values.as_slice() {
            [SortValue::Number(bits)] => Some((*bits, after.docid)),
            _ => None,
        });
        let descending = matches!(s.order, SortOrder::Desc);
        // Keyset continuation: a (v, id) pair is on the page iff it sits
        // strictly AFTER the cursor in walk order. Within an equal key the
        // walk emits docids ascending in BOTH directions, so the equal-key
        // remainder is `id > cursor_docid`.
        let after_bits = number_sort_after.map(|(bits, _)| bits);
        let past_cursor = |v_bits: u64, id: u32| -> bool {
            match number_sort_after {
                None => true,
                Some((k, d)) => {
                    if v_bits == k {
                        id > d
                    } else if descending {
                        v_bits < k
                    } else {
                        v_bits > k
                    }
                }
            }
        };
        if is_unbounded_range_on_field(&req.query, &s.field) {
            let mut page: Vec<(u32, f32)> = Vec::with_capacity(want.min(1024));
            let mut total: u64 = 0;
            if let Some(seg) = n.segment_ref() {
                n.sorted_walk_segment(seg.as_ref(), descending, after_bits, |v, id| {
                    if !past_cursor(SortableF64::new(v).map(|s| s.bits()).unwrap_or(0), id) {
                        return Ok(true);
                    }
                    total += 1;
                    if page.len() < want {
                        page.push((id, v as f32));
                    } else if !req.track_total {
                        return Ok(false);
                    }
                    Ok(true)
                })?;
            } else {
                let values = n.sorted_values();
                let after_key = number_sort_after.map(|(bits, _)| SortableF64::from_bits(bits));
                match s.order {
                    SortOrder::Asc => {
                        let range: Box<dyn Iterator<Item = (&SortableF64, &RoaringBitmap)>> =
                            match after_key {
                                Some(k) => Box::new(values.range(k..)),
                                None => Box::new(values.iter()),
                            };
                        'asc: for (v, docs) in range {
                            for id in docs {
                                if !past_cursor(v.bits(), id) {
                                    continue;
                                }
                                total += 1;
                                if page.len() < want {
                                    page.push((id, v.to_f64() as f32));
                                } else if !req.track_total {
                                    break 'asc;
                                }
                            }
                        }
                    }
                    SortOrder::Desc => {
                        let range: Box<dyn Iterator<Item = (&SortableF64, &RoaringBitmap)>> =
                            match after_key {
                                Some(k) => Box::new(values.range(..=k).rev()),
                                None => Box::new(values.iter().rev()),
                            };
                        'desc: for (v, docs) in range {
                            for id in docs {
                                if !past_cursor(v.bits(), id) {
                                    continue;
                                }
                                total += 1;
                                if page.len() < want {
                                    page.push((id, v.to_f64() as f32));
                                } else if !req.track_total {
                                    break 'desc;
                                }
                            }
                        }
                    }
                }
            }
            if !req.track_total {
                total = total.max(page.len() as u64);
            }
            return Ok(Some((page, total, PlanKind::SortedField)));
        }
        if number_sort_after.is_none() {
            if let Some(out) = eval_number_sort_keyword_term_page(
                coll,
                &req.query,
                n,
                descending,
                want,
                req.track_total,
            )? {
                return Ok(Some((out.0, out.1, PlanKind::SortedField)));
            }
        }
        let mut page: Vec<(u32, f32)> = Vec::with_capacity(want.min(1024));
        let mut total: u64 = 0;

        // SEGMENT ON (Phase 2m): walk the on-disk sorted-value index + the BOUNDED
        // per-value posting cache, MERGED with the live tail, in value order —
        // instead of materializing the whole-field `live_values()` BTreeMap on
        // EVERY query (the 525x sort regression). `sorted_walk_segment` subtracts
        // the tombstone per value (deleted base docids), unions the tail, and
        // emits `(value, docid)` in BYTE-IDENTICAL order to the in-RAM walk; the
        // per-doc `query_predicate` + page/early-term logic below is unchanged. The
        // walk short-circuits when `visit` returns `Ok(false)`, so a 10-hit page
        // only touches (and caches) the first few values' postings.
        if let Some(seg) = n.segment_ref() {
            let q = &req.query;
            let track_total = req.track_total;
            n.sorted_walk_segment(seg.as_ref(), descending, after_bits, |v, id| {
                if !past_cursor(SortableF64::new(v).map(|s| s.bits()).unwrap_or(0), id) {
                    return Ok(true);
                }
                if query_predicate(coll, q, id)? {
                    total += 1;
                    if page.len() < want {
                        page.push((id, v as f32));
                    } else if !track_total {
                        return Ok(false); // page full, no total needed → stop early
                    }
                }
                Ok(true)
            })?;
            if !req.track_total {
                total = total.max(page.len() as u64);
            }
            return Ok(Some((page, total, PlanKind::SortedField)));
        }

        // SEGMENT OFF (no segment attached): byte-for-byte the original
        // zero-clone walk over the in-RAM `values` BTreeMap. `sorted_values`
        // returns `Cow::Borrowed(&self.values)` here.
        // Walk docs in field-sorted order; emit those satisfying the query.
        // Score is the sort value (informational; ranking IS the walk order).
        // `track_total=false` lets us stop as soon as the page is full.
        let values = n.sorted_values();
        let after_key = number_sort_after.map(|(bits, _)| SortableF64::from_bits(bits));
        match s.order {
            SortOrder::Asc => {
                let range: Box<dyn Iterator<Item = (&SortableF64, &RoaringBitmap)>> =
                    match after_key {
                        Some(k) => Box::new(values.range(k..)),
                        None => Box::new(values.iter()),
                    };
                'asc: for (v, docs) in range {
                    for id in docs {
                        if !past_cursor(v.bits(), id) {
                            continue;
                        }
                        if query_predicate(coll, &req.query, id)? {
                            total += 1;
                            if page.len() < want {
                                page.push((id, v.to_f64() as f32));
                            } else if !req.track_total {
                                break 'asc;
                            }
                        }
                    }
                }
            }
            SortOrder::Desc => {
                let range: Box<dyn Iterator<Item = (&SortableF64, &RoaringBitmap)>> =
                    match after_key {
                        Some(k) => Box::new(values.range(..=k).rev()),
                        None => Box::new(values.iter().rev()),
                    };
                'desc: for (v, docs) in range {
                    for id in docs {
                        if !past_cursor(v.bits(), id) {
                            continue;
                        }
                        if query_predicate(coll, &req.query, id)? {
                            total += 1;
                            if page.len() < want {
                                page.push((id, v.to_f64() as f32));
                            } else if !req.track_total {
                                break 'desc;
                            }
                        }
                    }
                }
            }
        }
        if !req.track_total {
            total = total.max(page.len() as u64);
        }
        return Ok(Some((page, total, PlanKind::SortedField)));
    }

    // ---- no sort: standalone term — page = first `limit` of the posting,
    // total = posting length (no HashMap build / sort of the full posting). ----
    if let QueryNode::Term(t) = &req.query {
        // Keyword (2h-1) and Set (2h-2) route through the unified accessor;
        // Number stays by-reference. A `Cow` unifies the borrowed (segment OFF)
        // and owned (segment ON) postings so the page+total slice is one path.
        let posting: Option<std::borrow::Cow<RoaringBitmap>> =
            match (coll.fields.get(&t.field), &t.value) {
                (Some(FieldIndex::Keyword(k)), FieldValue::String(s)) => k.term_postings(s),
                // Phase 2h-3: Number exact-match through the unified accessor (segment
                // count-prefix index + tail when sealed, else the live posting).
                (Some(FieldIndex::Number(n)), FieldValue::Number(x)) => {
                    match SortableF64::new(*x) {
                        Ok(key) => n.value_postings(key),
                        Err(_) => return Ok(None),
                    }
                }
                (Some(FieldIndex::Set(s)), FieldValue::String(el)) => s.element_postings(el),
                _ => return Ok(None), // type mismatch → fall back (eval_term reports it)
            };
        let page: Vec<(u32, f32)> = posting
            .as_deref()
            .into_iter()
            .flat_map(|p| p.iter())
            .take(want)
            .map(|id| (id, 1.0))
            .collect();
        let total = posting.as_deref().map(|p| p.len()).unwrap_or(0);
        return Ok(Some((page, total, PlanKind::Posting)));
    }

    // ---- no sort: standalone range early-termination ----
    if let QueryNode::Range(r) = &req.query {
        let Some(FieldIndex::Number(n)) = coll.fields.get(&r.field) else {
            return Ok(None);
        };
        let (lo, hi) = range_bounds(r)?;
        // An empty/inverted range yields an empty page (and avoids the
        // `BTreeMap::range` panic on such bounds).
        if range_is_empty(lo, hi) {
            return Ok(Some((Vec::new(), 0, PlanKind::Posting)));
        }
        let mut page: Vec<(u32, f32)> = Vec::with_capacity(want.min(1024));
        let mut total: u64 = 0;
        if let Some(seg) = n.segment_ref() {
            let (page, total) =
                n.range_page_segment(seg.as_ref(), lo, hi, want, req.track_total)?;
            return Ok(Some((page, total, PlanKind::Posting)));
        }

        // Segment OFF: zero-clone range walk over the in-RAM `values` BTreeMap.
        let values = n.sorted_values();
        for (_v, docs) in values.range((lo, hi)) {
            // Exact total is a cheap sum of bucket lengths — no per-doc work.
            total += docs.len() as u64;
            if page.len() < want {
                for id in docs {
                    page.push((id, 1.0));
                    if page.len() >= want {
                        break;
                    }
                }
            } else if !req.track_total {
                break;
            }
        }
        return Ok(Some((page, total, PlanKind::Posting)));
    }

    // ---- no sort: filter-only AND/NOT-AND with exact total + first page ----
    //
    // The general evaluator materializes these constant-score shapes into a
    // HashMap and then ranks every matching doc. For a pure filter every score
    // is identical and result order is unspecified (same as standalone term /
    // range planners above), so the bitmap is already the answer set: return its
    // first page while preserving the exact total.
    if let Some((page, total)) = eval_filter_only_page(coll, &req.query, want)? {
        return Ok(Some((page, total, PlanKind::Posting)));
    }
    if let Some((bitmap, score)) = eval_filter_only_bitmap(coll, &req.query)? {
        let total = bitmap.len();
        let page = bitmap
            .into_iter()
            .take(want)
            .map(|id| (id, score))
            .collect();
        return Ok(Some((page, total, PlanKind::Posting)));
    }

    Ok(None)
}

// ---------------------------------------------------------------------------
// Cursor helpers — opaque base64 tokens in two generations:
//   v1 (legacy)  {"offset": N}                    — offset skip, O(offset) deep
//   v2 keyset    {"v":2,"m":"sort","k":bits,"d":docid}
//                {"v":2,"m":"score","k":score_bits,"t":external_id}
// Keyset cursors carry the LAST hit's position so the next page SEEKS to it
// (sorted walks: O(log n) range/binary-search start; score ranking: filter +
// top-`limit` heap instead of top-(offset+limit)) — deep pagination cost no
// longer grows with depth. v1 cursors keep working (legacy skip path).
// ---------------------------------------------------------------------------

/// A parsed pagination cursor.
#[derive(Debug)]
enum PageCursor {
    /// Legacy offset skip.
    Offset(u64),
    /// Continue a single-number-field sorted walk after (sort-value bits, docid).
    SortKeyset { bits: u64, docid: u32 },
    /// Continue a keyword or composite sorted walk after (sort-key tuple, docid).
    SortValuesKeyset { values: Vec<SortValue>, docid: u32 },
    /// Continue a score-ranked page after (score bits, external_id).
    ScoreKeyset { score_bits: u32, eid: String },
}

fn encode_cursor(json: String) -> String {
    use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
    STANDARD_NO_PAD.encode(json)
}

fn make_cursor(offset: usize) -> String {
    encode_cursor(format!("{{\"offset\":{offset}}}"))
}

fn make_sort_cursor(bits: u64, docid: u32) -> String {
    encode_cursor(format!(
        "{{\"v\":2,\"m\":\"sort\",\"k\":{bits},\"d\":{docid}}}"
    ))
}

fn make_sort_values_cursor(values: &[SortValue], docid: u32) -> String {
    let keys: Vec<serde_json::Value> = values
        .iter()
        .map(|value| match value {
            SortValue::Number(bits) => serde_json::json!({"n": bits}),
            SortValue::Keyword(term) => serde_json::json!({"s": term}),
        })
        .collect();
    let payload = serde_json::json!({"v": 2, "m": "sortv", "k": keys, "d": docid});
    encode_cursor(payload.to_string())
}

fn make_score_cursor(score: f32, eid: &str) -> String {
    let payload = serde_json::json!({"v": 2, "m": "score", "k": score.to_bits(), "t": eid});
    encode_cursor(payload.to_string())
}

fn parse_page_cursor(s: &str) -> Option<PageCursor> {
    use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
    let raw = STANDARD_NO_PAD.decode(s).ok()?;
    let v: serde_json::Value = serde_json::from_slice(&raw).ok()?;
    if let Some(offset) = v.get("offset").and_then(|o| o.as_u64()) {
        return Some(PageCursor::Offset(offset));
    }
    if v.get("v")?.as_u64()? != 2 {
        return None;
    }
    match v.get("m")?.as_str()? {
        "sort" => Some(PageCursor::SortKeyset {
            bits: v.get("k")?.as_u64()?,
            docid: v.get("d")?.as_u64()? as u32,
        }),
        "sortv" => {
            let values = v
                .get("k")?
                .as_array()?
                .iter()
                .map(|value| {
                    if let Some(bits) = value.get("n").and_then(|n| n.as_u64()) {
                        return Some(SortValue::Number(bits));
                    }
                    value
                        .get("s")
                        .and_then(|s| s.as_str())
                        .map(|s| SortValue::Keyword(s.to_string()))
                })
                .collect::<Option<Vec<_>>>()?;
            Some(PageCursor::SortValuesKeyset {
                values,
                docid: v.get("d")?.as_u64()? as u32,
            })
        }
        "score" => Some(PageCursor::ScoreKeyset {
            score_bits: v.get("k")?.as_u64()? as u32,
            eid: v.get("t")?.as_str()?.to_string(),
        }),
        _ => None,
    }
}

fn search_cache_key(req: &SearchRequest) -> Result<String> {
    serde_json::to_string(req).map_err(Into::into)
}

// ---------------------------------------------------------------------------
// Snapshot wire types
// ---------------------------------------------------------------------------

const SNAPSHOT_VERSION: u32 = 1;

/// Top-level snapshot document. JSON-serialisable.
#[derive(Debug, Serialize, Deserialize)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
pub struct SnapshotV1 {
    /// Format version. Bump when the wire layout changes
    /// incompatibly so old snapshots can be detected at restore.
    pub version: u32,
    pub collections: BTreeMap<String, CollectionSnapshot>,
}

#[derive(Debug, Serialize, Deserialize)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
pub struct CollectionSnapshot {
    pub schema: BTreeMap<String, FieldSpec>,
    pub version: u32,
    pub eid_fields: HashMap<String, BTreeSet<String>>,
    pub fields: BTreeMap<String, FieldIndexSnapshot>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
pub enum FieldIndexSnapshot {
    Text {
        analyzer: Analyzer,
        tokens: BTreeMap<String, BTreeMap<String, u32>>,
        forward: HashMap<String, (BTreeSet<String>, u32)>,
        doc_count: u64,
        total_doc_len: u64,
        bytes: u64,
    },
    Keyword {
        terms: BTreeMap<String, BTreeSet<String>>,
        forward: HashMap<String, String>,
        bytes: u64,
    },
    Number {
        /// Stored as `f64` on the wire; `SortableF64` is re-derived on
        /// restore.
        forward: HashMap<String, f64>,
        bytes: u64,
    },
    Set {
        elements: BTreeMap<String, BTreeSet<String>>,
        forward: HashMap<String, BTreeSet<String>>,
        bytes: u64,
    },
    /// Vector snapshot.
    ///
    /// HNSW graphs are not serialized directly — on restore the
    /// vectors are bulk-reinserted into a fresh graph, which is fast
    /// enough (millions per second on CPU) and avoids tying us to the
    /// upstream graph format. The codebook is carried verbatim when
    /// SQ is enabled so decoding reproduces the exact same f32 values
    /// that were originally indexed.
    Vector {
        spec: VectorSpec,
        vectors: Vec<(String, Vec<f32>)>,
        codebook: Option<ScalarCodebook>,
        bytes: u64,
    },
    Hash {
        /// external_id → 64-bit hash.
        forward: HashMap<String, u64>,
        bytes: u64,
    },
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl Collection {
    fn to_snapshot(&self) -> Result<CollectionSnapshot> {
        let mut fields: BTreeMap<String, FieldIndexSnapshot> = BTreeMap::new();
        for (name, fi) in &self.fields {
            fields.insert(name.clone(), fi.to_snapshot(&self.interner)?);
        }
        // The on-disk snapshot is String-keyed: resolve the dense doc-ids out
        // so the format is interner-independent and the e2e round-trips.
        let eid_fields = self
            .eid_fields
            .iter()
            .map(|(id, set)| (self.interner.resolve(*id).to_string(), set.to_btree_set()))
            .collect();
        Ok(CollectionSnapshot {
            schema: self.schema.clone(),
            version: self.version,
            eid_fields,
            fields,
        })
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl FieldIndex {
    fn to_snapshot(&self, interner: &Interner) -> Result<FieldIndexSnapshot> {
        let eid = |id: u32| interner.resolve(id).to_string();
        Ok(match self {
            FieldIndex::Text { analyzer, idx } => FieldIndexSnapshot::Text {
                analyzer: *analyzer,
                // After a Phase 2f-1 seal-and-drop the bulky `tokens` postings
                // live on the segment, not in RAM. A CBOR snapshot taken in that
                // state rebuilds them from the stored postings so the wire format
                // is unchanged; with no segment this is exactly the live `tokens`.
                tokens: {
                    let pairs: Vec<(String, Vec<u32>, Vec<u32>)> =
                        if idx.tokens.is_empty() && idx.segment.is_some() {
                            idx.segment
                                .as_ref()
                                .and_then(|s| s.text_tokens_all())
                                .ok_or_else(|| anyhow!("text segment postings torn on snapshot"))?
                        } else {
                            idx.tokens
                                .iter()
                                .map(|(tok, p)| (tok.clone(), p.docids.clone(), p.tfs.clone()))
                                .collect()
                        };
                    pairs
                        .into_iter()
                        .map(|(tok, docids, tfs)| {
                            (
                                tok,
                                docids
                                    .iter()
                                    .zip(&tfs)
                                    .map(|(id, tf)| (eid(*id), *tf))
                                    .collect(),
                            )
                        })
                        .collect()
                },
                // Wire forward = (distinct tokens, doc_len) per live doc. After a
                // Phase 2h-4 seal-and-drop `distinct`/`lens` live on the segment, not
                // in RAM; reconstruct each doc's distinct-token set by inverting the
                // stored postings (minus tombstoned base docids) and pair it with the
                // segment DocLen column, so the CBOR wire format is unchanged. With no
                // segment (live, pre-seal) this is exactly the live `distinct`/`lens`.
                forward: {
                    let map: HashMap<String, (BTreeSet<String>, u32)> =
                        if idx.distinct_is_empty() && idx.segment.is_some() {
                            let seg = idx.segment.as_ref().unwrap();
                            let entries = seg
                                .text_tokens_all()
                                .ok_or_else(|| anyhow!("text segment postings torn on snapshot"))?;
                            let mut rebuilt: HashMap<u32, BTreeSet<String>> = HashMap::new();
                            for (tok, docids, _tfs) in entries {
                                for &id in &docids {
                                    if !idx.tombstones.contains(id) {
                                        rebuilt.entry(id).or_default().insert(tok.clone());
                                    }
                                }
                            }
                            rebuilt
                                .into_iter()
                                .map(|(id, set)| (eid(id), (set, idx.doc_len(id))))
                                .collect()
                        } else {
                            idx.distinct_iter()
                                .map(|(id, set)| (eid(id), (set.to_btree_set(), idx.doc_len(id))))
                                .collect()
                        };
                    map
                },
                doc_count: idx.doc_count,
                total_doc_len: idx.total_doc_len,
                bytes: idx.bytes,
            },
            FieldIndex::Keyword(k) => FieldIndexSnapshot::Keyword {
                terms: k
                    .terms
                    .iter()
                    .map(|(v, set)| (v.clone(), set.iter().map(|id| eid(id)).collect()))
                    .collect(),
                forward: (0..interner.to_eid.len() as u32)
                    .filter_map(|id| k.keyword_at(id).map(|value| (eid(id), value)))
                    .collect(),
                bytes: k.bytes,
            },
            FieldIndex::Number(n) => FieldIndexSnapshot::Number {
                forward: (0..interner.to_eid.len() as u32)
                    .filter_map(|id| n.live_number_at(id).map(|key| (eid(id), key.to_f64())))
                    .collect(),
                bytes: n.bytes,
            },
            FieldIndex::Set(s) => FieldIndexSnapshot::Set {
                elements: s
                    .elements
                    .iter()
                    .map(|(el, set)| (el.clone(), set.iter().map(|id| eid(id)).collect()))
                    .collect(),
                forward: s
                    .forward
                    .iter()
                    .map(|(id, set)| (eid(*id), set.clone()))
                    .collect(),
                bytes: s.bytes,
            },
            FieldIndex::Vector { spec, idx, bytes } => {
                let (vectors, codebook) = idx.dump_for_snapshot()?;
                FieldIndexSnapshot::Vector {
                    spec: *spec,
                    vectors,
                    codebook,
                    bytes: *bytes,
                }
            }
            FieldIndex::Hash(h) => FieldIndexSnapshot::Hash {
                forward: h.forward.iter().map(|(id, v)| (eid(*id), *v)).collect(),
                bytes: h.bytes,
            },
        })
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl Collection {
    fn from_snapshot(snap: CollectionSnapshot) -> Result<Self> {
        // Re-intern every external_id (eid_fields covers all indexed docs) so
        // the field postings below resolve to the same dense ids.
        let mut interner = Interner::default();
        let mut eid_fields: FastHashMap<u32, FieldCoverage> = FastHashMap::default();
        for (eid, set) in snap.eid_fields {
            let id = interner.intern(&eid);
            eid_fields.insert(id, FieldCoverage::from_btree_set(set));
        }
        let mut fields: FastHashMap<String, FieldIndex> = FastHashMap::default();
        for (name, fi_snap) in snap.fields {
            fields.insert(name, FieldIndex::from_snapshot(fi_snap, &mut interner)?);
        }
        Ok(Self {
            version: snap.version,
            schema: snap.schema,
            fields,
            interner,
            eid_fields,
            seen_requests: VecDeque::new(),
            deleted_at: None,
            last_indexed_at: None,
            search_cache: RwLock::new(FastHashMap::default()),
        })
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl FieldIndex {
    fn from_snapshot(snap: FieldIndexSnapshot, interner: &mut Interner) -> Result<Self> {
        Ok(match snap {
            FieldIndexSnapshot::Text {
                analyzer,
                tokens,
                forward,
                doc_count,
                total_doc_len,
                bytes,
            } => {
                let mut t: FastHashMap<String, Postings> = FastHashMap::default();
                for (tok, m) in tokens {
                    // Re-intern (assigns fresh ids in arbitrary order), then sort
                    // by docid so the flat postings are ascending.
                    let mut pairs: Vec<(u32, u32)> = m
                        .into_iter()
                        .map(|(eid, tf)| (interner.intern(&eid), tf))
                        .collect();
                    pairs.sort_unstable_by_key(|(id, _)| *id);
                    let mut p = Postings::default();
                    for (id, tf) in pairs {
                        p.docids.push(id);
                        p.tfs.push(tf);
                    }
                    t.insert(tok, p);
                }
                let mut lens: Vec<u32> = Vec::new();
                let mut distinct: Vec<Option<TokenSet>> = Vec::new();
                for (eid, (set, doc_len)) in forward {
                    let id = interner.intern(&eid);
                    if lens.len() <= id as usize {
                        lens.resize(id as usize + 1, 0);
                    }
                    lens[id as usize] = doc_len;
                    if distinct.len() <= id as usize {
                        distinct.resize_with(id as usize + 1, || None);
                    }
                    distinct[id as usize] = Some(TokenSet::from_btree_set(set));
                }
                FieldIndex::Text {
                    analyzer,
                    idx: TextIndex {
                        tokens: t,
                        lens,
                        distinct,
                        doc_count,
                        total_doc_len,
                        bytes,
                        // A rehydrated snapshot has no sealed segment yet;
                        // sealing is a runtime disk-tier action, not restore.
                        segment: None,
                        match_rank_cache: RwLock::new(FastHashMap::default()),
                        // No segment → no sealed-base deletes pending; empty tombstone.
                        tombstones: RoaringBitmap::new(),
                    },
                }
            }
            FieldIndexSnapshot::Keyword {
                terms,
                forward,
                bytes,
            } => {
                let mut t: BTreeMap<String, RoaringBitmap> = BTreeMap::new();
                for (v, set) in terms {
                    t.insert(v, set.iter().map(|eid| interner.intern(eid)).collect());
                }
                let mut fwd: FastHashMap<u32, String> = FastHashMap::default();
                for (eid, v) in forward {
                    fwd.insert(interner.intern(&eid), v);
                }
                FieldIndex::Keyword(KeywordIndex {
                    dup_values: dup_values_of(&t),
                    terms: t,
                    dense_forward: Vec::new(),
                    forward: fwd,
                    bytes,
                    // A rehydrated snapshot has no sealed segment yet.
                    segment: None,
                    // No segment → no sealed-base deletes pending; empty tombstone.
                    tombstones: RoaringBitmap::new(),
                })
            }
            FieldIndexSnapshot::Number { forward, bytes } => {
                let mut idx = NumberIndex {
                    bytes,
                    ..NumberIndex::default()
                };
                for (eid, raw) in forward {
                    let id = interner.intern(&eid);
                    let key = SortableF64::new(raw)?;
                    idx.values.entry(key).or_default().insert(id);
                    idx.set_number(id, key);
                }
                idx.dup_values = dup_values_of(&idx.values);
                FieldIndex::Number(idx)
            }
            FieldIndexSnapshot::Set {
                elements,
                forward,
                bytes,
            } => {
                let mut e: BTreeMap<String, RoaringBitmap> = BTreeMap::new();
                for (el, set) in elements {
                    e.insert(el, set.iter().map(|eid| interner.intern(eid)).collect());
                }
                let mut fwd: FastHashMap<u32, BTreeSet<String>> = FastHashMap::default();
                for (eid, set) in forward {
                    fwd.insert(interner.intern(&eid), set);
                }
                FieldIndex::Set(SetIndex {
                    dup_values: dup_values_of(&e),
                    elements: e,
                    forward: fwd,
                    bytes,
                    // A rehydrated snapshot has no sealed segment yet.
                    segment: None,
                    // No segment → no sealed-base deletes pending; empty tombstone.
                    tombstones: RoaringBitmap::new(),
                })
            }
            FieldIndexSnapshot::Vector {
                spec,
                vectors,
                codebook,
                bytes,
            } => {
                // Restore the declared backend. Exact backends (flat-cpu)
                // restore exact; HNSW restores its graph (the graph is
                // rebuildable, the raw vectors persist).
                let idx: Box<dyn VectorIndex> = match spec.backend {
                    crate::types::VectorBackend::FlatCpu => {
                        Box::new(FlatCpuIndex::restore(spec, vectors, codebook)?)
                    }
                    _ => Box::new(HnswCpuIndex::restore(spec, vectors, codebook)?),
                };
                FieldIndex::Vector { spec, idx, bytes }
            }
            FieldIndexSnapshot::Hash { forward, bytes } => {
                let mut fwd: FastHashMap<u32, u64> = FastHashMap::default();
                for (eid, v) in forward {
                    fwd.insert(interner.intern(&eid), v);
                }
                FieldIndex::Hash(HashIndex {
                    forward: fwd,
                    bytes,
                    // A rehydrated snapshot has no sealed segment yet.
                    segment: None,
                })
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Production seal + reopen (Stage 2 Phase 2f-1): the RAM=hot / disk=all keystone
// ---------------------------------------------------------------------------
//
// `Collection::seal_to_segments` promotes the per-field test-seam logic into one
// collection-level call: for every field it writes `<field>.lseg` (reusing the
// `write_*_segment` writers), writes the collection EID column
// (`<collection>.lmeta.lseg`), attaches each `SegmentReader`, then DROPS the
// now-on-disk forward payload from RAM. For the INVERTED/RANGE-driver fields the
// driver index is ALSO dropped — the inverted postings live on disk: Keyword
// `terms` (2h-1, `ROLE_KEYWORD_POSTINGS`), Set `elements` (2h-2,
// `ROLE_SET_POSTINGS`), Number `values` (2h-3, the `ROLE_NUMBER_SORTED`
// sorted-value column + `ROLE_NUMBER_POSTINGS`), Text `tokens` (the
// not-rebuildable tf postings). After the drop, every forward read (predicate,
// value retrieval, drop_eid) routes through the segment-aware accessors
// (`number_at`/`keyword_at`/`set_at`/`hash_at`/`tok_postings`) and every inverted
// read through the unified posting accessors (`value_postings`/`range_postings`/
// `term_postings`/`element_postings`), so no code reads a dropped map for a base
// docid. A per-field query-time `tombstones` bitmap absorbs base deletes between
// seals (the on-disk postings are immutable).
//
// `Collection::open_from_segments` reopens a collection from those segments with
// NO CBOR snapshot and NO whole-collection load: it mmaps the EID column to
// rebuild the `Interner`, then mmaps each field segment. The inverted/range
// drivers are NOT rebuilt in RAM — Keyword `terms`, Set `elements`, and Number
// `values` stay EMPTY and queries drive from the on-disk posting / sorted-value
// columns (Text rebuilds `tokens` only because tf is not reconstructable from a
// forward column; Vector rebuilds its graph from the f32 column). The forward
// PAYLOAD stays on the mmap (demand-paged). RAM after reopen is O(live tail),
// not O(distinct values) — the 2h RAM win.

// The production seal/open are driven by the triple-path test today and by the
// runtime segment-persistence path (`--persistence=segment`); in a non-segment
// build they are reachable only from that runtime path, so this block silences
// dead-code in the default (CBOR) configuration rather than carry
// premature plumbing — mirroring `segment.rs`'s `#![cfg_attr(not(test), …)]`.
// The segment READ paths that serve live queries are fully exercised and are
// NOT covered by this allow.
#[cfg_attr(not(test), allow(dead_code))]
const EID_META_FILE: &str = "_collection.lmeta.lseg";

#[cfg_attr(not(test), allow(dead_code))]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl FieldIndex {
    /// Seal this field's forward payload into `<field>.lseg` under `dir`, attach
    /// the reader, and DROP the in-RAM forward payload. Keeps the inverted
    /// driver in RAM. `n_docs` is the collection's dense docid count; `applied_seq`
    /// is the WAL position this seal is current as of. Vector fields return their
    /// row→eid mapping (the vector segment stores only f32 rows), which the caller
    /// persists alongside; other field types return `None`. Idempotent fields
    /// (no value for an id) seal as absent rows.
    ///
    /// TOMBSTONE GC (Phase 2g-A): `live(id)` is the authoritative liveness
    /// predicate for THIS field — `true` iff `eid_fields[id]` still contains the
    /// field (i.e. the doc has NOT been deleted from it). The re-seal gather reads
    /// each base id's value through the segment-aware accessor, which still returns
    /// the IMMUTABLE prior-segment value for a deleted base doc; gating on `live`
    /// writes `None` for a non-live id so the new segment's present-bitset excludes
    /// it, and reopen's `record_field_coverage` never re-adds it — the delete is
    /// GC'd instead of resurrected. A first-time seal of an undeleted corpus has
    /// `live(id) == true` for every value-bearing id, so the gather is unchanged.
    fn seal_to_segment(
        &mut self,
        field_name: &str,
        dir: &std::path::Path,
        n_docs: u32,
        applied_seq: u64,
        live: &dyn Fn(u32) -> bool,
    ) -> Result<Option<Vec<String>>> {
        let path = dir.join(format!("{field_name}.lseg"));
        match self {
            FieldIndex::Number(n) => {
                // RE-SEAL-CAPABLE gather (Phase 2f-2): read each base doc's value
                // through the SEGMENT-AWARE accessor (`number_at`), NOT the raw
                // `forward` map. After a prior seal+drop the forward map is empty
                // for base ids; `number_at` re-materializes them from the prior
                // segment, while the live tail (ids >= prior n_docs) comes from
                // the in-RAM `forward`. A first-time seal reads entirely from
                // `forward` (no segment yet) — byte-identical to the old gather.
                let values: Vec<Option<f64>> = (0..n_docs)
                    .map(|id| {
                        if !live(id) {
                            return None; // deleted base doc → GC'd, not re-sealed
                        }
                        n.number_at(id).map(|s| s.to_f64())
                    })
                    .collect();
                crate::segment::write_number_segment(&path, applied_seq, &values)?;
                let reader = crate::segment::SegmentReader::open(&path)?;
                // Attach the NEW reader (dropping any prior segment Arc) and free
                // BOTH the forward tail AND the inverted/range `values` driver —
                // the whole [0..n_docs) index (sorted-value column + per-value
                // postings) is now on disk; reopen/queries drive from the mmap.
                // The RAM win Phase 2h-3 targets: RAM after reopen is O(live tail),
                // not O(distinct numeric values).
                n.segment = Some(std::sync::Arc::new(reader));
                n.forward = FastHashMap::default();
                n.dense_forward = Vec::new();
                n.values = BTreeMap::new();
                n.dup_values = BTreeSet::new();
                n.clear_keyword_range_cache();
                // The new segment's live(id) gather already EXCLUDED every
                // tombstoned base docid (deleted-since-last-seal), so the deletions
                // are baked in as absent — reset the query-time tombstone to empty
                // (Phase 2h-3, mirroring Keyword 2h-1 / Set 2h-2). The gather above
                // reads `number_at`, which serves the PRIOR segment ignorant of the
                // tombstone, so the `live(id)` predicate is what actually drops the
                // deleted ids; this reset just retires the now-stale set.
                n.tombstones = RoaringBitmap::new();
                Ok(None)
            }
            FieldIndex::Hash(h) => {
                let values: Vec<Option<u64>> = (0..n_docs)
                    .map(|id| if live(id) { h.hash_at(id) } else { None })
                    .collect();
                crate::segment::write_hash_segment(&path, applied_seq, &values)?;
                let reader = crate::segment::SegmentReader::open(&path)?;
                h.segment = Some(std::sync::Arc::new(reader));
                h.forward = FastHashMap::default();
                Ok(None)
            }
            FieldIndex::Keyword(k) => {
                // RE-SEAL-CAPABLE gather (Phase 2h-1): read each base doc's value
                // through the SEGMENT-AWARE accessor (`keyword_at`), NOT the raw
                // `forward` map — after a prior seal+drop `forward` is empty for
                // base ids and `keyword_at` re-materializes them from the prior
                // segment. `keyword_at` yields owned Strings (the segment can only
                // own); gather them first, then borrow for the writer.
                let owned: Vec<Option<String>> = (0..n_docs)
                    .map(|id| if live(id) { k.keyword_at(id) } else { None })
                    .collect();
                let values: Vec<Option<&str>> = owned.iter().map(|o| o.as_deref()).collect();
                // INVERTED postings to seal: fold the just-gathered live values
                // into a fresh `terms` index. This is the SEGMENT-AWARE union
                // (prior segment base for deleted-excluded live ids + live tail)
                // already collapsed into `owned`, so deleted docids are dropped
                // and the new segment's postings are exactly the live inverted
                // index. A first-time seal folds the live values verbatim.
                let mut terms: BTreeMap<String, RoaringBitmap> = BTreeMap::new();
                for (id, v) in values.iter().enumerate() {
                    if let Some(s) = v {
                        terms.entry((*s).to_string()).or_default().insert(id as u32);
                    }
                }
                crate::segment::write_keyword_segment(&path, applied_seq, &values, &terms)?;
                let reader = crate::segment::SegmentReader::open(&path)?;
                // Attach the NEW reader (dropping any prior segment Arc) and free
                // BOTH the forward tail AND the inverted `terms` driver — the
                // whole [0..n_docs) index is now on disk; reopen/queries drive
                // from the mmap. The RAM win Phase 2h-1 targets.
                k.segment = Some(std::sync::Arc::new(reader));
                k.forward = FastHashMap::default();
                k.dense_forward = Vec::new();
                k.terms = BTreeMap::new();
                k.dup_values = BTreeSet::new();
                // The new segment's live(id) gather already EXCLUDED every
                // tombstoned base docid (deleted-since-last-seal), so the
                // deletions are now baked in as absent — reset the query-time
                // tombstone to empty (Phase 2h-1 FIX). The gather above reads
                // `keyword_at`, which serves the PRIOR segment ignorant of the
                // tombstone, so the `live(id)` predicate is what actually drops
                // the deleted ids; this reset just retires the now-stale set.
                k.tombstones = RoaringBitmap::new();
                Ok(None)
            }
            FieldIndex::Set(s) => {
                // RE-SEAL-CAPABLE gather (Phase 2h-2): read each base doc's
                // members through the SEGMENT-AWARE accessor (`set_members`), NOT
                // the raw `forward` map — after a prior seal+drop `forward` is
                // empty for base ids and `set_members` re-materializes them from
                // the prior segment. The deleted-excluded live ids + live tail
                // collapse into `owned`, so the new segment is exactly the live state.
                let owned: Vec<Option<Vec<String>>> = (0..n_docs)
                    .map(|id| {
                        if !live(id) {
                            return None; // deleted base doc → GC'd, not re-sealed
                        }
                        s.set_members(id).map(|set| set.into_iter().collect())
                    })
                    .collect();
                let values: Vec<Option<&[String]>> = owned.iter().map(|o| o.as_deref()).collect();
                // INVERTED postings to seal (Phase 2h-2): fold the just-gathered
                // live members into a fresh `elements` index. Deleted docids are
                // already dropped (gather wrote `None`), so the new segment's
                // postings are exactly the live inverted index.
                let mut elements: BTreeMap<String, RoaringBitmap> = BTreeMap::new();
                for (id, v) in values.iter().enumerate() {
                    if let Some(members) = v {
                        for m in members.iter() {
                            elements.entry(m.clone()).or_default().insert(id as u32);
                        }
                    }
                }
                crate::segment::write_set_segment(&path, applied_seq, &values, &elements)?;
                let reader = crate::segment::SegmentReader::open(&path)?;
                // Attach the NEW reader (dropping any prior segment Arc) and free
                // BOTH the forward tail AND the inverted `elements` driver — the
                // whole [0..n_docs) index is now on disk. The RAM win 2h-2 targets.
                s.segment = Some(std::sync::Arc::new(reader));
                s.forward = FastHashMap::default();
                s.elements = BTreeMap::new();
                s.dup_values = BTreeSet::new();
                // The new segment's live(id) gather already EXCLUDED every
                // tombstoned base docid, so deletions are baked in — reset the
                // query-time tombstone to empty (Phase 2h-2).
                s.tombstones = RoaringBitmap::new();
                Ok(None)
            }
            FieldIndex::Text { idx, .. } => {
                // RE-SEAL-CAPABLE postings gather: a re-seal after a prior drop has
                // an EMPTY `tokens` map (postings live on the prior segment, served
                // via `tok_postings`). Materialize the postings to seal from the
                // SEGMENT-AWARE source — the prior segment for base tokens, the live
                // `tokens` for the tail — into a fresh BTreeMap, then write that.
                // A first-time seal reads entirely from the live `tokens` (no
                // segment), byte-identical to the old direct `&idx.tokens` write.
                //
                // TOMBSTONE GC (Phase 2g-A): a deleted base doc was already removed
                // from the live `tokens`/`distinct` and decremented out of the live
                // corpus scalars by `drop_eid`, but its postings still live on the
                // IMMUTABLE prior segment, which `tokens_for_seal` reads back. Pass
                // `live` so the merged base postings drop deleted docids; the live
                // corpus scalars (`corpus_for_seal`) and live `lens` (`lens_for_seal`)
                // already exclude the deleted doc, so they need no further filtering.
                let lens: Vec<u32> = idx.lens_for_seal(n_docs, live);
                let tokens = idx.tokens_for_seal(live);
                let (doc_count, total_doc_len) = idx.corpus_for_seal();
                crate::segment::write_text_segment(
                    &path,
                    applied_seq,
                    &tokens,
                    &lens,
                    doc_count,
                    total_doc_len,
                )?;
                let reader = crate::segment::SegmentReader::open(&path)?;
                // Phase 2h-4: DROP the bulky `tokens` postings AND `distinct` to
                // disk — neither is rebuilt. `drop_eid` no longer consumes
                // `distinct` for a sealed base id (it records the id in
                // `tombstones` instead — the base postings are immutable on disk),
                // so the eager `distinct` invert is gone (it made post-seal RAM
                // O(corpus); this is the same RAM bound a reopen now holds). `lens`
                // is dropped too — `doc_len()` reads the segment DocLen column for a
                // sealed id. A re-seal CLEARS `tombstones` (the new segment baked
                // the deletes in via `tokens_for_seal`'s `live(id)` gather).
                idx.segment = Some(std::sync::Arc::new(reader));
                idx.tokens = FastHashMap::default(); // bulky postings now on disk
                idx.distinct = Vec::new(); // drop_eid uses tombstones for base ids
                idx.lens = Vec::new(); // doc_len reads the segment DocLen column
                idx.clear_match_rank_cache();
                idx.tombstones = RoaringBitmap::new(); // re-seal baked deletes in
                Ok(None)
            }
            FieldIndex::Vector { idx, bytes, .. } => {
                // The flat-cpu backend seals + drops its data buffer and hands
                // back the row→eid mapping; the collection persists it. A
                // non-flat backend (HNSW) returns None and is left in RAM.
                let row_eids = idx.seal_to_segment_prod(&path)?;
                if row_eids.is_some() {
                    // The contiguous scan buffer left RAM; reflect that in bytes.
                    *bytes = 0;
                }
                Ok(row_eids)
            }
        }
    }

    /// Reopen this field from its `<field>.lseg` segment with NO snapshot: mmap
    /// the forward column, attach the reader, and REBUILD the inverted driver in
    /// RAM by scanning the forward column. The forward payload stays on the mmap
    /// (demand-paged); only the driver is in RAM. `spec` supplies the field type
    /// (and the Vector sub-spec); `vec_row_eids` is the persisted vector row→eid
    /// mapping (only consulted for a Vector field).
    fn open_from_segment(
        spec: &FieldSpec,
        dir: &std::path::Path,
        field_name: &str,
        vec_row_eids: Option<Vec<String>>,
    ) -> Result<FieldIndex> {
        let path = dir.join(format!("{field_name}.lseg"));
        let reader = std::sync::Arc::new(crate::segment::SegmentReader::open(&path)?);
        // The per-arm reopen reads coverage straight off the reader (e.g. the Text
        // arm via `text_doc_count`/`text_total_doc_len`); no whole-`n_docs` rebuild.
        let _n_docs = reader.n_docs();
        match spec.field_type {
            FieldType::Number => {
                // Phase 2h-3: the inverted/range `values` index is ON DISK (the
                // ROLE_NUMBER_SORTED sorted-value column + ROLE_NUMBER_POSTINGS).
                // Reopen does NOT rebuild it in RAM — `values` stays EMPTY and
                // range / exact / boolean queries drive from the mmap segment via
                // `range_postings` / `value_postings` (binary-search on the sorted
                // column, SELECTIVE — NOT the old O(n_docs) forward scan that
                // rebuilt `values`). RAM after reopen is O(live tail), not
                // O(distinct numeric values). The forward column stays demand-paged
                // on the mmap for per-doc predicate reads (`number_at`).
                Ok(FieldIndex::Number(NumberIndex {
                    values: BTreeMap::new(),
                    dup_values: BTreeSet::new(),
                    forward: FastHashMap::default(), // payload stays on the mmap
                    dense_forward: Vec::new(),
                    keyword_range_cache: RwLock::new(FastHashMap::default()),
                    keyword_range_bitmap_cache: RwLock::new(FastHashMap::default()),
                    range_stats: RwLock::new(None),
                    bytes: 0,
                    segment: Some(reader),
                    // Reopen starts with NO pending deletes — the on-disk segment
                    // already reflects every deletion baked in at its seal.
                    tombstones: RoaringBitmap::new(),
                }))
            }
            FieldType::Hash => Ok(FieldIndex::Hash(HashIndex {
                forward: FastHashMap::default(),
                bytes: 0,
                segment: Some(reader),
            })),
            FieldType::Keyword => {
                // Phase 2h-1: the inverted `terms` index is ON DISK (the
                // ROLE_KEYWORD_POSTINGS column). Reopen does NOT rebuild it in
                // RAM — `terms` stays EMPTY and Term/Terms/boolean queries drive
                // from the mmap segment via `term_postings`. RAM after reopen is
                // O(live tail), not O(distinct keyword values). The old fold-the-
                // forward-column rebuild loop is gone; the forward dict-id column
                // stays demand-paged on the mmap for per-doc predicate reads.
                Ok(FieldIndex::Keyword(KeywordIndex {
                    terms: BTreeMap::new(),
                    dup_values: BTreeSet::new(),
                    dense_forward: Vec::new(),
                    forward: FastHashMap::default(),
                    bytes: 0,
                    segment: Some(reader),
                    // Reopen starts with NO pending deletes — the on-disk segment
                    // already reflects every deletion baked in at its seal.
                    tombstones: RoaringBitmap::new(),
                }))
            }
            FieldType::Set => {
                // Phase 2h-2: the inverted `elements` index is ON DISK (the
                // ROLE_SET_POSTINGS column). Reopen does NOT rebuild it in RAM —
                // `elements` stays EMPTY and membership / Terms / boolean queries
                // drive from the mmap segment via `element_postings`. RAM after
                // reopen is O(live tail), not O(distinct set elements). The old
                // fold-the-forward-column rebuild loop is gone; the CSR forward
                // columns stay demand-paged on the mmap for per-doc predicate reads.
                Ok(FieldIndex::Set(SetIndex {
                    elements: BTreeMap::new(),
                    dup_values: BTreeSet::new(),
                    forward: FastHashMap::default(),
                    bytes: 0,
                    segment: Some(reader),
                    // Reopen starts with NO pending deletes — the on-disk segment
                    // already reflects every deletion baked in at its seal.
                    tombstones: RoaringBitmap::new(),
                }))
            }
            FieldType::Text => {
                // Phase 2h-4: the inverted `tokens` postings (text tf is STORED on
                // disk — the ROLE_TEXT_POSTINGS blocks) and the corpus-deriving
                // `distinct` map stay EMPTY on reopen — NO RAM rebuild. The old loop
                // decoded EVERY posting to re-materialize `tokens`/`distinct`, making
                // RAM O(total tokens); it is deleted. The BM25 scan / term lookup
                // drive entirely from the mmap via `tok_postings` (decodes a single
                // token's posting block on demand) + `doc_len` (segment DocLen
                // column), so RAM after reopen is O(live tail), not O(corpus).
                //
                // `lens` stays EMPTY too: `doc_len()` prefers the segment DocLen
                // column for a sealed id, and any post-reopen tail doc extends `lens`
                // through `set_doc_len`. The LIVE corpus scalars are INITIALIZED from
                // the seal-time header (`text_doc_count`/`text_total_doc_len`); from
                // here the index path increments and `drop_eid` decrements them, so
                // `bm25_corpus` (which now reads them, NOT the header) stays current
                // across deletes and the post-seal tail. Tombstones start empty — the
                // segment already reflects every deletion baked in at its seal.
                let doc_count = reader.text_doc_count();
                let total_doc_len = reader.text_total_doc_len();
                let analyzer = spec.analyzer.unwrap_or(Analyzer::WhitespaceLower);
                Ok(FieldIndex::Text {
                    analyzer,
                    idx: TextIndex {
                        tokens: FastHashMap::default(),
                        lens: Vec::new(),
                        distinct: Vec::new(),
                        doc_count,
                        total_doc_len,
                        bytes: 0,
                        segment: Some(reader),
                        match_rank_cache: RwLock::new(FastHashMap::default()),
                        tombstones: RoaringBitmap::new(),
                    },
                })
            }
            FieldType::Vector => {
                let vs = spec.vector_spec()?.ok_or_else(|| {
                    anyhow!("vector field `{field_name}` is missing its sub-spec")
                })?;
                let row_eids = vec_row_eids.ok_or_else(|| {
                    anyhow!("vector field `{field_name}` reopen needs its row→eid mapping")
                })?;
                let idx = FlatCpuIndex::open_from_segment(vs, reader, row_eids)?;
                Ok(FieldIndex::Vector {
                    spec: vs,
                    idx: Box::new(idx),
                    bytes: 0,
                })
            }
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl Collection {
    /// PRODUCTION seal (Phase 2f-1): seal EVERY field into a columnar mmap
    /// segment under `dir`, write the collection EID column, attach each reader,
    /// and DROP the now-on-disk forward payload from RAM (the inverted driver
    /// indexes stay). After this the collection is reopenable from `dir` alone
    /// (no CBOR snapshot) and answers every query from the segments + drivers,
    /// reading the forward payload demand-paged off the mmaps. `applied_seq` is
    /// the WAL position the seal is current as of. The per-Vector-field row→eid
    /// mapping is persisted into a small sidecar (`<field>.eids.lseg`) so the
    /// vector index rebuilds on reopen.
    ///
    /// TOMBSTONE GC (Phase 2g-A): each field's per-doc gather is gated on the
    /// AUTHORITATIVE liveness fact `eid_fields[id].contains(field)`. A base doc
    /// deleted post-seal was removed from `eid_fields` (and the inverted driver)
    /// by `delete`/`drop_eid`, but its value still sits on the IMMUTABLE prior
    /// segment; gating the re-seal gather on `eid_fields` writes `None` for that
    /// id so the new segment excludes it and reopen never resurrects it.
    pub fn seal_to_segments(&mut self, dir: &std::path::Path, applied_seq: u64) -> Result<()> {
        std::fs::create_dir_all(dir)
            .map_err(|e| anyhow!("create seal dir {}: {e}", dir.display()))?;
        let n_docs = self.interner.to_eid.len() as u32;

        // 1) The collection EID column: external_id of docid i at position i.
        let eids: Vec<&str> = self.interner.to_eid.iter().map(|s| s.as_str()).collect();
        let meta_path = dir.join(EID_META_FILE);
        crate::segment::write_eid_segment(&meta_path, applied_seq, &eids)?;

        // 2) Seal every field; persist any Vector row→eid sidecar. Borrow
        //    `eid_fields` immutably for the liveness predicate while `fields` is
        //    borrowed mutably — they are disjoint struct fields, so this is sound.
        let eid_fields = &self.eid_fields;
        for (name, fi) in self.fields.iter_mut() {
            let live =
                |id: u32| -> bool { eid_fields.get(&id).is_some_and(|fs| fs.contains(name)) };
            let row_eids = fi.seal_to_segment(name, dir, n_docs, applied_seq, &live)?;
            if let Some(row_eids) = row_eids {
                let sidecar = dir.join(format!("{name}.eids.lseg"));
                let refs: Vec<&str> = row_eids.iter().map(|s| s.as_str()).collect();
                crate::segment::write_eid_segment(&sidecar, applied_seq, &refs)?;
            }
        }
        Ok(())
    }

    /// PRODUCTION reopen (Phase 2f-1): reconstruct a collection from the segments
    /// under `dir` with NO CBOR snapshot and NO whole-collection load. Rebuilds
    /// the `Interner` from the EID column, then each field from its `<field>.lseg`
    /// (rebuilding the inverted driver in RAM; the forward payload stays on the
    /// mmap). `schema` is the collection's field specs (carried out-of-band, e.g.
    /// from the catalog), needed to know each field's type before opening its
    /// segment. `eid_fields` (which fields each doc wrote) is reconstructed from
    /// the per-field segment coverage.
    pub fn open_from_segments(
        dir: &std::path::Path,
        schema: BTreeMap<String, FieldSpec>,
        version: u32,
    ) -> Result<Self> {
        // 1) Rebuild the interner from the EID column.
        let meta_path = dir.join(EID_META_FILE);
        let meta = crate::segment::SegmentReader::open(&meta_path)
            .map_err(|e| anyhow!("open eid meta {}: {e}", meta_path.display()))?;
        let to_eid = meta
            .eids_all()
            .ok_or_else(|| anyhow!("eid meta column torn on reopen"))?;
        let mut interner = Interner::default();
        for eid in &to_eid {
            interner.intern(eid);
        }

        // 2) Rebuild each field from its segment + reconstruct eid_fields from
        //    per-field coverage (a doc "wrote" a field iff that field's segment
        //    has a value for the doc — exactly what the live eid_fields tracked).
        let mut fields: FastHashMap<String, FieldIndex> = FastHashMap::default();
        let mut eid_fields: FastHashMap<u32, FieldCoverage> = FastHashMap::default();
        for (name, spec) in &schema {
            // Vector fields carry a row→eid sidecar.
            let vec_row_eids = if spec.field_type == FieldType::Vector {
                let sidecar = dir.join(format!("{name}.eids.lseg"));
                let r = crate::segment::SegmentReader::open(&sidecar)
                    .map_err(|e| anyhow!("open vector eid sidecar {}: {e}", sidecar.display()))?;
                Some(
                    r.eids_all()
                        .ok_or_else(|| anyhow!("vector eid sidecar `{name}` torn on reopen"))?,
                )
            } else {
                None
            };
            let fi = FieldIndex::open_from_segment(spec, dir, name, vec_row_eids)?;
            record_field_coverage(&fi, name, &interner, &mut eid_fields);
            fields.insert(name.clone(), fi);
        }

        Ok(Self {
            version,
            schema,
            fields,
            interner,
            eid_fields,
            seen_requests: VecDeque::new(),
            deleted_at: None,
            last_indexed_at: None,
            search_cache: RwLock::new(FastHashMap::default()),
        })
    }
}

/// Reconstruct, into `eid_fields`, the set of fields each doc-id wrote, from a
/// single reopened field's segment coverage. A doc wrote `name` iff the field's
/// segment holds a value for that doc — the same fact the live `eid_fields`
/// tracked at index time. Phase 2f-1 reopen helper.
#[cfg_attr(not(test), allow(dead_code))]
fn record_field_coverage(
    fi: &FieldIndex,
    name: &str,
    interner: &Interner,
    eid_fields: &mut FastHashMap<u32, FieldCoverage>,
) {
    match fi {
        FieldIndex::Number(n) => {
            if let Some(seg) = &n.segment {
                for id in 0..seg.n_docs() {
                    if n.number_at(id).is_some() {
                        eid_fields.entry(id).or_default().insert(name.to_string());
                    }
                }
            }
        }
        FieldIndex::Hash(h) => {
            if let Some(seg) = &h.segment {
                for id in 0..seg.n_docs() {
                    if h.hash_at(id).is_some() {
                        eid_fields.entry(id).or_default().insert(name.to_string());
                    }
                }
            }
        }
        FieldIndex::Keyword(k) => {
            if let Some(seg) = &k.segment {
                for id in 0..seg.n_docs() {
                    if k.keyword_at(id).is_some() {
                        eid_fields.entry(id).or_default().insert(name.to_string());
                    }
                }
            }
        }
        FieldIndex::Set(s) => {
            if let Some(seg) = &s.segment {
                for id in 0..seg.n_docs() {
                    if seg.set_at(id).is_some() {
                        eid_fields.entry(id).or_default().insert(name.to_string());
                    }
                }
            }
        }
        FieldIndex::Text { idx, .. } => {
            // A doc "wrote" a text field iff it emitted ≥1 token. After Phase 2h-4
            // reopen `distinct` is EMPTY (no RAM rebuild), so drive coverage from
            // the segment: a base id is present iff its DocLen column entry is > 0
            // (doc_len == tokens emitted; the old reopen rebuilt `distinct` from the
            // posting docids, which is exactly the set of ids with doc_len > 0).
            // This is a transient O(n_docs) read of the demand-paged DocLen column —
            // it does NOT re-materialize `tokens`/`distinct` in RAM. Any live-tail
            // doc indexed after reopen is folded in from `distinct_ids()` (empty
            // until then). Without a segment (live, pre-seal) this falls back to the
            // in-RAM `distinct` ids — byte-identical to the old behavior.
            if let Some(seg) = &idx.segment {
                for id in 0..seg.n_docs() {
                    if seg.text_doc_len(id) > 0 {
                        eid_fields.entry(id).or_default().insert(name.to_string());
                    }
                }
            }
            for id in idx.distinct_ids() {
                eid_fields.entry(id).or_default().insert(name.to_string());
            }
        }
        FieldIndex::Vector { idx, .. } => {
            // Every stored vector row's eid wrote this field; resolve each row
            // eid back to its docid through the rebuilt interner.
            for (eid, _) in idx.dump_for_snapshot().into_iter().flat_map(|(v, _)| v) {
                if let Some(id) = interner.id(&eid) {
                    eid_fields.entry(id).or_default().insert(name.to_string());
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Production checkpoint (Stage 2 Phase 2f-2): the disk engine as the running
// binary's persistence — a segment checkpoint supersedes the CBOR RDB.
// ---------------------------------------------------------------------------
//
// A checkpoint is a directory `dir/<collection>/` per collection, each holding
// `<field>.lseg` segments, the `_collection.lmeta.lseg` EID column, any vector
// `<field>.eids.lseg` sidecars, and a `_schema.json` (the field specs + version
// + applied_seq, carried out-of-band so reopen knows each field's type without a
// CBOR snapshot). `flush_to_segments` is the periodic snapshotter's call:
// re-seal-capable (`Collection::seal_to_segments` gathers base-doc values through
// the segment-aware dispatch, so a checkpoint AFTER a prior seal+drop is correct),
// idempotent, and repeatable. `reopen_from_segment_dir` is cold-start: reopen
// every collection via `Collection::open_from_segments` (no whole-collection load)
// and return the max applied_seq so the WAL tail replays from there.
//
// Atomicity is the caller's (`SegmentRdbStore`): it stages a whole generation
// under a temp dir and atomically renames it into place, so a torn checkpoint
// never replaces a good one. `flush_to_segments` writes into whatever `dir` it is
// handed; it does not own the atomic-rename.

/// Per-collection checkpoint sidecar persisted next to the segments so a reopen
/// knows each field's type + the collection version + the WAL position the seal
/// is current as of — the schema the live `Collection::open_from_segments` needs
/// out-of-band. Phase 2f-2.
#[derive(Debug, Serialize, Deserialize)]
struct CheckpointSchema {
    version: u32,
    applied_seq: u64,
    fields: BTreeMap<String, FieldSpec>,
}

const CHECKPOINT_SCHEMA_FILE: &str = "_schema.json";

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl Engine {
    /// PRODUCTION checkpoint (Phase 2f-2): seal EVERY live collection into a
    /// segment checkpoint under `dir` — one subdir `dir/<collection>/` per
    /// collection holding its `<field>.lseg` segments, EID column, vector
    /// sidecars, and a `_schema.json` sidecar tagged with `up_to_seq`. Each
    /// collection is sealed under the state WRITE lock (so a concurrent index does
    /// not race the seal), via the re-seal-capable `Collection::seal_to_segments`
    /// (base docs are gathered through the segment-aware dispatch, so a checkpoint
    /// AFTER a prior seal+drop re-materializes correctly). Soft-deleted collections
    /// are skipped. The flush is idempotent and repeatable: flush → index more →
    /// flush again → reopen yields every doc identical.
    ///
    /// Atomicity is the caller's: `dir` is expected to be a staging directory that
    /// the store atomically renames into place, so this never half-replaces a good
    /// checkpoint. The state lock is taken and released PER collection (not held
    /// across the whole flush), so reads/writes to other collections proceed.
    pub fn flush_to_segments(&self, dir: &std::path::Path, up_to_seq: u64) -> Result<()> {
        std::fs::create_dir_all(dir)
            .map_err(|e| anyhow!("create checkpoint dir {}: {e}", dir.display()))?;
        // Snapshot the live collection names under a read lock, then seal each one
        // under its own short write-lock window.
        let names: Vec<String> = {
            let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
            state.collections.keys().cloned().collect()
        };
        for name in names {
            let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
            let Some(coll) = state.collections.get_mut(&name) else {
                continue; // dropped between snapshot and seal
            };
            if coll.deleted_at.is_some() {
                continue; // soft-deleted: not part of the checkpoint
            }
            let coll_dir = dir.join(collection_dir_name(&name));
            std::fs::create_dir_all(&coll_dir)
                .map_err(|e| anyhow!("create checkpoint subdir {}: {e}", coll_dir.display()))?;
            // Persist the schema sidecar BEFORE the segments so a reopen that finds
            // the segments always finds the schema too (the store's atomic rename
            // makes the whole subdir visible at once regardless of write order).
            let sidecar = CheckpointSchema {
                version: coll.version,
                applied_seq: up_to_seq,
                fields: coll.schema.clone(),
            };
            let schema_path = coll_dir.join(CHECKPOINT_SCHEMA_FILE);
            let json = serde_json::to_vec_pretty(&sidecar)
                .map_err(|e| anyhow!("encode checkpoint schema for `{name}`: {e}"))?;
            std::fs::write(&schema_path, &json)
                .map_err(|e| anyhow!("write checkpoint schema {}: {e}", schema_path.display()))?;
            coll.seal_to_segments(&coll_dir, up_to_seq)?;
        }
        Ok(())
    }

    /// PRODUCTION cold-start (Phase 2f-2): reopen EVERY collection's checkpoint
    /// under `dir` into THIS engine and return the max applied_seq across them (0
    /// if `dir` has no collections). Each `dir/<collection>/` is reopened via the
    /// re-seal-capable `Collection::open_from_segments` (no CBOR snapshot, no
    /// whole-collection load — the forward payload stays demand-paged on the
    /// mmaps). The returned seq is the WAL position the binary tails from.
    pub fn reopen_from_segment_dir(&self, dir: &std::path::Path) -> Result<u64> {
        if !dir.exists() {
            return Ok(0);
        }
        let mut max_seq = 0u64;
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        for entry in std::fs::read_dir(dir)
            .map_err(|e| anyhow!("read checkpoint dir {}: {e}", dir.display()))?
        {
            let entry = entry.map_err(|e| anyhow!("read checkpoint entry: {e}"))?;
            let coll_dir = entry.path();
            if !coll_dir.is_dir() {
                continue;
            }
            let schema_path = coll_dir.join(CHECKPOINT_SCHEMA_FILE);
            if !schema_path.exists() {
                continue; // not a collection checkpoint subdir
            }
            let json = std::fs::read(&schema_path)
                .map_err(|e| anyhow!("read checkpoint schema {}: {e}", schema_path.display()))?;
            let sidecar: CheckpointSchema = serde_json::from_slice(&json)
                .map_err(|e| anyhow!("decode checkpoint schema {}: {e}", schema_path.display()))?;
            let name = collection_name_from_dir(&coll_dir)
                .ok_or_else(|| anyhow!("undecodable checkpoint subdir {}", coll_dir.display()))?;
            let coll = Collection::open_from_segments(&coll_dir, sidecar.fields, sidecar.version)?;
            max_seq = max_seq.max(sidecar.applied_seq);
            state.collections.insert(name, coll);
        }
        Ok(max_seq)
    }

    /// PUBLIC PROBE (Stage 2 disk-tier validation): report whether a field's
    /// in-RAM forward/inverted driver has been DROPPED to disk and a segment is
    /// attached, so an out-of-crate consumer (the disk perf gate integration
    /// test) can assert the query path is GENUINELY segment-driven rather than
    /// silently still in RAM. Returns `(forward_or_tokens_len, has_segment)` for
    /// the named field: after a `flush_to_segments`, a sealed field reads
    /// `forward_or_tokens_len == 0` (driver dropped) AND `has_segment == true`
    /// (mmap attached). Mirrors the in-crate `__field_forward_probe`, but is a
    /// real `pub` API (not `#[cfg(test)]`) so `tests/perf_gate_vs_db.rs` — a
    /// separate crate — can read it. Experimental-gated alongside the rest of
    /// the disk tier.
    pub fn segment_field_probe(&self, collection_id: &str, field: &str) -> Result<(usize, bool)> {
        let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get(collection_id)
            .ok_or_else(|| anyhow!("unknown collection `{collection_id}`"))?;
        let fi = coll
            .fields
            .get(field)
            .ok_or_else(|| anyhow!("unknown field `{field}`"))?;
        Ok(match fi {
            FieldIndex::Number(n) => (n.forward_len(), n.segment.is_some()),
            FieldIndex::Hash(h) => (h.forward.len(), h.segment.is_some()),
            FieldIndex::Keyword(k) => (k.forward_len(), k.segment.is_some()),
            FieldIndex::Set(s) => (s.forward.len(), s.segment.is_some()),
            FieldIndex::Text { idx, .. } => (idx.tokens.len(), idx.segment.is_some()),
            FieldIndex::Vector { .. } => (0, true),
        })
    }
}

/// A collection id → filename-safe subdir name (hex-encoded), so any
/// collection id is a valid directory.
fn collection_dir_name(name: &str) -> String {
    name.bytes().map(|b| format!("{b:02x}")).collect()
}

/// Decode a checkpoint subdir's hex-encoded name back to the collection id.
/// `None` if the leaf is not valid hex (a stray file/dir in the checkpoint).
fn collection_name_from_dir(dir: &std::path::Path) -> Option<String> {
    let leaf = dir.file_name()?.to_str()?;
    if leaf.is_empty() || leaf.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::with_capacity(leaf.len() / 2);
    let raw = leaf.as_bytes();
    let mut i = 0;
    while i < raw.len() {
        let hi = (raw[i] as char).to_digit(16)?;
        let lo = (raw[i + 1] as char).to_digit(16)?;
        bytes.push((hi * 16 + lo) as u8);
        i += 2;
    }
    String::from_utf8(bytes).ok()
}

// ---------------------------------------------------------------------------
// Test seam: seal a Number field to a disk segment (disk tier)
// ---------------------------------------------------------------------------

#[cfg(test)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-storage-rs.md#source
impl Engine {
    /// TEST SEAM (Stage 2 Phase 2c): seal the current in-RAM state of a Number
    /// field into a columnar mmap segment under `dir`, then attach it so per-doc
    /// PREDICATE point lookups read the segment for the sealed id range. Mirrors
    /// what a real flush would do: dumps `forward` in dense docid order
    /// `[0..n_docs)` (absent docs → `None`) via [`crate::segment::write_number_segment`],
    /// opens a [`crate::segment::SegmentReader`], and sets `NumberIndex::segment`.
    ///
    /// `n_docs` is the interner's dense id count, so any doc indexed AFTER
    /// sealing (id >= n_docs) is NOT covered by the segment and stays served
    /// from the live `forward` tail — exactly the live/sealed split the runtime
    /// will use. Returns the sealed doc count.
    pub(crate) fn __seal_number_field_to_segment(
        &self,
        collection_id: &str,
        field: &str,
        dir: &std::path::Path,
    ) -> Result<u32> {
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get_mut(collection_id)
            .ok_or_else(|| anyhow!("unknown collection `{collection_id}`"))?;
        // Dense doc-id space is `[0..interner.to_eid.len())`.
        let n_docs = coll.interner.to_eid.len();
        let fi = coll
            .fields
            .get_mut(field)
            .ok_or_else(|| anyhow!("unknown field `{field}`"))?;
        let FieldIndex::Number(n) = fi else {
            bail!("field `{field}` is not a Number field");
        };
        // Column in dense docid order: each sealed id's live value (or None).
        let values: Vec<Option<f64>> = (0..n_docs as u32)
            .map(|id| n.live_number_at(id).map(|s| s.to_f64()))
            .collect();

        let path = dir.join(format!("{field}.lseg"));
        crate::segment::write_number_segment(&path, n_docs as u64, &values)?;
        let reader = crate::segment::SegmentReader::open(&path)?;
        debug_assert_eq!(reader.n_docs() as usize, n_docs);
        n.segment = Some(std::sync::Arc::new(reader));
        // Phase 2h-3: mirror PRODUCTION `seal_to_segment` — drop BOTH the in-RAM
        // `forward` tail AND the inverted/range `values` driver (the sorted-value
        // column + per-value postings are on disk now). Queries drive from the
        // mmap; a doc indexed AFTER sealing (id >= n_docs) re-populates the live
        // `values`/`forward` tail, which the unified accessors compose with the
        // segment base.
        n.forward = FastHashMap::default();
        n.dense_forward = Vec::new();
        n.values = BTreeMap::new();
        n.dup_values = BTreeSet::new();
        n.clear_keyword_range_cache();
        // First-time seal from the live `forward`: no prior tombstone exists, but
        // reset for symmetry with the production re-seal path (Phase 2h-3).
        n.tombstones = RoaringBitmap::new();
        Ok(n_docs as u32)
    }

    /// TEST SEAM (Stage 2 Phase 2e-A, extended 2h-1): seal a Keyword field's
    /// in-RAM state into a columnar mmap segment under `dir`, then attach it so
    /// per-doc Keyword PREDICATE lookups (`keyword_at`) AND the inverted
    /// Term/Terms/boolean driver (`term_postings`/`term_df`) serve the sealed id
    /// range from the segment. Dumps `forward` in dense docid order
    /// `[0..n_docs)` (absent docs → `None`) plus the INVERTED postings (folded
    /// from `forward`) via [`crate::segment::write_keyword_segment`] — a sorted
    /// prefix-compressed string DICT + a fixed `u32[n_docs]` dict-id forward
    /// column + a parallel per-term [`ROLE_KEYWORD_POSTINGS`] posting column.
    ///
    /// Phase 2h-1: this seam now mirrors PRODUCTION `seal_to_segment` — after
    /// attaching the reader it DROPS BOTH the in-RAM `forward` tail AND the
    /// inverted `terms` index (the RAM win). Queries then drive entirely from
    /// the mmap. A doc indexed AFTER sealing (id >= n_docs) re-populates the
    /// live `terms`/`forward` tail, which `term_postings`/`keyword_at` compose
    /// with the segment base. Returns the sealed doc count.
    pub(crate) fn __seal_keyword_field_to_segment(
        &self,
        collection_id: &str,
        field: &str,
        dir: &std::path::Path,
    ) -> Result<u32> {
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get_mut(collection_id)
            .ok_or_else(|| anyhow!("unknown collection `{collection_id}`"))?;
        let n_docs = coll.interner.to_eid.len();
        let fi = coll
            .fields
            .get_mut(field)
            .ok_or_else(|| anyhow!("unknown field `{field}`"))?;
        let FieldIndex::Keyword(k) = fi else {
            bail!("field `{field}` is not a Keyword field");
        };
        // Column in dense docid order: each sealed id's live keyword (or None).
        let owned: Vec<Option<String>> = (0..n_docs as u32).map(|id| k.keyword_at(id)).collect();
        let values: Vec<Option<&str>> = owned.iter().map(|o| o.as_deref()).collect();
        // INVERTED postings to seal: fold the live values (== the live `terms`
        // index restricted to the sealed id range) into a fresh BTreeMap.
        let mut terms: BTreeMap<String, RoaringBitmap> = BTreeMap::new();
        for (id, v) in values.iter().enumerate() {
            if let Some(s) = v {
                terms.entry((*s).to_string()).or_default().insert(id as u32);
            }
        }

        let path = dir.join(format!("{field}.lseg"));
        crate::segment::write_keyword_segment(&path, n_docs as u64, &values, &terms)?;
        let reader = crate::segment::SegmentReader::open(&path)?;
        debug_assert_eq!(reader.n_docs() as usize, n_docs);
        k.segment = Some(std::sync::Arc::new(reader));
        // Drop the RAM index — the whole [0..n_docs) inverted+forward state is
        // on disk now (Phase 2h-1). Queries drive from the mmap segment.
        k.forward = FastHashMap::default();
        k.dense_forward = Vec::new();
        k.terms = BTreeMap::new();
        k.dup_values = BTreeSet::new();
        // First-time seal from the live `forward`: no prior tombstone exists, but
        // reset for symmetry with the production re-seal path (Phase 2h-1 FIX).
        k.tombstones = RoaringBitmap::new();
        Ok(n_docs as u32)
    }

    /// TEST SEAM (Stage 2 Phase 2e-A, extended 2h-2): seal a Set field's in-RAM
    /// state into a columnar mmap segment under `dir`, then attach it so per-doc
    /// Set membership PREDICATE lookups (`set_contains` / `set_contains_any`)
    /// AND the inverted membership / Terms / boolean driver (`element_postings`
    /// / `element_df`) serve the sealed id range from the segment. Dumps
    /// `forward` in dense docid order `[0..n_docs)` (absent docs → `None`,
    /// present-empty → `Some(&[])`) plus the INVERTED postings (folded from
    /// `forward`) via [`crate::segment::write_set_segment`] — a shared sorted
    /// string DICT + a fixed `u32[n_docs + 1]` CSR offsets column + a fixed
    /// packed dict-id column + a parallel per-element [`ROLE_SET_POSTINGS`]
    /// posting column.
    ///
    /// Phase 2h-2: this seam now mirrors PRODUCTION `seal_to_segment` — after
    /// attaching the reader it DROPS BOTH the in-RAM `forward` tail AND the
    /// inverted `elements` index (the RAM win). Queries then drive entirely from
    /// the mmap. A doc indexed AFTER sealing (id >= n_docs) re-populates the live
    /// `elements`/`forward` tail, which `element_postings`/`set_contains` compose
    /// with the segment base. Returns the sealed doc count.
    pub(crate) fn __seal_set_field_to_segment(
        &self,
        collection_id: &str,
        field: &str,
        dir: &std::path::Path,
    ) -> Result<u32> {
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get_mut(collection_id)
            .ok_or_else(|| anyhow!("unknown collection `{collection_id}`"))?;
        let n_docs = coll.interner.to_eid.len();
        let fi = coll
            .fields
            .get_mut(field)
            .ok_or_else(|| anyhow!("unknown field `{field}`"))?;
        let FieldIndex::Set(s) = fi else {
            bail!("field `{field}` is not a Set field");
        };
        // Materialize each sealed doc's members as an ascending Vec<String>
        // (BTreeSet iterates sorted), or None for a doc with no set value. Owned
        // because the writer borrows the slices; keep them alive in `owned`.
        let owned: Vec<Option<Vec<String>>> = (0..n_docs as u32)
            .map(|id| s.forward.get(&id).map(|set| set.iter().cloned().collect()))
            .collect();
        let values: Vec<Option<&[String]>> = owned.iter().map(|o| o.as_deref()).collect();
        // INVERTED postings to seal: fold the live members (== the live `elements`
        // index restricted to the sealed id range) into a fresh BTreeMap.
        let mut elements: BTreeMap<String, RoaringBitmap> = BTreeMap::new();
        for (id, v) in values.iter().enumerate() {
            if let Some(members) = v {
                for m in members.iter() {
                    elements.entry(m.clone()).or_default().insert(id as u32);
                }
            }
        }

        let path = dir.join(format!("{field}.lseg"));
        crate::segment::write_set_segment(&path, n_docs as u64, &values, &elements)?;
        let reader = crate::segment::SegmentReader::open(&path)?;
        debug_assert_eq!(reader.n_docs() as usize, n_docs);
        s.segment = Some(std::sync::Arc::new(reader));
        // Drop the RAM index — the whole [0..n_docs) inverted+forward state is on
        // disk now (Phase 2h-2). Queries drive from the mmap segment.
        s.forward = FastHashMap::default();
        s.elements = BTreeMap::new();
        s.dup_values = BTreeSet::new();
        // First-time seal from the live `forward`: no prior tombstone exists, but
        // reset for symmetry with the production re-seal path (Phase 2h-2).
        s.tombstones = RoaringBitmap::new();
        Ok(n_docs as u32)
    }

    /// TEST SEAM (Stage 2 Phase 2e-B): seal a Text field's WHOLE in-RAM inverted
    /// index into a columnar mmap segment under `dir`, then attach it so the
    /// BM25 scan (`eval_match` / `match_doc_score`) and `estimate_selectivity`
    /// read entirely from the segment. Text term-frequency is NOT rebuildable,
    /// so unlike the Keyword/Set seams the inverted postings ARE stored: a sorted
    /// token DICT + a parallel per-token STORED posting block + a fixed
    /// `u32[n_docs]` DocLen column + the BM25 corpus scalars in the header (see
    /// [`crate::segment::write_text_segment`]).
    ///
    /// This slice seals the whole field for ids `[0..n_docs)` with no live tail
    /// (segment+live composition is Phase 2f). Phase 2h-4: after attaching, the
    /// bulky `tokens` postings AND `distinct` AND `lens` are DROPPED (no RAM
    /// rebuild) — `drop_eid` tombstones a sealed base id and `doc_len()` reads the
    /// segment DocLen column. Mirrors PRODUCTION `seal_to_segment`. Returns the
    /// sealed doc count.
    pub(crate) fn __seal_text_field_to_segment(
        &self,
        collection_id: &str,
        field: &str,
        dir: &std::path::Path,
    ) -> Result<u32> {
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get_mut(collection_id)
            .ok_or_else(|| anyhow!("unknown collection `{collection_id}`"))?;
        let n_docs = coll.interner.to_eid.len();
        let fi = coll
            .fields
            .get_mut(field)
            .ok_or_else(|| anyhow!("unknown field `{field}`"))?;
        let FieldIndex::Text { idx, .. } = fi else {
            bail!("field `{field}` is not a Text field");
        };
        // DocLen column in dense docid order `[0..n_docs)`: each id's stored
        // length (0 for an absent doc), reproducing `TextIndex::doc_len`.
        let lens: Vec<u32> = (0..n_docs as u32).map(|id| idx.doc_len(id)).collect();
        let tokens = idx.tokens_for_seal(&|_| true);

        let path = dir.join(format!("{field}.lseg"));
        crate::segment::write_text_segment(
            &path,
            n_docs as u64,
            &tokens,
            &lens,
            idx.doc_count,
            idx.total_doc_len,
        )?;
        let reader = crate::segment::SegmentReader::open(&path)?;
        debug_assert_eq!(reader.n_docs() as usize, n_docs);

        idx.segment = Some(std::sync::Arc::new(reader));
        // Phase 2h-4: mirror PRODUCTION `seal_to_segment` — DROP the bulky `tokens`
        // postings AND `distinct` AND `lens` to disk (no rebuild). `drop_eid`
        // tombstones a sealed base id instead of consuming `distinct`; `doc_len()`
        // reads the segment DocLen column. First-time seal: no prior tombstone, but
        // reset for symmetry with the production re-seal path.
        idx.tokens = FastHashMap::default();
        idx.distinct = Vec::new();
        idx.lens = Vec::new();
        idx.clear_match_rank_cache();
        idx.tombstones = RoaringBitmap::new();
        Ok(n_docs as u32)
    }

    /// TEST SEAM (Stage 2 Phase 2d): seal a Hash field's in-RAM state into a
    /// columnar mmap segment under `dir`, then attach it so the per-doc Hamming
    /// hash read (`hash_at`) serves the sealed id range from the segment. Dumps
    /// `forward` in dense docid order `[0..n_docs)` (absent docs → `None`) via
    /// [`crate::segment::write_hash_segment`]. Mirrors the Number seam. Returns
    /// the sealed doc count.
    pub(crate) fn __seal_hash_field_to_segment(
        &self,
        collection_id: &str,
        field: &str,
        dir: &std::path::Path,
    ) -> Result<u32> {
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get_mut(collection_id)
            .ok_or_else(|| anyhow!("unknown collection `{collection_id}`"))?;
        let n_docs = coll.interner.to_eid.len();
        let fi = coll
            .fields
            .get_mut(field)
            .ok_or_else(|| anyhow!("unknown field `{field}`"))?;
        let FieldIndex::Hash(h) = fi else {
            bail!("field `{field}` is not a Hash field");
        };
        let values: Vec<Option<u64>> = (0..n_docs as u32)
            .map(|id| h.forward.get(&id).copied())
            .collect();

        let path = dir.join(format!("{field}.lseg"));
        crate::segment::write_hash_segment(&path, n_docs as u64, &values)?;
        let reader = crate::segment::SegmentReader::open(&path)?;
        debug_assert_eq!(reader.n_docs() as usize, n_docs);
        h.segment = Some(std::sync::Arc::new(reader));
        Ok(n_docs as u32)
    }

    /// TEST SEAM (Stage 2 Phase 2d): seal a Vector field's exact-CPU
    /// (`flat-cpu`) corpus into a columnar mmap vector segment under `dir`,
    /// then attach it so the flat kNN scan reads each vector zero-copy off the
    /// page. Delegates to [`VectorIndex::__seal_flat_to_segment`]; returns the
    /// sealed vector count, or an error if the field is not a `flat-cpu` Vector
    /// (HNSW is out of scope for this slice).
    pub(crate) fn __seal_vector_field_to_segment(
        &self,
        collection_id: &str,
        field: &str,
        dir: &std::path::Path,
    ) -> Result<u32> {
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get_mut(collection_id)
            .ok_or_else(|| anyhow!("unknown collection `{collection_id}`"))?;
        let fi = coll
            .fields
            .get_mut(field)
            .ok_or_else(|| anyhow!("unknown field `{field}`"))?;
        let FieldIndex::Vector { idx, .. } = fi else {
            bail!("field `{field}` is not a Vector field");
        };
        let path = dir.join(format!("{field}.lseg"));
        idx.__seal_flat_to_segment(&path)?
            .ok_or_else(|| anyhow!("field `{field}` is not a flat-cpu vector backend"))
    }

    /// TEST HELPER (Phase 2f-1): run the PRODUCTION collection-level
    /// `seal_to_segments` on a collection in place (seal every field, write the
    /// EID column, drop the forward payload). Used by the triple-path diff test
    /// to materialize PATH B (engine after seal-and-drop).
    pub(crate) fn __seal_collection_to_segments(
        &self,
        collection_id: &str,
        dir: &std::path::Path,
        applied_seq: u64,
    ) -> Result<()> {
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get_mut(collection_id)
            .ok_or_else(|| anyhow!("unknown collection `{collection_id}`"))?;
        coll.seal_to_segments(dir, applied_seq)
    }

    /// TEST HELPER (Phase 2f-1): the collection's field-spec schema (for driving
    /// `Collection::open_from_segments`, which needs the field types out-of-band).
    pub(crate) fn __collection_schema(
        &self,
        collection_id: &str,
    ) -> Result<BTreeMap<String, FieldSpec>> {
        let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get(collection_id)
            .ok_or_else(|| anyhow!("unknown collection `{collection_id}`"))?;
        Ok(coll.schema.clone())
    }

    /// TEST HELPER (Phase 2f-1): build a fresh Engine whose single collection
    /// `collection_id` is reopened from the segments under `dir` via the
    /// PRODUCTION `Collection::open_from_segments` — NO CBOR snapshot, NO
    /// whole-collection load. Materializes PATH C of the triple-path diff test.
    pub(crate) fn __open_collection_from_segments(
        collection_id: &str,
        dir: &std::path::Path,
        schema: BTreeMap<String, FieldSpec>,
        version: u32,
    ) -> Result<std::sync::Arc<Engine>> {
        let coll = Collection::open_from_segments(dir, schema, version)?;
        let engine = Engine::new();
        {
            let mut state = engine
                .state
                .write()
                .map_err(|_| anyhow!("state poisoned"))?;
            state.collections.insert(collection_id.to_string(), coll);
        }
        Ok(std::sync::Arc::new(engine))
    }

    /// TEST HELPER (Phase 2f-1): a direct probe that a field's forward payload
    /// left RAM after a seal-and-drop — the "drop really frees RAM" assertion.
    /// Returns `(forward_len, tokens_len, has_segment)` for the named field.
    pub(crate) fn __field_forward_probe(
        &self,
        collection_id: &str,
        field: &str,
    ) -> Result<(usize, usize, bool)> {
        let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get(collection_id)
            .ok_or_else(|| anyhow!("unknown collection `{collection_id}`"))?;
        let fi = coll
            .fields
            .get(field)
            .ok_or_else(|| anyhow!("unknown field `{field}`"))?;
        Ok(match fi {
            FieldIndex::Number(n) => (n.forward_len(), 0, n.segment.is_some()),
            FieldIndex::Hash(h) => (h.forward.len(), 0, h.segment.is_some()),
            FieldIndex::Keyword(k) => (k.forward_len(), 0, k.segment.is_some()),
            FieldIndex::Set(s) => (s.forward.len(), 0, s.segment.is_some()),
            FieldIndex::Text { idx, .. } => (0, idx.tokens.len(), idx.segment.is_some()),
            FieldIndex::Vector { .. } => (0, 0, true),
        })
    }
}

// ---------------------------------------------------------------------------
// Dual-path diff test: segment-backed Number predicate read must be
// byte-identical to the live in-RAM read (Stage 2 Phase 2c).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod segment_predicate_diff_tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::Arc;

    fn fieldspec(t: FieldType, analyzer: Option<Analyzer>) -> FieldSpec {
        FieldSpec {
            field_type: t,
            analyzer,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    /// `age` (Number, the field we seal), `kw` (Keyword), `body` (Text).
    fn schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        fields.insert("age".into(), fieldspec(FieldType::Number, None));
        fields.insert("kw".into(), fieldspec(FieldType::Keyword, None));
        fields.insert(
            "body".into(),
            fieldspec(FieldType::Text, Some(Analyzer::WhitespaceLower)),
        );
        CreateCollectionRequest { fields }
    }

    fn req(query: QueryNode) -> SearchRequest {
        SearchRequest {
            query,
            limit: 100_000, // larger than any corpus → page == full match set
            cursor: None,
            sort: None,
            track_total: true,
            collapse: None,
        }
    }

    /// (external_id, score) pairs for a query, mirroring planner_diff's shape.
    fn run(e: &Engine, query: QueryNode) -> Vec<(String, f32)> {
        e.search("c", req(query))
            .unwrap()
            .hits
            .into_iter()
            .map(|h| (h.external_id, h.score))
            .collect()
    }

    fn set_of(rows: &[(String, f32)]) -> BTreeSet<String> {
        rows.iter().map(|(e, _)| e.clone()).collect()
    }

    /// Scores keyed by external_id — so we can assert score byte-equality
    /// independent of result ordering.
    fn scores_of(rows: &[(String, f32)]) -> BTreeMap<String, u32> {
        rows.iter().map(|(e, s)| (e.clone(), s.to_bits())).collect()
    }

    /// Index one doc: always writes `kw` + `body`; writes `age` only when
    /// `age` is `Some` (so absent-value docs are part of the corpus).
    fn index_doc(e: &Engine, eid: &str, age: Option<f64>, kw: &str, tok: bool) {
        let mut items = vec![
            crate::types::IndexItem {
                external_id: eid.into(),
                field: "kw".into(),
                value: FieldValue::String(kw.into()),
            },
            crate::types::IndexItem {
                external_id: eid.into(),
                field: "body".into(),
                value: FieldValue::String(if tok {
                    "tok filler".into()
                } else {
                    "filler".into()
                }),
            },
        ];
        if let Some(a) = age {
            items.push(crate::types::IndexItem {
                external_id: eid.into(),
                field: "age".into(),
                value: FieldValue::Number(a),
            });
        }
        e.index(
            "c",
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();
    }

    /// A match-DRIVEN AND so the `range age` conjunct is applied as a per-doc
    /// PREDICATE (`clause_matches` → `NumberIndex::number_at`), which is the
    /// segment-backed read site under test. `tok` is the rare driver token.
    fn bool_filter(gte: f64, lt: f64) -> QueryNode {
        QueryNode::And(vec![
            QueryNode::Match(MatchQuery {
                field: "body".into(),
                text: "tok".into(),
                op: MatchOp::And,
            }),
            QueryNode::Range(RangeQuery {
                field: "age".into(),
                gt: None,
                gte: Some(gte),
                lt: Some(lt),
                lte: None,
            }),
        ])
    }

    /// filtered_search: match-driven AND with BOTH a `term kw` and a
    /// `range age` predicate — exercises the Term-Number and Range-Number
    /// segment predicate sites together.
    fn filtered_search(kw: &str, gte: f64, lt: f64) -> QueryNode {
        QueryNode::And(vec![
            QueryNode::Match(MatchQuery {
                field: "body".into(),
                text: "tok".into(),
                op: MatchOp::And,
            }),
            QueryNode::Term(TermQuery {
                field: "kw".into(),
                value: FieldValue::String(kw.into()),
            }),
            QueryNode::Range(RangeQuery {
                field: "age".into(),
                gt: None,
                gte: Some(gte),
                lt: Some(lt),
                lte: None,
            }),
        ])
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        /// PATH A (segment OFF, live in-RAM) must equal PATH B (Number field
        /// sealed to an mmap segment, then served from it) — same result SET
        /// and byte-identical scores — for both query shapes over a randomized
        /// corpus (varied N, value distribution, absent-`age` docs).
        #[test]
        fn segment_read_matches_live_read(
            docs in proptest::collection::vec(
                (
                    // age: ~1-in-5 docs have NO age value (absent column entry).
                    proptest::option::weighted(0.8, 0u32..30),
                    prop::sample::select(vec!["a", "b", "c", "d"]),
                    any::<bool>(),
                ),
                1..60,
            ),
            lo in 0u32..30,
            span in 1u32..30,
        ) {
            let hi = lo + span;

            // --- PATH A: build the live engine, run both shapes (segment OFF). ---
            let e = Arc::new(Engine::new());
            e.create_collection("c", schema()).unwrap();
            for (i, (age, kw, tok)) in docs.iter().enumerate() {
                index_doc(&e, &format!("d{i}"), age.map(|a| a as f64), kw, *tok);
            }

            let a_bool = run(&e, bool_filter(lo as f64, hi as f64));
            let a_filt = run(&e, filtered_search("c", lo as f64, hi as f64));

            // --- PATH B: seal `age` to a segment, flip it ON, rerun. ---
            let dir = tempfile::tempdir().unwrap();
            let sealed = e.__seal_number_field_to_segment("c", "age", dir.path()).unwrap();
            prop_assert_eq!(sealed as usize, docs.len(), "all docs sealed");

            let b_bool = run(&e, bool_filter(lo as f64, hi as f64));
            let b_filt = run(&e, filtered_search("c", lo as f64, hi as f64));

            // Result SET equality.
            prop_assert_eq!(set_of(&a_bool), set_of(&b_bool), "bool_filter set diverged");
            prop_assert_eq!(set_of(&a_filt), set_of(&b_filt), "filtered_search set diverged");
            // Scores byte-identical (f64::to_bits keyed by eid).
            prop_assert_eq!(scores_of(&a_bool), scores_of(&b_bool), "bool_filter scores diverged");
            prop_assert_eq!(scores_of(&a_filt), scores_of(&b_filt), "filtered_search scores diverged");
        }
    }

    /// A doc indexed AFTER sealing (docid >= segment n_docs) lives in the live
    /// `forward` tail and must still match through `number_at`'s fallback.
    #[test]
    fn doc_indexed_after_sealing_served_from_live_tail() {
        let e = Arc::new(Engine::new());
        e.create_collection("c", schema()).unwrap();
        // Two docs sealed into the segment.
        index_doc(&e, "sealed_in", Some(10.0), "c", true);
        index_doc(&e, "sealed_out", Some(99.0), "c", true);

        let dir = tempfile::tempdir().unwrap();
        let n = e
            .__seal_number_field_to_segment("c", "age", dir.path())
            .unwrap();
        assert_eq!(n, 2, "two docs sealed");

        // A NEW doc after sealing → docid 2 (>= n_docs) → lives in the live tail.
        index_doc(&e, "tail", Some(15.0), "c", true);

        // Range [12,20): only the tail doc qualifies. It is NOT in the segment,
        // so this proves number_at's `id >= n_docs` fallback to `forward`.
        let got = set_of(&run(&e, bool_filter(12.0, 20.0)));
        let want: BTreeSet<String> = ["tail".to_string()].into_iter().collect();
        assert_eq!(got, want, "tail doc must match via live fallback");

        // Range [8,12): only the sealed doc qualifies — served from the segment.
        let got = set_of(&run(&e, bool_filter(8.0, 12.0)));
        let want: BTreeSet<String> = ["sealed_in".to_string()].into_iter().collect();
        assert_eq!(got, want, "sealed doc must match via segment read");
    }

    /// Direct check that `NumberIndex::number_at` reads from the segment for
    /// sealed ids and falls back to the live tail past `n_docs` — independent
    /// of the query planner.
    #[test]
    fn number_at_segment_then_live_split() {
        let e = Arc::new(Engine::new());
        e.create_collection("c", schema()).unwrap();
        index_doc(&e, "a", Some(1.5), "x", false); // id 0
        index_doc(&e, "b", None, "x", false); // id 1, absent age
        index_doc(&e, "c", Some(-3.0), "x", false); // id 2

        let dir = tempfile::tempdir().unwrap();
        e.__seal_number_field_to_segment("c", "age", dir.path())
            .unwrap();
        index_doc(&e, "d", Some(7.0), "x", false); // id 3, live tail

        let state = e.state.read().unwrap();
        let coll = state.collections.get("c").unwrap();
        let FieldIndex::Number(n) = coll.fields.get("age").unwrap() else {
            panic!("age must be a Number field");
        };
        assert!(n.segment.is_some(), "segment attached");
        assert_eq!(n.number_at(0), Some(SortableF64::new(1.5).unwrap())); // segment
        assert_eq!(n.number_at(1), None); // segment, absent
        assert_eq!(n.number_at(2), Some(SortableF64::new(-3.0).unwrap())); // segment
        assert_eq!(n.number_at(3), Some(SortableF64::new(7.0).unwrap())); // live tail
        assert_eq!(n.number_at(99), None); // unknown
    }
}

// ---------------------------------------------------------------------------
// Dual-path diff test: segment-backed Keyword predicate read must be
// byte-identical to the live in-RAM read (Stage 2 Phase 2e-A).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod segment_keyword_diff_tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::Arc;

    fn fieldspec(t: FieldType, analyzer: Option<Analyzer>) -> FieldSpec {
        FieldSpec {
            field_type: t,
            analyzer,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    /// `kw` (Keyword, the field we seal) + `body` (Text, the AND driver).
    fn schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        fields.insert("kw".into(), fieldspec(FieldType::Keyword, None));
        fields.insert(
            "body".into(),
            fieldspec(FieldType::Text, Some(Analyzer::WhitespaceLower)),
        );
        CreateCollectionRequest { fields }
    }

    fn req(query: QueryNode) -> SearchRequest {
        SearchRequest {
            query,
            limit: 100_000,
            cursor: None,
            sort: None,
            track_total: true,
            collapse: None,
        }
    }

    fn run(e: &Engine, query: QueryNode) -> Vec<(String, f32)> {
        e.search("c", req(query))
            .unwrap()
            .hits
            .into_iter()
            .map(|h| (h.external_id, h.score))
            .collect()
    }

    fn set_of(rows: &[(String, f32)]) -> BTreeSet<String> {
        rows.iter().map(|(e, _)| e.clone()).collect()
    }

    fn scores_of(rows: &[(String, f32)]) -> BTreeMap<String, u32> {
        rows.iter().map(|(e, s)| (e.clone(), s.to_bits())).collect()
    }

    /// Index one doc: always writes `body`; writes `kw` only when `kw` is
    /// `Some` (so absent-keyword docs are part of the corpus).
    fn index_doc(e: &Engine, eid: &str, kw: Option<&str>, tok: bool) {
        let mut items = vec![crate::types::IndexItem {
            external_id: eid.into(),
            field: "body".into(),
            value: FieldValue::String(if tok {
                "tok filler".into()
            } else {
                "filler".into()
            }),
        }];
        if let Some(k) = kw {
            items.push(crate::types::IndexItem {
                external_id: eid.into(),
                field: "kw".into(),
                value: FieldValue::String(k.into()),
            });
        }
        e.index(
            "c",
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();
    }

    /// A match-DRIVEN AND so the `term kw` conjunct is applied as a per-doc
    /// PREDICATE (`clause_matches` → `KeywordIndex::keyword_at`) — the
    /// segment-backed read site under test. `tok` is the rare driver token.
    fn term_conjunct(kw: &str) -> QueryNode {
        QueryNode::And(vec![
            QueryNode::Match(MatchQuery {
                field: "body".into(),
                text: "tok".into(),
                op: MatchOp::And,
            }),
            QueryNode::Term(TermQuery {
                field: "kw".into(),
                value: FieldValue::String(kw.into()),
            }),
        ])
    }

    /// A match-DRIVEN AND with a `terms kw` (multi-value OR-of-terms) conjunct
    /// — exercises the Terms-Keyword segment predicate site (`keyword_at` in
    /// the `Terms` arm of `clause_matches`).
    fn terms_conjunct(kws: &[&str]) -> QueryNode {
        QueryNode::And(vec![
            QueryNode::Match(MatchQuery {
                field: "body".into(),
                text: "tok".into(),
                op: MatchOp::And,
            }),
            QueryNode::Terms(TermsQuery {
                field: "kw".into(),
                values: kws
                    .iter()
                    .map(|s| FieldValue::String((*s).into()))
                    .collect(),
            }),
        ])
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        /// PATH A (segment OFF, live in-RAM `forward`) must equal PATH B
        /// (Keyword field sealed to a var-width DICT + dict-id segment, then
        /// served from it) — same result SET and byte-identical scores — for
        /// both the `term kw` conjunct and the `terms kw` conjunct over a
        /// randomized corpus (varied N, value distribution, absent-`kw` docs).
        #[test]
        fn segment_read_matches_live_read(
            docs in proptest::collection::vec(
                (
                    // kw: ~1-in-5 docs have NO keyword (absent column entry).
                    proptest::option::weighted(
                        0.8,
                        prop::sample::select(vec!["alpha", "beta", "gamma", "delta"]),
                    ),
                    any::<bool>(),
                ),
                1..60,
            ),
        ) {
            // --- PATH A: build the live engine, run both shapes (segment OFF). ---
            let e = Arc::new(Engine::new());
            e.create_collection("c", schema()).unwrap();
            for (i, (kw, tok)) in docs.iter().enumerate() {
                index_doc(&e, &format!("d{i}"), *kw, *tok);
            }

            let a_term = run(&e, term_conjunct("alpha"));
            let a_terms = run(&e, terms_conjunct(&["beta", "gamma"]));

            // --- PATH B: seal `kw` to a segment, flip it ON, rerun. ---
            let dir = tempfile::tempdir().unwrap();
            let sealed = e.__seal_keyword_field_to_segment("c", "kw", dir.path()).unwrap();
            prop_assert_eq!(sealed as usize, docs.len(), "all docs sealed");

            let b_term = run(&e, term_conjunct("alpha"));
            let b_terms = run(&e, terms_conjunct(&["beta", "gamma"]));

            prop_assert_eq!(set_of(&a_term), set_of(&b_term), "term conjunct set diverged");
            prop_assert_eq!(set_of(&a_terms), set_of(&b_terms), "terms conjunct set diverged");
            prop_assert_eq!(scores_of(&a_term), scores_of(&b_term), "term scores diverged");
            prop_assert_eq!(scores_of(&a_terms), scores_of(&b_terms), "terms scores diverged");
        }
    }

    /// A doc indexed AFTER sealing (docid >= segment n_docs) lives in the live
    /// `forward` tail and must still match through `keyword_at`'s fallback.
    #[test]
    fn doc_indexed_after_sealing_served_from_live_tail() {
        let e = Arc::new(Engine::new());
        e.create_collection("c", schema()).unwrap();
        index_doc(&e, "sealed", Some("alpha"), true);
        index_doc(&e, "absent", None, true); // present-but-no-kw, id 1

        let dir = tempfile::tempdir().unwrap();
        let n = e
            .__seal_keyword_field_to_segment("c", "kw", dir.path())
            .unwrap();
        assert_eq!(n, 2, "two docs sealed");

        // A NEW doc after sealing → docid 2 (>= n_docs) → lives in the live tail.
        index_doc(&e, "tail", Some("beta"), true);

        // term beta: only the tail doc qualifies, NOT in the segment → proves
        // keyword_at's id >= n_docs fallback to forward.
        let got = set_of(&run(&e, term_conjunct("beta")));
        let want: BTreeSet<String> = ["tail".to_string()].into_iter().collect();
        assert_eq!(got, want, "tail doc must match via live fallback");

        // term alpha: only the sealed doc qualifies → served from the segment.
        let got = set_of(&run(&e, term_conjunct("alpha")));
        let want: BTreeSet<String> = ["sealed".to_string()].into_iter().collect();
        assert_eq!(got, want, "sealed doc must match via segment read");
    }

    /// Direct planner-free check that `KeywordIndex::keyword_at` reads from the
    /// segment for sealed ids (incl an absent doc) and falls back to the live
    /// tail past `n_docs`.
    #[test]
    fn keyword_at_segment_then_live_split() {
        let e = Arc::new(Engine::new());
        e.create_collection("c", schema()).unwrap();
        index_doc(&e, "a", Some("alpha"), false); // id 0
        index_doc(&e, "b", None, false); // id 1, absent kw
        index_doc(&e, "c", Some("gamma"), false); // id 2

        let dir = tempfile::tempdir().unwrap();
        e.__seal_keyword_field_to_segment("c", "kw", dir.path())
            .unwrap();
        index_doc(&e, "d", Some("delta"), false); // id 3, live tail

        let state = e.state.read().unwrap();
        let coll = state.collections.get("c").unwrap();
        let FieldIndex::Keyword(k) = coll.fields.get("kw").unwrap() else {
            panic!("kw must be a Keyword field");
        };
        assert!(k.segment.is_some(), "segment attached");
        assert_eq!(k.keyword_at(0).as_deref(), Some("alpha")); // segment
        assert_eq!(k.keyword_at(1), None); // segment, absent
        assert_eq!(k.keyword_at(2).as_deref(), Some("gamma")); // segment
        assert_eq!(k.keyword_at(3).as_deref(), Some("delta")); // live tail
        assert_eq!(k.keyword_at(99), None); // unknown
    }
}

// ---------------------------------------------------------------------------
// Dual-path diff test for the INVERTED Keyword driver (Stage 2 Phase 2h-1):
// the segment-driven Term/Terms/boolean RoaringBitmap algebra (`term_postings`
// / `term_df`) must be byte-identical to the in-RAM `terms` index, AND after a
// seal the RAM index is DROPPED (the disk=all win) while queries keep serving
// entirely from the mmap segment. This is the keystone test for Phase 2h-1.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod segment_keyword_inverted_diff_tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::Arc;

    fn fieldspec(t: FieldType, analyzer: Option<Analyzer>) -> FieldSpec {
        FieldSpec {
            field_type: t,
            analyzer,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    /// `kw` (Keyword, sealed) + `cat` (a second Keyword, for boolean AND/OR
    /// cross-field algebra). Both are inverted-driver fields.
    fn schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        fields.insert("kw".into(), fieldspec(FieldType::Keyword, None));
        fields.insert("cat".into(), fieldspec(FieldType::Keyword, None));
        CreateCollectionRequest { fields }
    }

    fn req(query: QueryNode) -> SearchRequest {
        SearchRequest {
            query,
            limit: 100_000,
            cursor: None,
            sort: None,
            track_total: true,
            collapse: None,
        }
    }

    fn run(e: &Engine, query: QueryNode) -> Vec<(String, f32)> {
        e.search("c", req(query))
            .unwrap()
            .hits
            .into_iter()
            .map(|h| (h.external_id, h.score))
            .collect()
    }

    fn set_of(rows: &[(String, f32)]) -> BTreeSet<String> {
        rows.iter().map(|(e, _)| e.clone()).collect()
    }

    /// (external_id, total) result for a query — drives the `try_plan`
    /// standalone-Term page+total path too.
    fn search_total(e: &Engine, query: QueryNode) -> u64 {
        e.search("c", req(query)).unwrap().total
    }

    fn index_kw(e: &Engine, eid: &str, kw: Option<&str>, cat: Option<&str>) {
        let mut items = Vec::new();
        if let Some(k) = kw {
            items.push(crate::types::IndexItem {
                external_id: eid.into(),
                field: "kw".into(),
                value: FieldValue::String(k.into()),
            });
        }
        if let Some(c) = cat {
            items.push(crate::types::IndexItem {
                external_id: eid.into(),
                field: "cat".into(),
                value: FieldValue::String(c.into()),
            });
        }
        // A doc with neither field would have no postings anywhere; ensure at
        // least the kw field so the doc is interned.
        if items.is_empty() {
            items.push(crate::types::IndexItem {
                external_id: eid.into(),
                field: "kw".into(),
                value: FieldValue::String("zzz_filler".into()),
            });
        }
        e.index(
            "c",
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();
    }

    fn term(field: &str, v: &str) -> QueryNode {
        QueryNode::Term(TermQuery {
            field: field.into(),
            value: FieldValue::String(v.into()),
        })
    }

    fn terms(field: &str, vs: &[&str]) -> QueryNode {
        QueryNode::Terms(TermsQuery {
            field: field.into(),
            values: vs.iter().map(|s| FieldValue::String((*s).into())).collect(),
        })
    }

    /// Assert the Keyword field `kw` has an EMPTY in-RAM `terms` index but an
    /// attached segment — the RAM-bounded invariant after a seal/reopen.
    fn assert_terms_dropped(e: &Engine) {
        let state = e.state.read().unwrap();
        let coll = state.collections.get("c").unwrap();
        let FieldIndex::Keyword(k) = coll.fields.get("kw").unwrap() else {
            panic!("kw must be a Keyword field");
        };
        assert!(k.segment.is_some(), "segment must be attached");
        assert!(
            k.terms.is_empty(),
            "RAM `terms` index must be DROPPED after seal (got {} entries)",
            k.terms.len()
        );
        assert!(
            k.forward.is_empty(),
            "RAM `forward` must be dropped after seal"
        );
        assert!(
            k.dense_forward.is_empty(),
            "RAM dense `forward` must be dropped after seal"
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        /// PATH A (segment OFF, in-RAM `terms`) must equal PATH B (sealed: RAM
        /// `terms` DROPPED, driven from the mmap posting column) for the whole
        /// inverted-driver surface: standalone `Term`, standalone `Terms`,
        /// boolean `Or` of two terms, boolean `And` across two keyword fields,
        /// and the `try_plan` standalone-Term total. Byte-identical result sets
        /// AND identical totals on both paths.
        #[test]
        fn inverted_segment_matches_live(
            docs in proptest::collection::vec(
                (
                    proptest::option::weighted(
                        0.85,
                        prop::sample::select(vec!["alpha", "beta", "gamma", "delta"]),
                    ),
                    proptest::option::weighted(
                        0.7,
                        prop::sample::select(vec!["red", "green"]),
                    ),
                ),
                1..70,
            ),
        ) {
            // --- PATH A: in-RAM inverted index (segment OFF). ---
            let e = Arc::new(Engine::new());
            e.create_collection("c", schema()).unwrap();
            for (i, (kw, cat)) in docs.iter().enumerate() {
                index_kw(&e, &format!("d{i}"), *kw, *cat);
            }

            let a_term = run(&e, term("kw", "alpha"));
            let a_terms = run(&e, terms("kw", &["beta", "gamma"]));
            let a_or = run(&e, QueryNode::Or(vec![term("kw", "alpha"), term("kw", "delta")]));
            let a_and = run(&e, QueryNode::And(vec![term("kw", "beta"), term("cat", "red")]));
            let a_total = search_total(&e, term("kw", "gamma"));

            // --- PATH B: seal `kw` (drops RAM `terms`), rerun from the mmap. ---
            let dir = tempfile::tempdir().unwrap();
            let sealed = e.__seal_keyword_field_to_segment("c", "kw", dir.path()).unwrap();
            prop_assert_eq!(sealed as usize, docs.len(), "all docs sealed");
            assert_terms_dropped(&e); // RAM-bounded: terms index gone

            let b_term = run(&e, term("kw", "alpha"));
            let b_terms = run(&e, terms("kw", &["beta", "gamma"]));
            let b_or = run(&e, QueryNode::Or(vec![term("kw", "alpha"), term("kw", "delta")]));
            let b_and = run(&e, QueryNode::And(vec![term("kw", "beta"), term("cat", "red")]));
            let b_total = search_total(&e, term("kw", "gamma"));

            prop_assert_eq!(set_of(&a_term), set_of(&b_term), "Term set diverged");
            prop_assert_eq!(set_of(&a_terms), set_of(&b_terms), "Terms set diverged");
            prop_assert_eq!(set_of(&a_or), set_of(&b_or), "Or set diverged");
            prop_assert_eq!(set_of(&a_and), set_of(&b_and), "And set diverged");
            prop_assert_eq!(a_total, b_total, "standalone-term total diverged");
        }
    }

    /// RAM-BOUNDED + REOPEN-NO-REBUILD: seal the WHOLE collection to disk, reopen
    /// from the segments alone (no CBOR snapshot), and assert the reopened
    /// Keyword field's `terms` map is EMPTY while Term/Terms queries still return
    /// correct results entirely from the mmap segment.
    #[test]
    fn reopen_drives_from_segment_with_empty_terms() {
        let dir = tempfile::tempdir().unwrap();
        // Build the collection and capture the wanted answers (segment OFF).
        let e = Arc::new(Engine::new());
        e.create_collection("c", schema()).unwrap();
        index_kw(&e, "a", Some("alpha"), Some("red"));
        index_kw(&e, "b", Some("beta"), Some("green"));
        index_kw(&e, "c2", Some("gamma"), None);
        index_kw(&e, "d", Some("alpha"), Some("red"));
        index_kw(&e, "e", None, Some("green")); // present-but-no-kw

        let want_alpha = set_of(&run(&e, term("kw", "alpha")));
        let want_beta_gamma = set_of(&run(&e, terms("kw", &["beta", "gamma"])));

        // PRODUCTION whole-collection seal (drops every field's RAM driver), then
        // reopen from the segments alone (no CBOR snapshot, no whole-collection
        // load) via the production `open_from_segments`.
        e.__seal_collection_to_segments("c", dir.path(), 1).unwrap();
        let schema = e.__collection_schema("c").unwrap();
        let e2 = Engine::__open_collection_from_segments("c", dir.path(), schema, 1).unwrap();

        // The reopened Keyword `terms` map must be EMPTY (no RAM rebuild) ...
        {
            let state = e2.state.read().unwrap();
            let coll = state.collections.get("c").unwrap();
            let FieldIndex::Keyword(k) = coll.fields.get("kw").unwrap() else {
                panic!("kw must be a Keyword field");
            };
            assert!(k.segment.is_some(), "reopened segment attached");
            assert!(
                k.terms.is_empty(),
                "reopen must NOT rebuild `terms` in RAM (got {} entries)",
                k.terms.len()
            );
        }

        // ... yet the inverted queries still resolve, entirely from the mmap.
        assert_eq!(
            set_of(&run(&e2, term("kw", "alpha"))),
            want_alpha,
            "Term post-reopen"
        );
        assert_eq!(
            set_of(&run(&e2, terms("kw", &["beta", "gamma"]))),
            want_beta_gamma,
            "Terms post-reopen"
        );
    }

    /// LIVE-TAIL UNION: seal, then index more docs (tail into the live `terms`),
    /// and assert a Term query returns base (segment) + tail (RAM) composed.
    #[test]
    fn live_tail_unions_with_segment_base() {
        let e = Arc::new(Engine::new());
        e.create_collection("c", schema()).unwrap();
        index_kw(&e, "base0", Some("alpha"), None); // id 0, sealed base
        index_kw(&e, "base1", Some("beta"), None); // id 1, sealed base

        let dir = tempfile::tempdir().unwrap();
        let n = e
            .__seal_keyword_field_to_segment("c", "kw", dir.path())
            .unwrap();
        assert_eq!(n, 2);
        assert_terms_dropped(&e); // base index now on disk

        // Tail docs after seal → land in the live `terms` tail (ids >= n_docs).
        index_kw(&e, "tail0", Some("alpha"), None); // id 2, tail, SAME term as base0
        index_kw(&e, "tail1", Some("gamma"), None); // id 3, tail, NEW term

        // term alpha: base id (base0) UNION tail id (tail0).
        let got = set_of(&run(&e, term("kw", "alpha")));
        let want: BTreeSet<String> = ["base0".into(), "tail0".into()].into_iter().collect();
        assert_eq!(got, want, "alpha must union segment base + live tail");

        // term gamma: ONLY the tail (not in the segment).
        let got = set_of(&run(&e, term("kw", "gamma")));
        let want: BTreeSet<String> = ["tail1".into()].into_iter().collect();
        assert_eq!(got, want, "gamma must come from the live tail alone");

        // term beta: ONLY the segment base.
        let got = set_of(&run(&e, term("kw", "beta")));
        let want: BTreeSet<String> = ["base1".into()].into_iter().collect();
        assert_eq!(got, want, "beta must come from the segment base alone");

        // df composition: term_df(alpha) = 1 (base) + 1 (tail) = 2.
        let state = e.state.read().unwrap();
        let coll = state.collections.get("c").unwrap();
        let FieldIndex::Keyword(k) = coll.fields.get("kw").unwrap() else {
            panic!("kw must be a Keyword field");
        };
        assert_eq!(
            k.term_df("alpha"),
            2,
            "df must sum segment base + live tail"
        );
        assert_eq!(k.term_df("beta"), 1, "df beta segment-only");
        assert_eq!(k.term_df("gamma"), 1, "df gamma tail-only");
        assert_eq!(k.term_df("missing"), 0, "df absent term");
    }

    // -----------------------------------------------------------------------
    // Phase 2h-1 FIX: delete-after-seal query-time tombstone. After 2h-1 dropped
    // the in-RAM `terms` index at seal, `drop_eid` on a SEALED base docid was a
    // NO-OP (the on-disk posting is immutable), so segment-driven Keyword queries
    // LEAKED the deleted doc until the next re-seal. The fix records sealed-base
    // deletes in a per-field `tombstones` RoaringBitmap that every segment-ON
    // accessor subtracts. These tests pin the fix and prove its teeth.
    // -----------------------------------------------------------------------

    /// (value → external_ids) of every duplicate group, as a comparable map.
    fn dup_map(e: &Engine, field: &str) -> BTreeMap<String, BTreeSet<String>> {
        e.duplicates(
            "c",
            crate::types::DuplicatesRequest {
                field: field.into(),
                min_group_size: 2,
                limit: 100_000,
                offset: 0,
            },
        )
        .unwrap()
        .groups
        .into_iter()
        .map(|g| {
            let v = g.value.as_str().unwrap().to_string();
            (v, g.external_ids.into_iter().collect::<BTreeSet<String>>())
        })
        .collect()
    }

    /// `unique_terms` of `field` via the public stats surface.
    fn uniq(e: &Engine, field: &str) -> u64 {
        e.stats("c")
            .unwrap()
            .fields
            .get(field)
            .unwrap()
            .unique_terms
    }

    /// DELETE-AFTER-SEAL **WITHOUT** RE-SEAL: seal a Keyword corpus, delete
    /// several BASE docs (no re-seal), then assert every segment-ON query equals
    /// an in-RAM ORACLE built from the identical op sequence but NEVER sealed —
    /// byte-identical result SETS for standalone Term, multi-term Terms, boolean
    /// Or, cross-field And, plus identical `duplicates`/`unique_terms`. This is
    /// the window 2g-A's seal-time GC does NOT cover; the tombstone closes it.
    #[test]
    fn delete_after_seal_without_reseal_matches_oracle() {
        // Shared corpus builder so the oracle and the sealed engine see the
        // SAME index sequence. d0..d5 on `kw`/`cat`; alpha appears 3x, beta 2x,
        // red 3x → real duplicate groups before any delete.
        fn build(e: &Engine) {
            e.create_collection("c", schema()).unwrap();
            index_kw(e, "d0", Some("alpha"), Some("red"));
            index_kw(e, "d1", Some("alpha"), Some("red"));
            index_kw(e, "d2", Some("beta"), Some("green"));
            index_kw(e, "d3", Some("alpha"), Some("red"));
            index_kw(e, "d4", Some("beta"), Some("green"));
            index_kw(e, "d5", Some("gamma"), None);
        }
        // The deletes to apply on BOTH paths (whole-doc deletes).
        let to_delete = ["d1", "d3", "d4"];

        // --- ORACLE: in-RAM, never sealed. Same build + same deletes. ---
        let oracle = Arc::new(Engine::new());
        build(&oracle);
        for d in to_delete {
            oracle.delete("c", d, None).unwrap();
        }

        // --- SUBJECT: seal kw + cat to segments, THEN delete (no re-seal). ---
        let subject = Arc::new(Engine::new());
        build(&subject);
        let dir = tempfile::tempdir().unwrap();
        subject
            .__seal_keyword_field_to_segment("c", "kw", dir.path())
            .unwrap();
        subject
            .__seal_keyword_field_to_segment("c", "cat", dir.path())
            .unwrap();
        assert_terms_dropped(&subject); // RAM `terms` gone — drives from mmap
                                        // Delete AFTER the seal, with NO re-seal → exercises the tombstone path.
        for d in to_delete {
            subject.delete("c", d, None).unwrap();
        }

        // The deleted ids are now tombstoned (base ids < n_docs), NOT removed
        // from any on-disk posting. Confirm the bitmap actually recorded them.
        {
            let state = subject.state.read().unwrap();
            let coll = state.collections.get("c").unwrap();
            let FieldIndex::Keyword(k) = coll.fields.get("kw").unwrap() else {
                panic!("kw");
            };
            assert!(k.terms.is_empty(), "sealed: terms still dropped");
            assert_eq!(k.tombstones.len(), 3, "three base deletes tombstoned");
        }

        // Result SETS must match the oracle on every driver surface.
        let q_term = term("kw", "alpha");
        let q_terms = terms("kw", &["alpha", "beta"]);
        let q_or = QueryNode::Or(vec![term("kw", "alpha"), term("kw", "beta")]);
        let q_and = QueryNode::And(vec![term("kw", "alpha"), term("cat", "red")]);

        assert_eq!(
            set_of(&run(&subject, q_term.clone())),
            set_of(&run(&oracle, q_term)),
            "standalone Term leaked a deleted doc"
        );
        assert_eq!(
            set_of(&run(&subject, q_terms.clone())),
            set_of(&run(&oracle, q_terms)),
            "multi-term Terms leaked a deleted doc"
        );
        assert_eq!(
            set_of(&run(&subject, q_or.clone())),
            set_of(&run(&oracle, q_or)),
            "boolean Or leaked a deleted doc"
        );
        assert_eq!(
            set_of(&run(&subject, q_and.clone())),
            set_of(&run(&oracle, q_and)),
            "cross-field And leaked a deleted doc"
        );

        // duplicates + unique_terms must match the oracle on sealed data.
        // After deleting d1,d3,d4: alpha={d0}, beta={d2}, gamma={d5} on kw →
        // NO kw duplicate group survives; on cat red={d0}, green={d2} → none.
        assert_eq!(
            dup_map(&subject, "kw"),
            dup_map(&oracle, "kw"),
            "find_duplicates(kw) diverged on sealed-after-delete"
        );
        assert_eq!(
            dup_map(&subject, "cat"),
            dup_map(&oracle, "cat"),
            "find_duplicates(cat) diverged on sealed-after-delete"
        );
        assert_eq!(
            uniq(&subject, "kw"),
            uniq(&oracle, "kw"),
            "unique_terms(kw) diverged on sealed-after-delete"
        );
        assert_eq!(
            uniq(&subject, "cat"),
            uniq(&oracle, "cat"),
            "unique_terms(cat) diverged on sealed-after-delete"
        );

        // Concrete spot-checks (not just oracle-equality): the deleted docs are
        // GONE and a fully-deleted term yields None (empty result), not a leak.
        let alpha = set_of(&run(&subject, term("kw", "alpha")));
        assert_eq!(
            alpha,
            ["d0".to_string()].into_iter().collect::<BTreeSet<_>>(),
            "alpha must be only the surviving d0"
        );
        assert!(
            run(&subject, term("kw", "beta"))
                .iter()
                .all(|(eid, _)| eid != "d4"),
            "deleted d4 must not appear under beta"
        );
    }

    /// RE-SEAL (CHECKPOINT) after delete: once the field is re-sealed the
    /// deletions are BAKED into the new segment (via 2g-A's live(id) GC), the
    /// tombstone is CLEARED, and queries still match the oracle — composing the
    /// query-time tombstone with the seal-time GC.
    #[test]
    fn reseal_bakes_deletes_and_clears_tombstone() {
        fn build(e: &Engine) {
            e.create_collection("c", schema()).unwrap();
            index_kw(e, "d0", Some("alpha"), Some("red"));
            index_kw(e, "d1", Some("alpha"), Some("red"));
            index_kw(e, "d2", Some("beta"), Some("green"));
            index_kw(e, "d3", Some("gamma"), None);
        }
        let to_delete = ["d1", "d2"];

        let oracle = Arc::new(Engine::new());
        build(&oracle);
        for d in to_delete {
            oracle.delete("c", d, None).unwrap();
        }

        let subject = Arc::new(Engine::new());
        build(&subject);
        let dir = tempfile::tempdir().unwrap();
        // First seal (the WHOLE collection so re-seal has a working live(id)/
        // eid_fields GC), then delete, then RE-SEAL (checkpoint).
        subject
            .__seal_collection_to_segments("c", dir.path(), 1)
            .unwrap();
        for d in to_delete {
            subject.delete("c", d, None).unwrap();
        }
        // Tombstone holds the two base deletes pre-re-seal.
        {
            let state = subject.state.read().unwrap();
            let coll = state.collections.get("c").unwrap();
            let FieldIndex::Keyword(k) = coll.fields.get("kw").unwrap() else {
                panic!("kw");
            };
            assert_eq!(k.tombstones.len(), 2, "deletes tombstoned before re-seal");
        }
        // RE-SEAL: the live(id) gather (2g-A) excludes the tombstoned ids, so the
        // NEW segment has them absent; the tombstone is reset to empty.
        let dir2 = tempfile::tempdir().unwrap();
        subject
            .__seal_collection_to_segments("c", dir2.path(), 2)
            .unwrap();
        {
            let state = subject.state.read().unwrap();
            let coll = state.collections.get("c").unwrap();
            let FieldIndex::Keyword(k) = coll.fields.get("kw").unwrap() else {
                panic!("kw");
            };
            assert!(
                k.tombstones.is_empty(),
                "tombstone must be CLEARED after re-seal (deletes baked in)"
            );
        }

        // Post-re-seal queries match the oracle, now with an EMPTY tombstone (the
        // new segment itself has the deleted docs absent).
        let q_term = term("kw", "alpha");
        assert_eq!(
            set_of(&run(&subject, q_term.clone())),
            set_of(&run(&oracle, q_term)),
            "post-re-seal Term diverged from oracle"
        );
        assert_eq!(
            set_of(&run(&subject, term("kw", "beta"))),
            BTreeSet::new(),
            "beta fully deleted — must be empty after re-seal"
        );
        assert_eq!(
            uniq(&subject, "kw"),
            uniq(&oracle, "kw"),
            "unique_terms diverged after re-seal"
        );
    }
}

// ---------------------------------------------------------------------------
// Dual-path diff test: the segment-backed Number RANGE index (Phase 2h-3, WS2
// BKD) must answer range / exact / boolean / sort queries byte-identically to
// the in-RAM `values: BTreeMap<SortableF64, RoaringBitmap>` range walk. The crux
// is the on-disk SORTED-VALUE column (`ROLE_NUMBER_SORTED`) binary-searched by
// `number_range` honoring every inclusive/exclusive bound + open-endedness case,
// with the 2h tombstone reused for delete-after-seal.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod segment_number_range_diff_tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::Arc;

    fn fieldspec(t: FieldType) -> FieldSpec {
        FieldSpec {
            field_type: t,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    /// `price` (Number, the field we seal) + `cat` (Keyword, for boolean
    /// cross-field And/Or algebra).
    fn schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        fields.insert("price".into(), fieldspec(FieldType::Number));
        fields.insert("cat".into(), fieldspec(FieldType::Keyword));
        CreateCollectionRequest { fields }
    }

    fn req(query: QueryNode) -> SearchRequest {
        SearchRequest {
            query,
            limit: 100_000,
            cursor: None,
            sort: None,
            track_total: true,
            collapse: None,
        }
    }

    fn req_sort(query: QueryNode, field: &str, order: SortOrder) -> SearchRequest {
        SearchRequest {
            query,
            limit: 100_000,
            cursor: None,
            sort: Some(vec![crate::types::SortSpec {
                field: field.into(),
                order,
            }]),
            track_total: true,
            collapse: None,
        }
    }

    fn run(e: &Engine, query: QueryNode) -> Vec<(String, f32)> {
        e.search("c", req(query))
            .unwrap()
            .hits
            .into_iter()
            .map(|h| (h.external_id, h.score))
            .collect()
    }

    /// Sort-driven page (drives `try_plan`'s number-sort path over the
    /// sorted-value column). Returns the ORDERED external_ids.
    fn run_sorted(e: &Engine, query: QueryNode, field: &str, order: SortOrder) -> Vec<String> {
        e.search("c", req_sort(query, field, order))
            .unwrap()
            .hits
            .into_iter()
            .map(|h| h.external_id)
            .collect()
    }

    fn set_of(rows: &[(String, f32)]) -> BTreeSet<String> {
        rows.iter().map(|(e, _)| e.clone()).collect()
    }

    fn search_total(e: &Engine, query: QueryNode) -> u64 {
        e.search("c", req(query)).unwrap().total
    }

    fn index_num(e: &Engine, eid: &str, price: Option<f64>, cat: Option<&str>) {
        let mut items = Vec::new();
        if let Some(p) = price {
            items.push(crate::types::IndexItem {
                external_id: eid.into(),
                field: "price".into(),
                value: FieldValue::Number(p),
            });
        }
        if let Some(c) = cat {
            items.push(crate::types::IndexItem {
                external_id: eid.into(),
                field: "cat".into(),
                value: FieldValue::String(c.into()),
            });
        }
        // Ensure the doc is interned even when both fields are absent.
        if items.is_empty() {
            items.push(crate::types::IndexItem {
                external_id: eid.into(),
                field: "cat".into(),
                value: FieldValue::String("zzz_filler".into()),
            });
        }
        e.index(
            "c",
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();
    }

    fn rangeq(gte: Option<f64>, gt: Option<f64>, lte: Option<f64>, lt: Option<f64>) -> QueryNode {
        QueryNode::Range(RangeQuery {
            field: "price".into(),
            gte,
            gt,
            lte,
            lt,
        })
    }

    fn termnum(v: f64) -> QueryNode {
        QueryNode::Term(TermQuery {
            field: "price".into(),
            value: FieldValue::Number(v),
        })
    }

    fn termsnum(vs: &[f64]) -> QueryNode {
        QueryNode::Terms(TermsQuery {
            field: "price".into(),
            values: vs.iter().map(|v| FieldValue::Number(*v)).collect(),
        })
    }

    fn termkw(field: &str, v: &str) -> QueryNode {
        QueryNode::Term(TermQuery {
            field: field.into(),
            value: FieldValue::String(v.into()),
        })
    }

    /// The full battery of range/exact/boolean queries the dual-path test runs.
    /// Every inclusive/exclusive bound + open-endedness combination, exact-match,
    /// empty ranges, and boolean composition with another field.
    fn all_queries() -> Vec<(&'static str, QueryNode)> {
        vec![
            // closed ranges (inclusive both)
            ("closed_incl", rangeq(Some(-5.0), None, Some(5.0), None)),
            // closed range exclusive both
            ("closed_excl", rangeq(None, Some(-5.0), None, Some(5.0))),
            // closed range mixed inclusivity
            ("mixed_gte_lt", rangeq(Some(-2.0), None, None, Some(7.0))),
            ("mixed_gt_lte", rangeq(None, Some(-2.0), Some(7.0), None)),
            // open-low (..hi]
            ("open_low_incl", rangeq(None, None, Some(3.0), None)),
            ("open_low_excl", rangeq(None, None, None, Some(3.0))),
            // open-high [lo..
            ("open_high_incl", rangeq(Some(-3.0), None, None, None)),
            ("open_high_excl", rangeq(None, Some(-3.0), None, None)),
            // fully open
            ("fully_open", rangeq(None, None, None, None)),
            // exact via inclusive lo==hi (single-value)
            ("single_val", rangeq(Some(0.0), None, Some(0.0), None)),
            // empty range: lo > hi
            ("empty_inverted", rangeq(Some(5.0), None, Some(-5.0), None)),
            // empty range: exclusive at the same point
            ("empty_excl_point", rangeq(None, Some(1.0), None, Some(1.0))),
            // exact-match Term (lo==hi semantics)
            ("exact_term_0", termnum(0.0)),
            ("exact_term_neg", termnum(-3.0)),
            ("exact_term_missing", termnum(999.0)),
            // multi-value Terms
            ("terms_multi", termsnum(&[-3.0, 0.0, 3.0])),
            // boolean And with another field
            (
                "and_range_cat",
                QueryNode::And(vec![
                    rangeq(Some(-5.0), None, Some(5.0), None),
                    termkw("cat", "red"),
                ]),
            ),
            // boolean Or of two ranges
            (
                "or_two_ranges",
                QueryNode::Or(vec![
                    rangeq(None, None, None, Some(-2.0)),
                    rangeq(Some(2.0), None, None, None),
                ]),
            ),
            // boolean And of exact + range (drives the cheapest-clause planner)
            (
                "and_exact_range",
                QueryNode::And(vec![
                    termnum(0.0),
                    rangeq(Some(-5.0), None, Some(5.0), None),
                ]),
            ),
        ]
    }

    /// Assert the Number field `price` has an EMPTY in-RAM `values` index but an
    /// attached segment — the RAM-bounded invariant after a seal.
    fn assert_values_dropped(e: &Engine) {
        let state = e.state.read().unwrap();
        let coll = state.collections.get("c").unwrap();
        let FieldIndex::Number(n) = coll.fields.get("price").unwrap() else {
            panic!("price must be a Number field");
        };
        assert!(n.segment.is_some(), "segment must be attached");
        assert!(
            n.values.is_empty(),
            "RAM `values` index must be DROPPED after seal (got {} entries)",
            n.values.len()
        );
        assert!(
            n.forward.is_empty(),
            "RAM `forward` must be dropped after seal"
        );
    }

    /// (value → external_ids) duplicate groups for a Number field, comparable.
    fn dup_map(e: &Engine, field: &str) -> BTreeMap<String, BTreeSet<String>> {
        e.duplicates(
            "c",
            crate::types::DuplicatesRequest {
                field: field.into(),
                min_group_size: 2,
                limit: 100_000,
                offset: 0,
            },
        )
        .unwrap()
        .groups
        .into_iter()
        .map(|g| {
            // Number duplicate group values serialize as JSON numbers.
            let v = g.value.to_string();
            (v, g.external_ids.into_iter().collect::<BTreeSet<String>>())
        })
        .collect()
    }

    fn uniq(e: &Engine, field: &str) -> u64 {
        e.stats("c")
            .unwrap()
            .fields
            .get(field)
            .unwrap()
            .unique_terms
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// PATH A (segment OFF, in-RAM `values` range walk) must equal PATH B
        /// (sealed: RAM `values` DROPPED, range/exact driven from the on-disk
        /// SORTED-VALUE column binary-search) for the WHOLE battery: closed /
        /// open-low / open-high / fully-open ranges, EXCLUSIVE vs INCLUSIVE
        /// bounds, empty ranges, exact-match (lo==hi), single-value, multi-Terms,
        /// boolean And/Or with another field, AND the asc/desc number-sort page.
        /// Byte-identical result SETS + totals on both paths. Stress varied f64:
        /// negatives, zeros (±0.0), duplicates, large/small magnitudes.
        #[test]
        fn range_segment_matches_live(
            docs in proptest::collection::vec(
                (
                    proptest::option::weighted(
                        0.9,
                        prop::sample::select(vec![
                            -1000.5_f64, -7.0, -3.0, -2.0, -0.0, 0.0, 1.0, 2.0, 3.0,
                            3.0, 0.0, -3.0, 7.0, 42.0, 1e9, -1e9, 0.5, -0.5,
                        ]),
                    ),
                    proptest::option::weighted(
                        0.7,
                        prop::sample::select(vec!["red", "green", "blue"]),
                    ),
                ),
                1..80,
            ),
        ) {
            // --- PATH A: in-RAM range index (segment OFF). ---
            let e = Arc::new(Engine::new());
            e.create_collection("c", schema()).unwrap();
            for (i, (price, cat)) in docs.iter().enumerate() {
                index_num(&e, &format!("d{i}"), *price, *cat);
            }

            let qs = all_queries();
            let a_sets: Vec<BTreeSet<String>> =
                qs.iter().map(|(_, q)| set_of(&run(&e, q.clone()))).collect();
            let a_totals: Vec<u64> =
                qs.iter().map(|(_, q)| search_total(&e, q.clone())).collect();
            let a_sort_asc = run_sorted(&e, rangeq(None, None, None, None), "price", SortOrder::Asc);
            let a_sort_desc = run_sorted(&e, rangeq(None, None, None, None), "price", SortOrder::Desc);
            let a_uniq = uniq(&e, "price");

            // --- PATH B: seal `price` (drops RAM `values`), rerun from the mmap. ---
            let dir = tempfile::tempdir().unwrap();
            let sealed = e.__seal_number_field_to_segment("c", "price", dir.path()).unwrap();
            prop_assert_eq!(sealed as usize, docs.len(), "all docs sealed");
            assert_values_dropped(&e); // RAM-bounded: values index gone

            for (i, (name, q)) in qs.iter().enumerate() {
                let b_set = set_of(&run(&e, q.clone()));
                prop_assert_eq!(&a_sets[i], &b_set, "result SET diverged for `{}`", name);
                let b_total = search_total(&e, q.clone());
                prop_assert_eq!(a_totals[i], b_total, "total diverged for `{}`", name);
            }
            let b_sort_asc = run_sorted(&e, rangeq(None, None, None, None), "price", SortOrder::Asc);
            let b_sort_desc = run_sorted(&e, rangeq(None, None, None, None), "price", SortOrder::Desc);
            // Sort order is the field-sorted walk — must be value-identical (the
            // ORDERED sequence, not just the set), since `try_plan` drives the page
            // straight off the sorted-value column.
            prop_assert_eq!(&a_sort_asc, &b_sort_asc, "asc number-sort page diverged");
            prop_assert_eq!(&a_sort_desc, &b_sort_desc, "desc number-sort page diverged");
            prop_assert_eq!(a_uniq, uniq(&e, "price"), "unique_terms diverged");
        }
    }

    /// RAM-BOUNDED + REOPEN-NO-REBUILD: seal the WHOLE collection to disk, reopen
    /// from the segments alone (no CBOR snapshot), and assert the reopened Number
    /// field's `values` map is EMPTY while range/exact queries still answer from
    /// the mmap (a binary-search on the sorted-value column, NOT an O(n) forward
    /// scan — the rebuild loop in `open_from_segment` was deleted in 2h-3).
    #[test]
    fn reopen_drives_from_segment_with_empty_values() {
        let dir = tempfile::tempdir().unwrap();
        let e = Arc::new(Engine::new());
        e.create_collection("c", schema()).unwrap();
        index_num(&e, "a", Some(-3.0), Some("red"));
        index_num(&e, "b", Some(0.0), Some("green"));
        index_num(&e, "c2", Some(3.0), None);
        index_num(&e, "d", Some(0.0), Some("red")); // dup value 0.0
        index_num(&e, "e", None, Some("green")); // present-but-no-price

        let want_range = set_of(&run(&e, rangeq(Some(-3.0), None, None, Some(3.0))));
        let want_exact0 = set_of(&run(&e, termnum(0.0)));
        let want_uniq = uniq(&e, "price");

        e.__seal_collection_to_segments("c", dir.path(), 1).unwrap();
        let schema = e.__collection_schema("c").unwrap();
        let e2 = Engine::__open_collection_from_segments("c", dir.path(), schema, 1).unwrap();

        // The reopened Number `values` map must be EMPTY (no RAM rebuild) ...
        {
            let state = e2.state.read().unwrap();
            let coll = state.collections.get("c").unwrap();
            let FieldIndex::Number(n) = coll.fields.get("price").unwrap() else {
                panic!("price must be a Number field");
            };
            assert!(n.segment.is_some(), "reopened segment attached");
            assert!(
                n.values.is_empty(),
                "reopen must NOT rebuild `values` in RAM (got {} entries) — the \
                 range read is a binary-search on the mmap sorted-value column, \
                 not an O(N) forward scan that rebuilds `values`",
                n.values.len()
            );
            // The on-disk sorted-value column carries the distinct values, so the
            // segment can answer a range WITHOUT the RAM map.
            let seg = n.segment.as_ref().unwrap();
            assert_eq!(
                seg.number_distinct_count(),
                3,
                "distinct values {{-3.0, 0.0, 3.0}} live on the mmap sorted column"
            );
        }

        // ... yet range / exact queries still resolve, entirely from the mmap.
        assert_eq!(
            set_of(&run(&e2, rangeq(Some(-3.0), None, None, Some(3.0)))),
            want_range,
            "range post-reopen"
        );
        assert_eq!(
            set_of(&run(&e2, termnum(0.0))),
            want_exact0,
            "exact post-reopen"
        );
        assert_eq!(uniq(&e2, "price"), want_uniq, "unique_terms post-reopen");
    }

    #[test]
    fn segment_keyword_range_skip_cache_matches_live_after_delete_and_tail() {
        fn build(e: &Engine, n: usize) {
            e.create_collection("c", schema()).unwrap();
            for i in 0..n {
                index_num(
                    e,
                    &format!("d{i}"),
                    Some((i % 100) as f64),
                    Some(if i % 2 == 0 { "red" } else { "blue" }),
                );
            }
        }

        let n = 20_000usize; // red df=10k, high enough to take the dense fast path.
        let query = QueryNode::And(vec![
            termkw("cat", "red"),
            rangeq(Some(10.0), None, None, Some(20.0)),
        ]);

        let oracle = Arc::new(Engine::new());
        build(&oracle, n);
        oracle.delete("c", "d10", None).unwrap();
        index_num(&oracle, "tail", Some(12.0), Some("red"));
        let want = set_of(&run(&oracle, query.clone()));
        let want_total = search_total(&oracle, query.clone());
        let want_sort = run_sorted(&oracle, termkw("cat", "red"), "price", SortOrder::Asc);

        let subject = Arc::new(Engine::new());
        build(&subject, n);
        let dir = tempfile::tempdir().unwrap();
        subject
            .__seal_collection_to_segments("c", dir.path(), 1)
            .unwrap();
        {
            let state = subject.state.read().unwrap();
            let coll = state.collections.get("c").unwrap();
            let FieldIndex::Keyword(k) = coll.fields.get("cat").unwrap() else {
                panic!("cat");
            };
            let FieldIndex::Number(num) = coll.fields.get("price").unwrap() else {
                panic!("price");
            };
            assert!(k.segment.is_some(), "keyword segment attached");
            assert!(k.terms.is_empty(), "keyword RAM driver dropped after seal");
            assert!(num.segment.is_some(), "number segment attached");
            assert!(
                num.values.is_empty(),
                "number RAM driver dropped after seal"
            );
        }

        subject.delete("c", "d10", None).unwrap();
        index_num(&subject, "tail", Some(12.0), Some("red"));

        let got = set_of(&run(&subject, query.clone()));
        let got_total = search_total(&subject, query);
        let got_sort = run_sorted(&subject, termkw("cat", "red"), "price", SortOrder::Asc);
        assert_eq!(got, want, "segment term+range fast path set diverged");
        assert_eq!(
            got_total, want_total,
            "segment term+range fast path total diverged"
        );
        assert_eq!(
            got_sort, want_sort,
            "segment keyword-filtered number sort page diverged"
        );

        let state = subject.state.read().unwrap();
        let coll = state.collections.get("c").unwrap();
        let FieldIndex::Number(num) = coll.fields.get("price").unwrap() else {
            panic!("price");
        };
        let cache = num
            .keyword_range_cache
            .read()
            .expect("number keyword-range cache poisoned");
        assert!(
            cache.contains_key("cat\0red"),
            "planner should have built the lazy segment keyword+range skip cache"
        );
    }

    /// LIVE-TAIL UNION: seal, then index more docs (tail into the live `values`),
    /// and assert range/exact compose segment-base (sorted-value column) with the
    /// live-tail `values.range`.
    #[test]
    fn live_tail_unions_with_segment_base() {
        let e = Arc::new(Engine::new());
        e.create_collection("c", schema()).unwrap();
        index_num(&e, "base0", Some(1.0), None); // id 0, sealed base
        index_num(&e, "base1", Some(5.0), None); // id 1, sealed base

        let dir = tempfile::tempdir().unwrap();
        let n = e
            .__seal_number_field_to_segment("c", "price", dir.path())
            .unwrap();
        assert_eq!(n, 2);
        assert_values_dropped(&e); // base index now on disk

        // Tail docs after seal → live `values` tail (ids >= n_docs).
        index_num(&e, "tail0", Some(1.0), None); // id 2, tail, SAME value as base0
        index_num(&e, "tail1", Some(9.0), None); // id 3, tail, NEW value

        // range [0, 10]: base {base0=1, base1=5} ∪ tail {tail0=1, tail1=9}.
        let got = set_of(&run(&e, rangeq(Some(0.0), None, Some(10.0), None)));
        let want: BTreeSet<String> = ["base0", "base1", "tail0", "tail1"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(got, want, "range must union segment base + live tail");

        // exact 1.0: base0 (segment) ∪ tail0 (tail).
        let got = set_of(&run(&e, termnum(1.0)));
        let want: BTreeSet<String> = ["base0", "tail0"].iter().map(|s| s.to_string()).collect();
        assert_eq!(got, want, "exact 1.0 must union segment base + live tail");

        // exact 9.0: ONLY the tail (not in the segment).
        let got = set_of(&run(&e, termnum(9.0)));
        let want: BTreeSet<String> = ["tail1".to_string()].into_iter().collect();
        assert_eq!(got, want, "9.0 must come from the live tail alone");

        // exact 5.0: ONLY the segment base.
        let got = set_of(&run(&e, termnum(5.0)));
        let want: BTreeSet<String> = ["base1".to_string()].into_iter().collect();
        assert_eq!(got, want, "5.0 must come from the segment base alone");

        // df composition: value_df(1.0) = 1 (base) + 1 (tail) = 2.
        let state = e.state.read().unwrap();
        let coll = state.collections.get("c").unwrap();
        let FieldIndex::Number(nidx) = coll.fields.get("price").unwrap() else {
            panic!("price");
        };
        let k1 = SortableF64::new(1.0).unwrap();
        let k9 = SortableF64::new(9.0).unwrap();
        let k5 = SortableF64::new(5.0).unwrap();
        let k7 = SortableF64::new(7.0).unwrap();
        assert_eq!(
            nidx.value_df(k1),
            2,
            "df 1.0 must sum segment base + live tail"
        );
        assert_eq!(nidx.value_df(k5), 1, "df 5.0 segment-only");
        assert_eq!(nidx.value_df(k9), 1, "df 9.0 tail-only");
        assert_eq!(nidx.value_df(k7), 0, "df absent value");
    }

    /// DELETE-AFTER-SEAL **WITHOUT** RE-SEAL: seal a Number corpus, delete several
    /// BASE docs (no re-seal), then assert every segment-ON query equals an in-RAM
    /// ORACLE built from the identical op sequence but NEVER sealed — byte-identical
    /// result SETS for range, exact Term, multi Terms, boolean Or/And, plus
    /// duplicates/unique_terms. The 2h tombstone closes the immutable-posting gap.
    #[test]
    fn delete_after_seal_without_reseal_matches_oracle() {
        fn build(e: &Engine) {
            e.create_collection("c", schema()).unwrap();
            index_num(e, "d0", Some(0.0), Some("red"));
            index_num(e, "d1", Some(0.0), Some("red")); // dup 0.0
            index_num(e, "d2", Some(5.0), Some("green"));
            index_num(e, "d3", Some(0.0), Some("red")); // dup 0.0
            index_num(e, "d4", Some(5.0), Some("green")); // dup 5.0
            index_num(e, "d5", Some(-3.0), None);
        }
        let to_delete = ["d1", "d3", "d4"];

        // --- ORACLE: in-RAM, never sealed. ---
        let oracle = Arc::new(Engine::new());
        build(&oracle);
        for d in to_delete {
            oracle.delete("c", d, None).unwrap();
        }

        // --- SUBJECT: seal price + cat, THEN delete (no re-seal). ---
        let subject = Arc::new(Engine::new());
        build(&subject);
        let dir = tempfile::tempdir().unwrap();
        subject
            .__seal_number_field_to_segment("c", "price", dir.path())
            .unwrap();
        subject
            .__seal_keyword_field_to_segment("c", "cat", dir.path())
            .unwrap();
        assert_values_dropped(&subject);
        for d in to_delete {
            subject.delete("c", d, None).unwrap();
        }

        // The deleted base ids are tombstoned, NOT removed from any on-disk posting.
        {
            let state = subject.state.read().unwrap();
            let coll = state.collections.get("c").unwrap();
            let FieldIndex::Number(n) = coll.fields.get("price").unwrap() else {
                panic!("price");
            };
            assert!(n.values.is_empty(), "sealed: values still dropped");
            assert_eq!(n.tombstones.len(), 3, "three base deletes tombstoned");
        }

        // Result SETS must match the oracle on every driver surface.
        let q_range = rangeq(Some(-10.0), None, Some(10.0), None);
        let q_exact = termnum(0.0);
        let q_terms = termsnum(&[0.0, 5.0]);
        let q_or = QueryNode::Or(vec![termnum(0.0), termnum(5.0)]);
        let q_and = QueryNode::And(vec![
            rangeq(Some(-1.0), None, Some(1.0), None),
            termkw("cat", "red"),
        ]);
        // Or-of-RANGES forces the MATERIALIZED `eval_range` → `range_postings`
        // path (NOT the `try_plan` standalone shortcut), so the range tombstone
        // subtraction is exercised: each range child is fully materialized and the
        // deleted base docids must be subtracted from the on-disk union.
        let q_or_ranges = QueryNode::Or(vec![
            rangeq(Some(-1.0), None, Some(1.0), None),
            rangeq(Some(4.0), None, Some(6.0), None),
        ]);

        assert_eq!(
            set_of(&run(&subject, q_range.clone())),
            set_of(&run(&oracle, q_range)),
            "range leaked a deleted doc"
        );
        assert_eq!(
            set_of(&run(&subject, q_exact.clone())),
            set_of(&run(&oracle, q_exact)),
            "exact Term leaked a deleted doc"
        );
        assert_eq!(
            set_of(&run(&subject, q_terms.clone())),
            set_of(&run(&oracle, q_terms)),
            "Terms leaked a deleted doc"
        );
        assert_eq!(
            set_of(&run(&subject, q_or.clone())),
            set_of(&run(&oracle, q_or)),
            "Or leaked a deleted doc"
        );
        assert_eq!(
            set_of(&run(&subject, q_and.clone())),
            set_of(&run(&oracle, q_and)),
            "And leaked a deleted doc"
        );
        assert_eq!(
            set_of(&run(&subject, q_or_ranges.clone())),
            set_of(&run(&oracle, q_or_ranges)),
            "Or-of-ranges (eval_range) leaked a deleted doc"
        );
        assert_eq!(
            dup_map(&subject, "price"),
            dup_map(&oracle, "price"),
            "duplicates(price) diverged after delete"
        );
        assert_eq!(
            uniq(&subject, "price"),
            uniq(&oracle, "price"),
            "unique_terms(price) diverged after delete"
        );

        // Concrete spot-check: 0.0 now only d0 survives (d1, d3 deleted).
        let zero = set_of(&run(&subject, termnum(0.0)));
        assert_eq!(
            zero,
            ["d0".to_string()].into_iter().collect::<BTreeSet<_>>(),
            "0.0 must be only the surviving d0"
        );
        // 5.0: only d2 survives (d4 deleted).
        let five = set_of(&run(&subject, termnum(5.0)));
        assert_eq!(
            five,
            ["d2".to_string()].into_iter().collect::<BTreeSet<_>>(),
            "5.0 must be only the surviving d2"
        );
    }

    /// SORT-after-delete: the segment sort-via-sorted-index walk (Phase 2m) must
    /// drop tombstoned base docids per value, so the ORDERED page matches the
    /// in-RAM oracle exactly (asc + desc). Guards the sort walk's tombstone
    /// subtraction — without it the walk emits deleted base docs (proven by
    /// temporarily disabling the subtraction).
    #[test]
    fn sort_after_delete_matches_oracle() {
        fn build(e: &Engine) {
            e.create_collection("c", schema()).unwrap();
            index_num(e, "d0", Some(0.0), Some("red"));
            index_num(e, "d1", Some(0.0), Some("red"));
            index_num(e, "d2", Some(5.0), Some("green"));
            index_num(e, "d3", Some(0.0), Some("red"));
            index_num(e, "d4", Some(5.0), Some("green"));
            index_num(e, "d5", Some(-3.0), None);
        }
        let to_delete = ["d1", "d3", "d4"];
        let oracle = Arc::new(Engine::new());
        build(&oracle);
        for d in to_delete {
            oracle.delete("c", d, None).unwrap();
        }
        let subject = Arc::new(Engine::new());
        build(&subject);
        let dir = tempfile::tempdir().unwrap();
        subject
            .__seal_number_field_to_segment("c", "price", dir.path())
            .unwrap();
        for d in to_delete {
            subject.delete("c", d, None).unwrap();
        }
        let o_asc = run_sorted(
            &oracle,
            rangeq(None, None, None, None),
            "price",
            SortOrder::Asc,
        );
        let s_asc = run_sorted(
            &subject,
            rangeq(None, None, None, None),
            "price",
            SortOrder::Asc,
        );
        assert_eq!(o_asc, s_asc, "asc sort leaked a deleted doc");
        let o_desc = run_sorted(
            &oracle,
            rangeq(None, None, None, None),
            "price",
            SortOrder::Desc,
        );
        let s_desc = run_sorted(
            &subject,
            rangeq(None, None, None, None),
            "price",
            SortOrder::Desc,
        );
        assert_eq!(o_desc, s_desc, "desc sort leaked a deleted doc");
    }

    /// RE-SEAL (CHECKPOINT) after delete: once re-sealed the deletions are BAKED
    /// into the new segment (2g-A live(id) GC), the tombstone is CLEARED, and
    /// queries still match the oracle.
    #[test]
    fn reseal_bakes_deletes_and_clears_tombstone() {
        fn build(e: &Engine) {
            e.create_collection("c", schema()).unwrap();
            index_num(e, "d0", Some(0.0), Some("red"));
            index_num(e, "d1", Some(0.0), Some("red"));
            index_num(e, "d2", Some(5.0), Some("green"));
            index_num(e, "d3", Some(-3.0), None);
        }
        let to_delete = ["d1", "d2"];

        let oracle = Arc::new(Engine::new());
        build(&oracle);
        for d in to_delete {
            oracle.delete("c", d, None).unwrap();
        }

        let subject = Arc::new(Engine::new());
        build(&subject);
        let dir = tempfile::tempdir().unwrap();
        subject
            .__seal_collection_to_segments("c", dir.path(), 1)
            .unwrap();
        for d in to_delete {
            subject.delete("c", d, None).unwrap();
        }
        {
            let state = subject.state.read().unwrap();
            let coll = state.collections.get("c").unwrap();
            let FieldIndex::Number(n) = coll.fields.get("price").unwrap() else {
                panic!("price");
            };
            assert_eq!(n.tombstones.len(), 2, "deletes tombstoned before re-seal");
        }
        let dir2 = tempfile::tempdir().unwrap();
        subject
            .__seal_collection_to_segments("c", dir2.path(), 2)
            .unwrap();
        {
            let state = subject.state.read().unwrap();
            let coll = state.collections.get("c").unwrap();
            let FieldIndex::Number(n) = coll.fields.get("price").unwrap() else {
                panic!("price");
            };
            assert!(
                n.tombstones.is_empty(),
                "tombstone must be CLEARED after re-seal"
            );
        }

        let q_range = rangeq(Some(-10.0), None, Some(10.0), None);
        assert_eq!(
            set_of(&run(&subject, q_range.clone())),
            set_of(&run(&oracle, q_range)),
            "post-re-seal range diverged"
        );
        // value 5.0 fully deleted (d2 gone) → empty.
        assert_eq!(
            set_of(&run(&subject, termnum(5.0))),
            BTreeSet::new(),
            "5.0 fully deleted — empty after re-seal"
        );
        assert_eq!(
            uniq(&subject, "price"),
            uniq(&oracle, "price"),
            "unique_terms diverged after re-seal"
        );
    }

    /// TEETH: a wrong binary-search bound (inclusivity flip) MUST diverge from the
    /// in-RAM oracle. This pins that `number_range_window` honors INCLUSIVE vs
    /// EXCLUSIVE exactly: a value sitting exactly on a bound is the difference
    /// between the two, so flipping inclusivity changes the result SET.
    #[test]
    fn teeth_inclusivity_flip_changes_result() {
        let e = Arc::new(Engine::new());
        e.create_collection("c", schema()).unwrap();
        index_num(&e, "lo", Some(0.0), None);
        index_num(&e, "mid", Some(5.0), None);
        index_num(&e, "hi", Some(10.0), None);
        let dir = tempfile::tempdir().unwrap();
        e.__seal_number_field_to_segment("c", "price", dir.path())
            .unwrap();
        assert_values_dropped(&e);

        // [0, 10] inclusive → {lo, mid, hi}; (0, 10) exclusive → {mid}. If the
        // window math ignored inclusivity these would be equal — they are NOT, so
        // this is the teeth assertion the spec asks for.
        let incl = set_of(&run(&e, rangeq(Some(0.0), None, Some(10.0), None)));
        let excl = set_of(&run(&e, rangeq(None, Some(0.0), None, Some(10.0))));
        assert_eq!(
            incl.len(),
            3,
            "[0,10] inclusive must include both endpoints"
        );
        assert_eq!(
            excl,
            ["mid".to_string()].into_iter().collect::<BTreeSet<_>>(),
            "(0,10) exclusive must drop both endpoints"
        );
        assert_ne!(
            incl, excl,
            "inclusive and exclusive bounds MUST differ on boundary values"
        );

        // Direct reader-level teeth: an off-by-one in `number_range_window` would
        // make Included(5.0)..=Included(5.0) miss the exact value. Pin it.
        let state = e.state.read().unwrap();
        let coll = state.collections.get("c").unwrap();
        let FieldIndex::Number(n) = coll.fields.get("price").unwrap() else {
            panic!("price")
        };
        let seg = n.segment.as_ref().unwrap();
        let b5 = SortableF64::new(5.0).unwrap().bits();
        let r = seg
            .number_range(Some((b5, true)), Some((b5, true)))
            .unwrap();
        assert_eq!(
            r.len(),
            1,
            "[5,5] inclusive must select exactly the one 5.0 doc"
        );
        let r_excl = seg
            .number_range(Some((b5, false)), Some((b5, false)))
            .unwrap();
        assert_eq!(r_excl.len(), 0, "(5,5) exclusive must be empty");
    }
}

// ---------------------------------------------------------------------------
// Dual-path diff test: segment-backed Set membership read must be
// byte-identical to the live in-RAM read (Stage 2 Phase 2e-A).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod segment_set_diff_tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::Arc;

    fn fieldspec(t: FieldType, analyzer: Option<Analyzer>) -> FieldSpec {
        FieldSpec {
            field_type: t,
            analyzer,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    /// `tags` (Set, the field we seal) + `body` (Text, the AND driver).
    fn schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        fields.insert("tags".into(), fieldspec(FieldType::Set, None));
        fields.insert(
            "body".into(),
            fieldspec(FieldType::Text, Some(Analyzer::WhitespaceLower)),
        );
        CreateCollectionRequest { fields }
    }

    fn req(query: QueryNode) -> SearchRequest {
        SearchRequest {
            query,
            limit: 100_000,
            cursor: None,
            sort: None,
            track_total: true,
            collapse: None,
        }
    }

    fn run(e: &Engine, query: QueryNode) -> Vec<(String, f32)> {
        e.search("c", req(query))
            .unwrap()
            .hits
            .into_iter()
            .map(|h| (h.external_id, h.score))
            .collect()
    }

    fn set_of(rows: &[(String, f32)]) -> BTreeSet<String> {
        rows.iter().map(|(e, _)| e.clone()).collect()
    }

    fn scores_of(rows: &[(String, f32)]) -> BTreeMap<String, u32> {
        rows.iter().map(|(e, s)| (e.clone(), s.to_bits())).collect()
    }

    /// Index one doc: always writes `body`; writes `tags` only when `tags` is
    /// `Some` (so absent-set docs are part of the corpus). A present-but-empty
    /// set is `Some(&[])`.
    fn index_doc(e: &Engine, eid: &str, tags: Option<&[&str]>, tok: bool) {
        let mut items = vec![crate::types::IndexItem {
            external_id: eid.into(),
            field: "body".into(),
            value: FieldValue::String(if tok {
                "tok filler".into()
            } else {
                "filler".into()
            }),
        }];
        if let Some(ts) = tags {
            items.push(crate::types::IndexItem {
                external_id: eid.into(),
                field: "tags".into(),
                value: FieldValue::StringList(ts.iter().map(|s| (*s).to_string()).collect()),
            });
        }
        e.index(
            "c",
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();
    }

    /// A match-DRIVEN AND so the `term tags` membership conjunct is applied as a
    /// per-doc PREDICATE (`clause_matches` → `SetIndex::set_contains`).
    fn term_conjunct(el: &str) -> QueryNode {
        QueryNode::And(vec![
            QueryNode::Match(MatchQuery {
                field: "body".into(),
                text: "tok".into(),
                op: MatchOp::And,
            }),
            QueryNode::Term(TermQuery {
                field: "tags".into(),
                value: FieldValue::String(el.into()),
            }),
        ])
    }

    /// A match-DRIVEN AND with a `terms tags` (OR-of-members) conjunct —
    /// exercises `SetIndex::set_contains_any`.
    fn terms_conjunct(els: &[&str]) -> QueryNode {
        QueryNode::And(vec![
            QueryNode::Match(MatchQuery {
                field: "body".into(),
                text: "tok".into(),
                op: MatchOp::And,
            }),
            QueryNode::Terms(TermsQuery {
                field: "tags".into(),
                values: els
                    .iter()
                    .map(|s| FieldValue::String((*s).into()))
                    .collect(),
            }),
        ])
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        /// PATH A (segment OFF, live in-RAM `forward`) must equal PATH B (Set
        /// field sealed to a shared DICT + CSR offsets + packed segment, then
        /// served from it) — same result SET and byte-identical scores — for
        /// both the `term tags` and `terms tags` membership conjuncts over a
        /// randomized multi-valued corpus (varied N, cardinality incl 0,
        /// absent-`tags` docs).
        #[test]
        fn segment_read_matches_live_read(
            docs in proptest::collection::vec(
                (
                    // tags: ~1-in-5 docs have NO set value; others get 0..4
                    // members drawn from a small pool (multi-valued, deduped).
                    proptest::option::weighted(
                        0.8,
                        proptest::collection::vec(
                            prop::sample::select(vec!["x", "y", "z", "w", "v"]),
                            0..4,
                        ),
                    ),
                    any::<bool>(),
                ),
                1..60,
            ),
        ) {
            // --- PATH A: build the live engine, run both shapes (segment OFF). ---
            let e = Arc::new(Engine::new());
            e.create_collection("c", schema()).unwrap();
            for (i, (tags, tok)) in docs.iter().enumerate() {
                let slice: Option<Vec<&str>> =
                    tags.as_ref().map(|v| v.iter().map(|s| *s).collect());
                index_doc(&e, &format!("d{i}"), slice.as_deref(), *tok);
            }

            let a_term = run(&e, term_conjunct("x"));
            let a_terms = run(&e, terms_conjunct(&["y", "z"]));

            // --- PATH B: seal `tags` to a segment, flip it ON, rerun. ---
            let dir = tempfile::tempdir().unwrap();
            let sealed = e.__seal_set_field_to_segment("c", "tags", dir.path()).unwrap();
            prop_assert_eq!(sealed as usize, docs.len(), "all docs sealed");

            let b_term = run(&e, term_conjunct("x"));
            let b_terms = run(&e, terms_conjunct(&["y", "z"]));

            prop_assert_eq!(set_of(&a_term), set_of(&b_term), "term conjunct set diverged");
            prop_assert_eq!(set_of(&a_terms), set_of(&b_terms), "terms conjunct set diverged");
            prop_assert_eq!(scores_of(&a_term), scores_of(&b_term), "term scores diverged");
            prop_assert_eq!(scores_of(&a_terms), scores_of(&b_terms), "terms scores diverged");
        }
    }

    /// A doc indexed AFTER sealing (docid >= segment n_docs) lives in the live
    /// `forward` tail and must still match through `set_contains`'s fallback.
    #[test]
    fn doc_indexed_after_sealing_served_from_live_tail() {
        let e = Arc::new(Engine::new());
        e.create_collection("c", schema()).unwrap();
        index_doc(&e, "sealed", Some(&["x", "y"]), true); // id 0
        index_doc(&e, "empty", Some(&[]), true); // id 1, present-but-empty

        let dir = tempfile::tempdir().unwrap();
        let n = e
            .__seal_set_field_to_segment("c", "tags", dir.path())
            .unwrap();
        assert_eq!(n, 2, "two docs sealed");

        // New doc after sealing → docid 2 (>= n_docs) → lives in the live tail.
        index_doc(&e, "tail", Some(&["z"]), true);

        // term z: only the tail doc qualifies (NOT in the segment) → proves the
        // set_contains id >= n_docs fallback.
        let got = set_of(&run(&e, term_conjunct("z")));
        let want: BTreeSet<String> = ["tail".to_string()].into_iter().collect();
        assert_eq!(got, want, "tail doc must match via live fallback");

        // term x: only the sealed doc qualifies → served from the segment.
        let got = set_of(&run(&e, term_conjunct("x")));
        let want: BTreeSet<String> = ["sealed".to_string()].into_iter().collect();
        assert_eq!(got, want, "sealed doc must match via segment read");
    }

    /// Direct planner-free check that `SetIndex::set_contains` reads CSR-packed
    /// members from the segment (incl multi-valued + present-empty + absent
    /// docs) and falls back to the live tail past `n_docs`.
    #[test]
    fn set_contains_segment_then_live_split() {
        let e = Arc::new(Engine::new());
        e.create_collection("c", schema()).unwrap();
        index_doc(&e, "a", Some(&["x", "y"]), false); // id 0, multi
        index_doc(&e, "b", None, false); // id 1, absent
        index_doc(&e, "c", Some(&[]), false); // id 2, present-empty

        let dir = tempfile::tempdir().unwrap();
        e.__seal_set_field_to_segment("c", "tags", dir.path())
            .unwrap();
        index_doc(&e, "d", Some(&["z"]), false); // id 3, live tail

        let state = e.state.read().unwrap();
        let coll = state.collections.get("c").unwrap();
        let FieldIndex::Set(s) = coll.fields.get("tags").unwrap() else {
            panic!("tags must be a Set field");
        };
        assert!(s.segment.is_some(), "segment attached");
        // doc 0: {x, y} from the segment (CSR slice of 2 members).
        assert!(s.set_contains(0, "x"));
        assert!(s.set_contains(0, "y"));
        assert!(!s.set_contains(0, "z"));
        // doc 1: absent → no membership.
        assert!(!s.set_contains(1, "x"));
        // doc 2: present-but-empty → no membership but is present.
        assert!(!s.set_contains(2, "x"));
        // doc 3: {z} from the live tail (id >= n_docs).
        assert!(s.set_contains(3, "z"));
        assert!(!s.set_contains(3, "x"));
        // unknown id.
        assert!(!s.set_contains(99, "x"));
    }
}

// ---------------------------------------------------------------------------
// Dual-path diff test for the INVERTED Set driver (Stage 2 Phase 2h-2): the
// segment-driven membership / Terms / boolean RoaringBitmap algebra
// (`element_postings` / `element_df`) must be byte-identical to the in-RAM
// `elements` index, AND after a seal the RAM index is DROPPED (the disk=all
// win) while queries keep serving entirely from the mmap segment. This is the
// Set analogue of `segment_keyword_inverted_diff_tests` and the keystone test
// for Phase 2h-2 — it reuses the same query-time tombstone mechanism.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod segment_set_inverted_diff_tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::Arc;

    fn fieldspec(t: FieldType, analyzer: Option<Analyzer>) -> FieldSpec {
        FieldSpec {
            field_type: t,
            analyzer,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    /// `tags` (Set, sealed) + `cat` (a second Set, for boolean AND/OR
    /// cross-field algebra). Both are inverted-driver fields.
    fn schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        fields.insert("tags".into(), fieldspec(FieldType::Set, None));
        fields.insert("cat".into(), fieldspec(FieldType::Set, None));
        CreateCollectionRequest { fields }
    }

    fn req(query: QueryNode) -> SearchRequest {
        SearchRequest {
            query,
            limit: 100_000,
            cursor: None,
            sort: None,
            track_total: true,
            collapse: None,
        }
    }

    fn run(e: &Engine, query: QueryNode) -> Vec<(String, f32)> {
        e.search("c", req(query))
            .unwrap()
            .hits
            .into_iter()
            .map(|h| (h.external_id, h.score))
            .collect()
    }

    fn set_of(rows: &[(String, f32)]) -> BTreeSet<String> {
        rows.iter().map(|(e, _)| e.clone()).collect()
    }

    fn search_total(e: &Engine, query: QueryNode) -> u64 {
        e.search("c", req(query)).unwrap().total
    }

    /// Index one doc: writes `tags` (multi-valued) when `Some`, `cat` when
    /// `Some`. A doc with neither still interns via a filler `tags` member so it
    /// is part of the corpus (mirrors the keyword module's filler).
    fn index_set(e: &Engine, eid: &str, tags: Option<&[&str]>, cat: Option<&[&str]>) {
        let mut items = Vec::new();
        if let Some(ts) = tags {
            items.push(crate::types::IndexItem {
                external_id: eid.into(),
                field: "tags".into(),
                value: FieldValue::StringList(ts.iter().map(|s| (*s).to_string()).collect()),
            });
        }
        if let Some(cs) = cat {
            items.push(crate::types::IndexItem {
                external_id: eid.into(),
                field: "cat".into(),
                value: FieldValue::StringList(cs.iter().map(|s| (*s).to_string()).collect()),
            });
        }
        if items.is_empty() {
            items.push(crate::types::IndexItem {
                external_id: eid.into(),
                field: "tags".into(),
                value: FieldValue::StringList(vec!["zzz_filler".into()]),
            });
        }
        e.index(
            "c",
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();
    }

    fn term(field: &str, v: &str) -> QueryNode {
        QueryNode::Term(TermQuery {
            field: field.into(),
            value: FieldValue::String(v.into()),
        })
    }

    fn terms(field: &str, vs: &[&str]) -> QueryNode {
        QueryNode::Terms(TermsQuery {
            field: field.into(),
            values: vs.iter().map(|s| FieldValue::String((*s).into())).collect(),
        })
    }

    /// Assert the Set field `tags` has an EMPTY in-RAM `elements` index but an
    /// attached segment — the RAM-bounded invariant after a seal/reopen.
    fn assert_elements_dropped(e: &Engine) {
        let state = e.state.read().unwrap();
        let coll = state.collections.get("c").unwrap();
        let FieldIndex::Set(s) = coll.fields.get("tags").unwrap() else {
            panic!("tags must be a Set field");
        };
        assert!(s.segment.is_some(), "segment must be attached");
        assert!(
            s.elements.is_empty(),
            "RAM `elements` index must be DROPPED after seal (got {} entries)",
            s.elements.len()
        );
        assert!(
            s.forward.is_empty(),
            "RAM `forward` must be dropped after seal"
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        /// PATH A (segment OFF, in-RAM `elements`) must equal PATH B (sealed: RAM
        /// `elements` DROPPED, driven from the mmap posting column) for the whole
        /// inverted-driver surface: standalone `Term`, standalone `Terms`,
        /// boolean `Or` of two members, boolean `And` across two set fields, and
        /// the `try_plan` standalone-Term total. Byte-identical result sets AND
        /// identical totals on both paths.
        #[test]
        fn inverted_segment_matches_live(
            docs in proptest::collection::vec(
                (
                    // tags: ~1-in-5 docs absent; others 0..4 deduped members.
                    proptest::option::weighted(
                        0.8,
                        proptest::collection::vec(
                            prop::sample::select(vec!["alpha", "beta", "gamma", "delta"]),
                            0..4,
                        ),
                    ),
                    // cat: ~1-in-3 docs absent; others 0..3 members.
                    proptest::option::weighted(
                        0.7,
                        proptest::collection::vec(
                            prop::sample::select(vec!["red", "green"]),
                            0..3,
                        ),
                    ),
                ),
                1..70,
            ),
        ) {
            // --- PATH A: in-RAM inverted index (segment OFF). ---
            let e = Arc::new(Engine::new());
            e.create_collection("c", schema()).unwrap();
            for (i, (tags, cat)) in docs.iter().enumerate() {
                let ts: Option<Vec<&str>> = tags.as_ref().map(|v| v.iter().map(|s| *s).collect());
                let cs: Option<Vec<&str>> = cat.as_ref().map(|v| v.iter().map(|s| *s).collect());
                index_set(&e, &format!("d{i}"), ts.as_deref(), cs.as_deref());
            }

            let a_term = run(&e, term("tags", "alpha"));
            let a_terms = run(&e, terms("tags", &["beta", "gamma"]));
            let a_or = run(&e, QueryNode::Or(vec![term("tags", "alpha"), term("tags", "delta")]));
            let a_and = run(&e, QueryNode::And(vec![term("tags", "beta"), term("cat", "red")]));
            let a_total = search_total(&e, term("tags", "gamma"));

            // --- PATH B: seal `tags` (drops RAM `elements`), rerun from the mmap. ---
            let dir = tempfile::tempdir().unwrap();
            let sealed = e.__seal_set_field_to_segment("c", "tags", dir.path()).unwrap();
            prop_assert_eq!(sealed as usize, docs.len(), "all docs sealed");
            assert_elements_dropped(&e); // RAM-bounded: elements index gone

            let b_term = run(&e, term("tags", "alpha"));
            let b_terms = run(&e, terms("tags", &["beta", "gamma"]));
            let b_or = run(&e, QueryNode::Or(vec![term("tags", "alpha"), term("tags", "delta")]));
            let b_and = run(&e, QueryNode::And(vec![term("tags", "beta"), term("cat", "red")]));
            let b_total = search_total(&e, term("tags", "gamma"));

            prop_assert_eq!(set_of(&a_term), set_of(&b_term), "Term set diverged");
            prop_assert_eq!(set_of(&a_terms), set_of(&b_terms), "Terms set diverged");
            prop_assert_eq!(set_of(&a_or), set_of(&b_or), "Or set diverged");
            prop_assert_eq!(set_of(&a_and), set_of(&b_and), "And set diverged");
            prop_assert_eq!(a_total, b_total, "standalone-term total diverged");
        }
    }

    /// RAM-BOUNDED + REOPEN-NO-REBUILD: seal the WHOLE collection to disk, reopen
    /// from the segments alone (no CBOR snapshot), and assert the reopened Set
    /// field's `elements` map is EMPTY while membership/Terms queries still return
    /// correct results entirely from the mmap segment.
    #[test]
    fn reopen_drives_from_segment_with_empty_elements() {
        let dir = tempfile::tempdir().unwrap();
        let e = Arc::new(Engine::new());
        e.create_collection("c", schema()).unwrap();
        index_set(&e, "a", Some(&["alpha", "beta"]), Some(&["red"]));
        index_set(&e, "b", Some(&["beta"]), Some(&["green"]));
        index_set(&e, "c2", Some(&["gamma"]), None);
        index_set(&e, "d", Some(&["alpha"]), Some(&["red"]));
        index_set(&e, "e", None, Some(&["green"])); // present-but-no-tags

        let want_alpha = set_of(&run(&e, term("tags", "alpha")));
        let want_beta_gamma = set_of(&run(&e, terms("tags", &["beta", "gamma"])));

        e.__seal_collection_to_segments("c", dir.path(), 1).unwrap();
        let schema = e.__collection_schema("c").unwrap();
        let e2 = Engine::__open_collection_from_segments("c", dir.path(), schema, 1).unwrap();

        // The reopened Set `elements` map must be EMPTY (no RAM rebuild) ...
        {
            let state = e2.state.read().unwrap();
            let coll = state.collections.get("c").unwrap();
            let FieldIndex::Set(s) = coll.fields.get("tags").unwrap() else {
                panic!("tags must be a Set field");
            };
            assert!(s.segment.is_some(), "reopened segment attached");
            assert!(
                s.elements.is_empty(),
                "reopen must NOT rebuild `elements` in RAM (got {} entries)",
                s.elements.len()
            );
        }

        // ... yet the inverted queries still resolve, entirely from the mmap.
        assert_eq!(
            set_of(&run(&e2, term("tags", "alpha"))),
            want_alpha,
            "Term post-reopen"
        );
        assert_eq!(
            set_of(&run(&e2, terms("tags", &["beta", "gamma"]))),
            want_beta_gamma,
            "Terms post-reopen"
        );
    }

    /// LIVE-TAIL UNION: seal, then index more docs (tail into the live
    /// `elements`), and assert a Term query returns base (segment) + tail (RAM)
    /// composed, plus the `element_df` sum.
    #[test]
    fn live_tail_unions_with_segment_base() {
        let e = Arc::new(Engine::new());
        e.create_collection("c", schema()).unwrap();
        index_set(&e, "base0", Some(&["alpha"]), None); // id 0, sealed base
        index_set(&e, "base1", Some(&["beta"]), None); // id 1, sealed base

        let dir = tempfile::tempdir().unwrap();
        let n = e
            .__seal_set_field_to_segment("c", "tags", dir.path())
            .unwrap();
        assert_eq!(n, 2);
        assert_elements_dropped(&e); // base index now on disk

        // Tail docs after seal → land in the live `elements` tail (ids >= n_docs).
        index_set(&e, "tail0", Some(&["alpha"]), None); // id 2, SAME element as base0
        index_set(&e, "tail1", Some(&["gamma"]), None); // id 3, NEW element

        // element alpha: base id (base0) UNION tail id (tail0).
        let got = set_of(&run(&e, term("tags", "alpha")));
        let want: BTreeSet<String> = ["base0".into(), "tail0".into()].into_iter().collect();
        assert_eq!(got, want, "alpha must union segment base + live tail");

        // element gamma: ONLY the tail (not in the segment).
        let got = set_of(&run(&e, term("tags", "gamma")));
        let want: BTreeSet<String> = ["tail1".into()].into_iter().collect();
        assert_eq!(got, want, "gamma must come from the live tail alone");

        // element beta: ONLY the segment base.
        let got = set_of(&run(&e, term("tags", "beta")));
        let want: BTreeSet<String> = ["base1".into()].into_iter().collect();
        assert_eq!(got, want, "beta must come from the segment base alone");

        // df composition: element_df(alpha) = 1 (base) + 1 (tail) = 2.
        let state = e.state.read().unwrap();
        let coll = state.collections.get("c").unwrap();
        let FieldIndex::Set(s) = coll.fields.get("tags").unwrap() else {
            panic!("tags must be a Set field");
        };
        assert_eq!(
            s.element_df("alpha"),
            2,
            "df must sum segment base + live tail"
        );
        assert_eq!(s.element_df("beta"), 1, "df beta segment-only");
        assert_eq!(s.element_df("gamma"), 1, "df gamma tail-only");
        assert_eq!(s.element_df("missing"), 0, "df absent element");
    }

    // -----------------------------------------------------------------------
    // Phase 2h-2: delete-after-seal query-time tombstone (REUSED from 2h-1).
    // After the seal drops the in-RAM `elements` index, `drop_eid` on a SEALED
    // base docid is a NO-OP on the immutable on-disk posting, so segment-driven
    // Set queries would LEAK the deleted doc until the next re-seal. The fix
    // records sealed-base deletes in the per-field `tombstones` RoaringBitmap
    // (the same field shape as the Keyword index) that every segment-ON accessor
    // subtracts. These tests pin the fix and prove its teeth.
    // -----------------------------------------------------------------------

    /// (value → external_ids) of every duplicate group, as a comparable map.
    fn dup_map(e: &Engine, field: &str) -> BTreeMap<String, BTreeSet<String>> {
        e.duplicates(
            "c",
            crate::types::DuplicatesRequest {
                field: field.into(),
                min_group_size: 2,
                limit: 100_000,
                offset: 0,
            },
        )
        .unwrap()
        .groups
        .into_iter()
        .map(|g| {
            let v = g.value.as_str().unwrap().to_string();
            (v, g.external_ids.into_iter().collect::<BTreeSet<String>>())
        })
        .collect()
    }

    /// `unique_terms` of `field` via the public stats surface.
    fn uniq(e: &Engine, field: &str) -> u64 {
        e.stats("c")
            .unwrap()
            .fields
            .get(field)
            .unwrap()
            .unique_terms
    }

    /// DELETE-AFTER-SEAL **WITHOUT** RE-SEAL: seal a Set corpus, delete several
    /// BASE docs (no re-seal), then assert every segment-ON query equals an
    /// in-RAM ORACLE built from the identical op sequence but NEVER sealed —
    /// byte-identical result SETS for standalone Term, multi-member Terms,
    /// boolean Or, cross-field And, plus identical `duplicates`/`unique_terms`.
    /// This is the window the seal-time GC does NOT cover; the tombstone closes it.
    #[test]
    fn delete_after_seal_without_reseal_matches_oracle() {
        // Shared corpus builder so the oracle and the sealed engine see the SAME
        // index sequence. Multi-valued: alpha in 3 docs, beta in 2, red in 3.
        fn build(e: &Engine) {
            e.create_collection("c", schema()).unwrap();
            index_set(e, "d0", Some(&["alpha", "beta"]), Some(&["red"]));
            index_set(e, "d1", Some(&["alpha"]), Some(&["red"]));
            index_set(e, "d2", Some(&["beta"]), Some(&["green"]));
            index_set(e, "d3", Some(&["alpha"]), Some(&["red"]));
            index_set(e, "d4", Some(&["beta", "gamma"]), Some(&["green"]));
            index_set(e, "d5", Some(&["gamma"]), None);
        }
        let to_delete = ["d1", "d3", "d4"];

        // --- ORACLE: in-RAM, never sealed. Same build + same deletes. ---
        let oracle = Arc::new(Engine::new());
        build(&oracle);
        for d in to_delete {
            oracle.delete("c", d, None).unwrap();
        }

        // --- SUBJECT: seal tags + cat to segments, THEN delete (no re-seal). ---
        let subject = Arc::new(Engine::new());
        build(&subject);
        let dir = tempfile::tempdir().unwrap();
        subject
            .__seal_set_field_to_segment("c", "tags", dir.path())
            .unwrap();
        subject
            .__seal_set_field_to_segment("c", "cat", dir.path())
            .unwrap();
        assert_elements_dropped(&subject); // RAM `elements` gone — drives from mmap
        for d in to_delete {
            subject.delete("c", d, None).unwrap();
        }

        // The deleted ids are now tombstoned (base ids < n_docs), NOT removed
        // from any on-disk posting. Confirm the bitmap actually recorded them.
        {
            let state = subject.state.read().unwrap();
            let coll = state.collections.get("c").unwrap();
            let FieldIndex::Set(s) = coll.fields.get("tags").unwrap() else {
                panic!("tags");
            };
            assert!(s.elements.is_empty(), "sealed: elements still dropped");
            assert_eq!(s.tombstones.len(), 3, "three base deletes tombstoned");
        }

        // Result SETS must match the oracle on every driver surface.
        let q_term = term("tags", "alpha");
        let q_terms = terms("tags", &["alpha", "beta"]);
        let q_or = QueryNode::Or(vec![term("tags", "alpha"), term("tags", "beta")]);
        let q_and = QueryNode::And(vec![term("tags", "alpha"), term("cat", "red")]);

        assert_eq!(
            set_of(&run(&subject, q_term.clone())),
            set_of(&run(&oracle, q_term)),
            "standalone Term leaked a deleted doc"
        );
        assert_eq!(
            set_of(&run(&subject, q_terms.clone())),
            set_of(&run(&oracle, q_terms)),
            "multi-member Terms leaked a deleted doc"
        );
        assert_eq!(
            set_of(&run(&subject, q_or.clone())),
            set_of(&run(&oracle, q_or)),
            "boolean Or leaked a deleted doc"
        );
        assert_eq!(
            set_of(&run(&subject, q_and.clone())),
            set_of(&run(&oracle, q_and)),
            "cross-field And leaked a deleted doc"
        );

        // duplicates + unique_terms must match the oracle on sealed data.
        assert_eq!(
            dup_map(&subject, "tags"),
            dup_map(&oracle, "tags"),
            "find_duplicates(tags) diverged on sealed-after-delete"
        );
        assert_eq!(
            dup_map(&subject, "cat"),
            dup_map(&oracle, "cat"),
            "find_duplicates(cat) diverged on sealed-after-delete"
        );
        assert_eq!(
            uniq(&subject, "tags"),
            uniq(&oracle, "tags"),
            "unique_terms(tags) diverged on sealed-after-delete"
        );
        assert_eq!(
            uniq(&subject, "cat"),
            uniq(&oracle, "cat"),
            "unique_terms(cat) diverged on sealed-after-delete"
        );

        // Concrete spot-checks: deleted docs GONE; a fully-deleted element yields
        // None (empty result), not a leak. After deleting d1,d3,d4:
        // alpha={d0}, beta={d2}, gamma={d5}.
        let alpha = set_of(&run(&subject, term("tags", "alpha")));
        assert_eq!(
            alpha,
            ["d0".to_string()].into_iter().collect::<BTreeSet<_>>(),
            "alpha must be only the surviving d0"
        );
        assert!(
            run(&subject, term("tags", "beta"))
                .iter()
                .all(|(eid, _)| eid != "d4"),
            "deleted d4 must not appear under beta"
        );
        // gamma was on d4 (deleted) and d5 (alive) → only d5 survives.
        let gamma = set_of(&run(&subject, term("tags", "gamma")));
        assert_eq!(
            gamma,
            ["d5".to_string()].into_iter().collect::<BTreeSet<_>>(),
            "gamma must drop deleted d4, keep d5"
        );
    }

    /// RE-SEAL (CHECKPOINT) after delete: once the field is re-sealed the
    /// deletions are BAKED into the new segment (via the live(id) GC), the
    /// tombstone is CLEARED, and queries still match the oracle.
    #[test]
    fn reseal_bakes_deletes_and_clears_tombstone() {
        fn build(e: &Engine) {
            e.create_collection("c", schema()).unwrap();
            // beta lives ONLY on d2 (deleted) so it is fully removed after re-seal;
            // alpha lives on d0 (kept) and d1 (deleted).
            index_set(e, "d0", Some(&["alpha"]), Some(&["red"]));
            index_set(e, "d1", Some(&["alpha"]), Some(&["red"]));
            index_set(e, "d2", Some(&["beta"]), Some(&["green"]));
            index_set(e, "d3", Some(&["gamma"]), None);
        }
        let to_delete = ["d1", "d2"];

        let oracle = Arc::new(Engine::new());
        build(&oracle);
        for d in to_delete {
            oracle.delete("c", d, None).unwrap();
        }

        let subject = Arc::new(Engine::new());
        build(&subject);
        let dir = tempfile::tempdir().unwrap();
        subject
            .__seal_collection_to_segments("c", dir.path(), 1)
            .unwrap();
        for d in to_delete {
            subject.delete("c", d, None).unwrap();
        }
        // Tombstone holds the two base deletes pre-re-seal.
        {
            let state = subject.state.read().unwrap();
            let coll = state.collections.get("c").unwrap();
            let FieldIndex::Set(s) = coll.fields.get("tags").unwrap() else {
                panic!("tags");
            };
            assert_eq!(s.tombstones.len(), 2, "deletes tombstoned before re-seal");
        }
        // RE-SEAL: the live(id) gather excludes the tombstoned ids, so the NEW
        // segment has them absent; the tombstone is reset to empty.
        let dir2 = tempfile::tempdir().unwrap();
        subject
            .__seal_collection_to_segments("c", dir2.path(), 2)
            .unwrap();
        {
            let state = subject.state.read().unwrap();
            let coll = state.collections.get("c").unwrap();
            let FieldIndex::Set(s) = coll.fields.get("tags").unwrap() else {
                panic!("tags");
            };
            assert!(
                s.tombstones.is_empty(),
                "tombstone must be CLEARED after re-seal (deletes baked in)"
            );
        }

        // Post-re-seal queries match the oracle, now with an EMPTY tombstone.
        let q_term = term("tags", "alpha");
        assert_eq!(
            set_of(&run(&subject, q_term.clone())),
            set_of(&run(&oracle, q_term)),
            "post-re-seal Term diverged from oracle"
        );
        assert_eq!(
            set_of(&run(&subject, term("tags", "beta"))),
            BTreeSet::new(),
            "beta fully deleted — must be empty after re-seal"
        );
        assert_eq!(
            uniq(&subject, "tags"),
            uniq(&oracle, "tags"),
            "unique_terms diverged after re-seal"
        );
    }
}

// ---------------------------------------------------------------------------
// Dual-path diff test: the segment-backed BM25 scan (stored postings + DocLen
// column + header scalars) must be byte-identical to the live in-RAM scan
// (Stage 2 Phase 2e-B). Text tf is NOT rebuildable, so the postings are STORED.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod segment_text_diff_tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::Arc;

    fn fieldspec(t: FieldType, analyzer: Option<Analyzer>) -> FieldSpec {
        FieldSpec {
            field_type: t,
            analyzer,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    /// `body` (Text, the field we seal) + `price` (Number, a filter for the
    /// filtered_search shape).
    fn schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        fields.insert(
            "body".into(),
            fieldspec(FieldType::Text, Some(Analyzer::WhitespaceLower)),
        );
        fields.insert("price".into(), fieldspec(FieldType::Number, None));
        CreateCollectionRequest { fields }
    }

    fn req(query: QueryNode) -> SearchRequest {
        SearchRequest {
            query,
            limit: 100_000,
            cursor: None,
            sort: None,
            track_total: true,
            collapse: None,
        }
    }

    fn run(e: &Engine, query: QueryNode) -> Vec<(String, f32)> {
        e.search("c", req(query))
            .unwrap()
            .hits
            .into_iter()
            .map(|h| (h.external_id, h.score))
            .collect()
    }

    fn set_of(rows: &[(String, f32)]) -> BTreeSet<String> {
        rows.iter().map(|(e, _)| e.clone()).collect()
    }

    /// Key the f32 score BITS by external id, so the dual-path compare proves
    /// byte-identical scores (not just approximate equality).
    fn scores_of(rows: &[(String, f32)]) -> BTreeMap<String, u32> {
        rows.iter().map(|(e, s)| (e.clone(), s.to_bits())).collect()
    }

    /// `unique_terms` for a field via the stats surface (Phase 2h-4 `unique_terms`).
    fn uniq(e: &Engine, field: &str) -> u64 {
        e.stats("c")
            .unwrap()
            .fields
            .get(field)
            .unwrap()
            .unique_terms
    }

    /// Index one doc's `body` text and (optionally) a `price` filter value. The
    /// body is a tf-realistic bag: each chosen token is repeated `tf` times so
    /// the stored term-frequencies vary across the corpus.
    fn index_doc(e: &Engine, eid: &str, body: &str, price: Option<f64>) {
        let mut items = vec![crate::types::IndexItem {
            external_id: eid.into(),
            field: "body".into(),
            value: FieldValue::String(body.into()),
        }];
        if let Some(p) = price {
            items.push(crate::types::IndexItem {
                external_id: eid.into(),
                field: "price".into(),
                value: FieldValue::Number(p),
            });
        }
        e.index(
            "c",
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();
    }

    /// Build a tf-realistic body string from `(token, repeat)` pairs.
    fn body_from(parts: &[(&str, u32)]) -> String {
        let mut out: Vec<&str> = Vec::new();
        for (tok, rep) in parts {
            for _ in 0..*rep {
                out.push(tok);
            }
        }
        out.join(" ")
    }

    /// text_bm25: a single-token OR match — the pure BM25 scan over one token's
    /// posting list (the `score_token` hot loop).
    fn bm25_single(tok: &str) -> QueryNode {
        QueryNode::Match(MatchQuery {
            field: "body".into(),
            text: tok.into(),
            op: MatchOp::Or,
        })
    }

    /// text_and: a 2-token AND match — the intersect-and-sum path (drives from
    /// the rarer token, probes the other by binary-search).
    fn text_and(a: &str, b: &str) -> QueryNode {
        QueryNode::Match(MatchQuery {
            field: "body".into(),
            text: format!("{a} {b}"),
            op: MatchOp::And,
        })
    }

    /// filtered_search: a `match` AND a `price` range filter. The match BM25 is
    /// scored over the candidate set; both the bitmap-driven and match-driven
    /// AND plans route through `eval_match` / `match_doc_score`, so the sealed
    /// postings/doc-len feed the scoring on either plan.
    fn filtered(tok: &str, lo: f64, hi: f64) -> QueryNode {
        QueryNode::And(vec![
            QueryNode::Match(MatchQuery {
                field: "body".into(),
                text: tok.into(),
                op: MatchOp::Or,
            }),
            QueryNode::Range(RangeQuery {
                field: "price".into(),
                gte: Some(lo),
                lte: Some(hi),
                gt: None,
                lt: None,
            }),
        ])
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        /// PATH A (segment OFF, live in-RAM postings) must equal PATH B (Text
        /// field sealed to a token DICT + STORED posting blocks + DocLen column,
        /// then served entirely from it) — same result SET and BYTE-IDENTICAL
        /// f32 scores — across text_bm25 (single token), text_and (2-token AND),
        /// and filtered_search (match + range) over a tf-realistic corpus with
        /// varied token frequencies and document lengths.
        #[test]
        fn segment_bm25_matches_live_scan(
            docs in proptest::collection::vec(
                (
                    // tf of "alpha" (0 == token absent), 0..=4.
                    0u32..5,
                    // tf of "beta", 0..=4.
                    0u32..5,
                    // tf of "gamma" (filler that varies doc_len), 0..=3.
                    0u32..4,
                    // price (always present so the range filter is meaningful).
                    0u32..100,
                ),
                1..50,
            ),
        ) {
            // --- PATH A: build the live engine, run all shapes (segment OFF). ---
            let e = Arc::new(Engine::new());
            e.create_collection("c", schema()).unwrap();
            for (i, (a, b, g, price)) in docs.iter().enumerate() {
                // Every doc has SOME body token so doc_count tracks all docs;
                // a doc with all-zero tfs still posts a single "filler" token.
                let mut parts: Vec<(&str, u32)> = Vec::new();
                if *a > 0 { parts.push(("alpha", *a)); }
                if *b > 0 { parts.push(("beta", *b)); }
                if *g > 0 { parts.push(("gamma", *g)); }
                if parts.is_empty() { parts.push(("filler", 1)); }
                index_doc(&e, &format!("d{i}"), &body_from(&parts), Some(*price as f64));
            }

            let a_single = run(&e, bm25_single("alpha"));
            let a_and = run(&e, text_and("alpha", "beta"));
            let a_filt = run(&e, filtered("alpha", 10.0, 80.0));

            // --- PATH B: seal `body` to a segment, flip it ON, rerun. ---
            let dir = tempfile::tempdir().unwrap();
            let sealed = e.__seal_text_field_to_segment("c", "body", dir.path()).unwrap();
            prop_assert_eq!(sealed as usize, docs.len(), "all docs sealed");

            let b_single = run(&e, bm25_single("alpha"));
            let b_and = run(&e, text_and("alpha", "beta"));
            let b_filt = run(&e, filtered("alpha", 10.0, 80.0));

            prop_assert_eq!(set_of(&a_single), set_of(&b_single), "single-token set diverged");
            prop_assert_eq!(set_of(&a_and), set_of(&b_and), "2-token AND set diverged");
            prop_assert_eq!(set_of(&a_filt), set_of(&b_filt), "filtered set diverged");
            prop_assert_eq!(scores_of(&a_single), scores_of(&b_single), "single-token scores diverged");
            prop_assert_eq!(scores_of(&a_and), scores_of(&b_and), "2-token AND scores diverged");
            prop_assert_eq!(scores_of(&a_filt), scores_of(&b_filt), "filtered scores diverged");
        }
    }

    /// Direct planner-free check that the sealed `TextIndex` reads postings /
    /// doc-len / df / corpus scalars from the segment, bit-identical to live.
    #[test]
    fn text_reads_from_segment_after_seal() {
        let e = Arc::new(Engine::new());
        e.create_collection("c", schema()).unwrap();
        index_doc(&e, "a", &body_from(&[("alpha", 3), ("beta", 1)]), Some(5.0)); // id 0, len 4
        index_doc(&e, "b", &body_from(&[("alpha", 1)]), Some(6.0)); // id 1, len 1
        index_doc(&e, "c", &body_from(&[("gamma", 2)]), Some(7.0)); // id 2, len 2

        // Capture live BM25 scores before sealing.
        let live = scores_of(&run(&e, bm25_single("alpha")));

        let dir = tempfile::tempdir().unwrap();
        e.__seal_text_field_to_segment("c", "body", dir.path())
            .unwrap();

        // After sealing, the same query must yield byte-identical scores.
        let sealed_scores = scores_of(&run(&e, bm25_single("alpha")));
        assert_eq!(
            live, sealed_scores,
            "sealed BM25 must be bit-identical to live"
        );

        let state = e.state.read().unwrap();
        let coll = state.collections.get("c").unwrap();
        let FieldIndex::Text { idx, .. } = coll.fields.get("body").unwrap() else {
            panic!("body must be a Text field");
        };
        assert!(idx.segment.is_some(), "segment attached");
        // Postings come from the segment.
        let p = idx.tok_postings("alpha").unwrap();
        assert_eq!(p.docids(), &[0u32, 1]);
        assert_eq!(p.tfs(), &[3u32, 1]);
        // doc_len routes through the segment DocLen column.
        assert_eq!(idx.doc_len(0), 4);
        assert_eq!(idx.doc_len(1), 1);
        assert_eq!(idx.doc_len(2), 2);
        // df from the segment posting length.
        assert_eq!(idx.tok_df("alpha"), Some(2));
        assert_eq!(idx.tok_df("durian"), None);
        // corpus scalars now read the LIVE counters (initialized from the header
        // at seal: doc_count=3, total_doc_len=4+1+2). Phase 2h-4.
        assert_eq!(idx.bm25_corpus(), (3, 4 + 1 + 2));
        // Phase 2h-4: `distinct` (and `tokens`/`lens`) are DROPPED at seal — no RAM
        // rebuild. `drop_eid` tombstones a sealed base id instead of consuming
        // `distinct`. The BM25 scan answers entirely from the mmap segment.
        assert!(
            idx.tokens.is_empty(),
            "tokens dropped at seal (postings on disk)"
        );
        assert!(
            idx.distinct_is_empty(),
            "distinct dropped at seal (drop_eid uses tombstones)"
        );
        assert!(
            idx.lens.is_empty(),
            "lens dropped at seal (doc_len reads the segment column)"
        );
        assert!(idx.tombstones.is_empty(), "no deletes yet");
    }

    /// BM25 REOPEN BYTE-IDENTICAL + RAM-BOUNDED (Phase 2h-4): seal the WHOLE
    /// collection to disk, reopen into a FRESH engine from the segments alone (no
    /// CBOR snapshot), and assert (a) the reopened Text field has `tokens` AND
    /// `distinct` EMPTY (no RAM rebuild — RAM is O(live tail), not O(corpus)), yet
    /// (b) the BM25 scan — text_bm25 (single), text_and (multi), filtered_search —
    /// is byte-identical f32 (to_bits) AND same result-set as an in-RAM oracle,
    /// driven entirely from the mmap with `tokens.is_empty()`.
    #[test]
    fn reopen_drives_bm25_from_segment_with_empty_tokens() {
        // Build the same tf-realistic corpus into a SEAL-then-REOPEN subject and an
        // in-RAM oracle.
        fn build(e: &Engine) {
            e.create_collection("c", schema()).unwrap();
            index_doc(
                e,
                "d0",
                &body_from(&[("alpha", 3), ("beta", 1)]),
                Some(20.0),
            );
            index_doc(
                e,
                "d1",
                &body_from(&[("alpha", 1), ("gamma", 2)]),
                Some(40.0),
            );
            index_doc(e, "d2", &body_from(&[("beta", 4)]), Some(55.0));
            index_doc(
                e,
                "d3",
                &body_from(&[("alpha", 2), ("beta", 1), ("gamma", 1)]),
                Some(70.0),
            );
            index_doc(e, "d4", &body_from(&[("gamma", 3)]), Some(90.0));
        }

        let oracle = Arc::new(Engine::new());
        build(&oracle);
        let o_single = run(&oracle, bm25_single("alpha"));
        let o_and = run(&oracle, text_and("alpha", "beta"));
        let o_filt = run(&oracle, filtered("alpha", 10.0, 80.0));

        // Subject: build, seal the whole collection, reopen into a FRESH engine.
        let subject = Arc::new(Engine::new());
        build(&subject);
        let dir = tempfile::tempdir().unwrap();
        subject
            .__seal_collection_to_segments("c", dir.path(), 1)
            .unwrap();
        let sch = subject.__collection_schema("c").unwrap();
        let reopened = Engine::__open_collection_from_segments("c", dir.path(), sch, 1).unwrap();

        // RAM-BOUNDED: the reopened Text field drives BM25 from the mmap with NO
        // in-RAM tokens/distinct rebuild.
        {
            let state = reopened.state.read().unwrap();
            let coll = state.collections.get("c").unwrap();
            let FieldIndex::Text { idx, .. } = coll.fields.get("body").unwrap() else {
                panic!("body must be a Text field");
            };
            assert!(idx.segment.is_some(), "reopened segment attached");
            assert!(
                idx.tokens.is_empty(),
                "reopen must NOT rebuild `tokens` (got {})",
                idx.tokens.len()
            );
            assert!(
                idx.distinct_is_empty(),
                "reopen must NOT rebuild `distinct` (got {})",
                idx.distinct_iter().count()
            );
            assert!(
                idx.lens.is_empty(),
                "reopen must NOT rebuild `lens` (doc_len reads the segment column)"
            );
            // Live corpus scalars were initialized from the header.
            assert_eq!(
                idx.bm25_corpus(),
                (5, 4 + 3 + 4 + 4 + 3),
                "live corpus initialized from header"
            );
        }

        // BYTE-IDENTICAL BM25 from the mmap, same sets.
        let r_single = run(&reopened, bm25_single("alpha"));
        let r_and = run(&reopened, text_and("alpha", "beta"));
        let r_filt = run(&reopened, filtered("alpha", 10.0, 80.0));
        assert_eq!(
            set_of(&o_single),
            set_of(&r_single),
            "reopen single-token set diverged"
        );
        assert_eq!(
            set_of(&o_and),
            set_of(&r_and),
            "reopen 2-token AND set diverged"
        );
        assert_eq!(
            set_of(&o_filt),
            set_of(&r_filt),
            "reopen filtered set diverged"
        );
        assert_eq!(
            scores_of(&o_single),
            scores_of(&r_single),
            "reopen single-token scores diverged"
        );
        assert_eq!(
            scores_of(&o_and),
            scores_of(&r_and),
            "reopen 2-token AND scores diverged"
        );
        assert_eq!(
            scores_of(&o_filt),
            scores_of(&r_filt),
            "reopen filtered scores diverged"
        );
    }

    /// DELETE-AFTER-SEAL BM25 — THE CRUX (Phase 2h-4): seal a Text corpus, DELETE
    /// base docs (NO re-seal), and assert the segment-ON BM25 equals an in-RAM
    /// oracle that indexed the SAME docs and physically deleted the SAME ones —
    /// byte-identical f32 scores AND result-sets for single/multi-token + the
    /// corpus-sensitive filtered case. Deleting a doc shifts doc_count/avgdl/df, so
    /// EVERY surviving score changes; the tombstone subtraction + live corpus must
    /// reproduce that shift exactly. `tombstones.len()` matches the deletes.
    #[test]
    fn delete_after_seal_bm25_matches_oracle() {
        fn build(e: &Engine) {
            e.create_collection("c", schema()).unwrap();
            // A corpus where the deleted docs carry the query terms, so removing
            // them moves df AND the corpus length factor.
            index_doc(
                e,
                "d0",
                &body_from(&[("alpha", 3), ("beta", 1)]),
                Some(20.0),
            );
            index_doc(
                e,
                "d1",
                &body_from(&[("alpha", 1), ("gamma", 4)]),
                Some(35.0),
            );
            index_doc(
                e,
                "d2",
                &body_from(&[("alpha", 2), ("beta", 2)]),
                Some(50.0),
            );
            index_doc(
                e,
                "d3",
                &body_from(&[("beta", 3), ("gamma", 1)]),
                Some(65.0),
            );
            index_doc(
                e,
                "d4",
                &body_from(&[("alpha", 1), ("beta", 1)]),
                Some(80.0),
            );
            index_doc(e, "d5", &body_from(&[("gamma", 5)]), Some(95.0));
        }
        // Delete docs that DO carry alpha/beta so df + avgdl both shift.
        let to_delete = ["d1", "d2", "d4"];

        // ORACLE: in-RAM, physically delete, NEVER sealed.
        let oracle = Arc::new(Engine::new());
        build(&oracle);
        for d in to_delete {
            oracle.delete("c", d, None).unwrap();
        }

        // SUBJECT: seal body + price, THEN delete (no re-seal).
        let subject = Arc::new(Engine::new());
        build(&subject);
        let dir = tempfile::tempdir().unwrap();
        subject
            .__seal_collection_to_segments("c", dir.path(), 1)
            .unwrap();
        for d in to_delete {
            subject.delete("c", d, None).unwrap();
        }

        // The deletes are tombstoned, NOT removed from the on-disk postings; the
        // LIVE corpus scalars were decremented (so avgdl shifts).
        {
            let state = subject.state.read().unwrap();
            let coll = state.collections.get("c").unwrap();
            let FieldIndex::Text { idx, .. } = coll.fields.get("body").unwrap() else {
                panic!("body");
            };
            assert!(idx.tokens.is_empty(), "sealed: tokens still dropped");
            assert_eq!(idx.tombstones.len(), 3, "three base deletes tombstoned");
            // 6 docs - 3 deleted = 3 live; total_doc_len = d0(4)+d3(4)+d5(5)=13.
            assert_eq!(
                idx.bm25_corpus(),
                (3, 4 + 4 + 5),
                "live corpus reflects deletes (drives avgdl)"
            );
            // The oracle and subject must agree on the live corpus EXACTLY.
            let ostate = oracle.state.read().unwrap();
            let ocoll = ostate.collections.get("c").unwrap();
            let FieldIndex::Text { idx: oidx, .. } = ocoll.fields.get("body").unwrap() else {
                panic!("oracle body");
            };
            assert_eq!(
                idx.bm25_corpus(),
                (oidx.doc_count, oidx.total_doc_len),
                "corpus must match oracle"
            );
        }

        // BYTE-IDENTICAL f32 scores AND sets across the corpus-sensitive shapes.
        // (Deleting d1/d2/d4 changes N, avgdl, and df(alpha)/df(beta) — every
        // surviving doc's score shifts, and it must match the oracle bit-for-bit.)
        let s_single = run(&subject, bm25_single("alpha"));
        let o_single = run(&oracle, bm25_single("alpha"));
        let s_beta = run(&subject, bm25_single("beta"));
        let o_beta = run(&oracle, bm25_single("beta"));
        let s_and = run(&subject, text_and("alpha", "beta"));
        let o_and = run(&oracle, text_and("alpha", "beta"));
        let s_filt = run(&subject, filtered("alpha", 10.0, 90.0));
        let o_filt = run(&oracle, filtered("alpha", 10.0, 90.0));

        assert_eq!(
            set_of(&s_single),
            set_of(&o_single),
            "alpha set leaked a deleted doc"
        );
        assert_eq!(
            scores_of(&s_single),
            scores_of(&o_single),
            "alpha scores diverged (corpus shift not reproduced)"
        );
        assert_eq!(
            set_of(&s_beta),
            set_of(&o_beta),
            "beta set leaked a deleted doc"
        );
        assert_eq!(
            scores_of(&s_beta),
            scores_of(&o_beta),
            "beta scores diverged"
        );
        assert_eq!(
            set_of(&s_and),
            set_of(&o_and),
            "AND set leaked a deleted doc"
        );
        assert_eq!(scores_of(&s_and), scores_of(&o_and), "AND scores diverged");
        assert_eq!(
            set_of(&s_filt),
            set_of(&o_filt),
            "filtered set leaked a deleted doc"
        );
        assert_eq!(
            scores_of(&s_filt),
            scores_of(&o_filt),
            "filtered scores diverged"
        );

        // Concrete spot-check: alpha now only d0 survives (d1, d2, d4 deleted).
        assert_eq!(
            set_of(&s_single),
            ["d0".to_string()].into_iter().collect::<BTreeSet<_>>(),
            "alpha must be only the surviving d0"
        );
        // beta: d0 and d3 survive (d2, d4 deleted).
        assert_eq!(
            set_of(&s_beta),
            ["d0".to_string(), "d3".to_string()]
                .into_iter()
                .collect::<BTreeSet<_>>(),
            "beta must be d0 + d3"
        );
    }

    /// RE-SEAL after delete (Phase 2h-4): once re-sealed the deletes are BAKED into
    /// the new segment (2g-A live(id) GC), the tombstone is CLEARED, and a fresh
    /// reopen excludes the deleted docs with a correct corpus.
    #[test]
    fn reseal_bakes_text_deletes_and_clears_tombstone() {
        fn build(e: &Engine) {
            e.create_collection("c", schema()).unwrap();
            index_doc(
                e,
                "d0",
                &body_from(&[("alpha", 2), ("beta", 1)]),
                Some(20.0),
            );
            index_doc(e, "d1", &body_from(&[("alpha", 3)]), Some(40.0));
            index_doc(
                e,
                "d2",
                &body_from(&[("beta", 2), ("gamma", 1)]),
                Some(60.0),
            );
            index_doc(
                e,
                "d3",
                &body_from(&[("alpha", 1), ("gamma", 2)]),
                Some(80.0),
            );
        }
        let to_delete = ["d1", "d2"];

        let oracle = Arc::new(Engine::new());
        build(&oracle);
        for d in to_delete {
            oracle.delete("c", d, None).unwrap();
        }

        let subject = Arc::new(Engine::new());
        build(&subject);
        let dir = tempfile::tempdir().unwrap();
        subject
            .__seal_collection_to_segments("c", dir.path(), 1)
            .unwrap();
        for d in to_delete {
            subject.delete("c", d, None).unwrap();
        }
        {
            let state = subject.state.read().unwrap();
            let coll = state.collections.get("c").unwrap();
            let FieldIndex::Text { idx, .. } = coll.fields.get("body").unwrap() else {
                panic!("body");
            };
            assert_eq!(idx.tombstones.len(), 2, "deletes tombstoned before re-seal");
        }

        // RE-SEAL into a new dir: deletes baked in, tombstone cleared.
        let dir2 = tempfile::tempdir().unwrap();
        subject
            .__seal_collection_to_segments("c", dir2.path(), 2)
            .unwrap();
        {
            let state = subject.state.read().unwrap();
            let coll = state.collections.get("c").unwrap();
            let FieldIndex::Text { idx, .. } = coll.fields.get("body").unwrap() else {
                panic!("body");
            };
            assert!(
                idx.tombstones.is_empty(),
                "tombstone must be CLEARED after re-seal"
            );
        }

        // Reopen from the RE-SEALED dir: deleted docs gone, corpus correct.
        let sch = subject.__collection_schema("c").unwrap();
        let reopened = Engine::__open_collection_from_segments("c", dir2.path(), sch, 2).unwrap();
        {
            let state = reopened.state.read().unwrap();
            let coll = state.collections.get("c").unwrap();
            let FieldIndex::Text { idx, .. } = coll.fields.get("body").unwrap() else {
                panic!("body");
            };
            // 4 docs - 2 deleted = 2 live; total = d0(3) + d3(3) = 6.
            assert_eq!(
                idx.bm25_corpus(),
                (2, 3 + 3),
                "re-sealed corpus excludes deletes"
            );
        }

        let s_single = run(&reopened, bm25_single("alpha"));
        let o_single = run(&oracle, bm25_single("alpha"));
        assert_eq!(
            set_of(&s_single),
            set_of(&o_single),
            "post-re-seal alpha set diverged"
        );
        assert_eq!(
            scores_of(&s_single),
            scores_of(&o_single),
            "post-re-seal alpha scores diverged"
        );
        // beta fully deleted only via d2; d0 still has beta → survives.
        assert_eq!(
            set_of(&run(&reopened, bm25_single("beta"))),
            ["d0".to_string()].into_iter().collect::<BTreeSet<_>>(),
            "beta must be only the surviving d0 after re-seal"
        );
        assert_eq!(
            uniq(&reopened, "body"),
            uniq(&oracle, "body"),
            "unique_terms diverged after re-seal"
        );
    }
}

// ---------------------------------------------------------------------------
// Dual-path diff test: segment-backed Hash (Hamming) read must be
// byte-identical to the live in-RAM read (Stage 2 Phase 2d).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod segment_hash_diff_tests {
    use super::*;
    use proptest::prelude::*;
    use std::sync::Arc;

    fn fieldspec(t: FieldType) -> FieldSpec {
        FieldSpec {
            field_type: t,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    /// A single `sig` Hash field.
    fn schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        fields.insert("sig".into(), fieldspec(FieldType::Hash));
        CreateCollectionRequest { fields }
    }

    fn index_hash(e: &Engine, eid: &str, hash: u64) {
        e.index(
            "c",
            IndexRequest {
                items: vec![crate::types::IndexItem {
                    external_id: eid.into(),
                    field: "sig".into(),
                    value: FieldValue::String(format!("{hash:016x}")),
                }],
                request_id: None,
            },
        )
        .unwrap();
    }

    fn hamming(hash: u64, max: u32) -> SearchRequest {
        SearchRequest {
            query: QueryNode::Hamming(HammingQuery {
                field: "sig".into(),
                hash: format!("{hash:016x}"),
                max_distance: max,
            }),
            limit: 100_000,
            cursor: None,
            sort: None,
            track_total: true,
            collapse: None,
        }
    }

    /// (external_id, score_bits) keyed map — order-independent, byte-exact.
    fn run(e: &Engine, hash: u64, max: u32) -> BTreeMap<String, u32> {
        e.search("c", hamming(hash, max))
            .unwrap()
            .hits
            .into_iter()
            .map(|h| (h.external_id, h.score.to_bits()))
            .collect()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]

        /// PATH A (segment OFF, brute-force over `forward`) must equal PATH B
        /// (Hash field sealed to an mmap segment, hash read via the segment) —
        /// same matching docs AND byte-identical Hamming similarity scores —
        /// over a randomized corpus and query.
        #[test]
        fn hamming_segment_matches_live(
            hashes in proptest::collection::vec(any::<u64>(), 1..60),
            q in any::<u64>(),
            max in 0u32..=64,
        ) {
            // --- PATH A: build, query (segment OFF). ---
            let e = Arc::new(Engine::new());
            e.create_collection("c", schema()).unwrap();
            for (i, h) in hashes.iter().enumerate() {
                index_hash(&e, &format!("d{i}"), *h);
            }
            let a = run(&e, q, max);

            // --- PATH B: seal `sig`, flip it ON, rerun. ---
            let dir = tempfile::tempdir().unwrap();
            let sealed = e.__seal_hash_field_to_segment("c", "sig", dir.path()).unwrap();
            prop_assert_eq!(sealed as usize, hashes.len(), "all docs sealed");
            let b = run(&e, q, max);

            prop_assert_eq!(a, b, "hamming result/scores diverged after seal");
        }
    }

    /// A doc indexed AFTER sealing (docid >= n_docs) must still match through
    /// `hash_at`'s live-tail fallback; the sealed doc is served from the mmap.
    #[test]
    fn hamming_live_tail_after_seal() {
        let e = Arc::new(Engine::new());
        e.create_collection("c", schema()).unwrap();
        index_hash(&e, "sealed", 0x0000_0000_0000_0000); // id 0
        index_hash(&e, "other", 0xFFFF_FFFF_FFFF_FFFF); // id 1

        let dir = tempfile::tempdir().unwrap();
        let n = e
            .__seal_hash_field_to_segment("c", "sig", dir.path())
            .unwrap();
        assert_eq!(n, 2);

        index_hash(&e, "tail", 0x0000_0000_0000_0003); // id 2 (2 bits set), live tail

        // Query hash 0, max distance 2: sealed (dist 0) + tail (dist 2) match,
        // `other` (dist 64) does not. Proves segment read + live-tail fallback.
        let got = run(&e, 0, 2);
        let mut want = BTreeMap::new();
        want.insert("sealed".to_string(), (1.0f32).to_bits()); // dist 0 → 64/64
        want.insert("tail".to_string(), ((64 - 2) as f32 / 64.0).to_bits());
        assert_eq!(got, want);

        // Direct check on hash_at: segment for [0,2), live tail for id 2.
        let state = e.state.read().unwrap();
        let coll = state.collections.get("c").unwrap();
        let FieldIndex::Hash(h) = coll.fields.get("sig").unwrap() else {
            panic!("sig must be a Hash field");
        };
        assert!(h.segment.is_some(), "segment attached");
        assert_eq!(h.hash_at(0), Some(0)); // segment
        assert_eq!(h.hash_at(1), Some(u64::MAX)); // segment
        assert_eq!(h.hash_at(2), Some(3)); // live tail
        assert_eq!(h.hash_at(99), None);
    }
}

// ---------------------------------------------------------------------------
// Dual-path diff test: a flat-cpu Vector field's exact kNN scan served from an
// mmap'd segment must return IDENTICAL top-k (eids + byte-identical distances)
// as the in-RAM scan (Stage 2 Phase 2d).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod segment_vector_diff_tests {
    use super::*;
    use crate::types::{VectorBackend, VectorMetric};
    use proptest::prelude::*;
    use std::sync::Arc;

    const DIM: usize = 8;

    fn vec_fieldspec(metric: VectorMetric) -> FieldSpec {
        FieldSpec {
            field_type: FieldType::Vector,
            analyzer: None,
            multi: None,
            dim: Some(DIM as u32),
            metric: Some(metric),
            // The slice is FlatCpu/exact only — HNSW is untouched.
            backend: Some(VectorBackend::FlatCpu),
            quantize: None,
        }
    }

    fn schema(metric: VectorMetric) -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        fields.insert("emb".into(), vec_fieldspec(metric));
        CreateCollectionRequest { fields }
    }

    fn index_vec(e: &Engine, eid: &str, v: &[f32]) {
        e.index(
            "c",
            IndexRequest {
                items: vec![crate::types::IndexItem {
                    external_id: eid.into(),
                    field: "emb".into(),
                    value: FieldValue::Vector(v.to_vec()),
                }],
                request_id: None,
            },
        )
        .unwrap();
    }

    fn knn(query: Vec<f32>, k: u32) -> SearchRequest {
        SearchRequest {
            query: QueryNode::Knn(crate::types::KnnQuery {
                field: "emb".into(),
                vector: query,
                k,
            }),
            limit: k,
            cursor: None,
            sort: None,
            track_total: true,
            collapse: None,
        }
    }

    /// Ordered (eid, score_bits) pairs — the kNN result is RANKED, so order is
    /// part of the contract; scores compared as exact f32 bits.
    fn run(e: &Engine, query: Vec<f32>, k: u32) -> Vec<(String, u32)> {
        e.search("c", knn(query, k))
            .unwrap()
            .hits
            .into_iter()
            .map(|h| (h.external_id, h.score.to_bits()))
            .collect()
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(120))]

        /// PATH A (segment OFF, scan reads in-RAM `FlatVecs::data`) must equal
        /// PATH B (the corpus sealed to an f32 mmap segment, scan reads each
        /// row zero-copy off the page) — IDENTICAL ranked top-k, eids and
        /// byte-identical distances (the f32 bits on disk are the same bits).
        #[test]
        fn knn_segment_matches_live(
            raw in proptest::collection::vec(
                proptest::collection::vec(-4.0f32..4.0, DIM..=DIM),
                1..40,
            ),
            qraw in proptest::collection::vec(-4.0f32..4.0, DIM..=DIM),
            k in 1u32..12,
            metric in prop::sample::select(vec![
                VectorMetric::L2, VectorMetric::Cosine, VectorMetric::Dot,
            ]),
        ) {
            // --- PATH A: build the flat-cpu corpus, run kNN (segment OFF). ---
            let e = Arc::new(Engine::new());
            e.create_collection("c", schema(metric)).unwrap();
            for (i, v) in raw.iter().enumerate() {
                index_vec(&e, &format!("d{i}"), v);
            }
            let a = run(&e, qraw.clone(), k);

            // --- PATH B: seal `emb` to a segment, flip it ON, rerun. ---
            let dir = tempfile::tempdir().unwrap();
            let sealed = e.__seal_vector_field_to_segment("c", "emb", dir.path()).unwrap();
            prop_assert_eq!(sealed as usize, raw.len(), "all vectors sealed");
            let b = run(&e, qraw, k);

            // IDENTICAL ranked top-k: same eids in the same order, byte-exact scores.
            prop_assert_eq!(a, b, "kNN top-k diverged after seal");
        }
    }

    /// A direct, planner-free check: after sealing, the flat buffer's in-RAM
    /// `data` is dropped and every row is served from the segment, yet a kNN
    /// scan returns the same ranked neighbours as before the seal.
    #[test]
    fn knn_served_from_segment_after_seal() {
        let e = Arc::new(Engine::new());
        e.create_collection("c", schema(VectorMetric::L2)).unwrap();
        // Points on a 1-D ray so the nearest order is deterministic.
        for i in 0..10usize {
            let mut v = vec![0.0f32; DIM];
            v[0] = i as f32;
            index_vec(&e, &format!("p{i}"), &v);
        }
        let mut q = vec![0.0f32; DIM];
        q[0] = 0.0;
        let before = run(&e, q.clone(), 5);

        let dir = tempfile::tempdir().unwrap();
        let n = e
            .__seal_vector_field_to_segment("c", "emb", dir.path())
            .unwrap();
        assert_eq!(n, 10);

        let after = run(&e, q, 5);
        assert_eq!(before, after, "kNN diverged when served from the segment");
        // Nearest to [0,..] is p0, then p1, ... (L2 on the ray).
        let eids: Vec<String> = after.iter().map(|(e, _)| e.clone()).collect();
        assert_eq!(eids, ["p0", "p1", "p2", "p3", "p4"]);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DuplicatedQuery, ExistsQuery};

    fn build_users_schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        fields.insert(
            "bio".into(),
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
        fields.insert(
            "email".into(),
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
        fields.insert(
            "tags".into(),
            FieldSpec {
                field_type: FieldType::Set,
                analyzer: None,
                multi: None,
                dim: None,
                metric: None,
                backend: None,
                quantize: None,
            },
        );
        fields.insert(
            "age".into(),
            FieldSpec {
                field_type: FieldType::Number,
                analyzer: None,
                multi: None,
                dim: None,
                metric: None,
                backend: None,
                quantize: None,
            },
        );
        CreateCollectionRequest { fields }
    }

    fn item(eid: &str, field: &str, value: FieldValue) -> crate::types::IndexItem {
        crate::types::IndexItem {
            external_id: eid.into(),
            field: field.into(),
            value,
        }
    }

    /// Walk every page through the returned cursors; assert the
    /// concatenation equals one exhaustive query (order included), with no
    /// duplicates or gaps — the keyset-pagination contract.
    fn walk_pages(e: &Engine, base: &SearchRequest) -> Vec<String> {
        let mut out = Vec::new();
        let mut cursor: Option<String> = None;
        for _ in 0..1000 {
            let mut req = base.clone();
            req.cursor = cursor.clone();
            let resp = e.search("users", req).unwrap();
            let n = resp.hits.len();
            out.extend(resp.hits.into_iter().map(|h| h.external_id));
            match resp.cursor {
                Some(c) => cursor = Some(c),
                None => return out,
            }
            if n == 0 {
                return out;
            }
        }
        panic!("cursor never exhausted");
    }

    #[test]
    fn unsupported_text_sort_is_rejected_instead_of_silent_score_ranking() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        e.index(
            "users",
            IndexRequest {
                items: vec![item("u1", "bio", FieldValue::String("rust".into()))],
                request_id: None,
            },
        )
        .unwrap();
        let err = e
            .search(
                "users",
                SearchRequest {
                    query: QueryNode::Match(crate::types::MatchQuery {
                        field: "bio".into(),
                        text: "rust".into(),
                        op: MatchOp::And,
                    }),
                    limit: 10,
                    cursor: None,
                    sort: Some(vec![crate::types::SortSpec {
                        field: "bio".into(),
                        order: SortOrder::Asc,
                    }]),
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap_err();
        assert!(
            matches!(
                err.downcast_ref::<StorageError>(),
                Some(StorageError::UnsupportedSort(_))
            ),
            "text sort must be a 400-class unsupported sort error: {err:?}"
        );
    }

    #[test]
    fn keyword_sort_keyset_pagination_walks_lexicographically() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        let docs = [
            ("u00", "delta", 4.0),
            ("u01", "alpha", 1.0),
            ("u02", "charlie", 3.0),
            ("u03", "alpha", 2.0),
            ("u04", "bravo", 5.0),
        ];
        let mut items = Vec::new();
        for (eid, email, age) in docs {
            items.push(item(eid, "email", FieldValue::String(email.into())));
            items.push(item(eid, "age", FieldValue::Number(age)));
        }
        e.index(
            "users",
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();

        for (order, expected) in [
            (SortOrder::Asc, vec!["u01", "u03", "u04", "u02", "u00"]),
            (SortOrder::Desc, vec!["u00", "u02", "u04", "u01", "u03"]),
        ] {
            let base = SearchRequest {
                query: QueryNode::Range(crate::types::RangeQuery {
                    field: "age".into(),
                    gt: None,
                    gte: None,
                    lt: None,
                    lte: None,
                }),
                limit: 2,
                cursor: None,
                sort: Some(vec![crate::types::SortSpec {
                    field: "email".into(),
                    order,
                }]),
                track_total: true,
                collapse: None,
            };
            let first = e.search("users", base.clone()).unwrap();
            match parse_page_cursor(first.cursor.as_deref().expect("more pages")).unwrap() {
                PageCursor::SortValuesKeyset { values, .. } => {
                    assert!(matches!(values.as_slice(), [SortValue::Keyword(_)]));
                }
                other => panic!("expected keyword sort cursor, got {other:?}"),
            }
            assert_eq!(walk_pages(&e, &base), expected, "order {order:?}");
        }
    }

    #[test]
    fn composite_keyword_number_sort_keyset_paginates_to_oracle() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        let docs = [
            ("u00", "todo", 10.0),
            ("u01", "done", 20.0),
            ("u02", "todo", 30.0),
            ("u03", "done", 15.0),
            ("u04", "blocked", 40.0),
            ("u05", "todo", 30.0),
        ];
        let mut items = Vec::new();
        for (eid, status, age) in docs {
            items.push(item(eid, "email", FieldValue::String(status.into())));
            items.push(item(eid, "age", FieldValue::Number(age)));
        }
        e.index(
            "users",
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();
        let base = SearchRequest {
            query: QueryNode::Range(crate::types::RangeQuery {
                field: "age".into(),
                gt: None,
                gte: None,
                lt: None,
                lte: None,
            }),
            limit: 2,
            cursor: None,
            sort: Some(vec![
                crate::types::SortSpec {
                    field: "email".into(),
                    order: SortOrder::Asc,
                },
                crate::types::SortSpec {
                    field: "age".into(),
                    order: SortOrder::Desc,
                },
            ]),
            track_total: true,
            collapse: None,
        };
        let first = e.search("users", base.clone()).unwrap();
        match parse_page_cursor(first.cursor.as_deref().expect("more pages")).unwrap() {
            PageCursor::SortValuesKeyset { values, .. } => {
                assert!(matches!(
                    values.as_slice(),
                    [SortValue::Keyword(_), SortValue::Number(_)]
                ));
            }
            other => panic!("expected composite sort cursor, got {other:?}"),
        }
        assert_eq!(
            walk_pages(&e, &base),
            vec!["u04", "u01", "u03", "u02", "u05", "u00"]
        );
    }

    #[test]
    fn keyword_sort_paginates_over_sealed_segment_plus_live_tail() {
        let dir = tempfile::tempdir().unwrap();
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        let mut sealed = Vec::new();
        for (eid, email, age) in [
            ("u00", "delta", 4.0),
            ("u01", "alpha", 1.0),
            ("u02", "charlie", 3.0),
        ] {
            sealed.push(item(eid, "email", FieldValue::String(email.into())));
            sealed.push(item(eid, "age", FieldValue::Number(age)));
        }
        e.index(
            "users",
            IndexRequest {
                items: sealed,
                request_id: None,
            },
        )
        .unwrap();
        e.__seal_keyword_field_to_segment("users", "email", dir.path())
            .unwrap();
        let mut tail = Vec::new();
        for (eid, email, age) in [("u03", "alpha", 2.0), ("u04", "bravo", 5.0)] {
            tail.push(item(eid, "email", FieldValue::String(email.into())));
            tail.push(item(eid, "age", FieldValue::Number(age)));
        }
        e.index(
            "users",
            IndexRequest {
                items: tail,
                request_id: None,
            },
        )
        .unwrap();

        let base = SearchRequest {
            query: QueryNode::Range(crate::types::RangeQuery {
                field: "age".into(),
                gt: None,
                gte: None,
                lt: None,
                lte: None,
            }),
            limit: 2,
            cursor: None,
            sort: Some(vec![crate::types::SortSpec {
                field: "email".into(),
                order: SortOrder::Asc,
            }]),
            track_total: true,
            collapse: None,
        };
        assert_eq!(
            walk_pages(&e, &base),
            vec!["u01", "u03", "u04", "u02", "u00"]
        );
    }

    #[test]
    fn sorted_keyset_pagination_walks_exhaustively_with_ties() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        // 97 docs, age = i % 10 → heavy duplicate sort keys exercise the
        // (value, docid) tie-break across page boundaries.
        let items: Vec<_> = (0..97)
            .flat_map(|i| {
                vec![item(
                    &format!("u{i:03}"),
                    "age",
                    FieldValue::Number((i % 10) as f64),
                )]
            })
            .collect();
        e.index(
            "users",
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();

        for order in [SortOrder::Asc, SortOrder::Desc] {
            let base = SearchRequest {
                query: QueryNode::Range(crate::types::RangeQuery {
                    field: "age".into(),
                    gt: None,
                    gte: None,
                    lt: None,
                    lte: None,
                }),
                limit: 7,
                cursor: None,
                sort: Some(vec![crate::types::SortSpec {
                    field: "age".into(),
                    order,
                }]),
                track_total: true,
                collapse: None,
            };
            // One exhaustive page as the oracle.
            let mut oracle_req = base.clone();
            oracle_req.limit = 1000;
            let oracle: Vec<String> = e
                .search("users", oracle_req)
                .unwrap()
                .hits
                .into_iter()
                .map(|h| h.external_id)
                .collect();
            assert_eq!(oracle.len(), 97);

            let paged = walk_pages(&e, &base);
            assert_eq!(paged, oracle, "order {order:?}");
        }
    }

    #[test]
    fn sorted_keyset_cursor_is_v2_and_filtered_walks_match() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        let mut items = Vec::new();
        for i in 0..60 {
            items.push(item(
                &format!("u{i:03}"),
                "age",
                FieldValue::Number(i as f64),
            ));
            items.push(item(
                &format!("u{i:03}"),
                "email",
                FieldValue::String(format!("{}@x.com", if i % 2 == 0 { "even" } else { "odd" })),
            ));
        }
        e.index(
            "users",
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();

        // Filtered (query predicate) + sorted + paged.
        let base = SearchRequest {
            query: QueryNode::Term(crate::types::TermQuery {
                field: "email".into(),
                value: FieldValue::String("even@x.com".into()),
            }),
            limit: 4,
            cursor: None,
            sort: Some(vec![crate::types::SortSpec {
                field: "age".into(),
                order: SortOrder::Desc,
            }]),
            track_total: true,
            collapse: None,
        };
        let first = e.search("users", base.clone()).unwrap();
        // The first page of a sorted query hands out a v2 keyset cursor.
        let cursor = first.cursor.clone().expect("more pages");
        match parse_page_cursor(&cursor).expect("parseable") {
            PageCursor::SortKeyset { .. } => {}
            _ => panic!("expected a sort keyset cursor"),
        }

        let paged = walk_pages(&e, &base);
        let expected: Vec<String> = (0..60)
            .rev()
            .filter(|i| i % 2 == 0)
            .map(|i| format!("u{i:03}"))
            .collect();
        assert_eq!(paged, expected);
    }

    #[test]
    fn score_keyset_pagination_matches_full_ranking() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        let items: Vec<_> = (0..45)
            .map(|i| {
                item(
                    &format!("u{i:03}"),
                    "bio",
                    FieldValue::String(format!(
                        "engineer {}",
                        if i % 3 == 0 { "rust rust" } else { "rust" }
                    )),
                )
            })
            .collect();
        e.index(
            "users",
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();

        let base = SearchRequest {
            query: QueryNode::Match(crate::types::MatchQuery {
                field: "bio".into(),
                text: "rust".into(),
                op: MatchOp::And,
            }),
            limit: 6,
            cursor: None,
            sort: None,
            track_total: true,
            collapse: None,
        };
        let mut oracle_req = base.clone();
        oracle_req.limit = 1000;
        let oracle: Vec<String> = e
            .search("users", oracle_req)
            .unwrap()
            .hits
            .into_iter()
            .map(|h| h.external_id)
            .collect();
        assert_eq!(oracle.len(), 45);

        let first = e.search("users", base.clone()).unwrap();
        match parse_page_cursor(&first.cursor.clone().unwrap()).unwrap() {
            PageCursor::ScoreKeyset { .. } => {}
            _ => panic!("expected a score keyset cursor"),
        }
        let paged = walk_pages(&e, &base);
        assert_eq!(paged, oracle);
    }

    #[test]
    fn legacy_offset_cursor_still_pages() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        let items: Vec<_> = (0..30)
            .map(|i| item(&format!("u{i:03}"), "age", FieldValue::Number(i as f64)))
            .collect();
        e.index(
            "users",
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();
        let req = SearchRequest {
            query: QueryNode::Range(crate::types::RangeQuery {
                field: "age".into(),
                gt: None,
                gte: Some(0.0),
                lt: None,
                lte: None,
            }),
            limit: 10,
            cursor: Some(make_cursor(25)),
            sort: None,
            track_total: true,
            collapse: None,
        };
        let resp = e.search("users", req).unwrap();
        assert_eq!(resp.hits.len(), 5);
        assert_eq!(resp.total, 30);
        assert!(resp.cursor.is_none());
    }

    #[test]
    fn sorted_keyset_pagination_over_sealed_segment_plus_tail() {
        let dir = tempfile::tempdir().unwrap();
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        // 50 sealed docs with duplicate keys, then a live tail of 13 more —
        // the keyset walk must seek correctly across BOTH sources.
        let items: Vec<_> = (0..50)
            .map(|i| {
                item(
                    &format!("u{i:03}"),
                    "age",
                    FieldValue::Number((i % 7) as f64),
                )
            })
            .collect();
        e.index(
            "users",
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();
        e.__seal_number_field_to_segment("users", "age", dir.path())
            .unwrap();
        let tail: Vec<_> = (50..63)
            .map(|i| {
                item(
                    &format!("u{i:03}"),
                    "age",
                    FieldValue::Number((i % 7) as f64),
                )
            })
            .collect();
        e.index(
            "users",
            IndexRequest {
                items: tail,
                request_id: None,
            },
        )
        .unwrap();

        for order in [SortOrder::Asc, SortOrder::Desc] {
            let base = SearchRequest {
                query: QueryNode::Range(crate::types::RangeQuery {
                    field: "age".into(),
                    gt: None,
                    gte: None,
                    lt: None,
                    lte: None,
                }),
                limit: 5,
                cursor: None,
                sort: Some(vec![crate::types::SortSpec {
                    field: "age".into(),
                    order,
                }]),
                track_total: true,
                collapse: None,
            };
            let mut oracle_req = base.clone();
            oracle_req.limit = 1000;
            let oracle: Vec<String> = e
                .search("users", oracle_req)
                .unwrap()
                .hits
                .into_iter()
                .map(|h| h.external_id)
                .collect();
            assert_eq!(oracle.len(), 63);
            let paged = walk_pages(&e, &base);
            assert_eq!(paged, oracle, "order {order:?}");
        }
    }

    /// Deep-pagination latency proof (run explicitly, release):
    /// `cargo test -p lumen --release --lib deep_pagination_depth_invariance -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn deep_pagination_depth_invariance() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        let n = 100_000;
        for chunk in (0..n).collect::<Vec<_>>().chunks(5000) {
            let items: Vec<_> = chunk
                .iter()
                .map(|i| item(&format!("u{i:06}"), "age", FieldValue::Number(*i as f64)))
                .collect();
            e.index(
                "users",
                IndexRequest {
                    items,
                    request_id: None,
                },
            )
            .unwrap();
        }
        let base = SearchRequest {
            query: QueryNode::Range(crate::types::RangeQuery {
                field: "age".into(),
                gt: None,
                gte: None,
                lt: None,
                lte: None,
            }),
            limit: 10,
            cursor: None,
            sort: Some(vec![crate::types::SortSpec {
                field: "age".into(),
                order: SortOrder::Asc,
            }]),
            track_total: false,
            collapse: None,
        };

        // Page 1, then jump a keyset cursor to depth ~50_000 and time a page.
        let t0 = std::time::Instant::now();
        let first = e.search("users", base.clone()).unwrap();
        let first_us = t0.elapsed().as_micros();
        assert_eq!(first.hits.len(), 10);

        let deep_cursor = make_sort_cursor(
            SortableF64::new(50_000.0).unwrap().bits(),
            0, // before any docid at that key
        );
        let mut deep_req = base.clone();
        deep_req.cursor = Some(deep_cursor);
        let t1 = std::time::Instant::now();
        let deep = e.search("users", deep_req).unwrap();
        let deep_us = t1.elapsed().as_micros();
        assert_eq!(deep.hits.len(), 10);
        assert_eq!(deep.hits[0].external_id, "u050000");

        // Legacy offset to the same depth for contrast.
        let mut offset_req = base.clone();
        offset_req.cursor = Some(make_cursor(50_000));
        let t2 = std::time::Instant::now();
        let via_offset = e.search("users", offset_req).unwrap();
        let offset_us = t2.elapsed().as_micros();

        eprintln!(
            "page#1 {first_us}us | keyset@50k {deep_us}us | offset@50k {offset_us}us (hits {})",
            via_offset.hits.len()
        );
        // The keyset deep page must be the same order of magnitude as page 1 —
        // depth invariance. (Loose 20x bound to survive CI jitter.)
        assert!(
            deep_us < first_us.max(1) * 20,
            "keyset deep page degraded: first={first_us}us deep={deep_us}us"
        );
    }

    #[test]
    fn duplicates_side_index_tracks_inserts_and_deletes() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        e.index(
            "users",
            IndexRequest {
                items: vec![
                    item("u1", "email", FieldValue::String("a@x.com".into())),
                    item("u2", "email", FieldValue::String("a@x.com".into())),
                    item("u3", "email", FieldValue::String("b@y.com".into())),
                    item("u4", "email", FieldValue::String("b@y.com".into())),
                    item("u5", "email", FieldValue::String("solo@z.com".into())),
                ],
                request_id: None,
            },
        )
        .unwrap();
        let groups = |min: u32| {
            e.duplicates(
                "users",
                DuplicatesRequest {
                    field: "email".into(),
                    min_group_size: min,
                    limit: 10,
                    offset: 0,
                },
            )
            .unwrap()
            .groups
        };
        let g = groups(2);
        assert_eq!(g.len(), 2);
        // Delete one of the `a@x.com` pair — the group must drop out, and the
        // side-index must not leak a stale candidate.
        e.delete("users", "u2", None).unwrap();
        let g = groups(2);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].value, serde_json::Value::String("b@y.com".into()));
        // Re-index the deleted doc back into the pair — group returns.
        e.index(
            "users",
            IndexRequest {
                items: vec![item("u2", "email", FieldValue::String("a@x.com".into()))],
                request_id: None,
            },
        )
        .unwrap();
        assert_eq!(groups(2).len(), 2);
    }

    #[test]
    fn number_range_stats_matches_walk_on_all_bound_shapes() {
        use std::ops::Bound;
        let mut idx = NumberIndex::default();
        // Values 0.0, 1.0, ..., 99.0; value k carries k+1 docs so df != distinct.
        let mut id = 0u32;
        for k in 0..100u32 {
            let key = SortableF64::new(k as f64).unwrap();
            for _ in 0..=k {
                idx.values.entry(key).or_default().insert(id);
                id += 1;
            }
        }
        let stats = NumberRangeStats::build(&idx.values);
        let s = |x: f64| SortableF64::new(x).unwrap();
        let cases: Vec<(Bound<SortableF64>, Bound<SortableF64>)> = vec![
            (Bound::Unbounded, Bound::Unbounded),
            (Bound::Included(s(10.0)), Bound::Excluded(s(20.0))),
            (Bound::Excluded(s(10.0)), Bound::Included(s(20.0))),
            (Bound::Included(s(10.5)), Bound::Excluded(s(10.6))), // empty window
            (Bound::Unbounded, Bound::Excluded(s(0.0))),          // before first
            (Bound::Excluded(s(99.0)), Bound::Unbounded),         // after last
            (Bound::Included(s(99.0)), Bound::Included(s(99.0))), // single key
        ];
        for (lo, hi) in cases {
            let walk_distinct = idx.values.range((lo, hi)).count() as u64;
            let walk_df: u64 = idx.values.range((lo, hi)).map(|(_, s)| s.len()).sum();
            assert_eq!(
                stats.range(lo, hi),
                (walk_distinct, walk_df),
                "bounds {lo:?}..{hi:?}"
            );
            // The public estimate entry points agree with the walk too.
            assert_eq!(idx.range_df(lo, hi), walk_df, "range_df {lo:?}..{hi:?}");
            assert_eq!(
                idx.range_distinct_count(lo, hi),
                walk_distinct,
                "distinct {lo:?}..{hi:?}"
            );
        }
    }

    #[test]
    fn number_range_stats_invalidated_by_cache_clear() {
        use std::ops::Bound;
        let mut idx = NumberIndex::default();
        for k in 0..10u32 {
            let key = SortableF64::new(k as f64).unwrap();
            idx.values.entry(key).or_default().insert(k);
        }
        let all = (Bound::Unbounded, Bound::Unbounded);
        idx.build_range_stats();
        assert_eq!(idx.range_df(all.0, all.1), 10);
        // Mutate the tree the way the write path does, then clear caches —
        // the next estimate must see the new value, not the stale snapshot.
        idx.values
            .entry(SortableF64::new(100.0).unwrap())
            .or_default()
            .insert(10);
        idx.clear_keyword_range_cache();
        assert_eq!(idx.range_df(all.0, all.1), 11);
        assert_eq!(idx.range_distinct_count(all.0, all.1), 11);
    }

    #[test]
    fn create_collection_returns_version_one() {
        let e = Engine::new();
        let r = e.create_collection("users", build_users_schema()).unwrap();
        assert_eq!(r.collection_id, "users");
        assert_eq!(r.version, 1);
        assert_eq!(r.fields_count, 4);
    }

    #[test]
    fn index_and_term_search_keyword() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        e.index(
            "users",
            IndexRequest {
                items: vec![
                    item("u1", "email", FieldValue::String("a@x.com".into())),
                    item("u2", "email", FieldValue::String("b@y.com".into())),
                    item("u3", "email", FieldValue::String("a@x.com".into())),
                ],
                request_id: None,
            },
        )
        .unwrap();
        let resp = e
            .search(
                "users",
                SearchRequest {
                    query: QueryNode::Term(TermQuery {
                        field: "email".into(),
                        value: FieldValue::String("a@x.com".into()),
                    }),
                    limit: 10,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap();
        assert_eq!(resp.total, 2);
        let eids: Vec<_> = resp.hits.iter().map(|h| h.external_id.as_str()).collect();
        assert_eq!(eids, vec!["u1", "u3"]);
    }

    #[test]
    fn match_query_finds_text() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        e.index(
            "users",
            IndexRequest {
                items: vec![
                    item(
                        "u1",
                        "bio",
                        FieldValue::String("senior engineer in Taipei".into()),
                    ),
                    item(
                        "u2",
                        "bio",
                        FieldValue::String("designer in Hsinchu".into()),
                    ),
                    item("u3", "bio", FieldValue::String("engineer in Tokyo".into())),
                ],
                request_id: None,
            },
        )
        .unwrap();
        let resp = e
            .search(
                "users",
                SearchRequest {
                    query: QueryNode::Match(MatchQuery {
                        field: "bio".into(),
                        text: "engineer taipei".into(),
                        op: MatchOp::And,
                    }),
                    limit: 10,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap();
        assert_eq!(resp.total, 1);
        assert_eq!(resp.hits[0].external_id, "u1");
    }

    #[test]
    fn search_result_cache_is_cleared_by_index_write() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        e.index(
            "users",
            IndexRequest {
                items: vec![item(
                    "u1",
                    "bio",
                    FieldValue::String("engineer in Taipei".into()),
                )],
                request_id: None,
            },
        )
        .unwrap();

        let req = SearchRequest {
            query: QueryNode::Match(MatchQuery {
                field: "bio".into(),
                text: "engineer".into(),
                op: MatchOp::Or,
            }),
            limit: 10,
            cursor: None,
            sort: None,
            track_total: true,
            collapse: None,
        };
        let first = e.search("users", req.clone()).unwrap();
        assert_eq!(first.total, 1);
        {
            let state = e.state.read().unwrap();
            let coll = state.collections.get("users").unwrap();
            assert_eq!(coll.search_cache.read().unwrap().len(), 1);
        }

        e.index(
            "users",
            IndexRequest {
                items: vec![item(
                    "u2",
                    "bio",
                    FieldValue::String("engineer in Tokyo".into()),
                )],
                request_id: None,
            },
        )
        .unwrap();
        {
            let state = e.state.read().unwrap();
            let coll = state.collections.get("users").unwrap();
            assert!(coll.search_cache.read().unwrap().is_empty());
        }

        let second = e.search("users", req).unwrap();
        assert_eq!(second.total, 2);
    }

    #[test]
    fn duplicate_field_in_one_index_request_is_replacement() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        e.index(
            "users",
            IndexRequest {
                items: vec![
                    item("u1", "bio", FieldValue::String("old token".into())),
                    item("u1", "bio", FieldValue::String("new token".into())),
                ],
                request_id: None,
            },
        )
        .unwrap();

        let old = e
            .search(
                "users",
                SearchRequest {
                    query: QueryNode::Match(MatchQuery {
                        field: "bio".into(),
                        text: "old".into(),
                        op: MatchOp::Or,
                    }),
                    limit: 10,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap();
        let new = e
            .search(
                "users",
                SearchRequest {
                    query: QueryNode::Match(MatchQuery {
                        field: "bio".into(),
                        text: "new".into(),
                        op: MatchOp::Or,
                    }),
                    limit: 10,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap();

        assert_eq!(old.total, 0);
        assert_eq!(new.total, 1);
        assert_eq!(new.hits[0].external_id, "u1");
    }

    #[test]
    fn range_query_on_number() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        let items = (1..=5)
            .map(|i| item(&format!("u{i}"), "age", FieldValue::Number(i as f64 * 10.0)))
            .collect();
        e.index(
            "users",
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();
        let resp = e
            .search(
                "users",
                SearchRequest {
                    query: QueryNode::Range(RangeQuery {
                        field: "age".into(),
                        gte: Some(20.0),
                        lt: Some(50.0),
                        gt: None,
                        lte: None,
                    }),
                    limit: 10,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap();
        assert_eq!(resp.total, 3);
    }

    #[test]
    fn and_combines_text_term_range() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        e.index(
            "users",
            IndexRequest {
                items: vec![
                    item("u1", "bio", FieldValue::String("rust engineer".into())),
                    item(
                        "u1",
                        "tags",
                        FieldValue::StringList(vec!["rust".into(), "db".into()]),
                    ),
                    item("u1", "age", FieldValue::Number(30.0)),
                    item("u2", "bio", FieldValue::String("rust engineer".into())),
                    item("u2", "tags", FieldValue::StringList(vec!["go".into()])),
                    item("u2", "age", FieldValue::Number(30.0)),
                ],
                request_id: None,
            },
        )
        .unwrap();
        let q = QueryNode::And(vec![
            QueryNode::Match(MatchQuery {
                field: "bio".into(),
                text: "rust".into(),
                op: MatchOp::And,
            }),
            QueryNode::Term(TermQuery {
                field: "tags".into(),
                value: FieldValue::String("rust".into()),
            }),
            QueryNode::Range(RangeQuery {
                field: "age".into(),
                gte: Some(25.0),
                lt: Some(40.0),
                gt: None,
                lte: None,
            }),
        ]);
        let resp = e
            .search(
                "users",
                SearchRequest {
                    query: q,
                    limit: 10,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap();
        assert_eq!(resp.total, 1);
        assert_eq!(resp.hits[0].external_id, "u1");
    }

    #[test]
    fn duplicates_keyword() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        e.index(
            "users",
            IndexRequest {
                items: vec![
                    item("u1", "email", FieldValue::String("a@x.com".into())),
                    item("u2", "email", FieldValue::String("a@x.com".into())),
                    item("u3", "email", FieldValue::String("a@x.com".into())),
                    item("u4", "email", FieldValue::String("b@y.com".into())),
                    item("u5", "email", FieldValue::String("b@y.com".into())),
                    item("u6", "email", FieldValue::String("c@z.com".into())),
                ],
                request_id: None,
            },
        )
        .unwrap();
        let resp = e
            .duplicates(
                "users",
                DuplicatesRequest {
                    field: "email".into(),
                    min_group_size: 2,
                    limit: 100,
                    offset: 0,
                },
            )
            .unwrap();
        assert_eq!(resp.groups.len(), 2);
        // Largest first.
        assert_eq!(resp.groups[0].external_ids.len(), 3);
        assert_eq!(resp.groups[1].external_ids.len(), 2);
    }

    #[test]
    fn duplicates_text_rejected() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        let err = e
            .duplicates(
                "users",
                DuplicatesRequest {
                    field: "bio".into(),
                    min_group_size: 2,
                    limit: 10,
                    offset: 0,
                },
            )
            .unwrap_err();
        assert!(err.to_string().contains("duplicates not supported on text"));
    }

    // ----- Exists / Duplicated query primitives (DataTable composite search) -----

    fn search_ids(e: &Engine, coll: &str, query: QueryNode) -> (u64, Vec<String>) {
        let resp = e
            .search(
                coll,
                SearchRequest {
                    query,
                    limit: 100,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap();
        let mut ids: Vec<String> = resp.hits.iter().map(|h| h.external_id.clone()).collect();
        ids.sort();
        (resp.total, ids)
    }

    #[test]
    fn exists_filters_missing_field() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        e.index(
            "users",
            IndexRequest {
                items: vec![
                    item("u1", "email", FieldValue::String("a@x.com".into())),
                    item("u1", "age", FieldValue::Number(30.0)),
                    // u2 carries no email — only age. Exists("email") must skip it.
                    item("u2", "age", FieldValue::Number(40.0)),
                    item("u3", "email", FieldValue::String("c@z.com".into())),
                    // u4 multi-valued set; Exists must see it via element_postings.
                    item(
                        "u4",
                        "tags",
                        FieldValue::StringList(vec!["a".into(), "b".into()]),
                    ),
                ],
                request_id: None,
            },
        )
        .unwrap();

        // Keyword field: only docs that actually hold an email value.
        let (total, ids) = search_ids(
            &e,
            "users",
            QueryNode::Exists(ExistsQuery {
                field: "email".into(),
            }),
        );
        assert_eq!(total, 2);
        assert_eq!(ids, vec!["u1", "u3"]);

        // Set field: presence via element postings.
        let (total, ids) = search_ids(
            &e,
            "users",
            QueryNode::Exists(ExistsQuery {
                field: "tags".into(),
            }),
        );
        assert_eq!(total, 1);
        assert_eq!(ids, vec!["u4"]);

        // Number field.
        let (total, ids) = search_ids(
            &e,
            "users",
            QueryNode::Exists(ExistsQuery {
                field: "age".into(),
            }),
        );
        assert_eq!(total, 2);
        assert_eq!(ids, vec!["u1", "u2"]);
    }

    #[test]
    fn exists_composes_with_boolean() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        e.index(
            "users",
            IndexRequest {
                items: vec![
                    item("u1", "email", FieldValue::String("a@x.com".into())),
                    item("u1", "age", FieldValue::Number(30.0)),
                    item("u2", "email", FieldValue::String("b@y.com".into())),
                    item("u2", "age", FieldValue::Number(50.0)),
                    item("u3", "age", FieldValue::Number(30.0)), // no email
                ],
                request_id: None,
            },
        )
        .unwrap();
        // has-email AND age in [25,40): u1 only (u2 too old, u3 no email).
        let q = QueryNode::And(vec![
            QueryNode::Exists(ExistsQuery {
                field: "email".into(),
            }),
            QueryNode::Range(RangeQuery {
                field: "age".into(),
                gte: Some(25.0),
                lt: Some(40.0),
                gt: None,
                lte: None,
            }),
        ]);
        let (total, ids) = search_ids(&e, "users", q);
        assert_eq!(total, 1);
        assert_eq!(ids, vec!["u1"]);

        // Inverse: missing email (NOT Exists) → u3.
        let q = QueryNode::Not(Box::new(QueryNode::Exists(ExistsQuery {
            field: "email".into(),
        })));
        let (total, ids) = search_ids(&e, "users", q);
        assert_eq!(total, 1);
        assert_eq!(ids, vec!["u3"]);
    }

    #[test]
    fn duplicated_as_query_leaf() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        e.index(
            "users",
            IndexRequest {
                items: vec![
                    item("u1", "email", FieldValue::String("a@x.com".into())),
                    item("u2", "email", FieldValue::String("a@x.com".into())),
                    item("u3", "email", FieldValue::String("a@x.com".into())),
                    item("u4", "email", FieldValue::String("b@y.com".into())),
                    item("u5", "email", FieldValue::String("b@y.com".into())),
                    item("u6", "email", FieldValue::String("c@z.com".into())), // unique
                ],
                request_id: None,
            },
        )
        .unwrap();

        // min_group_size defaults to >=2: every doc whose email collides.
        let (total, ids) = search_ids(
            &e,
            "users",
            QueryNode::Duplicated(DuplicatedQuery {
                field: "email".into(),
                min_group_size: 2,
            }),
        );
        assert_eq!(total, 5);
        assert_eq!(ids, vec!["u1", "u2", "u3", "u4", "u5"]);

        // Raise the threshold: only the 3-way group survives.
        let (total, ids) = search_ids(
            &e,
            "users",
            QueryNode::Duplicated(DuplicatedQuery {
                field: "email".into(),
                min_group_size: 3,
            }),
        );
        assert_eq!(total, 3);
        assert_eq!(ids, vec!["u1", "u2", "u3"]);
    }

    #[test]
    fn duplicated_composes_with_boolean() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        e.index(
            "users",
            IndexRequest {
                items: vec![
                    item("u1", "email", FieldValue::String("a@x.com".into())),
                    item("u1", "age", FieldValue::Number(30.0)),
                    item("u2", "email", FieldValue::String("a@x.com".into())),
                    item("u2", "age", FieldValue::Number(60.0)),
                    item("u3", "email", FieldValue::String("a@x.com".into())),
                    item("u3", "age", FieldValue::Number(35.0)),
                    item("u4", "email", FieldValue::String("u@u.com".into())), // unique email
                    item("u4", "age", FieldValue::Number(30.0)),
                ],
                request_id: None,
            },
        )
        .unwrap();
        // duplicate-email AND age<40: u1, u3 (u2 too old, u4 not a duplicate).
        let q = QueryNode::And(vec![
            QueryNode::Duplicated(DuplicatedQuery {
                field: "email".into(),
                min_group_size: 2,
            }),
            QueryNode::Range(RangeQuery {
                field: "age".into(),
                gte: None,
                lt: Some(40.0),
                gt: None,
                lte: None,
            }),
        ]);
        let (total, ids) = search_ids(&e, "users", q);
        assert_eq!(total, 2);
        assert_eq!(ids, vec!["u1", "u3"]);
    }

    #[test]
    fn duplicated_min_group_size_floor_is_two() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        e.index(
            "users",
            IndexRequest {
                items: vec![
                    item("u1", "email", FieldValue::String("a@x.com".into())),
                    item("u2", "email", FieldValue::String("a@x.com".into())),
                    item("u3", "email", FieldValue::String("solo@x.com".into())),
                ],
                request_id: None,
            },
        )
        .unwrap();
        // min_group_size 0/1 would make every doc a "duplicate"; the leaf floors it
        // at 2 so a singleton never matches.
        let (total, ids) = search_ids(
            &e,
            "users",
            QueryNode::Duplicated(DuplicatedQuery {
                field: "email".into(),
                min_group_size: 0,
            }),
        );
        assert_eq!(total, 2);
        assert_eq!(ids, vec!["u1", "u2"]);
    }

    #[test]
    fn exists_duplicated_segment_paths_equal_tail_paths() {
        // Guards the eval_field_doc_union asymmetry: segment OFF answers from the
        // in-RAM map (+ dup_values candidates for min>=2), segment ON from the
        // segment-aware live_* accessors. Seal must not change any answer, a
        // checkpoint reopen must agree, and a post-seal delete must obey the
        // group-size semantics on the sealed path (a 2-group losing a member
        // drops BOTH docs from `duplicated`).
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        e.index(
            "users",
            IndexRequest {
                items: vec![
                    item("u1", "email", FieldValue::String("a@x.com".into())),
                    item("u2", "email", FieldValue::String("a@x.com".into())),
                    item("u3", "email", FieldValue::String("a@x.com".into())),
                    item("u4", "email", FieldValue::String("b@y.com".into())),
                    item("u5", "email", FieldValue::String("b@y.com".into())),
                    item("u6", "email", FieldValue::String("solo@z.com".into())),
                    item("u1", "age", FieldValue::Number(30.0)),
                    item("u2", "age", FieldValue::Number(30.0)),
                    item(
                        "u7",
                        "tags",
                        FieldValue::StringList(vec!["x".into(), "y".into()]),
                    ),
                    item("u8", "tags", FieldValue::StringList(vec!["x".into()])),
                ],
                request_id: None,
            },
        )
        .unwrap();

        let queries: Vec<(&str, QueryNode)> = vec![
            (
                "exists email",
                QueryNode::Exists(ExistsQuery {
                    field: "email".into(),
                }),
            ),
            (
                "exists age",
                QueryNode::Exists(ExistsQuery {
                    field: "age".into(),
                }),
            ),
            (
                "exists tags",
                QueryNode::Exists(ExistsQuery {
                    field: "tags".into(),
                }),
            ),
            (
                "dup email >=2",
                QueryNode::Duplicated(DuplicatedQuery {
                    field: "email".into(),
                    min_group_size: 2,
                }),
            ),
            (
                "dup email >=3",
                QueryNode::Duplicated(DuplicatedQuery {
                    field: "email".into(),
                    min_group_size: 3,
                }),
            ),
            (
                "dup age >=2",
                QueryNode::Duplicated(DuplicatedQuery {
                    field: "age".into(),
                    min_group_size: 2,
                }),
            ),
            (
                "dup tags >=2",
                QueryNode::Duplicated(DuplicatedQuery {
                    field: "tags".into(),
                    min_group_size: 2,
                }),
            ),
        ];
        let tail: Vec<_> = queries
            .iter()
            .map(|(_, q)| search_ids(&e, "users", q.clone()))
            .collect();

        // Seal in place → the same queries now answer off the segment path.
        let dir = tempfile::tempdir().unwrap();
        e.flush_to_segments(dir.path(), 1).unwrap();
        for ((label, q), want) in queries.iter().zip(&tail) {
            let got = search_ids(&e, "users", q.clone());
            assert_eq!(&got, want, "sealed path diverged from tail path: {label}");
        }

        // Checkpoint reopen must agree too.
        let reopened = Engine::new();
        reopened.reopen_from_segment_dir(dir.path()).unwrap();
        for ((label, q), want) in queries.iter().zip(&tail) {
            let got = search_ids(&reopened, "users", q.clone());
            assert_eq!(&got, want, "reopened path diverged from tail path: {label}");
        }

        // Post-seal delete: u5 leaves → b@y.com group shrinks 2→1, so u4 must
        // ALSO leave `duplicated`; exists drops u5 only.
        e.delete("users", "u5", None).unwrap();
        let (_, ids) = search_ids(
            &e,
            "users",
            QueryNode::Duplicated(DuplicatedQuery {
                field: "email".into(),
                min_group_size: 2,
            }),
        );
        assert_eq!(
            ids,
            vec!["u1", "u2", "u3"],
            "2-group survivor must exit duplicated"
        );
        let (_, ids) = search_ids(
            &e,
            "users",
            QueryNode::Exists(ExistsQuery {
                field: "email".into(),
            }),
        );
        assert_eq!(
            ids,
            vec!["u1", "u2", "u3", "u4", "u6"],
            "exists must drop only the deleted doc"
        );
    }

    #[test]
    fn exists_on_text_field_rejected() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        let err = e
            .search(
                "users",
                SearchRequest {
                    query: QueryNode::Exists(ExistsQuery {
                        field: "bio".into(),
                    }),
                    limit: 10,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("text"), "unexpected error: {msg}");
    }

    #[test]
    fn reindex_replaces_field_value() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        e.index(
            "users",
            IndexRequest {
                items: vec![item("u1", "email", FieldValue::String("old@x.com".into()))],
                request_id: None,
            },
        )
        .unwrap();
        e.index(
            "users",
            IndexRequest {
                items: vec![item("u1", "email", FieldValue::String("new@x.com".into()))],
                request_id: None,
            },
        )
        .unwrap();
        // Old value gone, new value present.
        let r_old = e
            .search(
                "users",
                SearchRequest {
                    query: QueryNode::Term(TermQuery {
                        field: "email".into(),
                        value: FieldValue::String("old@x.com".into()),
                    }),
                    limit: 10,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap();
        assert_eq!(r_old.total, 0);
        let r_new = e
            .search(
                "users",
                SearchRequest {
                    query: QueryNode::Term(TermQuery {
                        field: "email".into(),
                        value: FieldValue::String("new@x.com".into()),
                    }),
                    limit: 10,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap();
        assert_eq!(r_new.total, 1);
    }

    #[test]
    fn delete_external_id_removes_all_fields() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        e.index(
            "users",
            IndexRequest {
                items: vec![
                    item("u1", "email", FieldValue::String("a@x.com".into())),
                    item("u1", "bio", FieldValue::String("rust engineer".into())),
                ],
                request_id: None,
            },
        )
        .unwrap();
        e.delete("users", "u1", None).unwrap();
        let r = e
            .search(
                "users",
                SearchRequest {
                    query: QueryNode::Term(TermQuery {
                        field: "email".into(),
                        value: FieldValue::String("a@x.com".into()),
                    }),
                    limit: 10,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap();
        assert_eq!(r.total, 0);
    }

    #[test]
    fn idempotency_skips_duplicate_request_id() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        let req = IndexRequest {
            items: vec![item("u1", "email", FieldValue::String("a@x.com".into()))],
            request_id: Some("req-1".into()),
        };
        e.index("users", req.clone()).unwrap();
        let r = e.index("users", req).unwrap();
        assert_eq!(r.indexed, 0);
    }

    #[test]
    fn sortable_f64_round_trip_and_order() {
        let xs = [
            -f64::INFINITY,
            -1e10,
            -1.0,
            -0.0,
            0.0,
            1.0,
            1e10,
            f64::INFINITY,
        ];
        let mut keys: Vec<SortableF64> = xs.iter().map(|x| SortableF64::new(*x).unwrap()).collect();
        let original = keys.clone();
        keys.sort();
        assert_eq!(keys, original);
        for x in xs {
            assert_eq!(SortableF64::new(x).unwrap().to_f64(), x);
        }
    }

    #[test]
    fn sortable_f64_rejects_nan() {
        assert!(SortableF64::new(f64::NAN).is_err());
    }

    #[test]
    fn cursor_round_trip() {
        let c = make_cursor(42);
        assert!(matches!(
            parse_page_cursor(&c),
            Some(PageCursor::Offset(42))
        ));
        let c = make_sort_cursor(0x8000_0000_0000_0000, 7);
        assert!(matches!(
            parse_page_cursor(&c),
            Some(PageCursor::SortKeyset {
                bits: 0x8000_0000_0000_0000,
                docid: 7
            })
        ));
        let c = make_score_cursor(1.5, "u042");
        match parse_page_cursor(&c) {
            Some(PageCursor::ScoreKeyset { score_bits, eid }) => {
                assert_eq!(f32::from_bits(score_bits), 1.5);
                assert_eq!(eid, "u042");
            }
            _ => panic!("expected score keyset"),
        }
    }

    // -----------------------------------------------------------------
    // Ordering contract (Contract 1 of the coverage goal).
    //
    // These assert the *order* and relative magnitude of scores, not
    // just membership — so a mutated BM25 / score-combination operator
    // (e.g. `+`→`-`, `/`→`*`, `cmp(a,b)`→`cmp(b,a)`) makes at least one
    // of them fail. Membership-only tests above cannot catch those.
    // -----------------------------------------------------------------

    fn text_only_schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        fields.insert(
            "body".into(),
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
        CreateCollectionRequest { fields }
    }

    fn search_match(e: &Engine, coll: &str, text: &str, op: MatchOp) -> Vec<SearchHit> {
        e.search(
            coll,
            SearchRequest {
                query: QueryNode::Match(MatchQuery {
                    field: "body".into(),
                    text: text.into(),
                    op,
                }),
                limit: 50,
                cursor: None,
                sort: None,
                track_total: true,
                collapse: None,
            },
        )
        .unwrap()
        .hits
    }

    fn score_of(hits: &[SearchHit], eid: &str) -> f32 {
        hits.iter()
            .find(|h| h.external_id == eid)
            .unwrap_or_else(|| panic!("eid {eid} not in hits"))
            .score
    }

    #[test]
    fn bm25_higher_tf_scores_strictly_higher() {
        let e = Engine::new();
        e.create_collection("c", text_only_schema()).unwrap();
        e.index(
            "c",
            IndexRequest {
                items: vec![
                    // identical doc length, differing only in TF of "rust"
                    item(
                        "hi",
                        "body",
                        FieldValue::String("rust rust rust pad pad".into()),
                    ),
                    item(
                        "lo",
                        "body",
                        FieldValue::String("rust pad pad pad pad".into()),
                    ),
                ],
                request_id: None,
            },
        )
        .unwrap();
        let hits = search_match(&e, "c", "rust", MatchOp::Or);
        // hi has TF=3, lo has TF=1 → hi must score strictly higher AND
        // sort first. Kills `tf` numerator / denominator operator flips.
        assert_eq!(hits[0].external_id, "hi");
        assert!(
            score_of(&hits, "hi") > score_of(&hits, "lo"),
            "higher TF must score higher: {hits:?}"
        );
    }

    #[test]
    fn bm25_shorter_doc_scores_higher_for_same_tf() {
        let e = Engine::new();
        e.create_collection("c", text_only_schema()).unwrap();
        e.index(
            "c",
            IndexRequest {
                items: vec![
                    // same TF(rust)=1, but `short` is a shorter doc →
                    // length normalization gives it the higher score.
                    item("short", "body", FieldValue::String("rust pad".into())),
                    item(
                        "long",
                        "body",
                        FieldValue::String("rust pad pad pad pad pad pad pad".into()),
                    ),
                ],
                request_id: None,
            },
        )
        .unwrap();
        let hits = search_match(&e, "c", "rust", MatchOp::Or);
        assert_eq!(hits[0].external_id, "short");
        assert!(
            score_of(&hits, "short") > score_of(&hits, "long"),
            "BM25 length-norm: shorter doc ranks higher at equal TF: {hits:?}"
        );
    }

    #[test]
    fn bm25_rarer_term_contributes_more_idf() {
        let e = Engine::new();
        e.create_collection("c", text_only_schema()).unwrap();
        // "common" appears in every doc (low IDF); "rare" in just one
        // (high IDF). A doc matched by "rare" must outscore one matched
        // only by "common" — kills IDF numerator/denominator flips.
        let mut items = vec![
            item("rare_doc", "body", FieldValue::String("common rare".into())),
            item(
                "common_doc",
                "body",
                FieldValue::String("common filler".into()),
            ),
        ];
        for i in 0..8 {
            items.push(item(
                &format!("bg{i}"),
                "body",
                FieldValue::String("common filler".into()),
            ));
        }
        e.index(
            "c",
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();
        let hits = search_match(&e, "c", "rare common", MatchOp::Or);
        assert_eq!(
            hits[0].external_id, "rare_doc",
            "doc matching the rare (high-IDF) term must rank first: {hits:?}"
        );
        assert!(score_of(&hits, "rare_doc") > score_of(&hits, "common_doc"));
    }

    #[test]
    fn or_combines_scores_additively_doc_matching_both_ranks_first() {
        let e = Engine::new();
        e.create_collection("c", text_only_schema()).unwrap();
        e.index(
            "c",
            IndexRequest {
                items: vec![
                    item("both", "body", FieldValue::String("alpha beta".into())),
                    item(
                        "alpha_only",
                        "body",
                        FieldValue::String("alpha gamma".into()),
                    ),
                    item("beta_only", "body", FieldValue::String("beta gamma".into())),
                ],
                request_id: None,
            },
        )
        .unwrap();
        let hits = search_match(&e, "c", "alpha beta", MatchOp::Or);
        // `both` matched on two tokens → its summed score must exceed
        // either single-token doc. Kills the OR `+=`→`-=`/`*=` mutants.
        assert_eq!(
            hits[0].external_id, "both",
            "doc matching both tokens ranks first: {hits:?}"
        );
        let s_both = score_of(&hits, "both");
        assert!(s_both > score_of(&hits, "alpha_only"));
        assert!(s_both > score_of(&hits, "beta_only"));
    }

    #[test]
    fn and_sums_matched_token_scores() {
        let e = Engine::new();
        e.create_collection("c", text_only_schema()).unwrap();
        e.index(
            "c",
            IndexRequest {
                items: vec![
                    item("d1", "body", FieldValue::String("alpha beta".into())),
                    item(
                        "d2",
                        "body",
                        FieldValue::String("alpha beta gamma delta eps".into()),
                    ),
                ],
                request_id: None,
            },
        )
        .unwrap();
        let and_hits = search_match(&e, "c", "alpha beta", MatchOp::And);
        // Both docs contain both tokens → both returned, and the AND
        // score equals the sum of the two per-token contributions. The
        // shorter doc (d1) wins on length-norm. Kills AND `score + s`
        // → `score - s` (which would invert or zero the combination).
        assert_eq!(and_hits.len(), 2);
        assert_eq!(and_hits[0].external_id, "d1");
        assert!(score_of(&and_hits, "d1") > 0.0);
        assert!(score_of(&and_hits, "d1") >= score_of(&and_hits, "d2"));
    }

    #[test]
    fn search_sorts_by_score_desc_then_eid_asc() {
        let e = Engine::new();
        e.create_collection("c", text_only_schema()).unwrap();
        // Two docs with identical content → identical score → tie
        // broken by external_id ascending. Kills the tie-break
        // `a.cmp(b)`→`b.cmp(a)` mutant and the score-cmp flip.
        e.index(
            "c",
            IndexRequest {
                items: vec![
                    item("zeta", "body", FieldValue::String("rust rust".into())),
                    item("alpha", "body", FieldValue::String("rust rust".into())),
                    item("solo", "body", FieldValue::String("rust".into())),
                ],
                request_id: None,
            },
        )
        .unwrap();
        let hits = search_match(&e, "c", "rust", MatchOp::Or);
        // solo has lower TF → lowest score → must be last.
        assert_eq!(hits.last().unwrap().external_id, "solo");
        // zeta & alpha tie on score → alpha first (eid asc).
        let alpha_pos = hits.iter().position(|h| h.external_id == "alpha").unwrap();
        let zeta_pos = hits.iter().position(|h| h.external_id == "zeta").unwrap();
        assert!(alpha_pos < zeta_pos, "tie broken by eid asc: {hits:?}");
    }

    #[test]
    fn bm25_exact_golden_scores() {
        // Pins the formula to textbook BM25 (K1=1.2, B=0.75). Ordering
        // assertions can't catch magnitude-only mutations that preserve
        // relative order (e.g. `(n-df+0.5)/(df+0.5)` → `*`); a
        // hand-computed reference does.
        //
        // Corpus (field "body", whitespace_lower):
        //   a = "x y"      → tf(x)=1, doc_len=2
        //   b = "x x z w"  → tf(x)=2, doc_len=4
        // n=2, total_doc_len=6, avgdl=3, df(x)=2
        // idf = ln((2-2+0.5)/(2+0.5) + 1) = ln(1.2)             = 0.1823215
        // a: denom = 1 + 1.2*(1-0.75 + 0.75*2/3) = 1.9
        //    score = 0.1823215 * 1 * 2.2 / 1.9                  = 0.211110
        // b: denom = 2 + 1.2*(1-0.75 + 0.75*4/3) = 3.5
        //    score = 0.1823215 * 2 * 2.2 / 3.5                  = 0.229204
        let e = Engine::new();
        e.create_collection("c", text_only_schema()).unwrap();
        e.index(
            "c",
            IndexRequest {
                items: vec![
                    item("a", "body", FieldValue::String("x y".into())),
                    item("b", "body", FieldValue::String("x x z w".into())),
                ],
                request_id: None,
            },
        )
        .unwrap();
        let hits = search_match(&e, "c", "x", MatchOp::Or);
        let sa = score_of(&hits, "a");
        let sb = score_of(&hits, "b");
        assert!(
            (sa - 0.211110).abs() < 5e-4,
            "BM25(a) golden mismatch: got {sa}, want ≈0.211110"
        );
        assert!(
            (sb - 0.229204).abs() < 5e-4,
            "BM25(b) golden mismatch: got {sb}, want ≈0.229204"
        );
    }

    fn two_keyword_schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        for name in ["tag", "region"] {
            fields.insert(
                name.to_string(),
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
        CreateCollectionRequest { fields }
    }

    #[test]
    fn eval_query_and_sums_child_scores_exactly() {
        // QueryNode::And of two term queries. Each term contributes a
        // constant score of 1.0, so a doc matching both must score
        // exactly 2.0. Kills the eval_query AND `score + s` → `-`/`*`.
        let e = Engine::new();
        e.create_collection("c", two_keyword_schema()).unwrap();
        e.index(
            "c",
            IndexRequest {
                items: vec![
                    item("d", "tag", FieldValue::String("rust".into())),
                    item("d", "region", FieldValue::String("apac".into())),
                ],
                request_id: None,
            },
        )
        .unwrap();
        let resp = e
            .search(
                "c",
                SearchRequest {
                    query: QueryNode::And(vec![
                        QueryNode::Term(TermQuery {
                            field: "tag".into(),
                            value: FieldValue::String("rust".into()),
                        }),
                        QueryNode::Term(TermQuery {
                            field: "region".into(),
                            value: FieldValue::String("apac".into()),
                        }),
                    ]),
                    limit: 10,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap();
        assert_eq!(resp.hits.len(), 1);
        assert!(
            (resp.hits[0].score - 2.0).abs() < 1e-6,
            "AND of two terms must sum to exactly 2.0, got {}",
            resp.hits[0].score
        );
    }

    #[test]
    fn eval_query_or_sums_and_ranks_multi_match_first() {
        // QueryNode::Or of two terms. A doc matching both scores 2.0 and
        // must rank above a doc matching one (1.0). Kills eval_query OR
        // `+= score` → `-=`/`*=` (which would flip or zero the ranking).
        let e = Engine::new();
        e.create_collection("c", two_keyword_schema()).unwrap();
        e.index(
            "c",
            IndexRequest {
                items: vec![
                    item("both", "tag", FieldValue::String("rust".into())),
                    item("both", "region", FieldValue::String("apac".into())),
                    item("one", "tag", FieldValue::String("rust".into())),
                    item("one", "region", FieldValue::String("emea".into())),
                ],
                request_id: None,
            },
        )
        .unwrap();
        let resp = e
            .search(
                "c",
                SearchRequest {
                    query: QueryNode::Or(vec![
                        QueryNode::Term(TermQuery {
                            field: "tag".into(),
                            value: FieldValue::String("rust".into()),
                        }),
                        QueryNode::Term(TermQuery {
                            field: "region".into(),
                            value: FieldValue::String("apac".into()),
                        }),
                    ]),
                    limit: 10,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap();
        assert_eq!(resp.hits[0].external_id, "both");
        assert!((score_of(&resp.hits, "both") - 2.0).abs() < 1e-6);
        assert!((score_of(&resp.hits, "one") - 1.0).abs() < 1e-6);
    }

    #[test]
    fn eval_query_not_excludes_exactly_the_matched_set() {
        // NOT over a term: universe minus the matched eids, each scored
        // 1.0. Kills the `delete !` mutant on the Not branch.
        let e = Engine::new();
        e.create_collection("c", two_keyword_schema()).unwrap();
        e.index(
            "c",
            IndexRequest {
                items: vec![
                    item("a", "tag", FieldValue::String("rust".into())),
                    item("b", "tag", FieldValue::String("go".into())),
                    item("c", "tag", FieldValue::String("python".into())),
                ],
                request_id: None,
            },
        )
        .unwrap();
        let resp = e
            .search(
                "c",
                SearchRequest {
                    query: QueryNode::Not(Box::new(QueryNode::Term(TermQuery {
                        field: "tag".into(),
                        value: FieldValue::String("rust".into()),
                    }))),
                    limit: 10,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap();
        assert_eq!(resp.total, 2);
        let ids: BTreeSet<&str> = resp.hits.iter().map(|h| h.external_id.as_str()).collect();
        assert!(ids.contains("b") && ids.contains("c") && !ids.contains("a"));
    }

    #[test]
    fn validate_query_rejects_pathological_trees_but_allows_normal() {
        let e = Engine::new();
        e.create_collection("c", two_keyword_schema()).unwrap();
        let term = || {
            QueryNode::Term(TermQuery {
                field: "tag".into(),
                value: FieldValue::String("rust".into()),
            })
        };
        let is_too_complex = |r: Result<SearchResponse>| {
            matches!(
                r.unwrap_err().downcast_ref::<StorageError>(),
                Some(StorageError::QueryTooComplex(_))
            )
        };
        let search = |q: QueryNode| {
            e.search(
                "c",
                SearchRequest {
                    query: q,
                    limit: 10,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
        };

        // Deeply nested (would stack-overflow eval without the guard). Build
        // iteratively so the test itself doesn't recurse.
        let mut deep = term();
        for _ in 0..1000 {
            deep = QueryNode::And(vec![deep]);
        }
        assert!(is_too_complex(search(deep)), "deep query must be rejected");

        // Very wide (node-count DoS).
        let wide = QueryNode::And((0..100_000).map(|_| term()).collect());
        assert!(is_too_complex(search(wide)), "wide query must be rejected");

        // Huge terms fan-out.
        let huge_terms = QueryNode::Terms(TermsQuery {
            field: "tag".into(),
            values: (0..2000)
                .map(|i| FieldValue::String(format!("v{i}")))
                .collect(),
        });
        assert!(
            is_too_complex(search(huge_terms)),
            "huge terms must be rejected"
        );

        // A normal shallow query still works.
        assert!(
            search(QueryNode::And(vec![term()])).is_ok(),
            "normal query must pass"
        );
    }

    #[test]
    fn number_exact_term_query_matches_only_that_value() {
        // Exercises the (Number, Number) arm of eval_term — number
        // *exact* match, distinct from range. Without this, deleting
        // that match arm goes uncaught (the range tests never hit it).
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        e.index(
            "users",
            IndexRequest {
                items: vec![
                    item("a", "age", FieldValue::Number(30.0)),
                    item("b", "age", FieldValue::Number(30.0)),
                    item("c", "age", FieldValue::Number(31.0)),
                ],
                request_id: None,
            },
        )
        .unwrap();
        let resp = e
            .search(
                "users",
                SearchRequest {
                    query: QueryNode::Term(TermQuery {
                        field: "age".into(),
                        value: FieldValue::Number(30.0),
                    }),
                    limit: 10,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap();
        assert_eq!(resp.total, 2);
        let ids: BTreeSet<&str> = resp.hits.iter().map(|h| h.external_id.as_str()).collect();
        assert!(ids.contains("a") && ids.contains("b") && !ids.contains("c"));
    }

    #[test]
    fn and_match_score_is_sum_not_product_of_token_scores() {
        // Single doc "p q" (the only doc). Each token scores identically:
        //   n=1, df=1, idf=ln((0.5)/(1.5)+1)=ln(1.3333)=0.287682
        //   denom = 1 + 1.2*(1-0.75 + 0.75*2/2) = 2.2
        //   per-token = 0.287682 * 1 * 2.2 / 2.2 = 0.287682
        // AND(p,q) sums the two ⇒ 0.575364.
        // The `score + s` → `score * s` mutant would give
        // 0.287682² = 0.082761, so an exact assert kills it.
        let e = Engine::new();
        e.create_collection("c", text_only_schema()).unwrap();
        e.index(
            "c",
            IndexRequest {
                items: vec![item("d", "body", FieldValue::String("p q".into()))],
                request_id: None,
            },
        )
        .unwrap();
        let hits = search_match(&e, "c", "p q", MatchOp::And);
        assert_eq!(hits.len(), 1);
        let s = score_of(&hits, "d");
        assert!(
            (s - 0.575364).abs() < 5e-4,
            "AND match must SUM token scores (≈0.575364), got {s} (product would be ≈0.0828)"
        );
    }

    #[test]
    fn range_bounds_are_exclusive_vs_inclusive() {
        let e = Engine::new();
        e.create_collection("users", build_users_schema()).unwrap();
        let items = (0..=10)
            .map(|i| item(&format!("u{i}"), "age", FieldValue::Number(i as f64)))
            .collect();
        e.index(
            "users",
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();

        let run = |q: RangeQuery| {
            e.search(
                "users",
                SearchRequest {
                    query: QueryNode::Range(q),
                    limit: 50,
                    cursor: None,
                    sort: None,
                    track_total: true,
                    collapse: None,
                },
            )
            .unwrap()
            .total
        };

        // gte=2, lte=5 → {2,3,4,5} = 4
        assert_eq!(
            run(RangeQuery {
                field: "age".into(),
                gt: None,
                gte: Some(2.0),
                lt: None,
                lte: Some(5.0)
            }),
            4
        );
        // gt=2, lt=5 → {3,4} = 2  (exclusive both ends)
        assert_eq!(
            run(RangeQuery {
                field: "age".into(),
                gt: Some(2.0),
                gte: None,
                lt: Some(5.0),
                lte: None
            }),
            2
        );
        // gte=2, lt=5 → {2,3,4} = 3  (mixed)
        assert_eq!(
            run(RangeQuery {
                field: "age".into(),
                gt: None,
                gte: Some(2.0),
                lt: Some(5.0),
                lte: None
            }),
            3
        );
    }
}

// ---------------------------------------------------------------------------
// Triple-path diff test (Stage 2 Phase 2f-1): the RAM=hot / disk=all keystone.
//
// PATH A = a pure live engine (segments OFF).
// PATH B = the SAME engine after `seal_to_segments` + forward-payload drop.
// PATH C = a fresh engine whose collection is `open_from_segments`'d from B's
//          directory (NO CBOR snapshot, NO whole-collection load).
//
// Over a randomized multi-field corpus (Number + Keyword + Set + Text + Hash +
// Vector) the three paths must return IDENTICAL result-SETS, byte-identical f32
// scores (to_bits), identical retrieved field values, and identical kNN
// ordering — for point lookups, range, term, set-membership, BM25, kNN, AND
// direct value retrieval. A missed forward-read site or a wrong inverted-driver
// rebuild MUST fail this. The test also asserts the forward payload provably
// left RAM after the seal-and-drop.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod triple_path_diff_tests {
    use super::*;
    use crate::types::{VectorBackend, VectorMetric};
    use proptest::prelude::*;
    use std::sync::Arc;

    const DIM: usize = 6;

    /// One full query-battery snapshot of an engine: the seven query legs plus
    /// the direct value-retrieval leg. Fields (in order): range, term, setmem,
    /// point, bm25, knn (ordered), hamming, retrieval.
    type Snapshot = (
        Vec<(String, u32)>,
        Vec<(String, u32)>,
        Vec<(String, u32)>,
        Vec<(String, u32)>,
        Vec<(String, u32)>,
        Vec<(String, u32)>,
        Vec<(String, u32)>,
        Vec<(
            String,
            Option<u64>,
            Option<String>,
            Option<Vec<String>>,
            Option<u64>,
        )>,
    );

    fn fieldspec(t: FieldType, analyzer: Option<Analyzer>) -> FieldSpec {
        FieldSpec {
            field_type: t,
            analyzer,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    fn vec_fieldspec() -> FieldSpec {
        FieldSpec {
            field_type: FieldType::Vector,
            analyzer: None,
            multi: None,
            dim: Some(DIM as u32),
            metric: Some(VectorMetric::L2),
            backend: Some(VectorBackend::FlatCpu),
            quantize: None,
        }
    }

    /// Multi-field corpus: num (Number), kw (Keyword), tags (Set), body (Text),
    /// sig (Hash), emb (Vector).
    fn schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        fields.insert("num".into(), fieldspec(FieldType::Number, None));
        fields.insert("kw".into(), fieldspec(FieldType::Keyword, None));
        fields.insert("tags".into(), fieldspec(FieldType::Set, None));
        fields.insert(
            "body".into(),
            fieldspec(FieldType::Text, Some(Analyzer::WhitespaceLower)),
        );
        fields.insert("sig".into(), fieldspec(FieldType::Hash, None));
        fields.insert("emb".into(), vec_fieldspec());
        CreateCollectionRequest { fields }
    }

    fn req(query: QueryNode, limit: u32) -> SearchRequest {
        SearchRequest {
            query,
            limit,
            cursor: None,
            sort: None,
            track_total: true,
            collapse: None,
        }
    }

    fn run(e: &Engine, query: QueryNode, limit: u32) -> Vec<(String, u32)> {
        e.search("c", req(query, limit))
            .unwrap()
            .hits
            .into_iter()
            .map(|h| (h.external_id, h.score.to_bits()))
            .collect()
    }

    /// Result SET (eids only) — order-independent assertion for filter queries.
    fn set_of(rows: &[(String, u32)]) -> BTreeSet<String> {
        rows.iter().map(|(e, _)| e.clone()).collect()
    }

    /// Scores keyed by eid (f32 bits) — order-independent score byte-equality.
    fn scores_of(rows: &[(String, u32)]) -> BTreeMap<String, u32> {
        rows.iter().map(|(e, s)| (e.clone(), *s)).collect()
    }

    /// Index one doc across all six fields. `tok` selects the rare driver token
    /// so BM25 has a non-trivial corpus.
    #[allow(clippy::too_many_arguments)]
    fn index_doc(
        e: &Engine,
        eid: &str,
        num: Option<f64>,
        kw: &str,
        tags: &[&str],
        tok: bool,
        sig: u64,
        emb: &[f32],
    ) {
        let mut items = vec![
            crate::types::IndexItem {
                external_id: eid.into(),
                field: "kw".into(),
                value: FieldValue::String(kw.into()),
            },
            crate::types::IndexItem {
                external_id: eid.into(),
                field: "tags".into(),
                value: FieldValue::StringList(tags.iter().map(|s| s.to_string()).collect()),
            },
            crate::types::IndexItem {
                external_id: eid.into(),
                field: "body".into(),
                value: FieldValue::String(if tok {
                    "tok filler".into()
                } else {
                    "filler".into()
                }),
            },
            crate::types::IndexItem {
                external_id: eid.into(),
                field: "sig".into(),
                value: FieldValue::String(format!("{sig:016x}")),
            },
            crate::types::IndexItem {
                external_id: eid.into(),
                field: "emb".into(),
                value: FieldValue::Vector(emb.to_vec()),
            },
        ];
        if let Some(n) = num {
            items.push(crate::types::IndexItem {
                external_id: eid.into(),
                field: "num".into(),
                value: FieldValue::Number(n),
            });
        }
        e.index(
            "c",
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();
    }

    /// A match-DRIVEN AND so the non-text conjunct is applied as a per-doc
    /// PREDICATE (`number_at`/`keyword_at`/`set_contains` — the segment-backed
    /// read sites), not a posting-walk.
    fn driven(extra: QueryNode) -> QueryNode {
        QueryNode::And(vec![
            QueryNode::Match(MatchQuery {
                field: "body".into(),
                text: "tok".into(),
                op: MatchOp::And,
            }),
            extra,
        ])
    }

    /// Direct field-value RETRIEVAL through the segment-aware accessors, for
    /// every docid — the "value retrieval" leg of the contract. Resolves each
    /// docid to (eid, num, kw, tags, sig). Routes through `number_at` /
    /// `keyword_at` / `set_members` / `hash_at`, which after a seal-and-drop must
    /// read the segment, not the (empty) forward map.
    fn retrieve_all(
        e: &Engine,
    ) -> Vec<(
        String,
        Option<u64>,
        Option<String>,
        Option<Vec<String>>,
        Option<u64>,
    )> {
        let state = e.state.read().unwrap();
        let coll = state.collections.get("c").unwrap();
        let n = coll.interner.to_eid.len() as u32;
        let mut out = Vec::with_capacity(n as usize);
        for id in 0..n {
            let eid = coll.interner.resolve(id).to_string();
            let num = match coll.fields.get("num") {
                Some(FieldIndex::Number(nx)) => nx.number_at(id).map(|s| s.to_f64().to_bits()),
                _ => None,
            };
            let kw = match coll.fields.get("kw") {
                Some(FieldIndex::Keyword(k)) => k.keyword_at(id),
                _ => None,
            };
            let tags = match coll.fields.get("tags") {
                Some(FieldIndex::Set(s)) => s.set_members(id).map(|m| m.into_iter().collect()),
                _ => None,
            };
            let sig = match coll.fields.get("sig") {
                Some(FieldIndex::Hash(h)) => h.hash_at(id),
                _ => None,
            };
            out.push((eid, num, kw, tags, sig));
        }
        // Order by eid so the comparison is interner-order-independent (PATH C
        // re-interns in docid order, which equals PATH A/B docid order, but
        // sorting makes the contract explicit).
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(80))]

        #[test]
        fn triple_path_a_eq_b_eq_c(
            docs in proptest::collection::vec(
                (
                    proptest::option::weighted(0.8, 0u32..30),          // num (some absent)
                    prop::sample::select(vec!["a", "b", "c", "d"]),     // kw
                    proptest::collection::vec(
                        prop::sample::select(vec!["red", "green", "blue", "x"]),
                        0..3,
                    ),                                                  // tags
                    any::<bool>(),                                      // tok
                    0u64..64,                                           // sig (low bits)
                    proptest::collection::vec(-3.0f32..3.0, DIM..=DIM), // emb
                ),
                1..40,
            ),
            lo in 0u32..30,
            span in 1u32..30,
            qsig in 0u64..64,
            qraw in proptest::collection::vec(-3.0f32..3.0, DIM..=DIM),
            kw_pick in prop::sample::select(vec!["a", "b", "c", "d"]),
            tag_pick in prop::sample::select(vec!["red", "green", "blue", "x"]),
        ) {
            let hi = lo + span;

            // --- PATH A: live engine, segments OFF. ---
            let e = Arc::new(Engine::new());
            e.create_collection("c", schema()).unwrap();
            for (i, (num, kw, tags, tok, sig, emb)) in docs.iter().enumerate() {
                let tagrefs: Vec<&str> = tags.iter().copied().collect();
                index_doc(
                    &e,
                    &format!("d{i}"),
                    num.map(|n| n as f64),
                    kw,
                    &tagrefs,
                    *tok,
                    *sig,
                    emb,
                );
            }

            // The query battery (built once, reused across paths).
            let q_range = || driven(QueryNode::Range(RangeQuery {
                field: "num".into(), gt: None, gte: Some(lo as f64), lt: Some(hi as f64), lte: None,
            }));
            let q_term = || driven(QueryNode::Term(TermQuery {
                field: "kw".into(), value: FieldValue::String(kw_pick.to_string()),
            }));
            let q_setmem = || driven(QueryNode::Terms(TermsQuery {
                field: "tags".into(),
                values: vec![FieldValue::String(tag_pick.to_string())],
            }));
            let q_point = || QueryNode::Term(TermQuery {
                field: "kw".into(), value: FieldValue::String(kw_pick.to_string()),
            });
            let q_bm25 = || QueryNode::Match(MatchQuery {
                field: "body".into(), text: "tok".into(), op: MatchOp::And,
            });
            let q_knn = || QueryNode::Knn(crate::types::KnnQuery {
                field: "emb".into(), vector: qraw.clone(), k: 8,
            });
            let q_ham = || QueryNode::Hamming(crate::types::HammingQuery {
                field: "sig".into(), hash: format!("{qsig:016x}"), max_distance: 6,
            });

            let snapshot = |e: &Engine| -> Snapshot {
                (
                    run(e, q_range(), 100_000),
                    run(e, q_term(), 100_000),
                    run(e, q_setmem(), 100_000),
                    run(e, q_point(), 100_000),
                    run(e, q_bm25(), 100_000),
                    run(e, q_knn(), 8),
                    run(e, q_ham(), 100_000),
                    retrieve_all(e),
                )
            };

            let a = snapshot(&e);

            // --- PATH B: production seal_to_segments + drop, rerun. ---
            let dir = tempfile::tempdir().unwrap();
            e.__seal_collection_to_segments("c", dir.path(), 1).unwrap();
            let b = snapshot(&e);

            // The forward payload provably LEFT RAM (and the inverted driver did
            // NOT): every dropped field's forward map is empty / tokens dropped,
            // yet a segment is attached and queries still answer.
            for f in ["num", "kw", "tags", "sig"] {
                let (fwd, _toks, has_seg) = e.__field_forward_probe("c", f).unwrap();
                prop_assert_eq!(fwd, 0, "field `{}` forward map not freed after drop", f);
                prop_assert!(has_seg, "field `{}` has no segment after seal", f);
            }
            let (_f, toks, has_seg) = e.__field_forward_probe("c", "body").unwrap();
            prop_assert_eq!(toks, 0, "text tokens not freed after drop");
            prop_assert!(has_seg, "text field has no segment after seal");

            // --- PATH C: reopen from segments (no snapshot), rerun. ---
            let schema = e.__collection_schema("c").unwrap();
            let ce = Engine::__open_collection_from_segments("c", dir.path(), schema, 1).unwrap();
            let c = snapshot(&ce);

            // A == B and B == C, leg by leg. Filter legs compare result SET +
            // byte-identical scores; kNN compares the full ordered ranked vec;
            // retrieval compares the resolved field values. Filter tuple fields:
            // 0 range, 1 term, 2 setmem, 3 point, 4 bm25, 6 ham (5 knn, 7 retrieval
            // are compared separately because their contract is ordered / value).
            let filt = |x: &Snapshot| {
                vec![
                    (set_of(&x.0), scores_of(&x.0)),
                    (set_of(&x.1), scores_of(&x.1)),
                    (set_of(&x.2), scores_of(&x.2)),
                    (set_of(&x.3), scores_of(&x.3)),
                    (set_of(&x.4), scores_of(&x.4)),
                    (set_of(&x.6), scores_of(&x.6)),
                ]
            };
            let names = ["range", "term", "setmem", "point", "bm25", "hamming"];
            let (fa, fb, fc) = (filt(&a), filt(&b), filt(&c));
            for (i, name) in names.iter().enumerate() {
                prop_assert_eq!(&fa[i].0, &fb[i].0, "A!=B {} set", name);
                prop_assert_eq!(&fa[i].1, &fb[i].1, "A!=B {} scores", name);
                prop_assert_eq!(&fb[i].0, &fc[i].0, "B!=C {} set", name);
                prop_assert_eq!(&fb[i].1, &fc[i].1, "B!=C {} scores", name);
            }
            // kNN is RANKED: the whole ordered vec must match bit-for-bit.
            prop_assert_eq!(&a.5, &b.5, "A!=B knn ordering/score");
            prop_assert_eq!(&b.5, &c.5, "B!=C knn ordering/score");
            // Value retrieval through the segment-aware accessors.
            prop_assert_eq!(&a.7, &b.7, "A!=B value retrieval");
            prop_assert_eq!(&b.7, &c.7, "B!=C value retrieval");
        }
    }
}

// ---------------------------------------------------------------------------
// Engine-level checkpoint (Stage 2 Phase 2f-2): the disk engine as the running
// binary's persistence. Two contracts:
//   (a) ENGINE REOPEN — a multi-collection, all-field-type engine, flushed to a
//       checkpoint dir and reopened into a FRESH engine, answers every query leg
//       identically (the disk engine IS a faithful persistence).
//   (b) IDEMPOTENT DOUBLE-FLUSH — flush, index MORE docs, flush again, reopen
//       yields ALL docs (base + tail) identical to a pure-live engine. This is
//       the re-seal-after-drop proof: the second flush reads base docs from the
//       prior segment (their live forward was dropped), not the empty forward map.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod checkpoint_engine_tests {
    use super::*;
    use crate::types::{
        KnnQuery, MatchOp, MatchQuery, RangeQuery, TermQuery, TermsQuery, VectorBackend,
        VectorMetric,
    };
    use std::sync::Arc;

    const DIM: usize = 4;

    fn fieldspec(t: FieldType, analyzer: Option<Analyzer>) -> FieldSpec {
        FieldSpec {
            field_type: t,
            analyzer,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    fn vec_fieldspec() -> FieldSpec {
        FieldSpec {
            field_type: FieldType::Vector,
            analyzer: None,
            multi: None,
            dim: Some(DIM as u32),
            metric: Some(VectorMetric::L2),
            backend: Some(VectorBackend::FlatCpu),
            quantize: None,
        }
    }

    /// Multi-field corpus matching the triple-path schema: num (Number), kw
    /// (Keyword), tags (Set), body (Text), sig (Hash), emb (Vector).
    fn schema() -> CreateCollectionRequest {
        let mut fields = BTreeMap::new();
        fields.insert("num".into(), fieldspec(FieldType::Number, None));
        fields.insert("kw".into(), fieldspec(FieldType::Keyword, None));
        fields.insert("tags".into(), fieldspec(FieldType::Set, None));
        fields.insert(
            "body".into(),
            fieldspec(FieldType::Text, Some(Analyzer::WhitespaceLower)),
        );
        fields.insert("sig".into(), fieldspec(FieldType::Hash, None));
        fields.insert("emb".into(), vec_fieldspec());
        CreateCollectionRequest { fields }
    }

    fn index_doc(
        e: &Engine,
        coll: &str,
        eid: &str,
        n: f64,
        kw: &str,
        tag: &str,
        tok: bool,
        sig: u64,
        emb: &[f32],
    ) {
        let items = vec![
            crate::types::IndexItem {
                external_id: eid.into(),
                field: "num".into(),
                value: FieldValue::Number(n),
            },
            crate::types::IndexItem {
                external_id: eid.into(),
                field: "kw".into(),
                value: FieldValue::String(kw.into()),
            },
            crate::types::IndexItem {
                external_id: eid.into(),
                field: "tags".into(),
                value: FieldValue::StringList(vec![tag.into()]),
            },
            crate::types::IndexItem {
                external_id: eid.into(),
                field: "body".into(),
                value: FieldValue::String(if tok {
                    "tok filler".into()
                } else {
                    "filler".into()
                }),
            },
            crate::types::IndexItem {
                external_id: eid.into(),
                field: "sig".into(),
                value: FieldValue::String(format!("{sig:016x}")),
            },
            crate::types::IndexItem {
                external_id: eid.into(),
                field: "emb".into(),
                value: FieldValue::Vector(emb.to_vec()),
            },
        ];
        e.index(
            coll,
            IndexRequest {
                items,
                request_id: None,
            },
        )
        .unwrap();
    }

    fn req(query: QueryNode, limit: u32) -> SearchRequest {
        SearchRequest {
            query,
            limit,
            cursor: None,
            sort: None,
            track_total: true,
            collapse: None,
        }
    }

    fn run(e: &Engine, coll: &str, query: QueryNode, limit: u32) -> Vec<(String, u32)> {
        e.search(coll, req(query, limit))
            .unwrap()
            .hits
            .into_iter()
            .map(|h| (h.external_id, h.score.to_bits()))
            .collect()
    }

    fn set_of(rows: &[(String, u32)]) -> BTreeSet<String> {
        rows.iter().map(|(e, _)| e.clone()).collect()
    }
    fn scores_of(rows: &[(String, u32)]) -> BTreeMap<String, u32> {
        rows.iter().map(|(e, s)| (e.clone(), *s)).collect()
    }

    fn driven(extra: QueryNode) -> QueryNode {
        QueryNode::And(vec![
            QueryNode::Match(MatchQuery {
                field: "body".into(),
                text: "tok".into(),
                op: MatchOp::And,
            }),
            extra,
        ])
    }

    /// The full query battery for one collection (predicate legs go through the
    /// segment-aware per-doc accessors; kNN/hamming/bm25 through the segment scan).
    fn battery(e: &Engine, coll: &str) -> Vec<(BTreeSet<String>, BTreeMap<String, u32>)> {
        let legs = vec![
            driven(QueryNode::Range(RangeQuery {
                field: "num".into(),
                gt: None,
                gte: Some(2.0),
                lt: Some(8.0),
                lte: None,
            })),
            driven(QueryNode::Term(TermQuery {
                field: "kw".into(),
                value: FieldValue::String("a".into()),
            })),
            driven(QueryNode::Terms(TermsQuery {
                field: "tags".into(),
                values: vec![FieldValue::String("red".into())],
            })),
            QueryNode::Term(TermQuery {
                field: "kw".into(),
                value: FieldValue::String("b".into()),
            }),
            QueryNode::Match(MatchQuery {
                field: "body".into(),
                text: "tok".into(),
                op: MatchOp::And,
            }),
            QueryNode::Hamming(crate::types::HammingQuery {
                field: "sig".into(),
                hash: format!("{:016x}", 0u64),
                max_distance: 8,
            }),
        ];
        legs.into_iter()
            .map(|q| {
                let r = run(e, coll, q, 100_000);
                (set_of(&r), scores_of(&r))
            })
            .collect()
    }

    fn knn(e: &Engine, coll: &str, q: &[f32]) -> Vec<(String, u32)> {
        run(
            e,
            coll,
            QueryNode::Knn(KnnQuery {
                field: "emb".into(),
                vector: q.to_vec(),
                k: 8,
            }),
            8,
        )
    }

    // Some fixed multi-collection corpus. Two collections, all field types.
    fn seed(e: &Engine) {
        e.create_collection("alpha", schema()).unwrap();
        e.create_collection("beta", schema()).unwrap();
        let docs = [
            ("d0", 1.0, "a", "red", true, 0u64, [0.1f32, 0.2, 0.3, 0.4]),
            ("d1", 3.0, "b", "blue", true, 3, [0.9, 0.8, 0.7, 0.6]),
            ("d2", 5.0, "a", "red", false, 7, [0.5, 0.5, 0.5, 0.5]),
            ("d3", 7.0, "c", "green", true, 1, [0.2, 0.4, 0.6, 0.8]),
        ];
        for (eid, n, kw, tag, tok, sig, emb) in docs {
            index_doc(e, "alpha", eid, n, kw, tag, tok, sig, &emb);
            index_doc(
                e,
                "beta",
                &format!("b{eid}"),
                n + 1.0,
                kw,
                tag,
                tok,
                sig + 1,
                &emb,
            );
        }
    }

    // ----- (a) ENGINE REOPEN -------------------------------------------------
    #[test]
    fn flush_then_reopen_into_fresh_engine_is_identical() {
        let live = Arc::new(Engine::new());
        seed(&live);

        let qa = [0.15f32, 0.25, 0.35, 0.45];
        let live_battery_alpha = battery(&live, "alpha");
        let live_battery_beta = battery(&live, "beta");
        let live_knn_alpha = knn(&live, "alpha", &qa);

        let dir = tempfile::tempdir().unwrap();
        live.flush_to_segments(dir.path(), 11).unwrap();

        // Fresh engine reopened ONLY from the checkpoint dir (no CBOR, no log).
        let reopened = Arc::new(Engine::new());
        let seq = reopened.reopen_from_segment_dir(dir.path()).unwrap();
        assert_eq!(
            seq, 11,
            "applied_seq must round-trip through the checkpoint"
        );

        assert_eq!(
            reopened.list_collections().unwrap().len(),
            2,
            "both collections reopened"
        );
        assert_eq!(
            battery(&reopened, "alpha"),
            live_battery_alpha,
            "alpha legs diverged after reopen"
        );
        assert_eq!(
            battery(&reopened, "beta"),
            live_battery_beta,
            "beta legs diverged after reopen"
        );
        assert_eq!(
            knn(&reopened, "alpha", &qa),
            live_knn_alpha,
            "alpha kNN diverged after reopen"
        );
    }

    // ----- (b) IDEMPOTENT DOUBLE-FLUSH (re-seal-after-drop proof) -------------
    #[test]
    fn double_flush_with_tail_matches_pure_live() {
        // The persisted engine: seed, FLUSH (drops forward for base docs), index
        // MORE docs (the live tail), FLUSH AGAIN, reopen.
        let persisted = Arc::new(Engine::new());
        seed(&persisted);
        let dir = tempfile::tempdir().unwrap();
        persisted.flush_to_segments(dir.path(), 4).unwrap(); // first checkpoint

        // After the first flush the base docs' forward maps are dropped. Add a
        // tail of new docs whose docids are > the sealed n_docs.
        let tail = [
            (
                "d4",
                2.5,
                "b",
                "red",
                true,
                0u64,
                [0.11f32, 0.22, 0.33, 0.44],
            ),
            ("d5", 6.5, "a", "blue", true, 7, [0.6, 0.6, 0.6, 0.6]),
        ];
        for (eid, n, kw, tag, tok, sig, emb) in tail {
            index_doc(&persisted, "alpha", eid, n, kw, tag, tok, sig, &emb);
            index_doc(
                &persisted,
                "beta",
                &format!("b{eid}"),
                n + 1.0,
                kw,
                tag,
                tok,
                sig + 1,
                &emb,
            );
        }
        // SECOND flush: this RE-SEALS. Base docs must be gathered from the prior
        // segment (their forward is empty), the tail from the live forward. If the
        // gather read raw `forward`, the base docs would seal as ABSENT here.
        persisted.flush_to_segments(dir.path(), 6).unwrap();

        let reopened = Arc::new(Engine::new());
        let seq = reopened.reopen_from_segment_dir(dir.path()).unwrap();
        assert_eq!(seq, 6, "second checkpoint's seq must win");

        // The oracle: a pure-live engine that NEVER flushed, with the SAME docs.
        let pure = Arc::new(Engine::new());
        seed(&pure);
        for (eid, n, kw, tag, tok, sig, emb) in tail {
            index_doc(&pure, "alpha", eid, n, kw, tag, tok, sig, &emb);
            index_doc(
                &pure,
                "beta",
                &format!("b{eid}"),
                n + 1.0,
                kw,
                tag,
                tok,
                sig + 1,
                &emb,
            );
        }

        let qa = [0.15f32, 0.25, 0.35, 0.45];
        // EVERY leg of BOTH collections must match the pure-live oracle — base AND
        // tail docs (sets AND byte-identical scores). This is the re-seal-after-drop
        // correctness proof: the second flush gathered base docs from the prior
        // segment (their forward was dropped) and the tail from the live state.
        assert_eq!(
            battery(&reopened, "alpha"),
            battery(&pure, "alpha"),
            "alpha legs diverged after double-flush"
        );
        assert_eq!(
            battery(&reopened, "beta"),
            battery(&pure, "beta"),
            "beta legs diverged after double-flush"
        );
        assert_eq!(
            knn(&reopened, "alpha", &qa),
            knn(&pure, "alpha", &qa),
            "alpha kNN diverged after double-flush"
        );
        assert_eq!(
            knn(&reopened, "beta", &qa),
            knn(&pure, "beta", &qa),
            "beta kNN diverged after double-flush"
        );

        // And direct doc-count parity (base 4 + tail 2 = 6 per collection).
        assert_eq!(reopened.stats("alpha").unwrap().documents_indexed, 6);
        assert_eq!(reopened.stats("beta").unwrap().documents_indexed, 6);
    }

    // ----- (c) TOMBSTONE GC ACROSS A CHECKPOINT (Phase 2g-A) -----------------
    //
    // THE CRUX. A base doc DELETED after the first checkpoint must be GC'd by the
    // second checkpoint and stay absent on reopen — never resurrected, never an
    // inflated BM25 corpus. Sequence: seed → flush S1 (base docs' forward dropped,
    // values now ONLY on the immutable segment) → DELETE several base docs (in the
    // sealed range) + index a live tail → flush S2 (re-seal) → reopen fresh.
    //
    // The oracle is a PURE-LIVE engine that ran the identical op sequence but NEVER
    // flushed (no segments at all). The reopened-from-disk engine must match it on
    // every leg: result SETs, byte-identical f32 BM25 scores (corpus scalars must
    // exclude the deleted docs), ordered kNN (deleted vectors absent), retrieved
    // values, and doc_count. Plus a direct assertion that a deleted eid is gone.
    //
    // Without the liveness-aware gather, flush S2's `(0..n_docs).map(number_at)`
    // re-reads the deleted base doc's STALE value off the prior segment and writes
    // it back, so reopen RESURRECTS the doc and the BM25 corpus is inflated.
    #[test]
    fn delete_across_checkpoint_is_gc_not_resurrected() {
        // Persisted engine: seed (d0..d3), checkpoint, delete, tail, checkpoint.
        let persisted = Arc::new(Engine::new());
        seed(&persisted);
        let dir = tempfile::tempdir().unwrap();
        persisted.flush_to_segments(dir.path(), 4).unwrap(); // S1: base forward dropped

        // Delete BASE docs that live entirely in the sealed segment range. d0
        // (tok=true) is in the BM25 corpus, so deleting it MUST shrink doc_count /
        // total_doc_len; d2 (tok=false) exercises a non-text-bearing delete. On
        // beta, delete bd1 (tok=true) so both collections are GC-tested.
        let deletes_alpha = ["d0", "d2"];
        let deletes_beta = ["bd1"];
        for eid in deletes_alpha {
            persisted.delete("alpha", eid, None).unwrap();
        }
        for eid in deletes_beta {
            persisted.delete("beta", eid, None).unwrap();
        }

        // Live tail (docids > sealed n_docs), some sharing the deleted docs' terms
        // so a resurrected base doc would be detectable as an extra set member.
        let tail = [
            (
                "d4",
                2.5,
                "a",
                "red",
                true,
                0u64,
                [0.11f32, 0.22, 0.33, 0.44],
            ),
            ("d5", 6.5, "b", "blue", true, 7, [0.6, 0.6, 0.6, 0.6]),
        ];
        for (eid, n, kw, tag, tok, sig, emb) in tail {
            index_doc(&persisted, "alpha", eid, n, kw, tag, tok, sig, &emb);
            index_doc(
                &persisted,
                "beta",
                &format!("b{eid}"),
                n + 1.0,
                kw,
                tag,
                tok,
                sig + 1,
                &emb,
            );
        }
        persisted.flush_to_segments(dir.path(), 6).unwrap(); // S2: re-seal must GC deletes

        // Fresh reopen ONLY from the checkpoint dir.
        let reopened = Arc::new(Engine::new());
        let seq = reopened.reopen_from_segment_dir(dir.path()).unwrap();
        assert_eq!(seq, 6, "second checkpoint's seq must win");

        // Oracle: pure-live engine, identical op sequence, NEVER flushed.
        let pure = Arc::new(Engine::new());
        seed(&pure);
        for eid in deletes_alpha {
            pure.delete("alpha", eid, None).unwrap();
        }
        for eid in deletes_beta {
            pure.delete("beta", eid, None).unwrap();
        }
        for (eid, n, kw, tag, tok, sig, emb) in tail {
            index_doc(&pure, "alpha", eid, n, kw, tag, tok, sig, &emb);
            index_doc(
                &pure,
                "beta",
                &format!("b{eid}"),
                n + 1.0,
                kw,
                tag,
                tok,
                sig + 1,
                &emb,
            );
        }

        let qa = [0.15f32, 0.25, 0.35, 0.45];
        // Every leg of BOTH collections must match the pure-live oracle, SETS and
        // byte-identical f32 BM25 scores. The BM25 leg of `battery` is the corpus
        // teeth: if a deleted tok=true doc were resurrected, doc_count/avgdl shift
        // and EVERY surviving doc's BM25 score changes — a byte diff.
        assert_eq!(
            battery(&reopened, "alpha"),
            battery(&pure, "alpha"),
            "alpha legs diverged after delete+checkpoint"
        );
        assert_eq!(
            battery(&reopened, "beta"),
            battery(&pure, "beta"),
            "beta legs diverged after delete+checkpoint"
        );
        // Ordered kNN: a resurrected vector row would re-enter the scan and reorder.
        assert_eq!(
            knn(&reopened, "alpha", &qa),
            knn(&pure, "alpha", &qa),
            "alpha kNN diverged after delete+checkpoint"
        );
        assert_eq!(
            knn(&reopened, "beta", &qa),
            knn(&pure, "beta", &qa),
            "beta kNN diverged after delete+checkpoint"
        );

        // doc_count: base 4 - 2 deleted + 2 tail = 4 (alpha); 4 - 1 + 2 = 5 (beta).
        assert_eq!(
            reopened.stats("alpha").unwrap().documents_indexed,
            4,
            "alpha doc_count inflated by resurrected docs"
        );
        assert_eq!(
            reopened.stats("beta").unwrap().documents_indexed,
            5,
            "beta doc_count inflated by resurrected docs"
        );

        // DIRECT GC assertion: every deleted eid is absent from every leg AND from
        // direct value retrieval after reopen — it was GC'd, not resurrected.
        let alpha_hits: BTreeSet<String> = {
            // A broad query that would surface a resurrected doc on any field.
            let mut s = BTreeSet::new();
            for kwv in ["a", "b", "c", "d"] {
                let r = run(
                    &reopened,
                    "alpha",
                    QueryNode::Term(TermQuery {
                        field: "kw".into(),
                        value: FieldValue::String(kwv.into()),
                    }),
                    100_000,
                );
                s.extend(r.into_iter().map(|(e, _)| e));
            }
            s
        };
        for eid in deletes_alpha {
            assert!(
                !alpha_hits.contains(eid),
                "deleted alpha eid `{eid}` RESURRECTED after reopen"
            );
        }
        // And the deleted eid resolves to NO value through the segment-aware
        // accessors (its interner slot is a tombstone, excluded by eid_fields).
        {
            let state = reopened.state.read().unwrap();
            let coll = state.collections.get("alpha").unwrap();
            for eid in deletes_alpha {
                // A deleted eid is still interned (positionally stable docids), but
                // it must carry NO live field coverage.
                if let Some(id) = coll.interner.id(eid) {
                    assert!(
                        coll.eid_fields.get(&id).is_none_or(|fs| fs.is_empty()),
                        "deleted alpha eid `{eid}` (id {id}) still has live field coverage after reopen",
                    );
                }
            }
        }
    }
}
// CODEGEN-END

````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/storage.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/storage.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```
