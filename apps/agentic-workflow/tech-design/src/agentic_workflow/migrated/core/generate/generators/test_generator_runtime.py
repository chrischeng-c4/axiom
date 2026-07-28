"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/generators/test_generator_runtime.md`.

Migrated by batch `projection-core-generate-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-test-generator-runtime"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/test_generator_runtime.md"
__legacy_projection_digest__ = "sha256:56794ad234ca9b23cdbb80fcb9e67cf29388dd602c2538d0d901a8d445047e23"


def render_markdown() -> Annotated[str, "sha256:56794ad234ca9b23cdbb80fcb9e67cf29388dd602c2538d0d901a8d445047e23"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-generators-test-generator-runtime\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# TestGenerator Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/generators/test_generator.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `CoverageIssue` | apps/agentic-workflow/src/generate/generators/test_generator.rs | struct | pub | 41 |  |\n| `TestGenError` | apps/agentic-workflow/src/generate/generators/test_generator.rs | enum | pub | 53 |  |\n| `TestGenResult` | apps/agentic-workflow/src/generate/generators/test_generator.rs | struct | pub | 65 |  |\n| `TestGenerator` | apps/agentic-workflow/src/generate/generators/test_generator.rs | struct | pub | 76 |  |\n| `generate` | apps/agentic-workflow/src/generate/generators/test_generator.rs | function | pub | 96 | generate(&self, def: &RequirementDiagramDef) -> Result<TestGenResult, TestGenError> |\n| `new` | apps/agentic-workflow/src/generate/generators/test_generator.rs | function | pub | 91 | new(strict: bool) -> Self |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap generate-generators-test-generator-runtime -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/test_generator.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:generate-generators-test-generator-runtime>\"\n    description: \"Source template owns the test generator runtime and regression tests.\"\n```\n"
