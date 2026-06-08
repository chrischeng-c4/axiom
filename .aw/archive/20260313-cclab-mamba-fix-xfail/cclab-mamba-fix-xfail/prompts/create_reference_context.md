# Task: Gather Reference Context for Group 'mamba-conformance-xfail' (Change 'cclab-mamba-fix-xfail')

Issues: #753_py3-12-conformance-mbvalue-arithmetic-comparison-t, #754_py3-12-conformance-object-model-class-mro-descript, #758_py3-12-conformance-builtins-108-tests-full-verific, #756_py3-12-conformance-generator-iterator-protocol, #755_py3-12-conformance-exception-hierarchy, #759_py3-12-conformance-data-structure-ops-list-dict-se, #752_py3-12-conformance-test-harness-cpython-vs-mamba-c

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chris.cheng/cclab/cclab-mamba/cclab/changes/cclab-mamba-fix-xfail/groups/mamba-conformance-xfail/pre_clarifications.md`
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


Read these specs using the Read tool (file paths under `/Users/chris.cheng/cclab/cclab-mamba/cclab/specs/`).
Do NOT explore specs outside the scope above.

## MCP Tools

```
mcp__cclab-mcp__sdd_artifact_create_reference_context(project_path="/Users/chris.cheng/cclab/cclab-mamba", change_id="cclab-mamba-fix-xfail", group_id="mamba-conformance-xfail", specs=[{"spec_id": "...", "spec_group": "...", "relevance": "high", "key_requirements": ["R1", "R3"]}])
```