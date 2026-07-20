---
id: '2155'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: ec-runners-logic
entry: start
nodes:
  start: { kind: start, label: "Begin EC Runners Update" }
  build_vat: { kind: process, label: "Build repository VAT binary and define ec-efficiency-meter" }
  define_scenarios: { kind: process, label: "Make each scenario drive the production pipeline from #2153" }
  pin_versions: { kind: process, label: "Pin datasets, warmup, samples, hardware, oracles, thresholds, evidence" }
  implement_assertions: { kind: process, label: "Implement measured assertions (DDD overhead, GPU scaling, RAM/VRAM, pipeline overlap)" }
  generate_ec: { kind: process, label: "Generate EC scaffolds with AW and request independent agent semantic review" }
  verify: { kind: process, label: "Verify all EC commands pass with fresh evidence" }
  end: { kind: terminal, label: "Completion" }
edges:
  - { from: start, to: build_vat }
  - { from: build_vat, to: define_scenarios }
  - { from: define_scenarios, to: pin_versions }
  - { from: pin_versions, to: implement_assertions }
  - { from: implement_assertions, to: generate_ec }
  - { from: generate_ec, to: verify }
  - { from: verify, to: end }
---
flowchart TD
    start([Begin EC Runners Update]) --> build_vat[Build repository VAT binary and define ec-efficiency-meter]
    build_vat --> define_scenarios[Make each scenario drive the production pipeline from #2153]
    define_scenarios --> pin_versions[Pin datasets, warmup, samples, hardware, oracles, thresholds, evidence]
    pin_versions --> implement_assertions[Implement measured assertions]
    implement_assertions --> generate_ec[Generate EC scaffolds with AW and request independent agent semantic review]
    generate_ec --> verify[Verify all EC commands pass with fresh evidence]
    verify --> end_node([Completion])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/beam/vat.toml
    action: modify
    section: ec-runners-logic
    impl_mode: hand-written
  - path: apps/beam/meter-search-efficiency-ddd.toml
    action: create
    section: ec-runners-logic
    impl_mode: hand-written
  - path: apps/beam/meter-search-efficiency-gpu.toml
    action: create
    section: ec-runners-logic
    impl_mode: hand-written
  - path: apps/beam/meter-search-efficiency-ooc.toml
    action: create
    section: ec-runners-logic
    impl_mode: hand-written
  - path: apps/beam/meter-search-efficiency-overlap.toml
    action: create
    section: ec-runners-logic
    impl_mode: hand-written
  - path: apps/beam/tests/benchmark_beam_competitor_performance_ddd_overhead.rs
    action: create
    section: ec-runners-logic
    impl_mode: hand-written
  - path: apps/beam/tests/benchmark_beam_competitor_performance_gpu_batching.rs
    action: create
    section: ec-runners-logic
    impl_mode: hand-written
  - path: apps/beam/tests/benchmark_beam_competitor_performance_out_of_core.rs
    action: create
    section: ec-runners-logic
    impl_mode: hand-written
  - path: apps/beam/tests/benchmark_beam_competitor_performance_pipeline_overlap.rs
    action: create
    section: ec-runners-logic
    impl_mode: hand-written
```
