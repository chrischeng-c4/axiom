---
id: nx-graph-parity
summary: External contract for Nx Graph Parity.
fill_sections: [e2e-test]
---

# EC: Nx Graph Parity

---
id: nx-graph-parity
summary: External contract for Nx Graph Parity.
fill_sections: [e2e-test]
---

# EC: Nx Graph Parity

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: nx-graph-parity
    capability_id: workspace-task-runner
    claim_id: nx-graph-parity
    contract_id: nx-graph-parity
    category: behavior
    command: "cargo test -p jet --lib pkg_manager::nx -- --nocapture"
    assertions:
      - "Jet builds an Nx workspace dependency graph consistent with the workspace configuration."
```
