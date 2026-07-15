---
id: workspace-package-selection
summary: External contract for Workspace Package Selection.
fill_sections: [e2e-test]
---

# EC: Workspace Package Selection

---
id: workspace-package-selection
summary: External contract for Workspace Package Selection.
fill_sections: [e2e-test]
---

# EC: Workspace Package Selection

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: workspace-package-selection
    capability_id: workspace-task-runner
    claim_id: workspace-package-selection
    contract_id: workspace-package-selection
    category: behavior
    command: "cargo test -p jet --lib pkg_manager::workspace -- --nocapture"
    assertions:
      - "Jet workspace package selection resolves the requested workspace package and its metadata."
```
