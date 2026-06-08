# Task: Gather Reference Context for Group 'restructure-codebase-agent' (Change 'restructure-codebase-agent')

Issues: #953_feat-agent-add-restructurecodebaseagent-workspace-

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chris.cheng/cclab/cclab-agent/cclab/changes/restructure-codebase-agent/groups/restructure-codebase-agent/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. Run `cclab sdd artifact create-reference-context` with the structured `specs` array

## In-Scope Specs

### cclab-agent
- `read_path:specs/cclab-agent/README.md`
- `read_path:specs/cclab-agent/agents.md`
- `read_path:specs/cclab-agent/architecture.md`
- `read_path:specs/cclab-agent/context.md`
- `read_path:specs/cclab-agent/core-types.md`
- `read_path:specs/cclab-agent/error-handling.md`
- `read_path:specs/cclab-agent/fillback-agents.md`
- `read_path:specs/cclab-agent/integrations.md`
- `read_path:specs/cclab-agent/llm-providers.md`
- `read_path:specs/cclab-agent/reference-context-agent.md`
- `read_path:specs/cclab-agent/restructure-agent.md`
- `read_path:specs/cclab-agent/review-agent.md`
- `read_path:specs/cclab-agent/security.md`
- `read_path:specs/cclab-agent/storage.md`
- `read_path:specs/cclab-agent/streaming.md`
- `read_path:specs/cclab-agent/tools.md`
- `read_path:specs/cclab-agent/tools-analysis.md`
- `read_path:specs/cclab-agent/tools-coding.md`


Read these specs using the Read tool (file paths under `/Users/chris.cheng/cclab/cclab-agent/cclab/specs/`).
Do NOT explore specs outside the scope above.

## CLI Commands

```
# Write artifact (write payload JSON first, then run)
cclab sdd artifact create-reference-context restructure-codebase-agent cclab/changes/restructure-codebase-agent/payloads/create-reference-context.json
```