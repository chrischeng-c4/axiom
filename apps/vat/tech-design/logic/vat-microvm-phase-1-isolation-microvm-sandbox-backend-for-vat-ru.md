---
id: vat-microvm-phase-1-isolation-microvm-sandbox-backend-for-vat-ru
summary: (fill)
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: vat-microvm-phase1-pick-logic
entry: start
nodes:
  start: { kind: start, label: "vat run resolves EnvSpec isolation gpu egress microvm_image for the run" }
  preflight: { kind: decision, label: "gpu_satisfied gpu isolation info checked at the three run.rs GpuRequest Required call sites before any workspace clone begins" }
  preflight_err: { kind: terminal, label: "hard Err isolation micro_vm can never satisfy gpu required no workspace clone begins AC4 first fail-closed layer" }
  isolation_kind: { kind: decision, label: "sandbox pick spec isolation branch" }
  legacy_pick: { kind: process, label: "Isolation None or Seatbelt unchanged fail-closed egress handling per issue 1300" }
  legacy_effect: { kind: terminal, label: "process or seatbelt backend runs per existing 1300 fail-closed contract unaffected by this WI" }
  micro_gpu: { kind: decision, label: "gpu request required checked again inside pick second independent layer" }
  err_gpu_pick: { kind: terminal, label: "pick returns hard Err gpu categorically unreachable in a microvm second fail-closed layer alongside the run.rs preflight" }
  image_check: { kind: decision, label: "spec microvm_image is set" }
  err_no_image: { kind: terminal, label: "pick returns hard Err isolation micro_vm requires an OCI base image via microvm-image vat never guesses one" }
  avail_check: { kind: decision, label: "microvm available container CLI on PATH" }
  err_unavailable: { kind: terminal, label: "pick returns hard Err container CLI not installed install it and rerun vat doctor" }
  egress_kind: { kind: decision, label: "spec egress policy" }
  err_localhost: { kind: terminal, label: "pick returns hard Err localhost-only not enforceable guest 127.0.0.1 never reaches the host only a per-network container VM gateway IP is reachable and ordinary applications do not know to target it confirmed by the Phase 0 spike 1472" }
  construct: { kind: process, label: "construct MicroVmBackend from egress env workdir and image" }
  effect: { kind: terminal, label: "container run resolved argv exec'd rootfs bind-mounted workdir set egress enforced via network none or left open" }
edges:
  - { from: start, to: preflight }
  - { from: preflight, to: preflight_err, label: "gpu required and isolation micro_vm" }
  - { from: preflight, to: isolation_kind, label: "satisfied" }
  - { from: isolation_kind, to: legacy_pick, label: "none or seatbelt" }
  - { from: legacy_pick, to: legacy_effect }
  - { from: isolation_kind, to: micro_gpu, label: "micro_vm" }
  - { from: micro_gpu, to: err_gpu_pick, label: "required" }
  - { from: micro_gpu, to: image_check, label: "auto or none" }
  - { from: image_check, to: err_no_image, label: "none" }
  - { from: image_check, to: avail_check, label: "some image" }
  - { from: avail_check, to: err_unavailable, label: "container not on PATH" }
  - { from: avail_check, to: egress_kind, label: "available" }
  - { from: egress_kind, to: construct, label: "open or deny" }
  - { from: egress_kind, to: err_localhost, label: "localhost-only" }
  - { from: construct, to: effect }
---
```
