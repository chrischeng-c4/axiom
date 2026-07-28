"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/surface/specs/score-td-merge-accepts-cb-reviewed.md`.

Migrated by batch `semantic-surface-specs-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:surface-specs/surface-specs-score-td-merge-accepts-cb-reviewed"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/surface/specs/score-td-merge-accepts-cb-reviewed.md"
__legacy_td_digest__ = "sha256:6265301654a167692a7d965e16aeeb4f179e1cd4c1a1cb6538e65c9f63eb0b8e"


def render_markdown() -> Annotated[str, "sha256:6265301654a167692a7d965e16aeeb4f179e1cd4c1a1cb6538e65c9f63eb0b8e"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: score-td-merge-accepts-cb-reviewed\nstatus: superseded\nsuperseded_by: apps/agentic-workflow/tech-design/logic/remove-td-cb-crrr-collapse-to-linear-lifecycle.md\nsuperseded_on: 2026-07-03\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior.\"\n---\n\n# Score TD Merge — Accept cb_reviewed Phase (SUPERSEDED)\n\n> **SUPERSEDED.** This spec described the retired CRRR pre-merge acceptance\n> contract for `aw td code-check`: a guard that accepted `cb_reviewed` (plus\n> `cb_genned`, `cb_filled`, `td_gen_coded`, `td_reviewed`, `td_merged`) before\n> proceeding to a standalone `td_merged` step. That CRRR review/revise loop and\n> the `aw td merge` step it fed have been removed. The lifecycle is now\n> **linear**: `aw wi` -> `aw td create` (td_inited -> td_created) ->\n> `aw td gen` (-> cb_genned) -> `aw td fill` (-> cb_filled) ->\n> `aw td code-check` (-> td_merged, terminal). There is no review/revise\n> ceremony and no `aw td merge` verb.\n>\n> Superseded by\n> `apps/agentic-workflow/tech-design/logic/remove-td-cb-crrr-collapse-to-linear-lifecycle.md`\n> and the authoritative `td_phase` transition table in\n> `apps/agentic-workflow/src/issues/types.rs`\n> (`is_terminal_code_checkable` = `CB_GENNED | LEGACY_TD_GEN_CODED | CB_FILLED`)\n> — not the schema previously defined below, which asserted phase acceptance\n> contradicting that table.\n>\n> Retired phases no longer pass the guard directly; they normalize at read\n> instead: `td_reviewed` -> `td_created`; `cb_reviewed` / `cb_revised` /\n> `cb_arbitrated` -> `cb_filled` (#850, commit 4f07a38a8). The remainder of\n> this document is preserved for historical reference only; do not implement\n> against it.\n"
