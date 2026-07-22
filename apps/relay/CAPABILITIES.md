# Relay

## Brief

Machine-readable capability contract for Relay.

## Capabilities

Canonical field-style capability contracts below are machine-readable input for `aw capability`; YAML and legacy tables are migration input only.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| CLI Interface | 1204 | implemented | passing | conformance | ready | single `relay` bin — serve default plus spec/backup/k8s/dockerfile verbs and served OpenAPI |
| CLI Standard Surface | 1204 | implemented | passing | conformance | ready | shared `cli-std` llm/upgrade/issue surface with build-stamp provenance |
| Chainable Output Conformance | 1208 | implemented | passing | conformance | ready | raw artifact streams stay unwrapped and terminal write/backup/render paths use executable `next:` or explicit done markers |
| Competitive Broker Feature Parity | 108 | implemented | verified | dogfood | ready | RabbitMQ/NATS JetStream/Redis Streams work-queue replacement breadth plus real three-voter Kind failover |
| Competitive Broker Performance | 125 | implemented | passing | dogfood | ready | release-mode measured fsync-always lifecycle envelope plus advisory real-service comparison harness; claims remain workload-scoped |
| Long-Running Stability | 1205 | implemented | verified | dogfood | ready | recovery, bounded retention, lease reclaim, graceful drain, backup, two-cycle failover, and fixed-state 60-second error/RSS/FD/thread/p99 soak |
| Security Hardening | 1206 | implemented | passing | negative | ready | SecurityTool baseline with bearer/RBAC/admission rejection, untrusted peer-certificate rejection, last-known-good rotation, restricted pods, Secret projection, and NetworkPolicy |
| HTTP/2 API List | 108 | implemented | passing | conformance | ready | concise HTTP/1.1+h2c producer, bidi consumer, compatibility worker, probe, metrics, OpenAPI, and offline spec surfaces |
| Standard Operational Endpoints | 1205 | implemented | passing | conformance | ready | one-port `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs` plus offline `relay spec` |
| EC Gates Configured | 125 | implemented | passing | conformance | ready | aw.toml EC inventory, vat meter/guard runners, and external-contracts evidence stay wired |
| Kubernetes-Native Deployment | 1207 | implemented | verified | dogfood | ready | layered Kustomize base/overlays/components, CRD/operator/instance, PDB/PVC/NetworkPolicy/observability, and Kind leader-failover proof |
| Primary Replicas | - | implemented | verified | dogfood | ready | every lifecycle mutation is Raft committed with node/epoch fencing, durable restart recovery, snapshots, and leader failover |
| Stateful Service Workload | 1555 | implemented | passing | conformance | ready | mandatory baseline composes existing durable acknowledgement, stable StatefulSet identity, raft, backup, security, and deployment evidence without duplicating domain policy |
| Durable Ordered Log | - | implemented | passing | conformance | ready | domain: per-subject append/batch append, dedupe, retention, sparse index, and segment lifecycle |
| Work Queue Lifecycle | - | implemented | passing | conformance | ready | domain: committed lease/batch lease, heartbeat, ack/batch ack, redelivery, reconcile, and node/epoch fencing |
| HTTP/OpenAPI Worker Protocol | 108 | implemented | passing | conformance | ready | domain: polyglot h2c worker contract with preferred bidirectional consume and compatibility lease/ack routes |
| Raft HA | 1207 | implemented | verified | dogfood | ready | RelayStateMachine, committed full lease lifecycle, auto-mode topology, applied-index floor, real mTLS, and Kind failover |

### CLI Interface

ID: cli-interface
Root WI: 1204
Status: auditing
Type: RuntimeTool
Required Verification: conformance
Promise:
Expose relay as one runnable binary with a stable process entrypoint — serve by
default, locally and as the Kubernetes raft node — plus offline spec and deploy
verbs and a served OpenAPI contract for non-Rust clients.
Gate Inventory:
- apps/relay/tests/worker_loop.rs; apps/relay/tests/spec_cli.rs; apps/relay/tests/deploy_cli.rs; apps/relay/tests/raft_config.rs; apps/relay/tests/raft_cluster.rs
Surfaces:
- CLI: `relay` + `relay` - single bin; bare `relay` serves the h2c broker (auto-mode raft HA) and carries the spec/backup/k8s/dockerfile verbs.
- HTTP: `/openapi.json` - machine-readable worker contract served by the binary.
EC Dimensions:
- behavior: `cargo test -p relay --test worker_loop --test spec_cli --test deploy_cli` - binary-facing contract, offline spec, and deploy-verb smoke

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| single-relay-bin-serve-default | epic | 1204 | implemented | passing | conformance | apps/relay/src/bin/relay.rs; apps/relay/tests/worker_loop.rs |
| auto-mode-raft-node-entrypoint | epic | 1207 | implemented | passing | conformance | apps/relay/src/bin/relay.rs; apps/relay/tests/raft_config.rs; apps/relay/tests/raft_cluster.rs |
| served-openapi-contract | epic | 108 | implemented | passing | conformance | apps/relay/tests/worker_loop.rs; apps/relay/docs/worker-protocol.md |

### CLI Standard Surface

ID: cli-standard-surface
Root WI: 1204
Status: auditing
Type: RuntimeTool
Required Verification: conformance
Promise:
Ship the mandatory shared `cli-std` surface (llm/upgrade/issue) every ecosystem
CLI owes, backed by build-stamp provenance, without blurring it into relay's
domain serve/spec/backup/deploy verbs.
Gate Inventory:
- apps/relay/src/llm.rs; apps/relay/src/bin/relay.rs; apps/relay/tests/spec_cli.rs; libs/cli-std/src
Surfaces:
- CLI: `relay llm` - offline agent self-doc topics (outline, http-api, operations) required by the ecosystem CLI convention.
- CLI: `relay upgrade` + `--check` + `cli-std` - shared self-update and `--check` surface through `cli-std`.
- CLI: `relay issue search` + `relay issue view` + `relay issue create` + `app:relay` - shared tracker read/write surface scoped to `app:relay`.
EC Dimensions:
- behavior: `cargo test -p relay --test spec_cli` - the llm topic surface documents every shipped serve/deploy/backup knob through the compiled binary

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-llm-topic-surface | epic | 1204 | implemented | passing | conformance | apps/relay/src/llm.rs; apps/relay/tests/spec_cli.rs |
| shared-upgrade-issue-surface | epic | 1204 | implemented | passing | smoke | apps/relay/src/bin/relay.rs; libs/cli-std/src |

### Chainable Output Conformance

ID: chainable-output-conformance
Root WI: 1208
Status: auditing
Type: RuntimeTool
Required Verification: conformance
Promise:
Keep relay's operational CLI outputs chainable per the CLI convention: raw
artifact streams (spec, renders) stay unwrapped bytes, while file-writing and
admin verbs grow explicit `next:`/terminal markers.
Gate Inventory:
- apps/relay/tests/deploy_cli.rs; apps/relay/tests/spec_cli.rs; apps/relay/src/bin/relay.rs
Surfaces:
- CLI: `relay spec` + `relay k8s crd|operator|instance render` + `relay dockerfile render` + `--out` - raw artifact/data streams that intentionally stay unwrapped bytes.
- CLI: `--out` + `relay backup` + `relay issue ...` + `relay upgrade --check` - operational outputs that owe a runnable next step or explicit terminal marker.
EC Dimensions:
- behavior: `cargo test -p relay --test deploy_cli --test spec_cli` - render/spec stdout stays raw parseable artifact bytes through the compiled binary

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| raw-artifact-streams-stay-unwrapped | epic | 1208 | implemented | passing | conformance | `cargo test -p relay --test deploy_cli --test spec_cli` |
| shared-issue-upgrade-terminal-markers | epic | 1204 | implemented | passing | smoke | libs/cli-std/src |
| next-markers-on-write-and-backup-outputs | epic | - | implemented | passing | conformance | CLI contract tests cover artifact, backup, and terminal output paths |

### Competitive Broker Feature Parity

ID: competitor-feature-parity
Root WI: 108
Status: auditing
Type: RuntimeTool
Required Verification: conformance, dogfood
Promise:
Cover the baseline single-cast work-queue broker functions Relay needs to
compete with RabbitMQ, NATS JetStream, Redis Streams, and Dragonfly in Axiom
workloads. Redpanda/Kafka-class replay journals are tape competitors, not relay
competitors.
Gate Inventory:
- apps/relay/tests/relay_core.rs; apps/relay/tests/http2_transport.rs; apps/relay/tests/work_queue_api.rs; apps/relay/tests/worker_loop.rs; apps/relay/tests/raft_core.rs; apps/relay/tests/raft_persistence.rs; apps/relay/tests/raft_cluster.rs; apps/relay/scripts/kind-failover-smoke.sh
Surfaces:
- Rust API: `Relay` - durable ordered log, work-queue lease, and ack primitives.
- HTTP: `publish` + `lease` + `heartbeat` + `ack` - RabbitMQ/SQS-style single-cast work-queue workflows over h2c.
- CLI: `relay` - auto-mode failover-capable broker node.
EC Dimensions:
- behavior: `cargo test -p relay --test relay_core --test work_queue_api --test worker_loop --test raft_core --test raft_persistence --test raft_cluster` - functional parity conformance for core broker workflows

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| ordered-log-queue-and-raft-feature-breadth | epic | - | implemented | passing | conformance | `cargo test -p relay --test relay_core --test work_queue_api --test worker_loop --test raft_core --test raft_persistence --test raft_cluster` |
| per-subject-shard-append-ordering | epic | - | implemented | passing | conformance | apps/relay/tests/relay_core.rs |
| lease-heartbeat-ack-lifecycle | change | #1850 | implemented | passing | conformance | apps/relay/tests/work_queue_api.rs |
| http-worker-protocol-parity | epic | 108 | implemented | passing | conformance | apps/relay/tests/worker_loop.rs; apps/relay/docs/worker-protocol.md |
| in-process-raft-convergence | epic | - | implemented | passing | conformance | apps/relay/tests/raft_core.rs |
| durable-raft-hard-state-restore | epic | - | implemented | passing | conformance | apps/relay/tests/raft_persistence.rs |
| real-h2c-raft-cluster-smoke | epic | - | implemented | passing | dogfood | apps/relay/tests/raft_cluster.rs |
| kubernetes-kind-failover-smoke | epic | - | implemented | verified | dogfood | apps/relay/scripts/kind-failover-smoke.sh; apps/relay/k8s |

### Competitive Broker Performance

ID: competitor-performance
Root WI: 125
Status: auditing
Type: RuntimeTool
Required Verification: dogfood
Promise:
Keep Relay's performance claims tied to a release-mode, fsync-always durable
measurement whose report is parsed independently and fails closed on missing or
zero observations. Keep RabbitMQ, NATS JetStream, Redis Streams, and Dragonfly
comparisons as advisory dogfood until equivalent real-service calibration is
promoted into a required production gate.
Gate Inventory:
- apps/relay/vat.toml; apps/relay/scripts/ec-evidence.sh; apps/relay/tests/measured_performance.rs; apps/relay/tests/work_queue_throughput.rs; apps/relay/tests/perf_gate.rs
- apps/relay/scripts/soak.sh; apps/relay/examples/bench_compare.rs; apps/arena/examples/relay-vs-rabbitmq-nats-redis.toml
Surfaces:
- Meter/Vat: `apps/relay/vat.toml#meter-perf` + `cargo run -p relay --release --example bench_compare -- --backend <target>` + `apps/arena/examples/relay-vs-rabbitmq-nats-redis.toml` + `measured_performance` - isolated meter execution for the release-mode measured durable lifecycle gate.; Harness: `cargo run -p relay --release --example bench_compare -- --backend <target>` - durable-only closed-loop external broker comparison across relay, RabbitMQ, NATS JetStream, Redis Streams, and Dragonfly.; Arena: `apps/arena/examples/relay-vs-rabbitmq-nats-redis.toml` - advisory normalized ratio wrapper.; Rust test: `measured_performance` - machine-readable fsync-always local envelope with an independent parser.
EC Dimensions:
- behavior: `bash apps/relay/scripts/ec-evidence.sh performance-behavior` - all named work-queue and decision-model tests with per-binary non-zero counts
- efficiency: `bash apps/relay/scripts/ec-evidence.sh performance-efficiency` - exactly one independently parsed 2,000-message durable gate, one report marker, then Meter evidence
- stability: `RELAY_SOAK_AUTOSTART=1 bash apps/relay/scripts/soak.sh` - 60-second error, resource, and p99 plateau gate

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| o-1-lease-cursor-throughput | epic | - | implemented | passing | conformance | apps/relay/tests/work_queue_throughput.rs |
| normalized-win-ratchet-decision-model | epic | 125 | implemented | passing | conformance | apps/relay/tests/perf_gate.rs |
| measured-durable-lifecycle-production-gate | change | #2172 | implemented | passing | dogfood | `bash apps/relay/scripts/ec-evidence.sh performance-efficiency`; release-only child report plus independent parent parser and outer zero-test/marker oracle; fixed 2,000-message/128-byte/batch-100 envelope |
| vat-meter-throughput-gate | epic | 125 | implemented | passing | dogfood | `cd apps/relay && vat run meter-perf`; apps/relay/vat.toml#meter-perf |
| external-broker-comparison | epic | 125 | implemented | passing | dogfood | 2026-07-17 real Relay/RabbitMQ/NATS bulk lifecycle calibration; apps/relay/docs/perf-gate.md; apps/relay/examples/bench_compare.rs; apps/arena/examples/relay-vs-rabbitmq-nats-redis.toml |

### Long-Running Stability

ID: long-running-stability
Root WI: 1205
Status: auditing
Type: RuntimeTool
Required Verification: conformance, dogfood
Promise:
Run as a long-lived broker without losing committed entries, leaking stuck
leases forever, or corrupting recovery state across restarts, segment rotation,
graceful drains, and leader failover — with a backup/restore path for the rest.
The 2026-07-17 default 60-second run completed 5,622 fixed-state operations
with zero errors, RSS 14,352 -> 14,352 KiB, FD 14 -> 14, threads 11 -> 11,
and inspect p99 1 -> 1 ms. The cluster gate recovers the first stopped node
from its durable engine/Raft state before committing through a second leader
loss.
Gate Inventory:
- apps/relay/tests/durable.rs; apps/relay/tests/segments.rs; apps/relay/tests/reconciler.rs; apps/relay/tests/raft_persistence.rs; apps/relay/tests/raft_cluster.rs; apps/relay/tests/backup.rs
- apps/relay/scripts/soak.sh; libs/service-observability/scripts/soak-metrics.sh
Surfaces:
- CLI: `relay` + `apps/relay/k8s` - durable serve process with reconciler, graceful drain, and auto-mode failover.; K8s: `apps/relay/k8s` - StatefulSet-oriented raft deployment.
- CLI: `relay backup` - consistent snapshot capture from a running node.
EC Dimensions:
- stability: `cargo test -p relay --test durable --test segments --test reconciler --test raft_persistence --test raft_cluster` - recovery, retention, lease reclaim, and repeated failover conformance; dogfood: `RELAY_SOAK_AUTOSTART=1 bash apps/relay/scripts/soak.sh` - bounded error, RSS, FD, thread/task, and p99 plateaus

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| durable-power-safe-recovery | epic | - | implemented | passing | conformance | apps/relay/tests/durable.rs |
| segment-rotation-and-retention-recovery | epic | - | implemented | passing | conformance | apps/relay/tests/segments.rs |
| lease-reclaim-liveness | change | #1850 | implemented | passing | conformance | apps/relay/tests/reconciler.rs |
| graceful-drain-readiness | epic | 1205 | implemented | passing | conformance | apps/relay/tests/http2_transport.rs |
| backup-restore-consistent-snapshot | epic | 1209 | implemented | passing | conformance | apps/relay/tests/backup.rs; apps/relay/HA.md |
| raft-hard-state-restart-safety | epic | - | implemented | passing | conformance | apps/relay/tests/raft_persistence.rs |
| failover-without-committed-loss | change | #1850 | implemented | passing | dogfood | apps/relay/tests/raft_cluster.rs |

### Security Hardening

ID: security-hardening
Root WI: 1206
Status: auditing
Type: SecurityTool
Required Verification: conformance, negative
Promise:
Keep Relay safe as a long-running broker: opaque payload boundaries, the shared
bearer-token contract on the data plane (probes exempt), fail-fast peer-TLS
material validation, untrusted peer-certificate rejection, last-known-good
credential rotation, and explicit negative gates for request limits and
network policy before production readiness.
Gate Inventory:
- apps/relay/vat.toml; apps/relay/scripts/ec-evidence.sh; apps/relay/tests/auth.rs; apps/relay/tests/service_admission.rs
- apps/relay/tests/raft_peer_mtls.rs; apps/relay/tests/direct_k8s_assets.rs
- apps/relay/src/auth.rs; apps/relay/src/peer_tls.rs; libs/service-auth/src/reload.rs
Surfaces:
- HTTP: queue-scoped bearer RBAC with audited live registry rotation and bounded admission; peers: dedicated reloadable mTLS listener; K8s: read-only Secret projection, restricted pods, PDB and ingress NetworkPolicy.
EC Dimensions:
- behavior: `bash apps/relay/scripts/ec-evidence.sh security-behavior` - fail-closed bearer 401/403, subject/stream RBAC, tokenless probes, and bounded 429 admission
- security: `bash apps/relay/scripts/ec-evidence.sh security-boundaries` - fail-closed trusted/untrusted peer certificates, restricted pod/Secret/NetworkPolicy posture, and vat-isolated guard+dynamic Meter evidence
- stability: `bash apps/relay/scripts/ec-evidence.sh security-stability` - all shared reload cases plus exact Relay live rotation and trusted peer replication retain last-known-good operation

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| opaque-payload-boundary | epic | - | implemented | passing | smoke | apps/relay/tests/relay_core.rs; apps/relay/tests/worker_loop.rs |
| bearer-auth-token-registry | epic | 1206 | implemented | passing | conformance | apps/relay/tests/auth.rs; apps/relay/src/auth.rs |
| peer-tls-material-validation | epic | 1209 | implemented | passing | conformance | `bash apps/relay/scripts/ec-evidence.sh security-boundaries && bash apps/relay/scripts/ec-evidence.sh security-stability`; apps/relay/src/peer_tls.rs; apps/relay/HA.md |
| guard-static-runtime-evidence | epic | - | implemented | passing | negative | `cd apps/relay && vat run guard-security`; static scan plus a self-testing named-suite/zero-count oracle and auth/admission/peer-mTLS/K8s/reload Meter evidence |
| request-limit-and-malformed-frame-negative-tests | epic | - | implemented | passing | negative | apps/relay/tests/service_admission.rs plus HTTP/auth/consume negative coverage |
| network-policy-and-peer-mtls-termination | epic | - | implemented | passing | conformance | trusted replication and attacker-CA rejection in apps/relay/tests/raft_peer_mtls.rs; restricted workload and policy assertions in apps/relay/tests/direct_k8s_assets.rs |
| securitytool-negative-runtime-evidence | change | #2175 | implemented | passing | negative | `bash apps/relay/scripts/ec-evidence.sh security-behavior && bash apps/relay/scripts/ec-evidence.sh security-boundaries && bash apps/relay/scripts/ec-evidence.sh security-stability && cd apps/relay && vat run guard-security`; behavior/security/stability EC cases and vat/guard dynamic dispatch close the independent false-green findings |

### HTTP/2 API List

ID: http2-api-list
Root WI: 108
Status: auditing
Type: RuntimeTool
Required Verification: conformance
Promise:
Publish Relay's supported HTTP/2 API as a compact producer and worker endpoint
inventory, with OpenAPI/docs pointers, without making OpenAPI completeness the
capability definition.
Gate Inventory:
- apps/relay/tests/http2_transport.rs; apps/relay/tests/worker_loop.rs; apps/relay/tests/spec_cli.rs; apps/relay/docs/worker-protocol.md
Surfaces:
- HTTP: `/openapi.json` - publish, lease, heartbeat, ack, consume, `/openapi.json`, and probe routes - concise h2c API list for producers and workers.
- CLI: `relay spec` + `apps/relay/docs/worker-protocol.md` - offline OpenAPI twin of the served contract.; Docs: `apps/relay/docs/worker-protocol.md` - endpoint contract summary.
EC Dimensions:
- behavior: `cargo test -p relay --test http2_transport --test worker_loop` - h2c transport and worker protocol conformance

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| h2c-publish-and-consume-route-list | epic | - | implemented | passing | conformance | apps/relay/tests/http2_transport.rs |
| worker-lease-heartbeat-ack-route-list | epic | 108 | implemented | passing | conformance | apps/relay/tests/worker_loop.rs; apps/relay/docs/worker-protocol.md |
| served-openapi-contract | epic | 108 | implemented | passing | conformance | apps/relay/tests/worker_loop.rs |
| offline-spec-openapi-list | epic | 1209 | implemented | passing | conformance | apps/relay/tests/spec_cli.rs |

### Standard Operational Endpoints

ID: standard-operational-endpoints
Root WI: 1205
Status: auditing
Type: Service
Required Verification: conformance
Promise:
Expose the standard one-port operational surface the service trait requires —
probes, metrics scrape, live spec, and Swagger UI stay always-on and
auth-exempt on the serve port, with `relay spec` as the offline twin.
Gate Inventory:
- apps/relay/tests/http2_transport.rs; apps/relay/tests/auth.rs; apps/relay/tests/spec_cli.rs; apps/relay/src/server.rs
Surfaces:
- HTTP: `/healthz` + `/readyz` + `/metrics` + `/openapi.json` + `/docs` + `service_http::standard_probe_routes` - auth-exempt liveness, readiness, Prometheus scrape, live-spec, and Swagger UI endpoints on the one serve port via `service_http::standard_probe_routes`.
- CLI: `relay spec` - offline OpenAPI evidence for the same operational contract when no server is running.
EC Dimensions:
- behavior: `cargo test -p relay --test http2_transport` - probe surface, drain flip, and Prometheus metrics over h2c and HTTP/1.1

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| service-http-standard-probe-routes | epic | 1205 | implemented | passing | conformance | apps/relay/tests/http2_transport.rs; apps/relay/src/server.rs |
| drain-flips-readyz | epic | 1205 | implemented | passing | conformance | apps/relay/tests/http2_transport.rs |
| probes-stay-tokenless-under-auth | epic | 1206 | implemented | passing | conformance | apps/relay/tests/auth.rs |
| offline-spec-matches-served-contract | epic | 1209 | implemented | passing | conformance | apps/relay/tests/spec_cli.rs |

### EC Gates Configured

ID: ec-gates-configured
Root WI: 125
Status: auditing
Type: Devops
Required Verification: conformance
Promise:
Keep relay's service-trait EC baseline explicit and runnable: aw.toml owns the
EC inventory, vat owns the meter/guard runners, and external-contracts/ carries
the evidence contracts each gate closes against.
Gate Inventory:
- apps/relay/aw.toml; apps/relay/vat.toml; apps/relay/meter-relay-performance.toml; apps/relay/guard-relay-security.toml; apps/relay/external-contracts
Surfaces:
- Config: `apps/relay/aw.toml` - AW EC inventory and generated efficiency/security cases with dispatch commands.
- Config: `apps/relay/vat.toml` + `meter-perf` + `guard-security` + `apps/relay/tests/benchmark_relay_competitor_performance_meter_gate.rs` + `apps/relay/tests/security_relay_security_hardening_guard_scan.rs` - vat-managed `meter-perf` and `guard-security` runners.; Tests: `apps/relay/tests/benchmark_relay_competitor_performance_meter_gate.rs`, `apps/relay/tests/security_relay_security_hardening_guard_scan.rs` - generated EC evidence stubs tied back to the inventory.
EC Dimensions:
- efficiency: `cd apps/relay && vat run meter-perf` - meter-owned throughput ratchet dispatch inside vat
- security: `cd apps/relay && vat run guard-security` - guard-owned security evidence dispatch inside vat

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| aw-ec-generated-inventory-and-dispatch | epic | 125 | implemented | passing | conformance | `aw ec check --project relay`; apps/relay/aw.toml |
| vat-managed-meter-and-guard-runners | epic | 125 | implemented | passing | conformance | `cd apps/relay && vat run meter-perf && vat run guard-security`; apps/relay/vat.toml; apps/relay/meter-relay-performance.toml; apps/relay/guard-relay-security.toml |
| external-contract-evidence-docs | epic | - | implemented | passing | conformance | `aw ec check --project relay`; apps/relay/external-contracts |

### Kubernetes-Native Deployment

ID: kubernetes-native-deployment
Root WI: 1207
Status: auditing
Type: Devops
Required Verification: conformance, dogfood
Promise:
Run Relay as a Kubernetes-native broker: an operator-rendered StatefulSet with
the downward-API env auto-mode reads, offline CRD/operator/instance/dockerfile
render verbs whose committed files are byte-equal fixtures, and a kind failover
path for deployment dogfood.
Gate Inventory:
- apps/relay/k8s; apps/relay/tests/deploy_cli.rs; apps/relay/tests/operator.rs; apps/relay/scripts/kind-failover-smoke.sh; apps/relay/tests/raft_config.rs; apps/relay/tests/raft_cluster.rs
Surfaces:
- CLI: `relay k8s crd render` + `relay k8s operator render` + `relay k8s instance render --profile dev|staging|prod|template` + `relay k8s operator run` - RelaySpec CRD, controller, and instance deploy verbs (operator feature).
- CLI: `relay dockerfile render --variant source|release` + `apps/relay/k8s` + `apps/relay/scripts/kind-failover-smoke.sh` - image fixtures rendered from the binary.; K8s: `apps/relay/k8s` - single-node direct install for kind/smoke.; Script: `apps/relay/scripts/kind-failover-smoke.sh` - live failover dogfood path.
EC Dimensions:
- behavior: `cargo test -p relay --test raft_config --test raft_cluster` - node topology config and real h2c cluster smoke
- stability: `apps/relay/scripts/kind-failover-smoke.sh` - kind failover dogfood

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| statefulset-raft-service-topology | epic | - | implemented | verified | dogfood | `cargo test -p relay --test deploy_cli --test operator --test direct_k8s_assets`; apps/relay/k8s |
| auto-mode-node-topology-config | epic | 1207 | implemented | passing | conformance | apps/relay/tests/raft_config.rs |
| relayspec-crd-operator-instance-render | epic | 1208 | implemented | passing | conformance | `cargo test -p relay --test deploy_cli --test operator` |
| dockerfile-render-fixtures | epic | 1208 | implemented | passing | conformance | apps/relay/tests/deploy_cli.rs; apps/relay/Dockerfile |
| kubernetes-kind-failover-smoke | epic | - | implemented | verified | dogfood | apps/relay/scripts/kind-failover-smoke.sh |

### Primary Replicas

ID: primary-replicas
Root WI: -
Status: auditing
Type: RuntimeTool
Required Verification: conformance, dogfood
Promise:
Support a primary/replica broker topology where the raft leader owns writes,
followers replicate committed state, and failover preserves the ordered-log and
work-queue API contract.
Gate Inventory:
- apps/relay/tests/raft_core.rs; apps/relay/tests/raft_persistence.rs; apps/relay/tests/raft_cluster.rs; apps/relay/HA.md
Surfaces:
- CLI: `relay` + `REPLICAS_PER_SHARD` + `RelayStateMachine` - auto-mode raft-backed primary/replica broker node (`REPLICAS_PER_SHARD` > 1 flips replica mode).; Rust API: `RelayStateMachine` - publish replication, snapshot/compaction, and the applied-index floor.; K8s: operator-rendered StatefulSet - replica pods with stable identities.
EC Dimensions:
- stability: `cargo test -p relay --test raft_core --test raft_persistence --test raft_cluster` - leader/follower convergence, hard-state restore, and h2c cluster smoke

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| in-process-raft-convergence | epic | - | implemented | passing | conformance | apps/relay/tests/raft_core.rs |
| durable-raft-hard-state-restore | epic | - | implemented | passing | conformance | apps/relay/tests/raft_persistence.rs |
| real-h2c-raft-cluster-smoke | epic | - | implemented | passing | dogfood | apps/relay/tests/raft_cluster.rs |

### Stateful Service Workload

ID: stateful-service-workload
Root WI: 1555
Status: auditing
Type: Devops
Required Verification: conformance
Promise:
Compose Relay's stateful production workload from the shared storage, backup,
raft, peer-security, and Kubernetes mechanisms while keeping product policy in
Relay's existing durable-log, work-queue, HA, security, and deployment roots.
This root is an integration map, not a second copy of those domain contracts.
Gate Inventory:
- libs/raft-runtime; libs/service-backup; libs/service-auth; libs/service-k8s
- apps/relay/aw.toml; apps/relay/k8s; apps/relay/HA.md; apps/relay/tests/durable.rs; apps/relay/tests/raft_persistence.rs; apps/relay/tests/raft_cluster.rs; apps/relay/tests/backup.rs; apps/relay/tests/auth.rs; apps/relay/tests/direct_k8s_assets.rs
Surfaces:
- Config: `apps/relay/aw.toml` + `stateful_storage` + `apps/relay/k8s` + `RelayStateMachine` - the `stateful_storage` trait requires the common workload baseline.; K8s: `apps/relay/k8s` - RelaySpec/operator rendering gives the broker stable StatefulSet identity and persistent storage.; Rust API: `RelayStateMachine` - the raft-backed ordered log owns replicated durable acknowledgement and snapshot restore.
EC Dimensions:
- behavior: `aw capability check --project relay --skip-issue-inventory` - Relay's capability contract resolves the trait-derived stateful baseline and its evidence references

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| relay-stateful-service-workload | epic | 1555 | implemented | passing | conformance | Shared: libs/raft-runtime, libs/service-backup, libs/service-auth, and libs/service-k8s. Relay policy: Durable Ordered Log, Work Queue Lifecycle, Primary Replicas, Raft HA, Security Hardening, Kubernetes-Native Deployment, and Long-Running Stability sections above. |

### Durable Ordered Log

ID: durable-ordered-log
Root WI: -
Status: auditing
Type: RuntimeTool
Required Verification: conformance
Promise:
Append messages in per-subject/shard order, deduplicate idempotent retries on
message id, and recover committed log state across restarts and segment
rotation (delete-on-ack: segments are reclaimed once every entry in them is
acked, not by wall-clock age or size).
Gate Inventory:
- apps/relay/tests/relay_core.rs; apps/relay/tests/durable.rs; apps/relay/tests/segments.rs
Surfaces:
- Rust API: `Relay` - append, dedupe, subject/shard ordering.; Disk: segment log - durable local log lifecycle.
EC Dimensions:
- behavior: `cargo test -p relay --test relay_core --test durable --test segments` - ordered log and recovery conformance

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| per-subject-shard-append-ordering | epic | - | implemented | passing | conformance | apps/relay/tests/relay_core.rs |
| segment-rotation-and-retention-recovery | epic | - | implemented | passing | conformance | apps/relay/tests/segments.rs |

### Work Queue Lifecycle

ID: work-queue-lifecycle
Root WI: -
Status: auditing
Type: RuntimeTool
Required Verification: conformance
Promise:
Deliver competing-worker queue semantics with epoch-fenced leases, heartbeat
extension, idempotent ack, expired-lease reclaim, and redelivery liveness.
Gate Inventory:
- apps/relay/tests/work_queue_api.rs; apps/relay/tests/reconciler.rs
Surfaces:
- HTTP: `lease` + `heartbeat` + `ack` - competing-worker queue lifecycle.; Rust API: work queue engine - lease cursor, epoch fencing, redelivery.
EC Dimensions:
- behavior: `cargo test -p relay --test work_queue_api --test reconciler` - lease/ack/reclaim conformance

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| lease-heartbeat-ack-lifecycle | change | #1850 | implemented | passing | conformance | apps/relay/tests/work_queue_api.rs |
| lease-reclaim-liveness | change | #1850 | implemented | passing | conformance | apps/relay/tests/reconciler.rs |

### HTTP/OpenAPI Worker Protocol

ID: http-openapi-worker-protocol
Root WI: 108
Status: auditing
Type: RuntimeTool
Required Verification: conformance
Promise:
Expose Relay's broker and worker lifecycle through a polyglot h2c/OpenAPI
contract so non-Rust workers can publish, consume, lease, heartbeat, and ack.
Gate Inventory:
- apps/relay/tests/worker_loop.rs; apps/relay/docs/worker-protocol.md
Surfaces:
- HTTP: `/openapi.json` - h2c worker API - publish, consume, lease, heartbeat, ack.; OpenAPI: `/openapi.json` and docs/worker-protocol.md - polyglot worker contract.
EC Dimensions:
- behavior: `cargo test -p relay --test worker_loop` - worker protocol conformance

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| http-worker-protocol-parity | epic | 108 | implemented | passing | conformance | apps/relay/tests/worker_loop.rs; apps/relay/docs/worker-protocol.md |
| served-openapi-contract | epic | 108 | implemented | passing | conformance | apps/relay/tests/worker_loop.rs; apps/relay/docs/worker-protocol.md |

### Raft HA

ID: raft-ha
Root WI: 1207
Status: auditing
Type: RuntimeTool
Required Verification: conformance, dogfood
Promise:
Provide a raft-backed HA path on the shared raft-runtime driver that converges in
process, persists hard state and the applied-index floor, serves through real
h2c nodes, and can be dogfooded through a Kubernetes kind failover smoke.
Gate Inventory:
- apps/relay/src/raft.rs; apps/relay/HA.md; apps/relay/tests/raft_core.rs; apps/relay/tests/raft_persistence.rs; apps/relay/tests/raft_cluster.rs; apps/relay/scripts/kind-failover-smoke.sh
Surfaces:
- CLI: `relay` + `libs/raft-runtime` + `RelayStateMachine` + `apps/relay/k8s` - auto-mode raft node on `libs/raft-runtime` (no flags; downward-API env flips replica mode).; Rust API: `RelayStateMachine` - publish replication with the fsynced applied-index floor.; K8s: operator-rendered StatefulSet + `apps/relay/k8s` - deployment shapes.
EC Dimensions:
- stability: `cargo test -p relay --test raft_core --test raft_persistence --test raft_cluster` - raft convergence, persistence, and h2c cluster smoke

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| raft-runtime-adoption-auto-mode | epic | 1207 | implemented | passing | conformance | apps/relay/src/raft.rs; apps/relay/HA.md |
| in-process-raft-convergence | epic | - | implemented | passing | conformance | apps/relay/tests/raft_core.rs |
| durable-raft-hard-state-restore | epic | - | implemented | passing | conformance | apps/relay/tests/raft_persistence.rs |
| real-h2c-raft-cluster-smoke | epic | - | implemented | passing | dogfood | apps/relay/tests/raft_cluster.rs |
| kubernetes-kind-failover-smoke | epic | - | implemented | verified | dogfood | apps/relay/scripts/kind-failover-smoke.sh; apps/relay/k8s |
