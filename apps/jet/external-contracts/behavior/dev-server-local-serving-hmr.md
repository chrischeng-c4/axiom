---
id: dev-server-local-serving-hmr
summary: External contract for Dev Server Local Serving Hmr.
fill_sections: [e2e-test]
---

# EC: Dev Server Local Serving Hmr

---
id: dev-server-local-serving-hmr
summary: External contract for Dev Server Local Serving Hmr.
fill_sections: [e2e-test]
---

# EC: Dev Server Local Serving Hmr

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: dev-server-local-serving-hmr
    capability_id: dev-server-hmr
    claim_id: dev-server-local-serving-hmr
    contract_id: dev-server-local-serving-hmr
    category: behavior
    command: "cargo test -p jet --lib dev_server -- --nocapture"
    assertions:
      - "Jet dev server serves local modules and applies hot updates without requiring a full restart."
```
