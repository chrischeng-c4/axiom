"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/validate/rules/r3f_codegen_ready.md`.

Migrated by batch `semantic-core-validate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-validate/core-validate-rules-r3f-codegen-ready"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/validate/rules/r3f_codegen_ready.md"
__legacy_td_digest__ = "sha256:34519b7172ce4aeb34d7098c2d12d4cae68a191b088d8a0aaae041161b3b0d8b"


def render_markdown() -> Annotated[str, "sha256:34519b7172ce4aeb34d7098c2d12d4cae68a191b088d8a0aaae041161b3b0d8b"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-validate-r3f-codegen-ready-types\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: existing-project-standardization\n    role: primary\n    gap: cb-and-cold-verification-gates\n    claim: cb-and-cold-verification-gates\n    coverage: full\n    rationale: \"Codegen/audit validation TDs implement CB and cold verification gates for production readiness.\"\n---\n\n# CodegenReadyRule\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSingle CodegenReadyRule unit struct in validate/rules/r3f_codegen_ready.rs.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  CodegenReadyRule:\n    type: object\n    description: CodegenReadyRule validation rule (unit struct).\n    properties: {}\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/validate/rules/r3f_codegen_ready.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - CodegenReadyRule\n    description: Codegen replaces CodegenReadyRule unit struct.\n  - path: apps/agentic-workflow/src/validate/rules/r3f_codegen_ready.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: Module preamble, impl Rule, helpers, tests.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: review lang: markdown -->\n**Verdict:** approved\n\n- ok.\n"
