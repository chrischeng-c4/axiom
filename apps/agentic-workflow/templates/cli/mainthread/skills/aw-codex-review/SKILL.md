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
