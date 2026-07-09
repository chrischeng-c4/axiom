---
id: vat-sandbox-egress-policy-silently-unenforced-when-isolation-non
summary: Fix a security gap where vat's sandbox backends silently drop a non-`Open` egress policy instead of enforcing it — the passthrough `ProcessBackend` (isolation none) and the seatbelt-unavailable fallback both warn-and-continue with unrestricted network today; both paths now return a hard error instead, while `Isolation::None` + `EgressPolicy::Open` keeps working unchanged.
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: sandbox-egress-policy-fails-closed-when-isolation-cannot-enforce-it
    claim: sandbox-egress-policy-fails-closed-when-isolation-cannot-enforce-it
    coverage: partial
    rationale: "The v3 seatbelt egress policy (#518) and its runner-mode application (#527) can be silently bypassed: isolation=none warns-and-runs-open instead of enforcing Deny/LocalhostOnly, and a seatbelt-unavailable host silently downgrades to the unsandboxed ProcessBackend. Failing closed (hard error) instead of warning closes the enforcement gap so a caller-set egress policy is never silently ignored."
---

# vat sandbox: egress policy silently unenforced when Isolation::None

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-sandbox-egress-fail-closed-logic
entry: start
nodes:
  start: { kind: start, label: "vat run resolves EnvSpec isolation plus egress for the run" }
  policy: { kind: decision, label: "egress policy is open" }
  passthrough: { kind: process, label: "open unaffected process backend runs unrestricted no error" }
  backend: { kind: decision, label: "selected backend is process none" }
  seatbelt_avail: { kind: decision, label: "isolation seatbelt requested sandbox-exec available" }
  err_process: { kind: process, label: "process.rs returns hard Err egress policy requires seatbelt isolation none cannot enforce" }
  err_fallback: { kind: process, label: "mod.rs returns hard Err sandbox-exec unavailable cannot fall back to process while egress is non-open" }
  fail: { kind: terminal, label: "vat run exits non-zero clear error no command executed" }
  seatbelt_run: { kind: process, label: "seatbelt backend enforces egress via sandbox-exec profile" }
  effect: { kind: terminal, label: "runner executes under enforced egress policy or open passthrough" }
edges:
  - { from: start, to: policy }
  - { from: policy, to: passthrough, label: "open" }
  - { from: passthrough, to: effect }
  - { from: policy, to: backend, label: "localhost-only or deny" }
  - { from: backend, to: err_process, label: "process none" }
  - { from: backend, to: seatbelt_avail, label: "seatbelt requested" }
  - { from: seatbelt_avail, to: seatbelt_run, label: "available" }
  - { from: seatbelt_avail, to: err_fallback, label: "unavailable non-macOS or missing binary" }
  - { from: err_process, to: fail }
  - { from: err_fallback, to: fail }
  - { from: seatbelt_run, to: effect }
---
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: vat-sandbox-egress-fail-closed-unit-tests
---
requirementDiagram
    requirement process_backend_rejects_non_open_egress {
      id: UT1
      text: "ProcessBackend (isolation=none) with EgressPolicy::LocalhostOnly or EgressPolicy::Deny returns Err (not a printed warning) and does not execute the command; the error names the egress policy and states isolation=none cannot enforce it."
      risk: high
      verifymethod: test
    }
    requirement process_backend_open_unaffected {
      id: UT2
      text: "ProcessBackend (isolation=none) with EgressPolicy::Open still returns Ok and runs the command exactly as before (no regression on the common-case path)."
      risk: high
      verifymethod: test
    }
    requirement seatbelt_unavailable_rejects_non_open_egress {
      id: UT3
      text: "Resolving Isolation::Seatbelt with a non-Open egress policy on a host where sandbox-exec is unavailable returns Err instead of silently falling back to ProcessBackend; the error names the missing seatbelt backend and the requested egress policy."
      risk: high
      verifymethod: test
    }
    requirement seatbelt_unavailable_open_falls_back {
      id: UT4
      text: "Resolving Isolation::Seatbelt with EgressPolicy::Open on a host where sandbox-exec is unavailable still falls back to ProcessBackend and returns Ok (fallback is only rejected when it would silently drop enforcement)."
      risk: medium
      verifymethod: test
    }
    test sandbox_egress_fail_closed_tests {
      type: functional
      verifies: process_backend_rejects_non_open_egress
    }
    test sandbox_egress_open_unaffected_tests {
      type: functional
      verifies: process_backend_open_unaffected
    }
    test sandbox_seatbelt_fallback_fail_closed_tests {
      type: functional
      verifies: seatbelt_unavailable_rejects_non_open_egress
    }
    test sandbox_seatbelt_fallback_open_tests {
      type: functional
      verifies: seatbelt_unavailable_open_falls_back
    }
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/sandbox/process.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Logic section edge: the passthrough backend (isolation=none) must fail closed instead of warn-and-continue when the run's EgressPolicy is not Open — this is source-level control-flow surgery on an existing hand-written backend, not a generator-template concern."
  - path: projects/vat/src/sandbox/mod.rs
    action: modify
    section: logic
    impl_mode: hand-written
    reason: "Logic section edge: the seatbelt-unavailable fallback (around the existing lines 57-75) must fail closed instead of silently downgrading to ProcessBackend when the run's EgressPolicy is not Open — hand-written backend-selection logic."
  - path: projects/vat/tests/vat_sandbox_egress_fail_closed.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "Unit-test section edge: regression coverage for UT1-UT4 — process.rs rejects non-Open egress under isolation=none, Open stays unaffected, the seatbelt-unavailable fallback rejects non-Open egress, and the seatbelt-unavailable + Open case still falls back successfully."
```
