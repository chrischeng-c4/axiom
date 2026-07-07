# Cube

## Brief

Cube is the OLAP service in the Axiom service stack.

It owns columnar ingest, scan/filter/project, group-by aggregates, rollups,
partition pruning, and analytical query execution. It is intentionally separate
from `lumen`: Lumen is a low-latency search and dedup index; Cube is a
columnar analytical service for aggregations, dimensional queries, and retained
measurement/event facts.

## Boundaries

- `lumen` owns search, ranking, duplicate detection, and vector/text retrieval.
- `cube` owns columnar facts, analytical scans, grouping, rollups, and time
  partitions.
- `tape` may feed Cube from replayed topics, but Cube owns analytical storage.
- `meter` and `arena` can write facts into Cube, but measurement semantics stay
  with those tools.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Columnar Ingest | #767 | planned | planned | none | not_ready | append facts into partitioned columnar storage |
| Analytical Query API | #767 | planned | planned | none | not_ready | scan/filter/project/group-by/aggregate over facts |
| Rollups And Partitions | #767 | planned | planned | none | not_ready | time partition pruning and materialized rollups |
| HTTP/2 API List | #767 | planned | planned | none | not_ready | h2c/OpenAPI endpoint inventory |
| Kubernetes-Native Deployment | #767 | planned | planned | none | not_ready | dedicated StatefulSet/operator shape |
| Primary Replicas | #767 | planned | planned | none | not_ready | raft-backed metadata and partition ownership |
| CLI Interface | #767 | planned | planned | none | not_ready | `cube` CLI for ingest/query/admin and agent docs |
| Long-Running Stability | #767 | planned | planned | none | not_ready | ingest/query/rollup soak and recovery gates |
| Security Hardening | #767 | planned | planned | none | not_ready | table authz, tenant isolation, audit, and secret rotation |
| Competitor Feature Parity | #767 | planned | planned | none | not_ready | ClickHouse/DuckDB-style OLAP feature matrix |
| Competitor Performance | #767 | planned | planned | none | not_ready | pinned OLAP query baseline, rerun only on scope change |

### CLI Interface

ID: cli-interface
Type: RuntimeTool
Root WI: #767
Status: confirmed
Surfaces: CLI: `cube llm`, `cube upgrade`, `cube issue`, table ingest/query/rollup, and admin/debug verbs.
EC Dimensions: behavior: pending CLI convention gate - required standard verbs, analytical workflow ergonomics, and offline agent docs
Required Verification: smoke, conformance
Promise:
Cube ships an agent-drivable CLI for ingest, query, rollup, and admin workflows
while following the repository-wide CLI convention.
Gate Inventory:
- pending: projects/cube/tests/cli_contract.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| cube-cli-convention-and-query-verbs | epic | #767 | planned | planned | none | pending CLI convention gate |

### Long-Running Stability

ID: long-running-stability
Type: Runtime
Root WI: #767
Status: confirmed
Surfaces: Runtime: ingest writer, query executor, rollup refresher, partition compactor, snapshot, and recovery paths.
EC Dimensions: stability: pending long-running OLAP gate - soak, restart, rollup recovery, bounded memory, and backpressure behavior
Required Verification: conformance, dogfood
Promise:
Cube remains stable under sustained ingest, analytical query, and rollup load
without corrupting partitions or losing committed metadata.
Gate Inventory:
- pending: projects/cube/tests/long_running_stability.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| olap-ingest-query-soak-and-recovery | epic | #767 | planned | planned | none | pending long-running OLAP gate |

### Security Hardening

ID: security-hardening
Type: Devops
Root WI: #767
Status: confirmed
Surfaces: HTTP/K8s: table/query authn/authz, tenant/table isolation, network policy, audit events, secret rotation, and request limits.
EC Dimensions: behavior: pending security gate - auth failure cases, table isolation, audit emission, secret rotation, and abuse limits
Required Verification: negative, conformance
Promise:
Cube protects analytical tables and query APIs with explicit authorization,
auditability, network policy, and managed secret rotation.
Gate Inventory:
- pending: projects/cube/tests/security_hardening.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| olap-security-boundary | epic | #767 | planned | planned | none | pending security hardening gate |

### Competitor Feature Parity

ID: competitor-feature-parity
Type: RuntimeTool
Root WI: #767
Status: confirmed
Surfaces: Docs/Test: OLAP feature matrix against ClickHouse, DuckDB, and cloud warehouse-style analytical services.
EC Dimensions: behavior: pending competitor feature gate - ingest, schema, scan/filter/project, aggregates, rollups, partition pruning, and retention
Required Verification: conformance
Promise:
Cube keeps an explicit OLAP feature matrix against established analytical
systems, with comparison scope changed only when product requirements change.
Gate Inventory:
- pending: projects/cube/benchmark/competitor-feature-matrix.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| olap-competitor-feature-matrix | epic | #767 | planned | planned | none | pending competitor feature gate |

### Competitor Performance

ID: competitor-performance
Type: RuntimeTool
Root WI: #767
Status: confirmed
Surfaces: Meter/Vat: ingest throughput, scan/filter/group-by latency, rollup refresh cost, and partition pruning efficiency.
EC Dimensions: efficiency: pending competitor performance gate - pinned external baseline and Cube-owned OLAP measurements
Required Verification: dogfood
Promise:
Cube maintains a pinned competitor performance baseline and reruns external
benchmarks only when the comparison scope changes or a human explicitly asks.
Gate Inventory:
- pending: projects/cube/benchmark/competitor-performance-baseline.md
- pending: projects/cube/meter-cube-query.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| olap-competitor-performance-baseline | epic | #767 | planned | planned | none | pending competitor performance gate |

### Columnar Ingest

ID: columnar-ingest
Type: RuntimeTool
Root WI: #767
Status: confirmed
Surfaces: HTTP: `/v1/tables/{table}/ingest` - append fact batches into partitioned columnar storage.
EC Dimensions: behavior: pending ingest conformance gate - schema evolution, partition routing, and durable append
Required Verification: smoke, conformance
Promise:
Cube accepts fact batches into columnar storage with explicit schema, partition,
and durability semantics.
Gate Inventory:
- pending: projects/cube/tests/columnar_ingest.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| partitioned-columnar-ingest-contract | epic | #767 | planned | planned | none | pending ingest conformance gate |

### Analytical Query API

ID: analytical-query-api
Type: RuntimeTool
Root WI: #767
Status: confirmed
Surfaces: HTTP: `/v1/query` - scan, filter, project, group-by, aggregate, sort, and page analytical results.
EC Dimensions: behavior: pending analytical query gate - filters, group-by, aggregates, pagination, and deterministic result ordering; efficiency: pending meter query gate - retained latency/resource floors
Required Verification: smoke, conformance
Promise:
Cube answers OLAP-style queries over retained fact tables without depending on
Lumen search indexes.
Gate Inventory:
- pending: projects/cube/tests/query_api.rs
- pending: projects/cube/meter-cube-query.toml

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| scan-filter-group-aggregate-contract | epic | #767 | planned | planned | none | pending analytical query gate |

### Rollups And Partitions

ID: rollups-and-partitions
Type: RuntimeTool
Root WI: #767
Status: confirmed
Surfaces: HTTP/Admin: rollup definitions, refresh jobs, retention windows, and partition pruning.
EC Dimensions: behavior: pending rollup gate - rollup freshness, invalidation, partition pruning, and retention behavior
Required Verification: smoke, conformance
Promise:
Cube manages time partitions and materialized rollups so analytical queries can
stay bounded as data grows.
Gate Inventory:
- pending: projects/cube/tests/rollups_partitions.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| rollup-refresh-and-partition-pruning | epic | #767 | planned | planned | none | pending rollup/partition gate |

### HTTP/2 API List

ID: http2-api-list
Type: RuntimeTool
Root WI: #767
Status: confirmed
Surfaces: HTTP: `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs`, table ingest/query/admin routes.
EC Dimensions: behavior: pending h2c/OpenAPI route-list gate - probes, metrics, OpenAPI, and route inventory
Required Verification: smoke, conformance
Promise:
Cube exposes a compact h2c/OpenAPI API list for table ingest, analytical query,
rollup, and operator workflows.
Gate Inventory:
- pending: projects/cube/tests/http_api.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| h2c-openapi-route-list | epic | #767 | planned | planned | none | pending h2c/OpenAPI route-list gate |

### Kubernetes-Native Deployment

ID: kubernetes-native-deployment
Type: Devops
Root WI: #767
Status: confirmed
Surfaces: K8s: dedicated StatefulSet/operator topology for partitions, storage, probes, backups, and PDBs.
EC Dimensions: behavior: pending kustomize/operator render gate - CRD, operator, and instance render; stability: pending kind analytical dogfood
Required Verification: smoke, dogfood
Promise:
Cube runs as a dedicated k8s-native OLAP service with operator-managed storage,
backup policy, and partition/rollup lifecycle.
Gate Inventory:
- pending: projects/cube/k8s

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| dedicated-olap-service-topology | epic | #767 | planned | planned | none | pending k8s render/dogfood gates |

### Primary Replicas

ID: primary-replicas
Type: Runtime
Root WI: #767
Status: confirmed
Surfaces: Raft: metadata, partition ownership, and rollup job state over `libs/raft-core` and `libs/raft-host`.
EC Dimensions: stability: pending raft OLAP failover gate - metadata and partition ownership survive failover
Required Verification: conformance, dogfood
Promise:
Cube replicates metadata and partition/rollup ownership through raft so query and
ingest control state survives failover.
Gate Inventory:
- pending: projects/cube/tests/raft_metadata.rs

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| raft-backed-olap-metadata | epic | #767 | planned | planned | none | pending raft metadata failover gate |
