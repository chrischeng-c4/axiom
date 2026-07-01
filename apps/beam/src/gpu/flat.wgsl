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
