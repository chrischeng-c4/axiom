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
| Delayed Task Lifecycle | #766 | implemented | passing | conformance | ready | Raft-committed create/batch-create, ETA/priority, fenced lease, batch settle, retry, DLQ, cancel, inspect, snapshot and recovery |
| HTTP Dispatch And Retries | #766 | implemented | passing | conformance | ready | bounded concurrent real HTTP delivery with stable idempotency, optional HMAC signing, committed lease before side effect, committed settlement, and an executable ambiguous-outcome retry gate |
| Queue Rate Limits | #766 | implemented | passing | conformance | ready | committed per-queue pause/resume/disable, max-in-flight, dispatch budget, and token-bucket rate/burst state shared by all replicas |
| HTTP/2 API List | #766 | implemented | passing | conformance | ready | one HTTP/1.1+h2c service port, exact OpenAPI/docs/probes/metrics, queue/task/batch/dispatch/backup routes, and generated clients |
| Kubernetes-Native Deployment | #766 | implemented | verified | dogfood | ready | layered base/overlays/components plus CRD/operator/instance, PDB/PVC/NetworkPolicy/observability, and disposable Kind lifecycle-recovery gate |
| Primary Replicas | #766 | implemented | passing | conformance | ready | every mutation is Raft committed; replicas apply identical scheduler bytes with committed executor/epoch fencing and durable restart recovery |
| CLI Interface | #766 | implemented | passing | conformance | ready | `defer` service/spec/client/queue/task/dispatch/backup/k8s/dockerfile plus standard `llm`/`upgrade`/`issue` verbs |
| Long-Running Stability | #766 | implemented | verified | dogfood | ready | lease-expiry/two-cycle failover gates, PVC recovery, and 60-second fixed-state HTTP/Raft error/RSS/FD/thread/p99 plateau soak |
| Security Hardening | #766 | implemented | passing | conformance | ready | queue RBAC, audited live credential rotation, admission limits, signed targets, peer mTLS, restricted pods, NetworkPolicy, and secret projection |
| Competitor Feature Parity | #766 | implemented | verified | conformance | ready | explicit Google Cloud Tasks contract matrix and documented exclusions; no Celery/Sidekiq category mismatch |
| Competitor Performance | #766 | implemented | passing | dogfood | ready | same-host Defer/Relay full durable lifecycle gate requires `Defer / Relay >= 0.8`; no unmeasured Cloud Tasks win is claimed |
| Defer App Source Root Taxonomy | #1217 | implemented | verified | smoke | ready | source-root routing uses `apps/defer` while preserving legacy TD identity |

### Defer App Source Root Taxonomy

ID: defer-app-source-root-taxonomy
Type: Devops
Root WI: #1217
Status: confirmed
Surfaces: Repo taxonomy: root inventory, Cargo workspace routing, AW project config, project-local docs, scripts, tests, and generated evidence paths.
EC Dimensions: behavior: verified repo taxonomy gate - defer resolves through apps/defer while tracker identity remains app:defer
Required Verification: smoke
Promise:
Defer's app-facing source root resolves through `apps/defer` without changing
the GitHub label, AW project name, persistent branch convention, or TD bucket
identity.
Gate Inventory:
- verified: apps/defer/tech-design/logic/repo-taxonomy-move-defer-from-projects-defer-to-apps-defer.md
- verified: `aw capability check --project defer`
- verified: `aw wi list --project defer --state open`
- verified: stale source-root scan permits only preserved TD-bucket references

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| defer-app-source-root-taxonomy | change | #1217 | implemented | verified | smoke | TD merged; source-root migration smoke checks passed |

### CLI Interface

ID: cli-interface
Type: RuntimeTool
Root WI: #766
Status: confirmed
Surfaces: CLI: `defer llm`, `defer upgrade`, `defer issue`, queue/task create/status/cancel, and admin/debug verbs.
EC Dimensions: behavior: `cargo test -p defer --test cli_contract` - required standard verbs, task lifecycle ergonomics, exact spec/render output, and offline agent docs
Required Verification: smoke, conformance
Promise:
Defer ships an agent-drivable CLI for delayed task lifecycle, queue control,
and admin workflows while following the repository-wide CLI convention.
Gate Inventory:
- apps/defer/tests/cli_contract.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| defer-cli-convention-and-task-verbs | epic | #766 | implemented | passing | conformance | apps/defer/tests/cli_contract.rs |

### Long-Running Stability

ID: long-running-stability
Type: Runtime
Root WI: #766
Status: confirmed
Surfaces: Runtime: priority-ready set, committed queue-control/rate bucket, fenced lease store, retry scheduler, DLQ state, Raft snapshot/recovery, and bounded HTTP dispatcher.
EC Dimensions: stability: `cargo test -p defer --test task_lifecycle --test rate_limits --test raft_scheduler`; dogfood: `DEFER_SOAK_AUTOSTART=1 bash apps/defer/scripts/soak.sh`; Kubernetes: `bash apps/defer/scripts/kind-e2e.sh`
Required Verification: conformance, dogfood
Promise:
Defer remains stable under sustained scheduled-task load, retries, DLQ writes,
and restart cycles without losing committed tasks or duplicating terminal acks.
The 2026-07-17 default run crossed the 1,024-entry proposal-cache/snapshot
cadence before measuring 60 seconds: 1,043 fixed-state operations, zero errors,
RSS 41,584 -> 42,192 KiB (1%), FD 18 -> 18, threads 12 -> 12, and task-read
p99 2 -> 1 ms.
Gate Inventory:
- apps/defer/tests/task_lifecycle.rs; apps/defer/tests/rate_limits.rs
- apps/defer/tests/raft_scheduler.rs (same-directory node restart, snapshot recovery, and repeated failover)
- apps/defer/scripts/soak.sh; apps/defer/scripts/kind-e2e.sh
- libs/service-observability/scripts/soak-metrics.sh

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| delayed-task-soak-and-recovery | epic | #766 | implemented | verified | dogfood | lifecycle/raft tests plus bounded soak and operator/PVC Kind recovery |

### Security Hardening

ID: security-hardening
Type: Devops
Root WI: #766
Status: confirmed
Surfaces: HTTP/K8s: queue/task authn/authz, signed target dispatch, tenant isolation, network policy, audit events, secret rotation, and request limits.
EC Dimensions: security: shared audited queue RBAC and credential rotation, HMAC target signing, real peer mTLS, bounded admission, restricted K8s security contexts, read-only secret projection, and NetworkPolicy
Required Verification: negative, conformance
Promise:
Defer protects task control and target dispatch with explicit authorization,
signed delivery, auditability, network policy, and managed secret rotation.
Gate Inventory:
- apps/defer/tests/http_api.rs; apps/defer/tests/service_auth.rs
- apps/defer/tests/http_dispatch.rs; apps/defer/tests/service_admission.rs
- apps/defer/tests/raft_peer_mtls.rs; apps/defer/tests/direct_k8s_assets.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| delayed-task-security-boundary | epic | #766 | implemented | passing | conformance | auth rotation/audit, target signing, admission, peer mTLS, and K8s negative/static gates |

### Competitor Feature Parity

ID: competitor-feature-parity
Type: RuntimeTool
Root WI: #766
Status: confirmed
Surfaces: Docs/Test: delayed HTTP push-task feature matrix against Google Cloud Tasks; Celery and Sidekiq are excluded because they are worker frameworks, not managed HTTP push queues.
EC Dimensions: behavior: schedule, target delivery, terminal success, retry/DLQ, rate/burst/concurrency, cancel/inspect, dedupe/idempotency, and target authentication
Required Verification: conformance
Promise:
Defer keeps an explicit delayed-task feature matrix against established task
dispatch systems, with comparison scope changed only when product requirements change.
Gate Inventory:
- apps/defer/benchmarks/competitor-feature-matrix.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| delayed-task-competitor-feature-matrix | epic | #766 | implemented | verified | conformance | apps/defer/benchmarks/competitor-feature-matrix.md |

### Competitor Performance

ID: competitor-performance
Type: RuntimeTool
Root WI: #766
Status: confirmed
Surfaces: Test: identical single-voter fsync-always durable enqueue -> committed lease -> committed ack workload in Defer and Relay, with throughput, p50/p95/p99, CPU, process RSS, disk bytes, amplification, and error counts.
EC Dimensions: efficiency: `cargo test --release -p defer --test relay_performance_ceiling -- --ignored --nocapture` requires Defer throughput to remain at least 80% of Relay under the declared workload
Required Verification: dogfood
Promise:
Defer bounds the cost of its additional ETA, permit, retry, DLQ, and terminal
state machinery against the sibling Relay work queue. Cloud Tasks remains the
external feature-contract peer; no Cloud performance claim is made without a
real queue, public target, declared region, and equivalent network conditions.
Gate Inventory:
- apps/defer/tests/relay_performance_ceiling.rs
- apps/defer/benchmarks/competitor-feature-matrix.md
- apps/defer/benchmarks/relay-performance-ceiling.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| delayed-task-competitor-performance-baseline | epic | #766 | implemented | passing | dogfood | release-mode Defer/Relay ceiling gate (`minimum_ratio = 0.8`); apps/defer/benchmarks/relay-performance-ceiling.md |

### Delayed Task Lifecycle

ID: delayed-task-lifecycle
Type: RuntimeTool
Root WI: #766
Status: confirmed
Surfaces: Rust/Raft: `DeferScheduler` and `DeferRaft`; HTTP: queue/task single and batch routes, status, cancel, and dispatch.
EC Dimensions: behavior: lifecycle/rate-limit/Raft scheduler tests cover ordering, batch atomicity, cancellation, fenced leases, settlement, failover, and terminal states
Required Verification: smoke, conformance
Promise:
Defer manages delayed task state from creation through terminal success,
failure, cancellation, or dead-letter handoff. Task priority is `u8`
(`0..=255`, default `10`); it is applied only after the task is due and the
queue can dispatch more work.
Gate Inventory:
- apps/defer/tests/task_lifecycle.rs; apps/defer/tests/rate_limits.rs; apps/defer/tests/raft_scheduler.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| delayed-task-state-machine | epic | #766 | implemented | passing | conformance | apps/defer/tests/task_lifecycle.rs |
| due-task-priority-ordering | epic | #766 | implemented | passing | conformance | apps/defer/tests/task_lifecycle.rs |

### HTTP Dispatch And Retries

ID: http-dispatch-and-retries
Type: RuntimeTool
Root WI: #766
Status: confirmed
Surfaces: Rust API: `HttpDispatcher::dispatch_batch`; Raft: committed `LeaseDue` and `SettleBatch`; HTTP: real target method/headers/body/idempotency/signature and manual dispatch route.
EC Dimensions: behavior: `cargo test -p defer --test http_dispatch --test http_api --test rate_limits` - real delivery, signing, retry/DLQ, batch settlement, and lost-ownership fencing
Required Verification: smoke, conformance, negative
Promise:
Defer dispatches tasks to HTTP targets with bounded retries, dedupe keys, and
explicit dead-letter behavior. If a target accepts the effect while the worker
loses its committed fence before settlement, Defer reports `LostOwnership` and
retries with the same idempotency key; only the fenced retry may commit the
terminal outcome.
Gate Inventory:
- apps/defer/tests/task_lifecycle.rs; apps/defer/tests/http_dispatch.rs; apps/defer/tests/http_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| http-target-attempt-contract | epic | #766 | implemented | passing | conformance | real HTTP delivery, signing, stable idempotency, retry/DLQ, committed batch settlement, and accepted-HTTP/lost-fence retry proof |

### Queue Rate Limits

ID: queue-rate-limits
Type: RuntimeTool
Root WI: #766
Status: confirmed
Surfaces: Rust/Raft/HTTP: `QueuePolicy`, committed queue configuration/control, max dispatch per tick/second, burst, max-in-flight, snapshot, and priority among eligible tasks.
EC Dimensions: behavior: `cargo test -p defer --test rate_limits --test task_lifecycle` - per-queue dispatch budget/concurrency/rate enforcement, pause/resume/disable controls, policy update isolation, and priority-preserving dispatch among eligible tasks
Required Verification: smoke, conformance
Promise:
Defer enforces Cloud Tasks-style per-queue controls, rate limits, and
concurrency limits before dispatching tasks to external targets.
Gate Inventory:
- apps/defer/tests/rate_limits.rs; apps/defer/tests/task_lifecycle.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| per-queue-rate-limit-contract | epic | #766 | implemented | passing | conformance | apps/defer/tests/rate_limits.rs; apps/defer/tests/task_lifecycle.rs |

### HTTP/2 API List

ID: http2-api-list
Type: RuntimeTool
Root WI: #766
Status: confirmed
Surfaces: HTTP: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs`, queue/task/admin routes.
EC Dimensions: behavior: `cargo test -p defer --test http_api --test cli_contract` - live HTTP/1.1+h2c probes/metrics/OpenAPI, domain routes, offline spec twin, and client generation
Required Verification: smoke, conformance
Promise:
Defer exposes a compact h2c/OpenAPI API list for queue/task lifecycle and
operator workflows.
Gate Inventory:
- apps/defer/tests/http_api.rs; apps/defer/tests/cli_contract.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| h2c-openapi-route-list | epic | #766 | implemented | passing | conformance | apps/defer/tests/http_api.rs; apps/defer/tests/cli_contract.rs |

### Kubernetes-Native Deployment

ID: kubernetes-native-deployment
Type: Devops
Root WI: #766
Status: confirmed
Surfaces: K8s: dedicated StatefulSet/operator topology for queues, timers, storage, probes, backups, and PDBs.
EC Dimensions: behavior: direct Kustomize and CRD/operator/instance render tests; stability: disposable Kind operator/PVC queue-task lifecycle recovery
Required Verification: smoke, dogfood
Promise:
Defer runs as a dedicated k8s-native task dispatch service with operator-managed
queues, rate limits, storage, backup policy, and lifecycle.
Gate Inventory:
- apps/defer/k8s; apps/defer/tests/direct_k8s_assets.rs; apps/defer/tests/operator.rs
- apps/defer/scripts/kind-e2e.sh

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| dedicated-task-service-topology | epic | #766 | implemented | verified | dogfood | layered assets, operator conformance, and Kind PVC lifecycle recovery |

### Primary Replicas

ID: primary-replicas
Type: Runtime
Root WI: #766
Status: confirmed
Surfaces: Raft: delayed task state machine over `libs/raft-core` and `libs/raft-runtime`.
EC Dimensions: stability: `cargo test -p defer --test raft_scheduler --test raft_peer_mtls` - committed task recovery, same-directory recovered-node catch-up followed by a second leader loss, fencing, snapshots, and authenticated peers
Required Verification: conformance, dogfood
Promise:
Defer replicates task state and timer ownership through raft so failover
preserves scheduled tasks and terminal outcomes. The integration gate restarts
the first failed primary from durable state, waits for task-state convergence,
then removes the current leader and completes another full create/lease/ack
lifecycle through the surviving quorum.
Gate Inventory:
- apps/defer/tests/raft_scheduler.rs; apps/defer/tests/raft_peer_mtls.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| raft-backed-task-scheduler | epic | #766 | implemented | passing | conformance | three-voter failover, durable replay/snapshot recovery, and real peer mTLS |
