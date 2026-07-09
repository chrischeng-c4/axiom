---
id: add-workload-sensitive-native-command-gates
summary: Add shape-sensitive native command takeover so cap keeps unknown or shell-sensitive shapes on the original path while safe same-name subsets run natively at any size.
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
    rationale: "Native takeover uses behavior parity for safe shell-free subsets, while benchmarked CPU and peak RSS evidence remain regression gates for representative large workloads."
---

# TD: cap shape-sensitive native command takeover

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
  parity: { kind: process, label: "require parity test coverage for this shape" }
  bench: { kind: process, label: "record benchmark row for representative workloads" }
  promote: { kind: decision, label: "shape is safe and parity-covered?" }
  native: { kind: terminal, label: "native/replacement path active" }
  fallback_gate: { kind: terminal, label: "original path: shape not parity-covered" }
edges:
  - { from: start, to: shape }
  - { from: shape, to: fallback_shape, label: "no" }
  - { from: shape, to: parity, label: "yes" }
  - { from: parity, to: bench }
  - { from: bench, to: promote }
  - { from: promote, to: native, label: "yes" }
  - { from: promote, to: fallback_gate, label: "no" }
safe_subsets:
  ls:
    shape: "simple -1/-a/-A over one existing directory"
  sort:
    shape: "one regular file"
  cut_field:
    shape: "cut single field over one regular file with optional one-byte delimiter"
  tr_stream:
    shape: "tr ASCII byte translate sets or tr -d set over stdin"
  grep:
    shape: "recursive literal grep -R pattern root"
  find:
    shape: "root -type f -name pattern"
  sed_print:
    shape: "sed -n start,endp file"
  echo_plain:
    shape: "echo plain args; echo -n with non-option args"
  printf_string:
    shape: "printf exact %s or %s\\n over string args"
  seq_integer:
    shape: "seq integer ranges with one, two, or three args and nonzero step"
  whoami:
    shape: "whoami with no flags"
  id_identity:
    shape: "id -u, id -un, id -g, id -gn"
  uname_fields:
    shape: "uname with no flags or exact -s, -n, -r, -v, -m, -a"
  test_predicates:
    shape: "test/[ unary file/string predicates, string equality/inequality, integer comparisons, optional leading !; bracket argv requires trailing ]"
  awk_count:
    shape: "awk '/NEEDLE/ { c++ } END { print c }' file"
  xargs_safe:
    shape: "xargs echo over stdin tokens; xargs wc -l over stdin path tokens"
  fused_pipes:
    shape: "echo args | wc -l; echo args | head -n N; echo args | tail -n N; echo args | tr set1 set2; printf '%s\\n' args | wc -l; printf '%s\\n' args | head -n N; printf '%s\\n' args | tail -n N; printf '%s\\n' args | grep literal; printf '%s\\n' args | tr set1 set2; seq integer args | wc -l; seq integer args | head -n N; seq integer args | tail -n N; yes optional-single-word | head -n N; ls optional-1 dir | wc -l; ls optional-1 dir | head -n N; ls optional-1 dir | tail -n N; ls optional-1 dir | sort; ls optional-1 dir | sort | uniq; ls optional-1 dir | sort | uniq | wc -l; ls optional-1 dir | sort | wc -l; ls optional-1 dir | sort | head -n N; ls optional-1 dir | sort | tail -n N; ls optional-1 dir | grep literal; ls optional-1 dir | grep literal | wc -l; sort file | uniq; sort file | uniq | wc -l; sort file | head -n N; sort file | tail -n N; sort file | wc -l; cat file | wc -l; cat file | head -n N; cat file | tail -n N; cat file | grep literal; cat file | cut -d char -f field; cat file | tr set1 set2; cat file | uniq; cat file | uniq | wc -l; cat file | sort; cat file | sort | uniq; cat file | sort | uniq | wc -l; cat file | sort | wc -l; cat file | sort | head -n N; cat file | sort | tail -n N; grep literal file | wc -l; grep literal file | head -n N; grep literal file | tail -n N; grep literal file | sort; grep literal file | sort | uniq; grep literal file | sort | uniq | wc -l; grep literal file | sort | wc -l; grep literal file | sort | head -n N; grep literal file | sort | tail -n N; grep -R pattern root | head -n N; grep -R pattern root | tail -n N; grep -R pattern root | sort; grep -R pattern root | sort | uniq; grep -R pattern root | sort | uniq | wc -l; grep -R pattern root | sort | wc -l; grep -R pattern root | sort | head -n N; grep -R pattern root | sort | tail -n N; grep -R pattern root | wc -l; awk '/NEEDLE/ { print $1 }' file | xargs echo; which names | wc -l; which names | head -n N; which names | tail -n N; command -v names | wc -l; command -v names | head -n N; command -v names | tail -n N; find root -type f optional-name-safe-basename-glob | xargs wc -l; find root -type f optional-name-safe-basename-glob | xargs echo; find root -type f optional-name-safe-basename-glob | xargs; find root -type f optional-name-safe-basename-glob | wc -l; find root -type f optional-name-safe-basename-glob | head -n N; find root -type f optional-name-safe-basename-glob | tail -n N; find root -type f optional-name-safe-basename-glob | sort; find root -type f optional-name-safe-basename-glob | sort | uniq; find root -type f optional-name-safe-basename-glob | sort | uniq | wc -l; find root -type f optional-name-safe-basename-glob | sort | wc -l; find root -type f optional-name-safe-basename-glob | sort | xargs echo; find root -type f optional-name-safe-basename-glob | sort | xargs wc -l; find root -type f optional-name-safe-basename-glob | sort | head -n N; find root -type f optional-name-safe-basename-glob | sort | tail -n N"
  tiny_primitives:
    shape: "true, false, pwd, basename, dirname, positive-count head, nonnegative-count tail, mkdir, touch"
fallback_rule: "Any unsupported flag, unsupported stdin-dependent form, unsupported shell control syntax, or unproven shape preserves the original command path."
---
flowchart TD
    start([plan argv or shell-free command string]) --> shape{matches supported safe shape?}
    shape -- no --> fallback_shape([original path: unsupported shape])
    shape -- yes --> parity[require parity test coverage]
    parity --> bench[require benchmark row]
    bench --> promote{shape safe and parity-covered?}
    promote -- yes --> native([native/replacement path active])
    promote -- no --> fallback_gate([original path: shape not parity-covered])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: cap-workload-sensitive-native-gates-contract-tests
requirements:
  planner_safe_subset_native:
    id: R1
    text: "Planner returns native plans for supported safe shell-free command shapes at any size."
    kind: functional
    risk: high
    verify: test
  planner_unsupported_original:
    id: R2
    text: "Planner returns External Original for unsupported flags, stdin-dependent forms, and shell-sensitive shapes."
    kind: functional
    risk: high
    verify: test
  unsupported_errors_fail_open:
    id: R3
    text: "Missing paths where parity is not implemented and unsupported flags fail open to the original command path."
    kind: functional
    risk: high
    verify: test
  run_string_matches_argv:
    id: R4
    text: "Shell-free cap run strings and cap argv entrypoints make the same shape-sensitive decision."
    kind: functional
    risk: high
    verify: test
  benchmark_small_large_rows:
    id: R5
    text: "command_resources benchmark has takeover rows and large resource-gated rows."
    kind: functional
    risk: high
    verify: benchmark
  readme_describes_fast_paths:
    id: R6
    text: "README describes native commands as conservative shape-sensitive takeovers, not a replacement shell."
    kind: functional
    risk: medium
    verify: test
elements:
  planner_takeover_tests:
    kind: test
    type: "cargo test -p cap command_planner"
  run_string_takeover_tests:
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
  - { from: planner_takeover_tests, verifies: planner_safe_subset_native }
  - { from: planner_takeover_tests, verifies: planner_unsupported_original }
  - { from: planner_takeover_tests, verifies: unsupported_errors_fail_open }
  - { from: run_string_takeover_tests, verifies: run_string_matches_argv }
  - { from: parity_regression, verifies: run_string_matches_argv }
  - { from: parity_regression, verifies: unsupported_errors_fail_open }
  - { from: resource_benchmark_matrix, verifies: benchmark_small_large_rows }
  - { from: readme_wording_smoke, verifies: readme_describes_fast_paths }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "safe shell-free supported shapes use native path at any size"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "unsupported shapes keep original path"
      risk: high
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "unsupported flags fail open"
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
      text: "benchmarks include takeover and gated rows"
      risk: high
      verifymethod: benchmark
    }
    requirement R6 {
      id: R6
      text: "README says shape-sensitive takeovers"
      risk: medium
      verifymethod: test
    }
    element planner_takeover_tests {
      type: "cargo test"
    }
    element run_string_takeover_tests {
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
  - path: apps/cap/src/command_planner.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Add shape-sensitive native planning for safe shell-free same-name
      commands at any size. Unsupported flags, stdin-dependent forms, and shell
      control syntax must return an External Original plan.

  - path: apps/cap/src/cap_fast_frontend.c
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Mirror the Rust planner safe-subset rules in the public low-overhead C
      frontend so same-name native fast paths claim both small and large safe
      workloads while unsupported shapes fall through to cap-full.

  - path: apps/cap/src/command_planner.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: >
      Add planner tests proving small safe subsets use the active native or
      replacement path, unsupported options keep the original path, and
      shell-free cap run strings make the same shape-sensitive decision as argv.

  - path: apps/cap/benches/command_resources.rs
    action: modify
    section: e2e-test
    impl_mode: hand-written
    description: >
      Extend the benchmark matrix with takeover rows for tiny primitives and
      representative large rows for resource-gated workloads. Takeover rows are
      measured without CPU/RSS admission failure.

  - path: apps/cap/tests/behavior_cap_command_replacement_parity.rs
    action: modify
    section: e2e-test
    impl_mode: hand-written
    description: >
      Keep active replacement parity coverage for promoted same-name shapes and
      add regression coverage for tiny primitive takeover paths.

  - path: apps/cap/README.md
    action: modify
    section: overview
    impl_mode: hand-written
    description: >
      Reword native command replacement as conservative shape-sensitive
      takeover. Document that unknown or shell-sensitive workloads keep
      original-command behavior.

  - path: apps/cap/BENCHMARKS.md
    action: modify
    section: overview
    impl_mode: hand-written
    description: >
      Document takeover rows versus resource-gated large rows and the
      interpretation that default takeover depends on safe shape and parity,
      while large workloads keep CPU/RSS regression evidence.
```
