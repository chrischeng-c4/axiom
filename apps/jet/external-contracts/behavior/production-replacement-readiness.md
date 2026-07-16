---
id: production-replacement-readiness
summary: External contract for Production Replacement Readiness.
fill_sections: [e2e-test]
---

# EC: Production Replacement Readiness

---
id: production-replacement-readiness
summary: External contract for Production Replacement Readiness.
fill_sections: [e2e-test]
---

# EC: Production Replacement Readiness

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: production-replacement-readiness
    capability_id: rust-native-frontend-toolchain
    claim_id: production-replacement-readiness
    contract_id: production-replacement-readiness
    category: behavior
    command: "apps/jet/scripts/verify-basic-dom-gates.sh --all"
    assertions:
      - "The full production replacement flow passes its required end-to-end gates."
```
