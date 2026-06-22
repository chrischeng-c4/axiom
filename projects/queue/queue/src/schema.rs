//! JSON Schema and AsyncAPI generation utilities.
//!
//! This module provides utilities for generating JSON schemas from task types
//! and exporting AsyncAPI specifications.
//!
//! # Example
//! ```rust,ignore
//! use cclab_queue::schema::{generate_task_message_schema, generate_asyncapi};
//!
//! // Generate JSON schema for TaskMessage
//! let schema = generate_task_message_schema();
//! println!("{}", serde_json::to_string_pretty(&schema)?);
//!
//! // Generate full AsyncAPI spec
//! let asyncapi = generate_asyncapi()?;
//! std::fs::write("asyncapi.yaml", asyncapi)?;
//! ```

use schemars::schema_for;
use serde_json::Value;

use crate::message::TaskMessage;
use crate::retry::RetryPolicy;
use crate::state::{TaskResult, TaskState};

/// Generate JSON schema for TaskMessage
pub fn generate_task_message_schema() -> Value {
    let schema = schema_for!(TaskMessage);
    serde_json::to_value(schema).unwrap_or_default()
}

/// Generate JSON schema for TaskState
pub fn generate_task_state_schema() -> Value {
    let schema = schema_for!(TaskState);
    serde_json::to_value(schema).unwrap_or_default()
}

/// Generate JSON schema for TaskResult
pub fn generate_task_result_schema() -> Value {
    let schema = schema_for!(TaskResult);
    serde_json::to_value(schema).unwrap_or_default()
}

/// Generate JSON schema for RetryPolicy
pub fn generate_retry_policy_schema() -> Value {
    let schema = schema_for!(RetryPolicy);
    serde_json::to_value(schema).unwrap_or_default()
}

/// Generate all schemas as a map
pub fn generate_all_schemas() -> std::collections::HashMap<&'static str, Value> {
    let mut schemas = std::collections::HashMap::new();
    schemas.insert("TaskMessage", generate_task_message_schema());
    schemas.insert("TaskState", generate_task_state_schema());
    schemas.insert("TaskResult", generate_task_result_schema());
    schemas.insert("RetryPolicy", generate_retry_policy_schema());
    schemas
}

/// AsyncAPI specification template
const ASYNCAPI_TEMPLATE: &str = include_str!("../../../docs/tasks/asyncapi.yaml");

/// Generate AsyncAPI specification with current schemas
pub fn generate_asyncapi() -> Result<String, Box<dyn std::error::Error>> {
    // For now, return the base template
    // Future: inject generated schemas into the template
    Ok(ASYNCAPI_TEMPLATE.to_string())
}

/// Export AsyncAPI spec to a file
pub fn export_asyncapi(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let spec = generate_asyncapi()?;
    std::fs::write(path, spec)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_task_message_schema() {
        let schema = generate_task_message_schema();
        assert!(schema.is_object());

        let obj = schema.as_object().unwrap();
        assert!(obj.contains_key("$schema") || obj.contains_key("type"));
    }

    #[test]
    fn test_generate_task_state_schema() {
        let schema = generate_task_state_schema();
        assert!(schema.is_object());
    }

    #[test]
    fn test_generate_task_result_schema() {
        let schema = generate_task_result_schema();
        assert!(schema.is_object());
    }

    #[test]
    fn test_generate_retry_policy_schema() {
        let schema = generate_retry_policy_schema();
        assert!(schema.is_object());
    }

    #[test]
    fn test_generate_all_schemas() {
        let schemas = generate_all_schemas();
        assert_eq!(schemas.len(), 4);
        assert!(schemas.contains_key("TaskMessage"));
        assert!(schemas.contains_key("TaskState"));
        assert!(schemas.contains_key("TaskResult"));
        assert!(schemas.contains_key("RetryPolicy"));
    }

    #[test]
    fn test_asyncapi_template_loads() {
        assert!(!ASYNCAPI_TEMPLATE.is_empty());
        assert!(ASYNCAPI_TEMPLATE.contains("asyncapi: 3.0.0"));
    }
}
