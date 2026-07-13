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
