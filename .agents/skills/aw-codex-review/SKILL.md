---
name: aw:codex:review
description: Run Codex headless to review code or changes
user-invocable: true
---

# /aw:codex:review

Dispatches a review prompt to Codex CLI headlessly. Use this when you need a second opinion on code quality, security, correctness, or spec compliance.

## Usage

```
/aw:codex:review "<prompt>"
```

## Instructions

1. Parse the user's prompt. If empty, default to reviewing staged git changes:
   - Run `git diff --cached --stat` to check for staged changes
   - If staged changes exist, use: `"Review the staged changes for correctness, security, and code quality."`
   - If no staged changes, use: `"Review the recent changes in this repository for correctness, security, and code quality."`

2. Run Codex CLI via Bash (read-only review mode — no file writes):

```bash
codex review -c model=gpt-5.4 -c reasoning=medium "<prompt>"
```

3. Present the review findings to the user.

## Examples

```
# Review staged changes (default)
/aw:codex:review

# Review specific file
/aw:codex:review "Review src/auth.rs for security vulnerabilities"

# Review architecture
/aw:codex:review "Review the crate dependency graph for circular dependencies"

# Review a PR
/aw:codex:review "Review the diff between main and HEAD for breaking changes"
```

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
