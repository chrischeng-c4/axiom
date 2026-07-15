---
id: stories-controls-panel
summary: External contract for Stories Controls Panel.
fill_sections: [e2e-test]
---

# EC: Stories Controls Panel

---
id: stories-controls-panel
summary: External contract for Stories Controls Panel.
fill_sections: [e2e-test]
---

# EC: Stories Controls Panel

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: stories-controls-panel
    capability_id: component-workbench
    claim_id: stories-controls-panel
    contract_id: stories-controls-panel
    category: behavior
    command: "cargo test -p jet --test controls -- --nocapture"
    assertions:
      - "Stories controls render prop-derived inputs and update the preview when args change."
```
