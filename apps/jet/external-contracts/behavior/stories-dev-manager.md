---
id: stories-dev-manager
summary: External contract for Stories Dev Manager.
fill_sections: [e2e-test]
---

# EC: Stories Dev Manager

---
id: stories-dev-manager
summary: External contract for Stories Dev Manager.
fill_sections: [e2e-test]
---

# EC: Stories Dev Manager

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: stories-dev-manager
    capability_id: component-workbench
    claim_id: stories-dev-manager
    contract_id: stories-dev-manager
    category: behavior
    command: "cargo test -p jet --test manager -- --nocapture"
    assertions:
      - "Jet stories dev starts a manager with sidebar and preview behavior for discovered stories."
```
