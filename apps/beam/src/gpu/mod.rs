//! GPU-native flat vector search on wgpu (Metal on this Mac).
//!
//! [`GpuContext`] owns the wgpu `Instance` / `Adapter` / `Device` / `Queue`.
//! [`GpuContext::new`] returns `None` when no adapter is reachable, so callers
//! (bench + tests) can skip gracefully on a GPU-less host.
//!
//! [`GpuFlatIndex`] uploads a collection's row-major vectors to a GPU **storage
//! buffer once** at construction. Each `search_knn` uploads the (small) query,
//! dispatches the `flat.wgsl` compute kernel (one invocation per DB row,
//! workgroup size 64), reads the `n` per-row distances back, and runs the shared
//! CPU top-k. Scores use the SAME convention as the CPU oracle
//! ([`crate::collection::Metric::code`]), so GPU and CPU top-k agree.
//!
//! wgpu API usage is matched to the pinned `wgpu = "24"` already in the
//! workspace (see `crates/cclab-grid-render-webgpu`): `Instance::new(&desc)`,
//! `request_adapter` → `Option`, `request_device(&desc, None)`,
//! `Maintain::Wait` blocking readback, `entry_point: Some(..)` on the compute
//! pipeline.

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::collection::{l2_normalize, Collection, Metric};
use crate::index::{topk, Neighbor, VectorIndex};

/// Owns the wgpu handles for one GPU (adapter). Cheap to clone the `Device` /
/// `Queue` out of (both are Arc-backed handles).
pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GpuContext {
    /// Acquire the primary GPU adapter (Metal on macOS) and open a device.
    /// Returns `None` if no adapter is present or the device request fails —
    /// the graceful-skip signal for GPU-less CI.
    pub fn new() -> Option<Self> {
        pollster::block_on(Self::new_async())
    }

    async fn new_async() -> Option<Self> {
        // `Backends::PRIMARY` = Metal | Vulkan | DX12 — Metal on this Mac.
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await?;
        // Request exactly the adapter's limits so large storage buffers (the DB)
        // are bindable up to what Metal supports.
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("beam_gpu_device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: adapter.limits(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .ok()?;
        Some(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }

    /// `(backend, name)` from `adapter.get_info()` — backend via `Debug`
    /// (e.g. `"Metal"`), name the device string (e.g. `"Apple M-series GPU"`).
    pub fn adapter_info(&self) -> (String, String) {
        let info = self.adapter.get_info();
        (format!("{:?}", info.backend), info.name)
    }
}

/// The `Params` uniform handed to the kernel: row count, dimension, metric code.
/// 16 bytes (uniform-buffer alignment friendly). `_pad` rounds to 16.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Params {
    n: u32,
    dim: u32,
    metric: u32,
    _pad: u32,
}

/// GPU flat (brute-force) index. Owns the uploaded DB storage buffer and the
/// compute pipeline; `search_knn` is per-query upload → dispatch → readback.
pub struct GpuFlatIndex {
    device: wgpu::Device,
    queue: wgpu::Queue,
    dim: usize,
    n: usize,
    metric: Metric,
    external_ids: Vec<String>,
    /// The whole corpus (`n * dim` f32) uploaded once as a read-only storage buffer.
    db_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl GpuFlatIndex {
    /// Build the index: upload `collection`'s vectors to a GPU storage buffer
    /// once and compile the flat kernel. Reuses `ctx`'s device/queue.
    pub fn new(ctx: &GpuContext, collection: &Collection) -> Self {
        let device = ctx.device.clone();
        let queue = ctx.queue.clone();
        let dim = collection.dim();
        let n = collection.len();
        let metric = collection.metric();

        // Upload the whole corpus once. `mapped_at_creation` via the util init
        // helper is the one-shot upload path; the buffer is read-only in-shader.
        let db_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("beam_db_vectors"),
            contents: bytemuck::cast_slice(collection.data()),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("beam_flat_kernel"),
            source: wgpu::ShaderSource::Wgsl(include_str!("flat.wgsl").into()),
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("beam_flat_bgl"),
                entries: &[
                    // 0: db vectors (storage, read-only)
                    storage_entry(0, true),
                    // 1: query vector (storage, read-only)
                    storage_entry(1, true),
                    // 2: params (uniform)
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // 3: out distances (storage, read_write)
                    storage_entry(3, false),
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("beam_flat_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("beam_flat_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Self {
            device,
            queue,
            dim,
            n,
            metric,
            external_ids: collection.external_ids().to_vec(),
            db_buffer,
            bind_group_layout,
            pipeline,
        }
    }

    /// Number of stored vectors.
    pub fn len(&self) -> usize {
        self.n
    }

    /// Whether the index is empty.
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// Run the kernel over every row and return the raw `n` per-row distances.
    fn compute_distances(&self, query: &[f32]) -> Vec<f32> {
        // Cosine normalizes the query on the host so the (unit) DB rows give a
        // true cosine similarity; L2/Dot upload the query as-is.
        let q = match self.metric {
            Metric::Cosine => l2_normalize(query),
            _ => query.to_vec(),
        };

        let query_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("beam_query"),
                contents: bytemuck::cast_slice(&q),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let params = Params {
            n: self.n as u32,
            dim: self.dim as u32,
            metric: self.metric.code(),
            _pad: 0,
        };
        let params_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("beam_params"),
                    contents: bytemuck::bytes_of(&params),
                    usage: wgpu::BufferUsages::UNIFORM,
                });

        let out_bytes = (self.n * std::mem::size_of::<f32>()) as wgpu::BufferAddress;
        let out_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("beam_out_dist"),
            size: out_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("beam_readback"),
            size: out_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("beam_flat_bind_group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.db_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: query_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: out_buffer.as_entire_binding(),
                },
            ],
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("beam_flat_encoder"),
            });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("beam_flat_pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            // One invocation per row, workgroup size 64 → ceil(n / 64) workgroups.
            let workgroups = (self.n as u32).div_ceil(64).max(1);
            cpass.dispatch_workgroups(workgroups, 1, 1);
        }
        encoder.copy_buffer_to_buffer(&out_buffer, 0, &readback, 0, out_bytes);
        self.queue.submit(std::iter::once(encoder.finish()));

        // Blocking readback: map, then `Maintain::Wait` drives the queue to
        // completion and runs the map callback synchronously on this thread.
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
        let scores: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        readback.unmap();
        scores
    }
}

/// A `COMPUTE`-visible storage-buffer bind-group-layout entry (`read_only`
/// picks read vs read_write).
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

impl VectorIndex for GpuFlatIndex {
    fn search_knn(&self, query: &[f32], k: usize) -> Vec<Neighbor> {
        if query.len() != self.dim || self.n == 0 || k == 0 {
            return Vec::new();
        }
        let scores = self.compute_distances(query);
        topk(&scores, self.metric, k, &self.external_ids)
    }
}
