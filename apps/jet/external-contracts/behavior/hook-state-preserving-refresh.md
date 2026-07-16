---
id: hook-state-preserving-refresh
summary: External contract for Hook State Preserving Refresh.
fill_sections: [e2e-test]
---

# EC: Hook State Preserving Refresh

---
id: hook-state-preserving-refresh
summary: External contract for Hook State Preserving Refresh.
fill_sections: [e2e-test]
---

# EC: Hook State Preserving Refresh

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: hook-state-preserving-refresh
    capability_id: component-workbench
    claim_id: hook-state-preserving-refresh
    contract_id: hook-state-preserving-refresh
    category: behavior
    command: "cargo test -p jet --test preview_hmr -- --nocapture"
    assertions:
      - "Preview HMR preserves React hook state across a compatible source edit."
```
