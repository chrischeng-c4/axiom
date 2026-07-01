// Beam IVF candidate-scan kernels — the hot loop of ANN search on the GPU.
//
// After the host prunes the corpus to the candidates in the `nprobe` probed
// cells, each candidate still needs a distance. Both kernels below score ONE
// candidate per invocation (workgroup size 64, `i < num_cand` guard for the
// ragged final workgroup) and write it to `out_dist[i]`; top-k stays on the CPU
// (k is tiny). Each candidate carries a `cand_slot` selecting which probed
// cell's table / query-residual it is scored against.
//
// Two entry points, ONE module — so their (group, binding) numbers must be
// disjoint (naga tree-shakes the bindings the selected entry point does not
// use, and each pipeline supplies a layout for only its own bindings):
//
//   `adc`  (PQ, bindings 0-4)  — asymmetric distance via table lookups:
//       dist[i] = Σ_s tables[cand_slot[i]·m·256 + s·256 + codes[i·m + s]]
//
//   `flat` (Flat, bindings 5-9) — exact residual L2:
//       dist[i] = Σ_d (qresid[cand_slot[i]·dim + d] − resid[i·dim + d])²
//
// Both reproduce `QueryPlan::cpu_scan` in `index/ivf_pq.rs` VERBATIM, so the GPU
// and CPU reference candidate distances agree (the kernel-exactness test).

// ---- PQ ADC scan (entry point `adc`) --------------------------------------

struct AdcParams {
    num_cand: u32,   // number of candidates
    m: u32,          // PQ subspaces (code bytes per candidate)
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(0) var<storage, read> adc_tables: array<f32>;   // num_probed * m * 256
@group(0) @binding(1) var<storage, read> adc_codes: array<u32>;    // num_cand * m (one code per u32)
@group(0) @binding(2) var<storage, read> adc_slot: array<u32>;     // num_cand
@group(0) @binding(3) var<uniform> adc_params: AdcParams;
@group(0) @binding(4) var<storage, read_write> adc_out: array<f32>; // num_cand

@compute @workgroup_size(64)
fn adc(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i: u32 = gid.x;
    if (i >= adc_params.num_cand) {
        return;
    }
    let m: u32 = adc_params.m;
    let table_base: u32 = adc_slot[i] * m * 256u;
    let code_base: u32 = i * m;
    var acc: f32 = 0.0;
    for (var s: u32 = 0u; s < m; s = s + 1u) {
        let c: u32 = adc_codes[code_base + s];
        acc = acc + adc_tables[table_base + s * 256u + c];
    }
    adc_out[i] = acc;
}

// ---- Flat residual scan (entry point `flat`) ------------------------------

struct FlatParams {
    num_cand: u32,   // number of candidates
    dim: u32,        // vector dimension
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(5) var<storage, read> flat_qresid: array<f32>;   // num_probed * dim
@group(0) @binding(6) var<storage, read> flat_resid: array<f32>;    // num_cand * dim
@group(0) @binding(7) var<storage, read> flat_slot: array<u32>;     // num_cand
@group(0) @binding(8) var<uniform> flat_params: FlatParams;
@group(0) @binding(9) var<storage, read_write> flat_out: array<f32>; // num_cand

@compute @workgroup_size(64)
fn flat(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i: u32 = gid.x;
    if (i >= flat_params.num_cand) {
        return;
    }
    let dim: u32 = flat_params.dim;
    let q_base: u32 = flat_slot[i] * dim;
    let r_base: u32 = i * dim;
    var acc: f32 = 0.0;
    for (var d: u32 = 0u; d < dim; d = d + 1u) {
        let diff: f32 = flat_qresid[q_base + d] - flat_resid[r_base + d];
        acc = acc + diff * diff;
    }
    flat_out[i] = acc;
}
