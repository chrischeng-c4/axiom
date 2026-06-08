// Brute-force kNN compute shader for lumen vector search.
//
// Inputs:
//   - binding 0: storage<read> query     — [f32; dim]
//   - binding 1: storage<read> vectors   — flat [f32; n * dim]
//   - binding 2: storage<read_write> out — [f32; n]   (one distance per vector)
//   - binding 3: uniform<read>   params  — { n: u32, dim: u32, metric: u32 }
//
// Output:
//   out[i] = distance(query, vectors[i*dim ..])
//
// Metric encoding (matches `VectorMetric`):
//   0 = cosine  → out = 1 - cos(q, v)
//   1 = dot     → out = -dot(q, v)
//   2 = l2      → out = sqrt(sum (q-v)^2)
//
// One workgroup of 64 threads handles 64 consecutive vectors. Top-K
// selection runs on the host after a buffer readback.

struct Params {
    n: u32,
    dim: u32,
    metric: u32,
    _pad: u32,
};

@group(0) @binding(0) var<storage, read> query   : array<f32>;
@group(0) @binding(1) var<storage, read> vectors : array<f32>;
@group(0) @binding(2) var<storage, read_write> out_dist : array<f32>;
@group(0) @binding(3) var<uniform> params : Params;

@compute @workgroup_size(64)
fn kmain(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i: u32 = gid.x;
    if (i >= params.n) {
        return;
    }
    let base: u32 = i * params.dim;
    var dot_qv: f32 = 0.0;
    var dot_qq: f32 = 0.0;
    var dot_vv: f32 = 0.0;
    var l2_acc: f32 = 0.0;
    for (var d: u32 = 0u; d < params.dim; d = d + 1u) {
        let q: f32 = query[d];
        let v: f32 = vectors[base + d];
        dot_qv = dot_qv + q * v;
        dot_qq = dot_qq + q * q;
        dot_vv = dot_vv + v * v;
        let diff: f32 = q - v;
        l2_acc = l2_acc + diff * diff;
    }
    var dist: f32 = 0.0;
    if (params.metric == 0u) {
        // cosine
        let denom: f32 = max(sqrt(dot_qq) * sqrt(dot_vv), 1e-30);
        dist = 1.0 - (dot_qv / denom);
    } else if (params.metric == 1u) {
        // dot — negate so smaller = closer = higher similarity
        dist = -dot_qv;
    } else {
        // l2
        dist = sqrt(l2_acc);
    }
    out_dist[i] = dist;
}
