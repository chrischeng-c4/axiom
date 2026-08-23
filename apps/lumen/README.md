# Lumen

## Brief

Lumen is a Kubernetes-native search service.

It builds a derived index over caller-owned data. The caller keeps the source
records and chooses each `external_id`. Lumen returns those IDs with search
scores. It does not return or own the source records.

Lumen supports five search modes:

| Search mode | Main field types | What it finds |
|---|---|---|
| Exact | `keyword`, `number`, `set` | Exact values, ranges, prefixes, and set membership |
| Lexical | `text` | BM25-ranked text matches |
| Vector | `vector` | Exact or HNSW nearest neighbours |
| Perceptual | `hash` | Values within a Hamming distance |
| Duplicates | `keyword`, `number`, `set` | IDs that share the same value |

The caller or its ingest pipeline supplies raw vectors and perceptual hashes.
Lumen does not run an embedding model. Lumen tokenizes `text` fields and owns
schema validation, index mutation, query behavior, shard ownership,
replication, and recovery of the derived index.

The client API uses port `7373`. Replicated peers use a separate identity and
transport plane on port `7374`.

The HTTP examples below assume a reachable endpoint and omit deployment and
authentication setup. Use the [deployment guide](docs/deployment.md) for the
connection path.

## Primary workflow

A normal integration has four steps:

1. Declare a collection and its fields.
2. Write caller-owned values into the index.
3. Search and receive `external_id` values.
4. Bulk-load the full records by ID and restore Lumen's result order.

The request and response examples below use the current 0.4 contract. The 0.5
target is documented separately and is not implemented.

### 1. Declare a collection

```http
PUT /collections/users
Content-Type: application/json

{
  "fields": {
    "bio": { "type": "text", "analyzer": "whitespace_lower" },
    "email": { "type": "keyword" },
    "tags": { "type": "set" },
    "age": { "type": "number" },
    "embedding": {
      "type": "vector",
      "dim": 768,
      "metric": "cosine",
      "backend": "hnsw-cpu"
    },
    "avatar_phash": { "type": "hash" }
  }
}
```

Adding a field is an online schema extension. Changing an existing field type
is rejected. Drop and recreate that field when its type must change.

### 2. Write values

Use the merge write when the caller owns only some indexed fields:

```http
POST /collections/users/index
Content-Type: application/json

{
  "items": [
    { "external_id": "u_123", "field": "bio", "value": "search engineer in Taipei" },
    { "external_id": "u_123", "field": "email", "value": "person@example.com" },
    { "external_id": "u_123", "field": "tags", "value": ["rust", "search"] }
  ]
}
```

Writing the same `(external_id, field)` replaces that field. Other fields stay
unchanged.

Use `PUT /collections/{id}/docs:replace` when the caller owns the complete
indexed row. Fields omitted from a replacement are deleted from that row.

### 3. Search

```http
POST /collections/users/search
Content-Type: application/json

{
  "query": {
    "and": [
      { "match": { "field": "bio", "text": "search engineer" } },
      { "term": { "field": "tags", "value": "rust" } }
    ]
  },
  "limit": 20
}
```

```json
{
  "hits": [
    { "external_id": "u_123", "score": 4.21 }
  ],
  "total": 1,
  "cursor": null,
  "took_ms": 0,
  "took_us": 740
}
```

Search responses contain `external_id` and `score`. They do not contain source
field values or highlighted text.

### 4. Load source records

Use one ID-list request to load full records from the caller's database or
object store. Then restore the order returned by Lumen. Lumen does not keep a
source document for hydration.

## Use Lumen

The examples above show the current 0.4 request shapes. Use
`lumen spec --fields` for the current field list and `lumen spec --shapes` for
current request examples. The [indexing guide](docs/indexing.md) owns schema,
write, durability, and rebuild semantics. The
[querying guide](docs/querying.md) owns selection, scoring, result controls,
facets, limits, and source-record hydration.

Lumen has two runtime modes. Standalone runs one process or container and is
the normal local path. Managed runs stateful workloads through the operator.
`LumenFleet` is the default Managed entry point. Direct `Lumen` is the advanced
entry point. Fleet, high availability, sharding, and autoscaling are separate
choices.

The first Managed production target is GKE Standard Regional. Current GKE
acceptance is zonal and does not prove regional high availability. A one-Pod
Managed runtime can be persistent, but it is not highly available. The target
production baseline uses three voters for each shard across three zones. See
the [GKE guide](docs/gke.md) for the support tiers and topology contract.

Use the [deployment guide](docs/deployment.md) for installation and the
[authentication guide](docs/authentication.md) for request identity. Use the
[client integration guide](docs/client-integration.md) for generated-client,
workload-template, retry, and source-hydration responsibilities. Use the
[0.5 search migration guide](docs/migration-0.5-search.md) before adopting any
target schema, query, result, facet, or activation contract described in the
new guides.

## Contract discovery

The binary and the running service publish the detailed contract.

| Need | Source of truth |
|---|---|
| HTTP routes and request types | `lumen spec` or `GET /openapi.json` |
| Current schema, writes, durability, and rebuild plus the 0.5 target | [Indexing guide](docs/indexing.md) |
| Current selection and hydration plus the 0.5 query, result, facet, and limit target | `lumen llm --topic querying`; see the [querying guide](docs/querying.md) |
| Current QUERY/POST, consistency, retry, and connection behavior | [Protocol guide](docs/protocol.md); focused topics: `local-search`, `select-query`, and `integrate-source-db` |
| Copy-ready query examples | `lumen spec --shapes` |
| Field, analyzer, and vector choices | `lumen spec --fields` |
| Interactive API use | `GET /docs` |
| CLI commands | `lumen --help` and command-specific `--help` |
| Kubernetes API | `lumen k8s crd render` |
| Local and GKE support tiers, topology, placement, and operations | [GKE guide](docs/gke.md) |
| Typed clients | `lumen spec gen --lang <language> --out <dir>`; languages: `ts`, `py`, `rust` |
| Generated-client language differences | [Generated-client guide](clients/README.md) |
| Client connection profiles, workload projection, retry, and source integration | [Client integration guide](docs/client-integration.md) |
| 0.4.x to 0.5.0 caller changes | [0.5 search migration](docs/migration-0.5-search.md) |
| Agent task catalog | `lumen llm --topic outline --format json` |
| Focused integration help | `lumen llm --topic <id>` |

## Capabilities

Every entry below is a Lumen product capability. The list has no primary and
secondary classes.

A capability can have several sources. `apps/lumen` supplies Lumen-specific
behavior and composition. `libs/<name>` supplies a reusable mechanism.
`external:<name>` supplies an outside runtime, authority, or provisioned
contract. Each source below states its direct contribution.

### Capability index

| Capability | ID | User promise | Sources |
|---|---|---|---|
| Indexing | `indexing` | Build and maintain rebuildable indexes over caller-owned values. | `apps/lumen`, `libs/storage-durable`, `libs/raft-core`, `libs/raft-runtime` |
| Querying | `querying` | Return ranked, filtered, grouped, sorted, and paginated IDs. | `apps/lumen` |
| Kubernetes-native deployment | `kubernetes-native-deployment` | Reconcile a Lumen instance into stable Kubernetes resources. | `apps/lumen`, `libs/service-k8s`, `external:kubernetes` |
| Managed Fleet materialization | `managed-fleet-materialization` | Declare Lumen instances across existing namespaces from one cluster-scoped resource. | `apps/lumen`, `libs/service-k8s`, `external:kubernetes` |
| Security and access | `security-hardening` | Protect serving requests and Raft peers with separate identity planes. | `apps/lumen`, `libs/service-auth`, `libs/service-k8s`, `libs/peer-tls`, `libs/cli-std`, `external:certificate-provider`, `external:kubernetes` |
| Scaling and availability | `scaling-availability` | Apply declared shard changes and run a fixed replica topology without changing search behavior. | `apps/lumen`, `libs/raft-core`, `libs/raft-runtime`, `libs/service-k8s`, `libs/storage-durable`, `external:kubernetes` |
| Durability and recovery | `durability-recovery` | Restore the derived index after process, member, or cluster loss. | `apps/lumen`, `libs/storage-durable`, `libs/service-backup`, `libs/raft-core`, `libs/raft-runtime` |
| Operations and observability | `operations-observability` | Expose health, readiness, metrics, traces, events, alerts, and control status. | `apps/lumen`, `libs/service-http`, `libs/metrics-prometheus`, `libs/service-observability`, `libs/service-k8s`, `external:kubernetes`, `external:prometheus-stack` |
| API, CLI, and agent integration | `api-cli-agent-integration` | Publish one discoverable contract for people, applications, and agents. | `apps/lumen`, `libs/service-http`, `libs/server-http`, `libs/transport-h2c`, `libs/openapi-codegen`, `libs/cli-std` |

### Indexing

- ID: `indexing`
- Promise: Validate schemas and apply deterministic index mutations that can be
  retained or rebuilt.
- Sources:
  - [`apps/lumen`](./) defines field behavior, analyzers, mutation rules,
    segment formats, and the derived-index lifecycle. The
    [indexing guide](docs/indexing.md) separates current behavior from the 0.5
    schema, write, durability, and rebuild target.
  - [`libs/storage-durable`](../../libs/storage-durable/README.md) provides
    durable files, atomic replacement, fsync, and framed logs.
  - [`libs/raft-core`](../../libs/raft-core/README.md) orders replicated writes.
  - [`libs/raft-runtime`](../../libs/raft-runtime/README.md) hosts replication,
    snapshots, and the replicated log lifecycle.
- Gate: `cargo test -p lumen --test api_e2e --test drop_field_e2e --test reindex_stream_e2e --test stats_metadata_e2e`
- Gate: `cargo test -p lumen --test perf_gate --test perf_gate_vs_db`

### Querying

- ID: `querying`
- Promise: Evaluate the published query nodes and return deterministic caller
  IDs, ranks, groups, sorts, and pages.
- Sources:
  - [`apps/lumen`](./) defines query validation, planning, scoring, filtering,
    grouping, sorting, pagination, and read-consistency behavior. The
    [querying guide](docs/querying.md) owns source hydration and the 0.5 query,
    result, facet, metric, and limit target.
- Gate: `cargo test -p lumen --test api_e2e --test coverage_gaps_e2e --test prefix_query`
- Gate: `cargo test -p lumen --test vector_e2e --test hash_hamming --test hybrid_rrf --test collapse_nested`

### Kubernetes-native deployment

- ID: `kubernetes-native-deployment`
- Promise: Turn a `Lumen` custom resource into stable workloads, Services,
  configuration, status, and disruption protection.
- Sources:
  - [`apps/lumen`](./) defines the CRD, defaults, topology policy, conditions,
    and Lumen resource composition.
  - [`libs/service-k8s`](../../libs/service-k8s/README.md) provides reusable
    reconciliation, leader election, workload, Service, and status mechanisms.
  - `external:kubernetes` stores desired state and runs the workload, network,
    lease, RBAC, and Secret contracts.
- Gate: `cargo test -p lumen --features operator --test operator_render --test operator_backup_kubernetes_wiring`
- Gate: `apps/lumen/scripts/kind-e2e.sh`
- Gate: `acceptance/gcp/scripts/run.sh`

### Managed Fleet materialization

- ID: `managed-fleet-materialization`
- Promise: Materialize one namespaced `Lumen` object for each accepted entry in
  a cluster-scoped `LumenFleet`.
- Sources:
  - [`apps/lumen`](./) defines the Fleet API, RFC 7386 merge behavior,
    Lumen-specific protected topology fields, adoption rule, prune policy, and
    current materialization status.
  - [`libs/service-k8s`](../../libs/service-k8s/README.md) provides reusable
    Kubernetes lease and controller mechanisms used by the operator. The
    shared `service-k8s::fleet` controller is a future outcome, not a current
    library surface.
  - `external:kubernetes` stores the cluster-scoped Fleet and namespaced child
    resources, enforces RBAC, and runs the operator.
- Gate: `cargo test -p lumen --features operator --lib`
- Gate: `cargo test -p lumen --features operator --test operator_render`

### Security and access

- ID: `security-hardening`
- Promise: Use private serving TLS and Kubernetes request identity on `:7373`,
  and separate mandatory peer mTLS on `:7374`.
- Sources:
  - [`apps/lumen`](./) defines the security posture, Lumen permission mapping,
    anonymous route set, identity separation, and integration policy.
  - [`libs/service-auth`](../../libs/service-auth/README.md) provides token
    extraction, projected-token reading, TokenRequest, TokenReview,
    SubjectAccessReview, principal parsing, redaction, and fail-closed
    middleware.
  - [`libs/service-k8s`](../../libs/service-k8s/README.md) provides projected
    token, ServiceAccount, Role, RoleBinding, and auth-delegator rendering
    mechanisms used by Lumen-owned policy.
  - [`libs/peer-tls`](../../libs/peer-tls/README.md) provides certificate
    validation, serving TLS, peer mTLS, trust handling, and certificate reload.
  - [`libs/cli-std`](../../libs/cli-std/README.md) provides kubeconfig,
    TokenRequest, local proxy, and private-CA client mechanisms.
  - `external:certificate-provider` supplies the current serving and peer
    certificates, private keys, and trust roots. The planned operator-managed
    leaf lifecycle is tracked separately in the [roadmap](ROADMAP.md).
  - `external:kubernetes` acts as the request identity and authorization
    authority and stores externally provisioned TLS Secrets.
- Gate: `cargo test -p lumen -p service-auth -p service-k8s -p peer-tls`
- Gate: `cargo test -p lumen --test auth_e2e --test authz_matrix_e2e --test serving_tls_rotation`
- Gate: `cargo test -p lumen --features operator --test operator_render`
- Gate: `acceptance/gcp/scripts/verify-lumen-auth.sh`

### Scaling and availability

- ID: `scaling-availability`
- Promise: Move shard ownership through the published reshard workflow and run
  the declared fixed replica topology without changing visible indexing or
  query behavior.
- Sources:
  - [`apps/lumen`](./) defines virtual-bucket routing, shard ownership,
    reshard phases, write fences, checkpoints, and scatter/gather behavior.
  - [`libs/raft-core`](../../libs/raft-core/README.md) provides consensus,
    quorum, ordered replication, and leader election.
  - [`libs/raft-runtime`](../../libs/raft-runtime/README.md) provides topology,
    transport, forwarding, snapshots, and log compaction.
  - [`libs/service-k8s`](../../libs/service-k8s/README.md) provides the workload
    and controller mechanisms that apply declared topology.
  - [`libs/storage-durable`](../../libs/storage-durable/README.md) provides
    durable state transitions at reshard and restart boundaries.
  - `external:kubernetes` runs the declared members and networking contracts.
- Gate: `cargo test -p lumen --test reshard_admin_e2e --test reshard_driver_e2e --test routed_shard_e2e`
- Gate: `cargo test -p lumen --test wal_nats_e2e --test stability_lumen_claim_dynamic_multi_shard_replica_kind`
- Gate: `apps/lumen/scripts/kind-e2e.sh`

### Durability and recovery

- ID: `durability-recovery`
- Promise: Recover the derived index after restart, replica replacement,
  backup restore, or cold seed.
- Sources:
  - [`apps/lumen`](./) defines WAL records, segment and snapshot formats,
    checkpoint boundaries, and restore behavior.
  - [`libs/storage-durable`](../../libs/storage-durable/README.md) provides
    durable-file, atomic-state, fsync, and framed-log mechanisms.
  - [`libs/service-backup`](../../libs/service-backup/README.md) provides backup
    destinations, transfer, retention, and object-store mechanisms.
  - [`libs/raft-core`](../../libs/raft-core/README.md) replicates committed
    writes inside a shard group.
  - [`libs/raft-runtime`](../../libs/raft-runtime/README.md) installs replica
    snapshots and manages the replicated log lifecycle.
- Gate: `cargo test -p lumen --test backup_restore_e2e --test wal_nats_e2e`
- Gate: `acceptance/gcp/scripts/run.sh`

### Operations and observability

- ID: `operations-observability`
- Promise: Show whether serving and control-plane work is healthy, ready,
  progressing, stalled, degraded, or failed.
- Sources:
  - [`apps/lumen`](./) defines Lumen metrics, readiness inputs, conditions,
    alerts, and the meaning of domain state.
  - [`libs/service-http`](../../libs/service-http/README.md) provides health,
    readiness, admission, request tracing, and graceful drain.
  - [`libs/metrics-prometheus`](../../libs/metrics-prometheus/README.md) provides
    metric types and Prometheus encoding.
  - [`libs/service-observability`](../../libs/service-observability/README.md)
    provides shared structured logging and tracing setup.
  - [`libs/service-k8s`](../../libs/service-k8s/README.md) provides controller
    events, status, monitoring resources, and reconciliation mechanisms.
  - `external:kubernetes` stores status and events and runs probe contracts.
  - `external:prometheus-stack` consumes metrics and applies ServiceMonitor and
    alert-rule resources when that monitoring stack is installed.
- Gate: `cargo test -p lumen --test api_e2e --test structured_stdout_traceparent`
- Gate: `cargo test -p lumen --features operator --test operator_backup_kubernetes_wiring`

### API, CLI, and agent integration

- ID: `api-cli-agent-integration`
- Promise: Let people, applications, generated clients, and agents discover
  and use the same Lumen contract.
- Sources:
  - [`apps/lumen`](./) defines routes, request and response types, CLI verbs,
    deployment commands, domain errors, and agent topics.
  - [`libs/service-http`](../../libs/service-http/README.md) provides standard
    routes, request policy, trace context, timing, and shared error envelopes.
  - [`libs/server-http`](../../libs/server-http/README.md) owns the shared HTTP
    listener, lifecycle drain, and accept-time TLS configuration.
  - [`libs/transport-h2c`](../../libs/transport-h2c/README.md) provides the
    per-connection HTTP/1.1 and h2c protocol and client transport helpers.
  - [`libs/openapi-codegen`](../../libs/openapi-codegen/README.md) generates
    typed clients from OpenAPI.
  - [`libs/cli-std`](../../libs/cli-std/README.md) provides shared CLI
    conventions, output, operational commands, and agent discovery.
- Gate: `cargo test -p lumen --test spec_cli --test spec_route_parity --test api_e2e`
- Gate: `cargo test -p lumen --test cli_convention --test spec_gen_e2e --test generated_clients_crud_e2e`
- Gate: `cargo test -p lumen --features operator --test operator_render`

## Supporting documents

| Document | Use it for |
|---|---|
| [STATUS.md](STATUS.md) | Current support boundaries and evidence |
| [ROADMAP.md](ROADMAP.md) | Future outcomes and explicit non-goals |
| [Architecture](ARCHITECTURE.md) | Runtime planes, ownership axes, and source responsibilities |
| [Deployment](docs/deployment.md) | Standalone and Managed installation paths |
| [Configuration](docs/configuration.md) | Setting precedence and change activation |
| [Authentication](docs/authentication.md) | Current KSA flow and planned Managed access contract |
| [Protocol](docs/protocol.md) | Canonical source map, connection paths, operation families, and current protocol boundaries |
| [Generated clients](clients/README.md) | Client generation, language matrix, connection inputs, and current limits |
| [Indexing](docs/indexing.md) | Current and target schema, write, durability, rebuild, and activation contracts |
| [Querying](docs/querying.md) | Source hydration plus current and target query, result, facet, metric, and limit contracts |
| [GKE](docs/gke.md) | Local and GKE support tiers, runtime topology, placement, security, and production verification |
| [Client integration](docs/client-integration.md) | Generated-client, workload-template, retry, and source-hydration responsibilities |
| [0.5 search migration](docs/migration-0.5-search.md) | Versioned compatibility rules, caller actions, offline tools, and Managed activation |
| [Scale and benchmark notes](docs/benchmarks-scale.md) | Capacity, performance gates, and benchmark procedure |
| [Operator control-plane runbook](docs/runbooks/operator-control-plane.md) | Reconciliation and incident operations |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Lumen edit rules and required verification commands |

Historical evidence and implementation planning do not define the current
product contract. Use the capability gates and published runtime contracts.
