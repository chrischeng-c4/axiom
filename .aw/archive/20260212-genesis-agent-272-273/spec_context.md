---
change_id: genesis-agent-272-273
type: spec_context
created_at: 2026-02-12T11:26:09.339295+00:00
updated_at: 2026-02-12T11:26:09.339295+00:00
iteration: 1
complexity: high
stage: spec
scanned_groups:
  - genesis
  - cclab-genesis
---

# Spec Context

## Relevant Specs

- **delegate-agent** (group: cclab-genesis)
  - relevance: high
  - reason: Primary spec — defines the full action enum (30+ actions), artifact-oriented response schema, verification table, error recovery, and recursion prevention. Current code implements only 3 of 30+ actions.
  - key sections: OpenRPC Method Definition, Verification Table, Sequence Diagram, Error Recovery
- **delegate-agent-coverage** (group: cclab-genesis)
  - relevance: high
  - reason: Patch spec extending delegate-agent with gap-create, merge, and impl actions. All R1-R3 marked as implemented in spec but NOT in code.
  - key sections: R1 - gap actions, R2 - merge/impl actions, R3 - artifact names
- **agent-tool** (group: genesis)
  - relevance: medium
  - reason: Original v1 spec with only 3 actions (explore/review/custom). Superseded by delegate-agent spec but code still follows this outdated spec.
  - key sections: R3 - Action Templates, R5 - Response Format
- **prompt-registry** (group: genesis)
  - relevance: high
  - reason: Per-action prompt templates already exist in run_change/*.rs modules. The delegate_agent should leverage these rather than only using LLM_EXPLORE/LLM_REVIEW constants.
  - key sections: R2 - agent_prompt population, R5 - context artifacts in prompts
- **action-enum-sync** (group: cclab-genesis)
  - relevance: medium
  - reason: Documents action enum additions (review_task, revise_task, task_terminal_failure, etc.) that must be supported by delegate_agent.

## Dependencies

- delegate-agent depends on run-change (action routing)
- delegate-agent-coverage extends delegate-agent (patch)
- prompt-registry feeds into delegate-agent (per-action prompts)
- action-enum-sync extends run-change action enum

## Gaps

- Code implements only 3 actions (explore/review/custom) vs spec's 30+ actions
- Response returns raw stdout instead of artifact-oriented format per spec
- Tool name is genesis_agent instead of genesis_delegate_agent (per clarification)
- No error recovery (retry + fallback chain) implemented yet
- Prompt templates are only LLM_EXPLORE and LLM_REVIEW constants — not per-action
