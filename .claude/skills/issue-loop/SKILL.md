---
name: issue-loop
description: Schedule a recurring cron that drives a project's GitHub issue backlog. Each tick = one bounded action (pick / implement / gate / PR / merge / rebase — one of these per tick). Reads projects/<name>/issue-loop.md for per-project rules. Stops when the backlog is empty or the user halts via sentinel file.
user-invocable: true
aliases: [issue-patrol]
---

# /issue-loop

Recurring cron variant of `/issue-goal`. Polls GitHub by label and drives **one bounded action per tick**. Designed to run unattended over hours/days, surviving Claude Code restarts and context resets — each tick is **stateless** and rediscovers state from git + GitHub.

**Sibling skill**: `/issue-goal` is the synchronous mainthread version of this flow. Use `/issue-goal` for an interactive sprint; use `/issue-loop` to leave it running.

## Arguments

```
/issue-loop <project> [interval]
/issue-loop stop <job-id>
```

| Arg | Required | Default | Example |
|-----|----------|---------|---------|
| `project` | yes | — | `jet`, `mamba`, `agentic-workflow` |
| `interval` | no | `5m` | `1m`, `5m`, `15m`, `30m`, `1h` |

To switch projects, change the `project`. Same skill, different patrol.

## Instructions

### 1. Parse arguments

- First positional → `project` (required — if missing, `AskUserQuestion`).
- Second positional → `interval` (optional, default `5m`).
- Convert interval to cron expression:
  - `Nm` → `*/N * * * *`
  - `Nh` → `7 */N * * *`

If first arg is `stop` → call `CronDelete` with the job-id and exit.

### 2. Validate project config exists

Verify `projects/<project>/issue-loop.md` exists and has the required frontmatter keys (`branch`, `label`, `repo`, `verify`, `done_gates`, `pr`). If missing, refuse to create the cron and tell the user what to add.

### 3. Create cron job

Use `CronCreate` with `recurring: true` and the prompt below. Replace `<PROJECT>` with the resolved project name and `<INTERVAL>` with the literal interval string for logging.

```
You are an issue-backlog patrol agent for project `<PROJECT>`. Per-tick
stateless: rediscover state from git + GitHub every tick. One bounded
action per tick, then exit.

## Load project config

Read `projects/<PROJECT>/issue-loop.md`. Use its frontmatter for:
- `branch` — the working branch (e.g. `project-<PROJECT>`)
- `label` — the GitHub label to poll (e.g. `project:<PROJECT>`)
- `repo` — owner/repo (e.g. `chrischeng-c4/cclab`)
- `pick_order` — `oldest-first` or `priority` (default oldest-first)
- `verify.test` — the test command for the done gate
- `verify.perf` — perf gate policy
- `done_gates` — list of gates required before merge
- `pr.base`, `pr.merge_strategy`, `pr.rebase_after_merge` — PR strategy
- `build` — build policy (often `skip` for jet, tiered for mamba)

Apply per-project conventions from the body (known quirks, subagent
dispatch rules, PR body checklist).

## Pre-flight (skip tick if any fails)

1. `git status --porcelain` must be empty — dirty tree → log "dirty, skip" + STOP.
2. If `.issue-loop/<PROJECT>.halt` exists → log "halted by sentinel" + STOP.
3. `git fetch origin` — if working branch is behind, `git pull --ff-only`; on failure STOP.
4. Verify current branch == `branch` from config. If not, `git checkout <branch>`; if checkout fails (dirty / missing), STOP.

## State scan — pick one action, then STOP

Order matters: first match wins, do only that step, then exit.

### A. PR open from previous tick, not yet merged

Run:
    gh pr list --author @me --base <pr.base> --head <branch> \
      --state open --json number,url,statusCheckRollup,mergeable

If a PR exists:
- If checks still running → log "PR #<n> checks pending" + STOP.
- If checks passed and mergeable → `gh pr merge <pr-url> --<merge_strategy>`,
  then `git pull --rebase origin <pr.base>` if `pr.rebase_after_merge` is true,
  push, STOP.
- If checks failed → write `.aw/handoffs/<n>-patrol-handoff.md` with the
  failure summary; `gh issue edit <n> --add-label flagged:needs-human`; STOP.
- If conflicts → STOP and flag (don't auto-resolve in cron context).

### B. Local working branch ahead of origin (unpushed commits)

Run `git rev-list --count origin/<branch>..HEAD`. If > 0:
- `git push` → STOP.

### C. No in-flight PR — pick a new issue

Run:
    gh issue list --label "<label>" --state open \
      --search "-label:type:epic -label:type:tracking -label:flagged:needs-human -label:blocked:*" \
      --json number,title,labels,url --limit 50

If empty → log "no open issues for <label>, backlog clear" + STOP.

Sort per `pick_order`:
- `oldest-first` → lowest issue number first.
- `priority` → `priority:p0` > p1 > p2 > p3 > none; tiebreak oldest first.

Skip a candidate if `.aw/handoffs/<n>-patrol-handoff.md` exists with
mtime < 24h.

Pick the winner `#N`. Read its body via `gh issue view <N>`. Make a
**single bounded change**:

- If the issue can be resolved in one focused edit (1-3 files, <30 min of
  work), do it inline on the current branch:
  1. Implement the change.
  2. Run `verify.test` — must pass.
  3. Run perf gate per `verify.perf` and `done_gates` (or document N/A).
  4. Commit with conventional message (NO `Co-Authored-By: Claude` trailer).
  5. `gh pr create --base <pr.base> --title "<title>" --body "<body with Closes #N>"`.
  6. STOP. Next tick walks state A.

- If the issue needs more than one tick of work (multi-file, full stdlib
  shim, complex bench authoring):
  1. Write a scoping comment on the issue with the planned approach.
  2. Label it `in-progress:cron` so future ticks skip it.
  3. STOP. Operator picks it up via `/issue-goal <PROJECT>` for sustained work.

## Hard rules

- **At most one `git commit` per tick** (and at most one `gh pr create`, `gh pr merge`, or `git push`).
- **5 min budget** per non-test step; `cargo test` / bench commands get 30 min.
- Never `git push --force`, never delete `main` or `project-*` branches.
- Never skip pre-commit / pre-push hooks (`--no-verify`).
- Never commit with `Co-Authored-By: Claude` / `🤖 Generated with Claude Code` trailer (global rule).
- Don't run any build command the project marks as `build: skip`.
- If the issue requires `aw td` / `aw cb` lifecycle → defer to operator via `in-progress:cron` label (subagent Bash can't reach the PostToolUse hook lockfile chain; cron tick is effectively a subagent context).

## Recovery

If a tick is killed mid-action:
- Dirty tree → next tick's pre-flight (1) skips and logs.
- Commit landed but push didn't → next tick's state scan (B) pushes.
- PR opened but message malformed → operator fixes manually.
```

### 4. Confirm to user

After creating the cron, report:
- Project being watched (`<PROJECT>`)
- Label (`<label>` from config)
- Poll interval (`<INTERVAL>`)
- Cron job ID (for cancellation)
- Halt sentinel path: `.issue-loop/<PROJECT>.halt` (create this file to pause without deleting the cron)
- Reminder: recurring jobs auto-expire after 7 days
- Reminder: stop with `/issue-loop stop <job-id>` or `CronDelete`

## Stop

`/issue-loop stop <job-id>` → `CronDelete` with the id. Confirm to user.

## Pairing with `/issue-goal`

The two skills share the same `projects/<name>/issue-loop.md` config. Common pattern:
- `/issue-loop jet 15m` → leave running overnight, picks up easy issues.
- `/issue-goal jet` next morning → sweep the harder remaining issues interactively.

Don't run both simultaneously on the same project — they'll race on PR creation. The cron tick checks for in-flight PRs (state A) and would pick them up, but `/issue-goal` doesn't know about cron PRs, so disable the cron (`/issue-loop stop`) before starting `/issue-goal`.
