# Task: Gather Reference Context for Group 'codegen-hir-mir-coverage' (Change 'mamba-test-coverage')

Issues: #744_test-coverage-codegen-jit-aot-llvm-target-95-98-li, #743_test-coverage-hir-mir-lowering-target-95-98-line-c, #747_test-coverage-name-resolution-target-95-98-line-co

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chrischeng/projects/cclab/cclab/changes/mamba-test-coverage/groups/codegen-hir-mir-coverage/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. Call `sdd_artifact_create_reference_context` with the structured `specs` array

## In-Scope Specs

### cclab-mamba
- `read_path:specs/cclab-mamba/README.md`


Read these specs using the Read tool (file paths under `/Users/chrischeng/projects/cclab/cclab/specs/`).
Do NOT explore specs outside the scope above.

## MCP Tools

```
mcp__cclab-mcp__sdd_artifact_create_reference_context(project_path="/Users/chrischeng/projects/cclab", change_id="mamba-test-coverage", group_id="codegen-hir-mir-coverage", specs=[{"spec_id": "...", "spec_group": "...", "relevance": "high", "key_requirements": ["R1", "R3"]}])
```