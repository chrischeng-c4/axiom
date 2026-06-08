# Patrol handoff — #2028 (Refactor: agentkit workspace structure)

## Status

- GitHub issue: OPEN, labels show `phase:td_inited` and `project:agentkit`
- Local branch `td-2028` has commits well past td_inited:
  - `01b99ef58` td(2028) — Refactor: agentkit workspace structure (merge-style commit)
  - `7fc6c75f3` td(2028) — code generated
  - `d5595a628` td(2028) — approved (review #1)
  - `4de2fed20` td(2028) — spec authored
- Spec already on disk: `.score/tech_design/projects/agentkit/specs/workspace-slot-layout.md`

## Why patrol is parking this

1. **Out of scope by label.** This patrol cron is bound to `project:mamba`. Issue #2028 is `project:agentkit`. State-A picks up any in-flight branch regardless of label; that's how the patrol ended up here. Treating it as in-scope would mean driving an agentkit refactor from a mamba patrol — wrong owner.
2. **Phase drift.** GitHub label `phase:td_inited` contradicts the commit log on `td-2028` (`code generated` + `approved review #1` both present). `score td validate` would either no-op or fight the drift. Resolving it is operator judgement.

## Operator action required

Pick one:

1. **If the work on td-2028 is correct and ready to land:** FF-merge to `project-agentkit` (the right working branch for project:agentkit work), then `gh issue edit 2028 --add-label phase:td_merged --remove-label phase:td_inited` to align the snapshot.
2. **If it should restart cleanly:** `git branch -D td-2028` and let the agentkit project owner re-run `score td init 2028` from `project-agentkit`.
3. **If #2028 should run under this mamba patrol intentionally:** swap the label to `project:mamba` (only if the change actually belongs to mamba), then patrol will pick it up.

Until one of the above happens, every project:mamba patrol tick will hit state-A on `td-2028` and skip.
