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
