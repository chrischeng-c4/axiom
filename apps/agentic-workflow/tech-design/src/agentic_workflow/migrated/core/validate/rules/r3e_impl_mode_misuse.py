"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/validate/rules/r3e_impl_mode_misuse.md`.

Migrated by batch `semantic-core-validate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-validate/core-validate-rules-r3e-impl-mode-misuse"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/validate/rules/r3e_impl_mode_misuse.md"
__legacy_td_digest__ = "sha256:3d3d257ea43c65f74e63550d62467d4dc199eac3ab0abb50b6bbff5d9364de3d"


def render_markdown() -> Annotated[str, "sha256:3d3d257ea43c65f74e63550d62467d4dc199eac3ab0abb50b6bbff5d9364de3d"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-validate-r3e-impl-mode-misuse-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: existing-project-standardization\n    role: primary\n    gap: managed-and-semantic-production-gates\n    claim: managed-and-semantic-production-gates\n    coverage: full\n    rationale: \"Validation TDs implement managed and semantic production gates for standardization readiness.\"\n---\n\n# ImplModeMisuseRule\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle ImplModeMisuseRule unit struct in validate/rules/r3e_impl_mode_misuse.rs.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  ImplModeMisuseRule:\n    type: object\n    description: ImplModeMisuseRule validation rule (unit struct).\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/validate/rules/r3e_impl_mode_misuse.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - ImplModeMisuseRule\n    description: Codegen replaces ImplModeMisuseRule unit struct.\n  - path: apps/agentic-workflow/src/validate/rules/r3e_impl_mode_misuse.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: Module preamble, impl Rule, helpers, tests.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: review lang: markdown -->\n**Verdict:** approved\n\n- ok.\n"
