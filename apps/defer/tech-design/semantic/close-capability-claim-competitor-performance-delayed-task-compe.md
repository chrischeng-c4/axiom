---
id: '2217'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: defer-competitor-performance-ceiling-verification
entry: parse_contract
nodes:
  parse_contract: { kind: start, label: "parse exact local benchmark and scope contract" }
  contract_ok: { kind: decision, label: "workload metrics threshold and no-overclaim fields exact?" }
  run_defer: { kind: process, label: "complete Defer durable enqueue lease ack batches" }
  defer_ok: { kind: decision, label: "exact counts settlements samples and metrics positive?" }
  run_relay: { kind: process, label: "complete Relay durable enqueue lease ack batches" }
  relay_ok: { kind: decision, label: "exact counts acknowledgements samples and metrics positive?" }
  compare: { kind: process, label: "compute Defer to Relay throughput ratio" }
  ratio_ok: { kind: decision, label: "finite ratio at least zero point eight?" }
  emit: { kind: process, label: "emit numeric report with explicit same-host scope" }
  fail: { kind: terminal, label: "performance ceiling contract fails closed" }
  verified: { kind: terminal, label: "bounded local implementation overhead verified" }
edges:
  - { from: parse_contract, to: contract_ok }
  - { from: contract_ok, to: run_defer, label: "yes" }
  - { from: contract_ok, to: fail, label: "no" }
  - { from: run_defer, to: defer_ok }
  - { from: defer_ok, to: run_relay, label: "yes" }
  - { from: defer_ok, to: fail, label: "no" }
  - { from: run_relay, to: relay_ok }
  - { from: relay_ok, to: compare, label: "yes" }
  - { from: relay_ok, to: fail, label: "no" }
  - { from: compare, to: ratio_ok }
  - { from: ratio_ok, to: emit, label: "yes" }
  - { from: ratio_ok, to: fail, label: "no" }
  - { from: emit, to: verified }
---
flowchart TD
    parse_contract([parse benchmark contract]) --> contract_ok{contract exact?}
    contract_ok -->|yes| run_defer[run exact Defer durable lifecycle]
    contract_ok -->|no| fail([fail closed])
    run_defer --> defer_ok{Defer counts and metrics exact?}
    defer_ok -->|yes| run_relay[run exact Relay durable lifecycle]
    defer_ok -->|no| fail
    run_relay --> relay_ok{Relay counts and metrics exact?}
    relay_ok -->|yes| compare[compute throughput ratio]
    relay_ok -->|no| fail
    compare --> ratio_ok{ratio at least zero point eight?}
    ratio_ok -->|yes| emit[emit scoped numeric report]
    ratio_ok -->|no| fail
    emit --> verified([local overhead ceiling verified])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/defer/tests/relay_performance_ceiling.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: defer_stays_within_twenty_percent_of_relay_scheduler_ceiling
    reason: "Own the release-mode oracle for exact same-host workload identity, completed-operation cardinality, positive finite metrics, explicit no-overclaim scope, and the hard Defer-to-Relay ratio threshold."
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: defer-competitor-performance-ceiling-verification
requirements:
  bounded_scheduler_overhead:
    id: R3
    text: "The measured Defer throughput divided by Relay throughput is finite and at least 0.80 under the identical declared workload."
    kind: efficiency
    risk: high
    verify: cargo test --release -p defer --test relay_performance_ceiling -- --ignored --nocapture
  explicit_scope_boundary:
    id: R4
    text: "A parsed machine contract and emitted report identify Relay as a same-host sibling implementation comparator, mark RSS process-shared, reject dated observations as authoritative, and set Cloud Tasks performance and universal superiority claims to false."
    kind: regression
    risk: high
    verify: cargo test --release -p defer --test relay_performance_ceiling -- --ignored --nocapture
  fail_closed_measurements:
    id: R2
    text: "Every batch and settlement count is exact, and both sides require finite positive throughput, CPU, disk amplification plus non-zero p50, p95, p99, process-shared RSS, and durable disk bytes before emitting zero-error results."
    kind: efficiency
    risk: high
    verify: cargo test --release -p defer --test relay_performance_ceiling -- --ignored --nocapture
  generated_performance_ec:
    id: R5
    text: "The accepted competitor-performance EC remains generated as an explicit efficiency wrapper bound to delayed-task-competitor-performance-baseline."
    kind: regression
    risk: medium
    verify: aw ec check --project defer
  identical_durable_workload:
    id: R1
    text: "Defer and Relay each complete exactly 1,000 operations in ten 100-item batches with the same exactly asserted 128-byte serialized JSON payload, one voter, fsync-always durability, and durable enqueue, committed lease, committed acknowledgement lifecycle."
    kind: efficiency
    risk: high
    verify: cargo test --release -p defer --test relay_performance_ceiling -- --ignored --nocapture
---
flowchart TD
    r1[R1 identical durable workload] --> cargo_test_release_p_defer_test_relay_performance_ceiling_ignored_nocapture[cargo test --release -p defer --test relay_performance_ceiling -- --ignored --nocapture]
    r2[R2 fail closed measurements] --> cargo_test_release_p_defer_test_relay_performance_ceiling_ignored_nocapture
    r3[R3 bounded scheduler overhead] --> cargo_test_release_p_defer_test_relay_performance_ceiling_ignored_nocapture
    r4[R4 explicit scope boundary] --> cargo_test_release_p_defer_test_relay_performance_ceiling_ignored_nocapture
    r5[R5 generated performance ec] --> aw_ec_check_project_defer[aw ec check --project defer]
```
