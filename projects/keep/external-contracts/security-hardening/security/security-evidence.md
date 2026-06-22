---
id: keep-security-hardening-ec
summary: Keep security evidence is guard-owned and vat-isolated, with meter evidence attached for public-route smoke.
fill_sections: [e2e-test, tool-contract]
---

# EC: Security Hardening

Keep's security gate is the guard report over the HTTP/Kubernetes/container
surface. guard attaches meter evidence for public-route smoke so request-boundary
security cannot be marked complete without runtime proof.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: keep-security-hardening-guard-scan
    capability_id: security-hardening
    claim_id: guard-static-runtime-evidence
    contract_id: keep-guard-security-report
    category: security
    test_path: projects/keep/tests/security_keep_security_hardening_guard_scan.rs
    command: "cd projects/keep && ../../target/debug/vat run guard-security"
    required_for_production: false
    assertions:
      - "guard scan over keep reports no untriaged Docker, Kubernetes, or static security findings."
      - "guard attaches meter evidence for Keep's public HTTP route smoke."
      - "The security evidence runs inside vat so generated reports and transient files do not mutate the host checkout."
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: keep-guard-security
    tool: guard
    manifest: guard-keep-security.toml
    category: security
    command: "cd projects/keep && ../../target/debug/vat run guard-security"
    native:
      version: 1
      project: keep
      source_contract: keep-security-hardening-guard-scan
      delegate_command: "cd projects/keep && ../../target/debug/vat run guard-security"
```
