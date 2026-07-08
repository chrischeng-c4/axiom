---
id: renderer-neutral-ui-surface-model-contract
summary: External contract for Renderer-Neutral UI Surface Model.
fill_sections: [e2e-test]
---

# EC: Renderer-Neutral UI Surface Model

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: renderer-neutral-ui-surface-model-contract
    capability_id: renderer-neutral-ui-surface-model
    claim_id: renderer-neutral-ui-surface-model-contract
    contract_id: renderer-neutral-ui-surface-model-contract
    category: behavior
    command: "cargo test -p cclab-surface"
    assertions:
      - "Renderer-Neutral UI Surface Model public Rust API behavior remains covered by the configured library test suite."
      - "The library contract stays usable through its documented README capability surface."
```
