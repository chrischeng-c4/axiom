---
change: sdd-execution-modes
group: sdd-execution-modes
date: 2026-03-23
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Does this change completely remove the existing [workflow.agents] per-action arrays?
- **Answer**: Yes. Complete removal. Replace [workflow.agents] per-action arrays entirely with workflow.mode single field. Clean break, no backward compatibility.

### Q2: General
- **Question**: How does executor resolution handle claude_subagents mode in the Rust layer?
- **Answer**: Rust returns a structured signal: {executor: 'subagent', subagent_type: 'Explore', model: 'sonnet', prompt_path: '...'}. The mainthread skill reads this signal and is responsible for invoking the Claude Code Agent tool. run_agent() is NOT called for this mode.

### Q3: General
- **Question**: How is the phase-specific CLAUDE.md injected for multi_claude_agents?
- **Answer**: Write generated CLAUDE.md to a temp directory, then spawn claude --agent sdd-X --cwd /tmp/sdd-phase-dir/. The temp dir contains the phase-specific CLAUDE.md. Project files are accessed via absolute paths in the prompt.

### Q4: General
- **Question**: Should the fallback mechanism use 2-element array or new on_failure field?
- **Answer**: Neither. No config for fallback — presets are hardcoded in Rust code. Fallback to mainthread is implicit in the executor resolution logic (try agent, retry once, fall back to mainthread). No config representation needed.

### Q5: General
- **Question**: Is there a canonical schema for .claude/agents/ frontmatter?
- **Answer**: Yes, Claude Code documents the schema. Supported frontmatter fields: name, description, tools (allowlist), disallowedTools (denylist), model (sonnet/opus/haiku/inherit), maxTurns, permissionMode, hooks (PreToolUse/PostToolUse/Stop), memory, background, effort, isolation. The markdown body is the system prompt.

