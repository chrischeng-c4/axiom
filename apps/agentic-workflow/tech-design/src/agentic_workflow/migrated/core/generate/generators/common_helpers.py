"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/generators/common_helpers.md`.

Migrated by batch `projection-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-common-helpers"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/common_helpers.md"
__legacy_projection_digest__ = "sha256:3261aad70fa7b02496749d71cb1f91dd01f2dac81d1eb6a525297bdd151105c8"


def render_markdown() -> Annotated[str, "sha256:3261aad70fa7b02496749d71cb1f91dd01f2dac81d1eb6a525297bdd151105c8"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-generators-common-helpers\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Generator Common Helper Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/generators/common.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `FileStatus` | apps/agentic-workflow/src/generate/generators/common.rs | enum | pub | 16 |  |\n| `GeneratedFile` | apps/agentic-workflow/src/generate/generators/common.rs | struct | pub | 25 |  |\n| `GeneratorError` | apps/agentic-workflow/src/generate/generators/common.rs | enum | pub | 37 |  |\n| `GeneratorSettings` | apps/agentic-workflow/src/generate/generators/common.rs | struct | pub | 53 |  |\n| `Manifest` | apps/agentic-workflow/src/generate/generators/common.rs | struct | pub | 80 |  |\n| `OverwritePolicy` | apps/agentic-workflow/src/generate/generators/common.rs | enum | pub | 88 |  |\n| `add` | apps/agentic-workflow/src/generate/generators/common.rs | function | pub | 144 | add(&mut self, file: GeneratedFile) |\n| `error` | apps/agentic-workflow/src/generate/generators/common.rs | function | pub | 128 | error(path: PathBuf, error: impl Into<String>) -> Self |\n| `error_count` | apps/agentic-workflow/src/generate/generators/common.rs | function | pub | 162 | error_count(&self) -> usize |\n| `new` | apps/agentic-workflow/src/generate/generators/common.rs | function | pub | 140 | new() -> Self |\n| `skipped` | apps/agentic-workflow/src/generate/generators/common.rs | function | pub | 119 | skipped(path: PathBuf) -> Self |\n| `skipped_count` | apps/agentic-workflow/src/generate/generators/common.rs | function | pub | 155 | skipped_count(&self) -> usize |\n| `written` | apps/agentic-workflow/src/generate/generators/common.rs | function | pub | 103 | written(path: PathBuf, content: &str) -> Self |\n| `written_count` | apps/agentic-workflow/src/generate/generators/common.rs | function | pub | 148 | written_count(&self) -> usize |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap standardize:fold-shadow -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/common.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:standardize:fold-shadow>\"\n    description: \"Source template owns common generator helper impls and traits.\"\n```\n"
