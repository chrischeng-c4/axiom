---
id: aurora-codegen
type: proposal
version: 1
created_at: 2026-02-02T13:51:59.690667+00:00
updated_at: 2026-02-02T13:51:59.690667+00:00
author: mcp
status: proposed
iteration: 1
summary: "Implement a template-based code generation system in cclab-aurora using Tera, supporting FastAPI, Express, and Axum, with integration into cclab-probe for testing."
history:
  - timestamp: 2026-02-02T13:51:59.690667+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
  - timestamp: 2026-02-02T13:52:19.253846+00:00
    agent: "gemini:pro"
    tool: "revise_proposal"
    action: "revised"
  - timestamp: 2026-02-02T13:52:38.444051+00:00
    agent: "codex:max"
    tool: "review_proposal"
    action: "reviewed"
impact:
  scope: major
  affected_files: 25
  new_files: 15
affected_specs:
  - id: aurora-codegen-system
    path: specs/aurora-codegen-system.md
    depends: []
  - id: json-schema-core
    path: specs/json-schema-core.md
    depends: [aurora-codegen-system]
  - id: spec-validator
    path: specs/spec-validator.md
    depends: [json-schema-core]
  - id: template-engine
    path: specs/template-engine.md
    depends: [json-schema-core]
  - id: generator-fastapi
    path: specs/generator-fastapi.md
    depends: [template-engine]
  - id: generator-express
    path: specs/generator-express.md
    depends: [template-engine]
  - id: generator-axum
    path: specs/generator-axum.md
    depends: [template-engine]
  - id: test-generation
    path: specs/test-generation.md
    depends: [generator-fastapi, generator-express, generator-axum]---

<proposal>

# Change: aurora-codegen

## Summary

Implement a template-based code generation system in cclab-aurora using Tera, supporting FastAPI, Express, and Axum, with integration into cclab-probe for testing.

## Why

Code generation is currently ad-hoc and scattered. We need a standardized, template-based approach to scale to multiple languages and frameworks. A template-based system in `cclab-aurora` using Tera will provide a robust, maintainable, and extensible foundation for generating high-quality code from specs (JSON Schema, OpenAPI), supporting the 'Nova' agent's capabilities. This also integrates with `cclab-probe` for automated test generation.

## What Changes

- Add `tera` dependency to `crates/cclab-aurora/Cargo.toml`
- Implement `crates/cclab-aurora/src/schema/mod.rs` (JSON Schema IR)
- Implement `crates/cclab-aurora/src/validator/mod.rs` (Spec Completeness Validator)
- Implement `crates/cclab-aurora/src/engine/tera.rs` (Template Engine)
- Implement `crates/cclab-aurora/src/generators/fastapi/mod.rs`
- Implement `crates/cclab-aurora/src/generators/express/mod.rs`
- Implement `crates/cclab-aurora/src/generators/axum/mod.rs`
- Implement `crates/cclab-aurora/src/testing/probe.rs` (Integration with cclab-probe)
- Add templates in `crates/cclab-aurora/templates/`

## Impact

- **Scope**: major
- **Affected Files**: ~25
- **New Files**: ~15
- Affected specs:
  - `aurora-codegen-system` (no dependencies)
  - `json-schema-core` → depends on: `aurora-codegen-system`
  - `spec-validator` → depends on: `json-schema-core`
  - `template-engine` → depends on: `json-schema-core`
  - `generator-fastapi` → depends on: `template-engine`
  - `generator-express` → depends on: `template-engine`
  - `generator-axum` → depends on: `template-engine`
  - `test-generation` → depends on: `generator-fastapi`, `generator-express`, `generator-axum`
- Affected code: `crates/cclab-aurora/Cargo.toml`, `crates/cclab-aurora/src/lib.rs`, `crates/cclab-aurora/src/schema/`, `crates/cclab-aurora/src/validator/`, `crates/cclab-aurora/src/engine/`, `crates/cclab-aurora/src/generators/`, `crates/cclab-aurora/src/testing/`
- **Breaking Changes**: None. This is an additive change to cclab-aurora.

</proposal>
