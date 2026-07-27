"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/validate/validator/challenge_types.md`.

Migrated by batch `semantic-core-validate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-validate/core-validate-validator-challenge-types"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/validate/validator/challenge_types.md"
__legacy_td_digest__ = "sha256:4605be91fe2b127739b732d82e3afc4f5f3c57d8b16d63a6364bd1e28d14f373"


def render_markdown() -> Annotated[str, "sha256:4605be91fe2b127739b732d82e3afc4f5f3c57d8b16d63a6364bd1e28d14f373"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-validator-challenge-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: existing-project-standardization\n    role: primary\n    gap: managed-and-semantic-production-gates\n    claim: managed-and-semantic-production-gates\n    coverage: full\n    rationale: \"Validation TDs implement managed and semantic production gates for standardization readiness.\"\n---\n\n# ChallengeValidator\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle ChallengeValidator unit struct (placeholder).\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  ChallengeValidator:\n    type: object\n    description: ChallengeValidator placeholder unit struct.\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/validator/challenge.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - ChallengeValidator\n    description: Codegen replaces ChallengeValidator.\n  - path: apps/agentic-workflow/src/validator/challenge.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: Module preamble.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: review lang: markdown -->\n\n**Verdict:** approved\n\n- ok.\n"
