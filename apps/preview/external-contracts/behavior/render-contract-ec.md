---
id: render-contract-ec
summary: External contract for Render contract EC.
fill_sections: [e2e-test]
---

# EC: Render contract EC

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: render-contract-ec
    capability_id: preview-external-contracts
    claim_id: render-contract-ec
    contract_id: render-contract-ec
    category: behavior
    command: "cargo test -p preview --test render_contract"
    assertions:
      - "Render contract tests cover generated file names, base workload clone plan, namespace naming, route binding, and cleanup protected namespace output."
      - "The render EC remains runnable without a live cluster."
```
