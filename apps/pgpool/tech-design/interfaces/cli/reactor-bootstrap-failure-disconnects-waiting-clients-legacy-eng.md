---
id: '1880'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: auth-required-fresh-backend-safety
entry: bootstrap_or_transaction_acquire
nodes:
  bootstrap_challenge: { kind: process, label: "bootstrap backend reports auth challenge" }
  discard: { kind: process, label: "discard doomed backend only" }
  wait: { kind: terminal, label: "retain Waiting client for clean backend or timeout" }
  fresh_lease: { kind: decision, label: "legacy transaction lease is fresh" }
  reject: { kind: terminal, label: "close fresh socket and send synthesized rejection" }
  relay: { kind: terminal, label: "relay only authenticated reused or replay-bootstrapped lease" }
edges:
  - { from: bootstrap_or_transaction_acquire, to: bootstrap_challenge, label: reactor bootstrap }
  - { from: bootstrap_challenge, to: discard }
  - { from: discard, to: wait }
  - { from: bootstrap_or_transaction_acquire, to: fresh_lease, label: legacy acquire }
  - { from: fresh_lease, to: reject, label: fresh and no replay-safe startup }
  - { from: fresh_lease, to: relay, label: authenticated lease }
---
flowchart TD
  challenge["bootstrap gets auth challenge"] --> drop["discard backend only"] --> wait["client remains queued"]
  acquire["legacy transaction acquire"] --> fresh{"fresh unauthenticated lease?"}
  fresh -->|yes| reject["close lease + synthesized error"]
  fresh -->|no| relay["relay query"]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/pgpool/src/pool/reactor/runtime.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: close_backend
  - path: apps/pgpool/src/pool/transaction.rs
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: run_transaction_client
  - path: apps/pgpool/tests/trust_startup_replay.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: startup_mismatch_and_auth_challenges_never_replay
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: auth-required-fresh-backend-safety-verification
requirements:
  auth_fresh_lease_rejected:
    id: R1
    text: "An auth-required legacy client receiving a fresh transaction lease gets a synthesized error and its backend never receives a raw Query before Startup/authentication."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test trust_startup_replay auth_required_fresh_legacy_lease_is_rejected_before_query_relay
  bootstrap_keeps_waiter:
    id: R1
    text: "A failed reactor bootstrap discards only the backend; its Waiting client remains scheduler-owned for a later clean backend or normal timeout."
    kind: regression
    risk: high
    verify: cargo test -p pgpool --test trust_startup_replay auth_required_fresh_legacy_lease_is_rejected_before_query_relay
---
flowchart TD
    r1[R1 auth fresh lease rejected] --> cargo_test_p_pgpool_test_trust_startup_replay_auth_required_fresh_legacy_lease_is_rejected_before_query_relay[cargo test -p pgpool --test trust_startup_replay auth_required_fresh_legacy_lease_is_rejected_before_query_relay]
    r1[R1 bootstrap keeps waiter] --> cargo_test_p_pgpool_test_trust_startup_replay_auth_required_fresh_legacy_lease_is_rejected_before_query_relay
```
