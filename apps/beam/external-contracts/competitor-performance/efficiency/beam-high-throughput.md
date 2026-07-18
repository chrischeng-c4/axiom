---
id: beam-competitor-performance-ddd-ec
summary: Competitive performance — Beam high-throughput pipeline tests guarantee that the DDD architecture (Ports and Adapters) imposes near-zero overhead. Async NVMe I/O (via io_uring) and batched GPU execution (Tiled GEMM) must achieve saturation, beating baseline CPU-only vector engines in batched RAG query throughput.
fill_sections: [e2e-test, tool-contract]
---

# EC: Competitive High-Throughput Performance (DDD)

Competitive throughput gate: Beam must hold its own batched query throughput (QPS) targets under heavy RAG loads while ensuring no single hardware component idles during the pipeline. The `meter`-wrapped cargo perf gate verifies the simultaneous saturation of Disk (NVMe), RAM (Pinned memory buffers), CPU (Graph traversal), and GPU (Tiled GEMM refinement). 

This External Contract enforces strict performance floors across the DDD architecture:
1. **Domain Layer Overhead**: The clean architecture abstractions (Aggregates, Domain Services) must compile down to zero-cost abstractions, introducing < 1% latency overhead.
2. **Infrastructure Saturation (Pipeline Overlap)**: GPU compute (`WgpuDistanceEngine`) must perfectly hide Disk I/O latency (`IoUringVectorRepository`).
3. **Out-of-Core Memory Scalability**: RAM usage must remain constant for a fixed `HnswNavigator` graph, regardless of the `ColdPayload` vector size on NVMe Disk.
4. **GPU Batch Scaling**: `DistanceCalculator` port implementations must maintain linear QPS scaling up to the VRAM and PCIe bandwidth limits.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: beam-competitor-performance-ddd-overhead
    capability_id: competitor-performance
    claim_id: competitive-throughput-ddd-zero-cost
    contract_id: search-efficiency-ddd-overhead
    category: efficiency
    test_path: apps/beam/tests/benchmark_beam_competitor_performance_ddd.rs
    command: "cd apps/beam && ../../target/debug/vat run ec-efficiency-meter --scenario ddd-overhead"
    assertions:
      - "The Domain Service `PipelineScheduler` introduces less than 1% latency overhead compared to a tightly-coupled monolithic pipeline."
      - "The Hexagonal `VectorRepository` and `DistanceCalculator` trait dynamic dispatch (or static monomorphization) costs are invisible in the flamegraph."

  - id: beam-competitor-performance-pipeline-overlap
    capability_id: competitor-performance
    claim_id: competitive-throughput-gate-saturate-four-pillars
    contract_id: search-efficiency-batched-rag-throughput
    category: efficiency
    test_path: apps/beam/tests/benchmark_beam_competitor_performance_overlap.rs
    command: "cd apps/beam && ../../target/debug/vat run ec-efficiency-meter --scenario pipeline-overlap"
    assertions:
      - "Async `IoUringVectorRepository` fetches Uncompressed Vectors from NVMe concurrently with `WgpuDistanceEngine` calculations."
      - "GPU Tiled GEMM processing time strictly overlaps with NVMe fetch time on subsequent batches (Pipeline Stall < 5%)."
      - "PCIe DMA transfers (RAM to VRAM) achieve at least 80% of peak bandwidth (e.g. 12GB/s on PCIe Gen4 x16)."

  - id: beam-competitor-performance-memory-footprint
    capability_id: competitor-performance
    claim_id: competitive-throughput-out-of-core-scaling
    contract_id: search-efficiency-memory-footprint
    category: efficiency
    test_path: apps/beam/tests/benchmark_beam_competitor_performance_memory.rs
    command: "cd apps/beam && ../../target/debug/vat run ec-efficiency-meter --scenario out-of-core"
    assertions:
      - "Host RAM consumption is bounded strictly to the size of the `HnswNavigator` graph and compressed PQ codes."
      - "Scaling the `ColdPayload` from 10M to 100M vectors increases NVMe footprint but triggers less than 15% increase in Host RAM usage."
      - "GPU VRAM allocation never exceeds the statically allocated PinnedBuffer size for batched GEMM."

  - id: beam-competitor-performance-gpu-batch-scaling
    capability_id: competitor-performance
    claim_id: competitive-throughput-gpu-scaling
    contract_id: search-efficiency-gpu-scaling
    category: efficiency
    test_path: apps/beam/tests/benchmark_beam_competitor_performance_gpu.rs
    command: "cd apps/beam && ../../target/debug/vat run ec-efficiency-meter --scenario gpu-batching"
    assertions:
      - "System throughput (QPS) scales linearly as `QueryBatch` size increases from 1 to 512."
      - "Peak batched throughput demonstrates at least 10x QPS advantage over the `HnswCpuIndex` baseline from Lumen."
      - "Exact refinement parity: The final Top-K results emitted by the GPU exactly match the CPU oracle for the identical candidate set."
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: beam-meter-search-efficiency-throughput
    tool: meter
    manifest: meter-search-efficiency-throughput.toml
    category: efficiency
    command: "cd apps/beam && ../../target/debug/vat run ec-efficiency-meter"
    native:
      version: 1
      project: beam
      source_contract: beam-competitor-performance-pipeline-overlap
      delegate_command: "cd apps/beam && ../../target/debug/vat run ec-efficiency-meter --scenario pipeline-overlap"
  
  - id: beam-meter-search-efficiency-memory
    tool: meter
    manifest: meter-search-efficiency-memory.toml
    category: efficiency
    command: "cd apps/beam && ../../target/debug/vat run ec-efficiency-meter"
    native:
      version: 1
      project: beam
      source_contract: beam-competitor-performance-memory-footprint
      delegate_command: "cd apps/beam && ../../target/debug/vat run ec-efficiency-meter --scenario out-of-core"
```
