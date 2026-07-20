---
id: "1255"
summary: (fill)
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-pull-subscription-contract
entry: pull_request
nodes:
  pull_request:
    kind: start
    label: "PullSubscriptionBatch uses the durable topic/name checkpoint as a next-offset cursor"
  validate_limit:
    kind: decision
    label: "--limit defaults to 100 and must not exceed MAX_PULL_BATCH=1000"
  limit_error:
    kind: terminal
    label: "oversized request returns SubscriptionError::PullBatchTooLarge without replaying or advancing state"
  resolve_pull:
    kind: decision
    label: "topic/name resolves to an existing pull-only Subscription"
  mode_error:
    kind: terminal
    label: "missing resource returns SubscriptionError"
  replay_window:
    kind: process
    label: "read checkpoint offset or 0, then replay the bounded event window; next_offset is last event offset + 1 or cursor when empty"
  pull_result:
    kind: terminal
    label: "emit PullSubscriptionBatch { events, cursor, next_offset, limit }; no cursor mutation occurs on pull"
  ack_request:
    kind: process
    label: "subscription ack confirms the resource is pull, then calls put_checkpoint(topic, name, offset)"
  ack_result:
    kind: terminal
    label: "checkpoint success is the durable ack; stale and beyond-end errors propagate unchanged through SubscriptionAckError"
  inventory:
    kind: terminal
    label: "served and offline specs declare pull and ack routes; leases, push, consumer groups, and executor ownership stay excluded"
edges:
  - { from: pull_request, to: validate_limit }
  - { from: validate_limit, to: limit_error, label: "limit > 1000" }
  - { from: validate_limit, to: resolve_pull, label: "bounded" }
  - { from: resolve_pull, to: mode_error, label: "missing" }
  - { from: resolve_pull, to: replay_window, label: "exists" }
  - { from: replay_window, to: pull_result }
  - { from: pull_result, to: ack_request }
  - { from: ack_request, to: ack_result }
  - { from: ack_result, to: inventory }
---
flowchart TD
    pull_request["PullSubscriptionBatch from topic/name checkpoint cursor"] --> validate_limit{"limit <= 1000; default 100?"}
    validate_limit -->|oversized| limit_error(["PullBatchTooLarge; no state changes"])
    validate_limit -->|bounded| resolve_pull{"pull subscription exists?"}
    resolve_pull -->|missing| mode_error(["SubscriptionError; no delivery"])
    resolve_pull -->|exists| replay_window["replay bounded window from checkpoint or 0; compute next_offset"]
    replay_window --> pull_result(["events/cursor/next_offset; pull never advances cursor"])
    pull_result --> ack_request["ack validates pull then delegates to put_checkpoint"]
    ack_request --> ack_result(["durable ack or stale/beyond-end rejection"])
    ack_result --> inventory(["pull/ack schema only; no lease/push/consumer-group ownership"])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-pull-subscription-verification
requirements:
  backpressure_limit:
    id: R3
    text: "A pull request above MAX_PULL_BATCH is rejected without returning events or changing durable state."
    kind: negative
    risk: medium
    verify: cargo test -p tape tests::pull_subscription_rejects_oversized_window --lib -- --exact
  bounded_pull_cursor:
    id: R1
    text: "A pull subscription reads from its durable checkpoint (or offset zero), returns no more than its requested bounded window, and leaves the checkpoint unchanged until ack."
    kind: functional
    risk: high
    verify: cargo test -p tape tests::pull_subscription_uses_checkpoint_cursor_and_never_implicitly_acks --lib -- --exact
  cli_surface:
    id: R4
    text: "The Tape CLI exposes subscription pull and ack forms and their local file-backed round-trip follows explicit pull then ack semantics."
    kind: functional
    risk: medium
    verify: cargo test -p tape --test cli_contract pull_subscription_cli_roundtrip -- --exact
  inventory_scope:
    id: R5
    text: "Live routes/OpenAPI/JSON Schema declare pull and ack contracts while push, leases, consumer groups, and executor ownership remain absent."
    kind: contract
    risk: medium
    verify: cargo test -p tape --test cli_contract pull_subscription_spec_inventory -- --exact
  performance_scope:
    id: R6
    text: "The local performance gate remains the bounded pull/replay path and does not add push delivery or claim uncalibrated peer wins."
    kind: regression
    risk: medium
    verify: cargo test -p tape --test tape_perf_gate -- --nocapture
  pull_ack_safety:
    id: R2
    text: "Subscription ack requires an existing pull subscription and preserves stale and beyond-end checkpoint rejection semantics."
    kind: regression
    risk: high
    verify: cargo test -p tape tests::pull_subscription_ack_reuses_checkpoint_guards --lib -- --exact
---
flowchart TD
    r1[R1 bounded pull cursor] --> cargo_test_p_tape_tests_pull_subscription_uses_checkpoint_cursor_and_never_implicitly_acks_lib_exact[cargo test -p tape tests::pull_subscription_uses_checkpoint_cursor_and_never_implicitly_acks --lib -- --exact]
    r2[R2 pull ack safety] --> cargo_test_p_tape_tests_pull_subscription_ack_reuses_checkpoint_guards_lib_exact[cargo test -p tape tests::pull_subscription_ack_reuses_checkpoint_guards --lib -- --exact]
    r3[R3 backpressure limit] --> cargo_test_p_tape_tests_pull_subscription_rejects_oversized_window_lib_exact[cargo test -p tape tests::pull_subscription_rejects_oversized_window --lib -- --exact]
    r4[R4 cli surface] --> cargo_test_p_tape_test_cli_contract_pull_subscription_cli_roundtrip_exact[cargo test -p tape --test cli_contract pull_subscription_cli_roundtrip -- --exact]
    r5[R5 inventory scope] --> cargo_test_p_tape_test_cli_contract_pull_subscription_spec_inventory_exact[cargo test -p tape --test cli_contract pull_subscription_spec_inventory -- --exact]
    r6[R6 performance scope] --> cargo_test_p_tape_test_tape_perf_gate_nocapture[cargo test -p tape --test tape_perf_gate -- --nocapture]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/src/lib.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add bounded PullSubscriptionBatch reads from pull subscription checkpoint cursors, explicit pull ack delegation, MAX_PULL_BATCH validation, and regression tests. Preserve existing checkpoint errors through a subscription-specific ack error; do not add leases, workers, or raft cursor commands. generator gap: missing-generator:logic:tape-pull-subscription (#1255)."
  - path: apps/tape/src/bin/tape.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Add `tape subscription pull TOPIC NAME [--limit]` and `tape subscription ack TOPIC NAME --offset`; output cursor/window metadata and next markers, and keep explicit ack separate from pull. generator gap: missing-generator:cli:tape-pull-subscription (#1255)."
  - path: apps/tape/src/spec.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: "Add topic subscription pull/ack routes and PullSubscriptionBatch/PullAck schemas to both served and offline contracts. Pull stays side-effect-free; ack uses the committed checkpoint command in replica mode. generator gap: missing-generator:openapi:tape-pull-subscription (#1255)."
  - path: apps/tape/tests/cli_contract.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Add deterministic CLI and spec-inventory coverage for bounded pull followed by explicit ack. generator gap: missing-generator:test:tape-pull-subscription (#1255)."
  - path: apps/tape/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Describe bounded pull/replay as Tape's high-QPS subscription path, explain cursor/ack/backpressure semantics, and keep push worker/raft consensus claims excluded."
  - path: apps/tape/external-contracts/claim-closure/production-claims.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Scope the checkpoint and performance claims to pull cursor/ack and bounded replay, with no push reliability claim."
  - path: apps/tape/external-contracts/competitor-performance/efficiency/competitive-benchmark.md
    action: modify
    section: e2e-test
    impl_mode: hand-written
    description: "Call existing local and peer replay benchmarks the bounded pull/replay path; leave all existing calibration ratios and peer exclusions unchanged."
  - path: apps/tape/tech-design/semantic/source/apps-tape-src-lib-rs.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Refresh semantic source coverage for pull cursor, explicit ack, and bounded-window behavior."
  - path: apps/tape/tech-design/semantic/source/apps-tape-src-bin-tape-rs.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Refresh semantic source coverage for the pull/ack CLI surface."
  - path: apps/tape/tech-design/semantic/source/apps-tape-src-spec-rs.md
    action: modify
    section: schema
    impl_mode: hand-written
    description: "Refresh semantic source coverage for pull/ack offline contract inventory."
  - path: apps/tape/tech-design/semantic/source/apps-tape-tests-cli-contract-rs.md
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Refresh semantic source coverage for pull/ack CLI contract tests."
```
