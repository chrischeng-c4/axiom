---
id: production-bundle-output-parity
summary: External contract for Production Bundle Output Parity.
fill_sections: [e2e-test]
---

# EC: Production Bundle Output Parity

---
id: production-bundle-output-parity
summary: External contract for Production Bundle Output Parity.
fill_sections: [e2e-test]
---

# EC: Production Bundle Output Parity

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: production-bundle-output-parity
    capability_id: bundler-production-build
    claim_id: production-bundle-output-parity
    contract_id: production-bundle-output-parity
    category: behavior
    command: "cargo test -p jet --lib bundler -- --nocapture"
    assertions:
      - "Jet production bundler emits parseable output with required module resolution behavior."
```
