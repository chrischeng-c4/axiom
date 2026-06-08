---
change_id: sdd-p2
type: gap_codebase_spec
created_at: 2026-02-23T16:28:28.866834+00:00
updated_at: 2026-02-23T16:28:28.866834+00:00
---

# Gap Analysis: Codebase vs Spec (sdd-p2)

| Severity | Type | Action Needed | Repair Action |
| :--- | :--- | :--- | :--- |
| high | code_without_spec | true | Update SpecIR specifications to include and document the context cascade logic (injecting prior summaries) found in the implementation. |
| medium | spec_without_code | true | Update the run_change implementation to emit the 'executor' field in normal-path responses as defined in the specifications. |
| high | spec_without_code | true | Add post-clarification phases to the StatePhase enum within the models to align with the specification. |
| medium | spec_without_code | true | Refactor the task generation logic to utilize the dedicated sdd_generate_tasks tool instead of sdd_write_artifact. |
