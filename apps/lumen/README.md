# lumen

## Brief

A K8s-native, **log-replicated search specialist**. Five flavors of
"find":

- **Exact** — `keyword` / `number` / `set`
- **Lexical** — `text` (BM25, with tokenize built in)
- **Semantic** — `vector` (CPU: HNSW + exact flat brute-force)
- **Perceptual / structural** — `hash` (pHash / SimHash / b-bit MinHash, Hamming distance)
- **Duplicates** — find which `external_id`s share the same value (a search-flavor of group-by; bounded, posting-list-cheap)

The caller owns the representation:

- Embeddings? **Caller** runs CLIP / BGE / Whisper / VideoMAE; lumen never owns a model artefact.
- Perceptual hashes? **Caller** runs `imagehash` / `datasketch`; lumen indexes the bits.
- Lexical tokenization? **lumen** does it — that's the one place caller doesn't compute (`whitespace_lower` / `ngram` / `jieba`).

The caller also owns the **source of truth**: lumen is a parallel derived index,
never the system of record or an analytics engine — documents are *not* a lumen
concept, only the caller's `external_id` is.

- **Log-driven, derived, rebuildable**. A write is *published to a log*,
  not applied where it lands; every serving node tails the log and folds
  it into its own index. Lossable but rebuildable from the log + the
  caller.
- **Client API on `:7373`** (HTTP/1.1 + HTTP/2 cleartext — REST clients
  need nothing special; high-throughput clients should use HTTP/2 multiplexing;
  see [HTTP](#http--clients)).
- **Sharded**: `hash(collection_id, routing_key || external_id)` selects a
  virtual bucket, and a versioned operator-owned shard map assigns buckets to
  physical storage shards. `shardCount` controls storage ownership,
  `replicasPerShard` controls HA/raft quorum per shard, and HPA never changes
  data ownership.
- **Agent-first offline integration surface**: `lumen spec` emits the exact
  machine schema, including `lumen spec --format openapi-yaml` for LLM-readable
  OpenAPI, while `lumen llm --topic outline --format json` emits a typed
  `cclab.llm.v2` task manifest and `lumen llm --topic <id>` emits the smallest
  source-backed runbook needed to wire Lumen into an app without a docs site or
  running server.

## Contributing

Project-local authoring and verification rules live in
[`CONTRIBUTING.md`](CONTRIBUTING.md). Repository-wide rules remain
authoritative when the two differ.

## Capabilities

A promise with no gate under it is not claimed.

Every capability belongs to exactly one of two feature roots:

- **Core Features** define what Lumen fundamentally does: Indexing and
  Querying.
- **Non-Core Features** make those two jobs deployable, secure, scalable,
  recoverable, observable, and integrable. Non-core does not mean optional.

This section contains stable product promises, claim IDs, and verification
surfaces. Delivery planning lives outside this contract and references these
IDs one way.

### Capability Index

| Class | ID | Capability | State |
|---|---|---|---|
| Core | `indexing` | Indexing | ready |
| Core | `querying` | Querying | ready |
| Non-Core | `kubernetes-native-deployment` | Kubernetes-Native Deployment | ready |
| Non-Core | `security-hardening` | Security & Access | not ready |
| Non-Core | `scaling-availability` | Scaling & Availability | ready |
| Non-Core | `durability-recovery` | Durability & Recovery | ready |
| Non-Core | `operations-observability` | Operations & Observability | ready |
| Non-Core | `api-cli-agent-integration` | API, CLI & Agent Integration | ready |

### Core Features

#### Indexing

Build and maintain rebuildable indexes over caller-owned `external_id` values.
The caller supplies source data, embeddings, and perceptual hashes; Lumen owns
schema validation, lexical analysis, index mutation, segment/checkpoint
persistence, and deterministic rebuild.

- Root WI: none; this capability predates the tracker.
- Gate: `cargo test -p lumen --test api_e2e --test drop_field_e2e --test reindex_stream_e2e --test stats_metadata_e2e`
- Gate: `cargo test -p lumen --test perf_gate --test perf_gate_vs_db`
- Source: `apps/lumen/e2e/rig/cases`

Claims this capability makes:
- `schema-and-index-lifecycle` — schemas and mutations are validated and
  applied consistently.
- `derived-index-storage` — retained index state survives restart and can be
  rebuilt without becoming the source of truth.
- `indexing-quality` — indexing meets the declared throughput, footprint, and
  long-running stability floors.

#### Querying

Query Lumen indexes and return ranked or filtered caller-owned `external_id`
values. Supported semantics include lexical BM25, exact and range filters,
vector kNN, Hamming hash search, hybrid RRF, duplicates, nested/group/collapse
behavior, pagination, sorting, and explicit read consistency.

- Root WI: none; this capability predates the tracker.
- Gate: `cargo test -p lumen --test api_e2e --test coverage_gaps_e2e`
- Gate: `cargo test -p lumen --test vector_e2e --test hash_hamming --test hybrid_rrf --test collapse_nested`
- Gate: `cargo test -p lumen --test perf_gate --test perf_gate_vs_db`

Claims this capability makes:
- `lexical-and-structured-query` — lexical, exact, range, pagination, and sort
  behavior is deterministic.
- `semantic-and-similarity-query` — vector, hash, hybrid, duplicate, and nested
  queries preserve their documented semantics.
- `query-quality` — unsafe shapes are rejected and query latency, throughput,
  and footprint stay within the declared floors.

### Non-Core Features

#### Kubernetes-Native Deployment

Render the image, CRD, operator, and instance layers independently, then
reconcile each Lumen instance into stable Kubernetes workloads, networking,
conditions, disruption protection, and optional isolation. Reusable Kubernetes
mechanics stay in shared libraries; Lumen owns its CRD policy and app wiring.

- Root WI: none; this capability predates the tracker.
- Gate: `cargo test -p lumen --features operator --test operator_render --test operator_backup_kubernetes_wiring`
- Gate: `apps/lumen/scripts/kind-e2e.sh`
- Gate: `acceptance/gcp/scripts/run.sh`

Claims this capability makes:
- `layered-deployment-artifacts` — Dockerfile, CRD, operator, and instance
  renderers remain independently usable.
- `live-operator-reconciliation` — desired state converges and owned resources
  are repaired without taking over unrelated objects.

#### Security & Access

Use Kubernetes as the client request identity and authorization boundary, a
separate X.509 identity plane for replicated Raft traffic, and rustls for
serving transport confidentiality.

- Root WI: none; this capability predates the tracker.
- Gate: `cargo test -p lumen -p service-auth -p service-k8s -p peer-tls`
- Gate: `cargo test -p lumen --test auth_e2e`
- Gate: `cargo test -p lumen --test authz_matrix_e2e`
- Gate: `cargo test -p lumen --features operator --test operator_render`
- Gate: `cargo test -p lumen --lib`
- Gate: `cargo test -p lumen --test serving_tls_rotation`
- Gate: `acceptance/gcp/scripts/run.sh`
- Gate: `acceptance/gcp/scripts/verify-lumen-auth.sh`

Request path:
```text
Google user or Google service account
  -> kube-apiserver authentication through kubeconfig / GKE credential plugin
  -> RBAC permits TokenRequest for one named client KSA
  -> short-lived, Lumen-audience KSA token
  -> Lumen TokenReview
  -> strict system:serviceaccount:<namespace>:<name> principal
  -> Lumen SubjectAccessReview for lumencollections / lumenadmin
```
Peer path:
```text
Lumen peer pod
  -> instance-scoped X.509 certificate
  -> mandatory mTLS on :7374
```
Serving path:
```text
in-cluster caller
  -> https://<instance>.<namespace>.svc:7373 (ClusterIP, never published)
  -> TLS terminated by the serving pod itself, ALPN h2 / http/1.1
  -> leaf verified against the externally distributed public CA
```
Invariants:
- Serving TLS terminates in the Lumen process. No Ingress, Gateway,
  LoadBalancer, NodePort, or service mesh terminates it on Lumen's behalf, so
  no hop between a caller and Lumen carries a request token in the clear.
- The serving leaf asserts the instance's own Kubernetes Service DNS names and
  nothing else.
- A configured serving certificate never degrades to plaintext; the client port
  refuses connections while no valid leaf is active.
- Deployment administrators or an external certificate platform distribute the
  public CA separately from the private-key-bearing serving Secret; clients
  pass it as `--ca-file` and replace the public roots rather than joining them.
- Serving and peer certificates are distinct material. Neither authenticates on
  the other's port.
- Clients authenticate the server with the trust anchor and authenticate
  themselves with a KSA token; client certificates are not an identity source.
- GCP credentials stop at kube-apiserver. Lumen rejects Google access tokens,
  Google ID tokens, ADC/GSA credentials, and metadata-server identity tokens.
- TokenReview must return the expected audience and an exact Kubernetes
  ServiceAccount principal.
- SubjectAccessReview is authoritative. Lumen only maps its operations to
  `lumencollections` and `lumenadmin`.
- Serving, operator/reshard, backup, and external-client ServiceAccounts are
  distinct least-privilege identities.
- TokenRequest permission names one client ServiceAccount; it is never granted
  namespace-wide.
- No long-lived ServiceAccount token Secret, shared bearer/Google registry,
  token environment injection, or metadata-token path remains.
- KSA tokens never authenticate Raft peers. Peer certificates never grant API
  access. Raft `:7374` never falls back to plaintext.
- Delegated-auth, RBAC rendering, projected-token, and TLS mechanics belong in
  shared libraries; Lumen owns domain policy and wiring.
- Deployment administrators or an external platform provision the serving and
  peer TLS Secrets named by each Lumen instance. The operator only consumes
  those Secrets and does not resolve issuers or perform CAS automation.
Claims this capability makes:
- `kubernetes-native-request-identity-and-authorization` — permitted KSA
  requests pass TokenReview/SAR and invalid or denied requests fail closed.
- `instance-scoped-raft-peer-identity` — only valid instance peers can use the
  Raft transport, including through rotation and failover.
- `serving-transport-tls` — the rustls-backed serving transport terminates
  private ClusterIP TLS in-process and admits no plaintext or unverified path.
Ready when one retained GKE evidence bundle proves KSA allow/deny, direct
Google rejection, peer mTLS positive/negative behavior, credential rotation,
failover, and cleanup. Retired bearer/Google-registry evidence cannot close
this capability.

#### Scaling & Availability

Scale index state and serving capacity without changing indexing or query
semantics. Lumen uses RAM-hot/disk-all segments, a versioned virtual-bucket
shard map, checkpointed reshard transitions, one Raft group per shard, explicit
replica policy, failover, and replacement bootstrap.

- Root WI: none; this capability predates the tracker.
- Gate: `cargo test -p lumen --test reshard_admin_e2e`
- Gate: `cargo test -p lumen --test efficiency_lumen_claim_elastic_disk_tier`
- Gate: `cargo test -p lumen --test wal_nats_e2e --test stability_lumen_claim_dynamic_multi_shard_replica_kind`
- Gate: `apps/lumen/scripts/kind-e2e.sh`

Claims this capability makes:
- `elastic-segment-tier` — hot memory and retained disk tiers obey their
  resource contract.
- `dynamic-shard-topology` — resharding converges without losing readable
  indexed data.
- `primary-replica-failover-and-bootstrap` — replicas synchronize, fail over,
  and replace failed members.

#### Durability & Recovery

Recover derived index state through WAL/checkpoint replay, Raft replication,
backup/restore, and cold seed without claiming ownership of the caller's source
data.

- Root WI: none; this capability predates the tracker.
- Gate: `cargo test -p lumen --test backup_restore_e2e`
- Gate: `cargo test -p lumen --test wal_nats_e2e`
- Gate: `acceptance/gcp/scripts/run.sh`

Claims this capability makes:
- `wal-checkpoint-and-raft-recovery` — committed index mutations survive
  restart and member replacement.
- `backup-restore-and-cold-seed` — a retained snapshot restores into a fresh
  instance and remains readable after restart.

#### Operations & Observability

Expose health, readiness, conditions, metrics, events, alerts, tracing, and
long-running-operation state for both serving and control-plane behavior.

- Root WI: none; this capability predates the tracker.
- Gate: `cargo test -p lumen --test api_e2e`
- Gate: `cargo test -p lumen --features operator --test operator_backup_kubernetes_wiring`
- Source: `apps/lumen/e2e/rig/cases`
- Source: `apps/lumen/k8s/components/operator-monitoring`

Claims this capability makes:
- `standard-operational-surfaces` — health, readiness, metrics, and status
  reflect real service state.
- `control-plane-observability` — reconciliation, leadership, errors, and
  alerts are externally observable.
- `long-running-stability` — retained workloads stay within declared resource
  and correctness bounds.

#### API, CLI & Agent Integration

Expose the two core jobs through HTTP/1.1 and HTTP/2, served and offline
OpenAPI, generated clients, the standard `llm`/`upgrade`/`issue` surface,
deployment commands, chainable output, and offline agent guidance.

- Root WI: none; this capability predates the tracker.
- Gate: `cargo test -p lumen --test spec_cli --test api_e2e`
- Gate: `cargo test -p lumen --test cli_convention`
- Gate: `cargo test -p lumen --features operator --test operator_render`

Claims this capability makes:
- `http2-openapi-and-client-interface` — wire behavior and published schemas
  stay aligned.
- `standard-cli-and-agent-interface` — commands remain discoverable,
  executable, and explicit about their next step or terminal state.

## Verified Cloud Evidence

Standard GKE operator acceptance evidence for Lumen (epic #2434 ordered service
1, before Tape run `0723135853`). This section records real-cloud proof runs;
the capability contract itself is the `## Capabilities` section above. Harness:
`benchmarks/gcp-operator-acceptance` (mode noted per run).

### GKE acceptance run 0724105144 (2026-07-24, PASSED — auth+CSI Secret Manager stack proven, #2457/#2456)

Full two-service digest-mode run (GHCR `sha-54742a8d6e40` images — zero
Cloud Build) adding the first live validation of the auth+CSI regression
leg: a `lumen-authcsi` CR with `auth: required`,
`tokensSecretProviderClass`, and `tokensSecretCsiDriver:
secrets-store-gke.csi.k8s.io` against a run-scoped Secret Manager secret
(SecretProviderClass `provider: gke`, `principal://` secretAccessor grant,
no GSA). Proven: CSI volume mounted with the GKE driver name, pod Ready
with **zero FailedMount events** (the exact #2456 failure signature),
tokens genuinely loaded from the CSI mount — authenticated search returns
the seeded document (`total: 1`) while unauthenticated returns exactly 401
`{"error":"unauthenticated"}`. All prior legs re-passed on the 0.4.26
candidate HEAD (cold-restore, admission, backup, auto-split 1→2), and
verified cleanup covers the new Secret Manager resources. Cluster
prerequisite recorded: the GKE Secret Manager add-on
(`--enable-secret-manager`) registers the CSIDriver; the leg self-skips
with evidence when absent. Evidence root:
`axiom-gcp-run-backup/evidence/0724105144/` (`kubernetes/lumen-authcsi-*`).

### GKE acceptance run 0724061548 (2026-07-24, PASSED — #2489 fix + cold-restore #2492 proven)

- Full two-service mode (Lumen and Sift both passed; the Sift rows live in
  the shared `acceptance.json`). Cluster: persistent Standard GKE
  `axiom-operator-acceptance` (`asia-east1-a`, project `axiom-502607`),
  run-scoped bucket/GSA/Workload-Identity bindings plus the restore-reader
  grant created and destroyed by the run.
- Image: Cloud Build from commit `70fd48ca5c44` (the `lumen@0.4.25`
  candidate — carries the #2489 scatter fix `9ffdb30513`, #2497
  `spec.serviceAccountName`, and the #2487 alert fix), tag
  `70fd48ca5c44-0724061548`, dirty-tree gate clean.
- Terminal artifacts: `lumen-acceptance.json`
  (`axiom.gcp.lumen.acceptance.v1`, every claimed proof `passed`) and
  `cleanup.json` (`status: clean`, verified `2026-07-24T06:56:11Z`).
  Evidence root: `axiom-gcp-run-backup/evidence/0724061548/`.

| Proof | Result | Artifact |
|---|---|---|
| Post-split read visibility (#2489): after the CONVERGED 1→2 auto-split, the pre-split collection is searchable through the client Service immediately — readability lag 0 s (vs `collection not found` for 180 s+ on both 0.4.24 retest runs). Restores the Dynamic Shard Topology GKE claim. | passed | `kubernetes/lumen-search-after-split.json`; `kubernetes/lumen-split-readable-after-seconds.txt` (`0`) |
| Cold-restore onto a fresh PVC (#2492): a second `lumen-restore` CR with `spec.serving.bootstrap.seedUri` pointed at the run's backup object (271 B, carries the `acceptance` collection) boots a genuinely fresh PVC and the seeded document is queryable (`total: 1`) | passed | `kubernetes/lumen-restore-search.json`; `gcs/lumen-first-object.json` |
| Seed-set restart retention: the restored instance keeps the seeded document across a serving-pod replacement | passed | `kubernetes/lumen-restore-after-restart-search.json` |
| Admission CR exposure (#2477): patching `spec.admission` renders the five `LUMEN_ADMISSION_*` env vars onto the StatefulSet pod spec (operator-propagation-aware poll), and removing the block rolls them back off | passed | `kubernetes/lumen-admission-env.txt` |
| Re-proven from `0723041614`: 1x1 reconcile, domain lifecycle (create/index/search), pod-restart data retention, Workload-Identity GCS backup (271-byte object) | passed | `kubernetes/…` per the matching rows in the `0723041614` table below |
| Verified cleanup: 6 run-scoped resources destroyed; "no run-tagged Lumen/Sift operator acceptance resources remain"; persistent cluster and Artifact Registry preserved | passed | `cleanup.json`; `run.log` |

Exclusions unchanged from `0723041614` (`cpu_memory_actuator`,
`live_replica_membership`: `not_claimed`). Deployer note for cold-restore:
the SERVING ServiceAccount of a `seedUri` instance reads GCS itself — it
needs `roles/storage.objectViewer` on the seed bucket (the backup GSA's
write grant does not cover it). The harness provisions this via
`storage.tf`'s `lumen_restore_reader` principal binding; real deployments
carry the same responsibility.

### GKE retest runs 0723160506 / 0723163748 (2026-07-23, FAILED — post-split read visibility, #2489)

Retest with the released `lumen@0.4.24` GHCR image
(`ghcr.io/chrischeng-c4/lumen@sha256:f460c6cf…493e90`, pulled anonymously —
the GHCR distribution path itself works). Passed on both runs: 1x1
reconcile, operator cell, index/search, Workload-Identity GCS backup,
pod-restart retention, and the 1→2 split convergence with a fully converged
post-cutover fence. FAILED both runs at post-split read visibility:
searching the pre-split collection through the client Service returns
`collection not found` and stays unreadable through a bounded 180-second
poll while `phase: Complete` and `convergedShardMapVersion ==
shardMap.version` — tracked as #2489. The prior run `0723041614`'s
post-split pass asserted a single probe and cannot stand as disproof;
treat the Dynamic Shard Topology GKE claim as NOT proven until #2489
closes. Default 1-shard deployments (no reshardPolicy) are unaffected.
Evidence: `axiom-gcp-run-backup/evidence/<run>/`. Resolution: the #2489
scatter fix (`9ffdb30513`) is proven by run `0724061548` above — the claim
is restored there.

### GKE acceptance run 0723041614 (2026-07-23, PASSED)

- Cluster: persistent Standard GKE `axiom-operator-acceptance`
  (`asia-east1-a`, project `axiom-502607`), run-scoped
  bucket/GSA/Workload-Identity binding created and destroyed by the run.
- Image: pinned immutable
  `courier/lumen@sha256:da154652ff3fdf16fb406674240f0a3f4567047d5eb6e0e547bee0f389c68b1b`
  built from commit `f4762759d810` (`git_dirty: false`, `image_provenance:
  prebuilt`, tag `f4762759d810-0723041614`).
- Terminal artifacts: `acceptance.json`
  (`axiom.gcp.lumen.acceptance.v1`, every claimed proof `passed`) and
  `cleanup.json` (`status: clean`, verified `2026-07-23T04:25:33Z`).
  Evidence root: `axiom-gcp-run-backup/evidence/0723041614/` (home-dir
  mirror of the volatile `/tmp` tree); `run.log` carries the full
  transcript.

Proven in this run (each row names its artifact under the evidence root):

| Proof | Result | Artifact |
|---|---|---|
| Operator cell: RBAC, Lease creation, steady-state drift repair, leader-takeover reconcile (holder `...rrc6f` → `...5mlwx`) | passed | `lumen-operator-cell.json`; `kubernetes/lumen-lease-holder-*.txt` |
| 1x1 reconcile: one `Lumen` CR drives exactly one StatefulSet/shard to `Ready` on Standard GKE | passed | `kubernetes/lumen-crs.json`; `kubernetes/workloads-after-lumen-deploy.json` |
| Domain lifecycle through the client Service: create collection, index one document, search hit | passed | `kubernetes/lumen-create-collection.json`; `kubernetes/lumen-index.json`; `kubernetes/lumen-search-before-restart.json` |
| Pod-restart data retention: the indexed document survives a serving-pod replacement and is still searchable via the PVC-backed segment/WAL | passed | `kubernetes/lumen-search-after-restart.json` |
| Workload-Identity GCS backup: CronJob-triggered backup writes a non-empty 271-byte snapshot object; readback is non-empty | passed | `kubernetes/lumen-backup.log`; `gcs/lumen-first-object.json` (`gs://axiom-502607-axo-0723041614-backup/lumen/0723041614-1784780377.json`) |
| Acceptance-only disk-pressure auto-split: `reshardPolicy.maxShardBytes: 1` (a test-only trigger, not a production threshold) drives shard count 1 → 2, 2 ready pods, at least 2 PVCs, and the document stays searchable post-split | passed | `kubernetes/lumen-after-split.json`; `kubernetes/lumen-search-after-split.json` |
| Shard-map fence convergence: post-split CR status settles at `reshard.phase: Complete`, `targetShardCount` == `shardCount` == 2, `usageMeasuredAtMapVersion: 1`, `convergenceRemediationRestartCount: 0` (no remediation restarts needed) | passed | `kubernetes/lumen-after-split.json` |
| Verified cleanup: run-scoped GCS bucket, backup GSA, IAM bindings destroyed (`Destroy complete! Resources: 4 destroyed`, "no run-tagged Lumen/Sift operator acceptance resources remain"); persistent cluster, Artifact Registry, and pre-existing APIs preserved | passed | `cleanup.json`; `run.log` |

Exclusions (recorded, not claimed): CPU/memory pressure actuation
(`cpu_memory_actuator: not_claimed`) and live in-place replica-membership
change (`live_replica_membership: not_claimed`) — neither is exercised by
this harness. `reshardPolicy.maxShardBytes: 1` is an acceptance-only
trigger value chosen to force a split deterministically inside a short
run; it is not evidence of any production disk-pressure threshold. Sift
was deferred from this run (`sift_collection_deferred`); Tape's own run
(`0723135853`) is recorded in `apps/tape/README.md`.

## Benchmarks

### Performance contract — enforced & ratcheting

Beating Postgres and OpenSearch on search is a **standing CI commitment, not a
one-time measurement**: `tests/perf_gate_vs_db.rs` drives lumen, Postgres
(`tokio-postgres`) and OpenSearch (`reqwest`) against one byte-identical corpus
and **fails the build** if lumen loses any *gated* search cell. The authoritative
thresholds live in **`tests/perf-baseline.json`**; full methodology, per-tier
numbers, resource columns, and reproduction live in
**[`docs/benchmarks-scale.md`](docs/benchmarks-scale.md)**.

The product target is **not** "win the tiny loopback request." Lumen is built for
large index state and sustained request volume over HTTP/2 multiplexed
connections. Low-QPS rows remain in the matrix because they catch regressions
early and explain fixed overhead, but the release-relevant performance claim is
high-QPS / large-corpus stability: throughput, p99, RSS, footprint, and peer
comparison under enough concurrency for HTTP/2 pooling to matter.

How the comparison stays honest (separate metrics, never conflated):

- **End-to-end, single-client** is a smoke/regression metric — lumen and
  OpenSearch share HTTP/JSON so the transport tax is visible. pg's binary wire
  beats HTTP/JSON on cheap btree point/range lookups on loopback, so those cells
  are **HTTP-EXEMPT** (annotated) and gated instead through a **native
  prepared-binary** path (Rust wire over Unix socket) — the cheap predicates
  still carry a hard floor.
- **Concurrent qps (10/100/1000)** and **write-path qps** are report-only by
  default; `LUMEN_GATE_COMPARE_PEERS=1 LUMEN_PERF_STRICT=1` strict-gates the peer
  rows recorded in `perf-baseline.json`. Co-located CI keeps them report-only
  until CPU isolation; isolated-host high-QPS repeats are the release-stable bar.

Each cell carries a threshold in `perf-baseline.json`: a **WIN cell** must hold
`max(1.0, 0.8 × recorded margin)` — a **ratchet**, so improving a cell locks the
new bar and it can only get better. **HTTP-EXEMPT cells** (pg btree lookups on
loopback) are separately gated by `pg_native` floors through the native path.
**Scale tiers:** 1K smoke/trend, **10K routine AW/release regression**,
**100K explicit release-local calibration**, and 1M release-soak/research only.
The historical 1M proof is retained evidence; refresh it only with an explicit
soak (`LUMEN_GATE_RELEASE_SOAK=1` or `LUMEN_GATE_N=1000000`).

**Current status — GREEN** (routine gate defaults to 10K Lumen-only regression;
retained historical N=1M in-memory + disk-tier peer evidence). Representative
serial search margins (full set, qps 10/100/1000 tiers, and history in
[`docs/benchmarks-scale.md`](docs/benchmarks-scale.md) / `perf-baseline.json`):

| Cell | vs Postgres | vs OpenSearch (in-mem) | vs OpenSearch (disk) |
|---|---:|---:|---:|
| `text_bm25` | 815× | 4.5× | 23.0× |
| `text_and` | 96.9× | 7.7× | 10.9× |
| `filtered_search` | 61.4× | 7.3× | 4.6× |
| `filter_sort` | 43.9× | 4.1× | 6.0× |
| `pure_sort` | 83.6× | 3.9× | 5.2× |
| `kw_term` | EXEMPT¹ | 4.0× | 9.3× |
| `range` | EXEMPT¹ | 5.2× | 11.3× |
| `bool_filter` | EXEMPT¹ | 5.2× | 6.6× |

¹ pg cheap btree predicates are HTTP-EXEMPT; gated via the native prepared-binary
path — `kw_term` 6.2×, `range` 2.9×, `bool_filter` 39.6× vs pg prepared Unix socket.
Every OpenSearch cell holds a 3.0× WIN baseline (2.4× floor after the ratchet);
paced qps tiers stay ahead of OpenSearch on every WIN cell.

**Write path** — `tests/write_qps.rs` drives the real HTTP `POST /index`; the
legacy NATS/JetStream row remains the historical write-path comparison while
the serving/operator HA path uses Lumen-owned raft. Latest historical 100-worker JetStream run: **8.5× vs
Postgres**, **3.4× vs OpenSearch**, 0 errors. `LUMEN_PERF_STRICT=1` strict-gates
the write margins only when peer services are explicitly present; per-mode
numbers and tuning history live in `benchmarks-scale.md`.

### Footprint & stability

- **Index ~28.8 bytes/doc at 1M** — 5–7× smaller on disk than Postgres /
  OpenSearch; reported as a first-class disk-size metric alongside
  `pg_total_relation_size` and OpenSearch `_stats/store`.
- **RAM=hot/disk=all proven** (`tests/disk_scale_proof.rs`): a reopened
  collection's resident growth is ~30–47% of full-in-RAM and **does not grow with
  N** (forward payload demand-paged off the mmap).
- **Resident ~168 MB vs OpenSearch ~1.4 GB** (~8× smaller); tail p99
  `text_bm25` **1.0 ms** vs OpenSearch ~18 ms (no GC vs JVM pauses).
- **Stability:** 2M sustained searches held RSS flat with zero failed/errored/
  timed-out requests (Rust, no GC; mmap'd segments demand-paged by the kernel).

Full row-count x qps scaling, footprint tables, and retained vs-pg / vs-OS
breakdowns live in **[`docs/benchmarks-scale.md`](docs/benchmarks-scale.md)**.
Routine checks use the Lumen-only vat runner; peer comparisons are refreshed
only through explicit calibration/soak runners when a benchmark cell or peer
configuration changes. Docs-per-shard sizing and the per-shard
indexing/search throughput envelope derived from these same bench surfaces
live under "Capacity guidance" in the Elastic Scale capability above.

## Data model

There are exactly three concepts on the wire:

| Concept       | What it is                                                |
|---------------|-----------------------------------------------------------|
| `Collection`  | A namespace + a schema (a map of field name → field type) |
| `Field`       | One typed column inside a collection                      |
| `external_id` | An opaque string chosen by the caller; lumen never mints it |

There is **no `Document`**. lumen does not store original field values
beyond what the inverted index needs to answer search and duplicate
queries. Hydrating search hits back to full records is the caller's
responsibility against its own store.

## Field types

Schema-first DDL. The declared `FieldType` deterministically picks the
index structure — there is no separate "index options" knob and no
auto-inference.

| FieldType | Index built on write                                                          | Query support              | Duplicate detection |
|-----------|-------------------------------------------------------------------------------|----------------------------|---------------------|
| `text`    | Tokenized inverted index (`token → sorted posting`); analyzer per field       | `match` (BM25, bag-of-words) | No                  |
| `keyword` | Exact inverted index (whole value as one term)                                | `term`, `terms`            | Yes                 |
| `number`  | Sorted inverted index (range-scannable)                                       | `term`, `range`            | Yes                 |
| `set`     | Multi-keyword (one posting per element)                                       | `term` (matches any element) | Yes (per element) |
| `vector`  | Dense `[f32; dim]` + ANN graph (HNSW CPU default; exact flat CPU brute-force) | `knn { vector, k }` with `cosine` / `dot` / `l2` metric | No |
| `hash`    | Caller-supplied 64-bit perceptual/structural hash stored as hex bits         | `hamming { hash, max_distance }` | No; use `hamming` for near-duplicate lookup |

Analyzers available for `text`: `jieba` (Chinese), `whitespace_lower`
(English / generic), `ngram` (configurable min/max). A field is bound
to one analyzer at declaration time.

A field cannot be both `text` and `keyword`. If both are needed (e.g.
"search by email substring *and* find duplicate emails"), declare two
fields and write twice — this keeps write amplification predictable.

## Search concept boundaries

The parity promise is search-side breadth over Lumen's declared contract, not
an implicit claim that every PostGIS/OpenSearch/MongoDB search feature already
exists. These concepts are explicit so agents can choose the right engine or
adapter boundary:

| Concept | Disposition |
|---------|-------------|
| Geo / spatial search | **Roadmap candidate.** Use PostGIS/MongoDB/OpenSearch or a caller-owned geospatial prefilter today, then pass matching `external_id`s to lumen. |
| Phrase / proximity queries | **Roadmap candidate.** Current `match` is bag-of-words BM25 over analyzer tokens, not phrase order or slop. |
| Fuzzy / typo tolerance | **Roadmap candidate.** No edit-distance automaton today; for coarse prefix/substring recall, use the `ngram` analyzer recipe. |
| Synonyms | **Caller-owned.** Expand queries before calling lumen or write normalized companion fields; there is no managed synonym dictionary/analyzer. |
| Autocomplete / suggest | **Recipe.** Declare a dedicated `text` field with `analyzer: "ngram"` and run `match`; lumen returns candidate `external_id`s, not suggestion payloads. |
| Highlighting | **Non-goal.** Search responses contain only `external_id` + `score`; lumen does not store source text to return snippets/fragments. |
| Per-field / per-clause boost | **Boundary.** No arbitrary boost knob today; use separate fields/query legs plus `rrf`, then rerank in the caller if needed. |
| Document TTL / expiry | **Caller-owned lifecycle.** Delete/reindex expired `external_id`s from the source-of-truth event stream; collection soft-delete grace is not per-document TTL. |

## API surface

All endpoints are HTTP/2 JSON. The authoritative request / response
schemas are served by a running pod at `GET /openapi.json`. Offline
codegen pipes that spec out of the `lumen-openapi-dump` binary; see
[OpenAPI](#openapi) below.

### Schema (DDL)

```
PUT /collections/{id}
{
  "fields": {
    "bio":       { "type": "text",    "analyzer": "jieba" },
    "email":     { "type": "keyword" },
    "tags":      { "type": "keyword", "multi": true },
    "age":       { "type": "number" },
    "embedding": { "type": "vector",  "dim": 768, "metric": "cosine",
                   "backend": "hnsw-cpu", "quantize": "sq" },
    "avatar_phash": { "type": "hash" }
  }
}
→ 200 { "collection_id": "users", "version": 1, "fields_count": 6 }
```

Online: adding a new field is immediate (postings start empty).
Re-declaring an existing field with the same spec is a no-op (PUT is
upsert-merge). Changing a field's type is rejected — drop the field
(`DELETE /collections/{id}/fields/{name}`) and re-add. `vector` field
configuration (`dim` / `metric` / `backend` / `quantize`) is immutable
for the field's lifetime. `hash` has no schema-time hash-kind parameter:
the caller computes pHash, SimHash, b-bit MinHash, or another 64-bit signature
and writes it as a 16-hex-character string (optional `0x` prefix accepted).

### Index (write)

```
POST /collections/{id}/index
{
  "items": [
    { "external_id": "u_123", "field": "bio",   "value": "senior engineer in Taipei" },
    { "external_id": "u_123", "field": "email", "value": "a@x.com" },
    { "external_id": "u_123", "field": "tags",  "value": ["rust","db"] },
    { "external_id": "u_123", "field": "avatar_phash", "value": "f0e1d2c3b4a59687" }
  ],
  "request_id": "..."        // optional, dedup TTL 5 min
}
→ 200 { "indexed": 4, "bytes_written": { "bio": 412, "email": 33, "tags": 88, "avatar_phash": 12 }, "shard_lag_ms": 4 }
```

Re-writing `(external_id, field)` fully re-indexes that field. There
is no partial update. `/index` is a **merge**: only the fields you send are
touched. Own only some fields of a doc? Use `/index`. Own the doc's
**complete** row? Use `docs:replace` below.

### Full-replacement writes (docs:replace)

```
PUT /collections/{id}/docs:replace
{ "docs": [
    { "external_id": "row-42", "version": 7, "fields": { "title": "New title", "state": "open" } }
] }
→ 200 { "results": [
    { "status": "ok", "fields_written": 2, "fields_skipped": 0 }
] }

PUT /collections/{id}/docs/{external_id}          # single-resource sugar
{ "version": 7, "fields": { "title": "New title", "state": "open" } }
→ 200 { "status": "ok", "fields_written": 2, "fields_skipped": 0 }
```

`docs:replace` is a batch **full-replacement** upsert: each item's `fields`
becomes the doc's *entire* indexed state — a declared schema field the doc
has today but that is absent from `fields` is **implicitly deleted**.
`docs:replace` is one literal path segment appended after
`{collection_id}` (AIP-136 custom-method syntax), so it registers directly
in axum next to `/collections/{collection_id}/docs/{external_id}` with no
capture ambiguity — collection ids may not contain `:` for the same reason.

**PUT is deliberate**: this is idempotent full replacement, so replaying
the same request converges to the same state. **Own the complete row for a
doc? Use `docs:replace`. Own only some fields? Use `/index`** — `/index` is
a merge, `docs:replace` is a full replacement.

`version` is optional **doc-level** last-write-wins over the caller's own
source-row version — distinct from `/index`'s `IndexItem.version`, which is
per-`(external_id, field)` cell versioning. A strictly-older version
arriving later drops the *entire* item and is reported as its own
`{"status":"dropped","current_version":...}` result, kept separate from
both `ok` and `error` so callers can tell "a newer write already won" apart
from both success and failure. Each `ok` result carries `fields_written`
and `fields_skipped` counters; `fields_skipped` (unchanged-value no-op
suppression) is always `0` today.

**A per-item failure never fails the batch**: the batch-level HTTP status
stays 200 unless the body is malformed or the batch is over the size limit
(max 32 items — `MAX_BATCH_REPLACE_SIZE`, the same knob family as
`collections:search`'s `MAX_BATCH_SEARCH_SIZE`) — those return 400.
`PUT /collections/{id}/docs/{external_id}` is single-resource sugar for a
one-item batch, unwrapped back into a bare per-item result.

### Delete

```
DELETE /collections/{id}/index/{external_id}             → 204    # all fields
DELETE /collections/{id}/index/{external_id}?field=bio   → 204    # one field
```

### Search

```
POST /collections/{id}/search
{
  "query": {
    "and": [
      { "match": { "field": "bio",  "text": "engineer taipei", "op": "and" } },
      { "term":  { "field": "tags", "value": "rust" } },
      { "range": { "field": "age",  "gte": 25, "lt": 40 } }
    ]
  },
  "limit": 20,
  "cursor": null
}
→ 200 {
  "hits": [
    { "external_id": "u_123", "score": 4.21 },
    { "external_id": "u_087", "score": 3.95 }
  ],
  "total": 217,        // estimate; ">10000" when truncated
  "cursor": "eyJvZmZzZXQiOjIwfQ==",
  "took_ms": 6
}
```

Search responses **only carry `external_id` + `score`** — never field
values. There is no `_source`.

**Pagination is keyset (search-after), depth-invariant.** The `cursor` is an
opaque token bound to the query that produced it: echo it back unchanged to
get the next page. For sorted (single number field) and score-ranked results
the token carries the LAST hit's position, so every page **seeks** —
O(log n) on the sorted index — instead of skipping; deep pages cost the same
as page 1 (measured at depth 50k over 100k docs: 86µs vs 28.7ms offset
skip). Stop when `cursor` is null. Legacy `{"offset":N}` tokens keep working
(O(offset) skip). Note: when continuing from a keyset cursor with
`track_total: true`, `total` counts the REMAINING matches from the cursor,
not the full set — read the full total off the first page.

**`range` also accepts a string bound** (`gte`/`lte`/`gt`/`lt`) against a
`keyword` field for byte-lexicographic (string/date) ranges, e.g.
`{ "range": { "field": "created_at", "gte": "2026-01-01", "lt": "2026-02-01" } }`
— a numeric bound against a `keyword` field or a string bound against a
`number` field is rejected with 400 rather than silently misparsed.

**`QUERY /collections/{id}` is a dual-registered twin of this endpoint**
(RFC 10008): same request body, same handler, byte-identical response.
`OPTIONS`/`HEAD` on either target advertise `Accept-Query: application/json`.
POST remains the permanent fallback for clients without QUERY support.

The **`X-Read-Consistency`** request header (`leader` / `bounded(<ms>)` / `any`
— default and safest is `leader`; missing/unrecognized values also fall back
to `leader`, an owner decision kept as-is with no formal release yet to
force a compatibility bar) is enforced against live cluster state in
primary-replica (raft) mode: `leader` only succeeds on the pod currently
holding leadership, and a request that fails the check is rejected rather
than silently served from a possibly-stale replica. **`bounded(<ms>)`
succeeds on the leader (never stale) but currently always rejects on a
follower/learner**: lumen does not yet measure real replication lag between
peers, so a non-leader replica reports the conservative "lag unknown"
sentinel and is treated as over any bound rather than risk serving a stale
read. Real follower lag reporting (and `bounded` actually succeeding on a
caught-up follower) is future work — until then, `bounded(<ms>)` is
effectively `leader` with an extra rejection path for followers. Standalone
deployments (no raft) ignore the header.

### Batch search (msearch-style, multi-collection)

```
POST /collections:search
{ "searches": [
    { "collection": "users",    "query": { "term": { "field": "tags", "value": "rust" } }, "limit": 10 },
    { "collection": "products", "query": { "match": { "field": "title", "text": "earbuds" } }, "limit": 5 }
] }
→ 200 { "results": [
    { "status": "ok", "response": { "hits": [...], "total": 2, "cursor": null, "took_ms": 1 } },
    { "status": "error", "code": "collection_not_found", "message": "..." }
] }
```

`collections:search` is one literal path segment (AIP-136 custom-method
syntax), so it registers alongside `/collections/{collection_id}` with no
ambiguity — for the same reason, collection ids may not contain `:`. Each
item is a full `{"collection", ...SearchRequest}` — `limit`, `sort`,
`cursor`, `collapse`, `routing_key`, `track_total` may all differ per item.
`results` has the same order and length as `searches`. **A per-item failure
never fails the batch**: the batch-level HTTP status stays 200 unless the
body is malformed or the batch is over the size limit (max 32 items, which
also bounds the concurrent fan-out) — those return 400. Pagination stays
per-item: resubmit one item with its returned `cursor` to continue it. There
is no merged cursor and no cross-collection score merging/ranking.

### Duplicates

```
POST /collections/{id}/duplicates
{ "field": "email", "min_group_size": 2, "limit": 100 }
→ 200 {
  "groups": [
    { "value": "a@x.com", "external_ids": ["u_123","u_456","u_789"] },
    { "value": "b@y.com", "external_ids": ["u_201","u_990"] }
  ],
  "truncated": false,
  "took_ms": 12
}
```

`text` / `vector` fields do not support duplicates (semantics undefined).

### Exists / Duplicated (presence & collision filters)

Two query nodes for presence and collision. Both compose inside `and` / `or` /
`not` like any other leaf, so arbitrary combinations ("non-blank email **and**
duplicate phone") need no bespoke endpoint.

```
POST /collections/{id}/search
{
  "query": {
    "and": [
      { "exists":     { "field": "email" } },                      // email is non-blank
      { "duplicated": { "field": "phone", "min_group_size": 2 } }  // phone collides with another doc
    ]
  }
}
```

| Node | Matches |
|------|---------|
| `exists` | docs holding any value for `field`; `not exists` = "is empty" |
| `duplicated` | docs whose `field` value is shared by ≥ `min_group_size` docs (`min_group_size` defaults to / floors at 2) |

Both cover `keyword` / `number` / `set` fields. `text` / `vector` / `hash` are
rejected (presence/equality is undefined there — declare a `keyword` companion
field for a text "is empty" / duplicate filter).

`duplicated` vs the `/duplicates` endpoint: the endpoint returns *grouped*
results (`value → external_ids`) for an audit view; the `duplicated` query node
returns a *flat, composable* doc set you can intersect with other predicates in
one search.

### kNN (vector search)

```
POST /collections/{id}/search
{
  "query": {
    "knn": {
      "field": "embedding",
      "vector": [0.12, -0.04, ...],
      "k": 10
    }
  },
  "limit": 10
}
→ 200 {
  "hits": [
    { "external_id": "u_123", "score": 0.94 },
    { "external_id": "u_087", "score": 0.91 }
  ],
  "total": 10,
  "took_ms": 3
}
```

Scores are direction-normalised so higher = better regardless of
metric (`cosine` / `dot` use the raw similarity; `l2` reports
negated distance). `knn` can be composed inside `and` / `or` /
`not` with the other query nodes.

### Schema lifecycle

```
PUT    /collections/{id}                          # create or upsert-extend
DELETE /collections/{id}/fields/{field_name}      # online field drop
DELETE /collections/{id}                          # soft-delete (24h grace)
DELETE /collections/{id}?force=true               # immediate physical drop
GET    /collections                               # list (filtered by RBAC)
```

### Admin & ops

```
GET  /admin/backup                                # full SnapshotV1 JSON dump
POST /admin/restore                               # replace state from a snapshot
POST /admin/backup/local                          # snapshot → LocalFsSink (path + prefix)
POST /admin/backup:scoped                         # SnapshotV1 restricted to a set of virtual buckets
POST /admin/reshard:apply                         # additively merge one ReshardBatch into live state
POST /admin/reshard:evict                         # remove docs no longer owned under a newer shard map
POST /admin/reshard:fence                         # arm/clear a bounded write pause on a set of buckets
POST /admin/reshard:prune                         # accumulate + prune the final migration pass's keep set
POST /admin/checkpoint                            # force a synchronous full-state durability checkpoint
GET  /debug/cluster                               # pod/shard/role/peers/replication-lag
GET  /metrics                                     # Prometheus text format
GET  /healthz                                     # liveness
GET  /readyz                                      # readiness (503 while draining)
GET  /openapi.json                                # live OpenAPI spec
GET  /docs                                        # Swagger UI (interactive "Try it out")
```

`backup:scoped` / `reshard:apply` / `reshard:evict` / `reshard:fence` /
`reshard:prune` are the operator-driven reshard data-plane verbs:
`backup:scoped` exports only the documents routed to a requested set of
virtual buckets (the same hash the engine's own routing uses),
`reshard:apply` idempotently upserts one such export's batch into a target
shard, `reshard:evict` removes exactly the documents a supplied newer
virtual-bucket map no longer routes to that shard, `reshard:fence` arms or
clears a bounded (default 300s, max 3600s) write pause on a set of buckets
so a write mid-cutover is rejected with a retryable `503
bucket_write_paused` instead of racing the map change, and `reshard:prune`
accumulates a final migration pass's authoritative per-bucket "keep" id set
and prunes anything absent from it once complete. All five are
`Role::Admin`-gated and idempotent on retry; `reshard:fence` is
driver-owned (`service_k8s::reshard_driver::advance_catching_up`) — manual use
outside driver-orchestrated cutover risks a real write outage.
`reshard:apply`/`reshard:evict` mutate engine state directly, bypassing the
normal WriteCoordinator/AOF write path, so the reshard driver calls `POST
/admin/checkpoint` — a synchronous full-state segment checkpoint returning
`{"persisted": bool}` — on every touched shard before cutover, making the
migration durable ahead of the rolling-restart that flips the live shard
map.

### Stats

Engine **metadata** about one collection. Per the v1 non-goals, this
describes the *index* (size, cardinality, cache health) — not the
caller's data. There are no aggregations here.

```
GET /collections/{id}/stats
→ 200 {
  "documents_indexed": 1234567,
  "fields": {
    "email": { "type": "keyword", "unique_terms": 1233110, "bytes": 40128830 },
    "bio":   { "type": "text",    "unique_terms": 482113,  "bytes": 32108920, "avg_doc_len": 28.4 },
    "age":   { "type": "number",  "unique_terms": 81,      "bytes": 9876543 }
  },
  "storage": { "total_bytes": 82114293 },
  "cache":   { "posting_hit_ratio": 0.87 },
  "last_indexed_at": "2026-05-28T16:42:11Z"
}
```

`last_indexed_at` is the typical "did my writes land?" probe — caller
writes N docs, then asserts `documents_indexed == N` and
`last_indexed_at` advanced. For Prometheus-shaped continuous
monitoring, `/metrics` carries the same numbers as gauges.

## HTTP & clients

The client API speaks **HTTP/1.1 and HTTP/2 cleartext (h2c) on the same
port** (`auto`) — the server accepts both, no flag needed. **HTTP/2 is the
recommended connection for serving**: one connection multiplexes many concurrent
streams, which is how lumen sustains its high-QPS search/index throughput. Small
HTTP/1.1 calls are compatibility and smoke paths; production performance claims
are about pooled HTTP/2 traffic at volume. The
three setups, in order of preference:

- **Production (private ClusterIP TLS) — HTTP/2 by default, for free.** Lumen
  terminates TLS itself on `https://<instance>.<namespace>.svc:7373` and offers
  ALPN `h2, http/1.1`, so every client gets h2 transparently. Nothing sits in
  front: no ingress, no mesh, no other TLS terminator (see
  [Authentication and authorization](#authentication-and-authorization)). This
  is the recommended deployment.
- **Cleartext (dev / in-cluster) — h2c is opt-in.** h2c can't auto-negotiate (no
  ALPN), so a client must enable prior-knowledge (see table). A lumen connection
  *pool* over h2c is what the benchmark throughput numbers use.
- **Zero-driver fallback — plain HTTP/1.1 always works**, no special client:
  `requests`, `httpx`, `fetch`, `curl`, any REST client (lumen ships no client
  SDK — it's pure REST/OpenAPI; see `lumen llm`).

| Client | HTTP/1.1 | h2c (cleartext) opt-in | h2 over TLS (prod) |
|--------|----------|------------------------|--------------------|
| Python `requests` | ✅ | ✗ (no h2 support) | ✗ |
| Python `httpx` | ✅ | `pip install "httpx[http2]"` + `Client(http2=True)` | ✅ ALPN |
| `curl` | ✅ | `--http2-prior-knowledge` | `--http2` |
| Go `net/http` | ✅ | needs `x/net/http2` h2c transport | ✅ ALPN |
| browser (Swagger `/docs`) | ✅ | ✗ (browsers require TLS) | ✅ ALPN |

### Authentication and authorization

> This section describes what the binary enforces, not a target. The bearer
> registry, Google-token verifier, Secret Manager/CSI auth projection,
> metadata-server token path, and token environment injection are removed —
> the CRD fields that configured them no longer exist, so a manifest that
> restores one is rejected by the API server rather than quietly ignored.

#### Externally Provisioned TLS Secrets

Deployment administrators or an external platform provision the serving and peer
TLS Secrets named by each `Lumen` instance. The operator only consumes those
Secrets; it does not resolve issuers, perform CAS automation, or own a trust
domain.

Two independent checks stand between a caller and a collection, and neither
substitutes for the other: the **transport** proves which server you reached,
and the **request identity** proves who is asking.

#### Transport: private ClusterIP TLS, terminated by lumen

Production traffic is **not** published. An instance is reached at its Service
DNS name inside the cluster and nowhere else:

```text
LUMEN_URL=https://<instance>.<namespace>.svc:7373
```

There is no Ingress, no Gateway, no LoadBalancer, no NodePort, and no service
mesh terminating TLS on lumen's behalf. The serving pod holds the private key
itself, so the connection a caller authenticates is the connection lumen
serves — an edge that terminated TLS and re-originated plaintext would carry
the KSA token over an unauthenticated last hop while every client-side check
still passed.

`spec.servingTlsSecret` names the Secret holding `tls.crt`, `tls.key`, and
`ca.crt`; the operator projects it into every serving pod and switches the
client port from h2c to TLS with ALPN `h2, http/1.1`. The leaf asserts the
Service's own two DNS spellings and nothing else — a name in the certificate is
a name the instance can impersonate. While no valid leaf is active the port
refuses connections rather than falling back to plaintext. Omit the field only
for local and kind development.

Callers verify against the anchor alone: the deployment administrator or an
external certificate platform distributes the public CA separately from the
private-key-bearing serving Secret. Supply that CA with `lumen connect
--ca-file`, or as `PrivateTrust` in a generated client. It replaces the public
roots rather than joining them, so a public CA cannot vouch for this private
Service DNS name.

#### Request identity: a short-lived KSA token the cluster answers for

For `spec.auth: required`, Lumen accepts only a short-lived Kubernetes
ServiceAccount token with audience `lumen.axiom.dev`:

```text
Google user or Google service account
  -> authenticate to kube-apiserver through kubeconfig
  -> RBAC-authorized TokenRequest for one explicitly named client KSA
  -> short-lived KSA token
  -> Lumen TokenReview
  -> system:serviceaccount:<namespace>:<name>
  -> Lumen SubjectAccessReview
```

Google credentials stop at kube-apiserver. A Google access token, Google ID
token, ADC credential, GSA credential, or metadata-server token sent directly
to Lumen is rejected even if GKE would accept that principal at the Kubernetes
API boundary.

Lumen maps authenticated requests to virtual Kubernetes resources in API group
`lumen.axiom.dev`:

| Lumen decision | Kubernetes resource attribute |
|---|---|
| read one collection | `get` on `lumencollections/<collection-id>` |
| write one collection | `update` on `lumencollections/<collection-id>` |
| administer one collection | `delete` on `lumencollections/<collection-id>` |
| instance-level administration | the corresponding verb on `lumenadmin` |

The request namespace is part of every decision. Collection-list and
multi-collection operations authorize the concrete resources they touch; an
instance admin grant is not modeled as wildcard access to every collection.
Authentication failures return 401 and authenticated denials return 403.

The Lumen CLI uses the current kubeconfig, including the GKE credential plugin,
to request a 600-second token for an explicitly supplied namespace and client
KSA. `lumen query` keeps the token in memory. `lumen connect` gives its child
only a loopback URL and injects the header in a local proxy; it does not expose
the token through environment, argv, files, clipboard, or stdout.

The account is named per invocation and never inferred: `--client-sa` has no
environment fallback and the CLI does not pick a ServiceAccount by listing the
namespace. Omit it and the connection carries no identity at all, which is
correct only against a fleet with `auth: disabled`. Minting needs `create` on
that ServiceAccount's `token` subresource; `lumen k8s access render` emits the
grant, and a refusal names the Kubernetes username the cluster saw, the target
account, and the `kubectl auth can-i` that answers "may I?".

Serving, operator/reshard, backup, and external-client ServiceAccounts are
separate identities with least-privilege bindings. TokenRequest permission is
restricted to one named client KSA and is never a namespace-wide wildcard.
Probe/spec/scrape routes (`/healthz`, `/readyz`, `/metrics`, `/openapi.json`,
`/docs`) remain auth-exempt.

Raft peer identity is a separate plane. Replicated traffic on `:7374` requires
an instance-scoped X.509 certificate and mTLS, with no plaintext fallback. A
KSA token does not authenticate a peer, and a peer certificate grants no
collection or admin access. `spec.peerTlsSecret` is a separate field from
`spec.servingTlsSecret` for that reason: sharing one Secret would let either
listener's material authenticate on the other's port.

Public exposure of any shape — Gateway, Ingress, LoadBalancer, NodePort, VPN,
or a mesh terminating TLS — is outside the Security & Access capability, as are
Google IAM automation and general user/group management. So is client mTLS:
a caller proves who it is with a KSA token, not with a certificate.

## OpenAPI

| Artefact              | When to use                                                  |
|-----------------------|--------------------------------------------------------------|
| `GET /openapi.json`   | Live spec from a running pod — codegen against an actual env |
| `GET /docs`           | Interactive Swagger UI ("Try it out")                        |
| `lumen spec`          | Offline OpenAPI JSON from the installed binary               |
| `lumen spec --format openapi-yaml` | Offline OpenAPI YAML for agent review         |
| `lumen spec --format json-schema` | Component schemas for the request/response types |
| `lumen spec gen --lang ts\|py\|rust [--target <profile>] --out <dir>` | In-tree typed client generation with a pinned target contract |

`lumen spec` and the live endpoint generate from the same Rust code
(`#[derive(utoipa::OpenApi)]` on `api::ApiDoc`). There is no committed OpenAPI
snapshot; the binary and live endpoint are the source of truth.

`projects/lumen/clients/codegen.toml` pins the default TypeScript, Python, and
Rust targets. `spec gen` writes `.openapi-codegen.json` beside every
generated client; use `--target python-3.11` (or another supported profile)
only for a deliberate one-off compatibility override.

Generated Python clients include pydantic models plus a stdlib HTTP/2 runtime.
For auth-enabled deployments:

```python
from generated_api import Client

client = Client("http://lumen.default.svc.cluster.local:7373", auth_token="...")
```

`default_headers={"Authorization": "Bearer ..."}` is also supported. The
generated `h2c_runtime.py` exposes unary `request()` and bidirectional
`stream()` APIs; Lumen's current OpenAPI routes are unary, so generated
`client.py` uses `request()` today and the streaming surface is forward-looking
runtime capacity for services that add streaming operations.

## Design notes (from the retired HA.md, 2026-07)

Durable decisions folded from the retired `HA.md`; its session-era "Original
design notes (openraft)" framing was already superseded by the shipped
`raft-core`/`raft-runtime` implementation and is dropped as historical.

lumen is a **log-replicated, derived, rebuildable search index**: the caller
still owns the source of truth, and lumen indexes the caller's `external_id`s.
The deployment boundary changed once `libs/raft-core` existed: multi-pod lumen
owns its own write ordering and replica synchronization instead of requiring
an external broker as the default HA path. Mode split:

- **standalone**: one pod, embedded WAL, direct apply.
- **primary-replica**: multiple lumen pods, `raft-core` elects a leader, the
  leader owns the ordered write log, and followers replicate/apply the same
  raw `WalRecord::encode()` bytes.

`lumen serve --wal auto` is the production default: it starts embedded when no
k8s replica topology is present, and switches to raft when
`REPLICAS_PER_SHARD > 1` is injected by the operator/StatefulSet. The storage
topology contract is `totalPods = shardCount * replicasPerShard`:
`replicasPerShard` selects the HA mode for each shard group, while
`shardCount` selects how many physical storage shards own the corpus. A
deployment with `shardCount > 1` and `replicasPerShard = 1` is sharded but not
raft-replicated; a deployment with `shardCount = 1` and `replicasPerShard > 1`
is one shard with raft replicas. StatefulSet pod ordinals map deterministically
to `shardIndex = ordinal % shardCount` and
`replicaIndex = ordinal / shardCount`.

The operator never passes special cluster flags — topology comes from the
downward API (`POD_NAME`, `POD_NAMESPACE`, `SHARD_COUNT`,
`REPLICAS_PER_SHARD`, `VOTER_COUNT`, `LUMEN_HEADLESS_SERVICE`): one serving pod
renders a standalone Deployment + HPA, `replicasPerShard > 1` renders stable
serving StatefulSets + headless Services. For local multi-node work,
`LUMEN_PEERS=host:port,...` overrides headless DNS so several
`lumen serve --wal raft` processes can run on one machine.

Dynamic shard growth is an operator workflow, not a direct HPA response. The
default routing contract uses virtual buckets:
`bucket = hash(collection_id, routing_key || external_id) % virtualBucketCount`,
then a versioned bucket-to-physical-shard map decides ownership. Search without
a routing key scatters/gathers across shards; search with a routing key can
target one shard. Operators should prepare a split around storage pressure
(for example 50% of the configured shard ceiling), start or recommend split
based on growth and safety windows, treat high utilization as urgent, and avoid
auto-split when the max shard size or max shard count is unknown.

Raft responsibility is split by crate/module: `libs/raft-core` (consensus
state machine and log semantics), `libs/raft-runtime` (h2c peer transport,
leader forwarding, snapshot install, log compaction — snapshot upload/pruning
policy lives in `libs/service-backup`), `apps/lumen/src/raft_sm.rs`
(committed write records → engine mutations, snapshot produce/restore), and
`apps/lumen/src/raft.rs` (API-facing cluster/debug DTOs, read-consistency
parsing). Legacy broker-backed write logs are not part of the Lumen
deployment archetype; the NATS backend is compatibility/test surface only, and
Relay WAL support has been removed from Lumen.

Bootstrap modes are intentionally distinct. A restarted pod with its PVC
replays local raft state, snapshots, and logs. A new empty-PVC replica can catch
up through leader snapshot install and AppendEntries today, but the production
path is to seed from object-store/shard snapshot first and then catch up the
raft delta, with operator-visible progress and rate limits. Production Lumen
CRs must configure scheduled object snapshots; local filesystem snapshots are
local-dev or break-glass only. External backup is the cold disaster-recovery
and seed surface; it is not the normal live replica synchronization path.
