---
id: projects-lumen-src-vector_index-rs
capability_refs:
  - id: "search"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/vector_index.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/vector_index.rs` captured as a per-file rust-source-unit (td_ast) during lumen standardization onto the per-file codegen ladder.

### Symbols

| Name | Target | Kind | Visibility |
|------|--------|------|------------|
| `VectorIndex` | projects/lumen/src/vector_index.rs | trait | pub |
| `ScalarCodebook` | projects/lumen/src/vector_index.rs | struct | pub |
| `empty` | projects/lumen/src/vector_index.rs | function | pub |
| `widen` | projects/lumen/src/vector_index.rs | function | pub |
| `encode_sq` | projects/lumen/src/vector_index.rs | function | pub |
| `decode_sq` | projects/lumen/src/vector_index.rs | function | pub |
| `HnswCpuIndex` | projects/lumen/src/vector_index.rs | struct | pub |
| `new` | projects/lumen/src/vector_index.rs | function | pub |
| `restore` | projects/lumen/src/vector_index.rs | function | pub |
| `set_ef_search` | projects/lumen/src/vector_index.rs | function | pub |
| `FlatCpuIndex` | projects/lumen/src/vector_index.rs | struct | pub |
| `new` | projects/lumen/src/vector_index.rs | function | pub |
| `restore` | projects/lumen/src/vector_index.rs | function | pub |
| `open_from_segment` | projects/lumen/src/vector_index.rs | function | pub |
| `open_backend` | projects/lumen/src/vector_index.rs | function | pub |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
use std::sync::{Mutex, RwLock};

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

use crate::types::{VectorMetric, VectorQuantize, VectorSpec};

// ---------------------------------------------------------------------------
// VectorIndex trait
// ---------------------------------------------------------------------------

/// Common backend contract — every concrete index implementation
/// (HNSW, flat CPU brute force) goes through this trait so the storage
/// layer doesn't care which one is in use.
pub trait VectorIndex: Send + Sync {
    /// Insert (or overwrite) the vector associated with `external_id`.
    fn add(&self, external_id: &str, vector: &[f32]) -> Result<()>;

    /// Remove the vector for `external_id`. No-op if it isn't present.
    /// Returns `true` if a vector was removed.
    fn remove(&self, external_id: &str) -> Result<bool>;

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

    /// PRODUCTION seal (Stage 2 Phase 2f-1): seal the exact-CPU flat corpus into
    /// a columnar mmap vector segment at `path`, attach it, and DROP the in-RAM
    /// `data` buffer so the field's vectors leave RAM (the scan reads each row
    /// zero-copy off the mmap). Returns `Ok(Some(row_eids))` — the eid of each
    /// sealed vector row in segment-row order — for a [`FlatCpuIndex`], so the
    /// caller can persist that row→eid mapping (the vector segment stores only
    /// f32 rows) and rebuild the index on reopen. `Ok(None)` for any other
    /// backend (HNSW is out of scope) — the default trait impl no-ops, so a
    /// non-`FlatCpuIndex` backend simply returns `Ok(None)`.
    fn seal_to_segment_prod(&self, _path: &std::path::Path) -> Result<Option<Vec<String>>> {
        Ok(None)
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
        }
    }

    fn search(&self, vec: &[f32], k: usize, ef: usize) -> Vec<(usize, f32)> {
        let ef = k.max(ef);
        let raw = match self {
            HnswBackend::L2(h) => h.search(vec, k, ef),
            HnswBackend::Cosine(h) => {
                let q = normalize_unit_safe(vec);
                h.search(&q, k, ef)
            }
            HnswBackend::Dot(h) => h.search(vec, k, ef),
        };
        raw.into_iter().map(|n| (n.d_id, n.distance)).collect()
    }
}

impl HnswCpuIndex {
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
            let mut inner = idx.inner.write().map_err(|_| anyhow!("hnsw lock poisoned"))?;
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

    /// Override the search-time `ef` (beam width). Used by tuning/bench
    /// harnesses to sweep recall/latency on an already-built graph; production
    /// uses the [`hnsw_search_ef`] default set at construction.
    pub fn set_ef_search(&self, ef: usize) {
        if let Ok(mut inner) = self.inner.write() {
            inner.ef_search = ef.max(1);
        }
    }
}

impl VectorIndex for HnswCpuIndex {
    fn add(&self, external_id: &str, vector: &[f32]) -> Result<()> {
        let mut inner = self.inner.write().map_err(|_| anyhow!("hnsw lock poisoned"))?;
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
        let v_for_graph: Vec<f32> = if inner.store.codebook.is_some() {
            inner
                .store
                .get_decoded(external_id)
                .ok_or_else(|| anyhow!("just-inserted vector vanished"))?
        } else {
            vector.to_vec()
        };
        inner.hnsw.insert(&v_for_graph, id);
        if let Some(old_id) = inner.eid_to_id.insert(external_id.to_string(), id) {
            inner.id_to_eid.remove(&old_id);
        }
        inner.id_to_eid.insert(id, external_id.to_string());
        Ok(())
    }

    fn remove(&self, external_id: &str) -> Result<bool> {
        let mut inner = self.inner.write().map_err(|_| anyhow!("hnsw lock poisoned"))?;
        let removed = inner.store.drop(external_id);
        if let Some(id) = inner.eid_to_id.remove(external_id) {
            inner.id_to_eid.remove(&id);
        }
        Ok(removed)
    }

    fn search_knn_filtered(
        &self,
        query: &[f32],
        k: usize,
        allow: &dyn Fn(&str) -> bool,
    ) -> Result<Vec<(String, f32)>> {
        let inner = self.inner.read().map_err(|_| anyhow!("hnsw lock poisoned"))?;
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
        // hnsw_rs exposes no mid-traversal filter hook, so we over-fetch
        // a candidate pool and keep only allowed (and non-orphaned) ids,
        // doubling the pool until we have `k` allowed hits or have
        // scanned the whole index. The base over-fetch (`k*4 + k`) also
        // absorbs stale ids left by replaces. For an unfiltered query
        // the first pass already yields `k`, so the widening loop never
        // iterates — the hot path is unchanged.
        let mut pool = (k * 4 + k).min(n);
        let ef = inner.ef_search;
        loop {
            let raw = inner.hnsw.search(query, pool, ef);
            let mut out = Vec::with_capacity(k);
            for (id, dist) in raw {
                let Some(eid) = inner.id_to_eid.get(&id) else {
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
            // Enough allowed hits, or the pool already covers the whole
            // index (a wider search cannot surface more) → done.
            if out.len() == k || pool >= n {
                return Ok(out);
            }
            pool = (pool * 2).min(n);
        }
    }

    fn len(&self) -> usize {
        self.inner
            .read()
            .map(|i| i.store.len())
            .unwrap_or(0)
    }

    fn dump_for_snapshot(&self) -> Result<(Vec<(String, Vec<f32>)>, Option<ScalarCodebook>)> {
        let inner = self.inner.read().map_err(|_| anyhow!("hnsw lock poisoned"))?;
        let vectors: Vec<(String, Vec<f32>)> = inner.store.iter_decoded().collect();
        Ok((vectors, inner.store.codebook))
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

struct FlatVecs {
    /// Stage 2 Phase 2k-1: with no segment attached this is the full
    /// contiguous `N*dim` corpus (every row). With a segment
    /// attached it holds ONLY the live TAIL — rows `[n_base..total)` — appended
    /// after the seal; the base rows `[0..n_base)` live on the mmap (`seg`).
    /// Empty when there is no tail.
    data: Vec<f32>,
    eids: Vec<String>,
    dim: usize,
    /// Stage 2 disk-tier (Phase 2d): when present, the flat kNN scan reads each
    /// BASE row's `dim` f32s ZERO-COPY off this mmap'd segment instead of `data`.
    /// Base row `i` (`i < n_base`, paired with `eids[i]`) is
    /// `seg.vector_at(i, dim)`. DEFAULTS to `None`; while it is `None` (nothing
    /// sealed) the scan is byte-for-byte the
    /// in-RAM `&data[i*dim..]` read.
    seg: Option<std::sync::Arc<crate::segment::SegmentReader>>,
    /// Stage 2 Phase 2k-1: number of BASE rows served from `seg` (the mmap).
    /// Rows `[0..n_base)` are demand-paged off the segment; rows `[n_base..)`
    /// are the live tail in `data` (`data[(i-n_base)*dim..]`). `0` when no
    /// segment is attached (the freshly-built in-RAM buffer is all "tail").
    n_base: usize,
    /// Stage 2 Phase 2k-1: tombstoned ROW indices (over `0..eids.len()`). A
    /// removed row is skipped by the scan and excluded from `len`, mirroring the
    /// in-RAM remove's effect on subsequent searches WITHOUT mutating the
    /// immutable base segment. Empty when nothing was removed post-seal.
    tomb: roaring::RoaringBitmap,
    /// Stage 2 Phase 2k-1: eid → row index, for O(1) `remove` (tombstone the row)
    /// and tail de-dup (an overwrite of an existing eid). This is identity-level
    /// state (`String`+`u32` per row), NOT the O(N*dim) vector payload — the
    /// vectors stay on the mmap. Built when a segment is attached.
    eid_to_row: HashMap<String, u32>,
}

impl FlatVecs {
    /// Row `i`'s `dim`-long vector slice. BASE rows (`i < n_base`) come off the
    /// segment mmap (zero-copy); TAIL rows come from the in-RAM `data` buffer at
    /// `data[(i-n_base)*dim..]`. With no segment attached `n_base == 0` so every
    /// row reads `data` — byte-for-byte the old in-RAM path. The segment is built
    /// from the exact base rows, so a base hit is bit-identical to what `data`
    /// held; if a segment read ever misses (torn column) we have no in-RAM copy of
    /// the base, so we fall through and panic-index `data` — that index is
    /// out-of-range, surfacing the torn read loudly rather than returning garbage.
    #[inline]
    fn row(&self, i: usize) -> &[f32] {
        if i < self.n_base {
            if let Some(seg) = &self.seg {
                if let Some(v) = seg.vector_at(i as u32, self.dim) {
                    return v;
                }
            }
            // No in-RAM fallback exists for a base row (the payload is on the
            // mmap); a miss here is a torn segment and must surface.
        }
        let local = i - self.n_base;
        &self.data[local * self.dim..(local + 1) * self.dim]
    }
}

impl FlatCpuIndex {
    pub fn new(spec: VectorSpec) -> Self {
        Self { inner: Mutex::new(FlatInner { store: VectorStore::new(spec), flat: None }) }
    }

    /// Restore from a snapshot (re-store the saved vectors).
    pub fn restore(
        spec: VectorSpec,
        vectors: Vec<(String, Vec<f32>)>,
        codebook: Option<ScalarCodebook>,
    ) -> Result<Self> {
        let idx = Self::new(spec);
        {
            let mut inner = idx.inner.lock().map_err(|_| anyhow!("flat lock poisoned"))?;
            if codebook.is_some() {
                inner.store.codebook = codebook;
            }
        }
        for (eid, v) in vectors {
            idx.add(&eid, &v)?;
        }
        Ok(idx)
    }

    /// Ensure the cached scan buffer exists.
    ///
    /// FIRST-TIME build (`flat == None`): materialize every stored vector into a
    /// contiguous in-RAM `data` buffer (`n_base == 0`, no segment) — byte-for-byte
    /// the original behavior, and the only path taken while no segment is attached.
    ///
    /// Phase 2k-1: when a segment is ALREADY attached, the base rows live ONLY on
    /// the mmap and are NOT in `store`, so this MUST NOT rebuild from `store`
    /// (that would either lose the base or — pre-fix — re-materialize it). The
    /// mutation paths (`add`/`remove`) therefore keep the segment-backed `flat`
    /// LIVE in place instead of invalidating it, so a segment-backed index never
    /// re-enters this fn with `flat == None`. The `debug_assert` documents that.
    fn ensure_flat(inner: &mut FlatInner) {
        if inner.flat.is_some() {
            return;
        }
        let dim = inner.store.spec.dim as usize;
        let mut data = Vec::with_capacity(inner.store.len() * dim);
        let mut eids = Vec::with_capacity(inner.store.len());
        for (eid, v) in inner.store.iter_decoded() {
            data.extend_from_slice(&v);
            eids.push(eid);
        }
        inner.flat = Some(FlatVecs {
            data,
            eids,
            dim,
            // A freshly built flat buffer is wholly in-RAM (all "tail", no base):
            // sealing/reopen is what attaches a segment and sets `n_base`.
            seg: None,
            n_base: 0,
            tomb: roaring::RoaringBitmap::new(),
            eid_to_row: HashMap::new(),
        });
    }

    /// Live vector count (Phase 2k-1). When a segment is attached the base rows
    /// live on the mmap (NOT in `store`), so the count is `total rows − tombstoned`
    /// off the composed `flat`. Otherwise it's `store.len()` — the original
    /// in-RAM count, byte-for-byte unchanged while no segment is attached.
    #[inline]
    fn live_len(inner: &FlatInner) -> usize {
        if let Some(flat) = inner.flat.as_ref() {
            if flat.seg.is_some() {
                return flat.eids.len() - flat.tomb.len() as usize;
            }
        }
        inner.store.len()
    }

    /// Whether row `i` of `flat` is tombstoned. `false` whenever no row has been
    /// deleted after a seal (the tombstone bitmap is empty) so the scan is unchanged.
    #[inline]
    fn row_tombstoned(flat: &FlatVecs, i: usize) -> bool {
        flat.tomb.contains(i as u32)
    }

    /// PRODUCTION seal (Phase 2f-1): build/reuse the cached flat buffer, write a
    /// vector segment, attach it, and DROP the in-RAM `data`. Returns the eid of
    /// each sealed row in segment-row order so the caller can persist the
    /// row→eid mapping. Mirrors `__seal_flat_to_segment` but is reachable from
    /// production code and surfaces the row eids.
    fn seal_to_segment_prod(&self, path: &std::path::Path) -> Result<Option<Vec<String>>> {
        let mut inner = self.inner.lock().map_err(|_| anyhow!("flat lock poisoned"))?;
        Self::ensure_flat(&mut inner);
        let flat = inner.flat.as_mut().expect("flat built");
        let dim = flat.dim;
        let total = flat.eids.len();
        // RE-SEAL-CAPABLE, TOMBSTONE-AWARE gather (Phase 2f-2 + 2k-1): read each
        // LIVE row through `flat.row(i)`, NOT raw `&flat.data[..]`. With the
        // composed model a row is either a BASE row on the prior segment mmap
        // (`i < n_base`) or a live TAIL row in `data` (`i >= n_base`); `row(i)`
        // serves the right one. Tombstoned rows (deleted post-seal) are SKIPPED so
        // the new segment bakes the deletes in (it never resurrects them) and the
        // reopened index starts with an empty tombstone set. Copy each live row
        // into an owned buffer so the new segment write does not alias the old
        // mmap. A first-time seal has `n_base == 0`, an empty `tomb`, and `data`
        // holding every row → `row(i)` reads `data` for all `i`, byte-identical to
        // before.
        let mut owned: Vec<Vec<f32>> = Vec::with_capacity(total);
        let mut row_eids: Vec<String> = Vec::with_capacity(total);
        for i in 0..total {
            if flat.tomb.contains(i as u32) {
                continue;
            }
            owned.push(flat.row(i).to_vec());
            row_eids.push(flat.eids[i].clone());
        }
        let n = owned.len();
        let vectors: Vec<Option<&[f32]>> =
            owned.iter().map(|v| Some(v.as_slice())).collect();
        crate::segment::write_vector_segment(path, n as u64, dim, &vectors)?;
        let reader = crate::segment::SegmentReader::open(path)?;
        debug_assert_eq!(reader.n_docs() as usize, n);
        // Every sealed row is now a BASE row on the fresh mmap; the tail is empty
        // and tombstones are cleared (the deletes were baked into the new segment).
        let mut eid_to_row = HashMap::with_capacity(n);
        for (row, eid) in row_eids.iter().enumerate() {
            eid_to_row.insert(eid.clone(), row as u32);
        }
        flat.seg = Some(std::sync::Arc::new(reader)); // drops any prior segment Arc
        flat.data = Vec::new(); // payload now on the mmap, drop the RAM copy
        flat.eids = row_eids.clone();
        flat.n_base = n;
        flat.tomb = roaring::RoaringBitmap::new();
        flat.eid_to_row = eid_to_row;
        Ok(Some(row_eids))
    }

    /// Reopen a flat-cpu index DIRECTLY from a sealed vector segment plus its
    /// row→eid mapping (Phase 2f-1), with NO snapshot. Row `i`'s vector is
    /// `seg.vector_at(i, dim)` (demand-paged off the mmap) and its external_id is
    /// `row_eids[i]`. The reconstructed `FlatVecs` keeps the segment attached and
    /// leaves `data` empty, so the kNN scan still reads zero-copy off the page —
    /// the vectors never re-enter RAM. Used by `Collection::open_from_segments`.
    pub fn open_from_segment(
        spec: VectorSpec,
        seg: std::sync::Arc<crate::segment::SegmentReader>,
        row_eids: Vec<String>,
    ) -> Result<Self> {
        let dim = spec.dim as usize;
        let n = row_eids.len();
        // Phase 2k-1: the base vectors live ONLY on the mmap. We do NOT `store.put`
        // them — that O(N*dim) re-materialization is exactly the gap this phase
        // closes (it made a reopened vector collection NOT bound RAM; the 2i scale
        // proof showed the reopen delta growing ~1.5x when dim doubled). The store
        // holds NO base vectors (only spec/codebook, and tail vectors once added);
        // the kNN scan, snapshot, remove, and len all read the base off `seg`.
        let idx = Self::new(spec);
        {
            let mut inner = idx.inner.lock().map_err(|_| anyhow!("flat lock poisoned"))?;
            // Identity-level eid → base-row map (O(N) Strings + u32s, NOT vectors)
            // for remove + tail de-dup. The actual `dim`-wide rows stay on the mmap.
            let mut eid_to_row = HashMap::with_capacity(n);
            for (i, eid) in row_eids.iter().enumerate() {
                eid_to_row.insert(eid.clone(), i as u32);
            }
            // Attach the segment as the BASE (`n_base == n`), the exact row order it
            // was sealed with, an EMPTY tail (`data`), and an empty tombstone set.
            // `row(i)` serves rows `[0..n)` from the mmap.
            inner.flat = Some(FlatVecs {
                data: Vec::new(),
                eids: row_eids,
                dim,
                seg: Some(seg),
                n_base: n,
                tomb: roaring::RoaringBitmap::new(),
                eid_to_row,
            });
        }
        debug_assert_eq!(idx.len(), n);
        Ok(idx)
    }

    /// top-k of (idx, score) by score-desc, returned as (eid, score).
    fn topk(mut cand: Vec<(usize, f32)>, k: usize, eids: &[String]) -> Vec<(String, f32)> {
        let want = k.min(cand.len());
        if want > 0 && want < cand.len() {
            cand.select_nth_unstable_by(want - 1, |a, b| {
                b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
            });
            cand.truncate(want);
        }
        cand.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        cand.into_iter().map(|(i, s)| (eids[i].clone(), s)).collect()
    }
}

impl VectorIndex for FlatCpuIndex {
    fn add(&self, external_id: &str, vector: &[f32]) -> Result<()> {
        let mut inner = self.inner.lock().map_err(|_| anyhow!("flat lock poisoned"))?;
        // Phase 2k-1: when a SEGMENT is attached the base rows live ONLY on the
        // mmap (not in `store`), so we CANNOT invalidate `flat` and rebuild from
        // `store` — that would drop the base. Instead append the new vector as a
        // live TAIL row in place, keeping the segment. An overwrite of an existing
        // eid (base or tail) tombstones the old row first, mirroring the in-RAM
        // overwrite (the latest value wins). The vector also goes into `store` so
        // a later snapshot/codebook stays consistent for the TAIL.
        if inner.flat.as_ref().is_some_and(|f| f.seg.is_some()) {
            let dim = inner.store.spec.dim as usize;
            if vector.len() != dim {
                bail!("vector dim mismatch: expected {}, got {}", dim, vector.len());
            }
            // Decode-normalize through the store so SQ tail rows round-trip like
            // the base did, then read the (possibly re-quantized) f32 view back.
            inner.store.put(external_id, vector)?;
            let v_dec = inner
                .store
                .get_decoded(external_id)
                .unwrap_or_else(|| vector.to_vec());
            let flat = inner.flat.as_mut().expect("segment-backed flat present");
            // Overwrite: tombstone the prior row for this eid (it may be a base row
            // on the mmap or an earlier tail row), then append the fresh row.
            if let Some(&old_row) = flat.eid_to_row.get(external_id) {
                flat.tomb.insert(old_row);
            }
            let new_row = flat.eids.len() as u32;
            flat.data.extend_from_slice(&v_dec);
            flat.eids.push(external_id.to_string());
            flat.eid_to_row.insert(external_id.to_string(), new_row);
            // The freshly-appended row is live even if a same-eid base row was
            // tombstoned above.
            flat.tomb.remove(new_row);
            return Ok(());
        }
        inner.store.put(external_id, vector)?;
        inner.flat = None; // invalidate the cached scan buffer (in-RAM path)
        Ok(())
    }

    fn remove(&self, external_id: &str) -> Result<bool> {
        let mut inner = self.inner.lock().map_err(|_| anyhow!("flat lock poisoned"))?;
        // Phase 2k-1: when a SEGMENT is attached, TOMBSTONE the row (look up via
        // the eid → row map) so the scan skips it and `len` excludes it — WITHOUT
        // mutating the immutable base segment or rebuilding from `store`. This is
        // the same observable effect as the in-RAM remove on subsequent searches.
        // Also drop any tail copy from `store` so a snapshot won't re-emit it.
        if inner.flat.as_ref().is_some_and(|f| f.seg.is_some()) {
            let _ = inner.store.drop(external_id);
            let flat = inner.flat.as_mut().expect("segment-backed flat present");
            return match flat.eid_to_row.remove(external_id) {
                Some(row) if !flat.tomb.contains(row) => {
                    flat.tomb.insert(row);
                    Ok(true)
                }
                // Already tombstoned (or never present) → no live row removed.
                _ => Ok(false),
            };
        }
        let removed = inner.store.drop(external_id);
        inner.flat = None;
        Ok(removed)
    }

    fn search_knn_filtered(
        &self,
        query: &[f32],
        k: usize,
        allow: &dyn Fn(&str) -> bool,
    ) -> Result<Vec<(String, f32)>> {
        use rayon::prelude::*;
        let mut inner = self.inner.lock().map_err(|_| anyhow!("flat lock poisoned"))?;
        let dim = inner.store.spec.dim as usize;
        if query.len() != dim {
            bail!("kNN query dim mismatch: expected {}, got {}", dim, query.len());
        }
        if Self::live_len(&inner) == 0 || k == 0 {
            return Ok(Vec::new());
        }
        let metric = inner.store.spec.metric;
        Self::ensure_flat(&mut inner);
        let flat = inner.flat.as_ref().expect("flat built");
        let n = flat.eids.len();
        // Parallel distance over every row (the expensive part). The filter is
        // applied sequentially after, so `allow` need not be Send/Sync. Phase
        // 2k-1: tombstoned rows are skipped so a deleted base/tail row never enters
        // the candidate set. With no segment `tomb` is empty → the full scan as
        // before. `row(i)` reads the base off the mmap, the tail off `data`.
        let scores: Vec<f32> = (0..n)
            .into_par_iter()
            .map(|i| -distance(metric, query, flat.row(i)))
            .collect();
        let cand: Vec<(usize, f32)> = (0..n)
            .filter(|&i| !Self::row_tombstoned(flat, i))
            .filter(|&i| allow(&flat.eids[i]))
            .map(|i| (i, scores[i]))
            .collect();
        Ok(Self::topk(cand, k, &flat.eids))
    }

    fn search_knn_batch(&self, queries: &[Vec<f32>], k: usize) -> Result<Vec<Vec<(String, f32)>>> {
        use rayon::prelude::*;
        let mut inner = self.inner.lock().map_err(|_| anyhow!("flat lock poisoned"))?;
        let dim = inner.store.spec.dim as usize;
        for q in queries {
            if q.len() != dim {
                bail!("kNN query dim mismatch: expected {}, got {}", dim, q.len());
            }
        }
        if Self::live_len(&inner) == 0 || k == 0 {
            return Ok(queries.iter().map(|_| Vec::new()).collect());
        }
        let metric = inner.store.spec.metric;
        Self::ensure_flat(&mut inner);
        let flat = inner.flat.as_ref().expect("flat built");
        let n = flat.eids.len();
        // Parallel across queries; each query scans the shared flat buffer.
        // Tombstoned rows (Phase 2k-1) are skipped so a deleted vector never lands
        // in any query's candidate set.
        Ok(queries
            .par_iter()
            .map(|q| {
                let cand: Vec<(usize, f32)> = (0..n)
                    .filter(|&i| !Self::row_tombstoned(flat, i))
                    .map(|i| (i, -distance(metric, q, flat.row(i))))
                    .collect();
                Self::topk(cand, k, &flat.eids)
            })
            .collect())
    }

    fn len(&self) -> usize {
        self.inner.lock().map(|i| Self::live_len(&i)).unwrap_or(0)
    }

    fn dump_for_snapshot(&self) -> Result<(Vec<(String, Vec<f32>)>, Option<ScalarCodebook>)> {
        let inner = self.inner.lock().map_err(|_| anyhow!("flat lock poisoned"))?;
        // Phase 2k-1: when a segment is attached the base vectors are NOT in
        // `store` (they live on the mmap), so a CBOR snapshot must read every LIVE
        // (non-tombstoned) row through the composed `flat` — base off the mmap,
        // tail off `data` — instead of `store.iter_decoded()`. With no segment the
        // store still holds everything; fall through to the original path.
        if let Some(flat) = inner.flat.as_ref() {
            if flat.seg.is_some() {
                let mut vectors: Vec<(String, Vec<f32>)> = Vec::with_capacity(flat.eids.len());
                for i in 0..flat.eids.len() {
                    if flat.tomb.contains(i as u32) {
                        continue;
                    }
                    vectors.push((flat.eids[i].clone(), flat.row(i).to_vec()));
                }
                return Ok((vectors, inner.store.codebook.clone()));
            }
        }
        let vectors: Vec<(String, Vec<f32>)> = inner.store.iter_decoded().collect();
        Ok((vectors, inner.store.codebook.clone()))
    }

    fn seal_to_segment_prod(&self, path: &std::path::Path) -> Result<Option<Vec<String>>> {
        FlatCpuIndex::seal_to_segment_prod(self, path)
    }

    #[cfg(test)]
    fn __seal_flat_to_segment(&self, path: &std::path::Path) -> Result<Option<u32>> {
        let mut inner = self.inner.lock().map_err(|_| anyhow!("flat lock poisoned"))?;
        // Build (or reuse) the cached flat buffer; this fixes the eid↔row order
        // for the lifetime of the cache, so the sealed segment's row order and
        // the live `eids` agree exactly.
        Self::ensure_flat(&mut inner);
        let flat = inner.flat.as_mut().expect("flat built");
        let dim = flat.dim;
        let n = flat.eids.len();
        // Dense, decoded f32[n*dim] in the SAME row order as `eids` — every row
        // is present (the flat buffer holds only stored vectors).
        let vectors: Vec<Option<&[f32]>> =
            (0..n).map(|i| Some(&flat.data[i * dim..(i + 1) * dim])).collect();
        crate::segment::write_vector_segment(path, n as u64, dim, &vectors)?;
        let reader = crate::segment::SegmentReader::open(path)?;
        debug_assert_eq!(reader.n_docs() as usize, n);
        // Attach the segment and drop the in-RAM data so the scan provably reads
        // the mmap; with `n_base = n` every row is a BASE row served from
        // `seg.vector_at(i, dim)`. Phase 2k-1: also populate `eid_to_row` (so a
        // post-seal remove/overwrite can find its row) and leave `tomb` empty.
        let mut eid_to_row = HashMap::with_capacity(n);
        for (row, eid) in flat.eids.iter().enumerate() {
            eid_to_row.insert(eid.clone(), row as u32);
        }
        flat.seg = Some(std::sync::Arc::new(reader));
        flat.data = Vec::new();
        flat.n_base = n;
        flat.tomb = roaring::RoaringBitmap::new();
        flat.eid_to_row = eid_to_row;
        Ok(Some(n as u32))
    }
}

impl std::fmt::Debug for FlatCpuIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FlatCpuIndex").field("len", &self.len()).finish()
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
        assert!(d_high < d_low, "higher dot product must be closer (smaller distance)");
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
            assert_eq!(hits[0].0, "near", "metric {metric:?}: nearest is the query itself");
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
        let allowed_from = |eid: &str| -> bool {
            eid.trim_start_matches('v').parse::<usize>().unwrap() >= 20
        };

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
        let mean_err: f32 =
            v.iter().zip(back.iter()).map(|(a, b)| (a - b).abs()).sum::<f32>() / dim as f32;
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
        assert_eq!(a, b, "snapshot→restore must preserve every (eid, vector) exactly");

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
            VectorSpec { dim, metric, backend: crate::types::VectorBackend::FlatCpu, quantize: None }
        }

        for metric in [VectorMetric::L2, VectorMetric::Cosine, VectorMetric::Dot] {
            let mut rng = rand::rngs::StdRng::seed_from_u64(0xBADCAFE ^ metric as u64);
            let dim = 12u32;
            let s = flat_spec(dim, metric);

            // 30 BASE vectors. (Dot wants unit-norm inputs.)
            let mk = |rng: &mut rand::rngs::StdRng| -> Vec<f32> {
                let raw = rand_vec(rng, dim as usize);
                if matches!(metric, VectorMetric::Dot) { normalize(raw) } else { raw }
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
            let reader = std::sync::Arc::new(crate::segment::SegmentReader::open(&seg_path).unwrap());
            let reopened = FlatCpuIndex::open_from_segment(s, reader, row_eids).unwrap();

            // The store must NOT hold the base vectors (they live on the mmap) —
            // this is the RAM-bound invariant. `len` still reports all 30 (live).
            {
                let inner = reopened.inner.lock().unwrap();
                assert_eq!(inner.store.len(), 0, "{metric:?}: reopen must NOT re-store base vectors");
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
            assert!(!reopened.remove("b3").unwrap(), "{metric:?}: double-remove is no-op");
            assert_eq!(reopened.len(), 35, "{metric:?}: 40 - 5 deleted");

            // kNN must be BYTE-IDENTICAL to the in-RAM oracle: same eids, same order,
            // same f32 score bits — across a battery of probes (including exact-match
            // probes that land on base, tail, and deleted rows).
            let mut probes: Vec<Vec<f32>> = vec![all[5].1.clone(), all[35].1.clone(), all[3].1.clone()];
            for _ in 0..6 {
                probes.push(mk(&mut rng));
            }
            for (pi, q) in probes.iter().enumerate() {
                for k in [1usize, 5, 12, 40] {
                    let a = reopened.search_knn(q, k).unwrap();
                    let b = oracle.search_knn(q, k).unwrap();
                    let ab: Vec<(String, u32)> = a.iter().map(|(e, s)| (e.clone(), s.to_bits())).collect();
                    let bb: Vec<(String, u32)> = b.iter().map(|(e, s)| (e.clone(), s.to_bits())).collect();
                    assert_eq!(
                        ab, bb,
                        "{metric:?}: probe {pi} k={k} kNN diverged from in-RAM oracle (base-seg + tail + tombstone compose broke)"
                    );
                    // A deleted id must never appear.
                    for (e, _) in &a {
                        assert!(!["b3", "b17", "b29", "t0", "t7"].contains(&e.as_str()),
                            "{metric:?}: deleted id {e} leaked into kNN");
                    }
                }
            }

            // Snapshot of the reopened (sealed) index must read every LIVE row off
            // the mmap+tail (NOT store) and match the oracle's live set.
            let (mut snap, _) = reopened.dump_for_snapshot().unwrap();
            let (mut osnap, _) = oracle.dump_for_snapshot().unwrap();
            snap.sort_by(|a, b| a.0.cmp(&b.0));
            osnap.sort_by(|a, b| a.0.cmp(&b.0));
            assert_eq!(snap, osnap, "{metric:?}: snapshot of reopened index must match oracle live set");
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
        let q = idx
            .inner
            .read()
            .unwrap()
            .store
            .get_decoded(&q_eid)
            .unwrap();
        assert!(idx.remove(&q_eid).unwrap());
        let hits = idx.search_knn(&q, 5).unwrap();
        assert!(
            !hits.iter().any(|(e, _)| e == &q_eid),
            "removed eid {q_eid} still in {hits:?}"
        );
    }

}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/vector_index.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/vector_index.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```
