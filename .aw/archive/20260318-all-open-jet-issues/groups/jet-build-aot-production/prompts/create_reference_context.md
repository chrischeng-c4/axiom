# Task: Gather Reference Context for Group 'jet-build-aot-production' (Change 'all-open-jet-issues')

Issues: #903_jet-build-scope-hoisting-phase-2-true-module-flatt, #882_jet-build-bundle-size-215kb-vs-webpack-192kb-imple, #765_feat-jet-aot-production-build-tree-shaking-code-sp

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/all-open-jet-issues/groups/jet-build-aot-production/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. Run `cclab sdd artifact create-reference-context` with the structured `specs` array

## In-Scope Specs

### cclab-jet
- `read_path:specs/cclab-jet/aot-build.md`
- `read_path:specs/cclab-jet/jit-runner.md`
- `read_path:specs/cclab-jet/pkg-manager.md`
- `read_path:specs/cclab-jet/pkg-manager-pnpm-parity.md`
- `read_path:specs/cclab-jet/scope-hoisting.md`
- `read_path:specs/cclab-jet/tree-shaking.md`
- `read_path:specs/cclab-jet/variable-mangling.md`


Read these specs using the Read tool (file paths under `/Users/chris.cheng/cclab/cclab-jet/cclab/specs/`).
Do NOT explore specs outside the scope above.

## CLI Commands

```
# Write artifact (write payload JSON first, then run)
cclab sdd artifact create-reference-context all-open-jet-issues cclab/changes/all-open-jet-issues/payloads/create-reference-context.json
```