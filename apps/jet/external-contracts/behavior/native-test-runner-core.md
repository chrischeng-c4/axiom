---
id: native-test-runner-core
summary: External contract for Native Test Runner Core.
fill_sections: [e2e-test]
---

# EC: Native Test Runner Core

---
id: native-test-runner-core
summary: External contract for Native Test Runner Core.
fill_sections: [e2e-test]
---

# EC: Native Test Runner Core

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: native-test-runner-core
    capability_id: native-test-product-flow-e2e
    claim_id: native-test-runner-core
    contract_id: native-test-runner-core
    category: behavior
    command: "cargo test -p jet --lib test_runner -- --nocapture"
    assertions:
      - "Jet native test runner discovers, executes, and reports supported test cases."
```
