---
id: http2-cleartext-client-helpers-contract
summary: External contract for HTTP/2 Cleartext Client Helpers.
fill_sections: [e2e-test]
---

# EC: HTTP/2 Cleartext Client Helpers

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: http2-cleartext-client-helpers-contract
    capability_id: http2-cleartext-client-helpers
    claim_id: http2-cleartext-client-helpers-contract
    contract_id: http2-cleartext-client-helpers-contract
    category: behavior
    command: "cargo test -p h2c"
    assertions:
      - "HTTP/2 Cleartext Client Helpers public Rust API behavior remains covered by the configured library test suite."
      - "The library contract stays usable through its documented README capability surface."
```
