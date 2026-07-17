---
id: '1925'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-held-backend-query-ready-contract
entry: driver
nodes:
  driver: { kind: start, label: "Spawn the held backend connection driver." }
  ready_query: { kind: process, label: "Await SELECT 1 on the held Client." }
  poll: { kind: process, label: "Sample discovery until the held application_name is counted." }
  assert: { kind: terminal, label: "Assert pgpool count increases and non-pgpool usage remains derived correctly." }
edges:
  - { from: driver, to: ready_query }
  - { from: ready_query, to: poll }
  - { from: poll, to: assert }
---
flowchart LR
    driver([Driver running]) --> ready_query[SELECT 1 succeeds]
    ready_query --> poll[Discovery polls]
    poll --> assert([Classification invariant])
```

The query-ready step is deliberately before the polling loop. If startup/auth cannot complete, the existing integration convention skips rather than treating a missing local database as a classification failure. Once the query succeeds, the held client remains alive through the observation loop.
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
    reason: Drive the held pgpool backend to query-ready state before observing it through runtime discovery.
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-held-backend-query-ready-contract-verification
requirements:
  backend_visible_before_poll:
    id: R1
    text: "The held backend is query-ready before polling, so the regression deterministically proves pgpool-owned connections do not inflate foreign usage."
    kind: regression
    risk: medium
    verify: connection_discovery::pgpool_backend_connections_are_not_foreign_usage
---
flowchart TD
    r1[R1 backend visible before poll] --> connection_discovery_pgpool_backend_connections_are_not_foreign_usage[connection_discovery::pgpool_backend_connections_are_not_foreign_usage]
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
