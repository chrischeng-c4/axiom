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
