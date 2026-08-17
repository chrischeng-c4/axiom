# Mesh

## Brief

Mesh is the relationship/property-graph service in the Axiom service stack.
It owns typed node/edge storage with properties, and traversal/path query
over that graph. Like `lumen`, mesh is log-driven and derived: writes fold
through a raft-replicated log into a separate, rebuildable local index. The
caller owns the system of record; mesh never becomes the durable owner of
relationship data.

It is intentionally separate from its siblings: `beam` owns vector ANN
search, `lumen` owns lexical/semantic/perceptual search and dedup, and
`cube` owns OLAP-style columnar aggregation. Mesh owns the graph shape:
nodes, typed edges, properties, and traversal.

## Contributing

Repo-wide authoring rules live in
[../../CONTRIBUTING.md](../../CONTRIBUTING.md).

## Capabilities

The capability headings and tables below were machine-readable input for the
`aw` capability verb, which was deleted with the binary. Nothing reads them
now, so they are a record of what was claimed rather than a contract a tool
still enforces.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Property Graph Core | #1969 | planned | planned | none | not_ready | typed node/edge storage with properties |
| Traversal & Path Query | #1969 | planned | planned | none | not_ready | neighbor/path/pattern traversal execution |
| Log-Driven Derived Index | #1969 | planned | planned | none | not_ready | raft log fold into a rebuildable local index; never system of record |
| HTTP/2 API List | #1969 | planned | planned | none | not_ready | h2c/OpenAPI endpoint inventory |
| Kubernetes-Native Deployment | #1969 | planned | planned | none | not_ready | dedicated StatefulSet/operator shape |
| Primary Replicas | #1969 | planned | planned | none | not_ready | raft-backed shard ownership |
| CLI Interface | #1969 | partial | partial | smoke | not_ready | `mesh` CLI for graph write/query/admin and agent docs — standard llm/upgrade/issue verbs plus placeholder domain verbs landed (#1970); real graph write/query verbs still pending |
| Long-Running Stability | #1969 | planned | planned | none | not_ready | write/traversal soak and recovery gates |
| Security Hardening | #1969 | planned | planned | none | not_ready | collection authz, tenant isolation, audit, and secret rotation |
| Competitor Feature Parity | #1969 | planned | planned | none | not_ready | Neo4j/JanusGraph/DGraph/Neptune-style feature matrix |
| Competitor Performance | #1969 | planned | planned | none | not_ready | pinned graph query baseline, rerun only on scope change |

### CLI Interface

ID: cli-interface
Type: RuntimeTool
Root WI: #1969
Status: confirmed
Surfaces: CLI: `mesh llm`, `mesh upgrade`, `mesh issue`, graph node/edge/query verbs, and admin/debug verbs.
EC Dimensions: behavior: pending CLI convention gate - required standard verbs, graph workflow ergonomics, and offline agent docs
Required Verification: smoke, conformance
Promise:
Mesh ships an agent-drivable CLI for node/edge writes, traversal/path query, and
admin workflows while following the repository-wide CLI convention.
Gate Inventory:
- passing: apps/mesh/tests/cli_contract.rs (`cargo test -p mesh`)

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| mesh-cli-convention-and-graph-verbs | epic | #1969 | planned | planned | none | pending CLI convention gate |
| mesh-cli-shell-scaffold | change | #1970 | implemented | verified | smoke | `cargo test -p mesh` (5/5 passing); apps/mesh/tests/cli_contract.rs |

### Long-Running Stability

ID: long-running-stability
Type: Runtime
Root WI: #1969
Status: confirmed
Surfaces: Runtime: node/edge writer, traversal executor, shard rebalancer, snapshot, and recovery paths.
EC Dimensions: stability: pending long-running graph gate - soak, restart, index-rebuild recovery, bounded memory, and backpressure behavior
Required Verification: conformance, dogfood
Promise:
Mesh remains stable under sustained write and traversal load without corrupting
the derived index or losing committed log entries.
Gate Inventory:
- pending: apps/mesh/tests/long_running_stability.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| graph-write-traversal-soak-and-recovery | epic | #1969 | planned | planned | none | pending long-running graph gate |

### Security Hardening

ID: security-hardening
Type: Devops
Root WI: #1969
Status: confirmed
Surfaces: HTTP/K8s: collection/query authn/authz, tenant/collection isolation, network policy, audit events, secret rotation, and request limits.
EC Dimensions: behavior: pending security gate - auth failure cases, collection isolation, audit emission, secret rotation, and abuse limits
Required Verification: negative, conformance
Promise:
Mesh protects graph collections and query APIs with explicit authorization,
auditability, network policy, and managed secret rotation.
Gate Inventory:
- pending: apps/mesh/tests/security_hardening.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| graph-security-boundary | epic | #1969 | planned | planned | none | pending security hardening gate |

### Competitor Feature Parity

ID: competitor-feature-parity
Type: RuntimeTool
Root WI: #1969
Status: confirmed
Surfaces: Docs/Test: graph feature matrix against Neo4j, JanusGraph, DGraph, and Neptune-style graph services.
EC Dimensions: behavior: pending competitor feature gate - node/edge model, property schema, traversal/path query, indexing, and retention
Required Verification: conformance
Promise:
Mesh keeps an explicit graph feature matrix against established graph systems,
with comparison scope changed only when product requirements change.
Gate Inventory:
- pending: apps/mesh/benchmark/competitor-feature-matrix.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| graph-competitor-feature-matrix | epic | #1969 | planned | planned | none | pending competitor feature gate |

### Competitor Performance

ID: competitor-performance
Type: RuntimeTool
Root WI: #1969
Status: confirmed
Surfaces: Meter/Vat: write throughput, traversal/path-query latency, index-rebuild cost, and shard rebalance efficiency.
EC Dimensions: efficiency: pending competitor performance gate - pinned external baseline and Mesh-owned graph measurements
Required Verification: dogfood
Promise:
Mesh maintains a pinned competitor performance baseline and reruns external
benchmarks only when the comparison scope changes or a human explicitly asks.
Gate Inventory:
- pending: apps/mesh/benchmark/competitor-performance-baseline.md
- pending: apps/mesh/meter-mesh-query.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| graph-competitor-performance-baseline | epic | #1969 | planned | planned | none | pending competitor performance gate |

### Property Graph Core

ID: property-graph-core
Type: RuntimeTool
Root WI: #1969
Status: confirmed
Surfaces: HTTP: `/v1/collections/{collection}/nodes`, `/v1/collections/{collection}/edges` - typed node/edge upsert and delete with properties.
EC Dimensions: behavior: pending graph write conformance gate - schema/type validation, edge endpoint integrity, and durable append
Required Verification: smoke, conformance
Promise:
Mesh stores typed nodes and edges with properties as a caller-owned relationship
graph, with explicit schema and durability semantics.
Gate Inventory:
- pending: apps/mesh/tests/property_graph_core.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| typed-node-edge-property-write-contract | epic | #1969 | planned | planned | none | pending graph write conformance gate |

### Traversal & Path Query

ID: traversal-and-path-query
Type: RuntimeTool
Root WI: #1969
Status: confirmed
Surfaces: HTTP: `/v1/collections/{collection}/traverse`, `/v1/collections/{collection}/path` - neighbor expansion, filtered traversal, and shortest/bounded path query.
EC Dimensions: behavior: pending traversal query gate - depth/filter bounds, pattern match, path result correctness, and deterministic ordering; efficiency: pending meter traversal gate - retained latency/resource floors
Required Verification: smoke, conformance
Promise:
Mesh answers neighbor, pattern, and path queries over the stored graph without
depending on Lumen search indexes or Cube analytical storage.
Gate Inventory:
- pending: apps/mesh/tests/traversal_and_path_query.rs
- pending: apps/mesh/meter-mesh-query.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| neighbor-pattern-path-query-contract | epic | #1969 | planned | planned | none | pending traversal query gate |

### Log-Driven Derived Index

ID: log-driven-derived-index
Type: Runtime
Root WI: #1969
Status: confirmed
Surfaces: Raft/Storage: durable append log via `libs/storage-durable`, folded into a separate rebuildable local graph index; the caller remains system of record.
EC Dimensions: behavior: pending derived-index gate - log replay determinism and full index rebuild from log; stability: pending rebuild-under-load gate
Required Verification: conformance, dogfood
Promise:
Mesh treats its graph index as derived and rebuildable from the replicated log
at all times; it never becomes the durable owner of relationship data.
Gate Inventory:
- pending: apps/mesh/tests/log_driven_derived_index.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| raft-log-fold-and-rebuildable-index | epic | #1969 | planned | planned | none | pending derived-index gate |

### HTTP/2 API List

ID: http2-api-list
Type: RuntimeTool
Root WI: #1969
Status: confirmed
Surfaces: HTTP: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs`, node/edge/query/admin routes.
EC Dimensions: behavior: pending h2c/OpenAPI route-list gate - probes, metrics, OpenAPI, and route inventory
Required Verification: smoke, conformance
Promise:
Mesh exposes a compact h2c/OpenAPI API list for graph write, traversal/path
query, and operator workflows.
Gate Inventory:
- pending: apps/mesh/tests/http_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| h2c-openapi-route-list | epic | #1969 | planned | planned | none | pending h2c/OpenAPI route-list gate |

### Kubernetes-Native Deployment

ID: kubernetes-native-deployment
Type: Devops
Root WI: #1969
Status: confirmed
Surfaces: K8s: dedicated StatefulSet/operator topology for shards, storage, probes, backups, and PDBs.
EC Dimensions: behavior: pending kustomize/operator render gate - CRD, operator, and instance render; stability: pending kind graph dogfood
Required Verification: smoke, dogfood
Promise:
Mesh runs as a dedicated k8s-native graph service with operator-managed
storage, backup policy, and shard lifecycle.
Gate Inventory:
- pending: apps/mesh/k8s

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| dedicated-graph-service-topology | epic | #1969 | planned | planned | none | pending k8s render/dogfood gates |

### Primary Replicas

ID: primary-replicas
Type: Runtime
Root WI: #1969
Status: confirmed
Surfaces: Raft: shard ownership, membership, and rebalance job state over `libs/raft-core` and `libs/raft-runtime`.
EC Dimensions: stability: pending raft graph failover gate - shard ownership and derived index survive failover
Required Verification: conformance, dogfood
Promise:
Mesh replicates shard ownership through raft so write and traversal control
state survives failover.
Gate Inventory:
- pending: apps/mesh/tests/raft_shard_ownership.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| raft-backed-graph-shard-ownership | epic | #1969 | planned | planned | none | pending raft shard failover gate |
