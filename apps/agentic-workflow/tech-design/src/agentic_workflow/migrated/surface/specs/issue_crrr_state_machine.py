"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md`.

Migrated by batch `semantic-surface-specs-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:surface-specs/surface-specs-issue-crrr-state-machine"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/surface/specs/issue-crrr-state-machine.md"
__legacy_td_digest__ = "sha256:f1f2ac4ee86746abfbf3d42f86c0096347c711c403ad0cbd8ab5e24d4b9af838"


def render_markdown() -> Annotated[str, "sha256:f1f2ac4ee86746abfbf3d42f86c0096347c711c403ad0cbd8ab5e24d4b9af838"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: bug-init-change-phase-mapping-conflates-crrr-terminal-spec\nmain_spec_ref: \"apps/agentic-workflow/specs/issue-crrr-state-machine.md\"\nstatus: superseded\nsuperseded_by: apps/agentic-workflow/tech-design/logic/remove-td-cb-crrr-collapse-to-linear-lifecycle.md\nsuperseded_on: 2026-07-03\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Lifecycle TDs support TD/CB artifact authoring, review, revision, merge, or validation behavior.\"\n---\n\n# Bug Init Change Phase Mapping Conflates Crrr Terminal Spec (SUPERSEDED)\n\n> **SUPERSEDED.** This spec documented the WI-level CRRR (Create -> Review ->\n> Revise -> Reset/Arbitrate) state machine, its `Lifecycle-Stage` trailers,\n> and the `aw wi merge` verb that closed a CRRR loop and handed off into the\n> change/TD lifecycle (`parse_phase(\"merged\") -> ChangeInited`). `aw wi merge`\n> has been removed; there is no merge verb on `aw wi`. The former `aw wi\n> review`, `aw wi arbitrate`, and `aw wi draft review` commands are also\n> removed. This file is historical input only; current WI authoring is\n> skeleton -> fill -> validate, with ambiguity routed to HITL.\n>\n> Work-items now feed the tech-design lifecycle directly and linearly:\n> `aw wi` -> `aw td create` (td_inited -> td_created) -> `aw td gen`\n> (-> cb_genned) -> `aw td fill` (-> cb_filled) -> `aw td code-check`\n> (-> td_merged, terminal). There is no review/revise ceremony gating that\n> hop; the gate is EC via `aw td code-check`.\n>\n> Superseded by\n> `apps/agentic-workflow/tech-design/logic/remove-td-cb-crrr-collapse-to-linear-lifecycle.md`\n> and the authoritative `td_phase` transition table in\n> `apps/agentic-workflow/src/issues/types.rs`. The remainder of this\n> document is preserved for historical reference only; do not implement\n> against it.\n"
