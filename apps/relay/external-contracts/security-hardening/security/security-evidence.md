---
id: relay-security-hardening-ec
summary: Relay security evidence combines executable auth/admission behavior, untrusted peer and restricted Kubernetes boundaries, last-known-good rotation stability, and vat-isolated guard plus dynamic meter proof.
fill_sections: [e2e-test, tool-contract]
---

# EC: Security Hardening

Relay's SecurityTool contract separates behavior, security, and stability.
Bearer/RBAC and admission behavior run over the real service paths; the
security cases require attacker-CA rejection and restricted Kubernetes posture;
the stability case keeps last-known-good credentials and trusted peer
replication usable. Guard owns the static scan and, inside vat, attaches meter
evidence from the complete unfiltered dynamic security test binaries rather
than a generic broker-core smoke.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: relay-security-hardening-auth-and-admission-behavior
    capability_id: security-hardening
    claim_id: request-limit-and-malformed-frame-negative-tests
    contract_id: relay-auth-rbac-and-admission-behavior
    category: behavior
    command: "bash apps/relay/scripts/ec-evidence.sh security-behavior"
    assertions:
      - "Required auth returns 401 for missing or unknown bearer tokens and 403 when a reader attempts publish or a subject-scoped reader crosses its grant; the streaming consume route enforces the same boundary."
      - "Valid subject writers/readers and wildcard administrators retain their intended publish, lease, ack, heartbeat, and batch behavior, while health, readiness, metrics, OpenAPI, and docs remain tokenless."
      - "A configured one-write admission budget allows the first publish, rejects the second with 429 and Retry-After: 60, and never rate-limits health probes."
      - "The outer oracle requires all eight auth and two admission test names and independently rejects either suite when its executed count is zero."

  - id: relay-security-hardening-negative-peer-and-kubernetes-boundaries
    capability_id: security-hardening
    claim_id: network-policy-and-peer-mtls-termination
    contract_id: relay-untrusted-peer-and-restricted-workload-security
    category: security
    command: "bash apps/relay/scripts/ec-evidence.sh security-boundaries"
    assertions:
      - "A client that trusts Relay's legitimate server CA but presents an identity signed by an attacker CA is rejected by the required-mTLS server handshake before HTTP or Raft routing."
      - "Peers signed by the trusted CA still elect, replicate, and converge over the authenticated listener."
      - "The direct StatefulSet is non-root with a read-only root filesystem and durable PVC; the production overlay projects credentials read-only, enables NetworkPolicy and observability components, and does not apply an unsafe voter HPA."
      - "The outer oracle requires both named peer-mTLS tests and both named Kubernetes tests and rejects a zero-test result from either binary."

  - id: relay-security-hardening-guard-scan
    capability_id: security-hardening
    claim_id: guard-static-runtime-evidence
    contract_id: relay-guard-security-report
    category: security
    test_path: apps/relay/tests/security_relay_security_hardening_guard_scan.rs
    command: "cd apps/relay && ../../target/debug/vat run guard-security"
    assertions:
      - "guard scan over apps/relay reports no untriaged Docker, Kubernetes, or static security findings."
      - "guard runs the fail-closed evidence driver before attaching Meter evidence from auth, admission, peer-mTLS, direct-Kubernetes, and service-auth reload suites; missing required names, zero execution, a failed control, or an outer-oracle self-test regression makes the runner fail."
      - "The security evidence runs inside vat so generated reports and transient files do not mutate the host checkout."

  - id: relay-security-hardening-rotation-and-peer-stability
    capability_id: security-hardening
    claim_id: bearer-auth-token-registry
    contract_id: relay-security-last-known-good-stability
    category: stability
    command: "bash apps/relay/scripts/ec-evidence.sh security-stability"
    assertions:
      - "A valid registry rotation becomes visible without restarting Relay, invalid JSON/empty/read-failed rotations retain the last-known-good registry, and failure audit classes remain credential-free."
      - "The trusted three-peer required-mTLS group remains able to elect, replicate, converge, and shut down after the negative certificate-rejection case is present."
      - "The outer oracle requires all five reload tests plus the exact Relay rotation and trusted-peer tests, and each focused invocation must execute at least one test."
```
## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: relay-guard-security
    tool: guard
    manifest: guard-relay-security.toml
    category: security
    command: "cd apps/relay && ../../target/debug/vat run guard-security"
    native:
      version: 1
      project: relay
      source_contract: relay-security-hardening-guard-scan
      delegate_command: "cd apps/relay && ../../target/debug/vat run guard-security"
```
