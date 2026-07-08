---
id: score-td-merge-accepts-cb-reviewed
status: superseded
superseded_by: apps/agentic-workflow/tech-design/logic/remove-td-cb-crrr-collapse-to-linear-lifecycle.md
superseded_on: 2026-07-03
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior."
---

# Score TD Merge — Accept cb_reviewed Phase (SUPERSEDED)

> **SUPERSEDED.** This spec described the retired CRRR pre-merge acceptance
> contract for `aw td code-check`: a guard that accepted `cb_reviewed` (plus
> `cb_genned`, `cb_filled`, `td_gen_coded`, `td_reviewed`, `td_merged`) before
> proceeding to a standalone `td_merged` step. That CRRR review/revise loop and
> the `aw td merge` step it fed have been removed. The lifecycle is now
> **linear**: `aw wi` -> `aw td create` (td_inited -> td_created) ->
> `aw td gen` (-> cb_genned) -> `aw td fill` (-> cb_filled) ->
> `aw td code-check` (-> td_merged, terminal). There is no review/revise
> ceremony and no `aw td merge` verb.
>
> Superseded by
> `apps/agentic-workflow/tech-design/logic/remove-td-cb-crrr-collapse-to-linear-lifecycle.md`
> and the authoritative `td_phase` transition table in
> `apps/agentic-workflow/src/issues/types.rs`
> (`is_terminal_code_checkable` = `CB_GENNED | LEGACY_TD_GEN_CODED | CB_FILLED`)
> — not the schema previously defined below, which asserted phase acceptance
> contradicting that table.
>
> Retired phases no longer pass the guard directly; they normalize at read
> instead: `td_reviewed` -> `td_created`; `cb_reviewed` / `cb_revised` /
> `cb_arbitrated` -> `cb_filled` (#850, commit 4f07a38a8). The remainder of
> this document is preserved for historical reference only; do not implement
> against it.
