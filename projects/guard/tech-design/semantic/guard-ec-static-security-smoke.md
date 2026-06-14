---
id: guard-ec-static-security-smoke
summary: External-contract smoke evidence for guard's static security report.
capability_refs:
  - id: static-security-scan
    role: primary
    gap: json-report-envelope
    claim: json-report-envelope
    coverage: full
    rationale: "The EC case proves guard emits a clean guard.report/1 envelope for the project source tree."
  - id: security-policy-profile
    role: contributes
    gap: baseline-static-policy
    claim: baseline-static-policy
    coverage: full
    rationale: "The EC case proves the baseline static policy maps compass output into guard status and summary fields."
  - id: security-ec-profile
    role: primary
    gap: aw-health-security-metric
    claim: aw-health-security-metric
    coverage: full
    rationale: "The EC case is the AW health security metric that consumes guard.report/1 as first-class security evidence."
  - id: security-ec-profile
    role: primary
    gap: ec-security-evidence-command
    claim: ec-security-evidence-command
    coverage: full
    rationale: "The EC case defines the executable guard security evidence command used by AW health."
fill_sections: [e2e-test]
---

# Guard Static Security EC

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: guard-static-scan-clean-report
    capability_id: static-security-scan
    contract_id: guard-report-clean-static-scan
    category: security
    command: "target/debug/guard scan projects/guard --compact --no-persist"
    assertions:
      - command exits zero
      - stdout is a guard.report/1 JSON envelope
      - summary.security_findings is zero for the guard source tree
      - integrations.static_engine is compass
```
