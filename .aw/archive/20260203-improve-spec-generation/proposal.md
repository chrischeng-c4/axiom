---
id: improve-spec-generation
type: proposal
version: 1
created_at: 2026-01-27T05:11:11.648343+00:00
updated_at: 2026-01-27T05:11:11.648343+00:00
author: mcp
status: proposed
iteration: 1
summary: "Improve Gemini Spec Generation with comprehensive formal specification examples and stricter validation enforcement."
history:
  - timestamp: 2026-01-27T05:11:11.648343+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-27T05:12:21.177256+00:00
    agent: "gemini-3-flash-preview"
    tool: "revise_proposal"
    action: "revised"
    duration_secs: 340.25
  - timestamp: 2026-01-27T05:13:37.519707+00:00
    agent: "gemini-3-flash-preview"
    tool: "review_proposal"
    action: "reviewed"
    duration_secs: 76.34
impact:
  scope: minor
  affected_files: 3
  new_files: 0
affected_specs:
  - id: spec-generation-improvement
    path: specs/spec-generation-improvement.md
    depends: []---

<proposal>

# Change: improve-spec-generation

## Summary

Improve Gemini Spec Generation with comprehensive formal specification examples and stricter validation enforcement.

## Why

Ambiguous natural language in specs frequently blocks automated code generation. By providing detailed formal language examples in prompts and enforcing their use via automated validation, we ensure that specifications are machine-readable and ready for implementation. This reduces manual intervention and improves the overall quality and reliability of the spec-driven development workflow.

## What Changes

- Update create_spec.md and revise_spec.md prompt templates with detailed OpenAPI 3.1, AsyncAPI 2.6, OpenRPC 1.3, Serverless Workflow 0.8, and JSON Schema examples.
- Refactor orchestrator/prompts.rs to centralize spec_type guidance using the SpecType enum.
- Enhance prompt instructions to mandate formal languages for all non-utility spec types.
- Ensure validate_spec_completeness correctly flags missing formal specs as high-severity errors.

## Impact

- **Scope**: minor
- **Affected Files**: ~3
- **New Files**: ~0
- Affected specs:
  - `spec-generation-improvement` (no dependencies)
- Affected code: `crates/cclab-genesis/templates/prompts/create_spec.md`, `crates/cclab-genesis/templates/prompts/revise_spec.md`, `crates/cclab-genesis/src/orchestrator/prompts.rs`

</proposal>

<review iteration="1" reviewer="gemini" status="needs_revision">
## Summary
The proposal and technical specification are well-defined and address a significant technical debt by centralizing spec-type rules. However, the `tasks.md` file contains several broken dependencies and references to non-existent tasks.

## Issues
- **HIGH**: Broken dependencies in `tasks.md`.
    - Task 3.1 and 3.2 reference `depends_on: [1.1]`, but no task 1.1 exists in Layer 1 (Data Layer is empty).
    - Task 4.1 reference `depends_on: [2.1, 2.2, 2.3]`, but tasks 2.2 and 2.3 do not exist.
- **MEDIUM**: Task 4.1 (Testing) should depend on all implementation tasks (2.1, 3.1, 3.2, 3.3) to ensure a complete system is verified.

## Verdict
NEEDS_REVISION

## Next Steps
1. Correct the dependencies in `tasks.md`.
2. Ensure Layer 4 (Testing) depends on all relevant implementation tasks.
</review>

<review iteration="1" reviewer="codex" status="needs_revision">
## Summary
Tasks are internally consistent now, but they conflict with the proposal’s stated impact (affected_files/new_files) and include a new test file not reflected in the proposal.

## Issues
- **MEDIUM**: Tasks add a new test file despite proposal impact declaring `new_files: 0`. This mismatch should be reconciled to avoid scope confusion. (`crates/cclab-genesis/tests/validation_test.rs`)
- **MEDIUM**: Tasks include work on `crates/cclab-genesis/src/mcp/tools/validate_spec.rs`, but the proposal’s affected code list omits this file and understates affected_files (~3 vs 5 including tests). Update proposal impact to match tasks.
- **LOW**: Layer headings are numbered starting at 2 (Logic Layer) instead of 1, which is inconsistent with the overview and may confuse automation or readers.

## Verdict
NEEDS_REVISION

## Next Steps
1. Update the proposal impact section to include `validate_spec.rs` and the new test file, and adjust affected_files/new_files counts.
2. Normalize the layer heading numbering (e.g., start at 1) if required by tooling or conventions.
</review>
