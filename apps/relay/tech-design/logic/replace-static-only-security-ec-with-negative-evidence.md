---
id: '2175'
summary: Align Relay security-hardening with Lumen's SecurityTool contract and replace static-only evidence with executable auth, admission, untrusted peer TLS, Kubernetes posture, and last-known-good rotation journeys.
capability_refs:
  - id: security-hardening
    role: primary
    gap: bearer-auth-token-registry
    claim: bearer-auth-token-registry
    coverage: full
    rationale: Valid and invalid live-rotation evidence proves the shared registry remains usable without restarting Relay.
  - id: security-hardening
    role: primary
    gap: guard-static-runtime-evidence
    claim: guard-static-runtime-evidence
    coverage: full
    rationale: The guard tool contract now attaches meter evidence from the full dynamic Relay security suite rather than relay_core.
  - id: security-hardening
    role: primary
    gap: request-limit-and-malformed-frame-negative-tests
    claim: request-limit-and-malformed-frame-negative-tests
    coverage: full
    rationale: Bearer and subject authorization plus bounded write admission execute as required behavior and negative evidence.
  - id: security-hardening
    role: primary
    gap: network-policy-and-peer-mtls-termination
    claim: network-policy-and-peer-mtls-termination
    coverage: full
    rationale: Required peer mTLS rejects an attacker-CA identity and static Kubernetes assertions close the deployment boundary.
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
    description: Generalize the material fixture so identity and trust authorities can differ, then bind a required-mTLS server to an ephemeral listener and assert its accept seam rejects a client identity signed by a second CA even though that client trusts the real server CA.
  - path: apps/relay/tests/direct_k8s_assets.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: direct_base_is_a_restricted_durable_singleton
    description: Assert non-root/seccomp/capability-drop container posture, read-only projected registry credentials, NetworkPolicy ingress and peer-port boundaries, and the absence of voter HPA in the production composition.
  - path: apps/relay/README.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: Set Type SecurityTool, list behavior/security/stability commands, root the remediation at 2175, and include auth, admission, untrusted peer, K8s, and last-known-good evidence.
  - path: apps/relay/external-contracts/security-hardening/security/security-evidence.md
    action: modify
    section: e2e-test
    impl_mode: hand-written
    description: Add behavior, security, and stability cases; the security case runs every Relay negative suite without filters, and the stability case runs shared reload tests plus Relay adapter and peer-TLS continuity.
  - path: apps/relay/vat.toml
    action: modify
    section: config
    impl_mode: hand-written
    description: Build auth, service_admission, raft_peer_mtls, direct_k8s_assets, and service-auth reload tests, then make guard invoke the fail-closed evidence driver before Meter records the dynamic suites.
  - path: apps/relay/scripts/ec-evidence.sh
    action: create
    section: unit-test
    impl_mode: hand-written
    description: Provide a test-owned outer oracle that requires named tests, rejects zero executed suites, requires exactly one measured performance marker, self-tests its negative paths, and only then records Meter evidence.
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: relay-security-tool-contract-verification
requirements:
  agent_review_and_ec_verification_close:
    id: R6
    text: "AW structurally checks the revised SecurityTool dimensions, independent agent review accepts the digest, generated cases match sources, and every EC command verifies."
    kind: integration
    risk: high
    verify: aw ec check --project relay && aw ec gen --project relay --verify && aw ec verify --project relay
  bearer_and_admission_rejections_execute:
    id: R2
    text: "Missing and invalid tokens return 401, insufficient subject grants and streaming scope return 403, exhausted write admission returns 429 with Retry-After, and probes remain available."
    kind: negative
    risk: high
    verify: bash apps/relay/scripts/ec-evidence.sh security-behavior
  deployment_security_posture_executes:
    id: R3
    text: "The direct and production Kubernetes assets prove restricted containers, persistent state, read-only projected credentials, opt-in NetworkPolicy, and no unsafe voter HPA."
    kind: security
    risk: high
    verify: bash apps/relay/scripts/ec-evidence.sh security-boundaries
  guard_dispatch_includes_dynamic_suite:
    id: R5
    text: "Vat guard scans Relay, the evidence driver requires named tests and non-zero auth, admission, peer-mTLS, K8s, and reload suites, and Meter records the same dynamic surfaces."
    kind: integration
    risk: high
    verify: cd apps/relay && ../../target/debug/vat run guard-security
  rotation_and_peer_stability_execute:
    id: R4
    text: "Shared invalid registry reload retains the last-known-good snapshot, Relay valid rotation is live without restart, and trusted peer replication remains usable alongside the rejection path."
    kind: stability
    risk: high
    verify: bash apps/relay/scripts/ec-evidence.sh security-stability
  untrusted_identity_fails_server_handshake:
    id: R1
    text: "The peer server rejects an attacker-CA client identity even when the client trusts the legitimate server, and the test observes a server-side TLS handshake error before routing."
    kind: negative
    risk: high
    verify: bash apps/relay/scripts/ec-evidence.sh security-boundaries
---
flowchart TD
    r1[R1 untrusted identity fails server handshake] --> bash_apps_relay_scripts_ec_evidence_sh_security_boundaries[bash apps/relay/scripts/ec-evidence.sh security-boundaries]
    r2[R2 bearer and admission rejections execute] --> bash_apps_relay_scripts_ec_evidence_sh_security_behavior[bash apps/relay/scripts/ec-evidence.sh security-behavior]
    r3[R3 deployment security posture executes] --> bash_apps_relay_scripts_ec_evidence_sh_security_boundaries[bash apps/relay/scripts/ec-evidence.sh security-boundaries]
    r4[R4 rotation and peer stability execute] --> bash_apps_relay_scripts_ec_evidence_sh_security_stability[bash apps/relay/scripts/ec-evidence.sh security-stability]
    r5[R5 guard dispatch includes dynamic suite] --> cd_apps_relay_target_debug_vat_run_guard_security[cd apps/relay && ../../target/debug/vat run guard-security]
    r6[R6 agent review and ec verification close] --> aw_ec_check_project_relay_aw_ec_gen_project_relay_verify_aw_ec_verify_project_relay[aw ec check --project relay && aw ec gen --project relay --verify && aw ec verify --project relay]
```
