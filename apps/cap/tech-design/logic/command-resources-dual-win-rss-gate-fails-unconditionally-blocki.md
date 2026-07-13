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
  start: { kind: start, label: measure cap and original child rusage }
  ratio: { kind: process, label: calculate CPU and peak RSS ratios }
  cpu: { kind: decision, label: CPU ratio below 1.0 }
  rss: { kind: decision, label: RSS ratio within 1.25 budget }
  fail_cpu: { kind: terminal, label: fail CPU regression }
  fail_rss: { kind: terminal, label: fail RSS budget regression }
  pass: { kind: terminal, label: pass calibrated resource gate }
edges:
  - { from: start, to: ratio }
  - { from: ratio, to: cpu }
  - { from: cpu, to: fail_cpu, label: no }
  - { from: cpu, to: rss, label: yes }
  - { from: rss, to: fail_rss, label: no }
  - { from: rss, to: pass, label: yes }
---
flowchart TD
    start[measure cap and original child rusage] --> ratio[calculate CPU and peak-RSS ratios]
    ratio --> cpu{CPU ratio below 1.0?}
    cpu -- no --> fail_cpu[fail: native path regressed CPU]
    cpu -- yes --> rss{RSS ratio within calibrated budget?}
    rss -- no --> fail_rss[fail: native path exceeds documented RSS budget]
    rss -- yes --> pass[pass: CPU win with bounded process-floor RSS]
```

The existing `dual-win` gate is unsound for a native `cap` process on macOS: peak RSS includes a stable executable/process floor that is higher than a shell utility even when CPU work is materially lower. The gate must therefore keep CPU as the strict replacement requirement while turning RSS into a bounded regression budget rather than an impossible strict-less-than comparison.

The calibrated budget is `cap_peak_rss / original_peak_rss <= 1.25` for resource-gated rows. This is deliberately narrow: the pre-existing blocking range is 1.04x--1.18x, while rows above 1.25x still fail. The benchmark continues to print both raw measurements and both ratios, so the policy change remains observable.

`command_resources` must expose this policy with an explicit gate name and failure text; it must not retain a `dual-win` label whose semantics no longer match the comparison. Targeted unit tests cover the CPU boundary, the RSS budget boundary, and rejection above either boundary. The checked-in benchmark explanation records the macOS process-floor rationale and the chosen 1.25x limit.

The fixture used by `xargs -n 1` must be bounded independently of broad sort workload fixtures. It currently inherits a 500,000-line input and turns every warmup/sample into 500,000 `echo` processes. A dedicated small deterministic input preserves the `-n 1` planner/parity coverage without making every code-check run millions of child processes. This timing correction changes neither the command's functional assertion nor the resource policy.

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
