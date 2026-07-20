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

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: defer-delayed-task-state-recovery
    name: committed lifecycle and repeated Raft failover recovery
    command: "cargo test -p defer --test task_lifecycle --test rate_limits --test raft_scheduler -- --nocapture"
    assertions:
      - "Exactly fourteen lifecycle, rate-control, and three-node Raft tests execute with zero failures."
      - "Leader loss preserves the live lease, stale settlement fails, reassignment advances the fence, a restarted node converges, and a second leader loss still completes a new task."
  - id: defer-delayed-task-scheduler-efficiency
    name: durable scheduler overhead ceiling against Relay
    command: "cargo test --release -p defer --test relay_performance_ceiling -- --ignored --nocapture"
    assertions:
      - "Exactly 1,000 durable enqueue-lease-ack operations complete with errors = 0 and a complete numeric CPU, RSS, disk, amplification, throughput, and p50/p95/p99 report."
      - "The same-host Defer throughput is at least 80% of the identically shaped Relay control workload."
  - id: defer-delayed-task-live-soak
    name: fixed-keyspace retry progress and resource plateau
    command: "DEFER_SOAK_AUTOSTART=1 bash apps/defer/scripts/soak.sh"
    assertions:
      - "The fixed-keyspace warmup crosses the 1,024-entry proposal-cache and snapshot cadence before two 30-second measured windows."
      - "Measured operations are non-zero, errors are zero, retry counters advance in both windows, RSS drift is <= 10%, FD growth <= 8, thread/task growth <= 4, and task-read p99 is <= 250 ms with <= 100% growth."
  - id: defer-delayed-task-kind-pvc-recovery
    name: operator PVC pod replacement recovery and cleanup
    command: "bash apps/defer/scripts/kind-e2e.sh"
    assertions:
      - "The source image and real CRD/operator reconcile a StatefulSet whose PVC is observed Bound with exact 1Gi request and capacity."
      - "A different replacement pod UID recovers the two committed task records, accepts queue pause and task cancellation, and the successful journey verifies that the disposable cluster is absent after cleanup."
  - id: defer-delayed-task-ec-inventory
    name: accepted long-running stability EC inventory
    command: "aw ec check --project defer"
    assertions:
      - "The accepted behavior, efficiency, live-soak, and Kind/PVC cases remain generated, claim-bound, and structurally clean."
```
