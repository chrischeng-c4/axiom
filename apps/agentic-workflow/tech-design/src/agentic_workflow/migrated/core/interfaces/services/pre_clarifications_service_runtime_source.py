"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/services/pre_clarifications_service_runtime_source.md`.

Migrated by batch `projection-core-interfaces-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-services-pre-clarifications-service-runtime-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/services/pre_clarifications_service_runtime_source.md"
__legacy_projection_digest__ = "sha256:d63215774d035cdc4bc205c40e9abfa167d381150dd1ea549e0a8a875fe2256d"


def render_markdown() -> Annotated[str, "sha256:d63215774d035cdc4bc205c40e9abfa167d381150dd1ea549e0a8a875fe2256d"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-services-pre-clarifications-service-runtime-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps.\"\n---\n\n# Pre Clarifications Service Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/services/pre_clarifications_service.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `AppendClarificationsInput` | apps/agentic-workflow/src/services/pre_clarifications_service.rs | struct | pub | 20 |  |\n| `CreateClarificationsInput` | apps/agentic-workflow/src/services/pre_clarifications_service.rs | struct | pub | 32 |  |\n| `QuestionAnswer` | apps/agentic-workflow/src/services/pre_clarifications_service.rs | struct | pub | 42 |  |\n| `append_clarifications` | apps/agentic-workflow/src/services/pre_clarifications_service.rs | function | pub | 120 | append_clarifications(     input: AppendClarificationsInput,     project_root: &Path, ) -> Result<String> |\n| `create_clarifications` | apps/agentic-workflow/src/services/pre_clarifications_service.rs | function | pub | 58 | create_clarifications(     input: CreateClarificationsInput,     project_root: &Path, ) -> Result<String> |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap pre-clarifications-service-runtime -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/services/pre_clarifications_service.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:pre-clarifications-service-runtime>\"\n    description: \"Source template owns pre-clarifications runtime behavior and tests.\"\n```\n"
