---
id: relay-security-hardening-ec
summary: Relay security evidence is guard-owned and vat-isolated, with meter evidence attached for request-boundary regression checks.
fill_sections: [e2e-test, tool-contract]
---

# EC: Security Hardening

Relay's security gate is the guard report. Static Docker/Kubernetes/API findings
must be clean, and guard attaches meter evidence for the in-process opaque
payload boundary so security regressions cannot be marked ready without runtime
evidence. HTTP worker-loop tests bind localhost and remain behavior gates
outside the vat-isolated guard evidence runner.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: relay-security-hardening-guard-scan
    capability_id: security-hardening
    claim_id: guard-static-runtime-evidence
    contract_id: relay-guard-security-report
    category: security
    test_path: projects/relay/tests/security_relay_security_hardening_guard_scan.rs
    command: "cd projects/relay && ../../target/debug/vat run guard-security"
    required_for_production: false
    assertions:
      - "guard scan over relay reports no untriaged Docker, Kubernetes, or static security findings."
      - "guard attaches meter evidence for relay_core opaque-payload request-boundary smoke."
      - "The security evidence runs inside vat so generated reports and transient files do not mutate the host checkout."
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: relay-guard-security
    tool: guard
    manifest: guard-relay-security.toml
    category: security
    command: "cd projects/relay && ../../target/debug/vat run guard-security"
    native:
      version: 1
      project: relay
      source_contract: relay-security-hardening-guard-scan
      delegate_command: "cd projects/relay && ../../target/debug/vat run guard-security"
```
