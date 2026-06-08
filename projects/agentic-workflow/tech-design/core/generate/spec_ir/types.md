---
id: sdd-generate-spec-ir-types
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# SpecIR Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/spec_ir/types.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AttributeDef` | projects/agentic-workflow/src/generate/spec_ir/types.rs | struct | pub | 143 |  |
| `BundleMetadata` | projects/agentic-workflow/src/generate/spec_ir/types.rs | struct | pub | 303 |  |
| `ComponentSpec` | projects/agentic-workflow/src/generate/spec_ir/types.rs | struct | pub | 122 |  |
| `DeploySpec` | projects/agentic-workflow/src/generate/spec_ir/types.rs | struct | pub | 21 |  |
| `DesignTokenEntry` | projects/agentic-workflow/src/generate/spec_ir/types.rs | struct | pub | 197 |  |
| `DesignTokenSpec` | projects/agentic-workflow/src/generate/spec_ir/types.rs | struct | pub | 185 |  |
| `EnvVar` | projects/agentic-workflow/src/generate/spec_ir/types.rs | struct | pub | 45 |  |
| `EventDef` | projects/agentic-workflow/src/generate/spec_ir/types.rs | struct | pub | 171 |  |
| `PropDef` | projects/agentic-workflow/src/generate/spec_ir/types.rs | struct | pub | 89 |  |
| `ResourceLimits` | projects/agentic-workflow/src/generate/spec_ir/types.rs | struct | pub | 59 |  |
| `SlotDef` | projects/agentic-workflow/src/generate/spec_ir/types.rs | struct | pub | 160 |  |
| `SpecBundle` | projects/agentic-workflow/src/generate/spec_ir/types.rs | struct | pub | 315 |  |
| `SpecIR` | projects/agentic-workflow/src/generate/spec_ir/types.rs | enum | pub | 231 |  |
| `SpecMetadata` | projects/agentic-workflow/src/generate/spec_ir/types.rs | struct | pub | 212 |  |
| `WireframeNode` | projects/agentic-workflow/src/generate/spec_ir/types.rs | struct | pub | 108 |  |
| `WireframeSpec` | projects/agentic-workflow/src/generate/spec_ir/types.rs | struct | pub | 71 |  |
| `add_dependency` | projects/agentic-workflow/src/generate/spec_ir/types.rs | function | pub | 538 | add_dependency(&mut self, from: usize, to: usize) |
| `is_empty` | projects/agentic-workflow/src/generate/spec_ir/types.rs | function | pub | 548 | is_empty(&self) -> bool |
| `kind` | projects/agentic-workflow/src/generate/spec_ir/types.rs | function | pub | 354 | kind(&self) -> &'static str |
| `len` | projects/agentic-workflow/src/generate/spec_ir/types.rs | function | pub | 543 | len(&self) -> usize |
| `metadata` | projects/agentic-workflow/src/generate/spec_ir/types.rs | function | pub | 371 | metadata(&self) -> &SpecMetadata |
| `metadata_mut` | projects/agentic-workflow/src/generate/spec_ir/types.rs | function | pub | 388 | metadata_mut(&mut self) -> &mut SpecMetadata |
| `new` | projects/agentic-workflow/src/generate/spec_ir/types.rs | function | pub | 526 | new() -> Self |
| `push` | projects/agentic-workflow/src/generate/spec_ir/types.rs | function | pub | 531 | push(&mut self, spec: SpecIR) -> usize |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  DeploySpec:
    type: object
    required: [name, image, port, replicas, env]
    description: Kubernetes deployment specification (for the deploy section type). Targets k8s Deployment + Service manifest generation.
    properties:
      name:
        type: string
        x-serde-default: true
        description: "Application name — used as metadata.name in k8s resources."
      image:
        type: string
        x-serde-default: true
        description: "Container image reference (e.g. nginx:1.21)."
      port:
        type: integer
        x-rust-type: "u16"
        x-serde-default: "default_deploy_port"
        description: "Port the container listens on. Defaults to 8080."
      replicas:
        type: integer
        x-rust-type: "u32"
        x-serde-default: "default_replicas"
        description: "Number of desired pod replicas. Defaults to 1."
      env:
        type: array
        items: { type: object }
        x-rust-type: "Vec<EnvVar>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Container environment variables."
      resources:
        type: object
        x-rust-type: "Option<ResourceLimits>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Optional CPU/memory resource limits."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  EnvVar:
    type: object
    required: [name]
    description: A single environment variable entry.
    properties:
      name:
        type: string
        description: "Variable name."
      value:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Literal value (mutually exclusive with value_from)."
      value_from:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Source reference, e.g. secretKeyRef:my-secret:key."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ResourceLimits:
    type: object
    required: []
    description: CPU / memory resource constraints.
    properties:
      cpu:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "CPU limit, e.g. 500m."
      memory:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Memory limit, e.g. 256Mi."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  WireframeSpec:
    type: object
    required: [name, component_type, props, layout]
    description: Wireframe specification for React component scaffold generation (for the wireframe section type).
    properties:
      name:
        type: string
        x-serde-default: true
        description: "Component name in PascalCase (e.g. UserCard)."
      component_type:
        type: string
        x-serde-default: true
        description: "High-level component type: page, layout, card, form, etc."
      props:
        type: array
        items: { type: object }
        x-rust-type: "Vec<PropDef>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "TypeScript props exposed by the component."
      layout:
        type: array
        items: { type: object }
        x-rust-type: "Vec<WireframeNode>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Top-level layout nodes rendered by the component."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  PropDef:
    type: object
    required: [name, prop_type, required]
    description: A TypeScript prop definition for the React component.
    properties:
      name:
        type: string
        description: "Prop name in camelCase."
      prop_type:
        type: string
        description: "TypeScript type string, e.g. string, number, User."
      required:
        type: boolean
        x-serde-default: true
        description: "Whether the prop is required (no ? in the interface)."
      default_value:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Default value expression as a string (used in destructuring)."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Optional JSDoc description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  WireframeNode:
    type: object
    required: [kind]
    description: A single layout node in a wireframe tree.
    properties:
      kind:
        type: string
        description: "Element kind: text, button, input, list, container, etc."
      label:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Display label or placeholder text."
      children:
        type: array
        items: { type: object }
        x-rust-type: "Vec<WireframeNode>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Nested child nodes."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  ComponentSpec:
    type: object
    required: [tag_name, summary, attributes, slots, events]
    description: Component Element Model (CEM) spec for TypeScript interface + component skeleton generation (for the component section type).
    properties:
      tag_name:
        type: string
        x-serde-default: true
        description: "Kebab-case custom element tag name (e.g. my-button)."
      summary:
        type: string
        x-serde-default: true
        description: "One-line summary shown in generated JSDoc."
      attributes:
        type: array
        items: { type: object }
        x-rust-type: "Vec<AttributeDef>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Reflected HTML attributes / observed properties."
      slots:
        type: array
        items: { type: object }
        x-rust-type: "Vec<SlotDef>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Named slots."
      events:
        type: array
        items: { type: object }
        x-rust-type: "Vec<EventDef>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Custom events emitted by this component."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  AttributeDef:
    type: object
    required: [name, attr_type, required]
    description: A component attribute / property definition.
    properties:
      name:
        type: string
        description: "Attribute name in kebab-case."
      attr_type:
        type: string
        x-serde-rename: "type"
        x-serde-default: true
        description: "TypeScript type string."
      required:
        type: boolean
        x-serde-default: true
        description: "Whether the attribute is required."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Optional description for JSDoc."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  SlotDef:
    type: object
    required: [name]
    description: A named slot definition.
    properties:
      name:
        type: string
        description: "Slot name (empty string for the default slot)."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Optional slot description."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  EventDef:
    type: object
    required: [name]
    description: A custom event emitted by the component.
    properties:
      name:
        type: string
        description: "Event name in kebab-case."
      detail_type:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "TypeScript type for CustomEvent<T> detail."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Optional description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  DesignTokenSpec:
    type: object
    required: [name, tokens]
    description: Design Token Community Group (DTCG) spec for CSS custom property / Tailwind token generation (for the design-token section type).
    properties:
      name:
        type: string
        x-serde-default: true
        description: "Token collection name used as a CSS prefix (e.g. theme)."
      tokens:
        type: array
        items: { type: object }
        x-rust-type: "Vec<DesignTokenEntry>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Flat list of token entries."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  DesignTokenEntry:
    type: object
    required: [path, value, token_type]
    description: A single DTCG design token.
    properties:
      path:
        type: string
        description: "Dot-separated path, e.g. color.primary.500."
      value:
        type: string
        description: "Resolved token value as a string, e.g. #3B82F6."
      token_type:
        type: string
        description: "DTCG type: color, dimension, fontWeight, etc."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Optional description for generated comments."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  SpecMetadata:
    type: object
    required: []
    description: Common metadata carried by every SpecIR variant. Enables generators to make routing decisions (can_generate) without parsing the full spec payload.
    properties:
      source_path:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Source file path (relative to project root)."
      spec_group:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Spec group (e.g. sdd)."
      spec_id:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Spec identifier within the group."
      tags:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Tags for filtering / routing."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  SpecIR:
    type: object
    description: Specification Intermediate Representation. Universal input type for code generators. Each variant wraps an existing SDD schema type, carrying it along with SpecMetadata for routing.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      serde_tag: kind
      serde_rename_all: snake_case
      variants:
        - name: Api
          kind: struct
          doc: "API specification from OpenAPI / JSON Schema."
          fields:
            - { name: schema, rust_type: "JsonSchema" }
            - { name: metadata, rust_type: "SpecMetadata", flatten: true }
        - name: StateMachinePlus
          kind: struct
          doc: "StateMachine+ with states, transitions, guards, and actions."
          fields:
            - { name: def, rust_type: "StateMachineDef" }
            - { name: metadata, rust_type: "SpecMetadata", flatten: true }
        - name: FlowchartPlus
          kind: struct
          doc: "Flowchart+ with SemanticType annotations for codegen."
          fields:
            - { name: def, rust_type: "FlowchartDef" }
            - { name: metadata, rust_type: "SpecMetadata", flatten: true }
        - name: ClassPlus
          kind: struct
          doc: "Class+ with DDD stereotypes (entity, valueObject, aggregate, etc.)."
          fields:
            - { name: def, rust_type: "ClassDiagramDef" }
            - { name: metadata, rust_type: "SpecMetadata", flatten: true }
        - name: ErdPlus
          kind: struct
          doc: "ERD+ with PK/FK/UK key types."
          fields:
            - { name: def, rust_type: "ERDDef" }
            - { name: metadata, rust_type: "SpecMetadata", flatten: true }
        - name: SequencePlus
          kind: struct
          doc: "Sequence+ with loops, alt blocks, and activation."
          fields:
            - { name: def, rust_type: "SequenceDef" }
            - { name: metadata, rust_type: "SpecMetadata", flatten: true }
        - name: RequirementPlus
          kind: struct
          doc: "Requirement+ with N:M mapping for test generation."
          fields:
            - { name: def, rust_type: "RequirementDiagramDef" }
            - { name: metadata, rust_type: "SpecMetadata", flatten: true }
        - name: Deploy
          kind: struct
          doc: "Kubernetes deployment specification (deploy section type)."
          fields:
            - { name: spec, rust_type: "DeploySpec" }
            - { name: metadata, rust_type: "SpecMetadata", flatten: true }
        - name: Wireframe
          kind: struct
          doc: "Wireframe specification for React component scaffold (wireframe section type)."
          fields:
            - { name: spec, rust_type: "WireframeSpec" }
            - { name: metadata, rust_type: "SpecMetadata", flatten: true }
        - name: Component
          kind: struct
          doc: "Component Element Model for TypeScript interface generation (component section type)."
          fields:
            - { name: spec, rust_type: "ComponentSpec" }
            - { name: metadata, rust_type: "SpecMetadata", flatten: true }
        - name: DesignToken
          kind: struct
          doc: "Design Token Community Group format for CSS/Tailwind generation (design-token section type)."
          fields:
            - { name: spec, rust_type: "DesignTokenSpec" }
            - { name: metadata, rust_type: "SpecMetadata", flatten: true }

  BundleMetadata:
    type: object
    required: []
    description: Metadata for a SpecBundle.
    properties:
      change_id:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Change ID this bundle was produced for."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
        description: "Human-readable description."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]

  SpecBundle:
    type: object
    required: [specs, metadata]
    description: A bundle of related SpecIR items with a dependency graph. Allows generators to receive the complete context for a change — e.g. an ERD+ alongside the API spec that references its entities.
    properties:
      specs:
        type: array
        items: { type: object }
        x-rust-type: "Vec<SpecIR>"
        description: "Ordered list of specs."
      dependencies:
        type: array
        items: { type: object }
        x-rust-type: "Vec<(usize, usize)>"
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
        description: "Dependency edges as (from_index, to_index) pairs."
      metadata:
        type: object
        x-rust-type: "BundleMetadata"
        x-serde-default: true
        description: "Bundle-level metadata."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/spec_ir/types.rs -->
```rust
//! SpecIR type definitions
//!
//! The universal contract between SDD generate (spec ownership) and Lens (code generation).

use crate::generate::diagrams::{
    ClassDiagramDef, ERDDef, FlowchartDef, RequirementDiagramDef, SequenceDef, StateMachineDef,
};
use crate::generate::schema::JsonSchema;

// ---------------------------------------------------------------------------
// New spec types (deploy, wireframe, component, design-token section types)
// ---------------------------------------------------------------------------

use serde::{Deserialize, Serialize};

/// Kubernetes deployment specification (for the deploy section type). Targets k8s Deployment + Service manifest generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploySpec {
    /// Application name — used as metadata.name in k8s resources.
    #[serde(default)]
    pub name: String,
    /// Container image reference (e.g. nginx:1.21).
    #[serde(default)]
    pub image: String,
    /// Port the container listens on. Defaults to 8080.
    #[serde(default = "default_deploy_port")]
    pub port: u16,
    /// Number of desired pod replicas. Defaults to 1.
    #[serde(default = "default_replicas")]
    pub replicas: u32,
    /// Container environment variables.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<EnvVar>,
    /// Optional CPU/memory resource limits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceLimits>,
}

/// A single environment variable entry.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    /// Variable name.
    pub name: String,
    /// Literal value (mutually exclusive with value_from).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Source reference, e.g. secretKeyRef:my-secret:key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_from: Option<String>,
}

/// CPU / memory resource constraints.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// CPU limit, e.g. 500m.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu: Option<String>,
    /// Memory limit, e.g. 256Mi.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<String>,
}

/// Wireframe specification for React component scaffold generation (for the wireframe section type).
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WireframeSpec {
    /// Component name in PascalCase (e.g. UserCard).
    #[serde(default)]
    pub name: String,
    /// High-level component type: page, layout, card, form, etc.
    #[serde(default)]
    pub component_type: String,
    /// TypeScript props exposed by the component.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<PropDef>,
    /// Top-level layout nodes rendered by the component.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout: Vec<WireframeNode>,
}

/// A TypeScript prop definition for the React component.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropDef {
    /// Prop name in camelCase.
    pub name: String,
    /// TypeScript type string, e.g. string, number, User.
    pub prop_type: String,
    /// Whether the prop is required (no ? in the interface).
    #[serde(default)]
    pub required: bool,
    /// Default value expression as a string (used in destructuring).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    /// Optional JSDoc description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A single layout node in a wireframe tree.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WireframeNode {
    /// Element kind: text, button, input, list, container, etc.
    pub kind: String,
    /// Display label or placeholder text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Nested child nodes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<WireframeNode>,
}

/// Component Element Model (CEM) spec for TypeScript interface + component skeleton generation (for the component section type).
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentSpec {
    /// Kebab-case custom element tag name (e.g. my-button).
    #[serde(default)]
    pub tag_name: String,
    /// One-line summary shown in generated JSDoc.
    #[serde(default)]
    pub summary: String,
    /// Reflected HTML attributes / observed properties.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<AttributeDef>,
    /// Named slots.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slots: Vec<SlotDef>,
    /// Custom events emitted by this component.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<EventDef>,
}

/// A component attribute / property definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDef {
    /// Attribute name in kebab-case.
    pub name: String,
    /// TypeScript type string.
    #[serde(rename = "type", default)]
    pub attr_type: String,
    /// Whether the attribute is required.
    #[serde(default)]
    pub required: bool,
    /// Optional description for JSDoc.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A named slot definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SlotDef {
    /// Slot name (empty string for the default slot).
    pub name: String,
    /// Optional slot description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A custom event emitted by the component.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDef {
    /// Event name in kebab-case.
    pub name: String,
    /// TypeScript type for CustomEvent<T> detail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail_type: Option<String>,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Design Token Community Group (DTCG) spec for CSS custom property / Tailwind token generation (for the design-token section type).
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DesignTokenSpec {
    /// Token collection name used as a CSS prefix (e.g. theme).
    #[serde(default)]
    pub name: String,
    /// Flat list of token entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tokens: Vec<DesignTokenEntry>,
}

/// A single DTCG design token.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignTokenEntry {
    /// Dot-separated path, e.g. color.primary.500.
    pub path: String,
    /// Resolved token value as a string, e.g. #3B82F6.
    pub value: String,
    /// DTCG type: color, dimension, fontWeight, etc.
    pub token_type: String,
    /// Optional description for generated comments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Common metadata carried by every SpecIR variant. Enables generators to make routing decisions (can_generate) without parsing the full spec payload.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecMetadata {
    /// Source file path (relative to project root).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    /// Spec group (e.g. sdd).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_group: Option<String>,
    /// Spec identifier within the group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_id: Option<String>,
    /// Tags for filtering / routing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Specification Intermediate Representation. Universal input type for code generators. Each variant wraps an existing SDD schema type, carrying it along with SpecMetadata for routing.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SpecIR {
    /// API specification from OpenAPI / JSON Schema.
    Api {
        schema: JsonSchema,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// StateMachine+ with states, transitions, guards, and actions.
    StateMachinePlus {
        def: StateMachineDef,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// Flowchart+ with SemanticType annotations for codegen.
    FlowchartPlus {
        def: FlowchartDef,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// Class+ with DDD stereotypes (entity, valueObject, aggregate, etc.).
    ClassPlus {
        def: ClassDiagramDef,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// ERD+ with PK/FK/UK key types.
    ErdPlus {
        def: ERDDef,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// Sequence+ with loops, alt blocks, and activation.
    SequencePlus {
        def: SequenceDef,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// Requirement+ with N:M mapping for test generation.
    RequirementPlus {
        def: RequirementDiagramDef,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// Kubernetes deployment specification (deploy section type).
    Deploy {
        spec: DeploySpec,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// Wireframe specification for React component scaffold (wireframe section type).
    Wireframe {
        spec: WireframeSpec,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// Component Element Model for TypeScript interface generation (component section type).
    Component {
        spec: ComponentSpec,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
    /// Design Token Community Group format for CSS/Tailwind generation (design-token section type).
    DesignToken {
        spec: DesignTokenSpec,
        #[serde(flatten)]
        metadata: SpecMetadata,
    },
}

/// Metadata for a SpecBundle.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BundleMetadata {
    /// Change ID this bundle was produced for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_id: Option<String>,
    /// Human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A bundle of related SpecIR items with a dependency graph. Allows generators to receive the complete context for a change — e.g. an ERD+ alongside the API spec that references its entities.
/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecBundle {
    /// Ordered list of specs.
    pub specs: Vec<SpecIR>,
    /// Dependency edges as (from_index, to_index) pairs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<(usize, usize)>,
    /// Bundle-level metadata.
    #[serde(default)]
    pub metadata: BundleMetadata,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl Default for DeploySpec {
    fn default() -> Self {
        Self {
            name: String::new(),
            image: String::new(),
            port: default_deploy_port(),
            replicas: default_replicas(),
            env: Vec::new(),
            resources: None,
        }
    }
}

fn default_deploy_port() -> u16 {
    8080
}
fn default_replicas() -> u32 {
    1
}

// ---------------------------------------------------------------------------
// SpecIR helper methods
// ---------------------------------------------------------------------------

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl SpecIR {
    /// Return the kind tag as a static str.
    pub fn kind(&self) -> &'static str {
        match self {
            SpecIR::Api { .. } => "api",
            SpecIR::StateMachinePlus { .. } => "state_machine_plus",
            SpecIR::FlowchartPlus { .. } => "flowchart_plus",
            SpecIR::ClassPlus { .. } => "class_plus",
            SpecIR::ErdPlus { .. } => "erd_plus",
            SpecIR::SequencePlus { .. } => "sequence_plus",
            SpecIR::RequirementPlus { .. } => "requirement_plus",
            SpecIR::Deploy { .. } => "deploy",
            SpecIR::Wireframe { .. } => "wireframe",
            SpecIR::Component { .. } => "component",
            SpecIR::DesignToken { .. } => "design_token",
        }
    }

    /// Access the metadata regardless of variant.
    pub fn metadata(&self) -> &SpecMetadata {
        match self {
            SpecIR::Api { metadata, .. }
            | SpecIR::StateMachinePlus { metadata, .. }
            | SpecIR::FlowchartPlus { metadata, .. }
            | SpecIR::ClassPlus { metadata, .. }
            | SpecIR::ErdPlus { metadata, .. }
            | SpecIR::SequencePlus { metadata, .. }
            | SpecIR::RequirementPlus { metadata, .. }
            | SpecIR::Deploy { metadata, .. }
            | SpecIR::Wireframe { metadata, .. }
            | SpecIR::Component { metadata, .. }
            | SpecIR::DesignToken { metadata, .. } => metadata,
        }
    }

    /// Access the metadata mutably.
    pub fn metadata_mut(&mut self) -> &mut SpecMetadata {
        match self {
            SpecIR::Api { metadata, .. }
            | SpecIR::StateMachinePlus { metadata, .. }
            | SpecIR::FlowchartPlus { metadata, .. }
            | SpecIR::ClassPlus { metadata, .. }
            | SpecIR::ErdPlus { metadata, .. }
            | SpecIR::SequencePlus { metadata, .. }
            | SpecIR::RequirementPlus { metadata, .. }
            | SpecIR::Deploy { metadata, .. }
            | SpecIR::Wireframe { metadata, .. }
            | SpecIR::Component { metadata, .. }
            | SpecIR::DesignToken { metadata, .. } => metadata,
        }
    }
}

// ---------------------------------------------------------------------------
// From impls: generate types → SpecIR (R3)
// ---------------------------------------------------------------------------

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<JsonSchema> for SpecIR {
    fn from(schema: JsonSchema) -> Self {
        SpecIR::Api {
            schema,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<StateMachineDef> for SpecIR {
    fn from(def: StateMachineDef) -> Self {
        SpecIR::StateMachinePlus {
            def,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<FlowchartDef> for SpecIR {
    fn from(def: FlowchartDef) -> Self {
        SpecIR::FlowchartPlus {
            def,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<ClassDiagramDef> for SpecIR {
    fn from(def: ClassDiagramDef) -> Self {
        SpecIR::ClassPlus {
            def,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<ERDDef> for SpecIR {
    fn from(def: ERDDef) -> Self {
        SpecIR::ErdPlus {
            def,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<SequenceDef> for SpecIR {
    fn from(def: SequenceDef) -> Self {
        SpecIR::SequencePlus {
            def,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<RequirementDiagramDef> for SpecIR {
    fn from(def: RequirementDiagramDef) -> Self {
        SpecIR::RequirementPlus {
            def,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<DeploySpec> for SpecIR {
    fn from(spec: DeploySpec) -> Self {
        SpecIR::Deploy {
            spec,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<WireframeSpec> for SpecIR {
    fn from(spec: WireframeSpec) -> Self {
        SpecIR::Wireframe {
            spec,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<ComponentSpec> for SpecIR {
    fn from(spec: ComponentSpec) -> Self {
        SpecIR::Component {
            spec,
            metadata: SpecMetadata::default(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl From<DesignTokenSpec> for SpecIR {
    fn from(spec: DesignTokenSpec) -> Self {
        SpecIR::DesignToken {
            spec,
            metadata: SpecMetadata::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// SpecBundle helpers
// ---------------------------------------------------------------------------

/// @spec projects/agentic-workflow/tech-design/core/generate/spec_ir/types.md#source
impl SpecBundle {
    /// Create an empty bundle.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a spec and return its index.
    pub fn push(&mut self, spec: SpecIR) -> usize {
        let idx = self.specs.len();
        self.specs.push(spec);
        idx
    }

    /// Add a dependency edge (from_index depends on to_index).
    pub fn add_dependency(&mut self, from: usize, to: usize) {
        self.dependencies.push((from, to));
    }

    /// Number of specs in the bundle.
    pub fn len(&self) -> usize {
        self.specs.len()
    }

    /// Whether the bundle is empty.
    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    #[test]
    fn test_spec_ir_from_json_schema() {
        let schema = JsonSchema {
            title: Some("User".into()),
            ..Default::default()
        };
        let ir = SpecIR::from(schema);
        assert_eq!(ir.kind(), "api");
        assert!(ir.metadata().source_path.is_none());
    }

    #[test]
    fn test_spec_ir_from_flowchart_def() {
        let def = FlowchartDef {
            id: "test-flow".into(),
            direction: Default::default(),
            nodes: IndexMap::new(),
            edges: vec![],
            subgraphs: vec![],
            description: None,
        };
        let ir = SpecIR::from(def);
        assert_eq!(ir.kind(), "flowchart_plus");
    }

    #[test]
    fn test_spec_bundle_with_dependencies() {
        let schema1 = JsonSchema {
            title: Some("Entity".into()),
            ..Default::default()
        };
        let schema2 = JsonSchema {
            title: Some("API".into()),
            ..Default::default()
        };

        let mut bundle = SpecBundle::new();
        let idx0 = bundle.push(SpecIR::from(schema1));
        let idx1 = bundle.push(SpecIR::from(schema2));
        bundle.add_dependency(idx1, idx0); // API depends on Entity

        assert_eq!(bundle.len(), 2);
        assert_eq!(bundle.dependencies, vec![(1, 0)]);
    }

    #[test]
    fn test_spec_ir_serialize_roundtrip() {
        let schema = JsonSchema {
            title: Some("Test".into()),
            ..Default::default()
        };
        let ir = SpecIR::Api {
            schema,
            metadata: SpecMetadata {
                source_path: Some("specs/test.json".into()),
                spec_group: Some("test-group".into()),
                spec_id: Some("test-spec".into()),
                tags: vec!["api".into()],
            },
        };

        let json = serde_json::to_value(&ir).unwrap();
        assert_eq!(json["kind"], "api");
        assert_eq!(json["source_path"], "specs/test.json");
        assert_eq!(json["tags"], serde_json::json!(["api"]));

        // Roundtrip
        let deserialized: SpecIR = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.kind(), "api");
        assert_eq!(
            deserialized.metadata().source_path.as_deref(),
            Some("specs/test.json")
        );
    }

    #[test]
    fn test_metadata_access() {
        let schema = JsonSchema::default();
        let mut ir = SpecIR::from(schema);
        ir.metadata_mut().tags.push("generated".into());
        assert_eq!(ir.metadata().tags, vec!["generated"]);
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/spec_ir/types.rs
    action: modify
    section: source
    impl_mode: codegen
    description: Source template owns the complete SpecIR types module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Minor prose inaccuracy: the overview describes `SpecIR` as "externally-tagged" but the schema uses `serde_tag: kind`, which is serde's internally-tagged representation. The `## Changes` section correctly says "internally-tagged". No impact on codegen (the `x-rust-enum` block is authoritative), but worth correcting for doc consistency.
- [schema] All 11 `SpecIR` variants carry `{ name: metadata, rust_type: "SpecMetadata", flatten: true }` — R2 satisfied.
- [schema] `DeploySpec.port` and `DeploySpec.replicas` use `x-serde-default: "default_deploy_port"` and `x-serde-default: "default_replicas"` respectively — R3 satisfied.
- [schema] Every Vec field that lacks an explicit `x-rust-type` override is present in its parent's `required:` list; Vec fields with explicit `x-rust-type` (e.g. `SpecMetadata.tags`, `WireframeNode.children`) correctly rely on the type override to avoid Option-wrap — R4 satisfied.
- [changes] `replaces` list enumerates all 16 types; hand-written items (`default_deploy_port`, `default_replicas`, `impl Default for DeploySpec`, all `impl` blocks, tests) are correctly excluded — R5 satisfied.
