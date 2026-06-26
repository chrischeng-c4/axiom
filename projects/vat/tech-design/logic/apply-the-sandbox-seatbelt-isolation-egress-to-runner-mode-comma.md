---
id: apply-the-sandbox-seatbelt-isolation-egress-to-runner-mode-comma
summary: Wire vat's sandbox (seatbelt write-confinement + the #518 network egress policy) into runner-mode command execution, so `vat run <runner>` confines the runner/step commands the same way `vat run -- cmd` already does — while keeping vat-spawned services (emulators/proxy) unsandboxed so they keep their network.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: local-agent-test-runner-protocol
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "The sandbox is enforced only for direct mode today; applying it to runner mode makes seatbelt isolation + egress confinement protect the common `vat run <runner>` workflow, completing the network sandbox."
---

# Apply the sandbox (seatbelt isolation + egress) to runner-mode commands

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-sandbox-runner-mode-logic
entry: start
nodes:
  start: { kind: start, label: "vat run runner prepares services then runs runner commands" }
  svc: { kind: process, label: "start_service spawns emulator proxy via command_with_logs RAW no sandbox keeps network" }
  spec: { kind: process, label: "resolve EnvSpec isolation plus egress for the run" }
  pick: { kind: process, label: "backend equals sandbox pick spec" }
  isdir: { kind: decision, label: "runner or step command" }
  wrap: { kind: process, label: "backend resolve rootfs cmd0 cmdrest yields sandboxed argv" }
  exec: { kind: process, label: "command_with_logs runs the wrapped runner command" }
  effect: { kind: terminal, label: "runner confined writes rootfs egress per policy services untouched" }
edges:
  - { from: start, to: svc }
  - { from: start, to: spec }
  - { from: spec, to: pick }
  - { from: pick, to: isdir }
  - { from: isdir, to: wrap, label: "runner or step" }
  - { from: wrap, to: exec }
  - { from: exec, to: effect }
  - { from: svc, to: effect }
```
## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-sandbox-runner-mode.schema.json"
title: "Runner-mode sandbox wrapping"
type: object
properties:
  wrapped:
    type: array
    items: { type: string }
    description: "Command kinds wrapped via backend.resolve: runner.cmd, step.cmd."
  unwrapped:
    type: array
    items: { type: string }
    description: "Never wrapped: service/emulator spawns (start_service) — they need network."
  inputs:
    type: object
    description: "The wrap reuses the run's EnvSpec (isolation + egress), the vat rootfs as writable root, and the runner cwd."
    properties:
      isolation: { type: string, enum: [none, seatbelt] }
      egress: { type: string, enum: [open, localhost-only, deny] }
additionalProperties: true
```
## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-sandbox-runner-mode-config.schema.json"
title: "no new config"
type: object
properties:
  note:
    type: string
    description: "No new vat.toml/CLI surface. Reuses --isolation and [network].egress; this WI only changes WHERE they apply (runner commands too, not just direct mode)."
additionalProperties: true
```
## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat run
    behavior:
      - "Runner mode (`vat run <runner>`): runner and step commands are now wrapped through the same sandbox backend as direct mode — `--isolation seatbelt` confines their writes to the rootfs and applies the [network].egress policy."
      - "Service/emulator spawns are NOT sandboxed (they keep network to serve/forward)."
      - "`--isolation none` (default) is unchanged; the egress-needs-seatbelt warning still fires once when applicable."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-sandbox-runner-mode-unit-tests
---
requirementDiagram
    requirement runner_cmd_wrapped {
      id: UT1
      text: "Under isolation=seatbelt, a runner/step command's resolved argv is the sandbox-exec wrapped form (prog=sandbox-exec, -p profile); under isolation=none it is the raw command."
      risk: high
      verifymethod: test
    }
    requirement services_not_wrapped {
      id: UT2
      text: "A service spawn is never sandbox-wrapped regardless of isolation (it keeps its raw command)."
      risk: high
      verifymethod: test
    }
    test runner_cmd_wrapping_tests {
      type: functional
      verifies: runner_cmd_wrapped
    }
    test services_unwrapped_tests {
      type: functional
      verifies: services_not_wrapped
    }
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-runner-sandbox-egress-smoke
    name: "runner-mode seatbelt egress confines the runner, not services"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_runner_sandbox -- --nocapture"
    assertions:
      - "a `vat run <runner>` with isolation=seatbelt + egress=localhost-only runs the runner under sandbox-exec; the runner can reach a localhost listener but a connect to a non-loopback host is denied; an emulator service still reaches the network. Skips off-macOS / no sandbox-exec."
  - id: vat-runner-sandbox-build
    name: "default + lean build compile"
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo build -p vat --no-default-features"
    assertions:
      - "vat compiles with and without default features."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/commands/run.rs
    action: modify
    section: source
    impl_mode: hand-written
    reason: "At the runner + step exec sites (command_with_logs(&runner.cmd…) / (&step.cmd…)), build the sandbox backend from the run's EnvSpec and resolve the wrapped argv (rootfs writable root) before spawning; leave the start_service call site raw so services keep network. Thread spec/backend/rootfs to those sites."
  - path: projects/vat/tests/vat_runner_sandbox.rs
    action: create
    section: e2e-test
    impl_mode: hand-written
    reason: "Assert runner/step commands are wrapped under seatbelt and services are not; a localhost-only egress smoke for a runner (skips off-macOS)."
```
