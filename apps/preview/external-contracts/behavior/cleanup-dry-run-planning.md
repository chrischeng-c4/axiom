---
id: cleanup-dry-run-planning
summary: External contract for Cleanup dry-run planning.
fill_sections: [e2e-test]
---

# EC: Cleanup dry-run planning

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: cleanup-dry-run-planning
    capability_id: gke-uat-preview-environment-rendering
    claim_id: cleanup-dry-run-planning
    contract_id: cleanup-dry-run-planning
    category: behavior
    command: "cargo test -p preview cleanup_plan_marks_closed_mr_for_namespace_delete"
    assertions:
      - "Closed MR cleanup plans delete both the preview namespace and route binding."
      - "Cleanup output keeps the route target and namespace explicit for SRE review."
      - "Cleanup output lists the base namespace and control namespace as protected namespaces."
```
