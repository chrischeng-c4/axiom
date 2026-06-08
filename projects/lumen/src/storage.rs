//! In-memory storage and query execution.
//!
//! The v1 storage path is split in two layers:
//!
//! 1. **In-memory engine** (this module) — `BTreeMap`-backed inverted
//!    indexes per field. This is the only path when [`Engine::new`]
//!    is used (no backend attached). Single-pod, single-shard, no
//!    persistence.
//! 2. **LSM-attached engine** — created by [`Engine::new_with_backend`].
//!    Every `keyword` and `number` write **and** read flows through
//!    the [`SharedBackend`] (`storage_lsm.rs` ships the only impl).
//!    The in-memory `BTreeMap`s are still maintained as a write-through
//!    cache (kept for `stats`, fast iteration, snapshot/restore parity
//!    with the pure-memory path) but the durable read-path is the
//!    backend.
//!
//! ### v1 scope of the LSM read-path flip
//!
//! Only `keyword` and `number` field types route their reads through
//! the backend in v1. The other field types carry richer per-document
//! state and stay in-memory for now:
//!
//! * `text` — BM25 needs per-doc term frequency, doc length, and
//!   collection-wide stats. Encoding that in the posting payload is
//!   straightforward but the scoring loop would need to be rewritten
//!   around `backend.scan_range(...)`; that lands as a follow-up.
//! * `set` — every element is a separate posting; the forward map
//!   tracks distinct elements. The wiring is identical to keyword but
//!   was descoped to keep this slice small.
//! * `vector` — HNSW graphs live in a dedicated index structure and
//!   cannot be reconstructed from a flat posting list; LSM-backed
//!   vectors will need a side-table for the graph.
//!
//! When a backend is attached, indexing `text` / `set` / `vector`
//! returns [`StorageError::BackendNotSupported`].
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
//!
//! ### Backend key encoding
//!
//! Postings written through the backend share a single namespace per
//! `(collection, partition)`. To keep two fields on the same partition
//! from colliding, every key bytes are prefixed by the field name plus
//! a NUL byte separator:
//!
//! ```text
//! keyword: <field_name>\x00<value_utf8>
//! number:  <field_name>\x00<SortableF64 big-endian bytes>
//! ```
//!
//! The partition is computed once per field as
//! `crc32(field_name) % 4` (README §2: `hash(field) % 4`). A reserved
//! partition `255` holds one entry per collection that persists the
//! collection's `CreateCollectionRequest` JSON so cold recovery can
//! rebuild the schema map before re-hydrating postings.

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::metrics::Metrics;
use crate::storage_backend::{RecoveredPosting, SharedBackend};
use crate::tokenize;
use crate::types::{
    Analyzer, CacheStats, CreateCollectionRequest, CreateCollectionResponse, DuplicateGroup,
    DuplicatesRequest, DuplicatesResponse, FieldSpec, FieldStats, FieldType, FieldValue,
    HammingQuery, HasChildQuery, IndexRequest, IndexResponse, KnnQuery, MatchOp, MatchQuery,
    QueryNode, RangeQuery, RrfQuery, SearchHit, SearchRequest, SearchResponse, SortOrder,
    StatsResponse, StorageStats, TermQuery, TermsQuery, VectorSpec,
};
use crate::vector_index::{open_backend, HnswCpuIndex, ScalarCodebook, VectorIndex};
use roaring::RoaringBitmap;

const IDEMPOTENCY_TTL: Duration = Duration::from_secs(300);

/// Maximum items in a single `POST /index` request (README §1 v1 limit).
pub const MAX_INDEX_ITEMS: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    #[error("collection `{0}` was deleted and is pending physical removal")]
    Gone(String),
    #[error(
        "field `{field_name}` of type {field_type:?} cannot be indexed against the LSM \
         backend: v1 LSM only covers keyword + number; declare those field types in another \
         collection or use `Engine::new()` for full coverage."
    )]
    BackendNotSupported {
        field_type: FieldType,
        field_name: String,
    },
}

// ---------------------------------------------------------------------------
// Sortable f64 key
// ---------------------------------------------------------------------------

/// Total-ordered, bit-monotone wrapper around `f64`. NaN is rejected at
/// construction (the API layer must validate before reaching here).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SortableF64(u64);

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
}

// ---------------------------------------------------------------------------
// Backend partition / key helpers
// ---------------------------------------------------------------------------

/// Reserved partition for collection-schema entries. README §2 only
/// requires partitions 0..4 for data, so we tuck schema metadata into
/// the top of the `u8` space where it cannot collide with a real
/// posting partition.
const SCHEMA_PARTITION: u8 = 255;

/// Inside [`SCHEMA_PARTITION`], every schema entry shares this key —
/// the entries are distinguished by their `collection` field which is
/// already part of the backend address (collection, partition, key).
const SCHEMA_KEY: &[u8] = b"schema";

/// Inside [`SCHEMA_PARTITION`], the schema entry's `external_id` is
/// always this constant — there is exactly one schema per collection.
const SCHEMA_EID: &str = "_self";

/// README §2: partition := `hash(field) % 4`. We use CRC-32 to match
/// the collection-shard hash in `routing.rs` so the two layers behave
/// consistently and are independently testable.
pub(crate) fn partition_for_field(field: &str) -> u8 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(field.as_bytes());
    (hasher.finalize() % 4) as u8
}

/// Encode a keyword (string) value into backend-addressable bytes.
fn keyword_backend_key(field: &str, value: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(field.len() + 1 + value.len());
    out.extend_from_slice(field.as_bytes());
    out.push(0);
    out.extend_from_slice(value.as_bytes());
    out
}

/// Decode a backend key produced by [`keyword_backend_key`]. Returns
/// the raw value bytes (after the NUL separator) if the key matches
/// the given field, else `None`.
fn decode_keyword_key<'a>(field: &str, key: &'a [u8]) -> Option<&'a [u8]> {
    let prefix_len = field.len() + 1;
    if key.len() < prefix_len {
        return None;
    }
    if &key[..field.len()] != field.as_bytes() || key[field.len()] != 0 {
        return None;
    }
    Some(&key[prefix_len..])
}

/// Encode a `SortableF64` value into backend-addressable bytes. The
/// `SortableF64` representation already preserves total order under
/// lexicographic comparison, so big-endian bytes are correct for
/// range scans.
fn number_backend_key(field: &str, key: SortableF64) -> Vec<u8> {
    let mut out = Vec::with_capacity(field.len() + 1 + 8);
    out.extend_from_slice(field.as_bytes());
    out.push(0);
    out.extend_from_slice(&key.0.to_be_bytes());
    out
}

/// Inclusive lower bound on a number field — `field\0` (an empty
/// suffix is < any 8-byte suffix).
fn number_field_lo(field: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(field.len() + 1);
    out.extend_from_slice(field.as_bytes());
    out.push(0);
    out
}

/// Exclusive upper bound on a number field — `field\0\xff..` (8 0xff
/// bytes, which sorts just above any valid suffix).
fn number_field_hi(field: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(field.len() + 1 + 8);
    out.extend_from_slice(field.as_bytes());
    out.push(0);
    out.extend_from_slice(&[0xff; 8]);
    // Push one extra 0xff so `hi` is strictly above any 8-byte suffix
    // that legitimately ends with 0xff (e.g. positive infinity).
    out.push(0xff);
    out
}

/// Decode the 8-byte big-endian suffix of a backend number key back
/// into a `SortableF64`.
fn decode_number_key(field: &str, key: &[u8]) -> Option<SortableF64> {
    let suffix = decode_keyword_key(field, key)?;
    if suffix.len() != 8 {
        return None;
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(suffix);
    Some(SortableF64(u64::from_be_bytes(buf)))
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
    to_id: HashMap<String, u32>,
    to_eid: Vec<String>,
}

impl Interner {
    fn intern(&mut self, eid: &str) -> u32 {
        if let Some(&id) = self.to_id.get(eid) {
            return id;
        }
        let id = self.to_eid.len() as u32;
        self.to_eid.push(eid.to_string());
        self.to_id.insert(eid.to_string(), id);
        id
    }

    fn id(&self, eid: &str) -> Option<u32> {
        self.to_id.get(eid).copied()
    }

    fn resolve(&self, id: u32) -> &str {
        &self.to_eid[id as usize]
    }
}

#[derive(Debug, Default)]
struct TextIndex {
    /// token → doc-id → term frequency in that document.
    tokens: BTreeMap<String, BTreeMap<u32, u32>>,
    /// doc-id → (distinct tokens emitted, doc length in tokens).
    forward: HashMap<u32, (BTreeSet<String>, u32)>,
    doc_count: u64,
    total_doc_len: u64,
    bytes: u64,
}

#[derive(Debug, Default)]
struct KeywordIndex {
    /// term → docs (RoaringBitmap so AND/OR is compressed-SIMD, not random
    /// per-doc forward lookups — the difference at 1M-doc filter intersection).
    terms: BTreeMap<String, RoaringBitmap>,
    forward: HashMap<u32, String>,
    bytes: u64,
}

#[derive(Debug, Default)]
struct NumberIndex {
    values: BTreeMap<SortableF64, RoaringBitmap>,
    forward: HashMap<u32, SortableF64>,
    bytes: u64,
}

#[derive(Debug, Default)]
struct SetIndex {
    elements: BTreeMap<String, RoaringBitmap>,
    forward: HashMap<u32, BTreeSet<String>>,
    bytes: u64,
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
    forward: HashMap<u32, u64>,
    bytes: u64,
}

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
            FieldIndex::Text { idx, .. } => idx.tokens.len() as u64,
            FieldIndex::Keyword(k) => k.terms.len() as u64,
            FieldIndex::Number(n) => n.values.len() as u64,
            FieldIndex::Set(s) => s.elements.len() as u64,
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
                let Some((tokens, doc_len)) = idx.forward.remove(&id) else {
                    return 0;
                };
                let mut freed = 0u64;
                for tok in &tokens {
                    if let Some(map) = idx.tokens.get_mut(tok) {
                        if map.remove(&id).is_some() {
                            freed += (tok.len() + eid.len()) as u64;
                        }
                        if map.is_empty() {
                            idx.tokens.remove(tok);
                        }
                    }
                }
                idx.doc_count = idx.doc_count.saturating_sub(1);
                idx.total_doc_len = idx.total_doc_len.saturating_sub(doc_len as u64);
                idx.bytes = idx.bytes.saturating_sub(freed);
                freed
            }
            FieldIndex::Keyword(k) => {
                let Some(value) = k.forward.remove(&id) else {
                    return 0;
                };
                let mut freed = 0u64;
                if let Some(set) = k.terms.get_mut(&value) {
                    if set.remove(id) {
                        freed = (value.len() + eid.len()) as u64;
                    }
                    if set.is_empty() {
                        k.terms.remove(&value);
                    }
                }
                k.bytes = k.bytes.saturating_sub(freed);
                freed
            }
            FieldIndex::Number(n) => {
                let Some(key) = n.forward.remove(&id) else {
                    return 0;
                };
                let mut freed = 0u64;
                if let Some(set) = n.values.get_mut(&key) {
                    if set.remove(id) {
                        freed = (8 + eid.len()) as u64;
                    }
                    if set.is_empty() {
                        n.values.remove(&key);
                    }
                }
                n.bytes = n.bytes.saturating_sub(freed);
                freed
            }
            FieldIndex::Set(s) => {
                let Some(elems) = s.forward.remove(&id) else {
                    return 0;
                };
                let mut freed = 0u64;
                for el in &elems {
                    if let Some(set) = s.elements.get_mut(el) {
                        if set.remove(id) {
                            freed += (el.len() + eid.len()) as u64;
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

#[derive(Debug)]
struct Collection {
    version: u32,
    schema: BTreeMap<String, FieldSpec>,
    fields: BTreeMap<String, FieldIndex>,
    /// external_id ↔ dense u32 doc-id. Posting lists carry the u32.
    interner: Interner,
    /// Tracks which fields each doc-id wrote into — supports
    /// "delete all fields for this eid".
    eid_fields: HashMap<u32, BTreeSet<String>>,
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
    /// Monotonic access tick (collection-LRU). Bumped on every search/index;
    /// the lowest tick is evicted first when over the memory budget. Atomic so
    /// it can be touched under a read lock.
    last_access: AtomicU64,
}

impl Collection {
    /// Approximate resident RAM: the per-field byte counters plus the
    /// interner's external_id strings and the per-doc field-set map. Used by
    /// the LRU to decide when to evict.
    fn resident_bytes(&self) -> u64 {
        let fields: u64 = self.fields.values().map(|f| f.bytes()).sum();
        let interner: u64 = self
            .interner
            .to_eid
            .iter()
            .map(|s| s.len() as u64 + 32)
            .sum();
        let eid_fields = self.eid_fields.len() as u64 * 48;
        fields + interner + eid_fields
    }
}

impl Collection {
    fn new(schema: BTreeMap<String, FieldSpec>) -> Result<Self> {
        let mut fields = BTreeMap::new();
        for (name, spec) in &schema {
            fields.insert(name.clone(), FieldIndex::from_spec(spec)?);
        }
        Ok(Self {
            version: 1,
            schema,
            fields,
            interner: Interner::default(),
            eid_fields: HashMap::new(),
            seen_requests: VecDeque::new(),
            deleted_at: None,
            last_indexed_at: None,
            last_access: AtomicU64::new(0),
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
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct Engine {
    state: RwLock<EngineState>,
    metrics: Metrics,
    draining: AtomicBool,
    /// Optional persistent backend. When set:
    ///
    /// * `keyword` and `number` writes go through the backend (the
    ///   in-memory mirror is a write-through cache, rebuilt on cold
    ///   start by [`Engine::new_with_backend`]).
    /// * `keyword` and `number` reads (`eval_term`, `eval_range`,
    ///   `duplicates`) go through the backend.
    /// * Schema mutations are mirrored into the reserved
    ///   `SCHEMA_PARTITION` so cold restart can rebuild the schema
    ///   map before re-hydrating posting caches.
    /// * `text` / `set` / `vector` writes fail with
    ///   [`StorageError::BackendNotSupported`] — v1 only covers
    ///   keyword + number on the LSM path.
    backend: Option<SharedBackend>,
    /// collection-LRU: resident-RAM budget in bytes. When set (and `evict_dir`
    /// is too), the least-recently-accessed collections are snapshotted to disk
    /// and dropped once total resident bytes exceed this. `None` ⇒ never evict
    /// (pure in-memory, back-compat).
    mem_budget: Option<u64>,
    /// Directory holding per-collection eviction snapshots.
    evict_dir: Option<std::path::PathBuf>,
    /// Monotonic LRU clock; each access stamps a collection's `last_access`.
    access_tick: AtomicU64,
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine")
            .field("draining", &self.draining)
            .field("backend", &self.backend.as_ref().map(|_| "<backend>"))
            .finish()
    }
}

#[derive(Debug, Default)]
struct EngineState {
    collections: BTreeMap<String, Collection>,
    /// collection-LRU: names of collections snapshotted to disk + dropped from
    /// `collections`. A search/index on one restores it on demand.
    evicted: BTreeSet<String>,
}

impl Engine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct an engine that reads and writes through `backend`.
    ///
    /// Performs cold-start recovery synchronously:
    ///
    /// 1. `backend.recover()` returns every live posting; we split
    ///    the dump into schema entries (partition 255) and data
    ///    entries.
    /// 2. The schema partition rebuilds the per-collection
    ///    `CreateCollectionRequest` map, so we can construct empty
    ///    [`Collection`] objects with their original field types
    ///    before any posting lands.
    /// 3. Data postings (keyword + number only — those are the only
    ///    field types the backend stores in v1) re-hydrate the
    ///    in-memory mirror caches inside [`KeywordIndex`] /
    ///    [`NumberIndex`].
    ///
    /// For collections with millions of postings this scan is
    /// one-shot at boot; future versions can defer-load via a
    /// metadata SSTable, but v1 keeps the simpler contract.
    pub fn new_with_backend(backend: SharedBackend) -> Result<Self> {
        let recovered = backend.recover()?;
        let collections = rebuild_state_from_recovery(recovered)?;
        Ok(Self {
            state: RwLock::new(EngineState {
                collections,
                evicted: BTreeSet::new(),
            }),
            metrics: Metrics::default(),
            draining: AtomicBool::new(false),
            backend: Some(backend),
            mem_budget: None,
            evict_dir: None,
            access_tick: AtomicU64::new(0),
        })
    }

    /// In-memory engine with collection-LRU enabled: total resident RAM is
    /// capped at `budget_bytes`; least-recently-accessed collections are
    /// snapshotted under `evict_dir` and dropped, restored on next access.
    pub fn new_lru(budget_bytes: u64, evict_dir: std::path::PathBuf) -> Self {
        Self {
            mem_budget: Some(budget_bytes),
            evict_dir: Some(evict_dir),
            ..Self::default()
        }
    }

    fn next_tick(&self) -> u64 {
        self.access_tick.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// On-disk path of a collection's eviction snapshot (name hex-encoded so any
    /// collection id is filename-safe).
    fn evict_path(&self, name: &str) -> Option<std::path::PathBuf> {
        let hex: String = name.bytes().map(|b| format!("{b:02x}")).collect();
        self.evict_dir
            .as_ref()
            .map(|d| d.join(format!("{hex}.snap")))
    }

    /// Ensure `name` is resident: if it was evicted to disk, restore it
    /// (read→write re-lock, re-checking under the write lock to guard against a
    /// concurrent double-restore). After a restore, re-checks the budget.
    fn ensure_resident(&self, name: &str) -> Result<()> {
        {
            let s = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
            if s.collections.contains_key(name) || !s.evicted.contains(name) {
                return Ok(());
            }
        }
        let path = self
            .evict_path(name)
            .ok_or_else(|| anyhow!("collection-LRU: no evict_dir configured"))?;
        let restored = {
            let mut s = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
            if s.collections.contains_key(name) {
                false // another thread restored it first
            } else if s.evicted.remove(name) {
                let bytes = std::fs::read(&path)
                    .map_err(|e| anyhow!("read eviction snapshot {}: {e}", path.display()))?;
                let snap: CollectionSnapshot = serde_json::from_slice(&bytes)?;
                let coll = Collection::from_snapshot(snap)?;
                coll.last_access.store(self.next_tick(), Ordering::Relaxed);
                s.collections.insert(name.to_string(), coll);
                let _ = std::fs::remove_file(&path);
                true
            } else {
                false
            }
        };
        if restored {
            self.maybe_evict()?;
        }
        Ok(())
    }

    /// If over the RAM budget, evict least-recently-accessed collections to disk
    /// until under budget (always keeping ≥1 resident). Called after memory
    /// grows (index, create, restore). No-op unless both budget + evict_dir set.
    fn maybe_evict(&self) -> Result<()> {
        let (Some(budget), Some(_)) = (self.mem_budget, self.evict_dir.as_ref()) else {
            return Ok(());
        };
        let mut s = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        loop {
            let total: u64 = s.collections.values().map(|c| c.resident_bytes()).sum();
            if total <= budget || s.collections.len() <= 1 {
                break;
            }
            let victim = s
                .collections
                .iter()
                .filter(|(_, c)| c.deleted_at.is_none())
                .min_by_key(|(_, c)| c.last_access.load(Ordering::Relaxed))
                .map(|(k, _)| k.clone());
            let Some(name) = victim else { break };
            let coll = s.collections.remove(&name).expect("victim present");
            let path = self
                .evict_path(&name)
                .ok_or_else(|| anyhow!("collection-LRU: no evict_dir configured"))?;
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let snap = coll.to_snapshot()?;
            std::fs::write(&path, serde_json::to_vec(&snap)?)
                .map_err(|e| anyhow!("write eviction snapshot {}: {e}", path.display()))?;
            s.evicted.insert(name);
        }
        Ok(())
    }

    /// Currently-resident (in-RAM) collection names. (collection-LRU
    /// observability / tests.)
    pub fn resident_names(&self) -> Vec<String> {
        self.state
            .read()
            .map(|s| s.collections.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Evicted (on-disk) collection names. (collection-LRU observability / tests.)
    pub fn evicted_names(&self) -> Vec<String> {
        self.state
            .read()
            .map(|s| s.evicted.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Borrow the attached backend, if any. Tests and admin tooling
    /// use this; the API layer does not.
    pub fn backend(&self) -> Option<&SharedBackend> {
        self.backend.as_ref()
    }

    /// Persist a collection's schema into the backend so cold recovery
    /// can rebuild it. No-op when no backend is attached.
    fn persist_schema(
        &self,
        collection_id: &str,
        schema: &BTreeMap<String, FieldSpec>,
    ) -> Result<()> {
        let Some(backend) = self.backend.as_ref() else {
            return Ok(());
        };
        let req = CreateCollectionRequest {
            fields: schema.clone(),
        };
        let payload = serde_json::to_vec(&req)
            .map_err(|e| anyhow!("encode schema for `{collection_id}`: {e}"))?;
        backend.put_posting(
            collection_id,
            SCHEMA_PARTITION,
            SCHEMA_KEY,
            SCHEMA_EID,
            &payload,
        )?;
        Ok(())
    }

    /// Remove the persisted schema entry for `collection_id`. No-op
    /// when no backend is attached.
    fn forget_schema(&self, collection_id: &str) -> Result<()> {
        let Some(backend) = self.backend.as_ref() else {
            return Ok(());
        };
        backend.delete_posting(collection_id, SCHEMA_PARTITION, SCHEMA_KEY, SCHEMA_EID)?;
        Ok(())
    }

    /// Scan every posting under `collection_id` / `field_name` and
    /// remove it from the backend. Used when a field is dropped from a
    /// live collection — the backend would otherwise resurrect the
    /// data on the next cold recovery.
    fn purge_field_from_backend(
        &self,
        collection_id: &str,
        field_name: &str,
        field_type: FieldType,
    ) -> Result<()> {
        let Some(backend) = self.backend.as_ref() else {
            return Ok(());
        };
        // Only keyword + number live in the backend in v1.
        if !matches!(field_type, FieldType::Keyword | FieldType::Number) {
            return Ok(());
        }
        let partition = partition_for_field(field_name);
        let lo = number_field_lo(field_name);
        let hi = number_field_hi(field_name);
        let rows = backend.scan_range(collection_id, partition, Some(&lo), Some(&hi))?;
        for (key, entries) in rows {
            for entry in entries {
                backend.delete_posting(collection_id, partition, &key, &entry.external_id)?;
            }
        }
        Ok(())
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
                coll.version += 1;
            }
            let schema_to_persist = coll.schema.clone();
            let resp = CreateCollectionResponse {
                collection_id: collection_id.to_string(),
                version: coll.version,
                fields_count: coll.fields_count(),
            };
            drop(state);
            if added > 0 {
                self.persist_schema(collection_id, &schema_to_persist)?;
            }
            return Ok(resp);
        }

        let coll = Collection::new(schema)?;
        let version = coll.version;
        let fields_count = coll.fields_count();
        let schema_to_persist = coll.schema.clone();
        state.collections.insert(collection_id.to_string(), coll);
        drop(state);
        self.persist_schema(collection_id, &schema_to_persist)?;
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
        // For a physical drop we need to know the schema (to enumerate
        // fields to purge from the backend) before we remove the
        // collection from the in-memory state map. Collect the work
        // under the write lock, release it, then talk to the backend.
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        let Some(coll) = state.collections.get_mut(collection_id) else {
            return Ok(DropOutcome::NotFound);
        };
        if force {
            let schema = coll.schema.clone();
            state.collections.remove(collection_id);
            drop(state);
            if self.backend.is_some() {
                for (field_name, spec) in &schema {
                    self.purge_field_from_backend(collection_id, field_name, spec.field_type)?;
                }
                self.forget_schema(collection_id)?;
            }
            return Ok(DropOutcome::Physical);
        }
        if coll.deleted_at.is_some() {
            return Ok(DropOutcome::AlreadyMarked);
        }
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
        let dropped_type = coll
            .schema
            .get(field_name)
            .map(|s| s.field_type)
            .ok_or_else(|| StorageError::UnknownField {
                collection: collection_id.to_string(),
                field: field_name.to_string(),
            })?;
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
        let schema_to_persist = coll.schema.clone();
        drop(state);
        if self.backend.is_some() {
            self.purge_field_from_backend(collection_id, field_name, dropped_type)?;
            self.persist_schema(collection_id, &schema_to_persist)?;
        }
        Ok(new_version)
    }

    /// Physically remove every collection whose `deleted_at` is older
    /// than `grace`. Returns the number of physically removed entries.
    pub fn sweep_deleted(&self, grace: Duration) -> Result<usize> {
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        let now = Instant::now();
        let to_remove: Vec<(String, BTreeMap<String, FieldSpec>)> = state
            .collections
            .iter()
            .filter_map(|(id, c)| {
                let ts = c.deleted_at?;
                if now.duration_since(ts) >= grace {
                    Some((id.clone(), c.schema.clone()))
                } else {
                    None
                }
            })
            .collect();
        let n = to_remove.len();
        for (id, _) in &to_remove {
            state.collections.remove(id);
        }
        drop(state);
        if self.backend.is_some() {
            for (id, schema) in &to_remove {
                for (field_name, spec) in schema {
                    self.purge_field_from_backend(id, field_name, spec.field_type)?;
                }
                self.forget_schema(id)?;
            }
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
        coll.schema = new_schema;
        let idx = FieldIndex::from_spec(&spec)?;
        idx.add_field();
        coll.fields.insert(field_name.to_string(), idx);
        coll.version += 1;
        let new_version = coll.version;
        let schema_to_persist = coll.schema.clone();
        drop(state);
        self.persist_schema(collection_id, &schema_to_persist)?;
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
        if req.items.len() > MAX_INDEX_ITEMS {
            return Err(StorageError::BulkLimit {
                got: req.items.len(),
                max: MAX_INDEX_ITEMS,
            }
            .into());
        }
        // collection-LRU: page back in if evicted before mutating.
        self.ensure_resident(collection_id)?;
        let mut state = self.state.write().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get_mut(collection_id)
            .ok_or_else(|| StorageError::CollectionNotFound(collection_id.to_string()))?;
        coll.check_live(collection_id)?;
        coll.last_access.store(self.next_tick(), Ordering::Relaxed); // LRU touch

        if coll.check_request_id(req.request_id.as_deref()) {
            return Ok(IndexResponse {
                indexed: 0,
                bytes_written: BTreeMap::new(),
                shard_lag_ms: 0,
            });
        }

        let mut bytes_written: BTreeMap<String, u64> = BTreeMap::new();
        let mut indexed = 0u32;

        for item in req.items {
            let spec = coll.schema.get(&item.field).cloned().ok_or_else(|| {
                StorageError::UnknownField {
                    collection: collection_id.to_string(),
                    field: item.field.clone(),
                }
            })?;
            let id = coll.interner.intern(&item.external_id);
            // Drop any existing posting for (eid, field) before reapply
            // — re-indexing is a full replacement at field granularity.
            if let Some(fi) = coll.fields.get_mut(&item.field) {
                drop_eid_with_backend(
                    fi,
                    id,
                    &item.external_id,
                    collection_id,
                    &item.field,
                    self.backend.as_ref(),
                )?;
            }
            let bytes = apply_value(
                coll.fields.get_mut(&item.field).expect("field present"),
                &spec,
                id,
                &item.external_id,
                &item.value,
                &item.field,
                collection_id,
                self.backend.as_ref(),
            )?;
            coll.eid_fields
                .entry(id)
                .or_default()
                .insert(item.field.clone());
            *bytes_written.entry(item.field.clone()).or_default() += bytes;
            indexed += 1;
        }

        let total_bytes: u64 = bytes_written.values().sum();
        if indexed > 0 {
            coll.last_indexed_at = Some(std::time::SystemTime::now());
        }
        self.metrics.incr_index(indexed as u64, total_bytes);
        let resp = IndexResponse {
            indexed,
            bytes_written,
            shard_lag_ms: 0,
        };
        // Memory grew → drop the write lock, then enforce the LRU budget.
        drop(state);
        self.maybe_evict()?;
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

        let backend = self.backend.as_ref();
        // Unknown external_id → never interned → nothing to delete.
        let Some(id) = coll.interner.id(external_id) else {
            return Ok(());
        };
        match field {
            Some(f) => {
                if let Some(fi) = coll.fields.get_mut(f) {
                    drop_eid_with_backend(fi, id, external_id, collection_id, f, backend)?;
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
                        drop_eid_with_backend(fi, id, external_id, collection_id, &f, backend)?;
                    }
                }
                coll.eid_fields.remove(&id);
            }
        }
        Ok(())
    }

    // -- Search -------------------------------------------------------------

    pub fn search(&self, collection_id: &str, req: SearchRequest) -> Result<SearchResponse> {
        let start = Instant::now();
        // DoS guard: reject pathological query trees before evaluation.
        validate_query(&req.query)?;
        // collection-LRU: page the collection back in from disk if it was evicted.
        self.ensure_resident(collection_id)?;
        // ...and any child collections referenced by `has_child` clauses, so a
        // nested-group query against an evicted child restores it too. Must run
        // before the read lock below — `ensure_resident` takes the write lock.
        {
            let mut children = Vec::new();
            collect_child_collections(&req.query, &mut children);
            for child in children {
                self.ensure_resident(&child)?;
            }
        }
        let state = self.state.read().map_err(|_| anyhow!("state poisoned"))?;
        let coll = state
            .collections
            .get(collection_id)
            .ok_or_else(|| StorageError::CollectionNotFound(collection_id.to_string()))?;
        coll.check_live(collection_id)?;
        coll.last_access.store(self.next_tick(), Ordering::Relaxed); // LRU touch

        let interner = &coll.interner;
        let offset = req.cursor.as_deref().and_then(parse_cursor).unwrap_or(0) as usize;
        let limit = req.limit as usize;

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
            if self.backend.is_none()
                && offset == 0
                && !req.track_total
                && query_is_constant_score(&req.query)
            {
                if let Some(iter) = collapse_driver(coll, &req.query) {
                    let want = limit.max(1);
                    let mut seen: HashMap<&str, ()> = HashMap::new();
                    let mut order: Vec<&str> = Vec::with_capacity(want);
                    for doc in iter {
                        if order.len() >= want {
                            break;
                        }
                        if query_predicate(coll, &req.query, doc)? {
                            if let Some(val) = kidx.forward.get(&doc) {
                                if seen.insert(val.as_str(), ()).is_none() {
                                    order.push(val.as_str());
                                }
                            }
                        }
                    }
                    let hits: Vec<SearchHit> = order
                        .into_iter()
                        .map(|val| SearchHit {
                            external_id: val.to_string(),
                            score: 1.0,
                        })
                        .collect();
                    let total = hits.len() as u64; // lower bound (track_total=false)
                    let el = start.elapsed();
                    let took_ms = el.as_millis() as u64;
                    self.metrics.observe_search(took_ms);
                    return Ok(SearchResponse {
                        hits,
                        total,
                        cursor: None,
                        took_ms,
                        took_us: el.as_micros() as u64,
                    });
                }
            }

            let universe: BTreeSet<u32> = if query_needs_universe(&req.query) {
                coll.eid_fields.keys().copied().collect()
            } else {
                BTreeSet::new()
            };
            let scored = eval_query(
                coll,
                collection_id,
                &req.query,
                &universe,
                self.backend.as_ref(),
                &state,
            )?;
            // Group by the collapse value, keeping the max member score. Docs
            // with no value for the collapse field drop out (no group).
            let mut groups: HashMap<&str, f32> = HashMap::new();
            for (doc, score) in &scored {
                if let Some(val) = kidx.forward.get(doc) {
                    let slot = groups.entry(val.as_str()).or_insert(f32::NEG_INFINITY);
                    if *score > *slot {
                        *slot = *score;
                    }
                }
            }
            let total = groups.len() as u64;
            // Rank groups by score desc, then value asc; partition top-k.
            let cmp = |a: &(&str, f32), b: &(&str, f32)| {
                b.1.partial_cmp(&a.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.0.cmp(b.0))
            };
            let mut ranked: Vec<(&str, f32)> = groups.into_iter().collect();
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
            return Ok(SearchResponse {
                hits,
                total,
                cursor,
                took_ms,
                took_us: el.as_micros() as u64,
            });
        }

        // Planner fast paths (embedded mode, first page): sort-by-field and
        // standalone range early-terminate / avoid materializing a wide clause,
        // returning the final page directly. Everything else falls through to
        // the materialize-and-rank path below (identical result to before).
        let planned = if self.backend.is_none() {
            try_plan(coll, &req, offset)?
        } else {
            None
        };

        let (page, total): (Vec<(u32, f32)>, u64) = match planned {
            Some(pt) => pt,
            None => {
                // The full eid set ("universe") is only consumed by the `Not`
                // branch of eval_query; build it only when the query needs it.
                let universe: BTreeSet<u32> = if query_needs_universe(&req.query) {
                    coll.eid_fields.keys().copied().collect()
                } else {
                    BTreeSet::new()
                };
                let scored = eval_query(
                    coll,
                    collection_id,
                    &req.query,
                    &universe,
                    self.backend.as_ref(),
                    &state,
                )?;
                let total = scored.len() as u64;

                // Rank by score desc, then external_id asc (tie-break on the
                // resolved string, stable across snapshot rebuilds). Partition
                // the top-k to the front in O(n), then sort just that slice.
                let cmp = |a: &(u32, f32), b: &(u32, f32)| {
                    b.1.partial_cmp(&a.1)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| interner.resolve(a.0).cmp(interner.resolve(b.0)))
                };
                let mut ranked: Vec<(u32, f32)> = scored.into_iter().collect();
                let k = offset.saturating_add(limit);
                if k > 0 && ranked.len() > k {
                    ranked.select_nth_unstable_by(k - 1, cmp);
                    ranked.truncate(k);
                }
                ranked.sort_by(cmp);
                let page = ranked.into_iter().skip(offset).take(limit).collect();
                (page, total)
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

        // `total` is the full match count, so the "more results?" check holds
        // even when the page was truncated to the top-k.
        let next_offset = offset + hits.len();
        let cursor = if (next_offset as u64) < total {
            Some(make_cursor(next_offset))
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
        let backend = self.backend.as_ref();
        let interner = &coll.interner;
        let mut groups: Vec<DuplicateGroup> = match fi {
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
                if let Some(backend) = backend {
                    let partition = partition_for_field(&req.field);
                    let lo = number_field_lo(&req.field);
                    let hi = number_field_hi(&req.field);
                    let rows =
                        backend.scan_range(collection_id, partition, Some(&lo), Some(&hi))?;
                    rows.into_iter()
                        .filter_map(|(k_bytes, entries)| {
                            let value = decode_keyword_key(&req.field, &k_bytes)?;
                            let s = std::str::from_utf8(value).ok()?.to_string();
                            if entries.len() < min {
                                return None;
                            }
                            Some(DuplicateGroup {
                                value: serde_json::Value::String(s),
                                external_ids: entries.into_iter().map(|e| e.external_id).collect(),
                            })
                        })
                        .collect()
                } else {
                    k.terms
                        .iter()
                        .filter(|(_, set)| (set.len() as usize) >= min)
                        .map(|(v, set)| DuplicateGroup {
                            value: serde_json::Value::String(v.clone()),
                            external_ids: set
                                .iter()
                                .map(|id| interner.resolve(id).to_string())
                                .collect(),
                        })
                        .collect()
                }
            }
            FieldIndex::Number(n) => {
                if let Some(backend) = backend {
                    let partition = partition_for_field(&req.field);
                    let lo = number_field_lo(&req.field);
                    let hi = number_field_hi(&req.field);
                    let rows =
                        backend.scan_range(collection_id, partition, Some(&lo), Some(&hi))?;
                    rows.into_iter()
                        .filter_map(|(k_bytes, entries)| {
                            let key = decode_number_key(&req.field, &k_bytes)?;
                            if entries.len() < min {
                                return None;
                            }
                            Some(DuplicateGroup {
                                value: serde_json::Value::Number(
                                    serde_json::Number::from_f64(key.to_f64())
                                        .unwrap_or_else(|| serde_json::Number::from(0)),
                                ),
                                external_ids: entries.into_iter().map(|e| e.external_id).collect(),
                            })
                        })
                        .collect()
                } else {
                    n.values
                        .iter()
                        .filter(|(_, set)| (set.len() as usize) >= min)
                        .map(|(v, set)| DuplicateGroup {
                            value: serde_json::Value::Number(
                                serde_json::Number::from_f64(v.to_f64())
                                    .unwrap_or_else(|| serde_json::Number::from(0)),
                            ),
                            external_ids: set
                                .iter()
                                .map(|id| interner.resolve(id).to_string())
                                .collect(),
                        })
                        .collect()
                }
            }
            FieldIndex::Set(s) => s
                .elements
                .iter()
                .filter(|(_, set)| (set.len() as usize) >= min)
                .map(|(v, set)| DuplicateGroup {
                    value: serde_json::Value::String(v.clone()),
                    external_ids: set
                        .iter()
                        .map(|id| interner.resolve(id).to_string())
                        .collect(),
                })
                .collect(),
        };
        // Stable: largest groups first, ties broken by value.
        groups.sort_by(|a, b| {
            b.external_ids
                .len()
                .cmp(&a.external_ids.len())
                .then_with(|| a.value.to_string().cmp(&b.value.to_string()))
        });

        let offset = req.offset as usize;
        let limit = req.limit.max(1) as usize;
        let total = groups.len();
        let page: Vec<DuplicateGroup> = groups.into_iter().skip(offset).take(limit).collect();
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
    _spec: &FieldSpec,
    id: u32,
    eid: &str,
    value: &FieldValue,
    field_name: &str,
    collection_id: &str,
    backend: Option<&SharedBackend>,
) -> Result<u64> {
    // v1 LSM read-path scope (see module docs): only keyword + number
    // route through the backend. Reject the other field types loudly
    // when a backend is attached so callers can't silently lose data.
    if backend.is_some() {
        match fi {
            FieldIndex::Text { .. } => {
                return Err(StorageError::BackendNotSupported {
                    field_type: FieldType::Text,
                    field_name: field_name.to_string(),
                }
                .into());
            }
            FieldIndex::Set(_) => {
                return Err(StorageError::BackendNotSupported {
                    field_type: FieldType::Set,
                    field_name: field_name.to_string(),
                }
                .into());
            }
            FieldIndex::Vector { .. } => {
                return Err(StorageError::BackendNotSupported {
                    field_type: FieldType::Vector,
                    field_name: field_name.to_string(),
                }
                .into());
            }
            FieldIndex::Hash(_) => {
                return Err(StorageError::BackendNotSupported {
                    field_type: FieldType::Hash,
                    field_name: field_name.to_string(),
                }
                .into());
            }
            FieldIndex::Keyword(_) | FieldIndex::Number(_) => {}
        }
    }
    match (fi, value) {
        (FieldIndex::Text { analyzer, idx }, FieldValue::String(s)) => {
            let tokens = tokenize::tokenize(s, *analyzer);
            let doc_len = tokens.len() as u32;
            let mut tf: BTreeMap<String, u32> = BTreeMap::new();
            for tok in &tokens {
                *tf.entry(tok.clone()).or_default() += 1;
            }
            let mut bytes = 0u64;
            let distinct: BTreeSet<String> = tf.keys().cloned().collect();
            for (tok, count) in tf {
                bytes += (tok.len() + eid.len()) as u64;
                idx.tokens.entry(tok).or_default().insert(id, count);
            }
            idx.forward.insert(id, (distinct, doc_len));
            idx.doc_count += 1;
            idx.total_doc_len += doc_len as u64;
            idx.bytes += bytes;
            Ok(bytes)
        }
        (FieldIndex::Keyword(k), FieldValue::String(s)) => {
            let bytes = (s.len() + eid.len()) as u64;
            if let Some(backend) = backend {
                let partition = partition_for_field(field_name);
                let key = keyword_backend_key(field_name, s);
                backend.put_posting(collection_id, partition, &key, eid, &[])?;
            }
            k.terms.entry(s.clone()).or_default().insert(id);
            k.forward.insert(id, s.clone());
            k.bytes += bytes;
            Ok(bytes)
        }
        (FieldIndex::Number(n), FieldValue::Number(x)) => {
            let key =
                SortableF64::new(*x).map_err(|e| StorageError::InvalidNumber(e.to_string()))?;
            let bytes = (8 + eid.len()) as u64;
            if let Some(backend) = backend {
                let partition = partition_for_field(field_name);
                let bk = number_backend_key(field_name, key);
                backend.put_posting(collection_id, partition, &bk, eid, &[])?;
            }
            n.values.entry(key).or_default().insert(id);
            n.forward.insert(id, key);
            n.bytes += bytes;
            Ok(bytes)
        }
        (FieldIndex::Set(s), FieldValue::StringList(elems)) => {
            let mut bytes = 0u64;
            let mut seen = BTreeSet::new();
            for el in elems {
                if seen.insert(el.clone()) {
                    bytes += (el.len() + eid.len()) as u64;
                    s.elements.entry(el.clone()).or_default().insert(id);
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

/// Backend-aware version of [`FieldIndex::drop_eid`]. Honours the same
/// in-memory bookkeeping and additionally issues a tombstone on the
/// backend for keyword + number fields when one is attached.
fn drop_eid_with_backend(
    fi: &mut FieldIndex,
    id: u32,
    eid: &str,
    collection_id: &str,
    field_name: &str,
    backend: Option<&SharedBackend>,
) -> Result<u64> {
    // Capture the prior backend key (if any) before we mutate the
    // forward map — `FieldIndex::drop_eid` consumes it.
    let prior_backend_key: Option<Vec<u8>> = match (&fi, backend) {
        (FieldIndex::Keyword(k), Some(_)) => k
            .forward
            .get(&id)
            .map(|s| keyword_backend_key(field_name, s)),
        (FieldIndex::Number(n), Some(_)) => n
            .forward
            .get(&id)
            .map(|key| number_backend_key(field_name, *key)),
        _ => None,
    };
    let freed = fi.drop_eid(id, eid);
    if let (Some(key), Some(backend)) = (prior_backend_key, backend) {
        let partition = partition_for_field(field_name);
        backend.delete_posting(collection_id, partition, &key, eid)?;
    }
    Ok(freed)
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

/// Collect every child-collection name referenced by a `has_child` clause in
/// the query tree (recursively — a `has_child` sub-query may itself contain
/// `has_child`). Used to page evicted child collections back in before a search
/// takes the read lock, so a nested-group query against an evicted child
/// restores it instead of failing with `CollectionNotFound`.
fn collect_child_collections(node: &QueryNode, out: &mut Vec<String>) {
    match node {
        QueryNode::And(children) | QueryNode::Or(children) => {
            for c in children {
                collect_child_collections(c, out);
            }
        }
        QueryNode::Not(child) => collect_child_collections(child, out),
        QueryNode::HasChild(hc) => {
            out.push(hc.collection.clone());
            collect_child_collections(&hc.query, out);
        }
        QueryNode::Rrf(r) => {
            for c in &r.queries {
                collect_child_collections(c, out);
            }
        }
        _ => {}
    }
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
    backend: Option<&SharedBackend>,
    state: &EngineState,
) -> Result<ScoredHits> {
    Ok(match q {
        QueryNode::Match(m) => eval_match(coll, m)?,
        QueryNode::Term(t) => constant_score(eval_term(coll, collection_id, t, backend)?),
        QueryNode::Terms(t) => constant_score(eval_terms(coll, collection_id, t, backend)?),
        QueryNode::Range(r) => constant_score(eval_range(coll, collection_id, r, backend)?),
        QueryNode::Knn(k) => eval_knn(coll, k)?,
        QueryNode::Hamming(hq) => eval_hamming(coll, hq)?,
        QueryNode::Rrf(r) => eval_rrf(coll, collection_id, r, universe, backend, state)?,
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
                            backend,
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

            // Planner fast path (embedded mode). Evaluate the AND without
            // materializing a wide clause:
            //   * ≥1 filter conjunct (term/terms/range) → INTERSECT their
            //     RoaringBitmaps (compressed-SIMD AND, smallest first), subtract
            //     filter negations, then score `match` conjuncts over the small
            //     candidate set;
            //   * match-only positives → drive from the cheapest match
            //     (bulk-scored), predicate-filter the rest.
            // Score = sum of every conjunct's contribution → byte-identical to
            // the fallback. Embedded mode only (in-memory bitmaps/forward maps).
            let predicable = backend.is_none()
                && !positives.is_empty()
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
                    // Intersect filter bitmaps smallest-first; subtract filter
                    // negations. A wide clause is AND'd word-wise (compressed),
                    // never iterated doc-by-doc.
                    let mut order = filter_pos.clone();
                    order.sort_by_key(|c| estimate_selectivity(coll, c));
                    let mut cand = eval_filter_bitmap(coll, collection_id, order[0], backend)?;
                    for c in order.iter().skip(1) {
                        if cand.is_empty() {
                            break;
                        }
                        cand &= &eval_filter_bitmap(coll, collection_id, c, backend)?;
                    }
                    for nf in &filter_nots {
                        if cand.is_empty() {
                            break;
                        }
                        cand -= &eval_filter_bitmap(coll, collection_id, nf, backend)?;
                    }
                    // Each filter / negation contributes a constant 1.0; matches
                    // add BM25 on top and gate membership.
                    let base = filter_pos.len() as f32 + nots.len() as f32;
                    let preps = prep_matches(coll, &match_pos)?;
                    let not_preps = prep_matches(coll, &match_nots)?;
                    let mut acc = ScoredHits::new();
                    'doc: for id in &cand {
                        let mut score = base;
                        for (idx, toks, op) in &preps {
                            match match_doc_score(idx, toks, *op, id) {
                                Some(s) => score += s,
                                None => continue 'doc,
                            }
                        }
                        for (idx, toks, op) in &not_preps {
                            if match_doc_score(idx, toks, *op, id).is_some() {
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
                    let scored = eval_query(coll, collection_id, driver, universe, backend, state)?;
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
                // Fallback: materialize-and-intersect. Also the only correct
                // path under an LSM backend (forward maps not populated there).
                let mut pos = positives.iter();
                let mut acc = match pos.next() {
                    Some(first) => {
                        eval_query(coll, collection_id, first, universe, backend, state)?
                    }
                    // All-negative AND: start from the universe, then trim.
                    None => constant_score(universe.iter().cloned().collect()),
                };
                for c in pos {
                    let other = eval_query(coll, collection_id, c, universe, backend, state)?;
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
                    let exclude = eval_query(coll, collection_id, inner, universe, backend, state)?;
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
                for (eid, score) in eval_query(coll, collection_id, c, universe, backend, state)? {
                    *acc.entry(eid).or_insert(0.0) += score;
                }
            }
            acc
        }
        QueryNode::Not(child) => {
            let inner = eval_query(coll, collection_id, child, universe, backend, state)?;
            universe
                .iter()
                .filter(|id| !inner.contains_key(*id))
                .map(|id| (*id, 1.0))
                .collect()
        }
        QueryNode::HasChild(hc) => eval_has_child(coll, hc, backend, state)?,
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
    backend: Option<&SharedBackend>,
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
    let matched = eval_query(
        child,
        &hc.collection,
        &hc.query,
        &child_universe,
        backend,
        state,
    )?;
    // Distinct parent external_ids → PARENT docids → constant-score set.
    let mut parents = RoaringBitmap::new();
    for (child_doc, _) in &matched {
        if let Some(pid) = kidx.forward.get(child_doc) {
            if let Some(parent_doc) = parent.interner.id(pid) {
                parents.insert(parent_doc);
            }
        }
    }
    Ok(constant_score(parents))
}

fn constant_score(set: RoaringBitmap) -> ScoredHits {
    set.into_iter().map(|id| (id, 1.0)).collect()
}

/// BM25 over the text field. Implements the standard form:
///
/// ```text
/// score = Σ_t IDF(t) · TF(t,d) · (k1+1) / (TF(t,d) + k1 · (1 − b + b · |d| / avgdl))
/// ```
fn eval_match(coll: &Collection, m: &MatchQuery) -> Result<ScoredHits> {
    const K1: f32 = 1.2;
    const B: f32 = 0.75;

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
    if tokens.is_empty() || idx.doc_count == 0 {
        return Ok(HashMap::new());
    }
    let n = idx.doc_count as f32;
    let avgdl = if idx.doc_count == 0 {
        1.0
    } else {
        idx.total_doc_len as f32 / idx.doc_count as f32
    };

    // Score one token's BM25 contribution into `out` (`+=`, so a doc matching
    // several tokens accumulates). Postings carry the dense u32 doc-id — no
    // per-match String clone, and `forward.get(&id)` is a u32 hash.
    let score_token = |tok: &str, out: &mut ScoredHits| {
        let Some(postings) = idx.tokens.get(tok) else {
            return;
        };
        let df = postings.len() as f32;
        let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
        for (id, tf) in postings {
            let doc_len = idx.forward.get(id).map(|(_, l)| *l as f32).unwrap_or(1.0);
            let tf = *tf as f32;
            let denom = tf + K1 * (1.0 - B + B * doc_len / avgdl);
            *out.entry(*id).or_insert(0.0) += idf * tf * (K1 + 1.0) / denom;
        }
    };

    Ok(match m.op {
        MatchOp::Or => {
            // Union: accumulate every token directly into one map — no
            // intermediate per-token maps to build and then merge.
            let mut acc: ScoredHits = HashMap::new();
            for tok in &tokens {
                score_token(tok, &mut acc);
            }
            acc
        }
        MatchOp::And => {
            // Intersect: each token scored into its own map, then intersected.
            // Take the first map by value (no clone — matters for the common
            // single-token case, which is then just that map).
            let mut per_token: Vec<ScoredHits> = Vec::with_capacity(tokens.len());
            for tok in &tokens {
                let mut scored = ScoredHits::default();
                score_token(tok, &mut scored);
                per_token.push(scored);
            }
            let mut iter = per_token.into_iter();
            let Some(mut acc) = iter.next() else {
                return Ok(HashMap::new());
            };
            for other in iter {
                acc = acc
                    .into_iter()
                    .filter_map(|(id, score)| other.get(&id).map(|s| (id, score + s)))
                    .collect();
                if acc.is_empty() {
                    break;
                }
            }
            acc
        }
    })
}

fn eval_term(
    coll: &Collection,
    collection_id: &str,
    t: &TermQuery,
    backend: Option<&SharedBackend>,
) -> Result<RoaringBitmap> {
    let fi = coll
        .fields
        .get(&t.field)
        .ok_or_else(|| StorageError::UnknownField {
            collection: "<>".into(),
            field: t.field.clone(),
        })?;
    // Backend postings come back as String external_ids; map them to the
    // collection's dense doc-ids (an eid not yet interned has no postings).
    Ok(match (fi, &t.value) {
        (FieldIndex::Keyword(k), FieldValue::String(s)) => {
            if let Some(backend) = backend {
                let partition = partition_for_field(&t.field);
                let key = keyword_backend_key(&t.field, s);
                backend
                    .posting(collection_id, partition, &key)?
                    .into_iter()
                    .filter_map(|e| coll.interner.id(&e.external_id))
                    .collect()
            } else {
                k.terms.get(s).cloned().unwrap_or_default()
            }
        }
        (FieldIndex::Number(n), FieldValue::Number(x)) => {
            let key = SortableF64::new(*x)?;
            if let Some(backend) = backend {
                let partition = partition_for_field(&t.field);
                let bk = number_backend_key(&t.field, key);
                backend
                    .posting(collection_id, partition, &bk)?
                    .into_iter()
                    .filter_map(|e| coll.interner.id(&e.external_id))
                    .collect()
            } else {
                n.values.get(&key).cloned().unwrap_or_default()
            }
        }
        (FieldIndex::Set(s), FieldValue::String(el)) => {
            s.elements.get(el).cloned().unwrap_or_default()
        }
        _ => bail!("term query type mismatch on field `{}`", t.field),
    })
}

fn eval_terms(
    coll: &Collection,
    collection_id: &str,
    t: &TermsQuery,
    backend: Option<&SharedBackend>,
) -> Result<RoaringBitmap> {
    let mut acc = RoaringBitmap::new();
    // Fast path: no backend → union the in-memory posting bitmaps by reference
    // (word-wise OR, no per-value clone).
    if backend.is_none() {
        if let Some(fi) = coll.fields.get(&t.field) {
            for v in &t.values {
                match (fi, v) {
                    (FieldIndex::Keyword(k), FieldValue::String(s)) => {
                        if let Some(set) = k.terms.get(s) {
                            acc |= set;
                        }
                    }
                    (FieldIndex::Set(se), FieldValue::String(el)) => {
                        if let Some(set) = se.elements.get(el) {
                            acc |= set;
                        }
                    }
                    (FieldIndex::Number(nx), FieldValue::Number(x)) => {
                        let key = SortableF64::new(*x)?;
                        if let Some(set) = nx.values.get(&key) {
                            acc |= set;
                        }
                    }
                    _ => bail!("terms query type mismatch on field `{}`", t.field),
                }
            }
            return Ok(acc);
        }
    }
    for v in &t.values {
        let one = TermQuery {
            field: t.field.clone(),
            value: v.clone(),
        };
        acc |= eval_term(coll, collection_id, &one, backend)?;
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
    backend: Option<&SharedBackend>,
    state: &EngineState,
) -> Result<ScoredHits> {
    let k = r.k.max(1) as f32;
    let mut fused: ScoredHits = HashMap::new();
    for sub in &r.queries {
        let hits = eval_query(coll, collection_id, sub, universe, backend, state)?;
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
    for (&id, &doc) in &h.forward {
        let dist = (doc ^ query).count_ones();
        if dist <= max {
            hits.insert(id, (64 - dist) as f32 / 64.0);
        }
    }
    Ok(hits)
}

fn eval_range(
    coll: &Collection,
    collection_id: &str,
    r: &RangeQuery,
    backend: Option<&SharedBackend>,
) -> Result<RoaringBitmap> {
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

    if let Some(backend) = backend {
        // Backend `scan_range` is [lo, hi). We have to convert the
        // `gt` / `gte` / `lt` / `lte` bounds into a half-open range
        // expressed in bytes.
        let partition = partition_for_field(&r.field);
        let lo_key = match (r.gte, r.gt) {
            (Some(v), _) => Some(number_backend_key(&r.field, SortableF64::new(v)?)),
            (None, Some(v)) => {
                let key = SortableF64::new(v)?;
                // Strict lower bound — bump the trailing byte to skip
                // the exact value.
                let mut k = number_backend_key(&r.field, key);
                bump_be_suffix(&mut k);
                Some(k)
            }
            (None, None) => Some(number_field_lo(&r.field)),
        };
        let hi_key = match (r.lte, r.lt) {
            (Some(v), _) => {
                let key = SortableF64::new(v)?;
                // Inclusive upper bound — bump past the exact value
                // so it is still included in the half-open scan.
                let mut k = number_backend_key(&r.field, key);
                bump_be_suffix(&mut k);
                Some(k)
            }
            (None, Some(v)) => Some(number_backend_key(&r.field, SortableF64::new(v)?)),
            (None, None) => Some(number_field_hi(&r.field)),
        };
        let rows = backend.scan_range(
            collection_id,
            partition,
            lo_key.as_deref(),
            hi_key.as_deref(),
        )?;
        let mut acc = RoaringBitmap::new();
        for (_, entries) in rows {
            for e in entries {
                if let Some(id) = coll.interner.id(&e.external_id) {
                    acc.insert(id);
                }
            }
        }
        return Ok(acc);
    }

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
    let mut acc = RoaringBitmap::new();
    for (_, set) in n.values.range((low, high)) {
        acc |= set;
    }
    Ok(acc)
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
        QueryNode::Term(_) | QueryNode::Terms(_) | QueryNode::Range(_) | QueryNode::Match(_)
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
    use std::ops::Bound::*;
    let (lo, hi) = range_bounds(r)?;
    let lo_ok = match lo {
        Included(b) => v >= b,
        Excluded(b) => v > b,
        Unbounded => true,
    };
    let hi_ok = match hi {
        Included(b) => v <= b,
        Excluded(b) => v < b,
        Unbounded => true,
    };
    Ok(lo_ok && hi_ok)
}

/// Cheap upper bound on how many docs a positive conjunct matches, WITHOUT
/// materializing it — used to pick the conjunct to drive the AND from. Reads
/// only posting/bucket lengths. Returns `u64::MAX` for shapes we don't drive
/// from (so a leaf is always preferred).
fn estimate_selectivity(coll: &Collection, node: &QueryNode) -> u64 {
    match node {
        QueryNode::Term(t) => match (coll.fields.get(&t.field), &t.value) {
            (Some(FieldIndex::Keyword(k)), FieldValue::String(s)) => {
                k.terms.get(s).map(|p| p.len() as u64).unwrap_or(0)
            }
            (Some(FieldIndex::Number(n)), FieldValue::Number(x)) => SortableF64::new(*x)
                .ok()
                .and_then(|key| n.values.get(&key))
                .map(|p| p.len() as u64)
                .unwrap_or(0),
            (Some(FieldIndex::Set(s)), FieldValue::String(el)) => {
                s.elements.get(el).map(|p| p.len() as u64).unwrap_or(0)
            }
            _ => u64::MAX,
        },
        QueryNode::Terms(t) => match coll.fields.get(&t.field) {
            Some(FieldIndex::Keyword(k)) => t
                .values
                .iter()
                .map(|v| match v {
                    FieldValue::String(s) => k.terms.get(s).map(|p| p.len() as u64).unwrap_or(0),
                    _ => 0,
                })
                .sum(),
            Some(FieldIndex::Set(s)) => t
                .values
                .iter()
                .map(|v| match v {
                    FieldValue::String(el) => {
                        s.elements.get(el).map(|p| p.len() as u64).unwrap_or(0)
                    }
                    _ => 0,
                })
                .sum(),
            _ => u64::MAX,
        },
        QueryNode::Range(r) => match coll.fields.get(&r.field) {
            Some(FieldIndex::Number(n)) => match range_bounds(r) {
                Ok((lo, hi)) => n.values.range((lo, hi)).map(|(_, s)| s.len() as u64).sum(),
                Err(_) => u64::MAX,
            },
            _ => u64::MAX,
        },
        QueryNode::Match(m) => match coll.fields.get(&m.field) {
            Some(FieldIndex::Text { analyzer, idx }) => {
                let toks = tokenize::tokenize(&m.text, *analyzer);
                let dfs = toks
                    .iter()
                    .map(|t| idx.tokens.get(t).map(|p| p.len() as u64));
                match m.op {
                    // AND: a doc needs every token, so ≤ the rarest token's df.
                    MatchOp::And => dfs.map(|d| d.unwrap_or(0)).min().unwrap_or(0),
                    // OR: ≤ the sum of dfs.
                    MatchOp::Or => dfs.map(|d| d.unwrap_or(0)).sum(),
                }
            }
            _ => u64::MAX,
        },
        // Don't drive an AND from these.
        QueryNode::Knn(_)
        | QueryNode::And(_)
        | QueryNode::Or(_)
        | QueryNode::Not(_)
        | QueryNode::HasChild(_)
        | QueryNode::Hamming(_)
        | QueryNode::Rrf(_) => u64::MAX,
    }
}

/// Per-doc BM25 for a `match` conjunct, identical to [`eval_match`]'s formula,
/// so a doc scored as a predicate gets the same contribution it would as a
/// materialized clause. `None` ⇒ the doc does not satisfy the match.
fn match_doc_score(idx: &TextIndex, tokens: &[String], op: MatchOp, id: u32) -> Option<f32> {
    const K1: f32 = 1.2;
    const B: f32 = 0.75;
    if idx.doc_count == 0 || tokens.is_empty() {
        return None;
    }
    let doc_len = idx.forward.get(&id)?.1 as f32; // doc has no value for this field
    let n = idx.doc_count as f32;
    let avgdl = idx.total_doc_len as f32 / idx.doc_count as f32;
    let mut score = 0.0f32;
    let mut matched = 0usize;
    for tok in tokens {
        if let Some(postings) = idx.tokens.get(tok) {
            if let Some(tf) = postings.get(&id) {
                let df = postings.len() as f32;
                let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();
                let tf = *tf as f32;
                let denom = tf + K1 * (1.0 - B + B * doc_len / avgdl);
                score += idf * tf * (K1 + 1.0) / denom;
                matched += 1;
            }
        }
    }
    match op {
        MatchOp::And if matched == tokens.len() => Some(score),
        MatchOp::Or if matched > 0 => Some(score),
        _ => None,
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
                    k.forward.get(&id).map(|v| v == s).unwrap_or(false)
                }
                (FieldIndex::Number(n), FieldValue::Number(x)) => {
                    let key = SortableF64::new(*x)?;
                    n.forward.get(&id) == Some(&key)
                }
                (FieldIndex::Set(s), FieldValue::String(el)) => s
                    .forward
                    .get(&id)
                    .map(|set| set.contains(el))
                    .unwrap_or(false),
                _ => bail!("term query type mismatch on field `{}`", t.field),
            };
            hit.then_some(1.0)
        }
        QueryNode::Terms(t) => {
            let fi = coll.fields.get(&t.field).ok_or_else(|| unknown(&t.field))?;
            let hit = match fi {
                FieldIndex::Keyword(k) => k.forward.get(&id).is_some_and(|v| {
                    t.values
                        .iter()
                        .any(|val| matches!(val, FieldValue::String(s) if s == v))
                }),
                FieldIndex::Number(n) => n.forward.get(&id).is_some_and(|v| {
                    t.values.iter().any(|val| {
                        matches!(val, FieldValue::Number(x)
                            if SortableF64::new(*x).map(|k| &k == v).unwrap_or(false))
                    })
                }),
                FieldIndex::Set(s) => s.forward.get(&id).is_some_and(|set| {
                    t.values
                        .iter()
                        .any(|val| matches!(val, FieldValue::String(el) if set.contains(el)))
                }),
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
            match n.forward.get(&id) {
                Some(v) if in_range(*v, r)? => Some(1.0),
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
fn eval_filter_bitmap(
    coll: &Collection,
    collection_id: &str,
    node: &QueryNode,
    backend: Option<&SharedBackend>,
) -> Result<RoaringBitmap> {
    match node {
        QueryNode::Term(t) => eval_term(coll, collection_id, t, backend),
        QueryNode::Terms(t) => eval_terms(coll, collection_id, t, backend),
        QueryNode::Range(r) => eval_range(coll, collection_id, r, backend),
        _ => bail!("eval_filter_bitmap called on a non-filter node"),
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
            let posting: Option<&RoaringBitmap> = match (coll.fields.get(&t.field), &t.value) {
                (Some(FieldIndex::Keyword(k)), FieldValue::String(s)) => k.terms.get(s),
                (Some(FieldIndex::Number(n)), FieldValue::Number(x)) => {
                    SortableF64::new(*x).ok().and_then(|key| n.values.get(&key))
                }
                (Some(FieldIndex::Set(s)), FieldValue::String(el)) => s.elements.get(el),
                _ => return None,
            };
            Some(match posting {
                Some(set) => Box::new(set.iter()),
                None => Box::new(std::iter::empty()),
            })
        }
        QueryNode::Range(r) => {
            let Some(FieldIndex::Number(n)) = coll.fields.get(&r.field) else {
                return None;
            };
            let (lo, hi) = range_bounds(r).ok()?;
            Some(Box::new(
                n.values.range((lo, hi)).flat_map(|(_, s)| s.iter()),
            ))
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
fn try_plan(
    coll: &Collection,
    req: &SearchRequest,
    offset: usize,
) -> Result<Option<(Vec<(u32, f32)>, u64)>> {
    if offset != 0 {
        return Ok(None);
    }
    let want = req.limit as usize;

    // ---- sort by a single number field ----
    if let Some(sort) = &req.sort {
        let [s] = sort.as_slice() else {
            return Ok(None); // multi-key sort not specialized
        };
        let Some(FieldIndex::Number(n)) = coll.fields.get(&s.field) else {
            return Ok(None); // only number fields are sortable here
        };
        if query_has_knn(&req.query) {
            return Ok(None);
        }
        let mut page: Vec<(u32, f32)> = Vec::with_capacity(want.min(1024));
        let mut total: u64 = 0;
        // Walk docs in field-sorted order; emit those satisfying the query.
        // Score is the sort value (informational; ranking IS the walk order).
        // `track_total=false` lets us stop as soon as the page is full.
        match s.order {
            SortOrder::Asc => {
                'asc: for (v, docs) in n.values.iter() {
                    for id in docs {
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
                'desc: for (v, docs) in n.values.iter().rev() {
                    for id in docs {
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
        return Ok(Some((page, total)));
    }

    // ---- no sort: standalone term — page = first `limit` of the posting,
    // total = posting length (no HashMap build / sort of the full posting). ----
    if let QueryNode::Term(t) = &req.query {
        let posting: Option<&RoaringBitmap> = match (coll.fields.get(&t.field), &t.value) {
            (Some(FieldIndex::Keyword(k)), FieldValue::String(s)) => k.terms.get(s),
            (Some(FieldIndex::Number(n)), FieldValue::Number(x)) => match SortableF64::new(*x) {
                Ok(key) => n.values.get(&key),
                Err(_) => return Ok(None),
            },
            (Some(FieldIndex::Set(s)), FieldValue::String(el)) => s.elements.get(el),
            _ => return Ok(None), // type mismatch → fall back (eval_term reports it)
        };
        let page: Vec<(u32, f32)> = posting
            .into_iter()
            .flatten()
            .take(want)
            .map(|id| (id, 1.0))
            .collect();
        let total = posting.map(|p| p.len() as u64).unwrap_or(0);
        return Ok(Some((page, total)));
    }

    // ---- no sort: standalone range early-termination ----
    if let QueryNode::Range(r) = &req.query {
        let Some(FieldIndex::Number(n)) = coll.fields.get(&r.field) else {
            return Ok(None);
        };
        let (lo, hi) = range_bounds(r)?;
        let mut page: Vec<(u32, f32)> = Vec::with_capacity(want.min(1024));
        let mut total: u64 = 0;
        for (_v, docs) in n.values.range((lo, hi)) {
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
        return Ok(Some((page, total)));
    }

    Ok(None)
}

/// Increment a big-endian byte string by 1 with carry, used to turn
/// an inclusive bound into a strict half-open one. If the whole
/// buffer overflows we append a `0` byte (so `0xff..0xff` becomes
/// `0xff..0xff 0x00`, which sorts strictly after).
fn bump_be_suffix(bytes: &mut Vec<u8>) {
    for b in bytes.iter_mut().rev() {
        if *b == 0xff {
            *b = 0;
            continue;
        }
        *b += 1;
        return;
    }
    bytes.push(0);
}

// ---------------------------------------------------------------------------
// Backend cold-start recovery
// ---------------------------------------------------------------------------

/// Rebuild the engine's `EngineState::collections` map from the dump
/// returned by [`crate::storage_backend::Backend::recover`]. Schema
/// entries (`partition == SCHEMA_PARTITION`) are applied first so the
/// per-collection field map exists before postings land; data
/// postings then re-hydrate the in-memory mirrors in the keyword /
/// number indexes.
fn rebuild_state_from_recovery(
    recovered: Vec<RecoveredPosting>,
) -> Result<BTreeMap<String, Collection>> {
    // Partition the dump into schemas + per-(collection,field) buckets
    // so we can pay one pass over the schema set first.
    let mut schema_payloads: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut data_rows: Vec<RecoveredPosting> = Vec::new();
    for row in recovered {
        if row.partition == SCHEMA_PARTITION && row.key == SCHEMA_KEY {
            schema_payloads.insert(row.collection, row.payload);
        } else {
            data_rows.push(row);
        }
    }

    let mut collections: BTreeMap<String, Collection> = BTreeMap::new();
    for (collection_id, payload) in schema_payloads {
        let req: CreateCollectionRequest = serde_json::from_slice(&payload).map_err(|e| {
            anyhow!("decode persisted schema for collection `{collection_id}`: {e}")
        })?;
        let schema: BTreeMap<String, FieldSpec> = req
            .fields
            .into_iter()
            .map(|(k, v)| (k, v.normalize()))
            .collect();
        validate_schema(&schema)?;
        let coll = Collection::new(schema)?;
        collections.insert(collection_id, coll);
    }

    // Re-hydrate keyword + number in-memory mirrors from the data
    // postings. Unknown collections / fields are skipped with a
    // warning — they imply a schema was deleted but its postings
    // weren't fully purged (which is a backend bug we'd rather not
    // crash the engine over).
    for row in data_rows {
        let Some(coll) = collections.get_mut(&row.collection) else {
            tracing::warn!(
                collection = %row.collection,
                "skipping posting for unknown collection during recovery"
            );
            continue;
        };
        // Decode field name + value from the prefix-encoded backend key.
        let Some((field_name, value_bytes)) = split_field_key(&row.key) else {
            tracing::warn!(
                collection = %row.collection,
                key = ?row.key,
                "skipping posting with malformed backend key"
            );
            continue;
        };
        let id = coll.interner.intern(&row.external_id);
        let elen = row.external_id.len();
        let Some(fi) = coll.fields.get_mut(field_name) else {
            tracing::warn!(
                collection = %row.collection,
                field = %field_name,
                "skipping posting for unknown field during recovery"
            );
            continue;
        };
        match fi {
            FieldIndex::Keyword(k) => {
                let value = match std::str::from_utf8(value_bytes) {
                    Ok(s) => s,
                    Err(_) => {
                        tracing::warn!(
                            field = %field_name,
                            "skipping keyword posting with non-utf8 value"
                        );
                        continue;
                    }
                };
                k.terms.entry(value.to_string()).or_default().insert(id);
                k.forward.insert(id, value.to_string());
                k.bytes += (value.len() + elen) as u64;
            }
            FieldIndex::Number(n) => {
                if value_bytes.len() != 8 {
                    tracing::warn!(
                        field = %field_name,
                        "skipping number posting with malformed key (expected 8 bytes)"
                    );
                    continue;
                }
                let mut buf = [0u8; 8];
                buf.copy_from_slice(value_bytes);
                let key = SortableF64(u64::from_be_bytes(buf));
                n.values.entry(key).or_default().insert(id);
                n.forward.insert(id, key);
                n.bytes += (8 + elen) as u64;
            }
            _ => {
                tracing::warn!(
                    field = %field_name,
                    field_type = ?fi.field_type(),
                    "backend recovered a posting for a non-keyword/number field; skipping"
                );
                continue;
            }
        }
        coll.eid_fields
            .entry(id)
            .or_default()
            .insert(field_name.to_string());
    }

    Ok(collections)
}

/// Split a backend-encoded `<field_name>\x00<value>` key into the
/// `(field_name, value_bytes)` pair.
fn split_field_key(key: &[u8]) -> Option<(&str, &[u8])> {
    let nul = key.iter().position(|b| *b == 0)?;
    let name = std::str::from_utf8(&key[..nul]).ok()?;
    Some((name, &key[nul + 1..]))
}

// ---------------------------------------------------------------------------
// Cursor helpers (slice 1: opaque base64 of an offset)
// ---------------------------------------------------------------------------

fn make_cursor(offset: usize) -> String {
    use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
    STANDARD_NO_PAD.encode(format!("{{\"offset\":{offset}}}"))
}

fn parse_cursor(s: &str) -> Option<u64> {
    use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
    let raw = STANDARD_NO_PAD.decode(s).ok()?;
    let v: serde_json::Value = serde_json::from_slice(&raw).ok()?;
    v.get("offset")?.as_u64()
}

// ---------------------------------------------------------------------------
// Snapshot wire types
// ---------------------------------------------------------------------------

const SNAPSHOT_VERSION: u32 = 1;

/// Top-level snapshot document. JSON-serialisable.
#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotV1 {
    /// Format version. Bump when the wire layout changes
    /// incompatibly so old snapshots can be detected at restore.
    pub version: u32,
    pub collections: BTreeMap<String, CollectionSnapshot>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionSnapshot {
    pub schema: BTreeMap<String, FieldSpec>,
    pub version: u32,
    pub eid_fields: HashMap<String, BTreeSet<String>>,
    pub fields: BTreeMap<String, FieldIndexSnapshot>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
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
            .map(|(id, set)| (self.interner.resolve(*id).to_string(), set.clone()))
            .collect();
        Ok(CollectionSnapshot {
            schema: self.schema.clone(),
            version: self.version,
            eid_fields,
            fields,
        })
    }
}

impl FieldIndex {
    fn to_snapshot(&self, interner: &Interner) -> Result<FieldIndexSnapshot> {
        let eid = |id: u32| interner.resolve(id).to_string();
        Ok(match self {
            FieldIndex::Text { analyzer, idx } => FieldIndexSnapshot::Text {
                analyzer: *analyzer,
                tokens: idx
                    .tokens
                    .iter()
                    .map(|(tok, m)| {
                        (
                            tok.clone(),
                            m.iter().map(|(id, tf)| (eid(*id), *tf)).collect(),
                        )
                    })
                    .collect(),
                forward: idx
                    .forward
                    .iter()
                    .map(|(id, v)| (eid(*id), v.clone()))
                    .collect(),
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
                forward: k
                    .forward
                    .iter()
                    .map(|(id, v)| (eid(*id), v.clone()))
                    .collect(),
                bytes: k.bytes,
            },
            FieldIndex::Number(n) => FieldIndexSnapshot::Number {
                forward: n
                    .forward
                    .iter()
                    .map(|(id, key)| (eid(*id), key.to_f64()))
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

impl Collection {
    fn from_snapshot(snap: CollectionSnapshot) -> Result<Self> {
        // Re-intern every external_id (eid_fields covers all indexed docs) so
        // the field postings below resolve to the same dense ids.
        let mut interner = Interner::default();
        let mut eid_fields: HashMap<u32, BTreeSet<String>> = HashMap::new();
        for (eid, set) in snap.eid_fields {
            let id = interner.intern(&eid);
            eid_fields.insert(id, set);
        }
        let mut fields: BTreeMap<String, FieldIndex> = BTreeMap::new();
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
            last_access: AtomicU64::new(0),
        })
    }
}

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
                let mut t: BTreeMap<String, BTreeMap<u32, u32>> = BTreeMap::new();
                for (tok, m) in tokens {
                    let mut inner: BTreeMap<u32, u32> = BTreeMap::new();
                    for (eid, tf) in m {
                        inner.insert(interner.intern(&eid), tf);
                    }
                    t.insert(tok, inner);
                }
                let mut fwd: HashMap<u32, (BTreeSet<String>, u32)> = HashMap::new();
                for (eid, v) in forward {
                    fwd.insert(interner.intern(&eid), v);
                }
                FieldIndex::Text {
                    analyzer,
                    idx: TextIndex {
                        tokens: t,
                        forward: fwd,
                        doc_count,
                        total_doc_len,
                        bytes,
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
                let mut fwd: HashMap<u32, String> = HashMap::new();
                for (eid, v) in forward {
                    fwd.insert(interner.intern(&eid), v);
                }
                FieldIndex::Keyword(KeywordIndex {
                    terms: t,
                    forward: fwd,
                    bytes,
                })
            }
            FieldIndexSnapshot::Number { forward, bytes } => {
                let mut values: BTreeMap<SortableF64, RoaringBitmap> = BTreeMap::new();
                let mut fwd: HashMap<u32, SortableF64> = HashMap::new();
                for (eid, raw) in forward {
                    let id = interner.intern(&eid);
                    let key = SortableF64::new(raw)?;
                    values.entry(key).or_default().insert(id);
                    fwd.insert(id, key);
                }
                FieldIndex::Number(NumberIndex {
                    values,
                    forward: fwd,
                    bytes,
                })
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
                let mut fwd: HashMap<u32, BTreeSet<String>> = HashMap::new();
                for (eid, set) in forward {
                    fwd.insert(interner.intern(&eid), set);
                }
                FieldIndex::Set(SetIndex {
                    elements: e,
                    forward: fwd,
                    bytes,
                })
            }
            FieldIndexSnapshot::Vector {
                spec,
                vectors,
                codebook,
                bytes,
            } => {
                // Always restore into a CPU HNSW backend. wgpu can be
                // reattached at runtime if the operator wants — the
                // raw vectors are what's persisted, not the graph.
                let hnsw = HnswCpuIndex::restore(spec, vectors, codebook)?;
                FieldIndex::Vector {
                    spec,
                    idx: Box::new(hnsw),
                    bytes,
                }
            }
            FieldIndexSnapshot::Hash { forward, bytes } => {
                let mut fwd: HashMap<u32, u64> = HashMap::new();
                for (eid, v) in forward {
                    fwd.insert(interner.intern(&eid), v);
                }
                FieldIndex::Hash(HashIndex {
                    forward: fwd,
                    bytes,
                })
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(parse_cursor(&c), Some(42));
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
