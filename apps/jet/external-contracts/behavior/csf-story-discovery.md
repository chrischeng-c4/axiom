---
id: csf-story-discovery
summary: External contract for Csf Story Discovery.
fill_sections: [e2e-test]
---

# EC: Csf Story Discovery

---
id: csf-story-discovery
summary: External contract for Csf Story Discovery.
fill_sections: [e2e-test]
---

# EC: Csf Story Discovery

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: csf-story-discovery
    capability_id: component-workbench
    claim_id: csf-story-discovery
    contract_id: csf-story-discovery
    category: behavior
    command: "cargo test -p jet --test csf_discovery -- --nocapture"
    assertions:
      - "CSF story modules are discovered and parsed into a usable story index."
```
