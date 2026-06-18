# Queue

## Brief

Queue is the Rust distributed-task-queue library surface for cclab.

It owns the task envelope, worker runtime, broker/backend traits, routing,
rate limiting, revocation, workflow composition, scheduling, push receiver, and
optional Kubernetes execution surfaces. The active crate is
`projects/queue/queue`; the adjacent `projects/queue/kv` tree is only an
legacy Ion-style backend tree. The current manifest gate is runnable from the
root workspace and verifies the active queue crate without claiming production
readiness.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Task Worker Runtime | - | implemented | verified | smoke | not_ready | task/worker runtime surface is covered by the active queue crate smoke gate |
| Broker Backend Routing | - | implemented | verified | smoke | not_ready | broker/backend/routing surfaces are covered by the active queue crate smoke gate |
| Workflow Scheduling And Execution | - | implemented | verified | smoke | not_ready | workflow/scheduler/executor surfaces are covered by the active queue crate smoke gate |

### Task Worker Runtime

ID: task-worker-runtime
Type: RuntimeTool
Surfaces: Rust API: `cclab_queue::{Task, TaskRegistry, TaskMessage, TaskState, TaskResult, Worker, WorkerConfig, RetryPolicy, SignalDispatcher, RevocationStore}`
EC Dimensions: behavior: `cargo test --manifest-path projects/queue/queue/Cargo.toml` - task message/state/retry/worker/signal/revocation behavior
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab Queue defines the task envelope, task registry, worker runtime, retry policy, task state/result model, signal dispatch, and revocation controls for distributed task execution.
Gate Inventory: `cargo test --manifest-path projects/queue/queue/Cargo.toml`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Task worker runtime contract | epic | - | implemented | verified | smoke | `cargo test --manifest-path projects/queue/queue/Cargo.toml` |

### Broker Backend Routing

ID: broker-backend-routing
Type: Service
Surfaces: Rust API: `cclab_queue::{Broker, PullBroker, PushBroker, DelayedBroker, BrokerConfig, BrokerMessage, ResultBackend, Router, RateLimiter}`
EC Dimensions: behavior: `cargo test --manifest-path projects/queue/queue/Cargo.toml` - broker contracts, backend contracts, routing, and rate-limit behavior
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab Queue exposes broker and backend contracts for pull, push, delayed delivery, task result storage, queue routing, and rate-limit enforcement across NATS, Redis, Pub/Sub, Cloud Tasks, and Ion-backed feature sets.
Gate Inventory: `cargo test --manifest-path projects/queue/queue/Cargo.toml`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Broker/backend/routing contract | epic | - | implemented | verified | smoke | `cargo test --manifest-path projects/queue/queue/Cargo.toml` |

### Workflow Scheduling And Execution

ID: workflow-scheduling-and-execution
Type: RuntimeTool
Surfaces:
- Rust API: `cclab_queue::{Chain, Group, Chord, Map, Starmap, Chunks, WorkflowEngine, PeriodicScheduler, SchedulerBackend}` - workflow and scheduling primitives.
- Cargo features: `scheduler`, `cloud-scheduler`, `push-receiver`, `k8s` - optional scheduler/executor surfaces.
EC Dimensions:
- behavior: `cargo test --manifest-path projects/queue/queue/Cargo.toml` - workflow primitives, scheduler backends, push receiver auth, and optional executor behavior.
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab Queue provides workflow composition, periodic and delayed scheduling, push-trigger handling, and optional Kubernetes job execution surfaces for distributed task orchestration.
Gate Inventory:
- `cargo test --manifest-path projects/queue/queue/Cargo.toml`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Workflow/scheduler/executor contract | epic | - | implemented | verified | smoke | `cargo test --manifest-path projects/queue/queue/Cargo.toml` |
