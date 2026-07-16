---
id: '1815'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-lumen-ec-baseline-contract
entry: lumen_taxonomy
nodes:
  lumen_taxonomy:
    kind: start
    label: "Lumen EC taxonomy is the structural reference"
  tape_adapter:
    kind: process
    label: "Rewrite every command and assertion for Tape"
  tape_cases:
    kind: process
    label: "Emit CLI, topology, resilience, meta API, and security cases"
  aw_inventory:
    kind: process
    label: "AW generates and checks the EC inventory"
  failures:
    kind: decision
    label: "Classify failing proof by mechanism ownership"
  library_gap:
    kind: terminal
    label: "Shared capability follow-up under libs"
  tape_gap:
    kind: terminal
    label: "Topic/replay follow-up under Tape"
edges:
  - { from: lumen_taxonomy, to: tape_adapter }
  - { from: tape_adapter, to: tape_cases }
  - { from: tape_cases, to: aw_inventory }
  - { from: aw_inventory, to: failures }
  - { from: failures, to: library_gap, label: reusable }
  - { from: failures, to: tape_gap, label: domain }
---
flowchart TD
  lumen_taxonomy["Lumen EC taxonomy: structure only"] --> tape_adapter["Rewrite each command and assertion for Tape"] --> tape_cases["Tape CLI, topology, resilience, meta API, security cases"] --> aw_inventory["AW generates and checks inventory"] --> failures{"Who owns the missing proof?"}
  failures -->|reusable| library_gap(["libs follow-up"])
  failures -->|domain| tape_gap(["Tape follow-up"])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/external-contracts/cli-interface/behavior/cli-interface.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "Tape-owned CLI, offline OpenAPI, generated-client, h2c, and llm contract cases adapted from Lumen's taxonomy. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)."
  - path: apps/tape/external-contracts/topology/behavior/shard-topology.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "Tape shard/replica, durable replay, backup seed, and operator topology contract. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)."
  - path: apps/tape/external-contracts/long-running-stability/behavior/devops-render.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "Tape shared StatefulSet deployment render contract. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)."
  - path: apps/tape/external-contracts/long-running-stability/behavior/meta-api.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "Tape standard operational endpoint contract. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)."
  - path: apps/tape/external-contracts/long-running-stability/stability/replay-resilience.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "Tape restart/recovery and admission stability contract. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)."
  - path: apps/tape/external-contracts/long-running-stability/stability/resilience-survival.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "Tape leader-loss and durable replay survival contract. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)."
  - path: apps/tape/external-contracts/security-hardening/security/access-control.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "Tape topic/subscription authorization, admission-limit, and malformed-request security contract. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)."
  - path: apps/tape/external-contracts/security-hardening/security/auth-bearer-rbac.md
    action: create
    section: logic
    impl_mode: hand-written
    description: "Tape shared bearer-token and route-role authorization contract. generator gap: missing-generator:tape-ec-lumen-baseline (#1815)."
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
