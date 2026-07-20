---
id: '2214'
summary: Close Defer long-running stability with committed lifecycle and repeated Raft failover behavior, bounded durable scheduling efficiency, fixed-keyspace retry soak plateaus, and strict operator/PVC pod-replacement recovery.
fill_sections: [logic, changes, e2e-test]
capability_refs:
  - id: long-running-stability
    role: primary
    gap: delayed-task-soak-and-recovery
    claim: delayed-task-soak-and-recovery
    coverage: full
    rationale: "Defines the fail-closed behavior, efficiency, process-soak, and Kubernetes recovery oracles for Defer's delayed-task stability contract."
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: defer-delayed-task-stability-contract
entry: invoke
nodes:
  invoke: { kind: start, label: "exercise the Defer stability contract through committed scheduler public process and operator surfaces" }
  behavior: { kind: process, label: "run lifecycle queue policy and real three-node Raft recovery suites" }
  behavior_ok: { kind: decision, label: "fourteen tests preserve ordering rate control fencing and repeated failover state?" }
  efficiency: { kind: process, label: "measure one thousand durable enqueue lease ack operations against the Relay control" }
  efficiency_ok: { kind: decision, label: "zero errors complete metrics and defer to relay ratio at least 0.80?" }
  soak: { kind: process, label: "cross proposal cache and snapshot cadence then measure two fixed-keyspace retry windows" }
  soak_ok: { kind: decision, label: "non-zero operations retry progress and RSS FD task and p99 bounds all hold?" }
  kind: { kind: process, label: "reconcile operator StatefulSet and exact Bound 1Gi PVC then replace the serving pod" }
  kind_ok: { kind: decision, label: "different pod UID recovers tasks accepts mutations and cluster cleanup is verified?" }
  fail: { kind: terminal, label: "fail closed on skipped work stale fencing missing metrics resource drift PVC mismatch lost state or residual cluster" }
  verified: { kind: terminal, label: "behavior efficiency process stability and operator recovery are independently observed" }
  shared: { kind: terminal, label: "shared Raft observability executor and Kubernetes mechanisms remain library owned" }
edges:
  - { from: invoke, to: behavior }
  - { from: behavior, to: behavior_ok }
  - { from: behavior_ok, to: efficiency, label: "yes" }
  - { from: behavior_ok, to: fail, label: "no" }
  - { from: efficiency, to: efficiency_ok }
  - { from: efficiency_ok, to: soak, label: "yes" }
  - { from: efficiency_ok, to: fail, label: "no" }
  - { from: soak, to: soak_ok }
  - { from: soak_ok, to: kind, label: "yes" }
  - { from: soak_ok, to: fail, label: "no" }
  - { from: kind, to: kind_ok }
  - { from: kind_ok, to: verified, label: "yes" }
  - { from: kind_ok, to: fail, label: "no" }
  - { from: invoke, to: shared, label: "ownership boundary" }
---
flowchart TD
    invoke([exercise committed Defer surfaces]) --> behavior[run lifecycle rate and three-node Raft suites]
    behavior --> behavior_ok{fourteen behavior tests pass?}
    behavior_ok -->|yes| efficiency[measure one thousand durable lifecycle operations]
    behavior_ok -->|no| fail([fail closed])
    efficiency --> efficiency_ok{ratio at least 0.80 and metrics complete?}
    efficiency_ok -->|yes| soak[run fixed-keyspace retry soak]
    efficiency_ok -->|no| fail
    soak --> soak_ok{progress and resource bounds hold?}
    soak_ok -->|yes| kind[replace operator pod over exact Bound 1Gi PVC]
    soak_ok -->|no| fail
    kind --> kind_ok{state mutation and cleanup verified?}
    kind_ok -->|yes| verified([stability contract observed])
    kind_ok -->|no| fail
    invoke -->|ownership boundary| shared([shared mechanisms remain in libs])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/defer/scripts/soak.sh
    action: modify
    section: e2e-test
    impl_mode: hand-written
    reason: "Make the existing fixed-keyspace retry soak fail closed when the measured operation count is zero while retaining the shared service-observability resource and latency bounds."
  - path: apps/defer/scripts/kind-e2e.sh
    action: modify
    section: e2e-test
    impl_mode: hand-written
    reason: "Require an observed Bound PVC with exact 1Gi request and capacity, and require successful cluster deletion plus absence verification on a successful operator recovery journey."
```
