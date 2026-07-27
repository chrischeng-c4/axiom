"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/generators/react_preamble.md`.

Migrated by batch `projection-core-generate-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-react-preamble"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/react_preamble.md"
__legacy_projection_digest__ = "sha256:b1db2f1e003e58a2d4ee016fc1cb95e26f5178e2062e75afb290d5ad7db58d02"


def render_markdown() -> Annotated[str, "sha256:b1db2f1e003e58a2d4ee016fc1cb95e26f5178e2062e75afb290d5ad7db58d02"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-generators-react-preamble\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# ReactGenerator Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/generators/react.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `ReactGenerator` | apps/agentic-workflow/src/generate/generators/react.rs | struct | pub | 32 |  |\n| `new` | apps/agentic-workflow/src/generate/generators/react.rs | function | pub | 40 | new() -> Self |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap generate-generators-react-preamble -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/generators/react.rs -->\n```rust\n\n//! React component scaffold generator\n//!\n//! Generates a React functional component scaffold from a [`WireframeSpec`]\n//! (wireframe section type):\n//!\n//! | Output file                   | Description                                 |\n//! |-------------------------------|---------------------------------------------|\n//! | `{ComponentName}.tsx`         | React functional component (TypeScript)     |\n//! | `{ComponentName}.types.ts`    | TypeScript props interface                  |\n//! | `index.ts`                    | Barrel re-export                            |\n//!\n//! The generator implements [`SpecIRGenerator`] and only accepts\n//! [`SpecIR::Wireframe`] variants.\n\nuse super::common::{\n    GeneratedFile, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy, SpecIRGenerator,\n};\nuse crate::generate::engine::TemplateEngine;\nuse crate::generate::spec_ir::{PropDef, SpecIR, WireframeNode, WireframeSpec};\nuse serde::Serialize;\n\n// ---------------------------------------------------------------------------\n// ReactGenerator\n// ---------------------------------------------------------------------------\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/react.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:generate-generators-react-preamble>\"\n    description: \"Source template owns module docs, imports, and the generator section header.\"\n```\n"
