# relay

## Brief

`relay` is the durable ordered-log and queue broker in the Axiom stack. It owns
append, replay, broadcast fan-out, work-queue leasing, h2c/OpenAPI transport,
and the raft-backed HA path. Payloads stay opaque JSON so higher-level systems
such as lumen and worker runtimes can use relay without relay learning their
domain model.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Durable Log & Broadcast | - | implemented | passing | conformance | ready | per-subject/shard ordered append and broadcast replay |
| Work Queue Delivery | - | implemented | passing | conformance | ready | lease, heartbeat, ack, and redelivery contract |
| HTTP/2 OpenAPI Worker Surface | 108 | implemented | passing | conformance | ready | h2c worker-facing API and served OpenAPI document |
| HA Replication | - | implemented | planned | dogfood | not_ready | raftcore and h2c failover pass; release proof still needs kind/k8s failover smoke |
| Competitive Broker Benchmark | 125 | implemented | planned | dogfood | not_ready | arena comparison against NATS/RabbitMQ/Redpanda is configured but external-adapter dependent |

### Durable Log & Broadcast

ID: durable-log
Type: RuntimeTool
Surfaces: Rust API: `Relay` - append and read ordered entries.; HTTP: `POST /v1/{subject}/publish`, `GET /v1/{subject}/subscribe` - publish and broadcast replay over h2c.
EC Dimensions: behavior: `cargo test -p relay --test relay_core` - ordered append and broadcast replay conformance
Root WI: -
Status: auditing
Required Verification: conformance
Promise:
Append opaque messages into a durable, ordered log per subject/shard and replay
that order to every broadcast subscriber.
Gate Inventory:
- projects/relay/tests/relay_core.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Per-subject/shard append ordering | epic | - | implemented | passing | conformance | projects/relay/tests/relay_core.rs |
| Broadcast replay model | epic | - | implemented | passing | conformance | projects/relay/tests/relay_core.rs; projects/relay/tests/http2_transport.rs |

### Work Queue Delivery

ID: work-queue
Type: RuntimeTool
Surfaces: HTTP: `lease`, `heartbeat`, `ack` - worker delivery lifecycle.; Rust API: `Relay` - queue lease and acknowledgement primitives.
EC Dimensions: behavior: `cargo test -p relay --test work_queue_api` - queue lifecycle conformance
Root WI: -
Status: auditing
Required Verification: conformance
Promise:
Offer competing-worker delivery on top of the same ordered log through bounded
lease, heartbeat, ack, and redelivery semantics.
Gate Inventory:
- projects/relay/tests/work_queue_api.rs; projects/relay/tests/work_queue_throughput.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Lease / heartbeat / ack lifecycle | epic | - | implemented | passing | conformance | projects/relay/tests/work_queue_api.rs |
| O(1) lease cursor throughput | epic | - | implemented | passing | conformance | projects/relay/tests/work_queue_throughput.rs |

### HTTP/2 OpenAPI Worker Surface

ID: http2-openapi
Type: RuntimeTool
Surfaces: HTTP: `/openapi.json` and h2c routes - polyglot worker contract.; Test worker: `projects/relay/tests/worker_loop.rs` - reference loop over HTTP only.
EC Dimensions: behavior: `cargo test -p relay --test worker_loop` - reference worker and OpenAPI conformance
Root WI: 108
Status: auditing
Required Verification: conformance
Promise:
Expose relay's worker contract over HTTP/2 cleartext with enough OpenAPI shape
for non-Rust workers to lease, heartbeat, ack, and inspect the route contract.
Gate Inventory:
- projects/relay/tests/worker_loop.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Reference worker over h2c | epic | 108 | implemented | passing | conformance | projects/relay/tests/worker_loop.rs |
| Served OpenAPI worker verbs | epic | 108 | implemented | passing | conformance | projects/relay/tests/worker_loop.rs |

### HA Replication

ID: ha-replication
Type: RuntimeTool
Surfaces: Rust API: `raftcore` integration - replicated log state machine.; CLI: `relay-raft` - k8s-oriented raft node.
EC Dimensions: behavior: `cargo test -p relay --test raft_core --test raft_persistence --test raft_cluster` - raft and failover conformance
Root WI: -
Status: auditing
Required Verification: conformance, dogfood
Promise:
Replicate relay log commands through raftcore so committed broker entries
survive leader failover without duplicate or lost committed entries.
Gate Inventory:
- projects/relay/tests/raft_core.rs; projects/relay/tests/raft_persistence.rs; projects/relay/tests/raft_cluster.rs; projects/relay/scripts/kind-failover-smoke.sh

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| In-process raft convergence | epic | - | implemented | passing | conformance | projects/relay/tests/raft_core.rs |
| Durable raft hard-state restore | epic | - | implemented | passing | conformance | projects/relay/tests/raft_persistence.rs |
| Real h2c raft cluster smoke | epic | - | implemented | passing | dogfood | projects/relay/tests/raft_cluster.rs |
| Kubernetes kind failover smoke | epic | - | implemented | planned | dogfood | projects/relay/scripts/kind-failover-smoke.sh; projects/relay/k8s |

### Competitive Broker Benchmark

ID: broker-benchmark
Type: RuntimeTool
Surfaces: Arena: `projects/arena/examples/relay-vs-nats-rabbitmq-redpanda.toml` - competitor comparison spec.
EC Dimensions: efficiency: `arena` - compare and ratchet against NATS, RabbitMQ, and Redpanda
Root WI: 125
Status: auditing
Required Verification: dogfood
Promise:
Keep relay's broker performance claims tied to an external competitor benchmark
instead of local-only throughput anecdotes.
Gate Inventory:
- projects/relay/src/perf_gate.rs; projects/arena/examples/relay-vs-nats-rabbitmq-redpanda.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Normalized win/ratchet decision model | epic | 125 | implemented | passing | conformance | projects/relay/tests/perf_gate.rs |
| External broker comparison | epic | 125 | implemented | planned | dogfood | projects/arena/examples/relay-vs-nats-rabbitmq-redpanda.toml |
