---
change: sdd-codegen-completion
group: core-codegen
date: 2026-03-20
written_by: artifact_cli
---

# Reference Context

| Spec | Group | Relevance | Key Requirements |
|------|-------|-----------|------------------|
| code-generator-contract | core-codegen | high | Contract between specs (input) and generators (output): API spec requestBody → Pydantic/TS/Rust structs, security scheme → auth dependency, Sequence+ messages → function signatures and call ordering, Flowchart+ db_query → session injection, Class+ dependencies → DI wiring, ERD+ entities → ORM models, Requirement+ scenarios → test functions. Inference rules: flowchart with db_query, api_call, security → auto-inject dependencies. Sequence Plus to Code: detailed mapping for handler implementation, error handling, DI wiring patterns. |
| spec-ir-contract | core-codegen | high | R1-R5: SpecIR enum for all spec type variants, SpecMetadata struct (spec ID, version, scope), SpecBundle composition for multiple specs. From impls for SpecIR conversions. Type hierarchy class diagram. Core internal representation (IR) data model used by all generators and validators. Supports cross-section composition in Phase 2. |
| codegen-system | core-codegen | high | R1-R7: System architecture for unified code generation. R1: SpecIR unified internal representation. R2-R4: Schema, REST API, test generators. R5: Generator trait and pluggable architecture. R6-R7: Validation pipeline and template-based rendering. Data flow: SpecBundle → Validators → SpecIR → Generator routing → CodeOutput. Phase 1 covers single-section codegen (#932). |
| generator-axum | rest-api-codegen | high | R1-R4: Axum REST API code generator target. OpenAPI spec → Axum route handlers. Axum patterns: Router construction, Handler trait, extractors (Json, Path, Query). Generation flow: parse OpenAPI → build route definitions → render handler code → integration with state/middleware. Phase 1 target for #932. Request/response type generation. |
| generator-fastapi | rest-api-codegen | high | R1-R6: FastAPI REST API code generator target. OpenAPI spec → FastAPI endpoints. FastAPI patterns: FastAPI app, @app.route decorators, Pydantic models for request/response. Generation flow: parse OpenAPI → build endpoint definitions → render endpoint code. Phase 1 target for #932 alongside Axum. Dependency injection with Depends(). |
| template-engine | template-system | high | R1-R4: Tera template engine integration for code generation. TemplateEngine class with render() method. Template context building. Rendering flow: parse template → populate context → generate output. Embedded templates via include_str! macro (Q2 answer: templates in crates/cclab-sdd/templates/, no runtime filesystem dependency). Template loading, variable interpolation, filter functions. |
| test-generation | test-codegen | medium | R1-R4: Test generation integration (standalone test files only). Sequence diagram for test generation flow: SpecBundle → TestGenerator → standalone test files. RequirementPlus to test scaffold generation. Phase 1 (#933): standalone test file generation only, no cclab-probe DI/fixture integration. Deferred Q4: cclab-probe fixture DI and advanced test patterns for Phase 2. |
| spec-ir-schema | spec-ir-system | medium | R1-R3: SpecIR YAML manifest schema (Kubernetes-style: apiVersion, kind, metadata, spec). Standard envelope for specs. JSON Schema api_spec type. Class diagram for manifest model. Language-agnostic interface between SDD producer and Lens consumer. Supports roundtrip serialization of SpecIR back to YAML. |
| json-schema-core | schema-codegen | medium | R1-R3: JSON Schema core structures and parsing. Version support: Draft 7, 2019-09, 2020-12. Parsing JSON Schema strings into strongly-typed Rust structures. Schema validation logic. Used by schema generator in Phase 1 (#932): JSON Schema → Rust struct code generation. Type inference rules for properties, required fields, enums. |

