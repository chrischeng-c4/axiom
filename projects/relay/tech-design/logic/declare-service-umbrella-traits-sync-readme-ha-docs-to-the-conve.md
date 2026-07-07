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
    kind: step
  readme_sync:
    label: "README: replace relay-server/relay-raft rows with single relay bin + WI #1204-#1209 work-roots"
    kind: step
  readme_new:
    label: "README: add 4 umbrella-derived capability sections (std endpoints, cli-std, chainable, ec-gates)"
    kind: step
  ha_doc:
    label: "HA.md: auto-mode, RelayStateMachine, lease/ack limitation, RELAY_PEERS, operator CR, backup, peer-TLS gap"
    kind: step
  dockerfile:
    label: "Dockerfile EXPOSE 8080 -> 7000 (include_str! keeps render byte-equal)"
    kind: step
  llm_topic:
    label: "llm operations topic += --grace-secs / RELAY_GRACE_SECS"
    kind: step
  verify:
    label: "aw capability check >= baseline; cargo test -p relay green"
    kind: outcome
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
