---
id: csf2-template-bind-re-exports
summary: External contract for Csf2 Template Bind Re Exports.
fill_sections: [e2e-test]
---

# EC: Csf2 Template Bind Re Exports

---
id: csf2-template-bind-re-exports
summary: External contract for Csf2 Template Bind Re Exports.
fill_sections: [e2e-test]
---

# EC: Csf2 Template Bind Re Exports

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: csf2-template-bind-re-exports
    capability_id: component-workbench
    claim_id: csf2-template-bind-re-exports
    contract_id: csf2-template-bind-re-exports
    category: behavior
    command: "cargo test -p jet --test csf_discovery -- --nocapture"
    assertions:
      - "CSF2 Template.bind stories and re-exported stories remain discoverable with their bound args."
```
