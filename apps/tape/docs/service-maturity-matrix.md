<!-- HANDWRITE-BEGIN gap="missing-generator:logic:cross-service-maturity-matrix" tracker="#768" reason="Delivery audit matrix comparing the read-only Lumen baseline with Tape, Relay, Defer, and their shared-library owners." -->
# Lumen / Tape / Relay / Defer maturity matrix

This matrix records the delivered boundary, not an instruction to copy Lumen
domain code. Lumen is the read-only maturity reference. A row is at parity when
the same operational promise is owned by a shared library and the app supplies
only domain state, policy, schema, route wiring, and deployment defaults.

## Non-domain parity and ownership

| Operational promise | Shared owner | Lumen reference | Tape evidence | Relay evidence | Defer evidence | Decision |
|---|---|---|---|---|---|---|
| HTTP/1.1 + h2c, errors, probes, drain | `service-http`, `server-http`, `server-lifecycle`, `transport-h2c` | `apps/lumen/tests/behavior_lumen_cli_interface_protocol_transport.rs`, `spec_cli.rs` | `tests/http_transport.rs` | `tests/http2_transport.rs` | `tests/http_api.rs` | parity |
| Served/offline OpenAPI and generated clients | `openapi-codegen`, `service-http` | `tests/spec_cli.rs`, `generated_clients_crud_e2e.rs` | `tests/cli_contract.rs`, `src/openapi.rs` | `tests/spec_cli.rs`, `docs/worker-protocol.md` | `tests/cli_contract.rs`, `src/openapi.rs` | parity |
| OTLP/W3C context, Prometheus metrics, lifecycle counters | `service-observability`, `metrics-prometheus`, `service-http` | `tests/behavior_lumen_claim_observability_*` | `tests/shared_otlp_tracing.rs`, `observability_assets.rs` | `tests/http2_transport.rs`, K8s observability component | `tests/http_api.rs`, K8s observability component | parity |
| Authn/z, audited rotation, admission, stable errors | `service-auth`, `service-http` | `tests/auth_e2e.rs`, `admission_e2e.rs` | `tests/service_auth.rs`, `service_admission.rs`, `audit_contract.rs` | `tests/auth.rs`, `service_admission.rs` | `tests/service_auth.rs`, `service_admission.rs` | parity |
| Durable consensus, snapshot/restart, committed ownership/fencing | `raft-core`, `raft-runtime`, `storage-durable` | `tests/stability_lumen_topology_existing_raft_replica_sync.rs` | `tests/raft_cluster.rs`, `raft_persistence.rs`, `raft_failover.rs` | `tests/raft_cluster.rs`, `raft_persistence.rs` | `tests/raft_scheduler.rs` (same-directory restart + repeated failover) | parity; app owns state machine only |
| Backup transport and cold restore/bootstrap | `service-backup` | `tests/backup_restore_e2e.rs` | `tests/backup.rs`, `bootstrap.rs` | `tests/backup.rs` | `tests/http_api.rs`, Kind lifecycle gate | parity |
| Authenticated peer transport | `peer-tls`, `raft-runtime` | shared Raft transport boundary | `tests/raft_peer_mtls.rs` | `tests/raft_peer_mtls.rs` | `tests/raft_peer_mtls.rs` | parity; no app-local TLS stack |
| Bounded executor concurrency | `service-executor` | domain-independent runner boundary | not applicable to read-only replay | committed Relay ownership before delivery | `tests/http_dispatch.rs` proves committed ownership before delivery plus accepted-HTTP/lost-fence retry with a stable idempotency key | shared runner; policy remains domain-owned |
| K8s base/overlays/components, CRD/operator/instance, PDB/PVC/NetworkPolicy | `service-k8s` | `apps/lumen/k8s`, `tests/operator_render.rs` | `tests/deploy_cli.rs`, `operator.rs`, `direct_k8s_assets.rs`, `scripts/kind-e2e.sh` | same test classes plus `scripts/kind-failover-smoke.sh` | same test classes plus `scripts/kind-e2e.sh` | parity |
| HPA/topology safety | `service-k8s` topology primitives; app CR policy | Lumen may HPA stateless serving replicas | voter StatefulSet is explicitly not HPA-managed | voter StatefulSet is explicitly not HPA-managed | voter StatefulSet is explicitly not HPA-managed | deliberate difference: ordinary HPA must not churn consensus voters; disk scales by shard, HA by fixed replication factor, serving capacity by topology-aware policy |
| Long-running resource stability | `service-observability/scripts/soak-metrics.sh` | `apps/lumen/scripts/soak.sh`, Rig endurance cases | 60 s replay/checkpoint: errors 0, RSS 14,464→14,464 KiB, FD 13→13, threads 11→11, p99 1→1 ms | 60 s fixed lease: errors 0, RSS 14,352→14,352 KiB, FD 14→14, threads 11→11, p99 1→1 ms | snapshot/cache warmup + 60 s: errors 0, RSS 41,584→42,192 KiB, FD 18→18, threads 12→12, p99 2→1 ms | parity with stronger shared FD/thread/p99 evidence |
| CLI/DX: `llm`, `upgrade`, `issue`, spec, dockerfile, layered K8s | `cli-std`, `openapi-codegen`, `service-k8s` | `tests/cli_convention.rs`, `spec_cli.rs` | `tests/cli_contract.rs`, `deploy_cli.rs` | `tests/spec_cli.rs`, `deploy_cli.rs` | `tests/cli_contract.rs` | parity |

## Domain and competitor boundary

| App | Product peer set | Required journey | Current efficiency evidence | Explicitly excluded |
|---|---|---|---|---|
| Tape | Kafka/Redpanda/Pulsar topic logs, NATS JetStream streams, RabbitMQ Streams | append history → full offset/time replay; named bounded pull → explicit checkpoint ack; retention/backfill | real 20k×128-byte release replay: Tape/NATS p50 13,586/27,384 us (**2.02x**); Tape/Kafka KRaft 13,575/55,243 us (**4.07x**) | consumer groups, competing per-event leases, push, Relay-style bidirectional consume; RabbitMQ topic exchange is routing-only, not a replay baseline |
| Relay | RabbitMQ quorum queues, NATS JetStream work queues, Redis Streams | durable batch publish → committed fenced lease → committed ack/nack/heartbeat/reclaim/DLQ | real 100k×128-byte bulk lifecycle: Relay 57,847 msg/s; RabbitMQ 28,295; NATS 34,785; small-batch losses remain report-only | Kafka-style history/backfill and scheduler ETA/task records |
| Defer | Google Cloud Tasks feature contract; local Relay scheduler ceiling | committed task batch → global queue permit → fenced attempt → real HTTP → committed terminal/retry/DLQ | identical fsync-always Defer/Relay lifecycle: 1,261.25/134.15 ops/s, ratio **9.4019x**, required minimum **0.8x** | no unmeasured Cloud Tasks performance win; no Celery/Sidekiq worker-framework parity claim |

## Architecture invariants

- Every replica applies identical committed bytes. Wall-clock decisions,
  executor identity, epoch, expiry, and permit consumption are resolved before
  proposal and fenced in `raft-runtime`.
- Disk/data growth maps to shards. Replication factor maps to HA and remains
  stable during ordinary CPU/memory autoscaling. Serving/read capacity may be
  added only without silently changing the voter set.
- Relay and Defer commit assignment before any effect is sent. A stale or
  non-owner executor cannot heartbeat, settle, ack, or publish an outcome.
- Tape replay and pull are side-effect-free. Its only subscription mutation is
  explicit checkpoint ack through the committed checkpoint path; its public and
  persisted subscription model contains no delivery-mode switch.

<!-- HANDWRITE-END -->
