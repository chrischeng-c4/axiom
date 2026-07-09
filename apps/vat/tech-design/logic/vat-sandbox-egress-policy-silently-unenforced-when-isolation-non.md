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
```
