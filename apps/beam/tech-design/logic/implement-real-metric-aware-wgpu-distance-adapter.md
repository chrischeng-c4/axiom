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
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/beam/src/infrastructure/wgpu_engine.rs
    action: modify
    section: wgpu-engine
    impl_mode: hand-written
    anchor: "impl DistanceCalculator for WgpuDistanceEngine"
  - path: apps/beam/src/gpu/mod.rs
    action: modify
    section: gpu-batch-adapter
    impl_mode: hand-written
    anchor: "impl GpuContext"
  - path: apps/beam/tests/wgpu_distance_adapter.rs
    action: create
    section: adapter-tests
    impl_mode: hand-written
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: implement-real-metric-aware-wgpu-distance-adapter-verification
requirements:
  req_1:
    id: R1
    text: "Implement the WGPU-backed DistanceCalculator using Beam's existing GPU primitives"
    kind: functional
    risk: medium
    verify: test_wgpu_distance_adapter_batching_l2
  req_2_cosine:
    id: R2_COSINE
    text: "Support true multi-query batching and Cosine semantics"
    kind: functional
    risk: medium
    verify: test_wgpu_distance_adapter_batching_cosine
  req_2_dot:
    id: R2_DOT
    text: "Support true multi-query batching and Dot semantics"
    kind: functional
    risk: medium
    verify: test_wgpu_distance_adapter_batching_dot
  req_2_l2:
    id: R2_L2
    text: "Support true multi-query batching and L2 semantics"
    kind: functional
    risk: medium
    verify: test_wgpu_distance_adapter_batching_l2
  req_3:
    id: R3
    text: "Keep an explicit CPU fallback that reports which backend executed"
    kind: functional
    risk: low
    verify: test_wgpu_distance_adapter_fallback
---
flowchart TD
    r1[R1 req 1] --> test_wgpu_distance_adapter_batching_l2[test_wgpu_distance_adapter_batching_l2]
    r2_l2[R2_L2 req 2 l2] --> test_wgpu_distance_adapter_batching_l2
    r3[R3 req 3] --> test_wgpu_distance_adapter_fallback[test_wgpu_distance_adapter_fallback]
    r2_cosine[R2_COSINE req 2 cosine] --> test_wgpu_distance_adapter_batching_cosine[test_wgpu_distance_adapter_batching_cosine]
    r2_dot[R2_DOT req 2 dot] --> test_wgpu_distance_adapter_batching_dot[test_wgpu_distance_adapter_batching_dot]
```
