---
name: codex-review
description: Run `codex review` (headless Codex CLI) with a custom prompt. Use when the user asks to review specs, code, or design decisions with codex, providing context about what to focus on.
disable-model-invocation: true
allowed-tools: Bash
---

Run a codex review with the prompt provided by the user.

## Current repo state
- Working dir: !`pwd`
- Changed files: !`git diff --name-only && git diff --name-only --cached`

## Task

Run:

```bash
codex review "$ARGUMENTS"
```

Wait for it to complete and display the full review output.

If `$ARGUMENTS` is empty, ask the user to provide a prompt. Example:

```
/codex-review Review specs/workflows/project-spec/ — focus on whether the agent/prompt separation is consistent with platform/workflow/schema.md
```
