---
change_id: sdd-p1
type: gap_codebase_spec
created_at: 2026-02-23T14:23:11.778666+00:00
updated_at: 2026-02-23T14:23:11.778666+00:00
---

# Gap Analysis: Codebase vs Spec

## Missing Specification (Code exists but no spec)

- **crates/cclab-sdd/src/mcp/tools/run_change/scope.rs** (Severity: High)
  - Scope extraction and cascade logic for explore phases exists in code but has no corresponding specification.
- **crates/cclab-sdd/src/mcp/tools/run_change/task_graph.rs** (Severity: Medium)
  - Task graph construction logic in task_graph.rs is not explicitly covered by the change-tasks specification.
- **crates/cclab-sdd/src/mcp/tools/run_change/helpers.rs** (Severity: Low)
  - Suggest topic logic and other utilities.

## Missing Implementation (Spec exists but no code)

- **run-change-diagrams** (Severity: Medium)
  - Sequence diagrams and flowcharts specified in run-change-diagrams have no corresponding implementation for dynamic generation or validation.
- **spec-ir-evaluation** & **spec-ir-yaml-schema** (Severity: Medium)
  - No matching implementation in the sdd tools.
- **post_clarify phase logic** (Severity: High)
  - Described in dag-loop and diagram specs but explicitly skipped in code.

## Discrepancies (Behavioral/Content/Naming)

- **DAG Loop Behavior Mismatch** (Severity: High)
  - `dag_loop.rs` implementation uses hardcoded phase transitions and routing instead of the dynamic topological flow defined in the `run-change-dag-loop` spec.
- **Verdict Escalation Mismatch** (Severity: Medium)
  - Code auto-approves artifacts when revision thresholds are met, contradicting the spec requirement for mainthread evaluation (#467).
- **Checklist Content Mismatch** (Severity: Medium)
  - Review checklists in implementation are missing fields or have mismatched items compared to the workflow specifications (#469, #470).
- **Naming Inconsistency** (Severity: Low)
  - Tool and symbol names (explore_codebase, explore_spec) do not match the artifact-based naming (create-codebase-context, create-spec-context) defined in the specs.
