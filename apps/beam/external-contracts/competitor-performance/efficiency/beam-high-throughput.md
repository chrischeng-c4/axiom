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
    test_path: apps/beam/tests/benchmark_beam_competitor_performance_ddd_overhead.rs
    command: "cd apps/beam && ../../target/debug/vat run --scenario ddd-overhead # cargo test"
    assertions:
      - "The execution time ratio between the PipelineScheduler query batch execution and a monolithic inline direct loop remains under 1.3x."
      - "Dynamic dispatch overhead of the VectorRepository and DistanceCalculator traits does not exceed 30% of total query latency."

  - id: beam-competitor-performance-pipeline-overlap
    capability_id: competitor-performance
    claim_id: competitive-throughput-gate-saturate-four-pillars
    contract_id: search-efficiency-batched-rag-throughput
    category: efficiency
    test_path: apps/beam/tests/benchmark_beam_competitor_performance_pipeline_overlap.rs
    command: "cd apps/beam && ../../target/debug/vat run --scenario pipeline-overlap # cargo test"
    assertions:
      - "Pipelined scheduler execution hides Disk I/O latency concurrently with GPU computations."
      - "The total elapsed time for pipelined execution is shorter than sequential execution of the same batches."

  - id: beam-competitor-performance-memory-footprint
    capability_id: competitor-performance
    claim_id: competitive-throughput-out-of-core-scaling
    contract_id: search-efficiency-memory-footprint
    category: efficiency
    test_path: apps/beam/tests/benchmark_beam_competitor_performance_out_of_core.rs
    command: "cd apps/beam && ../../target/debug/vat run --scenario out-of-core # cargo test"
    assertions:
      - "Host RAM consumption is bounded and does not grow linearly with the number of vectors stored in the IoUringVectorRepository."
      - "The peak VRAM allocation remains bounded within the GPU context limit during active query batches."

  - id: beam-competitor-performance-gpu-batch-scaling
    capability_id: competitor-performance
    claim_id: competitive-throughput-gpu-scaling
    contract_id: search-efficiency-gpu-scaling
    category: efficiency
    test_path: apps/beam/tests/benchmark_beam_competitor_performance_gpu_batching.rs
    command: "cd apps/beam && ../../target/debug/vat run --scenario gpu-batching # cargo test"
    assertions:
      - "The system throughput (QPS) of a batch size of 64 is at least 2x higher than a batch size of 1."
      - "The GPU distance engine demonstrates at least 1.5x throughput (QPS) advantage over CPU distance calculations for a batch size of 128."
      - "Exact refinement parity: The Top-K query results returned by the GPU engine exactly match the CPU reference implementation."
```
## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: beam-meter-search-efficiency-ddd
    tool: meter
    manifest: meter-search-efficiency-ddd.toml
    category: efficiency
    command: "cd apps/beam && ../../target/debug/vat run ec-efficiency-meter-ddd"
    native:
      version: 1
      project: beam
      source_contract: beam-competitor-performance-ddd-overhead
      delegate_command: "cd apps/beam && ../../target/debug/vat run --scenario ddd-overhead"
  
  - id: beam-meter-search-efficiency-overlap
    tool: meter
    manifest: meter-search-efficiency-overlap.toml
    category: efficiency
    command: "cd apps/beam && ../../target/debug/vat run ec-efficiency-meter-overlap"
    native:
      version: 1
      project: beam
      source_contract: beam-competitor-performance-pipeline-overlap
      delegate_command: "cd apps/beam && ../../target/debug/vat run --scenario pipeline-overlap"

  - id: beam-meter-search-efficiency-ooc
    tool: meter
    manifest: meter-search-efficiency-ooc.toml
    category: efficiency
    command: "cd apps/beam && ../../target/debug/vat run ec-efficiency-meter-ooc"
    native:
      version: 1
      project: beam
      source_contract: beam-competitor-performance-memory-footprint
      delegate_command: "cd apps/beam && ../../target/debug/vat run --scenario out-of-core"

  - id: beam-meter-search-efficiency-gpu
    tool: meter
    manifest: meter-search-efficiency-gpu.toml
    category: efficiency
    command: "cd apps/beam && ../../target/debug/vat run ec-efficiency-meter-gpu"
    native:
      version: 1
      project: beam
      source_contract: beam-competitor-performance-gpu-batch-scaling
      delegate_command: "cd apps/beam && ../../target/debug/vat run --scenario gpu-batching"
```
