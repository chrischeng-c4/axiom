---
id: task-runner-graph-cache
summary: External contract for Task Runner Graph Cache.
fill_sections: [e2e-test]
---

# EC: Task Runner Graph Cache

---
id: task-runner-graph-cache
summary: External contract for Task Runner Graph Cache.
fill_sections: [e2e-test]
---

# EC: Task Runner Graph Cache

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: task-runner-graph-cache
    capability_id: workspace-task-runner
    claim_id: task-runner-graph-cache
    contract_id: task-runner-graph-cache
    category: behavior
    command: "cargo test -p jet --lib task_runner -- --nocapture"
    assertions:
      - "Jet task runner builds and executes a dependency graph with its cache contract intact."
```
