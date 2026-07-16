---
id: product-flow-e2e-review
summary: External contract for Product Flow E2e Review.
fill_sections: [e2e-test]
---

# EC: Product Flow E2e Review

---
id: product-flow-e2e-review
summary: External contract for Product Flow E2e Review.
fill_sections: [e2e-test]
---

# EC: Product Flow E2e Review

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: product-flow-e2e-review
    capability_id: native-test-product-flow-e2e
    claim_id: product-flow-e2e-review
    contract_id: product-flow-e2e-review
    category: behavior
    command: "cargo test -p jet --lib e2e -- --nocapture"
    assertions:
      - "Product-flow evidence exposes the required reviewable end-to-end outcome."
```
