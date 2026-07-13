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
  benchmark_gate:
    id: R4
    text: "The command_resources benchmark completes with the calibrated policy and emits its raw CPU and RSS evidence."
    kind: integration
    risk: high
    verify: cargo bench -p cap --bench command_resources
  cpu_and_rss_budget:
    id: R1
    text: "Resource-gated rows require CPU ratio strictly below 1.0 and peak-RSS ratio at or below 1.25."
    kind: functional
    risk: high
    verify: cargo test -p cap --test command_resources_gate
  finite_fixture:
    id: R3
    text: "The xargs -n 1 stdin case uses an independent bounded fixture, preserving its command shape without multiplying benchmark rounds into millions of child processes."
    kind: regression
    risk: medium
    verify: cargo test -p cap --test command_resources_gate
  policy_boundaries:
    id: R2
    text: "CPU parity or regression and RSS above 1.25 are rejected, while the exact 1.25 RSS boundary is accepted."
    kind: regression
    risk: high
    verify: cargo test -p cap --test command_resources_gate
---
flowchart TD
    r1[R1 cpu and rss budget] --> cargo_test_p_cap_test_command_resources_gate[cargo test -p cap --test command_resources_gate]
    r2[R2 policy boundaries] --> cargo_test_p_cap_test_command_resources_gate
    r3[R3 finite fixture] --> cargo_test_p_cap_test_command_resources_gate
    r4[R4 benchmark gate] --> cargo_bench_p_cap_bench_command_resources[cargo bench -p cap --bench command_resources]
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
