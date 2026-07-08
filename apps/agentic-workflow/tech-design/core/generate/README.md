---
id: core-generate-index
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generate spec index routes agents to TDs that support CB lifecycle generation."
---

# cclab-sdd Generate Specs

Diagram, code generation, and template library. Mirrors `src/tools/generate/`.

## Specs
<!-- type: doc lang: markdown -->

| Spec | Files | Description |
|------|-------|-------------|
| [architecture](architecture.md) | — | Generate subsystem architecture |
| [codegen-system](codegen-system.md) | generators/ | Code generation system architecture |
| [json-schema-core](json-schema-core.md) | schema/ | JSON Schema implementation |
| [spec-validator](spec-validator.md) | validator/ | Spec completeness checker |
| [template-engine](template-engine.md) | engine/ | Tera template integration |
| [generator-axum](generator-axum.md) | generators/axum.rs | Axum (Rust) code generator |
| [generator-express](generator-express.md) | generators/express.rs | Express.js code generator |
| [generator-fastapi](generator-fastapi.md) | generators/fastapi.rs | FastAPI (Python) code generator |
| [test-generation](test-generation.md) | — | Test generation integration |
| [mermaid-plus-format](mermaid-plus-format.md) | diagrams/mermaid_plus/ | Mermaid+ YAML frontmatter format |
| [mermaid-plus-conversion](mermaid-plus-conversion.md) | diagrams/mermaid_plus/generator.rs | YAML → Mermaid rendering |
| [block-plus-spec](block-plus-spec.md) | diagrams/block_plus/ | Block diagram+ schema |
| [requirement-plus-enhancement](requirement-plus-enhancement.md) | diagrams/requirement_plus/ | Requirement diagram+ spec |

## Spec IR
<!-- type: doc lang: markdown -->

| Spec | Files | Description |
|------|-------|-------------|
| [spec-ir-contract](spec-ir-contract.md) | spec_ir/types.rs | IR type contract |
| [spec-ir-evaluation](spec-ir-evaluation.md) | spec_ir/orchestrator.rs, spec_ir/types.rs | Spec↔Code gap analysis |
| [spec-ir-schema](spec-ir-schema.md) | spec_ir/types.rs | Spec IR YAML schema |

## Spec-to-Code
<!-- type: doc lang: markdown -->

| Spec | Files | Description |
|------|-------|-------------|
| [code-generator-contract](code-generator-contract.md) | spec_ir/codegen.rs | Generator contract definition |
| [spec-model](spec-model.md) | spec_ir/types.rs | Spec model for code generation |

## Templates
<!-- type: doc lang: markdown -->

| Spec | Description |
|------|-------------|
| [template-claude-md](template-claude-md.md) | CLAUDE.md SDD section template |
| [template-knowledge-index](template-knowledge-index.md) | Knowledge index template |
| [template-mcp-configs](template-mcp-configs.md) | Config template (legacy) |
