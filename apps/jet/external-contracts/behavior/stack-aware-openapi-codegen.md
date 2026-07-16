---
id: stack-aware-openapi-codegen
summary: External contract for Stack Aware Openapi Codegen.
fill_sections: [e2e-test]
---

# EC: Stack Aware Openapi Codegen

---
id: stack-aware-openapi-codegen
summary: External contract for Stack Aware Openapi Codegen.
fill_sections: [e2e-test]
---

# EC: Stack Aware Openapi Codegen

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: stack-aware-openapi-codegen
    capability_id: rust-native-frontend-toolchain
    claim_id: stack-aware-openapi-codegen
    contract_id: stack-aware-openapi-codegen
    category: behavior
    command: "cargo test -p jet --test openapi_golden"
    assertions:
      - "Jet OpenAPI code generation selects the correct stack-aware client and hook output."
```
