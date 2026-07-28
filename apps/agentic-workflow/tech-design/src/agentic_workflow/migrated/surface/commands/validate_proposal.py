"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/surface/commands/validate_proposal.md`.

Migrated by batch `semantic-surface-commands-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:surface-commands/surface-commands-validate-proposal"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/surface/commands/validate_proposal.md"
__legacy_td_digest__ = "sha256:9a19258581359bf38d32672cc9a07e8b5a1c2d603f5f88cf0a44359326cf90b6"


def render_markdown() -> Annotated[str, "sha256:9a19258581359bf38d32672cc9a07e8b5a1c2d603f5f88cf0a44359326cf90b6"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: score-validate-proposal\nfill_sections: [schema, changes]\ncapability_refs:\n  - id: workflow-root-runner\n    role: primary\n    gap: cli-workflow-chain\n    claim: cli-workflow-chain\n    coverage: full\n    rationale: \"Command/root TDs support CLI workflow chain routing and root-runner dispatch.\"\n---\n\n# ValidationSummary Type\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  ValidationSummary:\n    type: object\n    required: [high_count, medium_count, low_count, errors, validation_errors, stale_files]\n    description: Validation result summary.\n    properties:\n      high_count:\n        type: integer\n        x-rust-type: \"usize\"\n        description: \"Number of high-severity errors.\"\n      medium_count:\n        type: integer\n        x-rust-type: \"usize\"\n        description: \"Number of medium-severity errors.\"\n      low_count:\n        type: integer\n        x-rust-type: \"usize\"\n        description: \"Number of low-severity errors.\"\n      errors:\n        type: array\n        items: { type: string }\n        x-rust-type: \"Vec<String>\"\n        description: \"Plain-text error messages.\"\n      validation_errors:\n        type: array\n        items: { type: object }\n        x-rust-type: \"Vec<ValidationError>\"\n        description: \"Structured validation errors.\"\n      stale_files:\n        type: array\n        items: { type: string }\n        x-rust-type: \"Vec<String>\"\n        description: \"Files with stale content.\"\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/cli/validate_proposal.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - ValidationSummary\n    description: |\n      Codegen replaces the struct declaration only.\n  - path: apps/agentic-workflow/src/cli/validate_proposal.rs\n    action: modify\n    section: schema\n    impl_mode: hand-written\n    description: |\n      Hand-written outside CODEGEN: module imports, the\n      `impl ValidationSummary { is_valid, is_valid_strict, has_warnings,\n      to_json_output, ... }` block, the `validate_proposal` entry point,\n      and all helpers.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: review lang: markdown -->\n\n**Verdict:** approved\n\n- [overview] Single struct, no derives, with multiple impl methods.\n- [schema] All in `required:`; usize + Vec<T> via x-rust-type. Same shape as the sdd-side validate_proposal but in the score.\n- [changes] Standard split.\n"
