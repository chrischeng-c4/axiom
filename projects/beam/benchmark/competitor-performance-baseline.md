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
the per-query readback. That batched path is now **built and measured** (see
**[Batched query](#batched-query-the-throughput-lever-now-measured)**): it is a
large gain over beam's own serial path (up to 7.1× at 10k) but, honestly, still
does **not** beat faiss's *batched* throughput at any tested n — the crossover
would need GPU-side top-k to kill the `T × n` readback, which remains future work.

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
BLAS call. The interpretation above named the fix: **batch the queries into one
dispatch** so the fixed overhead amortizes. That path now exists
([`GpuFlatIndex::search_knn_batch`], `beam bench --index flat --batch <B>`): the
batch is tiled, each tile scores its `T × n` distance sub-matrix in **one**
dispatch (one GPU thread per DB row, looping the tile's queries), reads back
`T × n` f32, and does exact top-k per query on the CPU. Recall stays **1.000**
(same exact distances as the serial scan). Same machine, same dataset shape
(`dim=128`, `k=10`, `queries=200`, uniform seeded), measured with
`--batch 200` (all 200 queries in one batched call, directly comparable to
faiss's all-200-in-one-call batched throughput).

**Batched throughput — beam-GPU vs faiss-CPU, head-to-head (both batched):**

| n | beam-GPU-batched q/s | faiss-CPU-batched q/s | beam vs faiss batched |
|---:|---:|---:|---|
| 10_000 | ~4_050 | 37_174 | **9.2× slower** |
| 100_000 | ~467 | 3_483 | **7.5× slower** |
| 1_000_000 | ~52 | 234 | **4.5× slower** |

**What batching bought beam (its own serial → batched, same binary):**

| n | beam serial q/s (single) | beam batched q/s | batched speedup |
|---:|---:|---:|---:|
| 10_000 | 571 | ~4_050 | **7.1×** |
| 100_000 | 199 | ~467 | **2.4×** |
| 1_000_000 | 37.5 | ~52 | **1.4×** |

(beam-batched amortized latency: 0.25 / 2.14 / 19.2 ms/query at 10k / 100k / 1M.
Numbers are ±10% run-to-run; representative values shown.)

### Honest verdict: batched beam-GPU still does NOT beat faiss batched

At **every** tested size, batched beam-GPU-flat loses to batched faiss-CPU-flat —
by **~9.2× / 7.5× / 4.5×** at 10k / 100k / 1M. So the "batched path turns the
loss into a win" hope is **not** realized on this M1 Max: it is a real,
large improvement over beam's *own* serial path (up to **7.1×** at 10k), and the
gap to faiss narrows monotonically with n (9.2× → 4.5×, the same
GPU-scales-better-than-AMX trend the single-query rows show) — but no crossover
at n ≤ 1M. Do not claim a batched throughput win.

Two honest reasons batching helped less than hoped, especially at large n:

1. **faiss's batched mode also speeds up a lot.** Its all-queries-in-one-call
   flat scan is a single big BLAS GEMM, so faiss jumps 990 → 3_483 q/s (3.5×) at
   100k and 87 → 234 q/s (2.7×) at 1M going serial → batched — outrunning beam's
   batched gain there. beam is chasing a moving target, not a static one.
2. **beam's batched bottleneck moved off the dispatch floor onto the readback +
   CPU top-k.** With one dispatch per tile the ~1.7 ms floor is amortized away,
   but the kernel now streams back the full `T × n` distance matrix (e.g. 800 MB
   total at n=1M for 200 queries) and the host does single-threaded exact top-k
   over it. That transport + selection — not GPU compute — dominates at large n
   (measured: swapping the naive per-(query,row) kernel for the DB-traffic-light
   per-row kernel changed throughput by <10%, confirming compute is not the
   limiter). The next lever is **GPU-side per-query top-k**, which shrinks the
   readback from `T × n` to `T × k` (a ~n/k reduction, ~100_000× at n=1M/k=10)
   and offloads the selection — that is where a crossover would come from, and it
   is the recommended follow-up. It is deliberately out of scope here (the CPU
   top-k keeps the batched path exact and simple), so this section reports the
   simple-batched result honestly rather than the projected one.

[`GpuFlatIndex::search_knn_batch`]: ../src/gpu/mod.rs

## Reproducer

- **Competitor (CPU):** `projects/beam/benchmark/competitor_bench.py` — seeded
  faiss `IndexFlatL2` + `IndexIVFPQ` (auto-falls-back to numpy brute-force if
  faiss is unavailable). Run:

  ```bash
  python3 -m venv /tmp/beambench && /tmp/beambench/bin/pip install faiss-cpu numpy
  /tmp/beambench/bin/python projects/beam/benchmark/competitor_bench.py \
      --sizes 10000,100000,1000000
  ```

- **beam (GPU):** the exact commands measured (release binary):

  ```bash
  beam bench --index flat    --n <N> --dim 128 --k 10 --queries 200
  beam bench --index ivfflat --n <N> --dim 128 --k 10 --queries 200 --nlist 256 --nprobe 16
  # batched throughput (the "Batched query" section): all 200 queries in one call
  beam bench --index flat    --n <N> --dim 128 --k 10 --queries 200 --batch 200
  ```

Both sides are deterministic (fixed seeds), so a re-run on the same hardware
reproduces these numbers within measurement noise.
