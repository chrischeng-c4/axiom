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
id: defer-delayed-task-stability-verification
entry: observe_behavior
nodes:
  observe_behavior: { kind: start, label: "execute fourteen lifecycle rate-control and real three-node Raft tests" }
  behavior_ok: { kind: decision, label: "ordering control fencing durable restart convergence and repeated failover all observed?" }
  measure_efficiency: { kind: process, label: "execute one thousand durable enqueue lease ack operations and the identical Relay control" }
  efficiency_ok: { kind: decision, label: "metrics complete errors zero and defer throughput at least eighty percent of Relay?" }
  warm_soak: { kind: process, label: "warm fixed task keys through proposal cache and snapshot cadence" }
  measure_soak: { kind: process, label: "measure two thirty-second retry windows and process resource samples" }
  soak_ok: { kind: decision, label: "operations non-zero retry advances and RSS FD tasks and p99 remain bounded?" }
  deploy_kind: { kind: process, label: "build source image reconcile CRD operator StatefulSet and exact Bound 1Gi PVC" }
  replace_pod: { kind: process, label: "delete serving pod require a different ready UID and read the committed queue and tasks" }
  mutate_after_recovery: { kind: process, label: "pause the queue cancel a task and observe terminal accounting" }
  cleanup_ok: { kind: decision, label: "cluster deletion succeeds and the cluster name is absent?" }
  fail: { kind: terminal, label: "contract fails closed" }
  verified: { kind: terminal, label: "delayed-task stability claim is externally verified" }
edges:
  - { from: observe_behavior, to: behavior_ok }
  - { from: behavior_ok, to: measure_efficiency, label: "yes" }
  - { from: behavior_ok, to: fail, label: "no" }
  - { from: measure_efficiency, to: efficiency_ok }
  - { from: efficiency_ok, to: warm_soak, label: "yes" }
  - { from: efficiency_ok, to: fail, label: "no" }
  - { from: warm_soak, to: measure_soak }
  - { from: measure_soak, to: soak_ok }
  - { from: soak_ok, to: deploy_kind, label: "yes" }
  - { from: soak_ok, to: fail, label: "no" }
  - { from: deploy_kind, to: replace_pod }
  - { from: replace_pod, to: mutate_after_recovery }
  - { from: mutate_after_recovery, to: cleanup_ok }
  - { from: cleanup_ok, to: verified, label: "yes" }
  - { from: cleanup_ok, to: fail, label: "no" }
---
flowchart TD
    observe_behavior([run lifecycle rate and Raft suites]) --> behavior_ok{behavior and recovery observed?}
    behavior_ok -->|yes| measure_efficiency[measure Defer and Relay durable lifecycle]
    behavior_ok -->|no| fail([fail closed])
    measure_efficiency --> efficiency_ok{complete metrics errors zero ratio >= 0.80?}
    efficiency_ok -->|yes| warm_soak[warm fixed keys through snapshot cadence]
    efficiency_ok -->|no| fail
    warm_soak --> measure_soak[measure two retry and resource windows]
    measure_soak --> soak_ok{non-zero progress and bounds hold?}
    soak_ok -->|yes| deploy_kind[reconcile source image and exact Bound 1Gi PVC]
    soak_ok -->|no| fail
    deploy_kind --> replace_pod[replace serving pod and recover records]
    replace_pod --> mutate_after_recovery[pause queue and cancel task]
    mutate_after_recovery --> cleanup_ok{cluster deletion and absence verified?}
    cleanup_ok -->|yes| verified([stability claim externally verified])
    cleanup_ok -->|no| fail
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/defer/scripts/soak.sh
    action: modify
    section: e2e-test
    impl_mode: hand-written
    reason: "Add a fail-closed non-zero measured-operation assertion to the existing fixed-keyspace retry soak while retaining shared process/resource sampling."
  - path: apps/defer/scripts/kind-e2e.sh
    action: modify
    section: e2e-test
    impl_mode: hand-written
    reason: "Wait for and assert a Bound PVC with exact 1Gi request and capacity, then make successful cluster deletion and explicit absence verification part of the operator recovery result."
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
