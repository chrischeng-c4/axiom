---
id: '2159'
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    gap: "topic-replay-competitor-feature-matrix"
    claim: "topic-replay-competitor-feature-matrix"
    coverage: full
    rationale: "Replace self-asserted competitor rows with a versioned official-source baseline consumed by the executable Tape oracle."
  - id: "competitor-performance"
    role: primary
    gap: "topic-replay-competitor-performance-baseline"
    claim: "topic-replay-competitor-performance-baseline"
    coverage: full
    rationale: "Make local, NATS, and Kafka performance pass/fail independent, fixed, and fail closed."
  - id: "http2-api-list"
    role: primary
    gap: "backup-service-tls-spec-gen-clients"
    claim: "backup-service-tls-spec-gen-clients"
    coverage: full
    rationale: "Execute a non-zero three-language generated-client journey that inspects Tape's public route scope."
  - id: "security-hardening"
    role: primary
    gap: "topic-replay-security-boundary"
    claim: "topic-replay-security-boundary"
    coverage: full
    rationale: "Align security assertions with the real auth, rotation, admission, audit, guard, and meter journeys that run."
  - id: "ec-gates-configured"
    role: primary
    gap: "tape-vat-meter-guard-ec-gates-observability"
    claim: "tape-vat-meter-guard-ec-gates-observability"
    coverage: full
    rationale: "Regenerate the full EC inventory and close the six digest-bound false-green paths before production readiness."
summary: >
  Eliminate six independently reviewed Tape production EC false-green paths by
  using fail-closed prerequisites, fixed external-contract thresholds,
  independent competitor provenance, non-zero generated-client evidence, and
  security runners that execute the behavior they claim.
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-production-ec-independent-oracles
entry: load_review
nodes:
  load_review:
    kind: start
    label: "Load the six digest-bound false-green findings"
  local_perf:
    kind: process
    label: "Compare observed local metrics to EC-owned fixed limits"
  peer_perf:
    kind: process
    label: "Run real NATS and Kafka peers, fail closed, and compute ratios in the test layer"
  generated_clients:
    kind: process
    label: "Generate and inspect TypeScript, Python, and Rust Tape clients"
  competitor_oracle:
    kind: process
    label: "Read a versioned official-source competitor baseline fixture"
  security_oracle:
    kind: process
    label: "Align auth, audit, guard, and meter commands with executed journeys"
  regenerate:
    kind: process
    label: "Regenerate all EC cases and reject zero-test execution"
  semantic_review:
    kind: decision
    label: "Does independent agent review accept the current digest?"
  revise:
    kind: process
    label: "Revise only the finding's EC source or independent runner"
  verify:
    kind: process
    label: "Run EC verify and the owning TD code-check"
  done:
    kind: terminal
    label: "Tape production EC cannot pass through the six false-green paths"
edges:
  - { from: load_review, to: local_perf }
  - { from: local_perf, to: peer_perf }
  - { from: peer_perf, to: generated_clients }
  - { from: generated_clients, to: competitor_oracle }
  - { from: competitor_oracle, to: security_oracle }
  - { from: security_oracle, to: regenerate }
  - { from: regenerate, to: semantic_review }
  - { from: semantic_review, to: verify, label: "accepted" }
  - { from: semantic_review, to: revise, label: "needs revision" }
  - { from: revise, to: regenerate }
  - { from: verify, to: done }
---
flowchart TD
  load_review([Load six digest-bound findings]) --> local_perf[Independent fixed local limits]
  local_perf --> peer_perf[Fail-closed real peer ratios]
  peer_perf --> generated_clients[Inspect three generated client languages]
  generated_clients --> competitor_oracle[Consume versioned official-source baseline]
  competitor_oracle --> security_oracle[Align security claims and commands]
  security_oracle --> regenerate[Regenerate all EC cases]
  regenerate --> semantic_review{Independent review accepted?}
  semantic_review -->|accepted| verify[Run EC verify and TD code-check]
  semantic_review -->|needs revision| revise[Revise bounded source or runner]
  revise --> regenerate
  verify --> done([False-green paths closed])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/tape/tests/tape_perf_gate.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: local_replay_perf_gate_passes_without_external_win_claims
    description: "Replace Tape-owned verdict delegation with fixed EC-owned workload and threshold assertions. generator gap: missing-generator:test:independent-perf-oracle (#2159)."
  - path: apps/tape/tests/tape_vs_nats_jetstream.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: tape_beats_nats_jetstream_on_local_backlog_replay
    description: "Fail closed and independently compute the NATS/Tape p50 ratio. generator gap: missing-generator:test:real-peer-performance-oracle (#2159)."
  - path: apps/tape/tests/tape_vs_kafka.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: tape_beats_kafka_on_local_backlog_replay
    description: "Fail closed on every Kafka prerequisite and independently compute the Kafka/Tape ratio. generator gap: missing-generator:test:real-peer-performance-oracle (#2159)."
  - path: apps/tape/tests/spec_generated_clients.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Execute tape spec gen for TypeScript, Python, and Rust and inspect emitted route scope. generator gap: missing-generator:test:generated-client-journey (#2159)."
  - path: apps/tape/tests/fixtures/competitor_feature_baseline.json
    action: create
    section: unit-test
    impl_mode: hand-written
    description: "Pin versioned official-source provenance and the reviewed replay-log/topic-exchange feature baseline. generator gap: missing-generator:fixture:external-competitor-baseline (#2159)."
  - path: apps/tape/tests/competitor_feature_parity.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: matrix
    description: "Deserialize the external baseline fixture and compare Tape behavior against it instead of self-asserting rows. generator gap: missing-generator:test:external-competitor-baseline (#2159)."
  - path: apps/tape/external-contracts/cli-interface/behavior/cli-interface.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Point generated-client production evidence at the dedicated non-zero journey. generator gap: missing-generator:ec:runner-reconciliation (#2159)."
  - path: apps/tape/external-contracts/competitor-feature-parity/behavior/topic-exchange-functional.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Declare the versioned official-source fixture as the competitor oracle. generator gap: missing-generator:ec:runner-reconciliation (#2159)."
  - path: apps/tape/external-contracts/security-hardening/security/access-control.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Align access-control assertions with the dynamic Tape and shared service-auth journeys actually executed. generator gap: missing-generator:ec:runner-reconciliation (#2159)."
  - path: apps/tape/external-contracts/security-hardening/security/security-evidence.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Require meter evidence from service_auth rather than unrelated CLI cases. generator gap: missing-generator:ec:runner-reconciliation (#2159)."
  - path: apps/tape/vat.toml
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Compile and meter the real Tape bearer-auth integration journey in the guard runner. generator gap: missing-generator:vat:ec-runner-reconciliation (#2159)."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: tape-production-ec-independent-oracles-verification
requirements:
  external_competitor_baseline:
    id: R4
    text: "Feature parity consumes a versioned fixture with pinned official upstream provenance and compares Tape behavior against that oracle."
    kind: regression
    risk: high
    verify: cargo test -p tape --test competitor_feature_parity -- --nocapture
  generated_client_journey:
    id: R3
    text: "A non-zero integration test emits TypeScript, Python, and Rust clients and checks Tape's public route scope."
    kind: functional
    risk: high
    verify: cargo test -p tape --test spec_generated_clients -- --nocapture
  local_performance_oracle:
    id: R1
    text: "The local gate independently applies the EC-owned 1,000-event workload and fixed 5,000/50,000/5,000-us limits."
    kind: regression
    risk: high
    verify: cargo test -p tape --test tape_perf_gate -- --nocapture
  real_peer_performance_oracles:
    id: R2
    text: "Release NATS and Kafka gates fail closed and independently require a peer/Tape p50 ratio of at least 1.5."
    kind: functional
    risk: high
    verify: cargo test --release -p tape --test tape_vs_nats_jetstream --test tape_vs_kafka -- --nocapture
  security_runner_alignment:
    id: R5
    text: "Guard and meter evidence executes the real Tape service_auth journey and access-control assertions do not overclaim unexecuted audit behavior."
    kind: security
    risk: high
    verify: cd apps/tape && ../../target/debug/vat run guard-security
  terminal_ec_verification:
    id: R6
    text: "All EC sources are represented, independently agent-reviewed, generated, and verified before TD completion."
    kind: regression
    risk: high
    verify: aw ec gen --project tape --verify
---
flowchart TD
    r1[R1 local performance oracle] --> cargo_test_p_tape_test_tape_perf_gate_nocapture[cargo test -p tape --test tape_perf_gate -- --nocapture]
    r2[R2 real peer performance oracles] --> cargo_test_release_p_tape_test_tape_vs_nats_jetstream_test_tape_vs_kafka_nocapture[cargo test --release -p tape --test tape_vs_nats_jetstream --test tape_vs_kafka -- --nocapture]
    r3[R3 generated client journey] --> cargo_test_p_tape_test_spec_generated_clients_nocapture[cargo test -p tape --test spec_generated_clients -- --nocapture]
    r4[R4 external competitor baseline] --> cargo_test_p_tape_test_competitor_feature_parity_nocapture[cargo test -p tape --test competitor_feature_parity -- --nocapture]
    r5[R5 security runner alignment] --> cd_apps_tape_target_debug_vat_run_guard_security[cd apps/tape && ../../target/debug/vat run guard-security]
    r6[R6 terminal ec verification] --> aw_ec_gen_project_tape_verify[aw ec gen --project tape --verify]
```
