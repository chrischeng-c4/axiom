---
id: shared-http-service-scaffold-contract
summary: External contract for Shared HTTP Service Scaffold.
fill_sections: [e2e-test]
---

# EC: Shared HTTP Service Scaffold

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: shared-http-service-scaffold-contract
    capability_id: shared-http-service-scaffold
    claim_id: shared-http-service-scaffold-contract
    contract_id: shared-http-service-scaffold-contract
    category: behavior
    command: "cargo test -p service-http"
    assertions:
      - "Shared HTTP Service Scaffold public Rust API behavior remains covered by the configured library test suite."
      - "The library contract stays usable through its documented README capability surface."
```
