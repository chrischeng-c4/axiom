# relay

## Brief

`relay` is the durable ordered-log and queue broker in the Axiom stack. It owns
append, replay, broadcast fan-out, work-queue leasing, h2c/OpenAPI transport,
and the raft-backed HA path. Payloads stay opaque JSON so higher-level systems
such as lumen and worker runtimes can use relay without relay learning their
domain model.

## Capabilities

The RuntimeTool baseline capabilities selected by `aw.toml` are mandatory for
this long-running broker class. They do not replace Relay's product
capabilities; ordered log, work queue, worker protocol, and raft HA remain
first-class domain roots.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| CLI Interface | 108 | implemented | passing | conformance | not_ready | mandatory baseline: relay-server, relay-raft, and OpenAPI surfaces |
| Competitive Broker Feature Parity | - | implemented | planned | dogfood | not_ready | mandatory baseline: NATS/RabbitMQ/Redpanda replacement breadth; kind failover remains open |
| Competitive Broker Performance | 125 | implemented | planned | dogfood | not_ready | mandatory baseline: vat-isolated meter throughput ratchet; external broker arena is advisory |
| Long-Running Stability | - | implemented | passing | dogfood | not_ready | mandatory baseline: recovery, retention, lease reclaim, and raft restart/failover |
| Security Hardening | - | planned | planned | negative | not_ready | mandatory baseline: opaque payload boundary exists; auth/TLS/negative gates remain open |
| HTTP/2 API List | 108 | implemented | passing | conformance | not_ready | mandatory baseline: concise h2c producer, replay, worker, probe, and OpenAPI route list |
| Kubernetes-Native Deployment | - | implemented | planned | dogfood | not_ready | mandatory baseline: StatefulSet raft deployment and kind failover smoke |
| Primary Replicas | - | implemented | planned | dogfood | not_ready | mandatory baseline: raft leader/follower primary-replica topology |
| Durable Ordered Log | - | implemented | passing | conformance | not_ready | domain: per-subject append, replay, broadcast fan-out, and segment lifecycle |
| Work Queue Lifecycle | - | implemented | passing | conformance | not_ready | domain: lease, heartbeat, ack, redelivery, and reconciler behavior |
| HTTP/OpenAPI Worker Protocol | 108 | implemented | passing | conformance | not_ready | domain: polyglot h2c worker contract |
| Raft HA | - | implemented | planned | dogfood | not_ready | domain: raft state machine, hard-state persistence, h2c cluster, and kind failover |

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
| relay-server-process-interface | epic | - | implemented | passing | conformance | projects/relay/src/bin/relay_server.rs; projects/relay/tests/worker_loop.rs |
| relay-raft-process-interface | epic | - | implemented | passing | conformance | projects/relay/src/bin/relay_raft.rs; projects/relay/tests/raft_config.rs; projects/relay/tests/raft_cluster.rs |
| served-openapi-contract | epic | 108 | implemented | passing | conformance | projects/relay/tests/worker_loop.rs; projects/relay/docs/worker-protocol.md |

### Competitive Broker Feature Parity

ID: competitor-feature-parity
Type: RuntimeTool
Surfaces: Rust API: `Relay` - durable ordered log, broadcast, queue lease, and ack primitives.; HTTP: `publish`, `subscribe`, `lease`, `heartbeat`, `ack` - NATS/RabbitMQ-style broker workflows over h2c.; CLI: `relay-raft` - failover-capable broker node.
EC Dimensions: behavior: `cargo test -p relay --test relay_core --test work_queue_api --test worker_loop --test raft_core --test raft_persistence --test raft_cluster` - functional parity conformance for core broker workflows
Root WI: -
Status: auditing
Required Verification: conformance, dogfood
Promise:
Cover the baseline broker functions Relay needs to compete with NATS, RabbitMQ,
and Redpanda in Axiom workloads.
Gate Inventory:
- projects/relay/tests/relay_core.rs; projects/relay/tests/http2_transport.rs; projects/relay/tests/work_queue_api.rs; projects/relay/tests/worker_loop.rs; projects/relay/tests/raft_core.rs; projects/relay/tests/raft_persistence.rs; projects/relay/tests/raft_cluster.rs; projects/relay/scripts/kind-failover-smoke.sh

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| ordered-log-queue-and-raft-feature-breadth | epic | - | implemented | passing | conformance | projects/relay/tests/relay_core.rs; projects/relay/tests/work_queue_api.rs; projects/relay/tests/raft_core.rs |
| per-subject-shard-append-ordering | epic | - | implemented | passing | conformance | projects/relay/tests/relay_core.rs |
| broadcast-replay-model | epic | - | implemented | passing | conformance | projects/relay/tests/relay_core.rs; projects/relay/tests/http2_transport.rs |
| lease-heartbeat-ack-lifecycle | epic | - | implemented | passing | conformance | projects/relay/tests/work_queue_api.rs |
| http-worker-protocol-parity | epic | 108 | implemented | passing | conformance | projects/relay/tests/worker_loop.rs; projects/relay/docs/worker-protocol.md |
| in-process-raft-convergence | epic | - | implemented | passing | conformance | projects/relay/tests/raft_core.rs |
| durable-raft-hard-state-restore | epic | - | implemented | passing | conformance | projects/relay/tests/raft_persistence.rs |
| real-h2c-raft-cluster-smoke | epic | - | implemented | passing | dogfood | projects/relay/tests/raft_cluster.rs |
| kubernetes-kind-failover-smoke | epic | - | implemented | planned | dogfood | projects/relay/scripts/kind-failover-smoke.sh; projects/relay/k8s |

### Competitive Broker Performance

ID: competitor-performance
Type: RuntimeTool
Surfaces: Meter/Vat: `projects/relay/vat.toml#meter-perf` - isolated meter execution for the throughput ratchet.; Arena: `projects/arena/examples/relay-vs-nats-rabbitmq-redpanda.toml` - advisory external broker comparison spec.; Rust bench: `relay_bench` - local broker throughput baseline.
EC Dimensions: efficiency: `cd projects/relay && ../../target/debug/vat run meter-perf` - meter-owned throughput model and ratchet conformance; behavior: `cargo test -p relay --test work_queue_throughput --test perf_gate` - deterministic local gate shape
Root WI: 125
Status: auditing
Required Verification: dogfood
Promise:
Keep Relay's performance claims tied to repeatable throughput tests and an
vat-isolated meter gate and keep the external competitor arena against NATS,
RabbitMQ, and Redpanda as advisory dogfood until native adapters are promoted.
Gate Inventory:
- projects/relay/vat.toml; projects/relay/tests/work_queue_throughput.rs; projects/relay/tests/perf_gate.rs; projects/relay/src/perf_gate.rs; projects/arena/examples/relay-vs-nats-rabbitmq-redpanda.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| o-1-lease-cursor-throughput | epic | - | implemented | passing | conformance | projects/relay/tests/work_queue_throughput.rs |
| normalized-win-ratchet-decision-model | epic | 125 | implemented | passing | conformance | projects/relay/tests/perf_gate.rs |
| vat-meter-throughput-gate | epic | 125 | implemented | planned | dogfood | projects/relay/vat.toml#meter-perf |
| external-broker-comparison | epic | 125 | implemented | planned | dogfood | projects/arena/examples/relay-vs-nats-rabbitmq-redpanda.toml |

### Long-Running Stability

ID: long-running-stability
Type: RuntimeTool
Surfaces: CLI: `relay-server` - durable service process with reconciler.; CLI: `relay-raft` - failover-capable long-running broker node.; K8s: `projects/relay/k8s` - StatefulSet-oriented raft deployment.
EC Dimensions: stability: `cargo test -p relay --test durable --test segments --test reconciler --test raft_persistence --test raft_cluster` - recovery, retention, lease reclaim, and failover conformance
Root WI: -
Status: auditing
Required Verification: conformance, dogfood
Promise:
Run as a long-lived broker without losing committed entries, leaking stuck
leases forever, or corrupting recovery state across restarts, segment rotation,
and leader failover.
Gate Inventory:
- projects/relay/tests/durable.rs; projects/relay/tests/segments.rs; projects/relay/tests/reconciler.rs; projects/relay/tests/raft_persistence.rs; projects/relay/tests/raft_cluster.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| durable-power-safe-recovery | epic | - | implemented | passing | conformance | projects/relay/tests/durable.rs |
| segment-rotation-and-retention-recovery | epic | - | implemented | passing | conformance | projects/relay/tests/segments.rs |
| lease-reclaim-liveness | epic | - | implemented | passing | conformance | projects/relay/tests/reconciler.rs |
| raft-hard-state-restart-safety | epic | - | implemented | passing | conformance | projects/relay/tests/raft_persistence.rs |
| failover-without-committed-loss | epic | - | implemented | passing | dogfood | projects/relay/tests/raft_cluster.rs |

### Security Hardening

ID: security-hardening
Type: RuntimeTool
Surfaces: Guard/Vat: `projects/relay/vat.toml#guard-security` - isolated guard scan with meter runtime evidence.; HTTP: relay h2c API - worker and producer request boundary.; K8s: `projects/relay/k8s` - deployment boundary for future network policy and identity.
EC Dimensions: security: `cd projects/relay && ../../target/debug/vat run guard-security` - guard-owned static/runtime evidence for the in-process opaque payload boundary; behavior: `cargo test -p relay --test relay_core --test worker_loop` - opaque payload and worker contract boundary smoke
Root WI: -
Status: auditing
Required Verification: negative
Promise:
Keep Relay safe as a long-running broker by preserving opaque payload
boundaries and adding explicit negative gates for request limits, authn/z,
TLS/network policy, and deployment identity before production readiness.
Gate Inventory:
- projects/relay/vat.toml; projects/relay/tests/relay_core.rs; projects/relay/tests/worker_loop.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| opaque-payload-boundary | epic | - | implemented | passing | smoke | projects/relay/tests/relay_core.rs; projects/relay/tests/worker_loop.rs |
| guard-static-runtime-evidence | epic | - | implemented | planned | negative | projects/relay/vat.toml#guard-security |
| request-limit-and-malformed-frame-negative-tests | epic | - | planned | planned | negative | projects/relay/vat.toml#guard-security |
| auth-tls-network-policy-boundary | epic | - | planned | planned | negative | pending guard/negative security inventory |

### HTTP/2 API List

ID: http2-api-list
Type: RuntimeTool
Surfaces: HTTP: publish, subscribe, lease, heartbeat, ack, replay, `/openapi.json`, and probe routes - concise h2c API list for producers and workers.; Docs: `projects/relay/docs/worker-protocol.md` - endpoint contract summary.
EC Dimensions: behavior: `cargo test -p relay --test http2_transport --test worker_loop` - h2c transport and worker protocol conformance
Root WI: 108
Status: auditing
Required Verification: conformance
Promise:
Publish Relay's supported HTTP/2 API as a compact producer, replay, and worker
endpoint inventory, with OpenAPI/docs pointers, without making OpenAPI
completeness the capability definition.
Gate Inventory:
- projects/relay/tests/http2_transport.rs; projects/relay/tests/worker_loop.rs; projects/relay/docs/worker-protocol.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| h2c-producer-and-replay-route-list | epic | - | implemented | passing | conformance | projects/relay/tests/http2_transport.rs |
| worker-lease-heartbeat-ack-route-list | epic | 108 | implemented | passing | conformance | projects/relay/tests/worker_loop.rs; projects/relay/docs/worker-protocol.md |
| served-openapi-contract | epic | 108 | implemented | passing | conformance | projects/relay/tests/worker_loop.rs |

### Kubernetes-Native Deployment

ID: kubernetes-native-deployment
Type: Devops
Surfaces: K8s: `projects/relay/k8s` - StatefulSet-oriented raft deployment and service topology.; Script: `projects/relay/scripts/kind-failover-smoke.sh` - live failover dogfood path.; CLI: `relay-raft` - Kubernetes node process.
EC Dimensions: behavior: `cargo test -p relay --test raft_config --test raft_cluster` - node config and real h2c cluster smoke; stability: `projects/relay/scripts/kind-failover-smoke.sh` - kind failover dogfood
Root WI: -
Status: auditing
Required Verification: conformance, dogfood
Promise:
Run Relay as a Kubernetes-native broker with a StatefulSet-oriented raft node,
stable service discovery, persistent hard state, and a kind failover path for
deployment dogfood.
Gate Inventory:
- projects/relay/k8s; projects/relay/scripts/kind-failover-smoke.sh; projects/relay/tests/raft_config.rs; projects/relay/tests/raft_cluster.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| statefulset-raft-service-topology | epic | - | implemented | planned | dogfood | projects/relay/k8s |
| relay-raft-node-config | epic | - | implemented | passing | conformance | projects/relay/tests/raft_config.rs |
| kubernetes-kind-failover-smoke | epic | - | implemented | planned | dogfood | projects/relay/scripts/kind-failover-smoke.sh |

### Primary Replicas

ID: primary-replicas
Type: Runtime
Surfaces: CLI: `relay-raft` - raft-backed primary/replica broker node.; Rust API: raft state machine and hard-state persistence.; K8s: `projects/relay/k8s` - replica pods with stable identities.
EC Dimensions: stability: `cargo test -p relay --test raft_core --test raft_persistence --test raft_cluster` - leader/follower convergence, hard-state restore, and h2c cluster smoke
Root WI: -
Status: auditing
Required Verification: conformance, dogfood
Promise:
Support a primary/replica broker topology where the raft leader owns writes,
followers replicate committed state, and failover preserves the ordered-log and
work-queue API contract.
Gate Inventory:
- projects/relay/tests/raft_core.rs; projects/relay/tests/raft_persistence.rs; projects/relay/tests/raft_cluster.rs; projects/relay/k8s

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| in-process-leader-follower-convergence | epic | - | implemented | passing | conformance | projects/relay/tests/raft_core.rs |
| durable-primary-replica-hard-state | epic | - | implemented | passing | conformance | projects/relay/tests/raft_persistence.rs |
| real-h2c-replica-cluster-smoke | epic | - | implemented | passing | dogfood | projects/relay/tests/raft_cluster.rs |

### Durable Ordered Log

ID: durable-ordered-log
Type: Runtime
Surfaces: Rust API: `Relay` - append, replay, broadcast fan-out, subject/shard ordering.; Disk: segment log - durable local log lifecycle.
EC Dimensions: behavior: `cargo test -p relay --test relay_core --test durable --test segments` - ordered log and recovery conformance
Root WI: -
Status: auditing
Required Verification: conformance
Promise:
Append messages in per-subject/shard order, replay them through independent
subscriber cursors, broadcast fan-out without payload interpretation, and
recover committed log state across restarts and segment rotation.
Gate Inventory:
- projects/relay/tests/relay_core.rs; projects/relay/tests/durable.rs; projects/relay/tests/segments.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| per-subject-shard-append-ordering | epic | - | implemented | passing | conformance | projects/relay/tests/relay_core.rs |
| broadcast-replay-model | epic | - | implemented | passing | conformance | projects/relay/tests/relay_core.rs; projects/relay/tests/http2_transport.rs |
| segment-rotation-and-retention-recovery | epic | - | implemented | passing | conformance | projects/relay/tests/segments.rs |

### Work Queue Lifecycle

ID: work-queue-lifecycle
Type: Runtime
Surfaces: HTTP: `lease`, `heartbeat`, `ack` - competing-worker queue lifecycle.; Rust API: work queue engine - lease cursor, epoch fencing, redelivery.
EC Dimensions: behavior: `cargo test -p relay --test work_queue_api --test reconciler` - lease/ack/reclaim conformance
Root WI: -
Status: auditing
Required Verification: conformance
Promise:
Deliver competing-worker queue semantics with epoch-fenced leases, heartbeat
extension, idempotent ack, expired-lease reclaim, and redelivery liveness.
Gate Inventory:
- projects/relay/tests/work_queue_api.rs; projects/relay/tests/reconciler.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| lease-heartbeat-ack-lifecycle | epic | - | implemented | passing | conformance | projects/relay/tests/work_queue_api.rs |
| lease-reclaim-liveness | epic | - | implemented | passing | conformance | projects/relay/tests/reconciler.rs |

### HTTP/OpenAPI Worker Protocol

ID: http-openapi-worker-protocol
Type: Runtime
Surfaces: HTTP: h2c worker API - publish, subscribe, lease, heartbeat, ack.; OpenAPI: `/openapi.json` and docs/worker-protocol.md - polyglot worker contract.
EC Dimensions: behavior: `cargo test -p relay --test worker_loop` - worker protocol conformance
Root WI: 108
Status: auditing
Required Verification: conformance
Promise:
Expose Relay's broker and worker lifecycle through a polyglot h2c/OpenAPI
contract so non-Rust workers can lease, heartbeat, ack, publish, and replay.
Gate Inventory:
- projects/relay/tests/worker_loop.rs; projects/relay/docs/worker-protocol.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| http-worker-protocol-parity | epic | 108 | implemented | passing | conformance | projects/relay/tests/worker_loop.rs; projects/relay/docs/worker-protocol.md |
| served-openapi-contract | epic | 108 | implemented | passing | conformance | projects/relay/tests/worker_loop.rs; projects/relay/docs/worker-protocol.md |

### Raft HA

ID: raft-ha
Type: Runtime
Surfaces: CLI: `relay-raft` - raft-backed broker node.; K8s: `projects/relay/k8s` - StatefulSet-oriented deployment.; Rust API: raft state machine and persistence.
EC Dimensions: stability: `cargo test -p relay --test raft_core --test raft_persistence --test raft_cluster` - raft convergence, persistence, and h2c cluster smoke
Root WI: -
Status: auditing
Required Verification: conformance, dogfood
Promise:
Provide a raft-backed HA path that converges in process, persists hard state,
serves through real h2c nodes, and can be dogfooded through a Kubernetes kind
failover smoke.
Gate Inventory:
- projects/relay/tests/raft_core.rs; projects/relay/tests/raft_persistence.rs; projects/relay/tests/raft_cluster.rs; projects/relay/scripts/kind-failover-smoke.sh; projects/relay/k8s

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| in-process-raft-convergence | epic | - | implemented | passing | conformance | projects/relay/tests/raft_core.rs |
| durable-raft-hard-state-restore | epic | - | implemented | passing | conformance | projects/relay/tests/raft_persistence.rs |
| real-h2c-raft-cluster-smoke | epic | - | implemented | passing | dogfood | projects/relay/tests/raft_cluster.rs |
| kubernetes-kind-failover-smoke | epic | - | implemented | planned | dogfood | projects/relay/scripts/kind-failover-smoke.sh; projects/relay/k8s |
