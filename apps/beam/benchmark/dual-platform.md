<!-- HANDWRITE-BEGIN gap="missing-generator:logic:6d546b5b" tracker="pending-tracker" reason="scaffold for apps/beam/benchmark/dual-platform.md — fill in by hand and update tracker when codegen is ready" -->
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
2. **Native CUDA (max perf) = present as a compile-verified backend.** A native
   CUDA backend now exists behind the optional `cuda` cargo feature — see the
   next section. It is the CUDA driver-API path (cuVS/Faiss-GPU territory), for
   last-mile NVIDIA performance, and sits behind the same `VectorIndex` trait as
   every other backend.

## Native CUDA backend (`--features cuda`) — compile-verified only

`src/index/cuda.rs` adds `CudaFlatIndex`: the exact brute-force scan of
`CpuFlatIndex` / `GpuFlatIndex`, but the per-row distances are computed by a
**native CUDA C kernel** launched through the NVIDIA driver API (via the
`cudarc` crate), not WGSL-on-Vulkan. It is behind the optional `cuda` cargo
feature and is **NOT in the default build** — the Mac/CI default pulls no
cudarc, so the normal build (and every existing test) is byte-for-byte
unaffected.

**Honest status — read this carefully:**

- **Compiles on this Mac** (`aarch64-apple-darwin`):
  `cargo check -p beam --features cuda` succeeds here. `cudarc` is pinned with
  `default-features = false, features = ["cuda-12060", "dynamic-loading",
  "driver", "nvrtc"]`, so `libcuda` / `libnvrtc` are `dlopen`ed lazily at
  RUNTIME (no CUDA toolkit or `nvcc` needed at build time) and the distance
  kernel is compiled to PTX at RUNTIME by NVRTC. That is why it builds with no
  NVIDIA hardware, driver, or toolkit present.
- **Requires an NVIDIA GPU + driver at RUNTIME.** With no driver present (this
  Mac), `CudaContext::new(0)` returns an `Err`, so `CudaFlatIndex::new` fails
  gracefully and the runtime-skip test skips — mirroring the wgpu tests'
  GPU-less skip.
- **NOT runtime-verified in this Mac dev environment.** The CUDA path has **not
  been run on a GPU here** — it is **compile-verified only**. The parity
  assertion (`CudaFlatIndex::search_knn == CpuFlatIndex::search_knn`, exact flat,
  within 1e-3) is written and compiles, but only executes on an NVIDIA host;
  do not read this as "run" or "benchmarked" on a GPU.

## Where the pluggability lives

- `VectorIndex` trait (`src/index/mod.rs`) — the seam every backend implements.
- `GpuContext` (`src/gpu/mod.rs`) — backend-agnostic device/adapter; add a
  `--backend` override here if forcing a specific wgpu backend is ever needed.
- `CudaFlatIndex` (`#[cfg(feature = "cuda")]`, `src/index/cuda.rs`) now joins
  `CpuFlatIndex` / `GpuFlatIndex` / `IvfPqIndex` / `HnswIndex` behind the same
  `VectorIndex` trait — the native-CUDA seam is filled (compile-verified). A
  future `CudaIvfIndex` / CAGRA graph index would slot in the same way.

## Verdict

Goal 3 ("support Mac and CUDA") is **architecturally met for the portable
path**: one codebase, Metal on Mac (verified) + Vulkan on NVIDIA (same code,
pending hardware validation). The native-CUDA path is now present too, as an
optional max-perf backend behind the `cuda` feature — **compile-verified on this
Mac (cudarc dynamic-loading + runtime NVRTC), runtime requires an NVIDIA GPU +
driver, and NOT runtime-verified in this Mac dev environment.**

<!-- HANDWRITE-END -->
