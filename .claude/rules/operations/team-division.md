# Team division: main session × subagents × aw-* skills

## Intent

Fix which actor runs each `aw-*` skill and each `wi → e2e → impl` phase, so
the e2e contract stays with the strongest model, implementation stays cheap,
and every step that needs a human runs where a human can answer.

## Rules

- The main session (controller) exclusively runs `/aw-grill-me-to-meta`,
  `/aw-grill-meta-to-milestone`, `/aw-grill-milestone-to-issue`,
  `/aw-prepare-goal` and `/aw-ask-user`. The grills and ask-user interview
  through AskUserQuestion; subagents do not have that tool, and a subagent
  running them anyway is answering for the human.
- The controller also keeps dispatch scheduling, final acceptance, git land,
  tracker semantic decisions, AGY payload authorization, and long read-only
  investigations (the per-app research agents were deleted on 2026-09-02).
  Never delegate these.
- The fleet is two agents per project — 22 apps and 22 libs — plus `aw-dev`
  for `apps/aw` (Python; by decision it has no e2e agent):
  - `<p>-e2e-dev` (opus/max) owns the **e2e contract**: black-box cases
    covering the behavior, performance, and security facets, written to fail
    before the implementation exists. For apps it runs `/aw-e2e-for`'s four
    verbs itself, and the phase script's `commit` is its only Git write. For
    libs there is no ladder — `leg.leg_root` resolves under `apps/` only —
    so the lib e2e-dev authors and runs the cases directly and the
    controller owns the commit.
  - `<p>-dev` (sonnet/medium) owns **source + colocated unit tests** and
    verifies the source by running them. For apps it runs `/aw-impl-for`'s
    five verbs: unit tests land red first, `red` records that attribution
    evidence to `.aw/impl-red/<iid>.json`, `C2` refuses a drifted tree, and
    `C0` refuses an impl commit that touches no test file. Maintenance queue
    heads route through the same skill's maint leg. For libs it implements
    directly, verifies with `cargo test -p <crate> --lib` then the full
    crate suite, and the controller owns the commit.
- Weak self-serving unit tests are caught by the e2e-dev's independent
  cases, which impl has to turn green while they still refuse HEAD.
- The model tiers above are defaults, not ceilings. For a hard case the
  controller may raise the model at dispatch time via the Agent call's
  `model` override — without editing the agent definitions. Phase ownership
  does not move with the model.
- One writer per worktree at a time. Phase scripts measure named reds against
  HEAD, so two concurrent writers poison each other's baseline. Cross-app
  parallelism means separate persistent-branch worktrees (`app/<name>`,
  `lib/<name>`; see `.claude/rules/operations/persistent-branches.md`) or a
  hand-driven AGY dispatch.
- Phase `commit` is run by that phase's runner (apps only) — the script
  re-runs every gate before writing — and the controller's acceptance reads
  the commits, not the runner's summary. Lib work has no phase script, so
  every lib commit is the controller's.
- A dev stalled twice on the same task is not re-dispatched; the controller
  takes over directly.

## Verification

- The fleet is 44 `<p>-e2e-dev` agents plus 45 `<p>-dev` agents (44 crates
  and `aw-dev`), and only the 22 app-level pairs carry the ladder carve-out.
  The `*-dev.md` glob also matches the e2e agents, so the dev set excludes
  them by name; and the carve-out greps must match the frontmatter
  `skills:` entry, not the bare skill name — the lib agents' no-ladder note
  mentions `/aw-e2e-for` and `/aw-impl-for` in prose, so a bare grep counts
  all 44. All four numbers below are the point — a count of matches alone
  cannot tell a complete fleet from a larger fleet with holes in it.

  ```
  ls .claude/agents/*-e2e-dev.md | wc -l                          # 44
  ls .claude/agents/*-dev.md | grep -cv -- '-e2e-dev\.md$'        # 45
  grep -l '^  - aw-e2e-for' .claude/agents/*-e2e-dev.md | wc -l   # 22 apps
  grep -l '^  - aw-impl-for' .claude/agents/*-dev.md | wc -l      # 22 apps
  ```

- Before dispatching a ladder phase, run
  `git -c core.fsmonitor=false status --short` and confirm no other writer's
  uncommitted work sits in the target write root.

## References

- `CLAUDE.md` sections "Skills", "Work-item lifecycle", "Artifact write order".
- `.claude/rules/operations/persistent-branches.md` for the worktree
  allocation this composes with.
- `.claude/agents/<p>-{e2e-dev,dev}.md` each carry their own ladder
  carve-out (apps) or no-ladder note (libs).
