# Task: Gather Reference Context for Group 'type-inference-gaps' (Change 'lens-comprehensive')

Issues: #804_feat-lens-typescript-rust-type-inference-close-gen

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chris.cheng/cclab/cclab-lens/cclab/changes/lens-comprehensive/groups/type-inference-gaps/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. Call `sdd_artifact_create_reference_context` with the structured `specs` array

## In-Scope Specs

### cclab-lens
- `read_path:specs/cclab-lens/README.md`
- `read_path:specs/cclab-lens/analysis-tools.md`
- `read_path:specs/cclab-lens/class-diagram.md`
- `read_path:specs/cclab-lens/code-analysis-service-v2.md`
- `read_path:specs/cclab-lens/lens-cli-subcommands.md`
- `read_path:specs/cclab-lens/lens-codegen-unification.md`
- `read_path:specs/cclab-lens/lens-index-storage.md`
- `read_path:specs/cclab-lens/lens-lang-support.md`
- `read_path:specs/cclab-lens/lens-pdg-mcp-tools.md`
- `read_path:specs/cclab-lens/lens-yaml-codegen.md`
- `read_path:specs/cclab-lens/python-pdg-core.md`
- `read_path:specs/cclab-lens/refactoring-api.md`
- `read_path:specs/cclab-lens/rust-symbol-analysis.md`
- `read_path:specs/cclab-lens/semantic-search-api.md`
- `read_path:specs/cclab-lens/usage-examples.md`


Read these specs using the Read tool (file paths under `/Users/chris.cheng/cclab/cclab-lens/cclab/specs/`).
Do NOT explore specs outside the scope above.

## MCP Tools

```
mcp__cclab-mcp__sdd_artifact_create_reference_context(project_path="/Users/chris.cheng/cclab/cclab-lens", change_id="lens-comprehensive", group_id="type-inference-gaps", specs=[{"spec_id": "...", "spec_group": "...", "relevance": "high", "key_requirements": ["R1", "R3"]}])
```