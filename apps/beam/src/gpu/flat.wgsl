// Beam flat (brute-force) distance kernel.
//
// One invocation per DB row: invocation `row` computes the distance between the
// single query vector and DB row `row`, writing it to `out_dist[row]`. Top-k
// selection happens on the CPU (k is tiny). Workgroup size 64; the `row < n`
// guard covers the ragged final workgroup.
//
// Score convention — SHARED VERBATIM with the CPU oracle (`index/cpu_flat.rs`)
// so their top-k agree:
//   metric == 0 (L2)          -> sum of squared differences   (smaller = better)
//   metric == 1 (Dot)         -> dot product                  (larger  = better)
//   metric == 2 (Cosine)      -> dot product over unit vectors (larger = better;
//                                DB rows are normalized on insert, the query is
//                                normalized on the host before upload)
// Raw values are returned; the host orders them per-metric.

struct Params {
    n: u32,
    dim: u32,
    metric: u32,
    _pad: u32,
};

@group(0) @binding(0) var<storage, read> db: array<f32>;
@group(0) @binding(1) var<storage, read> query: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;
@group(0) @binding(3) var<storage, read_write> out_dist: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let row: u32 = gid.x;
    if (row >= params.n) {
        return;
    }
    let dim: u32 = params.dim;
    let base: u32 = row * dim;

    var acc: f32 = 0.0;
    if (params.metric == 0u) {
        // L2: sum of squared differences.
        for (var j: u32 = 0u; j < dim; j = j + 1u) {
            let d: f32 = db[base + j] - query[j];
            acc = acc + d * d;
        }
    } else {
        // Dot (metric 1) and Cosine (metric 2, vectors pre-normalized): dot product.
        for (var j: u32 = 0u; j < dim; j = j + 1u) {
            acc = acc + db[base + j] * query[j];
        }
    }
    out_dist[row] = acc;
}

// ---- Filtered flat scan (entry point `main_filtered`) ----------------------
//
// Identical to `main`, but each row also carries a host-built `keep` bit
// (`fmask[row]`: 1 = the row's payload matches the filter, 0 = drop). A dropped
// row is assigned the metric's WORST score (a large sentinel) so it can never
// enter top-k; a kept row gets the exact same distance `main` would compute, so
// filtered GPU top-k equals the filtered CPU oracle for the surviving rows.
// Bindings are disjoint from `main` (this module holds two entry points and
// each pipeline supplies a layout for only its own bindings).
//
//   metric == 0 (L2)          -> dropped rows get +SENTINEL (smaller is better)
//   metric != 0 (Dot/Cosine)  -> dropped rows get -SENTINEL (larger is better)
//
// SENTINEL is a large finite value (near f32::MAX) rather than a true infinity
// so the readback stays a plain finite f32; the host caps top-k at the match
// count regardless, so a dropped row's sentinel is never actually selected.

const SENTINEL: f32 = 3.0e38;

@group(0) @binding(4) var<storage, read> db_f: array<f32>;
@group(0) @binding(5) var<storage, read> query_f: array<f32>;
@group(0) @binding(6) var<uniform> params_f: Params;
@group(0) @binding(7) var<storage, read_write> out_dist_f: array<f32>;
@group(0) @binding(8) var<storage, read> fmask: array<u32>;

@compute @workgroup_size(64)
fn main_filtered(@builtin(global_invocation_id) gid: vec3<u32>) {
    let row: u32 = gid.x;
    if (row >= params_f.n) {
        return;
    }
    if (fmask[row] == 0u) {
        // Non-matching row: worst-possible score for the metric.
        if (params_f.metric == 0u) {
            out_dist_f[row] = SENTINEL;
        } else {
            out_dist_f[row] = -SENTINEL;
        }
        return;
    }
    let dim: u32 = params_f.dim;
    let base: u32 = row * dim;

    var acc: f32 = 0.0;
    if (params_f.metric == 0u) {
        for (var j: u32 = 0u; j < dim; j = j + 1u) {
            let d: f32 = db_f[base + j] - query_f[j];
            acc = acc + d * d;
        }
    } else {
        for (var j: u32 = 0u; j < dim; j = j + 1u) {
            acc = acc + db_f[base + j] * query_f[j];
        }
    }
    out_dist_f[row] = acc;
}

// ---- Batched flat scan (entry point `main_batch`) --------------------------
//
// Scores a TILE of `num_q` queries against all `n` DB rows in ONE dispatch, so
// the fixed per-dispatch + blocking-readback overhead amortizes across the whole
// tile — the batched-throughput lever. Invocation grid is 1D over DB rows:
// invocation `row` loads its DB row ONCE and scores it against ALL `num_q`
// queries in the tile, so each row's `dim` floats are read from global memory
// once and reused across the whole tile (a ~`num_q`× cut in global DB traffic vs
// one invocation per (query, row) — the naive scan is DB-bandwidth bound). The
// tile's queries are tiny (`num_q * dim` floats) and stay hot in cache across the
// inner loop.
//
// Output is query-major (`out_batch[q * n + row]`) so the host reads each query's
// `n` distances as one contiguous slice and runs the shared top-k per query.
//
// Every row also carries the host keep bit `keep_batch[row]` (the collection's
// live mask, folded in exactly like `main_filtered`): a tombstoned row is
// assigned the metric's worst sentinel so it can never enter any query's top-k.
// With no tombstones the mask is all-ones, so every row is scored. Distances are
// computed with the SAME per-metric summation as `main`, so a batched query's
// scores match the serial `main` path bit-for-intent.

struct BatchParams {
    n: u32,
    dim: u32,
    metric: u32,
    num_q: u32,
};

@group(0) @binding(9)  var<storage, read> db_b: array<f32>;
@group(0) @binding(10) var<storage, read> queries_b: array<f32>;
@group(0) @binding(11) var<uniform> params_b: BatchParams;
@group(0) @binding(12) var<storage, read_write> out_batch: array<f32>;
@group(0) @binding(13) var<storage, read> keep_batch: array<u32>;

@compute @workgroup_size(64)
fn main_batch(@builtin(global_invocation_id) gid: vec3<u32>) {
    let row: u32 = gid.x;
    if (row >= params_b.n) {
        return;
    }
    let n: u32 = params_b.n;
    let num_q: u32 = params_b.num_q;

    if (keep_batch[row] == 0u) {
        // Tombstoned / non-kept row: worst-possible score for the metric, for
        // every query — so it can never enter any query's top-k.
        var sentinel: f32 = SENTINEL;
        if (params_b.metric != 0u) {
            sentinel = -SENTINEL;
        }
        for (var q: u32 = 0u; q < num_q; q = q + 1u) {
            out_batch[q * n + row] = sentinel;
        }
        return;
    }

    let dim: u32 = params_b.dim;
    let dbase: u32 = row * dim;
    // Score this row against every query in the tile, reusing the DB row across
    // the whole inner loop (loaded once from global memory).
    if (params_b.metric == 0u) {
        for (var q: u32 = 0u; q < num_q; q = q + 1u) {
            let qbase: u32 = q * dim;
            var acc: f32 = 0.0;
            for (var j: u32 = 0u; j < dim; j = j + 1u) {
                let d: f32 = db_b[dbase + j] - queries_b[qbase + j];
                acc = acc + d * d;
            }
            out_batch[q * n + row] = acc;
        }
    } else {
        for (var q: u32 = 0u; q < num_q; q = q + 1u) {
            let qbase: u32 = q * dim;
            var acc: f32 = 0.0;
            for (var j: u32 = 0u; j < dim; j = j + 1u) {
                acc = acc + db_b[dbase + j] * queries_b[qbase + j];
            }
            out_batch[q * n + row] = acc;
        }
    }
}

// ---- Batched flat scan + GPU-side per-query top-k (entry `main_batch_topk`) --
//
// The throughput lever. The `main_batch` kernel above still streams back the full
// `num_q * n` distance matrix and selects the top-k on the CPU — that readback +
// single-threaded selection is the measured batched bottleneck. This kernel keeps
// the whole top-k ON the GPU and reads back only `num_q * want` (id, score) pairs
// (a ~n/want reduction; ~100000x at n=1M/k=10), so the batched path becomes
// (near-)compute-bound instead of readback-bound.
//
// Layout: ONE WORKGROUP PER QUERY in the tile (`workgroup_id.x == query`). The
// workgroup's `WG_TOPK` threads cooperatively scan all `n` DB rows grid-strided
// (thread `tid` handles rows tid, tid+WG_TOPK, tid+2*WG_TOPK, ...); each thread
// keeps a PRIVATE sorted top-`want` in registers (insertion into a small array,
// capped at MAX_K). Non-live rows (`keep_topk[row] == 0`) are simply SKIPPED — the
// live/tombstone mask folded in exactly like `main_filtered`/`main_batch`, but by
// omission rather than a sentinel, so a tombstoned row never enters any local list.
//
// Reduce: each thread publishes its local top-`want` into `var<workgroup>` shared
// memory, then a log2(WG_TOPK)-step TREE MERGE pairwise-merges the sorted lists
// (thread `tid` merges list `tid` with list `tid+stride`, keeping the best `want`)
// until thread 0 holds the query's global top-`want`, which it writes out as
// `(score_bits, row)` pairs. A partial list (a thread that saw fewer than `want`
// live rows, or none) is padded with the metric's worst sentinel so it loses every
// merge; because `want <= n_live`, the final `want` are always real rows (each
// global-top row is within its own scanning thread's top-`want`, so it survives).
//
// Distances use the SAME per-metric summation (and j-order) as `main`/`main_batch`,
// so a top-k row's score is bit-for-intent identical to the serial scan and the
// selected row SET equals the CPU oracle's (exact flat).
//
//   metric == 0 (L2)         -> sum of squared diffs, SMALLER is better
//   metric != 0 (Dot/Cosine) -> dot product,          LARGER  is better

// Workgroup width and the compile-time cap on k for this kernel. Shared memory is
// `WG_TOPK * MAX_K * (4 + 4)` bytes = 64*32*8 = 16 KB (half Metal's 32 KB
// threadgroup ceiling). `@workgroup_size` below MUST equal WG_TOPK. The host
// (`GpuFlatIndex::MAX_TOPK`) mirrors MAX_K and falls back to the `main_batch` +
// CPU-topk path for k > MAX_K, so large-k queries still work.
const WG_TOPK: u32 = 64u;
const MAX_K: u32 = 32u;
const SH_TOPK_LEN: u32 = 2048u; // WG_TOPK * MAX_K

struct TopkParams {
    n: u32,
    dim: u32,
    metric: u32,
    num_q: u32,
    want: u32, // min(k, n_live), <= MAX_K
    _p0: u32,
    _p1: u32,
    _p2: u32,
};

@group(0) @binding(14) var<storage, read> db_topk: array<f32>;
@group(0) @binding(15) var<storage, read> queries_topk: array<f32>;
@group(0) @binding(16) var<uniform> params_topk: TopkParams;
// Output: query-major `num_q * want` entries, each two u32s: [bitcast<u32>(score), row].
@group(0) @binding(17) var<storage, read_write> out_topk: array<u32>;
@group(0) @binding(18) var<storage, read> keep_topk: array<u32>;

// Each thread publishes its local sorted top-`want` here (region `tid*MAX_K`).
var<workgroup> sh_score: array<f32, SH_TOPK_LEN>;
var<workgroup> sh_row: array<u32, SH_TOPK_LEN>;

@compute @workgroup_size(64)
fn main_batch_topk(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let q: u32 = wid.x; // one workgroup per query in the tile (q is uniform)
    if (q >= params_topk.num_q) {
        return;
    }
    let tid: u32 = lid.x;
    let n: u32 = params_topk.n;
    let dim: u32 = params_topk.dim;
    let want: u32 = params_topk.want;
    let larger_better: bool = params_topk.metric != 0u;
    let qbase: u32 = q * dim;

    // Private sorted top-`want` (best-first), length `cnt` (<= want <= MAX_K).
    var best_s: array<f32, MAX_K>;
    var best_r: array<u32, MAX_K>;
    var cnt: u32 = 0u;

    // Grid-stride scan: this thread scores rows tid, tid+WG_TOPK, ... skipping
    // non-live rows, and keeps only its own top-`want`.
    var row: u32 = tid;
    loop {
        if (row >= n) {
            break;
        }
        if (keep_topk[row] != 0u) {
            let dbase: u32 = row * dim;
            var acc: f32 = 0.0;
            if (larger_better) {
                for (var j: u32 = 0u; j < dim; j = j + 1u) {
                    acc = acc + db_topk[dbase + j] * queries_topk[qbase + j];
                }
            } else {
                for (var j: u32 = 0u; j < dim; j = j + 1u) {
                    let d: f32 = db_topk[dbase + j] - queries_topk[qbase + j];
                    acc = acc + d * d;
                }
            }
            // Qualifies if the local list is not yet full, or `acc` beats the
            // current worst kept score.
            var qualifies: bool = cnt < want;
            if (!qualifies) {
                let worst: f32 = best_s[cnt - 1u];
                qualifies = select(acc < worst, acc > worst, larger_better);
            }
            if (qualifies) {
                if (cnt < want) {
                    cnt = cnt + 1u;
                }
                // Insertion sort: shift worse entries right, drop off the end.
                var p: u32 = cnt - 1u;
                loop {
                    if (p == 0u) {
                        break;
                    }
                    let prev: f32 = best_s[p - 1u];
                    let better: bool = select(acc < prev, acc > prev, larger_better);
                    if (better) {
                        best_s[p] = best_s[p - 1u];
                        best_r[p] = best_r[p - 1u];
                        p = p - 1u;
                    } else {
                        break;
                    }
                }
                best_s[p] = acc;
                best_r[p] = row;
            }
        }
        row = row + WG_TOPK;
    }

    // Publish local top-`want` to shared, padding unfilled slots with the metric's
    // worst sentinel so they lose every merge.
    let sbase: u32 = tid * MAX_K;
    let sentinel: f32 = select(3.0e38, -3.0e38, larger_better);
    for (var j: u32 = 0u; j < want; j = j + 1u) {
        if (j < cnt) {
            sh_score[sbase + j] = best_s[j];
            sh_row[sbase + j] = best_r[j];
        } else {
            sh_score[sbase + j] = sentinel;
            sh_row[sbase + j] = 0u;
        }
    }
    workgroupBarrier();

    // Tree merge of the WG_TOPK sorted lists down to thread 0. Each active thread
    // merges its list with the one `stride` away, keeping the best `want`.
    var stride: u32 = WG_TOPK >> 1u;
    loop {
        if (stride == 0u) {
            break;
        }
        if (tid < stride) {
            let abase: u32 = tid * MAX_K;
            let bbase: u32 = (tid + stride) * MAX_K;
            var ia: u32 = 0u;
            var ib: u32 = 0u;
            var out_j: u32 = 0u;
            // Merge the two sorted length-`want` lists into registers (ia+ib==out_j
            // stays < want at every read, so both reads are within [0, want)).
            loop {
                if (out_j >= want) {
                    break;
                }
                let sa: f32 = sh_score[abase + ia];
                let sb: f32 = sh_score[bbase + ib];
                let take_a: bool = select(sa <= sb, sa >= sb, larger_better);
                if (take_a) {
                    best_s[out_j] = sa;
                    best_r[out_j] = sh_row[abase + ia];
                    ia = ia + 1u;
                } else {
                    best_s[out_j] = sb;
                    best_r[out_j] = sh_row[bbase + ib];
                    ib = ib + 1u;
                }
                out_j = out_j + 1u;
            }
            for (var j: u32 = 0u; j < want; j = j + 1u) {
                sh_score[abase + j] = best_s[j];
                sh_row[abase + j] = best_r[j];
            }
        }
        workgroupBarrier();
        stride = stride >> 1u;
    }

    // Thread 0 emits this query's global top-`want` as (score_bits, row) pairs.
    if (tid == 0u) {
        let obase: u32 = q * want * 2u;
        for (var j: u32 = 0u; j < want; j = j + 1u) {
            out_topk[obase + j * 2u + 0u] = bitcast<u32>(sh_score[j]);
            out_topk[obase + j * 2u + 1u] = sh_row[j];
        }
    }
}

// ---- GEMM-tiled batched flat scan + GPU-side top-k (entry `main_batch_tiled`) ----
//
// The compute-bound lever. `main_batch_topk` above uses ONE workgroup per query,
// so every query re-reads the WHOLE DB from global memory (T × n × dim global
// reads, no reuse) — the plateau vs faiss's cache-tiled BLAS GEMM. This kernel
// applies the matmul tiling trick to get DB-row REUSE across a tile of queries: a
// block of DB rows is staged into `var<workgroup>` shared memory ONCE and then
// reused by every query in the tile, so each DB element is read from global memory
// once per TILE_Q queries instead of once per query (a ~TILE_Q× cut in global DB
// traffic).
//
// Layout — a 2D workgroup grid `(qtile, split)`:
//   * `workgroup_id.x = qtile` selects a tile of TILE_Q_T queries
//     (`q_start = qtile*TILE_Q_T`), one per thread (workgroup size == TILE_Q_T).
//   * `workgroup_id.y = split` selects a contiguous DB row range (split-k), so the
//     n rows are spread over `num_splits` workgroups PER query-tile — this keeps
//     the GPU's cores busy even though the batch has few queries (few query-tiles).
//     Reuse is unaffected by the split: within a workgroup the staged DB tile is
//     still shared by all TILE_Q_T queries.
//
// Each thread owns exactly ONE query and keeps a PRIVATE sorted register top-`want`
// (insertion sort, capped at MAX_K) over its split — so NO cross-thread merge is
// needed (unlike `main_batch_topk`, where 64 threads split one query and tree-merge).
// The workgroup walks its split in DB tiles of TILE_N_T rows:
//   1. Cooperatively stage the TILE_N_T × dim DB block CONTIGUOUSLY into `sh_db_t`
//      (a coalesced flat copy from the row-major `db`), plus the tile's keep bits
//      into `sh_keep_t`.
//   2. Barrier, then each thread scores its query against the staged rows straight
//      from shared memory (reusing the DB block across all TILE_Q_T queries) and
//      folds each live row into its register top-k. Non-live rows are SKIPPED (the
//      live/tombstone mask, folded in exactly like `main_batch_topk`).
//   3. Barrier, advance to the next DB tile.
// Finally each thread writes its per-(query, split) partial top-`want` as
// `(score_bits, row)` pairs; the HOST merges the `num_splits` disjoint partials per
// query (splits cover disjoint row ranges, so no duplicates) to the global top-k —
// a tiny merge (`num_splits × want` candidates), the split-k counterpart of the
// in-kernel tree-merge.
//
// OCCUPANCY: only the DB tile is staged in shared — the query is read from the
// (tiny, cache-resident) global `queries` buffer, NOT staged. This is deliberate:
// Metal gives each core ~32 KB of threadgroup memory, so a workgroup that hogged
// shared for a query block too (e.g. TILE_Q×dim = 16 KB) would let only ONE
// workgroup be resident per core, starving latency-hiding. Keeping shared at just
// TILE_N_T×dim (8 KB) lets several workgroups stay resident, which is what makes
// the tiled path actually beat the one-workgroup-per-query kernel. DB reuse (the
// lever) is preserved regardless — the query buffer is small and stays in cache.
//
// Precision: L2 is the DIRECT `sum((q-d)²)` from the shared DB tile (NOT the
// `‖q‖²+‖d‖²−2q·d` identity, which cancels catastrophically for near-duplicate
// vectors and would break the ≤1e-3 oracle check); Dot/Cosine is `sum(q·d)`. Same
// per-metric summation and j-order as `main`/`main_batch_topk`, so a selected row's
// score is bit-for-intent identical to the serial scan and the row SET equals the
// CPU oracle's (exact flat).
//
// Shared-memory budget: `sh_db_t` is TILE_N_T × MAX_TILE_DIM_T f32 = 16×128×4 =
// 8 KB (rows packed CONTIGUOUSLY at column stride `dim`, so dim < 128 uses less);
// `sh_keep_t` is 64 B — ~8 KB total, well under Metal's 32 KB ceiling and small
// enough for good residency. The tile array is sized at compile-time width
// MAX_TILE_DIM_T (=128, the bench dim); the host falls back to `main_batch_topk`
// for dim > MAX_TILE_DIM_T, and k > MAX_K also falls back.

const TILE_Q_T: u32 = 64u;         // queries per workgroup == @workgroup_size
const TILE_N_T: u32 = 16u;         // DB rows staged per shared tile
const MAX_TILE_DIMV_T: u32 = 32u;  // compile-time shared-tile capacity (max dim / 4)
const SH_DBV_LEN_T: u32 = 512u;    // TILE_N_T * MAX_TILE_DIMV_T (vec4 slots)
const EMPTY_ROW_T: u32 = 0xFFFFFFFFu; // partial-slot padding marker (host skips it)

struct TiledParams {
    n: u32,
    dim: u32,        // real dim; the host guarantees dim % 4 == 0 on this path
    metric: u32,
    num_q: u32,
    want: u32,       // min(k, n_live), <= MAX_K
    num_splits: u32, // DB-range splits per query-tile (split-k occupancy)
    split_len: u32,  // rows per split (ceil(n / num_splits))
    _p0: u32,
};

// DB + queries are viewed as `vec4<f32>` lanes (the host packs them so dim is a
// multiple of 4): a single vec4 load fetches 4 columns and a single vec4 multiply
// does 4 lanes, cutting the inner-loop load count AND the accumulation dependency
// chain 4× — the ILP lever for this latency-bound distance kernel.
@group(0) @binding(19) var<storage, read> db_tiled: array<vec4<f32>>;
@group(0) @binding(20) var<storage, read> queries_tiled: array<vec4<f32>>;
@group(0) @binding(21) var<uniform> params_tiled: TiledParams;
// Output: per (query, split) a length-`want` partial, each two u32s
// [bitcast<u32>(score), row]; empty slots carry row == EMPTY_ROW_T.
@group(0) @binding(22) var<storage, read_write> out_tiled: array<u32>;
@group(0) @binding(23) var<storage, read> keep_tiled: array<u32>;

var<workgroup> sh_db_t: array<vec4<f32>, SH_DBV_LEN_T>;
var<workgroup> sh_keep_t: array<u32, TILE_N_T>;

@compute @workgroup_size(64)
fn main_batch_tiled(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let tid: u32 = lid.x;
    let n: u32 = params_tiled.n;
    let dim_v: u32 = params_tiled.dim >> 2u; // vec4 lanes per row (dim / 4)
    let want: u32 = params_tiled.want;
    let num_q: u32 = params_tiled.num_q;
    let num_splits: u32 = params_tiled.num_splits;
    let split_len: u32 = params_tiled.split_len;
    let larger_better: bool = params_tiled.metric != 0u;

    let qtile: u32 = wid.x;
    let sp: u32 = wid.y;
    let qi: u32 = qtile * TILE_Q_T + tid; // this thread's global query index
    let has_q: bool = qi < num_q;
    let qvbase: u32 = qi * dim_v;          // this thread's query row (vec4 units)

    // This workgroup's DB row range (split-k). All uniform across the workgroup, so
    // the tile loop trip count — and thus the barriers below — are uniform.
    let split_start: u32 = sp * split_len;
    var split_end: u32 = split_start + split_len;
    if (split_end > n) {
        split_end = n;
    }

    // Private sorted register top-`want` for this thread's query over its split.
    var best_s: array<f32, MAX_K>;
    var best_r: array<u32, MAX_K>;
    var cnt: u32 = 0u;

    var base_row: u32 = split_start;
    loop {
        if (base_row >= split_end) {
            break;
        }
        var rows_this: u32 = split_end - base_row;
        if (rows_this > TILE_N_T) {
            rows_this = TILE_N_T;
        }

        // Cooperatively stage the DB tile (rows_this × dim_v vec4s) CONTIGUOUSLY into
        // `sh_db_t` — a coalesced flat copy from the row-major `db` — plus the tile's
        // keep bits. Every thread helps, so each staged DB element is read from
        // global memory exactly ONCE and then reused by all TILE_Q_T queries below.
        let total_v: u32 = rows_this * dim_v;
        let gvbase: u32 = base_row * dim_v;
        var idx: u32 = tid;
        loop {
            if (idx >= total_v) {
                break;
            }
            sh_db_t[idx] = db_tiled[gvbase + idx];
            idx = idx + TILE_Q_T;
        }
        if (tid < rows_this) {
            sh_keep_t[tid] = keep_tiled[base_row + tid];
        }
        workgroupBarrier();

        // Each thread scores its own query against the staged rows: the DB row comes
        // from shared (reused across the query tile), the query from global (cached).
        // vec4 loads + a vec4 accumulator keep 4 independent lanes in flight (ILP),
        // then a single horizontal sum — the exact-arithmetic direct L2/Dot form.
        if (has_q) {
            for (var r: u32 = 0u; r < rows_this; r = r + 1u) {
                if (sh_keep_t[r] != 0u) {
                    let db_vb: u32 = r * dim_v;
                    var acc4: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
                    if (larger_better) {
                        for (var jv: u32 = 0u; jv < dim_v; jv = jv + 1u) {
                            acc4 = acc4 + queries_tiled[qvbase + jv] * sh_db_t[db_vb + jv];
                        }
                    } else {
                        for (var jv: u32 = 0u; jv < dim_v; jv = jv + 1u) {
                            let d: vec4<f32> = queries_tiled[qvbase + jv] - sh_db_t[db_vb + jv];
                            acc4 = acc4 + d * d;
                        }
                    }
                    let acc: f32 = acc4.x + acc4.y + acc4.z + acc4.w;
                    let row_g: u32 = base_row + r;
                    // Insertion into the private sorted top-`want` (best-first).
                    var qualifies: bool = cnt < want;
                    if (!qualifies) {
                        let worst: f32 = best_s[cnt - 1u];
                        qualifies = select(acc < worst, acc > worst, larger_better);
                    }
                    if (qualifies) {
                        if (cnt < want) {
                            cnt = cnt + 1u;
                        }
                        var p: u32 = cnt - 1u;
                        loop {
                            if (p == 0u) {
                                break;
                            }
                            let prev: f32 = best_s[p - 1u];
                            let better: bool = select(acc < prev, acc > prev, larger_better);
                            if (better) {
                                best_s[p] = best_s[p - 1u];
                                best_r[p] = best_r[p - 1u];
                                p = p - 1u;
                            } else {
                                break;
                            }
                        }
                        best_s[p] = acc;
                        best_r[p] = row_g;
                    }
                }
            }
        }
        workgroupBarrier();
        base_row = base_row + TILE_N_T;
    }

    // Emit this thread's per-(query, split) partial top-`want`. Unfilled slots (a
    // split with fewer than `want` live rows) are padded with the worst sentinel and
    // EMPTY_ROW_T so the host skips them during the cross-split merge.
    if (has_q) {
        let sentinel: f32 = select(SENTINEL, -SENTINEL, larger_better);
        let obase: u32 = (qi * num_splits + sp) * want * 2u;
        for (var j: u32 = 0u; j < want; j = j + 1u) {
            if (j < cnt) {
                out_tiled[obase + j * 2u + 0u] = bitcast<u32>(best_s[j]);
                out_tiled[obase + j * 2u + 1u] = best_r[j];
            } else {
                out_tiled[obase + j * 2u + 0u] = bitcast<u32>(sentinel);
                out_tiled[obase + j * 2u + 1u] = EMPTY_ROW_T;
            }
        }
    }
}
