"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/specs/mod.md`.

Migrated by batch `projection-core-generate-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-specs-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/specs/mod.md"
__legacy_projection_digest__ = "sha256:93281da238c50dac11ea6679cbbed1a48f762dd41a3604f1717896429c02d78f"


def render_markdown() -> Annotated[str, "sha256:93281da238c50dac11ea6679cbbed1a48f762dd41a3604f1717896429c02d78f"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-generate-specs-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# Standardized apps/agentic-workflow/src/generate/specs/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/specs/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `asyncapi` | apps/agentic-workflow/src/generate/specs/mod.rs | module | pub | 7 |  |\n| `openapi` | apps/agentic-workflow/src/generate/specs/mod.rs | module | pub | 8 |  |\n| `openrpc` | apps/agentic-workflow/src/generate/specs/mod.rs | module | pub | 9 |  |\n| `serverless` | apps/agentic-workflow/src/generate/specs/mod.rs | module | pub | 10 |  |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-handwrite -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/specs/mod.rs -->\n```rust\n//! API Specification Generation\n//!\n//! Provides functions for generating various API specification formats.\n\npub mod asyncapi;\npub mod openapi;\npub mod openrpc;\npub mod serverless;\n\npub use asyncapi::{generate_asyncapi, AsyncApiInput};\npub use openapi::{generate_openapi, OpenApiInput};\npub use openrpc::{generate_openrpc, OpenRpcInput};\npub use serverless::{generate_serverless_workflow, ServerlessWorkflowInput};\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/specs/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Source template owns the complete API specification generation module aggregator.\n```\n"
