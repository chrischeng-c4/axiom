---
id: meter-capture-vitals-contract
summary: Single-knob meter.toml (level + [gate]) measurement contract, L1 vitals in capture (getrusage cpu/RSS + wall), until-exit sampling window with opaque --drive composition seam, collapsed artifact output, and removal of dead stress/load-test residue.
fill_sections: [logic, config, cli, unit-test]
capability_refs:
  - id: runtime-resource-attribution
    role: primary
    gap: capture-vitals-and-measurement-contract
    claim: capture-vitals-and-measurement-contract
    coverage: full
    rationale: "Closes the README known-limit 'Memory/RSS are not public gates yet' via L1 vitals findings plus declarative meter.toml [gate] adjudication."
  - id: legacy-carried-internals
    role: contributes
    gap: stress-residue-prune
    claim: stress-residue-prune
    coverage: full
    rationale: "Removes dead load-test residue (StressMetrics, TestType::Stress, reporter RPS table, orphaned fuzz_http) so the codebase stops advertising a capability meter must not have."
---

# TD: meter capture vitals + single-knob measurement contract

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: capture-vitals-flow
entry: start
nodes:
  start:           { kind: start,    label: "meter run/profile invoked" }
  load_config:     { kind: process,  label: "Load optional meter.toml" }
  resolve_level:   { kind: process,  label: "Resolve level: CLI flag > meter.toml > default vitals" }
  level_off:       { kind: decision, label: "level == off?" }
  spawn_target:    { kind: process,  label: "Spawn target child" }
  want_stacks:     { kind: decision, label: "level >= sample?" }
  attach_sampler:  { kind: process,  label: "Attach stack sampler to child pid" }
  has_drive:       { kind: decision, label: "--drive command given?" }
  exec_driver:     { kind: process,  label: "Exec opaque driver command" }
  wait_driver:     { kind: process,  label: "Driver exit ends measurement window" }
  wait_child:      { kind: process,  label: "Wait child exit, optional --duration-cap" }
  fold_stacks:     { kind: process,  label: "Fold sampled stacks into hotspot findings" }
  write_collapsed: { kind: process,  label: "Write collapsed artifact under .meter" }
  read_vitals:     { kind: process,  label: "Read getrusage cpu + peak RSS + wall clock" }
  emit_vital:      { kind: process,  label: "Emit Finding kind=vital" }
  gate_check:      { kind: decision, label: "[gate] threshold breached?" }
  gate_finding:    { kind: process,  label: "Severity >= Medium finding + escalate agent_prompt to --level sample" }
  build_report:    { kind: process,  label: "Fold worst-wins MeterReport" }
  done:            { kind: terminal, label: "Exit per ladder 0-5" }
  skip:            { kind: terminal, label: "No measurement performed" }
edges:
  - { from: start,           to: load_config }
  - { from: load_config,     to: resolve_level }
  - { from: resolve_level,   to: level_off }
  - { from: level_off,       to: skip,            label: "yes" }
  - { from: level_off,       to: spawn_target,    label: "no" }
  - { from: spawn_target,    to: want_stacks }
  - { from: want_stacks,     to: attach_sampler,  label: "yes" }
  - { from: want_stacks,     to: has_drive,       label: "no" }
  - { from: attach_sampler,  to: has_drive }
  - { from: has_drive,       to: exec_driver,     label: "yes" }
  - { from: exec_driver,     to: wait_driver }
  - { from: has_drive,       to: wait_child,      label: "no" }
  - { from: wait_driver,     to: fold_stacks }
  - { from: wait_child,      to: fold_stacks }
  - { from: fold_stacks,     to: write_collapsed }
  - { from: write_collapsed, to: read_vitals }
  - { from: read_vitals,     to: emit_vital }
  - { from: emit_vital,      to: gate_check }
  - { from: gate_check,      to: gate_finding,    label: "yes" }
  - { from: gate_check,      to: build_report,    label: "no" }
  - { from: gate_finding,    to: build_report }
  - { from: build_report,    to: done }
---
flowchart TD
    start([meter run/profile invoked]) --> load_config[Load optional meter.toml]
    load_config --> resolve_level[Resolve level: CLI flag > meter.toml > default vitals]
    resolve_level --> level_off{level == off?}
    level_off -->|yes| skip([No measurement performed])
    level_off -->|no| spawn_target[Spawn target child]
    spawn_target --> want_stacks{level >= sample?}
    want_stacks -->|yes| attach_sampler[Attach stack sampler to child pid]
    want_stacks -->|no| has_drive{--drive command given?}
    attach_sampler --> has_drive
    has_drive -->|yes| exec_driver[Exec opaque driver command]
    exec_driver --> wait_driver[Driver exit ends measurement window]
    has_drive -->|no| wait_child[Wait child exit, optional --duration-cap]
    wait_driver --> fold_stacks[Fold sampled stacks into hotspot findings]
    wait_child --> fold_stacks
    fold_stacks --> write_collapsed[Write collapsed artifact under .meter]
    write_collapsed --> read_vitals[Read getrusage cpu + peak RSS + wall clock]
    read_vitals --> emit_vital[Emit Finding kind=vital]
    emit_vital --> gate_check{[gate] threshold breached?}
    gate_check -->|yes| gate_finding[Severity >= Medium finding + escalate agent_prompt to --level sample]
    gate_check -->|no| build_report[Fold worst-wins MeterReport]
    gate_finding --> build_report
    build_report --> done([Exit per ladder 0-5])
```
