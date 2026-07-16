---
name: aw:guard
description: Enable, disable, or explain AW agent-runtime direct edit/create guards for Codex, Claude Code, and AGY. Use when the user asks to lock or unlock a project from agent direct writes, install hooks, prevent direct edits, or debug why an agent edit was denied.
user-invocable: true
aliases: ["aw:agent-guard", "aw:hooks"]
---

# /aw:guard

Use this skill for AW's agent-runtime guard layer. This is not a git hook and
not an OS-level lock. It installs Codex, Claude Code, and AGY lifecycle hooks
that block direct agent edits inside the selected AW project scope.

## Commands

Prefer the repo-built binary when working inside `app_aw` after a fresh
build; otherwise use installed `aw`.

```bash
aw guard on --project <project>
aw guard off --project <project>
aw guard on --project <project> --agent codex
aw guard on --project <project> --agent claude
aw guard on --project <project> --agent agy
aw guard bypass --project <project> [--minutes 30]
aw guard resume --project <project>
```

`on` writes AW-managed hook handlers:

- Codex: `.codex/hooks.json`, `PreToolUse`, matcher `Edit|Write|apply_patch`.
- Claude Code: `.claude/settings.json`, `PreToolUse`, matcher `Edit|Write|MultiEdit|NotebookEdit`.
- AGY: the local user configuration at `~/.gemini/config/hooks.json` (or its
  legacy `~/.gemini/antigravity-cli/hooks.json` fallback), `PreToolUse`, matcher
  `run_command`.

`on` and `off` are persistent policy changes. Codex/Claude config changes are
committed immediately, scoped only to their changed hook file(s), and the
command refuses to run if those files were already dirty. AGY's user-global
adapter is intentionally not committed. `off` removes only AW-managed
`aw guard pretool ...` handlers for that project and preserves unrelated hooks
and settings.

For a temporary local escape hatch, use `bypass`, never `off`. It writes an
auto-expiring TTL record in the current worktree's Git metadata; `resume`
clears it early without changing the committed guard policy.

## Policy

- Guard only direct edit/create tools: Codex `apply_patch`, Claude
  `Edit`/`Write` family tools, and AGY's explicit direct mutations through
  `run_command` (`touch`, `tee`, redirection, `sed -i`, `cp`, `mv`, `install`,
  and `rm`).
- AGY shell commands that do not present one of those explicit target forms are
  allowed. AW guard is not a general shell policy; the global destructive shell
  guard remains CAP's responsibility.
- Hook failures are fail-open. A broken hook should warn but should not silently
  block all editing.
- A denied edit means: use the AW CLI lifecycle, or explicitly run
  `aw guard bypass --project <project>` for a short manual bypass.
- Guard does not replace EC/TD locks. Use `aw ec lock`, `aw td lock`, and their
  clean checks for artifact-source consistency.

## HANDWRITE fill still works with guard on

Guard scopes denial to the project's registered `path` / `td_path` /
`cap_path` / workspace globs (see `GuardScope` in `src/cli/guard.rs`).
`/tmp/aw/workspaces/<workspace>/payloads/<slug>/...` lives under `/tmp`,
outside every project's registered scope, so writing a HANDWRITE marker
payload there is always allowed even with guard on. The
`aw td fill <slug> --apply --marker <id>`
call that actually merges the payload into the guarded HANDWRITE block runs
as a Bash-invoked binary, not as an `Edit`/`Write`/`MultiEdit`/`NotebookEdit`/
`apply_patch` tool call — guard's PreToolUse hook only intercepts those tool
names, never Bash, so the merge is never denied. This is the intended
"use the AW CLI lifecycle" path from the deny message, not a bypass.

## Hook Entry Point

The generated hook command is:

```bash
aw guard pretool --agent <codex|claude|agy> --project <project>
```

Do not run `pretool` directly except for smoke tests with a JSON PreToolUse
payload on stdin. Codex/Claude deny responses carry
`hookSpecificOutput.permissionDecision = "deny"`; their allowed payloads exit
0 with no stdout. AGY always requires stdout JSON: `{"decision":"allow"}` or
`{"decision":"deny","reason":"..."}`.

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

## AW CLI Drift & Defect Reporting

`aw` changes frequently. If this skill's documented invocation, result shape, or
semantics contradict the current `aw --help` output or CLI envelope, treat that
as a suspected AW defect; do not silently invent a compatibility command or
work around it.

Before reporting, reproduce the smallest failing command and capture the `aw`
version, exact command, expected result, actual stdout/stderr, and any relevant
envelope fields. Confirm the current surface with the relevant `aw <verb>
--help`; when working on AW itself, prefer a freshly built
`target/debug/aw` if the installed binary could be stale.

Once confirmed, report an AW-owned defect with `aw issue create --title "aw:
<short symptom>" "<reproduction and evidence>"`. Do not pass `--yes` unless
GitHub writes are already authorized. Expected validation failures or defects
owned by the target project belong in that project's tracker, not as AW bugs.
