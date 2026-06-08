---
change_id: 191
type: codebase_context
created_at: 2026-02-12T10:17:41.725487+00:00
updated_at: 2026-02-12T10:17:41.725487+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - prism_symbols
  - read_file
---

# Codebase Context

## Analyzed Files

- **crates/cclab-aurora/src/diagrams/mod.rs** — Diagram module registry and re-exports.
  - symbols: `requirement_plus`, `flowchart_plus`
- **crates/cclab-aurora/src/diagrams/requirement_plus/schema.rs** — Data model for Requirement+ diagrams, including SysML v1.6 types.
  - symbols: `RequirementDiagramDef`, `RequirementDefPlus`, `RequirementTypePlus`
- **crates/cclab-aurora/src/diagrams/requirement_plus/generator.rs** — Generator logic for Requirement+ diagrams.
  - symbols: `RequirementPlusGenerator`, `generate_mermaid`, `generate_frontmatter`
- **crates/cclab-aurora/src/mcp/tools.rs** — MCP tool definitions and registry.
  - symbols: `aurora_generate_requirement_plus`, `AuroraTools::list`
- **crates/cclab-aurora/src/lib.rs** — Library entry point.

## Prism Results

- **prism_symbols** (query: `crates/cclab-aurora/src/diagrams/requirement_plus/generator.rs`)
  - Confirmed the pattern for Plus generators: generate, generate_frontmatter, and generate_mermaid functions.
- **read_file** (query: `crates/cclab-aurora/src/diagrams/flowchart_plus/schema.rs`)
  - Verified that block diagram types are not currently part of the flowchart schema.

## Dependency Graph

- crates/cclab-aurora/src/lib.rs -> crates/cclab-aurora/src/diagrams/mod.rs
- crates/cclab-aurora/src/diagrams/mod.rs -> crates/cclab-aurora/src/diagrams/requirement_plus/mod.rs
- crates/cclab-aurora/src/diagrams/requirement_plus/mod.rs -> crates/cclab-aurora/src/diagrams/requirement_plus/schema.rs
- crates/cclab-aurora/src/diagrams/requirement_plus/mod.rs -> crates/cclab-aurora/src/diagrams/requirement_plus/generator.rs
- crates/cclab-aurora/src/mcp/tools.rs -> crates/cclab-aurora/src/diagrams/mod.rs
