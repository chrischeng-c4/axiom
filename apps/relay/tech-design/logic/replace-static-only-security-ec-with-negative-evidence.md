---
id: '2175'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-security-negative-evidence-contract
entry: request
nodes:
  request:
    kind: start
    label: "Relay security evidence command starts"
  behavior:
    kind: process
    label: "Behavior runs bearer RBAC streaming authz and admission limits"
  client:
    kind: process
    label: "Untrusted client trusts server CA but presents attacker-CA identity"
  handshake:
    kind: decision
    label: "Required-mTLS server accepts client certificate"
  reject:
    kind: process
    label: "Server handshake rejects before HTTP and Raft routing"
  k8s:
    kind: process
    label: "Security case asserts restricted pods Secret projection and NetworkPolicy"
  stability:
    kind: process
    label: "Stability keeps last-known-good auth on invalid rotation and trusted peers usable"
  guard:
    kind: process
    label: "Vat guard scan attaches non-zero meter evidence from dynamic cases"
  fail:
    kind: terminal
    label: "FAIL if untrusted peer is accepted or any journey is missing"
  pass:
    kind: terminal
    label: "PASS behavior security and stability dimensions"
edges:
  - { from: request, to: behavior }
  - { from: behavior, to: client }
  - { from: client, to: handshake }
  - { from: handshake, to: fail, label: "accepted" }
  - { from: handshake, to: reject, label: "rejected" }
  - { from: reject, to: k8s }
  - { from: k8s, to: stability }
  - { from: stability, to: guard }
  - { from: guard, to: pass }
---
flowchart TD
    request[security evidence] --> behavior[auth RBAC and admission]
    behavior --> client[attacker identity trusts server]
    client --> handshake{server accepts attacker cert?}
    handshake -->|yes| fail[FAIL]
    handshake -->|no| reject[reject before routing]
    reject --> k8s[restricted K8s posture]
    k8s --> stability[last-known-good rotation]
    stability --> guard[vat guard plus dynamic meter]
    guard --> pass[PASS dimensions]
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

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-security-negative-evidence-verification
requirements:
  runtime_security_journeys_pass:
    id: R3
    text: "Bearer 401/403, subject RBAC, streaming authz, write admission, trusted and untrusted peer TLS, restricted pod, Secret projection, and NetworkPolicy evidence execute together."
    kind: integration
    risk: high
    verify: cargo test -p relay --test auth --test service_admission --test raft_peer_mtls --test direct_k8s_assets -- --nocapture
  security_tool_dimensions_are_complete:
    id: R1
    text: "Relay security-hardening matches the Lumen SecurityTool baseline and supplies executable behavior, security, and stability cases."
    kind: functional
    risk: high
    verify: aw ec check --project relay
  untrusted_peer_is_rejected:
    id: R2
    text: "A client that trusts Relay's server CA but presents a certificate signed by an untrusted CA is rejected by the required-mTLS acceptor before any Raft request is handled."
    kind: negative
    risk: high
    verify: cargo test -p relay --test raft_peer_mtls untrusted_relay_peer_certificate_is_rejected -- --exact --nocapture
  vat_guard_attaches_dynamic_evidence:
    id: R4
    text: "The vat-isolated guard-security runner performs the static scan and attaches meter evidence from Relay's dynamic security suite, failing if those tests are missing or fail."
    kind: integration
    risk: high
    verify: cd apps/relay && ../../target/debug/vat run guard-security
---
flowchart TD
    r1[R1 security tool dimensions are complete] --> aw_ec_check_project_relay[aw ec check --project relay]
    r2[R2 untrusted peer is rejected] --> cargo_test_p_relay_test_raft_peer_mtls_untrusted_relay_peer_certificate_is_rejected_exact_nocapture[cargo test -p relay --test raft_peer_mtls untrusted_relay_peer_certificate_is_rejected -- --exact --nocapture]
    r3[R3 runtime security journeys pass] --> cargo_test_p_relay_test_auth_test_service_admission_test_raft_peer_mtls_test_direct_k8s_assets_nocapture[cargo test -p relay --test auth --test service_admission --test raft_peer_mtls --test direct_k8s_assets -- --nocapture]
    r4[R4 vat guard attaches dynamic evidence] --> cd_apps_relay_target_debug_vat_run_guard_security[cd apps/relay && ../../target/debug/vat run guard-security]
```
