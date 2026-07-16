---
id: built-in-ts-test-runtime
summary: External contract for Built In Ts Test Runtime.
fill_sections: [e2e-test]
---

# EC: Built In Ts Test Runtime

---
id: built-in-ts-test-runtime
summary: External contract for Built In Ts Test Runtime.
fill_sections: [e2e-test]
---

# EC: Built In Ts Test Runtime

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: built-in-ts-test-runtime
    capability_id: native-test-product-flow-e2e
    claim_id: built-in-ts-test-runtime
    contract_id: built-in-ts-test-runtime
    category: behavior
    command: "cargo test -p jet --lib test_runner -- --nocapture"
    assertions:
      - "Jet built-in TypeScript test runtime executes supported TypeScript tests with expected results."
```
