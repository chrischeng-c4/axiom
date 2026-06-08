---
change: score-agent-rename-hook
group: rename-agents-and-add-hook
date: 2026-04-10
---

# Requirements

Rename all sdd-* Claude Code agent definitions (.claude/agents/sdd-*.md) to score-*.md. Rename hook scripts (.claude/hooks/sdd-*.sh) to score-*.sh. Update all references in CLAUDE.md, crates/sdd/src/workflow/mod.rs, dispatch-model spec, and any CLI/prompt templates. Update .claude/settings.json hook patterns from sdd-* to score-*. Add a SubagentStop hook in .claude/settings.json that matches all score-* agents. Create a new hook script .claude/hooks/score-next-step.sh that runs score run-change and injects next phase info as additionalContext. The hook must be observation-only (no blocking, no output modification).
