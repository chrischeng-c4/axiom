---
id: '1886'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: provider-driven-discovery-tls
entry: remote_endpoint_provider
nodes:
  plain: { kind: process, label: "PlainPostgres selects NoTls" }
  managed: { kind: process, label: "CloudSql or AlloyDb selects rustls system-root connector" }
  connect: { kind: decision, label: "connect and query runtime PostgreSQL facts" }
  blocked: { kind: terminal, label: "publish discovery error and hold safe target" }
  facts: { kind: terminal, label: "publish connection facts" }
edges:
  - { from: remote_endpoint_provider, to: plain, label: plain_postgres }
  - { from: remote_endpoint_provider, to: managed, label: cloud_sql or alloy_db }
  - { from: plain, to: connect }
  - { from: managed, to: connect }
  - { from: connect, to: blocked, label: connect or query failure }
  - { from: connect, to: facts, label: success }
---
flowchart TD
  provider{"provider"} -->|plain_postgres| notls["NoTls"]
  provider -->|cloud_sql / alloy_db| rustls["rustls system roots"]
  notls --> query["discover runtime limits"]
  rustls --> query
  query -->|ok| facts["connection facts"]
  query -->|error| blocked["safe Blocked status"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
  - path: apps/pgpool/src/platform/discovery.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: discover_connection_facts
  - path: apps/pgpool/tests/connection_discovery.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: provider_advisory_is_projected_into_discovery_context
```
