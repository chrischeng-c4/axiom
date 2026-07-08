---
id: shared-peer-mtls-material-loading-contract
summary: External contract for Shared Peer mTLS Material Loading.
fill_sections: [e2e-test]
---

# EC: Shared Peer mTLS Material Loading

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: shared-peer-mtls-material-loading-contract
    capability_id: shared-peer-mtls-material-loading
    claim_id: shared-peer-mtls-material-loading-contract
    contract_id: shared-peer-mtls-material-loading-contract
    category: behavior
    command: "cargo test -p service-tls"
    assertions:
      - "Shared Peer mTLS Material Loading public Rust API behavior remains covered by the configured library test suite."
      - "The library contract stays usable through its documented README capability surface."
```
