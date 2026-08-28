# Team division: main session × subagents × aw-* skills

## Intent

Fix which actor runs each `aw-*` skill and each `wi → e2e → unit → logic`
phase, so contract authoring stays with a strong model, implementation stays
cheap, and every step that needs a human runs where a human can answer.

## Rules

- The main session (controller) exclusively runs the grill series —
  `/aw-grill-me-to-prd`, `/aw-grill-me-to-epic`, `/aw-grill-me-to-change`,
  `/aw-grill-epic-to-changes` and the two `/aw-grill-{change,epic}-to-td` —
  plus `/aw-prepare-goal` Route B and `/aw-ask-user`. All eight interview
  through AskUserQuestion;
  subagents do not have that tool, and a subagent running them anyway is
  answering for the human.
- The controller also keeps dispatch scheduling, final acceptance, git land,
  tracker semantic decisions, and AGY payload authorization. Never delegate
  these.
- `<app>-planner` (sonnet/xhigh) owns the **e2e** phase of
  `/aw-go-tdd-for-change`: its four commands run by the planner itself. The
  black-box contract is the strong model's work, and it exists before the dev
  starts.
- `<app>-dev` (haiku/medium) owns the **unit** and **logic** phases of
  `/aw-go-tdd-for-change` — in Rust, colocated unit tests are part of the
  source, so the whole `src/` write root is the dev's. The phase order still
  holds: unit tests land red in their own commit first, and `C0`'s filename
  gate (`leg.LEG_TEST_FILES`) refuses the logic phase touching a test file.
  Weak self-serving unit tests are caught by the planner's independent e2e
  cases, which `logic` has to turn green while they still refuse HEAD.
- The model tiers above are defaults, not ceilings. For a hard case the
  controller may raise the model at dispatch time via the Agent call's `model`
  override — planner to opus, dev to sonnet — without editing the agent
  definitions. Phase ownership does not move with the model.
- `<app>-research` (opus/max, read-only) is optional and rarely dispatched:
  a long read-only investigation the controller does not want in its own
  context. It may also run `/aw-check-meta` and `/aw-prepare-goal` Route A;
  its output returns to the controller, and a goal it drafted is not a goal
  anyone set.
- One writer per worktree at a time. Phase scripts measure named reds against
  HEAD, so two concurrent writers poison each other's baseline. Cross-app
  parallelism means separate persistent-branch worktrees (`app/<name>`,
  `lib/<name>`; see `.claude/rules/operations/persistent-branches.md`) or
  `/dispatch-to-agy`.
- Phase `commit` is run by that phase's runner — the script re-runs every gate
  before writing — and the controller's acceptance reads the commits, not the
  runner's summary.
- A dev stalled twice on the same task is not re-dispatched; the controller
  takes over directly.

## Verification

- The ladder fleet is the 22 planners and the 22 plain `<app>-dev` agents. The
  `*-dev.md` glob also matches the 88 codex-mirrored `*-sr-dev.md` /
  `*-jr-dev.md` agents added on 2026-08-27, which run no ladder phase, so the
  set has to exclude them by name — `*[!r]-dev.md` does not work, because it
  also drops `courier-dev.md` and `meter-dev.md`, whose own names end in `r`.
  Both numbers below must read 44; comparing the two is the point, because a
  count of matches alone cannot tell a complete fleet from a larger fleet with
  holes in it.

  ```
  set=$(ls .claude/agents/*-planner.md .claude/agents/*-dev.md \
        | grep -vE -- '-(sr|jr)-dev\.md$')
  echo "$set" | wc -l                                        # 44 in the fleet
  echo "$set" | xargs grep -l 'aw-go-tdd-for-change' | wc -l  # 44 carry it
  ```

  `grep -l 'aw-check-meta' .claude/agents/*-research.md | wc -l` returns 22.
- Before dispatching a ladder phase, run
  `git -c core.fsmonitor=false status --short` and confirm no other writer's
  uncommitted work sits in the target write root.

## References

- `CLAUDE.md` sections "Skills", "Work-item lifecycle", "Artifact write order".
- `.claude/rules/operations/persistent-branches.md` for the worktree
  allocation this composes with.
- `.claude/agents/<app>-{planner,dev,research}.md` each carry their own ladder
  carve-out text.
