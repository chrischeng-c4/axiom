"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/services/spec_service_runtime_source.md`.

Migrated by batch `projection-core-interfaces-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-services-spec-service-runtime-source"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/services/spec_service_runtime_source.md"
__legacy_projection_digest__ = "sha256:92cc414b89ab037968b33f832fffe9e587c208602e7077f8fc6f97f0e3ac850a"


def render_markdown() -> Annotated[str, "sha256:92cc414b89ab037968b33f832fffe9e587c208602e7077f8fc6f97f0e3ac850a"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-interfaces-services-spec-service-runtime-source\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: td-lifecycle-dispatch\n    claim: td-lifecycle-dispatch\n    coverage: full\n    rationale: \"Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps.\"\n---\n\n# Spec Service Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/services/spec_service.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `ApiSpecData` | apps/agentic-workflow/src/services/spec_service.rs | struct | pub | 38 |  |\n| `CreateSpecInput` | apps/agentic-workflow/src/services/spec_service.rs | struct | pub | 48 |  |\n| `DiagramData` | apps/agentic-workflow/src/services/spec_service.rs | struct | pub | 94 |  |\n| `RequirementData` | apps/agentic-workflow/src/services/spec_service.rs | struct | pub | 110 |  |\n| `ScenarioData` | apps/agentic-workflow/src/services/spec_service.rs | struct | pub | 124 |  |\n| `SpecChangeData` | apps/agentic-workflow/src/services/spec_service.rs | struct | pub | 138 |  |\n| `create_spec` | apps/agentic-workflow/src/services/spec_service.rs | function | pub | 435 | create_spec(input: CreateSpecInput, project_root: &Path) -> Result<String> |\n| `resolve_section_rules` | apps/agentic-workflow/src/services/spec_service.rs | function | pub | 921 | resolve_section_rules(     requirements_text: &str,     design_system: Option<&DesignSystem>, ) -> Vec<SectionEntry> |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap spec-service-runtime -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/services/spec_service.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:spec-service-runtime>\"\n    description: \"Source template owns spec service runtime behavior and tests.\"\n```\n"
