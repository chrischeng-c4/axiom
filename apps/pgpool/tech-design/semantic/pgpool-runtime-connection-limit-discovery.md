---
id: '1570'
summary: Discover authoritative remote PostgreSQL connection limits and usage while retaining Cloud SQL and AlloyDB endpoint identity.
fill_sections: [logic, unit-test]
capability_refs:
  - id: platform-adapter-boundary
    role: primary
    gap: runtime-connection-limit-discovery
    claim: runtime-connection-limit-discovery
    coverage: full
    rationale: "Keeps provider identity and advisory metadata above the core while deriving capacity from the live PostgreSQL endpoint."
---

## Logic
<!-- type: logic lang: mermaid -->

### Runtime accounting invariants

Discovery counts only `pg_stat_activity` rows whose `backend_type` is
`client backend`. A data-plane pgpool process replaces every client-provided
startup `application_name` with `pgpool-<pod>`, so its held remote sessions
are attributed to pgpool rather than foreign usage. The effective capacity is
the lowest runtime/configured/advisory ceiling minus
`superuser_reserved_connections`; PostgreSQL background workers are neither
allocatable capacity nor client demand.

```mermaid
---
id: pgpool-runtime-connection-limit-discovery
entry: endpoint
nodes:
  endpoint: { kind: start, label: "Provider-typed remote PostgreSQL endpoint" }
  connect: { kind: process, label: "Connect using caller-supplied tokio-postgres Config" }
  runtime: { kind: process, label: "Read pg_settings max_connections and pg_stat_activity usage" }
  effective: { kind: process, label: "Cap runtime maximum by optional configured and advisory ceilings" }
  facts: { kind: terminal, label: "Return provider role runtime effective and non-pgpool facts" }
edges:
  - { from: endpoint, to: connect }
  - { from: connect, to: runtime }
  - { from: runtime, to: effective }
  - { from: effective, to: facts }
---
flowchart TD
  endpoint([provider-typed endpoint]) --> connect[caller supplied PostgreSQL config]
  connect --> runtime[pg_settings and pg_stat_activity]
  runtime --> effective[min runtime configured advisory]
  effective --> facts([effective capacity and usage facts])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-runtime-connection-limit-discovery-tests
requirements:
  effective_limit:
    id: R1
    text: "Configured and advisory ceilings may reduce but never raise the authoritative runtime maximum."
    kind: functional
    risk: high
    verify: cargo test -p pgpool platform::discovery
  provider_identity:
    id: R2
    text: "Plain PostgreSQL, Cloud SQL, and independent AlloyDB primary/read-pool endpoints retain provider and role identity."
    kind: functional
    risk: medium
    verify: cargo test -p pgpool platform::discovery
  usage_accounting:
    id: R3
    text: "Non-pgpool usage is saturating total minus pgpool-attributed sessions."
    kind: negative
    risk: high
    verify: cargo test -p pgpool platform::discovery
  live_query:
    id: R4
    text: "A real PostgreSQL endpoint returns max_connections and activity usage when locally available."
    kind: integration
    risk: high
    verify: cargo test -p pgpool --test connection_discovery
---
flowchart TD
  r1[R1 effective cap] --> unit[cargo test -p pgpool platform::discovery]
  r2[R2 provider identity] --> unit
  r3[R3 usage accounting] --> unit
  r4[R4 live query] --> integration[cargo test -p pgpool --test connection_discovery]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/Cargo.toml
    action: modify
    impl_mode: hand-written
    section: logic
    description: Promote tokio-postgres to a runtime dependency for live endpoint discovery.
  - path: apps/pgpool/src/lib.rs
    action: modify
    impl_mode: hand-written
    section: logic
    description: Export the provider adapter boundary.
  - path: apps/pgpool/src/platform/mod.rs
    action: create
    impl_mode: hand-written
    section: logic
    description: Publish remote endpoint discovery models.
  - path: apps/pgpool/src/platform/discovery.rs
    action: create
    impl_mode: hand-written
    section: logic
    description: Query runtime connection facts and apply advisory caps.
  - path: apps/pgpool/tests/connection_discovery.rs
    action: create
    impl_mode: hand-written
    section: unit-test
    description: Exercise the real PostgreSQL discovery path when available.
```
