//! IVF-PQ (IVFADC) approximate-nearest-neighbor index — the scalable core.
//!
//! Brute-force flat search is `O(n·dim)` per query. IVF-PQ makes queries cheap
//! two ways:
//!
//! 1. **Inverted file (IVF).** A coarse quantizer (k-means over the corpus)
//!    partitions the vectors into `nlist` Voronoi cells. A query only scans the
//!    `nprobe` cells nearest its coarse centroid, touching a small fraction of
//!    the corpus.
//! 2. **Product quantization (PQ).** Each vector's **residual** to its cell
//!    centroid is split into `m` sub-vectors; each sub-vector is quantized to one
//!    of `256` codebook entries (`nbits = 8`). A vector shrinks from `dim` f32s
//!    to `m` bytes, and distance is a table lookup (asymmetric distance
//!    computation, ADC) instead of a full dot/L2.
//!
//! ## Residual handling (the #1 IVF-PQ bug source)
//!
//! Everything downstream of assignment is expressed **relative to the assigned
//! cell centroid**. For a query `q` probing cell `c` with centroid `μ_c`:
//!
//! - PQ codes encode `x − μ_{cell(x)}` (the corpus residual), NOT `x`.
//! - The per-cell ADC table is built from the **query residual** `qr = q − μ_c`,
//!   so `T_c[s][code] = ‖ qr_sub_s − codebook[s][code] ‖²`. A fresh table is
//!   computed for every probed cell.
//!
//! With [`Refine::Flat`] the full residual `x − μ_{cell}` is stored instead of
//! PQ codes, so a candidate's distance is the *exact* `‖ qr − residual ‖² =
//! ‖ q − x ‖²`. Probing every cell (`nprobe == nlist`) therefore reproduces the
//! flat oracle bit-for-intent — the invariant [`tests/ivf_recall.rs`] asserts.
//!
//! This module is [`Metric::L2`]-only (the standard IVF-PQ metric); Dot/Cosine
//! can reduce to it later. The GPU candidate scan lives in [`crate::gpu::ivfpq`]
//! and consumes the [`QueryPlan`] this module produces, so the GPU and the
//! [`QueryPlan::cpu_scan`] reference compute the same candidate distances.

use crate::collection::{Collection, Metric};
use crate::index::{Neighbor, VectorIndex};
use crate::payload::{Filter, Payload};

/// Number of PQ centroids per subspace. `nbits = 8` is the only supported width,
/// giving `2^8 = 256` codebook entries and one `u8` code byte per subspace.
pub const PQ_KSUB: usize = 256;

/// How a cell stores its members, trading exactness for compression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refine {
    /// Store each member's full residual vector (`dim` f32). Distance within a
    /// probed cell is exact; `nprobe == nlist` reproduces the flat oracle.
    Flat,
    /// Store each member as `m` PQ code bytes over its residual. `dim` must be
    /// divisible by `m`. Compressed and fast, but approximate (PQ quantization).
    Pq {
        /// Number of contiguous sub-vectors the residual is split into.
        m: usize,
    },
}

/// Training configuration for [`IvfPqIndex::train`].
#[derive(Debug, Clone, Copy)]
pub struct IvfPqConfig {
    /// Coarse-quantizer cell count (number of k-means centroids over the corpus).
    pub nlist: usize,
    /// Lloyd iterations for every k-means run (coarse + each PQ subspace).
    pub kmeans_iters: usize,
    /// PQ code width in bits. Must be `8` (256 centroids / 1 byte per subspace).
    pub nbits: u32,
    /// Cell storage mode: exact [`Refine::Flat`] or compressed [`Refine::Pq`].
    pub refine: Refine,
    /// Train the coarse quantizer + PQ codebooks on a bounded, deterministic
    /// **sample** of this many vectors instead of all `n` — standard Faiss
    /// practice that keeps k-means tractable at scale (e.g. `min(n, 100_000)` at
    /// `n = 1_000_000`). `0` means "train on all `n`". Every vector is still
    /// assigned + encoded; only the centroid/codebook *fitting* is sampled.
    pub train_sample: usize,
    /// Seed for every deterministic k-means (init + tie-breaks). No entropy.
    pub seed: u64,
}

impl Default for IvfPqConfig {
    fn default() -> Self {
        Self {
            nlist: 256,
            kmeans_iters: 20,
            nbits: 8,
            refine: Refine::Pq { m: 8 },
            train_sample: 0,
            seed: 0xBEA3_1FBE_A31F_BEA3,
        }
    }
}

/// A trained IVF-PQ index over one collection's vectors.
///
/// Built once by [`IvfPqIndex::train`]; then [`IvfPqIndex::search_cpu`] (CPU
/// reference) and the GPU path (via [`IvfPqIndex::plan`] +
/// [`crate::gpu::ivfpq::GpuIvfScanner`]) answer `k`-NN queries. Owns its coarse
/// centroids, PQ codebooks, and the inverted lists.
pub struct IvfPqIndex {
    dim: usize,
    nlist: usize,
    /// `m` for PQ, or `0` for [`Refine::Flat`].
    m: usize,
    /// `dim / m` (subspace width) for PQ; unused for Flat.
    dsub: usize,
    refine: Refine,
    /// Coarse centroids, row-major `nlist * dim`.
    coarse: Vec<f32>,
    /// PQ codebooks, `m * PQ_KSUB * dsub` f32 (empty for Flat). Layout:
    /// `codebooks[(s * PQ_KSUB + c) * dsub + d]`.
    codebooks: Vec<f32>,
    /// Inverted lists: `list_rows[cell]` = the corpus row ids assigned to `cell`.
    list_rows: Vec<Vec<u32>>,
    /// PQ codes per cell, flattened `count * m` (Pq only).
    list_codes: Vec<Vec<u8>>,
    /// Residual vectors per cell, flattened `count * dim` (Flat only).
    list_resid: Vec<Vec<f32>>,
    external_ids: Vec<String>,
    /// Row-aligned attribute payloads (snapshot of the collection), read to
    /// filter candidates in filtered search.
    payloads: Vec<Payload>,
    /// Per physical row liveness (snapshot of the collection), folded into the
    /// candidate keep-bitmask so tombstoned rows are excluded from search via the
    /// same sentinel path as a filtered-out candidate. A delete-only change is
    /// reflected by [`IvfPqIndex::refresh_mask`] without re-training.
    live: Vec<bool>,
    /// Cached live-row count.
    n_live: usize,
    /// Physical row count (live + tombstoned) — the rows trained + inverted.
    n: usize,
}

/// The GPU-ready per-cell scan payload for one probed cell set. Either PQ ADC
/// tables + codes, or exact residual vectors — chosen by the index's [`Refine`].
#[derive(Debug, Clone)]
pub enum ScanData {
    /// PQ asymmetric distance: `dist = Σ_s tables[slot·m·256 + s·256 + code]`.
    Pq {
        /// Per-probed-cell ADC tables, `num_probed * m * PQ_KSUB` f32.
        tables: Vec<f32>,
        /// Per-candidate codes as `u32` (one per subspace), `num_cand * m`.
        codes: Vec<u32>,
        /// Subspaces per code.
        m: usize,
    },
    /// Exact residual L2: `dist = Σ_d (qresid[slot·dim+d] − resid[i·dim+d])²`.
    Flat {
        /// Per-probed-cell query residuals `q − μ_cell`, `num_probed * dim` f32.
        qresid: Vec<f32>,
        /// Per-candidate stored residuals, `num_cand * dim` f32.
        resid: Vec<f32>,
        /// Vector dimension.
        dim: usize,
    },
}

/// A resolved query: the candidate set gathered from the probed cells plus the
/// data needed to score each candidate. Produced by [`IvfPqIndex::plan`] and
/// scored either by [`QueryPlan::cpu_scan`] (reference) or the GPU kernel — both
/// must agree (see the kernel-exactness test).
#[derive(Debug, Clone)]
pub struct QueryPlan {
    /// Number of probed cells (one table / query-residual slot each).
    pub num_probed: usize,
    /// Candidate corpus row ids, length `num_cand`. Laid out cell-by-cell in
    /// slot order, so slot `s`'s candidates occupy one contiguous block.
    pub rows: Vec<u32>,
    /// Per-candidate probed-cell slot (`0..num_probed`) selecting its table /
    /// query residual. Length `num_cand`. Non-decreasing (candidates are grouped
    /// by cell), which the per-candidate GPU kernels index directly.
    pub cand_slot: Vec<u32>,
    /// Start offset of each probed cell's candidate block in `rows`/`codes`,
    /// indexed by slot (`0..num_probed`). With [`QueryPlan::cell_counts`] this is
    /// what the per-cell workgroup ADC kernel grid-strides over.
    pub cell_offsets: Vec<u32>,
    /// Candidate count of each probed cell, indexed by slot (`0..num_probed`).
    /// `cell_offsets[s]..cell_offsets[s]+cell_counts[s]` is slot `s`'s block.
    pub cell_counts: Vec<u32>,
    /// The scan payload (PQ or Flat) matching the index's refine mode.
    pub data: ScanData,
}

impl QueryPlan {
    /// Number of candidates gathered across all probed cells (`<= n`). The
    /// scaling proof: at small `nprobe` this is a small fraction of `n`.
    pub fn num_candidates(&self) -> usize {
        self.rows.len()
    }

    /// CPU reference scan: the per-candidate distance, in `rows` order. The GPU
    /// kernel computes the identical math; tests assert they agree within 1e-3.
    #[allow(clippy::needless_range_loop)] // parallel arrays keyed by candidate `i`
    pub fn cpu_scan(&self) -> Vec<f32> {
        let num_cand = self.rows.len();
        let mut dist = vec![0.0f32; num_cand];
        match &self.data {
            ScanData::Pq { tables, codes, m } => {
                let m = *m;
                for i in 0..num_cand {
                    let slot = self.cand_slot[i] as usize;
                    let table_base = slot * m * PQ_KSUB;
                    let code_base = i * m;
                    let mut acc = 0.0f32;
                    for s in 0..m {
                        let c = codes[code_base + s] as usize;
                        acc += tables[table_base + s * PQ_KSUB + c];
                    }
                    dist[i] = acc;
                }
            }
            ScanData::Flat { qresid, resid, dim } => {
                let dim = *dim;
                for i in 0..num_cand {
                    let slot = self.cand_slot[i] as usize;
                    let q_base = slot * dim;
                    let r_base = i * dim;
                    let mut acc = 0.0f32;
                    for d in 0..dim {
                        let diff = qresid[q_base + d] - resid[r_base + d];
                        acc += diff * diff;
                    }
                    dist[i] = acc;
                }
            }
        }
        dist
    }
}

impl IvfPqIndex {
    /// Train an IVF-PQ index over `collection` (which must be [`Metric::L2`]).
    ///
    /// Steps: (1) coarse k-means → `nlist` centroids; (2) assign every vector to
    /// its nearest centroid and form its residual; (3) for PQ, k-means each of
    /// the `m` residual subspaces → codebooks, then encode every residual to `m`
    /// bytes; for Flat, store the residual vectors verbatim. Fully deterministic
    /// (seeded by `config.seed`).
    pub fn train(collection: &Collection, config: IvfPqConfig) -> anyhow::Result<Self> {
        let dim = collection.dim();
        // Train + invert over ALL physical rows (live + tombstoned). Tombstoned rows
        // are masked out at top-k, so at Refine::Flat + full probe every live
        // candidate is still scored exactly and the result equals the live oracle.
        let n = collection.capacity();
        let metric = collection.metric();
        if metric != Metric::L2 {
            anyhow::bail!("IvfPqIndex supports Metric::L2 only, got {metric:?}");
        }
        if config.nbits != 8 {
            anyhow::bail!("IvfPqIndex supports nbits = 8 only, got {}", config.nbits);
        }
        let nlist = config.nlist.max(1).min(n.max(1));
        if let Refine::Pq { m } = config.refine {
            if m == 0 || !dim.is_multiple_of(m) {
                anyhow::bail!("PQ m ({m}) must be > 0 and divide dim ({dim})");
            }
        }

        // Deterministic training sample: `n_train` rows spread evenly across the
        // corpus (a strided pick, seed-free and reproducible). k-means fits the
        // coarse centroids + PQ codebooks on this sample only; assignment and
        // encoding below still touch every one of the `n` vectors. This is what
        // keeps training tractable at n = 1_000_000 (train on ~100k, index all).
        let n_train = if config.train_sample == 0 {
            n
        } else {
            config.train_sample.min(n).max(1)
        };
        let sample_idx: Vec<usize> = if n_train == n {
            (0..n).collect()
        } else {
            (0..n_train)
                .map(|j| ((j as u64 * n as u64) / n_train as u64) as usize)
                .collect()
        };

        // (1) Coarse quantizer over the (sampled) raw vectors.
        let coarse = if n_train == n {
            kmeans(collection.data(), n, dim, nlist, config.kmeans_iters, config.seed)
        } else {
            let mut td = vec![0.0f32; n_train * dim];
            for (j, &i) in sample_idx.iter().enumerate() {
                td[j * dim..(j + 1) * dim]
                    .copy_from_slice(&collection.data()[i * dim..(i + 1) * dim]);
            }
            kmeans(&td, n_train, dim, nlist, config.kmeans_iters, config.seed)
        };

        // (2) Assign each vector to its nearest coarse centroid + form residual.
        let mut assign = vec![0u32; n];
        let mut residuals = vec![0.0f32; n * dim];
        for i in 0..n {
            let row = &collection.data()[i * dim..(i + 1) * dim];
            let cell = nearest_centroid(row, &coarse, nlist, dim);
            assign[i] = cell as u32;
            let cbase = cell * dim;
            for d in 0..dim {
                residuals[i * dim + d] = row[d] - coarse[cbase + d];
            }
        }

        // (3) PQ codebooks over residual subspaces (or nothing, for Flat).
        let (m, dsub, codebooks) = match config.refine {
            Refine::Flat => (0usize, 0usize, Vec::new()),
            Refine::Pq { m } => {
                let dsub = dim / m;
                // One codebook per subspace: gather the subspace columns of the
                // SAMPLED residuals, then k-means them to 256 centroids. (Every
                // residual is still encoded against these codebooks below.)
                let mut codebooks = vec![0.0f32; m * PQ_KSUB * dsub];
                let mut sub = vec![0.0f32; n_train * dsub];
                for s in 0..m {
                    for (j, &i) in sample_idx.iter().enumerate() {
                        let src = i * dim + s * dsub;
                        sub[j * dsub..(j + 1) * dsub]
                            .copy_from_slice(&residuals[src..src + dsub]);
                    }
                    // Distinct per-subspace seed so subspaces don't share init.
                    let cb = kmeans(
                        &sub,
                        n_train,
                        dsub,
                        PQ_KSUB,
                        config.kmeans_iters,
                        config.seed ^ (0x1000_0001u64.wrapping_mul(s as u64 + 1)),
                    );
                    codebooks[s * PQ_KSUB * dsub..(s + 1) * PQ_KSUB * dsub].copy_from_slice(&cb);
                }
                (m, dsub, codebooks)
            }
        };

        // Build the inverted lists (rows + codes/residuals per cell).
        let mut list_rows: Vec<Vec<u32>> = vec![Vec::new(); nlist];
        let mut list_codes: Vec<Vec<u8>> = vec![Vec::new(); nlist];
        let mut list_resid: Vec<Vec<f32>> = vec![Vec::new(); nlist];
        for i in 0..n {
            let cell = assign[i] as usize;
            list_rows[cell].push(i as u32);
            let res = &residuals[i * dim..(i + 1) * dim];
            match config.refine {
                Refine::Flat => list_resid[cell].extend_from_slice(res),
                Refine::Pq { .. } => {
                    for s in 0..m {
                        let code =
                            encode_subvector(&res[s * dsub..(s + 1) * dsub], &codebooks, s, dsub);
                        list_codes[cell].push(code);
                    }
                }
            }
        }

        Ok(Self {
            dim,
            nlist,
            m,
            dsub,
            refine: config.refine,
            coarse,
            codebooks,
            list_rows,
            list_codes,
            list_resid,
            external_ids: collection.external_ids().to_vec(),
            payloads: collection.payloads().to_vec(),
            live: collection.live().to_vec(),
            n_live: collection.len(),
            n,
        })
    }

    /// Vector dimension.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Number of coarse cells (`nlist`, clamped to `n`).
    pub fn nlist(&self) -> usize {
        self.nlist
    }

    /// Number of **live** indexed vectors (tombstoned rows excluded).
    pub fn len(&self) -> usize {
        self.n_live
    }

    /// Whether the index holds zero **live** vectors.
    pub fn is_empty(&self) -> bool {
        self.n_live == 0
    }

    /// Physical indexed-row count (live + tombstoned).
    pub fn capacity(&self) -> usize {
        self.n
    }

    /// Number of tombstoned rows still resident in the inverted lists (masked out
    /// of search).
    pub fn tombstoned(&self) -> usize {
        self.n - self.n_live
    }

    /// Re-sync the live-mask from `collection` without re-training — the mask-only
    /// path for reflecting deletes on the IVF index. Valid only when no physical
    /// rows were added since training (`collection.capacity() == self.capacity()`,
    /// i.e. delete-only); returns `false` (a no-op) otherwise, signalling the caller
    /// to rebuild to pick up appended rows.
    pub fn refresh_mask(&mut self, collection: &Collection) -> bool {
        if collection.capacity() != self.n {
            return false;
        }
        self.live = collection.live().to_vec();
        self.n_live = collection.len();
        true
    }

    /// The refine mode this index was trained with.
    pub fn refine(&self) -> Refine {
        self.refine
    }

    /// The dominant per-vector memory footprint, in bytes — the number that makes
    /// PQ win at scale. [`Refine::Flat`] stores each residual as `dim` f32
    /// (`n·dim·4` bytes, ≈ 512 MB at n=1M, dim=128); [`Refine::Pq`] stores `m`
    /// code bytes per vector (`n·m` bytes, ≈ 16 MB at n=1M, m=16) — a `dim·4/m`
    /// (≈ 32×) reduction. Excludes the fixed, `n`-independent coarse centroids +
    /// PQ codebooks (see [`Self::overhead_bytes`]) and the shared row-id lists.
    pub fn payload_bytes(&self) -> usize {
        match self.refine {
            Refine::Flat => self.n * self.dim * std::mem::size_of::<f32>(),
            Refine::Pq { .. } => self.n * self.m,
        }
    }

    /// The fixed, `n`-independent index overhead in bytes: coarse centroids
    /// (`nlist·dim` f32) plus PQ codebooks (`m·256·dsub` f32, zero for Flat).
    pub fn overhead_bytes(&self) -> usize {
        (self.coarse.len() + self.codebooks.len()) * std::mem::size_of::<f32>()
    }

    /// Build a [`QueryPlan`]: pick the `nprobe` nearest cells, gather their
    /// candidates, and assemble the scan payload (PQ tables+codes or Flat
    /// residuals). Both the CPU reference and the GPU kernel score this plan.
    pub fn plan(&self, query: &[f32], nprobe: usize) -> QueryPlan {
        let dim = self.dim;
        let nprobe = nprobe.clamp(1, self.nlist);

        // Nearest `nprobe` coarse cells to the query (brute force over nlist).
        let mut cell_dist: Vec<(f32, usize)> = (0..self.nlist)
            .map(|c| {
                let base = c * dim;
                let d: f32 = (0..dim)
                    .map(|j| {
                        let e = query[j] - self.coarse[base + j];
                        e * e
                    })
                    .sum();
                (d, c)
            })
            .collect();
        let take = nprobe.min(self.nlist);
        if take < cell_dist.len() {
            cell_dist.select_nth_unstable_by(take - 1, |a, b| {
                a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)
            });
            cell_dist.truncate(take);
        }
        // Ascending cell distance → deterministic slot order.
        cell_dist.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let num_probed = cell_dist.len();
        // Total candidates across the probed cells — reserve exactly, so the
        // gather below is a handful of bulk copies (no Vec regrowth). This is the
        // dominant host cost per query, so preallocating matters (especially in
        // unoptimized builds).
        let total_cand: usize = cell_dist
            .iter()
            .map(|&(_, cell)| self.list_rows[cell].len())
            .sum();
        let mut rows: Vec<u32> = Vec::with_capacity(total_cand);
        let mut cand_slot: Vec<u32> = Vec::with_capacity(total_cand);
        let mut cell_offsets = vec![0u32; num_probed];
        let mut cell_counts = vec![0u32; num_probed];

        match self.refine {
            Refine::Pq { .. } => {
                let m = self.m;
                let mut tables = vec![0.0f32; num_probed * m * PQ_KSUB];
                let mut codes: Vec<u32> = Vec::with_capacity(total_cand * m);
                for (slot, &(_, cell)) in cell_dist.iter().enumerate() {
                    // Per-cell ADC table from the query residual qr = q − μ_cell.
                    self.build_adc_table(query, cell, &mut tables[slot * m * PQ_KSUB..]);
                    let cell_rows = &self.list_rows[cell];
                    let count = cell_rows.len();
                    cell_offsets[slot] = rows.len() as u32;
                    cell_counts[slot] = count as u32;
                    // Bulk-append this cell's rows, slot tags, and codes.
                    rows.extend_from_slice(cell_rows);
                    cand_slot.resize(cand_slot.len() + count, slot as u32);
                    codes.extend(self.list_codes[cell].iter().map(|&b| b as u32));
                }
                QueryPlan {
                    num_probed,
                    rows,
                    cand_slot,
                    cell_offsets,
                    cell_counts,
                    data: ScanData::Pq { tables, codes, m },
                }
            }
            Refine::Flat => {
                let mut qresid = vec![0.0f32; num_probed * dim];
                let mut resid: Vec<f32> = Vec::with_capacity(total_cand * dim);
                for (slot, &(_, cell)) in cell_dist.iter().enumerate() {
                    let cbase = cell * dim;
                    let qbase = slot * dim;
                    for d in 0..dim {
                        qresid[qbase + d] = query[d] - self.coarse[cbase + d];
                    }
                    let cell_rows = &self.list_rows[cell];
                    let count = cell_rows.len();
                    cell_offsets[slot] = rows.len() as u32;
                    cell_counts[slot] = count as u32;
                    // Bulk-append this cell's rows, slot tags, and residual block.
                    rows.extend_from_slice(cell_rows);
                    cand_slot.resize(cand_slot.len() + count, slot as u32);
                    resid.extend_from_slice(&self.list_resid[cell]);
                }
                QueryPlan {
                    num_probed,
                    rows,
                    cand_slot,
                    cell_offsets,
                    cell_counts,
                    data: ScanData::Flat { qresid, resid, dim },
                }
            }
        }
    }

    /// Fill an `m * PQ_KSUB` ADC table for `cell`: entry `[s*256 + c]` is the
    /// squared L2 distance between the query's subspace-`s` residual and PQ
    /// codebook centroid `c`. Residual math is relative to `μ_cell`.
    fn build_adc_table(&self, query: &[f32], cell: usize, table: &mut [f32]) {
        let dim = self.dim;
        let m = self.m;
        let dsub = self.dsub;
        let cbase = cell * dim;
        for s in 0..m {
            let cb_base = s * PQ_KSUB * dsub;
            let off = s * dsub;
            // Query residual for this subspace: qr_d = query_d − μ_cell_d.
            for c in 0..PQ_KSUB {
                let cent = &self.codebooks[cb_base + c * dsub..cb_base + (c + 1) * dsub];
                let mut acc = 0.0f32;
                for d in 0..dsub {
                    // Query residual for this subspace component: q_d − μ_cell_d.
                    let qr = query[off + d] - self.coarse[cbase + off + d];
                    let diff = qr - cent[d];
                    acc += diff * diff;
                }
                table[s * PQ_KSUB + c] = acc;
            }
        }
    }

    /// CPU reference `k`-NN over `nprobe` cells. Deterministic; the correctness
    /// baseline the GPU path is checked against.
    pub fn search_cpu(&self, query: &[f32], k: usize, nprobe: usize) -> Vec<Neighbor> {
        if query.len() != self.dim || self.n == 0 || k == 0 {
            return Vec::new();
        }
        let plan = self.plan(query, nprobe);
        let dist = plan.cpu_scan();
        self.topk_candidates(&plan.rows, &dist, k)
    }

    /// The attribute payload of stored row `row`.
    pub fn payload(&self, row: u32) -> &Payload {
        &self.payloads[row as usize]
    }

    /// CPU reference **filtered** `k`-NN: scan the probed cells, then keep only
    /// candidates whose payload matches `filter` and return the top
    /// `min(k, #matching-candidates)`. With `Refine::Flat` + `nprobe == nlist`
    /// this scans every cell exactly, so it reproduces the filtered flat oracle.
    pub fn search_cpu_filtered(
        &self,
        query: &[f32],
        k: usize,
        nprobe: usize,
        filter: &Filter,
    ) -> Vec<Neighbor> {
        if query.len() != self.dim || self.n == 0 || k == 0 {
            return Vec::new();
        }
        let plan = self.plan(query, nprobe);
        let dist = plan.cpu_scan();
        self.topk_candidates_filtered(&plan.rows, &dist, k, filter)
    }

    /// Assemble best-first [`Neighbor`]s from candidate `rows` + their `dist`
    /// (smaller = better, L2), excluding tombstoned candidates. Shared by the CPU
    /// and GPU search paths. A tombstoned candidate is sunk with the L2 sentinel
    /// (`+∞`) — the SAME mechanism [`Self::topk_candidates_filtered`] uses — so a
    /// deleted row is folded into the keep-set exactly like a filtered-out one, and
    /// the result caps at the live-candidate count.
    pub fn topk_candidates(&self, rows: &[u32], dist: &[f32], k: usize) -> Vec<Neighbor> {
        let mut nlive = 0usize;
        let masked: Vec<f32> = rows
            .iter()
            .zip(dist)
            .map(|(&row, &d)| {
                if self.live[row as usize] {
                    nlive += 1;
                    d
                } else {
                    f32::INFINITY
                }
            })
            .collect();
        self.topk_candidates_raw(rows, &masked, k.min(nlive))
    }

    /// Filtered variant of [`Self::topk_candidates`]: keep only candidates that are
    /// LIVE and match `filter` (live AND filter), sinking the rest with the L2
    /// sentinel (`+∞`), then take the top `min(k, #kept)` — so tombstoned and
    /// non-matching candidates are both excluded, and the result length is the kept
    /// count when fewer than `k` survive. Shared by the CPU reference and GPU
    /// filtered search.
    pub fn topk_candidates_filtered(
        &self,
        rows: &[u32],
        dist: &[f32],
        k: usize,
        filter: &Filter,
    ) -> Vec<Neighbor> {
        // IVF-PQ is L2-only, so smaller is better and `+∞` sinks a candidate.
        let mut nmatch = 0usize;
        let fdist: Vec<f32> = rows
            .iter()
            .zip(dist)
            .map(|(&row, &d)| {
                if self.live[row as usize] && filter.matches(&self.payloads[row as usize]) {
                    nmatch += 1;
                    d
                } else {
                    f32::INFINITY
                }
            })
            .collect();
        self.topk_candidates_raw(rows, &fdist, nmatch.min(k))
    }

    /// The raw best-first selection over candidate `rows` + `dist` with no masking
    /// (callers pre-apply the live/filter sentinels and cap `k`). `+∞`-sentineled
    /// candidates sort last, so a `k` capped at the survivor count never returns one.
    fn topk_candidates_raw(&self, rows: &[u32], dist: &[f32], k: usize) -> Vec<Neighbor> {
        let num = rows.len();
        let want = k.min(num);
        if want == 0 {
            return Vec::new();
        }
        let better = |a: usize, b: usize| {
            dist[a]
                .partial_cmp(&dist[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        };
        let mut idx: Vec<usize> = (0..num).collect();
        if want < num {
            idx.select_nth_unstable_by(want - 1, |&a, &b| better(a, b));
            idx.truncate(want);
        }
        idx.sort_by(|&a, &b| better(a, b));
        idx.into_iter()
            .map(|i| {
                let row = rows[i];
                Neighbor {
                    row,
                    external_id: self.external_ids[row as usize].clone(),
                    score: dist[i],
                }
            })
            .collect()
    }
}

/// The [`VectorIndex`] contract uses a default `nprobe = nlist` (probe every
/// cell — the most accurate setting). Call [`IvfPqIndex::search_cpu`] (or the GPU
/// path) directly to control `nprobe`. Named `search_cpu`/plan+GPU rather than an
/// inherent `search_knn/3` so this 2-arg trait method has no arity clash.
impl VectorIndex for IvfPqIndex {
    fn search_knn(&self, query: &[f32], k: usize) -> Vec<Neighbor> {
        self.search_cpu(query, k, self.nlist)
    }

    fn num_vectors(&self) -> usize {
        self.n
    }

    fn row_payload(&self, row: u32) -> &Payload {
        &self.payloads[row as usize]
    }

    fn search_knn_filtered(&self, query: &[f32], k: usize, filter: &Filter) -> Vec<Neighbor> {
        self.search_cpu_filtered(query, k, self.nlist, filter)
    }
}

/// Encode one residual subvector to its nearest PQ codebook centroid index in
/// subspace `s` (`0..256`).
fn encode_subvector(sub: &[f32], codebooks: &[f32], s: usize, dsub: usize) -> u8 {
    let cb_base = s * PQ_KSUB * dsub;
    let mut best = 0usize;
    let mut best_d = f32::INFINITY;
    for c in 0..PQ_KSUB {
        let cent = &codebooks[cb_base + c * dsub..cb_base + (c + 1) * dsub];
        let mut acc = 0.0f32;
        for d in 0..dsub {
            let diff = sub[d] - cent[d];
            acc += diff * diff;
        }
        if acc < best_d {
            best_d = acc;
            best = c;
        }
    }
    best as u8
}

/// Squared-L2 nearest centroid index for `row` among `k` centroids.
fn nearest_centroid(row: &[f32], centroids: &[f32], k: usize, dim: usize) -> usize {
    let mut best = 0usize;
    let mut best_d = f32::INFINITY;
    for c in 0..k {
        let base = c * dim;
        let mut acc = 0.0f32;
        for d in 0..dim {
            let diff = row[d] - centroids[base + d];
            acc += diff * diff;
        }
        if acc < best_d {
            best_d = acc;
            best = c;
        }
    }
    best
}

/// Deterministic Lloyd k-means over `n` `dim`-vectors (`data` is `n*dim`
/// row-major) → `k` centroids (`k*dim`, row-major). k-means++ init seeded from
/// the LCG, `iters` Lloyd refinements, empty clusters reseeded to the farthest
/// point. No entropy — the same `(data, k, iters, seed)` is byte-reproducible.
#[allow(clippy::needless_range_loop)] // dense row-major math over parallel arrays
fn kmeans(data: &[f32], n: usize, dim: usize, k: usize, iters: usize, seed: u64) -> Vec<f32> {
    use crate::dataset::Lcg;
    let k = k.max(1).min(n.max(1));
    let mut rng = Lcg::new(seed);

    // --- k-means++ initialization -------------------------------------------
    let mut centroids = vec![0.0f32; k * dim];
    if n == 0 {
        return centroids;
    }
    // First centroid: a uniformly-picked point.
    let first = ((rng.next_unit() * n as f32) as usize).min(n - 1);
    centroids[0..dim].copy_from_slice(&data[first * dim..(first + 1) * dim]);
    // Remaining: D²-weighted sampling (k-means++).
    let mut dist2 = vec![f32::INFINITY; n];
    for c in 1..k {
        // Update each point's distance to the nearest chosen centroid.
        let prev = (c - 1) * dim;
        for i in 0..n {
            let mut acc = 0.0f32;
            for d in 0..dim {
                let diff = data[i * dim + d] - centroids[prev + d];
                acc += diff * diff;
            }
            if acc < dist2[i] {
                dist2[i] = acc;
            }
        }
        let total: f32 = dist2.iter().sum();
        let pick = if total > 0.0 {
            // Sample proportional to D².
            let target = rng.next_unit() * total;
            let mut cum = 0.0f32;
            let mut chosen = n - 1;
            for (i, &d2) in dist2.iter().enumerate() {
                cum += d2;
                if cum >= target {
                    chosen = i;
                    break;
                }
            }
            chosen
        } else {
            // All points coincide with chosen centroids: pick any.
            ((rng.next_unit() * n as f32) as usize).min(n - 1)
        };
        centroids[c * dim..(c + 1) * dim].copy_from_slice(&data[pick * dim..(pick + 1) * dim]);
    }

    // --- Lloyd iterations ----------------------------------------------------
    let mut assign = vec![0usize; n];
    let mut sums = vec![0.0f32; k * dim];
    let mut counts = vec![0u32; k];
    for _ in 0..iters {
        // Assign.
        for i in 0..n {
            assign[i] = nearest_centroid(&data[i * dim..(i + 1) * dim], &centroids, k, dim);
        }
        // Accumulate means.
        sums.iter_mut().for_each(|x| *x = 0.0);
        counts.iter_mut().for_each(|x| *x = 0);
        for i in 0..n {
            let a = assign[i];
            counts[a] += 1;
            let sbase = a * dim;
            let dbase = i * dim;
            for d in 0..dim {
                sums[sbase + d] += data[dbase + d];
            }
        }
        // Update centroids; reseed empties to the point farthest from its own
        // centroid (a deterministic split of the largest cluster's tail).
        for c in 0..k {
            if counts[c] > 0 {
                let inv = 1.0 / counts[c] as f32;
                for d in 0..dim {
                    centroids[c * dim + d] = sums[c * dim + d] * inv;
                }
            } else {
                let far = farthest_point(data, n, dim, &centroids, &assign);
                centroids[c * dim..(c + 1) * dim]
                    .copy_from_slice(&data[far * dim..(far + 1) * dim]);
            }
        }
    }
    centroids
}

/// Index of the point with the largest squared distance to its assigned
/// centroid — a deterministic seed for an empty cluster.
fn farthest_point(
    data: &[f32],
    n: usize,
    dim: usize,
    centroids: &[f32],
    assign: &[usize],
) -> usize {
    let mut best = 0usize;
    let mut best_d = -1.0f32;
    for i in 0..n {
        let base = assign[i] * dim;
        let mut acc = 0.0f32;
        for d in 0..dim {
            let diff = data[i * dim + d] - centroids[base + d];
            acc += diff * diff;
        }
        if acc > best_d {
            best_d = acc;
            best = i;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::clustered_collection;
    use crate::index::cpu_flat::CpuFlatIndex;
    use std::collections::HashSet;

    fn recall_at(a: &[Neighbor], truth: &HashSet<u32>) -> f64 {
        if a.is_empty() {
            return 1.0;
        }
        let hit = a.iter().filter(|nb| truth.contains(&nb.row)).count();
        hit as f64 / a.len() as f64
    }

    #[test]
    fn flat_full_probe_is_exact() {
        let dim = 32;
        let c = clustered_collection("t", 1500, dim, Metric::L2, 12, 0.03, 3);
        let cfg = IvfPqConfig {
            nlist: 32,
            kmeans_iters: 12,
            nbits: 8,
            refine: Refine::Flat,
            train_sample: 0,
            seed: 7,
        };
        let idx = IvfPqIndex::train(&c, cfg).unwrap();
        let oracle = CpuFlatIndex::new(&c);
        let queries = crate::dataset::clustered_queries(6, dim, 12, 0.03, 3);
        for q in &queries {
            let truth: HashSet<u32> =
                oracle.search_knn(q, 10).iter().map(|n| n.row).collect();
            let got = idx.search_cpu(q, 10, idx.nlist());
            assert_eq!(recall_at(&got, &truth), 1.0, "full-probe Flat must be exact");
        }
    }

    #[test]
    fn pq_full_probe_recall_is_high() {
        let dim = 32;
        let c = clustered_collection("t", 2000, dim, Metric::L2, 16, 0.03, 4);
        let cfg = IvfPqConfig {
            nlist: 32,
            kmeans_iters: 12,
            nbits: 8,
            refine: Refine::Pq { m: 8 },
            train_sample: 0,
            seed: 9,
        };
        let idx = IvfPqIndex::train(&c, cfg).unwrap();
        let oracle = CpuFlatIndex::new(&c);
        let queries = crate::dataset::clustered_queries(6, dim, 16, 0.03, 4);
        let mut sum = 0.0;
        for q in &queries {
            let truth: HashSet<u32> =
                oracle.search_knn(q, 10).iter().map(|n| n.row).collect();
            let got = idx.search_cpu(q, 10, idx.nlist());
            sum += recall_at(&got, &truth);
        }
        let mean = sum / queries.len() as f64;
        assert!(mean >= 0.7, "PQ full-probe recall {mean} should clear 0.70");
    }

    #[test]
    fn rejects_non_l2_and_bad_m() {
        let c = clustered_collection("t", 100, 30, Metric::L2, 4, 0.03, 1);
        // m must divide dim (30 % 8 != 0).
        let bad = IvfPqConfig {
            nlist: 8,
            kmeans_iters: 4,
            nbits: 8,
            refine: Refine::Pq { m: 8 },
            train_sample: 0,
            seed: 1,
        };
        assert!(IvfPqIndex::train(&c, bad).is_err());
        // Non-L2 rejected.
        let cos = clustered_collection("t", 100, 32, Metric::Cosine, 4, 0.03, 1);
        let cfg = IvfPqConfig {
            nlist: 8,
            kmeans_iters: 4,
            nbits: 8,
            refine: Refine::Flat,
            train_sample: 0,
            seed: 1,
        };
        assert!(IvfPqIndex::train(&cos, cfg).is_err());
    }
}
