---
id: second-clarification
type: proposal
version: 1
created_at: 2026-01-27T03:18:29.975732+00:00
updated_at: 2026-01-27T03:18:29.975732+00:00
author: mcp
status: proposed
iteration: 1
summary: "Implement second clarification mechanism after exploration phase."
history:
  - timestamp: 2026-01-27T03:18:29.975732+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-01-27T03:33:40.651843+00:00
    agent: "gemini-3-flash-preview"
    tool: "create_proposal"
    action: "created"
    duration_secs: 1289.23
  - timestamp: 2026-01-27T03:34:17.582736+00:00
    agent: "gemini-3-flash-preview"
    tool: "review_proposal"
    action: "reviewed"
    duration_secs: 36.93
  - timestamp: 2026-01-27T03:39:12.009222+00:00
    agent: "gemini-3-flash-preview"
    tool: "revise_proposal"
    action: "revised"
    duration_secs: 294.43
  - timestamp: 2026-01-27T03:39:41.940773+00:00
    agent: "gemini-3-flash-preview"
    tool: "review_proposal"
    action: "reviewed"
    duration_secs: 29.93
impact:
  scope: minor
  affected_files: 5
  new_files: 0
affected_specs:
  - id: second-clarification-mechanism
    path: specs/second-clarification-mechanism.md
    depends: []---

<proposal>

# Change: second-clarification

## Summary

Implement second clarification mechanism after exploration phase.

## Why

When exploration discovers critical decision points or multiple viable approaches, the current workflow often proceeds with a best-guess approach or fails. A dedicated second clarification phase allows for more precise planning by letting users provide additional decisions after the initial analysis is complete.

## What Changes

- Add NeedsSecondClarification phase to StatePhase enum in models/frontmatter.rs
- Add append_clarifications function to clarifications_service.rs to support multi-phase Q&A
- Update exploration tool to transition to NeedsSecondClarification when clarification is needed
- Add genesis_append_clarifications MCP tool to allow agents to submit follow-up decisions
- Update CLI status display to include the new phase with appropriate icon and color

## Impact

- **Scope**: minor
- **Affected Files**: ~5
- **New Files**: ~0
- Affected specs:
  - `second-clarification-mechanism` (no dependencies)
- Affected code: `crates/cclab-genesis/src/models/frontmatter.rs`, `crates/cclab-genesis/src/services/clarifications_service.rs`, `crates/cclab-genesis/src/mcp/tools/exploration.rs`, `crates/cclab-genesis/src/mcp/tools/clarifications.rs`, `crates/cclab-genesis/src/mcp/tools/mod.rs`, `crates/cclab-genesis/src/cli/status.rs`

</proposal>

<review iteration="1" reviewer="gemini-3-flash-preview" status="approved">
## Summary
The implementation tasks for the second clarification mechanism have been thoroughly reviewed. The plan follows a logical progression from data model changes to business logic implementation, followed by MCP tool integration and CLI updates.

## Issues
No logical issues or missing requirements were identified. The task dependencies correctly reflect the implementation order (Data -> Logic -> Integration -> Testing).

## Verdict
approved

## Next Steps
Proceed to implement the changes according to the task list.
</review>

<review iteration="1" reviewer="codex" status="needs_revision">
## Summary
Tasks cover the major touchpoints (state enum, exploration transition, MCP tool, CLI), but there are requirement/acceptance gaps around phase markers and phase transitions after appending clarifications.

## Issues
1. **R4 not implemented or referenced**: No task explicitly covers the required phase marker for appended clarifications (R4). Task 2.1 mentions a phase header, but it is scoped to R3 and doesn’t ensure the marker requirement is implemented/validated.
2. **Acceptance criteria gap: phase reset to exploring**: The spec’s second scenario requires the append_clarifications tool to update the phase back to `exploring`. None of tasks 2.1/3.2/3.3 explicitly include this state update, so it is easy to miss.
3. **Testing coverage mismatch**: Task 4.1 says it tests append_clarifications and exploration transition, but the acceptance criteria also require a phase update on append and a phase marker in clarifications. Those behaviors aren’t called out for testing.

## Verdict
needs_revision

## Next Steps
- Add/adjust tasks to explicitly implement and verify R4 (phase marker) and the phase reset to `exploring` after append.
- Update testing task to cover both the marker and phase transition requirements.
</review>
