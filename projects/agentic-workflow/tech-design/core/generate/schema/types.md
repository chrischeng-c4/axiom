---
id: sdd-generate-schema-types
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# JSON Schema Type Definitions

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/schema/types.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AdditionalProperties` | projects/agentic-workflow/src/generate/schema/types.rs | enum | pub | 98 |  |
| `JsonSchema` | projects/agentic-workflow/src/generate/schema/types.rs | struct | pub | 107 |  |
| `SchemaType` | projects/agentic-workflow/src/generate/schema/types.rs | enum | pub | 27 |  |
| `SchemaVersion` | projects/agentic-workflow/src/generate/schema/types.rs | enum | pub | 14 |  |
| `StringFormat` | projects/agentic-workflow/src/generate/schema/types.rs | enum | pub | 41 |  |
| `TypeConstraint` | projects/agentic-workflow/src/generate/schema/types.rs | enum | pub | 89 |  |
| `all_definitions` | projects/agentic-workflow/src/generate/schema/types.rs | function | pub | 274 | all_definitions(&self) -> HashMap<String, &JsonSchema> |
| `array` | projects/agentic-workflow/src/generate/schema/types.rs | function | pub | 252 | array(items: JsonSchema) -> Self |
| `boolean` | projects/agentic-workflow/src/generate/schema/types.rs | function | pub | 247 | boolean() -> Self |
| `effective_type` | projects/agentic-workflow/src/generate/schema/types.rs | function | pub | 295 | effective_type(&self) -> Option<SchemaType> |
| `integer` | projects/agentic-workflow/src/generate/schema/types.rs | function | pub | 237 | integer() -> Self |
| `is_ref` | projects/agentic-workflow/src/generate/schema/types.rs | function | pub | 290 | is_ref(&self) -> bool |
| `new` | projects/agentic-workflow/src/generate/schema/types.rs | function | pub | 219 | new() -> Self |
| `number` | projects/agentic-workflow/src/generate/schema/types.rs | function | pub | 242 | number() -> Self |
| `object` | projects/agentic-workflow/src/generate/schema/types.rs | function | pub | 261 | object() -> Self |
| `reference` | projects/agentic-workflow/src/generate/schema/types.rs | function | pub | 266 | reference(ref_path: impl Into<String>) -> Self |
| `string` | projects/agentic-workflow/src/generate/schema/types.rs | function | pub | 232 | string() -> Self |
| `with_type` | projects/agentic-workflow/src/generate/schema/types.rs | function | pub | 224 | with_type(type_: SchemaType) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  SchemaVersion:
    type: string
    enum: [Draft7, Draft202012]
    description: JSON Schema version.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]
      variants:
        - { name: Draft7, rename: "http://json-schema.org/draft-07/schema#", doc: "Draft 7 schema URI." }
        - { name: Draft202012, rename: "https://json-schema.org/draft/2020-12/schema", doc: "Draft 2020-12 schema URI." }

  SchemaType:
    type: string
    enum: [String, Number, Integer, Boolean, Array, Object, "Null"]
    description: Primitive JSON Schema types.
    x-rust-enum:
      derive: [Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize]
      serde_rename_all: lowercase

  StringFormat:
    type: string
    enum: [DateTime, Date, Time, Duration, Email, IdnEmail, Hostname, IdnHostname, Ipv4, Ipv6, Uri, UriReference, Iri, IriReference, Uuid, UriTemplate, JsonPointer, RelativeJsonPointer, Regex, Custom]
    description: String format keywords.
    x-rust-enum:
      derive: [Debug, Clone, PartialEq, Eq, Serialize, Deserialize]
      serde_rename_all: kebab-case
      variants:
        - { name: DateTime, doc: "RFC 3339 date-time." }
        - { name: Date, doc: "RFC 3339 date." }
        - { name: Time, doc: "RFC 3339 time." }
        - { name: Duration, doc: "RFC 3339 duration." }
        - { name: Email, doc: "Email address." }
        - { name: IdnEmail, doc: "Internationalised email." }
        - { name: Hostname, doc: "Hostname." }
        - { name: IdnHostname, doc: "Internationalised hostname." }
        - { name: Ipv4, doc: "IPv4 address." }
        - { name: Ipv6, doc: "IPv6 address." }
        - { name: Uri, doc: "URI." }
        - { name: UriReference, doc: "URI reference." }
        - { name: Iri, doc: "IRI." }
        - { name: IriReference, doc: "IRI reference." }
        - { name: Uuid, doc: "UUID." }
        - { name: UriTemplate, doc: "URI template." }
        - { name: JsonPointer, doc: "JSON Pointer." }
        - { name: RelativeJsonPointer, doc: "Relative JSON Pointer." }
        - { name: Regex, doc: "Regex." }
        - { name: Custom, is_other: true, doc: "Catch-all for unknown formats." }

  TypeConstraint:
    type: string
    enum: [Single, Multiple]
    description: Type constraint - single type or array of types.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      serde_untagged: true
      variants:
        - name: Single
          kind: tuple
          fields:
            - { rust_type: SchemaType }
        - name: Multiple
          kind: tuple
          fields:
            - { rust_type: "Vec<SchemaType>" }

  AdditionalProperties:
    type: string
    enum: [Bool, Schema]
    description: Additional properties constraint.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      serde_untagged: true
      variants:
        - name: Bool
          kind: tuple
          fields:
            - { rust_type: bool }
        - name: Schema
          kind: tuple
          fields:
            - { rust_type: "Box<JsonSchema>" }

  JsonSchema:
    type: object
    required: [schema, id, ref_, title, description, type_, properties, required, additional_properties, items, enum_, const_, default, format, min_length, max_length, pattern, minimum, maximum, exclusive_minimum, exclusive_maximum, multiple_of, min_items, max_items, unique_items, min_properties, max_properties, all_of, any_of, one_of, not, definitions, defs]
    description: A JSON Schema definition.
    properties:
      schema:
        type: object
        x-rust-type: "Option<SchemaVersion>"
        x-serde-rename: "$schema"
        x-serde-skip-if: "Option::is_none"
        description: "Schema version URI."
      id:
        type: string
        x-rust-type: "Option<String>"
        x-serde-rename: "$id"
        x-serde-skip-if: "Option::is_none"
        description: "Schema identifier."
      ref_:
        type: string
        x-rust-type: "Option<String>"
        x-serde-rename: "$ref"
        x-serde-skip-if: "Option::is_none"
        description: "Reference to another schema."
      title:
        type: string
        x-rust-type: "Option<String>"
        x-serde-skip-if: "Option::is_none"
        description: "Title."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-skip-if: "Option::is_none"
        description: "Description."
      type_:
        type: object
        x-rust-type: "Option<TypeConstraint>"
        x-serde-rename: "type"
        x-serde-skip-if: "Option::is_none"
        description: "Type constraint."
      properties:
        type: object
        x-rust-type: "Option<HashMap<String, Box<JsonSchema>>>"
        x-serde-skip-if: "Option::is_none"
        description: "Properties for object types."
      required:
        type: array
        x-rust-type: "Option<Vec<String>>"
        x-serde-skip-if: "Option::is_none"
        description: "Required property names."
      additional_properties:
        type: object
        x-rust-type: "Option<AdditionalProperties>"
        x-serde-skip-if: "Option::is_none"
        description: "Additional properties constraint."
      items:
        type: object
        x-rust-type: "Option<Box<JsonSchema>>"
        x-serde-skip-if: "Option::is_none"
        description: "Items schema for arrays."
      enum_:
        type: array
        x-rust-type: "Option<Vec<serde_json::Value>>"
        x-serde-rename: "enum"
        x-serde-skip-if: "Option::is_none"
        description: "Enum values."
      const_:
        type: object
        x-rust-type: "Option<serde_json::Value>"
        x-serde-rename: "const"
        x-serde-skip-if: "Option::is_none"
        description: "Constant value."
      default:
        type: object
        x-rust-type: "Option<serde_json::Value>"
        x-serde-skip-if: "Option::is_none"
        description: "Default value."
      format:
        type: string
        x-rust-type: "Option<StringFormat>"
        x-serde-skip-if: "Option::is_none"
        description: "String format keyword."
      min_length:
        type: integer
        x-rust-type: "Option<u64>"
        x-serde-skip-if: "Option::is_none"
        description: "Minimum string length."
      max_length:
        type: integer
        x-rust-type: "Option<u64>"
        x-serde-skip-if: "Option::is_none"
        description: "Maximum string length."
      pattern:
        type: string
        x-rust-type: "Option<String>"
        x-serde-skip-if: "Option::is_none"
        description: "Regex pattern."
      minimum:
        type: number
        x-rust-type: "Option<f64>"
        x-serde-skip-if: "Option::is_none"
        description: "Minimum numeric value."
      maximum:
        type: number
        x-rust-type: "Option<f64>"
        x-serde-skip-if: "Option::is_none"
        description: "Maximum numeric value."
      exclusive_minimum:
        type: number
        x-rust-type: "Option<f64>"
        x-serde-skip-if: "Option::is_none"
        description: "Exclusive minimum."
      exclusive_maximum:
        type: number
        x-rust-type: "Option<f64>"
        x-serde-skip-if: "Option::is_none"
        description: "Exclusive maximum."
      multiple_of:
        type: number
        x-rust-type: "Option<f64>"
        x-serde-skip-if: "Option::is_none"
        description: "Multiple-of constraint."
      min_items:
        type: integer
        x-rust-type: "Option<u64>"
        x-serde-skip-if: "Option::is_none"
        description: "Minimum array items."
      max_items:
        type: integer
        x-rust-type: "Option<u64>"
        x-serde-skip-if: "Option::is_none"
        description: "Maximum array items."
      unique_items:
        type: boolean
        x-rust-type: "Option<bool>"
        x-serde-skip-if: "Option::is_none"
        description: "Whether array items must be unique."
      min_properties:
        type: integer
        x-rust-type: "Option<u64>"
        x-serde-skip-if: "Option::is_none"
        description: "Minimum object properties."
      max_properties:
        type: integer
        x-rust-type: "Option<u64>"
        x-serde-skip-if: "Option::is_none"
        description: "Maximum object properties."
      all_of:
        type: array
        x-rust-type: "Option<Vec<JsonSchema>>"
        x-serde-skip-if: "Option::is_none"
        description: "All-of composition."
      any_of:
        type: array
        x-rust-type: "Option<Vec<JsonSchema>>"
        x-serde-skip-if: "Option::is_none"
        description: "Any-of composition."
      one_of:
        type: array
        x-rust-type: "Option<Vec<JsonSchema>>"
        x-serde-skip-if: "Option::is_none"
        description: "One-of composition."
      not:
        type: object
        x-rust-type: "Option<Box<JsonSchema>>"
        x-serde-skip-if: "Option::is_none"
        description: "Not composition."
      definitions:
        type: object
        x-rust-type: "Option<HashMap<String, JsonSchema>>"
        x-serde-skip-if: "Option::is_none"
        description: "Definitions Draft 7."
      defs:
        type: object
        x-rust-type: "Option<HashMap<String, JsonSchema>>"
        x-serde-rename: "$defs"
        x-serde-skip-if: "Option::is_none"
        description: "Definitions Draft 2020-12."
    x-rust-struct:
      derive: [Debug, Clone, Default, Serialize, Deserialize]
      serde_rename_all: camelCase
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/schema/types.rs -->
```rust
//! JSON Schema type definitions
//!
//! Supports Draft 7 and Draft 2020-12

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// JSON Schema version.
/// @spec projects/agentic-workflow/tech-design/core/generate/schema/types.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaVersion {
    /// Draft 7 schema URI.
    #[serde(rename = "http://json-schema.org/draft-07/schema#")]
    Draft7,
    /// Draft 2020-12 schema URI.
    #[serde(rename = "https://json-schema.org/draft/2020-12/schema")]
    Draft202012,
}

/// Primitive JSON Schema types.
/// @spec projects/agentic-workflow/tech-design/core/generate/schema/types.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
    Null,
}

/// String format keywords.
/// @spec projects/agentic-workflow/tech-design/core/generate/schema/types.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StringFormat {
    /// RFC 3339 date-time.
    DateTime,
    /// RFC 3339 date.
    Date,
    /// RFC 3339 time.
    Time,
    /// RFC 3339 duration.
    Duration,
    /// Email address.
    Email,
    /// Internationalised email.
    IdnEmail,
    /// Hostname.
    Hostname,
    /// Internationalised hostname.
    IdnHostname,
    /// IPv4 address.
    Ipv4,
    /// IPv6 address.
    Ipv6,
    /// URI.
    Uri,
    /// URI reference.
    UriReference,
    /// IRI.
    Iri,
    /// IRI reference.
    IriReference,
    /// UUID.
    Uuid,
    /// URI template.
    UriTemplate,
    /// JSON Pointer.
    JsonPointer,
    /// Relative JSON Pointer.
    RelativeJsonPointer,
    /// Regex.
    Regex,
    /// Catch-all for unknown formats.
    #[serde(other)]
    Custom,
}

/// Type constraint - single type or array of types.
/// @spec projects/agentic-workflow/tech-design/core/generate/schema/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TypeConstraint {
    Single(SchemaType),
    Multiple(Vec<SchemaType>),
}

/// Additional properties constraint.
/// @spec projects/agentic-workflow/tech-design/core/generate/schema/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AdditionalProperties {
    Bool(bool),
    Schema(Box<JsonSchema>),
}

/// A JSON Schema definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/schema/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonSchema {
    /// Schema version URI.
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<SchemaVersion>,
    /// Schema identifier.
    #[serde(rename = "$id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Reference to another schema.
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,
    /// Title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Type constraint.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<TypeConstraint>,
    /// Properties for object types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, Box<JsonSchema>>>,
    /// Required property names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    /// Additional properties constraint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<AdditionalProperties>,
    /// Items schema for arrays.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<JsonSchema>>,
    /// Enum values.
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_: Option<Vec<serde_json::Value>>,
    /// Constant value.
    #[serde(rename = "const", skip_serializing_if = "Option::is_none")]
    pub const_: Option<serde_json::Value>,
    /// Default value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// String format keyword.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<StringFormat>,
    /// Minimum string length.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u64>,
    /// Maximum string length.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u64>,
    /// Regex pattern.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    /// Minimum numeric value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    /// Maximum numeric value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    /// Exclusive minimum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<f64>,
    /// Exclusive maximum.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<f64>,
    /// Multiple-of constraint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<f64>,
    /// Minimum array items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<u64>,
    /// Maximum array items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u64>,
    /// Whether array items must be unique.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,
    /// Minimum object properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<u64>,
    /// Maximum object properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<u64>,
    /// All-of composition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_of: Option<Vec<JsonSchema>>,
    /// Any-of composition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<JsonSchema>>,
    /// One-of composition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<JsonSchema>>,
    /// Not composition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not: Option<Box<JsonSchema>>,
    /// Definitions Draft 7.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definitions: Option<HashMap<String, JsonSchema>>,
    /// Definitions Draft 2020-12.
    #[serde(rename = "$defs", skip_serializing_if = "Option::is_none")]
    pub defs: Option<HashMap<String, JsonSchema>>,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/schema/types.md#source
impl Default for SchemaVersion {
    fn default() -> Self {
        Self::Draft202012
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/schema/types.md#source
impl JsonSchema {
    /// Create a new empty schema
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a schema with a specific type
    pub fn with_type(type_: SchemaType) -> Self {
        Self {
            type_: Some(TypeConstraint::Single(type_)),
            ..Default::default()
        }
    }

    /// Create a string schema
    pub fn string() -> Self {
        Self::with_type(SchemaType::String)
    }

    /// Create an integer schema
    pub fn integer() -> Self {
        Self::with_type(SchemaType::Integer)
    }

    /// Create a number schema
    pub fn number() -> Self {
        Self::with_type(SchemaType::Number)
    }

    /// Create a boolean schema
    pub fn boolean() -> Self {
        Self::with_type(SchemaType::Boolean)
    }

    /// Create an array schema
    pub fn array(items: JsonSchema) -> Self {
        Self {
            type_: Some(TypeConstraint::Single(SchemaType::Array)),
            items: Some(Box::new(items)),
            ..Default::default()
        }
    }

    /// Create an object schema
    pub fn object() -> Self {
        Self::with_type(SchemaType::Object)
    }

    /// Create a reference schema
    pub fn reference(ref_path: impl Into<String>) -> Self {
        Self {
            ref_: Some(ref_path.into()),
            ..Default::default()
        }
    }

    /// Get all definitions (merging definitions and $defs)
    pub fn all_definitions(&self) -> HashMap<String, &JsonSchema> {
        let mut result = HashMap::new();
        if let Some(defs) = &self.definitions {
            for (k, v) in defs {
                result.insert(k.clone(), v);
            }
        }
        if let Some(defs) = &self.defs {
            for (k, v) in defs {
                result.insert(k.clone(), v);
            }
        }
        result
    }

    /// Check if this schema is a reference
    pub fn is_ref(&self) -> bool {
        self.ref_.is_some()
    }

    /// Get the effective type (resolving single-element arrays)
    pub fn effective_type(&self) -> Option<SchemaType> {
        match &self.type_ {
            Some(TypeConstraint::Single(t)) => Some(*t),
            Some(TypeConstraint::Multiple(types)) if types.len() == 1 => Some(types[0]),
            _ => None,
        }
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/schema/types.rs
    action: modify
    section: source
    impl_mode: codegen
    description: Source template owns the complete JSON Schema types module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] 6 types: 3 enums incl untagged + variant-rename + serde-other; 1 huge struct with 33 optional fields and per-field renames; 2 untagged tuple enums.
- [schema] All gap fixes exercised: variant rename, serde_other, untagged tuple, deny_unknown_fields not needed here.
- [changes] All six in `replaces`; Default impl + JsonSchema methods preserved.
