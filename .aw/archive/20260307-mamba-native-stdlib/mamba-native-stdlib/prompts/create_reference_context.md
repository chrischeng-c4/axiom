# Task: Gather Reference Context for Group 'mamba-native-stdlib-rewrite' (Change 'mamba-native-stdlib')


## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chrischeng/projects/cclab/cclab/changes/mamba-native-stdlib/groups/mamba-native-stdlib-rewrite/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. Call `sdd_artifact_create_reference_context` with the structured `specs` array

## Specs

- List specs under `/Users/chrischeng/projects/cclab/cclab/specs/` using Glob
- Read at most 5 specs. Focus on the most relevant ones.

## MCP Tools

```
mcp__cclab-mcp__sdd_artifact_create_reference_context(project_path="/Users/chrischeng/projects/cclab", change_id="mamba-native-stdlib", group_id="mamba-native-stdlib-rewrite", specs=[{"spec_id": "...", "spec_group": "...", "relevance": "high", "key_requirements": ["R1", "R3"]}])
```