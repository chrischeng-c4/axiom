# Task: Gather Reference Context for Group 'compiler-infrastructure' (Change 'mamba-all-p1')

Issues: #840_error-diagnostics-quality-rich-compiler-messages-w, #837_incremental-compilation-and-module-caching

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chris.cheng/cclab/cclab-mamba/cclab/changes/mamba-all-p1/groups/compiler-infrastructure/pre_clarifications.md`
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
- `read_path:specs/cclab-mamba/all-mamba-p0.md`
- `read_path:specs/cclab-mamba/pattern-matching.md`


Read these specs using the Read tool (file paths under `/Users/chris.cheng/cclab/cclab-mamba/cclab/specs/`).
Do NOT explore specs outside the scope above.

## CLI Commands

```
# Write artifact (write payload JSON first, then run)
cclab sdd artifact create-reference-context mamba-all-p1 cclab/changes/mamba-all-p1/payloads/create-reference-context.json
```