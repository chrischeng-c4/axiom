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
## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat run
    behavior:
      - "No new flags. `--isolation seatbelt` and `[network].egress` already confine runner/step commands identically to `vat run -- <cmd>` (spawn_runner_process / run_setup_step both call sandbox_wrap(backend, rootfs, cmd) before command_with_logs)."
      - "sandbox::pick(spec) is called once per `vat run` and now returns Result<Box<dyn Sandbox>, String> (issue #1300, fail-closed); run_configured propagates its Err via `.map_err(anyhow::Error::msg)?` before any service or runner work starts — an unenforceable isolation/egress combination aborts the whole run instead of silently degrading."
      - "Service/emulator spawns (start_service) remain unwrapped by sandbox_wrap — they keep network to serve/forward regardless of --isolation/[network].egress. This WI adds an explicit test proving that exemption is intentional, not an oversight."
      - "`--isolation none` (default) is unchanged."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-sandbox-runner-coverage-unit-tests
---
requirementDiagram
    requirement pick_fails_closed_reaches_runner_mode {
      id: UT1
      text: "sandbox::pick(spec) returns Err (not a silently-degraded backend) when the run's isolation/egress combination cannot be enforced, BEFORE run_configured spawns any service or runner; the error propagates via `.map_err(anyhow::Error::msg)?` at the runner-mode call site exactly as it already does at the direct-mode call site (issue #1300)."
      risk: high
      verifymethod: test
    }
    requirement runner_and_setup_step_wrapped {
      id: UT2
      text: "spawn_runner_process and run_setup_step both resolve their command through sandbox_wrap(backend, rootfs, cmd) before command_with_logs — under isolation=seatbelt the resolved argv is the sandbox-exec wrapped form; under isolation=none it is the raw command unchanged."
      risk: high
      verifymethod: test
    }
    requirement services_never_wrapped {
      id: UT3
      text: "start_service never calls sandbox_wrap for a real service spawn (service_sandbox stays None on the non-hermetic-proxy path); this is asserted directly, not just implied, as the explicit proof of the intentional vat-services exemption (R2/AC2)."
      risk: high
      verifymethod: test
    }
    test pick_fail_closed_tests {
      type: functional
      verifies: pick_fails_closed_reaches_runner_mode
    }
    test runner_setup_step_wrap_tests {
      type: functional
      verifies: runner_and_setup_step_wrapped
    }
    test service_exemption_tests {
      type: functional
      verifies: services_never_wrapped
    }
```
## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-runner-sandbox-deny-egress
    name: "runner-mode command with EgressPolicy::Deny is denied outbound network, same as direct-mode"
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: sandbox-applied-to-runner-mode-commands
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_runner_sandbox -- --nocapture"
    assertions:
      - "AC4: a `vat run <runner>` with `--isolation seatbelt` + `[network].egress = deny` denies a runner command's outbound connection (loopback and non-loopback alike), exit code non-zero, matching the existing direct-mode `vat_sandbox_egress` proof for EgressPolicy::Deny. Skips cleanly off-macOS / no sandbox-exec / no bash."
  - id: vat-runner-sandbox-service-exemption
    name: "vat's own spawned services stay unsandboxed under runner-mode Deny egress (intentional exemption, not an oversight)"
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: sandbox-applied-to-runner-mode-commands
    contract_id: local-agent-test-runner-protocol
    category: behavior
    command: "cargo test -p vat --test vat_runner_sandbox -- --nocapture"
    assertions:
      - "AC2/R2: in the same `vat run` invoked with `[network].egress = deny`, a declared service (started via start_service, never sandbox_wrap'd) still binds/serves and remains reachable on its local port, while a sibling runner attempting outbound network under the same run is denied — proving the services exemption is enforced by construction, not a byproduct of a permissive default."
  - id: vat-runner-sandbox-build
    name: "default + lean build compile"
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: sandbox-applied-to-runner-mode-commands
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
  - path: projects/vat/tests/vat_runner_sandbox.rs
    action: modify
    section: e2e-test
    impl_mode: hand-written
    reason: "AC4: add a runner-mode `EgressPolicy::Deny` denial test alongside the existing localhost-only allow/deny test — a runner command with `--isolation seatbelt` + `[network].egress = deny` must be denied outbound network the same way `vat_sandbox_egress`'s direct-mode Deny case already is. AC2: add a sibling assertion in the same run that a declared service (started via start_service, never sandbox_wrap'd) still reaches/serves its local port under the same Deny policy, proving the vat-service exemption is intentional rather than a permissive-default accident."
  - path: projects/vat/src/commands/run.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    reason: "UT3: extend the existing `#[cfg(test)]` module with a direct unit assertion that `start_service`'s real (non-hermetic-proxy) call path never threads a `Some(sandbox)` into `service_start_command` — the explicit code-level proof (next to the existing `direct_start_service_command_uses_supplied_sandbox_only_for_direct_services` fixture-style test) that the services-stay-unsandboxed decision is asserted, not merely commented."
  - path: projects/vat/tech-design/logic/apply-the-sandbox-seatbelt-isolation-egress-to-runner-mode-comma.md
    action: modify
    section: changes
    impl_mode: hand-written
    reason: "AC3: reconcile the predecessor design doc's `capability_refs[0].coverage` frontmatter from `partial` to `full` now that runner-mode commands (spawn_runner_process, run_setup_step) are proven to honor EnvSpec.isolation/egress identically to direct-mode, the fail-closed `sandbox::pick` Result signature (#1300) is threaded through every call site, and the services exemption plus the EgressPolicy::Deny runner-mode case both have explicit regression coverage."
```
