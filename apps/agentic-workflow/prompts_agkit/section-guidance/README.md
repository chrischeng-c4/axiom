# Section Guidance

Per-section-type fill guidance for spec authoring. Each file provides instructions for writing one section type, including format, annotation, and content requirements.

Extracted from `sdd/src/tools/create_change_spec.rs::section_fill_guidance()`.

## Usage

When filling a spec section of type `{type}`, load `{type}.md` for guidance. The guidance includes:
- Content requirements
- Annotation format (`<!-- type: X lang: Y -->`)
- Format specifications (Mermaid syntax, OpenAPI version, etc.)
- Example structures

## Section Types (21)

| Type | Lang | File |
|------|------|------|
| overview | markdown | overview.md |
| requirements | markdown | requirements.md |
| scenarios | markdown | scenarios.md |
| test-plan | markdown | test-plan.md |
| changes | yaml | changes.md |
| doc | markdown | doc.md |
| db-model | mermaid | db-model.md |
| schema | json | schema.md |
| state-machine | mermaid | state-machine.md |
| logic | mermaid | logic.md |
| dependency | mermaid | dependency.md |
| interaction | mermaid | interaction.md |
| mindmap | mermaid | mindmap.md |
| rest-api | yaml | rest-api.md |
| rpc-api | json | rpc-api.md |
| async-api | yaml | async-api.md |
| cli | yaml | cli.md |
| config | json | config.md |
| wireframe | yaml | wireframe.md |
| component | json | component.md |
| design-token | json | design-token.md |
