---
id: product-flow-e2e-readiness
summary: External contract for Product Flow E2e Readiness.
fill_sections: [e2e-test]
---

# EC: Product Flow E2e Readiness

---
id: product-flow-e2e-readiness
summary: External contract for Product Flow E2e Readiness.
fill_sections: [e2e-test]
---

# EC: Product Flow E2e Readiness

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: product-flow-e2e-readiness
    capability_id: native-test-product-flow-e2e
    claim_id: product-flow-e2e-readiness
    contract_id: product-flow-e2e-readiness
    category: behavior
    command: "cargo test -p jet --lib e2e -- --nocapture"
    assertions:
      - "Jet native product-flow end-to-end scenarios execute through the supported user journey."
```
