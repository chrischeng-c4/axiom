---
id: ci-template-lifecycle
summary: External contract for copyable local-first CI/CD lifecycle templates.
fill_sections: [e2e-test]
---

# EC: CI template lifecycle

## External Contract
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: ci-template-lifecycle
    capability_id: preview-external-contracts
    claim_id: ci-template-lifecycle
    contract_id: ci-template-lifecycle
    category: behavior
    command: "cargo test -p preview --test local_cicd_contract ci_templates_document_required_variables_and_command_order"
    assertions:
      - "GitHub Actions, GitLab CI, and local kind templates define all required PREVIEW_* variables."
      - "Templates preserve open/update/rerun command order from discover-base through render, apply plan, dry-run, apply, rollout, router resolve, and comment."
      - "Templates preserve close/merge command order from cleanup plan to cleanup apply."
      - "The kind lifecycle gate validates the documented local path stays runnable."
```
