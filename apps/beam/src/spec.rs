//! Offline, machine-readable self-description for agent integration.
//!
//! The `beam spec` CLI subset emits everything an LLM agent needs to wire
//! beam into a retrieval pipeline — OpenAPI JSON, OpenAPI YAML, and JSON-Schema
//! views of the request/response payloads.

use serde_json::Value;

/// The full OpenAPI document as pretty JSON.
pub fn openapi_json() -> String {
    serde_json::to_string_pretty(&openapi_value()).expect("OpenApi value serializes to JSON")
}

/// The full OpenAPI document as YAML for LLM/agent reading.
pub fn openapi_yaml() -> String {
    serde_yaml::to_string(&openapi_value()).expect("OpenApi value serializes to YAML")
}

/// Force version to OpenAPI 3.2.0 for agent-level validation compatibility.
fn openapi_value() -> Value {
    let mut v =
        serde_json::to_value(crate::service::openapi()).expect("OpenApi serializes to JSON");
    if let Value::Object(map) = &mut v {
        map.insert("openapi".to_string(), Value::String("3.2.0".to_string()));
    }
    v
}

/// Just the component schemas (the request/response data types) as pretty JSON.
pub fn json_schema_json() -> String {
    let api = crate::service::openapi();
    serde_json::to_string_pretty(&serde_json::json!({
        "components": api.components
    }))
    .expect("components serialize to JSON")
}
