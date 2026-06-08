# Task: Gather Reference Context for Group 'sdd-frontend-doc-artifacts' (Change 'sdd-frontend-doc-support')

Issues: #898_feat-sdd-support-user-facing-doc-as-change-artifac, #897_feat-sdd-add-wireframe-yaml-dsl-for-frontend-inter

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/sdd-frontend-doc-support/groups/sdd-frontend-doc-artifacts/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. Run `cclab sdd artifact create-reference-context` with the structured `specs` array

## In-Scope Specs

### cclab-sdd
- `read_path:specs/cclab-sdd/README.md`
- `read_path:specs/cclab-sdd/sdd-cli.md`


Read these specs using the Read tool (file paths under `/Users/chris.cheng/cclab/cclab-sdd/cclab/specs/`).
Do NOT explore specs outside the scope above.

## CLI Commands

```
# Write artifact (write payload JSON first, then run)
cclab sdd artifact create-reference-context sdd-frontend-doc-support cclab/changes/sdd-frontend-doc-support/payloads/create-reference-context.json
```