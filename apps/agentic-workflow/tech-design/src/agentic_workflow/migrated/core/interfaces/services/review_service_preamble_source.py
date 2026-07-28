"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/services/review_service_preamble_source.md`.

Migrated by batch `projection-core-interfaces-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-services-review-service-preamble-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/services/review_service_preamble_source.md"
__legacy_projection_digest__ = "sha256:0595f71ff26891e46df3bf8a93f118e8fac6f6283ac693e8c6090eecb6868229"


def render_markdown() -> Annotated[str, "sha256:0595f71ff26891e46df3bf8a93f118e8fac6f6283ac693e8c6090eecb6868229"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-services-review-service-preamble-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps.\"\n---\n\n# Review Service Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/services/review_service.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `ReviewInput` | apps/agentic-workflow/src/services/review_service.rs | struct | pub | 99 |  |\n| `VALID_FILES` | apps/agentic-workflow/src/services/review_service.rs | constant | pub | 20 |  |\n| `review_phase_transition` | apps/agentic-workflow/src/services/review_service.rs | function | pub | 62 | review_phase_transition(artifact: &str, verdict: &str) -> Option<StatePhase> |\n| `write_review` | apps/agentic-workflow/src/services/review_service.rs | function | pub | 131 | write_review(input: ReviewInput, project_root: &Path) -> Result<String> |\n## Source\n<!-- type: source lang: rust -->\n\n```rust\n//! Review service — business logic for writing inline reviews.\n//!\n//! Extracted from `mcp/tools/review.rs`. Handles all artifact types with\n//! full spec-aligned phase transition matrix.\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/services/review_service.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<module-preamble>\"\n    description: \"Source template owns the review service module documentation.\"\n```\n"
