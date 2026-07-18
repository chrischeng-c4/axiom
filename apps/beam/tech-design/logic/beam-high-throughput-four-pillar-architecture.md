---
id: 'throughput-arch-ddd-001'
summary: Beam High-Throughput Engine (Domain-Driven Design Architecture)
capability_refs:
  - id: competitor-performance
    role: primary
    claim: four-pillar-throughput-saturation-ddd
    coverage: full
    rationale: "Adopting a strict DDD architecture to decouple hardware orchestration (io_uring/WGPU) from the core vector database domain, maximizing throughput via asynchronous pipelining."
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: ddd-domain-model
title: "Beam High-Throughput Core Domain Model (DDD)"
---
classDiagram
    %% Bounded Contexts
    namespace CoreDomain {
        class Collection {
            <<Aggregate Root>>
            +String id
            +rebuild_navigation()
        }
        class HnswNavigator {
            <<Entity>>
            +find_candidates(query: Vector) -> List~BatchId~
        }
        class ColdPayload {
            <<Entity>>
            +resolve_offsets(ids: List~String~) -> List~u64~
        }
        class QueryBatch {
            <<Aggregate Root>>
            +BatchId id
            +List~Vector~ queries
            +transition_state()
        }
        class PipelineScheduler {
            <<Domain Service>>
            +schedule(batch: QueryBatch, collection: Collection)
        }
    }

    namespace InfrastructureInterfaces {
        class VectorRepository {
            <<Interface>>
            +fetch_async(offsets: List~u64~) -> Future~PinnedBuffer~
        }
        class DistanceCalculator {
            <<Interface>>
            +compute_batched(queries: PinnedBuffer, targets: PinnedBuffer) -> Future~TopK~
        }
    }

    namespace Infrastructure {
        class IoUringVectorRepository {
            <<Infrastructure>>
            -io_uring_sq queue
            +fetch_async()
        }
        class WgpuDistanceEngine {
            <<Infrastructure>>
            -wgpu_device device
            +compute_batched()
        }
    }

    Collection *-- HnswNavigator
    Collection *-- ColdPayload
    PipelineScheduler ..> QueryBatch : orchestrates
    PipelineScheduler ..> VectorRepository : uses
    PipelineScheduler ..> DistanceCalculator : uses
    IoUringVectorRepository ..|> VectorRepository : implements
    WgpuDistanceEngine ..|> DistanceCalculator : implements
```

<!-- type: logic lang: mermaid -->

```mermaid
---
id: ddd-pipeline-sequence
title: "PipelineScheduler Async Flow"
---
sequenceDiagram
    participant App as SearchAppService
    participant Sched as PipelineScheduler
    participant Nav as HnswNavigator (RAM)
    participant Disk as IoUringRepo (NVMe)
    participant GPU as WgpuEngine (VRAM)

    App->>Sched: submit(QueryBatch)
    activate Sched
    Sched->>Nav: find_candidates()
    Note right of Nav: Phase 1: CPU Graph Traversal
    Nav-->>Sched: Candidate IDs
    
    Sched->>Disk: fetch_async(Candidate Offsets)
    Note right of Disk: Phase 2: Async NVMe DMA to Pinned RAM
    Disk-->>Sched: PinnedBuffer (Raw Vectors)
    
    Sched->>GPU: compute_batched(Queries, Raw Vectors)
    Note right of GPU: Phase 3: PCIe Transfer & Tiled GEMM
    GPU-->>Sched: TopK Scores
    
    Sched-->>App: Sorted Neighbors
    deactivate Sched
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/beam/src/domain/collection.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "Define Collection Aggregate, HnswNavigator Entity, and ColdPayload Entity. Ensure domain layer contains no hardware-specific dependencies."
  - path: apps/beam/src/domain/scheduler.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "Implement PipelineScheduler Domain Service to orchestrate QueryBatch state transitions."
  - path: apps/beam/src/domain/ports.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "Define VectorRepository and DistanceCalculator Hexagonal/DDD ports (Interfaces)."
  - path: apps/beam/src/infrastructure/io_uring_repo.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "Implement VectorRepository port using Linux io_uring for O_DIRECT NVMe fetching to Pinned Memory."
  - path: apps/beam/src/infrastructure/wgpu_engine.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "Implement DistanceCalculator port wrapping wgpu Compute Pipelines for Tiled GEMM."
  - path: apps/beam/src/application/search_service.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: "Implement SearchApplicationService to wire HTTP requests to the PipelineScheduler."
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: ddd-architecture-verification
requirements:
  domain_isolation:
    id: R1
    text: "The Domain Layer (Collection, PipelineScheduler) can be fully unit-tested using MockVectorRepository and MockDistanceCalculator without actual GPU or NVMe hardware."
    kind: unit
    risk: low
    verify: cargo test -p beam --lib domain::tests
  infrastructure_integration:
    id: R2
    text: "IoUringVectorRepository correctly fetches data from physical disk and WgpuDistanceEngine correctly calculates distances on GPU."
    kind: integration
    risk: high
    verify: cargo test -p beam --test infrastructure_integration
  end_to_end_pipeline:
    id: R3
    text: "SearchApplicationService successfully wires domain and infrastructure, achieving full saturation in a simulated batched load."
    kind: functional
    risk: high
    verify: cargo test -p beam --test ddd_pipeline_e2e
---
flowchart TD
    r1[R1 Domain Isolation] --> cargo_test_domain[cargo test -p beam --lib domain::tests]
    r2[R2 Infrastructure Integration] --> cargo_test_infra[cargo test -p beam --test infrastructure_integration]
    r3[R3 E2E Pipeline] --> cargo_test_e2e[cargo test -p beam --test ddd_pipeline_e2e]
```
