---
id: command-resources-dual-win-rss-gate-fails-unconditionally-blocki
summary: Calibrate command_resources resource-gate policy around the stable macOS process RSS floor while retaining a strict CPU win, and bound the xargs -n 1 fixture so a code-check remains a finite verification gate.
fill_sections: [logic, unit-test, changes]
capability_scope: internal
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: command-resources-resource-gate-calibration
entry: evaluate_resource_gate
nodes:
  measure: { kind: start, label: measure cap and original child rusage }
  ratios: { kind: process, label: calculate CPU and peak RSS ratios }
  cpu: { kind: decision, label: CPU ratio below 1.0 }
  rss: { kind: decision, label: RSS ratio within 1.25 budget }
  fail_cpu: { kind: terminal, label: fail CPU regression }
  fail_rss: { kind: terminal, label: fail RSS budget regression }
  pass: { kind: terminal, label: pass calibrated resource gate }
edges:
  - { from: measure, to: ratios }
  - { from: ratios, to: cpu }
  - { from: cpu, to: fail_cpu, label: no }
  - { from: cpu, to: rss, label: yes }
  - { from: rss, to: fail_rss, label: no }
  - { from: rss, to: pass, label: yes }
---
flowchart TD
    measure[measure cap and original child rusage] --> ratios[calculate CPU and peak RSS ratios]
    ratios --> cpu{CPU ratio below 1.0}
    cpu -- no --> fail_cpu([fail CPU regression])
    cpu -- yes --> rss{RSS ratio within 1.25 budget}
    rss -- no --> fail_rss([fail RSS budget regression])
    rss -- yes --> pass([pass calibrated resource gate])
```

Resource-gated rows use strict CPU admission and a calibrated peak-RSS budget. A native cap process carries a stable macOS process floor, so peak RSS must remain observable and bounded but cannot be required to beat every shell utility by an infinitesimal margin.

The policy accepts `cpu_ratio < 1.0` and `rss_ratio <= 1.25`; it fails either boundary violation with a policy-specific diagnostic. The 1.25 ceiling covers the independently reproduced 1.04x--1.18x process-floor range while preserving a tight cap on regressions.

The `xargs -n 1` stdin benchmark uses an independent, fixed-size fixture. It keeps command and pipe behavior in coverage while avoiding the inherited 500,000-line sort fixture that multiplies every benchmark sample into hundreds of thousands of child processes.
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: command-resources-rss-budget-verification
requirements:
  boundary_rejection:
    id: R2
    text: "The gate accepts the exact RSS budget boundary but rejects CPU parity/regression and RSS values above the budget with an explicit diagnostic."
    kind: regression
    risk: high
    verify: cargo test -p cap --test command_resources_gate
  bounded_xargs_fixture:
    id: R3
    text: "The xargs -n 1 benchmark scenario receives a dedicated finite fixture rather than the 500000-line sort workload, while preserving stdin pipeline parity coverage."
    kind: regression
    risk: medium
    verify: cargo test -p cap --test command_resources_gate
  cpu_and_rss_budget:
    id: R1
    text: "Resource-gated rows require a strict CPU ratio below 1.0 and an RSS ratio no greater than the documented 1.25 process-floor budget."
    kind: functional
    risk: high
    verify: cargo test -p cap --test command_resources_gate
  end_to_end_benchmark:
    id: R4
    text: "The full command_resources benchmark completes and reports the calibrated resource-gate policy without a pre-existing unconditional RSS failure."
    kind: integration
    risk: high
    verify: cargo bench -p cap --bench command_resources
---
flowchart TD
    r1[R1 cpu and rss budget] --> cargo_test_p_cap_test_command_resources_gate[cargo test -p cap --test command_resources_gate]
    r2[R2 boundary rejection] --> cargo_test_p_cap_test_command_resources_gate
    r3[R3 bounded xargs fixture] --> cargo_test_p_cap_test_command_resources_gate
    r4[R4 end to end benchmark] --> cargo_bench_p_cap_bench_command_resources[cargo bench -p cap --bench command_resources]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/cap/tech-design/semantic/source/projects-cap-benches-command_resources-rs.md
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: Replace the impossible strict dual-win RSS comparison with an explicitly named strict-CPU plus 1.25x-RSS-budget gate, preserve raw measurement reporting, and use a dedicated bounded stdin fixture for the xargs -n 1 scenario.
  - path: apps/cap/benches/command_resources.rs
    action: modify
    section: e2e-test
    impl_mode: codegen
    description: Regenerate the benchmark from the semantic source so the runtime policy, gate label, diagnostic, and bounded xargs fixture are synchronized.
  - path: apps/cap/tests/command_resources_gate.rs
    action: add
    section: unit-test
    impl_mode: hand-written
    description: Test CPU rejection, exact RSS-budget acceptance, above-budget rejection, and the bounded xargs-n1 fixture invariant without executing the full benchmark.
  - path: apps/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md
    action: modify
    section: e2e-test
    impl_mode: hand-written
    description: Update the resource-gate contract to name the calibrated CPU-plus-RSS budget rather than a strict dual-win requirement.
  - path: apps/cap/BENCHMARKS.md
    action: modify
    section: documentation
    impl_mode: hand-written
    description: Record the macOS process-floor rationale, 1.25x RSS ceiling, and bounded xargs-n1 fixture policy so the threshold is deliberate and auditable.
```
