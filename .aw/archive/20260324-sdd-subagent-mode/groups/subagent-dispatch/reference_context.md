---
change: sdd-subagent-mode
group: subagent-dispatch
date: 2026-03-24
written_by: artifact_cli
review_verdict: APPROVED
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| crates/cclab-sdd/logic/executor-resolution.md | execution-modes | high | Defines the claude_subagents mode returning subagent:X:model executor strings from preset table, Documents that Rust returns executor: ['subagent:Explore:sonnet'] and mainthread skill dispatches via Agent tool, Contains the full preset table for claude_subagents mapping phase actions to Subagent Type + Model, Defines the 4 ExecutionMode variants including ClaudeSubagents, Agent failure retry and fallback chain logic applies to subagent mode too |
| crates/cclab-sdd/config/agents.md | configuration | high | Defines workflow.mode = claude_subagents config field, Contains claude_subagents preset: subagent_type + model per phase action, Dispatch pattern: Rust returns executor='subagent', subagent_type, model, prompt_path; mainthread invokes Agent tool, Lists all claude_subagents preset rows (Explore:sonnet for reference_context, general-purpose:opus for spec, etc.) |
| crates/cclab-sdd/skills/run-change.md | skills | high | Current skill loop says 'Workflow tool internally handles agent delegation (no need to check executor)' — must update for subagent:* dispatch, When executor[0] starts with subagent:, skill must parse subagent_type and model, then invoke Agent tool, Must read prompt_path from workflow tool response and pass prompt content to Agent tool, Skill is the only place where Claude Code Agent tool can be invoked (LLM mainthread, not Rust CLI) |
| crates/cclab-sdd/tools/utils/delegate-agent.md | tools | high | run_agent() currently handles gemini:, codex:, claude:, claude-agent: providers — does NOT handle subagent: prefix, build_workflow_response() currently routes non-mainthread executors to run_agent() — must add third branch for subagent:* (return to mainthread, not call run_agent()), Verification logic after agent completion applies to subagent mode (STATE.yaml phase + artifact check), Sequence diagram and behavior flowchart need claude_subagents path added |
| crates/cclab-sdd/interfaces/tools/workflow-tools.md | interfaces | medium | executor field in workflow tool response schema currently typed as array with const=['mainthread'], Schema must be updated to allow subagent:X:model executor strings for claude_subagents mode, prompt_path is returned alongside executor for mainthread/subagent to read |
| crates/cclab-sdd/logic/state-machine.md | logic | medium | Phase verification after subagent dispatch: STATE.yaml phase + artifact existence check, The delegation_guard and phase tracking apply regardless of executor mode, DelegationGuard prevents concurrent agents on same change_id |
| crates/cclab-sdd/logic/reference-context.md | logic | medium | Reference context is the first phase where claude_subagents dispatches (Explore:sonnet subagent), Agent retry mechanism and mainthread fallback apply to subagent mode, verify_artifact_cli_called() must work after subagent completes (same as other agent types) |
| crates/cclab-sdd/interfaces/cli/commands.md | interfaces | low | No new CLI commands for claude_subagents mode — mode is purely config-driven via workflow.mode |

## Spec Plan

| Spec ID | Action | Main Spec Ref | Sections |
|---------|--------|---------------|----------|
| subagent-skill-dispatch | modify | crates/cclab-sdd/skills/run-change.md | overview, logic, interaction, changes |
| subagent-workflow-dispatch | modify | crates/cclab-sdd/tools/utils/delegate-agent.md | overview, logic, interaction, changes |
| subagent-executor-resolution | modify | crates/cclab-sdd/logic/executor-resolution.md | overview, config, changes |

# Reviews

## Review: reviewer (Iteration 1)

**Change ID**: sdd-subagent-mode

**Verdict**: APPROVED

### Summary

Comprehensive reference context covering executor-resolution, agents config, run-change skill, delegate-agent, workflow-tools, state-machine, and reference-context specs. Three spec_plan entries correctly target the skill, delegate-agent, and executor-resolution specs.

### Issues

No issues found.
