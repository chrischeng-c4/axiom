// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/schema/parser.md#source
// CODEGEN-BEGIN
//! JSON Schema parsing utilities

use super::types::JsonSchema;
use crate::generate::GenerateError;

/// Parse a JSON Schema from a JSON string
/// @spec projects/agentic-workflow/tech-design/core/generate/schema/parser.md#source
pub fn parse_json(json: &str) -> Result<JsonSchema, GenerateError> {
    serde_json::from_str(json).map_err(|e| GenerateError::Serialization(e.to_string()))
}

/// Parse a JSON Schema from a YAML string
/// @spec projects/agentic-workflow/tech-design/core/generate/schema/parser.md#source
pub fn parse_yaml(yaml: &str) -> Result<JsonSchema, GenerateError> {
    serde_yaml::from_str(yaml).map_err(|e| GenerateError::Serialization(e.to_string()))
}

/// Serialize a JSON Schema to JSON string
/// @spec projects/agentic-workflow/tech-design/core/generate/schema/parser.md#source
pub fn to_json(schema: &JsonSchema) -> Result<String, GenerateError> {
    serde_json::to_string_pretty(schema).map_err(|e| GenerateError::Serialization(e.to_string()))
}

/// Serialize a JSON Schema to YAML string
/// @spec projects/agentic-workflow/tech-design/core/generate/schema/parser.md#source
pub fn to_yaml(schema: &JsonSchema) -> Result<String, GenerateError> {
    serde_yaml::to_string(schema).map_err(|e| GenerateError::Serialization(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::schema::{SchemaType, SchemaVersion};

    #[test]
    fn test_parse_draft7_schema() {
        let json = r#"{
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "User",
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" }
            },
            "required": ["name"]
        }"#;

        let schema = parse_json(json).unwrap();
        assert_eq!(schema.schema, Some(SchemaVersion::Draft7));
        assert_eq!(schema.title, Some("User".to_string()));
        assert!(schema.properties.is_some());
    }

    #[test]
    fn test_parse_draft202012_schema() {
        let json = r#"{
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "Product",
            "type": "object",
            "$defs": {
                "Price": {
                    "type": "number",
                    "minimum": 0
                }
            }
        }"#;

        let schema = parse_json(json).unwrap();
        assert_eq!(schema.schema, Some(SchemaVersion::Draft202012));
        assert!(schema.defs.is_some());
    }

    #[test]
    fn test_parse_with_ref() {
        let json = r##"{
            "type": "object",
            "properties": {
                "user": { "$ref": "#/definitions/User" }
            },
            "definitions": {
                "User": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "integer" }
                    }
                }
            }
        }"##;

        let schema = parse_json(json).unwrap();
        let props = schema.properties.as_ref().unwrap();
        let user_prop = props.get("user").unwrap();
        assert!(user_prop.is_ref());
        assert_eq!(user_prop.ref_.as_deref(), Some("#/definitions/User"));
    }

    #[test]
    fn test_roundtrip() {
        let schema = JsonSchema {
            title: Some("Test".to_string()),
            type_: Some(super::super::types::TypeConstraint::Single(
                SchemaType::Object,
            )),
            ..Default::default()
        };

        let json = to_json(&schema).unwrap();
        let parsed = parse_json(&json).unwrap();
        assert_eq!(parsed.title, Some("Test".to_string()));
    }
}

// CODEGEN-END
