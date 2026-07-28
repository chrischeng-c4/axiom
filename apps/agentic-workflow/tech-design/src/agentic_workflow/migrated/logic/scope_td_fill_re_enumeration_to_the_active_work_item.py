"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/logic/scope-td-fill-re-enumeration-to-the-active-work-item.md`.

Migrated by batch `semantic-logic-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:logic/logic-scope-td-fill-re-enumeration-to-the-active-work-item"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/logic/scope-td-fill-re-enumeration-to-the-active-work-item.md"
__legacy_td_digest__ = "sha256:393b0877167f6e42bd832e5a8a28a52ce56bc8083f36e09c79dccb6a7bee8a69"


def render_markdown() -> Annotated[str, "sha256:393b0877167f6e42bd832e5a8a28a52ce56bc8083f36e09c79dccb6a7bee8a69"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: '1717'\nsummary: (fill)\nfill_sections: [logic, changes, unit-test]\n---\n\n## Logic\n<!-- type: logic lang: mermaid -->\n\n```mermaid\n---\nid: td-fill-active-scope-contract\nentry: load\nnodes:\n  load: { kind: start, label: Load active issue and TD spec }\n  scope: { kind: process, label: Parse TD Changes into marker scope }\n  apply: { kind: process, label: Replace requested local marker body }\n  remaining: { kind: decision, label: Scoped queue has another marker }\n  next: { kind: terminal, label: Lock and dispatch the next local marker }\n  check: { kind: terminal, label: Mark filled and dispatch code-check }\nedges:\n  - { from: load, to: scope }\n  - { from: scope, to: apply }\n  - { from: apply, to: remaining }\n  - { from: remaining, to: next, label: yes }\n  - { from: remaining, to: check, label: no }\n---\nflowchart TD\n    load([load issue + TD]) --> scope[parse Changes scope]\n    scope --> apply[apply local payload]\n    apply --> remaining{scoped markers remain?}\n    remaining -->|yes| next([dispatch next local marker])\n    remaining -->|no| check([dispatch code-check])\n```\n\n`run_apply` derives the active TD spec exactly as brief mode does, parses its Changes paths, and uses `markers_for_td_changes` both to locate the requested marker and to compute `remaining`. If the local queue is empty after the replacement, it advances to `cb_filled` and dispatches code-check even when unrelated unfilled markers exist elsewhere.\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/cli/cb_fill.rs\n    action: modify\n    section: logic\n    impl_mode: hand-written\n  - path: apps/agentic-workflow/tech-design/surface/interfaces/src/cb_fill.md\n    action: modify\n    section: logic\n    impl_mode: codegen\n  - path: apps/agentic-workflow/CAPABILITIES.md\n    action: modify\n    section: logic\n    impl_mode: hand-written\n```\n## Unit Test\n<!-- type: unit-test lang: mermaid -->\n\n```mermaid\n---\nid: scope-td-fill-re-enumeration-to-the-active-work-item-verification\nrequirements:\n  active_td_marker_scope:\n    id: R1\n    text: \"Applying a marker only queues unresolved HANDWRITE markers whose source paths are declared by the active TD Changes section; a marker in an unrelated app cannot prevent code-check for this work item.\"\n    kind: regression\n    risk: high\n    verify: cb_fill_apply_scopes_remaining_markers_to_active_changes\n---\nflowchart TD\n    r1[R1 active td marker scope] --> cb_fill_apply_scopes_remaining_markers_to_active_changes[cb_fill_apply_scopes_remaining_markers_to_active_changes]\n```\n"
