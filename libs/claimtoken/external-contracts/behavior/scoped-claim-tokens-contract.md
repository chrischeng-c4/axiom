---
id: scoped-claim-tokens-contract
summary: External contract for Scoped Claim Tokens.
fill_sections: [e2e-test]
---

# EC: Scoped Claim Tokens

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: scoped-claim-tokens-contract
    capability_id: scoped-claim-tokens
    claim_id: scoped-claim-tokens-contract
    contract_id: scoped-claim-tokens-contract
    category: behavior
    command: "cargo test -p claimtoken"
    assertions:
      - "Scoped Claim Tokens public Rust API behavior remains covered by the configured library test suite."
      - "The library contract stays usable through its documented README capability surface."
```
