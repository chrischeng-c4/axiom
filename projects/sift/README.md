# Sift

## Brief

Sift is the GCP/GKE-first operational event platform in the Axiom stack.

It is not a traditional logging service. Logs are Sift's first producer and
first materialized view, but the source of truth is a replayable raw operational
event journal. Sift standardizes, validates, stores, indexes, correlates, and
replays operational facts so logging, trace, error-report, metric, audit, and
change stores share one event backbone while remaining independently queryable
and rebuildable.

Sift is one service, not a first-wave microservice fleet: public API, auth,
raw journal, correlation, and query live behind one service boundary. The
logging, trace, error-report, metric, and audit/change stores are internal
modules and materialized physical layouts that may be deployed as separate
roles only when scale or SLO evidence requires it.

Sift owns the operational event domain:

- GCP/GKE-oriented operational event schema and validation.
- Raw event journal, replayable archive, hot storage, and rebuildable indexes.
- First-class signal records for logs, spans, metrics, exceptions, audit
  events, and change events.
- First-class materialized stores for logging search, trace topology,
  error-report grouping, direct metric time series and exemplars, audit search,
  and change correlation.
- Query, tail, replay, and incident-time CLI/API ergonomics.
- Governance for schema versions, indexed fields, high-cardinality attributes,
  retention, redaction, and access boundaries.

Sift does not own generic search, topic replay, or online broker delivery:

- `lumen` owns reusable search/index behavior and can supply primitives or a
  derived index layer, but Sift owns operational event semantics and view
  materialization.
- `tape` owns generic topic replay/archive workflows. Sift owns replay for Sift
  views and the GCS raw operational event archive.
- `relay` owns online broker delivery. Sift may integrate with a broker later,
  but its product contract is the operational event platform.

The first implementation should prioritize the Sift core API and storage path.
The GKE DaemonSet collector remains a planned producer path, not the first
runtime dependency.

## Capabilities

The Service baseline capabilities selected by `aw.toml` are mandatory for this
long-running service class. They do not replace Sift's product capabilities;
operational event ingest, raw journal/archive, schema governance, shard-aware
hot storage, materialized views, replay, RBAC, and operational stability remain
first-class domain roots.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Operational Event Ingest | 1157 | planned | planned | conformance | not_ready | structured GCP/GKE and OTLP ingest with bounded batches, idempotency, and normalized resource/context |
| Raw Event Journal And Archive | 1157 | planned | planned | conformance | not_ready | canonical append-only journal, replay, and GCS archive manifest |
| Durability And Acknowledgment | 1157 | planned | planned | conformance | not_ready | an accepted mutation is durable before its success response |
| Shard-Aware Hot Storage | 1157 | planned | planned | conformance | not_ready | logical shards, epochs, sealed segments, and autonomous disk-driven splits |
| Replica Sync And Bootstrap | 1157 | planned | planned | conformance | not_ready | `raft-core` + `raft-host` replicated state machine, follower bootstrap, and read consistency |
| Backup And Restore | 1157 | planned | planned | conformance | not_ready | consistent snapshot/restore and scheduled off-node object-storage backup |
| Schema Governance | 1157 | planned | planned | conformance | not_ready | signal taxonomy, schema versions, validation, indexed-field policy, and high-cardinality controls |
| Materialized Observability Stores | 1157 | planned | planned | conformance | not_ready | first-class logging, trace, error-report, metric, and audit/change stores rebuildable from raw events |
| Query Tail And Replay | 1157 | planned | planned | conformance | not_ready | cross-signal query, log tail, correlation, cursoring, and replay-driven rebuilds |
| GKE Event Collection | 1157 | planned | planned | conformance | not_ready | later DaemonSet producer reads node-local CRI logs and emits structured operational events |
| Security Audit And Governance | 1157 | planned | planned | conformance | not_ready | immutable audit/change projections, stricter retention, scoped access, and export controls |
| HTTP2 API List | 1157 | planned | planned | conformance | not_ready | h2c/OpenAPI service routes and generated clients |
| Standard Operational Endpoints | 1157 | implemented | verified | conformance | ready | auth-exempt `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, and `/docs` on the service port |
| Kubernetes-Native Deployment | 1157 | planned | planned | conformance | not_ready | service/operator/instance artifacts, durable StatefulSet, HPA, and later collector DaemonSet |
| CLI Interface | 1157 | planned | planned | conformance | not_ready | service, domain, spec, deploy, and connect command surface |
| CLI Standard Surface | 1157 | planned | planned | conformance | not_ready | shared `llm`, `upgrade`, and `issue` command contract |
| Chainable Output Conformance | 1157 | planned | planned | conformance | not_ready | operational CLI commands emit executable next steps or terminal markers |
| EC Gates Configured | 1157 | planned | planned | conformance | not_ready | behavior, security, stability, and performance claims are executable gates |
| Developer And Agent Experience | 1157 | planned | planned | conformance | not_ready | offline contract, onboarding, interactive tooling, and integration contract |
| Long-Running Stability | 1157 | planned | planned | conformance | not_ready | ingest/query/replay soak, restart recovery, retention workers, and bounded disk/memory behavior |
| Security Hardening | 1157 | planned | planned | conformance | not_ready | bearer-token auth, scoped access, audit controls, redaction, and guard evidence |
| Competitor Feature Parity | 1157 | planned | planned | conformance | not_ready | explicit GCP Cloud Logging, Monitoring, Trace, and Error Reporting comparison boundaries |
| Competitor Performance | 1157 | planned | planned | conformance | not_ready | retained performance floors for ingest, query, tail, and replay workloads |
| GCP Cloud Logging Compatibility | 1157 | planned | planned | conformance | not_ready | GCP/GKE-first structured log compatibility without claiming every Cloud Logging feature |

### Operational Event Ingest

ID: operational-event-ingest
Type: Service
Root WI: -
Status: confirmed
Surfaces: HTTP: `POST /v1/events:write` and OTLP signal ingest; CLI: event
write/import paths; OpenAPI: offline event schema and error contract.
EC Dimensions: behavior: pending ingest API gate - batch validation,
compression, idempotency, backpressure, quota errors, and signal-specific
schema rejection.
Required Verification: conformance
Promise:
Accept bounded batches of structured GCP/GKE and OpenTelemetry operational
events, including direct `metric` points with temporality and exemplars.
Validate their envelope and signal schema, normalize resource and trace context,
and make write pressure explicit before the storage path is overrun.
Gate Inventory:
- pending: projects/sift/tests/operational_event_ingest.rs
- pending: projects/sift/tests/event_schema_validation.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| h2c-openapi-event-write-route | epic | - | planned | planned | none | pending ingest API contract gate |
| gcp-gke-event-envelope-validation | epic | - | planned | planned | none | pending schema validation gate |
| otlp-log-span-metric-normalization | epic | 1157 | planned | planned | conformance | pending OTLP signal/resource/context gate |
| quota-backpressure-and-idempotency | epic | - | planned | planned | none | pending overload and duplicate gate |

### Raw Event Journal And Archive

ID: raw-event-journal-and-archive
Type: Service
Root WI: -
Status: confirmed
Surfaces: Storage: append-only raw operational event journal, GCS archive
writer, archive manifest, replay cursor, and rebuild checkpoints; HTTP/CLI:
replay and archive inspection.
EC Dimensions: behavior: pending journal/archive gate - append/read, archive
manifest integrity, replay cursor correctness, and rebuild idempotency;
stability: pending archive/replay soak.
Required Verification: conformance, dogfood
Promise:
Treat raw operational events as Sift's source of truth so every materialized
view can be rebuilt from the journal or GCS archive instead of becoming the only
copy of the facts.
Gate Inventory:
- pending: projects/sift/tests/raw_event_journal.rs
- pending: projects/sift/tests/gcs_archive_replay.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| append-only-operational-event-journal | epic | - | planned | planned | none | pending journal append/read gate |
| gcs-raw-archive-manifest | epic | - | planned | planned | none | pending archive manifest gate |
| replayable-view-rebuild | epic | - | planned | planned | none | pending replay rebuild gate |

### Durability And Acknowledgment

ID: durability-and-acknowledgment
Type: Service
Root WI: 1157
Status: confirmed
Surfaces: Storage: service-owned durable journal/state store and projection
checkpoints; HTTP: accepted event responses carry the durable cursor and commit
index.
EC Dimensions: behavior: pending durable-ack gate - acknowledge only after
fsync and replicated state-machine commit; stability: pending crash/restart
gate - no acknowledged event is lost or duplicated after recovery.
Required Verification: conformance, dogfood
Promise:
Never report a successful state-changing ingest, replay, retention, or admin
operation from an in-memory-only path. The production acknowledgement boundary
is the durable raw-journal append plus committed `raft-host` state-machine
application; a restart must recover every acknowledged event exactly once by
event id.
Gate Inventory:
- pending: projects/sift/tests/durable_ack_boundary.rs
- pending: projects/sift/tests/crash_restart_recovery.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| fsync-before-success-response | epic | 1157 | planned | planned | conformance | pending durable acknowledgement contract gate |
| committed-raft-apply-before-success | epic | 1157 | planned | planned | conformance | pending primary/quorum commit gate |
| crash-restart-acknowledged-event-recovery | epic | 1157 | planned | planned | dogfood | pending power-loss/restart fixture gate |

### Shard-Aware Hot Storage

ID: shard-aware-hot-storage
Type: Service
Root WI: -
Status: confirmed
Surfaces: Storage: bucket-scoped logical shards, epoch shard maps, sealed
segments, hot indexes, placement metadata, retention workers, and
snapshot/restore paths.
EC Dimensions: behavior: pending storage conformance gate - shard routing,
append/read, epoch split, sealed segment movement, retention delete, cursor
pagination, and rebuildable index behavior; stability: pending retention and
capacity soak.
Required Verification: conformance, dogfood
Promise:
Store hot operational events with logical sharding from day one, even when all
shards initially live on one local placement, so future capacity growth can
split new writes by epoch and move sealed segments without rewriting the entire
history.
Gate Inventory:
- pending: projects/sift/tests/shard_hot_storage.rs
- pending: projects/sift/tests/shard_epoch_split.rs
- pending: projects/sift/tests/retention.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| logical-shard-routing | epic | - | planned | planned | none | pending shard routing gate |
| epoch-based-future-write-split | epic | - | planned | planned | none | pending epoch split gate |
| sealed-segment-retention-and-move | epic | - | planned | planned | none | pending segment lifecycle gate |
| rebuildable-hot-index | epic | - | planned | planned | none | pending index rebuild gate |

### Replica Sync And Bootstrap

ID: replica-sync-and-bootstrap
Type: Service
Root WI: 1157
Status: confirmed
Surfaces: Storage: `raft-core` consensus and `raft-host` transport, apply,
snapshot, compaction, and read-consistency primitives; HTTP: cluster and
read-consistency introspection.
EC Dimensions: behavior: pending replication gate - a `RaftStateMachine`
applies ordered journal/projection commands on primary and follower; stability:
pending failover, snapshot install, and empty-PVC bootstrap gate.
Required Verification: conformance, dogfood
Promise:
Run Sift's stateful control and durable event plane through the shared
`raft-core` and `raft-host` path, not a later DTO-only clustering stub. A new
or restored replica bootstraps from a snapshot/object seed, catches up through
the log, and serves reads according to the shared consistency contract.
Gate Inventory:
- pending: projects/sift/tests/raft_replica_sync.rs
- pending: projects/sift/tests/replica_bootstrap.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| sift-raft-state-machine | epic | 1157 | planned | planned | conformance | pending `RaftStateMachine` apply/snapshot/restore gate |
| h2c-follower-replication | epic | 1157 | planned | planned | conformance | pending leader/follower replication gate |
| snapshot-seed-and-catchup-bootstrap | epic | 1157 | planned | planned | dogfood | pending empty-PVC bootstrap gate |

### Backup And Restore

ID: backup-and-restore
Type: Service
Root WI: 1157
Status: confirmed
Surfaces: CLI: `sift backup export|restore`; Storage: consistent raw-journal
and state-machine snapshots, archive manifests, and object-storage destination
policy; K8s: scheduled backup job and restore status.
EC Dimensions: behavior: pending snapshot/restore gate - a consistent snapshot
recreates raw journal, shard map, and projection checkpoint; stability: pending
scheduled object-storage backup and cold-restore gate.
Required Verification: conformance, dogfood
Promise:
Expose consistent snapshot and restore through the Sift state machine and the
shared `service-backup` policy/runner shape. A production Sift instance has a
scheduled off-node object-storage backup; GCS is the required GCP destination
when the shared adapter is available, and no local-only backup is called
production-ready.
Gate Inventory:
- pending: projects/sift/tests/backup_restore_e2e.rs
- pending: projects/sift/tests/backup_schedule_render.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| consistent-state-machine-snapshot | epic | 1157 | planned | planned | conformance | pending snapshot byte/restore gate |
| service-backup-policy-and-runner | epic | 1157 | planned | planned | conformance | pending backup destination/policy gate |
| scheduled-gcs-object-backup | epic | 1157 | planned | planned | dogfood | pending operator CronJob and GCS restore gate |

### Schema Governance

ID: schema-governance
Type: Service
Root WI: -
Status: confirmed
Surfaces: Schema: operational event envelope, signal taxonomy, schema registry,
indexed-field policy, high-cardinality guardrails, redaction policy, and
compatibility checks.
EC Dimensions: behavior: pending schema governance gate - signal acceptance,
schema version compatibility, index allowlist enforcement, high-cardinality
rejection, and PII redaction policy.
Required Verification: conformance
Promise:
Keep operational events structured and governable by requiring schema versions,
known signals, bounded indexed fields, controlled high-cardinality attributes,
and explicit redaction/access policies.
Gate Inventory:
- pending: projects/sift/tests/schema_registry.rs
- pending: projects/sift/tests/index_policy.rs
- pending: projects/sift/tests/redaction_policy.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| operational-event-envelope | epic | - | planned | planned | none | pending envelope schema gate |
| signal-taxonomy-and-versioning | epic | - | planned | planned | none | pending signal schema gate |
| indexed-field-and-cardinality-policy | epic | - | planned | planned | none | pending index policy gate |
| pii-redaction-and-policy-check | epic | - | planned | planned | none | pending redaction gate |

### Materialized Observability Stores

ID: materialized-observability-stores
Type: Service
Root WI: 1157
Status: confirmed
Surfaces: Storage: first-class logging, trace, error-report, metric, and
audit/change stores with independent schemas, indexes, retention, and rebuild
checkpoints; HTTP/CLI: store-specific query and correlation routes.
EC Dimensions: behavior: pending store gate - logging search, trace topology,
error fingerprint/group lifecycle, direct metric time-series and exemplar
ingest, audit/change timeline, and replay rebuild consistency.
Required Verification: conformance, dogfood
Promise:
Expose logging, tracing, error reporting, metrics, and audit/change as
first-class Sift stores over the raw operational-event journal. Each store is
materialized and rebuildable, but metrics are also accepted as the direct
`metric` signal with points, temporality, exemplars, and resource dimensions;
they are not merely log/span-derived counters.
Gate Inventory:
- pending: projects/sift/tests/logging_store.rs
- pending: projects/sift/tests/trace_store.rs
- pending: projects/sift/tests/error_report_store.rs
- pending: projects/sift/tests/metric_store.rs
- pending: projects/sift/tests/audit_change_store.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| logging-store-over-events | epic | 1157 | planned | planned | conformance | pending log stream/index/query/rebuild gate |
| trace-store-topology-and-correlation | epic | 1157 | planned | planned | conformance | pending span tree, trace-log, and resource correlation gate |
| error-report-store-grouping-lifecycle | epic | 1157 | planned | planned | conformance | pending fingerprint, group, occurrence, and state-transition gate |
| metric-store-direct-points-and-exemplars | epic | 1157 | planned | planned | conformance | pending direct metric point, temporality, dimension, and exemplar gate |
| audit-and-change-store-timeline | epic | 1157 | planned | planned | conformance | pending immutable audit/change timeline and scoped access gate |
| store-rebuild-from-raw-journal | epic | 1157 | planned | planned | dogfood | pending independent projection rebuild gate |

### Query Tail And Replay

ID: query-tail-and-replay
Type: Service
Root WI: -
Status: confirmed
Surfaces: HTTP: `POST /v1/events:query`, log tail stream route, trace/error
lookup routes, audit/change lookup routes, and replay routes; CLI: `sift query`,
`sift tail`, and `sift replay`.
EC Dimensions: behavior: pending query gate - time/resource/signal/event
type/severity/trace/request/schema filters, pagination, ordering, live tail
resume, and replay cursor correctness.
Required Verification: conformance, dogfood
Promise:
Let SRE, developer, and security users find correlated operational events by
time, service, resource, signal, trace, request, severity, actor, subject, and
change context, then tail or replay matching events during incidents.
Gate Inventory:
- pending: projects/sift/tests/event_query_api.rs
- pending: projects/sift/tests/tail_api.rs
- pending: projects/sift/tests/replay_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| indexed-event-query | epic | - | planned | planned | none | pending query API gate |
| cursor-pagination-and-ordering | epic | - | planned | planned | none | pending pagination gate |
| live-tail-resume | epic | - | planned | planned | none | pending tail gate |
| replay-cursor-and-view-rebuild | epic | - | planned | planned | none | pending replay gate |

### GKE Event Collection

ID: gke-event-collection
Type: Service
Root WI: -
Status: confirmed
Surfaces: K8s: later Sift collector DaemonSet for GKE nodes; File: container
runtime CRI log files under the node log directory; HTTP: collector to Sift
event ingest API.
EC Dimensions: behavior: pending collector fixture gate - CRI stdout/stderr
parse, JSON payload validation, GCP/GKE metadata enrichment, rotation handoff,
and duplicate prevention.
Required Verification: conformance, dogfood
Promise:
Collect structured application logs from GKE workloads without requiring
application code changes, convert them into Sift operational events, preserve
trace context when present, and reject or quarantine unstructured payloads.
Gate Inventory:
- pending: projects/sift/tests/collector_cri.rs
- pending: projects/sift/tests/gke_metadata.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| daemonset-collector-node-log-read | epic | - | planned | planned | none | pending collector CRI fixture gate |
| structured-json-payload-validation | epic | - | planned | planned | none | pending structured-only gate |
| kubernetes-metadata-enrichment | epic | - | planned | planned | none | pending metadata enrichment gate |
| cloud-logging-coexistence-duplicate-prevention | epic | - | planned | planned | none | pending GKE coexistence gate |

### Security Audit And Governance

ID: security-audit-and-governance
Type: SecurityTool
Root WI: 1157
Status: confirmed
Surfaces: Storage: immutable audit and change-event stores with stricter
retention and legal-hold policy; HTTP/CLI: actor, subject, resource, and change
timeline lookup plus controlled export.
EC Dimensions: behavior: pending audit-store gate - append-only actor/subject
history, correlation to the causal request/trace, and ordered change timeline;
security: pending scoped-access, retention, redaction, and export-control gate.
Required Verification: conformance, security
Promise:
Treat security audit and change history as first-class operational facts, not
application log conventions. Sift preserves their integrity, causality, and
stricter governance policy while keeping the same raw-event correlation model
as the logging, trace, error, and metric stores.
Gate Inventory:
- pending: projects/sift/tests/audit_change_store.rs
- pending: projects/sift/tests/audit_governance.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| immutable-audit-event-projection | epic | 1157 | planned | planned | conformance | pending audit append/order gate |
| change-event-causality-timeline | epic | 1157 | planned | planned | conformance | pending change correlation gate |
| audit-retention-hold-and-export-controls | epic | 1157 | planned | planned | negative | pending governance policy gate |

### HTTP2 API List

ID: http2-api-list
Type: Service
Root WI: -
Status: confirmed
Surfaces: HTTP: h2c and HTTP/1.1 on one port, `/healthz`, `/readyz`,
`/metrics`, `/openapi.json`, `/docs`, `POST /v1/events:write`,
`POST /v1/events:query`, tail stream routes, replay routes, and view lookup
routes; CLI: `sift spec` and `sift spec gen`.
EC Dimensions: behavior: pending HTTP API gate - OpenAPI parity, standard
endpoint availability, h2c client compatibility, and generated client smoke.
Required Verification: conformance
Promise:
Expose Sift's operational event platform through the shared service archetype:
h2c plus HTTP/1.1 on one port, one OpenAPI contract available offline and
online, standard operational endpoints, and generated clients from the same
spec.
Gate Inventory:
- pending: projects/sift/tests/http2_api.rs
- pending: projects/sift/tests/openapi_contract.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| h2c-openapi-one-port-service | epic | - | planned | planned | none | pending h2c/OpenAPI gate |
| standard-operational-endpoints | epic | - | planned | planned | none | pending standard endpoint gate |
| offline-and-served-spec-parity | epic | - | planned | planned | none | pending spec parity gate |
| generated-client-smoke | epic | - | planned | planned | none | pending generated client gate |

### Standard Operational Endpoints

ID: standard-operational-endpoints
Type: Service
Root WI: 1157
Status: verified
Surfaces: HTTP: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, and
`/docs` are auth-exempt, always-on routes on the same h2c/HTTP/1.1 service
port.
EC Dimensions: behavior: `cargo test -p sift --test ingest_api http_ingest_and_standard_operational_routes_share_the_journal_contract -- --exact` - liveness, readiness, Prometheus, OpenAPI, and docs are available on the shared data-plane port.
Required Verification: conformance
Promise:
Provide the full shared operational surface through `service-http` so a Sift
deployment can be probed, scraped, and inspected without a separate admin
listener or custom endpoint names.
Gate Inventory:
- projects/sift/tests/ingest_api.rs (http_ingest_and_standard_operational_routes_share_the_journal_contract); projects/sift/external-contracts/behavior/standard-operational-endpoints-contract.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| one-port-health-readiness-metrics | epic | 1157 | implemented | passing | conformance | projects/sift/tests/ingest_api.rs |
| served-openapi-and-docs | epic | 1157 | implemented | passing | conformance | projects/sift/tests/ingest_api.rs |

### Kubernetes-Native Deployment

ID: kubernetes-native-deployment
Type: Service
Root WI: -
Status: confirmed
Surfaces: CLI: `sift dockerfile render`, `sift k8s crd render`,
`sift k8s operator render`, `sift k8s operator run`, and
`sift k8s instance render`; K8s: service CRD, operator, instance custom
resource, StatefulSet topology, services, probes, metrics, and later collector
DaemonSet.
EC Dimensions: behavior: pending k8s render gate - Dockerfile render, CRD
render, operator render, instance render, kustomize build, and kind smoke.
Required Verification: conformance, dogfood
Promise:
Deploy Sift as a dedicated, Kubernetes-native service using the repo service
archetype, with direct single-node install for smoke tests and operator-managed
StatefulSet topology for production.
Gate Inventory:
- pending: projects/sift/tests/k8s_render.rs
- pending: projects/sift/tests/dockerfile_render.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| dockerfile-render-surface | epic | - | planned | planned | none | pending Dockerfile render gate |
| crd-operator-instance-render | epic | - | planned | planned | none | pending k8s render gate |
| dedicated-stateful-service-topology | epic | - | planned | planned | none | pending topology gate |
| collector-daemonset-artifacts-later | epic | - | planned | planned | none | pending collector deployment gate |

### EC Gates Configured

ID: ec-gates-configured
Type: Devops
Root WI: 1157
Status: confirmed
Surfaces: Config: `aw.toml` EC bindings and generated claim manifest; Tests:
behavior, security, stability, and efficiency gates for every production claim.
EC Dimensions: behavior: pending claim-closure suite; security: pending Guard
gate; stability: pending Rig resilience suite; efficiency: pending Meter
ratchet gate.
Required Verification: conformance
Promise:
Keep Sift's public and operational promises executable: every claim has a TD
reference, generated or hand-written test path, runnable command, and a
production-required gate when its capability is in release scope.
Gate Inventory:
- pending: projects/sift/external-contracts/claim-closure/production-claims.md
- pending: projects/sift/tests/ec_claims.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| behavior-and-claim-closure-manifest | epic | 1157 | planned | planned | conformance | pending `aw ec gen --verify` gate |
| guard-security-contract-runner | epic | 1157 | planned | planned | negative | pending Guard runner gate |
| rig-resilience-contract-runner | epic | 1157 | planned | planned | dogfood | pending Rig runner gate |
| meter-performance-contract-runner | epic | 1157 | planned | planned | conformance | pending Meter ratchet gate |

### CLI Interface

ID: cli-interface
Type: Service
Root WI: -
Status: confirmed
Surfaces: CLI: `sift llm`, `sift upgrade`, `sift issue`, `sift event`,
`sift query`, `sift tail`, `sift replay`, `sift view`, `sift spec`,
`sift dockerfile`, `sift k8s`, and `sift connect`.
EC Dimensions: behavior: pending CLI gate - help surface, required standard
commands, event/query/tail/replay smoke, and chainable output markers.
Required Verification: conformance
Promise:
Give agents and operators a self-describing command surface for driving Sift
without prior project knowledge, including standard ecosystem commands and
event-platform commands whose outputs carry runnable next steps or terminal
markers.
Gate Inventory:
- pending: projects/sift/tests/cli_surface.rs
- pending: projects/sift/tests/cli_chainable_output.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| cli-std-llm-upgrade-issue | epic | - | planned | planned | none | pending cli-std gate |
| event-query-tail-replay-commands | epic | - | planned | planned | none | pending event command gate |
| spec-dockerfile-k8s-commands | epic | - | planned | planned | none | pending service command gate |
| chainable-output-contract | epic | - | planned | planned | none | pending chainable output gate |

### CLI Standard Surface

ID: cli-standard-surface
Type: AgentFirst
Root WI: 1157
Status: confirmed
Surfaces: CLI: `sift llm [--topic <topic>] [--format md|json]`, `sift upgrade
[--version <tag>] [--check]`, and `sift issue search|view|create` composed from
`cli-std`.
EC Dimensions: behavior: pending CLI standard-surface gate - every required
verb appears in `sift --help`, honors shared argument shape, and its dry-run or
terminal output is machine-readable.
Required Verification: conformance
Promise:
Ship the shared agent-facing `llm`, `upgrade`, and `issue` surface as real Sift
commands, without replacing the domain `event`, `query`, `tail`, `replay`, or
`view` verbs.
Gate Inventory:
- pending: projects/sift/tests/cli_standard_surface.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| offline-llm-topics | epic | 1157 | planned | planned | conformance | pending `sift llm` topic gate |
| upgrade-check-and-atomic-replace | epic | 1157 | planned | planned | conformance | pending `sift upgrade --check` gate |
| project-scoped-issue-surface | epic | 1157 | planned | planned | conformance | pending `sift issue` help and request gate |

### Chainable Output Conformance

ID: chainable-output-conformance
Type: AgentFirst
Root WI: 1157
Status: confirmed
Surfaces: CLI: every non-streaming operational command emits a final executable
`next:` command or explicit terminal marker; raw event, tail, and artifact
streams remain unwrapped.
EC Dimensions: behavior: pending chainability gate - emitted `next` commands
are executable and contain required arguments; artifact writing and streaming
have their respective trailing-output rules.
Required Verification: conformance
Promise:
Make Sift safe to drive in agent loops by guaranteeing a usable continuation or
done marker for its operational commands while preserving raw data streams for
pipelines and tailing.
Gate Inventory:
- pending: projects/sift/tests/cli_chainable_output.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| executable-next-command-validation | epic | 1157 | planned | planned | conformance | pending CLI next/done parser gate |
| artifact-and-stream-output-separation | epic | 1157 | planned | planned | conformance | pending render vs stream output gate |

### Developer And Agent Experience

ID: developer-and-agent-experience
Type: AgentFirst
Root WI: 1157
Status: confirmed
Surfaces: Offline contract: committed OpenAPI, schema, CLI help, and generated
clients; Agent onboarding: README quickstart and `sift llm`; Interactive
tooling: `sift connect`, `sift query`, and `sift tail`; Integration contract:
idempotency, pagination, error envelopes, and versioning.
EC Dimensions: behavior: pending offline/interactive contract gate - committed
and served OpenAPI parity, `llm` topics, connect lifecycle, and cross-call
retry/error semantics.
Required Verification: conformance
Promise:
Let an agent understand and safely integrate with Sift before a cluster is
available, then provide the shared Kubernetes connection and query tooling once
it is deployed. Client-visible retry, pagination, idempotency, and error
semantics are explicit, versioned contracts rather than incidental behavior.
Gate Inventory:
- pending: projects/sift/tests/spec_cli.rs
- pending: projects/sift/tests/connect_cli.rs
- pending: projects/sift/tests/integration_contract.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| offline-contract | epic | 1157 | planned | planned | conformance | pending OpenAPI/schema/help parity gate |
| agent-onboarding | epic | 1157 | planned | planned | conformance | pending quickstart and `sift llm` gate |
| interactive-tooling | epic | 1157 | planned | planned | conformance | pending `sift connect` and tail/query workflow gate |
| integration-contract | epic | 1157 | planned | planned | conformance | pending retry/error/pagination/idempotency gate |

### Long-Running Stability

ID: long-running-stability
Type: Service
Root WI: -
Status: confirmed
Surfaces: Runtime: graceful drain, restart recovery, WAL recovery, replay
resume, retention workers, archive workers, shard placement workers, metrics,
and readiness gates.
EC Dimensions: stability: pending long-run gate - ingest soak, query soak,
restart recovery, replay recovery, retention lag, bounded disk growth, and
bounded memory behavior.
Required Verification: conformance, dogfood
Promise:
Run Sift as a long-lived operational service with predictable resource use,
bounded backpressure, recoverable writers, resumable replay, and health signals
that protect ingest before disk or memory exhaustion.
Gate Inventory:
- pending: projects/sift/tests/stability_soak.rs
- pending: projects/sift/tests/restart_recovery.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| ingest-query-replay-soak | epic | - | planned | planned | none | pending soak gate |
| restart-and-wal-recovery | epic | - | planned | planned | none | pending recovery gate |
| retention-and-archive-worker-lag | epic | - | planned | planned | none | pending worker lag gate |
| bounded-disk-memory-backpressure | epic | - | planned | planned | none | pending resource bound gate |

### Security Hardening

ID: security-hardening
Type: Service
Root WI: -
Status: confirmed
Surfaces: HTTP: bearer-token auth, scoped authorization, auth-exempt standard
endpoints, audit event writes, redaction policy, and request limits; Config:
`SIFT_AUTH` and `SIFT_TOKEN_REGISTRY_FILE`.
EC Dimensions: security: pending guard gate - auth enforcement, scoped access,
audit retention, redaction, request limits, and forbidden high-cardinality or
PII index fields.
Required Verification: conformance, security
Promise:
Protect operational events with the shared bearer-token service contract,
resource-scoped authorization, strict audit semantics, redaction policy, and
guard evidence for network-exposed deployments.
Gate Inventory:
- pending: projects/sift/tests/auth.rs
- pending: projects/sift/tests/audit_security.rs
- pending: projects/sift/guard-sift-security.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-bearer-token-auth | epic | - | planned | planned | none | pending auth gate |
| scoped-event-view-access | epic | - | planned | planned | none | pending authorization gate |
| audit-event-retention-policy | epic | - | planned | planned | none | pending audit policy gate |
| pii-redaction-and-index-denylist | epic | - | planned | planned | none | pending redaction/security gate |

### Competitor Feature Parity

ID: competitor-feature-parity
Type: Service
Root WI: -
Status: confirmed
Surfaces: Docs: GCP Cloud Logging parity matrix; Tests: structured event ingest,
GKE resource metadata, trace correlation, log query, audit/change lookup, and
replay rebuild comparison cases.
EC Dimensions: behavior: pending parity gate - selected Cloud Logging structured
log capabilities, GKE metadata fidelity, query filters, and trace/log
correlation.
Required Verification: conformance
Promise:
Track Sift's selected replacement boundary against GCP Cloud Logging without
claiming full feature parity before the corresponding ingest, storage, query,
and governance gates exist.
Gate Inventory:
- pending: projects/sift/tests/gcp_cloud_logging_parity.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| gcp-structured-log-envelope-parity | epic | - | planned | planned | none | pending structured log parity gate |
| gke-resource-metadata-parity | epic | - | planned | planned | none | pending metadata parity gate |
| trace-log-correlation-parity | epic | - | planned | planned | none | pending trace correlation gate |
| query-filter-parity-slice | epic | - | planned | planned | none | pending query parity gate |

### Competitor Performance

ID: competitor-performance
Type: Service
Root WI: -
Status: confirmed
Surfaces: Benchmarks: ingest throughput, query latency, tail latency, replay
throughput, shard fanout, archive writer throughput, and retained comparison
floors for the selected Cloud Logging slice.
EC Dimensions: performance: pending performance gate - local Sift regression
floors and explicit peer recalibration workloads when Cloud Logging comparison
data is refreshed.
Required Verification: conformance
Promise:
Keep Sift's selected operational event workloads within retained performance
floors and make any Cloud Logging comparison explicit, reproducible, and
separate from every-run service regression gates.
Gate Inventory:
- pending: projects/sift/meter-sift-performance.toml
- pending: external-contracts/competitor-performance/sift/

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| ingest-throughput-floor | epic | - | planned | planned | none | pending ingest benchmark gate |
| query-tail-latency-floor | epic | - | planned | planned | none | pending query/tail benchmark gate |
| replay-archive-throughput-floor | epic | - | planned | planned | none | pending replay/archive benchmark gate |
| cloud-logging-peer-recalibration | epic | - | planned | planned | none | pending peer calibration gate |

### GCP Cloud Logging Compatibility

ID: gcp-cloud-logging-compatibility
Type: Service
Root WI: -
Status: confirmed
Surfaces: Schema: GCP-style `jsonPayload` compatibility, `k8s_container`
resource labels, severity mapping, trace/span/request correlation, and
structured-only log payload handling.
EC Dimensions: behavior: pending compatibility gate - representative GCP/GKE
structured log fixtures, severity normalization, resource label normalization,
and trace context preservation.
Required Verification: conformance
Promise:
Make the first Sift log producer and log view comfortable for teams familiar
with GCP Cloud Logging's structured `jsonPayload` model, while keeping Sift's
source of truth at the broader operational event layer.
Gate Inventory:
- pending: projects/sift/tests/gcp_structured_logs.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| jsonpayload-style-body-compatibility | epic | - | planned | planned | none | pending jsonPayload compatibility gate |
| k8s-container-resource-labels | epic | - | planned | planned | none | pending GKE resource label gate |
| severity-and-trace-context-normalization | epic | - | planned | planned | none | pending severity/trace normalization gate |
