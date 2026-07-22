# Sift

## Brief

Sift is the GCP/GKE-first operational event platform in the Axiom stack.

It is not a traditional logging service. Logs are Sift's first producer and
first materialized view, but the source of truth is a replayable raw operational
event journal. Sift standardizes, validates, stores, indexes, correlates, and
replays operational facts so logging, trace, error-report, metric, audit/change,
profile, and GenAI/evaluation stores share one event backbone while remaining
independently queryable and rebuildable.

Sift is one service, not a first-wave microservice fleet: public API, auth,
raw journal, correlation, and query live behind one service boundary. The
logging, trace, error-report, metric, audit/change, profile, and GenAI stores
are internal modules and materialized physical layouts that may be deployed as
separate roles only when scale or SLO evidence requires it. Lumen is embedded
as a rebuildable index crate; it does not run as a second service or Raft group.

Sift owns the operational event domain:

- GCP/GKE-oriented operational event schema and validation.
- Raw event journal, replayable archive, hot storage, and rebuildable indexes.
- First-class signal records for logs, spans, metrics, exceptions, audit
  events, change events, profiles, and evaluations.
- First-class materialized stores for logging search, trace topology,
  error-report grouping, direct metric time series and exemplars, audit search,
  change correlation, profile analysis, and GenAI session/cost/quality views.
- Query, tail, replay, rules, SLO/error-budget, monitor, incident, and
  deterministic diagnosis-evidence CLI/API ergonomics.
- Governance for schema versions, indexed fields, high-cardinality attributes,
  retention, redaction, and access boundaries.
- Native HTTP/TCP uptime checks, Jet-delegated browser journey results, and a
  privacy-governed RUM/Web-Vitals backend.

Sift does not own generic search, topic replay, or online broker delivery:

- `lumen` owns reusable search/index behavior and can supply primitives or a
  derived index layer, but Sift owns operational event semantics and view
  materialization.
- `tape` owns generic topic replay/archive workflows. Sift owns replay for Sift
  views and the GCS raw operational event archive.
- `relay` owns online broker delivery. Sift may integrate with a broker later,
  but its product contract is the operational event platform.
- External agents own narrative diagnosis and developer interaction. Sift does
  not embed an LLM, generate unsupported prose, manage prompts/datasets, execute
  experiments, or ship a GUI; `sift.diagnosis.v1` exposes deterministic facts,
  correlations, evidence references, data gaps, and executable next queries.

The first implementation should prioritize the Sift core API and storage path.
The Sift-owned GKE DaemonSet collector is an optional producer path over the
same collector core; it is not a Sift startup dependency and applications stay
coupled only to structured stdout.

## Capabilities

The Service baseline capabilities selected by `aw.toml` are mandatory for this
long-running service class. They do not replace Sift's product capabilities;
operational event ingest, raw journal/archive, schema governance, shard-aware
hot storage, materialized views, replay, RBAC, and operational stability remain
first-class domain roots.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Operational Event Ingest | 1658 | implemented | verified | conformance | ready | bounded JSON and OTLP JSON/protobuf/gzip ingest with ordered outcomes, project admission, and normalized resource/context |
| Raw Event Journal And Archive | 1659 | implemented | partial | conformance | not_ready | sharded source-of-truth journal, durable blobs, GCS archive/restore, and projection replay equality pass; retained soak remains #1676 |
| Durability And Acknowledgment | 1659 | implemented | partial | conformance | not_ready | blob and sharded raw fsync-before-ack plus single-node Raft apply pass; three-node proof remains #1676 |
| Shard-Aware Hot Storage | 1659 | implemented | partial | conformance | not_ready | 4096 buckets, future-only epochs, sealed segments, movement, recovery, and embedded rebuild index pass; retention remains |
| Replica Sync And Bootstrap | 1676 | implemented | partial | conformance | not_ready | existing Raft state machine plus pending three-node/domain recovery proof |
| Backup And Restore | 1659 | implemented | partial | conformance | not_ready | protected live snapshot plus real GCS transport and Vat-backed cold restore pass locally; GKE and three-node proof remain #1676 |
| Schema Governance | 1657 | implemented | partial | conformance | not_ready | OperationalEventV2/upcast/privacy, fixed projection-index allowlist, metric cardinality, and profile blob reference validation are verified; GenAI/evaluation schemas remain |
| Materialized Observability Stores | 1660 | implemented | partial | conformance | not_ready | logging, trace, error, metric, audit/change, and OTel profile stores pass; GenAI/evaluation remains #1670 |
| Query Tail And Replay | 1671 | implemented | partial | conformance | not_ready | durable replay plus authorized logging, trace, error, metric, audit/change, and profile analysis reads pass; unified cross-signal query and streaming tail remain #1671 |
| GKE Event Collection | 1675 | implemented | verified | conformance | not_ready | Sift-owned DaemonSet, CRI partial/rotation/restart, metadata, outage/loss accounting, and coexistence fixtures pass locally; live GKE and EC approval remain |
| Security Audit And Governance | 1668 | implemented | verified | conformance | ready | immutable hash-chained audit/change projections, retention with legal hold, scoped reads, controlled hashed exports, and rebuild equality pass |
| Profile Observability | 1669 | implemented | verified | conformance | not_ready | current OTel JSON/protobuf dictionary profiles, blob-before-ack, missing/corrupt rejection, flamegraphs, top functions, diffs, trace correlation, retention, and rebuild equality pass; multi-node cold-restore proof remains #1676 |
| AI And Agent Observability | 1670 | planned | planned | conformance | not_ready | GenAI observations, sessions, token/cost views, and typed evaluation scores |
| Alert Rules And Incident Lifecycle | 1672 | planned | planned | conformance | not_ready | durable typed rules, deduplicated incidents, and audited lifecycle transitions |
| SLO And Error Budget | 1672 | planned | planned | conformance | not_ready | SLIs, objectives, error budgets, and multi-window burn-rate evaluation |
| Uptime Synthetic And RUM | 1674 | planned | planned | conformance | not_ready | native HTTP/TCP uptime, Jet journey results, and OTel/Web-Vitals RUM backend |
| Agent Diagnosis Evidence | 1673 | planned | planned | conformance | not_ready | deterministic `sift.diagnosis.v1` facts, correlations, gaps, refs, and next queries |
| HTTP2 API List | 1604 | implemented | partial | conformance | not_ready | h2c/OpenAPI and Domain v1 ingest routes are verified; query/operations and generated-client expansion remain |
| Standard Operational Endpoints | 1576 | implemented | verified | conformance | ready | auth-exempt `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, and `/docs` on the service port |
| Kubernetes-Native Deployment | 1606 | implemented | verified | conformance | ready | Dockerfile, CRD/operator/instance, StatefulSet, PVC, probes, and backup schedule |
| CLI Interface | 1576 | implemented | partial | conformance | not_ready | baseline service/deploy commands exist; Domain v1 query/ops commands remain under #1671-#1674 |
| CLI Standard Surface | 1604 | implemented | verified | conformance | ready | shared `llm`, `upgrade`, and `issue` command contract |
| Chainable Output Conformance | 1604 | implemented | partial | conformance | not_ready | baseline outputs are chainable; new Domain v1 commands must preserve the contract |
| EC Gates Configured | 1607 | implemented | partial | conformance | not_ready | baseline behavior/security/stability gates exist; Domain v1 claim closure is #1676 |
| Developer And Agent Experience | 1604 | implemented | partial | conformance | not_ready | offline spec/client/onboarding baseline exists; unified Domain v1 operations remain |
| Long-Running Stability | 1607 | implemented | partial | conformance | not_ready | restart/resilience baseline exists; full ingest/query/replay/retention soak is #1676 |
| Security Hardening | 1616 | implemented | partial | conformance | not_ready | shared auth and deployment hardening exist; content governance and audit controls remain |
| Competitor Feature Parity | 1676 | planned | planned | conformance | not_ready | explicit GCP Observability, OTel, and agent-observability comparison boundaries |
| Competitor Performance | 1676 | planned | planned | conformance | not_ready | retained performance floors for ingest, query, tail, replay, rules, and rebuild |
| GCP Cloud Logging Compatibility | 1664 | implemented | partial | conformance | not_ready | structured ingest, log projection/query/rebuild, and canonical Axiom service-log coexistence with preserved Cloud insertId pass; broader compatibility remains under #1664 |

### Operational Event Ingest

ID: operational-event-ingest
Type: Service
Root WI: 1658
Status: verified
Surfaces: HTTP: `POST /v1/events:write` and OTLP signal ingest; CLI: event
write/import paths; OpenAPI: offline event schema and error contract.
EC Dimensions: behavior: `cargo test -p sift --test otlp_gcp_ingest` - bounded
batch validation, compression, idempotency, project authorization,
backpressure/quota errors, GCP normalization, and signal-specific partial
success.
Required Verification: conformance
Promise:
Accept bounded batches of structured GCP/GKE and OpenTelemetry operational
events, including direct `metric` points with temporality and exemplars.
Validate their envelope and signal schema, normalize resource and trace context,
and make write pressure explicit before the storage path is overrun.
Gate Inventory:
- implemented V2/upcast/privacy: projects/sift/tests/event_v2_governance.rs
- implemented eight-signal validation: projects/sift/tests/ingest_api.rs
- implemented OTLP/GCP transport: projects/sift/tests/otlp_gcp_ingest.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| operational-event-v2-and-policy | change | 1657 | implemented | passing | conformance | projects/sift/tests/event_v2_governance.rs |
| h2c-openapi-event-write-route | change | 1658 | implemented | passing | conformance | projects/sift/tests/otlp_gcp_ingest.rs |
| gcp-gke-event-envelope-validation | change | 1658 | implemented | passing | conformance | projects/sift/tests/otlp_gcp_ingest.rs |
| otlp-log-span-metric-profile-normalization | change | 1658 | implemented | passing | conformance | JSON/protobuf/gzip and partial-success integration gate |
| quota-backpressure-and-idempotency | change | 1658 | implemented | passing | conformance | project auth, overload, quota, duplicate, and body-limit gate |

### Raw Event Journal And Archive

ID: raw-event-journal-and-archive
Type: Service
Root WI: 1659
Status: confirmed
Surfaces: Storage: append-only raw operational event journal, GCS archive
writer, archive manifest, replay cursor, and rebuild checkpoints; HTTP/CLI:
replay and archive inspection.
EC Dimensions: behavior: `cargo test -p sift --test sharded_journal` and
`cargo test -p sift --test gcs_archive` - append/read, torn-tail recovery,
blob-before-reference durability, manifest integrity, and cold restore;
stability: projection rebuild equality and durable replay jobs pass under #1660;
archive/replay soak remains #1676.
Required Verification: conformance, dogfood
Promise:
Treat raw operational events as Sift's source of truth so every materialized
view can be rebuilt from the journal or GCS archive instead of becoming the only
copy of the facts.
Gate Inventory:
- implemented raw/shard/blob recovery: projects/sift/tests/sharded_journal.rs
- implemented Vat GCS archive/restore: projects/sift/tests/gcs_archive.rs
- implemented projection equality/replay jobs: projects/sift/tests/projection_runtime.rs and projects/sift/tests/replay_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| append-only-operational-event-journal | change | 1659 | implemented | passing | conformance | projects/sift/tests/sharded_journal.rs |
| durable-content-addressed-blobs | change | 1659 | implemented | passing | conformance | projects/sift/tests/sharded_journal.rs |
| gcs-raw-archive-manifest | change | 1659 | implemented | passing | dogfood | projects/sift/tests/gcs_archive.rs |
| replayable-view-rebuild | change | #1660 | implemented | passing | dogfood | durable replay jobs, restart recovery, and semantic projection equality tests |

### Durability And Acknowledgment

ID: durability-and-acknowledgment
Type: Service
Root WI: 1659
Status: confirmed
Surfaces: Storage: service-owned durable journal/state store and projection
checkpoints; HTTP: accepted event responses carry the durable cursor and commit
index.
EC Dimensions: behavior: raw blob/segment fsync and ordered single-node
`RaftStateMachine` apply pass before acknowledgement; stability: torn-tail and
restart recovery pass locally, while three-node failover remains #1676.
Required Verification: conformance, dogfood
Promise:
Never report a successful state-changing ingest, replay, retention, or admin
operation from an in-memory-only path. The production acknowledgement boundary
is the durable raw-journal append plus committed `raft-host` state-machine
application; a restart must recover every acknowledged event exactly once by
event id.
Gate Inventory:
- implemented raw fsync/recovery: projects/sift/tests/sharded_journal.rs
- implemented ordered Raft apply/snapshot: projects/sift/tests/ha_backup_e2e.rs
- pending three-node failover/power-loss proof: 1676

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| fsync-before-success-response | change | 1659 | implemented | passing | conformance | projects/sift/tests/sharded_journal.rs |
| committed-raft-apply-before-success | change | 1659 | implemented | passing | conformance | projects/sift/tests/ha_backup_e2e.rs; multi-node proof remains #1676 |
| crash-restart-acknowledged-event-recovery | change | 1676 | planned | planned | dogfood | three-node failover and power-loss recovery gate |

### Shard-Aware Hot Storage

ID: shard-aware-hot-storage
Type: Service
Root WI: 1659
Status: confirmed
Surfaces: Storage: bucket-scoped logical shards, epoch shard maps, sealed
segments, hot indexes, placement metadata, retention workers, and
snapshot/restore paths.
EC Dimensions: behavior: `cargo test -p sift --test sharded_journal` verifies
shard routing, append/read, future-only epoch split, torn-tail recovery, and
sealed segment movement; retention delete and rebuildable index behavior remain
#1660/#1676. Stability: pending retention and capacity soak.
Required Verification: conformance, dogfood
Promise:
Store hot operational events with logical sharding from day one, even when all
shards initially live on one local placement, so future capacity growth can
split new writes by epoch and move sealed segments without rewriting the entire
history.
Gate Inventory:
- implemented routing/epoch/segment recovery: projects/sift/tests/sharded_journal.rs
- implemented projection-index equality: projects/sift/tests/embedded_lumen_projection.rs and projects/sift/tests/projection_runtime.rs
- pending retention/capacity soak: 1676

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| 4096-virtual-bucket-routing | change | 1659 | implemented | passing | conformance | projects/sift/tests/sharded_journal.rs |
| epoch-based-future-write-split | change | 1659 | implemented | passing | conformance | projects/sift/tests/sharded_journal.rs |
| sealed-segment-retention-and-move | change | 1659 | implemented | passing | dogfood | byte-preserving move passes; retention worker remains #1676 |
| rebuildable-hot-index | change | #1660 | implemented | passing | dogfood | fixed-field embedded-Lumen snapshot/restore and rebuild equality gate |

### Replica Sync And Bootstrap

ID: replica-sync-and-bootstrap
Type: Service
Root WI: 1676
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
| sift-raft-state-machine | change | 1605 | implemented | passing | conformance | projects/sift/tests/ha_backup_e2e.rs |
| h2c-follower-replication | change | 1676 | planned | planned | dogfood | three-node leader/follower failover gate |
| snapshot-seed-and-catchup-bootstrap | change | 1676 | planned | planned | dogfood | empty-PVC snapshot seed and catch-up gate |

### Backup And Restore

ID: backup-and-restore
Type: Service
Root WI: 1659
Status: confirmed
Surfaces: CLI: live `sift backup --url <service> --dest gs://...`, explicit
offline `sift backup --data-dir <stopped-journal> --dest <uri>`, and `sift
restore --data-dir <dir> --source <uri>`; HTTP: protected `GET /admin/backup`;
Storage: consistent raw-journal and state-machine snapshots, archive manifests,
and object-storage destination policy; K8s: scheduled backup job and restore
status.
EC Dimensions: behavior: snapshot restore plus Vat-backed GCS segment/blob/
epoch-map cold restore pass; projection checkpoints remain #1660. Stability:
protected live snapshot transport passes locally; GKE scheduled-object and
three-node restore evidence remain #1676.
Required Verification: conformance, dogfood
Promise:
Expose consistent snapshot and restore through the Sift state machine and the
shared `service-backup` policy/runner shape. Scheduled runners fetch exact bytes
from the protected live `GET /admin/backup` boundary, optionally authenticate
with an admin token, and write them to a real `gs://` destination without
mounting the serving PVC. `--data-dir` remains a legacy offline-only path for a
stopped journal; opening it beside a live writer is unsupported. No local-only
backup is called production-ready.
Gate Inventory:
- implemented state snapshot restore: projects/sift/tests/ha_backup_e2e.rs
- implemented Vat GCS archive/cold restore: projects/sift/tests/gcs_archive.rs
- implemented protected live snapshot/transport: projects/sift/tests/live_backup.rs
- pending GKE scheduled object/three-node restore proof: 1676

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| consistent-state-machine-snapshot | change | 1605 | implemented | passing | conformance | projects/sift/tests/ha_backup_e2e.rs |
| service-backup-policy-and-runner | change | 1605 | implemented | passing | conformance | projects/sift/tests/ha_backup_e2e.rs |
| real-service-backup-gcs-sink | change | 1659 | implemented | passing | conformance | projects/sift/tests/gcs_archive.rs |
| scheduled-gcs-object-backup | change | 1676 | implemented | partial | dogfood | protected live-snapshot CronJob is wired; GKE object and cold-restore evidence remain |

### Schema Governance

ID: schema-governance
Type: Service
Root WI: 1657
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
- implemented: projects/sift/tests/event_v2_governance.rs
- pending: projects/sift/tests/index_policy.rs
- implemented: projects/sift/tests/event_v2_governance.rs (pre-journal privacy policy)

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| operational-event-v2-envelope | change | 1657 | implemented | passing | conformance | V2 typed round-trip and legacy journal/snapshot upcast tests |
| signal-taxonomy-and-versioning | change | 1657 | implemented | passing | conformance | eight-signal schema and compatibility tests |
| typed-attribute-policy | change | 1657 | implemented | passing | conformance | allow/deny and recursive typed-value validation tests |
| projection-index-allowlist | change | #1660 | implemented | passing | conformance | arbitrary payload/attribute fields are rejected by the embedded index adapter |
| metric-cardinality-policy | change | 1667 | implemented | passing | conformance | deterministic project budget, overflow series, and diagnostic gate |
| pii-and-genai-content-policy | change | 1657 | implemented | passing | negative | raw-byte absence, allow/deny, truncation, and project-policy tests |

### Materialized Observability Stores

ID: materialized-observability-stores
Type: Service
Root WI: 1660
Status: confirmed
Surfaces: Storage: first-class logging, trace, error-report, metric,
audit/change, profile, and GenAI/evaluation stores with independent schemas,
indexes, retention, and rebuild checkpoints; HTTP/CLI: store-specific query
and correlation routes.
EC Dimensions: behavior: logging search/query/tail, trace topology,
links/events, partial diagnostics, critical path, error fingerprint/group
lifecycle, direct metric time-series/chunks/rollups, OTel temporality,
histograms, exemplars, cardinality overflow, project authorization, and raw
rebuild equality pass; immutable audit/change timeline, legal hold, controlled
export, integrity verification, OTel profile blob durability, deterministic
analysis, correlation, retention, and rebuild pass; GenAI views remain pending.
Required Verification: conformance, dogfood
Promise:
Expose logging, tracing, error reporting, metrics, audit/change, profiles, and
GenAI/evaluation as first-class Sift stores over the raw operational-event
journal. Each store is materialized and rebuildable, but metrics are also
accepted as the direct `metric` signal with points, temporality, exemplars, and
resource dimensions; they are not merely log/span-derived counters.
Gate Inventory:
- implemented runtime/rebuild foundation: projects/sift/tests/projection_runtime.rs
- implemented embedded index boundary: projects/sift/tests/embedded_lumen_projection.rs
- implemented logging projection/query: projects/sift/tests/logging_store.rs and projects/sift/tests/logging_api.rs
- implemented trace topology/query: projects/sift/tests/trace_store.rs and projects/sift/tests/trace_api.rs
- implemented error grouping/lifecycle: projects/sift/tests/error_report_store.rs and projects/sift/tests/error_report_api.rs
- implemented metric projection/query: projects/sift/tests/metric_store.rs and projects/sift/tests/metric_api.rs
- implemented audit/change projection/governance: projects/sift/tests/audit_change_store.rs and projects/sift/tests/audit_change_api.rs
- implemented profile projection/blob/query: projects/sift/tests/profile_store.rs and projects/sift/tests/profile_blob_durability.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| projection-runtime-and-checkpoints | change | #1660 | implemented | passing | conformance | idempotent apply, atomic independent checkpoint, typed lag, and restart gate |
| logging-store-over-events | change | 1664 | implemented | passing | conformance | projects/sift/tests/logging_store.rs and projects/sift/tests/logging_api.rs |
| trace-store-topology-and-correlation | change | 1665 | implemented | passing | conformance | projects/sift/tests/trace_store.rs and projects/sift/tests/trace_api.rs |
| error-report-store-grouping-lifecycle | change | 1666 | implemented | passing | conformance | versioned fingerprints, ordered occurrences, durable authorized lifecycle, deterministic reopen/mute expiry, audit/change evidence, restart, and raw rebuild equality |
| metric-store-direct-points-and-exemplars | change | 1667 | implemented | passing | conformance | gauge/delta/cumulative reset semantics, explicit/exponential histograms, exemplars, late points, chunks, 60s/1h rollups, overflow diagnostics, auth, pagination, lag, snapshot, and raw rebuild equality |
| audit-and-change-store-timeline | change | 1668 | implemented | passing | conformance | immutable per-project hash chain, normalized actor/action/change context, raw rebuild equality, retention, legal hold, scoped query, and controlled hashed export |
| profile-store-and-analysis | change | 1669 | implemented | passing | conformance | current OTel dictionary samples/functions/locations/mappings, JSON/protobuf normalization, content-addressed blobs, flamegraph/top/diff, trace/span filters, retention, scoped API, and raw rebuild equality |
| genai-session-cost-evaluation-views | change | 1670 | planned | planned | conformance | observation, session, token/cost, and evaluation gate |
| store-rebuild-from-raw-journal | change | #1660 | implemented | passing | dogfood | fresh raw replay compares canonical semantic digest before atomic install |

### Query Tail And Replay

ID: query-tail-and-replay
Type: Service
Root WI: 1671
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
- implemented log-specific query/tail primitives: projects/sift/tests/logging_api.rs
- implemented trace-specific retrieval and partial topology: projects/sift/tests/trace_api.rs
- implemented audit/change query and controlled export: projects/sift/tests/audit_change_api.rs
- implemented profile list/analysis and scoped reads: projects/sift/tests/profile_store.rs and projects/sift/tests/profile_blob_durability.rs
- pending unified query: projects/sift/tests/event_query_api.rs
- pending streaming tail: projects/sift/tests/tail_api.rs
- implemented: projects/sift/tests/replay_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| typed-cross-signal-query | change | 1671 | planned | planned | conformance | store routes and typed DSL contract gate |
| cursor-pagination-and-ordering | change | 1671 | planned | planned | conformance | stable cursor and sort gate |
| live-tail-resume | change | 1671 | planned | planned | dogfood | reconnect/resume streaming gate |
| replay-cursor-and-view-rebuild | change | #1660 | implemented | passing | dogfood | durable start/status, shared command ordering, restart, and equality gate |

### GKE Event Collection

ID: gke-event-collection
Type: Service
Root WI: 1675
Status: confirmed
Surfaces: K8s: Sift collector DaemonSet for GKE nodes; File: container
runtime CRI log files under the node log directory; HTTP: collector to Sift
event ingest API.
EC Dimensions: behavior: local collector fixture gate passes for CRI
stdout/stderr parse, JSON validation, GCP/GKE metadata, rotation/restart,
outage/loss accounting, and duplicate prevention; live-cluster EC remains.
Required Verification: conformance, dogfood
Promise:
Collect structured application logs from GKE workloads without requiring
application code changes, convert them into Sift operational events, preserve
trace context when present, and reject or quarantine unstructured payloads.
Gate Inventory:
- implemented: projects/sift/tests/collector_cri.rs
- implemented: projects/sift/tests/deployment_cli.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| daemonset-collector-node-log-read | change | 1675 | implemented | passing | dogfood | CRI partial, rename, inode replacement, restart, and checkpoint gate |
| structured-json-payload-validation | change | 1675 | implemented | passing | conformance | shared `axiom.service.log.v1` decoder and quarantine gate |
| kubernetes-metadata-enrichment | change | 1675 | implemented | passing | conformance | real query asserts GCP, cluster, namespace, pod, uid, container, node, and stream |
| cloud-logging-coexistence-duplicate-prevention | change | 1675 | implemented | passing | dogfood | canonical fallback identity, outage recovery, and explicit loss gate |

### Security Audit And Governance

ID: security-audit-and-governance
Type: SecurityTool
Root WI: 1668
Status: confirmed
Surfaces: Storage: immutable audit and change-event stores with stricter
retention and legal-hold policy; HTTP/CLI: actor, subject, resource, and change
timeline lookup plus controlled export.
EC Dimensions: behavior: append-only actor/subject history, correlation to the
causal request/trace, ordered change timeline, per-project integrity chain, and
raw rebuild equality pass; security: scoped read/admin access, retention, legal
hold, and controlled hashed export pass.
Required Verification: conformance, security
Promise:
Treat security audit and change history as first-class operational facts, not
application log conventions. Sift preserves their integrity, causality, and
stricter governance policy while keeping the same raw-event correlation model
as the logging, trace, error, and metric stores.
Gate Inventory:
- implemented store/integrity/rebuild: projects/sift/tests/audit_change_store.rs
- implemented retention/hold/auth/export: projects/sift/tests/audit_change_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| immutable-audit-event-projection | change | 1668 | implemented | passing | conformance | normalized append-only records, per-project SHA-256 chain, duplicate immutability, tamper rejection, and raw rebuild equality |
| change-event-causality-timeline | change | 1668 | implemented | passing | conformance | ordered actor/action/target/version records preserve resource, trace, span, request, and session correlation |
| audit-retention-hold-and-export-controls | change | 1668 | implemented | passing | negative | project read/admin authorization, retention expiry, legal-hold override/release, and durable content-hashed export manifest |

### Profile Observability

ID: profile-observability
Type: Service
Root WI: 1669
Status: confirmed
Surfaces: HTTP: OTLP `/v1/profiles`, `POST /v1/profiles:query`; CLI: `sift
query profiles`; Storage: content-addressed profile blobs and rebuildable
profile projections.
EC Dimensions: behavior: current OTel JSON/protobuf dictionary ingest,
flamegraph, top-functions, diff, trace correlation, retention, and raw rebuild
equality pass; stability: blob-before-ack plus missing/deleted/digest-mismatch
rejection pass, with multi-node cold-restore proof retained in #1676.
Required Verification: conformance, dogfood
Promise:
Store OpenTelemetry profiles as first-class operational evidence. Large profile
payloads become durable content-addressed blobs before their bounded metadata
event is acknowledged; agents can query flamegraphs, top functions, diffs, and
trace correlations without a GUI.
Gate Inventory:
- implemented store/analysis/rebuild: projects/sift/tests/profile_store.rs
- implemented blob durability/protobuf/auth: projects/sift/tests/profile_blob_durability.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| otel-profile-ingest-and-blob-durability | change | 1669 | implemented | passing | conformance | OTel JSON/protobuf dictionary normalization, original-payload and whole-profile externalization, bounded raw metadata, durable hash/size validation, and missing/deleted blob rejection |
| flamegraph-top-functions-and-diff | change | 1669 | implemented | passing | conformance | deterministic root-to-leaf stack aggregation, inclusive/self totals, stable ordering, and baseline/comparison deltas |
| profile-trace-correlation-and-rebuild | change | 1669 | implemented | passing | dogfood | sample/event trace/span correlation, project/time filters, hot retention, restart blob validation, and raw-plus-blob rebuild equality |

### AI And Agent Observability

ID: ai-and-agent-observability
Type: Service
Root WI: 1670
Status: confirmed
Surfaces: HTTP: `POST /v1/genai:query`, `GET /v1/sessions/{id}`, evaluation
event append; CLI: `sift query genai`; Storage: OTel span specializations,
session groupings, cost/token views, and typed evaluations.
EC Dimensions: behavior: pending GenAI gate - generation/tool/RAG observation,
session grouping, token/cost accounting, and evaluation append; security:
pending default-off prompt/response content and pre-journal redaction gate.
Required Verification: conformance, security
Promise:
Expose generation, tool, agent, retrieval, and RAG work as specialized OTel
spans; group observations into cross-trace sessions and append typed evaluation
scores without mutating the source observation. Prompt and response content is
disabled by default and governed before raw durability.
Gate Inventory:
- pending: projects/sift/tests/genai_observations.rs
- pending: projects/sift/tests/genai_content_policy.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| genai-span-specializations | change | 1670 | planned | planned | conformance | OTel GenAI generation/tool/RAG normalization gate |
| cross-trace-sessions-and-cost-views | change | 1670 | planned | planned | conformance | session, provider/model, token, cost, and latency gate |
| typed-append-only-evaluations | change | 1670 | planned | planned | conformance | trace/span/session evaluation linkage gate |
| prompt-response-content-governance | change | 1657 | planned | planned | negative | default-off, truncation, and pre-journal redaction gate |

### Alert Rules And Incident Lifecycle

ID: alert-rules-and-incident-lifecycle
Type: Service
Root WI: 1672
Status: confirmed
Surfaces: HTTP/CLI: CRUD for rules and incidents, incident stream, lifecycle
commands, and webhook/Relay integration records; Storage: durable definitions,
evaluator state, transitions, and audit/change events.
EC Dimensions: behavior: pending rule/incident gate - evaluation, deduplication,
acknowledge, resolve, mute, reopen, and audit; stability: pending evaluator
restart/failover exactly-once transition gate.
Required Verification: conformance, dogfood
Promise:
Evaluate typed operational rules and maintain deduplicated incidents through
the same Sift Raft state machine as every other mutation. Every definition and
transition is durable and audited; v1 exposes streams/webhooks for external
automation instead of embedding Slack or email connectors.
Gate Inventory:
- pending: projects/sift/tests/rule_incident_lifecycle.rs
- pending: projects/sift/tests/incident_failover.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| typed-rule-definitions-and-evaluation | change | 1672 | planned | planned | conformance | threshold, absence, error-rate, and recovery gate |
| incident-dedup-and-lifecycle | change | 1672 | planned | planned | conformance | dedupe, ack, resolve, mute, and reopen gate |
| durable-audited-incident-stream | change | 1672 | planned | planned | dogfood | failover, audit/change, stream, and webhook record gate |

### SLO And Error Budget

ID: slo-and-error-budget
Type: Service
Root WI: 1672
Status: confirmed
Surfaces: HTTP/CLI: CRUD and status for SLOs; Storage: SLI windows, objectives,
error-budget state, and burn-rate evaluations over Sift metric/error/uptime
facts.
EC Dimensions: behavior: pending SLO gate - SLI selection, objective/error
budget math, windowing, and multi-window burn rate; stability: pending restart
and late-point recomputation gate.
Required Verification: conformance, dogfood
Promise:
Calculate service objectives and remaining error budget from typed Sift facts,
including deterministic multi-window burn-rate rules. SLO definitions and state
are durable, project-scoped, queryable, and linked to incident evidence.
Gate Inventory:
- pending: projects/sift/tests/slo_error_budget.rs
- pending: projects/sift/tests/slo_burn_rate.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| sli-objective-and-error-budget | change | 1672 | planned | planned | conformance | objective, good/total, and remaining-budget gate |
| multi-window-burn-rate | change | 1672 | planned | planned | conformance | short/long window burn-rate and recovery gate |
| durable-slo-state-and-incident-linkage | change | 1672 | planned | planned | dogfood | late data, restart, audit, and incident evidence gate |

### Uptime Synthetic And RUM

ID: uptime-synthetic-and-rum
Type: Service
Root WI: 1674
Status: confirmed
Surfaces: HTTP/CLI: monitor CRUD/status and RUM event ingest; Runtime: native
HTTP/TCP check runner; Integration: Jet browser journey dispatch/result events;
Storage: uptime, synthetic, and Web-Vitals projections.
EC Dimensions: behavior: pending monitor/RUM gate - uptime transitions, Jet
result ingest, Web-Vitals normalization, regression detection, and incident
correlation; stability: pending scheduler failover gate; security: pending RUM
privacy policy gate.
Required Verification: conformance, dogfood, security
Promise:
Run bounded HTTP/TCP uptime checks natively, delegate browser execution to Jet,
and accept OTel/Web-Vitals structured RUM facts. Results feed rules, SLOs,
incidents, and diagnosis without placing a browser engine or frontend UI inside
Sift.
Gate Inventory:
- pending: projects/sift/tests/uptime_synthetic.rs
- pending: projects/sift/tests/rum_web_vitals.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| native-http-tcp-uptime-runner | change | 1674 | planned | planned | dogfood | schedule, timeout, retry, failover, and recovery gate |
| jet-browser-journey-results | change | 1674 | planned | planned | conformance | external dispatch/result event contract gate |
| otel-web-vitals-rum-backend | change | 1674 | planned | planned | negative | RUM privacy, release context, regression, and incident gate |

### Agent Diagnosis Evidence

ID: agent-diagnosis-evidence
Type: AgentFirst
Root WI: 1673
Status: confirmed
Surfaces: HTTP: `GET /v1/incidents/{id}/evidence`; CLI: `sift diagnose`;
Schema: deterministic `sift.diagnosis.v1` fact, timeline, correlation,
candidate-cause, evidence-reference, data-gap, and next-query bundle.
EC Dimensions: behavior: pending diagnosis gate - seeded incident completeness,
reference resolution, deterministic candidate ranking, explicit gaps, and
executable next queries; security: pending project-scoped evidence gate.
Required Verification: conformance, dogfood, security
Promise:
Give an external agent enough deterministic evidence to answer what happened,
what was affected, when it began, which changes correlate, and what to query
next. Sift returns facts and evidence-backed candidates only; the external
agent owns natural-language explanation and must not receive invented causal
conclusions from Sift.
Gate Inventory:
- pending: projects/sift/tests/diagnosis_bundle.rs
- pending: projects/sift/tests/diagnosis_authorization.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| diagnosis-v1-schema-and-ordering | change | 1673 | planned | planned | conformance | deterministic schema and stable ordering gate |
| cross-signal-evidence-correlation | change | 1673 | planned | planned | dogfood | changes/SLO/metrics/errors/traces/logs/profiles/synthetic/RUM/GenAI gate |
| evidence-gaps-and-executable-next-queries | change | 1673 | planned | planned | conformance | reference resolution, explicit gaps, and command execution gate |

### HTTP2 API List

ID: http2-api-list
Type: Service
Root WI: 1604
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
- implemented: projects/sift/tests/behavior_one_port_health_readiness_metrics_contract.rs
- implemented: projects/sift/tests/behavior_served_openapi_and_docs_contract.rs
- implemented: projects/sift/tests/cli_contract.rs
- pending Domain v1 expansion: projects/sift/tests/http2_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| h2c-openapi-one-port-service | change | 1576 | implemented | passing | conformance | behavior one-port contract tests |
| standard-operational-endpoints | change | 1576 | implemented | passing | conformance | behavior standard endpoint contract tests |
| offline-and-served-spec-parity | change | 1604 | implemented | passing | conformance | projects/sift/tests/cli_contract.rs |
| generated-client-smoke | change | 1604 | implemented | passing | conformance | projects/sift/tests/cli_contract.rs |
| domain-v1-api-and-client-expansion | change | 1671 | planned | planned | conformance | complete ingest/query/ops OpenAPI and generated-client gate |

### Standard Operational Endpoints

ID: standard-operational-endpoints
Type: Service
Root WI: 1576
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
| one-port-health-readiness-metrics | change | 1576 | implemented | passing | conformance | projects/sift/tests/ingest_api.rs |
| served-openapi-and-docs | change | 1576 | implemented | passing | conformance | projects/sift/tests/ingest_api.rs |

### Kubernetes-Native Deployment

ID: kubernetes-native-deployment
Type: Service
Root WI: 1606
Status: verified
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
- projects/sift/tests/deployment_cli.rs
- projects/sift/tests/ha_backup_e2e.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| dockerfile-render-surface | change | 1606 | implemented | passing | conformance | projects/sift/tests/deployment_cli.rs |
| crd-operator-instance-render | change | 1606 | implemented | passing | conformance | projects/sift/tests/deployment_cli.rs |
| dedicated-stateful-service-topology | change | 1606 | implemented | passing | dogfood | projects/sift/tests/ha_backup_e2e.rs |
| deployment-guard-hardening | change | 1616 | implemented | passing | negative | projects/sift/tests/deployment_cli.rs |

### EC Gates Configured

ID: ec-gates-configured
Type: Devops
Root WI: 1607
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
- implemented: projects/sift/vat.toml
- implemented: projects/sift/guard.toml
- implemented: projects/sift/rig.toml
- implemented: projects/sift/meter-stability.toml
- pending Domain v1 claim closure: 1676

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| behavior-and-claim-closure-manifest | change | 1607 | implemented | passing | conformance | projects/sift/external-contracts/behavior/ |
| guard-security-contract-runner | change | 1607 | implemented | passing | negative | projects/sift/guard.toml |
| rig-resilience-contract-runner | change | 1607 | implemented | passing | dogfood | projects/sift/rig.toml |
| meter-stability-contract-runner | change | 1607 | implemented | passing | conformance | projects/sift/meter-stability.toml |
| domain-v1-claim-closure-and-performance | change | 1676 | planned | planned | dogfood | full `aw ec gen --verify` and retained performance gate |

### CLI Interface

ID: cli-interface
Type: Service
Root WI: 1576
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
- implemented baseline: projects/sift/tests/operational_cli.rs
- implemented deployment: projects/sift/tests/deployment_cli.rs
- pending Domain v1 expansion: projects/sift/tests/cli_surface.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| cli-std-llm-upgrade-issue | change | 1604 | implemented | passing | conformance | projects/sift/tests/cli_contract.rs |
| baseline-event-query-replay-commands | change | 1576 | implemented | passing | conformance | projects/sift/tests/operational_cli.rs |
| domain-v1-query-tail-ops-commands | change | 1671 | planned | planned | conformance | complete query/tail/replay command gate |
| rule-slo-incident-monitor-diagnose-commands | change | 1672 | planned | planned | conformance | headless operations CLI gate |
| spec-dockerfile-k8s-commands | change | 1606 | implemented | passing | conformance | projects/sift/tests/deployment_cli.rs |

### CLI Standard Surface

ID: cli-standard-surface
Type: AgentFirst
Root WI: 1604
Status: verified
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
- projects/sift/tests/cli_contract.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| offline-llm-topics | change | 1604 | implemented | passing | conformance | projects/sift/tests/cli_contract.rs |
| upgrade-check-and-atomic-replace | change | 1604 | implemented | passing | conformance | projects/sift/tests/cli_contract.rs |
| project-scoped-issue-surface | change | 1604 | implemented | passing | conformance | projects/sift/tests/cli_contract.rs |

### Chainable Output Conformance

ID: chainable-output-conformance
Type: AgentFirst
Root WI: 1604
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
- implemented baseline: projects/sift/tests/cli_contract.rs
- implemented deployment: projects/sift/tests/deployment_cli.rs
- pending Domain v1 expansion: 1671, 1672, 1673, 1674

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| executable-next-command-validation | change | 1604 | implemented | passing | conformance | projects/sift/tests/cli_contract.rs |
| artifact-and-stream-output-separation | change | 1606 | implemented | passing | conformance | projects/sift/tests/deployment_cli.rs |
| domain-v1-command-chainability | change | 1671 | planned | planned | conformance | all new non-stream command next/done gate |

### Developer And Agent Experience

ID: developer-and-agent-experience
Type: AgentFirst
Root WI: 1604
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
- implemented baseline: projects/sift/tests/cli_contract.rs
- implemented operational tooling: projects/sift/tests/operational_cli.rs
- pending Domain v1 integration: projects/sift/tests/integration_contract.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| offline-contract | change | 1604 | implemented | passing | conformance | projects/sift/tests/cli_contract.rs |
| agent-onboarding | change | 1604 | implemented | passing | conformance | `sift llm` and README baseline |
| interactive-tooling | change | 1576 | implemented | planned | conformance | operational CLI baseline exists; tail and unified query are #1671 |
| integration-contract | change | 1671 | planned | planned | conformance | retry/error/pagination/idempotency/projection-lag gate |

### Long-Running Stability

ID: long-running-stability
Type: Service
Root WI: 1607
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
- implemented baseline: projects/sift/tests/stability_e2e.rs
- implemented recovery: projects/sift/tests/stability_sift_long_running_stability_resilience.rs
- pending full Domain v1 soak: 1676

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| baseline-ingest-and-restart-resilience | change | 1607 | implemented | passing | conformance | projects/sift/tests/stability_e2e.rs |
| ingest-query-replay-soak | change | 1676 | planned | planned | dogfood | full signal/store/ops soak gate |
| retention-and-archive-worker-lag | change | 1676 | planned | planned | dogfood | retention/GCS/segment-move lag gate |
| bounded-disk-memory-backpressure | change | 1676 | planned | planned | dogfood | sustained resource and cardinality pressure gate |

### Security Hardening

ID: security-hardening
Type: Service
Root WI: 1616
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
- implemented auth: projects/sift/tests/runtime_security_e2e.rs
- implemented guard: projects/sift/guard.toml
- pending content governance: 1657
- implemented immutable audit controls: projects/sift/tests/audit_change_store.rs and projects/sift/tests/audit_change_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| shared-bearer-token-auth | change | 1604 | implemented | passing | conformance | projects/sift/tests/runtime_security_e2e.rs |
| deployment-guard-hardening | change | 1616 | implemented | passing | negative | projects/sift/guard.toml |
| scoped-event-view-access | change | 1671 | planned | planned | negative | store/query project authorization gate |
| audit-event-retention-policy | change | 1668 | implemented | passing | negative | retention expiry, legal-hold override/release, scoped administration, and content-hashed controlled export pass |
| pii-redaction-and-index-denylist | change | 1657 | implemented | planned | negative | pre-journal policy passes; projection index denylist remains in #1660 |

### Competitor Feature Parity

ID: competitor-feature-parity
Type: Service
Root WI: 1676
Status: confirmed
Surfaces: Docs: selected GCP Observability, OpenTelemetry, and Langfuse-model
parity matrices; Tests: ingest, resource metadata, correlation, store/query,
incident, diagnosis, and rebuild comparison cases.
EC Dimensions: behavior: pending parity gate - selected Cloud Logging structured
log capabilities, GKE metadata fidelity, query filters, and trace/log
correlation.
Required Verification: conformance
Promise:
Track Sift's selected replacement boundary against Google Cloud Observability,
OpenTelemetry, and the Langfuse observation/session model without claiming full
feature parity before the corresponding ingest, storage, operations, query,
governance, and recovery gates exist.
Gate Inventory:
- pending: projects/sift/tests/gcp_cloud_logging_parity.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| gcp-observability-domain-parity | change | 1676 | planned | planned | conformance | selected logs/traces/errors/metrics/profile/SLO comparison gate |
| otel-signal-and-protocol-parity | change | 1676 | planned | planned | conformance | OTLP and semantic-convention fixture gate |
| langfuse-observation-session-parity | change | 1676 | planned | planned | conformance | GenAI trace/session/observation/evaluation comparison gate |
| headless-operations-and-diagnosis-parity | change | 1676 | planned | planned | dogfood | rule/SLO/incident/evidence comparison gate |

### Competitor Performance

ID: competitor-performance
Type: Service
Root WI: 1676
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
| ingest-throughput-floor | change | 1676 | planned | planned | conformance | retained OTLP/GCP batch ingest floor |
| query-tail-latency-floor | change | 1676 | planned | planned | conformance | retained store query/tail floor |
| replay-archive-throughput-floor | change | 1676 | planned | planned | conformance | retained rebuild/GCS archive floor |
| rules-diagnosis-overhead-floor | change | 1676 | planned | planned | conformance | evaluator and evidence-bundle overhead floor |
| peer-recalibration | change | 1676 | planned | planned | dogfood | explicit GCP/OTel/Langfuse peer calibration evidence |

### GCP Cloud Logging Compatibility

ID: gcp-cloud-logging-compatibility
Type: Service
Root WI: 1664
Status: confirmed
Surfaces: Schema: GCP-style `jsonPayload` compatibility, `k8s_container`
resource labels, severity mapping, trace/span/request correlation, and
structured-only log payload handling.
EC Dimensions: behavior: `cargo test -p sift --test otlp_gcp_ingest --test
logging_store --test logging_api` verifies representative GCP/GKE structured
payload normalization before durability and dedicated-view jsonPayload,
severity, monitored-resource labels, trace/span/request/session correlation,
full-text search, read authorization, cursor/tail resume, retention, and raw
rebuild equality.
Required Verification: conformance
Promise:
Make the first Sift log producer and log view comfortable for teams familiar
with GCP Cloud Logging's structured `jsonPayload` model, while keeping Sift's
source of truth at the broader operational event layer.
Gate Inventory:
- implemented ingest compatibility: projects/sift/tests/otlp_gcp_ingest.rs
- implemented logging projection/query compatibility: projects/sift/tests/logging_store.rs and projects/sift/tests/logging_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| jsonpayload-style-body-compatibility | change | 1658 | implemented | passing | conformance | projects/sift/tests/otlp_gcp_ingest.rs |
| k8s-container-resource-labels | change | 1658 | implemented | passing | conformance | monitored-resource label normalization gate |
| severity-and-trace-context-normalization | change | 1658 | implemented | passing | conformance | severity/trace/span/request normalization gate |
| logging-view-query-compatibility | change | 1664 | implemented | passing | conformance | dedicated log schema, embedded-Lumen full-text, typed filters, project auth, cursor/tail resume, retention, projection lag, and raw rebuild equality |
| cloud-logging-coexistence-dedupe | change | 1675 | planned | planned | dogfood | collector coexistence identity gate |
