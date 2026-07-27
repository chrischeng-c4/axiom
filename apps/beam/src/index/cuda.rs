//! Native **CUDA** flat vector search — NVIDIA the CUDA-native way.
//!
//! [`CudaFlatIndex`] is the exact brute-force scan of [`CpuFlatIndex`] /
//! [`GpuFlatIndex`], but the per-row distances are computed by a CUDA C kernel
//! compiled at runtime with NVRTC and launched through the NVIDIA driver API via
//! the [`cudarc`] crate. It joins `CpuFlatIndex` / `GpuFlatIndex` /
//! `IvfPqIndex` / `HnswIndex` behind the shared [`VectorIndex`] contract, so its
//! results are directly comparable to the CPU oracle.
//!
//! [`CpuFlatIndex`]: crate::index::cpu_flat::CpuFlatIndex
//! [`GpuFlatIndex`]: crate::gpu::GpuFlatIndex
//!
//! ## The wgpu backend vs this CUDA backend
//!
//! beam's default GPU engine ([`crate::gpu`]) is written once in WGSL on `wgpu`
//! and drives NVIDIA GPUs **through Vulkan** — real NVIDIA support, portable, no
//! CUDA. This module is the *other* NVIDIA path: the native **CUDA driver API**
//! (cuVS/Faiss-GPU territory), behind the optional `cuda` cargo feature. It is
//! NOT in the default build; enable it with `--features cuda`.
//!
//! ## Build vs runtime (why this compiles on a Mac with no NVIDIA GPU)
//!
//! `cudarc` is configured with `dynamic-loading` + `nvrtc`, so **nothing about
//! CUDA is needed at build time**:
//!
//! - `dynamic-loading` `dlopen`s `libcuda` / `libnvrtc` lazily at RUNTIME (no
//!   link against a CUDA toolkit), so `cargo check --features cuda` compiles on
//!   any host — including `aarch64-apple-darwin`.
//! - `nvrtc` compiles the embedded kernel string ([`KERNEL_SRC`]) to PTX at
//!   RUNTIME on the NVIDIA host, so there is no offline `nvcc`/`.cu` build step.
//!
//! It therefore **compiles on this Mac dev environment but only RUNS on an
//! NVIDIA GPU + driver**. With no driver present, [`CudaContext::new`] returns an
//! `Err`, so [`CudaFlatIndex::new`] fails gracefully at runtime (the tests skip);
//! see `benchmark/dual-platform.md`. This path is **compile-verified only** here
//! — it has NOT been run on a GPU in this environment.
//!
//! ## Design (mirrors [`GpuFlatIndex`])
//!
//! - [`CudaFlatIndex::new`] opens a [`CudaContext`], uploads the collection's
//!   whole row-major corpus to device memory once (`memcpy_stod`, htod), and
//!   compiles + loads the distance kernel.
//! - Each `search_knn` uploads the (host-normalized-for-cosine) query, launches
//!   the kernel (one thread per physical DB row) to write the `n` per-row
//!   distances, reads them back (`memcpy_dtov`, dtoh), masks tombstoned rows with
//!   the metric sentinel, and runs the shared host [`topk`].
//! - Scores use the SAME convention as the CPU oracle
//!   ([`crate::collection::Metric::code`]): L2 is the sum of squared differences
//!   (smaller first), Dot/Cosine are a dot product (larger first). Cosine rows are
//!   already unit-normalized on insert, so the query is normalized on the host and
//!   the kernel runs the plain dot path.

use std::sync::Arc;

use cudarc::driver::{
    CudaContext, CudaFunction, CudaSlice, CudaStream, LaunchConfig, PushKernelArg,
};
use cudarc::nvrtc::compile_ptx;

use crate::collection::{l2_normalize, Collection, Metric};
use crate::index::{topk, Neighbor, VectorIndex};
use crate::payload::Payload;

/// Threads per block for the distance-kernel launch. One thread scores one
/// physical DB row, so the grid is `ceil(n / BLOCK_SIZE)` blocks.
const BLOCK_SIZE: u32 = 256;

/// The embedded CUDA C distance kernel, compiled to PTX at runtime by NVRTC (see
/// the module docs). `extern "C"` keeps the symbol name unmangled so
/// [`CudaModule::load_function`](cudarc::driver::CudaModule::load_function) finds
/// `flat_distance`. One thread computes one `(query, row)` distance under the
/// shared metric convention — `metric == 0` is squared-L2 (`Σ(q−d)²`), anything
/// else (Dot / Cosine) is the dot product (`Σ q·d`); the host normalizes the
/// cosine query and stores unit rows, so Cosine reuses the dot path.
const KERNEL_SRC: &str = r#"
extern "C" __global__ void flat_distance(
    const float* db,
    const float* query,
    float* out,
    const int n,
    const int dim,
    const int metric
) {
    int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= n) {
        return;
    }
    const float* d = db + (size_t)row * (size_t)dim;
    float acc = 0.0f;
    if (metric == 0) {
        for (int j = 0; j < dim; ++j) {
            float diff = query[j] - d[j];
            acc += diff * diff;
        }
    } else {
        for (int j = 0; j < dim; ++j) {
            acc += query[j] * d[j];
        }
    }
    out[row] = acc;
}
"#;

/// Native-CUDA flat (brute-force) index. Owns the CUDA context/stream, the
/// uploaded DB device buffer, and the compiled distance kernel; `search_knn` is
/// per-query upload → launch → readback → host top-k.
///
/// Self-contained after construction (snapshots the collection's ids / payloads /
/// live bits), like [`crate::gpu::GpuFlatIndex`].
pub struct CudaFlatIndex {
    /// The CUDA context (device 0). Kept alive for the buffer/stream lifetimes.
    _ctx: Arc<CudaContext>,
    /// The stream every upload / launch / readback is scheduled on.
    stream: Arc<CudaStream>,
    /// The compiled `flat_distance` kernel (holds its `Arc<CudaModule>` alive).
    func: CudaFunction,
    /// The whole corpus (`n * dim` f32, live + tombstoned) uploaded once to the
    /// device as a read-only buffer.
    db: CudaSlice<f32>,
    dim: usize,
    /// Physical row count (live + tombstoned) = the number of rows uploaded to
    /// `db` and the kernel launch width.
    n: usize,
    metric: Metric,
    external_ids: Vec<String>,
    /// Row-aligned attribute payloads (snapshot), read by filtered search.
    payloads: Vec<Payload>,
    /// Per physical row liveness (`true` = live). A tombstoned row is masked out
    /// of top-k on the host with the metric sentinel — the SAME exclusion path
    /// [`crate::index::cpu_flat::CpuFlatIndex`] uses.
    live: Vec<bool>,
    /// Cached live-row count (`== live.iter().filter(|&&b| b).count()`).
    n_live: usize,
}

impl CudaFlatIndex {
    /// Build the index: open the CUDA context on device 0, upload `collection`'s
    /// vectors to device memory once, and compile + load the distance kernel via
    /// NVRTC.
    ///
    /// Returns `Err` if no NVIDIA driver / device is reachable
    /// ([`CudaContext::new`] fails) or if any upload / kernel compilation fails —
    /// so on a host with no CUDA driver (e.g. this Mac) it errors gracefully at
    /// runtime and callers can skip (see the tests below).
    pub fn new(collection: &Collection) -> anyhow::Result<Self> {
        // Device 0. Returns Err with no NVIDIA driver present — the graceful-skip
        // signal on a CUDA-less host (e.g. this Mac dev environment).
        let ctx = CudaContext::new(0)
            .map_err(|e| anyhow::anyhow!("CUDA context (device 0) unavailable: {e:?}"))?;
        let stream = ctx.default_stream();

        let dim = collection.dim();
        // Physical rows (live + tombstoned): the whole `data` buffer is uploaded
        // and the live-mask excludes tombstones at scoring time.
        let n = collection.capacity();
        let metric = collection.metric();

        // Upload the whole corpus once (host to device).
        let db = stream
            .memcpy_stod(collection.data())
            .map_err(|e| anyhow::anyhow!("upload DB vectors to device: {e:?}"))?;

        // Compile the embedded kernel to PTX at runtime (NVRTC), load the module,
        // and resolve the `flat_distance` function.
        let ptx = compile_ptx(KERNEL_SRC)
            .map_err(|e| anyhow::anyhow!("NVRTC compile of distance kernel: {e:?}"))?;
        let module = ctx
            .load_module(ptx)
            .map_err(|e| anyhow::anyhow!("load CUDA module: {e:?}"))?;
        let func = module
            .load_function("flat_distance")
            .map_err(|e| anyhow::anyhow!("load CUDA kernel `flat_distance`: {e:?}"))?;

        Ok(Self {
            _ctx: ctx,
            stream,
            func,
            db,
            dim,
            n,
            metric,
            external_ids: collection.external_ids().to_vec(),
            payloads: collection.payloads().to_vec(),
            live: collection.live().to_vec(),
            n_live: collection.len(),
        })
    }

    /// Number of **live** vectors (tombstoned rows excluded).
    pub fn len(&self) -> usize {
        self.n_live
    }

    /// Whether the index has zero live vectors.
    pub fn is_empty(&self) -> bool {
        self.n_live == 0
    }

    /// Physical row count (live + tombstoned) uploaded to the device.
    pub fn capacity(&self) -> usize {
        self.n
    }

    /// The vector dimension.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// The collection metric this index was built under.
    pub fn metric(&self) -> Metric {
        self.metric
    }

    /// Launch the kernel over every physical row and return the raw `n` per-row
    /// distances (tombstones NOT yet masked — the caller applies the sentinel).
    /// Cosine normalizes the query on the host so the (unit) DB rows give a true
    /// cosine similarity; L2/Dot upload the query as-is.
    fn compute_distances(&self, query: &[f32]) -> anyhow::Result<Vec<f32>> {
        let q = match self.metric {
            Metric::Cosine => l2_normalize(query),
            _ => query.to_vec(),
        };

        // Upload the query and allocate the output distance buffer on the device.
        let d_query = self
            .stream
            .memcpy_stod(&q)
            .map_err(|e| anyhow::anyhow!("upload query to device: {e:?}"))?;
        let mut d_out = self
            .stream
            .alloc_zeros::<f32>(self.n)
            .map_err(|e| anyhow::anyhow!("allocate device output buffer: {e:?}"))?;

        // Kernel scalar args (bound to locals so their addresses outlive the launch).
        let n_i = self.n as i32;
        let dim_i = self.dim as i32;
        let metric_i = self.metric.code() as i32;

        // One thread per DB row: ceil(n / BLOCK_SIZE) blocks.
        let grid_x = (self.n as u32).div_ceil(BLOCK_SIZE).max(1);
        let cfg = LaunchConfig {
            grid_dim: (grid_x, 1, 1),
            block_dim: (BLOCK_SIZE, 1, 1),
            shared_mem_bytes: 0,
        };

        // SAFETY: the arg count/types/order match `flat_distance`'s signature
        // (db, query, out, n, dim, metric) and every buffer is sized `>= n`.
        unsafe {
            self.stream
                .launch_builder(&self.func)
                .arg(&self.db)
                .arg(&d_query)
                .arg(&mut d_out)
                .arg(&n_i)
                .arg(&dim_i)
                .arg(&metric_i)
                .launch(cfg)
                .map_err(|e| anyhow::anyhow!("launch distance kernel: {e:?}"))?;
        }

        // Ensure the launch has completed before reading the distances back.
        self.stream
            .synchronize()
            .map_err(|e| anyhow::anyhow!("synchronize stream: {e:?}"))?;
        let scores = self
            .stream
            .memcpy_dtov(&d_out)
            .map_err(|e| anyhow::anyhow!("read distances back to host: {e:?}"))?;
        Ok(scores)
    }
}

impl VectorIndex for CudaFlatIndex {
    /// Exact top-`k` nearest neighbors to `query`, best-first (see the module
    /// ordering contract). Scores every physical row on the GPU, masks tombstoned
    /// rows with the metric sentinel, and runs the shared host top-k capped at the
    /// live count — so results equal the [`CpuFlatIndex`](crate::index::cpu_flat::CpuFlatIndex)
    /// oracle (exact flat). Returns empty on a dimension mismatch, `k == 0`, an
    /// empty/all-tombstoned index, or a device error.
    fn search_knn(&self, query: &[f32], k: usize) -> Vec<Neighbor> {
        if query.len() != self.dim || self.n == 0 || self.n_live == 0 || k == 0 {
            return Vec::new();
        }
        let Ok(mut scores) = self.compute_distances(query) else {
            // Device/launch error (e.g. no driver) → empty, like a GPU-less skip.
            return Vec::new();
        };
        // Fold in the live mask: a tombstoned row gets the metric's worst score,
        // so it sorts last and never enters the top-k (mirrors CpuFlatIndex).
        let sentinel = self.metric.worst_score();
        for (i, s) in scores.iter_mut().enumerate() {
            if !self.live[i] {
                *s = sentinel;
            }
        }
        // Cap at the live count so sentinels are never selected — top `min(k, n_live)`.
        topk(&scores, self.metric, k.min(self.n_live), &self.external_ids)
    }

    fn num_vectors(&self) -> usize {
        // Physical row count: rows are addressed 0..capacity and `row_payload`
        // indexes physical rows, so this is the correct over-fetch ceiling for the
        // default `search_knn_filtered`.
        self.n
    }

    fn row_payload(&self, row: u32) -> &Payload {
        &self.payloads[row as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Collection;
    use crate::index::cpu_flat::CpuFlatIndex;

    /// Build a small deterministic dataset under `metric` for the parity check.
    fn small_collection(metric: Metric) -> Collection {
        let mut c = Collection::new("cuda-test", 4, metric);
        // A handful of fixed rows so the nearest-neighbor answer is unambiguous.
        c.add("a", &[1.0, 0.0, 0.0, 0.0]).unwrap();
        c.add("b", &[0.0, 1.0, 0.0, 0.0]).unwrap();
        c.add("c", &[0.9, 0.1, 0.0, 0.0]).unwrap();
        c.add("d", &[0.0, 0.0, 1.0, 0.0]).unwrap();
        c.add("e", &[0.2, 0.2, 0.2, 0.2]).unwrap();
        c
    }

    /// Runtime-skip parity test: on a host with no NVIDIA driver (this Mac)
    /// [`CudaFlatIndex::new`] errors and we skip, exactly like the wgpu tests'
    /// graceful GPU-less skip. On an NVIDIA host it runs and asserts the native
    /// CUDA backend matches the exact CPU oracle (row set identical, scores within
    /// 1e-3). This test is COMPILE-verified here; it is NOT run on a GPU in this
    /// environment.
    #[test]
    fn cuda_matches_cpu_oracle_or_skips() {
        for metric in [Metric::L2, Metric::Dot, Metric::Cosine] {
            let coll = small_collection(metric);
            let Ok(idx) = CudaFlatIndex::new(&coll) else {
                eprintln!("no CUDA device; skipping cuda_matches_cpu_oracle_or_skips ({metric:?})");
                return;
            };
            let cpu = CpuFlatIndex::new(&coll);
            let query = [0.85, 0.15, 0.0, 0.0];
            let k = 3;
            let cuda_res = idx.search_knn(&query, k);
            let cpu_res = cpu.search_knn(&query, k);

            assert_eq!(cuda_res.len(), cpu_res.len(), "{metric:?}: result length");
            let cuda_rows: Vec<u32> = cuda_res.iter().map(|n| n.row).collect();
            let cpu_rows: Vec<u32> = cpu_res.iter().map(|n| n.row).collect();
            assert_eq!(cuda_rows, cpu_rows, "{metric:?}: top-k row order");
            for (cu, cp) in cuda_res.iter().zip(cpu_res.iter()) {
                assert!(
                    (cu.score - cp.score).abs() <= 1e-3,
                    "{metric:?}: score {} vs oracle {} exceeds 1e-3",
                    cu.score,
                    cp.score
                );
            }
        }
    }
}
