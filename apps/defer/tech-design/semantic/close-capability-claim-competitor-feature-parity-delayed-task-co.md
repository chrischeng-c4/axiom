---
id: '2216'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: defer-cloud-tasks-shaped-contract-verification
entry: parse_matrix
nodes:
  parse_matrix: { kind: start, label: "parse exact managed HTTP push comparison contract" }
  scope_ok: { kind: decision, label: "competitor category exclusions and performance boundary exact?" }
  exercise_lifecycle: { kind: process, label: "run ETA priority retry cancel inspect and terminal-state journey" }
  exercise_limits: { kind: process, label: "drive committed rate burst and in-flight limits through different replicas" }
  failover_ok: { kind: decision, label: "aggregate limits and DeadLettered survive failover and restart?" }
  exercise_target: { kind: process, label: "deliver exact method headers body idempotency and signed retries to real HTTP target" }
  target_ok: { kind: decision, label: "target effects and independent signing negatives exact?" }
  exercise_public_api: { kind: process, label: "create cancel and inspect through authenticated public h2c API" }
  ec_ok: { kind: decision, label: "accepted EC wrappers map every promised row to executing evidence?" }
  fail: { kind: terminal, label: "competitor feature contract fails" }
  verified: { kind: terminal, label: "scoped Defer competitor parity externally verified" }
edges:
  - { from: parse_matrix, to: scope_ok }
  - { from: scope_ok, to: exercise_lifecycle, label: "yes" }
  - { from: scope_ok, to: fail, label: "no" }
  - { from: exercise_lifecycle, to: exercise_limits }
  - { from: exercise_limits, to: failover_ok }
  - { from: failover_ok, to: exercise_target, label: "yes" }
  - { from: failover_ok, to: fail, label: "no" }
  - { from: exercise_target, to: target_ok }
  - { from: target_ok, to: exercise_public_api, label: "yes" }
  - { from: target_ok, to: fail, label: "no" }
  - { from: exercise_public_api, to: ec_ok }
  - { from: ec_ok, to: verified, label: "yes" }
  - { from: ec_ok, to: fail, label: "no" }
---
flowchart TD
    parse_matrix([parse exact comparison contract]) --> scope_ok{scope and boundaries exact?}
    scope_ok -->|yes| exercise_lifecycle[exercise delayed-task lifecycle]
    scope_ok -->|no| fail([contract fails])
    exercise_lifecycle --> exercise_limits[exercise cross-replica aggregate limits]
    exercise_limits --> failover_ok{limits and terminal state durable?}
    failover_ok -->|yes| exercise_target[exercise real HTTP effects and signatures]
    failover_ok -->|no| fail
    exercise_target --> target_ok{target behavior exact?}
    target_ok -->|yes| exercise_public_api[exercise authenticated public API]
    target_ok -->|no| fail
    exercise_public_api --> ec_ok{all accepted EC wrappers execute?}
    ec_ok -->|yes| verified([scoped competitor parity verified])
    ec_ok -->|no| fail
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/defer/tests/competitor_feature_matrix.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: cloud_tasks_shaped_contract_and_exclusions_are_explicit
    reason: "Own the parsed, duplicate-free 15-row Google Cloud Tasks-shaped comparison, explicit category exclusions, accurate retry-exhaustion semantics, and bounded performance-claim boundary."
  - path: apps/defer/tests/raft_scheduler.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: committed_queue_limits_survive_cross_replica_proposals_and_failover
    reason: "Own real three-node proofs that rate, burst, and in-flight limits are one committed aggregate and that DeadLettered converges and survives same-directory restart."
  - path: apps/defer/tests/http_dispatch.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: dispatches_real_http_and_retries_with_stable_task_idempotency
    reason: "Own the real target oracle for exact per-task method, header, body, retry identity, stable idempotency, and terminal success."
  - path: apps/defer/tests/http_api.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: h2c_routes_probes_openapi_metrics_dispatch_and_auth_are_live
    reason: "Own the authenticated public h2c create, cancel, and terminal inspection journey while preserving tenant isolation."
```
