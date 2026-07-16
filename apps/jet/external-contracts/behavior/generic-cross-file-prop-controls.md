---
id: generic-cross-file-prop-controls
summary: External contract for Generic Cross File Prop Controls.
fill_sections: [e2e-test]
---

# EC: Generic Cross File Prop Controls

---
id: generic-cross-file-prop-controls
summary: External contract for Generic Cross File Prop Controls.
fill_sections: [e2e-test]
---

# EC: Generic Cross File Prop Controls

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: generic-cross-file-prop-controls
    capability_id: component-workbench
    claim_id: generic-cross-file-prop-controls
    contract_id: generic-cross-file-prop-controls
    category: behavior
    command: "cargo test -p jet --test controls -- --nocapture"
    assertions:
      - "Stories controls derive editable values for generic, cross-file, and intersection prop types."
```
