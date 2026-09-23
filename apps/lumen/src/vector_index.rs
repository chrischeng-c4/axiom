// CODEGEN-BEGIN
//! Vector index backends for `FieldType::Vector`.
//!
//! Two CPU backends ship in this version:
//!
//! - [`HnswCpuIndex`] — pure-Rust HNSW via `hnsw_rs`. Default. Sub-ms
//!   kNN at ≤ 10 M vectors per field.
//! - [`FlatCpuIndex`] — exact CPU brute-force full scan. 100% recall;
//!   no index build. GPU-native vector search is a future chapter.
//!
//! Scalar quantization (f32 → u8 linear) is applied transparently
//! when the field's `VectorSpec::quantize` slot is `Some(Sq)`. The
//! codebook is learned at insert time (running min/max) and snapshot
//! together with the raw vectors on `Engine::snapshot()`.
//!
//! Score semantics: every backend returns `score = -distance` so
//! `score` is monotone-decreasing across the top-K — higher = better,
//! matching the BM25 contract used by the text path.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

use crate::types::{VectorMetric, VectorQuantize, VectorSpec};

thread_local! {
    // Committed apply serializes live HNSW mutations under the Engine writer
    // lock. A thread-local handoff keeps timing state out of the index and
    // lets the caller publish it only after that outer lock drops.
    static HNSW_LAST_WRITE_LOCK_TIMING: std::cell::RefCell<Option<(Duration, Duration)>> =
        const { std::cell::RefCell::new(None) };
    /// A rebuild is a rare add-path event. Keep its timing separate from the
    /// normal add interval so the caller can identify it without new shared
    /// state on the index.
    static HNSW_LAST_GRAPH_REBUILD_TIMING: std::cell::RefCell<Option<Duration>> =
        const { std::cell::RefCell::new(None) };
}

#[path = "vector_index/graph_cache.rs"]
mod graph_cache;

#[cfg(test)]
thread_local! {
    pub(crate) static HNSW_CHECKPOINT_FULL_SCANS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static HNSW_SEARCH_POOLS: std::cell::RefCell<Vec<usize>> = const { std::cell::RefCell::new(Vec::new()) };
    static VECTOR_STORE_OWNED_SCANS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

// ---------------------------------------------------------------------------
// VectorIndex trait
// ---------------------------------------------------------------------------

/// Common backend contract — every concrete index implementation
/// (HNSW, flat CPU brute force) goes through this trait so the storage
/// layer doesn't care which one is in use.
pub trait VectorIndex: Send + Sync {
    /// Optional shutdown-only acceleration. This is never durable authority.
    #[doc(hidden)]
    fn save_graph_cache(&self, _directory: &std::path::Path) -> Result<bool> {
        Ok(false)
    }
    /// Insert (or overwrite) the vector associated with `external_id`.
    fn add(&self, external_id: &str, vector: &[f32]) -> Result<()>;

    /// Consume the timings for the most recent HNSW write-lock acquisition on
    /// this calling path. Other backends have no HNSW lock and return `None`.
    /// The caller records this only after releasing its engine writer lock.
    fn take_hnsw_write_lock_timing(&self) -> Option<(Duration, Duration)> {
        None
    }

    /// Consume the graph-rebuild interval from the latest HNSW add on this
    /// calling path. Other backends, and normal HNSW adds, return `None`.
    fn take_hnsw_graph_rebuild_timing(&self) -> Option<Duration> {
        None
    }

    /// Remove the vector for `external_id`. No-op if it isn't present.
    /// Returns `true` if a vector was removed.
    fn remove(&self, external_id: &str) -> Result<bool>;

    /// Read one current vector for an incremental checkpoint. Backends must
    /// provide a keyed lookup; a full-corpus snapshot is not a fallback.
    fn checkpoint_vector(&self, _external_id: &str) -> Result<Option<Vec<f32>>> {
        bail!("backend does not support keyed checkpoint capture")
    }

    /// Internal preparation snapshot for a staged Vector checkpoint row. This
    /// copies only the scalar-quantization codebook; it never enumerates or
    /// decodes stored vectors. Callers must separately recheck their apply cut.
    #[doc(hidden)]
    fn checkpoint_codebook_for_preparation(&self) -> Result<Option<ScalarCodebook>> {
        bail!("vector backend does not support codebook preparation snapshots")
    }

    /// Install an already decoded checkpoint value. Cold readers use this to
    /// avoid quantizing persisted vectors again while composing their layers.
    fn restore_checkpoint_vector(&self, external_id: &str, vector: &[f32]) -> Result<()> {
        self.add(external_id, vector)
    }

    /// Like [`VectorIndex::search_knn`] but only `external_id`s for
    /// which `allow` returns `true` are eligible for the result.
    ///
    /// This is the primitive behind filter-correct kNN (`knn AND
    /// <filter>`): instead of taking the global top-`k` and intersecting
    /// the filter afterwards (post-filter — recall collapses when the
    /// filter is selective), the candidate pool is widened until `k`
    /// *allowed* neighbours are found or the index is exhausted, so the
    /// caller gets the nearest `k` neighbours **within the filtered
    /// set**. For an always-true `allow` this is identical to
    /// `search_knn`, with no extra work on the hot path.
    fn search_knn_filtered(
        &self,
        query: &[f32],
        k: usize,
        allow: &dyn Fn(&str) -> bool,
    ) -> Result<Vec<(String, f32)>>;

    /// Return the top-`k` nearest external_ids and their scores
    /// (`score = -distance` so higher = better). Vectors shorter than
    /// the index's declared `dim` are rejected.
    fn search_knn(&self, query: &[f32], k: usize) -> Result<Vec<(String, f32)>> {
        self.search_knn_filtered(query, k, &|_| true)
    }

    /// Batched top-`k` kNN: answer many query vectors in one call, returning
    /// one result list per query (same order as `queries`). This is the heavy
    /// RAG / re-rank / fan-out access pattern. The default implementation loops
    /// [`VectorIndex::search_knn`]; a backend may override it to amortize
    /// per-query setup across the batch.
    fn search_knn_batch(&self, queries: &[Vec<f32>], k: usize) -> Result<Vec<Vec<(String, f32)>>> {
        queries.iter().map(|q| self.search_knn(q, k)).collect()
    }

    /// Number of vectors currently held by the index.
    fn len(&self) -> usize;

    /// Whether the index has zero vectors.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Materialize every stored (eid, vector) pair for snapshot, plus
    /// the scalar codebook when SQ is enabled. Default impl returns
    /// an empty dump — backends that intend to be snapshotted must
    /// override this.
    fn dump_for_snapshot(&self) -> Result<(Vec<(String, Vec<f32>)>, Option<ScalarCodebook>)> {
        Ok((Vec::new(), None))
    }

    /// TEST SEAM (Stage 2 Phase 2d): seal the exact-CPU flat corpus into a
    /// columnar mmap vector segment at `path` and attach it, so the flat kNN
    /// scan reads each vector zero-copy off the page instead of the in-RAM
    /// `FlatVecs::data`. The corpus row order (the eid↔row mapping) is locked
    /// when the cached flat buffer is built and is reused verbatim, so the
    /// segment-backed scan returns byte-identical (eid, distance) pairs.
    ///
    /// Returns `Ok(Some(n))` with the sealed vector count for a [`FlatCpuIndex`];
    /// `Ok(None)` for any other backend (HNSW is out of scope for this slice).
    /// No scalar quantization — the segment stores decoded `f32`, so the scan
    /// stays a pure `&[f32]` read.
    #[cfg(test)]
    fn __seal_flat_to_segment(&self, _path: &std::path::Path) -> Result<Option<u32>> {
        Ok(None)
    }

    /// PRODUCTION seal (Stage 2 Phase 2f-1): seal this backend's corpus into a
    /// columnar mmap vector segment at `path` — decoded `f32` rows in row
    /// order — and return `Ok(Some(row_eids))`, the eid of each sealed row in
    /// segment-row order, so the caller can persist that row→eid mapping (the
    /// segment stores only the f32 rows) and rebuild the index on reopen.
    ///
    /// Every backend that can be checkpointed implements this: reopen requires
    /// the `<field>.eids.lseg` sidecar unconditionally for any `Vector` field,
    /// so a backend returning `Ok(None)` here cannot be checkpointed at all
    /// (issue #3951). What backends differ on is whether sealing also FREES the
    /// in-RAM copy — see [`VectorIndex::seal_releases_ram`], which is what the
    /// caller asks instead of inspecting the declared backend. The default impl
    /// no-ops for a backend with no segment format yet.
    fn seal_to_segment_prod(&self, _path: &std::path::Path) -> Result<Option<Vec<String>>> {
        Ok(None)
    }

    /// Seal an unpublished checkpoint with its actual WAL cut. The default
    /// preserves compatibility for third-party backends using the legacy hook.
    fn seal_to_segment_prod_at(
        &self,
        path: &std::path::Path,
        _sequence: u64,
    ) -> Result<Option<Vec<String>>> {
        self.seal_to_segment_prod(path)
    }

    /// Whether a successful [`VectorIndex::seal_to_segment_prod`] left this
    /// index holding NO in-RAM copy of the sealed vectors, so the caller should
    /// zero the field's reported byte footprint.
    ///
    /// This is a property of what the seal implementation actually did, not of
    /// the backend the schema declares, and only the index can answer it: a
    /// flat index hands the mmap its whole `data` buffer and keeps nothing,
    /// while an HNSW index keeps its graph and raw vectors resident so the live
    /// process goes on serving approximate kNN off the graph. A caller that
    /// decided this by matching on `VectorSpec::backend` would have to be
    /// edited again for every backend added, and would silently report the
    /// wrong footprint until someone remembered to.
    fn seal_releases_ram(&self) -> bool {
        false
    }

    fn attach_checkpoint_delta(
        &self,
        _reader: Arc<crate::segment::SegmentReader>,
        _external_ids: &[String],
        _acknowledged: &[bool],
    ) -> Result<()> {
        Ok(())
    }

    /// Immutable inputs used to verify a staged compaction at publication.
    fn checkpoint_delta_readers(&self) -> Vec<Arc<crate::segment::SegmentReader>> {
        Vec::new()
    }

    fn checkpoint_base_reader(&self) -> Option<Arc<crate::segment::SegmentReader>> {
        None
    }

    fn replace_checkpoint_base(
        &self,
        _base: &Arc<crate::segment::SegmentReader>,
        _inputs: &[Arc<crate::segment::SegmentReader>],
        _reader: Arc<crate::segment::SegmentReader>,
        _external_ids: &[String],
    ) -> Result<()> {
        bail!("vector backend does not support mapped base replacement")
    }

    fn replace_checkpoint_deltas(
        &self,
        _inputs: &[Arc<crate::segment::SegmentReader>],
        _reader: Arc<crate::segment::SegmentReader>,
        _external_ids: &[String],
    ) -> Result<()> {
        bail!("vector backend does not support checkpoint delta replacement")
    }

    fn resident_vector_payload_rows(&self) -> usize {
        self.len()
    }
    /// The existing field-byte estimate for payloads still resident after a
    /// checkpoint. Backends that retain their complete corpus keep the
    /// caller's estimate by returning None.
    fn checkpoint_resident_bytes(&self) -> Option<u64> {
        None
    }
    fn has_checkpoint_mapping(&self) -> bool {
        false
    }

    /// Install a newly written full vector base.  `acknowledged[eid] == false`
    /// means this live index has a later update or delete which must remain
    /// above the new mmap base.
    fn install_checkpoint_base(
        &self,
        _reader: Arc<crate::segment::SegmentReader>,
        _external_ids: &[String],
        _acknowledged: &HashMap<String, bool>,
    ) -> Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Scalar quantization
// ---------------------------------------------------------------------------

/// Codebook for the linear f32→u8 scalar quantizer. The codebook is
/// learned at insert time: every `add()` widens `(min, max)` if needed.
///
/// Re-quantizing already-stored vectors on codebook growth is a v2
/// nice-to-have; v1 simply accepts that earlier inserts will saturate
/// at the codebook's edges. In practice this is fine because callers
/// L2-normalize embeddings before insertion, which bounds the input
/// range tightly.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScalarCodebook {
    pub min: f32,
    pub max: f32,
    pub dim: usize,
}

impl ScalarCodebook {
    /// Empty codebook — the next `widen` call defines the range.
    pub fn empty(dim: usize) -> Self {
        Self {
            min: f32::INFINITY,
            max: f32::NEG_INFINITY,
            dim,
        }
    }

    /// Grow the codebook to cover `vec`. No-op when the vector already
    /// fits.
    pub fn widen(&mut self, vec: &[f32]) {
        for &v in vec {
            if v < self.min {
                self.min = v;
            }
            if v > self.max {
                self.max = v;
            }
        }
        // Degenerate fallback when only one value was ever seen — give
        // the codec a 1-unit window so the divisor isn't zero.
        if self.min == self.max {
            self.max = self.min + 1.0;
        }
    }

    fn range(&self) -> f32 {
        (self.max - self.min).max(f32::MIN_POSITIVE)
    }
}

/// Encode a vector to one byte per dimension using `cb`. Out-of-range
/// values saturate at `0` / `255`.
pub fn encode_sq(vec: &[f32], cb: &ScalarCodebook) -> Vec<u8> {
    let span = cb.range();
    vec.iter()
        .map(|&v| {
            let t = ((v - cb.min) / span).clamp(0.0, 1.0);
            (t * 255.0).round() as u8
        })
        .collect()
}

/// Decode a u8-encoded vector back to f32 using `cb`.
pub fn decode_sq(bytes: &[u8], cb: &ScalarCodebook) -> Vec<f32> {
    let span = cb.range();
    bytes
        .iter()
        .map(|&b| cb.min + (b as f32 / 255.0) * span)
        .collect()
}

// ---------------------------------------------------------------------------
// Distance helpers
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn distance(metric: VectorMetric, a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    match metric {
        VectorMetric::L2 => l2_squared(a, b).sqrt(),
        VectorMetric::Cosine => 1.0 - cosine_similarity(a, b),
        // For dot product we store *negative* dot as distance so that
        // smaller = closer = higher similarity, matching the HNSW
        // ordering contract.
        VectorMetric::Dot => -dot(a, b),
    }
}

#[allow(dead_code)]
fn l2_squared(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let d = x - y;
            d * d
        })
        .sum()
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Scale `v` to *just under* unit norm (‖·‖ = 1 − 1e-6) for the DistDot cosine
/// path. Scaling both the stored and the query vector by a constant is exactly
/// rank-preserving for cosine (the cosine angle is scale-invariant), so recall
/// is unaffected. Staying a hair under unit keeps `dot(v,v) < 1`, which dodges
/// anndists `scalar_dot_f32`'s `assert!(1 - dot >= 0)` — that fires on
/// near-duplicate clustered vectors when float rounding pushes a unit self-dot
/// just above 1. A zero vector passes through unchanged (its dot is 0 →
/// distance 1, identical to what DistCosine yields for a zero-norm input).
fn normalize_unit_safe(v: &[f32]) -> Vec<f32> {
    let norm = dot(v, v).sqrt();
    if norm == 0.0 {
        return v.to_vec();
    }
    let inv = (1.0 - 1e-6) / norm;
    v.iter().map(|x| x * inv).collect()
}

#[allow(dead_code)]
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let d = dot(a, b);
    let na = dot(a, a).sqrt();
    let nb = dot(b, b).sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        d / (na * nb)
    }
}

// ---------------------------------------------------------------------------
// Quantization-aware storage helper
// ---------------------------------------------------------------------------

/// In-memory store of either raw f32 vectors or SQ-encoded bytes plus
/// a learned codebook. Both backends below share this.
#[derive(Debug)]
struct VectorStore {
    spec: VectorSpec,
    raw: HashMap<String, Vec<f32>>,
    encoded: HashMap<String, Vec<u8>>,
    codebook: Option<ScalarCodebook>,
}

impl VectorStore {
    fn new(spec: VectorSpec) -> Self {
        let codebook = match spec.quantize {
            Some(VectorQuantize::Sq) => Some(ScalarCodebook::empty(spec.dim as usize)),
            _ => None,
        };
        Self {
            spec,
            raw: HashMap::new(),
            encoded: HashMap::new(),
            codebook,
        }
    }

    fn put(&mut self, eid: &str, vec: &[f32]) -> Result<()> {
        if vec.len() != self.spec.dim as usize {
            bail!(
                "vector dim mismatch: expected {}, got {}",
                self.spec.dim,
                vec.len()
            );
        }
        match self.spec.quantize {
            Some(VectorQuantize::Sq) => {
                let cb = self
                    .codebook
                    .as_mut()
                    .expect("codebook present when SQ enabled");
                cb.widen(vec);
                let bytes = encode_sq(vec, cb);
                self.encoded.insert(eid.to_string(), bytes);
            }
            _ => {
                self.raw.insert(eid.to_string(), vec.to_vec());
            }
        }
        Ok(())
    }

    fn drop(&mut self, eid: &str) -> bool {
        self.raw.remove(eid).is_some() | self.encoded.remove(eid).is_some()
    }

    fn len(&self) -> usize {
        if self.spec.quantize.is_some() {
            self.encoded.len()
        } else {
            self.raw.len()
        }
    }

    /// Materialize the f32 view of every stored vector. Decoded on the
    /// fly when SQ is on.
    fn iter_decoded(&self) -> Box<dyn Iterator<Item = (String, Vec<f32>)> + '_> {
        #[cfg(test)]
        VECTOR_STORE_OWNED_SCANS.with(|scans| scans.set(scans.get() + 1));
        if let Some(cb) = self.codebook.as_ref() {
            Box::new(
                self.encoded
                    .iter()
                    .map(move |(k, b)| (k.clone(), decode_sq(b, cb))),
            )
        } else {
            Box::new(self.raw.iter().map(|(k, v)| (k.clone(), v.clone())))
        }
    }

    /// Decode a single eid's vector.
    #[allow(dead_code)]
    fn get_decoded(&self, eid: &str) -> Option<Vec<f32>> {
        if let Some(cb) = self.codebook.as_ref() {
            self.encoded.get(eid).map(|b| decode_sq(b, cb))
        } else {
            self.raw.get(eid).cloned()
        }
    }
}

// ---------------------------------------------------------------------------
// HNSW CPU backend
// ---------------------------------------------------------------------------

/// HNSW-on-CPU backend. Uses `hnsw_rs` 0.3 under the hood with the
/// appropriate distance type for the requested metric.
///
/// Because `hnsw_rs::Hnsw` is parameterised by a concrete distance
/// type, we keep three internal variants in `HnswInner` and dispatch
/// on the field's metric at construction time.
pub struct HnswCpuIndex {
    inner: RwLock<HnswCpuInner>,
    exact_scan_fallbacks: AtomicU64,
}

/// Diagnostic-only outcome for one HNSW restore attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HnswGraphCacheResult {
    Hit,
    Absent,
    Rejected,
}

impl HnswGraphCacheResult {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Hit => "hit",
            Self::Absent => "absent",
            Self::Rejected => "rejected",
        }
    }
}

/// Timings for one authoritative HNSW restoration. They are emitted by the
/// storage layer and do not alter graph-cache acceptance or fallback behavior.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct HnswRestoreTiming {
    pub(crate) cache_result: Option<HnswGraphCacheResult>,
    pub(crate) cache_prepare: Duration,
    pub(crate) cache_fingerprint: Duration,
    pub(crate) cache_manifest: Duration,
    pub(crate) cache_payload_hash: Duration,
    pub(crate) cache_materialize: Duration,
    pub(crate) cache_deserialize: Duration,
    pub(crate) cache_validate: Duration,
    pub(crate) fallback_rebuild: Duration,
    pub(crate) total: Duration,
}

impl HnswRestoreTiming {
    fn record_cache_timing(&mut self, cache: graph_cache::LoadTiming) {
        self.cache_prepare = cache.prepare;
        self.cache_fingerprint = cache.fingerprint;
        self.cache_manifest = cache.manifest;
        self.cache_payload_hash = cache.payload_hash;
        self.cache_materialize = cache.materialize;
        self.cache_deserialize = cache.deserialize;
        self.cache_validate = cache.validate;
    }
}

#[derive(Debug)]
pub(crate) struct HnswRestoreFailure {
    pub(crate) error: anyhow::Error,
    pub(crate) timing: HnswRestoreTiming,
}

struct HnswCpuInner {
    store: VectorStore,
    /// external_id ↔ internal id (HNSW DataId).
    eid_to_id: HashMap<String, usize>,
    id_to_eid: HashMap<usize, String>,
    next_id: usize,
    hnsw: HnswBackend,
    /// Search-time beam width (recall/latency trade-off). Defaults to
    /// [`hnsw_search_ef`]; a tuning/bench harness can override it via
    /// [`HnswCpuIndex::set_ef_search`] without rebuilding the graph.
    ef_search: usize,
}

/// HNSW dispatch by metric. We keep an `Option` because for f32-SQ
/// paths we sometimes want to lazily rebuild on demand; for the
/// metric-specific cases below we eagerly build at construction.
enum HnswBackend {
    L2(hnsw_rs::hnsw::Hnsw<'static, f32, hnsw_rs::anndists::dist::DistL2>),
    // Cosine is served by DistDot over internally unit-normalized vectors:
    // dot(â,b̂) == cos, so the distance (1 − dot) is identical to DistCosine's,
    // but DistDot has a NEON/AVX kernel and skips the two per-comparison norm
    // recomputations that make DistCosine the hot-path cost.
    Cosine(hnsw_rs::hnsw::Hnsw<'static, f32, hnsw_rs::anndists::dist::DistDot>),
    Dot(hnsw_rs::hnsw::Hnsw<'static, f32, hnsw_rs::anndists::dist::DistDot>),
    Cached(Box<graph_cache::OwnedGraph>),
}

// SAFETY-equivalent: `hnsw_rs::Hnsw` is `Send + Sync` for the distance
// types we use; the wrapping `RwLock` enforces external borrow rules.
unsafe impl Send for HnswBackend {}
unsafe impl Sync for HnswBackend {}

// Graph-build quality drives recall (more so than search ef). M=32 +
// ef_construction=400 keeps recall@10 ≥ 0.95 out to 1M densely-clustered
// vectors, at the cost of a larger graph + slower build.
const HNSW_MAX_NB_CONNECTION: usize = 32;
const HNSW_EF_CONSTRUCTION: usize = 400;
const HNSW_MAX_LAYER: usize = 16;
const HNSW_DEFAULT_MAX_ELEMENTS: usize = 10_000;
// ef controls the recall/latency trade-off at query time. The previous default
// (512) over-fetched ~4–5x more graph nodes than needed: the dense M=32 graph
// already holds recall@10 ≥ 0.95 at a far smaller beam. 128 is the recall-matched
// default (validated against brute-force ground truth on a clustered corpus);
// override with `LUMEN_HNSW_EF` for tuning sweeps.
const HNSW_SEARCH_EF_DEFAULT: usize = 128;

/// Resolve the default search-`ef` (env `LUMEN_HNSW_EF`, else the const).
/// Read once per index at construction; the bench harness can still override
/// per-index via [`HnswCpuIndex::set_ef_search`].
fn hnsw_search_ef() -> usize {
    std::env::var("LUMEN_HNSW_EF")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&e| e > 0)
        .unwrap_or(HNSW_SEARCH_EF_DEFAULT)
}

impl HnswBackend {
    fn new(metric: VectorMetric, max_elements: usize) -> Self {
        match metric {
            VectorMetric::L2 => HnswBackend::L2(hnsw_rs::hnsw::Hnsw::new(
                HNSW_MAX_NB_CONNECTION,
                max_elements.max(HNSW_DEFAULT_MAX_ELEMENTS),
                HNSW_MAX_LAYER,
                HNSW_EF_CONSTRUCTION,
                hnsw_rs::anndists::dist::DistL2,
            )),
            VectorMetric::Cosine => HnswBackend::Cosine(hnsw_rs::hnsw::Hnsw::new(
                HNSW_MAX_NB_CONNECTION,
                max_elements.max(HNSW_DEFAULT_MAX_ELEMENTS),
                HNSW_MAX_LAYER,
                HNSW_EF_CONSTRUCTION,
                hnsw_rs::anndists::dist::DistDot,
            )),
            VectorMetric::Dot => HnswBackend::Dot(hnsw_rs::hnsw::Hnsw::new(
                HNSW_MAX_NB_CONNECTION,
                max_elements.max(HNSW_DEFAULT_MAX_ELEMENTS),
                HNSW_MAX_LAYER,
                HNSW_EF_CONSTRUCTION,
                hnsw_rs::anndists::dist::DistDot,
            )),
        }
    }

    fn insert(&self, vec: &[f32], id: usize) {
        match self {
            HnswBackend::L2(h) => h.insert((vec, id)),
            // Cosine: feed the unit-normalized vector so DistDot == cosine.
            HnswBackend::Cosine(h) => h.insert((&normalize_unit_safe(vec), id)),
            HnswBackend::Dot(h) => h.insert((vec, id)),
            HnswBackend::Cached(h) => h.insert(vec, id),
        }
    }

    fn search(&self, vec: &[f32], k: usize, ef: usize) -> Vec<(usize, f32)> {
        #[cfg(test)]
        HNSW_SEARCH_POOLS.with(|pools| pools.borrow_mut().push(k));
        let ef = k.max(ef);
        let raw = match self {
            HnswBackend::L2(h) => h.search(vec, k, ef),
            HnswBackend::Cosine(h) => {
                let q = normalize_unit_safe(vec);
                h.search(&q, k, ef)
            }
            HnswBackend::Dot(h) => h.search(vec, k, ef),
            HnswBackend::Cached(h) => return h.search(vec, k, ef),
        };
        raw.into_iter().map(|n| (n.d_id, n.distance)).collect()
    }
}

impl HnswCpuIndex {
    pub(crate) fn restore_with_graph_cache(
        spec: VectorSpec,
        vectors: Vec<(String, Vec<f32>)>,
        codebook: Option<ScalarCodebook>,
        directory: Option<&std::path::Path>,
    ) -> Result<Self> {
        Self::restore_with_graph_cache_timed(spec, vectors, codebook, directory)
            .map(|(index, _)| index)
            .map_err(|failure| failure.error)
    }

    pub(crate) fn restore_with_graph_cache_timed(
        spec: VectorSpec,
        vectors: Vec<(String, Vec<f32>)>,
        codebook: Option<ScalarCodebook>,
        directory: Option<&std::path::Path>,
    ) -> std::result::Result<(Self, HnswRestoreTiming), HnswRestoreFailure> {
        let started = Instant::now();
        let mut timing = HnswRestoreTiming::default();
        if let Some(directory) = directory {
            match graph_cache::load_timed(spec, &vectors, codebook, directory) {
                Ok((Some(index), cache_timing)) => {
                    timing.cache_result = Some(HnswGraphCacheResult::Hit);
                    timing.record_cache_timing(cache_timing);
                    timing.total = started.elapsed();
                    tracing::info!(rows = vectors.len(), "HNSW graph cache loaded");
                    return Ok((index, timing));
                }
                Ok((None, cache_timing)) => {
                    timing.cache_result = Some(HnswGraphCacheResult::Absent);
                    timing.record_cache_timing(cache_timing);
                }
                Err(failure) => {
                    timing.cache_result = Some(HnswGraphCacheResult::Rejected);
                    timing.record_cache_timing(failure.timing);
                    tracing::warn!(error = %failure.error, "ignoring optional HNSW graph cache");
                }
            }
        } else {
            timing.cache_result = Some(HnswGraphCacheResult::Absent);
        }
        let rebuild_started = Instant::now();
        let rebuilt = Self::restore(spec, vectors, codebook);
        timing.fallback_rebuild = rebuild_started.elapsed();
        timing.total = started.elapsed();
        rebuilt
            .map(|index| (index, timing))
            .map_err(|error| HnswRestoreFailure { error, timing })
    }

    /// Construct a fresh, empty CPU HNSW index for the given spec.
    pub fn new(spec: VectorSpec) -> Self {
        Self {
            inner: RwLock::new(HnswCpuInner {
                store: VectorStore::new(spec),
                eid_to_id: HashMap::new(),
                id_to_eid: HashMap::new(),
                next_id: 0,
                hnsw: HnswBackend::new(spec.metric, HNSW_DEFAULT_MAX_ELEMENTS),
                ef_search: hnsw_search_ef(),
            }),
            exact_scan_fallbacks: AtomicU64::new(0),
        }
    }

    /// Restore an HNSW index from a snapshot — bulk-inserts the saved
    /// vectors back into a fresh graph. The codebook is carried over
    /// directly because the snapshot already encoded under it.
    pub fn restore(
        spec: VectorSpec,
        vectors: Vec<(String, Vec<f32>)>,
        codebook: Option<ScalarCodebook>,
    ) -> Result<Self> {
        let idx = Self::new(spec);
        {
            let mut inner = idx
                .inner
                .write()
                .map_err(|_| anyhow!("hnsw lock poisoned"))?;
            // Override the freshly-initialized codebook with the one
            // that was actually used to encode the persisted bytes.
            // Required so decode_sq() round-trips exactly.
            if codebook.is_some() {
                inner.store.codebook = codebook;
            }
        }
        for (eid, v) in vectors {
            idx.add(&eid, &v)?;
        }
        Ok(idx)
    }

    /// Reopen an HNSW index from a sealed vector segment plus its row→eid
    /// mapping, with NO snapshot — the segment counterpart of
    /// [`HnswCpuIndex::restore`], and the reason a field declared `hnsw-cpu`
    /// is still HNSW-backed after a restart.
    ///
    /// Row `i`'s vector is `seg.vector_at(i, dim)` (read once off the mmap) and
    /// its external id is `row_eids[i]`; each is re-inserted into a fresh
    /// graph. Unlike [`FlatCpuIndex::open_from_segment`], the vectors DO come
    /// back into RAM — a graph cannot be traversed against a demand-paged
    /// column it has no edges for — which is the cost the declared backend
    /// asks for and what [`VectorIndex::seal_releases_ram`] reports as `false`.
    /// The segment is consumed and dropped: once the rows are re-inserted the
    /// store owns them.
    ///
    /// This fallback builds the graph synchronously before serving. A matched
    /// graph cache can avoid this path during checkpoint recovery. Without
    /// cached edges, the graph must be rebuilt from the authoritative vectors.
    /// Log both edges so an operator can distinguish recovery from a hung node.
    pub fn open_from_segment(
        spec: VectorSpec,
        seg: std::sync::Arc<crate::segment::SegmentReader>,
        row_eids: Vec<String>,
    ) -> Result<Self> {
        let dim = spec.dim as usize;
        let rows = row_eids.len();
        tracing::info!(
            rows,
            dim,
            "rebuilding an hnsw-cpu graph from its sealed segment; the node does \
             not serve this field until it completes"
        );
        let started = std::time::Instant::now();
        let idx = Self::new(spec);
        for (row, eid) in row_eids.iter().enumerate() {
            let vector = seg.vector_at(row as u32, dim).ok_or_else(|| {
                anyhow!("vector segment is missing row {row} of {rows} (eid `{eid}`)")
            })?;
            idx.add(eid, vector)?;
        }
        debug_assert_eq!(idx.len(), rows);
        tracing::info!(
            rows,
            elapsed_ms = started.elapsed().as_millis() as u64,
            "hnsw-cpu graph rebuilt"
        );
        Ok(idx)
    }

    /// Override the search-time `ef` (beam width). Used by tuning/bench
    /// harnesses to sweep recall/latency on an already-built graph; production
    /// uses the [`hnsw_search_ef`] default set at construction.
    pub fn set_ef_search(&self, ef: usize) {
        if let Ok(mut inner) = self.inner.write() {
            inner.ef_search = ef.max(1);
        }
    }

    /// Returns the monotonic count of queries that fell through to the exact store scan.
    pub fn exact_scan_fallbacks(&self) -> u64 {
        self.exact_scan_fallbacks.load(Ordering::Relaxed)
    }
}

impl VectorIndex for HnswCpuIndex {
    fn save_graph_cache(&self, directory: &std::path::Path) -> Result<bool> {
        let inner = self.inner.read().map_err(|_| anyhow!("hnsw lock poisoned"))?;
        graph_cache::save(&inner, directory)
    }
    fn checkpoint_vector(&self, external_id: &str) -> Result<Option<Vec<f32>>> {
        let inner = self
            .inner
            .read()
            .map_err(|_| anyhow!("hnsw lock poisoned"))?;
        Ok(inner.store.get_decoded(external_id))
    }

    fn checkpoint_codebook_for_preparation(&self) -> Result<Option<ScalarCodebook>> {
        let inner = self
            .inner
            .read()
            .map_err(|_| anyhow!("hnsw lock poisoned"))?;
        Ok(inner.store.codebook)
    }

    fn add(&self, external_id: &str, vector: &[f32]) -> Result<()> {
        HNSW_LAST_WRITE_LOCK_TIMING.with(|timing| *timing.borrow_mut() = None);
        HNSW_LAST_GRAPH_REBUILD_TIMING.with(|timing| *timing.borrow_mut() = None);
        let write_wait_started = Instant::now();
        let mut inner = self
            .inner
            .write()
            .map_err(|_| anyhow!("hnsw lock poisoned"))?;
        let write_wait = write_wait_started.elapsed();
        let write_hold_started = Instant::now();
        let mut graph_rebuild = None;
        let result = (|| {
            if vector.len() != inner.store.spec.dim as usize {
                bail!(
                    "vector dim mismatch on add: expected {}, got {}",
                    inner.store.spec.dim,
                    vector.len()
                );
            }
            // Replace path: if the eid already has a vector, allocate a
            // new internal id and orphan the old one. hnsw_rs 0.3 has no
            // public "remove" — orphaning is the documented workaround.
            // The forward map decides what is reachable.
            let id = inner.next_id;
            inner.next_id += 1;
            inner.store.put(external_id, vector)?;
            // We always feed the *decoded* vector to HNSW so the same graph
            // works whether SQ is on or off. The codebook only affects
            // storage and recall, not the graph topology.
            if inner.store.codebook.is_some() {
                let decoded = inner
                    .store
                    .get_decoded(external_id)
                    .ok_or_else(|| anyhow!("just-inserted vector vanished"))?;
                inner.hnsw.insert(&decoded, id);
            } else {
                inner.hnsw.insert(vector, id);
            }
            if let Some(old_id) = inner.eid_to_id.insert(external_id.to_string(), id) {
                inner.id_to_eid.remove(&old_id);
            }
            inner.id_to_eid.insert(id, external_id.to_string());

            let live_len = inner.store.len();
            if inner.next_id >= 2 * live_len && live_len > 0 {
                let rebuild_started = Instant::now();
                let fresh_hnsw =
                    HnswBackend::new(inner.store.spec.metric, HNSW_DEFAULT_MAX_ELEMENTS);
                inner.eid_to_id.clear();
                inner.id_to_eid.clear();
                inner.next_id = 0;
                let mut live_vecs: Vec<(String, Vec<f32>)> = inner.store.iter_decoded().collect();
                live_vecs.sort_by(|a, b| a.0.cmp(&b.0));
                for (eid, v) in live_vecs {
                    let new_id = inner.next_id;
                    inner.next_id += 1;
                    fresh_hnsw.insert(&v, new_id);
                    inner.eid_to_id.insert(eid.clone(), new_id);
                    inner.id_to_eid.insert(new_id, eid);
                }
                inner.hnsw = fresh_hnsw;
                graph_rebuild = Some(rebuild_started.elapsed());
            }
            Ok(())
        })();
        drop(inner);
        let write_hold = write_hold_started.elapsed();
        HNSW_LAST_WRITE_LOCK_TIMING.with(|timing| {
            *timing.borrow_mut() = Some((write_wait, write_hold));
        });
        HNSW_LAST_GRAPH_REBUILD_TIMING.with(|timing| {
            *timing.borrow_mut() = graph_rebuild;
        });
        result
    }

    fn take_hnsw_write_lock_timing(&self) -> Option<(Duration, Duration)> {
        HNSW_LAST_WRITE_LOCK_TIMING.with(|timing| timing.borrow_mut().take())
    }

    fn take_hnsw_graph_rebuild_timing(&self) -> Option<Duration> {
        HNSW_LAST_GRAPH_REBUILD_TIMING.with(|timing| timing.borrow_mut().take())
    }

    fn remove(&self, external_id: &str) -> Result<bool> {
        HNSW_LAST_WRITE_LOCK_TIMING.with(|timing| *timing.borrow_mut() = None);
        let write_wait_started = Instant::now();
        let mut inner = self
            .inner
            .write()
            .map_err(|_| anyhow!("hnsw lock poisoned"))?;
        let write_wait = write_wait_started.elapsed();
        let write_hold_started = Instant::now();
        let removed = inner.store.drop(external_id);
        if let Some(id) = inner.eid_to_id.remove(external_id) {
            inner.id_to_eid.remove(&id);
        }
        drop(inner);
        let write_hold = write_hold_started.elapsed();
        HNSW_LAST_WRITE_LOCK_TIMING.with(|timing| {
            *timing.borrow_mut() = Some((write_wait, write_hold));
        });
        Ok(removed)
    }

    fn search_knn_filtered(
        &self,
        query: &[f32],
        k: usize,
        allow: &dyn Fn(&str) -> bool,
    ) -> Result<Vec<(String, f32)>> {
        let inner = self
            .inner
            .read()
            .map_err(|_| anyhow!("hnsw lock poisoned"))?;
        if query.len() != inner.store.spec.dim as usize {
            bail!(
                "kNN query dim mismatch: expected {}, got {}",
                inner.store.spec.dim,
                query.len()
            );
        }
        let n = inner.store.len();
        if n == 0 || k == 0 {
            return Ok(Vec::new());
        }
        // hnsw_rs exposes no mid-traversal filter hook and does not support node
        // removal. Bound graph expansion by the configured search beam or the
        // initial over-fetch, whichever is larger. Asking the graph for the
        // whole corpus also raises its traversal ef to corpus size, while an
        // exact filtered scan already provides the required fallback. In
        // particular, repeating a traversal cannot certify an unresolved
        // nearest orphan, so that case goes directly to the exact live store.
        let graph_len = inner.next_id;
        let mut pool = k.saturating_mul(5).min(graph_len);
        let ef = inner.ef_search;
        let pool_limit = pool.max(ef).min(graph_len);
        loop {
            let raw = inner.hnsw.search(query, pool, ef);
            let mut out: Vec<(String, f32)> = Vec::with_capacity(k);
            let mut has_unresolved_orphan = false;
            for (id, dist) in &raw {
                let Some(eid) = inner.id_to_eid.get(id) else {
                    if out.len() < k {
                        has_unresolved_orphan = true;
                    }
                    continue; // orphaned by a replace
                };
                if !allow(eid) {
                    continue;
                }
                out.push((eid.clone(), -dist));
                if out.len() == k {
                    break;
                }
            }

            if out.len() == k && !has_unresolved_orphan {
                return Ok(out);
            }

            if has_unresolved_orphan || pool >= pool_limit {
                break;
            }
            pool = pool.saturating_mul(2).min(pool_limit);
        }

        // When the bounded graph search or an orphan prevents a conclusive
        // approximate answer, scan the exact live store instead of returning
        // a short or degraded result. Membership and scoring are unchanged.
        self.exact_scan_fallbacks.fetch_add(1, Ordering::Relaxed);
        let metric = inner.store.spec.metric;
        // The read guard pins the store until the selected IDs are copied.
        // Borrow raw vectors and candidate IDs. For SQ, reject the ID before
        // decoding its vector; only allowed values need a temporary f32 view.
        let mut cand: Vec<(&str, f32)> = if let Some(codebook) = inner.store.codebook.as_ref() {
            inner
                .store
                .encoded
                .iter()
                .filter(|(eid, _)| allow(eid))
                .map(|(eid, bytes)| {
                    let vector = decode_sq(bytes, codebook);
                    (eid.as_str(), -distance(metric, query, &vector))
                })
                .collect()
        } else {
            inner
                .store
                .raw
                .iter()
                .filter(|(eid, _)| allow(eid))
                .map(|(eid, vector)| (eid.as_str(), -distance(metric, query, vector)))
                .collect()
        };
        let want = k.min(cand.len());
        if want > 0 && want < cand.len() {
            cand.select_nth_unstable_by(want - 1, |a, b| {
                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            });
            cand.truncate(want);
        }
        cand.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(cand
            .into_iter()
            .map(|(eid, score)| (eid.to_owned(), score))
            .collect())
    }

    fn len(&self) -> usize {
        self.inner.read().map(|i| i.store.len()).unwrap_or(0)
    }

    fn dump_for_snapshot(&self) -> Result<(Vec<(String, Vec<f32>)>, Option<ScalarCodebook>)> {
        let inner = self
            .inner
            .read()
            .map_err(|_| anyhow!("hnsw lock poisoned"))?;
        let vectors: Vec<(String, Vec<f32>)> = inner.store.iter_decoded().collect();
        Ok((vectors, inner.store.codebook))
    }

    /// PRODUCTION seal (issue #3951): persist the HNSW corpus into the SAME
    /// columnar vector-segment format [`FlatCpuIndex::seal_to_segment_prod`]
    /// writes — decoded `f32` rows in row order, plus the row→eid mapping the
    /// caller persists as the `<field>.eids.lseg` sidecar.
    ///
    /// This is not optional: `Collection::open_from_segments` requires the
    /// `<field>.eids.lseg` sidecar unconditionally for any `Vector` field, so
    /// with the inherited no-op default a default-backend vector field wrote
    /// neither file, and `SegmentRdbStore::save_inner`'s pre-commit
    /// verification reopen (and, unpatched, every later real restart) failed
    /// for any collection that had one.
    ///
    /// The GRAPH is deliberately not what gets persisted — only the vectors
    /// are. `HnswCpuIndex::open_from_segment` reads them back off the mmap and
    /// re-inserts them, paying the build cost once at reopen, so the field
    /// answers kNN with the backend its schema declares on both sides of a
    /// restart. Persisting the graph itself would pin this index's internal
    /// layout into the on-disk format for no behavioural gain.
    ///
    /// No scalar quantization on the wire, matching the flat seal contract:
    /// the segment stores decoded `f32`, so recovery reads plain rows.
    fn seal_to_segment_prod(&self, path: &std::path::Path) -> Result<Option<Vec<String>>> {
        self.seal_checkpoint(path, None)
    }

    fn seal_to_segment_prod_at(
        &self,
        path: &std::path::Path,
        sequence: u64,
    ) -> Result<Option<Vec<String>>> {
        self.seal_checkpoint(path, Some(sequence))
    }
}

impl std::fmt::Debug for HnswCpuIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HnswCpuIndex")
            .field("len", &self.len())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Exact CPU brute-force backend (`flat-cpu`)
// ---------------------------------------------------------------------------

/// Exact CPU brute-force kNN. No graph, no build cost: it stores the raw
/// vectors in a contiguous `[N*dim]` buffer and scans them all per query —
/// parallel across rows (rayon) with an auto-vectorized distance kernel. For
/// moderate N this beats both an approximate index's build cost and a
/// single-threaded exact scan (e.g. pgvector's `seqscan`), while giving 100%
/// recall. The flat buffer is cached and rebuilt lazily after a mutation.
pub struct FlatCpuIndex {
    inner: Mutex<FlatInner>,
}

struct FlatInner {
    store: VectorStore,
    flat: Option<FlatVecs>,
}

#[derive(Clone, Copy)]
enum FlatLocation {
    Base(u32),
    Layer { layer: u32, row: u32 },
    Ram,
}

struct FlatLayer {
    reader: Arc<crate::segment::SegmentReader>,
}

/// A stable EID slot has one current source.  Mmap sources are immutable;
/// `data` contains only current, uncheckpointed decoded payloads.
struct FlatVecs {
    data: HashMap<u32, Vec<f32>>,
    eids: Vec<String>,
    dim: usize,
    seg: Option<Arc<crate::segment::SegmentReader>>,
    n_base: usize,
    tomb: roaring::RoaringBitmap,
    eid_to_row: HashMap<String, u32>,
    locations: Vec<Option<FlatLocation>>,
    layers: Vec<FlatLayer>,
}

impl FlatVecs {
    #[inline]
    fn row(&self, slot: usize) -> &[f32] {
        match self.locations[slot].expect("query selected a deleted vector slot") {
            FlatLocation::Base(row) => self
                .seg
                .as_ref()
                .and_then(|seg| seg.vector_at(row, self.dim))
                .expect("base vector row is present"),
            FlatLocation::Layer { layer, row } => self.layers[layer as usize]
                .reader
                .vector_at(row, self.dim)
                .expect("delta vector row is present"),
            FlatLocation::Ram => self
                .data
                .get(&(slot as u32))
                .map(Vec::as_slice)
                .expect("RAM vector slot is present"),
        }
    }

    fn live_slots(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.eids.len())
            .filter(|slot| !self.tomb.contains(*slot as u32) && self.locations[*slot].is_some())
    }
}

impl FlatCpuIndex {
    pub fn new(spec: VectorSpec) -> Self {
        Self {
            inner: Mutex::new(FlatInner {
                store: VectorStore::new(spec),
                flat: None,
            }),
        }
    }

    pub fn restore(
        spec: VectorSpec,
        vectors: Vec<(String, Vec<f32>)>,
        codebook: Option<ScalarCodebook>,
    ) -> Result<Self> {
        let idx = Self::new(spec);
        if codebook.is_some() {
            idx.inner
                .lock()
                .map_err(|_| anyhow!("flat lock poisoned"))?
                .store
                .codebook = codebook;
        }
        for (eid, value) in vectors {
            idx.add(&eid, &value)?;
        }
        Ok(idx)
    }

    fn ensure_flat(inner: &mut FlatInner) {
        if inner.flat.is_some() {
            return;
        }
        let dim = inner.store.spec.dim as usize;
        let mut data = HashMap::with_capacity(inner.store.len());
        let mut eids = Vec::with_capacity(inner.store.len());
        let mut eid_to_row = HashMap::with_capacity(inner.store.len());
        for (slot, (eid, value)) in inner.store.iter_decoded().enumerate() {
            let slot = slot as u32;
            data.insert(slot, value);
            eid_to_row.insert(eid.clone(), slot);
            eids.push(eid);
        }
        let locations = (0..eids.len()).map(|_| Some(FlatLocation::Ram)).collect();
        inner.flat = Some(FlatVecs {
            data,
            eids,
            dim,
            seg: None,
            n_base: 0,
            tomb: roaring::RoaringBitmap::new(),
            eid_to_row,
            locations,
            layers: Vec::new(),
        });
    }

    #[inline]
    fn live_len(inner: &FlatInner) -> usize {
        inner
            .flat
            .as_ref()
            .map(|flat| flat.live_slots().count())
            .unwrap_or_else(|| inner.store.len())
    }

    fn seal_checkpoint(
        &self,
        path: &std::path::Path,
        sequence: Option<u64>,
    ) -> Result<Option<Vec<String>>> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| anyhow!("flat lock poisoned"))?;
        Self::ensure_flat(&mut inner);
        let flat = inner.flat.as_mut().unwrap();
        let rows: Vec<(String, Vec<f32>)> = flat
            .live_slots()
            .map(|slot| (flat.eids[slot].clone(), flat.row(slot).to_vec()))
            .collect();
        let refs: Vec<Option<&[f32]>> = rows
            .iter()
            .map(|(_, value)| Some(value.as_slice()))
            .collect();
        crate::segment::write_vector_segment(
            path,
            sequence.unwrap_or(rows.len() as u64),
            flat.dim,
            &refs,
        )?;
        let reader = Arc::new(crate::segment::SegmentReader::open(path)?);
        let eids: Vec<String> = rows.into_iter().map(|(eid, _)| eid).collect();
        flat.data.clear();
        flat.eid_to_row = eids
            .iter()
            .enumerate()
            .map(|(i, eid)| (eid.clone(), i as u32))
            .collect();
        flat.eids = eids;
        flat.locations = (0..flat.eids.len())
            .map(|row| Some(FlatLocation::Base(row as u32)))
            .collect();
        flat.tomb.clear();
        flat.layers.clear();
        flat.n_base = flat.eids.len();
        flat.seg = Some(reader);
        let eids = flat.eids.clone();
        inner.store.raw.clear();
        inner.store.encoded.clear();
        Ok(Some(eids))
    }

    fn seal_to_segment_prod(&self, path: &std::path::Path) -> Result<Option<Vec<String>>> {
        self.seal_checkpoint(path, None)
    }

    pub fn open_from_segment(
        spec: VectorSpec,
        seg: Arc<crate::segment::SegmentReader>,
        row_eids: Vec<String>,
    ) -> Result<Self> {
        let dim = spec.dim as usize;
        if seg.n_docs() as usize != row_eids.len() {
            bail!("vector sidecar row count does not match segment");
        }
        let eid_to_row = row_eids
            .iter()
            .enumerate()
            .map(|(i, eid)| (eid.clone(), i as u32))
            .collect();
        let n = row_eids.len();
        let absent: roaring::RoaringBitmap = (0..n as u32)
            .filter(|row| seg.vector_at(*row, dim).is_none())
            .collect();
        let locations = (0..n as u32)
            .map(|row| (!absent.contains(row)).then_some(FlatLocation::Base(row)))
            .collect();
        Ok(Self {
            inner: Mutex::new(FlatInner {
                store: VectorStore::new(spec),
                flat: Some(FlatVecs {
                    data: HashMap::new(),
                    eids: row_eids,
                    dim,
                    seg: Some(seg),
                    n_base: n,
                    tomb: absent,
                    eid_to_row,
                    locations,
                    layers: Vec::new(),
                }),
            }),
        })
    }

    fn topk(mut candidates: Vec<(usize, f32)>, k: usize, eids: &[String]) -> Vec<(String, f32)> {
        let want = k.min(candidates.len());
        if want > 0 && want < candidates.len() {
            candidates.select_nth_unstable_by(want - 1, |a, b| {
                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            });
            candidates.truncate(want);
        }
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        candidates
            .into_iter()
            .map(|(slot, score)| (eids[slot].clone(), score))
            .collect()
    }

    fn set_ram(inner: &mut FlatInner, eid: &str, vector: &[f32]) -> Result<()> {
        let dim = inner.store.spec.dim as usize;
        if vector.len() != dim {
            bail!(
                "vector dim mismatch: expected {}, got {}",
                dim,
                vector.len()
            );
        }
        inner.store.put(eid, vector)?;
        let decoded = inner
            .store
            .get_decoded(eid)
            .unwrap_or_else(|| vector.to_vec());
        Self::ensure_flat(inner);
        let flat = inner.flat.as_mut().unwrap();
        let slot = *flat.eid_to_row.entry(eid.to_owned()).or_insert_with(|| {
            let slot = flat.eids.len() as u32;
            flat.eids.push(eid.to_owned());
            flat.locations.push(None);
            slot
        });
        flat.data.insert(slot, decoded);
        flat.locations[slot as usize] = Some(FlatLocation::Ram);
        flat.tomb.remove(slot);
        Ok(())
    }
}

impl VectorIndex for FlatCpuIndex {
    fn checkpoint_vector(&self, eid: &str) -> Result<Option<Vec<f32>>> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| anyhow!("flat lock poisoned"))?;
        Ok(inner
            .flat
            .as_ref()
            .and_then(|flat| {
                flat.eid_to_row
                    .get(eid)
                    .copied()
                    .filter(|slot| !flat.tomb.contains(*slot))
                    .map(|slot| flat.row(slot as usize).to_vec())
            })
            .or_else(|| inner.store.get_decoded(eid)))
    }

    fn checkpoint_codebook_for_preparation(&self) -> Result<Option<ScalarCodebook>> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| anyhow!("flat lock poisoned"))?;
        Ok(inner.store.codebook)
    }
    fn restore_checkpoint_vector(&self, eid: &str, vector: &[f32]) -> Result<()> {
        self.add(eid, vector)
    }
    fn add(&self, eid: &str, vector: &[f32]) -> Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| anyhow!("flat lock poisoned"))?;
        Self::set_ram(&mut inner, eid, vector)
    }
    fn remove(&self, eid: &str) -> Result<bool> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| anyhow!("flat lock poisoned"))?;
        if let Some(flat) = inner.flat.as_mut() {
            if let Some(slot) = flat.eid_to_row.get(eid).copied() {
                if !flat.tomb.contains(slot) {
                    flat.tomb.insert(slot);
                    flat.locations[slot as usize] = None;
                    flat.data.remove(&slot);
                    inner.store.drop(eid);
                    return Ok(true);
                }
            }
        }
        let removed = inner.store.drop(eid);
        if removed {
            inner.flat = None;
        }
        Ok(removed)
    }
    fn search_knn_filtered(
        &self,
        query: &[f32],
        k: usize,
        allow: &dyn Fn(&str) -> bool,
    ) -> Result<Vec<(String, f32)>> {
        use rayon::prelude::*;
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| anyhow!("flat lock poisoned"))?;
        let dim = inner.store.spec.dim as usize;
        if query.len() != dim {
            bail!(
                "kNN query dim mismatch: expected {}, got {}",
                dim,
                query.len()
            );
        }
        if k == 0 {
            return Ok(Vec::new());
        }
        let metric = inner.store.spec.metric;
        Self::ensure_flat(&mut inner);
        let flat = inner.flat.as_ref().unwrap();
        let slots: Vec<usize> = flat
            .live_slots()
            .filter(|slot| allow(&flat.eids[*slot]))
            .collect();
        let candidates = slots
            .into_par_iter()
            .map(|slot| (slot, -distance(metric, query, flat.row(slot))))
            .collect();
        Ok(Self::topk(candidates, k, &flat.eids))
    }
    fn search_knn_batch(&self, queries: &[Vec<f32>], k: usize) -> Result<Vec<Vec<(String, f32)>>> {
        for query in queries {
            if query.len()
                != self
                    .inner
                    .lock()
                    .map_err(|_| anyhow!("flat lock poisoned"))?
                    .store
                    .spec
                    .dim as usize
            {
                bail!("kNN query dim mismatch");
            }
        }
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| anyhow!("flat lock poisoned"))?;
        let metric = inner.store.spec.metric;
        Self::ensure_flat(&mut inner);
        let flat = inner.flat.as_ref().unwrap();
        let slots: Vec<usize> = flat.live_slots().collect();
        Ok(queries
            .iter()
            .map(|query| {
                Self::topk(
                    slots
                        .iter()
                        .map(|slot| (*slot, -distance(metric, query, flat.row(*slot))))
                        .collect(),
                    k,
                    &flat.eids,
                )
            })
            .collect())
    }
    fn len(&self) -> usize {
        self.inner
            .lock()
            .map(|inner| Self::live_len(&inner))
            .unwrap_or(0)
    }
    fn dump_for_snapshot(&self) -> Result<(Vec<(String, Vec<f32>)>, Option<ScalarCodebook>)> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| anyhow!("flat lock poisoned"))?;
        if let Some(flat) = inner.flat.as_ref() {
            Ok((
                flat.live_slots()
                    .map(|slot| (flat.eids[slot].clone(), flat.row(slot).to_vec()))
                    .collect(),
                inner.store.codebook.clone(),
            ))
        } else {
            Ok((
                inner.store.iter_decoded().collect(),
                inner.store.codebook.clone(),
            ))
        }
    }
    fn seal_to_segment_prod(&self, path: &std::path::Path) -> Result<Option<Vec<String>>> {
        self.seal_to_segment_prod(path)
    }
    fn seal_to_segment_prod_at(
        &self,
        path: &std::path::Path,
        seq: u64,
    ) -> Result<Option<Vec<String>>> {
        self.seal_checkpoint(path, Some(seq))
    }
    fn seal_releases_ram(&self) -> bool {
        true
    }
    fn resident_vector_payload_rows(&self) -> usize {
        self.inner
            .lock()
            .map(|inner| {
                let mut ids: std::collections::HashSet<&str> = inner
                    .store
                    .raw
                    .keys()
                    .chain(inner.store.encoded.keys())
                    .map(String::as_str)
                    .collect();
                if let Some(flat) = &inner.flat {
                    ids.extend(
                        flat.data
                            .keys()
                            .map(|slot| flat.eids[*slot as usize].as_str()),
                    );
                }
                ids.len()
            })
            .unwrap_or(0)
    }
    fn has_checkpoint_mapping(&self) -> bool {
        self.inner
            .lock()
            .map(|inner| {
                inner
                    .flat
                    .as_ref()
                    .is_some_and(|flat| flat.seg.is_some() || !flat.layers.is_empty())
            })
            .unwrap_or(false)
    }

    fn checkpoint_resident_bytes(&self) -> Option<u64> {
        let inner = self.inner.lock().expect("flat lock poisoned");
        let row_bytes = |eid: &str| u64::from(inner.store.spec.dim) * 4 + eid.len() as u64;
        let stored = inner.store.raw.keys().chain(inner.store.encoded.keys());
        let mut bytes: u64 = stored.map(|eid| row_bytes(eid)).sum();
        if let Some(flat) = &inner.flat {
            for slot in flat.data.keys() {
                let eid = &flat.eids[*slot as usize];
                if !inner.store.raw.contains_key(eid) && !inner.store.encoded.contains_key(eid) {
                    bytes += row_bytes(eid);
                }
            }
        }
        Some(bytes)
    }
    #[cfg(test)]
    fn __seal_flat_to_segment(&self, path: &std::path::Path) -> Result<Option<u32>> {
        self.seal_checkpoint(path, None)
            .map(|rows| rows.map(|rows| rows.len() as u32))
    }

    fn install_checkpoint_base(
        &self,
        reader: Arc<crate::segment::SegmentReader>,
        external_ids: &[String],
        acknowledged: &HashMap<String, bool>,
    ) -> Result<()> {
        if reader.n_docs() as usize != external_ids.len() {
            bail!("vector base row map does not match payload");
        }
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| anyhow!("flat lock poisoned"))?;
        Self::ensure_flat(&mut inner);
        let mut old = inner.flat.take().unwrap();
        let mut eids = external_ids.to_vec();
        let mut eid_to_row: HashMap<String, u32> = eids
            .iter()
            .enumerate()
            .map(|(slot, eid)| (eid.clone(), slot as u32))
            .collect();
        let mut tomb: roaring::RoaringBitmap = (0..eids.len() as u32)
            .filter(|row| reader.vector_at(*row, old.dim).is_none())
            .collect();
        let mut locations: Vec<Option<FlatLocation>> = (0..eids.len() as u32)
            .map(|row| (!tomb.contains(row)).then_some(FlatLocation::Base(row)))
            .collect();
        let mut data = HashMap::new();
        // Move only newer RAM payloads. No captured vector payload is retained.
        for (eid, keep_newer) in acknowledged {
            if *keep_newer {
                inner.store.drop(eid);
                continue;
            }
            let Some(old_slot) = old.eid_to_row.get(eid).copied() else {
                continue;
            };
            let slot = *eid_to_row.entry(eid.clone()).or_insert_with(|| {
                let slot = eids.len() as u32;
                eids.push(eid.clone());
                locations.push(None);
                slot
            });
            if old.tomb.contains(old_slot) {
                tomb.insert(slot);
                continue;
            }
            tomb.remove(slot);
            let location = old.locations[old_slot as usize];
            if let Some(FlatLocation::Ram) = location {
                if let Some(value) = old.data.remove(&old_slot) {
                    data.insert(slot, value);
                }
            }
            locations[slot as usize] = location;
        }
        inner.flat = Some(FlatVecs {
            data,
            eids,
            dim: old.dim,
            seg: Some(reader),
            n_base: external_ids.len(),
            tomb,
            eid_to_row,
            locations,
            layers: old.layers,
        });
        Ok(())
    }

    fn checkpoint_delta_readers(&self) -> Vec<Arc<crate::segment::SegmentReader>> {
        self.inner
            .lock()
            .expect("flat lock poisoned")
            .flat
            .as_ref()
            .map(|flat| {
                flat.layers
                    .iter()
                    .map(|layer| layer.reader.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn checkpoint_base_reader(&self) -> Option<Arc<crate::segment::SegmentReader>> {
        self.inner
            .lock()
            .expect("flat lock poisoned")
            .flat
            .as_ref()
            .and_then(|flat| flat.seg.clone())
    }

    fn replace_checkpoint_base(
        &self,
        base: &Arc<crate::segment::SegmentReader>,
        inputs: &[Arc<crate::segment::SegmentReader>],
        reader: Arc<crate::segment::SegmentReader>,
        external_ids: &[String],
    ) -> Result<()> {
        if reader.n_docs() as usize != external_ids.len() {
            bail!("compacted vector base row count mismatch");
        }
        let rows: HashMap<&str, u32> = external_ids
            .iter()
            .enumerate()
            .map(|(row, eid)| (eid.as_str(), row as u32))
            .collect();
        if rows.len() != external_ids.len() {
            bail!("duplicate compacted vector base ID");
        }
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| anyhow!("flat lock poisoned"))?;
        let flat = inner
            .flat
            .as_mut()
            .ok_or_else(|| anyhow!("vector base replacement has no live base"))?;
        if !flat
            .seg
            .as_ref()
            .is_some_and(|actual| Arc::ptr_eq(actual, base))
            || inputs.len() > flat.layers.len()
            || !flat
                .layers
                .iter()
                .zip(inputs)
                .all(|(layer, input)| Arc::ptr_eq(&layer.reader, input))
        {
            bail!("compacted vector base inputs no longer match live layers");
        }
        let retargets: Vec<(usize, u32)> = flat.locations.iter().enumerate().filter_map(|(slot, source)| {
            (matches!(source, Some(FlatLocation::Base(_)))
                || matches!(source, Some(FlatLocation::Layer { layer, .. }) if (*layer as usize) < inputs.len()))
                .then_some(slot)
        }).map(|slot| {
            let row = *rows.get(flat.eids[slot].as_str()).ok_or_else(|| anyhow!("compacted vector base lost a live ID"))?;
            if reader.vector_at(row, flat.dim).is_none() { bail!("compacted vector base lost a live row"); }
            Ok((slot, row))
        }).collect::<Result<_>>()?;
        for source in &mut flat.locations {
            if let Some(FlatLocation::Layer { layer, .. }) = source {
                if (*layer as usize) >= inputs.len() {
                    *layer -= u32::try_from(inputs.len())?;
                }
            }
        }
        for (slot, row) in retargets {
            flat.locations[slot] = Some(FlatLocation::Base(row));
        }
        flat.layers.drain(..inputs.len());
        flat.seg = Some(reader);
        flat.n_base = external_ids.len();
        Ok(())
    }

    fn replace_checkpoint_deltas(
        &self,
        inputs: &[Arc<crate::segment::SegmentReader>],
        reader: Arc<crate::segment::SegmentReader>,
        external_ids: &[String],
    ) -> Result<()> {
        if inputs.is_empty() || reader.n_docs() as usize != external_ids.len() {
            bail!("invalid compacted vector inputs or row map");
        }
        let rows: HashMap<&str, u32> = external_ids
            .iter()
            .enumerate()
            .map(|(row, eid)| (eid.as_str(), row as u32))
            .collect();
        if rows.len() != external_ids.len() {
            bail!("duplicate compacted vector external ID");
        }
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| anyhow!("flat lock poisoned"))?;
        let flat = inner
            .flat
            .as_mut()
            .ok_or_else(|| anyhow!("vector compaction has no live layers"))?;
        let start = flat
            .layers
            .windows(inputs.len())
            .position(|window| {
                window
                    .iter()
                    .zip(inputs)
                    .all(|(layer, expected)| Arc::ptr_eq(&layer.reader, expected))
            })
            .ok_or_else(|| anyhow!("compacted vector inputs no longer match live layers"))?;
        let end = start + inputs.len();
        let output_layer = u32::try_from(start)?;
        // Validate every retarget before changing a slot. RAM mutations and
        // tombstones are newer than these immutable inputs and stay untouched.
        let retargets: Vec<(usize, u32)> = flat.locations.iter().enumerate().filter_map(|(slot, source)| {
            matches!(source, Some(FlatLocation::Layer { layer, .. }) if (start..end).contains(&(*layer as usize)))
                .then_some(slot)
        }).map(|slot| {
            let row = *rows.get(flat.eids[slot].as_str())
                .ok_or_else(|| anyhow!("compacted vector lost a live input ID"))?;
            if reader.vector_at(row, flat.dim).is_none() {
                bail!("compacted vector lost a live input row");
            }
            Ok((slot, row))
        }).collect::<Result<_>>()?;
        for source in &mut flat.locations {
            if let Some(FlatLocation::Layer { layer, .. }) = source {
                if (*layer as usize) >= end {
                    *layer -= u32::try_from(inputs.len() - 1)?;
                }
            }
        }
        for (slot, row) in retargets {
            flat.locations[slot] = Some(FlatLocation::Layer {
                layer: output_layer,
                row,
            });
        }
        flat.layers.splice(start..end, [FlatLayer { reader }]);
        Ok(())
    }

    fn attach_checkpoint_delta(
        &self,
        reader: Arc<crate::segment::SegmentReader>,
        external_ids: &[String],
        acknowledged: &[bool],
    ) -> Result<()> {
        if reader.n_docs() as usize != external_ids.len()
            || external_ids.len() != acknowledged.len()
        {
            bail!("vector checkpoint row map does not match payload");
        }
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| anyhow!("flat lock poisoned"))?;
        Self::ensure_flat(&mut inner);
        let FlatInner { store, flat } = &mut *inner;
        let flat = flat.as_mut().unwrap();
        let layer = flat.layers.len() as u32;
        flat.layers.push(FlatLayer {
            reader: reader.clone(),
        });
        for (row, (eid, ack)) in external_ids.iter().zip(acknowledged).enumerate() {
            if !ack {
                continue;
            };
            let slot = *flat.eid_to_row.entry(eid.clone()).or_insert_with(|| {
                let slot = flat.eids.len() as u32;
                flat.eids.push(eid.clone());
                flat.locations.push(None);
                slot
            });
            flat.data.remove(&slot);
            store.drop(eid);
            if reader.vector_at(row as u32, flat.dim).is_some() {
                flat.tomb.remove(slot);
                flat.locations[slot as usize] = Some(FlatLocation::Layer {
                    layer,
                    row: row as u32,
                });
            } else {
                flat.tomb.insert(slot);
                flat.locations[slot as usize] = None;
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for FlatCpuIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FlatCpuIndex")
            .field("len", &self.len())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Backend selection
// ---------------------------------------------------------------------------

/// Construct the backend implied by `spec.backend`. This version ships
/// CPU backends only (HNSW + flat brute-force); GPU-native vector search
/// is a future chapter.
pub fn open_backend(spec: VectorSpec) -> Box<dyn VectorIndex> {
    match spec.backend {
        crate::types::VectorBackend::HnswCpu => Box::new(HnswCpuIndex::new(spec)),
        crate::types::VectorBackend::FlatCpu => Box::new(FlatCpuIndex::new(spec)),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[test]
    fn preparation_codebook_snapshot_is_copy_sized_for_raw_and_sq_backends() {
        for index in [
            Box::new(HnswCpuIndex::new(spec(3, VectorMetric::L2, None))) as Box<dyn VectorIndex>,
            Box::new(FlatCpuIndex::new(spec(3, VectorMetric::L2, None))) as Box<dyn VectorIndex>,
        ] {
            assert!(index
                .checkpoint_codebook_for_preparation()
                .unwrap()
                .is_none());
        }
        for index in [
            Box::new(HnswCpuIndex::new(spec(
                3,
                VectorMetric::L2,
                Some(VectorQuantize::Sq),
            ))) as Box<dyn VectorIndex>,
            Box::new(FlatCpuIndex::new(spec(
                3,
                VectorMetric::L2,
                Some(VectorQuantize::Sq),
            ))) as Box<dyn VectorIndex>,
        ] {
            let initial = index
                .checkpoint_codebook_for_preparation()
                .unwrap()
                .unwrap();
            assert_eq!(initial.dim, 3);
            assert!(initial.min.is_infinite() && initial.min.is_sign_positive());
            assert!(initial.max.is_infinite() && initial.max.is_sign_negative());
            index.add("first", &[-2.0, 0.0, 5.0]).unwrap();
            let snapshot = index
                .checkpoint_codebook_for_preparation()
                .unwrap()
                .unwrap();
            assert_eq!((snapshot.min, snapshot.max, snapshot.dim), (-2.0, 5.0, 3));
            let mut widened = snapshot;
            widened.widen(&[-20.0, 0.0, 20.0]);
            assert_eq!(
                index
                    .checkpoint_codebook_for_preparation()
                    .unwrap()
                    .unwrap()
                    .min,
                -2.0
            );
            assert_eq!(
                index
                    .checkpoint_codebook_for_preparation()
                    .unwrap()
                    .unwrap()
                    .max,
                5.0
            );
            index.add("later", &[10.0, 0.0, 20.0]).unwrap();
            let later = index
                .checkpoint_codebook_for_preparation()
                .unwrap()
                .unwrap();
            assert_eq!((later.min, later.max, later.dim), (-2.0, 20.0, 3));
        }
    }

    use super::*;

    fn rand_vec(rng: &mut rand::rngs::StdRng, dim: usize) -> Vec<f32> {
        use rand::Rng;
        (0..dim).map(|_| rng.gen_range(-1.0_f32..1.0)).collect()
    }

    fn spec(dim: u32, metric: VectorMetric, q: Option<VectorQuantize>) -> VectorSpec {
        VectorSpec {
            dim,
            metric,
            backend: crate::types::VectorBackend::HnswCpu,
            quantize: q,
        }
    }

    #[test]
    fn hnsw_add_reports_one_consumable_write_lock_timing() {
        let index = HnswCpuIndex::new(spec(3, VectorMetric::L2, None));
        index.add("one", &[1.0, 0.0, 0.0]).unwrap();
        let _timing = index
            .take_hnsw_write_lock_timing()
            .expect("HNSW add must report its lock split");
        assert!(
            index.take_hnsw_write_lock_timing().is_none(),
            "committed apply must not publish the same HNSW add twice"
        );
        assert!(
            index.take_hnsw_graph_rebuild_timing().is_none(),
            "the first HNSW add does not rebuild the graph"
        );
    }

    #[test]
    fn hnsw_rebuild_reports_one_consumable_timing() {
        let index = HnswCpuIndex::new(spec(3, VectorMetric::L2, None));
        index.add("one", &[1.0, 0.0, 0.0]).unwrap();
        let _ = index.take_hnsw_write_lock_timing();
        index.remove("one").unwrap();
        let _ = index.take_hnsw_write_lock_timing();
        index.add("one", &[0.0, 1.0, 0.0]).unwrap();

        assert!(
            index.take_hnsw_graph_rebuild_timing().is_some(),
            "replacing the only orphaned HNSW vector rebuilds the graph"
        );
        assert!(
            index.take_hnsw_graph_rebuild_timing().is_none(),
            "committed apply must not publish the same rebuild twice"
        );
    }

    #[test]
    fn hnsw_poisoned_write_operations_clear_stale_lock_timing() {
        fn poison(index: &HnswCpuIndex) {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _guard = index.inner.write().expect("fresh HNSW lock");
                panic!("poison HNSW lock for timing test");
            }));
        }

        let add_index = HnswCpuIndex::new(spec(3, VectorMetric::L2, None));
        add_index.add("one", &[1.0, 0.0, 0.0]).unwrap();
        poison(&add_index);
        assert!(add_index.add("two", &[0.0, 1.0, 0.0]).is_err());
        assert!(
            add_index.take_hnsw_write_lock_timing().is_none(),
            "a failed poisoned add must not expose timing from a prior write"
        );

        let remove_index = HnswCpuIndex::new(spec(3, VectorMetric::L2, None));
        remove_index.add("one", &[1.0, 0.0, 0.0]).unwrap();
        poison(&remove_index);
        assert!(remove_index.remove("one").is_err());
        assert!(
            remove_index.take_hnsw_write_lock_timing().is_none(),
            "a failed poisoned remove must not expose timing from a prior write"
        );
    }

    #[test]
    fn hnsw_exact_fallback_does_not_enter_the_owned_store_iterator() {
        for quantize in [None, Some(VectorQuantize::Sq)] {
            let index = HnswCpuIndex::new(spec(3, VectorMetric::L2, quantize));
            index.set_ef_search(32);
            for i in 0..128 {
                index.add(&format!("v{i}"), &[i as f32, 1.0, 0.0]).unwrap();
            }
            VECTOR_STORE_OWNED_SCANS.with(|scans| scans.set(0));
            let before = index.exact_scan_fallbacks();
            assert!(index
                .search_knn_filtered(&[0.0, 1.0, 0.0], 5, &|_| false)
                .unwrap()
                .is_empty());
            assert_eq!(index.exact_scan_fallbacks(), before + 1);
            assert_eq!(
                VECTOR_STORE_OWNED_SCANS.with(|scans| scans.get()),
                0,
                "fallback must filter and score borrowed values, quantize={quantize:?}"
            );
        }
    }

    #[test]
    fn hnsw_nearest_orphans_do_not_repeat_an_inconclusive_graph_search() {
        const N: usize = 256;
        const K: usize = 5;
        let index = HnswCpuIndex::new(spec(3, VectorMetric::L2, None));
        // Exhaustive on this small fixture, so topology cannot hide the orphan.
        index.set_ef_search(N + K);
        for i in 0..N {
            index
                .add(&format!("v{i:03}"), &[i as f32, 0.0, 0.0])
                .unwrap();
        }
        for i in 0..K {
            index
                .add(&format!("v{i:03}"), &[(1000 + i) as f32, 0.0, 0.0])
                .unwrap();
        }
        HNSW_SEARCH_POOLS.with(|pools| pools.borrow_mut().clear());
        let before = index.exact_scan_fallbacks();
        let hits = index
            .search_knn_filtered(&[0.0, 0.0, 0.0], K, &|_| true)
            .unwrap();
        let expected: Vec<_> = (K..2 * K)
            .map(|i| (format!("v{i:03}"), -(i as f32)))
            .collect();
        assert_eq!(hits, expected);
        assert_eq!(index.exact_scan_fallbacks(), before + 1);
        let pools = HNSW_SEARCH_POOLS.with(|pools| pools.borrow().clone());
        assert_eq!(
            pools.len(),
            1,
            "an unresolved nearest orphan must go directly to the exact fallback: {pools:?}"
        );
    }

    #[test]
    fn hnsw_selective_search_bounds_graph_pool_before_exact_fallback() {
        const N: usize = 512;
        const K: usize = 5;
        const EF: usize = 64;
        let index = HnswCpuIndex::new(spec(3, VectorMetric::L2, None));
        index.set_ef_search(EF);
        for i in 0..N {
            index
                .add(&format!("v{i:03}"), &[i as f32, 0.0, 0.0])
                .unwrap();
        }
        HNSW_SEARCH_POOLS.with(|pools| pools.borrow_mut().clear());
        let before = index.exact_scan_fallbacks();
        let hits = index
            .search_knn_filtered(&[0.0, 0.0, 0.0], K, &|eid| eid == "v511")
            .unwrap();
        assert_eq!(hits, vec![("v511".to_owned(), -511.0)]);
        assert_eq!(index.exact_scan_fallbacks(), before + 1);
        let pools = HNSW_SEARCH_POOLS.with(|pools| pools.borrow().clone());
        assert!(
            !pools.is_empty(),
            "the declared graph backend must be consulted"
        );
        assert!(
            pools.iter().all(|&pool| pool <= EF.max(5 * K)),
            "a selective filter must not grow traversal to corpus size: {pools:?}"
        );
    }

    #[test]
    fn hnsw_clean_permissive_search_keeps_the_graph_answer() {
        let index = HnswCpuIndex::new(spec(3, VectorMetric::L2, None));
        index.set_ef_search(64);
        for i in 0..128 {
            index
                .add(&format!("v{i:03}"), &[i as f32, 0.0, 0.0])
                .unwrap();
        }
        HNSW_SEARCH_POOLS.with(|pools| pools.borrow_mut().clear());
        let before = index.exact_scan_fallbacks();
        let hits = index
            .search_knn_filtered(&[0.0, 0.0, 0.0], 5, &|_| true)
            .unwrap();
        let expected: Vec<_> = (0..5).map(|i| (format!("v{i:03}"), -(i as f32))).collect();
        assert_eq!(hits, expected);
        assert_eq!(index.exact_scan_fallbacks(), before);
        assert_eq!(
            HNSW_SEARCH_POOLS.with(|pools| pools.borrow().clone()),
            vec![25]
        );
    }

    // -----------------------------------------------------------------
    // Metric directionality (Contract 1). These assert the *sign* and
    // *ordering* of each distance, so a mutated operator in distance() /
    // l2_squared() / dot() / cosine_similarity() flips a comparison and
    // fails. The score = -distance convention means "closer ⇒ larger
    // score" must hold for every metric.
    // -----------------------------------------------------------------

    #[test]
    fn l2_distance_grows_with_separation() {
        let q = [0.0_f32, 0.0, 0.0];
        let near = [1.0_f32, 0.0, 0.0];
        let far = [5.0_f32, 0.0, 0.0];
        let dn = distance(VectorMetric::L2, &q, &near);
        let df = distance(VectorMetric::L2, &q, &far);
        assert!(dn >= 0.0, "L2 distance is non-negative");
        assert!(df > dn, "farther vector must have larger L2 distance");
        // identical vectors → zero distance
        assert!(distance(VectorMetric::L2, &q, &q).abs() < 1e-6);
    }

    #[test]
    fn cosine_distance_smaller_for_aligned_vectors() {
        let q = [1.0_f32, 0.0];
        let aligned = [2.0_f32, 0.0]; // same direction
        let orthogonal = [0.0_f32, 3.0];
        let opposite = [-1.0_f32, 0.0];
        let d_aligned = distance(VectorMetric::Cosine, &q, &aligned);
        let d_orth = distance(VectorMetric::Cosine, &q, &orthogonal);
        let d_opp = distance(VectorMetric::Cosine, &q, &opposite);
        // cosine distance = 1 - cos θ : aligned≈0, orthogonal≈1, opposite≈2
        assert!(d_aligned < d_orth, "aligned closer than orthogonal");
        assert!(d_orth < d_opp, "orthogonal closer than opposite");
        assert!(d_aligned.abs() < 1e-5, "aligned cosine distance ≈ 0");
    }

    #[test]
    fn dot_distance_is_negative_dot_so_larger_dot_is_closer() {
        let q = [1.0_f32, 1.0];
        let high = [2.0_f32, 2.0]; // dot = 4
        let low = [0.5_f32, 0.5]; // dot = 1
        let d_high = distance(VectorMetric::Dot, &q, &high);
        let d_low = distance(VectorMetric::Dot, &q, &low);
        // distance = -dot ; higher dot ⇒ smaller (more negative) distance
        assert!(
            d_high < d_low,
            "higher dot product must be closer (smaller distance)"
        );
        assert!((d_high + 4.0).abs() < 1e-5, "dot distance == -dot");
    }

    #[test]
    fn knn_orders_by_increasing_distance_for_each_metric() {
        // L2 + Cosine accept arbitrary vectors. (Dot's HNSW backend
        // requires unit-normalized input — its directionality is pinned
        // by `dot_distance_is_negative_dot_so_larger_dot_is_closer` at
        // the math level instead.)
        for metric in [VectorMetric::L2, VectorMetric::Cosine] {
            let idx = HnswCpuIndex::new(spec(3, metric, None));
            idx.add("near", &[1.0, 0.0, 0.0]).unwrap();
            idx.add("mid", &[1.0, 1.0, 0.0]).unwrap();
            idx.add("far", &[-1.0, 0.0, 0.0]).unwrap();
            let hits = idx.search_knn(&[1.0, 0.0, 0.0], 3).unwrap();
            // score = -distance ⇒ scores must be non-increasing down the list.
            for w in hits.windows(2) {
                assert!(
                    w[0].1 >= w[1].1,
                    "metric {metric:?}: scores must be sorted desc, got {hits:?}"
                );
            }
            // the exact-match query vector ("near") must be the top hit.
            assert_eq!(
                hits[0].0, "near",
                "metric {metric:?}: nearest is the query itself"
            );
        }
    }

    #[test]
    fn filtered_knn_returns_nearest_within_allowlist_not_global_topk() {
        // 50 vectors along a 1-D ray: v{i} at distance i from the query.
        // Enough nodes that HNSW recall is reliable (a 3-node graph is
        // randomized enough to flake), and enough that we can deny a
        // prefix longer than the initial over-fetch pool to exercise the
        // widening loop.
        let idx = HnswCpuIndex::new(spec(3, VectorMetric::L2, None));
        let n = 50usize;
        for i in 0..n {
            idx.add(&format!("v{i:02}"), &[i as f32, 0.0, 0.0]).unwrap();
        }
        let query = [0.0_f32, 0.0, 0.0];
        let allowed_from =
            |eid: &str| -> bool { eid.trim_start_matches('v').parse::<usize>().unwrap() >= 20 };

        // Baseline: unfiltered nearest is v00.
        let all = idx.search_knn(&query, 3).unwrap();
        assert_eq!(all[0].0, "v00", "unfiltered nearest is the closest vector");

        // Deny v00..=v19 — a 20-wide prefix, wider than the k*4+k=15
        // initial pool, so a post-filter over the global top-k would
        // return nothing and the widening loop must kick in.
        let k = 3;
        let hits = idx.search_knn_filtered(&query, k, &allowed_from).unwrap();
        assert_eq!(hits.len(), k, "selective filter must not collapse recall");
        for (eid, _) in &hits {
            let i: usize = eid.trim_start_matches('v').parse().unwrap();
            assert!(i >= 20, "denied id {eid} leaked past the allow-list");
        }
        for w in hits.windows(2) {
            assert!(w[0].1 >= w[1].1, "scores sorted desc: {hits:?}");
        }
        // Nearest allowed neighbour ranks first.
        assert_eq!(hits[0].0, "v20", "nearest allowed neighbour leads");

        // Allow-nothing → empty, never an error.
        let none = idx.search_knn_filtered(&query, k, &|_| false).unwrap();
        assert!(none.is_empty(), "empty allow-list yields no hits");
    }

    #[test]
    fn knn_dot_orders_normalized_vectors_by_alignment() {
        // Dot HNSW requires unit-normalized vectors. With those, the
        // most-aligned vector to the query must rank first.
        let idx = HnswCpuIndex::new(spec(2, VectorMetric::Dot, None));
        idx.add("aligned", &[1.0, 0.0]).unwrap();
        idx.add("diag", &[0.7071, 0.7071]).unwrap();
        idx.add("orthogonal", &[0.0, 1.0]).unwrap();
        let hits = idx.search_knn(&[1.0, 0.0], 3).unwrap();
        for w in hits.windows(2) {
            assert!(w[0].1 >= w[1].1, "dot scores must be sorted desc: {hits:?}");
        }
        assert_eq!(hits[0].0, "aligned", "most-aligned vector ranks first");
    }

    #[test]
    fn hnsw_returns_self_as_top_neighbour() {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(7);
        let idx = HnswCpuIndex::new(spec(64, VectorMetric::L2, None));
        let mut all = Vec::new();
        for i in 0..200 {
            let v = rand_vec(&mut rng, 64);
            idx.add(&format!("e{i}"), &v).unwrap();
            all.push((format!("e{i}"), v));
        }
        // Each inserted vector should be its own nearest neighbour.
        let (eid, q) = &all[42];
        let hits = idx.search_knn(q, 1).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0, *eid);
    }

    #[test]
    fn hnsw_1000_vectors_topk_returns_reasonable_neighbours() {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(11);
        let dim = 128;
        let idx = HnswCpuIndex::new(spec(dim, VectorMetric::L2, None));
        let mut data: Vec<(String, Vec<f32>)> = Vec::new();
        for i in 0..1_000 {
            let v = rand_vec(&mut rng, dim as usize);
            idx.add(&format!("v{i}"), &v).unwrap();
            data.push((format!("v{i}"), v));
        }
        let q = rand_vec(&mut rng, dim as usize);
        let hits = idx.search_knn(&q, 10).unwrap();
        assert_eq!(hits.len(), 10);
        // Scores should be monotone-non-increasing (higher = better).
        for w in hits.windows(2) {
            assert!(w[0].1 >= w[1].1, "non-monotone scores: {:?}", hits);
        }
        // The top-10 should be a reasonable approximation of the true
        // top-10 by brute force — at minimum, overlap ≥ 4. (HNSW with
        // 1k random points and dim=128 is approximate, not exact.)
        let mut by_dist: Vec<(String, f32)> = data
            .iter()
            .map(|(e, v)| (e.clone(), l2_squared(&q, v).sqrt()))
            .collect();
        by_dist.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let truth_top10: std::collections::HashSet<&str> =
            by_dist.iter().take(10).map(|(e, _)| e.as_str()).collect();
        let hnsw_top10: std::collections::HashSet<&str> =
            hits.iter().map(|(e, _)| e.as_str()).collect();
        let overlap = truth_top10.intersection(&hnsw_top10).count();
        assert!(overlap >= 4, "overlap with truth top-10 was {overlap}");
    }

    fn normalize(mut v: Vec<f32>) -> Vec<f32> {
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-12);
        for x in &mut v {
            *x /= norm;
        }
        v
    }

    #[test]
    fn cosine_dot_l2_all_produce_well_formed_results() {
        use rand::SeedableRng;
        for metric in [VectorMetric::Cosine, VectorMetric::Dot, VectorMetric::L2] {
            let mut rng = rand::rngs::StdRng::seed_from_u64(3);
            let idx = HnswCpuIndex::new(spec(32, metric, None));
            for i in 0..50 {
                // `DistDot` in `hnsw_rs` assumes unit-norm inputs
                // (returns `1 - dot` with an assert that the result is
                // non-negative). Normalize for the dot case; the other
                // two work on raw vectors.
                let raw = rand_vec(&mut rng, 32);
                let v = if matches!(metric, VectorMetric::Dot) {
                    normalize(raw)
                } else {
                    raw
                };
                idx.add(&format!("e{i}"), &v).unwrap();
            }
            let raw_q = rand_vec(&mut rng, 32);
            let q = if matches!(metric, VectorMetric::Dot) {
                normalize(raw_q)
            } else {
                raw_q
            };
            let hits = idx.search_knn(&q, 5).unwrap();
            assert_eq!(hits.len(), 5, "metric {metric:?}");
            for w in hits.windows(2) {
                assert!(w[0].1 >= w[1].1, "metric {metric:?} non-monotone");
            }
        }
    }

    #[test]
    fn sq_codec_round_trip_within_tolerance() {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(2);
        let dim = 128;
        let v = rand_vec(&mut rng, dim);
        let mut cb = ScalarCodebook::empty(dim);
        cb.widen(&v);
        let bytes = encode_sq(&v, &cb);
        assert_eq!(bytes.len(), dim);
        let back = decode_sq(&bytes, &cb);
        let span = cb.max - cb.min;
        let tol = span / 255.0;
        let mean_err: f32 = v
            .iter()
            .zip(back.iter())
            .map(|(a, b)| (a - b).abs())
            .sum::<f32>()
            / dim as f32;
        assert!(
            mean_err <= tol,
            "mean SQ error {mean_err} > 1/255 of range {tol}"
        );
    }

    #[test]
    fn sq_round_trip_through_hnsw_index() {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(5);
        let idx = HnswCpuIndex::new(spec(64, VectorMetric::L2, Some(VectorQuantize::Sq)));
        let mut data = Vec::new();
        for i in 0..100 {
            let v = rand_vec(&mut rng, 64);
            idx.add(&format!("e{i}"), &v).unwrap();
            data.push((format!("e{i}"), v));
        }
        // Query the index with one of the inserted vectors; the index
        // should still return that vector as the top hit even though
        // storage is u8-quantized.
        let (eid, v) = &data[10];
        let hits = idx.search_knn(v, 1).unwrap();
        assert_eq!(hits[0].0, *eid);
    }

    #[test]
    fn one_item_insert_search_and_reopen_works_with_and_without_sq() {
        for quantize in [None, Some(VectorQuantize::Sq)] {
            let spec = spec(3, VectorMetric::L2, quantize);
            let idx = HnswCpuIndex::new(spec);
            let vector = [1.0_f32, 2.0, 3.0];
            idx.add("one", &vector).unwrap();
            assert_eq!(idx.search_knn(&vector, 1).unwrap()[0].0, "one");

            let dir = tempfile::tempdir().unwrap();
            let segment = dir.path().join("vectors.lseg");
            let row_eids = idx.seal_to_segment_prod(&segment).unwrap().unwrap();
            let reader = std::sync::Arc::new(crate::segment::SegmentReader::open(&segment).unwrap());
            let reopened = HnswCpuIndex::open_from_segment(spec, reader, row_eids).unwrap();
            assert_eq!(reopened.search_knn(&vector, 1).unwrap()[0].0, "one");
        }
    }

    #[test]
    fn snapshot_round_trip_preserves_neighbours() {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(13);
        let dim = 32u32;
        let s = spec(dim, VectorMetric::L2, None);
        let idx = HnswCpuIndex::new(s);
        for i in 0..50 {
            let v = rand_vec(&mut rng, dim as usize);
            idx.add(&format!("e{i}"), &v).unwrap();
        }
        // Use a stored vector as the query so the top-1 neighbour is an
        // exact match — that ranking is stable across the two
        // independently-built (approximate) HNSW graphs.
        let q = idx.inner.read().unwrap().store.get_decoded("e25").unwrap();
        let before = idx.search_knn(&q, 5).unwrap();

        let (vecs, cb) = idx.dump_for_snapshot().unwrap();

        // Invariant 1 (deterministic): snapshot preserves the exact set
        // of stored vectors. This is the real durability contract — the
        // approximate graph is rebuilt, but no vector is lost or altered.
        let restored = HnswCpuIndex::restore(s, vecs.clone(), cb).unwrap();
        assert_eq!(restored.len(), idx.len());
        let (vecs2, _) = restored.dump_for_snapshot().unwrap();
        let mut a: Vec<_> = vecs.iter().map(|(e, v)| (e.clone(), v.clone())).collect();
        let mut b: Vec<_> = vecs2.iter().map(|(e, v)| (e.clone(), v.clone())).collect();
        a.sort_by(|x, y| x.0.cmp(&y.0));
        b.sort_by(|x, y| x.0.cmp(&y.0));
        assert_eq!(
            a, b,
            "snapshot→restore must preserve every (eid, vector) exactly"
        );

        // Invariant 2 (robust): the exact-match query still tops kNN
        // after restore.
        let after = restored.search_knn(&q, 5).unwrap();
        assert_eq!(before[0].0, "e25");
        assert_eq!(after[0].0, "e25", "exact-match neighbour survives restore");
    }

    // -----------------------------------------------------------------
    // Phase 2k-1: composed base-segment + live-tail + tombstone model for
    // the flat-cpu index. After a reopen-from-segment the base vectors live
    // ONLY on the mmap; a TAIL of adds and a mix of base+tail DELETEs must
    // compose so kNN is byte-identical to an in-RAM oracle that ran the same
    // ops. This is the direct proof that the three pieces compose correctly.
    // -----------------------------------------------------------------
    #[test]
    fn reopen_base_seg_plus_tail_plus_tombstone_equals_inram_oracle() {
        use rand::SeedableRng;

        fn flat_spec(dim: u32, metric: VectorMetric) -> VectorSpec {
            VectorSpec {
                dim,
                metric,
                backend: crate::types::VectorBackend::FlatCpu,
                quantize: None,
            }
        }

        for metric in [VectorMetric::L2, VectorMetric::Cosine, VectorMetric::Dot] {
            let mut rng = rand::rngs::StdRng::seed_from_u64(0xBADCAFE ^ metric as u64);
            let dim = 12u32;
            let s = flat_spec(dim, metric);

            // 30 BASE vectors. (Dot wants unit-norm inputs.)
            let mk = |rng: &mut rand::rngs::StdRng| -> Vec<f32> {
                let raw = rand_vec(rng, dim as usize);
                if matches!(metric, VectorMetric::Dot) {
                    normalize(raw)
                } else {
                    raw
                }
            };
            let base_idx = FlatCpuIndex::new(s);
            let mut all: Vec<(String, Vec<f32>)> = Vec::new();
            for i in 0..30usize {
                let v = mk(&mut rng);
                base_idx.add(&format!("b{i}"), &v).unwrap();
                all.push((format!("b{i}"), v));
            }

            // SEAL the base to a segment, then REOPEN from it: the base vectors are
            // now ONLY on the mmap (open_from_segment does NOT store.put them).
            let dir = tempfile::tempdir().unwrap();
            let seg_path = dir.path().join("emb.lseg");
            let row_eids = base_idx
                .seal_to_segment_prod(&seg_path)
                .unwrap()
                .expect("flat-cpu seal returns row eids");
            let reader =
                std::sync::Arc::new(crate::segment::SegmentReader::open(&seg_path).unwrap());
            let reopened = FlatCpuIndex::open_from_segment(s, reader, row_eids).unwrap();

            // The store must NOT hold the base vectors (they live on the mmap) —
            // this is the RAM-bound invariant. `len` still reports all 30 (live).
            {
                let inner = reopened.inner.lock().unwrap();
                assert_eq!(
                    inner.store.len(),
                    0,
                    "{metric:?}: reopen must NOT re-store base vectors"
                );
                let flat = inner.flat.as_ref().unwrap();
                assert!(flat.seg.is_some(), "{metric:?}: segment attached");
                assert_eq!(flat.n_base, 30, "{metric:?}: all 30 rows are base");
                assert!(flat.data.is_empty(), "{metric:?}: empty tail after reopen");
            }
            assert_eq!(reopened.len(), 30, "{metric:?}: reopened live count");

            // ADD a TAIL of 10 vectors (appended in `data`, base stays on mmap).
            let oracle = FlatCpuIndex::new(s);
            for (eid, v) in &all {
                oracle.add(eid, v).unwrap();
            }
            for i in 0..10usize {
                let v = mk(&mut rng);
                let eid = format!("t{i}");
                reopened.add(&eid, &v).unwrap();
                oracle.add(&eid, &v).unwrap();
                all.push((eid, v));
            }
            assert_eq!(reopened.len(), 40, "{metric:?}: base 30 + tail 10");

            // DELETE a mix: some BASE rows (on the mmap) and some TAIL rows.
            for eid in ["b3", "b17", "b29", "t0", "t7"] {
                assert!(reopened.remove(eid).unwrap(), "{metric:?}: {eid} was live");
                assert!(oracle.remove(eid).unwrap());
            }
            // Double-remove of a base id is a no-op (already tombstoned).
            assert!(
                !reopened.remove("b3").unwrap(),
                "{metric:?}: double-remove is no-op"
            );
            assert_eq!(reopened.len(), 35, "{metric:?}: 40 - 5 deleted");

            // kNN must be BYTE-IDENTICAL to the in-RAM oracle: same eids, same order,
            // same f32 score bits — across a battery of probes (including exact-match
            // probes that land on base, tail, and deleted rows).
            let mut probes: Vec<Vec<f32>> =
                vec![all[5].1.clone(), all[35].1.clone(), all[3].1.clone()];
            for _ in 0..6 {
                probes.push(mk(&mut rng));
            }
            for (pi, q) in probes.iter().enumerate() {
                for k in [1usize, 5, 12, 40] {
                    let a = reopened.search_knn(q, k).unwrap();
                    let b = oracle.search_knn(q, k).unwrap();
                    let ab: Vec<(String, u32)> =
                        a.iter().map(|(e, s)| (e.clone(), s.to_bits())).collect();
                    let bb: Vec<(String, u32)> =
                        b.iter().map(|(e, s)| (e.clone(), s.to_bits())).collect();
                    assert_eq!(
                        ab, bb,
                        "{metric:?}: probe {pi} k={k} kNN diverged from in-RAM oracle (base-seg + tail + tombstone compose broke)"
                    );
                    // A deleted id must never appear.
                    for (e, _) in &a {
                        assert!(
                            !["b3", "b17", "b29", "t0", "t7"].contains(&e.as_str()),
                            "{metric:?}: deleted id {e} leaked into kNN"
                        );
                    }
                }
            }

            // Snapshot of the reopened (sealed) index must read every LIVE row off
            // the mmap+tail (NOT store) and match the oracle's live set.
            let (mut snap, _) = reopened.dump_for_snapshot().unwrap();
            let (mut osnap, _) = oracle.dump_for_snapshot().unwrap();
            snap.sort_by(|a, b| a.0.cmp(&b.0));
            osnap.sort_by(|a, b| a.0.cmp(&b.0));
            assert_eq!(
                snap, osnap,
                "{metric:?}: snapshot of reopened index must match oracle live set"
            );
        }
    }

    #[test]
    fn remove_drops_vector_from_subsequent_search() {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(17);
        let idx = HnswCpuIndex::new(spec(16, VectorMetric::L2, None));
        for i in 0..20 {
            let v = rand_vec(&mut rng, 16);
            idx.add(&format!("e{i}"), &v).unwrap();
        }
        // The vector at e7 is its own top neighbour; remove it.
        let q_idx = 7;
        let q_eid = format!("e{q_idx}");
        let q = idx.inner.read().unwrap().store.get_decoded(&q_eid).unwrap();
        assert!(idx.remove(&q_eid).unwrap());
        let hits = idx.search_knn(&q, 5).unwrap();
        assert!(
            !hits.iter().any(|(e, _)| e == &q_eid),
            "removed eid {q_eid} still in {hits:?}"
        );
    }
    #[test]
    fn flat_base_publication_retires_both_payload_stores_and_keeps_newer_changes() {
        let spec = VectorSpec {
            dim: 2,
            metric: VectorMetric::L2,
            backend: crate::types::VectorBackend::FlatCpu,
            quantize: None,
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("base.lseg");
        crate::segment::write_vector_segment(
            &path,
            1,
            2,
            &[Some(&[0.0, 0.0]), Some(&[1.0, 1.0]), Some(&[2.0, 2.0])],
        )
        .unwrap();
        let index = FlatCpuIndex::new(spec);
        for (eid, value) in [("ack", 0.0), ("updated", 1.0), ("deleted", 2.0)] {
            index.add(eid, &[value, value]).unwrap();
        }
        index.add("updated", &[3.0, 3.0]).unwrap();
        index.remove("deleted").unwrap();
        index.add("new", &[4.0, 4.0]).unwrap();
        index
            .install_checkpoint_base(
                Arc::new(crate::segment::SegmentReader::open(&path).unwrap()),
                &["ack".into(), "updated".into(), "deleted".into()],
                &HashMap::from([
                    ("ack".into(), true),
                    ("updated".into(), false),
                    ("deleted".into(), false),
                    ("new".into(), false),
                ]),
            )
            .unwrap();
        let inner = index.inner.lock().unwrap();
        assert!(
            !inner.store.raw.contains_key("ack"),
            "acknowledged base vector must leave VectorStore"
        );
        assert_eq!(inner.store.len(), 2);
        assert_eq!(inner.flat.as_ref().unwrap().data.len(), 2);
        drop(inner);
        assert_eq!(index.resident_vector_payload_rows(), 2);
        assert_eq!(index.checkpoint_resident_bytes(), Some(26));
        assert_eq!(
            index.checkpoint_vector("ack").unwrap(),
            Some(vec![0.0, 0.0])
        );
        assert_eq!(
            index.checkpoint_vector("updated").unwrap(),
            Some(vec![3.0, 3.0])
        );
        assert_eq!(index.checkpoint_vector("deleted").unwrap(), None);
        assert_eq!(
            index.checkpoint_vector("new").unwrap(),
            Some(vec![4.0, 4.0])
        );
        index
            .seal_to_segment_prod(&dir.path().join("next.lseg"))
            .unwrap();
        let inner = index.inner.lock().unwrap();
        assert_eq!(
            inner.store.len(),
            0,
            "sealing must release the VectorStore as well as decoded rows"
        );
        assert_eq!(inner.flat.as_ref().unwrap().data.len(), 0);
    }

    #[test]
    fn flat_compacted_base_keeps_absent_rows_deleted_on_open_and_install() {
        let spec = VectorSpec {
            dim: 2,
            metric: VectorMetric::L2,
            backend: crate::types::VectorBackend::FlatCpu,
            quantize: None,
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mapped-base.lseg");
        crate::segment::write_vector_segment(&path, 7, 2, &[None, Some(&[2., 2.]), None]).unwrap();
        let reader = Arc::new(crate::segment::SegmentReader::open(&path).unwrap());
        let ids = vec!["a".to_owned(), "b".to_owned(), "c".to_owned()];
        let cold = FlatCpuIndex::open_from_segment(spec, reader.clone(), ids.clone()).unwrap();
        assert_eq!(
            cold.len(),
            1,
            "absent base rows cannot count as live vectors"
        );
        let oracle = FlatCpuIndex::new(spec);
        oracle.add("b", &[2., 2.]).unwrap();
        let expected = oracle.search_knn(&[0., 0.], 10).unwrap();
        assert_eq!(cold.search_knn(&[0., 0.], 10).unwrap(), expected);
        let live = FlatCpuIndex::new(spec);
        live.add("a", &[9., 9.]).unwrap();
        live.add("b", &[2., 2.]).unwrap();
        live.install_checkpoint_base(
            reader,
            &ids,
            &ids.iter().map(|id| (id.clone(), true)).collect(),
        )
        .unwrap();
        assert_eq!(
            live.len(),
            1,
            "install must apply the compacted base presence column"
        );
        assert_eq!(live.search_knn(&[0., 0.], 10).unwrap(), expected);
    }

    #[test]
    fn flat_base_compaction_preserves_newer_layers_ram_and_deletions() {
        let spec = VectorSpec {
            dim: 2,
            metric: VectorMetric::L2,
            backend: crate::types::VectorBackend::FlatCpu,
            quantize: None,
        };
        let dir = tempfile::tempdir().unwrap();
        let write = |name: &str, rows: &[Option<&[f32]>]| {
            let path = dir.path().join(name);
            crate::segment::write_vector_segment(&path, 1, 2, rows).unwrap();
            Arc::new(crate::segment::SegmentReader::open(&path).unwrap())
        };
        let base = write("base.lseg", &[Some(&[0., 0.]), Some(&[9., 9.])]);
        let first = write("first.lseg", &[Some(&[1., 1.]), Some(&[2., 2.])]);
        let second = write("second.lseg", &[None, Some(&[3., 3.])]);
        let later = write("later.lseg", &[Some(&[4., 4.]), Some(&[5., 5.])]);
        let index =
            FlatCpuIndex::open_from_segment(spec, base.clone(), vec!["a".into(), "base".into()])
                .unwrap();
        index
            .attach_checkpoint_delta(first.clone(), &["a".into(), "b".into()], &[true, true])
            .unwrap();
        index
            .attach_checkpoint_delta(second.clone(), &["a".into(), "c".into()], &[true, true])
            .unwrap();
        index
            .attach_checkpoint_delta(later.clone(), &["c".into(), "d".into()], &[true, true])
            .unwrap();
        index.add("b", &[6., 6.]).unwrap();
        index.remove("d").unwrap();
        let before = index.search_knn(&[0., 0.], 10).unwrap();
        let compacted = write(
            "compacted.lseg",
            &[None, Some(&[2., 2.]), Some(&[9., 9.]), Some(&[3., 3.])],
        );
        let ids = ["a".into(), "b".into(), "base".into(), "c".into()];
        index
            .replace_checkpoint_base(
                &base,
                &[first.clone(), second.clone()],
                compacted.clone(),
                &ids,
            )
            .unwrap();
        assert_eq!(index.search_knn(&[0., 0.], 10).unwrap(), before);
        assert_eq!(index.resident_vector_payload_rows(), 1);
        assert_eq!(index.checkpoint_vector("a").unwrap(), None);
        assert_eq!(index.checkpoint_vector("d").unwrap(), None);
        assert_eq!(index.checkpoint_vector("b").unwrap(), Some(vec![6., 6.]));
        assert_eq!(index.checkpoint_vector("c").unwrap(), Some(vec![4., 4.]));
        assert_eq!(Arc::strong_count(&base), 1, "old base must be released");
        assert_eq!(
            Arc::strong_count(&first),
            1,
            "first base input must be released"
        );
        assert_eq!(
            Arc::strong_count(&second),
            1,
            "second base input must be released"
        );
        let remaining = index.checkpoint_delta_readers();
        assert_eq!(remaining.len(), 1);
        assert!(Arc::ptr_eq(&remaining[0], &later));
        assert!(Arc::ptr_eq(
            &index.checkpoint_base_reader().unwrap(),
            &compacted
        ));
        assert!(index
            .replace_checkpoint_base(&base, &[first, second], compacted, &ids)
            .is_err());
        assert_eq!(index.search_knn(&[0., 0.], 10).unwrap(), before);
    }

    #[test]
    fn flat_compaction_retargets_only_selected_layers_and_releases_input_readers() {
        let spec = VectorSpec {
            dim: 2,
            metric: VectorMetric::L2,
            backend: crate::types::VectorBackend::FlatCpu,
            quantize: None,
        };
        let dir = tempfile::tempdir().unwrap();
        let write = |name: &str, values: &[Option<&[f32]>]| {
            let path = dir.path().join(name);
            crate::segment::write_vector_segment(&path, 1, 2, values).unwrap();
            Arc::new(crate::segment::SegmentReader::open(&path).unwrap())
        };
        let index = FlatCpuIndex::open_from_segment(
            spec,
            write("base.lseg", &[Some(&[0., 0.]), Some(&[9., 9.])]),
            vec!["a".into(), "base".into()],
        )
        .unwrap();
        let first = write("first.lseg", &[Some(&[1., 1.]), Some(&[2., 2.])]);
        let second = write("second.lseg", &[None, Some(&[3., 3.])]);
        let third = write("third.lseg", &[Some(&[4., 4.]), Some(&[5., 5.])]);
        index
            .attach_checkpoint_delta(first.clone(), &["a".into(), "b".into()], &[true, true])
            .unwrap();
        index
            .attach_checkpoint_delta(second.clone(), &["a".into(), "c".into()], &[true, true])
            .unwrap();
        index
            .attach_checkpoint_delta(third.clone(), &["c".into(), "d".into()], &[true, true])
            .unwrap();
        index.add("b", &[6., 6.]).unwrap();
        index.remove("d").unwrap();
        let before = index.search_knn(&[0., 0.], 10).unwrap();
        let compacted = write("compacted.lseg", &[None, Some(&[2., 2.]), Some(&[3., 3.])]);
        index
            .replace_checkpoint_deltas(
                &[first.clone(), second.clone()],
                compacted.clone(),
                &["a".into(), "b".into(), "c".into()],
            )
            .expect("replace the exact selected vector delta range");
        assert_eq!(index.search_knn(&[0., 0.], 10).unwrap(), before);
        assert_eq!(
            index.resident_vector_payload_rows(),
            1,
            "newer RAM update survives compaction"
        );
        assert_eq!(
            Arc::strong_count(&first),
            1,
            "first input must be released by the live index"
        );
        assert_eq!(
            Arc::strong_count(&second),
            1,
            "second input must be released by the live index"
        );
        let readers = index.checkpoint_delta_readers();
        assert_eq!(readers.len(), 2);
        assert!(Arc::ptr_eq(&readers[0], &compacted));
        assert!(Arc::ptr_eq(&readers[1], &third));
        assert!(
            index
                .replace_checkpoint_deltas(
                    &[first, second],
                    compacted,
                    &["a".into(), "b".into(), "c".into()],
                )
                .is_err(),
            "stale input identities must not match the current layers"
        );
        assert_eq!(index.search_knn(&[0., 0.], 10).unwrap(), before);
    }

    #[test]
    fn flat_mmap_deltas_release_acknowledged_payload_and_preserve_newer_ram() {
        let spec = VectorSpec {
            dim: 2,
            metric: VectorMetric::L2,
            backend: crate::types::VectorBackend::FlatCpu,
            quantize: None,
        };
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("base.lseg");
        crate::segment::write_vector_segment(&base, 1, 2, &[Some(&[0.0, 0.0]), Some(&[4.0, 4.0])])
            .unwrap();
        let index = FlatCpuIndex::open_from_segment(
            spec,
            Arc::new(crate::segment::SegmentReader::open(&base).unwrap()),
            vec!["a".into(), "b".into()],
        )
        .unwrap();
        let oracle = FlatCpuIndex::new(spec);
        oracle.add("a", &[0.0, 0.0]).unwrap();
        oracle.add("b", &[4.0, 4.0]).unwrap();
        let first = dir.path().join("one.lseg");
        crate::segment::write_vector_segment(&first, 2, 2, &[Some(&[1.0, 1.0]), Some(&[9.0, 9.0])])
            .unwrap();
        index
            .attach_checkpoint_delta(
                Arc::new(crate::segment::SegmentReader::open(&first).unwrap()),
                &["a".into(), "c".into()],
                &[true, true],
            )
            .unwrap();
        oracle.add("a", &[1.0, 1.0]).unwrap();
        oracle.add("c", &[9.0, 9.0]).unwrap();
        assert_eq!(index.resident_vector_payload_rows(), 0);
        assert_eq!(index.checkpoint_resident_bytes(), Some(0));
        let second = dir.path().join("two.lseg");
        crate::segment::write_vector_segment(&second, 3, 2, &[Some(&[2.0, 2.0]), None]).unwrap();
        index
            .attach_checkpoint_delta(
                Arc::new(crate::segment::SegmentReader::open(&second).unwrap()),
                &["a".into(), "b".into()],
                &[true, true],
            )
            .unwrap();
        oracle.add("a", &[2.0, 2.0]).unwrap();
        oracle.remove("b").unwrap();
        index.add("a", &[3.0, 3.0]).unwrap();
        oracle.add("a", &[3.0, 3.0]).unwrap();
        assert_eq!(index.resident_vector_payload_rows(), 1);
        assert_eq!(index.checkpoint_resident_bytes(), Some(9));
        for query in [[0.0, 0.0], [3.0, 3.0], [10.0, 10.0]] {
            assert_eq!(
                index.search_knn(&query, 8).unwrap(),
                oracle.search_knn(&query, 8).unwrap()
            );
        }
        assert!(index
            .search_knn(&[4.0, 4.0], 8)
            .unwrap()
            .iter()
            .all(|(eid, _)| eid != "b"));
    }
}
// CODEGEN-END

impl HnswCpuIndex {
    fn seal_checkpoint(
        &self,
        path: &std::path::Path,
        sequence: Option<u64>,
    ) -> Result<Option<Vec<String>>> {
        let inner = self
            .inner
            .read()
            .map_err(|_| anyhow!("hnsw lock poisoned"))?;
        let dim = inner.store.spec.dim as usize;
        #[cfg(test)]
        HNSW_CHECKPOINT_FULL_SCANS.with(|count| count.set(count.get() + 1));
        let rows: Vec<(String, Vec<f32>)> = inner.store.iter_decoded().collect();
        drop(inner);
        let n = rows.len();
        // Split the pairs into the two shapes the writer and the caller each
        // need, without cloning the eids: the vectors are borrowed for the
        // duration of the write and the Strings are moved straight out.
        let mut row_eids: Vec<String> = Vec::with_capacity(n);
        let mut row_vecs: Vec<Vec<f32>> = Vec::with_capacity(n);
        for (eid, v) in rows {
            row_eids.push(eid);
            row_vecs.push(v);
        }
        let vectors: Vec<Option<&[f32]>> = row_vecs.iter().map(|v| Some(v.as_slice())).collect();
        crate::segment::write_vector_segment(path, sequence.unwrap_or(n as u64), dim, &vectors)?;
        // Reopen what was just written, in every build. Once the caller commits
        // this checkpoint it drops the in-RAM store, so these bytes become the
        // only copy of the vectors; a segment that cannot be read back has to
        // fail HERE, while the RAM copy is still there to retry from, rather
        // than at the next restart with nothing left to recover. The reopen is
        // one header read against a file the page cache still holds — it is not
        // the cost that would justify compiling it out.
        let reader = crate::segment::SegmentReader::open(path).map_err(|e| {
            anyhow!(
                "vector segment written to {} could not be read back: {e}",
                path.display()
            )
        })?;
        if reader.n_docs() as usize != n {
            bail!(
                "vector segment written to {} reopened with {} rows, expected {n}",
                path.display(),
                reader.n_docs()
            );
        }
        Ok(Some(row_eids))
    }
}
