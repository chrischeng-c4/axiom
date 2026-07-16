---
id: transform-resolver-parity
summary: External contract for Transform Resolver Parity.
fill_sections: [e2e-test]
---

# EC: Transform Resolver Parity

---
id: transform-resolver-parity
summary: External contract for Transform Resolver Parity.
fill_sections: [e2e-test]
---

# EC: Transform Resolver Parity

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: transform-resolver-parity
    capability_id: bundler-production-build
    claim_id: transform-resolver-parity
    contract_id: transform-resolver-parity
    category: behavior
    command: "cargo test -p jet --lib transform -- --nocapture"
    assertions:
      - "Jet transform resolver resolves supported source and package imports consistently."
```
