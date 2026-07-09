---
id: guarded-cleanup-janitor
summary: External contract for guarded cleanup janitor planning and apply.
fill_sections: [e2e-test]
---

# EC: Guarded cleanup janitor

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: guarded-cleanup-janitor
    capability_id: gke-uat-preview-environment-rendering
    claim_id: guarded-cleanup-janitor
    contract_id: guarded-cleanup-janitor
    category: behavior
    command: "cargo test -p preview --test local_cicd_contract local_cleanup_janitor_plan_reports_guarded_actions"
    assertions:
      - "`preview cleanup plan` emits keep, drain, and delete decisions from MR/TTL/orphan state."
      - "Protected base/control namespaces are reported as skipped and are not deleted."
      - "`preview cleanup apply --plan <json>` is covered by the kind lifecycle gate and deletes only preview namespaces and route-binding ConfigMaps."
      - "Repeated cleanup runs are idempotent."
```
