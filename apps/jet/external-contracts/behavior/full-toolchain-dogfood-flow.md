---
id: full-toolchain-dogfood-flow
summary: External contract for Full Toolchain Dogfood Flow.
fill_sections: [e2e-test]
---

# EC: Full Toolchain Dogfood Flow

---
id: full-toolchain-dogfood-flow
summary: External contract for Full Toolchain Dogfood Flow.
fill_sections: [e2e-test]
---

# EC: Full Toolchain Dogfood Flow

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: full-toolchain-dogfood-flow
    capability_id: rust-native-frontend-toolchain
    claim_id: full-toolchain-dogfood-flow
    contract_id: full-toolchain-dogfood-flow
    category: behavior
    command: "apps/jet/scripts/verify-basic-dom-gates.sh --all"
    assertions:
      - "The end-to-end Jet toolchain dogfood flow passes all required package, build, browser, and production gates."
```
