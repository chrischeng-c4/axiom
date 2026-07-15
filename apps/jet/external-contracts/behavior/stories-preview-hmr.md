---
id: stories-preview-hmr
summary: External contract for Stories Preview Hmr.
fill_sections: [e2e-test]
---

# EC: Stories Preview Hmr

---
id: stories-preview-hmr
summary: External contract for Stories Preview Hmr.
fill_sections: [e2e-test]
---

# EC: Stories Preview Hmr

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: stories-preview-hmr
    capability_id: component-workbench
    claim_id: stories-preview-hmr
    contract_id: stories-preview-hmr
    category: behavior
    command: "cargo test -p jet --test preview_hmr -- --nocapture"
    assertions:
      - "Stories preview HMR refreshes modules while preserving compatible component state."
```
