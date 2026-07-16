---
id: stories-bare-import-resolution
summary: External contract for Stories Bare Import Resolution.
fill_sections: [e2e-test]
---

# EC: Stories Bare Import Resolution

---
id: stories-bare-import-resolution
summary: External contract for Stories Bare Import Resolution.
fill_sections: [e2e-test]
---

# EC: Stories Bare Import Resolution

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: stories-bare-import-resolution
    capability_id: component-workbench
    claim_id: stories-bare-import-resolution
    contract_id: stories-bare-import-resolution
    category: behavior
    command: "cargo test -p jet --test manager -- --nocapture"
    assertions:
      - "Stories manager resolves bare imports from node_modules for development and static export."
```
