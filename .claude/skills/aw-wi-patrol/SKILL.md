---
name: aw:wi:patrol
description: Set up a cron to poll GitHub issues by label and auto-run the aw td CRRR loop when idle
user-invocable: true
aliases: [aw:issue-patrol]
---

# /aw:wi:patrol

Recurring cron that polls GitHub issues by label. When new work-items are
found and no SDD CRRR loop is in-flight, picks the highest-priority issue
and drives it through `aw td` (issue → TD → merge). Hand-written
implementation step is the operator's job; this skill stops after
`aw td merge`.

## Arguments

```
/aw:wi:patrol <label> [interval]
/aw:wi:patrol stop <job-id>
```

| Arg | Required | Default | Example |
|-----|----------|---------|---------|
| `label` | yes | — | `project:mamba`, `project:agentic-workflow`, `priority:p0` |
| `interval` | no | `5m` | `1m`, `5m`, `15m`, `30m`, `1h` |

To switch projects, change the `label`. Same skill, different patrol.

## Instructions

### 1. Parse arguments

- First positional → `label` (required — if missing, `AskUserQuestion`)
- Second positional → `interval` (optional, default `5m`)
- Convert interval to cron expression:
  - `Nm` → `*/N * * * *`
  - `Nh` → `7 */N * * *`

If first arg is `stop` → call `CronDelete` with the job-id and exit.

### 2. Create cron job

Use `CronCreate` with `recurring: true` and the prompt below. Replace
`<LABEL>` with the resolved label value.

```
You are a work-item patrol agent. Per-tick stateless: rediscover state
from git + GitHub every tick. One bounded action per tick, then exit.

## Pre-flight (skip tick if any fails)

1. `git status --porcelain` must be empty — dirty tree → log "dirty, skip" + STOP.
2. If `.aw/cron-patrol.halt` exists → log "halted by sentinel" + STOP.
3. `git fetch origin` — if the project working branch is behind, `git pull --ff-only`; on failure STOP.

## State scan — pick one slug to advance

Order matters: first match wins, do only that step, then STOP.

### A. In-flight tracking branch exists
Run:
    git for-each-ref --format='%(refname:short)' refs/heads/td-* refs/heads/issue-*
If any branch exists → that slug has CRRR in-flight.
- `td-<slug>` → `git checkout td-<slug>` and run `aw td create <slug>`.
- `issue-<slug>` → `git checkout issue-<slug>` and run `aw wi create <slug>` (or whatever next-phase verb the envelope wants).

Follow exactly ONE envelope step:
- `dispatch` → write the payload to `.aw/payloads/<slug>/<file>.md`
  (per the section the envelope names), then run `invoke.command`. If
  that command is `*--apply`, follow up with one `validate` so the phase
  advances. STOP after that.
- `done` → STOP. Next tick will pick up the next state.
- `error` → write `.aw/handoffs/<slug>-patrol-handoff.md` with the
  error text; `gh issue edit <N> --add-label flagged:needs-human`; STOP.

### B. td-<slug> already at phase `td_merged` but not merged into the working branch
(known `aw td merge` bug)
→ `git checkout <working-branch> && git merge --no-ff td-<slug> -m "merge td-<slug>"`,
delete the td branch, STOP.

### C. No in-flight branch — pick a new issue
Run:
    gh issue list --label "<LABEL>" --state open \
      --search "-label:type:epic -label:type:tracking -label:flagged:needs-human" \
      --json number,title,labels,url --limit 50

If empty → log "no open issues for <LABEL>" + STOP.

Sort: `priority:p0` > p1 > p2 > p3 > none; within tier pick the oldest
(lowest issue number). For each candidate, skip if
`.aw/handoffs/<slug>-patrol-handoff.md` exists with mtime < 24h.

Pick the winner `#N`. Then:
1. `git checkout <working-branch> && git checkout -b issue-<slug>`
   (slug = the issue's `slug:*` label if present, else numeric `<N>`).
2. Materialize the issue into the temp LocalBackend under `/tmp/aw`
   (frontmatter from `gh issue view <N> --json` + body from GitHub).
3. `git add` + commit `chore(aw): materialize #<N> for local td create`.
4. STOP. Next tick walks state A.

## Hard rules

- **At most one `git commit` per tick.**
- **5 min budget** per non-test step; `cargo test` / `cargo bench` get 30 min.
- Never `git push --force`, never delete `main` or `project-*` branches.
- Never run `aw cb` (codegen pipeline still experimental).
- Never open a PR to `main` — operator batches manually.
- Commit messages: conventional format, no `Co-Authored-By: Claude` trailer.

## Recovery

If a tick is killed mid-`--apply` (envelope ran but `validate` didn't), the
next tick will re-enter state A on the same branch and re-emit the
envelope — `aw td validate` is idempotent.
```

### 3. Confirm to user

After creating the cron, report:
- Label being watched
- Poll interval
- Cron job ID (for cancellation)
- Reminder: recurring jobs auto-expire after 7 days
- Reminder: stop with `/aw:wi:patrol stop <job-id>` or `CronDelete`
