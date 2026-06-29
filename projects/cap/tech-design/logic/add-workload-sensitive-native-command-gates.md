---
id: add-workload-sensitive-native-command-gates
summary: Add workload-sensitive native command gates so cap keeps tiny or unknown command shapes on the original path and promotes only large, parity-covered workloads with benchmark evidence.
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: command-lease-throttling
    role: primary
    gap: lease-admission-and-process-supervision
    claim: lease-admission-and-process-supervision
    coverage: partial
    rationale: "The command planner decides whether cap run and same-name command entrypoints use native fast paths or preserve the original command."
  - id: command-lease-throttling
    role: primary
    gap: memory-and-cpu-pressure-sampling
    claim: memory-and-cpu-pressure-sampling
    coverage: partial
    rationale: "Workload-sensitive native gates use benchmarked CPU and peak RSS evidence before promoting a command shape."
---

# TD: cap workload-sensitive native command gates

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cap-workload-sensitive-native-gates-contract
entry: start
nodes:
  start: { kind: start, label: "plan argv or shell-free command string" }
  shape: { kind: decision, label: "matches supported safe shape?" }
  fallback_shape: { kind: terminal, label: "original path: unsupported shape" }
  probe: { kind: process, label: "collect cheap workload facts without executing command output" }
  classify: { kind: decision, label: "meets command-specific minimum?" }
  fallback_small: { kind: terminal, label: "original path: tiny or unmeasured workload" }
  parity: { kind: process, label: "require parity test coverage for this shape" }
  bench: { kind: process, label: "require benchmark row for representative large workload" }
  promote: { kind: decision, label: "resource gate passes?" }
  native: { kind: terminal, label: "native/replacement path active" }
  fallback_gate: { kind: terminal, label: "original path: benchmark gate not proven" }
edges:
  - { from: start, to: shape }
  - { from: shape, to: fallback_shape, label: "no" }
  - { from: shape, to: probe, label: "yes" }
  - { from: probe, to: classify }
  - { from: classify, to: fallback_small, label: "no" }
  - { from: classify, to: parity, label: "yes" }
  - { from: parity, to: bench }
  - { from: bench, to: promote }
  - { from: promote, to: native, label: "dual-win or approved RSS-fallback" }
  - { from: promote, to: fallback_gate, label: "not proven" }
thresholds:
  ls:
    shape: "simple -1/-a/-A over one existing directory"
    minimum: "directory entry count >= 1024"
  sort:
    shape: "one regular file"
    minimum: "file size >= 1048576 bytes"
  grep:
    shape: "recursive literal grep -R pattern root"
    minimum: "root contains >= 64 regular files or estimated text bytes >= 1048576"
  find:
    shape: "root -type f -name pattern"
    minimum: "root contains >= 512 directory entries before full traversal finishes"
  sed_print:
    shape: "sed -n start,endp file"
    minimum: "regular file size >= 1048576 bytes or requested span >= 1024 lines"
fallback_rule: "Any probe error, symlink ambiguity, unsupported flag, shell control syntax, or below-threshold workload preserves the original command path."
---
flowchart TD
    start([plan argv or shell-free command string]) --> shape{matches supported safe shape?}
    shape -- no --> fallback_shape([original path: unsupported shape])
    shape -- yes --> probe[collect cheap workload facts]
    probe --> classify{meets command-specific minimum?}
    classify -- no --> fallback_small([original path: tiny or unmeasured workload])
    classify -- yes --> parity[require parity test coverage]
    parity --> bench[require benchmark row]
    bench --> promote{resource gate passes?}
    promote -- yes --> native([native/replacement path active])
    promote -- no --> fallback_gate([original path: benchmark gate not proven])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: cap-workload-sensitive-native-gates-contract-tests
requirements:
  planner_below_threshold_original:
    id: R1
    text: "Planner returns External Original for supported command shapes when cheap workload facts are below threshold."
    kind: functional
    risk: high
    verify: test
  planner_above_threshold_native:
    id: R2
    text: "Planner returns the existing native/replacement implementation for supported command shapes when cheap workload facts meet threshold."
    kind: functional
    risk: high
    verify: test
  probe_errors_fail_open:
    id: R3
    text: "Probe errors, missing paths, and unsupported flags fail open to the original command path."
    kind: functional
    risk: high
    verify: test
  run_string_matches_argv:
    id: R4
    text: "Shell-free cap run strings and cap argv entrypoints make the same workload-sensitive decision."
    kind: functional
    risk: high
    verify: test
  benchmark_small_large_rows:
    id: R5
    text: "command_resources benchmark has explicit small and large rows for ls, sort, grep, find, and sed -n."
    kind: functional
    risk: high
    verify: benchmark
  readme_describes_fast_paths:
    id: R6
    text: "README describes native commands as workload-sensitive fast paths, not unconditional replacements."
    kind: functional
    risk: medium
    verify: test
elements:
  planner_threshold_tests:
    kind: test
    type: "cargo test -p cap command_planner"
  run_string_threshold_tests:
    kind: test
    type: "cargo test -p cap command_planner"
  parity_regression:
    kind: test
    type: "cargo test -p cap active_replacements_match_success_and_error_behavior"
  resource_benchmark_matrix:
    kind: benchmark
    type: "cargo bench -p cap --bench command_resources"
  readme_wording_smoke:
    kind: test
    type: "cargo test -p cap docs"
relations:
  - { from: planner_threshold_tests, verifies: planner_below_threshold_original }
  - { from: planner_threshold_tests, verifies: planner_above_threshold_native }
  - { from: planner_threshold_tests, verifies: probe_errors_fail_open }
  - { from: run_string_threshold_tests, verifies: run_string_matches_argv }
  - { from: parity_regression, verifies: run_string_matches_argv }
  - { from: parity_regression, verifies: probe_errors_fail_open }
  - { from: resource_benchmark_matrix, verifies: benchmark_small_large_rows }
  - { from: readme_wording_smoke, verifies: readme_describes_fast_paths }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "below-threshold supported shapes use original path"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "above-threshold supported shapes can use native path"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "probe errors and unsupported flags fail open"
      risk: high
      verifymethod: test
    }
    requirement R4 {
      id: R4
      text: "run string and argv decisions match"
      risk: high
      verifymethod: test
    }
    requirement R5 {
      id: R5
      text: "benchmarks include small and large rows"
      risk: high
      verifymethod: benchmark
    }
    requirement R6 {
      id: R6
      text: "README says workload-sensitive fast paths"
      risk: medium
      verifymethod: test
    }
    element planner_threshold_tests {
      type: "cargo test"
    }
    element run_string_threshold_tests {
      type: "cargo test"
    }
    element parity_regression {
      type: "cargo test"
    }
    element resource_benchmark_matrix {
      type: "cargo bench"
    }
    element readme_wording_smoke {
      type: "cargo test"
    }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/cap/src/command_planner.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Add workload fact probing and command-specific threshold checks before
      activating native or replacement paths. Supported candidates below their
      threshold, with probe errors, or with unknown materiality must return an
      External Original plan. Preserve existing shell-sensitive fallback.

  - path: projects/cap/src/cap_fast_frontend.c
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Mirror the Rust planner workload thresholds in the public low-overhead C
      frontend so same-name native fast paths only claim large, material
      workloads and otherwise fall through to cap-full's original-command path.

  - path: projects/cap/src/command_planner.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: >
      Add planner tests proving below-threshold ls, sort, grep, find, and sed
      shapes use the original path; above-threshold fixtures use the active
      native or replacement path; shell-free cap run strings make the same
      workload-sensitive decision as argv.

  - path: projects/cap/benches/command_resources.rs
    action: modify
    section: e2e-test
    impl_mode: hand-written
    description: >
      Extend the benchmark matrix with explicit small and large scenarios for
      ls, sort, grep, find, and sed -n. Small scenarios document expected
      original-path behavior; large scenarios remain gated by dual-win or an
      explicitly approved RSS-fallback policy.

  - path: projects/cap/tests/behavior_cap_command_replacement_parity.rs
    action: modify
    section: e2e-test
    impl_mode: hand-written
    description: >
      Keep active replacement parity coverage for promoted large-workload
      shapes and add regression coverage that below-threshold command shapes
      do not bypass the original command path.

  - path: projects/cap/README.md
    action: modify
    section: overview
    impl_mode: hand-written
    description: >
      Reword native command replacement as workload-sensitive fast paths.
      Document the threshold policy and make clear that tiny, unknown, or
      unproven workloads keep original-command behavior.

  - path: projects/cap/BENCHMARKS.md
    action: modify
    section: overview
    impl_mode: hand-written
    description: >
      Document small-vs-large benchmark rows and the interpretation that
      default replacement promotion depends on representative workload size,
      parity, and resource-gate evidence.
```
