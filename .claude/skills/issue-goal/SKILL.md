---
name: issue-goal
description: Drive a project's GitHub issue backlog to zero — one issue per iteration, mainthread-driven, runs continuously until the backlog is empty or the user interrupts. Reads projects/<name>/issue-loop.md for per-project rules (label, branch, build policy, test/perf gates, PR strategy).
user-invocable: true
---

# /issue-goal

Synchronous, mainthread-driven backlog driver. One iteration = one issue picked → implemented → tested → PR'd → squash-merged → rebased back onto the working branch. Loops until `gh issue list` returns 0 for the configured label, or the user interrupts.

**Sibling skill**: `/issue-loop` is the cron/background variant of this same flow. Use `/issue-goal` for an interactive sprint; use `/issue-loop` to leave it running unattended.

## Arguments

```
/issue-goal <project>
```

| Arg | Required | Example |
|-----|----------|---------|
| `project` | yes | `jet`, `mamba`, `agentic-workflow` |

If `project` is omitted, detect it from the current branch (`project-<name>` → `<name>`); if still ambiguous, `AskUserQuestion`.

## Pre-flight (do once before entering the loop)

1. `git status --porcelain` — must be clean. If dirty, stop and report.
2. Load config from `projects/<project>/issue-loop.md` (frontmatter). Required keys: `branch`, `label`, `repo`, `verify`, `done_gates`, `pr`.
3. Ensure on the configured `branch` (default `project-<project>`). If on a different branch:
   - If current branch is a feature branch with unpushed work → merge it into the working branch first (don't escape to a side branch — the project working branch IS the working branch).
   - If current branch is `main` or another project's branch → `git checkout <working-branch>`.
4. `git fetch origin && git pull --ff-only origin <working-branch>` (or rebase if FF fails — investigate before forcing).
5. Read the per-issue rules in `issue-loop.md` body (build policy, when-to-dispatch-subagent, known quirks, PR body checklist) and apply them throughout the loop.

## Iteration (repeat until exit condition)

### 1. Pick one issue

```
gh issue list --label "<label>" --state open \
  --search "-label:type:epic -label:type:tracking -label:flagged:needs-human" \
  --json number,title,labels,url --limit 50
```

If empty → **exit successfully**: report "backlog clear for label `<label>`".

Sort per config `pick_order`:
- `oldest-first` (default) → lowest issue number first.
- `priority` → `priority:p0` > p1 > p2 > p3 > none; within tier, oldest first.

Skip a candidate if:
- It has `flagged:needs-human` or `blocked:*` labels.
- `.aw/handoffs/<n>-patrol-handoff.md` exists with mtime < 24h (recent failure).
- It's already been deferred in this session (track in-conversation, don't re-pick).

### 2. Plan + implement

- Read the issue body via `gh issue view <n>`.
- Decide: mainthread inline (triage, single-file 1-3 line fix, label/comment work) vs. dispatch a subagent (multi-file refactor, full stdlib shim, bench authoring, anything >10 focused tool calls). Follow the per-project rules in `issue-loop.md` ("When to dispatch a subagent" section if present).
- Apply known runtime quirks documented in the project's `issue-loop.md` (e.g. mamba's `import` quirks, JIT branch drops, integer-handle pattern).
- **Don't run any build command the project marks as `build: skip`.** Rely on the configured `verify.test` command for compile-gate.

### 3. Run done gates

For each gate in `done_gates`:
- `test_passes` → run `verify.test`, must exit 0.
- `perf_verified` / `perf_ge_cpython` → run the perf gate per project rules. For pure refactor/docs/UI-state with no hot-path impact, explicitly document "perf N/A because …" in the PR body — never silently skip.
- `realworld_or_typeshed_verified` → run the configured real-world fixture or typeshed check.

If a gate fails:
- If recoverable in a few edits → fix and re-run.
- If the failure is a known pre-existing blocker (per `issue-loop.md` "Known blockers" section) → comment on the issue referencing the blocker, label `blocked:<reason>`, and **move to the next issue**. Don't loop on a stuck issue.
- If perf gate shows the lib can't beat baseline on any reasonable workload (mamba-style) → comment with numbers, close as `wontfix` or defer per project rules. Move on.

### 4. Commit + PR + merge + rebase

1. Commit on the working branch. Conventional format. **No `Co-Authored-By: Claude` / `🤖 Generated with Claude Code` trailer** (global rule).
2. Push the working branch.
3. `gh pr create --base <pr.base> --title "<conventional title>" --body "$(cat <<'EOF' ... EOF)"`. PR body must include:
   - `Closes #<n>`
   - Test gate output (passing `cargo test` summary line).
   - Perf gate output OR explicit "perf N/A because …" line.
   - Any project-specific items from `issue-loop.md` "PR body must include" section.
4. Wait for required CI checks (if the repo has them) — `gh pr checks <pr-url> --watch`. If checks fail, fix and force-push the branch; never `--force-with-lease` over someone else's commits without checking.
5. Merge per `pr.merge_strategy` (`squash` default): `gh pr merge <pr-url> --squash --delete-branch=false` (keep the working branch).
6. If `pr.rebase_after_merge: true`: `git fetch origin && git pull --rebase origin main` (back onto the working branch). Push.

### 5. Continue

Loop back to step 1.

## Exit conditions

- **Success**: `gh issue list` returns empty for the label.
- **User interrupt**: report current state (last issue closed, next candidate) so the user can resume.
- **Hard error budget**: if 3 consecutive issues fail in unexpected ways (not a known blocker), stop and report — something systemic is wrong.

## Hard rules

- **Never** `git push --force` to `main` or any `project-*` branch without explicit user confirmation.
- **Never** skip pre-commit / pre-push hooks (`--no-verify`).
- **Never** commit with the `Co-Authored-By: Claude` trailer (global rule).
- **Never** silently skip a done gate — either run it, or explicitly document N/A in the PR body with a reason.
- **One issue per PR.** Don't batch issues unless the project's `issue-loop.md` explicitly opts in.
- If `aw td` / `aw cb` lifecycle is required for an issue, run it on **mainthread** — the PostToolUse hook lockfile chain doesn't reach subagent Bash (see memory `feedback_no_subagent_for_aw_crrr.md`).

## Recovery

If the loop is interrupted mid-iteration:
- Dirty tree, no PR yet → next invocation's pre-flight will stop and ask. User can commit/discard.
- PR open, not merged → next invocation will see no in-flight issue but a stale PR; resume by checking PR state with `gh pr list --author @me --base main`.
- Merged but not rebased → next invocation's pre-flight pulls `--ff-only`; if non-FF, rebase manually.

## Status reporting

After each issue completes, emit a one-line status to the user:
```
✓ #<n> <title> — merged as <pr-url>, <N> issues remaining
```

Before exiting (success or interrupt), emit a summary:
```
Closed <K> issues this session. Remaining for label `<label>`: <N>.
```
