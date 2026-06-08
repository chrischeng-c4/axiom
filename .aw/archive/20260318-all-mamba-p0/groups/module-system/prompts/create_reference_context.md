# Task: Gather Reference Context for Group 'module-system' (Change 'all-mamba-p0')

Issues: #828_import-aliases-import-x-as-y-from-x-import-y-as-z, #841_multi-file-compilation-module-graph-and-project-co, #829_relative-imports-from-import-from-module-import-na

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chris.cheng/cclab/cclab-mamba/cclab/changes/all-mamba-p0/groups/module-system/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. Run `cclab sdd artifact create-reference-context` with the structured `specs` array

## In-Scope Specs

### cclab-mamba
- `read_path:specs/cclab-mamba/README.md`
- `read_path:specs/cclab-mamba/pattern-matching.md`


Read these specs using the Read tool (file paths under `/Users/chris.cheng/cclab/cclab-mamba/cclab/specs/`).
Do NOT explore specs outside the scope above.

## CLI Commands

```
# Write artifact (write payload JSON first, then run)
cclab sdd artifact create-reference-context all-mamba-p0 cclab/changes/all-mamba-p0/payloads/create-reference-context.json
```