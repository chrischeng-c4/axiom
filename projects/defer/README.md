# Defer

## Brief

Defer is the Cloud Tasks-like delayed task dispatch service in the Axiom stack.

It owns scheduled execution, HTTP target dispatch, per-queue rate limits,
leases/acks, retries, dead-letter queues, and dedupe keys. It is intentionally
separate from `relay`: Relay is a broker for message delivery and worker queues;
Defer is a task execution control plane where every item has a target, schedule,
attempt policy, and terminal outcome.

## Boundaries

- `relay` owns online broker delivery and opaque queue leases.
- `defer` owns scheduled task lifecycle, target dispatch, retry policy, and DLQ.
- `loom` may use Defer for timers/callbacks, but workflow state remains in Loom.
- `keep` may store large request/response bodies by reference.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Delayed Task Lifecycle | #766 | planned | planned | none | not_ready | create, schedule, lease, ack, cancel, and inspect tasks |
| HTTP Dispatch And Retries | #766 | planned | planned | none | not_ready | HTTP target delivery with retry/DLQ policy |
| Queue Rate Limits | #766 | planned | planned | none | not_ready | per-queue concurrency, QPS, and backoff enforcement |
| HTTP/2 API List | #766 | planned | planned | none | not_ready | h2c/OpenAPI endpoint inventory |
| Kubernetes-Native Deployment | #766 | planned | planned | none | not_ready | dedicated StatefulSet/operator shape |
| Primary Replicas | #766 | planned | planned | none | not_ready | raft-backed task state and timers |
| CLI Interface | #766 | planned | planned | none | not_ready | `defer` CLI for queue/task/admin and agent docs |
| Long-Running Stability | #766 | planned | planned | none | not_ready | timer, retry, DLQ, and recovery gates |
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
Surfaces: Runtime: timer wheel, lease store, retry scheduler, DLQ writer, snapshot, and recovery paths.
EC Dimensions: stability: pending long-running task gate - soak, restart, timer recovery, duplicate prevention, bounded memory, and backpressure
Required Verification: conformance, dogfood
Promise:
Defer remains stable under sustained scheduled-task load, retries, DLQ writes,
and restart cycles without losing committed tasks or duplicating terminal acks.
Gate Inventory:
- pending: projects/defer/tests/long_running_stability.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| delayed-task-soak-and-recovery | epic | #766 | planned | planned | none | pending long-running task gate |

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
Surfaces: HTTP: `/v1/queues/{queue}/tasks` - create, schedule, lease, ack, cancel, and inspect delayed tasks.
EC Dimensions: behavior: pending task lifecycle conformance gate - schedule ordering, cancellation, leases, and terminal states
Required Verification: smoke, conformance
Promise:
Defer manages delayed task state from creation through terminal success,
failure, cancellation, or dead-letter handoff.
Gate Inventory:
- pending: projects/defer/tests/task_lifecycle.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| delayed-task-state-machine | epic | #766 | planned | planned | none | pending task lifecycle conformance gate |

### HTTP Dispatch And Retries

ID: http-dispatch-and-retries
Type: RuntimeTool
Root WI: #766
Status: confirmed
Surfaces: HTTP worker target dispatch - signed delivery attempts, retry policy, and DLQ transitions.
EC Dimensions: behavior: pending dispatch gate - target call, retry backoff, idempotency key, and DLQ behavior
Required Verification: smoke, conformance, negative
Promise:
Defer dispatches tasks to HTTP targets with bounded retries, dedupe keys, and
explicit dead-letter behavior.
Gate Inventory:
- pending: projects/defer/tests/http_dispatch.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| http-target-attempt-contract | epic | #766 | planned | planned | none | pending dispatch/retry gate |

### Queue Rate Limits

ID: queue-rate-limits
Type: RuntimeTool
Root WI: #766
Status: confirmed
Surfaces: HTTP/Admin: queue config and runtime scheduler - QPS, concurrency, backoff, and pause/resume controls.
EC Dimensions: behavior: pending rate-limit gate - per-queue QPS/concurrency enforcement and pause/resume
Required Verification: smoke, conformance
Promise:
Defer enforces per-queue rate limits and concurrency limits before dispatching
tasks to external targets.
Gate Inventory:
- pending: projects/defer/tests/rate_limits.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| per-queue-rate-limit-contract | epic | #766 | planned | planned | none | pending rate-limit conformance gate |

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
