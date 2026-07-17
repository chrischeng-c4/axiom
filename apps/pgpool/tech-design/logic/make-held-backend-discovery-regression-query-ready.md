---
id: '1925'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-held-backend-query-ready
entry: connect
nodes:
  connect: { kind: start, label: "Open pgpool-named backend and spawn its connection driver." }
  query: { kind: process, label: "Execute SELECT 1 to prove the backend is query-ready." }
  poll: { kind: process, label: "Poll runtime discovery until pgpool connection count includes the held backend." }
  classify: { kind: terminal, label: "Assert owned backend is excluded from foreign usage." }
edges:
  - { from: connect, to: query }
  - { from: query, to: poll }
  - { from: poll, to: classify }
---
flowchart LR
    connect([Connect held pgpool backend]) --> query[Run SELECT 1]
    query --> poll[Poll discovery]
    poll --> classify([Owned backend is not foreign usage])
```

The test driver must run before the readiness query. A successful query creates an observable client-backend session while retaining the `Client` for the polling interval; discovery then measures a real held connection rather than an asynchronous connection attempt.

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: apps/pgpool/tests/connection_discovery.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: pgpool_backend_connections_are_not_foreign_usage
    reason: Make the held backend query-ready before verifying pgpool-owned connection accounting.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-held-backend-query-ready-verification
requirements:
  query_ready_connection:
    id: R1
    text: "The held pgpool backend executes a query after its driver starts before discovery polling asserts its client-backend classification."
    kind: regression
    risk: medium
    verify: connection_discovery::pgpool_backend_connections_are_not_foreign_usage
---
flowchart TD
    r1[R1 query ready connection] --> connection_discovery_pgpool_backend_connections_are_not_foreign_usage[connection_discovery::pgpool_backend_connections_are_not_foreign_usage]
```
