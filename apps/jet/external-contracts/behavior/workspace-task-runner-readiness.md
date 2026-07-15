---
id: workspace-task-runner-readiness
summary: External contract for Workspace Task Runner Readiness.
fill_sections: [e2e-test]
---

# EC: Workspace Task Runner Readiness

---
id: workspace-task-runner-readiness
summary: External contract for Workspace Task Runner Readiness.
fill_sections: [e2e-test]
---

# EC: Workspace Task Runner Readiness

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: workspace-task-runner-readiness
    capability_id: workspace-task-runner
    claim_id: workspace-task-runner-readiness
    contract_id: workspace-task-runner-readiness
    category: behavior
    command: "cargo test -p jet --lib task_runner -- --nocapture"
    assertions:
      - "Jet workspace task runner passes its readiness suite for graph execution and caching."
```
