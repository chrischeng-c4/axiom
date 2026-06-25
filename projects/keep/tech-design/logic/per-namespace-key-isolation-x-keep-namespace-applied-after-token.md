---
id: per-namespace-key-isolation-x-keep-namespace-applied-after-token
capability_refs:
  - id: "security-hardening"
    role: primary
    gap: "auth-tls-network-policy-boundary"
    claim: "auth-tls-network-policy-boundary"
    coverage: partial
    rationale: "Adds per-namespace storage-key isolation on the claim-check data plane (X-Keep-Namespace applied after the token scope check), one access-boundary dimension toward the security-hardening negative gates."
fill_sections: [unit-test, changes]
---

# Keep Per-Namespace Key Isolation
