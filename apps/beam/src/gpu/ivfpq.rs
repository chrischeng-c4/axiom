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

/// Owns the two IVF-PQ scan pipelines (PQ ADC + flat residual) and the device /
/// queue to run them. Build once per [`GpuContext`], reuse across queries.
pub struct GpuIvfScanner {
    device: wgpu::Device,
    queue: wgpu::Queue,
    adc_layout: wgpu::BindGroupLayout,
    adc_pipeline: wgpu::ComputePipeline,
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
            flat_layout,
            flat_pipeline,
        }
    }

    /// Score every candidate in `plan` on the GPU, returning the per-candidate
    /// distance in `plan.rows` order — matching [`QueryPlan::cpu_scan`]. An empty
    /// candidate set returns an empty vector without touching the GPU.
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
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}
