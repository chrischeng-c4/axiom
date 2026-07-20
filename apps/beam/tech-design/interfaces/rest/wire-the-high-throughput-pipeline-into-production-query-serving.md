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
