---
id: reporter-artifacts
summary: External contract for Reporter Artifacts.
fill_sections: [e2e-test]
---

# EC: Reporter Artifacts

---
id: reporter-artifacts
summary: External contract for Reporter Artifacts.
fill_sections: [e2e-test]
---

# EC: Reporter Artifacts

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: reporter-artifacts
    capability_id: native-test-product-flow-e2e
    claim_id: reporter-artifacts
    contract_id: reporter-artifacts
    category: behavior
    command: "cargo test -p jet --lib reporter -- --nocapture"
    assertions:
      - "Jet reporter emits the expected test-result and artifact records for completed runs."
```
