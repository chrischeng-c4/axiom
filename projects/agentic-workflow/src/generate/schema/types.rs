// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/schema/types.md#source
// CODEGEN-BEGIN
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

// CODEGEN-END
