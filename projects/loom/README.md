# Loom

## Brief

Loom is the workflow scheduler in the Axiom service stack.

It owns workflow run state, dynamic DAG orchestration, runner selection,
timers, fair dispatch, and client control APIs. It coordinates through small
messages and references only: payload/result bytes live in `keep`, online task
delivery happens through `relay`, historical replay belongs to `tape`, and
delayed external callbacks can use `defer`.

## Boundaries

Loom is a control plane, never a data path.

```
client -> loom submit/status/result-ref + keep payload/result bytes
loom   -> relay publish task / observe ack + keep result refs
worker -> relay lease/ack/heartbeat + keep input/result bytes
```

- `loom` owns workflow state and orchestration decisions.
- `relay` owns online broker delivery and worker leasing.
- `keep` owns payload/result bytes and claim-check refs.
- `tape` owns replay/audit history when workflow events must be replayed later.
- `defer` owns delayed HTTP task dispatch; Loom may use it for timers/callbacks.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Workflow Orchestration | #116 | planned | planned | none | not_ready | dynamic DAG ready-node scheduling and fan-in/fan-out |
| Workflow Data Model | #112 | planned | planned | none | not_ready | WorkflowRun, Node, Stage, attempts, refs, and history records |
| State Durability | #110 | planned | planned | none | not_ready | sharded raft-backed workflow state and crash recovery |
| Runner And Execution Selection | #164 | planned | planned | none | not_ready | resident, k8s-job, and local runner metadata and dispatch |
| Client Control API | #165 | planned | planned | none | not_ready | h2c/OpenAPI submit/status/result-ref surface |
| Worker Harness | #164 | planned | planned | none | not_ready | reference worker harness over relay + keep |
| Fair Dispatch | #107 | planned | planned | none | not_ready | weighted fairness, quota, and bounded materialization |
| Competitive Perf Gate | #127 | planned | planned | none | not_ready | scheduler throughput and dormant-axis benchmark |
| CLI Interface | #165 | planned | planned | none | not_ready | `loom` CLI for submit/status/worker/admin and agent docs |
| HTTP/2 API List | #165 | planned | planned | none | not_ready | h2c/OpenAPI route inventory and contract tests |
| Kubernetes-Native Deployment | #165 | planned | planned | none | not_ready | dedicated StatefulSet/operator topology |
| Long-Running Stability | #110 | planned | planned | none | not_ready | soak, crash recovery, and bounded resource gates |
| Primary Replicas | #110 | planned | planned | none | not_ready | raft-backed primary/replica topology |
| Security Hardening | #165 | planned | planned | none | not_ready | authn/authz, tenant isolation, audit events, and secret rotation |

### CLI Interface

ID: cli-interface
Type: RuntimeTool
Root WI: #165
Status: confirmed
Surfaces: CLI: `loom llm`, `loom upgrade`, `loom issue`, `loom submit`, `loom status`, `loom worker`, and admin/debug verbs.
EC Dimensions: behavior: pending CLI convention gate - required standard verbs, workflow submit/status ergonomics, worker harness wiring, and offline agent docs
Required Verification: smoke, conformance
Promise:
Loom ships an agent-drivable CLI that follows the repository CLI convention and
can operate the workflow control plane without bespoke scripts.
Gate Inventory:
- pending: projects/loom/tests/cli_contract.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| loom-cli-convention-and-control-verbs | epic | #165 | planned | planned | none | pending CLI convention gate |

### HTTP/2 API List

ID: http2-api-list
Type: RuntimeTool
Root WI: #165
Status: confirmed
Surfaces: HTTP: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs`, workflow submit/status/control/result-ref routes.
EC Dimensions: behavior: pending h2c/OpenAPI route-list gate - probes, metrics, OpenAPI, route inventory, validation, and status codes
Required Verification: smoke, conformance
Promise:
Loom exposes a compact h2c/OpenAPI route list for workflow submission, run
control, status, result references, probes, metrics, and generated docs.
Gate Inventory:
- pending: projects/loom/tests/http_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| h2c-openapi-workflow-route-list | epic | #165 | planned | planned | none | pending h2c/OpenAPI route-list gate |

### Kubernetes-Native Deployment

ID: kubernetes-native-deployment
Type: Devops
Root WI: #165
Status: confirmed
Surfaces: K8s: dedicated StatefulSet/operator topology for workflow state, probes, metrics, backup/restore, PDBs, and runner integration.
EC Dimensions: behavior: pending kustomize/operator render gate - CRD, operator, instance render, and kind dogfood
Required Verification: smoke, dogfood
Promise:
Loom runs as a dedicated k8s-native workflow scheduler with operator-managed
state, probes, backup policy, rolling upgrades, and stable network identity.
Gate Inventory:
- pending: projects/loom/k8s

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| dedicated-workflow-scheduler-topology | epic | #165 | planned | planned | none | pending k8s render/dogfood gates |

### Long-Running Stability

ID: long-running-stability
Type: Runtime
Root WI: #110
Status: confirmed
Surfaces: Runtime: scheduler event loop, raft state, relay/keep client pools, compaction, snapshot, and recovery paths.
EC Dimensions: stability: pending long-running gate - soak, restart, crash recovery, bounded memory, and backpressure behavior
Required Verification: conformance, dogfood
Promise:
Loom remains stable under long-running workflow load, restart cycles, and
backpressure without leaking resources or losing committed workflow transitions.
Gate Inventory:
- pending: projects/loom/tests/long_running_stability.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| workflow-scheduler-soak-and-recovery | epic | #110 | planned | planned | none | pending long-running stability gate |

### Primary Replicas

ID: primary-replicas
Type: Runtime
Root WI: #110
Status: confirmed
Surfaces: Raft: workflow state primary/replica topology over `libs/raft-core` and `libs/raft-host`.
EC Dimensions: stability: pending raft primary/replica gate - leader failover, replica catch-up, snapshot restore, and committed-transition safety
Required Verification: conformance, dogfood
Promise:
Loom replicates workflow state through a primary/replica topology so committed
workflow transitions survive leader failure and replica recovery.
Gate Inventory:
- pending: projects/loom/tests/primary_replicas.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| raft-backed-workflow-primary-replicas | epic | #110 | planned | planned | none | pending primary/replica failover gate |

### Security Hardening

ID: security-hardening
Type: Devops
Root WI: #165
Status: confirmed
Surfaces: HTTP/K8s: authn/authz, tenant/workflow authorization, network policy, audit events, secret rotation, and request limits.
EC Dimensions: behavior: pending security gate - auth failure cases, tenant isolation, audit emission, secret rotation, and abuse limits
Required Verification: negative, conformance
Promise:
Loom protects workflow control APIs and worker integration with explicit tenant
authorization, auditability, network policy, and managed secret rotation.
Gate Inventory:
- pending: projects/loom/tests/security_hardening.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| workflow-control-security-boundary | epic | #165 | planned | planned | none | pending security hardening gate |

### Workflow Orchestration

ID: workflow-orchestration
Type: RuntimeTool
Root WI: #116
Status: confirmed
Surfaces: HTTP: workflow submit/status/control routes; Scheduler: ready-node frontier, dynamic fan-out/fan-in, and fan-in barriers.
EC Dimensions: behavior: pending workflow orchestration conformance gate - DAG transitions, retry, fan-in, fan-out, and completion state
Required Verification: smoke, conformance
Promise:
Loom schedules dynamic workflow DAGs by selecting ready nodes, publishing work
through Relay, observing completion, and advancing workflow state deterministically.
Gate Inventory:
- pending: projects/loom/tests/workflow_orchestration.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| dynamic-dag-frontier-and-barrier | epic | #116 | planned | planned | none | pending workflow orchestration gate |

### Workflow Data Model

ID: workflow-data-model
Type: RuntimeTool
Root WI: #112
Status: confirmed
Surfaces: Rust/API schema: WorkflowRun, Node, Stage, Attempt, ResultRef, InputRef, and workflow history records.
EC Dimensions: behavior: pending data-model conformance gate - serialization, versioning, validation, and compatibility
Required Verification: smoke, conformance
Promise:
Loom defines the durable workflow model used by clients, controllers, runners,
and status/history views.
Gate Inventory:
- pending: projects/loom/tests/workflow_data_model.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| workflow-run-node-stage-schema | epic | #112 | planned | planned | none | pending data-model conformance gate |

### State Durability

ID: state-durability
Type: RuntimeTool
Root WI: #110
Status: confirmed
Surfaces: Raft: sharded workflow state machine over `libs/raft-core` and `libs/raft-host`; Snapshot: service-owned workflow state snapshots.
EC Dimensions: stability: pending raft durability gate - crash recovery, snapshot restore, and failover without committed workflow loss
Required Verification: conformance, dogfood
Promise:
Loom persists workflow state before acknowledgement and survives process or
leader failure without losing committed workflow transitions.
Gate Inventory:
- pending: projects/loom/tests/raft_workflow_state.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| sharded-raft-workflow-state | epic | #110 | planned | planned | none | pending raft durability/failover gate |

### Runner And Execution Selection

ID: runner-execution-selection
Type: RuntimeTool
Root WI: #164
Status: confirmed
Surfaces: Scheduler: runner metadata and dispatch policy for resident, k8s-job, and local runners.
EC Dimensions: behavior: pending runner-selection gate - runner routing, fallback, retry, and status attribution
Required Verification: smoke, conformance
Promise:
Loom selects the runner class for each task, while Relay routes delivery and
workers own actual execution.
Gate Inventory:
- pending: projects/loom/tests/runner_selection.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| per-task-runner-selection-contract | epic | #164 | planned | planned | none | pending runner-selection gate |

### Client Control API

ID: client-control-api
Type: RuntimeTool
Root WI: #165
Status: confirmed
Surfaces: HTTP: `/v1/workflows`, `/v1/runs/{run_id}`, `/v1/runs/{run_id}/result-ref`, `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs`.
EC Dimensions: behavior: pending h2c/OpenAPI API gate - submit, status, result-ref, validation, probes, metrics, and OpenAPI
Required Verification: smoke, conformance
Promise:
Loom exposes a thin h2c/OpenAPI control API for submitting workflows, checking
status, and obtaining result refs without moving payload bytes.
Gate Inventory:
- pending: projects/loom/tests/client_control_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| h2c-workflow-control-api | epic | #165 | planned | planned | none | pending client-control API gate |

### Worker Harness

ID: worker-harness
Type: RuntimeTool
Root WI: #164
Status: confirmed
Surfaces: CLI: `loom worker` reference harness; Worker protocol: relay lease/heartbeat/ack plus keep input/result refs.
EC Dimensions: behavior: pending worker-harness gate - lease, keep IO, ack exactly once, heartbeat, and retry boundaries
Required Verification: smoke, conformance
Promise:
Loom ships a reference worker harness while preserving the polyglot boundary:
workers talk to Relay and Keep, not to Loom.
Gate Inventory:
- pending: projects/loom/tests/worker_harness.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| reference-worker-relay-keep-loop | epic | #164 | planned | planned | none | pending worker-harness gate |

### Fair Dispatch

ID: fair-dispatch
Type: RuntimeTool
Root WI: #107
Status: confirmed
Surfaces: Scheduler: weighted fair share, quota, namespace/workflow priority, and lazy bounded materialization.
EC Dimensions: behavior: pending fairness conformance gate - quota, priority, starvation prevention, and bounded materialization
Required Verification: smoke, conformance
Promise:
Loom schedules ready work fairly across tenants, workflows, and runner classes
without materializing unbounded dormant branches.
Gate Inventory:
- pending: projects/loom/tests/fair_dispatch.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| weighted-fair-frontier-dispatch | epic | #107 | planned | planned | none | pending fairness conformance gate |

### Competitive Perf Gate

ID: competitive-perf-gate
Type: RuntimeTool
Root WI: #127
Status: confirmed
Surfaces: Meter/Vat: scheduler throughput and dormant-axis benchmark; Benchmark: Temporal/Celery-style comparison harnesses.
EC Dimensions: efficiency: pending loom scheduler meter gate - frontier throughput, memory, dormant branches, and dispatch latency
Required Verification: dogfood
Promise:
Loom keeps workflow scheduling performance tied to repeatable meter/vat gates
instead of anecdotal benchmark runs.
Gate Inventory:
- pending: projects/loom/vat.toml
- pending: projects/loom/benchmark

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| scheduler-throughput-and-dormant-axis-gate | epic | #127 | planned | planned | none | pending meter/vat benchmark gate |
