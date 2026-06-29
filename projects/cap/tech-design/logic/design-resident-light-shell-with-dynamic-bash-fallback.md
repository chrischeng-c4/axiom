---
id: design-resident-light-shell-with-dynamic-bash-fallback
summary: Design and land the first resident light-shell runtime slice for cap run command strings with Bash fallback.
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: command-lease-throttling
    role: primary
    gap: lease-admission-and-process-supervision
    claim: lease-admission-and-process-supervision
    coverage: partial
    rationale: "The resident light shell is an execution path behind cap run and must preserve supervised original-command fallback semantics."
  - id: agent-hook-installation
    role: primary
    gap: hook-payload-rewrite-adapters
    claim: hook-payload-rewrite-adapters
    coverage: partial
    rationale: "Agent hooks route Bash Tool payloads into cap run command strings, so the light shell must preserve hook compatibility."
---

# Design Resident Light Shell With Dynamic Bash Fallback

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: cap-resident-light-shell-contract
entry: cap_run
nodes:
  cap_run: { kind: start, label: "cap run receives original command string" }
  session: { kind: process, label: "ResidentLightShellSession captures cwd/env snapshot" }
  parse: { kind: decision, label: "conservative light-shell subset?" }
  planner: { kind: process, label: "route shell-free argv through command_planner" }
  native_stage: { kind: process, label: "run native command stage in process" }
  bash_fallback: { kind: process, label: "bash -lc original command" }
  managed_run: { kind: process, label: "existing managed_run lease path for external fallback" }
  boundary: { kind: terminal, label: "Bash-compatible optimizer/resource governor, not full shell or sandbox" }
edges:
  - { from: cap_run, to: session, label: "agent hook or direct CLI" }
  - { from: session, to: parse, label: "preserve session cwd/env" }
  - { from: parse, to: planner, label: "single shell-free argv" }
  - { from: parse, to: bash_fallback, label: "unsupported syntax / unproven shape" }
  - { from: planner, to: native_stage, label: "Native plan" }
  - { from: planner, to: bash_fallback, label: "External Original/Replacement plan" }
  - { from: native_stage, to: boundary, label: "stdout/stderr/exit parity evidence" }
  - { from: bash_fallback, to: managed_run, label: "resource protection preserved" }
  - { from: managed_run, to: boundary, label: "daemon-backed fallback" }
---
flowchart TB
  cap_run["cap run receives original command string"] --> session["ResidentLightShellSession captures cwd/env snapshot"]
  session --> parse{"conservative light-shell subset?"}
  parse -->|single shell-free argv| planner["command_planner::plan"]
  parse -->|unsupported syntax / unproven shape| bash_fallback["bash -lc original command"]
  planner -->|Native plan| native_stage["run native command stage in process"]
  planner -->|External Original/Replacement plan| bash_fallback
  native_stage --> boundary["Bash-compatible optimizer/resource governor, not full shell or sandbox"]
  bash_fallback --> managed_run["existing managed_run lease path"]
  managed_run --> boundary
```

The resident-light-shell design belongs in this TD because it changes the
`cap run "<command string>"` execution boundary. The first implementation slice
is intentionally narrow: a per-invocation `ResidentLightShellSession` owns the
current cwd/env snapshot, attempts one conservative native stage using the
existing planner/native runner, and returns a structured fallback for every
unsupported form. This keeps the daemon/client resource-protection path intact
for Bash fallback while making an observable in-process native path available
for future resident/session reuse.

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: cap-resident-light-shell-contract-tests
requirements:
  native_path:
    id: RLS-UT-1
    text: "A shell-free command string with a workload-qualified native plan runs through ResidentLightShellSession without spawning Bash."
    kind: functional
    risk: medium
    verify: test
  fallback_path:
    id: RLS-UT-2
    text: "Unsupported shell syntax returns a Bash fallback plan that preserves the original command string exactly."
    kind: functional
    risk: high
    verify: test
  parity:
    id: RLS-UT-3
    text: "The first resident native path and the Bash fallback path both preserve stdout, stderr, and exit status against the original command."
    kind: functional
    risk: high
    verify: test
  product_boundary:
    id: RLS-UT-4
    text: "README/TD state that cap is a Bash-compatible optimizer and resource governor, not a sandbox or full replacement shell."
    kind: functional
    risk: medium
    verify: test
elements:
  resident_shell_unit_tests:
    kind: test
    type: "cargo test -p cap resident_light_shell"
  resident_run_parity:
    kind: test
    type: "cargo test -p cap resident_light_shell_run_parity"
  readme_boundary_smoke:
    kind: test
    type: "cargo test -p cap docs"
relations:
  - { from: resident_shell_unit_tests, verifies: native_path }
  - { from: resident_shell_unit_tests, verifies: fallback_path }
  - { from: resident_run_parity, verifies: parity }
  - { from: readme_boundary_smoke, verifies: product_boundary }
---
requirementDiagram
  requirement native_path {
    id: RLS-UT-1
    text: "A shell-free command string with a workload-qualified native plan runs through ResidentLightShellSession without spawning Bash."
    risk: medium
    verifymethod: test
  }
  requirement fallback_path {
    id: RLS-UT-2
    text: "Unsupported shell syntax returns a Bash fallback plan that preserves the original command string exactly."
    risk: high
    verifymethod: test
  }
  requirement parity {
    id: RLS-UT-3
    text: "The first resident native path and the Bash fallback path both preserve stdout, stderr, and exit status against the original command."
    risk: high
    verifymethod: test
  }
  requirement product_boundary {
    id: RLS-UT-4
    text: "README/TD state that cap is a Bash-compatible optimizer and resource governor, not a sandbox or full replacement shell."
    risk: medium
    verifymethod: inspection
  }
```

The applicability proof requires unit coverage on the resident session's
planning boundary plus behavior coverage through the public `cap run` path. A
minimal test fixture can use a threshold-sized `ls -1 <dir>` native path because
#117 made the workload gate explicit; fallback parity can use a pipe or `cd &&
pwd` shape that must remain under Bash semantics.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/cap/src/resident_shell.rs
    action: create
    section: logic
    impl_mode: hand-written
    description: >
      Add the first resident light-shell session boundary. The session captures
      cwd/env, plans a command string through the existing command planner, runs
      native command stages in process, and returns a Bash fallback plan for
      unsupported or unproven command strings.

  - path: projects/cap/src/resident_shell.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: >
      Cover the native observable path and dynamic Bash fallback path with
      byte-for-byte stdout/stderr/exit parity checks against original system
      commands.

  - path: projects/cap/src/command_planner.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Expose the native runner's capture helper inside the crate so the
      resident shell can verify native-stage output without spawning a second
      cap process.

  - path: projects/cap/src/cli.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Route `cap run "<command string>"` through ResidentLightShellSession
      before falling back to the existing external managed_run path. Keep argv
      mode behavior unchanged.

  - path: projects/cap/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: >
      Export the resident shell module inside the cap crate.

  - path: projects/cap/README.md
    action: modify
    section: overview
    impl_mode: hand-written
    description: >
      Document that cap is adding a resident light-shell optimizer layer with
      dynamic Bash fallback, while remaining a resource governor rather than a
      sandbox or full shell replacement.
```
