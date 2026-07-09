# relay

## Brief

`relay` is the online **single-cast pull work-queue broker** in the Axiom stack
(RabbitMQ/SQS-shaped): a producer publishes a task, a worker **pulls** (leases)
it, runs it, and acks — each message is delivered exactly once to one of the
competing consumers, then reclaimed (**delete-on-ack**). It owns work-queue
leasing with lease-expiry redelivery, dead-lettering, priority, short delayed
visibility, h2c/OpenAPI transport, and raft-backed HA. Payloads stay opaque JSON
so higher-level systems (loom, worker runtimes) use relay without relay learning
their domain model.

Relay is the pull side of Axiom's queue family; [`defer`](../defer) is the push
side for scheduled HTTP dispatch. Relay is deliberately **not** a Kafka/pub-sub
platform: durable topic replay/retention is [`tape`](../tape)'s job, and
push/ETA task dispatch is [`defer`](../defer)'s. Concurrency is
**client-driven** — more workers = more throughput; relay has no server-side
rate/concurrency governor (that is `defer`). Tenancy is a deployment concern —
relay is single-tenant per deployment; run one per tenant (k8s namespace). See
the boundary notes in the ecosystem docs.

Priority is shared with `defer` as an unsigned byte: `0` is lowest, `255` is
highest, the default is `10`, and higher values lease first. In relay, priority
orders only entries that are already visible in the work queue; long-horizon ETA,
rate limiting, and HTTP target dispatch remain Defer concerns.

Relay ships as **one binary**: bare `relay` serves (h2c broker; raft-host
auto-mode HA — see [`HA.md`](HA.md)), and the same bin carries the offline
`spec`/`spec gen`, `backup`, `k8s crd|operator|instance`, `dockerfile render`,
and shared `llm`/`upgrade`/`issue` verbs (WIs #1204-#1209, the
service-archetype convergence).

## Capabilities

The baseline capabilities selected by aw.toml's `service` umbrella profile are
mandatory for this long-running broker class. They do not replace Relay's
product capabilities; ordered log, work queue, worker protocol, and raft HA
remain first-class domain roots.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| CLI Interface | 1204 | implemented | passing | conformance | not_ready | mandatory baseline: single `relay` bin — serve default plus spec/backup/k8s/dockerfile verbs and served OpenAPI |
| CLI Standard Surface | 1204 | implemented | passing | conformance | not_ready | mandatory baseline: shared `cli-std` llm/upgrade/issue surface with build-stamp provenance |
| Chainable Output Conformance | - | implemented | planned | smoke | not_ready | mandatory baseline: raw artifact streams stay unwrapped; `next:`/terminal markers on write/backup outputs remain open |
| Competitive Broker Feature Parity | - | implemented | planned | dogfood | not_ready | mandatory baseline: RabbitMQ/NATS JetStream/Redis Streams work-queue replacement breadth; kind failover remains open |
| Competitive Broker Performance | 125 | implemented | planned | dogfood | not_ready | mandatory baseline: vat-isolated meter throughput ratchet; external broker arena is advisory |
| Long-Running Stability | - | implemented | passing | dogfood | not_ready | mandatory baseline: recovery, retention, lease reclaim, graceful drain, backup, and raft restart/failover |
| Security Hardening | 1206 | implemented | passing | negative | not_ready | mandatory baseline: RELAY_AUTH bearer contract + tokenless probes + peer-TLS config shipped; negative/network-policy gates remain open |
| HTTP/2 API List | 108 | implemented | passing | conformance | not_ready | mandatory baseline: concise h2c producer, worker, probe, and OpenAPI route list with an offline spec twin |
| Standard Operational Endpoints | 1205 | implemented | passing | conformance | not_ready | mandatory baseline: one-port `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs` surface plus offline `relay spec` |
| EC Gates Configured | 125 | implemented | passing | conformance | not_ready | mandatory baseline: aw.toml EC inventory, vat meter/guard runners, and external-contracts evidence stay wired |
| Kubernetes-Native Deployment | 1208 | implemented | planned | dogfood | not_ready | mandatory baseline: RelaySpec CRD/operator/instance render, auto-mode StatefulSet topology, and kind failover smoke |
| Primary Replicas | 1207 | implemented | planned | dogfood | not_ready | mandatory baseline: raft-host auto-mode leader/follower primary-replica topology |
| Durable Ordered Log | - | implemented | passing | conformance | not_ready | domain: per-subject append, dedupe, and segment lifecycle |
| Work Queue Lifecycle | - | implemented | passing | conformance | not_ready | domain: lease, heartbeat, ack, redelivery, and reconciler behavior |
| HTTP/OpenAPI Worker Protocol | 108 | implemented | passing | conformance | not_ready | domain: polyglot h2c worker contract |
| Raft HA | 1207 | implemented | planned | dogfood | not_ready | domain: raft-host RelayStateMachine, auto-mode topology, applied-index floor, and kind failover |

### CLI Interface

ID: cli-interface
Type: RuntimeTool
Surfaces: CLI: `relay` - single bin; bare `relay` serves the h2c broker (auto-mode raft HA) and carries the spec/backup/k8s/dockerfile verbs.; HTTP: `/openapi.json` - machine-readable worker contract served by the binary.
EC Dimensions: behavior: `cargo test -p relay --test worker_loop --test spec_cli --test deploy_cli` - binary-facing contract, offline spec, and deploy-verb smoke
Root WI: 1204
Status: auditing
Required Verification: conformance
Promise:
Expose relay as one runnable binary with a stable process entrypoint — serve by
default, locally and as the Kubernetes raft node — plus offline spec and deploy
verbs and a served OpenAPI contract for non-Rust clients.
Gate Inventory:
- apps/relay/tests/worker_loop.rs; apps/relay/tests/spec_cli.rs; apps/relay/tests/deploy_cli.rs; apps/relay/tests/raft_config.rs; apps/relay/tests/raft_cluster.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| single-relay-bin-serve-default | epic | 1204 | implemented | passing | conformance | apps/relay/src/bin/relay.rs; apps/relay/tests/worker_loop.rs |
| auto-mode-raft-node-entrypoint | epic | 1207 | implemented | passing | conformance | apps/relay/src/bin/relay.rs; apps/relay/tests/raft_config.rs; apps/relay/tests/raft_cluster.rs |
| served-openapi-contract | epic | 108 | implemented | passing | conformance | apps/relay/tests/worker_loop.rs; apps/relay/docs/worker-protocol.md |

### CLI Standard Surface

ID: cli-standard-surface
Type: RuntimeTool
Surfaces: CLI: `relay llm` - offline agent self-doc topics (outline, http-api, operations) required by the ecosystem CLI convention.; CLI: `relay upgrade` - shared self-update and `--check` surface through `cli-std`.; CLI: `relay issue search`, `relay issue view`, `relay issue create` - shared tracker read/write surface scoped to `app:relay`.
EC Dimensions: behavior: `cargo test -p relay --test spec_cli` - the llm topic surface documents every shipped serve/deploy/backup knob through the compiled binary
Root WI: 1204
Status: auditing
Required Verification: conformance
Promise:
Ship the mandatory shared `cli-std` surface (llm/upgrade/issue) every ecosystem
CLI owes, backed by build-stamp provenance, without blurring it into relay's
domain serve/spec/backup/deploy verbs.
Gate Inventory:
- apps/relay/src/llm.rs; apps/relay/src/bin/relay.rs; apps/relay/tests/spec_cli.rs; libs/cli-std/src

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-llm-topic-surface | epic | 1204 | implemented | passing | conformance | apps/relay/src/llm.rs; apps/relay/tests/spec_cli.rs |
| shared-upgrade-issue-surface | epic | 1204 | implemented | passing | smoke | apps/relay/src/bin/relay.rs; libs/cli-std/src |

### Chainable Output Conformance

ID: chainable-output-conformance
Type: RuntimeTool
Surfaces: CLI: `relay spec`, `relay k8s crd|operator|instance render`, and `relay dockerfile render` without `--out` - raw artifact/data streams that intentionally stay unwrapped bytes.; CLI: `--out` file writes, `relay backup`, `relay issue ...`, and `relay upgrade --check` - operational outputs that owe a runnable next step or explicit terminal marker.
EC Dimensions: behavior: `cargo test -p relay --test deploy_cli --test spec_cli` - render/spec stdout stays raw parseable artifact bytes through the compiled binary
Root WI: -
Status: auditing
Required Verification: conformance
Promise:
Keep relay's operational CLI outputs chainable per the CLI convention: raw
artifact streams (spec, renders) stay unwrapped bytes, while file-writing and
admin verbs grow explicit `next:`/terminal markers.
Gate Inventory:
- apps/relay/tests/deploy_cli.rs; apps/relay/tests/spec_cli.rs; apps/relay/src/bin/relay.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| raw-artifact-streams-stay-unwrapped | epic | 1208 | implemented | passing | conformance | apps/relay/tests/deploy_cli.rs; apps/relay/tests/spec_cli.rs |
| shared-issue-upgrade-terminal-markers | epic | 1204 | implemented | passing | smoke | libs/cli-std/src |
| next-markers-on-write-and-backup-outputs | epic | - | planned | planned | smoke | apps/relay/src/bin/relay.rs (write_or_print prints `wrote <path>` without a `next:` continuation) |

### Competitive Broker Feature Parity

ID: competitor-feature-parity
Type: RuntimeTool
Surfaces: Rust API: `Relay` - durable ordered log, work-queue lease, and ack primitives.; HTTP: `publish`, `lease`, `heartbeat`, `ack` - RabbitMQ/SQS-style single-cast work-queue workflows over h2c.; CLI: `relay` - auto-mode failover-capable broker node.
EC Dimensions: behavior: `cargo test -p relay --test relay_core --test work_queue_api --test worker_loop --test raft_core --test raft_persistence --test raft_cluster` - functional parity conformance for core broker workflows
Root WI: -
Status: auditing
Required Verification: conformance, dogfood
Promise:
Cover the baseline single-cast work-queue broker functions Relay needs to
compete with RabbitMQ, NATS JetStream, Redis Streams, and Dragonfly in Axiom
workloads. Redpanda/Kafka-class replay journals are tape competitors, not relay
competitors.
Gate Inventory:
- apps/relay/tests/relay_core.rs; apps/relay/tests/http2_transport.rs; apps/relay/tests/work_queue_api.rs; apps/relay/tests/worker_loop.rs; apps/relay/tests/raft_core.rs; apps/relay/tests/raft_persistence.rs; apps/relay/tests/raft_cluster.rs; apps/relay/scripts/kind-failover-smoke.sh

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| ordered-log-queue-and-raft-feature-breadth | epic | - | implemented | passing | conformance | apps/relay/tests/relay_core.rs; apps/relay/tests/work_queue_api.rs; apps/relay/tests/raft_core.rs |
| per-subject-shard-append-ordering | epic | - | implemented | passing | conformance | apps/relay/tests/relay_core.rs |
| lease-heartbeat-ack-lifecycle | epic | - | implemented | passing | conformance | apps/relay/tests/work_queue_api.rs |
| http-worker-protocol-parity | epic | 108 | implemented | passing | conformance | apps/relay/tests/worker_loop.rs; apps/relay/docs/worker-protocol.md |
| in-process-raft-convergence | epic | - | implemented | passing | conformance | apps/relay/tests/raft_core.rs |
| durable-raft-hard-state-restore | epic | - | implemented | passing | conformance | apps/relay/tests/raft_persistence.rs |
| real-h2c-raft-cluster-smoke | epic | - | implemented | passing | dogfood | apps/relay/tests/raft_cluster.rs |
| kubernetes-kind-failover-smoke | epic | - | implemented | planned | dogfood | apps/relay/scripts/kind-failover-smoke.sh; apps/relay/k8s |

### Competitive Broker Performance

ID: competitor-performance
Type: RuntimeTool
Surfaces: Meter/Vat: `apps/relay/vat.toml#meter-perf` - isolated meter execution for the throughput ratchet.; Harness: `cargo run -p relay --release --example bench_compare -- --backend <target>` - durable-only closed-loop external broker comparison across relay, RabbitMQ, NATS JetStream, Redis Streams, and Dragonfly.; Arena: `apps/arena/examples/relay-vs-rabbitmq-nats-redis.toml` - advisory normalized ratio wrapper.; Rust bench: `relay_bench` - local broker throughput baseline.
EC Dimensions: efficiency: `cd apps/relay && ../../target/debug/vat run meter-perf` - meter-owned throughput model and ratchet conformance; behavior: `cargo test -p relay --test work_queue_throughput --test perf_gate` - deterministic local gate shape
Root WI: 125
Status: auditing
Required Verification: dogfood
Promise:
Keep Relay's performance claims tied to repeatable throughput tests and an
vat-isolated meter gate and keep the external competitor comparison against
RabbitMQ, NATS JetStream, Redis Streams, and Dragonfly as advisory dogfood until
the native broker harness is promoted into a required gate.
Gate Inventory:
- apps/relay/vat.toml; apps/relay/tests/work_queue_throughput.rs; apps/relay/tests/perf_gate.rs; apps/relay/src/perf_gate.rs; apps/relay/examples/bench_compare.rs; apps/arena/examples/relay-vs-rabbitmq-nats-redis.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| o-1-lease-cursor-throughput | epic | - | implemented | passing | conformance | apps/relay/tests/work_queue_throughput.rs |
| normalized-win-ratchet-decision-model | epic | 125 | implemented | passing | conformance | apps/relay/tests/perf_gate.rs |
| vat-meter-throughput-gate | epic | 125 | implemented | planned | dogfood | apps/relay/vat.toml#meter-perf |
| external-broker-comparison | epic | 125 | implemented | planned | dogfood | apps/relay/examples/bench_compare.rs; apps/arena/examples/relay-vs-rabbitmq-nats-redis.toml |

### Long-Running Stability

ID: long-running-stability
Type: RuntimeTool
Surfaces: CLI: `relay` - durable serve process with reconciler, graceful drain, and auto-mode failover.; K8s: `apps/relay/k8s` - StatefulSet-oriented raft deployment.; CLI: `relay backup` - consistent snapshot capture from a running node.
EC Dimensions: stability: `cargo test -p relay --test durable --test segments --test reconciler --test raft_persistence --test raft_cluster` - recovery, retention, lease reclaim, and failover conformance
Root WI: -
Status: auditing
Required Verification: conformance, dogfood
Promise:
Run as a long-lived broker without losing committed entries, leaking stuck
leases forever, or corrupting recovery state across restarts, segment rotation,
graceful drains, and leader failover — with a backup/restore path for the rest.
Gate Inventory:
- apps/relay/tests/durable.rs; apps/relay/tests/segments.rs; apps/relay/tests/reconciler.rs; apps/relay/tests/raft_persistence.rs; apps/relay/tests/raft_cluster.rs; apps/relay/tests/backup.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| durable-power-safe-recovery | epic | - | implemented | passing | conformance | apps/relay/tests/durable.rs |
| segment-rotation-and-retention-recovery | epic | - | implemented | passing | conformance | apps/relay/tests/segments.rs |
| lease-reclaim-liveness | epic | - | implemented | passing | conformance | apps/relay/tests/reconciler.rs |
| graceful-drain-readiness | epic | 1205 | implemented | passing | conformance | apps/relay/tests/http2_transport.rs |
| backup-restore-consistent-snapshot | epic | 1209 | implemented | passing | conformance | apps/relay/tests/backup.rs; apps/relay/HA.md |
| raft-hard-state-restart-safety | epic | - | implemented | passing | conformance | apps/relay/tests/raft_persistence.rs |
| failover-without-committed-loss | epic | - | implemented | passing | dogfood | apps/relay/tests/raft_cluster.rs |

### Security Hardening

ID: security-hardening
Type: RuntimeTool
Surfaces: HTTP: `Authorization: Bearer` on the /v1 data plane - `RELAY_AUTH=off|required` + `RELAY_TOKEN_REGISTRY_FILE` role registry; probes stay tokenless.; Env: `RELAY_PEER_TLS_CERT|KEY|CA`, `RELAY_PEER_MTLS` - peer mTLS material validated fail-fast at startup.; Guard/Vat: `apps/relay/vat.toml#guard-security` - isolated guard scan with meter runtime evidence.; K8s: `apps/relay/k8s` - deployment boundary for future network policy and identity.
EC Dimensions: security: `cd apps/relay && ../../target/debug/vat run guard-security` - guard-owned static/runtime evidence for the in-process opaque payload boundary; behavior: `cargo test -p relay --test auth` - bearer authn/z conformance and the tokenless probe boundary
Root WI: 1206
Status: auditing
Required Verification: negative
Promise:
Keep Relay safe as a long-running broker: opaque payload boundaries, the shared
bearer-token contract on the data plane (probes exempt), fail-fast peer-TLS
material validation, and explicit negative gates for request limits and
network policy before production readiness.
Gate Inventory:
- apps/relay/vat.toml; apps/relay/tests/auth.rs; apps/relay/src/auth.rs; apps/relay/src/peer_tls.rs; apps/relay/tests/relay_core.rs; apps/relay/tests/worker_loop.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| opaque-payload-boundary | epic | - | implemented | passing | smoke | apps/relay/tests/relay_core.rs; apps/relay/tests/worker_loop.rs |
| bearer-auth-token-registry | epic | 1206 | implemented | passing | conformance | apps/relay/tests/auth.rs; apps/relay/src/auth.rs |
| peer-tls-material-validation | epic | 1209 | implemented | passing | conformance | apps/relay/src/peer_tls.rs; apps/relay/HA.md |
| guard-static-runtime-evidence | epic | - | implemented | planned | negative | apps/relay/vat.toml#guard-security |
| request-limit-and-malformed-frame-negative-tests | epic | - | planned | planned | negative | apps/relay/vat.toml#guard-security |
| network-policy-and-peer-mtls-termination | epic | - | planned | planned | negative | pending raft-host TLS seam (peer RPCs stay cleartext h2c; see apps/relay/HA.md) |

### HTTP/2 API List

ID: http2-api-list
Type: RuntimeTool
Surfaces: HTTP: publish, lease, heartbeat, ack, consume, `/openapi.json`, and probe routes - concise h2c API list for producers and workers.; CLI: `relay spec` - offline OpenAPI twin of the served contract.; Docs: `apps/relay/docs/worker-protocol.md` - endpoint contract summary.
EC Dimensions: behavior: `cargo test -p relay --test http2_transport --test worker_loop` - h2c transport and worker protocol conformance
Root WI: 108
Status: auditing
Required Verification: conformance
Promise:
Publish Relay's supported HTTP/2 API as a compact producer and worker endpoint
inventory, with OpenAPI/docs pointers, without making OpenAPI completeness the
capability definition.
Gate Inventory:
- apps/relay/tests/http2_transport.rs; apps/relay/tests/worker_loop.rs; apps/relay/tests/spec_cli.rs; apps/relay/docs/worker-protocol.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| h2c-publish-and-consume-route-list | epic | - | implemented | passing | conformance | apps/relay/tests/http2_transport.rs |
| worker-lease-heartbeat-ack-route-list | epic | 108 | implemented | passing | conformance | apps/relay/tests/worker_loop.rs; apps/relay/docs/worker-protocol.md |
| served-openapi-contract | epic | 108 | implemented | passing | conformance | apps/relay/tests/worker_loop.rs |
| offline-spec-openapi-list | epic | 1209 | implemented | passing | conformance | apps/relay/tests/spec_cli.rs |

### Standard Operational Endpoints

ID: standard-operational-endpoints
Type: Service
Surfaces: HTTP: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs` - auth-exempt liveness, readiness, Prometheus scrape, live-spec, and Swagger UI endpoints on the one serve port via `service_http::standard_probe_routes`.; CLI: `relay spec` - offline OpenAPI evidence for the same operational contract when no server is running.
EC Dimensions: behavior: `cargo test -p relay --test http2_transport` - probe surface, drain flip, and Prometheus metrics over h2c and HTTP/1.1; behavior: `cargo test -p relay --test auth` - the five endpoints stay tokenless when RELAY_AUTH=required; behavior: `cargo test -p relay --test spec_cli` - offline spec mirrors the served contract
Root WI: 1205
Status: auditing
Required Verification: conformance
Promise:
Expose the standard one-port operational surface the service trait requires —
probes, metrics scrape, live spec, and Swagger UI stay always-on and
auth-exempt on the serve port, with `relay spec` as the offline twin.
Gate Inventory:
- apps/relay/tests/http2_transport.rs; apps/relay/tests/auth.rs; apps/relay/tests/spec_cli.rs; apps/relay/src/server.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| service-http-standard-probe-routes | epic | 1205 | implemented | passing | conformance | apps/relay/tests/http2_transport.rs; apps/relay/src/server.rs |
| drain-flips-readyz | epic | 1205 | implemented | passing | conformance | apps/relay/tests/http2_transport.rs |
| probes-stay-tokenless-under-auth | epic | 1206 | implemented | passing | conformance | apps/relay/tests/auth.rs |
| offline-spec-matches-served-contract | epic | 1209 | implemented | passing | conformance | apps/relay/tests/spec_cli.rs |

### EC Gates Configured

ID: ec-gates-configured
Type: Devops
Surfaces: Config: `apps/relay/aw.toml` - AW EC inventory and generated efficiency/security cases with dispatch commands.; Config: `apps/relay/vat.toml` - vat-managed `meter-perf` and `guard-security` runners.; Tests: `apps/relay/tests/benchmark_relay_competitor_performance_meter_gate.rs`, `apps/relay/tests/security_relay_security_hardening_guard_scan.rs` - generated EC evidence stubs tied back to the inventory.
EC Dimensions: efficiency: `cd apps/relay && ../../target/debug/vat run meter-perf` - meter-owned throughput ratchet dispatch inside vat; security: `cd apps/relay && ../../target/debug/vat run guard-security` - guard-owned security evidence dispatch inside vat
Root WI: 125
Status: auditing
Required Verification: conformance
Promise:
Keep relay's service-trait EC baseline explicit and runnable: aw.toml owns the
EC inventory, vat owns the meter/guard runners, and external-contracts/ carries
the evidence contracts each gate closes against.
Gate Inventory:
- apps/relay/aw.toml; apps/relay/vat.toml; apps/relay/meter-relay-performance.toml; apps/relay/guard-relay-security.toml; apps/relay/external-contracts

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| aw-ec-generated-inventory-and-dispatch | epic | 125 | implemented | passing | conformance | apps/relay/aw.toml |
| vat-managed-meter-and-guard-runners | epic | 125 | implemented | passing | conformance | apps/relay/vat.toml; apps/relay/meter-relay-performance.toml; apps/relay/guard-relay-security.toml |
| external-contract-evidence-docs | epic | - | implemented | passing | conformance | apps/relay/external-contracts |

### Kubernetes-Native Deployment

ID: kubernetes-native-deployment
Type: Devops
Surfaces: CLI: `relay k8s crd render`, `relay k8s operator render`, `relay k8s instance render --profile dev|staging|prod|template`, `relay k8s operator run` - RelaySpec CRD, controller, and instance deploy verbs (operator feature).; CLI: `relay dockerfile render --variant source|release` - image fixtures rendered from the binary.; K8s: `apps/relay/k8s` - single-node direct install for kind/smoke.; Script: `apps/relay/scripts/kind-failover-smoke.sh` - live failover dogfood path.
EC Dimensions: behavior: `cargo test -p relay --test raft_config --test raft_cluster` - node topology config and real h2c cluster smoke; behavior: `cargo test -p relay --test deploy_cli` - offline render verbs and committed-fixture byte-equality; stability: `apps/relay/scripts/kind-failover-smoke.sh` - kind failover dogfood
Root WI: 1208
Status: auditing
Required Verification: conformance, dogfood
Promise:
Run Relay as a Kubernetes-native broker: an operator-rendered StatefulSet with
the downward-API env auto-mode reads, offline CRD/operator/instance/dockerfile
render verbs whose committed files are byte-equal fixtures, and a kind failover
path for deployment dogfood.
Gate Inventory:
- apps/relay/k8s; apps/relay/tests/deploy_cli.rs; apps/relay/tests/operator.rs; apps/relay/scripts/kind-failover-smoke.sh; apps/relay/tests/raft_config.rs; apps/relay/tests/raft_cluster.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| statefulset-raft-service-topology | epic | - | implemented | planned | dogfood | apps/relay/k8s |
| auto-mode-node-topology-config | epic | 1207 | implemented | passing | conformance | apps/relay/tests/raft_config.rs |
| relayspec-crd-operator-instance-render | epic | 1208 | implemented | passing | conformance | apps/relay/tests/deploy_cli.rs; apps/relay/tests/operator.rs |
| dockerfile-render-fixtures | epic | 1208 | implemented | passing | conformance | apps/relay/tests/deploy_cli.rs; apps/relay/Dockerfile |
| kubernetes-kind-failover-smoke | epic | - | implemented | planned | dogfood | apps/relay/scripts/kind-failover-smoke.sh |

### Primary Replicas

ID: primary-replicas
Type: Runtime
Surfaces: CLI: `relay` - auto-mode raft-backed primary/replica broker node (`REPLICAS_PER_SHARD` > 1 flips replica mode).; Rust API: `RelayStateMachine` - publish replication, snapshot/compaction, and the applied-index floor.; K8s: operator-rendered StatefulSet - replica pods with stable identities.
EC Dimensions: stability: `cargo test -p relay --test raft_core --test raft_persistence --test raft_cluster` - leader/follower convergence, hard-state restore, and h2c cluster smoke
Root WI: 1207
Status: auditing
Required Verification: conformance, dogfood
Promise:
Support a primary/replica broker topology where the raft leader owns writes,
followers replicate committed state, and failover preserves the ordered-log and
work-queue API contract.
Gate Inventory:
- apps/relay/tests/raft_core.rs; apps/relay/tests/raft_persistence.rs; apps/relay/tests/raft_cluster.rs; apps/relay/HA.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| in-process-leader-follower-convergence | epic | - | implemented | passing | conformance | apps/relay/tests/raft_core.rs |
| durable-primary-replica-hard-state | epic | - | implemented | passing | conformance | apps/relay/tests/raft_persistence.rs |
| real-h2c-replica-cluster-smoke | epic | - | implemented | passing | dogfood | apps/relay/tests/raft_cluster.rs |

### Durable Ordered Log

ID: durable-ordered-log
Type: Runtime
Surfaces: Rust API: `Relay` - append, dedupe, subject/shard ordering.; Disk: segment log - durable local log lifecycle.
EC Dimensions: behavior: `cargo test -p relay --test relay_core --test durable --test segments` - ordered log and recovery conformance
Root WI: -
Status: auditing
Required Verification: conformance
Promise:
Append messages in per-subject/shard order, deduplicate idempotent retries on
message id, and recover committed log state across restarts and segment
rotation (delete-on-ack: segments are reclaimed once every entry in them is
acked, not by wall-clock age or size).
Gate Inventory:
- apps/relay/tests/relay_core.rs; apps/relay/tests/durable.rs; apps/relay/tests/segments.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| per-subject-shard-append-ordering | epic | - | implemented | passing | conformance | apps/relay/tests/relay_core.rs |
| segment-rotation-and-retention-recovery | epic | - | implemented | passing | conformance | apps/relay/tests/segments.rs |

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
- apps/relay/tests/work_queue_api.rs; apps/relay/tests/reconciler.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| lease-heartbeat-ack-lifecycle | epic | - | implemented | passing | conformance | apps/relay/tests/work_queue_api.rs |
| lease-reclaim-liveness | epic | - | implemented | passing | conformance | apps/relay/tests/reconciler.rs |

### HTTP/OpenAPI Worker Protocol

ID: http-openapi-worker-protocol
Type: Runtime
Surfaces: HTTP: h2c worker API - publish, consume, lease, heartbeat, ack.; OpenAPI: `/openapi.json` and docs/worker-protocol.md - polyglot worker contract.
EC Dimensions: behavior: `cargo test -p relay --test worker_loop` - worker protocol conformance
Root WI: 108
Status: auditing
Required Verification: conformance
Promise:
Expose Relay's broker and worker lifecycle through a polyglot h2c/OpenAPI
contract so non-Rust workers can publish, consume, lease, heartbeat, and ack.
Gate Inventory:
- apps/relay/tests/worker_loop.rs; apps/relay/docs/worker-protocol.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| http-worker-protocol-parity | epic | 108 | implemented | passing | conformance | apps/relay/tests/worker_loop.rs; apps/relay/docs/worker-protocol.md |
| served-openapi-contract | epic | 108 | implemented | passing | conformance | apps/relay/tests/worker_loop.rs; apps/relay/docs/worker-protocol.md |

### Raft HA

ID: raft-ha
Type: Runtime
Surfaces: CLI: `relay` - auto-mode raft node on `libs/raft-host` (no flags; downward-API env flips replica mode).; Rust API: `RelayStateMachine` - publish replication with the fsynced applied-index floor.; K8s: operator-rendered StatefulSet + `apps/relay/k8s` - deployment shapes.
EC Dimensions: stability: `cargo test -p relay --test raft_core --test raft_persistence --test raft_cluster` - raft convergence, persistence, and h2c cluster smoke
Root WI: 1207
Status: auditing
Required Verification: conformance, dogfood
Promise:
Provide a raft-backed HA path on the shared raft-host driver that converges in
process, persists hard state and the applied-index floor, serves through real
h2c nodes, and can be dogfooded through a Kubernetes kind failover smoke.
Gate Inventory:
- apps/relay/src/raft.rs; apps/relay/HA.md; apps/relay/tests/raft_core.rs; apps/relay/tests/raft_persistence.rs; apps/relay/tests/raft_cluster.rs; apps/relay/scripts/kind-failover-smoke.sh

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| raft-host-adoption-auto-mode | epic | 1207 | implemented | passing | conformance | apps/relay/src/raft.rs; apps/relay/HA.md |
| in-process-raft-convergence | epic | - | implemented | passing | conformance | apps/relay/tests/raft_core.rs |
| durable-raft-hard-state-restore | epic | - | implemented | passing | conformance | apps/relay/tests/raft_persistence.rs |
| real-h2c-raft-cluster-smoke | epic | - | implemented | passing | dogfood | apps/relay/tests/raft_cluster.rs |
| kubernetes-kind-failover-smoke | epic | - | implemented | planned | dogfood | apps/relay/scripts/kind-failover-smoke.sh; apps/relay/k8s |
