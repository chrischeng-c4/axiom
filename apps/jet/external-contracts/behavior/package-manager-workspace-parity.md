---
id: package-manager-workspace-parity
summary: External contract for Package Manager Workspace Parity.
fill_sections: [e2e-test]
---

# EC: Package Manager Workspace Parity

---
id: package-manager-workspace-parity
summary: External contract for Package Manager Workspace Parity.
fill_sections: [e2e-test]
---

# EC: Package Manager Workspace Parity

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: package-manager-workspace-parity
    capability_id: package-manager
    claim_id: package-manager-workspace-parity
    contract_id: package-manager-workspace-parity
    category: behavior
    command: "cargo test -p jet --lib pkg_manager::workspace -- --nocapture"
    assertions:
      - "Jet workspace package discovery and linking preserve declared workspace relationships."
```
