"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/tools/common_change_spec.md`.

Migrated by batch `semantic-core-tools-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-common-change-spec"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/tools/common_change_spec.md"
__legacy_td_digest__ = "sha256:d5e7e9650e939e95a97024b22e8b685afc1a84ab918e974359d4684f411ded7b"


def render_markdown() -> Annotated[str, "sha256:d5e7e9650e939e95a97024b22e8b685afc1a84ab918e974359d4684f411ded7b"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-tools-common-change-spec\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# SpecSubState Type\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPer-spec sub-state enum in\n`apps/agentic-workflow/src/tools/common_change_spec.rs`. One shape:\n\n- `SpecSubState` — 5-variant enum:\n  - `Create { spec_id, depends }` (struct variant)\n  - `Review { spec_id }` (struct variant)\n  - `Revise { spec_id }` (struct variant)\n  - `MainthreadMustFix { spec_id }` (struct variant)\n  - `AdvanceToImplementation` (unit)\n  Derives `[Debug]`.\n\nCodegen replaces the enum declaration. Source fragments in\n`tools/common_change_spec/` own the universal skeleton, helper runtime, and\nregression tests.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  SpecSubState:\n    type: string\n    enum: [Create, Review, Revise, MainthreadMustFix, AdvanceToImplementation]\n    description: Per-spec sub-state within the change-spec lifecycle.\n    x-rust-enum:\n      derive: [Debug]\n      variants:\n        - name: Create\n          kind: struct\n          doc: \"No spec file — needs skeleton + create loop.\"\n          fields:\n            - { name: spec_id, rust_type: String }\n            - { name: depends, rust_type: \"Vec<String>\" }\n        - name: Review\n          kind: struct\n          doc: \"Spec exists with create_complete, no review — needs review.\"\n          fields:\n            - { name: spec_id, rust_type: String }\n        - name: Revise\n          kind: struct\n          doc: \"Reviewed with issues — re-fill flagged sections.\"\n          fields:\n            - { name: spec_id, rust_type: String }\n        - name: MainthreadMustFix\n          kind: struct\n          doc: \"REJECTED after revision limit — mainthread must intervene.\"\n          fields:\n            - { name: spec_id, rust_type: String }\n        - { name: AdvanceToImplementation, doc: \"All specs created + approved.\" }\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/tools/common_change_spec.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - SpecSubState\n    description: |\n      Codegen replaces the enum declaration only.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: review lang: markdown -->\n**Verdict:** approved\n\n- [overview] Single enum, mixed unit + struct variants.\n- [schema] All variants well-formed.\n- [changes] Standard split.\n"
