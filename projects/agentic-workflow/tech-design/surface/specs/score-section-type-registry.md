---
id: score-section-type-registry
fill_sections: [schema, changes]
summary: Authoritative registry of approved TD section types — name, family, generator binding, format-priority tier, heading-match rule. Source-of-truth surface that `aw td check --section-type-conformance` parses.
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Score Section-Type Registry

## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: https://json-schema.org/draft/2020-12/schema
$id: score-section-type-registry
title: ScoreSectionTypeRegistry
type: object
required: [version, format_priority, families, section_types, deprecated, extension_protocol]
properties:
  version:
    const: "1"
  format_priority:
    description: Format-precedence tiers used when authoring TD sections; first wins.
    type: array
    items: { type: string }
    const: [openrpc, openapi, asyncapi, json-schema, mermaid-plus, yaml-manifest, rust-source-unit, text-source-unit, source-template, markdown]
  families:
    description: SectionKind families and their format tier. Parser is shared.
    type: object
    additionalProperties: false
    properties:
      MermaidFamily:    { tier: mermaid-plus,  parser: projects/agentic-workflow/src/td_ast/parse.rs }
      JsonSchemaFamily: { tier: json-schema,   parser: projects/agentic-workflow/src/td_ast/parse.rs }
      OpenRpcFamily:    { tier: openrpc,       parser: projects/agentic-workflow/src/td_ast/parse.rs }
      OpenApiFamily:    { tier: openapi,       parser: projects/agentic-workflow/src/td_ast/parse.rs }
      AsyncApiFamily:   { tier: asyncapi,      parser: projects/agentic-workflow/src/td_ast/parse.rs }
      MarkdownFamily:   { tier: markdown,      parser: projects/agentic-workflow/src/td_ast/parse.rs }
      CliFamily:        { tier: yaml-manifest, parser: projects/agentic-workflow/src/td_ast/parse.rs }
      ConfigFamily:     { tier: yaml-manifest, parser: projects/agentic-workflow/src/td_ast/parse.rs }
      ChangesFamily:    { tier: yaml-manifest, parser: projects/agentic-workflow/src/td_ast/parse.rs }
      SourceFamily:     { tier: source-template, parser: projects/agentic-workflow/src/td_ast/parse.rs }
      RustSourceUnitFamily: { tier: rust-source-unit, parser: projects/agentic-workflow/src/generate/rust_source_unit.rs }
      TextSourceUnitFamily: { tier: text-source-unit, parser: projects/agentic-workflow/src/generate/apply.rs }
  section_types:
    description: Approved section types that may appear in `fill_sections`. heading_match is the canonical Title-Case H2 form.
    type: array
    items:
      type: object
      required: [name, family, generator, heading_match, use_for]
      properties:
        name:          { type: string }
        family:        { type: string }
        generator:     { type: string, description: "Path to the generator module that emits target-lang code from this section." }
        heading_match: { type: string }
        use_for:       { type: string }
    const:
      - { name: changes,       family: ChangesFamily,    generator: projects/agentic-workflow/src/generators/changes.rs,        heading_match: "Changes",         use_for: "File-change manifest (path + action) consumed by td gen-code" }
      - { name: source,        family: SourceFamily,     generator: workspace-profile source-template,           heading_match: "Source",          use_for: "Whole-file or region source template; language-specific rendering is selected by workspace codegen.profile" }
      - { name: rust-source-unit, family: RustSourceUnitFamily, generator: projects/agentic-workflow/src/generate/rust_source_unit.rs, heading_match: "Source", use_for: "Lossless Rust CST source-unit regeneration; this is the regenerable source path, unlike source-template or artifact replay" }
      - { name: text-source-unit, family: TextSourceUnitFamily, generator: projects/agentic-workflow/src/generate/apply.rs, heading_match: "Source", use_for: "TD-owned verbatim shell/text source-unit regeneration; this is the regenerable source path for opaque scripts, unlike source-template or artifact replay" }
      - { name: scenarios,     family: MermaidFamily,    generator: projects/agentic-workflow/src/generators/scenarios.rs,      heading_match: "Scenarios",       use_for: "BDD acceptance — Given/When/Then" }
      - { name: unit-test,     family: MermaidFamily,    generator: projects/agentic-workflow/src/generators/test_plan.rs,      heading_match: "Unit Test",       use_for: "requirementDiagram of unit test coverage" }
      - { name: e2e-test,      family: JsonSchemaFamily, generator: projects/agentic-workflow/src/generate/gen/rust/tests.rs,   heading_match: "E2E Test",        use_for: "Product journey tests with CLI/API/web/script driver assertions and side effects" }
      - { name: interaction,   family: MermaidFamily,    generator: projects/agentic-workflow/src/generators/sequence.rs,       heading_match: "Interaction",     use_for: "Actor sequenceDiagram" }
      - { name: logic,         family: MermaidFamily,    generator: projects/agentic-workflow/src/generators/flowchart.rs,      heading_match: "Logic",           use_for: "Business-logic flowchart" }
      - { name: dependency,    family: MermaidFamily,    generator: projects/agentic-workflow/src/generators/dependency.rs,     heading_match: "Dependency",      use_for: "classDiagram type hierarchy" }
      - { name: state-machine, family: MermaidFamily,    generator: projects/agentic-workflow/src/generators/state_machine.rs, heading_match: "State Machine",   use_for: "FSM transitions (stateDiagram-v2)" }
      - { name: db-model,      family: MermaidFamily,    generator: projects/agentic-workflow/src/generators/db_model.rs,       heading_match: "Db Model",        use_for: "erDiagram entity-relationship" }
      - { name: mindmap,       family: MermaidFamily,    generator: projects/agentic-workflow/src/generators/mindmap.rs,        heading_match: "Mindmap",         use_for: "Hierarchical overview" }
      - { name: rest-api,      family: OpenApiFamily,    generator: projects/agentic-workflow/src/generators/rest_api.rs,       heading_match: "Rest Api",        use_for: "OpenAPI 3.1 REST contracts" }
      - { name: rpc-api,       family: OpenRpcFamily,    generator: projects/agentic-workflow/src/generators/rpc_api.rs,        heading_match: "Rpc Api",         use_for: "OpenRPC 1.3 JSON-RPC contracts" }
      - { name: async-api,     family: AsyncApiFamily,   generator: projects/agentic-workflow/src/generators/async_api.rs,      heading_match: "Async Api",       use_for: "AsyncAPI 2.6 pub/sub contracts" }
      - { name: cli,           family: CliFamily,        generator: projects/agentic-workflow/src/generators/frontend.rs,       heading_match: "Cli",             use_for: "CLI command tree + args" }
      - { name: schema,        family: JsonSchemaFamily, generator: projects/agentic-workflow/src/generators/frontend.rs,       heading_match: "Schema",          use_for: "Data schemas (JSON Schema with x-mamba-* annotations)" }
      - { name: config,        family: ConfigFamily,     generator: projects/agentic-workflow/src/generators/frontend.rs,       heading_match: "Config",          use_for: "Config-file schema" }
      - { name: wireframe,     family: JsonSchemaFamily, generator: projects/agentic-workflow/src/generators/frontend.rs,       heading_match: "Wireframe",       use_for: "UI layout DSL" }
      - { name: component,     family: JsonSchemaFamily, generator: projects/agentic-workflow/src/generators/frontend.rs,       heading_match: "Component",       use_for: "UI component contract (Custom Elements Manifest)" }
      - { name: design-token,  family: JsonSchemaFamily, generator: projects/agentic-workflow/src/generators/frontend.rs,       heading_match: "Design Token",    use_for: "Design tokens (W3C DTCG)" }
      - { name: manifest,      family: JsonSchemaFamily, generator: projects/agentic-workflow/src/generators/frontend.rs,       heading_match: "Manifest",        use_for: "Package-manifest deps" }
      - { name: tool-contract, family: ConfigFamily,     generator: projects/agentic-workflow/src/cli/ec.rs,                   heading_match: "Tool Contract",   use_for: "AW EC bridge from TD contracts to native vat/rig/meter/guard manifests; arena remains legacy compatibility" }
      - { name: runtime-image, family: JsonSchemaFamily, generator: projects/agentic-workflow/src/generate/gen/operations.rs,   heading_match: "Runtime Image",   use_for: "Container image build contract (Dockerfile, build context, entrypoint, image metadata)" }
      - { name: deployment,    family: JsonSchemaFamily, generator: projects/agentic-workflow/src/generate/gen/operations.rs,   heading_match: "Deployment",      use_for: "Runtime deployment manifests (Kubernetes, Kustomize base/component/overlay resources)" }
  deprecated:
    description: SectionType enum variants the validator rejects in `fill_sections` for new specs.
    type: array
    items:
      type: object
      required: [name, status, replaced_by]
    const:
      - { name: overview,     status: deprecated, replaced_by: "frontmatter `summary:` field or PRD" }
      - { name: requirements, status: deprecated, replaced_by: "issue body (Requirements section) — moved out of TD" }
      - { name: doc,          status: deprecated, replaced_by: "user-facing markdown outside TD" }
  extension_protocol:
    description: How to add a new approved section type. All four steps required.
    type: array
    items: { type: object, required: [step, action, where] }
    const:
      - { step: 1, action: "Extend SectionType enum",        where: projects/agentic-workflow/src/models/spec_rules.rs }
      - { step: 2, action: "Extend SectionKind dispatch",    where: projects/agentic-workflow/src/td_ast/types.rs::SectionKind::for_section_type }
      - { step: 3, action: "Add generator module",           where: projects/agentic-workflow/src/generators/<name>.rs }
      - { step: 4, action: "Add registry entry",             where: this spec's section_types array }
      - { step: 5, action: "Cross-reference in AUTHORING.md", where: .aw/tech-design/AUTHORING.md }
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tech-design/surface/specs/score-section-type-registry.md
    action: create
    section: schema
    impl_mode: hand-written
    reason: R1 of #1212 — authoritative section-type registry
  - path: .aw/tech-design/AUTHORING.md
    action: modify
    section: changes
    impl_mode: hand-written
    reason: cross-reference the registry as the single source of truth for approved section types
```

# Reviews

## Review 1
<!-- type: review lang: markdown -->

**Verdict:** approved

- [schema] Registry covers the approved non-deprecated `SectionType` enum variants, including UI (`wireframe`, `component`, `design-token`) and operations (`runtime-image`, `deployment`) sections, plus the legacy `source` template type and the 3 deprecated types in a separate `deprecated` array. Family→tier mapping is consistent with `SectionKind::for_section_type` in `projects/agentic-workflow/src/td_ast/types.rs`. Generator paths verified against current SDD generator surfaces — frontend-emitted types bind to `frontend.rs`; operations artifacts bind to `generate/gen/operations.rs`.
