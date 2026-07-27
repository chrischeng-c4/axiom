"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/services/review_service_runtime_source.md`.

Migrated by batch `projection-core-interfaces-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-services-review-service-runtime-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/services/review_service_runtime_source.md"
__legacy_projection_digest__ = "sha256:f29e1d7428022d60f82386f4d0fa117f8ebf30f8df141c7c0dee8e0a3b9dfda9"


def render_markdown() -> Annotated[str, "sha256:f29e1d7428022d60f82386f4d0fa117f8ebf30f8df141c7c0dee8e0a3b9dfda9"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-services-review-service-runtime-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps.\"\n---\n\n# Review Service Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/services/review_service.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `ReviewInput` | apps/agentic-workflow/src/services/review_service.rs | struct | pub | 99 |  |\n| `VALID_FILES` | apps/agentic-workflow/src/services/review_service.rs | constant | pub | 20 |  |\n| `review_phase_transition` | apps/agentic-workflow/src/services/review_service.rs | function | pub | 62 | review_phase_transition(artifact: &str, verdict: &str) -> Option<StatePhase> |\n| `write_review` | apps/agentic-workflow/src/services/review_service.rs | function | pub | 131 | write_review(input: ReviewInput, project_root: &Path) -> Result<String> |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap review-service-runtime -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/services/review_service.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:review-service-runtime>\"\n    description: \"Source template owns review service runtime behavior and tests.\"\n```\n"
