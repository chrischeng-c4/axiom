# Lumen architecture

## Purpose

This document explains how Lumen fits between a caller and its source data. It
also separates runtime traffic from management work and separates three
deployment choices that are easy to confuse.

Use the [README](README.md) for the product workflow. Use
[STATUS.md](STATUS.md) for current support. Use
[ROADMAP.md](ROADMAP.md) for future outcomes.

## Source-data flow

PostgreSQL, another database, or an object store is the source of truth. Lumen
stores a derived search index. It does not own or return source records.

```text
source database
  -> caller-owned CDC, outbox, or ingest
  -> Lumen derived index
  -> ordered external IDs and search metadata
  -> source database lookup by ID or ID list
  -> caller restores Lumen order and returns hydrated records
```

The caller owns source writes, CDC or outbox checkpoints, freshness, source
authorization, and hydration. Lumen owns index mutation, business filter,
scoring, sort, limit, cursor, facets, rebuild, backup, and restore. The source
database loads the IDs selected by Lumen in one bulk request. It does not repeat
the business `WHERE`, `ORDER BY`, or `LIMIT` work.

The generated client will own the mechanics of one HTTP request. This includes
typed errors, safe retry, idempotency input, and ordered-result helpers. The
caller still owns its source transaction and delivery policy. Lumen accepts raw
vectors and perceptual hashes. It does not execute an embedding model.

The [indexing guide](docs/indexing.md) owns schema, write, durability, and
rebuild semantics. The [querying guide](docs/querying.md) owns selection,
scoring, results, facets, limits, and hydration. The
[0.5 migration guide](docs/migration-0.5-search.md) owns the version boundary.

## Runtime planes

### Data plane

The data plane accepts index and search traffic.

In Standalone mode, one `lumen serve` process is the data plane. In Managed
mode, the operator renders one StatefulSet per `Lumen` resource. Its pods form
the data plane.

Client traffic uses port `7373`. Replicated Raft peers use a separate identity
and transport plane on port `7374`.

Standalone auth is off by default. A Managed runtime defaults to delegated
Kubernetes auth. The current runtime uses TokenReview for the caller identity
and SubjectAccessReview for per-collection or instance-admin permission.

### Control plane

The control plane declares and reconciles runtime state.

- A namespaced `Lumen` resource declares one Managed instance.
- A cluster-scoped `LumenFleet` declares a set of namespaced `Lumen` resources.
- The operator materializes Fleet children and reconciles each child into
  Kubernetes resources.
- Kubernetes stores desired state, schedules pods, mounts Secrets, and reports
  workload status.

Fleet status currently reports child materialization only. The child `Lumen`
status is the current source for runtime readiness.

## Three independent axes

### Lifecycle owner

| Choice | Lifecycle owner | Intended use |
|---|---|---|
| Standalone | The caller starts, configures, restarts, and stops one process or container. | Local work, tests, and small single-process installations. |
| Managed | The operator reconciles the declared runtime inside an existing Kubernetes cluster. | Stateful Kubernetes operation. GKE Standard Regional is the first production target. |

### Management scope

| Choice | Scope | Entry point |
|---|---|---|
| Fleet | A platform declaration manages a set of instances across existing namespaces. | Cluster-scoped `LumenFleet`; this is the Managed default. |
| Direct | A deployer manages one instance explicitly. | Namespaced `Lumen`; this is the advanced Managed entry point. |

Management scope does not imply high availability. A Fleet entry can still
declare one shard with one replica.

### Data topology

| Control | Meaning |
|---|---|
| `shardCount` | Number of physical search shards. |
| `replicasPerShard` | Number of members in each shard group. Values above one enable replicated Raft topology. |
| `voterCount` | Voting members in each replicated shard group. |
| `reshardPolicy` | Lumen-owned policy and workflow for shard expansion. |

Topology does not select Fleet or Direct management. Generic HPA is not a
topology controller. Membership-aware autoscaling remains future work.

One shard with one replica can use persistent storage, but it is not highly
available. More shards with one replica add capacity without replica failure
tolerance. The production HA target uses three voters for each shard across
three zones. Two replicas remain a compatibility shape, not a production HA
baseline. The [GKE guide](docs/gke.md) owns these support tiers and placement
rules.

## Source responsibilities

Every source contributes one part of the Managed result.

| Source | Current responsibility | Planned boundary |
|---|---|---|
| `apps/lumen` | Derived-index schemas, writes, queries, ordered-ID results, route meaning, OpenAPI composition, shard policy, `Lumen` and `LumenFleet` APIs, current Fleet materialization, current permission mapping, Lumen rendering, backup and restore operations, and Lumen health meaning. | Keep the Lumen protocol policy, search contract, Fleet adapter, typed access API, whole-runtime permission meaning, anonymous route policy, Kubernetes-native placement intent, protected topology fields, child projection, and health mapping. It does not take source-record, CDC, outbox, embedding-model, or hydration ownership. |
| `libs/service-auth` | Bearer middleware, projected-token reading, TokenRequest, TokenReview, SubjectAccessReview, ServiceAccount principal parsing, redaction, and fail-closed decisions. | Keep the service-neutral identity mechanisms and projected-token behavior. Lumen supplies audience, path, and resource mapping. |
| `libs/service-http` | Standard probe, metrics, OpenAPI, and docs routes; request limits; admission; trace context; timing; and the shared error envelope. | Project shared middleware responses into service OpenAPI without taking ownership of domain errors or routes. |
| `libs/server-http` | HTTP listener ownership, lifecycle drain and reporting, HTTP/1.1 plus h2c composition, TLS serving, and accept-time TLS config reload. | Keep listener and TLS-runtime mechanics separate from route and certificate-material policy. |
| `libs/transport-h2c` | h2c client helpers, managed connections, pool sizing, optional per-connection HTTP/1.1 plus h2c serving, and graceful protocol drain. | Keep transport mechanics separate from listener ownership, TLS policy, and Lumen routes. |
| `libs/openapi-codegen` | Generates TypeScript, Python, and Rust source from the declared OpenAPI subset. TypeScript and Python have static header or token inputs. Rust has no default Authorization input. | Add non-JSON and streaming operations, typed errors, cross-language schema parity, complete output provenance and dependencies, operation-aware retry hooks, and a service-neutral request-header provider. It does not own Lumen protocol, KSA, retry policy, or Fleet policy. |
| `libs/service-k8s` | Generic controller, leader election, server-side apply, workload rendering, projected-token and RBAC object shapes, status conditions, lifecycle helpers, stateful planning, and PVC resize primitives. | Add reusable Fleet mechanics, access and public-CA ConfigMap ownership, certificate lifecycle wiring, scheduling primitives, and controlled StatefulSet rollout mechanics. Lumen supplies shard groups, quorum rules, certificate identities, and readiness meaning. |
| `external:kubernetes` | API storage, ServiceAccount identity, TokenRequest, token rotation, TokenReview, SubjectAccessReview, RBAC, scheduling, Services, StatefulSets, PVCs, Secrets, probes, events, and leases. | Continue to provide the cluster runtime and final auth decision. Lumen does not create the cluster, target namespaces, or client ServiceAccounts. |
| `external:certificate-provider` | Serving and peer certificates, keys, and trust roots are currently provisioned before runtime reconciliation. | The platform supplies the issuer and CA policy. The Lumen operator will request and rotate separate serving and peer leaf certificates and publish public client trust. |
| GCP Terraform capacity module | Current GCE node pools and `lumen-system/lumen-capacity-catalog`. | Remain a legacy compatibility substrate. The Kubernetes-native contract uses resources, StorageClass, selectors, tolerations, and topology intent. A GKE profile can select a platform-owned ComputeClass. |

The planned shared Fleet library is not implemented yet. Current Fleet code
still lives in `apps/lumen`.

The [protocol guide](docs/protocol.md) is an index over these sources. It is not
a new protocol implementation or a second copy of the route contract. The
[generated-client guide](clients/README.md) describes how the current OpenAPI
projection appears in each target language. The
[client integration guide](docs/client-integration.md) owns connection
profiles, workload projection, request mechanics, and source integration. The
[GKE guide](docs/gke.md) owns the first production environment profile.

## Authentication boundary

Managed operation keeps the operator ServiceAccount, runtime ServiceAccount,
and client ServiceAccount separate. The operator reconciles objects. The
runtime identity calls the Kubernetes review APIs. The client identity is the
subject of the serving request.

Binding a client pod to a ServiceAccount does not add a bearer credential to
an HTTP request. The workload must receive a projected token for audience
`lumen.axiom.dev`. The client must read that token and set the Authorization
header. The current generated clients do not perform that projection or
request-time read.

The planned access flow starts from `Lumen.spec.access` or Fleet typed access.
The operator then owns namespaced access Roles and RoleBindings. The runtime
asks one `lumen.axiom.dev/lumenruntimes` check with the child name and verb
`use`. This is a whole-runtime permission. It replaces current per-collection
and instance-admin permissions only after the migration outcome lands. It
allows query, index, collection-management, and admin requests. Therefore, one
runtime is one complete trust boundary. The access list is not a fine-grained
least-privilege model.

Kubernetes RBAC is authoritative. Fleet can converge the grants that it owns.
It cannot prevent a cluster administrator from creating another valid grant.
See the [authentication guide](docs/authentication.md) for the complete current
and planned contracts.

## Managed reconciliation

The current Managed path has two stages.

1. The Fleet loop merges `defaults` with each `instances[].spec` patch.
2. It creates or applies one child `Lumen` resource in the target namespace.
3. The generic service controller observes the child `Lumen` resource.
4. Lumen builds a service-specific reconcile plan.
5. `service-k8s` applies child Kubernetes objects and reads workload readiness.
6. Lumen projects domain conditions into the child status.

The Fleet loop runs a 30-second full pass. It does not watch child readiness.
One Fleet entry can fail without invalidating the pure merge plan for later
entries. Some Kubernetes API failures can still abort the current status pass.

The planned Fleet rollout orders changes across child runtimes. It cannot make
a member update quorum-safe inside one runtime. The planned runtime rollout
updates one member in a Raft group at a time. It waits for generation,
searchable readiness, quorum, and replication recovery before it continues.

## Ownership and deletion

A Fleet child carries the `lumen.dev/fleet` label. Fleet does not use a
cross-namespace owner reference.

`prunePolicy: Retain` leaves a removed child running. `prunePolicy: Delete`
deletes the child `Lumen` object. It does not currently guarantee PVC deletion.
Kubernetes and the StorageClass retain control of PVC and PV lifecycle.

The reshard driver owns `shardCount`, `shardMap`, and
`reshardPolicy.workflow` after initial materialization. Fleet omits these paths
from steady-state apply so it does not revert a completed topology transition.

## Readiness meaning

`GET /readyz` answers whether a process can serve searches and is not draining.
The child `Lumen` `Ready` condition combines pod readiness with selected auth,
peer identity, and reshard blockers.

A storage-full runtime can continue to search while it refuses writes. The
current API does not expose complete `Writable`, `ReplicationReady`,
`ConfigReady`, `CapacityReady`, and `Degraded` dimensions. The
[roadmap](ROADMAP.md#fleet-production-convergence) records that separation.
