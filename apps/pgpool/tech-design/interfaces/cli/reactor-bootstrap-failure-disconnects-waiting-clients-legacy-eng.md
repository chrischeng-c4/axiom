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
entry: transaction_backend_lifecycle
nodes:
  challenge: { kind: process, label: "bootstrap sees auth challenge or backend error" }
  retain: { kind: terminal, label: "drop backend; keep Waiting client and state queue" }
  lease: { kind: decision, label: "acquired legacy lease is fresh" }
  reject: { kind: terminal, label: "close fresh lease and emit synthesized pool rejection" }
  relay: { kind: terminal, label: "relay transaction only on authenticated lease" }
edges:
  - { from: transaction_backend_lifecycle, to: challenge, label: reactor bootstrap }
  - { from: challenge, to: retain }
  - { from: transaction_backend_lifecycle, to: lease, label: legacy acquire }
  - { from: lease, to: reject, label: fresh }
  - { from: lease, to: relay, label: reused or locally bootstrapped }
---
flowchart TD
  bootstrap["bootstrap challenge/error"] --> retain["discard backend; retain waiter"]
  acquire["legacy acquire"] --> fresh{"fresh unauthenticated?"}
  fresh -->|yes| reject["synthesized rejection"]
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
