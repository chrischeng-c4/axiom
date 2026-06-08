---
change: sdd-subagent-mode
group: subagent-dispatch
date: 2026-03-24
---

# Requirements

The claude_subagents execution mode uses executor strings like subagent:Explore:sonnet, subagent:general-purpose:opus. These are dispatched via the Claude Code Agent tool (not CLI subprocess). The workflow tools need to recognize subagent:* executors and dispatch them using the Agent tool with the appropriate subagent_type and model parameters.

1. Workflow tools must detect subagent:* executor prefix and invoke the Agent tool
2. Agent tool dispatch: subagent_type from executor string part 2, model from part 3
3. Prompt is read from prompt_path and passed to the Agent tool
4. Agent result is captured and returned to the workflow loop
5. Fallback: if Agent tool fails, retry once, then fallback to mainthread
