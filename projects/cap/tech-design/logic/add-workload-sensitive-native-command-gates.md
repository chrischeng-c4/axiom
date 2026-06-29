---
id: add-workload-sensitive-native-command-gates
summary: Add workload-sensitive native command gates so cap keeps tiny or unknown command shapes on the original path and promotes only large, parity-covered workloads with benchmark evidence.
fill_sections: [logic, unit-test]
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
id: cap-workload-sensitive-native-gates-logic
entry: start
nodes:
  start: { kind: start, label: "cap command request" }
  normalize: { kind: process, label: "normalize cap <cmd>, cap run argv, or shell-free cap run string into argv" }
  shell_check: { kind: decision, label: "requires shell semantics or unsupported flags?" }
  shell_fallback: { kind: terminal, label: "run original command path" }
  candidate: { kind: process, label: "match native candidate subset: ls, sort, grep, find, sed -n" }
  candidate_found: { kind: decision, label: "candidate subset matched?" }
  classify: { kind: process, label: "classify workload with cheap observable thresholds" }
  tiny_or_unknown: { kind: decision, label: "tiny or unknown materiality?" }
  original: { kind: terminal, label: "keep original command; fixed cap overhead would dominate" }
  parity: { kind: process, label: "require stdout, stderr, and exit-status parity coverage" }
  resource_gate: { kind: process, label: "require representative benchmark gate for large workload" }
  gate_pass: { kind: decision, label: "dual-win or approved RSS-fallback?" }
  native_fast: { kind: terminal, label: "run workload-sensitive native fast path" }
edges:
  - { from: start, to: normalize }
  - { from: normalize, to: shell_check }
  - { from: shell_check, to: shell_fallback, label: "yes" }
  - { from: shell_check, to: candidate, label: "no" }
  - { from: candidate, to: candidate_found }
  - { from: candidate_found, to: original, label: "no" }
  - { from: candidate_found, to: classify, label: "yes" }
  - { from: classify, to: tiny_or_unknown }
  - { from: tiny_or_unknown, to: original, label: "yes" }
  - { from: tiny_or_unknown, to: parity, label: "large enough" }
  - { from: parity, to: resource_gate }
  - { from: resource_gate, to: gate_pass }
  - { from: gate_pass, to: native_fast, label: "yes" }
  - { from: gate_pass, to: original, label: "no" }
---
flowchart TD
    start([cap command request]) --> normalize[normalize cap <cmd>, cap run argv, or shell-free cap run string into argv]
    normalize --> shell_check{requires shell semantics or unsupported flags?}
    shell_check -- yes --> shell_fallback([run original command path])
    shell_check -- no --> candidate[match native candidate subset: ls, sort, grep, find, sed -n]
    candidate --> candidate_found{candidate subset matched?}
    candidate_found -- no --> original([keep original command])
    candidate_found -- yes --> classify[classify workload with cheap observable thresholds]
    classify --> tiny_or_unknown{tiny or unknown materiality?}
    tiny_or_unknown -- yes --> original
    tiny_or_unknown -- large enough --> parity[require stdout, stderr, and exit-status parity coverage]
    parity --> resource_gate[require representative benchmark gate for large workload]
    resource_gate --> gate_pass{dual-win or approved RSS-fallback?}
    gate_pass -- yes --> native_fast([run workload-sensitive native fast path])
    gate_pass -- no --> original
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: cap-workload-sensitive-native-gates-tests
requirements:
  tiny_workloads_fallback:
    id: R1
    text: "Tiny or unknown workloads for candidate commands stay on the original command path when cap overhead would dominate."
    kind: functional
    risk: high
    verify: test
  large_workloads_gate_native:
    id: R2
    text: "Large workloads for supported subsets can use native fast paths only after cheap threshold classification and resource-gate evidence."
    kind: functional
    risk: high
    verify: test
  shell_semantics_preserved:
    id: R3
    text: "Shell-sensitive strings, unsupported flags, and unsafe shapes keep bash/original fallback semantics instead of partial native execution."
    kind: functional
    risk: high
    verify: test
  benchmark_matrix_covers_size:
    id: R4
    text: "Resource benchmarks include representative small and large scenarios with expected planner decisions for promoted candidates."
    kind: functional
    risk: high
    verify: test
  parity_still_required:
    id: R5
    text: "Every promoted native path keeps stdout, stderr, and exit-status parity with the original command."
    kind: functional
    risk: high
    verify: test
elements:
  planner_tiny_workloads_use_original:
    kind: test
    type: "rs/#[test]"
  planner_large_workloads_use_native_after_threshold:
    kind: test
    type: "rs/#[test]"
  planner_shell_or_unsupported_shapes_fallback:
    kind: test
    type: "rs/#[test]"
  command_resource_bench_small_large_matrix:
    kind: benchmark
    type: "cargo bench"
  active_replacements_match_success_and_error_behavior:
    kind: test
    type: "cargo test"
relations:
  - { from: planner_tiny_workloads_use_original, verifies: tiny_workloads_fallback }
  - { from: planner_large_workloads_use_native_after_threshold, verifies: large_workloads_gate_native }
  - { from: planner_shell_or_unsupported_shapes_fallback, verifies: shell_semantics_preserved }
  - { from: command_resource_bench_small_large_matrix, verifies: benchmark_matrix_covers_size }
  - { from: active_replacements_match_success_and_error_behavior, verifies: parity_still_required }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "tiny or unknown workloads fall back"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "large workloads need threshold and gate evidence"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "shell and unsupported shapes preserve fallback semantics"
      risk: high
      verifymethod: test
    }
    requirement R4 {
      id: R4
      text: "benchmarks cover small and large scenarios"
      risk: high
      verifymethod: test
    }
    requirement R5 {
      id: R5
      text: "promoted native paths preserve parity"
      risk: high
      verifymethod: test
    }
    element planner_tiny_workloads_use_original {
      type: "rs/#[test]"
    }
    element planner_large_workloads_use_native_after_threshold {
      type: "rs/#[test]"
    }
    element planner_shell_or_unsupported_shapes_fallback {
      type: "rs/#[test]"
    }
    element command_resource_bench_small_large_matrix {
      type: "cargo bench"
    }
    element active_replacements_match_success_and_error_behavior {
      type: "cargo test"
    }
```
