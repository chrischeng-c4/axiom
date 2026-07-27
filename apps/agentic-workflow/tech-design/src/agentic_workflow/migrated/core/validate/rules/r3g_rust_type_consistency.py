"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/validate/rules/r3g_rust_type_consistency.md`.

Migrated by batch `semantic-core-validate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-validate/core-validate-rules-r3g-rust-type-consistency"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/validate/rules/r3g_rust_type_consistency.md"
__legacy_td_digest__ = "sha256:781978fb302baa8faeca529a8ebbf1f9e49714a29dc8e2dce40c3cd508095491"


def render_markdown() -> Annotated[str, "sha256:781978fb302baa8faeca529a8ebbf1f9e49714a29dc8e2dce40c3cd508095491"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-validate-r3g-rust-type-consistency-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: existing-project-standardization\n    role: primary\n    gap: managed-and-semantic-production-gates\n    claim: managed-and-semantic-production-gates\n    coverage: full\n    rationale: \"Validation TDs implement managed and semantic production gates for standardization readiness.\"\n---\n\n# RustTypeConsistencyRule\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle RustTypeConsistencyRule unit struct in validate/rules/r3g_rust_type_consistency.rs.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  RustTypeConsistencyRule:\n    type: object\n    description: RustTypeConsistencyRule validation rule (unit struct).\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/validate/rules/r3g_rust_type_consistency.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - RustTypeConsistencyRule\n    description: Codegen replaces RustTypeConsistencyRule unit struct.\n  - path: apps/agentic-workflow/src/validate/rules/r3g_rust_type_consistency.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: Module preamble, impl Rule, helpers, tests.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: review lang: markdown -->\n**Verdict:** approved\n\n- ok.\n"
