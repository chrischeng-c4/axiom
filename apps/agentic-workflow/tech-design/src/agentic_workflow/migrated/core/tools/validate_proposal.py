"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/tools/validate_proposal.md`.

Migrated by batch `semantic-core-tools-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-tools/core-tools-validate-proposal"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/tools/validate_proposal.md"
__legacy_td_digest__ = "sha256:59c88b92091b0fd32e02a8298501609acd64054d386c6a289198344ce5381b42"


def render_markdown() -> Annotated[str, "sha256:59c88b92091b0fd32e02a8298501609acd64054d386c6a289198344ce5381b42"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: sdd-tools-validate-proposal\nfill_sections: [overview, schema, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands.\"\n---\n\n# ValidationSummary Type\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nValidation result summary in `apps/agentic-workflow/src/tools/validate_proposal.rs`.\nOne public shape:\n\n- `ValidationSummary` — `high_count: usize`, `medium_count: usize`,\n  `low_count: usize`, `errors: Vec<String>`,\n  `validation_errors: Vec<ValidationError>`, `stale_files: Vec<String>`.\n  No derives.\n\nThe public struct is schema generated. Source fragments in\n`tools/validate_proposal/` own summary behavior, error accumulation, and the\nmain validation command flow.\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ndefinitions:\n  ValidationSummary:\n    type: object\n    required: [high_count, medium_count, low_count, errors, validation_errors, stale_files]\n    description: Validation result summary.\n    properties:\n      high_count:\n        type: integer\n        x-rust-type: \"usize\"\n        description: \"Number of high-severity errors.\"\n      medium_count:\n        type: integer\n        x-rust-type: \"usize\"\n        description: \"Number of medium-severity errors.\"\n      low_count:\n        type: integer\n        x-rust-type: \"usize\"\n        description: \"Number of low-severity errors.\"\n      errors:\n        type: array\n        items: { type: string }\n        x-rust-type: \"Vec<String>\"\n        description: \"Plain-text error messages.\"\n      validation_errors:\n        type: array\n        items: { type: object }\n        x-rust-type: \"Vec<ValidationError>\"\n        description: \"Structured validation errors.\"\n      stale_files:\n        type: array\n        items: { type: string }\n        x-rust-type: \"Vec<String>\"\n        description: \"Files with stale content.\"\n    x-rust-struct:\n      derive: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/tools/validate_proposal.rs\n    action: modify\n    section: schema\n    impl_mode: codegen\n    replaces:\n      - ValidationSummary\n    description: |\n      Codegen replaces the public struct only.\n```\n\n# Reviews\n\n## Review 1\n<!-- type: review lang: markdown -->\n**Verdict:** approved\n\n- [overview] Single public struct; private accumulator preserved.\n- [schema] All in `required:`; usize + Vec<T> via x-rust-type.\n- [changes] Standard split.\n"
