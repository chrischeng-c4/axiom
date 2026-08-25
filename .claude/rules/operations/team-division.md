# Team division: main session × subagents × aw:* skills

## Intent

Fix which actor runs each `aw:*` skill and each `wi → e2e → unit → logic`
phase, so contract authoring stays with a strong model, implementation stays
cheap, and every step that needs a human runs where a human can answer.

## Rules

- The main session (controller) exclusively runs the grill series —
  `/aw:wi-change-grill`, `/aw:wi-epic-grill`, `/aw:wi-epic-reconcile` — plus
  `/aw:prepare-goal` Route B. All four interview through AskUserQuestion;
  subagents do not have that tool, and a subagent running them anyway is
  answering for the human.
- The controller also keeps dispatch scheduling, final acceptance, git land,
  tracker semantic decisions, and AGY payload authorization. Never delegate
  these.
- `<app>-planner` (sonnet/xhigh) owns the **e2e** phase of `/aw:wi-tdd`: its
  four commands run by the planner itself, plus `/aw:codex-e2e-review` as a
  verbatim pipe. The black-box contract is the strong model's work, and it
  exists before the dev starts.
- `<app>-dev` (haiku/medium) owns the **unit** and **logic** phases of
  `/aw:wi-tdd` — in Rust, colocated unit tests are part of the source, so the
  whole `src/` write root is the dev's. The phase order still holds: unit
  tests land red in their own commit first, and `C0`'s filename gate
  (`leg.LEG_TEST_FILES`) refuses the logic phase touching a test file. Weak
  self-serving unit tests are caught by the planner's independent e2e cases
  and by `/aw:codex-code-review`, which the dev runs as a verbatim pipe.
- `<app>-research` (opus/max, read-only) is optional and rarely dispatched:
  a long read-only investigation the controller does not want in its own
  context. It may also run `/aw:meta-check` and `/aw:prepare-goal` Route A;
  its output returns to the controller, and a goal it drafted is not a goal
  anyone set.
- One writer per worktree at a time. Phase scripts measure named reds against
  HEAD, so two concurrent writers poison each other's baseline. Cross-app
  parallelism means separate persistent-branch worktrees (`app/<name>`,
  `lib/<name>`; see `.claude/rules/operations/persistent-branches.md`) or
  `/dispatch-to-agy`.
- Phase `commit` is run by that phase's runner — the script re-runs every gate
  before writing — and the controller's acceptance reads the commits, not the
  runner's summary. A verdict is always codex's; no agent writes a `VERDICT:`
  line.
- A dev stalled twice on the same task is not re-dispatched; the controller
  takes over directly.

## Verification

- `grep -l 'aw:wi-tdd' .claude/agents/*-planner.md .claude/agents/*-dev.md | wc -l`
  returns 44 (22 planners + 22 devs);
  `grep -l 'aw:meta-check' .claude/agents/*-research.md | wc -l` returns 22.
- Before dispatching a ladder phase, run
  `git -c core.fsmonitor=false status --short` and confirm no other writer's
  uncommitted work sits in the target write root.

## References

- `CLAUDE.md` sections "Skills", "Work-item lifecycle", "Artifact write order".
- `.claude/rules/operations/persistent-branches.md` for the worktree
  allocation this composes with.
- `.claude/agents/<app>-{planner,dev,research}.md` each carry their own ladder
  carve-out text.
