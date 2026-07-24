//! GPU candidate scan for IVF-PQ — the hot loop, on wgpu/Metal.
//!
//! [`GpuIvfScanner`] compiles the two `ivfpq.wgsl` kernels (`adc` for PQ,
//! `flat` for exact residual L2) once, then scores a [`QueryPlan`]'s candidates
//! in a single dispatch. The host has already pruned the corpus to the probed
//! cells and packed the candidates into flat arrays; the kernel just does one
//! table-lookup sum (PQ) or one residual L2 (Flat) per candidate. Top-k stays on
//! the CPU (k is tiny) — GPU top-k is a later refinement.
//!
//! The GPU result equals [`QueryPlan::cpu_scan`] within float tolerance, which
//! is the kernel-exactness gate in `tests/ivf_recall.rs`. wgpu usage matches the
//! pinned `wgpu = "24"` conventions in [`crate::gpu`] (blocking `Maintain::Wait`
//! readback, `entry_point: Some(..)`).

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::gpu::GpuContext;
use crate::index::ivf_pq::{IvfPqIndex, QueryPlan, ScanData};
use crate::index::Neighbor;
use crate::payload::Filter;

/// 16-byte scan uniform: `(num_cand, secondary)` where `secondary` is `m` for
/// the ADC kernel or `dim` for the flat kernel. Padded to uniform alignment.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct ScanParams {
    num_cand: u32,
    secondary: u32,
    _pad0: u32,
    _pad1: u32,
}

/// The most PQ subspaces the shared-memory ADC kernel can hold on-chip: its
/// `var<workgroup>` table is `16 * 256` f32 = 16 KB (Metal's threadgroup ceiling
/// is 32 KB). Plans with `m` above this fall back to the per-candidate `adc`
/// kernel — see [`GpuIvfScanner::scan`].
const MAX_SHARED_M: usize = 16;

/// Threads per workgroup in the cell-tiled shared ADC kernel — must equal the
/// `@workgroup_size(SH_WG)` in `ivfpq.wgsl`. Each probed cell is split into
/// tiles of this many candidates (one thread per candidate), so a small
/// `nprobe` still yields many workgroups and keeps the GPU busy.
const SHARED_TILE: usize = 128;

/// Owns the IVF-PQ scan pipelines (per-cell shared ADC, per-candidate ADC
/// fallback, and flat residual) and the device / queue to run them. Build once
/// per [`GpuContext`], reuse across queries.
pub struct GpuIvfScanner {
    device: wgpu::Device,
    queue: wgpu::Queue,
    adc_layout: wgpu::BindGroupLayout,
    adc_pipeline: wgpu::ComputePipeline,
    adc_shared_layout: wgpu::BindGroupLayout,
    adc_shared_pipeline: wgpu::ComputePipeline,
    flat_layout: wgpu::BindGroupLayout,
    flat_pipeline: wgpu::ComputePipeline,
}

impl GpuIvfScanner {
    /// Compile both candidate-scan kernels from `ivfpq.wgsl` on `ctx`'s device.
    pub fn new(ctx: &GpuContext) -> Self {
        let device = ctx.device.clone();
        let queue = ctx.queue.clone();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("beam_ivfpq_kernels"),
            source: wgpu::ShaderSource::Wgsl(include_str!("ivfpq.wgsl").into()),
        });

        // ADC kernel: bindings 0..4.
        let adc_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("beam_ivfpq_adc_bgl"),
            entries: &[
                storage_entry(0, true),  // tables
                storage_entry(1, true),  // codes
                storage_entry(2, true),  // cand_slot
                uniform_entry(3),        // params
                storage_entry(4, false), // out
            ],
        });
        let adc_pipeline = compute_pipeline(&device, &shader, &adc_layout, "adc", "beam_ivfpq_adc");

        // Cell-tiled shared-memory ADC kernel: bindings 10..16.
        let adc_shared_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("beam_ivfpq_adc_shared_bgl"),
                entries: &[
                    storage_entry(10, true),  // tables
                    storage_entry(11, true),  // codes
                    storage_entry(12, true),  // tile_slot
                    storage_entry(13, true),  // tile_base
                    storage_entry(14, true),  // tile_len
                    uniform_entry(15),        // params
                    storage_entry(16, false), // out
                ],
            });
        let adc_shared_pipeline = compute_pipeline(
            &device,
            &shader,
            &adc_shared_layout,
            "adc_shared",
            "beam_ivfpq_adc_shared",
        );

        // Flat kernel: bindings 5..9.
        let flat_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("beam_ivfpq_flat_bgl"),
            entries: &[
                storage_entry(5, true),  // qresid
                storage_entry(6, true),  // resid
                storage_entry(7, true),  // cand_slot
                uniform_entry(8),        // params
                storage_entry(9, false), // out
            ],
        });
        let flat_pipeline =
            compute_pipeline(&device, &shader, &flat_layout, "flat", "beam_ivfpq_flat");

        Self {
            device,
            queue,
            adc_layout,
            adc_pipeline,
            adc_shared_layout,
            adc_shared_pipeline,
            flat_layout,
            flat_pipeline,
        }
    }

    /// Score every candidate in `plan` on the GPU, returning the per-candidate
    /// distance in `plan.rows` order — matching [`QueryPlan::cpu_scan`]. An empty
    /// candidate set returns an empty vector without touching the GPU.
    ///
    /// PQ plans use the per-candidate **global-table** `adc` kernel. On Apple
    /// Silicon this is the *faster* ADC path: the ADC table (`m·256` f32 =
    /// 16–32 KB) stays hot in the GPU's L2, so a one-thread-per-candidate scan
    /// with full occupancy beats staging the table in `var<workgroup>` — the
    /// shared-memory load + `workgroupBarrier` cost more than the cached global
    /// reads they replace (measured ~2× slower here; see [`Self::scan_shared`]).
    /// The shared-memory design still wins on discrete GPUs / large cells and is
    /// kept as [`Self::scan_shared`], validated bit-for-intent against this path.
    pub fn scan(&self, plan: &QueryPlan) -> Vec<f32> {
        let num_cand = plan.rows.len();
        if num_cand == 0 {
            return Vec::new();
        }
        match &plan.data {
            ScanData::Pq { tables, codes, m } => self.dispatch(
                &self.adc_pipeline,
                &self.adc_layout,
                &[0, 1, 2],
                3,
                4,
                bytemuck::cast_slice(tables),
                bytemuck::cast_slice(codes),
                bytemuck::cast_slice(&plan.cand_slot),
                ScanParams {
                    num_cand: num_cand as u32,
                    secondary: *m as u32,
                    _pad0: 0,
                    _pad1: 0,
                },
                num_cand,
            ),
            ScanData::Flat { qresid, resid, dim } => self.dispatch(
                &self.flat_pipeline,
                &self.flat_layout,
                &[5, 6, 7],
                8,
                9,
                bytemuck::cast_slice(qresid),
                bytemuck::cast_slice(resid),
                bytemuck::cast_slice(&plan.cand_slot),
                ScanParams {
                    num_cand: num_cand as u32,
                    secondary: *dim as u32,
                    _pad0: 0,
                    _pad1: 0,
                },
                num_cand,
            ),
        }
    }

    /// Score a PQ plan with the **per-cell shared-memory** ADC kernel — the P0
    /// design: the host tiles each probed cell's candidate block into
    /// workgroup-sized chunks; each workgroup stages that cell's `m·256` ADC
    /// table in `var<workgroup>` once, barriers, then scores one candidate per
    /// thread from the on-chip table. Requires `m ≤ 16` (the 16 KB shared
    /// table); a wider `m` or a [`ScanData::Flat`] plan transparently falls back
    /// to [`Self::scan`]. Returns the identical distances as [`Self::scan`]
    /// (validated to 1e-3 in `tests/ivf_recall.rs`); exposed separately because
    /// on Apple Silicon it is measurably slower than the cached global path, so
    /// it is not the default but remains the correct design for discrete GPUs.
    pub fn scan_shared(&self, plan: &QueryPlan) -> Vec<f32> {
        let num_cand = plan.rows.len();
        if num_cand == 0 {
            return Vec::new();
        }
        match &plan.data {
            ScanData::Pq { tables, codes, m } if *m <= MAX_SHARED_M => {
                self.dispatch_shared(tables, codes, plan, *m)
            }
            // Wide-m PQ or Flat: no shared-table kernel, use the standard path.
            _ => self.scan(plan),
        }
    }

    /// Full GPU `k`-NN: plan the query on the host, scan candidates on the GPU,
    /// top-k on the CPU. Deterministic and equal to [`IvfPqIndex::search_cpu`]
    /// (their candidate distances match; ties aside).
    pub fn search(
        &self,
        index: &IvfPqIndex,
        query: &[f32],
        k: usize,
        nprobe: usize,
    ) -> Vec<Neighbor> {
        if query.len() != index.dim() || index.is_empty() || k == 0 {
            return Vec::new();
        }
        let plan = index.plan(query, nprobe);
        let dist = self.scan(&plan);
        index.topk_candidates(&plan.rows, &dist, k)
    }

    /// Full GPU **filtered** `k`-NN: plan + GPU candidate scan as [`Self::search`],
    /// but keep only candidates from the probed cells whose payload matches
    /// `filter` (applied during the host top-k, where the candidate set is
    /// already pruned and small). Returns the top `min(k, #matching-candidates)`.
    /// Equal to [`IvfPqIndex::search_cpu_filtered`] (their candidate distances
    /// match; ties aside), so with `Refine::Flat` + `nprobe == nlist` it
    /// reproduces the filtered flat oracle.
    pub fn search_filtered(
        &self,
        index: &IvfPqIndex,
        query: &[f32],
        k: usize,
        nprobe: usize,
        filter: &Filter,
    ) -> Vec<Neighbor> {
        if query.len() != index.dim() || index.is_empty() || k == 0 {
            return Vec::new();
        }
        let plan = index.plan(query, nprobe);
        let dist = self.scan(&plan);
        index.topk_candidates_filtered(&plan.rows, &dist, k, filter)
    }

    /// One dispatch: upload the three inputs + params, run `pipeline` over
    /// `num_cand` invocations, read back `num_cand` f32 distances.
    #[allow(clippy::too_many_arguments)]
    fn dispatch(
        &self,
        pipeline: &wgpu::ComputePipeline,
        layout: &wgpu::BindGroupLayout,
        input_bindings: &[u32; 3],
        params_binding: u32,
        out_binding: u32,
        buf0: &[u8],
        buf1: &[u8],
        buf2: &[u8],
        params: ScanParams,
        num_cand: usize,
    ) -> Vec<f32> {
        let b0 = self.storage_from(buf0, "beam_ivf_in0");
        let b1 = self.storage_from(buf1, "beam_ivf_in1");
        let b2 = self.storage_from(buf2, "beam_ivf_in2");
        let params_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("beam_ivf_params"),
                contents: bytemuck::bytes_of(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let out_bytes = (num_cand * std::mem::size_of::<f32>()) as wgpu::BufferAddress;
        let out_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("beam_ivf_out"),
            size: out_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("beam_ivf_readback"),
            size: out_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("beam_ivf_bind_group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: input_bindings[0],
                    resource: b0.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: input_bindings[1],
                    resource: b1.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: input_bindings[2],
                    resource: b2.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: params_binding,
                    resource: params_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: out_binding,
                    resource: out_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("beam_ivf_encoder"),
            });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("beam_ivf_pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            let workgroups = (num_cand as u32).div_ceil(64).max(1);
            cpass.dispatch_workgroups(workgroups, 1, 1);
        }
        encoder.copy_buffer_to_buffer(&out_buffer, 0, &readback, 0, out_bytes);
        self.queue.submit(std::iter::once(encoder.finish()));

        let slice = readback.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv()
            .expect("map_async callback dropped")
            .expect("gpu buffer map failed");

        let data = slice.get_mapped_range();
        let out: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        readback.unmap();
        out
    }

    /// One dispatch of the cell-tiled shared-memory ADC kernel. Splits each
    /// probed cell's candidate block into `SHARED_TILE`-wide tiles (one workgroup
    /// each, so a small `nprobe` still fills the GPU), uploads the tables, codes,
    /// and tile descriptors, runs one workgroup per tile, and reads back the
    /// per-candidate f32 distances in `plan.rows` order.
    fn dispatch_shared(
        &self,
        tables: &[f32],
        codes: &[u32],
        plan: &QueryPlan,
        m: usize,
    ) -> Vec<f32> {
        let num_cand = plan.rows.len();
        // Build the tile descriptors: for each probed cell, one tile per
        // `SHARED_TILE` candidates. Empty cells contribute no tiles.
        let mut tile_slot: Vec<u32> = Vec::new();
        let mut tile_base: Vec<u32> = Vec::new();
        let mut tile_len: Vec<u32> = Vec::new();
        for slot in 0..plan.num_probed {
            let off = plan.cell_offsets[slot];
            let cnt = plan.cell_counts[slot];
            let mut c = 0u32;
            while c < cnt {
                let len = (cnt - c).min(SHARED_TILE as u32);
                tile_slot.push(slot as u32);
                tile_base.push(off + c);
                tile_len.push(len);
                c += SHARED_TILE as u32;
            }
        }
        let num_tiles = tile_slot.len();

        let params = ScanParams {
            num_cand: num_tiles as u32,
            secondary: m as u32,
            _pad0: 0,
            _pad1: 0,
        };

        let b_tables = self.storage_from(bytemuck::cast_slice(tables), "beam_ivf_sh_tables");
        let b_codes = self.storage_from(bytemuck::cast_slice(codes), "beam_ivf_sh_codes");
        let b_tile_slot = self.storage_from(bytemuck::cast_slice(&tile_slot), "beam_ivf_sh_tslot");
        let b_tile_base = self.storage_from(bytemuck::cast_slice(&tile_base), "beam_ivf_sh_tbase");
        let b_tile_len = self.storage_from(bytemuck::cast_slice(&tile_len), "beam_ivf_sh_tlen");
        let params_buf = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("beam_ivf_sh_params"),
                contents: bytemuck::bytes_of(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let out_bytes = (num_cand * std::mem::size_of::<f32>()) as wgpu::BufferAddress;
        let out_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("beam_ivf_sh_out"),
            size: out_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("beam_ivf_sh_readback"),
            size: out_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("beam_ivf_sh_bind_group"),
            layout: &self.adc_shared_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: b_tables.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: b_codes.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: b_tile_slot.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 13,
                    resource: b_tile_base.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 14,
                    resource: b_tile_len.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 15,
                    resource: params_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 16,
                    resource: out_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("beam_ivf_sh_encoder"),
            });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("beam_ivf_sh_pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.adc_shared_pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            // One workgroup per cell-tile: each loads its cell's ADC table into
            // shared memory, then one thread scores one candidate.
            cpass.dispatch_workgroups((num_tiles as u32).max(1), 1, 1);
        }
        encoder.copy_buffer_to_buffer(&out_buffer, 0, &readback, 0, out_bytes);
        self.queue.submit(std::iter::once(encoder.finish()));

        let slice = readback.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv()
            .expect("map_async callback dropped")
            .expect("gpu buffer map failed");

        let data = slice.get_mapped_range();
        let out: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        readback.unmap();
        out
    }

    fn storage_from(&self, bytes: &[u8], label: &str) -> wgpu::Buffer {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytes,
                usage: wgpu::BufferUsages::STORAGE,
            })
    }
}

/// A `COMPUTE`-visible storage-buffer bind-group-layout entry at `binding`.
fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

/// A `COMPUTE`-visible uniform-buffer bind-group-layout entry at `binding`.
fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

/// Build a compute pipeline for `entry_point` bound to `layout`.
fn compute_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    layout: &wgpu::BindGroupLayout,
    entry_point: &str,
    label: &str,
) -> wgpu::ComputePipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[layout],
        push_constant_ranges: &[],
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(&pipeline_layout),
        module: shader,
        entry_point: Some(entry_point),
        compilation_options: wgpu::PipelineCompilationOptions {
            zero_initialize_workgroup_memory: false,
            ..Default::default()
        },
        cache: None,
    })
}
