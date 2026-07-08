---
id: shared-http-request-auth-middleware-contract
summary: External contract for Shared HTTP Request Auth Middleware.
fill_sections: [e2e-test]
---

# EC: Shared HTTP Request Auth Middleware

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: shared-http-request-auth-middleware-contract
    capability_id: shared-http-request-auth-middleware
    claim_id: shared-http-request-auth-middleware-contract
    contract_id: shared-http-request-auth-middleware-contract
    category: behavior
    command: "cargo test -p service-auth"
    assertions:
      - "Shared HTTP Request Auth Middleware public Rust API behavior remains covered by the configured library test suite."
      - "The library contract stays usable through its documented README capability surface."
```
