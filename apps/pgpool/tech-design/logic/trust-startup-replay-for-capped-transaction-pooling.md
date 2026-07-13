---
id: '1599'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: pgpool-trust-startup-replay
entry: frontend_startup
nodes:
  frontend_startup:
    kind: process
    label: "Read the client startup message before backend admission."
  cached_reply:
    kind: decision
    label: "Does an exact no-challenge startup reply exist?"
  replay_ready:
    kind: process
    label: "Replay the cached ready response without leasing a backend."
  fresh_handshake:
    kind: process
    label: "Lease one fresh backend and relay startup authentication."
  challenge_seen:
    kind: decision
    label: "Did the backend require a client authentication challenge?"
  publish_reply:
    kind: process
    label: "Store the complete safe startup reply for this exact startup."
  transaction_loop:
    kind: terminal
    label: "Lease and reset backends per transaction as normal."
edges:
  - from: frontend_startup
    to: cached_reply
  - from: cached_reply
    to: replay_ready
    label: hit
  - from: replay_ready
    to: transaction_loop
  - from: cached_reply
    to: fresh_handshake
    label: miss
  - from: fresh_handshake
    to: challenge_seen
  - from: challenge_seen
    to: publish_reply
    label: no challenge
  - from: publish_reply
    to: transaction_loop
  - from: challenge_seen
    to: transaction_loop
    label: challenge passthrough
---
flowchart TD
    frontend_startup[Read startup before admission] --> cached_reply{Exact safe reply cached?}
    cached_reply -->|hit| replay_ready[Replay ready response with no backend lease]
    replay_ready --> transaction_loop([Normal transaction leasing])
    cached_reply -->|miss| fresh_handshake[Fresh backend startup/auth relay]
    fresh_handshake --> challenge_seen{Authentication challenge observed?}
    challenge_seen -->|no| publish_reply[Publish exact safe startup reply]
    publish_reply --> transaction_loop
    challenge_seen -->|yes| transaction_loop
```

### Safety boundary

A cache key is the complete ordered `StartupMessage`, not merely user or database. A cached reply is publishable only when the backend handshake reaches `ReadyForQuery` without any client-response authentication challenge. Cleartext-password, MD5, and SASL paths stay on the existing pass-through flow and never populate or consume this cache.

A replay hit sends the cached protocol-ready frames to the matching client and starts the ordinary transaction loop with no retained backend lease. The first successful trust/no-challenge handshake returns its backend through the existing `DISCARD ALL` reset path before any later transaction is acquired.

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: apps/pgpool/src/pool/backend_pool.rs
    action: update
    section: logic
    impl_mode: hand-written
    reason: Store and select exact safe startup replies while waiters re-check the cache before consuming backend capacity.
  - path: apps/pgpool/src/pool/transaction.rs
    action: update
    section: logic
    impl_mode: hand-written
    reason: Read startup before admission and branch between cached reply replay and the fresh handshake path.
  - path: apps/pgpool/src/proxy/relay.rs
    action: update
    section: logic
    impl_mode: hand-written
    reason: Capture only complete no-challenge backend startup replies for safe replay.
  - path: apps/pgpool/tests/trust_startup_replay.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: Exercise exact-match replay, challenge exclusion, and capped concurrent trust startup behavior.
  - path: apps/pgpool/benchmarks/pgbouncer-transaction-pooling/run.sh
    action: update
    section: unit-test
    impl_mode: hand-written
    reason: Assert the fixed 64-client capped benchmark detects startup-cap rejection instead of silently reporting a partial run.
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: pgpool-trust-startup-replay-verification
requirements:
  ac1_capped_trust_clients_complete_without_startup_rejection:
    id: AC1
    text: "With a local trust-auth PostgreSQL backend, 64 same-startup simple-protocol clients complete transactions through a 16-backend transaction pool without the startup-cap saturation failure that #1597 established."
    kind: integration
    risk: high
    verify: trust_startup_replay::capped_trust_clients_complete_without_startup_rejection
  r1_exact_no_challenge_startup_replays_without_a_backend_lease:
    id: R1
    text: "A complete no-challenge AuthenticationOk-to-ReadyForQuery reply is replayed only to a client whose ordered StartupMessage exactly matches the captured startup, and the replay client can issue a simple query without increasing active backend leases."
    kind: functional
    risk: high
    verify: trust_startup_replay::exact_no_challenge_startup_replays_without_a_backend_lease
  r2_startup_mismatch_and_auth_challenges_never_replay:
    id: R2
    text: "A startup parameter mismatch, cleartext-password challenge, MD5 challenge, or SASL challenge never consumes or publishes a cached reply and remains on the fresh passthrough path."
    kind: security
    risk: high
    verify: trust_startup_replay::startup_mismatch_and_auth_challenges_never_replay
---
flowchart TD
    ac1[AC1 ac1 capped trust clients complete without startup rejection] --> trust_startup_replay_capped_trust_clients_complete_without_startup_rejection[trust_startup_replay::capped_trust_clients_complete_without_startup_rejection]
    r1[R1 r1 exact no challenge startup replays without a backend lease] --> trust_startup_replay_exact_no_challenge_startup_replays_without_a_backend_lease[trust_startup_replay::exact_no_challenge_startup_replays_without_a_backend_lease]
    r2[R2 r2 startup mismatch and auth challenges never replay] --> trust_startup_replay_startup_mismatch_and_auth_challenges_never_replay[trust_startup_replay::startup_mismatch_and_auth_challenges_never_replay]
```
