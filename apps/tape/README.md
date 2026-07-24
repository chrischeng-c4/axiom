# Tape

## Brief

Tape is the topic replay journal in the Axiom service stack.

It owns append-only topic history, replay by offset or timestamp, consumer
checkpoints, retention windows, and backfill/audit workflows. It is intentionally
separate from `relay`: Relay is the online broker and delivery surface; Tape is
the durable replay/archive surface that long-running systems can re-read after
the fact.

Tape combines domain-owned topic/subscription semantics with the shared service
and Raft libraries: durable append/replay/checkpoints, pull subscriptions and
explicit ack, HTTP/1.1+h2c/OpenAPI, authenticated replicas, operator/PVC
recovery, backup/bootstrap, observability, security, and bounded
soak/competitor gates. Retention/backfill policy remains the explicit Tape
domain work root rather than a shared-service concern.

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
| Topic Replay Journal | #768 | implemented | verified | conformance | ready | durable append and offset/time replay through local and Raft-committed HTTP/CLI paths |
| Consumer Checkpoints | #768 | implemented | verified | smoke | ready | local durable consumer cursor and stale-write rejection |
| Subscription Delivery Resources | #1254, #1255 | implemented | verified | smoke | ready | Topic-to-N named pull subscriptions use bounded checkpoint cursors plus explicit ack; no push, lease, or consumer-group surface |
| Retention And Backfill | #768 | implemented | passing | conformance | ready | Raft-committed offset/age retention with protected consumers, stable sequence space, and bounded offset/timestamp backfill reads |
| HTTP/2 API List | #768 | implemented | verified | smoke | ready | offline `tape spec` route/OpenAPI inventory plus a real h2c + HTTP/1.1 server (#1325) serving `/topics` append/replay/checkpoint; `GET /admin/backup` + `tape backup`/`tape spec gen` client codegen (#1329) |
| Standard Operational Endpoints | #768 | implemented | verified | smoke | ready | `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs` served for real via `libs/service-http` (#1325), with drain-aware readiness and `tape serve` |
| Kubernetes-Native Deployment | #768, #1590 | implemented | verified | dogfood | ready | shared layered Kustomize base/overlays plus CRD/operator/instance and disposable Kind proof of operator reconciliation, PVC-backed append/replay, and replay after pod replacement; multi-shard topology remains domain work |
| Stateful Service Workload | #1554 | implemented | verified | smoke | ready | shared stateful-storage baseline composed from Tape's journal, raft, snapshot/backup, security boundary, and StatefulSet evidence; no duplicated runtime contract |
| Observability | #1588 | implemented | verified | conformance | ready | shared Prometheus pull metrics plus optional ServiceMonitor/PrometheusRule and opt-in OTLP tracing through `libs/service-http` |
| Backup & Restore | #1585 | implemented | passing | conformance | ready | exact snapshots ship through `libs/service-backup`; cold recovery seeds only a fresh PVC before Raft catch-up, never a live in-place restore |
| Replica Sync & Bootstrap | #1327, #1585 | implemented | passing | conformance | ready | Raft owns live replica synchronization; `TAPE_BOOTSTRAP_SEED_URI` seeds only an empty replacement PVC before normal delta catch-up |
| Primary Replicas | #1327, #1805 | implemented | verified | dogfood | ready | shared raft-runtime leader/follower topology, durable restart/Kill-9 recovery, and required peer mTLS on a dedicated authenticated listener |
| CLI Interface | #768 | implemented | verified | smoke | ready | `tape` CLI for local replay/admin, spec, and agent docs |
| CLI Standard Surface | #768 | implemented | verified | smoke | ready | shared `llm`, `upgrade`, and `issue` command groups |
| Chainable Output Conformance | #768 | implemented | verified | smoke | ready | replay/admin commands emit terminal `next:` hints |
| EC Gates Configured | #768, #1330 | implemented | verified | smoke | ready | crate smoke tests + vat/meter/guard EC inventory (vat.toml, meter-tape-performance.toml, guard-tape-security.toml) |
| Long-Running Stability | #768, #1589 | implemented | verified | dogfood | ready | two-cycle Raft failover/recovery plus a bounded HTTP replay/checkpoint soak proving error, RSS, FD, thread/task, and p99 plateaus; retention/compaction stays a Tape domain root |
| Security Hardening | #768, #1593 | implemented | verified | conformance | ready | shared bearer topic authz, audited projected-secret rotation, request admission, restricted pod hardening, and opt-in CNI-enforced NetworkPolicy |
| Competitor Feature Parity | #768 | implemented | verified | smoke | ready | Kafka/Redpanda/Pulsar/JetStream/RabbitMQ Streams replay matrix; feature win only over RabbitMQ topic exchange replay gap |
| Competitor Performance | #768 | implemented | verified | dogfood | ready | release Tape h2c stream passes 1.5x real-service replay gates against NATS JetStream and Kafka KRaft; other peers remain unclaimed |

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
EC Dimensions: stability: `cargo test -p tape --test long_running_stability --test raft_cluster --test raft_failover` - bounded restart and repeated failover recovery; `TAPE_SOAK_AUTOSTART=1 bash apps/tape/scripts/soak.sh` - bounded HTTP replay/checkpoint error, RSS, FD, thread/task, and p99 plateaus. Retention and compaction are separate Tape domain work.
Required Verification: conformance, dogfood
Promise:
Tape's shared service baseline remains stable under bounded replay/checkpoint
load and restart/failover cycles without losing committed events or corrupting
checkpoints. Retention and compaction remain separate Tape domain work.
The 2026-07-17 default 60-second run completed 5,235 fixed-state operations
with zero errors, RSS 14,464 -> 14,464 KiB, FD 13 -> 13, threads 11 -> 11,
and replay p99 1 -> 1 ms.
Gate Inventory:
- apps/tape/tests/long_running_stability.rs
- apps/tape/tests/{raft_cluster,raft_failover}.rs
- apps/tape/scripts/soak.sh
- libs/service-observability/scripts/soak-metrics.sh

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| repeated-raft-restart-endurance | change | #1589 | implemented | passing | conformance | apps/tape/tests/long_running_stability.rs |
| bounded-http-replay-soak | change | #1589 | implemented | passing | dogfood | `TAPE_SOAK_AUTOSTART=1 bash apps/tape/scripts/soak.sh` |

### Security Hardening

ID: security-hardening
Type: Devops
Root WI: #768
Status: confirmed
Surfaces: HTTP/K8s: shared bearer producer/consumer authn/authz and topic isolation; audited projected-secret rotation; opt-in request admission; ingress NetworkPolicy for Tape server pods; non-root contexts and read-only Secret projection.
EC Dimensions: behavior: `cargo test -p tape --test service_auth --test service_admission --test audit_contract --test network_policy_assets` proves bearer topic-role enforcement, atomic credential rotation, bounded write admission, redacted management audit, and the static ingress boundary; a NetworkPolicy-capable CNI is required for cluster enforcement.
Required Verification: negative, conformance
Promise:
Tape protects topic replay data with shared bearer producer/consumer
authorization, audited projected-secret rotation, bounded write admission, and
an opt-in NetworkPolicy that permits the public server port only from explicit
client or Prometheus namespaces. The policy complements rather than replaces
HTTP authorization and is enforced only by a NetworkPolicy-capable CNI. It does
not claim Lumen's search/collection RBAC.
Gate Inventory:
- apps/tape/tests/service_auth.rs
- apps/tape/tests/service_admission.rs
- apps/tape/tests/audit_contract.rs
- apps/tape/tests/network_policy_assets.rs
- apps/tape/k8s/components/network-policy

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| topic-replay-security-boundary | epic | #768 | implemented | passing | conformance | shared bearer role map, audited live registry rotation, bounded write admission, redacted backup audit, and non-root operator workload |
| opt-in-server-ingress-network-policy | change | #1593 | implemented | passing | conformance | apps/tape/k8s/components/network-policy<br>apps/tape/tests/network_policy_assets.rs |

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
Surfaces: CLI/Test: `tape-bench run`, local append/replay throughput proxy, p50/p95 append latency, full replay latency, checkpoint write latency, real NATS JetStream replay comparison, real Kafka (KRaft) replay comparison, and explicit peer calibration ledger.
EC Dimensions: efficiency: `cargo test --release -p tape --test tape_perf_gate -- --nocapture`; `cargo test --release -p tape --test tape_vs_nats_jetstream -- --ignored --nocapture`; `cargo test --release -p tape --test tape_vs_kafka -- --ignored --nocapture` - local regression budget plus real-service h2c JetStream and Kafka replay gates
Required Verification: smoke, conformance
Promise:
Tape maintains a local replay performance regression gate and executable
real-service competitor benchmarks. For the current local backlog full-replay
workload, release-mode Tape serves a compact, read-only h2c replay stream while
real NATS JetStream and Kafka services replay the same 20,000-event,
128-byte-payload durable backlog across their own network clients. Both gates
require at least 1.5x. On the latest 2026-07-18 five-sample run, NATS p50
measured 2.02x (13,586 us versus 27,384 us), while pinned single-node Kafka KRaft
(`apache/kafka:3.9.0`) measured 4.07x (13,575 us versus 55,243 us). Redpanda,
Pulsar, and RabbitMQ
Streams performance wins remain unclaimed until their own real-service gates
exist; RabbitMQ topic exchange remains routing-only and is not a replay
performance baseline.
Gate Inventory:
- apps/tape/src/bench.rs
- apps/tape/src/bin/tape-bench.rs
- apps/tape/tests/tape_perf_gate.rs
- apps/tape/tests/tape_vs_nats_jetstream.rs
- apps/tape/tests/tape_vs_kafka.rs
- apps/tape/external-contracts/competitor-performance/efficiency/competitive-benchmark.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| topic-replay-competitor-performance-baseline | epic | #768 | implemented | passing | smoke | apps/tape/tests/tape_perf_gate.rs<br>apps/tape/tests/tape_vs_nats_jetstream.rs<br>apps/tape/tests/tape_vs_kafka.rs |

### Topic Replay Journal

ID: topic-replay-journal
Type: RuntimeTool
Root WI: #768
Status: verified
Surfaces: CLI: `tape append`, `tape replay` - durable append and replay; HTTP: `/topics/{topic}/append`, `/topics/{topic}/replay`, `/topics/{topic}/replay/stream` - JSON replay plus compact read-only h2c bulk replay.
EC Dimensions: behavior: `cargo test -p tape` - append ordering plus replay range smoke
Required Verification: smoke, conformance
Promise:
Tape provides a durable append-only topic journal for replay/backfill workloads
without becoming the online broker. When `limit` is omitted, replay returns
at most 1000 oldest-first events; page with offset+limit.
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

### Subscription Delivery Resources

ID: subscription-delivery-resources
Type: RuntimeTool
Root WI: #1254, #1255
Status: verified
Surfaces: CLI: `tape subscription create|list|show|pull|ack|delete`; API inventory: `/topics/{topic}/subscriptions`, `/topics/{topic}/subscriptions/{subscription}/pull`, and `/topics/{topic}/subscriptions/{subscription}/ack`.
EC Dimensions: behavior: `cargo test -p tape --test cli_contract` - local pull-only resource and spec inventory contract, including negative proof that mode flags/push are absent
Required Verification: smoke, conformance
Promise:
Tape exposes named topic delivery resources without becoming Relay: bounded
pull reads use the durable `topic/name` checkpoint as their next-offset cursor,
return at most the caller's requested (maximum 1000) window, and require an
explicit ack to advance it. This is Tape's high-QPS pull/replay comparison
path. Subscription creation is intrinsically pull-only: Tape exposes no push,
consumer-group, lease, or bidirectional consume surface. Cursor mutations use
the existing committed checkpoint path rather than executor ownership state.
`ack` accepts any monotonic in-range offset without verifying it was pulled;
consumer libraries must self-enforce pull-then-ack.
Gate Inventory:
- apps/tape/src/lib.rs
- apps/tape/src/bin/tape.rs
- apps/tape/src/spec.rs
- apps/tape/tests/cli_contract.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| topic-subscription-resource-contract | change | #1254 | implemented | passing | smoke | apps/tape/tests/cli_contract.rs |
| pull-subscription-cursor-contract | change | #1255 | implemented | passing | smoke | apps/tape/src/lib.rs<br>apps/tape/tests/cli_contract.rs |

### Retention And Backfill

ID: retention-and-backfill
Type: RuntimeTool
Root WI: #768
Status: confirmed
Surfaces: HTTP: `GET|PUT /topics/{topic}/retention` plus bounded replay by offset or timestamp; Raft: committed `RetentionPut` transition.
EC Dimensions: behavior: `cargo test -p tape --test retention_backfill` plus journal and three-node Raft retention coverage - window trimming, protected checkpoints, stable offsets, and consistent backfill reads
Required Verification: smoke, conformance
Promise:
Tape applies retention and compaction policies while preserving explicitly
protected replay windows and producing consistent backfill exports.
Gate Inventory:
- apps/tape/tests/retention_backfill.rs
- apps/tape/src/lib.rs; apps/tape/src/raft.rs; apps/tape/tests/raft_cluster.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| retention-window-and-backfill-contract | epic | #768 | implemented | passing | conformance | protected compaction and offset/timestamp backfill over local and Raft paths |

### HTTP/2 API List

ID: http2-api-list
Type: RuntimeTool
Root WI: #768
Status: verified
Surfaces: CLI: `tape spec --format routes|openapi|openapi-yaml|json-schema`, `tape spec gen --lang ts|py|rust --out <dir>`, `tape serve`, `tape backup --url --dest --token --retention-secs` (feature `backup`); HTTP: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs`, topic append/replay/checkpoint routes served for real over h2c + HTTP/1.1 on one port, plus admin-gated `GET /admin/backup` streaming a whole-journal snapshot.
EC Dimensions: behavior: `cargo test -p tape --test cli_contract spec_routes_list_topic_contract -- --exact` - offline route inventory; `cargo test -p tape --test http_transport` - real h2c+HTTP/1.1 transport, drain-aware readiness, and per-op metrics; `cargo test -p tape --features backup --test backup` - live admin-gated snapshot endpoint + `tape backup` fetch/ship/retention round trip
Required Verification: smoke, conformance
Promise:
Tape exposes a compact h2c/OpenAPI API list for producer, replay, checkpoint,
and operator workflows, and serves it for real on one h2c + HTTP/1.1 port via
`tape serve` (shared `libs/service-http` shell). `tape spec gen` generates
typed ts/py/rust clients from tape's own OpenAPI document via the shared
`libs/openapi-codegen` crate (`apps/tape/clients/` scaffold), and `GET
/admin/backup` + `tape backup` (feature `backup`) ship a consistent
whole-journal snapshot to a `libs/service-backup` destination sink.
Gate Inventory:
- apps/tape/src/spec.rs
- apps/tape/tests/cli_contract.rs
- apps/tape/src/server.rs
- apps/tape/src/openapi.rs
- apps/tape/tests/http_transport.rs
- apps/tape/src/backup.rs
- apps/tape/tests/backup.rs
- apps/tape/clients/

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| h2c-openapi-route-list | epic | #768 | implemented | passing | smoke | apps/tape/src/spec.rs<br>apps/tape/tests/cli_contract.rs |
| service-http-shell-h2c-serve-standard-endpoints | change | #1325 | implemented | passing | smoke | apps/tape/src/server.rs<br>apps/tape/src/openapi.rs<br>apps/tape/tests/http_transport.rs |
| backup-service-tls-spec-gen-clients | change | #1329 | implemented | passing | smoke | apps/tape/src/backup.rs<br>apps/tape/src/server.rs<br>apps/tape/src/bin/tape.rs<br>apps/tape/clients/<br>apps/tape/tests/backup.rs |

### Standard Operational Endpoints

ID: standard-operational-endpoints
Type: Service
Root WI: #768
Status: verified
Surfaces: CLI: `tape spec --format routes`, `tape serve [--bind] [--store] [--grace-secs]`; HTTP: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs` served for real via the shared `libs/service-http` shell, with SIGTERM-aware graceful drain.
EC Dimensions: behavior: `cargo test -p tape --test cli_contract spec_routes_list_topic_contract -- --exact` - offline standard endpoint inventory; `cargo test -p tape --test http_transport` - real probe surface, drain-aware `/readyz`, and Prometheus `/metrics`
Required Verification: smoke
Promise:
Serve the standard service endpoint contract for real over one h2c + HTTP/1.1
port, with drain-aware readiness and per-op request metrics.
Gate Inventory:
- apps/tape/src/spec.rs
- apps/tape/tests/cli_contract.rs
- apps/tape/src/server.rs
- apps/tape/src/metrics.rs
- apps/tape/src/bin/tape.rs
- apps/tape/tests/http_transport.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| standard-service-route-inventory | epic | #768 | implemented | passing | smoke | apps/tape/src/spec.rs<br>apps/tape/tests/cli_contract.rs |
| service-http-shell-h2c-serve-standard-endpoints | change | #1325 | implemented | passing | smoke | apps/tape/src/server.rs<br>apps/tape/src/metrics.rs<br>apps/tape/src/bin/tape.rs<br>apps/tape/tests/http_transport.rs |

### Observability

ID: observability
Type: Devops
Root WI: #1588
Status: confirmed
Surfaces: HTTP: `/metrics` from shared `service-metrics`; K8s: optional
ServiceMonitor and PrometheusRule component; Logs: structured
`axiom.service.log.v1` stdout with per-request trace correlation — the shared
`service-http` trace layer accepts a valid W3C version-00 `traceparent`
(invalid input is treated as absent) and generates a fresh local root context
otherwise, so every request span and log line carries
`trace_id`/`span_id`/`parent_span_id`/`trace_flags`.
EC Dimensions: behavior: `cargo test -p tape --test observability_assets` -
offline manifest and metric-name conformance.
Required Verification: conformance
Promise:
Tape exports bounded pull metrics and provides an optional Prometheus Operator
bundle that preserves `app`/`role` labels and alerts on actual append/replay
latency series and pod restart loops. Every HTTP request is correlatable end
to end: W3C `traceparent` is honored when present and a local root trace is
created when absent, with the ids flowing into the structured stdout the sift
collector ingests. OTLP export and service identity are provided by the
shared observability/service HTTP libraries.
Gate Inventory:
- apps/tape/src/metrics.rs
- apps/tape/k8s/components/observability
- apps/tape/tests/observability_assets.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| prometheus-operator-scrape-alert-component | change | #1588 | implemented | passing | conformance | apps/tape/k8s/components/observability<br>apps/tape/tests/observability_assets.rs |

### EC Gates Configured

ID: ec-gates-configured
Type: Devops
Root WI: #768
Status: confirmed
Surfaces: Tests: `cargo test -p tape`; Vat/Meter/Guard gates under `apps/tape/`.
EC Dimensions: behavior: full crate/integration gates; efficiency: meter-owned vat-isolated and real-service replay gates; security: guard plus live auth/admission/mTLS gates; stability: restart/failover, Kind PVC recovery, and bounded 60-second soak
Required Verification: smoke, efficiency, security
Promise:
Keep the first Tape implementation behind executable gates, with vat-isolated
meter/guard EC inventories now wired up alongside the local replay smoke gate.
Gate Inventory:
- apps/tape/tests/cli_contract.rs
- apps/tape/vat.toml
- apps/tape/meter-tape-performance.toml
- apps/tape/guard-tape-security.toml
- apps/tape/external-contracts/competitor-performance/efficiency/meter-gate.md
- apps/tape/external-contracts/security-hardening/security/security-evidence.md
- apps/tape/tests/shared_otlp_tracing.rs
- apps/tape/observability/ (prometheus.yml, otel-collector-config.yaml, grafana-datasources.yaml)
- apps/tape/compose.yaml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| crate-smoke-gate | epic | #768 | implemented | passing | conformance | cargo test -p tape |
| tape-vat-meter-guard-ec-gates-observability | change | #1330 | implemented | passing | smoke | apps/tape/vat.toml, apps/tape/meter-tape-performance.toml, apps/tape/guard-tape-security.toml |
| shared-otlp-trace-export | change | #1662 | implemented | passing | conformance | `cargo test -p tape --test shared_otlp_tracing` |

### Kubernetes-Native Deployment

ID: kubernetes-native-deployment
Type: Devops
Root WI: #768
Status: confirmed
Surfaces: K8s: dedicated StatefulSet/operator topology for topic partitions, storage, probes, and PDBs (#1328); `tape k8s crd|operator|instance render`, `tape k8s operator run` (behind the `operator` cargo feature), and `tape dockerfile render --variant source|release`.
EC Dimensions: behavior: offline render/CLI gates (`tests/deploy_cli.rs`, `tests/operator.rs`) - CRD structural-schema safety, operator render shape, instance profiles, dockerfile fixture parity; stability: `bash apps/tape/scripts/kind-e2e.sh` builds the real image, creates a disposable Kind cluster, and proves append/replay survives one single-node StatefulSet pod replacement with its PVC retained.
Required Verification: smoke, dogfood
Promise:
Tape runs as a dedicated k8s-native replay service with stable identity,
persistent storage, and operator-managed lifecycle. The bounded Kind dogfood
gate covers one single-node replacement; multi-shard and long-running soak
remain separate work roots.
Gate Inventory:
- apps/tape/k8s/operator/{crd,rbac,deployment}.yaml
- apps/tape/tests/deploy_cli.rs
- apps/tape/tests/operator.rs
- apps/tape/scripts/kind-e2e.sh

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| dedicated-statefulset-operator-topology | epic | #768 | implemented | verified | smoke | apps/tape/tests/{deploy_cli,operator}.rs; #1328 |
| operator-kind-pvc-restart-replay | change | #1590 | implemented | passing | dogfood | apps/tape/scripts/kind-e2e.sh |

### Stateful Service Workload

ID: stateful-service-workload
Type: Service
Root WI: #1554
Status: verified
Surfaces: Durable journal state plus stateful deployment: `apps/tape/src/lib.rs`,
`libs/raft-core`, `libs/raft-host`, `apps/tape/src/backup.rs`, and the dedicated
StatefulSet/operator rendering surface under `apps/tape/k8s/`.
EC Dimensions: behavior: `aw capability check --project tape --skip-issue-inventory` -
the `stateful_storage` profile resolves its shared baseline; stability: raft
failover/restart, backup snapshot, Kind dogfood, authenticated peer mTLS, and
the shared security boundary remain authoritative in their linked capability
roots.
Required Verification: smoke
Promise:
Tape projects the shared stateful-service workload baseline without a duplicate
service implementation. Its durable append log, stable identity/PVC lifecycle,
raft primary-replica recovery, snapshot/backup path, deployment artifacts, and
security boundary are owned by the linked capability roots below; domain
retention/backfill and subscription behavior remain separately verified roots.
Gate Inventory:
- `aw capability check --project tape --skip-issue-inventory`
- apps/tape/tests/{raft_cluster,raft_failover,raft_persistence}.rs
- apps/tape/tests/{backup,deploy_cli,operator}.rs
- apps/tape/k8s/operator/{crd,rbac,deployment}.yaml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| stateful-service-workload-projection | change | #1554 | implemented | passing | smoke | `aw capability check --project tape --skip-issue-inventory`; composes Topic Replay Journal, Primary Replicas, HTTP/2 API List, Kubernetes-Native Deployment, and Security Hardening without duplicating their claims |

### Backup & Restore

ID: backup-restore
Type: Service
Root WI: #1585
Status: confirmed
Surfaces: Admin HTTP: `GET /admin/backup` yields the exact whole-journal
`JournalSnapshot`; CLI: `tape backup` ships those bytes through
`libs/service-backup`; Runtime: `tape serve --bootstrap-seed-uri` restores only
into a fresh replica PVC before Raft starts.
EC Dimensions: behavior: `cargo test -p tape --test backup --test bootstrap` -
snapshot transport and cold-seed conformance.
Required Verification: conformance
Promise:
Tape writes no second backup format. A backup object is the same
`JournalSnapshot` bytes that the state machine snapshots and restores. A
disaster-recovery seed is deliberately cold and destructive only to an *empty*
PVC: Tape validates the object, atomically prepares the per-node state-machine
snapshot and applied floor, then lets normal Raft log/snapshot catch-up resume.
There is no online `POST /admin/restore` that can overwrite a live leader or
follower.
Gate Inventory:
- apps/tape/src/{backup,raft}.rs
- libs/service-backup/src/source.rs
- apps/tape/tests/{backup,bootstrap}.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| exact-journal-snapshot-backup | epic | #1329 | implemented | passing | conformance | apps/tape/src/backup.rs<br>apps/tape/tests/backup.rs |
| fresh-pvc-cold-recovery-seed | change | #1585 | implemented | passing | conformance | apps/tape/src/raft.rs<br>apps/tape/tests/bootstrap.rs |

### Replica Sync & Bootstrap

ID: replica-sync-bootstrap
Type: Service
Root WI: #1327, #1585
Status: confirmed
Surfaces: RaftHost: leader forwarding, committed apply, InstallSnapshot, and
follower catch-up; Backup seed: exact `file://`, `s3://`, or `gs://` (workload-identity
ADC in-cluster, GKE-proven) object read through `libs/service-backup` before an
empty PVC joins the group.
EC Dimensions: behavior: `cargo test -p tape --test raft_cluster --test
bootstrap` - live replica convergence and seed-before-catch-up conformance;
stability: `cargo test -p tape --test raft_failover --test raft_persistence` -
kill-9 failover and restart recovery.
Required Verification: conformance, dogfood
Promise:
Existing PVCs recover their local Raft state and synchronise through
`raft-host`; a replacement with no local state may load one exact external
snapshot before it catches up. Backup artifacts are a cold seed or DR path,
not a substitute for ordinary live replication, leader forwarding, or
InstallSnapshot.
Gate Inventory:
- apps/tape/src/{raft,bin/tape}.rs
- apps/tape/tests/{raft_cluster,raft_failover,raft_persistence,bootstrap}.rs
- libs/raft-host
- libs/service-backup/src/source.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| raft-log-existing-pvc-sync | epic | #1327 | implemented | passing | conformance | apps/tape/tests/raft_cluster.rs<br>apps/tape/tests/raft_persistence.rs |
| empty-pvc-external-backup-seed | change | #1585 | implemented | passing | conformance | apps/tape/src/raft.rs<br>apps/tape/tests/bootstrap.rs |

### Primary Replicas

ID: primary-replicas
Type: Runtime
Root WI: #768, #1327
Status: confirmed
Surfaces: Raft: topic journal state machine over `libs/raft-core` and `libs/raft-runtime`'s `TapeRaft`/`TapeStateMachine` (#1327); auto-mode leader/follower topology activated by `REPLICAS_PER_SHARD>1` (plus the standard `POD_NAME`/`SHARD_COUNT`/`VOTER_COUNT` downward-API quartet) — no tape-specific `--raft` flag. Required peer mTLS (`TAPE_PEER_TLS_CERT`/`_KEY`/`_CA`, `TAPE_PEER_MTLS`) uses the shared `raft-runtime` transport on the dedicated raft listener.
EC Dimensions: behavior: real 3-node in-process raft group - election, leader-applied writes replicate to followers, follower-received appends forward to the leader, direct follower peer-route POST answers 421, recovered-node catch-up followed by a second leader loss, and fresh-node catch-up via InstallSnapshot; required-mTLS peers replicate over the authenticated listener and an untrusted certificate never reaches the Raft router; stability: live 3-node `kill -9` leader failover with no committed event loss and restart-recovery of the durable applied-index floor across process restarts.
Required Verification: conformance, dogfood
Promise:
Tape replicates committed topic journal state through raft so replay ranges and
checkpoints survive leader failover. Required peer mTLS is shared transport
infrastructure, selected by Tape rather than reimplemented in the service.
Gate Inventory:
- apps/tape/tests/raft_cluster.rs
- apps/tape/tests/raft_failover.rs
- apps/tape/tests/raft_persistence.rs
- apps/tape/tests/raft_peer_mtls.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| raft-backed-replay-journal | epic | #768, #1805 | implemented | passing | dogfood | apps/tape/tests/{raft_cluster,raft_failover,raft_persistence,raft_peer_mtls}.rs prove election/replication/failover/restart-recovery and trusted-peer mTLS |
