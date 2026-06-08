---
id: design-elements-enforcement
type: proposal
version: 1
created_at: 2026-01-26T10:10:17.656752+00:00
updated_at: 2026-01-26T10:10:17.656752+00:00
author: mcp
status: proposed
iteration: 1
summary: "Enforce Mermaid and API spec generation based on spec_type to ensure code generation readiness."
history:
  - timestamp: 2026-01-26T10:10:17.656752+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-26T10:11:31.284807+00:00
    agent: "gemini-3-flash-preview"
    tool: "revise_proposal"
    action: "revised"
    duration_secs: 132.74
  - timestamp: 2026-01-26T10:13:30.335413+00:00
    agent: "gemini-3-flash-preview"
    tool: "review_proposal"
    action: "reviewed"
    duration_secs: 119.05
impact:
  scope: minor
  affected_files: 7
  new_files: 0
affected_specs:
  - id: spec-enforcement-rules
    path: specs/spec-enforcement-rules.md
    depends: []
  - id: validator-enhancement
    path: specs/validator-enhancement.md
    depends: [spec-enforcement-rules]---

<proposal>

# Change: design-elements-enforcement

## Summary

Enforce Mermaid and API spec generation based on spec_type to ensure code generation readiness.

## Why

Currently, spec_type requirements (like OpenAPI for http-api) are partially enforced in the create_spec tool but missing from general validation and the SemanticValidator. Centralizing these rules ensures consistency across creation, validation, and AI instructions, improving the quality of generated specs and their suitability for downstream code generation.

## What Changes

- Centralize spec_type requirements in models/spec_rules.rs
- Update SemanticValidator to enforce required diagrams and API specs per spec_type
- Enhance create_spec and validate_spec_completeness tools with centralized validation logic
- Update get_task tool to provide spec_type-specific guidance in instructions
- Align prompts.rs with centralized spec rules
- Create technical specifications for centralized rules and validation enforcement

## Impact

- **Scope**: minor
- **Affected Files**: ~7
- **New Files**: ~0
- Affected specs:
  - `spec-enforcement-rules` (no dependencies)
  - `validator-enhancement` → depends on: `spec-enforcement-rules`
- Affected code: `crates/cclab-genesis/src/models/spec_rules.rs`, `crates/cclab-genesis/src/services/spec_service.rs`, `crates/cclab-genesis/src/validator/semantic.rs`, `crates/cclab-genesis/src/mcp/tools/spec.rs`, `crates/cclab-genesis/src/mcp/tools/validate_spec.rs`, `crates/cclab-genesis/src/mcp/tools/task.rs`, `crates/cclab-genesis/src/orchestrator/prompts.rs`
- **Breaking Changes**: None. Internal validation enhancements.

</proposal>

<review iteration="1" reviewer="gemini-3-flash-preview" status="approved">
## Summary
The implementation tasks for `design-elements-enforcement` are well-structured and accurately reflect the requirements defined in the specifications. The layering correctly follows the build order (Data -> Logic -> Integration -> Testing), and all tasks have clear references to the corresponding specifications and requirements.

## Issues
None identified. The task breakdown is comprehensive and covers all affected files identified in the proposal.

## Verdict
approved

## Next Steps
Proceed to implementation phase.
</review>

<review iteration="1" reviewer="codex" status="needs_revision">
## Summary
Tasks map well to the specs, but dependency IDs are inconsistent with the task IDs and likely invalid.

## Issues
- Task dependency identifiers use layer-prefixed labels like `data.1`, `logic.1`, `integration.1` instead of the actual task IDs (`1.1`, `2.1`, `3.1`, etc.). This makes the dependency graph ambiguous or unresolvable and could break tooling that expects task IDs.

## Verdict
needs_revision

## Next Steps
Update all `depends_on` entries to reference the real task IDs (e.g., `1.1`, `2.1`, `3.1`, `3.2`, `3.3`) and re-run the review.
</review>
