---
id: tape-production-claim-closure-ec
summary: Production EC closure for the implemented Tape CLI, replay journal, checkpoint, route inventory, competitor evidence, and standard service claims.
fill_sections: [e2e-test]
---

# EC: Tape Production Claim Closure

This EC binds Tape's implemented README claims to executable gates. It covers
the first Lumen-style local service slice only: CLI conventions, chainable local
commands, append/replay behavior, durable consumer checkpoints, route inventory,
competitor feature/performance evidence, and standard operational route
declarations. Planned raft, h2c server, retention workers, k8s, security, and
real-service external broker calibration remain outside this production slice.

## External Contracts
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: tape-cli-interface-replay-verbs
    capability_id: cli-interface
    claim_id: tape-cli-convention-and-replay-verbs
    contract_id: tape-cli-command-surface-and-replay-ergonomics
    category: behavior
    test_path: projects/tape/tests/behavior_tape_claim_cli_interface.rs
    command: "cargo test -p tape --test cli_contract -- --nocapture"
    assertions:
      - "The tape binary exposes append, replay, checkpoint, spec, llm, upgrade, and issue commands."
      - "The local append/replay/checkpoint workflow runs end to end through the CLI."

  - id: tape-cli-standard-surface
    capability_id: cli-standard-surface
    claim_id: shared-llm-upgrade-issue-surface
    contract_id: tape-shared-cli-std-surface
    category: behavior
    test_path: projects/tape/tests/behavior_tape_claim_cli_standard_surface.rs
    command: "cargo test -p tape --test cli_contract help_ships_standard_and_replay_commands -- --exact --nocapture"
    assertions:
      - "The mandatory llm, upgrade, and issue command groups remain visible on top-level help."

  - id: tape-chainable-output-conformance
    capability_id: chainable-output-conformance
    claim_id: local-replay-command-next-markers
    contract_id: tape-local-command-chainability
    category: behavior
    test_path: projects/tape/tests/behavior_tape_claim_chainable_output.rs
    command: "cargo test -p tape --test cli_contract append_replay_checkpoint_roundtrip -- --exact --nocapture"
    assertions:
      - "Append, replay, and checkpoint commands emit parseable JSON payloads with next-step markers for agent chaining."

  - id: tape-topic-replay-journal
    capability_id: topic-replay-journal
    claim_id: append-and-replay-contract
    contract_id: tape-append-replay-local-journal
    category: behavior
    test_path: projects/tape/tests/behavior_tape_claim_topic_replay_journal.rs
    command: "cargo test -p tape tests::append_and_replay_by_offset_and_time --lib -- --exact --nocapture"
    assertions:
      - "Tape assigns ordered offsets per topic."
      - "Tape replays by offset and by timestamp."

  - id: tape-consumer-checkpoints
    capability_id: consumer-checkpoints
    claim_id: durable-consumer-cursor-contract
    contract_id: tape-durable-consumer-checkpoint
    category: behavior
    test_path: projects/tape/tests/behavior_tape_claim_consumer_checkpoints.rs
    command: "cargo test -p tape tests::checkpoints_advance_and_reject_stale_offsets --lib -- --exact --nocapture"
    assertions:
      - "Tape stores consumer checkpoints."
      - "Tape rejects stale checkpoint writes."
      - "Tape rejects checkpoints beyond the topic end offset."

  - id: tape-http2-api-list
    capability_id: http2-api-list
    claim_id: h2c-openapi-route-list
    contract_id: tape-offline-openapi-route-inventory
    category: behavior
    test_path: projects/tape/tests/behavior_tape_claim_http2_api_list.rs
    command: "cargo test -p tape --test cli_contract spec_routes_list_topic_contract -- --exact --nocapture"
    assertions:
      - "The offline route inventory includes topic append, replay, and checkpoint routes."
      - "The offline route inventory includes the OpenAPI route."

  - id: tape-standard-operational-endpoints
    capability_id: standard-operational-endpoints
    claim_id: standard-service-route-inventory
    contract_id: tape-standard-service-endpoint-inventory
    category: behavior
    test_path: projects/tape/tests/behavior_tape_claim_standard_operational_endpoints.rs
    command: "cargo test -p tape --test cli_contract spec_routes_list_topic_contract -- --exact --nocapture"
    assertions:
      - "The offline route inventory includes /healthz, /readyz, /metrics, /openapi.json, and /docs."

  - id: tape-competitor-performance-claim-closure
    capability_id: competitor-performance
    claim_id: topic-replay-competitor-performance-baseline
    contract_id: topic-replay-performance-local-and-nats-win
    category: efficiency
    test_path: projects/tape/tests/behavior_tape_claim_competitor_performance_claim_closure.rs
    command: "cargo test -p tape --test tape_perf_gate -- --nocapture && cargo test -p tape --test tape_vs_nats_jetstream -- --nocapture"
    assertions:
      - "The local Tape performance regression gate passes for append, replay, and checkpoint operations."
      - "Tape's NATS JetStream local backlog replay win is backed by a real-service benchmark gate."
      - "Other replay-log peer performance wins remain unclaimed until calibrated real-service benchmark runs exist."
```
