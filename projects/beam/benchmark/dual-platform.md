# Beam Dual-Platform Strategy (Mac + NVIDIA)

Goal: beam runs GPU vector search on **Apple Silicon (Mac)** *and* **NVIDIA
(CUDA-class)** hardware. This is beam's competitive wedge — every other GPU
vector system (Faiss-GPU, cuVS/CAGRA, Milvus-GPU) is **CUDA/NVIDIA-only** and
does not run on Metal at all.

## How it works today: one wgpu codebase, three backends

beam's GPU engine is written once in WGSL + `wgpu = "24"` and is **backend-
agnostic** — `GpuContext` opens the instance with `Backends::PRIMARY` and
`PowerPreference::HighPerformance`, so wgpu picks the right native backend per
host:

| Host | wgpu backend | Status |
|---|---|---|
| **Apple Silicon (Mac)** | **Metal** | ✅ verified (Apple M1 Max) — all tests + bench |
| **NVIDIA GPU (Linux/Win)** | **Vulkan** | 🟡 same code, backend-agnostic; not yet run on NVIDIA hardware here |
| **Windows** | DX12 | 🟡 same code; untested here |
| CPU-only host | none | graceful skip (`GpuContext::new() -> None`) |

Inspect the resolved backend on any machine:

```
$ beam info
beam 0.4.x (aarch64-apple-darwin, git …)
GPU backend: Metal
GPU device:  Apple M1 Max
wgpu selects the backend automatically: Metal (Apple Silicon),
Vulkan (NVIDIA/Linux), DX12 (Windows) — one WGSL codebase.
```

On an NVIDIA box the same binary prints `GPU backend: Vulkan` / the RTX/A100
device — no rebuild, no code change. That is "Mac + NVIDIA" satisfied by the
portable path.

## "CUDA" — two honest readings

1. **NVIDIA GPU support (portable) = today, via Vulkan.** wgpu drives NVIDIA
   GPUs through Vulkan. The kernels, IVF-PQ, filtered search, CRUD — all run
   unchanged. This is real NVIDIA support; it is just not the *CUDA* API.
   Remaining work is **validation on NVIDIA hardware/CI**, not new code.
2. **Native CUDA / cuVS (max perf) = B2, deferred.** For last-mile NVIDIA
   performance (CAGRA-class graph index, cuVS kernels), a native CUDA backend
   would sit behind the existing `VectorIndex` trait as a `cuda` cargo feature
   (cudarc / cuVS FFI). It needs an NVIDIA GPU + CUDA toolkit to build and test,
   which this dev environment lacks, so it is **scaffolded-in-design, not built**
   — pursued once NVIDIA CI/hardware is available. Do not add a broken/untestable
   CUDA dependency to the default build.

## Where the pluggability lives

- `VectorIndex` trait (`src/index/mod.rs`) — the seam every backend implements.
- `GpuContext` (`src/gpu/mod.rs`) — backend-agnostic device/adapter; add a
  `--backend` override here if forcing a specific wgpu backend is ever needed.
- A future `CudaFlatIndex` / `CudaIvfIndex` (`#[cfg(feature = "cuda")]`) would
  join `CpuFlatIndex` / `GpuFlatIndex` / `IvfPqIndex` behind the same trait.

## Verdict

Goal 3 ("support Mac and CUDA") is **architecturally met for the portable
path**: one codebase, Metal on Mac (verified) + Vulkan on NVIDIA (same code,
pending hardware validation). Native-CUDA/cuVS is an optional max-perf backend
(B2) behind the trait, deferred until NVIDIA hardware is in the loop.
