---
number: 1046
title: "feat(sdd): subagent execution mode — Claude Code agents with least-privilege tooling"
state: open
labels: [type:enhancement, priority:p1, crate:sdd]
group: "subagent-dispatch"
---

# #1046 — feat(sdd): subagent execution mode — Claude Code agents with least-privilege tooling

## Problem

SDD needs 4 standalone execution modes with **fixed, built-in agent:model mappings per phase**. User picks one mode in `config.toml` — all phase routing is predetermined. Agent failure → retry once → fallback to mainthread.

## 4 Execution Modes

| Mode | Execution | Characteristics |
|------|-----------|----------------|
| **multi_agents** | External CLI subprocess (gemini/codex/claude) | Model diversity, cost optimization, cross-model review |
| **multi_claude_agents** | `claude --agent sdd-X` CLI subprocess | Claude Code tools + isolation + tool restriction + CLAUDE.md swap |
| **claude_subagents** | Claude Code Agent tool (mainthread invokes) | Low overhead, inherits session, non-deterministic |
| **mainthread** | Current LLM context | Zero overhead, shared context, no isolation |

```toml
[workflow]
# "multi_agents" | "multi_claude_agents" | "claude_subagents" | "mainthread"
mode = "multi_claude_agents"
```

No per-action overrides. Mode determines all mappings. Fallback: fail → retry once → mainthread.

---

## Phase Complexity Reference

| Phase Action | Complexity | Nature |
|---|---|---|
| `restructure_input` | Low | Parse/transform |
| `create_pre_clarifications` | Low | Interactive Q&A |
| `create_reference_context` | Medium | Explore codebase/specs |
| `review_reference_context` | Low | Check completeness |
| `revise_reference_context` | Low | Targeted fix |
| `create_post_clarifications` | Low | Interactive Q&A |
| `create_change_spec` | **High** | Design/write spec |
| `review_change_spec` | Medium-High | Evaluate design |
| `revise_change_spec` | Medium-High | Fix design |
| `implement` | **High** | Write code |
| `review_implementation` | Medium-High | Code review |
| `revise_implementation` | **High** | Fix code |
| `create_change_merge` | Low | Programmatic (always mainthread) |

---

## Preset 1: `multi_agents`

External CLIs — leverages each provider's strength.

| Phase Action | Agent:Model | Rationale |
|---|---|---|
| `restructure_input` | mainthread | Low complexity, shared context |
| `create_pre_clarifications` | mainthread | Interactive |
| `create_reference_context` | **gemini:flash** | 1M+ context, fast explore, $0.10/1M |
| `review_reference_context` | **codex:balanced** | Different model = different perspective |
| `revise_reference_context` | mainthread | Targeted fix |
| `create_post_clarifications` | mainthread | Interactive |
| `create_change_spec` | **gemini:pro** | Strong reasoning + large context for specs |
| `review_change_spec` | **codex:max** | Cross-model review, extra-high reasoning |
| `revise_change_spec` | **gemini:pro** | Consistent with spec creator |
| `implement` | mainthread | Claude Code tools essential for coding |
| `review_implementation` | **codex:balanced** | Sandbox code review |
| `revise_implementation` | mainthread | Claude Code tools essential |
| `create_change_merge` | mainthread | Programmatic |

**Why:** Gemini for spec work (large context + cheap), Codex for reviews (different perspective + sandbox), mainthread for implementation (no CLI matches Claude Code tooling).

**Est. cost/change:** ~$5.55 delegated

---

## Preset 2: `multi_claude_agents`

`claude --agent sdd-X` — full Claude Code tooling with isolation + least-privilege.

| Phase Action | Agent Definition | Model | Rationale |
|---|---|---|---|
| `restructure_input` | mainthread | — | Low complexity |
| `create_pre_clarifications` | mainthread | — | Interactive |
| `create_reference_context` | **sdd-reference-context** | sonnet | Read-heavy explore, no write needed |
| `review_reference_context` | **sdd-review** | haiku | Quick check, cheap |
| `revise_reference_context` | mainthread | — | Targeted fix |
| `create_post_clarifications` | mainthread | — | Interactive |
| `create_change_spec` | **sdd-change-spec** | opus | Heavy spec writing, no Bash |
| `review_change_spec` | **sdd-review** | sonnet | Analytical review, read-only |
| `revise_change_spec` | **sdd-change-spec** | opus | Consistent with creator |
| `implement` | **sdd-change-implementation** | opus | Full tools, safe Bash |
| `review_implementation` | **sdd-review** | sonnet | Read-only + cargo test/check |
| `revise_implementation` | **sdd-change-implementation** | opus | Full tools |
| `create_change_merge` | mainthread | — | Programmatic |

**Why:** Deterministic dispatch via `claude --agent`. Each agent has tool restrictions + Bash hooks. Phase-specific CLAUDE.md swap.

**Est. cost/change:** ~$49.50 delegated (Claude pricing higher, full tooling everywhere)

---

## Preset 3: `claude_subagents`

Claude Code Agent tool — mainthread invokes, low overhead, inherits session.

| Phase Action | Subagent Type | Model | Rationale |
|---|---|---|---|
| `restructure_input` | mainthread | — | Low complexity |
| `create_pre_clarifications` | mainthread | — | Interactive |
| `create_reference_context` | **Explore** | sonnet | Built-in Explore agent optimized for read-heavy |
| `review_reference_context` | **general-purpose** | haiku | Quick review |
| `revise_reference_context` | mainthread | — | Targeted fix |
| `create_post_clarifications` | mainthread | — | Interactive |
| `create_change_spec` | **general-purpose** | opus | Heavy spec writing |
| `review_change_spec` | **general-purpose** | sonnet | Analytical review |
| `revise_change_spec` | **general-purpose** | opus | Consistent |
| `implement` | **general-purpose** | opus | Full tools |
| `review_implementation` | **general-purpose** | sonnet | Code review |
| `revise_implementation` | **general-purpose** | opus | Full tools |
| `create_change_merge` | mainthread | — | Programmatic |

**Why:** Lowest overhead for delegation. Inherits MCP/tools/permissions from session. Non-deterministic (relies on mainthread correctly invoking Agent tool via skill instructions). No tool restriction beyond prompt guidance.

**Est. cost/change:** ~$49.50 delegated (same Claude pricing, less control)

**Tradeoff vs multi_claude_agents:**
| | multi_claude_agents | claude_subagents |
|---|---|---|
| Tool restriction | `.claude/agents/` frontmatter | None (prompt only) |
| Bash hooks | Per-agent PreToolUse | None |
| CLAUDE.md | Phase-specific swap | Inherited from mainthread |
| Overhead | Medium (new CLI process) | Low (fork in session) |
| Deterministic | Yes | No (skill instructions guide) |
| Isolation | Full process isolation | Context isolation only |

---

## Preset 4: `mainthread`

Everything in current context. Zero delegation.

| Phase Action | Executor | Rationale |
|---|---|---|
| All phases | mainthread | Simplest, zero overhead, shared context |

**Why:** For simple changes, debugging, or when you want full control. Risk: context pollution on large changes.

---

## Agent Definitions (`.claude/agents/`) — for `multi_claude_agents` mode

### `sdd-reference-context.md`
| Field | Value |
|-------|-------|
| tools | Read, Glob, Grep, Bash |
| disallowedTools | Write, Edit, Agent |
| model | sonnet |
| maxTurns | 20 |
| Bash hook | readonly: `git log/diff/status`, `ls`, `find`, `cargo test` |

### `sdd-change-spec.md`
| Field | Value |
|-------|-------|
| tools | Read, Write, Edit, Glob, Grep |
| disallowedTools | Bash, Agent |
| model | opus |
| maxTurns | 30 |

### `sdd-review.md`
| Field | Value |
|-------|-------|
| tools | Read, Glob, Grep, Bash |
| disallowedTools | Write, Edit, Agent |
| model | sonnet |
| maxTurns | 15 |
| Bash hook | readonly + `cargo test/check` |

### `sdd-change-implementation.md`
| Field | Value |
|-------|-------|
| tools | Read, Write, Edit, Glob, Grep, Bash |
| disallowedTools | Agent |
| model | opus |
| maxTurns | 50 |
| Bash hook | safe: no `rm -rf`, `git push/reset`, destructive ops |

### Bash Hook Scripts (per-agent, not global)

| Script | Used by | Allows |
|--------|---------|--------|
| `.claude/hooks/sdd-readonly-bash.sh` | reference-context, review | `git log/diff/status/show`, `ls`, `cat`, `find`, `cargo test/check` |
| `.claude/hooks/sdd-safe-bash.sh` | change-implementation | Everything except `rm -rf`, `git push/reset`, `chmod 777` |

### Phase-Specific CLAUDE.md

`run_agent()` generates tailored CLAUDE.md before spawning `claude --agent`:

| Phase | CLAUDE.md focus |
|-------|----------------|
| reference_context | Specs structure, knowledge paths, explore strategy |
| change_spec | Spec format rules, JSON Schema/Mermaid conventions |
| change_implementation | Code style, test requirements, file size limits |
| review | Review checklist, severity criteria |

## Spec Updates

| Spec | Change |
|------|--------|
| `config/agents.md` | Replace per-action config with `workflow.mode` + 4 preset tables |
| `logic/executor-resolution.md` | Update flowchart: mode → preset lookup → dispatch |
| `tools/utils/delegate-agent.md` | Add multi_claude_agents + claude_subagents execution paths |

## New Files

| File | Purpose |
|------|---------|
| `.claude/agents/sdd-reference-context.md` | Explorer agent (multi_claude_agents) |
| `.claude/agents/sdd-change-spec.md` | Spec writer agent (multi_claude_agents) |
| `.claude/agents/sdd-review.md` | Reviewer agent (multi_claude_agents) |
| `.claude/agents/sdd-change-implementation.md` | Implementer agent (multi_claude_agents) |
| `.claude/hooks/sdd-readonly-bash.sh` | Read-only Bash filter |
| `.claude/hooks/sdd-safe-bash.sh` | Safe Bash filter |

## Acceptance Criteria

- [ ] `config.toml` `workflow.mode`: `"multi_agents"` | `"multi_claude_agents"` | `"claude_subagents"` | `"mainthread"`
- [ ] Each mode has fixed phase-to-agent mapping — no per-action overrides
- [ ] `multi_agents`: gemini for specs, codex for reviews, mainthread for impl
- [ ] `multi_claude_agents`: `claude --agent sdd-X` with tool restrictions + Bash hooks
- [ ] `claude_subagents`: Claude Code Agent tool with model selection
- [ ] `mainthread`: all phases route to mainthread
- [ ] Agent failure → retry once → fallback to mainthread (all modes)
- [ ] 4 agent definitions in `.claude/agents/` for multi_claude_agents mode
- [ ] Per-agent Bash hooks only affect that agent
- [ ] Phase-specific CLAUDE.md generation for multi_claude_agents
