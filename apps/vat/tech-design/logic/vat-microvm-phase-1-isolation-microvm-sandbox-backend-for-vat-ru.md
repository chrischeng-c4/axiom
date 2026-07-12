---
id: vat-microvm-phase-1-isolation-microvm-sandbox-backend-for-vat-ru
summary: Add an additive `Isolation::MicroVm` sandbox backend to `vat run`, built on Apple's `container` CLI, that runs a workload inside an ephemeral microVM and fails closed whenever it cannot satisfy the requested GPU or egress policy. Phase 1 of the microVM epic (#1471); Phase 0 (#1472) already verified the `container` CLI mechanics with a "go" verdict.
fill_sections: [logic, schema, config, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: microvm-sandbox-backend-for-vat-run
    claim: microvm-sandbox-backend-for-vat-run
    coverage: full
    rationale: "Adds the Isolation::MicroVm variant, sandbox/microvm.rs (MicroVmBackend), the fail-closed pick() branch, the run.rs GPU-preflight bug fix (dual fail-closed layer per #1300 precedent), capabilities.rs/doctor.rs probing, and the --microvm-image CLI flag — the complete Phase 1 design for the new MicroVm sandbox backend work root."
---

# vat MicroVm Phase 1: Isolation::MicroVm sandbox backend for vat run

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

## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-microvm-phase1.schema.json"
title: "MicroVm sandbox backend Phase 1: data model additions"
type: object
properties:
  isolation_variant:
    type: object
    description: "New unit variant on the existing Isolation enum in spec.rs (currently None | Seatbelt, derives clap::ValueEnum). Additive only \u2014 no existing variant renamed or removed."
    properties:
      enum: { type: string, const: "Isolation" }
      new_variant: { type: string, const: "MicroVm" }
      cli_token: { type: string, const: "micro_vm", description: "clap::ValueEnum rename maps MicroVm to the --isolation micro_vm token, consistent with existing snake_case tokens." }
  env_spec_field:
    type: object
    description: "New optional field on the existing EnvSpec struct in spec.rs, alongside base/workdir/env/setup/isolation/egress/gpu/limits."
    properties:
      name: { type: string, const: "microvm_image" }
      rust_type: { type: string, const: "Option<String>" }
      default: { type: string, const: "None", description: "No default image is ever guessed; None + Isolation::MicroVm is a hard pick() rejection (R3)." }
      serde: { type: string, description: "skip_serializing_if = Option::is_none, consistent with EnvSpec's existing optional-field convention." }
  microvm_backend_struct:
    type: object
    description: "New apps/vat/src/sandbox/microvm.rs struct implementing the existing Sandbox trait (same trait seatbelt::SeatbeltBackend and process::ProcessBackend already implement)."
    properties:
      name: { type: string, const: "MicroVmBackend" }
      fields:
        type: array
        items: { type: string }
        description: "egress: EgressPolicy, env: BTreeMap<String,String>, workdir: Option<String>, image: String \u2014 BTreeMap (not HashMap) so resolve()'s -e ordering is deterministic (AC2)."
      resolve_contract:
        type: string
        description: "resolve(rootfs, cmd) -> (\"container\", argv: Vec<String>) shaped: run --rm -v <rootfs>:/workspace -w /workspace/<workdir> -e K=V... [--network none if egress==Deny, omitted if Open] <image> <program> <args...>."
      available_fn:
        type: string
        const: "pub fn available() -> bool"
        description: "Checks the `container` binary is resolvable on PATH (mirrors seatbelt::available()'s sandbox-exec check); no real container invocation, no image pull."
additionalProperties: true
```

## Config
<!-- type: config lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: "vat-microvm-phase1-config.schema.json"
title: "MicroVm sandbox backend Phase 1: config surface"
type: object
properties:
  note:
    type: string
    description: "No new vat.toml key. Isolation::MicroVm is selected the same way Isolation::Seatbelt already is \u2014 via the --isolation flag (Isolation already derives clap::ValueEnum) \u2014 plus the one new --microvm-image flag documented in the CLI section. [network].egress in vat.toml is reused unchanged: EgressPolicy::Open/Deny are enforceable under MicroVm, EgressPolicy::LocalhostOnly is a hard pick() rejection (R3), never a silent downgrade."
  microvm_image_source:
    type: string
    description: "spec.microvm_image is set exclusively from the new --microvm-image CLI flag (R7); there is no vat.toml [runner]/[service] equivalent in Phase 1 \u2014 out of scope per the WI (vat build/vat compose config surface is Phase 2/Phase 3)."
additionalProperties: true
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat run
    behavior:
      - "New flag `--microvm-image <ref>` on Cmd::Run, threaded into the three `EnvSpec { ... }` construction sites in run.rs (~163/334/518) as `microvm_image` (R7). `--isolation micro_vm` is already accepted (Isolation derives clap::ValueEnum); this flag is the only new one."
      - "`sandbox::pick(spec)` gains a fail-closed `Isolation::MicroVm` branch (R3): `GpuRequest::Required` is rejected outright; a missing `microvm_image` is rejected (vat never guesses a base image); `microvm::available()` (container CLI on PATH) is required; `EgressPolicy::Open`/`Deny` are enforced, `EgressPolicy::LocalhostOnly` is rejected with the Phase-0-confirmed gateway-IP reasoning, not a generic 'no bridge' message."
      - "GPU preflight bug fix (R4): the three existing `GpuRequest::Required` checks in run.rs (~163/334/518) now call a shared `gpu_satisfied(gpu, isolation, info)` helper that also factors in isolation mode, so `--isolation micro_vm --gpu required` is rejected before any workspace clone begins \u2014 a second, independent fail-closed layer alongside `pick()`'s own rejection (dual-defense pattern per #1300 precedent)."
      - "`--isolation none`/`--isolation seatbelt` behavior is unchanged (out of scope)."
  - name: vat capabilities --json
    behavior:
      - "The `vm` row (R5) reports real probing instead of a hardcoded stub: `implemented: true`, `available: sandbox::microvm::available()`, `gpu_native: false` (GPU passthrough is categorically impossible in an Apple Silicon microVM), `network_egress` mirrors `available` (Open/Deny enforceable, LocalhostOnly not), with a reason string naming the LocalhostOnly gap explicitly."
  - name: vat doctor
    behavior:
      - "`check_network_isolation()` adds `\"vm\"` to its existing OR condition (R6), so a MicroVm-isolated run is recognized as network-isolation-capable on hosts where the `container` CLI is available, consistent with how `seatbelt` is already recognized."
```
