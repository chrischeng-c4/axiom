---
id: sdd-models-tech-stack
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# Tech Stack

## Overview
<!-- type: overview lang: markdown -->

Tech stack inference types for the `sdd` crate, located in `projects/agentic-workflow/src/models/tech_stack.rs`:

- `DesignSystem` — struct (3 required fields: `library: String`, `provides_tokens: bool`, `provides_components: bool`) representing a detected design system library and its capabilities. Used by the section optionality filter to mark `design-token` and `component` sections as optional when a library is detected.
- `Language` — 5-variant unit enum (`Rust`, `Python`, `JavaScript`, `TypeScript`, `Schemas`) representing the primary programming language detected from a project manifest. The first dogfood use of `x-rust-enum.serde_rename_all: lowercase`, which emits a container-level `#[serde(rename_all = "lowercase")]` attribute on the generated enum.
- `TechStack` — struct (3 optional fields: `language: Option<Language>`, `framework: Option<String>`, `design_system: Option<DesignSystem>`) representing the full inferred project tech stack. Computed from manifest files; read-only, never serialised to `config.toml`.
- `DesignSystemRegistryEntry` — struct (4 required fields: `package: &'static str`, `library: &'static str`, `provides_tokens: bool`, `provides_components: bool`) used by `infer_tech_stack()` to map npm package names to `DesignSystem` capability flags. Carries only `Debug` (no serde derives) because it is used only in the compile-time `DESIGN_SYSTEM_REGISTRY` constant.

The `DESIGN_SYSTEM_REGISTRY` top-level `pub const` is explicitly out of scope — the generator does not produce top-level `pub const` declarations; the constant remains hand-written outside any `CODEGEN-BEGIN`/`CODEGEN-END` block.

This spec is the dogfood example for `x-rust-enum.serde_rename_all` (implemented in commit 88b58ebc). Codegen inserts `CODEGEN-BEGIN`/`CODEGEN-END` blocks replacing all four type declarations and the serde import they need, while the `DESIGN_SYSTEM_REGISTRY` constant remains untouched as a hand-written region.
## Schema
<!-- type: schema lang: yaml -->

```yaml
$id: sdd-models-tech-stack
description: |
  Tech stack inference types for the sdd crate, located in
  projects/agentic-workflow/src/models/tech_stack.rs. Defines project tech stack
  information inferred from manifest files. Used by the section
  optionality filter to determine which sections are optional.

definitions:

  DesignSystem:
    type: object
    required: [library, provides_tokens, provides_components]
    description: |
      Detected design system library and its capabilities.
      When detected, the optionality filter uses provides_tokens and
      provides_components to mark design-token and component sections
      as optional in spec_plan.sections.
    properties:
      library:
        type: string
        description: "Canonical library identifier (e.g., \"mui\", \"antd\", \"chakra\")."
      provides_tokens:
        type: boolean
        description: |
          True if the library ships a complete design token system
          (colors, spacing, typography). When true, design-token section
          becomes optional.
      provides_components:
        type: boolean
        description: |
          True if the library provides a full component set (buttons, inputs,
          layouts). When true, component section becomes optional.
    x-rust-struct:
      derive: [Debug, Clone, PartialEq, Eq, Serialize, Deserialize]

  Language:
    type: string
    enum:
      - Rust
      - Python
      - JavaScript
      - TypeScript
      - Schemas
    description: "Primary programming language detected from manifest."
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]
      serde_rename_all: lowercase
      variants:
        - name: Rust
          doc: "Rust language (Cargo.toml manifest)."
        - name: Python
          doc: "Python language (pyproject.toml manifest)."
        - name: JavaScript
          doc: "JavaScript language (package.json without TypeScript)."
        - name: TypeScript
          doc: "TypeScript language (package.json with TypeScript dependency)."
        - name: Schemas
          doc: "Schema-only directories with no executable language manifest."

  TechStack:
    type: object
    required: []
    description: |
      Inferred project tech stack.
      Computed from manifest files (Cargo.toml, pyproject.toml, package.json).
      Read-only — never serialized to config.toml.
      Consumed by the section optionality filter (apply_section_optionality)
      and the wireframe generator (framework-specific output).
    properties:
      language:
        $ref: "#/definitions/Language"
        description: "Primary programming language detected from manifest."
      framework:
        type: [string, "null"]
        description: "Detected web/app framework (e.g., \"react\", \"vue\", \"axum\", \"fastapi\")."
      design_system:
        $ref: "#/definitions/DesignSystem"
        description: |
          Detected design system library and capabilities.
          None when no known design system found — downstream consumers
          treat None as all frontend sections required.
    x-rust-struct:
      derive: [Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize]

  DesignSystemRegistryEntry:
    type: object
    required: [package, library, provides_tokens, provides_components]
    description: |
      Known design system registry entry for package detection.
      Used by infer_tech_stack() to map npm package names to DesignSystem
      capability flags.
    properties:
      package:
        type: string
        description: "npm package name to match in dependencies/devDependencies."
        x-rust-type: "&'static str"
      library:
        type: string
        description: "Canonical library identifier."
        x-rust-type: "&'static str"
      provides_tokens:
        type: boolean
        description: "Whether the library ships a complete design token system."
      provides_components:
        type: boolean
        description: "Whether the library provides a full component set."
    x-rust-struct:
      derive: [Debug]
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/models/tech_stack.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - DesignSystem
      - Language
      - TechStack
      - DesignSystemRegistryEntry
    description: |
      Codegen replaces all four type declarations (DesignSystem, Language,
      TechStack, DesignSystemRegistryEntry) with a single CODEGEN-BEGIN/CODEGEN-END
      block. No impl blocks exist for these types in the source — only the
      struct/enum declarations are replaced. All generated items carry @spec
      markers referencing this file's #schema anchor.
      Key codegen output:
      - Language enum emits container-level #[serde(rename_all = "lowercase")]
        via x-rust-enum.serde_rename_all (first dogfood use of this field).
      - DesignSystemRegistryEntry emits only #[derive(Debug)] (no serde derives)
        because it is used exclusively in the compile-time DESIGN_SYSTEM_REGISTRY
        constant which is hand-written outside the CODEGEN block.
      - TechStack fields (language, design_system) are Option-wrapped; generator
        emits #[serde(default)] per the existing Option field rule.

  - path: projects/agentic-workflow/src/models/tech_stack.rs
    action: modify
    section: schema
    impl_mode: hand-written
    description: |
      Hand-written regions OUTSIDE CODEGEN-BEGIN/END (not touched by codegen):
      - pub const DESIGN_SYSTEM_REGISTRY: &[DesignSystemRegistryEntry] — top-level
        static array initializer; the generator does not produce pub const
        declarations so this constant remains hand-written indefinitely.
      This region carries no @spec marker (healthy hand-written region per
      audit policy). aw td gen-code must not touch them on action: modify.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] All four types described accurately with correct field counts, types, and derive rationale. `DESIGN_SYSTEM_REGISTRY` out-of-scope note is clear and precise.
- [schema] `Language` enum correctly carries `serde_rename_all: lowercase` at container level — first dogfood use per R2. `DesignSystemRegistryEntry` correctly has only `[Debug]` derive with `x-rust-type: "&'static str"` on string fields per R3. `TechStack.required: []` correctly maps all fields to `Option<T>`. Cross-refs (`$ref`) between `TechStack` and its sub-types are consistent.
- [changes] Two-entry changes block cleanly separates `impl_mode: codegen` (four type declarations) from `impl_mode: hand-written` (DESIGN_SYSTEM_REGISTRY + infer_tech_stack). The `replaces:` list covers all four types. No ambiguity for codegen boundary enforcement.
