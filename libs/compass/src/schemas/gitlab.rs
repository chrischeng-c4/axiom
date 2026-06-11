//! Programmatic GitLab CI JSON-Schema definition.
//!
//! Defines valid top-level keys and the job structure so that `GL002`
//! (unknown keywords) can also be caught via schema validation.

use serde_json::{json, Value};

/// Build the GitLab CI config schema.
///
/// The schema validates:
/// - Top-level keys are either reserved global keywords or job definitions.
/// - Job objects have the correct keyword set.
///
/// Because `additionalProperties` + `patternProperties` is the idiomatic way
/// to allow arbitrary job names while constraining their shape, we use that
/// approach.
pub(super) fn build_gitlab_ci_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "stages": {
                "type": "array",
                "items": { "type": "string" }
            },
            "variables": { "type": "object" },
            "default": { "type": "object" },
            "include": {},
            "image": {},
            "services": { "type": "array" },
            "before_script": {},
            "after_script": {},
            "cache": {},
            "workflow": { "type": "object" },
            "pages": { "type": "object" }
        },
        "additionalProperties": {
            "anyOf": [
                job_schema(),
                // Allow non-object values for edge cases (anchors, etc.)
                { "type": ["string", "number", "boolean", "null", "array"] }
            ]
        }
    })
}

/// Schema for a single CI job object.
fn job_schema() -> Value {
    let known_keywords = vec![
        "script",
        "stage",
        "image",
        "services",
        "variables",
        "rules",
        "only",
        "except",
        "needs",
        "artifacts",
        "cache",
        "before_script",
        "after_script",
        "allow_failure",
        "when",
        "timeout",
        "retry",
        "tags",
        "environment",
        "extends",
        "trigger",
        "parallel",
        "resource_group",
        "release",
        "coverage",
        "dependencies",
        "interruptible",
        "secrets",
        "pages",
        "inherit",
        "dast_configuration",
        "id_tokens",
    ];

    // Build the properties map
    let mut props = serde_json::Map::new();
    for kw in &known_keywords {
        props.insert(kw.to_string(), json!({}));
    }

    json!({
        "type": "object",
        "properties": props,
        "additionalProperties": false
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::SchemaRegistry;

    #[test]
    fn test_gitlab_schema_compiles() {
        let schema = build_gitlab_ci_schema();
        jsonschema::Validator::new(&schema).expect("GitLab CI schema must compile");
    }

    #[test]
    fn test_valid_ci_config() {
        let registry = SchemaRegistry::new("1.30");
        let config = json!({
            "stages": ["build", "test", "deploy"],
            "variables": { "CI": "true" },
            "build_job": {
                "stage": "build",
                "script": ["make build"],
                "tags": ["docker"]
            },
            "test_job": {
                "stage": "test",
                "script": ["make test"],
                "needs": ["build_job"],
                "artifacts": { "paths": ["coverage/"] }
            }
        });
        let diags = registry.validate_gitlab_ci(&config);
        assert!(diags.is_empty(), "valid config should pass: {:?}", diags);
    }

    #[test]
    fn test_unknown_job_keyword() {
        let registry = SchemaRegistry::new("1.30");
        let config = json!({
            "stages": ["build"],
            "build_job": {
                "stage": "build",
                "script": ["make"],
                "invalid_key": true
            }
        });
        let diags = registry.validate_gitlab_ci(&config);
        assert!(
            !diags.is_empty(),
            "should detect unknown keyword 'invalid_key'"
        );
        assert!(diags
            .iter()
            .any(|d| d.message.contains("invalid_key") || d.message.contains("additional")));
    }
}
