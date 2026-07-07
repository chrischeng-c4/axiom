---
id: relay-declare-service-umbrella-sync-docs
summary: >
  Sync relay's declaration layer to the converged single-bin service surface
  delivered by WIs #1204-#1209. aw.toml [capability.profile].traits switches
  to the `service` umbrella (standard_endpoints + ec_gated + cli_std +
  chainable_output + http2_api + kubernetes_native) plus the honest extras
  relay satisfies: long_running, cli_facing, competitive_replacement,
  network_exposed, primary_replicas. README.md capability contract drops
  every reference to the deleted relay-server/relay-raft bins and
  src/bin/relay_server.rs / src/bin/relay_raft.rs evidence paths in favor of
  the single `relay` bin (serve default with raft-host auto-mode HA), adds
  field-style capability sections for the four umbrella-derived baselines
  (standard-operational-endpoints WI #1205, cli-standard-surface WI #1204,
  chainable-output-conformance with honest partial maturity,
  ec-gates-configured), and updates security-hardening for the shipped
  RELAY_AUTH bearer contract (#1206) and peer-TLS config surface (#1209).
  A new HA.md (archetype Deploy row requirement) documents auto-mode
  (REPLICAS_PER_SHARD > 1 flips raft), RelayStateMachine (publish
  replication, snapshot/compaction, fsynced applied-index marker), the
  node-local lease/ack at-least-once failover limitation, RELAY_PEERS,
  the operator CR as the production HA path, backup/restore semantics, and
  the peer-TLS surface + raft-host TLS seam gap. The committed Dockerfile
  fixture moves EXPOSE 8080 -> 7000 (render reads the fixture via
  include_str!, so dockerfile-render byte-equality is preserved one-sided),
  and the `relay llm` operations topic gains the --grace-secs
  (RELAY_GRACE_SECS) drain knob. No runtime behavior changes.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-declare-service-umbrella-sync-docs-logic
entry: traits
nodes:
  traits:
    label: "aw.toml traits -> service umbrella + honest extras"
    kind: start
  readme_sync:
    label: "README: replace relay-server/relay-raft rows with single relay bin + WI #1204-#1209 work-roots"
    kind: process
  readme_new:
    label: "README: add 4 umbrella-derived capability sections (std endpoints, cli-std, chainable, ec-gates)"
    kind: process
  ha_doc:
    label: "HA.md: auto-mode, RelayStateMachine, lease/ack limitation, RELAY_PEERS, operator CR, backup, peer-TLS gap"
    kind: process
  dockerfile:
    label: "Dockerfile EXPOSE 8080 -> 7000 (include_str! keeps render byte-equal)"
    kind: process
  llm_topic:
    label: "llm operations topic += --grace-secs / RELAY_GRACE_SECS"
    kind: process
  verify:
    label: "aw capability check >= baseline; cargo test -p relay green"
    kind: terminal
edges:
  - from: traits
    to: readme_new
    label: "umbrella derives 4 baseline capabilities"
  - from: readme_sync
    to: verify
  - from: readme_new
    to: verify
  - from: ha_doc
    to: verify
    label: "Deploy-row requirement satisfied"
  - from: dockerfile
    to: verify
    label: "byte-equality test"
  - from: llm_topic
    to: verify
    label: "spec_cli llm topic test"
---
flowchart TD
  traits["aw.toml traits -> service umbrella + honest extras"]
  readme_sync["README: single relay bin rows + WI #1204-#1209 work-roots"]
  readme_new["README: 4 umbrella-derived capability sections"]
  ha_doc["HA.md: auto-mode HA story"]
  dockerfile["Dockerfile EXPOSE 8080 -> 7000"]
  llm_topic["llm operations topic += --grace-secs"]
  verify["aw capability check >= baseline; cargo test green"]
  traits -->|"umbrella derives 4 baseline capabilities"| readme_new
  readme_sync --> verify
  readme_new --> verify
  ha_doc -->|"Deploy-row requirement satisfied"| verify
  dockerfile -->|"byte-equality test"| verify
  llm_topic -->|"spec_cli llm topic test"| verify
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-declare-service-umbrella-sync-docs-verification
requirements:
  capability-contract-parses-at-baseline:
    id: R1R2
    text: "With the service-umbrella traits in aw.toml and the synced README capability contract (no relay-server/relay-raft references; four umbrella-derived capability sections added), `aw capability check --project relay` parses the contract and reports no new blockers beyond the pre-existing baseline."
    kind: regression
    risk: medium
    verify: aw capability check --project relay (manual gate, compared against the pre-change baseline output)
  dockerfile-byte-equality:
    id: R4
    text: "After EXPOSE 8080 -> 7000 in the committed Dockerfile, `relay dockerfile render --variant source` reproduces the committed fixture byte-for-byte (render reads the fixture via include_str!)."
    kind: regression
    risk: low
    verify: projects/relay/tests/deploy_cli.rs dockerfile_render_reproduces_committed_fixtures
  ha-doc-matches-shipped-semantics:
    id: R3
    text: "HA.md's claims (auto-mode flip on REPLICAS_PER_SHARD>1, publish-only replication, fsynced applied-index marker, node-local lease/ack at-least-once failover, RELAY_PEERS override, backup/restore merge semantics) match the shipped raft/backup behavior already gated by the raft and backup test suites."
    kind: functional
    risk: low
    verify: projects/relay/tests/raft_cluster.rs, projects/relay/tests/raft_persistence.rs, projects/relay/tests/backup.rs (existing gates; HA.md is descriptive)
  llm-grace-knob-documented:
    id: R5
    text: "The `relay llm` operations topic documents the --grace-secs / RELAY_GRACE_SECS graceful-drain knob alongside the existing serve/auth/peer-TLS/backup/deploy surfaces."
    kind: functional
    risk: low
    verify: projects/relay/tests/spec_cli.rs llm_operations_topic_documents_the_new_surfaces
---
flowchart TD
    r3[R3 ha doc matches shipped semantics] --> projects_relay_tests_raft_cluster_rs_projects_relay_tests_raft_persistence_rs_projects_relay_tests_backup_rs_existing_gates_ha_md_is_descriptive[projects/relay/tests/raft_cluster.rs, projects/relay/tests/raft_persistence.rs, projects/relay/tests/backup.rs (existing gates; HA.md is descriptive)]
    r4[R4 dockerfile byte equality] --> projects_relay_tests_deploy_cli_rs_dockerfile_render_reproduces_committed_fixtures[projects/relay/tests/deploy_cli.rs dockerfile_render_reproduces_committed_fixtures]
    r5[R5 llm grace knob documented] --> projects_relay_tests_spec_cli_rs_llm_operations_topic_documents_the_new_surfaces[projects/relay/tests/spec_cli.rs llm_operations_topic_documents_the_new_surfaces]
    r1r2[R1R2 capability contract parses at baseline] --> aw_capability_check_project_relay_manual_gate_compared_against_the_pre_change_baseline_output[aw capability check --project relay (manual gate, compared against the pre-change baseline output)]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/relay/aw.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "[capability.profile].traits -> [\"service\", \"long_running\", \"cli_facing\", \"competitive_replacement\", \"network_exposed\", \"primary_replicas\"] (the umbrella expands to http2_api/kubernetes_native/standard_endpoints/ec_gated/cli_std/chainable_output)."
  - path: projects/relay/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Capability contract sync: drop every relay-server/relay-raft/src-bin evidence reference for the single relay bin + WI #1204-#1209 work-roots; add the four umbrella-derived capability sections (standard-operational-endpoints, cli-standard-surface, chainable-output-conformance with honest partial maturity, ec-gates-configured); update security-hardening for shipped RELAY_AUTH + peer-TLS surface; keep the field-contract + work-root-table format."
  - path: projects/relay/HA.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "New archetype-required HA doc: auto-mode (REPLICAS_PER_SHARD>1 flips raft), RelayStateMachine (publish replication, snapshot/compaction, fsynced applied-index marker), node-local lease/ack at-least-once failover limitation, RELAY_PEERS override, operator CR as the production HA path, backup/restore semantics, peer-TLS surface + raft-host TLS seam gap."
  - path: projects/relay/Dockerfile
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "EXPOSE 8080 -> 7000 (the standard serve port; RELAY_BIND default 0.0.0.0:7000). Render reads this fixture via include_str!, so dockerfile_render_reproduces_committed_fixtures stays byte-equal."
  - path: projects/relay/src/llm.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Operations topic documents --grace-secs (RELAY_GRACE_SECS, default 10) graceful-drain knob; covered by spec_cli llm_operations_topic_documents_the_new_surfaces."
```
