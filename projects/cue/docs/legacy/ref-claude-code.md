# Claude Code Design Concepts - legacy reference for cue

Status: legacy reference.

This document was written for the old terminal SDD-runner direction. cue is now
a web-based Prompt-to-Governed-App platform, so these notes are not product
architecture. Keep only the generally useful harness ideas: tool gating, hooks,
structured output, audit events, and model-evaluation patterns.

This document distills key Claude Code architectural patterns that cue may still
borrow for enterprise governance and harness engineering.

## Tool Loop Architecture

**Flow**: `User prompt → Messages API call with tools → Claude reasons → tool_use block → Execute tool locally → tool_result → Append to conversation → Repeat until stop_reason="end_turn"`

- **Tool definitions**: Passed to API in `tools` array; Claude chooses which to call based on task
- **Streaming**: Streaming can return tokens + tool_use blocks in-flight; tool execution waits for all blocks before continuing
- **stop_reason values**: `"end_turn"` (normal stop), `"tool_use"` (more tools needed), `"max_tokens"`, `"stop_sequence"`
- **Result feedback**: Tool outputs feed directly back as `user` role messages (type `tool_result`); no re-prompting needed
- **No built-in retry logic**: Model decides if/when to retry a tool; hook layer can force decision

**Cue insight**: Model routing (Opus vs Sonnet vs Haiku per SDD phase) is transparent to tool loop — change the model param, same tools work.

## Subagent Dispatch (Agent Tool)

**Pattern**: Parent context calls `Agent` tool with subagent type + prompt → subagent runs in isolated context → returns summary.

- **Context isolation**: Subagent starts fresh; conversation history not shared; subagent reads only from CLAUDE.md + preloaded skills
- **Tool access**: Subagent has its own `allowed_tools` config; can be stricter than parent
- **Result format**: Subagent always returns a single summary message (text); no transcript sharing back
- **Parallel safe**: Each subagent worktree is independent; no shared file mutations until merge phase
- **Session-like**: Subagents persist in `~/.claude/projects/` same as sessions; can resume if interrupted

**Cue pattern**: Dispatch domain agents (jet-spec, mamba-spec) per SDD phase via envelope-driven CLI → subagent reads envelope, does work, returns status.

## Hooks System

**Lifecycle events** (execution points):
- **SessionStart**: When session begins; can set env vars via `CLAUDE_ENV_FILE`
- **PreToolUse**: Before tool execution; can block, deny, defer, or modify tool input
- **PostToolUse**: After tool success; can add context (e.g., linting feedback)
- **UserPromptSubmit**: Before Claude sees user input; can block or inject context
- **SubagentStart/Stop**: When subagents spawn/finish; can validate or log
- **Stop**: When Claude finishes turn; can block to force continuation

**Handler types**:
1. **Command** (shell script): Reads JSON on stdin, exits 0 (allow), 2 (block), or 1 (warn)
2. **HTTP**: POST JSON to endpoint; non-2xx is non-blocking
3. **Prompt**: Single-turn Claude eval; useful for policy checks
4. **Agent**: Spawn subagent for complex validation

**Matchers**: Filter which tools/events trigger hooks.
- `"Bash"`, `"Edit"` — exact match
- `"Bash(npm run *)"` — glob match (with word boundaries)
- `"mcp__server__.*"` — regex for MCP tool patterns
- `|` separator for multiple: `"Edit|Write"`

**settings.json structure**:
```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash(rm *)",
        "hooks": [
          {
            "type": "command",
            "command": "./.claude/hooks/block-destructive.sh",
            "if": "Bash(rm -rf)"
          }
        ]
      }
    ]
  }
}
```

**Key fields**: `type`, `command|url|prompt`, `timeout`, `if` (permission rule), `async`

**Cue insight**: Hooks tie SDD envelope validation to harness events; e.g., `SubagentStop` → run CLI `aw wi validate`.

## MCP (Model Context Protocol)

**Three primitives**:
1. **Tools** — callable functions (like API endpoints); Claude calls them
2. **Resources** — read-only data/files; Claude reads for context
3. **Prompts** — reusable instruction templates Claude can request

**Server types**:
- **Stdio**: Local process; tool definitions + execution over stdin/stdout
- **HTTP/SSE**: Remote server; WebSocket or SSE for streaming

**In Claude Code**:
- MCP servers configured in `.claude/settings.json` under `mcp.servers`
- Tools discovered at session start; full definitions deferred until tool is called (tool search)
- Tool names follow pattern `mcp__<server>__<tool>`

**Context cost**:
- Tool names always loaded (small)
- Full definitions loaded on first use only (large defs don't block startup)
- Resources not loaded unless Claude explicitly requests them
- Prompt templates stored server-side; descriptions loaded

**Cue insight**: MCP is how cue connects domain-specific tools (cclab-cli, score binary, etc.) as first-class agent capabilities.

## Slash Commands / Skills

**Location**: `.claude/skills/<name>/SKILL.md` or `.claude/commands/<name>.md`

**Frontmatter schema** (YAML between `---` markers):
```yaml
---
name: fix-issue
description: Fix a GitHub issue by number
disable-model-invocation: true
allowed-tools: Bash(gh *) Read Edit Write
argument-hint: "[issue-number]"
arguments: [issue]
context: fork
agent: Explore
---
```

**Key fields**:
- `name`: Becomes `/slash-command` (lowercase, hyphens)
- `description`: When Claude auto-invokes; front-load key use case
- `disable-model-invocation`: true = only manual invoke via `/name`
- `allowed-tools`: Tools pre-approved when skill active
- `arguments`: Named args for `$name` substitution in body
- `context: fork`: Run in isolated subagent; requires `agent` type

**Parameter passing**:
- `$ARGUMENTS` → full arg string
- `$0`, `$1`, `$ARGUMENTS[N]` → positional args
- `$name` → named arg (from `arguments:` list)
- `` !`shell command` `` → run command, inject output (preprocessing, not Claude-executed)

**Auto-invocation**: Claude reads skill descriptions at session start; if description matches current task, Claude auto-invokes with matching arguments.

**Cue pattern**: `/aw:td:init`, `/aw:td:create` are skills that dispatch domain agents + envelope CLI → subagents own spec modifications, mainthread validates + commits.

## Permission Model

**Three rule types**:
- **allow**: Auto-approve; no prompt (most tools don't prompt unless asking)
- **ask**: Prompt user each time
- **deny**: Block; cannot be overridden by allow

**Evaluation order**: Deny wins → Ask → Allow (first match applied)

**Tool-specific syntax**:
- `Bash` / `Bash(*)` → all bash
- `Bash(npm run *)` → exact prefix match with word boundary (space before `*`)
- `Bash(rm -rf /*)` → destructive rm (compound cmds split; each subcommand matched separately)
- `Edit(src/**)` → gitignore-style patterns (relative to project root)
- `Read(~/.ssh/**)` → home-relative paths
- `WebFetch(domain:example.com)` → domain filtering
- `mcp__memory__*` → all tools from MCP server
- `Agent(Explore)` → specific subagent

**Process wrappers auto-stripped**: `timeout`, `time`, `nohup`, `xargs` (bareword only); rule `Bash(npm *)` matches `timeout 30 npm test`

**Symlink handling**:
- Allow rules: check both symlink + target path (stricter)
- Deny rules: block if either path matches (safer)

**Cue insight**: Score commands (marked in permissions as `Bash(score *)`) need careful allow rules to avoid prompting on every invocation.

## Settings Hierarchy

**Scopes (precedence high→low)**:
1. **Managed** (`~/.anthropic/managed-settings.json` or system policy) — cannot override; org enforcement
2. **Command line** (`--allowed-tools`, `--disallowed-tools`, etc.) — session only
3. **Local** (`.claude/settings.local.json`) — gitignored; personal per-project overrides
4. **Project** (`.claude/settings.json`) — committed to repo; team-shared config
5. **User** (`~/.claude/settings.json`) — personal global; lowest priority

**Merge behavior**:
- **Objects**: Deep merge; lower-scope fields fill gaps
- **Arrays** (permissions, MCP servers, etc.): **Merge, not replace** — allows are combined, denies are combined
- **Conflict rule**: Deny at any scope blocks (deny precedence)

**Example conflict**:
```
User: "allow": ["Bash(npm *)"]
Project: "deny": ["Bash(npm *)"]
Result: npm blocked (project deny wins)
```

**Load order**: User → Project → Local → Managed; managed always evaluated last + can't be overridden

**Verify with** `/status` — shows active scopes + sources.

**Cue insight**: Share `.claude/settings.json` in repo for team consistency (permissions, hooks, MCP servers); `.claude/settings.local.json` for personal tweaks (env vars, extra permissions for local testing).

## Session / Context Management

**Session storage**: `~/.claude/projects/<project>/<session-id>/` as plaintext JSONL (messages + tool calls + results)

**Context window**:
- Limit: model-dependent (claude-opus-4-1 = ~200k tokens)
- Fills: conversation + file contents + command outputs + CLAUDE.md + auto memory
- **Auto-compaction**: Clears old tool outputs first, then summarizes conversation; preserves user requests + key code
- **Manual compaction**: `/compact focus on <topic>` to guide summarization
- **MCP overhead**: Tool definitions deferred; tool names always loaded (~small). Run `/mcp` to check per-server cost

**CLAUDE.md loading**:
- First 200 lines or 25KB loaded at session start
- Sections marked "Compact Instructions" preserved during compaction
- Can include path-specific rules to load different context per directory

**Skills context cost**:
- Descriptions always in context (to trigger auto-invocation)
- Full content only loaded when skill is used
- After compaction, most-recent skills re-attached (budget 25KB)

**Sessions are independent**:
- Each new session starts fresh
- `/resume` picks up conversation history from previous session
- `--fork-session` creates new session ID while preserving history

**Cue insight**: Subagents reduce bloat in mainthread context by isolating research/exploration; mainthread stays focused on envelope dispatch + validation.

---

## Takeaways for Cue

1. **Tool loop is transparent to orchestration**: Model routing (switching Opus↔Sonnet↔Haiku per phase) doesn't break agentic loop or tool calling. Hook into PreToolUse/PostToolUse to enforce SDD constraints.

2. **Subagent isolation is key**: Don't share conversation history with domain agents. Use envelope + forked subagent context to keep mainthread clean; subagents return only summary.

3. **Hooks bridge CLI + agentic decisions**: PreToolUse hooks can validate envelope state before mainthread dispatches phase CLI; SubagentStop hooks can trigger CLI `validate` after agent completes (though async Agent dispatch makes hooks unreliable—prefer explicit CLI call).

4. **MCP is extension point for domain tools**: cclab-cli, score binary, jet/mamba generators all register as MCP tools or Bash permission rules; Claude treats them same as built-ins.

5. **Settings hierarchy enables team+personal split**: Share `.claude/settings.json` (MCP servers, project hooks, team permissions) in repo; use `.claude/settings.local.json` for personal env-var overrides. Deny rules at project scope enforce mandatory constraints.

6. **Skills + skills directory discovery** enable playbook reuse: Store shared SDD workflows (issue create, spec review, code gen) as skills under `.claude/skills/`; nested discovery auto-loads per-package variants in monorepos.

7. **Context management via subagents + compaction** beats bloat: Keep mainthread lean by delegating research to Explore subagent; use skill skill descriptions (not full content) to signal Claude; preserve CLAUDE.md across compaction for persistent rules.
