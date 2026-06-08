# Task: Gather Reference Context for Group 'token-counting-and-compact' (Change 'cclab-agent-p0')

Issues: #786_feat-agent-add-accurate-token-counting, #876_feat-agent-smart-auto-compact-llm-summarization-ac

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/cclab-agent-p0/groups/token-counting-and-compact/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. Call `sdd_artifact_create_reference_context` with the structured `specs` array

## In-Scope Specs

### cclab-agent
- `read_path:specs/cclab-agent/README.md`
- `read_path:specs/cclab-agent/agents.md`
- `read_path:specs/cclab-agent/architecture.md`
- `read_path:specs/cclab-agent/context.md`
- `read_path:specs/cclab-agent/core-types.md`
- `read_path:specs/cclab-agent/error-handling.md`
- `read_path:specs/cclab-agent/integrations.md`
- `read_path:specs/cclab-agent/llm-providers.md`
- `read_path:specs/cclab-agent/security.md`
- `read_path:specs/cclab-agent/storage.md`
- `read_path:specs/cclab-agent/streaming.md`
- `read_path:specs/cclab-agent/tools.md`
- `read_path:specs/cclab-agent/tools-analysis.md`
- `read_path:specs/cclab-agent/tools-coding.md`


Read these specs using the Read tool (file paths under `/Users/chris.cheng/cclab/cclab-agent/cclab/specs/`).
Do NOT explore specs outside the scope above.

## MCP Tools

```
mcp__cclab-mcp__sdd_artifact_create_reference_context(project_path="/Users/chris.cheng/cclab/cclab-agent", change_id="cclab-agent-p0", group_id="token-counting-and-compact", specs=[{"spec_id": "...", "spec_group": "...", "relevance": "high", "key_requirements": ["R1", "R3"]}])
```