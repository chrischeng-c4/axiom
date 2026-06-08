---
change: sdd-workflow-cleanup
group: spec-plan
date: 2026-03-17
---

# Requirements

Add a structured Spec Plan YAML block to `reference_context.md`. This allows `create-change-spec` to auto-determine `main_spec_ref` and `merge_strategy`. Update `render_specs_markdown` in `common_reference_context.rs`, extend `SpecInfo` struct in `workflow/helpers.rs`, and update skeleton generation logic in `create_change_spec.rs`. Update logic specs for reference-context and change-spec.
