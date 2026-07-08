# Tape

## Brief

Tape is the topic replay journal in the Axiom service stack.

It owns append-only topic history, replay by offset or timestamp, consumer
checkpoints, retention windows, and backfill/audit workflows. It is intentionally
separate from `relay`: Relay is the online broker and delivery surface; Tape is
the durable replay/archive surface that long-running systems can re-read after
the fact.

Current implementation slice: `apps/tape` is now a Rust workspace crate and
binary with a local file-backed journal for `tape append`, `tape replay`, and
`tape checkpoint`, plus `tape spec`, `tape llm`, `tape upgrade`, `tape issue`,
and a local `tape-bench` performance gate. This is the first Lumen-style service
slice; raft replication, the h2c server, k8s operator, retention workers, and
real-service external peer calibration remain separate work roots.

## Boundaries

- `relay` owns low-latency publish/subscribe and work-queue delivery.
- `tape` owns historical replay, backfill, retention, and audit.
- `loom` may record workflow events into Tape, but workflow decisions remain in
  Loom state.
- `keep` stores payload/result bytes; Tape stores event envelopes and refs.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Topic Replay Journal | #768 | implemented | verified | smoke | ready | local file-backed append and offset/time replay; raft/h2c deferred |
| Consumer Checkpoints | #768 | implemented | verified | smoke | ready | local durable consumer cursor and stale-write rejection |
| Retention And Backfill | #768 | planned | planned | none | not_ready | retention windows, compaction policy, and batch backfill |
| HTTP/2 API List | #768 | implemented | verified | smoke | ready | offline `tape spec` route/OpenAPI inventory; serving h2c deferred |
| Standard Operational Endpoints | #768 | implemented | verified | smoke | ready | offline route inventory for `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs`; server deferred |
| Kubernetes-Native Deployment | #768 | planned | planned | none | not_ready | dedicated StatefulSet/operator shape |
| Primary Replicas | #768 | planned | planned | none | not_ready | raft-backed replicated topic journal |
| CLI Interface | #768 | implemented | verified | smoke | ready | `tape` CLI for local replay/admin, spec, and agent docs |
| CLI Standard Surface | #768 | implemented | verified | smoke | ready | shared `llm`, `upgrade`, and `issue` command groups |
| Chainable Output Conformance | #768 | implemented | verified | smoke | ready | replay/admin commands emit terminal `next:` hints |
| EC Gates Configured | #768 | partial | verified | smoke | not_ready | crate smoke tests exist; vat/meter/guard EC inventory deferred |
| Long-Running Stability | #768 | planned | planned | none | not_ready | soak, retention, compaction, and replay recovery gates |
| Security Hardening | #768 | planned | planned | none | not_ready | producer/consumer authz, tenant isolation, audit, and secret rotation |
| Competitor Feature Parity | #768 | implemented | verified | smoke | ready | Kafka/Redpanda/Pulsar/JetStream/RabbitMQ Streams replay matrix; feature win only over RabbitMQ topic exchange replay gap |
| Competitor Performance | #768 | implemented | verified | smoke | ready | Tape zero-copy replay beats NATS JetStream 20k-event local backlog replay >=1.5x; other replay-log broker wins remain unclaimed |

### CLI Interface

ID: cli-interface
Type: RuntimeTool
Root WI: #768
Status: verified
Surfaces: CLI: `tape append`, `tape replay`, `tape checkpoint`, `tape spec`, `tape llm`, `tape upgrade`, and `tape issue`.
EC Dimensions: behavior: `cargo test -p tape --test cli_contract` - command surface and local replay ergonomics
Required Verification: smoke, conformance
Promise:
Tape ships an agent-drivable CLI for replay, checkpoint, and admin workflows
while following the repository-wide CLI convention.
Gate Inventory:
- apps/tape/tests/cli_contract.rs
- apps/tape/src/bin/tape.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| tape-cli-convention-and-replay-verbs | epic | #768 | implemented | passing | smoke | apps/tape/tests/cli_contract.rs |

### CLI Standard Surface

ID: cli-standard-surface
Type: RuntimeTool
Root WI: #768
Status: verified
Surfaces: CLI: `tape llm`, `tape upgrade`, `tape issue search`, `tape issue view`, `tape issue create`, and `tape issue comment`.
EC Dimensions: behavior: `cargo test -p tape --test cli_contract` - top-level help keeps shared command groups visible
Required Verification: smoke
Promise:
Ship the mandatory shared `cli-std` surface every ecosystem CLI owes without
mixing it into Tape-specific append/replay/checkpoint commands.
Gate Inventory:
- apps/tape/tests/cli_contract.rs
- libs/cli-std/src/llm.rs
- libs/cli-std/src/upgrade.rs
- libs/cli-std/src/issue.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-llm-upgrade-issue-surface | epic | #768 | implemented | passing | smoke | apps/tape/tests/cli_contract.rs |

### Chainable Output Conformance

ID: chainable-output-conformance
Type: RuntimeTool
Root WI: #768
Status: verified
Surfaces: CLI: `tape append`, `tape replay`, and `tape checkpoint` - operational commands that print a runnable follow-up or `next: done`.
EC Dimensions: behavior: `cargo test -p tape --test cli_contract` - local append/replay/checkpoint roundtrip
Required Verification: smoke
Promise:
Keep Tape's operational CLI outputs simple and chainable while raw spec/LLM
payload commands remain direct streams.
Gate Inventory:
- apps/tape/tests/cli_contract.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| local-replay-command-next-markers | epic | #768 | implemented | passing | smoke | apps/tape/tests/cli_contract.rs |

### Long-Running Stability

ID: long-running-stability
Type: Runtime
Root WI: #768
Status: confirmed
Surfaces: Runtime: append log, replay readers, retention/compaction workers, checkpoint store, snapshot, and recovery paths.
EC Dimensions: stability: pending long-running replay gate - soak, restart, retention, compaction, bounded memory, and replay continuity
Required Verification: conformance, dogfood
Promise:
Tape remains stable under sustained append/replay load, retention work, and
restart cycles without losing committed events or corrupting checkpoints.
Gate Inventory:
- pending: apps/tape/tests/long_running_stability.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| topic-replay-soak-and-recovery | epic | #768 | planned | planned | none | pending long-running replay gate |

### Security Hardening

ID: security-hardening
Type: Devops
Root WI: #768
Status: confirmed
Surfaces: HTTP/K8s: producer/consumer authn/authz, tenant/topic isolation, network policy, audit events, secret rotation, and request limits.
EC Dimensions: behavior: pending security gate - auth failure cases, topic isolation, audit emission, secret rotation, and abuse limits
Required Verification: negative, conformance
Promise:
Tape protects topic replay data with explicit producer/consumer authorization,
auditability, network policy, and managed secret rotation.
Gate Inventory:
- pending: apps/tape/tests/security_hardening.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| topic-replay-security-boundary | epic | #768 | planned | planned | none | pending security hardening gate |

### Competitor Feature Parity

ID: competitor-feature-parity
Type: RuntimeTool
Root WI: #768
Status: verified
Surfaces: Docs/Test: replay feature matrix against Kafka, Redpanda, Pulsar, NATS JetStream, RabbitMQ Streams, and RabbitMQ topic exchange routing semantics.
EC Dimensions: behavior: `cargo test -p tape --test competitor_feature_parity` - replay-log functionality, topic-exchange classification, and win/loss claim boundaries
Required Verification: conformance
Promise:
Tape keeps an explicit replay feature matrix against established topic replay
systems. RabbitMQ topic exchange is included as a routing/fanout comparison row,
while RabbitMQ Streams and Kafka-style topic logs remain replay baselines. Tape
claims a replay-feature advantage only over RabbitMQ topic exchange, not over
Kafka, Redpanda, Pulsar, JetStream, or RabbitMQ Streams.
Gate Inventory:
- apps/tape/external-contracts/competitor-feature-parity/behavior/topic-exchange-functional.md
- apps/tape/tests/competitor_feature_parity.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| topic-replay-competitor-feature-matrix | epic | #768 | implemented | passing | smoke | apps/tape/tests/competitor_feature_parity.rs |

### Competitor Performance

ID: competitor-performance
Type: RuntimeTool
Root WI: #768
Status: verified
Surfaces: CLI/Test: `tape-bench run`, local append/replay throughput proxy, p50/p95 append latency, full replay latency, checkpoint write latency, real NATS JetStream replay comparison, and explicit peer calibration ledger.
EC Dimensions: efficiency: `cargo test -p tape --test tape_perf_gate -- --nocapture`; `cargo test -p tape --test tape_vs_nats_jetstream -- --nocapture` - local regression budget plus real JetStream replay win gate
Required Verification: smoke, conformance
Promise:
Tape maintains a local replay performance regression gate and an executable
real-service competitor benchmark. For the current local backlog full-replay
workload, Tape's zero-copy `replay_refs` path must beat NATS JetStream by at
least 1.5x using a test that starts `nats-server -js`, publishes the same
20,000-event, 128-byte-payload backlog, and replays it from the beginning.
Kafka, Redpanda, Pulsar, and RabbitMQ Streams performance wins remain unclaimed
until their own real-service calibration gates exist; RabbitMQ topic exchange
remains routing-only and is not a replay performance baseline.
Gate Inventory:
- apps/tape/src/bench.rs
- apps/tape/src/bin/tape-bench.rs
- apps/tape/tests/tape_perf_gate.rs
- apps/tape/tests/tape_vs_nats_jetstream.rs
- apps/tape/external-contracts/competitor-performance/efficiency/competitive-benchmark.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| topic-replay-competitor-performance-baseline | epic | #768 | implemented | passing | smoke | apps/tape/tests/tape_perf_gate.rs<br>apps/tape/tests/tape_vs_nats_jetstream.rs |

### Topic Replay Journal

ID: topic-replay-journal
Type: RuntimeTool
Root WI: #768
Status: verified
Surfaces: CLI: `tape append`, `tape replay` - local file-backed append and replay; HTTP: `/topics/{topic}/append`, `/topics/{topic}/replay` - declared service route inventory.
EC Dimensions: behavior: `cargo test -p tape` - append ordering plus replay range smoke
Required Verification: smoke, conformance
Promise:
Tape provides a durable append-only topic journal for replay/backfill workloads
without becoming the online broker.
Gate Inventory:
- apps/tape/src/lib.rs
- apps/tape/tests/cli_contract.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| append-and-replay-contract | epic | #768 | implemented | passing | smoke | apps/tape/src/lib.rs<br>apps/tape/tests/cli_contract.rs |

### Consumer Checkpoints

ID: consumer-checkpoints
Type: RuntimeTool
Root WI: #768
Status: verified
Surfaces: CLI: `tape checkpoint get|put`; HTTP: `/topics/{topic}/consumers/{consumer}/checkpoint` - declared service route inventory.
EC Dimensions: behavior: `cargo test -p tape` - checkpoint advance, resume, and stale-write rejection smoke
Required Verification: smoke, conformance
Promise:
Tape persists consumer replay positions so backfills and long-running consumers
resume deterministically after restart or handoff.
Gate Inventory:
- apps/tape/src/lib.rs
- apps/tape/tests/cli_contract.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| durable-consumer-cursor-contract | epic | #768 | implemented | passing | smoke | apps/tape/src/lib.rs<br>apps/tape/tests/cli_contract.rs |

### Retention And Backfill

ID: retention-and-backfill
Type: RuntimeTool
Root WI: #768
Status: confirmed
Surfaces: HTTP: retention policy endpoints and replay export jobs - bounded history and backfill control plane.
EC Dimensions: behavior: pending retention/backfill gate - window trimming, protected checkpoints, and export consistency
Required Verification: smoke, conformance
Promise:
Tape applies retention and compaction policies while preserving explicitly
protected replay windows and producing consistent backfill exports.
Gate Inventory:
- pending: apps/tape/tests/retention_backfill.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| retention-window-and-backfill-contract | epic | #768 | planned | planned | none | pending retention/backfill gate |

### HTTP/2 API List

ID: http2-api-list
Type: RuntimeTool
Root WI: #768
Status: verified
Surfaces: CLI: `tape spec --format routes|openapi|openapi-yaml|json-schema`; HTTP: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs`, topic append/replay/checkpoint routes.
EC Dimensions: behavior: `cargo test -p tape --test cli_contract spec_routes_list_topic_contract -- --exact` - offline route inventory
Required Verification: smoke, conformance
Promise:
Tape exposes a compact h2c/OpenAPI API list for producer, replay, checkpoint,
and operator workflows.
Gate Inventory:
- apps/tape/src/spec.rs
- apps/tape/tests/cli_contract.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| h2c-openapi-route-list | epic | #768 | implemented | passing | smoke | apps/tape/src/spec.rs<br>apps/tape/tests/cli_contract.rs |

### Standard Operational Endpoints

ID: standard-operational-endpoints
Type: Service
Root WI: #768
Status: verified
Surfaces: CLI: `tape spec --format routes`; HTTP: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs` route inventory.
EC Dimensions: behavior: `cargo test -p tape --test cli_contract spec_routes_list_topic_contract -- --exact` - offline standard endpoint inventory
Required Verification: smoke
Promise:
Expose the standard service endpoint contract in Tape's offline spec before the
serving h2c implementation lands.
Gate Inventory:
- apps/tape/src/spec.rs
- apps/tape/tests/cli_contract.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| standard-service-route-inventory | epic | #768 | implemented | passing | smoke | apps/tape/src/spec.rs<br>apps/tape/tests/cli_contract.rs |

### EC Gates Configured

ID: ec-gates-configured
Type: Devops
Root WI: #768
Status: confirmed
Surfaces: Tests: `cargo test -p tape`; future Vat/Meter/Guard gates under `apps/tape/`.
EC Dimensions: behavior: current smoke gate; efficiency/security/stability inventories pending
Required Verification: smoke
Promise:
Keep the first Tape implementation behind executable gates now, then add
vat/meter/guard EC inventories as the service grows beyond local replay smoke.
Gate Inventory:
- apps/tape/tests/cli_contract.rs
- pending: apps/tape/vat.toml
- pending: apps/tape/meter-tape-replay.toml
- pending: apps/tape/guard-tape-security.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| crate-smoke-gate | epic | #768 | partial | passing | smoke | cargo test -p tape |

### Kubernetes-Native Deployment

ID: kubernetes-native-deployment
Type: Devops
Root WI: #768
Status: confirmed
Surfaces: K8s: dedicated StatefulSet/operator topology for topic partitions, storage, probes, backups, and PDBs.
EC Dimensions: behavior: pending kustomize/operator render gate - CRD, operator, and instance render; stability: pending kind replay dogfood
Required Verification: smoke, dogfood
Promise:
Tape runs as a dedicated k8s-native replay service with stable identity,
persistent storage, backup policy, and operator-managed lifecycle.
Gate Inventory:
- pending: apps/tape/k8s

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| dedicated-statefulset-operator-topology | epic | #768 | planned | planned | none | pending k8s render/dogfood gates |

### Primary Replicas

ID: primary-replicas
Type: Runtime
Root WI: #768
Status: confirmed
Surfaces: Raft: topic journal state machine over `libs/raft-core` and `libs/raft-host`.
EC Dimensions: stability: pending raft replay failover gate - leader failover without committed event loss
Required Verification: conformance, dogfood
Promise:
Tape replicates committed topic journal state through raft so replay ranges and
checkpoints survive leader failover.
Gate Inventory:
- pending: apps/tape/tests/raft_replay.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| raft-backed-replay-journal | epic | #768 | planned | planned | none | pending raft replay failover gate |
