---
id: '1764'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: shared-service-library-semantic-name-migration
entry: inventory
nodes:
  inventory: { kind: start, label: "inventory shared library responsibilities and all consumers" }
  classify: { kind: process, label: "classify each crate by stable responsibility family" }
  ambiguous: { kind: decision, label: "does the current path expose responsibility and abstraction level?" }
  retain: { kind: process, label: "retain already explicit library identity" }
  rename: { kind: process, label: "rename directory package and Rust crate identity atomically" }
  references: { kind: process, label: "rewrite Cargo source test script TD EC and active doc references" }
  docs: { kind: process, label: "project canonical inventory and naming grammar into README and CONTRIBUTING" }
  stale: { kind: decision, label: "do retired active identifiers remain?" }
  repair: { kind: process, label: "repair remaining active references" }
  verify: { kind: process, label: "run Cargo metadata focused crate tests and representative adopter tests" }
  done: { kind: terminal, label: "semantic library taxonomy is internally consistent" }
edges:
  - { from: inventory, to: classify }
  - { from: classify, to: ambiguous }
  - { from: ambiguous, to: retain, label: "no" }
  - { from: ambiguous, to: rename, label: "yes" }
  - { from: retain, to: references }
  - { from: rename, to: references }
  - { from: references, to: docs }
  - { from: docs, to: stale }
  - { from: stale, to: repair, label: "yes" }
  - { from: repair, to: stale }
  - { from: stale, to: verify, label: "no" }
  - { from: verify, to: done }
---
flowchart TD
  inventory([inventory responsibilities and consumers]) --> classify[classify by responsibility family]
  classify --> ambiguous{ambiguous path or abstraction?}
  ambiguous -->|no| retain[retain identity]
  ambiguous -->|yes| rename[atomic directory package crate rename]
  retain --> references[rewrite all active references]
  rename --> references
  references --> docs[update README and CONTRIBUTING]
  docs --> stale{retired active identifier remains?}
  stale -->|yes| repair[repair reference]
  repair --> stale
  stale -->|no| verify[metadata and focused tests]
  verify --> done([consistent semantic taxonomy])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: Project the responsibility-family inventory and links.
  - path: CONTRIBUTING.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: Define the library naming grammar and update the shared service kit contract.
  - path: Cargo.lock
    action: modify
    section: logic
    impl_mode: hand-written
    description: Refresh package identities after the workspace rename.
  - path: libs/service-k8s/Cargo.toml
    action: create
    section: logic
    impl_mode: hand-written
  - path: libs/raft-runtime/Cargo.toml
    action: create
    section: logic
    impl_mode: hand-written
  - path: libs/transport-h2c/Cargo.toml
    action: create
    section: logic
    impl_mode: hand-written
  - path: libs/server-lifecycle/Cargo.toml
    action: create
    section: logic
    impl_mode: hand-written
  - path: libs/server-tcp/Cargo.toml
    action: create
    section: logic
    impl_mode: hand-written
  - path: libs/server-http/Cargo.toml
    action: create
    section: logic
    impl_mode: hand-written
  - path: libs/storage-durable/Cargo.toml
    action: create
    section: logic
    impl_mode: hand-written
  - path: libs/metrics-prometheus/Cargo.toml
    action: create
    section: logic
    impl_mode: hand-written
  - path: libs/peer-tls/Cargo.toml
    action: create
    section: logic
    impl_mode: hand-written
  - path: libs/claim-token/Cargo.toml
    action: create
    section: logic
    impl_mode: hand-written
  - path: apps/courier/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
  - path: apps/keep/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
  - path: apps/loom/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
  - path: apps/lumen/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
  - path: apps/pgpool/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
  - path: apps/relay/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
  - path: apps/tape/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
  - path: examples/client-transport-policy/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
  - path: libs/service-backup/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
  - path: libs/service-http/Cargo.toml
    action: modify
    section: logic
    impl_mode: hand-written
  - path: apps/agentic-workflow/tests/fixtures/shared_service_library_names/assert_semantic_names.sh
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Reject retired paths, package identities, and Rust crate identifiers in active source and docs.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: shared-service-library-semantic-names-verification
requirements:
  all_active_references_migrate:
    id: R2
    text: "Cargo manifests, Rust source, tests, scripts, TDs, EC fixtures, and active documentation contain no retired library path or crate identity."
    kind: regression
    risk: high
    verify: bash apps/agentic-workflow/tests/fixtures/shared_service_library_names/assert_semantic_names.sh
  documentation_projects_taxonomy:
    id: R3
    text: "Root README inventories the new identities and CONTRIBUTING defines the stable responsibility prefixes and service-kit boundaries."
    kind: functional
    risk: medium
    verify: bash apps/agentic-workflow/tests/fixtures/shared_service_library_names/assert_semantic_names.sh
  semantic_identity_map:
    id: R1
    text: "Every ambiguous shared service library is renamed as one atomic directory, package, and Rust crate identity using the responsibility-family map."
    kind: regression
    risk: high
    verify: bash apps/agentic-workflow/tests/fixtures/shared_service_library_names/assert_semantic_names.sh
  workspace_and_adopters_preserve_behavior:
    id: R4
    text: "Cargo resolves the renamed workspace and focused shared-library plus representative service adopter tests pass without public behavior changes."
    kind: regression
    risk: high
    verify: cargo metadata --no-deps && cargo test -p service-k8s -p raft-runtime -p server-lifecycle -p server-tcp -p server-http -p transport-h2c -p storage-durable -p metrics-prometheus -p peer-tls -p claim-token
---
flowchart TD
    r1[R1 semantic identity map] --> bash_apps_agentic_workflow_tests_fixtures_shared_service_library_names_assert_semantic_names_sh[bash apps/agentic-workflow/tests/fixtures/shared_service_library_names/assert_semantic_names.sh]
    r2[R2 all active references migrate] --> bash_apps_agentic_workflow_tests_fixtures_shared_service_library_names_assert_semantic_names_sh
    r3[R3 documentation projects taxonomy] --> bash_apps_agentic_workflow_tests_fixtures_shared_service_library_names_assert_semantic_names_sh
    r4[R4 workspace and adopters preserve behavior] --> cargo_metadata_no_deps_cargo_test_p_service_k8s_p_raft_runtime_p_server_lifecycle_p_server_tcp_p_server_http_p_transport_h2c_p_storage_durable_p_metrics_prometheus_p_peer_tls_p_claim_token[cargo metadata --no-deps && cargo test -p service-k8s -p raft-runtime -p server-lifecycle -p server-tcp -p server-http -p transport-h2c -p storage-durable -p metrics-prometheus -p peer-tls -p claim-token]
```
