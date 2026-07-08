# Pgpool

## Brief

`pgpool` is the working app id for Axiom's Kubernetes-native PostgreSQL
connection pooler. The final product name is not settled yet; until then the
repository path, crate name, binary name, and tracker label use `pgpool` as a
stable implementation placeholder.

The app owns the database-pooling data plane: frontend PostgreSQL TCP
admission, backend connection budgeting, drain-aware shutdown, pool health, and
standard operational HTTP endpoints. Cloud SQL, AlloyDB, and other platform
connectors stay explicit adapters above this core rather than being baked into
the pooler runtime.

Current implementation slice: `apps/pgpool` is a Rust workspace crate and
binary with an offline runtime plan, OpenAPI-shaped admin route inventory,
agent docs, and shared server substrate dependencies wired through
`server-core`, `tcp-server`, and `http-server`. PostgreSQL wire handling,
auth/backend adapters, k8s operator artifacts, and real database integration
gates are separate work roots.

## Boundaries

- `pgpool` owns Postgres-compatible pooling and proxy admission.
- `server-core`, `tcp-server`, and `http-server` own generic server runtime
  mechanics; `pgpool` composes them instead of duplicating accept loops,
  connection budgets, h2c serving, drain, or tracing.
- Platform adapters such as Cloud SQL Proxy or AlloyDB endpoint discovery stay
  optional integration layers, not required runtime dependencies.
- Application services should connect to `pgpool` over the PostgreSQL wire
  protocol and inspect operations through the admin HTTP surface.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Working-Name App Scaffold | pending | implemented | verified | smoke | partial | crate/bin/README/AW metadata and route inventory are present under `apps/pgpool` |
| Shared Server Substrate Adoption | pending | implemented | verified | smoke | partial | runtime plan composes `server-core`, `tcp-server`, and `http-server` types |
| PostgreSQL Pooler Core | pending | planned | planned | none | not_ready | frontend pg wire parser, backend pool, transaction/session modes |
| Platform Adapter Boundary | pending | planned | planned | none | not_ready | Cloud SQL/AlloyDB discovery/auth adapters remain outside core runtime |
| Kubernetes-Native Deployment | pending | planned | planned | none | not_ready | CRD/operator/instance render and pod drain behavior |
| CLI Standard Surface | pending | implemented | verified | smoke | partial | `pgpool llm`, `pgpool upgrade`, and `pgpool issue` exist |

### Shared Server Substrate Adoption

ID: shared-server-substrate-adoption
Type: Runtime
Status: partial
Required Verification: smoke
Promise:
`pgpool` starts from the shared service substrate instead of inventing a local
accept loop or HTTP admin server. The TCP data-plane listener uses
`tcp-server` concepts, the admin listener uses `http-server`/h2c concepts, and
connection limits/readiness/drain are represented by `server-core`.
Gate Inventory:
- `cargo test -p pgpool`
- `cargo run -p pgpool -- spec --format routes`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| app-scaffold-and-runtime-plan | change | pending | implemented | passing | smoke | `apps/pgpool/src/lib.rs` |

### PostgreSQL Pooler Core

ID: postgres-pooler-core
Type: Runtime
Status: confirmed
Required Verification: conformance, performance, negative
Promise:
Provide a high-throughput PostgreSQL pooler with bounded frontend admission,
backend connection reuse, transaction/session pool modes, graceful drain, and
clear observability before platform-specific adapters are added.
Gate Inventory:
- pending: pg wire parser conformance
- pending: backend pool saturation and drain tests
- pending: Postgres/AlloyDB/Cloud SQL adapter integration gates
