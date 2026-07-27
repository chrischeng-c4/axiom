"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/services/pre_clarifications_service_preamble_source.md`.

Migrated by batch `projection-core-interfaces-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-services-pre-clarifications-service-preamble-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/services/pre_clarifications_service_preamble_source.md"
__legacy_projection_digest__ = "sha256:aea2584f6fce8f004861da730884cfd3471c19a059961f4f96f043e68972eb26"


def render_markdown() -> Annotated[str, "sha256:aea2584f6fce8f004861da730884cfd3471c19a059961f4f96f043e68972eb26"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-services-pre-clarifications-service-preamble-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps.\"\n---\n\n# Pre Clarifications Service Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/services/pre_clarifications_service.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `AppendClarificationsInput` | apps/agentic-workflow/src/services/pre_clarifications_service.rs | struct | pub | 20 |  |\n| `CreateClarificationsInput` | apps/agentic-workflow/src/services/pre_clarifications_service.rs | struct | pub | 32 |  |\n| `QuestionAnswer` | apps/agentic-workflow/src/services/pre_clarifications_service.rs | struct | pub | 42 |  |\n| `append_clarifications` | apps/agentic-workflow/src/services/pre_clarifications_service.rs | function | pub | 120 | append_clarifications(     input: AppendClarificationsInput,     project_root: &Path, ) -> Result<String> |\n| `create_clarifications` | apps/agentic-workflow/src/services/pre_clarifications_service.rs | function | pub | 58 | create_clarifications(     input: CreateClarificationsInput,     project_root: &Path, ) -> Result<String> |\n## Source\n<!-- type: source lang: rust -->\n\n```rust\n//! Clarifications service - Business logic for creating pre_clarifications.md\n//!\n//! Provides structured Q&A capture from user interactions during planning.\n\nuse crate::models::state::StatePhase;\nuse crate::state::StateManager;\nuse crate::Result;\nuse chrono::Local;\nuse std::path::Path;\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/services/pre_clarifications_service.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<module-preamble>\"\n      - \"<handwrite-gap:pre-clarifications-service-preamble>\"\n    description: \"Source template owns pre-clarifications service docs and imports.\"\n```\n"
