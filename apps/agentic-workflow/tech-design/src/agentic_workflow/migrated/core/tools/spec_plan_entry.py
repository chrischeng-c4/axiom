"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/tools/spec_plan_entry.md`.

Migrated by batch `semantic-core-tools-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-spec-plan-entry"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/tools/spec_plan_entry.md"
__legacy_td_digest__ = "sha256:a519098ff4be15e33d250c22bdbf723eb38a375454fbdc2d58c161907c59120b"


def render_markdown() -> Annotated[str, "sha256:a519098ff4be15e33d250c22bdbf723eb38a375454fbdc2d58c161907c59120b"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-tools-spec-plan-entry\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# SpecPlanEntry\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle struct `SpecPlanEntry` in `apps/agentic-workflow/src/tools/spec_plan.rs`.\nCarries one row of the spec-planning table found in issue Reference\nContext bodies — `(spec_id, action, main_spec_ref, source, sections)`.\nPlain data shape with partial-derives `[Debug, Clone, Serialize,\nDeserialize]`.\n\nHand-written outside CODEGEN: module preamble, all `use` statements,\nall free fns / helpers, and the `#[cfg(test)] mod tests` block.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  SpecPlanEntry:\n    type: object\n    required: [spec_id, action, main_spec_ref, sections]\n    description: One row of the spec-planning table in an issue's Reference Context.\n    properties:\n      spec_id:\n        type: string\n        description: \"Spec identifier slug.\"\n      action:\n        type: string\n        description: \"Action keyword (create / update / merge / ...).\"\n      main_spec_ref:\n        type: string\n        description: \"Reference to the main spec this entry plans.\"\n      source:\n        type: string\n        description: \"Optional source artifact (e.g. issue slug).\"\n        x-serde-default: true\n      sections:\n        type: array\n        items: { type: string }\n        description: \"Section names this spec covers.\"\n        x-serde-default: true\n    x-rust-struct:\n      derive: [Debug, Clone, \"serde::Serialize\", \"serde::Deserialize\"]\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/tools/spec_plan.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - SpecPlanEntry\n    description: |\n      Codegen replaces the SpecPlanEntry struct declaration only.\n  - path: apps/agentic-workflow/src/tools/spec_plan.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: |\n      Hand-written outside CODEGEN: module preamble, all `use`\n      statements, all free fns / helpers, and the `#[cfg(test)] mod tests`\n      block.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: review lang: markdown -->\n\n**Verdict:** approved\n\n- [overview] Single-struct scope correctly identified. Hand-written boundary explicit.\n- [schema] Partial-derive list, Vec-in-required convention, Option auto-wrap on `source` all match the source.\n- [changes] Two-entry split clean.\n"
