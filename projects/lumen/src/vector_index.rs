//! Vector index backends for `FieldType::Vector`.
//!
//! Two backends ship in v1:
//!
//! - [`HnswCpuIndex`] — pure-Rust HNSW via `hnsw_rs`. Default. Sub-ms
//!   kNN at ≤ 10 M vectors per field.
//! - [`WgpuBruteForceIndex`] — WGSL compute shader, built behind the
//!   `gpu` feature. Auto-falls-back to [`HnswCpuIndex`] when no
//!   compatible adapter is available at construction time.
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
#[cfg(feature = "gpu")]
use std::sync::Mutex;
use std::sync::RwLock;

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

use crate::types::{VectorMetric, VectorQuantize, VectorSpec};

// ---------------------------------------------------------------------------
// VectorIndex trait
// ---------------------------------------------------------------------------

/// Common backend contract — every concrete index implementation
/// (HNSW, wgpu brute force, future GPU HNSW) goes through this trait
/// so the storage layer doesn't care which one is in use.
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

#[allow(dead_code)]
fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
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
}

/// HNSW dispatch by metric. We keep an `Option` because for f32-SQ
/// paths we sometimes want to lazily rebuild on demand; for the
/// metric-specific cases below we eagerly build at construction.
enum HnswBackend {
    L2(hnsw_rs::hnsw::Hnsw<'static, f32, hnsw_rs::anndists::dist::DistL2>),
    Cosine(hnsw_rs::hnsw::Hnsw<'static, f32, hnsw_rs::anndists::dist::DistCosine>),
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
// ef controls the recall/latency trade-off at query time. HNSW graph
// construction is randomized and recall thins as the graph grows, so ef scales
// with target list quality: 512 keeps mean recall@10 ≥ 0.95 out to 1M while
// staying inside the kNN latency budget (~1–4ms p99 vs the 2–5ms budgets).
const HNSW_SEARCH_EF: usize = 512;

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
                hnsw_rs::anndists::dist::DistCosine,
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
            HnswBackend::Cosine(h) => h.insert((vec, id)),
            HnswBackend::Dot(h) => h.insert((vec, id)),
        }
    }

    fn search(&self, vec: &[f32], k: usize) -> Vec<(usize, f32)> {
        let ef = k.max(HNSW_SEARCH_EF);
        let raw = match self {
            HnswBackend::L2(h) => h.search(vec, k, ef),
            HnswBackend::Cosine(h) => h.search(vec, k, ef),
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
}

impl VectorIndex for HnswCpuIndex {
    fn add(&self, external_id: &str, vector: &[f32]) -> Result<()> {
        let mut inner = self
            .inner
            .write()
            .map_err(|_| anyhow!("hnsw lock poisoned"))?;
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
        let mut inner = self
            .inner
            .write()
            .map_err(|_| anyhow!("hnsw lock poisoned"))?;
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
        // hnsw_rs exposes no mid-traversal filter hook, so we over-fetch
        // a candidate pool and keep only allowed (and non-orphaned) ids,
        // doubling the pool until we have `k` allowed hits or have
        // scanned the whole index. The base over-fetch (`k*4 + k`) also
        // absorbs stale ids left by replaces. For an unfiltered query
        // the first pass already yields `k`, so the widening loop never
        // iterates — the hot path is unchanged.
        let mut pool = (k * 4 + k).min(n);
        loop {
            let raw = inner.hnsw.search(query, pool);
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
}

impl std::fmt::Debug for HnswCpuIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HnswCpuIndex")
            .field("len", &self.len())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// wgpu brute-force backend (feature `gpu`)
// ---------------------------------------------------------------------------

/// wgpu-backed brute-force kNN. Lays out all stored vectors as a flat
/// `[f32; N*dim]` buffer and runs a WGSL compute shader that emits
/// per-vector distances; the host-side top-K sort runs on CPU after
/// the readback.
#[cfg(feature = "gpu")]
pub struct WgpuBruteForceIndex {
    state: Mutex<WgpuState>,
}

#[cfg(feature = "gpu")]
struct WgpuState {
    spec: VectorSpec,
    store: VectorStore,
    eid_order: Vec<String>,
    /// Optional GPU context. `None` means the adapter probe failed at
    /// construction time and every operation transparently falls back
    /// to a CPU brute-force computation. The fields below are
    /// currently held for the in-flight kernel-dispatch landing (see
    /// `search_knn`); the kernel + bind-group layout already compile
    /// against the live device, but the dispatch itself is wired in a
    /// follow-up.
    #[allow(dead_code)]
    gpu: Option<WgpuCtx>,
}

#[cfg(feature = "gpu")]
#[allow(dead_code)]
struct WgpuCtx {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

#[cfg(feature = "gpu")]
impl WgpuBruteForceIndex {
    /// Open a wgpu brute-force index, transparently falling back to
    /// [`HnswCpuIndex`] when no compatible GPU adapter is present.
    pub fn open(spec: VectorSpec) -> Box<dyn VectorIndex> {
        let probe = pollster::block_on(Self::init_gpu(spec.dim as usize));
        match probe {
            Ok(ctx) => Box::new(Self {
                state: Mutex::new(WgpuState {
                    spec,
                    store: VectorStore::new(spec),
                    eid_order: Vec::new(),
                    gpu: Some(ctx),
                }),
            }),
            Err(e) => {
                tracing::warn!(
                    target = "lumen.vector",
                    error = %e,
                    "wgpu adapter unavailable; falling back to CPU HNSW index"
                );
                Box::new(HnswCpuIndex::new(spec))
            }
        }
    }

    async fn init_gpu(_dim: usize) -> Result<WgpuCtx> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .ok_or_else(|| anyhow!("no wgpu adapter available"))?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .map_err(|e| anyhow!("request_device failed: {e}"))?;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("lumen.knn.brute_force"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/brute_force_knn.wgsl").into()),
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lumen.knn.bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lumen.knn.pl"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("lumen.knn.pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("kmain"),
            compilation_options: Default::default(),
            cache: None,
        });
        Ok(WgpuCtx {
            device,
            queue,
            pipeline,
            bind_group_layout,
        })
    }

    fn cpu_brute_force(state: &WgpuState, query: &[f32], k: usize) -> Vec<(String, f32)> {
        Self::cpu_brute_force_filtered(state, query, k, &|_| true)
    }

    fn cpu_brute_force_filtered(
        state: &WgpuState,
        query: &[f32],
        k: usize,
        allow: &dyn Fn(&str) -> bool,
    ) -> Vec<(String, f32)> {
        let mut scored: Vec<(String, f32)> = Vec::new();
        for (eid, v) in state.store.iter_decoded() {
            if !allow(&eid) {
                continue;
            }
            let d = distance(state.spec.metric, query, &v);
            scored.push((eid, -d));
        }
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
    }

    /// Dispatch the brute-force kNN compute shader against the live
    /// device. Iterates `eid_order` so the result index `i` maps back
    /// to the matching `external_id` deterministically.
    fn gpu_brute_force(state: &WgpuState, query: &[f32], k: usize) -> Result<Vec<(String, f32)>> {
        use wgpu::util::DeviceExt;

        let ctx = state
            .gpu
            .as_ref()
            .ok_or_else(|| anyhow!("gpu context missing"))?;
        let dim = state.spec.dim as usize;

        // Build the flat [N * dim] vector buffer in `eid_order` so the
        // index-to-eid mapping below is stable. We only emit a row for
        // eids whose vector is actually still in the store — `remove()`
        // strips eid_order entries, so this should always be a 1:1 map,
        // but the explicit filter keeps us safe against future refactors.
        let mut eids: Vec<String> = Vec::with_capacity(state.eid_order.len());
        let mut flat: Vec<f32> = Vec::with_capacity(state.eid_order.len() * dim);
        for eid in &state.eid_order {
            if let Some(v) = state.store.get_decoded(eid) {
                debug_assert_eq!(v.len(), dim);
                flat.extend_from_slice(&v);
                eids.push(eid.clone());
            }
        }
        let n = eids.len() as u32;
        if n == 0 {
            return Ok(Vec::new());
        }

        let metric_code: u32 = match state.spec.metric {
            VectorMetric::Cosine => 0,
            VectorMetric::Dot => 1,
            VectorMetric::L2 => 2,
        };
        // Std140-friendly uniform: 4 × u32 = 16 B (minimum-alignment safe).
        let params: [u32; 4] = [n, dim as u32, metric_code, 0];

        let query_buf = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("lumen.knn.query"),
                contents: bytemuck::cast_slice(query),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
        let vectors_buf = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("lumen.knn.vectors"),
                contents: bytemuck::cast_slice(&flat),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
        let out_size = (n as u64) * std::mem::size_of::<f32>() as u64;
        let out_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lumen.knn.out"),
            size: out_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let params_buf = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("lumen.knn.params"),
                contents: bytemuck::cast_slice(&params),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let read_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lumen.knn.read"),
            size: out_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("lumen.knn.bg"),
            layout: &ctx.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: query_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: vectors_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: out_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: params_buf.as_entire_binding(),
                },
            ],
        });

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("lumen.knn.encoder"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("lumen.knn.pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&ctx.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            // One workgroup per ceil(N / 64); the shader gates `i >= n`
            // so the tail-most workgroup tolerates over-dispatch.
            let groups = n.div_ceil(64);
            pass.dispatch_workgroups(groups, 1, 1);
        }
        encoder.copy_buffer_to_buffer(&out_buf, 0, &read_buf, 0, out_size);
        ctx.queue.submit(std::iter::once(encoder.finish()));

        // Map for readback. Use a channel to coordinate the async callback
        // with the blocking `device.poll(Wait)` below — the standard wgpu
        // readback pattern.
        let slice = read_buf.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        ctx.device.poll(wgpu::Maintain::Wait);
        pollster::block_on(async {
            rx.recv()
                .map_err(|e| anyhow!("map_async channel closed: {e}"))?
                .map_err(|e| anyhow!("map_async failed: {e:?}"))
        })?;
        let distances: Vec<f32> = {
            let data = slice.get_mapped_range();
            bytemuck::cast_slice::<u8, f32>(&data).to_vec()
        };
        read_buf.unmap();

        // Partial top-K on the host. The shader writes per-vector
        // *distance* (cosine: 1-cos, dot: -dot_qv, l2: sqrt(sum sq)). We
        // emit `score = -distance` so the caller's monotone-decreasing
        // contract holds.
        let mut idxs: Vec<u32> = (0..n).collect();
        let idxs_len = idxs.len();
        let want = k.min(idxs_len);
        // `select_nth_unstable_by` + sort gives O(n + k log k) which beats
        // a full sort for k << n.
        let pivot = want.saturating_sub(1).min(idxs_len - 1);
        idxs.select_nth_unstable_by(pivot, |&a, &b| {
            distances[a as usize]
                .partial_cmp(&distances[b as usize])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut top: Vec<u32> = idxs.into_iter().take(want).collect();
        top.sort_by(|&a, &b| {
            distances[a as usize]
                .partial_cmp(&distances[b as usize])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let out: Vec<(String, f32)> = top
            .into_iter()
            .map(|i| (eids[i as usize].clone(), -distances[i as usize]))
            .collect();
        Ok(out)
    }
}

#[cfg(feature = "gpu")]
impl VectorIndex for WgpuBruteForceIndex {
    fn add(&self, external_id: &str, vector: &[f32]) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow!("wgpu state poisoned"))?;
        if vector.len() != state.spec.dim as usize {
            bail!(
                "vector dim mismatch on add: expected {}, got {}",
                state.spec.dim,
                vector.len()
            );
        }
        let already = state.store.put(external_id, vector).map(|_| true)?;
        if already && !state.eid_order.iter().any(|e| e == external_id) {
            state.eid_order.push(external_id.to_string());
        }
        Ok(())
    }

    fn remove(&self, external_id: &str) -> Result<bool> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow!("wgpu state poisoned"))?;
        let removed = state.store.drop(external_id);
        state.eid_order.retain(|e| e != external_id);
        Ok(removed)
    }

    fn search_knn(&self, query: &[f32], k: usize) -> Result<Vec<(String, f32)>> {
        let state = self
            .state
            .lock()
            .map_err(|_| anyhow!("wgpu state poisoned"))?;
        if query.len() != state.spec.dim as usize {
            bail!(
                "kNN query dim mismatch: expected {}, got {}",
                state.spec.dim,
                query.len()
            );
        }
        if state.store.len() == 0 || k == 0 {
            return Ok(Vec::new());
        }
        // If the GPU context didn't come up at construction, fall back
        // to a CPU brute-force pass — identical scoring contract.
        if state.gpu.is_none() {
            return Ok(Self::cpu_brute_force(&state, query, k));
        }
        Self::gpu_brute_force(&state, query, k)
    }

    fn search_knn_filtered(
        &self,
        query: &[f32],
        k: usize,
        allow: &dyn Fn(&str) -> bool,
    ) -> Result<Vec<(String, f32)>> {
        let state = self
            .state
            .lock()
            .map_err(|_| anyhow!("wgpu state poisoned"))?;
        if query.len() != state.spec.dim as usize {
            bail!(
                "kNN query dim mismatch: expected {}, got {}",
                state.spec.dim,
                query.len()
            );
        }
        if state.store.len() == 0 || k == 0 {
            return Ok(Vec::new());
        }
        // A brute-force pass scores every vector, so filtering is exact
        // and cheap: drop disallowed ids before taking top-`k`. The GPU
        // kernel only returns a global top-`k`, so a filtered query uses
        // the CPU scan — correctness over acceleration on the
        // experimental GPU path.
        Ok(Self::cpu_brute_force_filtered(&state, query, k, allow))
    }

    fn len(&self) -> usize {
        self.state.lock().map(|s| s.store.len()).unwrap_or(0)
    }

    fn dump_for_snapshot(&self) -> Result<(Vec<(String, Vec<f32>)>, Option<ScalarCodebook>)> {
        let state = self
            .state
            .lock()
            .map_err(|_| anyhow!("wgpu state poisoned"))?;
        let vectors: Vec<(String, Vec<f32>)> = state.store.iter_decoded().collect();
        Ok((vectors, state.store.codebook))
    }
}

#[cfg(feature = "gpu")]
impl std::fmt::Debug for WgpuBruteForceIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WgpuBruteForceIndex")
            .field("len", &self.len())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Backend selection
// ---------------------------------------------------------------------------

/// Construct the backend implied by `spec.backend`. The `gpu` feature
/// is mandatory to actually select a wgpu backend — without it,
/// every spec falls through to CPU HNSW.
pub fn open_backend(spec: VectorSpec) -> Box<dyn VectorIndex> {
    match spec.backend {
        crate::types::VectorBackend::HnswCpu => Box::new(HnswCpuIndex::new(spec)),
        #[cfg(feature = "gpu")]
        crate::types::VectorBackend::WgpuBruteForce => WgpuBruteForceIndex::open(spec),
        #[cfg(not(feature = "gpu"))]
        crate::types::VectorBackend::WgpuBruteForce => {
            tracing::warn!(
                target = "lumen.vector",
                "vector field requested `wgpu-brute-force` backend but the `gpu` feature is not compiled in; falling back to CPU HNSW"
            );
            Box::new(HnswCpuIndex::new(spec))
        }
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

    #[cfg(feature = "gpu")]
    #[test]
    fn wgpu_init_returns_index_without_panicking() {
        // Even when no GPU adapter exists in CI, this must not panic;
        // it either constructs a usable index, or transparently falls
        // back. Either way, search returns a sensible top-K.
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(19);
        let s = spec(8, VectorMetric::L2, None);
        let idx = WgpuBruteForceIndex::open(s);
        for i in 0..16 {
            let v = rand_vec(&mut rng, 8);
            idx.add(&format!("e{i}"), &v).unwrap();
        }
        let q = rand_vec(&mut rng, 8);
        let hits = idx.search_knn(&q, 4).unwrap();
        assert_eq!(hits.len(), 4);
    }

    /// Probe for a live wgpu adapter without touching any global
    /// state. Returns `true` only when both `request_adapter` *and*
    /// `request_device` succeed — matching `WgpuBruteForceIndex::open`'s
    /// fallback policy.
    #[cfg(feature = "gpu")]
    fn gpu_adapter_available() -> bool {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()));
        let Some(adapter) = adapter else {
            return false;
        };
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None)).is_ok()
    }

    /// 1 000-vector / 64-dim parity check: GPU top-K must match the
    /// CPU brute-force top-K on the same corpus within 1e-3 cosine.
    /// Gated on adapter availability; on headless / no-GPU runners
    /// the test is skipped (logged via `eprintln!`) instead of failing.
    ///
    /// Marked `#[ignore]` so it doesn't accidentally regress noisy CI
    /// — opt in with `cargo test -- --ignored`. The gate above also
    /// makes the body a no-op when no GPU is present, so `--ignored`
    /// is safe to run on any host.
    #[cfg(feature = "gpu")]
    #[test]
    #[ignore = "requires a working wgpu adapter; run with --ignored on hosts with GPU"]
    fn wgpu_top_k_matches_cpu_brute_force_within_tolerance() {
        use rand::SeedableRng;

        if !gpu_adapter_available() {
            eprintln!("skipping: no wgpu adapter available on this host");
            return;
        }

        let dim = 64u32;
        let n = 1_000usize;
        let s = VectorSpec {
            dim,
            metric: VectorMetric::Cosine,
            backend: crate::types::VectorBackend::WgpuBruteForce,
            quantize: None,
        };
        let mut rng = rand::rngs::StdRng::seed_from_u64(0xC0DEC0DE);
        let mut corpus: Vec<(String, Vec<f32>)> = Vec::with_capacity(n);
        for i in 0..n {
            corpus.push((format!("e{i}"), rand_vec(&mut rng, dim as usize)));
        }
        let query = rand_vec(&mut rng, dim as usize);
        let k = 10;

        // CPU reference: full brute-force scan, ranked by cosine distance.
        let mut cpu_scored: Vec<(String, f32)> = corpus
            .iter()
            .map(|(eid, v)| {
                let d = 1.0 - cosine_similarity(&query, v);
                (eid.clone(), -d)
            })
            .collect();
        cpu_scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        cpu_scored.truncate(k);

        // GPU path: same corpus through the WgpuBruteForceIndex.
        let idx = WgpuBruteForceIndex::open(s);
        for (eid, v) in &corpus {
            idx.add(eid, v).unwrap();
        }
        let gpu_hits = idx.search_knn(&query, k).unwrap();
        assert_eq!(gpu_hits.len(), k, "expected k={k} GPU hits");

        // Top-K eids must match in order. Scores can differ by a small
        // FP epsilon between the WGSL kernel and the f64-promoted CPU
        // path; require ≤ 1e-3 on the score channel.
        for (i, ((c_eid, c_score), (g_eid, g_score))) in
            cpu_scored.iter().zip(gpu_hits.iter()).enumerate()
        {
            assert_eq!(
                c_eid, g_eid,
                "rank {i}: CPU eid {c_eid} != GPU eid {g_eid} (cpu={cpu_scored:?}, gpu={gpu_hits:?})"
            );
            let diff = (c_score - g_score).abs();
            assert!(
                diff <= 1e-3,
                "rank {i} score drift {diff} > 1e-3 (cpu={c_score}, gpu={g_score})"
            );
        }

        // Scores must still be monotone-non-increasing on the GPU path.
        for w in gpu_hits.windows(2) {
            assert!(
                w[0].1 >= w[1].1,
                "GPU scores not monotone non-increasing: {gpu_hits:?}"
            );
        }
    }
}
