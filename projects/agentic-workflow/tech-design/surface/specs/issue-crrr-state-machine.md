---
id: bug-init-change-phase-mapping-conflates-crrr-terminal-spec
main_spec_ref: "projects/agentic-workflow/specs/issue-crrr-state-machine.md"
status: superseded
superseded_by: projects/agentic-workflow/tech-design/logic/remove-td-cb-crrr-collapse-to-linear-lifecycle.md
superseded_on: 2026-07-03
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior."
---

# Bug Init Change Phase Mapping Conflates Crrr Terminal Spec (SUPERSEDED)

> **SUPERSEDED.** This spec documented the WI-level CRRR (Create -> Review ->
> Revise -> Reset/Arbitrate) state machine, its `Lifecycle-Stage` trailers,
> and the `aw wi merge` verb that closed a CRRR loop and handed off into the
> change/TD lifecycle (`parse_phase("merged") -> ChangeInited`). `aw wi merge`
> has been removed; there is no merge verb on `aw wi`. `aw wi review` and
> `aw wi arbitrate` remain as standalone, manual-only escalation verbs an
> operator can invoke, but nothing auto-dispatches a merge or hands a CRRR
> terminal state off to a change lifecycle anymore.
>
> Work-items now feed the tech-design lifecycle directly and linearly:
> `aw wi` -> `aw td create` (td_inited -> td_created) -> `aw td gen`
> (-> cb_genned) -> `aw td fill` (-> cb_filled) -> `aw td code-check`
> (-> td_merged, terminal). There is no review/revise ceremony gating that
> hop; the gate is EC via `aw td code-check`.
>
> Superseded by
> `projects/agentic-workflow/tech-design/logic/remove-td-cb-crrr-collapse-to-linear-lifecycle.md`
> and the authoritative `td_phase` transition table in
> `projects/agentic-workflow/src/issues/types.rs`. The remainder of this
> document is preserved for historical reference only; do not implement
> against it.
