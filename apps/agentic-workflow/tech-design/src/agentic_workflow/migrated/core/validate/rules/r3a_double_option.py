"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/validate/rules/r3a_double_option.md`.

Migrated by batch `semantic-core-validate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-validate/core-validate-rules-r3a-double-option"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/validate/rules/r3a_double_option.md"
__legacy_td_digest__ = "sha256:a217dabb2b36079a6112d8a27f6d22b1b4e7b276f55409eb3e8fb36d5741fdfb"


def render_markdown() -> Annotated[str, "sha256:a217dabb2b36079a6112d8a27f6d22b1b4e7b276f55409eb3e8fb36d5741fdfb"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-validate-r3a-double-option-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: existing-project-standardization\n    role: primary\n    gap: managed-and-semantic-production-gates\n    claim: managed-and-semantic-production-gates\n    coverage: full\n    rationale: \"Validation TDs implement managed and semantic production gates for standardization readiness.\"\n---\n\n# DoubleOptionRule\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle DoubleOptionRule unit struct in validate/rules/r3a_double_option.rs.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  DoubleOptionRule:\n    type: object\n    description: DoubleOptionRule validation rule (unit struct).\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/validate/rules/r3a_double_option.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - DoubleOptionRule\n    description: Codegen replaces DoubleOptionRule unit struct.\n  - path: apps/agentic-workflow/src/validate/rules/r3a_double_option.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: Module preamble, impl Rule, helpers, tests.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: review lang: markdown -->\n**Verdict:** approved\n\n- ok.\n"
