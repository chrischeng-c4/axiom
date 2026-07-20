---
id: '2153'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: high-throughput-pipeline-overlap
entry: start
---
sequenceDiagram
    participant App as SearchApplicationService
    participant Sched as Scheduler
    participant Fetch as Candidate Fetch (NVMe)
    participant GPU as GPU Compute (wgpu)

    App->>Sched: submit(Batch 1)
    App->>Sched: submit(Batch 2)
    
    activate Sched
    Sched->>Fetch: fetch(Batch 1 candidates)
    Fetch-->>Sched: Batch 1 vectors
    
    par Overlapped Execution
        Sched->>GPU: compute(Batch 1 vectors)
        Sched->>Fetch: fetch(Batch 2 candidates)
    end
    
    GPU-->>Sched: Batch 1 TopK
    Sched-->>App: Batch 1 Results
    Fetch-->>Sched: Batch 2 vectors
    
    Sched->>GPU: compute(Batch 2 vectors)
    GPU-->>Sched: Batch 2 TopK
    Sched-->>App: Batch 2 Results
    deactivate Sched
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/beam/src/domain/scheduler.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: pub async fn execute_batch
  - path: apps/beam/src/application/search_service.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: pub async fn search
  - path: apps/beam/src/service.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: async fn query_collection
  - path: apps/beam/tests/throughput_pipeline.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: async fn test_r2_r3_infrastructure_and_e2e_pipeline
  - path: apps/beam/tests/service.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: async fn service_end_to_end
  - path: apps/beam/tests/pipeline_overlap.rs
    action: create
    section: logic
    impl_mode: hand-written
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: 2153-verification
requirements:
  overlap_concurrent_batching:
    id: R2
    text: "Implement bounded concurrent batching that overlaps candidate fetch for a later batch with GPU compute for the current batch."
    kind: functional
    risk: high
    verify: cargo test -p beam --test pipeline_overlap
  preserve_semantics:
    id: R3
    text: "Preserve backpressure, cancellation, result order, metric semantics, and storage-error propagation."
    kind: functional
    risk: high
    verify: cargo test -p beam --test throughput_pipeline
  wire_ddd_search:
    id: R1
    text: "Wire the corrected DDD search application service into the real Beam query path without maintaining two divergent collection/index models."
    kind: functional
    risk: high
    verify: cargo test -p beam --test service
---
flowchart TD
    r1[R1 wire ddd search] --> cargo_test_p_beam_test_service[cargo test -p beam --test service]
    r2[R2 overlap concurrent batching] --> cargo_test_p_beam_test_pipeline_overlap[cargo test -p beam --test pipeline_overlap]
    r3[R3 preserve semantics] --> cargo_test_p_beam_test_throughput_pipeline[cargo test -p beam --test throughput_pipeline]
```
