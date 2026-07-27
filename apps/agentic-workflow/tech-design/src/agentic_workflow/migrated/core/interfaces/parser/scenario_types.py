"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/interfaces/parser/scenario_types.md`.

Migrated by batch `semantic-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-parser-scenario-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/interfaces/parser/scenario_types.md"
__legacy_td_digest__ = "sha256:cd8b436a87bf35c79e981d68a347264310b5ec57264dc5112efbfb87f2df8d01"


def render_markdown() -> Annotated[str, "sha256:cd8b436a87bf35c79e981d68a347264310b5ec57264dc5112efbfb87f2df8d01"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-parser-scenario-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: aw-core-client-model-workitem-first-artifact-lifecycle\n    role: primary\n    gap: core-concept-model-and-invariants\n    claim: core-concept-model-and-invariants\n    coverage: full\n    rationale: \"Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure.\"\n---\n\n# ScenarioParser\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle ScenarioParser unit struct (placeholder).\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  ScenarioParser:\n    type: object\n    required: []\n    description: ScenarioParser placeholder unit struct.\n    properties: {}\n    x-rust-struct:\n      derive: []\n      unit: true\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/parser/scenario.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - ScenarioParser\n    description: Codegen replaces ScenarioParser.\n  - path: apps/agentic-workflow/src/parser/scenario.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: Module preamble.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: doc lang: markdown -->\n**Verdict:** approved\n\n- ok.\n"
