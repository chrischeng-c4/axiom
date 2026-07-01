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
