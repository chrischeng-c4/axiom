# Defer

## Brief

Defer is the Cloud Tasks-like delayed push-queue dispatch service in the Axiom
stack.

It owns scheduled execution, HTTP target dispatch, per-queue controls, rate
limits, leases/acks, retries, dead-letter queues, and dedupe keys. It is
intentionally separate from `relay`: Relay is the pull queue for worker-leased
delivery; Defer is the push queue for scheduled HTTP dispatch, where every item
has a target, schedule, attempt policy, and terminal outcome.

Task priority uses the same field shape as relay: an unsigned byte where `0` is
lowest, `255` is highest, and omitted priority defaults to `10`. Defer first
filters tasks by schedule/ETA and queue rate/concurrency limits; priority orders
only the due, dispatch-eligible candidates, with same-priority tasks remaining
FIFO by task creation order.

## Boundaries

- `relay` owns pull-queue broker delivery and opaque worker leases.
- `defer` owns scheduled task lifecycle, target dispatch, retry policy, DLQ, and
  rate/concurrency-limited push dispatch.
- `loom` may use Defer for timers/callbacks, but workflow state remains in Loom.
- `keep` may store large request/response bodies by reference.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Delayed Task Lifecycle | #766 | implemented | passing | conformance | not_ready | in-memory core: create, schedule, priority, lease, ack, cancel, and inspect tasks |
| HTTP Dispatch And Retries | #766 | partial | passing | smoke | not_ready | retry/DLQ core implemented; HTTP target delivery pending |
| Queue Rate Limits | #766 | implemented | passing | conformance | not_ready | per-queue pause/resume/disable, max-in-flight, dispatch budget, and token-bucket dispatch rate implemented in core |
| HTTP/2 API List | #766 | planned | planned | none | not_ready | h2c/OpenAPI endpoint inventory |
| Kubernetes-Native Deployment | #766 | planned | planned | none | not_ready | dedicated StatefulSet/operator shape |
| Primary Replicas | #766 | planned | planned | none | not_ready | raft-backed task state and timers |
| CLI Interface | #766 | planned | planned | none | not_ready | `defer` CLI for queue/task/admin and agent docs |
| Long-Running Stability | #766 | partial | passing | smoke | not_ready | in-memory timer, retry, lease expiry, and DLQ gates |
| Security Hardening | #766 | planned | planned | none | not_ready | target signing, authz, tenant isolation, audit, and secret rotation |
| Competitor Feature Parity | #766 | planned | planned | none | not_ready | Cloud Tasks/Celery/Sidekiq-style delayed task feature matrix |
| Competitor Performance | #766 | planned | planned | none | not_ready | pinned schedule/dispatch baseline, rerun only on scope change |

### CLI Interface

ID: cli-interface
Type: RuntimeTool
Root WI: #766
Status: confirmed
Surfaces: CLI: `defer llm`, `defer upgrade`, `defer issue`, queue/task create/status/cancel, and admin/debug verbs.
EC Dimensions: behavior: pending CLI convention gate - required standard verbs, task lifecycle ergonomics, and offline agent docs
Required Verification: smoke, conformance
Promise:
Defer ships an agent-drivable CLI for delayed task lifecycle, queue control,
and admin workflows while following the repository-wide CLI convention.
Gate Inventory:
- pending: projects/defer/tests/cli_contract.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| defer-cli-convention-and-task-verbs | epic | #766 | planned | planned | none | pending CLI convention gate |

### Long-Running Stability

ID: long-running-stability
Type: Runtime
Root WI: #766
Status: confirmed
Surfaces: Runtime: timer wheel, priority-ready set, queue control state, rate bucket, lease store, retry scheduler, DLQ writer, snapshot, and recovery paths.
EC Dimensions: stability: `cargo test -p defer --test task_lifecycle --test rate_limits` - in-memory timer, retry, lease expiry, duplicate prevention, per-queue control, and dispatch backpressure smoke
Required Verification: conformance, dogfood
Promise:
Defer remains stable under sustained scheduled-task load, retries, DLQ writes,
and restart cycles without losing committed tasks or duplicating terminal acks.
Gate Inventory:
- projects/defer/tests/task_lifecycle.rs; projects/defer/tests/rate_limits.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| delayed-task-soak-and-recovery | epic | #766 | partial | passing | smoke | projects/defer/tests/task_lifecycle.rs; projects/defer/tests/rate_limits.rs |

### Security Hardening

ID: security-hardening
Type: Devops
Root WI: #766
Status: confirmed
Surfaces: HTTP/K8s: queue/task authn/authz, signed target dispatch, tenant isolation, network policy, audit events, secret rotation, and request limits.
EC Dimensions: behavior: pending security gate - auth failure cases, target signature validation, tenant isolation, audit emission, secret rotation, and abuse limits
Required Verification: negative, conformance
Promise:
Defer protects task control and target dispatch with explicit authorization,
signed delivery, auditability, network policy, and managed secret rotation.
Gate Inventory:
- pending: projects/defer/tests/security_hardening.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| delayed-task-security-boundary | epic | #766 | planned | planned | none | pending security hardening gate |

### Competitor Feature Parity

ID: competitor-feature-parity
Type: RuntimeTool
Root WI: #766
Status: confirmed
Surfaces: Docs/Test: delayed task feature matrix against Cloud Tasks, Celery, and Sidekiq-style services.
EC Dimensions: behavior: pending competitor feature gate - schedule, lease, ack, cancel, retry, DLQ, rate limits, dedupe, and target signing
Required Verification: conformance
Promise:
Defer keeps an explicit delayed-task feature matrix against established task
dispatch systems, with comparison scope changed only when product requirements change.
Gate Inventory:
- pending: projects/defer/benchmark/competitor-feature-matrix.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| delayed-task-competitor-feature-matrix | epic | #766 | planned | planned | none | pending competitor feature gate |

### Competitor Performance

ID: competitor-performance
Type: RuntimeTool
Root WI: #766
Status: confirmed
Surfaces: Meter/Vat: schedule throughput, dispatch latency, retry/DLQ overhead, and queue rate-limit behavior.
EC Dimensions: efficiency: pending competitor performance gate - pinned external baseline and Defer-owned dispatch measurements
Required Verification: dogfood
Promise:
Defer maintains a pinned competitor performance baseline and reruns external
benchmarks only when the comparison scope changes or a human explicitly asks.
Gate Inventory:
- pending: projects/defer/benchmark/competitor-performance-baseline.md
- pending: projects/defer/meter-defer-dispatch.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| delayed-task-competitor-performance-baseline | epic | #766 | planned | planned | none | pending competitor performance gate |

### Delayed Task Lifecycle

ID: delayed-task-lifecycle
Type: RuntimeTool
Root WI: #766
Status: confirmed
Surfaces: Rust API: `DeferScheduler` - create, schedule, priority, lease, ack, cancel, and inspect delayed tasks.; HTTP: `/v1/queues/{queue}/tasks` - future transport wrapper.
EC Dimensions: behavior: `cargo test -p defer --test task_lifecycle` - schedule ordering, priority ordering among due tasks, cancellation, leases, and terminal states
Required Verification: smoke, conformance
Promise:
Defer manages delayed task state from creation through terminal success,
failure, cancellation, or dead-letter handoff. Task priority is `u8`
(`0..=255`, default `10`); it is applied only after the task is due and the
queue can dispatch more work.
Gate Inventory:
- projects/defer/tests/task_lifecycle.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| delayed-task-state-machine | epic | #766 | implemented | passing | conformance | projects/defer/tests/task_lifecycle.rs |
| due-task-priority-ordering | epic | #766 | implemented | passing | conformance | projects/defer/tests/task_lifecycle.rs |

### HTTP Dispatch And Retries

ID: http-dispatch-and-retries
Type: RuntimeTool
Root WI: #766
Status: confirmed
Surfaces: Rust API: `DeferScheduler::lease_due` / `nack` - dispatch attempt, retry policy, and DLQ transitions.; HTTP worker target dispatch - future signed target call wrapper.
EC Dimensions: behavior: `cargo test -p defer --test task_lifecycle` - retry backoff, idempotency key, and DLQ behavior; pending HTTP target call gate
Required Verification: smoke, conformance, negative
Promise:
Defer dispatches tasks to HTTP targets with bounded retries, dedupe keys, and
explicit dead-letter behavior.
Gate Inventory:
- projects/defer/tests/task_lifecycle.rs; pending: projects/defer/tests/http_dispatch.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| http-target-attempt-contract | epic | #766 | partial | passing | smoke | projects/defer/tests/task_lifecycle.rs; pending HTTP target dispatch gate |

### Queue Rate Limits

ID: queue-rate-limits
Type: RuntimeTool
Root WI: #766
Status: confirmed
Surfaces: Rust API: `QueuePolicy` and queue control methods - max dispatch per tick, max dispatches per second, max burst size, max in-flight, pause/resume/disable, queue snapshot, priority among due tasks, and backoff.; HTTP/Admin: future queue config transport wrapper.
EC Dimensions: behavior: `cargo test -p defer --test rate_limits --test task_lifecycle` - per-queue dispatch budget/concurrency/rate enforcement, pause/resume/disable controls, policy update isolation, and priority-preserving dispatch among eligible tasks
Required Verification: smoke, conformance
Promise:
Defer enforces Cloud Tasks-style per-queue controls, rate limits, and
concurrency limits before dispatching tasks to external targets.
Gate Inventory:
- projects/defer/tests/rate_limits.rs; projects/defer/tests/task_lifecycle.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| per-queue-rate-limit-contract | epic | #766 | implemented | passing | conformance | projects/defer/tests/rate_limits.rs; projects/defer/tests/task_lifecycle.rs |

### HTTP/2 API List

ID: http2-api-list
Type: RuntimeTool
Root WI: #766
Status: confirmed
Surfaces: HTTP: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs`, queue/task/admin routes.
EC Dimensions: behavior: pending h2c/OpenAPI route-list gate - probes, metrics, OpenAPI, and route inventory
Required Verification: smoke, conformance
Promise:
Defer exposes a compact h2c/OpenAPI API list for queue/task lifecycle and
operator workflows.
Gate Inventory:
- pending: projects/defer/tests/http_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| h2c-openapi-route-list | epic | #766 | planned | planned | none | pending h2c/OpenAPI route-list gate |

### Kubernetes-Native Deployment

ID: kubernetes-native-deployment
Type: Devops
Root WI: #766
Status: confirmed
Surfaces: K8s: dedicated StatefulSet/operator topology for queues, timers, storage, probes, backups, and PDBs.
EC Dimensions: behavior: pending kustomize/operator render gate - CRD, operator, and instance render; stability: pending kind dispatch dogfood
Required Verification: smoke, dogfood
Promise:
Defer runs as a dedicated k8s-native task dispatch service with operator-managed
queues, rate limits, storage, backup policy, and lifecycle.
Gate Inventory:
- pending: projects/defer/k8s

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| dedicated-task-service-topology | epic | #766 | planned | planned | none | pending k8s render/dogfood gates |

### Primary Replicas

ID: primary-replicas
Type: Runtime
Root WI: #766
Status: confirmed
Surfaces: Raft: delayed task state machine over `libs/raft-core` and `libs/raft-host`.
EC Dimensions: stability: pending raft scheduler failover gate - no committed task loss or duplicate terminal ack
Required Verification: conformance, dogfood
Promise:
Defer replicates task state and timer ownership through raft so failover
preserves scheduled tasks and terminal outcomes.
Gate Inventory:
- pending: projects/defer/tests/raft_scheduler.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| raft-backed-task-scheduler | epic | #766 | planned | planned | none | pending raft scheduler failover gate |
