//! HNSW graph ANN index — the default graph-based approximate-nearest-neighbor
//! index (Faiss / Qdrant / Milvus / pgvector all ship one).
//!
//! [`HnswIndex`] wraps the `hnsw_rs` crate (0.3 — the SAME crate + version lumen
//! uses in `projects/lumen`, so the shared workspace `Cargo.lock` stays
//! single-versioned) behind beam's [`VectorIndex`] contract. It is a **CPU**
//! index: the hierarchical navigable small-world graph is built and traversed on
//! the host, complementing the GPU flat / IVF-PQ backends.
//!
//! ## Metric mapping (the #1 correctness detail)
//!
//! `hnsw_rs`/`anndists` distance types order **smaller = closer**; beam's three
//! metrics map onto them so the graph's nearest = beam's best:
//!
//! - [`Metric::L2`]  → `DistL2` on the raw vectors. `DistL2` returns the true
//!   Euclidean distance (`√Σ(q−x)²`), which ranks identically to beam's stored
//!   squared-L2 score, so the graph's nearest neighbor is beam's nearest.
//! - [`Metric::Cosine`] → `DistDot` on **unit-normalized** vectors (exactly what
//!   lumen does): for unit vectors `1 − dot = 1 − cos`, so a smaller `DistDot`
//!   is a larger cosine. Beam already L2-normalizes cosine rows on insert; we
//!   scale to *just under* unit ([`normalize_unit_safe`]) to dodge `anndists`'
//!   `assert(dot ≤ 1)` when float rounding pushes a unit self-dot past 1.
//! - [`Metric::Dot`] → `DistL2` over a **MIPS→L2 augmentation**. Plain `DistDot`
//!   would panic on beam's arbitrary-norm Dot vectors (its assert requires
//!   `dot ≤ 1`), and cosine is the wrong ranking for raw inner product. Instead
//!   each stored `x` gets an extra coordinate `√(M²−‖x‖²)` (`M` = max stored
//!   norm) so every point lies on a radius-`M` sphere in `dim+1` space, and the
//!   query is augmented with a `0`. Then `‖q'−x'‖² = ‖q‖² + M² − 2·(q·x)`, so
//!   the L2-nearest augmented point is exactly the largest `q·x` — an exact
//!   ranking reduction of maximum-inner-product search to L2 nearest-neighbor.
//!
//! The returned [`Neighbor::score`] is always **recomputed in beam's native
//! convention** from the stored row (squared-L2 for L2, dot for Dot/Cosine), so
//! an `HnswIndex` result is directly comparable to the [`CpuFlatIndex`] oracle —
//! the graph distance is used only to *find* candidates, never to score them.
//!
//! [`CpuFlatIndex`]: crate::index::cpu_flat::CpuFlatIndex
//!
//! ## Id mapping, tombstones, and mutation
//!
//! Each **live** collection row is inserted with its physical row index as the
//! `hnsw_rs` point id, so a returned point id *is* the beam row (→ external id +
//! payload) with no side table. Tombstoned rows are simply never inserted, and
//! search additionally drops any point whose live bit is unset — the "live-mask
//! post-filter" that excludes deletes from results.
//!
//! `hnsw_rs` (like HNSW generally) is **insert-only**: there is no cheap graph
//! delete. So for beam an [`HnswIndex`] is a **build-then-query** index. A delete
//! present at build time is excluded by construction (it is never inserted); an
//! insert or an update (LSM-style tombstone + append) that changes the row set
//! requires **rebuilding** the index from the collection — the same operational
//! model every production HNSW system uses. This is deliberate: we do not
//! over-engineer graph deletion.
//!
//! ## Persistence
//!
//! The graph is not serialized. `hnsw_rs` has its own dump format, but beam's
//! collection segment is already the durable source-of-truth, so a cold start
//! **rebuilds** the `HnswIndex` from the loaded [`Collection`] — acceptable and
//! documented, matching how the GPU buffers are rebuilt on load.

use std::cmp::Ordering;

use hnsw_rs::anndists::dist::{DistDot, DistL2};
use hnsw_rs::hnsw::Hnsw;

use crate::collection::{l2_normalize, Collection, Metric};
use crate::index::{Neighbor, VectorIndex};
use crate::payload::{Filter, Payload};

/// Ceiling on the HNSW layer count (`hnsw_rs` internally clamps to its own max);
/// 16 matches lumen's construction and is ample for ≤ 10 M points.
const HNSW_MAX_LAYER: usize = 16;

/// Filtered-search over-fetch factor: the initial candidate pool is
/// `k * OVER_FACTOR`, doubled until `k` filter-surviving neighbors are found or
/// the whole live set has been scanned (see [`HnswIndex::search_knn_filtered`]).
const FILTER_OVER_FACTOR: usize = 8;

/// Build/query knobs for an [`HnswIndex`], with sane defaults (M = 16,
/// ef_construction = 200, ef_search = 64) that clear recall@10 ≥ 0.9 on
/// clustered data with margin.
#[derive(Debug, Clone, Copy)]
pub struct HnswConfig {
    /// `M` — the number of bidirectional links kept per node per layer. Larger
    /// `M` builds a denser graph: higher recall, more memory + slower build.
    pub max_nb_connection: usize,
    /// `ef_construction` — the beam width while building the graph. Larger =
    /// higher-quality graph (higher recall) at a higher build cost.
    pub ef_construction: usize,
    /// `ef_search` — the beam width at query time (the recall/latency lever).
    /// Larger = higher recall, slower query. Tunable per-query without rebuild
    /// via [`HnswIndex::set_ef_search`].
    pub ef_search: usize,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            max_nb_connection: 16,
            ef_construction: 200,
            ef_search: 64,
        }
    }
}

/// Scale `v` to *just under* unit norm (`‖·‖ = 1 − 1e-6`) for the `DistDot`
/// cosine path — the same trick lumen uses. A plain unit vector has `dot(v, v)
/// == 1`, which float rounding can push a hair above 1 and trip `anndists`'
/// `assert(dot ≤ 1)`; scaling under unit keeps every pairwise `dot < 1` while
/// preserving cosine ranking exactly (cosine is scale-invariant). A zero vector
/// passes through unchanged (its dot with anything is 0).
fn normalize_unit_safe(v: &[f32]) -> Vec<f32> {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm == 0.0 {
        return v.to_vec();
    }
    let inv = (1.0 - 1e-6) / norm;
    v.iter().map(|x| x * inv).collect()
}

/// The MIPS→L2 augmented DB vector for the Dot metric: append `√(M²−‖x‖²)` so
/// every stored point lands on a radius-`M` sphere in `dim+1` space (see the
/// module docs). `M²` = `max_norm_sq` is the max stored norm² over live rows, so
/// the radicand is always non-negative (clamped for float safety).
fn augment_db(row: &[f32], max_norm_sq: f32) -> Vec<f32> {
    let norm_sq: f32 = row.iter().map(|x| x * x).sum();
    let extra = (max_norm_sq - norm_sq).max(0.0).sqrt();
    let mut out = Vec::with_capacity(row.len() + 1);
    out.extend_from_slice(row);
    out.push(extra);
    out
}

/// The augmented Dot query: the raw query with a trailing `0` (its extra
/// coordinate contributes nothing, so L2-nearest reduces to max dot).
fn augment_query(query: &[f32]) -> Vec<f32> {
    let mut out = Vec::with_capacity(query.len() + 1);
    out.extend_from_slice(query);
    out.push(0.0);
    out
}

/// The concrete `hnsw_rs` graph, one variant per metric mapping (see the module
/// docs). Cosine + Dot both route through a graph whose points are transformed
/// (unit-normalized / augmented) at insert and query time.
enum HnswBackend {
    /// `DistL2` over raw `dim`-vectors (L2).
    L2(Hnsw<'static, f32, DistL2>),
    /// `DistDot` over unit-normalized `dim`-vectors (Cosine).
    Cosine(Hnsw<'static, f32, DistDot>),
    /// `DistL2` over MIPS-augmented `dim+1`-vectors (Dot).
    Dot(Hnsw<'static, f32, DistL2>),
}

impl HnswBackend {
    /// Insert stored row `row` (already in the metric's transformed space) under
    /// point id `id`.
    fn insert(&self, row: &[f32], id: usize, max_norm_sq: f32) {
        match self {
            HnswBackend::L2(h) => h.insert((row, id)),
            HnswBackend::Cosine(h) => h.insert((&normalize_unit_safe(row), id)),
            HnswBackend::Dot(h) => h.insert((&augment_db(row, max_norm_sq), id)),
        }
    }

    /// Approximate `k`-nearest point ids to `query`, `ef` beam width. Returns the
    /// raw `hnsw_rs` point ids (== beam physical rows) best-first by graph
    /// distance; the caller masks tombstones + recomputes beam scores.
    fn search(&self, query: &[f32], k: usize, ef: usize) -> Vec<usize> {
        let ef = k.max(ef);
        let raw = match self {
            HnswBackend::L2(h) => h.search(query, k, ef),
            HnswBackend::Cosine(h) => h.search(&normalize_unit_safe(query), k, ef),
            HnswBackend::Dot(h) => h.search(&augment_query(query), k, ef),
        };
        raw.into_iter().map(|n| n.d_id).collect()
    }
}

/// A build-then-query HNSW graph index over a snapshot of one collection.
///
/// Owns everything a query needs — the graph, a row-major copy of the stored
/// vectors (for exact score recomputation), external ids, payloads, and the
/// `live` tombstone bits — so it is self-contained (no borrow of the source
/// [`Collection`]). Built by [`HnswIndex::build`]; queried via [`VectorIndex`].
pub struct HnswIndex {
    dim: usize,
    metric: Metric,
    config: HnswConfig,
    backend: HnswBackend,
    /// Row-major stored vectors (`capacity * dim`, exactly `collection.data()`),
    /// used to recompute each result's score in beam's native metric convention.
    data: Vec<f32>,
    external_ids: Vec<String>,
    payloads: Vec<Payload>,
    /// Per physical row liveness at build time; folded into search as the
    /// live-mask post-filter so tombstoned rows are excluded from results.
    live: Vec<bool>,
    n_live: usize,
    n: usize,
}

impl HnswIndex {
    /// Build an HNSW graph over `collection`'s **live** rows (each inserted under
    /// its physical row index as the point id) with `config`. Tombstoned rows are
    /// not inserted, so a delete present at build time is excluded by
    /// construction. Fully self-contained afterward.
    pub fn build(collection: &Collection, config: HnswConfig) -> Self {
        let dim = collection.dim();
        let metric = collection.metric();
        let n = collection.capacity();
        let n_live = collection.len();
        let max_elements = n_live.max(1);
        let m = config.max_nb_connection.max(1);
        let efc = config.ef_construction.max(1);
        let live = collection.live();

        // Dot needs the max stored norm² over live rows for the MIPS→L2
        // augmentation (all points share a radius-M sphere); harmless otherwise.
        let mut max_norm_sq = 0.0f32;
        if metric == Metric::Dot {
            for (i, &is_live) in live.iter().enumerate() {
                if is_live {
                    let ns: f32 = collection.row(i).iter().map(|x| x * x).sum();
                    if ns > max_norm_sq {
                        max_norm_sq = ns;
                    }
                }
            }
        }

        let backend = match metric {
            Metric::L2 => HnswBackend::L2(Hnsw::new(m, max_elements, HNSW_MAX_LAYER, efc, DistL2)),
            Metric::Cosine => {
                HnswBackend::Cosine(Hnsw::new(m, max_elements, HNSW_MAX_LAYER, efc, DistDot))
            }
            Metric::Dot => HnswBackend::Dot(Hnsw::new(m, max_elements, HNSW_MAX_LAYER, efc, DistL2)),
        };

        for (i, &is_live) in live.iter().enumerate() {
            if is_live {
                backend.insert(collection.row(i), i, max_norm_sq);
            }
        }

        Self {
            dim,
            metric,
            config,
            backend,
            data: collection.data().to_vec(),
            external_ids: collection.external_ids().to_vec(),
            payloads: collection.payloads().to_vec(),
            live: live.to_vec(),
            n_live,
            n,
        }
    }

    /// Number of **live** indexed vectors.
    pub fn len(&self) -> usize {
        self.n_live
    }

    /// Whether the index holds zero live vectors.
    pub fn is_empty(&self) -> bool {
        self.n_live == 0
    }

    /// Physical indexed-row count (live + tombstoned at build time).
    pub fn capacity(&self) -> usize {
        self.n
    }

    /// Tombstoned rows excluded from the graph at build time (`capacity − live`).
    pub fn tombstoned(&self) -> usize {
        self.n - self.n_live
    }

    /// The vector dimension.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// The collection metric this graph was built under.
    pub fn metric(&self) -> Metric {
        self.metric
    }

    /// The build/query config in force.
    pub fn config(&self) -> HnswConfig {
        self.config
    }

    /// The current query-time `ef_search` beam width.
    pub fn ef_search(&self) -> usize {
        self.config.ef_search
    }

    /// Override the query-time `ef_search` (recall/latency lever) on the already
    /// built graph, no rebuild — used by the bench + tests to sweep recall.
    pub fn set_ef_search(&mut self, ef: usize) {
        self.config.ef_search = ef.max(1);
    }

    /// Recompute row `row`'s score against `query` in beam's native metric
    /// convention (squared-L2 for L2, dot for Dot/Cosine over the stored — for
    /// Cosine, unit — row), identical to [`CpuFlatIndex`](crate::index::cpu_flat::CpuFlatIndex).
    fn score_row(&self, query: &[f32], row: usize) -> f32 {
        let r = &self.data[row * self.dim..(row + 1) * self.dim];
        match self.metric {
            Metric::L2 => query
                .iter()
                .zip(r)
                .map(|(q, x)| {
                    let d = q - x;
                    d * d
                })
                .sum(),
            Metric::Dot => query.iter().zip(r).map(|(q, x)| q * x).sum(),
            Metric::Cosine => {
                // Stored cosine rows are unit; normalize the query so dot == cos.
                let qn = l2_normalize(query);
                qn.iter().zip(r).map(|(q, x)| q * x).sum()
            }
        }
    }

    /// Order two results best-first under the metric (L2 ascending; Dot/Cosine
    /// descending), NaN last.
    fn better(&self, a: &Neighbor, b: &Neighbor) -> Ordering {
        let ord = a.score.partial_cmp(&b.score).unwrap_or(Ordering::Equal);
        if self.metric.larger_is_better() {
            ord.reverse()
        } else {
            ord
        }
    }

    /// Turn raw graph point ids into best-first [`Neighbor`]s: drop any
    /// non-live id (the live-mask post-filter), recompute beam scores, and sort.
    fn resolve(&self, query: &[f32], ids: Vec<usize>) -> Vec<Neighbor> {
        let mut out: Vec<Neighbor> = ids
            .into_iter()
            .filter(|&row| self.live.get(row).copied().unwrap_or(false))
            .map(|row| Neighbor {
                row: row as u32,
                external_id: self.external_ids[row].clone(),
                score: self.score_row(query, row),
            })
            .collect();
        out.sort_by(|a, b| self.better(a, b));
        out
    }
}

impl VectorIndex for HnswIndex {
    /// Approximate top-`k` nearest neighbors to `query`, best-first. Traverses the
    /// graph at `ef_search`, drops tombstoned points, recomputes beam scores.
    fn search_knn(&self, query: &[f32], k: usize) -> Vec<Neighbor> {
        if query.len() != self.dim || self.n_live == 0 || k == 0 {
            return Vec::new();
        }
        let ids = self.backend.search(query, k, self.config.ef_search);
        let mut out = self.resolve(query, ids);
        out.truncate(k);
        out
    }

    fn num_vectors(&self) -> usize {
        self.n
    }

    fn row_payload(&self, row: u32) -> &Payload {
        &self.payloads[row as usize]
    }

    /// Filtered top-`k`: `hnsw_rs` has no mid-traversal filter hook, so over-fetch
    /// a candidate pool (`k * FILTER_OVER_FACTOR` initially, widened by `ef`),
    /// keep only live rows whose payload matches `filter`, and double the pool
    /// until `k` survive or the whole live set has been scanned — the nearest `k`
    /// *within the filtered set*, never a post-filter of a fixed global top-k. All
    /// results satisfy the filter and are a subset of the unfiltered candidates.
    fn search_knn_filtered(&self, query: &[f32], k: usize, filter: &Filter) -> Vec<Neighbor> {
        if query.len() != self.dim || self.n_live == 0 || k == 0 {
            return Vec::new();
        }
        let cap = self.n_live;
        let mut pool = k
            .saturating_mul(FILTER_OVER_FACTOR)
            .max(self.config.ef_search)
            .min(cap);
        loop {
            let ids = self.backend.search(query, pool, pool.max(self.config.ef_search));
            let mut out: Vec<Neighbor> = self
                .resolve(query, ids)
                .into_iter()
                .filter(|nb| filter.matches(&self.payloads[nb.row as usize]))
                .collect();
            // Enough survivors, or the pool already covers the whole live set (a
            // wider search cannot surface more) → finalize.
            if out.len() >= k || pool >= cap {
                out.truncate(k);
                return out;
            }
            pool = pool.saturating_mul(2).min(cap);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::clustered_collection;
    use crate::index::cpu_flat::CpuFlatIndex;
    use std::collections::HashSet;

    fn recall_at(got: &[Neighbor], truth: &HashSet<u32>) -> f64 {
        if got.is_empty() {
            return 1.0;
        }
        let hit = got.iter().filter(|nb| truth.contains(&nb.row)).count();
        hit as f64 / got.len() as f64
    }

    #[test]
    fn l2_recall_is_high_vs_flat_oracle() {
        let dim = 32;
        let c = clustered_collection("t", 2000, dim, Metric::L2, 16, 0.03, 5);
        let idx = HnswIndex::build(&c, HnswConfig::default());
        let oracle = CpuFlatIndex::new(&c);
        let queries = crate::dataset::clustered_queries(8, dim, 16, 0.03, 5);
        let mut sum = 0.0;
        for q in &queries {
            let truth: HashSet<u32> = oracle.search_knn(q, 10).iter().map(|n| n.row).collect();
            sum += recall_at(&idx.search_knn(q, 10), &truth);
        }
        let mean = sum / queries.len() as f64;
        assert!(mean >= 0.9, "HNSW L2 recall@10 {mean} should clear 0.9");
    }

    #[test]
    fn dot_metric_does_not_panic_and_ranks_by_inner_product() {
        // The MIPS→L2 augmentation must handle arbitrary norms without tripping
        // anndists' DistDot assert. A large-norm row should win a dot query.
        let mut c = Collection::new("t", 3, Metric::Dot);
        c.add("small", &[0.1, 0.1, 0.1]).unwrap();
        c.add("big", &[9.0, 0.0, 0.0]).unwrap();
        c.add("mid", &[1.0, 1.0, 0.0]).unwrap();
        let idx = HnswIndex::build(&c, HnswConfig::default());
        let out = idx.search_knn(&[1.0, 0.0, 0.0], 3);
        assert_eq!(out[0].external_id, "big", "largest inner product wins");
    }
}
