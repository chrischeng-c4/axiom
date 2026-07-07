# Tape

## Brief

Tape is the topic replay journal in the Axiom service stack.

It owns append-only topic history, replay by offset or timestamp, consumer
checkpoints, retention windows, and backfill/audit workflows. It is intentionally
separate from `relay`: Relay is the online broker and delivery surface; Tape is
the durable replay/archive surface that long-running systems can re-read after
the fact.

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
| Topic Replay Journal | #768 | planned | planned | none | not_ready | append-only topic log with offset/time replay |
| Consumer Checkpoints | #768 | planned | planned | none | not_ready | durable consumer cursor and replay resume state |
| Retention And Backfill | #768 | planned | planned | none | not_ready | retention windows, compaction policy, and batch backfill |
| HTTP/2 API List | #768 | planned | planned | none | not_ready | h2c/OpenAPI endpoint inventory for producers and consumers |
| Kubernetes-Native Deployment | #768 | planned | planned | none | not_ready | dedicated StatefulSet/operator shape |
| Primary Replicas | #768 | planned | planned | none | not_ready | raft-backed replicated topic journal |
| CLI Interface | #768 | planned | planned | none | not_ready | `tape` CLI for replay/admin and agent docs |
| Long-Running Stability | #768 | planned | planned | none | not_ready | soak, retention, compaction, and replay recovery gates |
| Security Hardening | #768 | planned | planned | none | not_ready | producer/consumer authz, tenant isolation, audit, and secret rotation |
| Competitor Feature Parity | #768 | planned | planned | none | not_ready | Kafka/Pulsar/JetStream-style replay feature matrix |
| Competitor Performance | #768 | planned | planned | none | not_ready | pinned replay throughput/latency baseline, rerun only on scope change |

### CLI Interface

ID: cli-interface
Type: RuntimeTool
Root WI: #768
Status: confirmed
Surfaces: CLI: `tape llm`, `tape upgrade`, `tape issue`, topic append/replay/checkpoint, and admin/debug verbs.
EC Dimensions: behavior: pending CLI convention gate - required standard verbs, replay ergonomics, and offline agent docs
Required Verification: smoke, conformance
Promise:
Tape ships an agent-drivable CLI for replay, checkpoint, and admin workflows
while following the repository-wide CLI convention.
Gate Inventory:
- pending: projects/tape/tests/cli_contract.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| tape-cli-convention-and-replay-verbs | epic | #768 | planned | planned | none | pending CLI convention gate |

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
- pending: projects/tape/tests/long_running_stability.rs

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
- pending: projects/tape/tests/security_hardening.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| topic-replay-security-boundary | epic | #768 | planned | planned | none | pending security hardening gate |

### Competitor Feature Parity

ID: competitor-feature-parity
Type: RuntimeTool
Root WI: #768
Status: confirmed
Surfaces: Docs/Test: replay feature matrix against Kafka, Pulsar, and JetStream-style topic replay.
EC Dimensions: behavior: pending competitor feature gate - append, offset replay, timestamp replay, checkpoints, retention, backfill, and operational controls
Required Verification: conformance
Promise:
Tape keeps an explicit replay feature matrix against established topic replay
systems, with comparison scope changed only when product requirements change.
Gate Inventory:
- pending: projects/tape/benchmark/competitor-feature-matrix.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| topic-replay-competitor-feature-matrix | epic | #768 | planned | planned | none | pending competitor feature gate |

### Competitor Performance

ID: competitor-performance
Type: RuntimeTool
Root WI: #768
Status: confirmed
Surfaces: Meter/Vat: append/replay throughput, p50/p95 replay latency, checkpoint write latency, and retention/backfill cost.
EC Dimensions: efficiency: pending competitor performance gate - pinned external baseline and Lumen-owned replay measurements
Required Verification: dogfood
Promise:
Tape maintains a pinned competitor performance baseline and reruns external
benchmarks only when the comparison scope changes or a human explicitly asks.
Gate Inventory:
- pending: projects/tape/benchmark/competitor-performance-baseline.md
- pending: projects/tape/meter-tape-replay.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| topic-replay-competitor-performance-baseline | epic | #768 | planned | planned | none | pending competitor performance gate |

### Topic Replay Journal

ID: topic-replay-journal
Type: RuntimeTool
Root WI: #768
Status: confirmed
Surfaces: HTTP: `/v1/topics/{topic}/append`, `/v1/topics/{topic}/replay` - append and replay event envelopes by offset or timestamp.
EC Dimensions: behavior: pending topic replay conformance gate - append ordering, replay ranges, and idempotent producer behavior
Required Verification: smoke, conformance
Promise:
Tape provides a durable append-only topic journal for replay/backfill workloads
without becoming the online broker.
Gate Inventory:
- pending: projects/tape/tests/topic_replay.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| append-and-replay-contract | epic | #768 | planned | planned | none | pending topic replay conformance gate |

### Consumer Checkpoints

ID: consumer-checkpoints
Type: RuntimeTool
Root WI: #768
Status: confirmed
Surfaces: HTTP: `/v1/topics/{topic}/consumers/{consumer}/checkpoint` - durable replay cursor management.
EC Dimensions: behavior: pending checkpoint conformance gate - checkpoint create, advance, resume, and stale-write rejection
Required Verification: smoke, conformance
Promise:
Tape persists consumer replay positions so backfills and long-running consumers
resume deterministically after restart or handoff.
Gate Inventory:
- pending: projects/tape/tests/consumer_checkpoints.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| durable-consumer-cursor-contract | epic | #768 | planned | planned | none | pending checkpoint conformance gate |

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
- pending: projects/tape/tests/retention_backfill.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| retention-window-and-backfill-contract | epic | #768 | planned | planned | none | pending retention/backfill gate |

### HTTP/2 API List

ID: http2-api-list
Type: RuntimeTool
Root WI: #768
Status: confirmed
Surfaces: HTTP: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs`, topic append/replay/checkpoint routes.
EC Dimensions: behavior: pending h2c/OpenAPI route-list gate - probes, metrics, OpenAPI, and route inventory
Required Verification: smoke, conformance
Promise:
Tape exposes a compact h2c/OpenAPI API list for producer, replay, checkpoint,
and operator workflows.
Gate Inventory:
- pending: projects/tape/tests/http_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| h2c-openapi-route-list | epic | #768 | planned | planned | none | pending h2c/OpenAPI route-list gate |

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
- pending: projects/tape/k8s

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
- pending: projects/tape/tests/raft_replay.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| raft-backed-replay-journal | epic | #768 | planned | planned | none | pending raft replay failover gate |
