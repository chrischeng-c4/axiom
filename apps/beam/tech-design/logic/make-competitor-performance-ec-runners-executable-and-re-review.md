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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: 2155-verification
requirements:
  req1:
    id: R1
    text: "Define Beam-owned VAT scenarios and meter manifests for DDD overhead, GPU batching, out-of-core memory, and pipeline overlap."
    kind: functional
    risk: medium
    verify: cargo test benchmark_beam_competitor_performance
  req2:
    id: R2
    text: "Execute the real production pipeline and pin independent oracle/baseline versions, datasets, hardware identity, warmup, samples, thresholds, and evidence paths."
    kind: functional
    risk: medium
    verify: vat run ec-efficiency-meter scenarios
  req3:
    id: R3
    text: "Reject missing tools, zero executed cases, unavailable required hardware, stale evidence, and placeholder/simulated adapters."
    kind: functional
    risk: high
    verify: vat run ec-efficiency-meter scenarios failure cases
  req4:
    id: R4
    text: "After the EC source digest changes, obtain a new independent agent-backed semantic review; do not reuse or fabricate human approval."
    kind: functional
    risk: high
    verify: aw ec check --project beam
---
flowchart TD
    r1[R1 req1] --> cargo_test_benchmark_beam_competitor_performance[cargo test benchmark_beam_competitor_performance]
    r2[R2 req2] --> vat_run_ec_efficiency_meter_scenarios[vat run ec-efficiency-meter scenarios]
    r3[R3 req3] --> vat_run_ec_efficiency_meter_scenarios_failure_cases[vat run ec-efficiency-meter scenarios failure cases]
    r4[R4 req4] --> aw_ec_check_project_beam[aw ec check --project beam]
```
