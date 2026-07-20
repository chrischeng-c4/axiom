---
id: '2148'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: beam-ddd-wgpu-distance-adapter
entry: start
nodes:
  start: { kind: start, label: "WgpuDistanceEngine::compute_batched()" }
  check_gpu: { kind: decision, label: "Is GPU context available?" }
  run_gpu: { kind: process, label: "Execute WGPU flat batch distance kernels" }
  run_cpu: { kind: process, label: "Fallback to CPU metric calculation" }
  done: { kind: terminal, label: "Return results and explicit backend evidence" }
edges:
  - { from: start, to: check_gpu }
  - { from: check_gpu, to: run_gpu, label: "Yes" }
  - { from: check_gpu, to: run_cpu, label: "No" }
  - { from: run_gpu, to: done }
  - { from: run_cpu, to: done }
---
flowchart TD
    start([compute_batched]) --> check_gpu{Is GPU available?}
    check_gpu -->|Yes| run_gpu[Execute WGPU batch kernels]
    check_gpu -->|No| run_cpu[Fallback to CPU calculations]
    run_gpu --> done([Return results & evidence])
    run_cpu --> done
```
