// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/validator/completeness.md#source
// CODEGEN-BEGIN
//! Schema completeness validation

use crate::generate::schema::JsonSchema;
use std::collections::HashSet;

/// Severity level for validation issues.
/// @spec projects/agentic-workflow/tech-design/core/generate/validator/completeness.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Error severity (blocks validation).
    Error,
    /// Warning severity (informational).
    Warning,
}

/// A validation issue found in a schema.
/// @spec projects/agentic-workflow/tech-design/core/generate/validator/completeness.md#schema
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    /// JSON pointer path to the issue.
    pub path: String,
    /// Human-readable issue message.
    pub message: String,
    /// Severity level.
    pub severity: Severity,
}

/// Result of schema validation.
/// @spec projects/agentic-workflow/tech-design/core/generate/validator/completeness.md#schema
#[derive(Debug, Default)]
pub struct ValidationResult {
    /// Issues found during validation.
    pub issues: Vec<ValidationIssue>,
}
/// @spec projects/agentic-workflow/tech-design/core/generate/validator/completeness.md#source
impl ValidationIssue {
    pub fn error(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            severity: Severity::Error,
        }
    }

    pub fn warning(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
            severity: Severity::Warning,
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/validator/completeness.md#source
impl ValidationResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    pub fn is_valid(&self) -> bool {
        !self.issues.iter().any(|i| i.severity == Severity::Error)
    }

    pub fn errors(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues.iter().filter(|i| i.severity == Severity::Error)
    }

    pub fn warnings(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
    }

    pub fn error_count(&self) -> usize {
        self.errors().count()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings().count()
    }
}

/// Validate a JSON Schema for completeness
/// @spec projects/agentic-workflow/tech-design/core/generate/validator/completeness.md#source
pub fn validate_schema(schema: &JsonSchema) -> ValidationResult {
    let mut result = ValidationResult::new();
    let definitions = schema.all_definitions();
    let def_names: HashSet<_> = definitions.keys().cloned().collect();

    validate_schema_recursive(schema, "#", &def_names, &mut result);

    result
}

fn validate_schema_recursive(
    schema: &JsonSchema,
    path: &str,
    definitions: &HashSet<String>,
    result: &mut ValidationResult,
) {
    // R1: Type Validation - check that properties have types or refs
    if schema.type_.is_none()
        && schema.ref_.is_none()
        && schema.any_of.is_none()
        && schema.one_of.is_none()
        && schema.all_of.is_none()
    {
        // Only error if this is a property schema (not root)
        if path != "#" && !path.ends_with("/definitions") && !path.ends_with("/$defs") {
            result.add(ValidationIssue::error(
                path,
                "Schema has no type, $ref, or composition (anyOf/oneOf/allOf)",
            ));
        }
    }

    // R2: Reference Validation - check $ref targets exist
    if let Some(ref_path) = &schema.ref_ {
        if let Some(def_name) = extract_definition_name(ref_path) {
            if !definitions.contains(&def_name) {
                result.add(ValidationIssue::error(
                    path,
                    format!("Broken reference: {} does not exist", ref_path),
                ));
            }
        }
    }

    // R3: Completeness Check - warn if descriptions are missing
    if schema.description.is_none() && path != "#" {
        // Only warn for top-level properties (exactly one /properties/ segment)
        let prop_count = path.matches("/properties/").count();
        if prop_count == 1 && !path.ends_with("/properties/") {
            result.add(ValidationIssue::warning(
                path,
                "Missing description for property",
            ));
        }
    }

    // Recursively validate properties
    if let Some(props) = &schema.properties {
        for (name, prop_schema) in props {
            let prop_path = format!("{}/properties/{}", path, name);
            validate_schema_recursive(prop_schema, &prop_path, definitions, result);
        }
    }

    // Validate array items
    if let Some(items) = &schema.items {
        let items_path = format!("{}/items", path);
        validate_schema_recursive(items, &items_path, definitions, result);
    }

    // Validate definitions
    if let Some(defs) = &schema.definitions {
        for (name, def_schema) in defs {
            let def_path = format!("{}/definitions/{}", path, name);
            validate_schema_recursive(def_schema, &def_path, definitions, result);
        }
    }

    if let Some(defs) = &schema.defs {
        for (name, def_schema) in defs {
            let def_path = format!("{}/$defs/{}", path, name);
            validate_schema_recursive(def_schema, &def_path, definitions, result);
        }
    }

    // Validate composition schemas
    for (schemas, keyword) in [
        (&schema.all_of, "allOf"),
        (&schema.any_of, "anyOf"),
        (&schema.one_of, "oneOf"),
    ] {
        if let Some(schemas) = schemas {
            for (i, sub_schema) in schemas.iter().enumerate() {
                let sub_path = format!("{}/{}/{}", path, keyword, i);
                validate_schema_recursive(sub_schema, &sub_path, definitions, result);
            }
        }
    }
}

/// Extract definition name from a $ref path
fn extract_definition_name(ref_path: &str) -> Option<String> {
    // Handle #/definitions/Name or #/$defs/Name
    if let Some(name) = ref_path.strip_prefix("#/definitions/") {
        return Some(name.to_string());
    }
    if let Some(name) = ref_path.strip_prefix("#/$defs/") {
        return Some(name.to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::schema::parse_json;

    #[test]
    fn test_missing_type() {
        let json = r#"{
            "type": "object",
            "properties": {
                "age": {}
            }
        }"#;

        let schema = parse_json(json).unwrap();
        let result = validate_schema(&schema);

        assert!(!result.is_valid());
        assert!(result.errors().any(|i| i.path.contains("age")));
    }

    #[test]
    fn test_broken_reference() {
        let json = r##"{
            "type": "object",
            "properties": {
                "user": { "$ref": "#/definitions/Unknown" }
            }
        }"##;

        let schema = parse_json(json).unwrap();
        let result = validate_schema(&schema);

        assert!(!result.is_valid());
        assert!(result.errors().any(|i| i.message.contains("Unknown")));
    }

    #[test]
    fn test_valid_schema() {
        let json = r##"{
            "type": "object",
            "properties": {
                "name": { "type": "string" },
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
        let result = validate_schema(&schema);

        assert!(result.is_valid());
    }

    #[test]
    fn test_ref_is_valid_type() {
        let json = r##"{
            "type": "object",
            "properties": {
                "user": { "$ref": "#/definitions/User" }
            },
            "definitions": {
                "User": { "type": "object" }
            }
        }"##;

        let schema = parse_json(json).unwrap();
        let result = validate_schema(&schema);

        // $ref is considered a valid type specification
        assert!(result.is_valid());
    }
}

// CODEGEN-END
