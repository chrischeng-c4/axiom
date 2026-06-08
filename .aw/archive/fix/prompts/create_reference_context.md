# Task: Gather Reference Context for Group 'jet-bundler-fixes' (Change 'fix')

Issues: #796_jet-build-expand-mini-react-example-with-advanced-

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/fix/groups/jet-bundler-fixes/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. Call `sdd_artifact_create_reference_context` with the structured `specs` array

## In-Scope Specs

### cclab-jet
- `read_path:specs/cclab-jet/aot-build.md`
- `read_path:specs/cclab-jet/jit-runner.md`
- `read_path:specs/cclab-jet/pkg-manager.md`
- `read_path:specs/cclab-jet/pkg-manager-pnpm-parity.md`


Read these specs using the Read tool (file paths under `/Users/chris.cheng/cclab/cclab-jet/cclab/specs/`).
Do NOT explore specs outside the scope above.

## MCP Tools

```
mcp__cclab-mcp__sdd_artifact_create_reference_context(project_path="/Users/chris.cheng/cclab/cclab-jet", change_id="fix", group_id="jet-bundler-fixes", specs=[{"spec_id": "...", "spec_group": "...", "relevance": "high", "key_requirements": ["R1", "R3"]}])
```