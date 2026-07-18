// Beam IVF candidate-scan kernels — the hot loop of ANN search on the GPU.
//
// After the host prunes the corpus to the candidates in the `nprobe` probed
// cells, each candidate still needs a distance. Both kernels below score ONE
// candidate per invocation (workgroup size 64, `i < num_cand` guard for the
// ragged final workgroup) and write it to `out_dist[i]`; top-k stays on the CPU
// (k is tiny). Each candidate carries a `cand_slot` selecting which probed
// cell's table / query-residual it is scored against.
//
// Three entry points, ONE module — so their (group, binding) numbers must be
// disjoint (naga tree-shakes the bindings the selected entry point does not
// use, and each pipeline supplies a layout for only its own bindings):
//
//   `adc_shared` (PQ, bindings 10-16) — the FAST default ADC scan. The host
//       tiles each probed cell's candidate block into `SH_WG`-sized chunks and
//       dispatches ONE workgroup per tile; every workgroup belongs to exactly
//       one cell. The workgroup cooperatively loads that cell's `m·256` ADC
//       table into `var<workgroup>` (fast on-chip memory) ONCE, barriers, then
//       each thread scores ONE candidate with `m` lookups into the SHARED table
//       (not global). This turns the old kernel's `m` uncoalesced global loads
//       per candidate into `m` on-chip hits WHILE keeping one-thread-per-
//       candidate occupancy (many tiles ⇒ the whole GPU stays busy). Requires
//       `m ≤ 16` (the 16 KB shared table); larger `m` falls back to `adc`.
//
//   `adc`  (PQ, bindings 0-4)  — per-candidate ADC via global table lookups;
//       the m>16 fallback and the shape the `adc_shared` result must match:
//       dist[i] = Σ_s tables[cand_slot[i]·m·256 + s·256 + codes[i·m + s]]
//
//   `flat` (Flat, bindings 5-9) — exact residual L2:
//       dist[i] = Σ_d (qresid[cand_slot[i]·dim + d] − resid[i·dim + d])²
//
// All reproduce `QueryPlan::cpu_scan` in `index/ivf_pq.rs` VERBATIM, so the GPU
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

// ---- PQ ADC scan, cell-tiled workgroups + shared table (entry `adc_shared`)
//
// The fast path. The host splits each probed cell's candidate block into
// `SH_WG`-wide TILES and dispatches one workgroup per tile; `tile_slot[wid]`
// names the cell, `tile_base[wid]` the tile's first global candidate index, and
// `tile_len[wid]` how many of the `SH_WG` lanes are live. The `SH_WG` threads
// cooperatively copy the cell's `m·256` ADC table out of global `sh_tables`
// into the on-chip `sh_table`, barrier, then thread `lid` scores the ONE
// candidate `tile_base + lid` with `m` lookups that now hit SHARED memory. Many
// tiles ⇒ one-thread-per-candidate occupancy; cell-aligned tiles ⇒ the table is
// loaded once per tile and reused. `sh_out[cand]` is written exactly once (tiles
// partition `0..num_cand`), so the result equals the `adc` kernel and
// `QueryPlan::cpu_scan`.

// 16 KB shared table: max m=16 subspaces × 256 centroids of f32. Metal's
// threadgroup memory ceiling is 32 KB, so this leaves comfortable headroom; the
// host uses the per-candidate `adc` fallback whenever m > 16.
const SH_MAX_TABLE: u32 = 16u * 256u;
const SH_WG: u32 = 128u;

struct SharedParams {
    num_tiles: u32,  // workgroups dispatched == number of cell tiles
    m: u32,          // PQ subspaces (code bytes per candidate), must be <= 16
    _pad0: u32,
    _pad1: u32,
};

@group(0) @binding(10) var<storage, read> sh_tables: array<f32>;   // num_probed * m * 256
@group(0) @binding(11) var<storage, read> sh_codes: array<u32>;    // num_cand * m
@group(0) @binding(12) var<storage, read> tile_slot: array<u32>;   // num_tiles
@group(0) @binding(13) var<storage, read> tile_base: array<u32>;   // num_tiles
@group(0) @binding(14) var<storage, read> tile_len: array<u32>;    // num_tiles
@group(0) @binding(15) var<uniform> sh_params: SharedParams;
@group(0) @binding(16) var<storage, read_write> sh_out: array<f32>; // num_cand

var<workgroup> sh_table: array<f32, SH_MAX_TABLE>;

@compute @workgroup_size(SH_WG)
fn adc_shared(
    @builtin(workgroup_id) wid: vec3<u32>,
    @builtin(local_invocation_id) lid: vec3<u32>,
) {
    let tile: u32 = wid.x;
    let slot: u32 = tile_slot[tile];
    let m: u32 = sh_params.m;
    let table_len: u32 = m * 256u;
    let table_base: u32 = slot * table_len;

    // Cooperative load of this cell's ADC table into shared memory. All lanes
    // reach the barrier (the load loop is uniform over the workgroup).
    for (var idx: u32 = lid.x; idx < table_len; idx = idx + SH_WG) {
        sh_table[idx] = sh_tables[table_base + idx];
    }
    workgroupBarrier();

    // One thread per candidate in this tile; padding lanes (lid >= len) idle.
    if (lid.x < tile_len[tile]) {
        let cand: u32 = tile_base[tile] + lid.x;
        let code_base: u32 = cand * m;
        var acc: f32 = 0.0;
        for (var s: u32 = 0u; s < m; s = s + 1u) {
            let code: u32 = sh_codes[code_base + s];
            acc = acc + sh_table[s * 256u + code];
        }
        sh_out[cand] = acc;
    }
}
