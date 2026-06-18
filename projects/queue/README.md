# Queue

## Brief

Queue is the Rust distributed-task-queue library surface for cclab.

It owns the task envelope, worker runtime, broker/backend traits, routing,
rate limiting, revocation, workflow composition, scheduling, push receiver, and
optional Kubernetes execution surfaces. The active crate is
`projects/queue/queue`; the adjacent `projects/queue/kv` tree is only an
optional Ion-style backend dependency. The current manifest gate is blocked by
workspace dependency drift, so the capability map records confirmed public
surfaces without claiming production readiness.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Task Worker Runtime | - | partial | failing | smoke | not_ready | task/worker runtime surface exists, but manifest gate currently fails before tests run |
| Broker Backend Routing | - | partial | failing | smoke | not_ready | broker/backend/routing surfaces exist behind optional features, but manifest gate currently fails before tests run |
| Workflow Scheduling And Execution | - | partial | failing | smoke | not_ready | workflow/scheduler/executor surfaces exist, but manifest gate currently fails before tests run |

### Task Worker Runtime

ID: task-worker-runtime
Type: RuntimeTool
Surfaces: Rust API: `cclab_queue::{Task, TaskRegistry, TaskMessage, TaskState, TaskResult, Worker, WorkerConfig, RetryPolicy, SignalDispatcher, RevocationStore}`
EC Dimensions: behavior: `cargo test --manifest-path projects/queue/queue/Cargo.toml` - task message/state/retry/worker/signal/revocation behavior; currently blocked by manifest dependency drift
Root WI: -
Status: blocked
Required Verification: smoke
Promise:
Cclab Queue defines the task envelope, task registry, worker runtime, retry policy, task state/result model, signal dispatch, and revocation controls for distributed task execution.
Gate Inventory: `cargo test --manifest-path projects/queue/queue/Cargo.toml`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Task worker runtime contract | epic | - | partial | failing | smoke | `cargo test --manifest-path projects/queue/queue/Cargo.toml` fails before tests run: missing workspace dependency `pythonize` |

### Broker Backend Routing

ID: broker-backend-routing
Type: Service
Surfaces: Rust API: `cclab_queue::{Broker, PullBroker, PushBroker, DelayedBroker, BrokerConfig, BrokerMessage, ResultBackend, Router, RateLimiter}`
EC Dimensions: behavior: `cargo test --manifest-path projects/queue/queue/Cargo.toml` - broker contracts, backend contracts, routing, and rate-limit behavior; currently blocked by manifest dependency drift
Root WI: -
Status: blocked
Required Verification: smoke
Promise:
Cclab Queue exposes broker and backend contracts for pull, push, delayed delivery, task result storage, queue routing, and rate-limit enforcement across NATS, Redis, Pub/Sub, Cloud Tasks, and Ion-backed feature sets.
Gate Inventory: `cargo test --manifest-path projects/queue/queue/Cargo.toml`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Broker/backend/routing contract | epic | - | partial | failing | smoke | `cargo test --manifest-path projects/queue/queue/Cargo.toml` fails before tests run: missing workspace dependency `pythonize` |

### Workflow Scheduling And Execution

ID: workflow-scheduling-and-execution
Type: RuntimeTool
Surfaces:
- Rust API: `cclab_queue::{Chain, Group, Chord, Map, Starmap, Chunks, WorkflowEngine, PeriodicScheduler, SchedulerBackend}` - workflow and scheduling primitives.
- Cargo features: `scheduler`, `cloud-scheduler`, `push-receiver`, `k8s` - optional scheduler/executor surfaces.
EC Dimensions:
- behavior: `cargo test --manifest-path projects/queue/queue/Cargo.toml` - workflow primitives, scheduler backends, push receiver auth, and optional executor behavior; currently blocked by manifest dependency drift.
Root WI: -
Status: blocked
Required Verification: smoke
Promise:
Cclab Queue provides workflow composition, periodic and delayed scheduling, push-trigger handling, and optional Kubernetes job execution surfaces for distributed task orchestration.
Gate Inventory:
- `cargo test --manifest-path projects/queue/queue/Cargo.toml`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Workflow/scheduler/executor contract | epic | - | partial | failing | smoke | `cargo test --manifest-path projects/queue/queue/Cargo.toml` fails before tests run: missing workspace dependency `pythonize` |
