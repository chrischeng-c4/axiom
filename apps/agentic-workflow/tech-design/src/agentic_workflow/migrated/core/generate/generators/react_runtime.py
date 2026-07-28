"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/generators/react_runtime.md`.

Migrated by batch `projection-core-generate-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-react-runtime"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/react_runtime.md"
__legacy_projection_digest__ = "sha256:46d4462f6a2fe312379a1c5941f9015b7e23ea08c9916c17c65d28bfb1acc051"


def render_markdown() -> Annotated[str, "sha256:46d4462f6a2fe312379a1c5941f9015b7e23ea08c9916c17c65d28bfb1acc051"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-generators-react-runtime\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# ReactGenerator Runtime Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/generators/react.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `ReactGenerator` | apps/agentic-workflow/src/generate/generators/react.rs | struct | pub | 32 |  |\n| `new` | apps/agentic-workflow/src/generate/generators/react.rs | function | pub | 40 | new() -> Self |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap generate-generators-react-runtime -->\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/react.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:generate-generators-react-runtime>\"\n    description: \"Source template owns the React generator runtime and regression tests.\"\n```\n"
