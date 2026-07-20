<!-- HANDWRITE-BEGIN gap="missing-generator:logic:be2e75c4" tracker="pending-tracker" reason="scaffold for apps/beam/benchmark/competitor-performance-baseline.md — fill in by hand and update tracker when codegen is ready" -->
# Beam Competitor Performance Baseline

Pinned same-machine performance evidence for the `competitor-performance`
capability (goal 2: "beat competitors on performance"). This is the honest,
head-to-head number: **beam-GPU (Metal via wgpu) vs a real CPU competitor on the
same Mac**, exact L2 kNN, identical dataset shape.

**Policy** (per `../README.md` and `competitor-feature-matrix.md`): this baseline
is **pinned**. Do **not** re-run it unless the comparison scope changes (new
competitor, new hardware class, a beam query-path change that could move the
numbers) or a human explicitly asks. Numbers are date-agnostic — they describe
fixed params + versions, not a wall-clock run.

## What "beating a competitor" means on Apple Silicon

There is **no GPU vector system on a Mac** — faiss-GPU, NVIDIA cuVS/CAGRA, and
Milvus-GPU are all CUDA/NVIDIA-only and do not run on Metal. So on Apple Silicon
the **strongest vector search a user can actually run is faiss-CPU** (BLAS on the
AMX/Accelerate matrix units — genuinely fast, *not* a weak fallback). That makes
faiss-CPU the **legitimate competitor on this platform**: beating it means beam is
the **fastest vector search available on a Mac**. This baseline is exactly that
head-to-head, and beam-GPU wins it (batched, n ≥ 100k). The orthogonal question —
beam vs GPU competitors (cuVS/faiss-GPU) on the *same NVIDIA GPU* — is out of scope
here: it needs NVIDIA hardware; beam's wgpu code already runs there via Vulkan but
is unbenchmarked against those systems.

## Setup

| Item | Value |
|---|---|
| Machine | Apple **M1 Max**, 10 CPU cores, Metal (wgpu adapter reports `Apple M1 Max (Metal)`) |
| beam | `beam bench` (release build), GPU flat + IVF-flat backends on wgpu/Metal |
| Competitor | **faiss-cpu 1.14.3** (Meta), the gold-standard ANN library, CPU-only on this Mac (no CUDA) |
| Competitor threads | faiss default = **10 OMP threads** (all cores); BLAS-backed flat scan |
| numpy | 2.5.0 (dataset generation) · Python 3.12 |
| Dataset | `dim=128`, components uniform in `[-1, 1)`, seeded (deterministic) |
| Query set | `queries=200`, `k=10`, same uniform distribution, separate seed |
| Sizes | `n ∈ {10_000, 100_000, 1_000_000}` |
| Metric | L2 |
| IVF knobs | `nlist=256`, `nprobe=16`; IVF-PQ `m=16`, `nbits=8` (beam & faiss matched) |

**Why faiss (not a fallback):** `uv`/`pip install faiss-cpu numpy` succeeded on
the first attempt in a throwaway `/tmp` venv, so the strongest available CPU
competitor was used directly. No fallback to hnswlib or numpy-brute-force was
needed (the reproducer still auto-falls-back to numpy if faiss is ever absent).

**Measurement methodology (fair, same machine):** `beam bench` runs queries
**one at a time** on the GPU (per-query dispatch + blocking readback, no query
batching). The competitor is measured the **same way** — one `index.search`
per query in a loop — so `avg query ms` is directly comparable single-query
latency. One warmup query is discarded on each side. `q/s = 1000 / query_ms`
(single-stream). faiss's *batched* throughput (all 200 queries in one call) is
reported separately as faiss's best case; beam now has a matching batched path,
measured head-to-head in the **[Batched query](#batched-query-the-throughput-lever-now-measured)**
section below (`beam bench --index flat --batch 200`).

## Results

Rows = system × n. `query ms` = avg single-query latency; `q/s` = single-stream
throughput (`1000/ms`); `recall@10` vs the exact ground truth.

| System | n | build (s) | query ms | q/s (single) | recall@10 |
|---|---:|---:|---:|---:|---:|
| competitor-CPU-flat (faiss `IndexFlatL2`) | 10_000 | 0.000 | **0.176** | 5693 | 1.000 |
| competitor-CPU-flat (faiss `IndexFlatL2`) | 100_000 | 0.002 | **1.010** | 990 | 1.000 |
| competitor-CPU-flat (faiss `IndexFlatL2`) | 1_000_000 | 0.015 | **11.440** | 87 | 1.000 |
| competitor-CPU-IVFPQ (faiss, nlist256 m16 nprobe16) | 10_000 | 0.831 | 0.036 | 27524 | 0.196 |
| competitor-CPU-IVFPQ (faiss, nlist256 m16 nprobe16) | 100_000 | 1.627 | 0.071 | 14159 | 0.133 |
| competitor-CPU-IVFPQ (faiss, nlist256 m16 nprobe16) | 1_000_000 | 6.614 | 0.384 | 2606 | 0.101 |
| **beam-GPU-flat** (Metal, uniform) | 10_000 | n/a¹ | 1.769 | 565 | 1.000 |
| **beam-GPU-flat** (Metal, uniform) | 100_000 | n/a¹ | 5.035 | 199 | 1.000 |
| **beam-GPU-flat** (Metal, uniform) | 1_000_000 | n/a¹ | 25.221 | 40 | 1.000 |
| **beam-GPU-ivfflat** (Metal, clustered) | 10_000 | n/a¹ | 1.746 | 573 | 1.000 |
| **beam-GPU-ivfflat** (Metal, clustered) | 100_000 | n/a¹ | 2.559 | 391 | 1.000 |
| **beam-GPU-ivfflat** (Metal, clustered) | 1_000_000 | n/a¹ | 13.317 | 75 | 1.000 |

¹ `beam bench` does not isolate index-build time (it also builds a CPU oracle
for the recall check), so beam build time is not reported here rather than
reporting a number that mixes in oracle construction. (Measurement gap noted;
`beam bench` src not modified.)

**faiss batched throughput (faiss's optimal mode, all 200 queries in one
call — NOT the head-to-head number, since beam does not batch):** flat =
37174 / 3483 / 234 q/s at n = 10k / 100k / 1M; IVFPQ = 116414 / 55867 / 15149
q/s. beam's serial per-query path cannot match this.

## Honest interpretation

**The clean apples-to-apples comparison is the exact-flat row** (both sides
exact, `recall = 1.000`, identical uniform dataset shape). The IVF rows are
*not* directly comparable: beam-ivfflat uses beam's built-in **clustered**
corpus with **exact** residual refine (recall 1.000, scanning ~8% of vectors),
while faiss-IVFPQ uses the **uniform** corpus with **lossy** PQ compression
(recall ~0.1–0.2 — uniform data is PQ's documented worst case). They are shown
as each engine's ANN datapoint, not a winner/loser pair.

### Where beam does NOT win (state it plainly)

On this M1 Max, **faiss-CPU-flat is faster than beam-GPU-flat at every tested
size**, on both single-query latency and single-stream throughput:

| n | faiss-CPU-flat ms | beam-GPU-flat ms | beam is |
|---:|---:|---:|---|
| 10_000 | 0.176 | 1.769 | **10.1× slower** |
| 100_000 | 1.010 | 5.035 | **5.0× slower** |
| 1_000_000 | 11.440 | 25.221 | **2.2× slower** |

This is a stronger loss than the feature-matrix's optimistic framing predicted
(it expected beam to lose only on *small-n* single-query latency due to GPU
dispatch overhead). Two real causes, measured:

1. **The CPU baseline is not weak on Apple Silicon.** faiss's flat L2 scan is a
   BLAS matrix-multiply; on M1 it saturates the AMX/Accelerate matrix units and
   is extraordinarily fast — even **single-threaded** faiss beats beam-GPU-flat
   (0.103 / 1.027 / 10.500 ms at 10k / 100k / 1M). There is no CPU "fallback
   penalty" to exploit here.
2. **beam pays fixed per-query GPU overhead.** beam-GPU-flat has a ~1.7 ms floor
   at n=10k that is dispatch + blocking readback, not compute, and beam issues
   **one GPU dispatch per query with no batching** — so it cannot amortize
   dispatch the way faiss amortizes a BLAS call.

### Where beam's GPU parallelism does show up (honestly, but not a win yet)

The **gap narrows monotonically with n** — 10.1× → 5.0× → 2.2× — because the
GPU parallel scan scales better than even AMX-accelerated CPU as n grows. beam
also scales sublinearly: beam-GPU-flat goes 1.77 → 5.04 → 25.2 ms for a 100×
data increase, and **beam-GPU-ivfflat stays exact (recall 1.000) while scanning
only ~8% of vectors** (100k in 2.56 ms). Extrapolating the trend, a crossover
would need n well beyond 1M **and/or** a batched GPU query path + elimination of
the per-query readback. Both are now **built and measured** (see
**[Batched query](#batched-query-the-throughput-lever-now-measured)**): the
batched path with **GPU-side per-query top-k** — the top-k is selected on the GPU
and the readback shrinks from the `T × n` distance matrix to just `T × k`
(id, score) pairs — is a large gain over beam's own serial path (up to ~18× at
10k) and narrows the gap to faiss's *batched* throughput to **3.5× / 2.6× / 1.4×**
at 10k / 100k / 1M. Honestly, it still does **not** cross over at any tested n
(closest is **1.4×** at n = 1M), but the readback bottleneck is gone.

### Beam's actual, defensible advantages (not raw same-machine latency)

Per `competitor-feature-matrix.md`, and consistent with these measurements:

- **Availability / portability:** beam runs the search on the **Metal GPU** via
  wgpu; faiss (and every competitor surveyed) has **no GPU path on this Mac**.
  This is a "uses the GPU at all / portable beyond CUDA" claim — it is **not** a
  "faster on this machine" claim for flat search, which this baseline retracts.
- **IVF-PQ memory at scale:** PQ codes are **32× smaller** than full residuals
  (measured in `beam bench`: 488 MB → 15 MB at n=1M/dim128/m16), so beam indexes
  corpora that don't fit as full vectors.
- **Exact ANN pruning:** beam-ivfflat is lossless (recall 1.000) at ~8% scan.

**Bottom-line verdict:** on same-machine raw latency/throughput for exact flat
L2, **beam-GPU does not beat faiss-CPU on M1 Max** at n ≤ 1M — Apple's AMX makes
the CPU competitor too strong and beam's per-query dispatch/readback overhead
too costly. beam's honest goal-2 story here is (a) the *scaling trend* (the gap
shrinks with n), plus (b) the availability + memory wins — not a raw-speed win.
The "faster on this machine" claim in the feature matrix is **not supported for
flat search on this hardware** and is corrected by this pinned baseline.

## Batched query (the throughput lever, now measured)

The pinned rows above are **single-query** — one GPU dispatch per query, so the
~1.7 ms dispatch+readback floor dominates and beam cannot match faiss's *batched*
BLAS call. The fix has two stages, both now built into
[`GpuFlatIndex::search_knn_batch`] (`beam bench --index flat --batch <B>`):

1. **Batch the queries** so the fixed dispatch overhead amortizes (the earlier
   result, superseded below): each tile scored its `T × n` distance sub-matrix in
   one dispatch, read back `T × n` f32, and did exact top-k per query on the CPU.
2. **GPU-side per-query top-k** (the throughput lever, this update): a new WGSL
   kernel (`main_batch_topk`) runs **one workgroup per query** — the workgroup's
   threads cooperatively scan all `n` rows (grid-strided, folding in the live
   mask), each keeps a private register top-k, and a workgroup tree-merge reduces
   them to the query's global top-k. Only **`T × k` (id, score) pairs** are read
   back (a ~`n/k` cut — ~100_000× at n=1M/k=10), and the single-threaded CPU
   selection over the `T × n` matrix is gone. `search_knn_batch` auto-selects this
   path for `k ≤ MAX_TOPK` (=32) and falls back to the stage-1 distance-matrix
   path for larger `k`.

Recall stays **1.000** (same exact distances as the serial scan; the GPU selects
the same rows as the CPU oracle — verified exact in `tests/batched_query.rs`).
Same machine, same dataset shape (`dim=128`, `k=10`, `queries=200`, uniform
seeded), measured with `--batch 200` (all 200 queries in one batched call,
directly comparable to faiss's all-200-in-one-call batched throughput).

**Batched throughput — beam-GPU (GPU-side top-k) vs faiss-CPU, head-to-head
(both batched):**

| n | beam-GPU-topk q/s | faiss-CPU-batched q/s | beam vs faiss batched |
|---:|---:|---:|---|
| 10_000 | ~10_500 | 37_174 | **3.5× slower** |
| 100_000 | ~1_350 | 3_483 | **2.6× slower** |
| 1_000_000 | ~170 | 234 | **1.4× slower** |

**What GPU-side top-k bought (same binary, same methodology):**

| n | beam serial q/s | beam batched (dist-matrix, CPU top-k) q/s | **beam GPU-top-k q/s** | topk vs serial | topk vs dist-matrix |
|---:|---:|---:|---:|---:|---:|
| 10_000 | 571 | ~4_050 | **~10_500** | **18×** | **2.6×** |
| 100_000 | 199 | ~467 | **~1_350** | **6.8×** | **2.9×** |
| 1_000_000 | 37.5 | ~52 | **~170** | **4.5×** | **3.3×** |

(beam-GPU-topk amortized latency: 0.095 / 0.74 / 5.9 ms/query at 10k / 100k / 1M.
Numbers are ±10–15% run-to-run at small n; representative values shown. The
dist-matrix column is the previous pinned batched path, retained for contrast.)

### Honest verdict: GPU-top-k closes most of the gap, but still does NOT beat faiss batched

> **Superseded:** this verdict is for the `main_batch_topk` path. The **tiled
> kernel** ([Tiled distance kernel](#tiled-gemm-style-distance-kernel-beam-now-beats-faiss-batched-at-n--100k))
> now BEATS faiss batched at n ≥ 100k — see that section for the current result.

GPU-side top-k worked as predicted — it removed the `T × n` readback + CPU
selection that was the measured batched bottleneck, giving a **2.6× / 2.9× / 3.3×**
gain over beam's *own* previous batched path and an **18× / 6.8× / 4.5×** gain
over serial. The gap to faiss's batched flat scan narrows to **3.5× / 2.6× /
1.4×** at 10k / 100k / 1M — from the previous **9.2× / 7.5× / 4.5×** — and keeps
the same monotonic *GPU-scales-better-than-AMX-with-n* trend. **At n = 1M beam is
now within 1.4×** (~170 vs 234 q/s), a near-crossover.

But honestly: **beam-GPU-topk still loses to faiss-CPU-batched at every tested
n.** Do not claim a batched throughput win. Two reasons it does not cross over:

1. **faiss batched is a single big BLAS GEMM** — still extraordinarily fast on
   M1's AMX/Accelerate matrix units, and it also amortizes its call across all 200
   queries. beam is chasing a strong, moving target.
2. **beam's bottleneck moved from readback onto GPU compute/occupancy.** With the
   readback gone, the batched path is now (near-)compute-bound: one workgroup per
   query pins each query's `n × dim` scan to a single GPU core, so with 200 queries
   the work spreads over ~32 cores at ~64 threads/query. The remaining ~1.4× at
   1M would come from **more GPU parallelism per query** (split-k: multiple
   workgroups per query scanning row ranges, then a merge) or lower per-row memory
   traffic (fp16 DB / register-blocked scan) — not from further readback cuts.
   That is the recommended next lever; it is out of scope for this exact,
   one-workgroup-per-query kernel, so this section reports the measured result
   honestly rather than a projected crossover.

**Design + correctness (for reproducers):** the kernel is
`src/gpu/flat.wgsl :: main_batch_topk` (workgroup size 64, `MAX_TOPK = 32`
per-thread register top-k, log-step workgroup tree-merge, readback `T × k`
(id, score) pairs). It is exercised in `tests/batched_query.rs`: GPU-top-k ==
CPU oracle for k ∈ {1, 10, 32} on L2 and Dot, == the previous distance-matrix
path, tombstones excluded, and `k > MAX_TOPK` falls back to the distance-matrix
path and stays exact.

[`GpuFlatIndex::search_knn_batch`]: ../src/gpu/mod.rs

### Large-n crossover check (empirical): the gap plateaus, no crossover

> **SUPERSEDED for the batched flat path by the tiled kernel below** — this
> subsection describes the *one-workgroup-per-query* `main_batch_topk` path (kept
> as a fallback). The tiled kernel that follows **does** cross over.

The near-parity at 1M (1.4×) raised the question of whether beam simply crosses
over at larger n. Measured at n = 2M and 4M (beam GPU-top-k batched vs faiss
batched, same M1 Max, batch 200, recall 1.000):

| n | beam-GPU-topk q/s | faiss batched q/s | beam is |
|---:|---:|---:|---|
| 1_000_000 | 170 | 234 | 1.4× slower |
| 2_000_000 | 97 | 120 | 1.2× slower |
| 4_000_000 | 43 | 60 | 1.4× slower |

**No crossover (for `main_batch_topk`).** Both are O(n) at scale, so once the
fixed overheads are amortized the ratio stabilizes at beam's ~1.2–1.4×
constant-factor disadvantage — it does not vanish with more data. The 9.2×→1.4×
collapse (serial→GPU-top-k) was overhead amortization; the residual ~1.3× is
beam's naive per-query DB re-read. Closing it, we hypothesized, needs a
**GEMM-tiled distance kernel** (reuse a DB-row tile across a query tile in shared
memory). That kernel is now built and measured below — and it crosses over.

## Tiled (GEMM-style) distance kernel: beam now BEATS faiss batched at n ≥ 100k

The **shared-memory tiled** batched flat kernel (`src/gpu/flat.wgsl ::
main_batch_tiled`, host [`GpuFlatIndex::search_knn_batch_gemm`]) is now the
default batched flat path (`search_knn_batch` auto-selects it for `k ≤ MAX_TOPK`
and `dim % 4 == 0`; `main_batch_topk` stays the fallback). Same M1 Max, same
deterministic dataset shape (`dim=128`, `k=10`, `queries=200`, uniform seeded),
`--batch 200`, **recall 1.000** (exact — the tiled top-k selects the same rows as
the CPU oracle, verified in `tests/batched_query.rs`).

**Batched throughput — beam-GPU tiled vs faiss-CPU, head-to-head (both batched):**

| n | beam-GPU tiled q/s | faiss-CPU batched q/s | beam vs faiss |
|---:|---:|---:|---|
| 10_000 | ~18_800 | 37_174 | **1.98× slower** |
| 100_000 | ~3_880 | 3_483 | **1.11× FASTER** |
| 1_000_000 | ~522 | 234 | **2.23× FASTER** |
| 2_000_000 | ~277 | 120 | **2.31× FASTER** |
| 4_000_000 | ~139 | 60 | **2.32× FASTER** |

**Honest verdict: beam-GPU tiled BEATS faiss-CPU-batched at every tested n ≥
100k** — by 1.1× at 100k, growing to a steady **~2.3× at 1M–4M** — and loses only
at n = 10k (1.98× slower, where the fixed GPU dispatch/readback floor still
dominates a tiny scan). This is a real crossover, not a projection: the earlier
"no crossover, beam ~1.3× slower forever" conclusion held for the
one-workgroup-per-query kernel and is **overturned** for the tiled kernel. Recall
stays 1.000, so it is an exact-flat win.

**Speedup vs beam's own prior path** (`main_batch_topk`, same binary/methodology):

| n | prior beam-topk q/s | **beam tiled q/s** | tiled vs prior |
|---:|---:|---:|---:|
| 10_000 | ~9_700 | **~18_800** | **1.94×** |
| 100_000 | ~1_370 | **~3_880** | **2.83×** |
| 1_000_000 | ~174 | **~522** | **3.00×** |
| 2_000_000 | ~97 | **~277** | **2.86×** |
| 4_000_000 | ~43 | **~139** | **3.23×** |

(beam tiled amortized latency: 0.053 / 0.26 / 1.9 / 3.6 / 7.2 ms/query at 10k /
100k / 1M / 2M / 4M. Numbers are ±10% run-to-run at small n; representative values
shown.)

### What actually moved the needle (honest attribution)

The measured breakdown corrects the a-priori "memory-bandwidth-bound" framing:

- **The workload is latency/ILP-bound, not DRAM-bandwidth-bound.** `main_batch_topk`
  sustains a flat ~86–89 GB/s regardless of n (measured 1M–4M) — only ~22% of M1
  Max's ~400 GB/s peak — so it is *not* saturating DRAM. Reducing DB traffic by
  tiling therefore is **not** the lever it would be on a bandwidth-bound GPU.
  Confirming this: a *scalar* tiled kernel (shared-memory DB reuse, but the plain
  128-wide scalar distance loop) measured **140 q/s at 1M — slower than the 174 q/s
  prior path.** Explicit shared-memory staging alone did not help (a no-staging
  variant relying on cache measured the same), because the per-tile barrier exposes
  the same load latency it saves.
- **vec4 vectorization is the dominant lever.** Reading DB + query as `vec4<f32>`
  and accumulating in a vec4 register (4 independent lanes) cuts the inner-loop load
  count and the accumulation dependency chain 4×. That is what took the tiled kernel
  from 140 → **522 q/s at 1M** (3.7×). The distance reduction's serial dependency
  chain, not memory bandwidth, was the real bottleneck.
- **The tiling's DB reuse still earns its keep at scale.** The win ratio vs faiss
  *grows* with n (1.1× at 100k → ~2.3× at 1M–4M) — exactly where the DB stops fitting
  in cache (51 MB at 100k vs 512 MB–2 GB at 1M–4M), so the query-tile DB reuse
  (each DB element read from global once per `TILE_Q=64` queries instead of once per
  query) becomes the differentiator on top of the vec4 ILP. Tiling + vec4 together
  are what beat AMX-backed faiss; neither alone did.

### Tiled kernel design + correctness (for reproducers)

- **Tiling:** one workgroup per `(query-tile, DB-split)`. `TILE_Q = 64` queries
  share a workgroup (workgroup size 64, one query per thread); the DB row range is
  split-k'd across workgroups for occupancy. Each workgroup walks its split in
  `TILE_N = 16`-row tiles, cooperatively staging each `TILE_N × dim` DB block into
  `var<workgroup>` shared memory ONCE (a coalesced contiguous copy) and reusing it
  across all 64 queries. **Shared memory = `TILE_N × dim × 4` = 8 KB** (only the DB
  tile; the tiny query buffer is read from cache-resident global memory — staging it
  too would push shared to 24 KB and let only one workgroup be resident per Metal
  core, starving latency-hiding). Each thread keeps a private register top-k
  (`MAX_K = 32`) over its split; the host merges the disjoint per-split partials.
- **Precision:** L2 is the **direct** `sum((q-d)²)` from the shared tile (vec4
  lanes), NOT the `‖q‖²+‖d‖²−2q·d` identity — so no catastrophic cancellation, and
  the ≤1e-3 oracle check holds. Dot/Cosine is `sum(q·d)`. L2 and Dot both supported;
  Cosine = normalized Dot. The live/tombstone mask is folded in (non-live rows
  skipped).
- **Fallbacks:** `k > MAX_TOPK` → distance-matrix path; `dim > 128` or `dim % 4 ≠ 0`
  → the `main_batch_topk` path. Both stay exact.
- **Tests** (`tests/batched_query.rs`, not skipped on this Mac): tiled == CPU oracle
  for k ∈ {1, 10, 32} on L2 and Dot; tiled == the `main_batch_topk` path; tombstones
  excluded; ragged tile boundaries (n off `TILE_N`, query count off `TILE_Q`);
  unsupported-dim fallback stays exact.

[`GpuFlatIndex::search_knn_batch_gemm`]: ../src/gpu/mod.rs

### Revised bottom line

On exact flat L2/Dot batched search, **beam-GPU tiled now beats faiss-CPU-batched
on M1 Max at every tested n ≥ 100k (~1.1×–2.3×), losing only at n = 10k** where GPU
dispatch overhead dominates. This is beam's goal-2 flat-search win — on top of the
prior honest advantages (ANN pruning via IVF-flat at ~8% scan, IVF-PQ 32× memory,
Metal portability). The single-query rows at the top of this doc are unchanged
(that path is still dispatch-bound); the win is specifically the **batched** path.

## Reproducer

- **Competitor (CPU):** `apps/beam/benchmark/competitor_bench.py` — seeded
  faiss `IndexFlatL2` + `IndexIVFPQ` (auto-falls-back to numpy brute-force if
  faiss is unavailable). Run:

  ```bash
  python3 -m venv /tmp/beambench && /tmp/beambench/bin/pip install faiss-cpu numpy
  /tmp/beambench/bin/python apps/beam/benchmark/competitor_bench.py \
      --sizes 10000,100000,1000000
  ```

- **beam (GPU):** the exact commands measured (release binary):

  ```bash
  beam bench --index flat    --n <N> --dim 128 --k 10 --queries 200
  beam bench --index ivfflat --n <N> --dim 128 --k 10 --queries 200 --nlist 256 --nprobe 16
  # batched throughput: all 200 queries in one call. k=10 ≤ MAX_TOPK and
  # dim=128 (% 4 == 0), so this runs the GEMM-tiled path (the "Tiled distance
  # kernel" section) — the current default; `main_batch_topk` is the fallback.
  beam bench --index flat    --n <N> --dim 128 --k 10 --queries 200 --batch 200
  ```

Both sides are deterministic (fixed seeds), so a re-run on the same hardware
reproduces these numbers within measurement noise.

<!-- HANDWRITE-END -->
