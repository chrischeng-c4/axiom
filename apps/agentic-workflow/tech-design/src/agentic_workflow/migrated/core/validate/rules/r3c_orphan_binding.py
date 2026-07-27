"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/validate/rules/r3c_orphan_binding.md`.

Migrated by batch `semantic-core-validate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-validate/core-validate-rules-r3c-orphan-binding"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/validate/rules/r3c_orphan_binding.md"
__legacy_td_digest__ = "sha256:237fefdde20f2577d5bbdb48d45b241a92402fa6b3c824d24d243ab4f799ea50"


def render_markdown() -> Annotated[str, "sha256:237fefdde20f2577d5bbdb48d45b241a92402fa6b3c824d24d243ab4f799ea50"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-validate-r3c-orphan-binding-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: existing-project-standardization\n    role: primary\n    gap: managed-and-semantic-production-gates\n    claim: managed-and-semantic-production-gates\n    coverage: full\n    rationale: \"Validation TDs implement managed and semantic production gates for standardization readiness.\"\n---\n\n# OrphanBindingRule\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle OrphanBindingRule unit struct in validate/rules/r3c_orphan_binding.rs.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  OrphanBindingRule:\n    type: object\n    description: OrphanBindingRule validation rule (unit struct).\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/validate/rules/r3c_orphan_binding.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - OrphanBindingRule\n    description: Codegen replaces OrphanBindingRule unit struct.\n  - path: apps/agentic-workflow/src/validate/rules/r3c_orphan_binding.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: Module preamble, impl Rule, helpers, tests.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: review lang: markdown -->\n**Verdict:** approved\n\n- ok.\n"
