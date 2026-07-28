"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/generators/sequence_plus_gen_preamble.md`.

Migrated by batch `projection-core-generate-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-sequence-plus-gen-preamble"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/sequence_plus_gen_preamble.md"
__legacy_projection_digest__ = "sha256:b4a9202aa68c3db61ccb2836f13b4753fdb826866a477db6e05869c0ebb81678"


def render_markdown() -> Annotated[str, "sha256:b4a9202aa68c3db61ccb2836f13b4753fdb826866a477db6e05869c0ebb81678"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-generators-sequence-plus-gen-preamble\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# SequencePlusGenerator Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/generators/sequence_plus_gen.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `SequencePlusGenerator` | apps/agentic-workflow/src/generate/generators/sequence_plus_gen.rs | struct | pub | 30 |  |\n| `new` | apps/agentic-workflow/src/generate/generators/sequence_plus_gen.rs | function | pub | 38 | new() -> Self |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap generate-generators-sequence-plus-gen-preamble -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/generators/sequence_plus_gen.rs -->\n```rust\n\n//! Sequence+ code generator\n//!\n//! Generates Python async call chain from a [`SequenceDef`]\n//! (interaction/sequence section type):\n//!\n//! | Output file                    | Description                                  |\n//! |--------------------------------|----------------------------------------------|\n//! | `{sequence_id}_handlers.py`    | Async handler functions with `@sdd:implement` markers |\n//!\n//! The generator implements [`SpecIRGenerator`] and only accepts\n//! [`SpecIR::SequencePlus`] variants.\n\nuse super::common::{\n    GeneratedFile, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy, SpecIRGenerator,\n};\nuse crate::generate::diagrams::{ArrowType, MessageDef, SequenceDef};\nuse crate::generate::engine::TemplateEngine;\nuse crate::generate::spec_ir::SpecIR;\n\n// ---------------------------------------------------------------------------\n// SequencePlusGenerator\n// ---------------------------------------------------------------------------\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/sequence_plus_gen.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:generate-generators-sequence-plus-gen-preamble>\"\n    description: \"Source template owns module docs, imports, and the generator section header.\"\n```\n"
