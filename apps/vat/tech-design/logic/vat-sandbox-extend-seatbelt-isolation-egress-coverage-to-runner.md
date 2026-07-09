---
id: vat-sandbox-extend-seatbelt-isolation-egress-coverage-to-runner
summary: Close out the runner-mode sandbox coverage gap for vat's network sandbox — runner-mode commands already run through the fail-closed `sandbox::pick` + `sandbox_wrap` path (issue #1300's Result-based signature included), so this WI adds the missing regression proof (an `EgressPolicy::Deny` runner-mode denial and an explicit intentional-exemption test for vat's own spawned services) and reconciles the predecessor design doc's `coverage: partial` frontmatter to `full`.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: sandbox-applied-to-runner-mode-commands
    claim: sandbox-applied-to-runner-mode-commands
    coverage: full
    rationale: "Runner-mode commands (spawn_runner_process, run_setup_step) already resolve through sandbox::pick (fail-closed per #1300) and sandbox_wrap identically to the direct-mode path; this WI adds the EgressPolicy::Deny runner-mode denial proof and the explicit vat-service-exemption test that were the remaining gap, and reconciles the predecessor doc's coverage frontmatter to full."
---

# Extend Seatbelt isolation/egress coverage to runner-mode commands

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-sandbox-runner-coverage-logic
entry: start
nodes:
  start: { kind: start, label: "vat run configured resolves EnvSpec isolation plus egress for the whole run" }
  pick: { kind: process, label: "sandbox pick spec returns Result fails closed per issue 1300 when a non open egress cannot be enforced" }
  pickerr: { kind: terminal, label: "pick returns Err run_configured maps to anyhow and aborts before any service or runner work" }
  svc: { kind: process, label: "start_service spawns emulator or http-mock proxy via command_with_logs RAW no sandbox_wrap call intentionally unsandboxed so services keep network" }
  runner: { kind: process, label: "spawn_runner_process and run_setup_step call sandbox_wrap backend rootfs cmd before command_with_logs" }
  wrapdecide: { kind: decision, label: "backend name" }
  seatwrap: { kind: process, label: "seatbelt resolve yields sandbox-exec -p profile program args confining writes to rootfs and enforcing egress" }
  passthrough: { kind: process, label: "process resolve returns cmd unchanged" }
  denytest: { kind: process, label: "regression coverage adds EgressPolicy Deny alongside existing LocalhostOnly for the runner-mode e2e proof" }
  exempttest: { kind: process, label: "regression coverage asserts a spawned service reaches the network under EgressPolicy Deny while the sibling runner in the same run is denied confirming the exemption is intentional not an oversight" }
  effect: { kind: terminal, label: "runner and setup-step commands are sandboxed identically to direct mode services remain unsandboxed by design old partial-coverage doc reconciled to full" }
edges:
  - { from: start, to: pick }
  - { from: pick, to: pickerr, label: "Err" }
  - { from: pick, to: svc, label: "Ok" }
  - { from: pick, to: runner, label: "Ok" }
  - { from: svc, to: exempttest }
  - { from: runner, to: wrapdecide }
  - { from: wrapdecide, to: seatwrap, label: "seatbelt" }
  - { from: wrapdecide, to: passthrough, label: "process" }
  - { from: seatwrap, to: denytest }
  - { from: passthrough, to: effect }
  - { from: denytest, to: effect }
  - { from: exempttest, to: effect }
---
```

## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-sandbox-runner-coverage.schema.json"
title: "Runner-mode sandbox coverage: proof surface"
type: object
properties:
  already_wired:
    type: array
    items: { type: string }
    description: "Runner-mode call sites that already resolve through sandbox::pick (Result, fail-closed per #1300) and sandbox_wrap: spawn_runner_process (runner.cmd), run_setup_step (step.cmd)."
  intentionally_unwrapped:
    type: array
    items: { type: string }
    description: "vat-spawned services that must never be passed through sandbox_wrap: start_service (emulator/proxy spawns) — kept RAW so they retain network."
  fail_closed:
    type: object
    description: "sandbox::pick's Result<Box<dyn Sandbox>, String> contract (issue #1300): isolation=none + non-Open egress errors; seatbelt requested + unavailable + non-Open egress errors; both propagate via `.map_err(anyhow::Error::msg)?` at every call site, runner-mode included."
    properties:
      ok: { type: string, description: "Box<dyn Sandbox> backend to thread into sandbox_wrap at runner/setup/service call sites." }
      err: { type: string, description: "String explaining why the requested isolation/egress combination cannot be enforced; propagated, never downgraded." }
  new_test_coverage:
    type: array
    items: { type: string }
    description: "Regression proof this WI adds: (1) a runner-mode command with EgressPolicy::Deny is denied outbound network the same way direct-mode already is; (2) a service spawned in the same run still reaches the network under EgressPolicy::Deny, confirming the exemption is intentional."
additionalProperties: true
```

## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-sandbox-runner-coverage-config.schema.json"
title: "no new config surface"
type: object
properties:
  note:
    type: string
    description: "No new vat.toml or CLI flag. Reuses the existing --isolation flag and [network].egress from vat.toml; this WI does not add config, it closes the remaining regression-proof and doc-reconciliation gap for coverage that already applies --isolation/[network].egress to runner-mode commands (spawn_runner_process, run_setup_step) via the fail-closed sandbox::pick (#1300) + sandbox_wrap path, while start_service spawns stay intentionally unwrapped."
additionalProperties: true
```
