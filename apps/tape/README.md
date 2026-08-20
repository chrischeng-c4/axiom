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

A promise with no gate under it is not claimed.

### Capability Index

| Capability | Root WI | Notes |
|---|---:|---|
| Topic Replay Journal | #768 | durable append and offset/time replay through local and Raft-committed HTTP/CLI paths |
| Consumer Checkpoints | #768 | local durable consumer cursor and stale-write rejection |
| Subscription Delivery Resources | #1254, #1255 | Topic-to-N named pull subscriptions use bounded checkpoint cursors plus explicit ack; no push, lease, or consumer-group surface |
| Retention And Backfill | #768 | Raft-committed offset/age retention with protected consumers, stable sequence space, and bounded offset/timestamp backfill reads |
| HTTP/2 API List | #768 | offline `tape spec` route/OpenAPI inventory plus a real h2c + HTTP/1.1 server (#1325) serving `/topics` append/replay/checkpoint; `GET /admin/backup` + `tape backup`/`tape spec gen` client codegen (#1329) |
| Standard Operational Endpoints | #768 | `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs` served for real via `libs/service-http` (#1325), with drain-aware readiness and `tape serve` |
| Kubernetes-Native Deployment | #768, #1590 | shared layered Kustomize base/overlays plus CRD/operator/instance and disposable Kind proof of operator reconciliation, PVC-backed append/replay, and replay after pod replacement; multi-shard topology remains domain work |
| Stateful Service Workload | #1554 | shared stateful-storage baseline composed from Tape's journal, raft, snapshot/backup, security boundary, and StatefulSet evidence; no duplicated runtime contract |
| Observability | #1588 | shared Prometheus pull metrics plus optional ServiceMonitor/PrometheusRule and opt-in OTLP tracing through `libs/service-http` |
| Backup & Restore | #1585 | exact snapshots ship through `libs/service-backup`; cold recovery seeds only a fresh PVC before Raft catch-up, never a live in-place restore |
| Replica Sync & Bootstrap | #1327, #1585 | Raft owns live replica synchronization; `TAPE_BOOTSTRAP_SEED_URI` seeds only an empty replacement PVC before normal delta catch-up |
| Primary Replicas | #1327, #1805 | shared raft-runtime leader/follower topology, durable restart/Kill-9 recovery, and required peer mTLS on a dedicated authenticated listener |
| CLI Interface | #768 | `tape` CLI for local replay/admin, spec, and agent docs |
| CLI Standard Surface | #768 | shared `llm`, `upgrade`, and `issue` command groups |
| Chainable Output Conformance | #768 | replay/admin commands emit terminal `next:` hints |
| EC Gates Configured | #768, #1330 | crate smoke tests + vat/meter/guard EC inventory (vat.toml, meter-tape-performance.toml, guard-tape-security.toml) |
| Long-Running Stability | #768, #1589 | two-cycle Raft failover/recovery plus a bounded HTTP replay/checkpoint soak proving error, RSS, FD, thread/task, and p99 plateaus; retention/compaction stays a Tape domain root |
| Security Hardening | #768, #1593 | shared bearer topic authz, audited projected-secret rotation, request admission, restricted pod hardening, and opt-in CNI-enforced NetworkPolicy |
| Competitor Feature Parity | #768 | Kafka/Redpanda/Pulsar/JetStream/RabbitMQ Streams replay matrix; feature win only over RabbitMQ topic exchange replay gap |
| Competitor Performance | #768 | release Tape h2c stream passes 1.5x real-service replay gates against NATS JetStream and Kafka KRaft; other peers remain unclaimed |

### CLI Interface

Tape ships an agent-drivable CLI for replay, checkpoint, and admin workflows
while following the repository-wide CLI convention.

- Root WI: #768
- Surfaces: CLI: `tape append`, `tape replay`, `tape checkpoint`, `tape spec`,
  `tape llm`, `tape upgrade`, and `tape issue`.
- Gate — behavior: `cargo test -p tape --test cli_contract` - command surface
  and local replay ergonomics
- Source: `apps/tape/tests/cli_contract.rs`, `apps/tape/src/bin/tape.rs`
- Evidence: apps/tape/tests/cli_contract.rs

### CLI Standard Surface

Ship the mandatory shared `cli-std` surface every ecosystem CLI owes without
mixing it into Tape-specific append/replay/checkpoint commands.

- Root WI: #768
- Surfaces: CLI: `tape llm`, `tape upgrade`, `tape issue search`,
  `tape issue view`, `tape issue create`, and `tape issue comment`.
- Gate — behavior: `cargo test -p tape --test cli_contract` - top-level help
  keeps shared command groups visible
- Source: `apps/tape/tests/cli_contract.rs`, `libs/cli-std/src/llm.rs`,
  `libs/cli-std/src/upgrade.rs`, `libs/cli-std/src/issue.rs`
- Evidence: apps/tape/tests/cli_contract.rs

### Chainable Output Conformance

Keep Tape's operational CLI outputs simple and chainable while raw spec/LLM
payload commands remain direct streams.

- Root WI: #768
- Surfaces: CLI: `tape append`, `tape replay`, and `tape checkpoint` -
  operational commands that print a runnable follow-up or `next: done`.
- Gate — behavior: `cargo test -p tape --test cli_contract` - local
  append/replay/checkpoint roundtrip
- Source: `apps/tape/tests/cli_contract.rs`
- Evidence: apps/tape/tests/cli_contract.rs

### Long-Running Stability

Tape's shared service baseline remains stable under bounded replay/checkpoint
load and restart/failover cycles without losing committed events or corrupting
checkpoints. Retention and compaction remain separate Tape domain work. The
2026-07-17 default 60-second run completed 5,235 fixed-state operations with
zero errors, RSS 14,464 -> 14,464 KiB, FD 13 -> 13, threads 11 -> 11, and
replay p99 1 -> 1 ms.

- Root WI: #768
- Surfaces: Runtime: append log, replay readers, retention/compaction workers,
  checkpoint store, snapshot, and recovery paths.
- Gate — stability:
  `cargo test -p tape --test long_running_stability --test raft_cluster --test raft_failover`
  - bounded restart and repeated failover recovery
- Gate: `TAPE_SOAK_AUTOSTART=1 bash apps/tape/scripts/soak.sh` - bounded HTTP
  replay/checkpoint error, RSS, FD, thread/task, and p99 plateaus. Retention
  and compaction are separate Tape domain work.
- Source: `apps/tape/tests/long_running_stability.rs`,
  `apps/tape/tests/{raft_cluster,raft_failover}.rs`,
  `apps/tape/scripts/soak.sh`,
  `libs/service-observability/scripts/soak-metrics.sh`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| repeated-raft-restart-endurance | change | #1589 | apps/tape/tests/long_running_stability.rs |
| bounded-http-replay-soak | change | #1589 | `TAPE_SOAK_AUTOSTART=1 bash apps/tape/scripts/soak.sh` |

### Security Hardening

Tape protects topic replay data with shared bearer producer/consumer
authorization, audited projected-secret rotation, bounded write admission, and
an opt-in NetworkPolicy that permits the public server port only from explicit
client or Prometheus namespaces. The policy complements rather than replaces
HTTP authorization and is enforced only by a NetworkPolicy-capable CNI. It does
not claim Lumen's search/collection RBAC.

- Root WI: #768
- Surfaces: HTTP/K8s: shared bearer producer/consumer authn/authz and topic
  isolation; audited projected-secret rotation; opt-in request admission;
  ingress NetworkPolicy for Tape server pods; non-root contexts and read-only
  Secret projection.
- Gate — behavior:
  `cargo test -p tape --test service_auth --test service_admission --test audit_contract --test network_policy_assets`
  proves bearer topic-role enforcement, atomic credential rotation, bounded
  write admission, redacted management audit, and the static ingress boundary
- Gate: a NetworkPolicy-capable CNI is required for cluster enforcement.
- Source: `apps/tape/tests/service_auth.rs`,
  `apps/tape/tests/service_admission.rs`, `apps/tape/tests/audit_contract.rs`,
  `apps/tape/tests/network_policy_assets.rs`,
  `apps/tape/k8s/components/network-policy`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| topic-replay-security-boundary | epic | #768 | shared bearer role map, audited live registry rotation, bounded write admission, redacted backup audit, and non-root operator workload |
| opt-in-server-ingress-network-policy | change | #1593 | apps/tape/k8s/components/network-policy<br>apps/tape/tests/network_policy_assets.rs |
| closed-auth-mode-enum-defaulting-to-required | change | #2765 | `cargo test -p tape --features operator --test operator`<br>`cargo test -p tape --test deploy_cli`<br>live server-side-dry-run proof of every accept/reject case named in this section (cluster-bound, deliberately not a backtick gate) |

### Competitor Feature Parity

Tape keeps an explicit replay feature matrix against established topic replay
systems. RabbitMQ topic exchange is included as a routing/fanout comparison
row, while RabbitMQ Streams and Kafka-style topic logs remain replay baselines.
Tape claims a replay-feature advantage only over RabbitMQ topic exchange, not
over Kafka, Redpanda, Pulsar, JetStream, or RabbitMQ Streams.

- Root WI: #768
- Surfaces: Docs/Test: replay feature matrix against Kafka, Redpanda, Pulsar,
  NATS JetStream, RabbitMQ Streams, and RabbitMQ topic exchange routing
  semantics.
- Gate — behavior: `cargo test -p tape --test competitor_feature_parity` -
  replay-log functionality, topic-exchange classification, and win/loss claim
  boundaries
- Source:
  `apps/tape/external-contracts/competitor-feature-parity/behavior/topic-exchange-functional.md`,
  `apps/tape/tests/competitor_feature_parity.rs`
- Evidence: apps/tape/tests/competitor_feature_parity.rs

### Competitor Performance

Tape maintains a local replay performance regression gate and executable
real-service competitor benchmarks. For the current local backlog full-replay
workload, release-mode Tape serves a compact, read-only h2c replay stream while
real NATS JetStream and Kafka services replay the same 20,000-event,
128-byte-payload durable backlog across their own network clients. Both gates
require at least 1.5x. On the latest 2026-07-18 five-sample run, NATS p50
measured 2.02x (13,586 us versus 27,384 us), while pinned single-node Kafka
KRaft (`apache/kafka:3.9.0`) measured 4.07x (13,575 us versus 55,243 us).
Redpanda, Pulsar, and RabbitMQ Streams performance wins remain unclaimed until
their own real-service gates exist; RabbitMQ topic exchange remains
routing-only and is not a replay performance baseline.

- Root WI: #768
- Surfaces: CLI/Test: `tape-bench run`, local append/replay throughput proxy,
  p50/p95 append latency, full replay latency, checkpoint write latency, real
  NATS JetStream replay comparison, real Kafka (KRaft) replay comparison, and
  explicit peer calibration ledger.
- Gate — efficiency:
  `cargo test --release -p tape --test tape_perf_gate -- --nocapture`
- Gate:
  `cargo test --release -p tape --test tape_vs_nats_jetstream -- --ignored --nocapture`
- Gate:
  `cargo test --release -p tape --test tape_vs_kafka -- --ignored --nocapture`
  - local regression budget plus real-service h2c JetStream and Kafka replay
  gates
- Source: `apps/tape/src/bench.rs`, `apps/tape/src/bin/tape-bench.rs`,
  `apps/tape/tests/tape_perf_gate.rs`,
  `apps/tape/tests/tape_vs_nats_jetstream.rs`,
  `apps/tape/tests/tape_vs_kafka.rs`,
  `apps/tape/external-contracts/competitor-performance/efficiency/competitive-benchmark.md`
- Evidence: apps/tape/tests/tape_perf_gate.rs,
  apps/tape/tests/tape_vs_nats_jetstream.rs, apps/tape/tests/tape_vs_kafka.rs

### Topic Replay Journal

Tape provides a durable append-only topic journal for replay/backfill workloads
without becoming the online broker. When `limit` is omitted, replay returns at
most 1000 oldest-first events; page with offset+limit.

- Root WI: #768
- Surfaces: CLI: `tape append`, `tape replay` - durable append and replay;
  HTTP: `/topics/{topic}/append`, `/topics/{topic}/replay`,
  `/topics/{topic}/replay/stream` - JSON replay plus compact read-only h2c bulk
  replay.
- Gate — behavior: `cargo test -p tape` - append ordering plus replay range
  smoke
- Gate — behavior:
  `uv run --frozen --offline --project apps/tape/external-contracts python apps/tape/external-contracts/src/runner.py ec-3052-durability`
  - acknowledged appends survive SIGKILL, refused ones stay absent, replay
  applies nothing twice
- Gate — efficiency:
  `uv run --frozen --offline --project apps/tape/external-contracts python apps/tape/external-contracts/src/runner.py ec-3052-scaling`
  - durable append throughput rises with concurrency, bounded below so the
  ratio cannot be won by starving the lone writer and above so it cannot be won
  by skipping the durability barrier (red until #3052 lands)
- Source: `apps/tape/src/lib.rs`, `apps/tape/tests/cli_contract.rs`,
  `apps/tape/external-contracts/src/ec-3052-durability-under-sigkill.py`,
  `apps/tape/external-contracts/src/ec-3052-durable-append-scaling.py`
- Evidence: apps/tape/src/lib.rs, apps/tape/tests/cli_contract.rs

### Consumer Checkpoints

Tape persists consumer replay positions so backfills and long-running consumers
resume deterministically after restart or handoff.

- Root WI: #768
- Surfaces: CLI: `tape checkpoint get|put`; HTTP:
  `/topics/{topic}/consumers/{consumer}/checkpoint` - declared service route
  inventory.
- Gate — behavior: `cargo test -p tape` - checkpoint advance, resume, and
  stale-write rejection smoke
- Source: `apps/tape/src/lib.rs`, `apps/tape/tests/cli_contract.rs`
- Evidence: apps/tape/src/lib.rs, apps/tape/tests/cli_contract.rs

### Subscription Delivery Resources

Tape exposes named topic delivery resources without becoming Relay: bounded
pull reads use the durable `topic/name` checkpoint as their next-offset cursor,
return at most the caller's requested (maximum 1000) window, and require an
explicit ack to advance it. This is Tape's high-QPS pull/replay comparison
path. Subscription creation is intrinsically pull-only: Tape exposes no push,
consumer-group, lease, or bidirectional consume surface. Cursor mutations use
the existing committed checkpoint path rather than executor ownership state.
`ack` accepts any monotonic in-range offset without verifying it was pulled;
consumer libraries must self-enforce pull-then-ack.

- Root WI: #1254, #1255
- Surfaces: CLI: `tape subscription create|list|show|pull|ack|delete`; API
  inventory: `/topics/{topic}/subscriptions`,
  `/topics/{topic}/subscriptions/{subscription}/pull`, and
  `/topics/{topic}/subscriptions/{subscription}/ack`.
- Gate — behavior: `cargo test -p tape --test cli_contract` - local pull-only
  resource and spec inventory contract, including negative proof that mode
  flags/push are absent
- Source: `apps/tape/src/lib.rs`, `apps/tape/src/bin/tape.rs`,
  `apps/tape/src/spec.rs`, `apps/tape/tests/cli_contract.rs`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| topic-subscription-resource-contract | change | #1254 | apps/tape/tests/cli_contract.rs |
| pull-subscription-cursor-contract | change | #1255 | apps/tape/src/lib.rs<br>apps/tape/tests/cli_contract.rs |

### Retention And Backfill

Tape applies retention and compaction policies while preserving explicitly
protected replay windows and producing consistent backfill exports.

- Root WI: #768
- Surfaces: HTTP: `GET|PUT /topics/{topic}/retention` plus bounded replay by
  offset or timestamp; Raft: committed `RetentionPut` transition.
- Gate — behavior: `cargo test -p tape --test retention_backfill` plus journal
  and three-node Raft retention coverage - window trimming, protected
  checkpoints, stable offsets, and consistent backfill reads
- Source: `apps/tape/tests/retention_backfill.rs`, `apps/tape/src/lib.rs`,
  `apps/tape/src/raft.rs`, `apps/tape/tests/raft_cluster.rs`
- Evidence: protected compaction and offset/timestamp backfill over local and
  Raft paths

### HTTP/2 API List

Tape exposes a compact h2c/OpenAPI API list for producer, replay, checkpoint,
and operator workflows, and serves it for real on one h2c + HTTP/1.1 port via
`tape serve` (shared `libs/service-http` shell). `tape spec gen` generates
typed ts/py/rust clients from tape's own OpenAPI document via the shared
`libs/openapi-codegen` crate (`apps/tape/clients/` scaffold), and
`GET /admin/backup` + `tape backup` (feature `backup`) ship a consistent
whole-journal snapshot to a `libs/service-backup` destination sink.

- Root WI: #768
- Surfaces: CLI: `tape spec --format routes|openapi|openapi-yaml|json-schema`,
  `tape spec gen --lang ts|py|rust [--target <profile>] --out <dir>`,
  `tape serve`, `tape backup --url --dest --token --retention-secs` (feature
  `backup`); HTTP: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs`,
  topic append/replay/checkpoint routes served for real over h2c + HTTP/1.1 on
  one port, plus admin-gated `GET /admin/backup` streaming a whole-journal
  snapshot.
- Gate — behavior:
  `cargo test -p tape --test cli_contract`
  - offline route inventory
- Gate: `cargo test -p tape --test http_transport` - real h2c+HTTP/1.1
  transport, drain-aware readiness, and per-op metrics
- Gate: `cargo test -p tape --features backup --test backup` - live admin-gated
  snapshot endpoint + `tape backup` fetch/ship/retention round trip
- Source: `apps/tape/src/spec.rs`, `apps/tape/tests/cli_contract.rs`,
  `apps/tape/src/server.rs`, `apps/tape/src/openapi.rs`,
  `apps/tape/tests/http_transport.rs`, `apps/tape/src/backup.rs`,
  `apps/tape/tests/backup.rs`, `apps/tape/clients/`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| h2c-openapi-route-list | epic | #768 | apps/tape/src/spec.rs<br>apps/tape/tests/cli_contract.rs |
| service-http-shell-h2c-serve-standard-endpoints | change | #1325 | apps/tape/src/server.rs<br>apps/tape/src/openapi.rs<br>apps/tape/tests/http_transport.rs |
| backup-service-tls-spec-gen-clients | change | #1329 | apps/tape/src/backup.rs<br>apps/tape/src/server.rs<br>apps/tape/src/bin/tape.rs<br>apps/tape/clients/<br>apps/tape/tests/backup.rs |

### Standard Operational Endpoints

Serve the standard service endpoint contract for real over one h2c + HTTP/1.1
port, with drain-aware readiness and per-op request metrics.

- Root WI: #768
- Surfaces: CLI: `tape spec --format routes`,
  `tape serve [--bind] [--store] [--grace-secs]`; HTTP: `/healthz`, `/readyz`,
  `/metrics`, `/openapi.json`, `/docs` served for real via the shared
  `libs/service-http` shell, with SIGTERM-aware graceful drain.
- Gate — behavior:
  `cargo test -p tape --test cli_contract --test behavior_tape_claim_standard_operational_endpoints`
  - offline standard endpoint inventory
- Gate: `cargo test -p tape --test http_transport` - real probe surface,
  drain-aware `/readyz`, and Prometheus `/metrics`
- Source: `apps/tape/src/spec.rs`, `apps/tape/tests/cli_contract.rs`,
  `apps/tape/src/server.rs`, `apps/tape/src/metrics.rs`,
  `apps/tape/src/bin/tape.rs`, `apps/tape/tests/http_transport.rs`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| standard-service-route-inventory | epic | #768 | apps/tape/src/spec.rs<br>apps/tape/tests/cli_contract.rs |
| service-http-shell-h2c-serve-standard-endpoints | change | #1325 | apps/tape/src/server.rs<br>apps/tape/src/metrics.rs<br>apps/tape/src/bin/tape.rs<br>apps/tape/tests/http_transport.rs |

### Observability

Tape exports bounded pull metrics (per-op request counts/latency) plus
operational gauges (topic latest offset and subscription lag computed at scrape
time, with label escaping) and provides an optional Prometheus Operator bundle
that preserves `app`/`role` labels and alerts on actual append/replay latency
series, subscription lag growth, and pod restart loops with seed-failure
diagnostics (#2485). Every HTTP request is correlatable end to end: W3C
`traceparent` is honored when present and a local root trace is created when
absent, with the ids flowing into the structured stdout the sift collector
ingests. OTLP export and service identity are provided by the shared
observability/service HTTP libraries. Server-Timing per-response latency
attribution (the shared `service-http::server_timing` contract) is wired into
tape's HTTP stack: every response carries a `Server-Timing: app;dur=<ms>`
baseline (#2490).

- Root WI: #1588
- Surfaces: HTTP: `/metrics` from shared `service-metrics`; K8s: optional
  ServiceMonitor and PrometheusRule component; Logs: structured
  `axiom.service.log.v1` stdout with per-request trace correlation — the shared
  `service-http` trace layer accepts a valid W3C version-00 `traceparent`
  (invalid input is treated as absent) and generates a fresh local root context
  otherwise, so every request span and log line carries
  `trace_id`/`span_id`/`parent_span_id`/`trace_flags`.; HTTP: Server-Timing
  response attribution — shared `service-http::server_timing` contract
  (`Server-Timing: app;dur=` per-response latency) on every response.
- Gate — behavior: `cargo test -p tape --test observability_assets` - offline
  manifest and metric-name conformance.
- Source: `apps/tape/src/metrics.rs`,
  `apps/tape/src/server.rs (MetricsProvider::render_metrics)`,
  `apps/tape/k8s/components/observability`,
  `apps/tape/tests/observability_assets.rs`,
  `apps/tape/tests/http_transport.rs (metrics_expose_topic_and_subscription_lag_gauges)`
- Evidence: apps/tape/k8s/components/observability,
  apps/tape/tests/observability_assets.rs

### EC Gates Configured

Keep the first Tape implementation behind executable gates, with vat-isolated
meter/guard EC inventories now wired up alongside the local replay smoke gate.

- Root WI: #768
- Surfaces: Tests: `cargo test -p tape`; Vat/Meter/Guard gates under
  `apps/tape/`.
- Gate — behavior: full crate/integration gates
- Gate — efficiency: meter-owned vat-isolated and real-service replay gates
- Gate — security: guard plus live auth/admission/mTLS gates
- Gate — stability: restart/failover, Kind PVC recovery, and bounded 60-second
  soak
- Source: `apps/tape/tests/cli_contract.rs`, `apps/tape/vat.toml`,
  `apps/tape/meter-tape-performance.toml`,
  `apps/tape/guard-tape-security.toml`,
  `apps/tape/external-contracts/competitor-performance/efficiency/meter-gate.md`,
  `apps/tape/external-contracts/security-hardening/security/security-evidence.md`,
  `apps/tape/tests/shared_otlp_tracing.rs`,
  `apps/tape/observability/ (prometheus.yml, otel-collector-config.yaml, grafana-datasources.yaml)`,
  `apps/tape/compose.yaml`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| crate-smoke-gate | epic | #768 | cargo test -p tape |
| tape-vat-meter-guard-ec-gates-observability | change | #1330 | apps/tape/vat.toml, apps/tape/meter-tape-performance.toml, apps/tape/guard-tape-security.toml |
| shared-otlp-trace-export | change | #1662 | `cargo test -p tape --test shared_otlp_tracing` |

### Kubernetes-Native Deployment

Tape runs as a dedicated k8s-native replay service with stable identity,
persistent storage, and operator-managed lifecycle. The bounded Kind dogfood
gate covers one single-node replacement; multi-shard and long-running soak
remain separate work roots.

- Root WI: #768
- Surfaces: K8s: dedicated StatefulSet/operator topology for topic partitions,
  storage, probes, and PDBs (#1328); `tape k8s crd|operator|instance render`,
  `tape k8s operator run` (behind the `operator` cargo feature), and
  `tape dockerfile render --variant source|release`.
- Gate — behavior: offline render/CLI gates (`tests/deploy_cli.rs`,
  `tests/operator.rs`) - CRD structural-schema safety, operator render shape,
  instance profiles, dockerfile fixture parity
- Gate — stability: `bash apps/tape/scripts/kind-e2e.sh` builds the real image,
  creates a disposable Kind cluster, and proves append/replay survives one
  single-node StatefulSet pod replacement with its PVC retained
- Gate — availability: `bash apps/tape/scripts/kind-operator-ha.sh` needs three
  nodes because a single-node drain evicts the replacement too, and proves the
  control plane installs as one kustomization with two replicas, exactly one
  Lease holder cross-checked against each replica's own `tape_operator_leader`
  gauge, a handover that goes on to *apply* a spec change to the child
  StatefulSet, and a drain of the leader's node that completes with
  reconciliation unbroken from before the drain until a surviving replica has
  taken the Lease. The window deliberately outlives the drain: `kubectl drain`
  returns on eviction while the orphaned Lease cannot be reacquired for at
  least its 15s duration, so the drain's own window never contains the handover
  it causes. Every sample records the holder and the holder must be seen to
  change — reconciliation is leader-gated, so continuity under an unmoved
  holder is a true statement about the surviving replica and says nothing about
  a handover.
- Source:
  `apps/tape/k8s/operator/{crd,rbac,deployment,service,pdb,kustomization}.yaml`,
  `apps/tape/k8s/components/operator-monitoring/`,
  `apps/tape/tests/deploy_cli.rs`, `apps/tape/tests/operator.rs`,
  `apps/tape/scripts/kind-e2e.sh`, `apps/tape/scripts/kind-operator-ha.sh`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| dedicated-statefulset-operator-topology | epic | #768 | apps/tape/tests/{deploy_cli,operator}.rs; #1328 |
| operator-kind-pvc-restart-replay | change | #1590 | apps/tape/scripts/kind-e2e.sh |
| operator-control-plane-ha | change | #3053 | apps/tape/scripts/kind-operator-ha.sh |

### Stateful Service Workload

Tape projects the shared stateful-service workload baseline without a duplicate
service implementation. Its durable append log, stable identity/PVC lifecycle,
raft primary-replica recovery, snapshot/backup path, deployment artifacts, and
security boundary are owned by the linked capability roots below; domain
retention/backfill and subscription behavior remain separately verified roots.

- Root WI: #1554
- Surfaces: Durable journal state plus stateful deployment:
  `apps/tape/src/lib.rs`, `libs/raft-core`, `libs/raft-host`,
  `apps/tape/src/backup.rs`, and the dedicated StatefulSet/operator rendering
  surface under `apps/tape/k8s/`.
- Gate — behavior: the `stateful_storage` profile resolved its shared baseline
  under the `aw` capability gate, which was deleted with the binary
- Gate — stability: raft failover/restart, backup snapshot, Kind dogfood,
  authenticated peer mTLS, and the shared security boundary remain
  authoritative in their linked capability roots.
- Gate: the `aw` capability gate, deleted with the binary
- Source: `the rows below are what still runs`,
  `apps/tape/tests/{raft_cluster,raft_failover,raft_persistence}.rs`,
  `apps/tape/tests/{backup,deploy_cli,operator}.rs`,
  `apps/tape/k8s/operator/{crd,rbac,deployment}.yaml`
- Evidence: the `aw` capability gate, deleted with the binary; composes Topic
  Replay Journal, Primary Replicas, HTTP/2 API List, Kubernetes-Native
  Deployment, and Security Hardening without duplicating their claims

### Backup & Restore

Tape writes no second backup format. A backup object is the same
`JournalSnapshot` bytes that the state machine snapshots and restores. A
disaster-recovery seed is deliberately cold and destructive only to an *empty*
PVC: Tape validates the object, atomically prepares the per-node state-machine
snapshot and applied floor, then lets normal Raft log/snapshot catch-up resume.
There is no online `POST /admin/restore` that can overwrite a live leader or
follower.

- Root WI: #1585
- Surfaces: Admin HTTP: `GET /admin/backup` yields the exact whole-journal
  `JournalSnapshot`; CLI: `tape backup` ships those bytes through
  `libs/service-backup`; Runtime: `tape serve --bootstrap-seed-uri` restores
  only into a fresh replica PVC before Raft starts.
- Gate — behavior: `cargo test -p tape --test backup --test bootstrap` -
  snapshot transport and cold-seed conformance.
- Source: `apps/tape/src/{backup,raft}.rs`,
  `libs/service-backup/src/source.rs`, `apps/tape/tests/{backup,bootstrap}.rs`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| exact-journal-snapshot-backup | epic | #1329 | apps/tape/src/backup.rs<br>apps/tape/tests/backup.rs |
| fresh-pvc-cold-recovery-seed | change | #1585 | apps/tape/src/raft.rs<br>apps/tape/tests/bootstrap.rs |

### Replica Sync & Bootstrap

Existing PVCs recover their local Raft state and synchronise through
`raft-host`; a replacement with no local state may load one exact external
snapshot before it catches up. Backup artifacts are a cold seed or DR path, not
a substitute for ordinary live replication, leader forwarding, or
InstallSnapshot.

- Root WI: #1327, #1585
- Surfaces: RaftHost: leader forwarding, committed apply, InstallSnapshot, and
  follower catch-up; Backup seed: exact `file://`, `s3://` (feature `backup`,
  `service-backup/s3`), or `gs://` (always linked, unconditional) object — the
  scheme set `libs/service-backup/src/destination.rs`'s `SUPPORTED_SCHEMES`
  accepts — read through `libs/service-backup` before an empty PVC joins the
  group. `gs://` authenticates via workload-identity ADC in-cluster and is
  GKE-proven end-to-end: CronJob backup + cold restore both via `gs://` (GKE
  acceptance run `0723135853`, under "Verified Cloud Evidence" below).
- Gate — behavior: `cargo test -p tape --test raft_cluster --test bootstrap` -
  live replica convergence and seed-before-catch-up conformance
- Gate — stability:
  `cargo test -p tape --test raft_failover --test raft_persistence` - kill-9
  failover and restart recovery.
- Source: `apps/tape/src/{raft,bin/tape}.rs`,
  `apps/tape/tests/{raft_cluster,raft_failover,raft_persistence,bootstrap}.rs`,
  `libs/raft-host`, `libs/service-backup/src/{destination,source}.rs`

| Work Root | Kind | WI | Gate / Evidence |
|---|---|---:|---|
| raft-log-existing-pvc-sync | epic | #1327 | apps/tape/tests/raft_cluster.rs<br>apps/tape/tests/raft_persistence.rs |
| empty-pvc-external-backup-seed | change | #1585 | apps/tape/src/raft.rs<br>apps/tape/tests/bootstrap.rs |

### Primary Replicas

Tape replicates committed topic journal state through raft so replay ranges and
checkpoints survive leader failover. Required peer mTLS is shared transport
infrastructure, selected by Tape rather than reimplemented in the service.

- Root WI: #768, #1327
- Surfaces: Raft: topic journal state machine over `libs/raft-core` and
  `libs/raft-runtime`'s `TapeRaft`/`TapeStateMachine` (#1327); auto-mode
  leader/follower topology activated by `REPLICAS_PER_SHARD>1` (plus the
  standard `POD_NAME`/`SHARD_COUNT`/`VOTER_COUNT` downward-API quartet) — no
  tape-specific `--raft` flag. Required peer mTLS
  (`TAPE_PEER_TLS_CERT`/`_KEY`/`_CA`, `TAPE_PEER_MTLS`) uses the shared
  `raft-runtime` transport on the dedicated raft listener.
- Gate — behavior: real 3-node in-process raft group - election, leader-applied
  writes replicate to followers, follower-received appends forward to the
  leader, direct follower peer-route POST answers 421, recovered-node catch-up
  followed by a second leader loss, and fresh-node catch-up via InstallSnapshot
- Gate: required-mTLS peers replicate over the authenticated listener and an
  untrusted certificate never reaches the Raft router
- Gate — stability: live 3-node `kill -9` leader failover with no committed
  event loss and restart-recovery of the durable applied-index floor across
  process restarts.
- Source: `apps/tape/tests/raft_cluster.rs`,
  `apps/tape/tests/raft_failover.rs`, `apps/tape/tests/raft_persistence.rs`,
  `apps/tape/tests/raft_peer_mtls.rs`
- Evidence:
  apps/tape/tests/{raft_cluster,raft_failover,raft_persistence,raft_peer_mtls}.rs
  prove election/replication/failover/restart-recovery and trusted-peer mTLS

## Verified Cloud Evidence

Standard GKE operator acceptance evidence for Tape (epic #2434 ordered service
2, after Lumen run `0723041614`). This section records real-cloud proof runs;
the capability contract itself is the `## Capabilities` section above. Harness:
`benchmarks/gcp-operator-acceptance` (`ACCEPTANCE_APPS=tape`).

### Release tape@0.4.11 (2026-07-25, published — binaries + digest-pinned multi-arch GHCR image)

The GKE-proven 0.4.11 candidate shipped. Release run `30114475151`: all five
targets built (`aarch64-apple-darwin`, `x86_64`/`aarch64-unknown-linux-gnu`,
`x86_64`/`aarch64-unknown-linux-musl`), 10 assets attached, and the
`publish ghcr image` job pushed

```
ghcr.io/chrischeng-c4/tape:0.4.11@sha256:5af09a72a9e89edc30090183f7d4ce59f5a146b9229d567a55815253ec8b543a
```

verified by `docker manifest inspect` on the digest with no credentials — an
OCI image index carrying `linux/amd64` + `linux/arm64` (the #2462 tape leg).

The first attempt at this tag failed the image job with `curl: (22) 404` on the
musl tarball: the release matrix had been reverted to gnu-only by the rebase
onto main while `Dockerfile.release` still fetched musl for its glibc-free
`distroless/static-debian12` runtime. Producer and consumer of the release
assets must move together; restored in `c04fe67cdb`, drift gate tracked by
#2563.

### GKE acceptance run 0724164220 (2026-07-24, PASSED — final 0.4.11 candidate, #2557 declarative provisioning proven)

Tape-only run from HEAD `c06504d6e1` (the full 0.4.11 candidate). All 13
proofs `passed` again, and the run's pull/ack legs now execute against a
subscription **pre-provisioned via CR `spec.topics`** with zero imperative
setup (`kubernetes/tape-subscription-cr-provisioned.txt`) — the #2557
dual-path provisioning contract (declarative additive-only ensure alongside
the untouched client API) proven end to end on Standard GKE. Cleanup clean.
Evidence root: `axiom-gcp-run-backup/evidence/0724164220/`.

### GKE acceptance run 0724161853 (2026-07-24, PASSED — 0.4.11 candidate, #2468 restart + #2485 lag gauges proven)

Tape-only run on the unified harness (restored `ACCEPTANCE_APPS=tape` mode
after the app/tape→main rebase; Cloud Build from HEAD `7d063ff3d5`), all 13
proofs `passed` in `tape-acceptance.json` (`axiom.gcp.tape.acceptance.v1`)
and `cleanup.json` `status: clean` (verified `2026-07-24T16:38:13Z`):

- **`bootstrap_seed_uri_restart` (NEW, #2468)**: a pod restart while the CR
  still carries `bootstrapSeedUri` returns Ready with data intact —
  the bootstrap-if-empty fix (`562ff7ecfe`) proven in-cluster; the field is
  declarative bootstrap-if-empty, no longer one-shot.
- **`subscription_lag_gauge` (NEW, #2485)**: `/metrics` serves
  `tape_subscription_lag{topic,subscription}` after the append/consume
  steps (`895d8699cf` scrape-time gauges).
- Re-proven regression base on the 0.4.11-candidate code (which also
  carries #2484 end-to-end body limits and #2483's call-time backup-scheme
  docs): 1x1 reconcile, append/replay lifecycle, subscription pull/ack
  cursor, pod-restart data retention, Workload-Identity GCS backup (635-byte
  object), cold restore from the exact backup object, seed-cleared rolling
  restart retention, 1→3 topology (3 ready), raft leader-pod-replaced
  failover (term 111→113), post-failover write committed.

Evidence root: `axiom-gcp-run-backup/evidence/0724161853/`. Harness
hardening shipped en route (runs t1-t4): mode-aware
render/deploy/verify-clean/operator-cell, the tape backup CronJob restored
as hand-rolled+suspended (the Tape CRD has no CR-native backup field), and
a completion sentinel that makes bash expansion-error aborts exit non-zero
— false-green runs are structurally impossible for BOTH harness modes now.

### GKE acceptance run 0723135853 (2026-07-23, PASSED — full)

- Cluster: persistent Standard GKE `axiom-operator-acceptance`
  (`asia-east1-a`, project `axiom-502607`), run-scoped bucket/GSA/Workload
  Identity created and destroyed by the run.
- Image: pinned immutable
  `courier/tape@sha256:05d2a6f0ff3a6de1ba2be2c9566a3c509e75c62025995713e6475a87792bd619`
  built by Cloud Build from commit `b69ad94947` (includes the #2443
  `lost+found` and #2465 genesis-index seed fixes) with features
  `operator backup`.
- Terminal artifacts: `acceptance.json` (`axiom.gcp.tape.acceptance.v1`,
  every claimed proof `passed`) and `cleanup.json` (`status: clean`,
  2026-07-23T14:18:01Z). Evidence root:
  `axiom-gcp-run-backup/evidence/0723135853/` (home-dir mirror of the
  volatile `/tmp` tree); `run.log` carries the full transcript.

Proven in this run (each row names its artifact under the evidence root):

| Proof | Result | Artifact |
|---|---|---|
| Operator cell: RBAC, Lease, steady-state drift repair, leader-takeover reconcile | passed | `tape-operator-cell.json`; `kubernetes/tape-lease-holder-*.txt` |
| 1x1 reconcile with status-generation fence on Standard GKE | passed | `kubernetes/tape-crs.json`; `kubernetes/workloads-after-tape-deploy.json` |
| Domain lifecycle through the client Service: append offsets 0-2, replay, subscription create, pull cursor 0→3, cumulative ack, empty re-pull | passed | `kubernetes/tape-append.jsonl`; `kubernetes/tape-replay-initial.json`; `kubernetes/tape-pull-*.json`; `kubernetes/tape-ack.json` |
| Pod-restart data retention on the PVC journal (3 events + checkpoint offset 3 survive `tape-0` replacement) | passed | `kubernetes/tape-replay-after-restart.json`; `kubernetes/tape-checkpoint-after-restart.json` |
| Workload-Identity GCS backup: CronJob-triggered `tape backup` writes a 635-byte `JournalSnapshot`; readback carries the appended events | passed | `kubernetes/tape-backup.log`; `gcs/tape-first-object.json` |
| Cold restore: the exact GCS object seeds a fresh-PVC 3-replica/3-voter topology via `bootstrapSeedUri`; restored replay shows exactly offsets 0-2 and checkpoint 3 | passed | `kubernetes/tape-restore-cr.yaml`; `kubernetes/tape-replay-after-restore.json`; `kubernetes/tape-checkpoint-after-restore.json` |
| One-shot seed cleared (#2468 contract): field removed post-restore, full no-seed rolling restart retains all events | passed | `kubernetes/tape-replay-after-seed-clear.json` |
| Raft leader disruption: leader pod deleted (grace 1s), group re-elects (term 12→14; the replaced pod legitimately re-won, `distinct: false` recorded honestly), post-failover write commits at offset 3 and replays as the fourth event | passed | `kubernetes/tape-raftz-initial.json`; `kubernetes/tape-raftz-after-failover.json`; `kubernetes/tape-append-after-failover.json`; `kubernetes/tape-replay-after-failover.json` |
| Verified cleanup: run-scoped bucket, GSA, IAM, image tag, namespaces, CRD destroyed; persistent cluster and pre-existing APIs preserved | passed | `cleanup.json` |

Product defects found and fixed by this campaign (each verified by the
passing run above): #2443 (`lost+found` on cloud PVs rejected cold seeds),
#2465 (single-node-origin `up_to: 0` seeds silently restored an EMPTY
group — genesis-index mapping added, regression
`apps/tape/tests/seed_ha_bootstrap.rs`), #2468 (`bootstrapSeedUri` is
one-shot: leaving it on the CR crash-loops any pod replacement; the
operator-side lifecycle decision remains open).

### GKE acceptance run 0723155311 (2026-07-23, PASSED — released GHCR image)

Re-run of the full 8-proof acceptance with the PUBLISHED release image —
`image_provenance: prebuilt`, zero Cloud Build (#2462's acceptance
condition): `ghcr.io/chrischeng-c4/tape@sha256:ca2928c83fd76681924fd419f35d128933c9abbd1da42342062f96b264b10a12`
(the `tape@0.4.10` musl-static release, pulled anonymously from public
GHCR). All eight proofs passed again on the release binary, which also
carries the adoption fixes (#2482 GET-retention contract, #2484 body
limit + bounded replay, #2468 runbook semantics). `cleanup.json`
`status: clean` (2026-07-23T16:04:36Z). Evidence root:
`axiom-gcp-run-backup/evidence/0723155311/`.

Exclusions (recorded, not claimed): shard migration (`shardCount` pinned
to 1), live in-place replica membership change (startup-static
membership), and CPU/memory pressure actuation — product gaps tracked in
#2437. Earlier partial runs' evidence (0723080156 six-proof subset and the
intermediate diagnosis runs) is retained under the same backup root.
---

## Operations

### Node Drain and PodDisruptionBudget

The direct-install Tape data-plane StatefulSet has `maxUnavailable: 0` in its
PodDisruptionBudget. The eviction API will reject `kubectl drain`, which keeps
retrying forever. With exactly one durable member, an eviction would mean
data-plane downtime and potential data loss if the pod cannot rejoin before the
journal is cleaned or the PVC is recycled.

**Recognizing the issue:** when a GKE node auto-upgrade or a manual drain hangs,
you will see:

```
error when evicting pods/"tape-0" (will retry after 5s):
Cannot evict pod as it would violate the disruption budget.
```

**How to unblock the drain:**

The PDB gates only the eviction API. Delete the pod directly via the delete API,
which bypasses the PDB:

```bash
kubectl delete pod tape-0 -n <namespace>
```

This unblocks the drain immediately. The node continues draining, and Tape goes
through graceful shutdown and recovery.

**Alternative: temporarily widen the PDB.** Patch it, wait for the eviction to
actually land, then restore it. The wait is not optional — restoring
`maxUnavailable: 0` while the eviction is still in flight re-blocks the drain,
and you are back where you started with a PDB that now looks correct:

```bash
kubectl patch pdb tape -n <namespace> -p '{"spec":{"maxUnavailable":1}}'
kubectl wait --for=delete pod/tape-0 -n <namespace> --timeout=120s
kubectl patch pdb tape -n <namespace> -p '{"spec":{"maxUnavailable":0}}'
```

Prefer the direct delete. It is one step, it cannot leave the PDB widened if you
are interrupted, and it makes the outage an explicit act rather than a side
effect of a policy edit.

**During graceful shutdown:**

When the pod is deleted, the kubelet sends SIGTERM. Tape then
(`libs/server-lifecycle/src/signal.rs`, `shutdown_with_drain`):

1. Calls `start_drain()`, so `/readyz` returns 503.
2. The kubelet's readiness probe fails and the endpoints controller withdraws the
   pod from the `tape` Service. New clients stop being routed here; this is the
   whole mechanism, there is no flag clients read.
3. Sleeps for `TAPE_GRACE_SECS` (ConfigMap `tape-config`, default `30`) so
   in-flight requests can finish, then exits.

**Nothing is flushed at shutdown, and nothing needs to be.** Every acked write
was already made durable at the time it was acked: `AppState::persist`
(`src/server.rs`) writes the journal through `storage_durable::atomic_write` with
`FsyncPolicy::Always` on every mutation. The grace window buys in-flight requests
time to complete — it is not a durability window, and cutting it short costs
open requests, not data.

> **Keep `TAPE_GRACE_SECS` ≤ `terminationGracePeriodSeconds`.** These are two
> independent knobs on the direct-install path: `k8s/base/statefulset.yaml`
> hardcodes `terminationGracePeriodSeconds: 30` and `TAPE_GRACE_SECS` comes from
> the ConfigMap. They ship equal, so there is zero margin — raise the ConfigMap
> value alone and the kubelet SIGKILLs the process partway through its own drain
> sleep. Raise both together. (On the operator path this cannot happen:
> `src/operator/render.rs` derives `terminationGracePeriodSeconds` from
> `spec.graceSecs`.)

**After pod deletion (write outage begins):**

Once the old pod is gone and before the new one is ready, nothing answers the
Service, so both reads and writes fail. With one durable member this outage is
unavoidable; it is the cost you accepted when you unblocked the drain.

**Pod recovery (the replacement starts):**

1. Expect the replacement to be **Pending, not scheduled back onto the drained
   node** — `kubectl drain` cordons the node first (`SchedulingDisabled`), which
   is the point. Do not `kubectl uncordon` to make it schedule: that defeats the
   node upgrade or repair you were draining for.
2. Where it can land depends on the volume. A regional/zonal PD binds it to the
   same zone; a node-local PV (`local-path`, LVM, hostPath) pins it to the
   drained node, in which case Pending until that node returns is the correct and
   expected state. Check with:
   ```bash
   kubectl get pod tape-0 -n <namespace> -o wide
   kubectl describe pod tape-0 -n <namespace> | tail -20   # scheduler's reason
   ```
3. Whichever node it lands on, it rebinds the same PVC (`data-tape-0`).
   Kubernetes does not delete a PVC when its pod is deleted.
4. On startup Tape loads `/data/journal.json` (`TAPE_DATA_DIR=/data`) and serves
   from it. The direct-install StatefulSet is explicitly single-member
   (`VOTER_COUNT=1`), so there is no Raft state to replay and no cluster to
   rejoin — readiness *is* recovery.

**Verify recovery:**

```bash
kubectl rollout status statefulset/tape -n <namespace>
kubectl get endpoints tape -n <namespace>          # tape-0's IP must be listed
kubectl exec -n <namespace> tape-0 -- \
  wget -qO- http://127.0.0.1:7137/readyz
```

Then confirm the journal survived by replaying a topic you know had events
before the drain — an empty replay from a *ready* pod means the PVC did not come
back with it:

```bash
kubectl exec -n <namespace> tape-0 -- \
  wget -qO- 'http://127.0.0.1:7137/topics/<topic>/replay?from_offset=0'
```

**Optional: export the journal before the outage:**

If the release has the backup feature compiled in (`tape backup` is
feature-gated), export the journal state before deletion:

```bash
tape backup --url http://tape-0.tape-headless.<namespace>.svc.cluster.local:7137 \
  --dest file:///tmp/tape-backup --token "$TAPE_BACKUP_TOKEN"
```

(The per-pod DNS name goes through the headless Service `tape-headless`, which is
the StatefulSet's `serviceName`; `tape` is the load-balanced ClusterIP and does
not resolve per-pod.)

See `apps/tape/docs/deployment-handoff.md` § 7 for the full backup/restore
runbook.
