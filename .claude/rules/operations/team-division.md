# Team division: main session × subagents × aw-* skills

## Intent

Fix which actor runs each `aw-*` skill and each `wi → e2e → impl` phase, so
contract authoring stays with a strong model, implementation stays cheap, and
every step that needs a human runs where a human can answer.

## Rules

- The main session (controller) exclusively runs `/aw-grill-me-to-meta`,
  `/aw-grill-meta-to-wis`, `/aw-prepare-goal` Route B and `/aw-ask-user`. All
  four interview through AskUserQuestion; subagents do not have that tool,
  and a subagent running them anyway is answering for the human.
- The controller also keeps dispatch scheduling, final acceptance, git land,
  tracker semantic decisions, and AGY payload authorization. Never delegate
  these.
- `<app>-sr-dev` (sonnet/high) owns the **e2e** phase: it runs
  `/aw-e2e-for-wi`'s four verbs itself. The black-box contract is the strong
  model's work, and it exists before the dev starts. The 22 `*-planner`
  agents that used to carry this phase were deleted on 2026-08-28; only the
  22 app-level sr-devs carry the ladder carve-out — the 22 lib-level sr-devs
  do not, because `leg.leg_root` resolves under `apps/` only.
- `<app>-dev` (haiku/medium) owns the **impl** phase — in Rust, colocated
  unit tests are part of the source, so the whole `src/` write root is the
  dev's. It runs `/aw-impl-for-wi`'s five verbs: the colocated tests land red
  first, `red` records that attribution evidence to
  `.aw/impl-red/<iid>.json`, and `C2` refuses a later `verify`/`test` whose
  tree has drifted from that record. Weak self-serving unit tests are caught
  by the sr-dev's independent e2e cases, which `impl` has to turn green
  while they still refuse HEAD.
- The model tiers above are defaults, not ceilings. For a hard case the
  controller may raise the model at dispatch time via the Agent call's `model`
  override — sr-dev to opus, dev to sonnet — without editing the agent
  definitions. Phase ownership does not move with the model.
- `<app>-research` (opus/max, read-only) is optional and rarely dispatched:
  a long read-only investigation the controller does not want in its own
  context. It may also run `meta.py check` directly and `/aw-prepare-goal`
  Route A; its output returns to the controller, and a goal it drafted is not
  a goal anyone set.
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

- The ladder fleet is the 22 plain `<app>-dev` agents (impl) plus their 22
  same-app `<app>-sr-dev` twins (e2e). The `*-dev.md` glob also matches the
  jr/sr agents, so the plain set has to exclude them by name —
  `*[!r]-dev.md` does not work, because it also drops `courier-dev.md` and
  `meter-dev.md`, whose own names end in `r`. The sr-dev set is then derived
  from the plain set rather than globbed, because only the 22 app-level
  sr-devs carry the carve-out and a bare `*-sr-dev.md` glob matches all 44.
  All three numbers below must read 22; comparing them is the point, because
  a count of matches alone cannot tell a complete fleet from a larger fleet
  with holes in it.

  ```
  set=$(ls .claude/agents/*-dev.md | grep -vE -- '-(sr|jr)-dev\.md$')
  echo "$set" | wc -l                                          # 22 plain devs
  echo "$set" | xargs grep -l 'aw-impl-for-wi' | wc -l          # 22 carry impl
  echo "$set" | sed 's/-dev\.md$/-sr-dev.md/' \
    | xargs grep -l 'aw-e2e-for-wi' | wc -l                     # 22 carry e2e
  ```

  `grep -l 'meta.py check' .claude/agents/*-research.md | wc -l` returns 22.
- Before dispatching a ladder phase, run
  `git -c core.fsmonitor=false status --short` and confirm no other writer's
  uncommitted work sits in the target write root.

## References

- `CLAUDE.md` sections "Skills", "Work-item lifecycle", "Artifact write order".
- `.claude/rules/operations/persistent-branches.md` for the worktree
  allocation this composes with.
- `.claude/agents/<app>-{sr-dev,dev,research}.md` each carry their own ladder
  carve-out text.
