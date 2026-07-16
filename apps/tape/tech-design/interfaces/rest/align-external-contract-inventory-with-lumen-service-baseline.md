---
id: '1815'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-lumen-ec-baseline-alignment
entry: inventory
nodes:
  inventory:
    kind: start
    label: "Compare Lumen and Tape EC taxonomies"
  project:
    kind: process
    label: "Project shared-service categories onto Tape commands and tests"
  verify:
    kind: process
    label: "Generate EC inventory and run focused Tape gates"
  classify:
    kind: decision
    label: "Does a failure expose a shared mechanism?"
  shared:
    kind: terminal
    label: "Create a libs follow-up"
  domain:
    kind: terminal
    label: "Create a Tape domain follow-up"
edges:
  - { from: inventory, to: project }
  - { from: project, to: verify }
  - { from: verify, to: classify }
  - { from: classify, to: shared, label: shared }
  - { from: classify, to: domain, label: domain }
---
flowchart TD
  inventory["Compare Lumen and Tape EC taxonomies"] --> project["Project only shared-service categories onto Tape"] --> verify["Generate EC inventory and run focused Tape gates"] --> classify{"Shared mechanism missing?"}
  classify -->|yes| shared(["Create libs follow-up"])
  classify -->|no| domain(["Create Tape-domain follow-up"])
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/external-contracts/cli-interface/behavior/cli-interface.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "Add Tape-owned CLI, offline OpenAPI, generated-client, h2c, and llm contract cases adapted from the Lumen EC taxonomy. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)."
  - path: apps/tape/external-contracts/topology/behavior/shard-topology.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "State Tape shard and replica topology, durable replay, backup seed, and operator ownership rules with Tape raft and operator tests. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)."
  - path: apps/tape/external-contracts/long-running-stability/behavior/devops-render.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "Add the Tape operator render contract for shared StatefulSet, Services, PDB, backup, and policy resources. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)."
  - path: apps/tape/external-contracts/long-running-stability/behavior/meta-api.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "Add Tape standard liveness, readiness, metrics, version, and OpenAPI operational-surface contract. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)."
  - path: apps/tape/external-contracts/long-running-stability/stability/replay-resilience.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "Add Tape replay admission, restart, and recovery stability contract without importing search latency assertions. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)."
  - path: apps/tape/external-contracts/long-running-stability/stability/resilience-survival.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "Add Tape leader-loss and durable replay survival contract using Tape raft-focused gates. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)."
  - path: apps/tape/external-contracts/security-hardening/security/access-control.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "Add Tape topic and subscription authorization, admission-limit, and malformed-request security contract. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)."
  - path: apps/tape/external-contracts/security-hardening/security/auth-bearer-rbac.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "Add shared bearer-token authentication and route-role authorization contract for Tape. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)."
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-lumen-ec-baseline-alignment-verification
requirements:
  auth_ec_contract:
    id: R6
    text: "Tape's copied bearer/RBAC and access-control categories must enforce topic write and replay/checkpoint read grants."
    kind: security
    risk: high
    verify: apps/tape/tests/service_auth.rs::append_requires_write_grant_on_topic
  authz_ec_contract:
    id: R7
    text: "Tape's access-control EC category must prevent replay and checkpoint access without a topic read grant."
    kind: security
    risk: high
    verify: apps/tape/tests/service_auth.rs::replay_and_checkpoint_require_read_grant_on_topic
  cli_ec_contract:
    id: R1
    text: "Tape's copied CLI/DX EC category must exercise only the Tape offline spec, CLI, generated-client, h2c, and llm surfaces."
    kind: functional
    risk: medium
    verify: apps/tape/tests/behavior_tape_claim_cli_interface.rs::tape_cli_interface_replay_verbs
  meta_api_ec_contract:
    id: R3
    text: "Tape's operational EC category must preserve the standard liveness, readiness, metrics, and OpenAPI surface."
    kind: regression
    risk: medium
    verify: apps/tape/tests/behavior_tape_claim_standard_operational_endpoints.rs::tape_standard_operational_endpoints
  operator_ec_contract:
    id: R2
    text: "Tape's deployment-render EC category must prove the shared StatefulSet operator output and Tape-specific policy wiring."
    kind: regression
    risk: high
    verify: apps/tape/tests/operator.rs::render_emits_expected_child_objects
  stability_ec_contract:
    id: R4
    text: "Tape's resilience EC categories must retain committed append history and consumer checkpoint progress across repeated restarts."
    kind: regression
    risk: high
    verify: apps/tape/tests/long_running_stability.rs::repeated_restarts_preserve_append_history_and_checkpoint_progress
  topology_ec_contract:
    id: R5
    text: "Tape's topology EC category must prove leader loss, reelection, and no committed replay event loss."
    kind: regression
    risk: high
    verify: apps/tape/tests/raft_failover.rs::kill_9_leader_survivors_reelect_with_no_committed_event_loss
---
flowchart TD
    r1[R1 cli ec contract] --> apps_tape_tests_behavior_tape_claim_cli_interface_rs_tape_cli_interface_replay_verbs[apps/tape/tests/behavior_tape_claim_cli_interface.rs::tape_cli_interface_replay_verbs]
    r2[R2 operator ec contract] --> apps_tape_tests_operator_rs_render_emits_expected_child_objects[apps/tape/tests/operator.rs::render_emits_expected_child_objects]
    r3[R3 meta api ec contract] --> apps_tape_tests_behavior_tape_claim_standard_operational_endpoints_rs_tape_standard_operational_endpoints[apps/tape/tests/behavior_tape_claim_standard_operational_endpoints.rs::tape_standard_operational_endpoints]
    r4[R4 stability ec contract] --> apps_tape_tests_long_running_stability_rs_repeated_restarts_preserve_append_history_and_checkpoint_progress[apps/tape/tests/long_running_stability.rs::repeated_restarts_preserve_append_history_and_checkpoint_progress]
    r5[R5 topology ec contract] --> apps_tape_tests_raft_failover_rs_kill_9_leader_survivors_reelect_with_no_committed_event_loss[apps/tape/tests/raft_failover.rs::kill_9_leader_survivors_reelect_with_no_committed_event_loss]
    r6[R6 auth ec contract] --> apps_tape_tests_service_auth_rs_append_requires_write_grant_on_topic[apps/tape/tests/service_auth.rs::append_requires_write_grant_on_topic]
    r7[R7 authz ec contract] --> apps_tape_tests_service_auth_rs_replay_and_checkpoint_require_read_grant_on_topic[apps/tape/tests/service_auth.rs::replay_and_checkpoint_require_read_grant_on_topic]
```
