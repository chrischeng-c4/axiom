---
name: git:commit
description: Look at diff and commit with a meaningful message
user-invocable: true
---

# /git:commit

Review the current diff and create a commit with a meaningful conventional commit message.

## Instructions

### Step 1: Check for changes

```bash
git status --porcelain
git diff HEAD
```

If there are no changes, tell the user **"Nothing to commit."** and stop.

### Step 2: Analyze the diff

Read the diff output and determine:
- What changed (files, functions, logic)
- Why it changed (feature, fix, refactor, docs, test, chore)
- The scope (which module/crate/component)

### Step 3: Stage and commit

Stage all changed files and commit with a conventional commit message:

```
<type>(<scope>): <short summary>
```

- Use `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `perf`, `ci` as type
- Keep the subject line under 72 characters
- Add a body only if the change is non-trivial
- Do NOT add `Co-Authored-By` or generated-with lines
