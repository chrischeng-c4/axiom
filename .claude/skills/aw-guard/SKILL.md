---
name: aw:guard
description: Enable, disable, or explain AW agent-runtime direct edit/create guards for Codex and Claude Code. Use when the user asks to lock or unlock a project from agent direct writes, install Codex/Claude hooks, prevent direct Edit/Write/apply_patch changes, or debug why an agent edit was denied.
user-invocable: true
aliases: ["aw:agent-guard", "aw:hooks"]
---

# /aw:guard

Use this skill for AW's agent-runtime guard layer. This is not a git hook and
not an OS-level lock. It installs Codex/Claude Code lifecycle hooks that block
direct agent edit/create tools inside the selected AW project scope.

## Commands

Prefer the repo-built binary when working inside `project-aw` after a fresh
build; otherwise use installed `aw`.

```bash
aw guard on --project <project>
aw guard off --project <project>
aw guard on --project <project> --agent codex
aw guard on --project <project> --agent claude
```

`on` writes AW-managed hook handlers:

- Codex: `.codex/hooks.json`, `PreToolUse`, matcher `Edit|Write|apply_patch`.
- Claude Code: `.claude/settings.json`, `PreToolUse`, matcher `Edit|Write|MultiEdit|NotebookEdit`.

`off` removes only AW-managed `aw guard pretool ...` handlers for that project.
It must preserve unrelated hooks and settings.

## Policy

- Guard only direct edit/create tools: Codex `apply_patch` and Claude
  `Edit`/`Write` family tools.
- Do not add Bash, sed, tee, `>`, or `>>` blocking unless the user explicitly
  asks for a stronger policy. Blocking shell writes can deadlock work when the
  AW CLI cannot yet express a needed mutation.
- Hook failures are fail-open. A broken hook should warn but should not silently
  block all editing.
- A denied edit means: use the AW CLI lifecycle, or explicitly run
  `aw guard off --project <project>` before a manual bypass.
- Guard does not replace EC/TD locks. Use `aw ec lock`, `aw td lock`, and their
  clean checks for artifact-source consistency.

## Hook Entry Point

The generated hook command is:

```bash
aw guard pretool --agent <codex|claude> --project <project>
```

Do not run `pretool` directly except for smoke tests with a JSON PreToolUse
payload on stdin. A deny response should be JSON with
`hookSpecificOutput.permissionDecision = "deny"`. An allowed payload should
exit 0 with no stdout.

## Validation

After changing guard implementation or this skill, run the focused checks:

```bash
cargo fmt -p agentic-workflow --check
cargo test -p agentic-workflow --lib guard -- --nocapture
cargo build -p agentic-workflow --bin aw
./target/debug/aw guard --help
```

For behavior smoke, feed one Codex `apply_patch` payload targeting the project
and expect a deny JSON. Feed one Bash `sed` payload and expect exit 0 with no
stdout.
