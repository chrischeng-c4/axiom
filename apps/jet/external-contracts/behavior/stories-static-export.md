---
id: stories-static-export
summary: External contract for Stories Static Export.
fill_sections: [e2e-test]
---

# EC: Stories Static Export

---
id: stories-static-export
summary: External contract for Stories Static Export.
fill_sections: [e2e-test]
---

# EC: Stories Static Export

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: stories-static-export
    capability_id: component-workbench
    claim_id: stories-static-export
    contract_id: stories-static-export
    category: behavior
    command: "cargo test -p jet --test stories_build -- --nocapture"
    assertions:
      - "Jet stories build emits a static workbench with manager, previews, and relative asset URLs."
```
