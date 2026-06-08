---
name: claude-review
description: Run `claude -p` (headless Claude Code) with a custom review prompt. Use when the user asks to review specs, code, or design decisions with Claude Code in headless/programmatic mode, providing context about what to focus on.
disable-model-invocation: true
allowed-tools: Bash
---

Run a headless Claude Code review with the prompt provided by the user.

## Current repo state
- Working dir: !`pwd`
- Changed files: !`git diff --name-only && git diff --name-only --cached`

## Task

Run:

```bash
claude \
  --allowedTools "Read Grep Glob LS WebFetch WebSearch NotebookRead Bash(git diff:*) Bash(git log:*) Bash(git show:*) Bash(git status:*) Bash(git blame:*) Bash(ls:*) Bash(cat:*) Bash(rg:*) Bash(find:*)" \
  --disallowedTools "Edit Write NotebookEdit MultiEdit" \
  -p "$ARGUMENTS"
```

Notes:
- `-p "$ARGUMENTS"` MUST come after the variadic `--allowedTools` / `--disallowedTools` flags. The variadic `<tools...>` parser stops at the next `-`-prefixed flag, so `-p` cleanly terminates it. If you put the prompt before the variadic flags as a bare positional, it gets swallowed and `claude` errors with `Input must be provided either through stdin or as a prompt argument`.
- The allow/deny lists make the headless review read-only — no file edits, no shell mutations, only inspection commands.

Wait for it to complete and display the full review output.

If `$ARGUMENTS` is empty, ask the user to provide a prompt. Example:

```
/claude-review Review specs/workflows/project-spec/ — focus on whether the agent/prompt separation is consistent with platform/workflow/schema.md
```
