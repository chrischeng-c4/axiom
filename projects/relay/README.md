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
| CLI Interface | 108 | implemented | passing | conformance | not_ready | relay-server and relay-raft expose the binary surfaces; install/build artifacts still need standardization |
| Competitive Broker Feature Parity | - | implemented | planned | dogfood | not_ready | durable log, queue lifecycle, OpenAPI worker flow, and raft failover are implemented; kind failover remains the dogfood gap |
| Competitive Broker Performance | 125 | implemented | planned | dogfood | not_ready | local throughput and ratchet logic exist; external NATS/RabbitMQ/Redpanda arena comparison remains dogfood |

### CLI Interface

ID: cli-interface
Type: RuntimeTool
Surfaces: CLI: `relay-server` - single-node h2c broker process.; CLI: `relay-raft` - raft-backed Kubernetes node process.; HTTP: `/openapi.json` - machine-readable worker contract served by the binary.
EC Dimensions: behavior: `cargo test -p relay --test worker_loop --test raft_config --test raft_cluster` - binary-facing contract and raft node smoke
Root WI: 108
Status: auditing
Required Verification: conformance
Promise:
Expose relay as runnable binaries with stable process entrypoints for local h2c
workers and Kubernetes raft nodes, including a served OpenAPI contract for
non-Rust clients.
Gate Inventory:
- projects/relay/tests/worker_loop.rs; projects/relay/tests/raft_config.rs; projects/relay/tests/raft_cluster.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Relay server process interface | epic | - | implemented | passing | conformance | projects/relay/src/bin/relay_server.rs; projects/relay/tests/worker_loop.rs |
| Relay raft process interface | epic | - | implemented | passing | conformance | projects/relay/src/bin/relay_raft.rs; projects/relay/tests/raft_config.rs; projects/relay/tests/raft_cluster.rs |
| Served OpenAPI contract | epic | 108 | implemented | passing | conformance | projects/relay/tests/worker_loop.rs; projects/relay/docs/worker-protocol.md |

### Competitive Broker Feature Parity

ID: competitor-feature-parity
Type: RuntimeTool
Surfaces: Rust API: `Relay` - durable ordered log, broadcast, queue lease, and ack primitives.; HTTP: `publish`, `subscribe`, `lease`, `heartbeat`, `ack` - NATS/RabbitMQ-style broker workflows over h2c.; CLI: `relay-raft` - failover-capable broker node.
EC Dimensions: behavior: `cargo test -p relay --test relay_core --test work_queue_api --test worker_loop --test raft_core --test raft_persistence --test raft_cluster` - functional parity conformance for core broker workflows
Root WI: -
Status: auditing
Required Verification: conformance, dogfood
Promise:
Cover the core broker functions Relay needs to compete with NATS, RabbitMQ, and
Redpanda in Axiom workloads: ordered publish/replay, competing-worker delivery,
polyglot worker access, and failover without losing committed entries.
Gate Inventory:
- projects/relay/tests/relay_core.rs; projects/relay/tests/http2_transport.rs; projects/relay/tests/work_queue_api.rs; projects/relay/tests/worker_loop.rs; projects/relay/tests/raft_core.rs; projects/relay/tests/raft_persistence.rs; projects/relay/tests/raft_cluster.rs; projects/relay/scripts/kind-failover-smoke.sh

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Per-subject/shard append ordering | epic | - | implemented | passing | conformance | projects/relay/tests/relay_core.rs |
| Broadcast replay model | epic | - | implemented | passing | conformance | projects/relay/tests/relay_core.rs; projects/relay/tests/http2_transport.rs |
| Lease / heartbeat / ack lifecycle | epic | - | implemented | passing | conformance | projects/relay/tests/work_queue_api.rs |
| HTTP worker protocol parity | epic | 108 | implemented | passing | conformance | projects/relay/tests/worker_loop.rs; projects/relay/docs/worker-protocol.md |
| In-process raft convergence | epic | - | implemented | passing | conformance | projects/relay/tests/raft_core.rs |
| Durable raft hard-state restore | epic | - | implemented | passing | conformance | projects/relay/tests/raft_persistence.rs |
| Real h2c raft cluster smoke | epic | - | implemented | passing | dogfood | projects/relay/tests/raft_cluster.rs |
| Kubernetes kind failover smoke | epic | - | implemented | planned | dogfood | projects/relay/scripts/kind-failover-smoke.sh; projects/relay/k8s |

### Competitive Broker Performance

ID: competitor-performance
Type: RuntimeTool
Surfaces: Arena: `projects/arena/examples/relay-vs-nats-rabbitmq-redpanda.toml` - external broker comparison spec.; Rust bench: `relay_bench` - local broker throughput baseline.
EC Dimensions: efficiency: `arena` - compare and ratchet against NATS, RabbitMQ, and Redpanda; behavior: `cargo test -p relay --test work_queue_throughput --test perf_gate` - throughput model and ratchet conformance
Root WI: 125
Status: auditing
Required Verification: dogfood
Promise:
Keep Relay's performance claims tied to repeatable throughput tests and an
external competitor arena against NATS, RabbitMQ, and Redpanda instead of
local-only anecdotes.
Gate Inventory:
- projects/relay/tests/work_queue_throughput.rs; projects/relay/tests/perf_gate.rs; projects/relay/src/perf_gate.rs; projects/arena/examples/relay-vs-nats-rabbitmq-redpanda.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| O(1) lease cursor throughput | epic | - | implemented | passing | conformance | projects/relay/tests/work_queue_throughput.rs |
| Normalized win/ratchet decision model | epic | 125 | implemented | passing | conformance | projects/relay/tests/perf_gate.rs |
| External broker comparison | epic | 125 | implemented | planned | dogfood | projects/arena/examples/relay-vs-nats-rabbitmq-redpanda.toml |
