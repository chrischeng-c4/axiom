---
id: '1573'
summary: Reconcile quota-admitted Deployment scale and drain transitions while exposing endpoint and Pod status metrics.
fill_sections: [logic, unit-test]
capability_refs:
  - id: kubernetes-native-deployment
    role: primary
    gap: drain-safe-control-plane-status
    claim: drain-safe-control-plane-status
    coverage: full
    rationale: "Connects the global endpoint invariant to readiness, drain-before-release, rollout peak, status, and metrics."
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-drain-safe-control-plane-status
entry: scale
nodes:
  scale: { kind: start, label: "Desired and rollout-surge Pod set" }
  reserve: { kind: process, label: "Atomically reserve all quotas before readiness" }
  ready: { kind: process, label: "Mark admitted Pod ready" }
  remove_ready: { kind: process, label: "Remove readiness before replacement or scale-in" }
  drain: { kind: process, label: "Request drain and retain quota" }
  complete: { kind: decision, label: "Active sessions zero or drain deadline reached" }
  hold: { kind: process, label: "Keep draining allocation held" }
  release: { kind: process, label: "Release allocation" }
  observe: { kind: terminal, label: "Publish status and Prometheus budget facts" }
edges:
  - { from: scale, to: reserve }
  - { from: reserve, to: ready }
  - { from: ready, to: remove_ready }
  - { from: remove_ready, to: drain }
  - { from: drain, to: complete }
  - { from: complete, to: hold, label: "no" }
  - { from: hold, to: complete }
  - { from: complete, to: release, label: "yes" }
  - { from: release, to: observe }
---
flowchart TD
  scale([desired plus surge Pods]) --> reserve[atomic quota reserve]
  reserve --> ready[mark admitted Pod ready]
  ready --> remove_ready[remove readiness]
  remove_ready --> drain[request drain and hold quota]
  drain --> complete{active zero or deadline?}
  complete -->|no| hold[retain draining quota]
  hold --> complete
  complete -->|yes| release[release allocation]
  release --> observe([status and Prometheus])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-drain-safe-control-plane-status-tests
requirements:
  scale_admission:
    id: R1
    text: "Desired and surge Pods reserve quota atomically before readiness and expose blocked state on failure."
    kind: functional
    risk: high
    verify: cargo test -p pgpool k8s::control
  drain_release:
    id: R2
    text: "Readiness is removed before drain and quota releases only at zero active sessions or deadline."
    kind: functional
    risk: high
    verify: cargo test -p pgpool k8s::control
  observability:
    id: R3
    text: "Status and metrics expose capacity inputs, allocation, backend activity, blocked reason, and drain state."
    kind: regression
    risk: high
    verify: cargo test -p pgpool k8s::control
  runtime_quota:
    id: R4
    text: "The rendered per-Pod backend quota is consumed by pgpool serve through PGPOOL_MAX_BACKEND_CONNECTIONS."
    kind: integration
    risk: high
    verify: cargo test -p pgpool --test cli_contract
---
flowchart TD
  r1[R1 scale admission] --> control[cargo test -p pgpool k8s::control]
  r2[R2 drain release] --> control
  r3[R3 observability] --> control
  r4[R4 runtime quota] --> cli[cargo test -p pgpool --test cli_contract]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/k8s/mod.rs
    action: modify
    impl_mode: hand-written
    section: logic
    description: Export control-plane reconciliation and status models.
  - path: apps/pgpool/src/k8s/budget.rs
    action: modify
    impl_mode: hand-written
    section: logic
    description: Expose endpoint iteration for status projection.
  - path: apps/pgpool/src/k8s/control.rs
    action: create
    impl_mode: hand-written
    section: logic
    description: Implement scale admission, drain-before-release, status, and metrics.
  - path: apps/pgpool/src/bin/pgpool.rs
    action: modify
    impl_mode: hand-written
    section: logic
    description: Consume the admitted per-Pod backend quota from environment or CLI.
  - path: apps/pgpool/tests/cli_contract.rs
    action: modify
    impl_mode: hand-written
    section: unit-test
    description: Verify the runtime quota flag is present and parseable.
```
