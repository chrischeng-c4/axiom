---
id: expand-high-volume-native-command-coverage
summary: Expand cap native command coverage for high-volume workloads with conservative workload gates, parity tests, and benchmark evidence.
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: command-lease-throttling
    role: primary
    gap: lease-admission-and-process-supervision
    claim: lease-admission-and-process-supervision
    coverage: partial
    rationale: "High-volume same-name command replacement changes how cap admits and runs wrapped commands while preserving original fallback behavior."
  - id: command-lease-throttling
    role: primary
    gap: memory-and-cpu-pressure-sampling
    claim: memory-and-cpu-pressure-sampling
    coverage: partial
    rationale: "Promotion requires benchmark evidence that the replacement improves resource use on representative large workloads."
---

# Expand High-Volume Native Command Coverage

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cap-high-volume-native-command-coverage-logic
entry: start
nodes:
  start: { kind: start, label: "cap argv or shell-free cap run command" }
  classify: { kind: decision, label: "matches high-volume candidate shape?" }
  fallback_shape: { kind: terminal, label: "original path: unsupported or shell-sensitive shape" }
  wc_shape: { kind: decision, label: "wc -l over many regular files?" }
  probe_files: { kind: process, label: "probe file count, total bytes, and regular-file safety" }
  threshold: { kind: decision, label: "meets large-workload gate?" }
  fallback_small: { kind: terminal, label: "original path: small workload or probe failure" }
  native_wc: { kind: process, label: "run cap native line-count aggregate" }
  explain: { kind: process, label: "cap explain reports native_wc_lines vs original fallback" }
  parity: { kind: process, label: "parity tests cover stdout/stderr/exit behavior" }
  bench: { kind: process, label: "benchmark proves CPU/RSS large-workload win" }
  done: { kind: terminal, label: "promoted high-volume shape" }
edges:
  - { from: start, to: classify }
  - { from: classify, to: fallback_shape, label: "no" }
  - { from: classify, to: wc_shape, label: "yes" }
  - { from: wc_shape, to: fallback_shape, label: "no" }
  - { from: wc_shape, to: probe_files, label: "yes" }
  - { from: probe_files, to: threshold }
  - { from: threshold, to: fallback_small, label: "no" }
  - { from: threshold, to: native_wc, label: "yes" }
  - { from: native_wc, to: explain }
  - { from: explain, to: parity }
  - { from: parity, to: bench }
  - { from: bench, to: done }
thresholds:
  wc_lines:
    shape: "wc -l FILE... with no byte/word/char/max-line options and every operand a regular file"
    minimum: "file count >= 64 or total regular-file bytes >= 1048576"
    fallback: "missing files, directories, stdin operands, unsupported flags, or below-threshold workloads preserve the original command"
scout_next:
  - "find ... -type f ... | xargs wc -l can be promoted later by reusing the native wc_lines aggregate after a conservative pipeline parser exists."
  - "grep -R ... | head ... remains scout-only until early-close stderr/exit behavior has parity and benchmark evidence."
---
flowchart TD
    start([cap argv or shell-free cap run command]) --> classify{matches high-volume candidate shape?}
    classify -- no --> fallback_shape([original path: unsupported or shell-sensitive shape])
    classify -- yes --> wc_shape{wc -l over many regular files?}
    wc_shape -- no --> fallback_shape
    wc_shape -- yes --> probe_files[probe file count, total bytes, and regular-file safety]
    probe_files --> threshold{meets large-workload gate?}
    threshold -- no --> fallback_small([original path: small workload or probe failure])
    threshold -- yes --> native_wc[run cap native line-count aggregate]
    native_wc --> explain[cap explain reports native_wc_lines vs original fallback]
    explain --> parity[stdout/stderr/exit parity tests]
    parity --> bench[CPU/RSS large-workload benchmark]
    bench --> done([promoted high-volume shape])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: cap-high-volume-native-command-coverage-tests
requirements:
  wc_small_fallback:
    id: HV-UT-1
    text: "Planner keeps wc -l small file sets on the original command path."
    kind: functional
    risk: high
    verify: test
  wc_large_native:
    id: HV-UT-2
    text: "Planner promotes wc -l many-file or large-byte regular-file operands to the native aggregate line-count path."
    kind: functional
    risk: high
    verify: test
  wc_parity_success:
    id: HV-UT-3
    text: "Native wc -l preserves stdout and exit status for single-file and multi-file success cases, including total rows."
    kind: functional
    risk: high
    verify: test
  wc_parity_errors:
    id: HV-UT-4
    text: "Missing paths, directories, stdin operands, and unsupported wc flags fail open to the original path."
    kind: functional
    risk: high
    verify: test
  explain_visibility:
    id: HV-UT-5
    text: "cap explain reports native_wc_lines for promoted workloads and original fallback for small or unsupported workloads."
    kind: functional
    risk: medium
    verify: test
  benchmark_evidence:
    id: HV-UT-6
    text: "command_resources includes a large wc -l row comparing cap native aggregate against the original system command with CPU and peak RSS evidence."
    kind: functional
    risk: high
    verify: benchmark
elements:
  planner_threshold_tests:
    kind: test
    type: "cargo test -p cap command_planner"
  replacement_parity_tests:
    kind: test
    type: "cargo test -p cap behavior_cap_command_replacement_parity"
  explain_tests:
    kind: test
    type: "cargo test -p cap explain"
  resource_benchmark_matrix:
    kind: benchmark
    type: "cargo bench -p cap --bench command_resources"
relations:
  - { from: planner_threshold_tests, verifies: wc_small_fallback }
  - { from: planner_threshold_tests, verifies: wc_large_native }
  - { from: replacement_parity_tests, verifies: wc_parity_success }
  - { from: replacement_parity_tests, verifies: wc_parity_errors }
  - { from: explain_tests, verifies: explain_visibility }
  - { from: resource_benchmark_matrix, verifies: benchmark_evidence }
---
requirementDiagram
  requirement wc_small_fallback {
    id: HV-UT-1
    text: "small wc -l workloads use original path"
    risk: high
    verifymethod: test
  }
  requirement wc_large_native {
    id: HV-UT-2
    text: "large wc -l workloads use native aggregate path"
    risk: high
    verifymethod: test
  }
  requirement wc_parity_success {
    id: HV-UT-3
    text: "native wc -l success output matches system wc"
    risk: high
    verifymethod: test
  }
  requirement wc_parity_errors {
    id: HV-UT-4
    text: "unsupported or error cases fail open"
    risk: high
    verifymethod: test
  }
  requirement explain_visibility {
    id: HV-UT-5
    text: "explain shows promoted versus fallback path"
    risk: medium
    verifymethod: test
  }
  requirement benchmark_evidence {
    id: HV-UT-6
    text: "benchmarks prove high-volume resource win"
    risk: high
    verifymethod: benchmark
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
      Add a workload-gated WcLines native command plan for `wc -l FILE...`.
      The planner accepts only regular-file operands with no stdin, directory,
      or unsupported flag semantics, and promotes only file sets meeting the
      high-volume gate. Small, missing, or unsupported shapes remain External
      Original so behavior stays delegated to the system command.

  - path: projects/cap/src/command_planner.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Add the Rust native aggregate line-count runner and `cap explain`
      rendering for promoted `wc -l` workloads. The runner must preserve the
      system `wc -l` success shape, including per-file rows and the multi-file
      total row.

  - path: projects/cap/src/cap_fast_frontend.c
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Add the low-overhead production fast path for `wc -l FILE...`, sharing
      the same workload gate as the Rust planner. The C frontend returns
      unsupported for small or unsafe shapes so the public `cap` launcher
      continues through `cap-full` and original-command fallback.

  - path: projects/cap/src/cap_fast_frontend.c
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Register `wc` as an active same-name candidate only after the fast path
      can prove regular-file safety and large-workload eligibility. Do not add
      arbitrary `wc` option support in this slice.

  - path: projects/cap/tests/behavior_cap_command_replacement_parity.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: >
      Extend installed-frontend parity coverage with large `wc -l` success,
      `cap run "wc -l ..."` success, and fallback/error cases for missing
      paths or unsupported operands.

  - path: projects/cap/benches/command_resources.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: >
      Add a high-volume `wc -l` benchmark scenario with the dual-win gate,
      comparing the production C frontend against `/usr/bin/wc` using median
      child CPU time and peak RSS.

  - path: projects/cap/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Document `wc -l` as an active high-volume fast path and keep pipe-shaped
      candidates such as `find ... | xargs wc -l` listed as scout-only until
      shell/pipeline fusion has parity and benchmark proof.

  - path: projects/cap/BENCHMARKS.md
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: >
      Record the measured `wc -l` resource result and the gating decision after
      running the command resource benchmark.
```
