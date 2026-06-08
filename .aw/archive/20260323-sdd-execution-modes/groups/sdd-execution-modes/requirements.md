---
change: sdd-execution-modes
group: sdd-execution-modes
date: 2026-03-23
---

# Requirements

Replace the per-action `[workflow.agents]` config with a single `workflow.mode` field supporting 4 fixed execution-mode presets. Each preset defines a complete phase-to-executor mapping — no per-action overrides.

**Config change** (`cclab/config.toml`):
- Remove `[workflow.agents]` per-action arrays
- Add `workflow.mode: "multi_agents" | "multi_claude_agents" | "claude_subagents" | "mainthread"`
- Default: `"mainthread"`

**4 Execution Modes**:

| Mode | Dispatch mechanism |
|------|-------------------|
| `multi_agents` | External CLI subprocesses (gemini/codex/claude) — existing run_agent() path |
| `multi_claude_agents` | `claude --agent sdd-X` subprocess with tool restrictions + Bash hooks + phase-specific CLAUDE.md |
| `claude_subagents` | Claude Code Agent tool — mainthread invokes Agent tool with type + model |
| `mainthread` | No delegation; all phases execute in current LLM context |

**Preset tables**: Each mode embeds a fixed mapping of every phase action to its executor/agent/model. Executor resolution reads `workflow.mode` once, loads the corresponding preset, and dispatches accordingly. No config lookup per action.

**Fallback for all modes**: agent failure → retry once → fall back to mainthread.

**`multi_claude_agents` agent definitions** (new files in `.claude/agents/`):
- `sdd-reference-context.md` — tools: Read/Glob/Grep/Bash; disallowed: Write/Edit/Agent; model: sonnet; maxTurns: 20; readonly Bash hook
- `sdd-change-spec.md` — tools: Read/Write/Edit/Glob/Grep; disallowed: Bash/Agent; model: opus; maxTurns: 30
- `sdd-review.md` — tools: Read/Glob/Grep/Bash; disallowed: Write/Edit/Agent; model: sonnet; maxTurns: 15; readonly+test Bash hook
- `sdd-change-implementation.md` — tools: Read/Write/Edit/Glob/Grep/Bash; disallowed: Agent; model: opus; maxTurns: 50; safe Bash hook

**Bash hook scripts** (per-agent, not global):
- `.claude/hooks/sdd-readonly-bash.sh` — allows: git log/diff/status/show, ls, cat, find, cargo test/check
- `.claude/hooks/sdd-safe-bash.sh` — blocks: rm -rf, git push/reset, chmod 777

**Phase-specific CLAUDE.md generation**: Before spawning `claude --agent`, `run_agent()` generates a tailored CLAUDE.md for the phase (reference_context / change_spec / change_implementation / review).

**Spec updates**:
- `cclab/specs/crates/cclab-sdd/config/agents.md` — replace per-action tables with `workflow.mode` + 4 preset tables
- `cclab/specs/crates/cclab-sdd/logic/executor-resolution.md` — update flowchart: mode → preset lookup → dispatch (per-mode branches)
- `cclab/specs/crates/cclab-sdd/tools/utils/delegate-agent.md` — document multi_claude_agents and claude_subagents execution paths
