---
id: prism-codegen-unification
type: spec
title: "Prism Codegen Unification & Generator Migration"
version: 1
spec_type: utility
spec_group: cclab-prism
merge_strategy: new
created_at: 2026-02-14T11:30:06.396922+00:00
updated_at: 2026-02-14T11:30:06.396922+00:00
requirements:
  total: 6
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
depends:
  - spec-ir-contract
changes:
  - file: crates/cclab-prism/src/gen/framework/mod.rs
    action: CREATE
    description: "New framework/ submodule holding migrated FastAPI, Express, Axum generators"
  - file: crates/cclab-prism/src/gen/framework/fastapi.rs
    action: CREATE
    description: "Migrated FastApiGenerator adapted for SpecIR input"
  - file: crates/cclab-prism/src/gen/framework/express.rs
    action: CREATE
    description: "Migrated ExpressGenerator adapted for SpecIR input"
  - file: crates/cclab-prism/src/gen/framework/axum.rs
    action: CREATE
    description: "Migrated AxumGenerator adapted for SpecIR input"
  - file: crates/cclab-prism/src/gen/traits.rs
    action: MODIFY
    description: "Update CodeGenerator trait to use SpecIR input"
  - file: crates/cclab-prism/src/gen/registry.rs
    action: CREATE
    description: "GeneratorRegistry for dispatching SpecIR to generators"
  - file: crates/cclab-prism/src/gen/mod.rs
    action: MODIFY
    description: "Add framework/ and registry submodules"
  - file: crates/cclab-prism/src/mcp/tools.rs
    action: MODIFY
    description: "Update prism_generate_from_spec to accept SpecIR"
  - file: crates/cclab-aurora/src/generators/
    action: DELETE
    description: "Remove entire generators module from Aurora"
  - file: crates/cclab-aurora/src/engine/
    action: DELETE
    description: "Remove TemplateEngine from Aurora (moved to Prism)"
  - file: crates/cclab-aurora/src/lib.rs
    action: MODIFY
    description: "Remove generators and engine exports"
history:
  - timestamp: 2026-02-14T11:30:06.396922+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Prism Codegen Unification & Generator Migration

## Overview

Migrate Aurora's framework generators (FastAPI, Express, Axum) to Prism's gen/ module and unify all code generators under a single CodeGenerator trait that consumes SpecIR. This addresses issues #326 (migrate generators), #327 (Aurora scope down), and #328 (Prism unify). After migration, Aurora only owns spec format/validation/diagrams; Prism owns all code generation. Resolves the template-engine vs direct-string generation split by standardizing on SpecIR input.

## Requirements

### R1 - Migrate Aurora generators to Prism

```yaml
id: R1
priority: high
status: draft
```

Move FastApiGenerator, ExpressGenerator, and AxumGenerator from crates/cclab-aurora/src/generators/ to crates/cclab-prism/src/gen/framework/. Include the Tera TemplateEngine dependency. After migration, delete crates/cclab-aurora/src/generators/ entirely.

### R2 - Unify CodeGenerator trait with SpecIR input

```yaml
id: R2
priority: high
status: draft
```

Update Prism's CodeGenerator trait in gen/traits.rs so can_generate() and generate() accept SpecIR (from cclab-aurora) instead of the existing per-crate typed methods. All generators (both migrated framework generators and existing per-crate generators) must implement the unified trait.

### R3 - Framework generator SpecIR adaptation

```yaml
id: R3
priority: high
status: draft
```

Adapt migrated FastAPI/Express/Axum generators to accept SpecIR::Api variant instead of raw SchemaIR. The generators extract the JsonSchema from SpecIR::Api and proceed with existing template-based generation logic.

### R4 - Remove generators from Aurora

```yaml
id: R4
priority: high
status: draft
```

After migration, remove the generators/ module from cclab-aurora/src/ and its engine/ module (TemplateEngine). Update Aurora's lib.rs to no longer export generators. Aurora retains: schema/, diagrams/, specs/, validator/, mcp/, spec_ir/.

### R5 - Update Prism MCP tools

```yaml
id: R5
priority: medium
status: draft
```

Update prism_generate_from_spec MCP tool to accept SpecIR input and route to the appropriate generator based on SpecIR variant and target framework. Expose framework generators (FastAPI, Express, Axum) through the existing tool.

### R6 - Generator registry

```yaml
id: R6
priority: medium
status: draft
```

Implement a GeneratorRegistry in Prism that holds all registered CodeGenerator implementations and dispatches SpecIR to the correct generator based on can_generate() results. This replaces ad-hoc generator selection.

## Acceptance Criteria

### Scenario: Generate FastAPI from SpecIR

- **GIVEN** A SpecIR::Api variant containing an OpenAPI schema for a user service
- **WHEN** GeneratorRegistry.generate(spec_ir, target='fastapi') is called
- **THEN** FastApiGenerator produces Python FastAPI project files identical to previous Aurora output

### Scenario: Generate Axum from SpecIR

- **GIVEN** A SpecIR::Api variant containing an OpenAPI schema
- **WHEN** GeneratorRegistry.generate(spec_ir, target='axum') is called
- **THEN** AxumGenerator produces Rust Axum project files

### Scenario: Aurora no longer exports generators

- **GIVEN** The migration is complete
- **WHEN** Code tries to import from cclab_aurora::generators
- **THEN** Compilation fails — generators module no longer exists in Aurora

### Scenario: Per-crate generators still work

- **GIVEN** Existing ShieldGenerator in Prism gen/python/
- **WHEN** ShieldGenerator.can_generate(spec_ir) is called with a matching SpecIR
- **THEN** Returns true and generate() produces Shield-specific Python code

</spec>
