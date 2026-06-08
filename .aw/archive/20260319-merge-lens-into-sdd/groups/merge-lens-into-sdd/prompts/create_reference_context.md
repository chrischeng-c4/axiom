# Task: Gather Reference Context for Group 'merge-lens-into-sdd' (Change 'merge-lens-into-sdd')

Issues: #942_refactor-merge-cclab-lens-into-cclab-sdd

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chris.cheng/cclab/cclab-sdd/cclab/changes/merge-lens-into-sdd/groups/merge-lens-into-sdd/pre_clarifications.md`
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

### cclab-lens
- `read_path:specs/cclab-lens/README.md`
- `read_path:specs/cclab-lens/analysis-tools.md`
- `read_path:specs/cclab-lens/class-diagram.md`
- `read_path:specs/cclab-lens/code-analysis-service-v2.md`
- `read_path:specs/cclab-lens/lens-beyond-ide.md`
- `read_path:specs/cclab-lens/lens-cli-subcommands.md`
- `read_path:specs/cclab-lens/lens-codegen-unification.md`
- `read_path:specs/cclab-lens/lens-comprehensive.md`
- `read_path:specs/cclab-lens/lens-index-storage.md`
- `read_path:specs/cclab-lens/lens-lang-support.md`
- `read_path:specs/cclab-lens/lens-markdown.md`
- `read_path:specs/cclab-lens/lens-pdg-mcp-tools.md`
- `read_path:specs/cclab-lens/lens-yaml-codegen.md`
- `read_path:specs/cclab-lens/python-pdg-core.md`
- `read_path:specs/cclab-lens/refactoring-api.md`
- `read_path:specs/cclab-lens/rust-symbol-analysis.md`
- `read_path:specs/cclab-lens/semantic-search-api.md`
- `read_path:specs/cclab-lens/usage-examples.md`


Read these specs using the Read tool (file paths under `/Users/chris.cheng/cclab/cclab-sdd/cclab/specs/`).
Do NOT explore specs outside the scope above.

## CLI Commands

```
# Write artifact (write payload JSON first, then run)
cclab sdd artifact create-reference-context merge-lens-into-sdd cclab/changes/merge-lens-into-sdd/payloads/create-reference-context.json
```