---
id: multi-language-openapi-client-generation-contract
summary: External contract for Multi-Language OpenAPI Client Generation.
fill_sections: [e2e-test]
---

# EC: Multi-Language OpenAPI Client Generation

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: multi-language-openapi-client-generation-contract
    capability_id: multi-language-openapi-client-generation
    claim_id: multi-language-openapi-client-generation-contract
    contract_id: multi-language-openapi-client-generation-contract
    category: behavior
    command: "cargo test -p cclab-openapi-codegen"
    assertions:
      - "Multi-Language OpenAPI Client Generation public Rust API behavior remains covered by the configured library test suite."
      - "The library contract stays usable through its documented README capability surface."
```
