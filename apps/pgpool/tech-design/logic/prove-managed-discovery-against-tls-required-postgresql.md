---
id: '1924'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-managed-discovery-tls-required-proof-contract
entry: proof_script
nodes:
  proof_script: { kind: start, label: "Docker proof script creates certificate, key, and hostssl-only HBA." }
  ready: { kind: process, label: "Wait for the final PostgreSQL process to report readiness." }
  reject: { kind: decision, label: "Does sslmode=disable fail?" }
  run_test: { kind: process, label: "Pass mapped port and copied CA path to the targeted discovery test." }
  cleanup: { kind: terminal, label: "Trap removes container and temporary certificate material." }
edges:
  - { from: proof_script, to: ready }
  - { from: ready, to: reject }
  - { from: reject, to: run_test, label: rejected }
  - { from: run_test, to: cleanup }
---
flowchart TD
    proof_script([Start hostssl-only postgres]) --> ready[Wait ready]
    ready --> reject{Plaintext rejected?}
    reject -->|yes| run_test[Run CloudSql TLS discovery]
    reject -->|no| cleanup([Fail proof and clean up])
    run_test --> cleanup
```

The server certificate is self-signed for `localhost`, then copied out as the exact configured CA used by `RemoteEndpoint::tls_ca_pem`. The test endpoint uses `EndpointProvider::CloudSql`, ensuring the path selects `MakeRustlsConnect`; a successful runtime facts query proves TLS negotiation, certificate trust, PostgreSQL authentication, and query decoding together.
## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: apps/pgpool/tests/connection_discovery.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: cloudsql_discovery_succeeds_against_tls_required_postgres
    reason: Exercise configured-CA Rustls discovery against the endpoint supplied by the TLS-required container proof.
  - path: apps/pgpool/tests/tls_required_discovery.sh
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: Start and clean up a disposable hostssl-only PostgreSQL 15 container and run the targeted Rust test.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-managed-discovery-tls-required-proof-verification
requirements:
  cloudsql_tls_discovery:
    id: R2
    text: "CloudSql discovery trusts the fixture CA and returns runtime connection facts after a real Rustls PostgreSQL handshake."
    kind: integration
    risk: high
    verify: connection_discovery::cloudsql_discovery_succeeds_against_tls_required_postgres
  plaintext_rejected:
    id: R1
    text: "The disposable PostgreSQL fixture accepts only hostssl connections and its proof script fails if an explicit plaintext client is accepted."
    kind: integration
    risk: high
    verify: apps/pgpool/tests/tls_required_discovery.sh
---
flowchart TD
    r1[R1 plaintext rejected] --> apps_pgpool_tests_tls_required_discovery_sh[apps/pgpool/tests/tls_required_discovery.sh]
    r2[R2 cloudsql tls discovery] --> connection_discovery_cloudsql_discovery_succeeds_against_tls_required_postgres[connection_discovery::cloudsql_discovery_succeeds_against_tls_required_postgres]
```
