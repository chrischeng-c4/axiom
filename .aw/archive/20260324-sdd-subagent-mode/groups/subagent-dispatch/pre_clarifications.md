---
change: sdd-subagent-mode
group: subagent-dispatch
date: 2026-03-24
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Where does subagent dispatch happen?
- **Answer**: subagent:* dispatch is NOT in the Rust CLI. It's handled by the Claude Code mainthread (the skill loop in run-change.md). The workflow tool returns executor: ['subagent:Explore:sonnet'] and the skill/mainthread uses the Agent tool to dispatch. The Rust side only needs to: (1) return the correct executor string from AgentsConfig preset, (2) provide the prompt_path. The Agent tool invocation is done by the LLM mainthread, not by cclab CLI.

### Q2: General
- **Question**: How does multi_claude_agents differ?
- **Answer**: multi_claude_agents uses claude-agent:X executors which are dispatched by the Rust CLI (tools/agent.rs) via 'claude --agent sdd-X' subprocess. claude_subagents uses subagent:X:model executors which are dispatched by the LLM mainthread via the Agent tool. Different dispatch mechanism entirely — CLI subprocess vs LLM tool call.

