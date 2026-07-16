---
id: trace-evidence-artifacts
summary: External contract for Trace Evidence Artifacts.
fill_sections: [e2e-test]
---

# EC: Trace Evidence Artifacts

---
id: trace-evidence-artifacts
summary: External contract for Trace Evidence Artifacts.
fill_sections: [e2e-test]
---

# EC: Trace Evidence Artifacts

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: trace-evidence-artifacts
    capability_id: browser-trace-parity
    claim_id: trace-evidence-artifacts
    contract_id: trace-evidence-artifacts
    category: behavior
    command: "cargo test -p jet --lib trace -- --nocapture"
    assertions:
      - "Jet trace subsystem emits required trace evidence artifacts for supported runs."
```
