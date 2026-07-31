# Lumen Capabilities

## Brief

Lumen is a derived-index service. Its core job is to build indexes over
caller-owned data and query those indexes. Lumen is not a system of record,
analytics engine, identity provider, or certificate authority.

## Capabilities

Every capability belongs to exactly one of two feature roots:

- **Core Features** define what Lumen fundamentally does: Indexing and
  Querying.
- **Non-Core Features** make those two jobs deployable, secure, scalable,
  recoverable, observable, and integrable. Non-core does not mean optional.

This file contains stable product promises, claim IDs, and verification
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

ID: `indexing`

Status: ready

Promise: Build and maintain rebuildable indexes over caller-owned
`external_id` values. The caller supplies source data, embeddings, and
perceptual hashes; Lumen owns schema validation, lexical analysis, index
mutation, segment/checkpoint persistence, and deterministic rebuild.

Claims:

- `schema-and-index-lifecycle` — schemas and mutations are validated and
  applied consistently.
- `derived-index-storage` — retained index state survives restart and can be
  rebuilt without becoming the source of truth.
- `indexing-quality` — indexing meets the declared throughput, footprint, and
  long-running stability floors.

Verification:

- `cargo test -p lumen --test api_e2e --test drop_field_e2e --test reindex_stream_e2e --test stats_metadata_e2e`
- `cargo test -p lumen --test perf_gate --test perf_gate_vs_db`
- `apps/lumen/tests/rig/cases`

#### Querying

ID: `querying`

Status: ready

Promise: Query Lumen indexes and return ranked or filtered caller-owned
`external_id` values. Supported semantics include lexical BM25, exact and
range filters, vector kNN, Hamming hash search, hybrid RRF, duplicates,
nested/group/collapse behavior, pagination, sorting, and explicit read
consistency.

Claims:

- `lexical-and-structured-query` — lexical, exact, range, pagination, and sort
  behavior is deterministic.
- `semantic-and-similarity-query` — vector, hash, hybrid, duplicate, and nested
  queries preserve their documented semantics.
- `query-quality` — unsafe shapes are rejected and query latency, throughput,
  and footprint stay within the declared floors.

Verification:

- `cargo test -p lumen --test api_e2e --test coverage_gaps_e2e`
- `cargo test -p lumen --test vector_e2e --test hash_hamming --test hybrid_rrf --test collapse_nested`
- `cargo test -p lumen --test perf_gate --test perf_gate_vs_db`

### Non-Core Features

#### Kubernetes-Native Deployment

ID: `kubernetes-native-deployment`

Status: ready

Promise: Render the image, CRD, operator, and instance layers independently,
then reconcile each Lumen instance into stable Kubernetes workloads,
networking, conditions, disruption protection, and optional isolation.
Reusable Kubernetes mechanics stay in shared libraries; Lumen owns its CRD
policy and app wiring.

Claims:

- `layered-deployment-artifacts` — Dockerfile, CRD, operator, and instance
  renderers remain independently usable.
- `live-operator-reconciliation` — desired state converges and owned resources
  are repaired without taking over unrelated objects.

Verification:

- `cargo test -p lumen --features operator --test operator_render --test operator_backup_kubernetes_wiring`
- `apps/lumen/scripts/kind-e2e.sh`
- `acceptance/gcp/scripts/run.sh`

#### Security & Access

ID: `security-hardening`

Status: not ready

Promise: Use Kubernetes as the client request identity and authorization
boundary, a separate X.509 identity plane for replicated Raft traffic, and
rustls for serving transport confidentiality.

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
  -> leaf verified against the operator-published trust anchor
```

Invariants:

- Serving TLS terminates in the Lumen process. No Ingress, Gateway,
  LoadBalancer, NodePort, or service mesh terminates it on Lumen's behalf, so
  no hop between a caller and Lumen carries a request token in the clear.
- The serving leaf asserts the instance's own Kubernetes Service DNS names and
  nothing else.
- A configured serving certificate never degrades to plaintext; the client port
  refuses connections while no valid leaf is active.
- The client trust anchor is published without the private key, and replaces
  the public roots for callers rather than joining them.
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

Claims:

- `kubernetes-native-request-identity-and-authorization` — permitted KSA
  requests pass TokenReview/SAR and invalid or denied requests fail closed.
- `instance-scoped-raft-peer-identity` — only valid instance peers can use the
  Raft transport, including through rotation and failover.
- `serving-transport-tls` — the rustls-backed serving transport terminates
  private ClusterIP TLS in-process, publishes a key-free trust anchor, and
  admits no plaintext or unverified path.

Verification:

- `cargo test -p lumen -p service-auth -p service-k8s -p peer-tls`
- `apps/lumen/tests/auth_e2e.rs`
- `apps/lumen/tests/authz_matrix_e2e.rs`
- `apps/lumen/tests/operator_render.rs`
- `apps/lumen/tests/security_lumen_claim_security_tls_rustls.rs`
- `apps/lumen/tests/serving_tls_rotation.rs`
- `acceptance/gcp/scripts/run.sh`
- `acceptance/gcp/scripts/verify-lumen-auth.sh`

Ready when one retained GKE evidence bundle proves KSA allow/deny, direct
Google rejection, peer mTLS positive/negative behavior, credential rotation,
failover, and cleanup. Retired bearer/Google-registry evidence cannot close
this capability.

#### Scaling & Availability

ID: `scaling-availability`

Status: ready

Promise: Scale index state and serving capacity without changing indexing or
query semantics. Lumen uses RAM-hot/disk-all segments, a versioned
virtual-bucket shard map, checkpointed reshard transitions, one Raft group per
shard, explicit replica policy, failover, and replacement bootstrap.

Claims:

- `elastic-segment-tier` — hot memory and retained disk tiers obey their
  resource contract.
- `dynamic-shard-topology` — resharding converges without losing readable
  indexed data.
- `primary-replica-failover-and-bootstrap` — replicas synchronize, fail over,
  and replace failed members.

Verification:

- `cargo test -p lumen --test reshard_admin_e2e`
- `cargo test -p lumen --test efficiency_lumen_claim_elastic_disk_tier`
- `cargo test -p lumen --test stability_lumen_topology_existing_raft_replica_sync --test stability_lumen_claim_dynamic_multi_shard_replica_kind`
- `apps/lumen/scripts/kind-e2e.sh`

#### Durability & Recovery

ID: `durability-recovery`

Status: ready

Promise: Recover derived index state through WAL/checkpoint replay, Raft
replication, backup/restore, and cold seed without claiming ownership of the
caller's source data.

Claims:

- `wal-checkpoint-and-raft-recovery` — committed index mutations survive
  restart and member replacement.
- `backup-restore-and-cold-seed` — a retained snapshot restores into a fresh
  instance and remains readable after restart.

Verification:

- `cargo test -p lumen --test backup_restore_e2e`
- `cargo test -p lumen --test stability_lumen_topology_existing_raft_replica_sync`
- `acceptance/gcp/scripts/run.sh`

#### Operations & Observability

ID: `operations-observability`

Status: ready

Promise: Expose health, readiness, conditions, metrics, events, alerts,
tracing, and long-running-operation state for both serving and control-plane
behavior.

Claims:

- `standard-operational-surfaces` — health, readiness, metrics, and status
  reflect real service state.
- `control-plane-observability` — reconciliation, leadership, errors, and
  alerts are externally observable.
- `long-running-stability` — retained workloads stay within declared resource
  and correctness bounds.

Verification:

- `cargo test -p lumen --test api_e2e`
- `cargo test -p lumen --features operator --test operator_backup_kubernetes_wiring`
- `apps/lumen/tests/rig/cases`
- `apps/lumen/k8s/components/operator-monitoring`

#### API, CLI & Agent Integration

ID: `api-cli-agent-integration`

Status: ready

Promise: Expose the two core jobs through HTTP/1.1 and HTTP/2, served and
offline OpenAPI, generated clients, the standard `llm`/`upgrade`/`issue`
surface, deployment commands, chainable output, and offline agent guidance.

Claims:

- `http2-openapi-and-client-interface` — wire behavior and published schemas
  stay aligned.
- `standard-cli-and-agent-interface` — commands remain discoverable,
  executable, and explicit about their next step or terminal state.

Verification:

- `cargo test -p lumen --test spec_cli --test api_e2e`
- `cargo test -p lumen --test cli_convention`
- `cargo test -p lumen --test behavior_lumen_claim_cli_deployment_operator_command_surface`
