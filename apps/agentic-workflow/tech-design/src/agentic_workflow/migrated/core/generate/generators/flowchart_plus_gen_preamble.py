"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/generators/flowchart_plus_gen_preamble.md`.

Migrated by batch `projection-core-generate-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-flowchart-plus-gen-preamble"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/flowchart_plus_gen_preamble.md"
__legacy_projection_digest__ = "sha256:e3a95aa65bfd3af45ee147e1582f7af76d724cebaf8e14f400f44ee83b348748"


def render_markdown() -> Annotated[str, "sha256:e3a95aa65bfd3af45ee147e1582f7af76d724cebaf8e14f400f44ee83b348748"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-generators-flowchart-plus-gen-preamble\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# FlowchartPlusGenerator Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/generators/flowchart_plus_gen.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `FlowchartPlusGenerator` | apps/agentic-workflow/src/generate/generators/flowchart_plus_gen.rs | struct | pub | 32 |  |\n| `new` | apps/agentic-workflow/src/generate/generators/flowchart_plus_gen.rs | function | pub | 40 | new() -> Self |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap generate-generators-flowchart-plus-gen-preamble -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/generators/flowchart_plus_gen.rs -->\n```rust\n\n//! Flowchart+ code generator\n//!\n//! Generates Python function skeletons from a [`FlowchartDef`]\n//! (flowchart/logic section type) with YAML metadata:\n//!\n//! | Output file                 | Description                                    |\n//! |-----------------------------|------------------------------------------------|\n//! | `{flowchart_id}_flow.py`    | Python function skeletons with `@sdd:implement` markers |\n//!\n//! The generator implements [`SpecIRGenerator`] and only accepts\n//! [`SpecIR::FlowchartPlus`] variants.\n\nuse super::common::{\n    GeneratedFile, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy, SpecIRGenerator,\n};\nuse crate::generate::diagrams::{\n    FlowchartDef, FlowchartEdgeDef as EdgeDef, FlowchartNodeDef as NodeDef, NodeShape, SemanticType,\n};\nuse crate::generate::engine::TemplateEngine;\nuse crate::generate::spec_ir::SpecIR;\n\n// ---------------------------------------------------------------------------\n// FlowchartPlusGenerator\n// ---------------------------------------------------------------------------\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/flowchart_plus_gen.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:generate-generators-flowchart-plus-gen-preamble>\"\n    description: \"Source template owns FlowchartPlus generator module docs and imports.\"\n```\n"
