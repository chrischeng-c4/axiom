"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/generate/README.md`.

Migrated by batch `semantic-core-generate-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-readme"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/generate/README.md"
__legacy_td_digest__ = "sha256:e6828b21aacd9c0d9ce53fcaccd9b6ede997c8e633b9662417d8b6fb29b9df68"


def render_markdown() -> Annotated[str, "sha256:e6828b21aacd9c0d9ce53fcaccd9b6ede997c8e633b9662417d8b6fb29b9df68"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: core-generate-index\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generate spec index routes agents to TDs that support CB lifecycle generation.\"\n---\n\n# cclab-sdd Generate Specs\n\nDiagram, code generation, and template library. Mirrors `src/tools/generate/`.\n\n## Specs\n<!-- type: doc lang: markdown -->\n\n| Spec | Files | Description |\n|------|-------|-------------|\n| [architecture](architecture.md) | — | Generate subsystem architecture |\n| [codegen-system](codegen-system.md) | generators/ | Code generation system architecture |\n| [json-schema-core](json-schema-core.md) | schema/ | JSON Schema implementation |\n| [spec-validator](spec-validator.md) | validator/ | Spec completeness checker |\n| [template-engine](template-engine.md) | engine/ | Tera template integration |\n| [generator-axum](generator-axum.md) | generators/axum.rs | Axum (Rust) code generator |\n| [generator-express](generator-express.md) | generators/express.rs | Express.js code generator |\n| [generator-fastapi](generator-fastapi.md) | generators/fastapi.rs | FastAPI (Python) code generator |\n| [test-generation](test-generation.md) | — | Test generation integration |\n| [mermaid-plus-format](mermaid-plus-format.md) | diagrams/mermaid_plus/ | Mermaid+ YAML frontmatter format |\n| [mermaid-plus-conversion](mermaid-plus-conversion.md) | diagrams/mermaid_plus/generator.rs | YAML → Mermaid rendering |\n| [block-plus-spec](block-plus-spec.md) | diagrams/block_plus/ | Block diagram+ schema |\n| [requirement-plus-enhancement](requirement-plus-enhancement.md) | diagrams/requirement_plus/ | Requirement diagram+ spec |\n\n## Spec IR\n<!-- type: doc lang: markdown -->\n\n| Spec | Files | Description |\n|------|-------|-------------|\n| [spec-ir-contract](spec-ir-contract.md) | spec_ir/types.rs | IR type contract |\n| [spec-ir-evaluation](spec-ir-evaluation.md) | spec_ir/orchestrator.rs, spec_ir/types.rs | Spec↔Code gap analysis |\n| [spec-ir-schema](spec-ir-schema.md) | spec_ir/types.rs | Spec IR YAML schema |\n\n## Spec-to-Code\n<!-- type: doc lang: markdown -->\n\n| Spec | Files | Description |\n|------|-------|-------------|\n| [code-generator-contract](code-generator-contract.md) | spec_ir/codegen.rs | Generator contract definition |\n| [spec-model](spec-model.md) | spec_ir/types.rs | Spec model for code generation |\n\n## Templates\n<!-- type: doc lang: markdown -->\n\n| Spec | Description |\n|------|-------------|\n| [template-claude-md](template-claude-md.md) | CLAUDE.md SDD section template |\n| [template-knowledge-index](template-knowledge-index.md) | Knowledge index template |\n| [template-mcp-configs](template-mcp-configs.md) | Config template (legacy) |\n"
