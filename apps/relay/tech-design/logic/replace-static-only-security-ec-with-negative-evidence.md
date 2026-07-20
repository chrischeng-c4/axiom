---
id: '2175'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-security-negative-evidence-applicability
entry: rejected
nodes:
  rejected:
    kind: start
    label: "Independent review rejects static-only Relay security evidence"
  shared:
    kind: decision
    label: "Is a shared security mechanism missing?"
  shared_wi:
    kind: terminal
    label: "Stop and route a separate lib WI"
  classify:
    kind: process
    label: "Align capability with Lumen SecurityTool dimensions"
  negative:
    kind: process
    label: "Exercise Relay auth admission peer TLS and K8s rejection posture"
  stability:
    kind: process
    label: "Exercise last-known-good rotation and trusted peer continuity"
  done:
    kind: terminal
    label: "Behavior security and stability evidence fail closed"
edges:
  - { from: rejected, to: shared }
  - { from: shared, to: shared_wi, label: "yes" }
  - { from: shared, to: classify, label: "no" }
  - { from: classify, to: negative }
  - { from: negative, to: stability }
  - { from: stability, to: done }
---
flowchart TD
    rejected[static-only EC rejected] --> shared{shared mechanism missing?}
    shared -->|yes| shared_wi[separate lib WI]
    shared -->|no| classify[SecurityTool contract]
    classify --> negative[Relay negative journeys]
    negative --> stability[rotation and peer stability]
    stability --> done[fail-closed evidence]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/relay/tests/raft_peer_mtls.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: trusted_relay_peers_replicate_messages_over_mtls
    description: Add a direct required-mTLS accept/connect journey whose client trusts the server CA but presents an identity signed by an untrusted CA; assert the server rejects it before HTTP/Raft handling.
  - path: apps/relay/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: Align security-hardening with Lumen's SecurityTool classification and declare behavior, security, and stability dimensions.
  - path: apps/relay/external-contracts/security-hardening/security/security-evidence.md
    action: modify
    section: e2e-test
    impl_mode: hand-written
    description: Replace the advisory static-only case with executable behavior, negative-security, and last-known-good stability journeys while retaining guard as the static tool owner.
  - path: apps/relay/vat.toml
    action: modify
    section: config
    impl_mode: hand-written
    description: Make guard-security attach meter evidence from auth, admission, peer-mTLS, and direct K8s tests rather than relay_core.
```
