"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/generators/state_machine_gen_preamble.md`.

Migrated by batch `projection-core-generate-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-state-machine-gen-preamble"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/state_machine_gen_preamble.md"
__legacy_projection_digest__ = "sha256:e78c560ec2f4016f223364f7c26576d364cc672b4c3bc769b36465f3e3342df4"


def render_markdown() -> Annotated[str, "sha256:e78c560ec2f4016f223364f7c26576d364cc672b4c3bc769b36465f3e3342df4"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-generators-state-machine-gen-preamble\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# StateMachineGenerator Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/generators/state_machine_gen.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `StateMachineGenerator` | apps/agentic-workflow/src/generate/generators/state_machine_gen.rs | struct | pub | 29 |  |\n| `new` | apps/agentic-workflow/src/generate/generators/state_machine_gen.rs | function | pub | 37 | new() -> Self |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap generate-generators-state-machine-gen-preamble -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/generators/state_machine_gen.rs -->\n```rust\n\n//! State machine code generator\n//!\n//! Generates Python Enum + transition function from a [`StateMachineDef`]\n//! (state-machine section type):\n//!\n//! | Output file                 | Description                                    |\n//! |-----------------------------|------------------------------------------------|\n//! | `{machine_id}_states.py`    | Python Enum class + `transition()` function    |\n//!\n//! The generator implements [`SpecIRGenerator`] and only accepts\n//! [`SpecIR::StateMachinePlus`] variants.\n\nuse super::common::{\n    GeneratedFile, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy, SpecIRGenerator,\n};\nuse crate::generate::engine::TemplateEngine;\nuse crate::generate::spec_ir::SpecIR;\n\n// ---------------------------------------------------------------------------\n// StateMachineGenerator\n// ---------------------------------------------------------------------------\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/state_machine_gen.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:generate-generators-state-machine-gen-preamble>\"\n    description: \"Source template owns module docs, imports, and the generator section header.\"\n```\n"
