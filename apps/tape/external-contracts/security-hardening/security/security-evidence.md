---
id: tape-security-hardening-ec
summary: Tape's security gate is the guard report, vat-isolated, with meter evidence attached for the bearer-auth request-boundary smoke.
fill_sections: [e2e-test, tool-contract]
---

# EC: Security Hardening

Tape's security gate is the guard report over `apps/tape`: static
Docker/Kubernetes/API findings must be clean, and guard attaches meter
evidence from tape's `cli_contract` auth-boundary smoke so security
regressions cannot be marked ready without runtime evidence. The gate runs
inside vat so generated reports and transient files never mutate the host
checkout.

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: tape-security-hardening-guard-scan
    capability_id: security-hardening
    claim_id: guard-static-runtime-evidence
    contract_id: tape-guard-security-report
    category: security
    test_path: apps/tape/tests/security_tape_security_hardening_guard_scan.rs
    command: "cd apps/tape && ../../target/debug/vat run guard-security"
    required_for_production: false
    assertions:
      - "guard scan over apps/tape reports no untriaged Docker, Kubernetes, or static security findings."
      - "guard attaches meter evidence for tape's bearer-auth request-boundary smoke (cli_contract auth-relevant cases)."
      - "The security evidence runs inside vat so generated reports and transient files do not mutate the host checkout."
```

## Tool Contract
<!-- type: tool-contract lang: yaml -->

```yaml
tool_contracts:
  - id: tape-guard-security
    tool: guard
    manifest: guard-tape-security.toml
    category: security
    command: "cd apps/tape && ../../target/debug/vat run guard-security"
    native:
      version: 1
      project: tape
      source_contract: tape-security-hardening-guard-scan
      delegate_command: "cd apps/tape && ../../target/debug/vat run guard-security"
```
