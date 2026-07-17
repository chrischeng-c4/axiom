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
entry: provider
nodes:
  plain: { kind: process, label: "NoTls" }
  tls: { kind: process, label: "rustls system roots" }
  facts: { kind: terminal, label: "query connection facts" }
edges:
  - { from: provider, to: plain, label: plain_postgres }
  - { from: provider, to: tls, label: cloud_sql or alloy_db }
  - { from: plain, to: facts }
  - { from: tls, to: facts }
---
flowchart TD
  provider{"provider"} -->|plain| notls["NoTls"] --> facts["discover facts"]
  provider -->|managed| rustls["rustls roots"] --> facts
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
    anchor: discovers_runtime_limit_from_real_postgres_when_available
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: provider-driven-discovery-tls-verification
requirements:
  managed_providers_require_tls:
    id: R1
    text: "CloudSql and AlloyDb select the rustls discovery connector while PlainPostgres preserves NoTls, so provider fixtures do not take the impossible plaintext path."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test connection_discovery managed_provider_selects_tls_discovery
---
flowchart TD
    r1[R1 managed providers require tls] --> cargo_test_p_pgpool_test_connection_discovery_managed_provider_selects_tls_discovery[cargo test -p pgpool --test connection_discovery managed_provider_selects_tls_discovery]
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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: provider-driven-discovery-tls-verification
requirements:
  managed_providers_require_tls:
    id: R1
    text: "CloudSql and AlloyDb select the rustls discovery connector while PlainPostgres preserves NoTls, so provider fixtures do not take the impossible plaintext path."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test connection_discovery managed_provider_selects_tls_discovery
---
flowchart TD
    r1[R1 managed providers require tls] --> cargo_test_p_pgpool_test_connection_discovery_managed_provider_selects_tls_discovery[cargo test -p pgpool --test connection_discovery managed_provider_selects_tls_discovery]
```
