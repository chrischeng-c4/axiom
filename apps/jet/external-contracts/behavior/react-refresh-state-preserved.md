---
id: react-refresh-state-preserved
summary: External contract for React Refresh State Preserved.
fill_sections: [e2e-test]
---

# EC: React Refresh State Preserved

---
id: react-refresh-state-preserved
summary: External contract for React Refresh State Preserved.
fill_sections: [e2e-test]
---

# EC: React Refresh State Preserved

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: react-refresh-state-preserved
    capability_id: dev-server-hmr
    claim_id: react-refresh-state-preserved
    contract_id: react-refresh-state-preserved
    category: behavior
    command: "cargo test -p jet --lib dev_server::hmr -- --nocapture"
    assertions:
      - "React refresh chooses a hot update that preserves state when the edit is compatible."
```
