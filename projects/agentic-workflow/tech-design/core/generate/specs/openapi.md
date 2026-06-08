---
id: sdd-generate-specs-openapi
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# OpenAPI Spec Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/specs/openapi.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ApiInfo` | projects/agentic-workflow/src/generate/specs/openapi.rs | struct | pub | 39 |  |
| `ApiResponse` | projects/agentic-workflow/src/generate/specs/openapi.rs | struct | pub | 101 |  |
| `HttpMethod` | projects/agentic-workflow/src/generate/specs/openapi.rs | enum | pub | 17 |  |
| `OpenApiInput` | projects/agentic-workflow/src/generate/specs/openapi.rs | struct | pub | 147 |  |
| `Parameter` | projects/agentic-workflow/src/generate/specs/openapi.rs | struct | pub | 63 |  |
| `ParameterIn` | projects/agentic-workflow/src/generate/specs/openapi.rs | enum | pub | 29 |  |
| `PathOperation` | projects/agentic-workflow/src/generate/specs/openapi.rs | struct | pub | 117 |  |
| `RequestBody` | projects/agentic-workflow/src/generate/specs/openapi.rs | struct | pub | 83 |  |
| `Server` | projects/agentic-workflow/src/generate/specs/openapi.rs | struct | pub | 52 |  |
| `generate_openapi` | projects/agentic-workflow/src/generate/specs/openapi.rs | function | pub | 161 | generate_openapi(input: &OpenApiInput) -> Result<String> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  HttpMethod:
    type: string
    enum: [Get, Post, Put, Patch, Delete]
    description: HTTP method.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      serde_rename_all: lowercase

  ParameterIn:
    type: string
    enum: [Path, Query, Header, Cookie]
    description: Parameter location.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, Deserialize]
      serde_rename_all: lowercase

  ApiInfo:
    type: object
    required: [title, version, description]
    description: API info.
    properties:
      title:
        type: string
        description: "API title."
      version:
        type: string
        description: "API version."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Optional description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  Server:
    type: object
    required: [url, description]
    description: Server definition.
    properties:
      url:
        type: string
        description: "Server URL."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Optional description."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  Parameter:
    type: object
    required: [name, location, required, description, schema]
    description: API parameter.
    properties:
      name:
        type: string
        description: "Parameter name."
      location:
        type: string
        x-rust-type: "ParameterIn"
        x-serde-rename: "in"
        description: "Parameter location."
      required:
        type: boolean
        x-rust-type: "Option<bool>"
        x-serde-default: true
        description: "Whether parameter is required."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Parameter description."
      schema:
        type: object
        x-rust-type: "Option<Value>"
        x-serde-default: true
        description: "JSON Schema for the parameter."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  RequestBody:
    type: object
    required: [description, required, content_type, schema]
    description: Request body.
    properties:
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Body description."
      required:
        type: boolean
        x-rust-type: "Option<bool>"
        x-serde-default: true
        description: "Whether required."
      content_type:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Content type."
      schema:
        type: object
        x-rust-type: "Option<Value>"
        x-serde-default: true
        description: "JSON Schema."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  ApiResponse:
    type: object
    required: [status, description, content_type, schema]
    description: API response.
    properties:
      status:
        type: string
        description: "HTTP status code."
      description:
        type: string
        description: "Response description."
      content_type:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Content type."
      schema:
        type: object
        x-rust-type: "Option<Value>"
        x-serde-default: true
        description: "JSON Schema."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  PathOperation:
    type: object
    required: [path, method, summary, description, operation_id, tags, parameters, request_body, responses]
    description: Path operation.
    properties:
      path:
        type: string
        description: "URL path template."
      method:
        type: string
        x-rust-type: "HttpMethod"
        description: "HTTP method."
      summary:
        type: string
        description: "Operation summary."
      description:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Operation description."
      operation_id:
        type: string
        x-rust-type: "Option<String>"
        x-serde-default: true
        description: "Operation identifier."
      tags:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        x-serde-default: true
        description: "Operation tags."
      parameters:
        type: array
        items: { $ref: "#/definitions/Parameter" }
        x-rust-type: "Vec<Parameter>"
        x-serde-default: true
        description: "Operation parameters."
      request_body:
        type: object
        x-rust-type: "Option<RequestBody>"
        x-serde-default: true
        description: "Request body."
      responses:
        type: array
        items: { $ref: "#/definitions/ApiResponse" }
        x-rust-type: "Vec<ApiResponse>"
        x-serde-default: true
        description: "Operation responses."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  OpenApiInput:
    type: object
    required: [info, paths, servers, schemas]
    description: Input for OpenAPI generation.
    properties:
      info:
        type: object
        x-rust-type: "ApiInfo"
        description: "API info."
      paths:
        type: array
        items: { $ref: "#/definitions/PathOperation" }
        x-rust-type: "Vec<PathOperation>"
        description: "Path operations."
      servers:
        type: array
        items: { $ref: "#/definitions/Server" }
        x-rust-type: "Vec<Server>"
        x-serde-default: true
        description: "Server definitions."
      schemas:
        type: object
        x-rust-type: "HashMap<String, Value>"
        x-serde-default: true
        description: "Component schemas keyed by name."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/specs/openapi.rs -->
```rust
//! OpenAPI 3.1 Specification Generation
//!
//! Generates OpenAPI 3.1 specifications for HTTP REST APIs.

use crate::generate::{GenerateError, Result};
use serde_json::{json, Value};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// HTTP method.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

/// Parameter location.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterIn {
    Path,
    Query,
    Header,
    Cookie,
}

/// API info.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiInfo {
    /// API title.
    pub title: String,
    /// API version.
    pub version: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Server definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    /// Server URL.
    pub url: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
}

/// API parameter.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name.
    pub name: String,
    /// Parameter location.
    #[serde(rename = "in")]
    pub location: ParameterIn,
    /// Whether parameter is required.
    #[serde(default)]
    pub required: Option<bool>,
    /// Parameter description.
    #[serde(default)]
    pub description: Option<String>,
    /// JSON Schema for the parameter.
    #[serde(default)]
    pub schema: Option<Value>,
}

/// Request body.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    /// Body description.
    #[serde(default)]
    pub description: Option<String>,
    /// Whether required.
    #[serde(default)]
    pub required: Option<bool>,
    /// Content type.
    #[serde(default)]
    pub content_type: Option<String>,
    /// JSON Schema.
    #[serde(default)]
    pub schema: Option<Value>,
}

/// API response.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    /// HTTP status code.
    pub status: String,
    /// Response description.
    pub description: String,
    /// Content type.
    #[serde(default)]
    pub content_type: Option<String>,
    /// JSON Schema.
    #[serde(default)]
    pub schema: Option<Value>,
}

/// Path operation.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathOperation {
    /// URL path template.
    pub path: String,
    /// HTTP method.
    pub method: HttpMethod,
    /// Operation summary.
    pub summary: String,
    /// Operation description.
    #[serde(default)]
    pub description: Option<String>,
    /// Operation identifier.
    #[serde(default)]
    pub operation_id: Option<String>,
    /// Operation tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Operation parameters.
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    /// Request body.
    #[serde(default)]
    pub request_body: Option<RequestBody>,
    /// Operation responses.
    #[serde(default)]
    pub responses: Vec<ApiResponse>,
}

/// Input for OpenAPI generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiInput {
    /// API info.
    pub info: ApiInfo,
    /// Path operations.
    pub paths: Vec<PathOperation>,
    /// Server definitions.
    #[serde(default)]
    pub servers: Vec<Server>,
    /// Component schemas keyed by name.
    #[serde(default)]
    pub schemas: HashMap<String, Value>,
}
/// Generate an OpenAPI 3.1 specification
/// @spec projects/agentic-workflow/tech-design/core/generate/specs/openapi.md#source
pub fn generate_openapi(input: &OpenApiInput) -> Result<String> {
    if input.paths.is_empty() {
        return Err(GenerateError::InvalidValue(
            "At least one path required".to_string(),
        ));
    }

    // Build info object
    let mut info_obj = json!({
        "title": input.info.title,
        "version": input.info.version
    });
    if let Some(ref desc) = input.info.description {
        info_obj["description"] = json!(desc);
    }

    // Build paths object
    let mut paths_obj = serde_json::Map::new();
    for op in &input.paths {
        let method = match op.method {
            HttpMethod::Get => "get",
            HttpMethod::Post => "post",
            HttpMethod::Put => "put",
            HttpMethod::Patch => "patch",
            HttpMethod::Delete => "delete",
        };

        let mut operation = json!({
            "summary": op.summary
        });

        if let Some(ref desc) = op.description {
            operation["description"] = json!(desc);
        }
        if let Some(ref op_id) = op.operation_id {
            operation["operationId"] = json!(op_id);
        }
        if !op.tags.is_empty() {
            operation["tags"] = json!(op.tags);
        }

        // Parameters
        if !op.parameters.is_empty() {
            let params: Vec<Value> = op
                .parameters
                .iter()
                .map(|p| {
                    let mut param_obj = json!({
                        "name": p.name,
                        "in": match p.location {
                            ParameterIn::Path => "path",
                            ParameterIn::Query => "query",
                            ParameterIn::Header => "header",
                            ParameterIn::Cookie => "cookie",
                        }
                    });
                    if let Some(required) = p.required {
                        param_obj["required"] = json!(required);
                    }
                    if let Some(ref desc) = p.description {
                        param_obj["description"] = json!(desc);
                    }
                    if let Some(ref schema) = p.schema {
                        param_obj["schema"] = schema.clone();
                    }
                    param_obj
                })
                .collect();
            operation["parameters"] = json!(params);
        }

        // Request body
        if let Some(ref req_body) = op.request_body {
            let content_type = req_body
                .content_type
                .as_deref()
                .unwrap_or("application/json");
            let mut req_body_obj = json!({});
            if let Some(ref desc) = req_body.description {
                req_body_obj["description"] = json!(desc);
            }
            if let Some(required) = req_body.required {
                req_body_obj["required"] = json!(required);
            }
            if let Some(ref schema) = req_body.schema {
                req_body_obj["content"] = json!({
                    content_type: { "schema": schema }
                });
            }
            operation["requestBody"] = req_body_obj;
        }

        // Responses
        if !op.responses.is_empty() {
            let mut responses_obj = serde_json::Map::new();
            for resp in &op.responses {
                let content_type = resp.content_type.as_deref().unwrap_or("application/json");
                let mut resp_obj = json!({ "description": resp.description });
                if let Some(ref schema) = resp.schema {
                    resp_obj["content"] = json!({
                        content_type: { "schema": schema }
                    });
                }
                responses_obj.insert(resp.status.clone(), resp_obj);
            }
            operation["responses"] = Value::Object(responses_obj);
        } else {
            operation["responses"] = json!({ "200": { "description": "Success" } });
        }

        // Add to paths
        let path_entry = paths_obj
            .entry(op.path.clone())
            .or_insert_with(|| json!({}));
        if let Value::Object(ref mut path_map) = path_entry {
            path_map.insert(method.to_string(), operation);
        }
    }

    // Build final OpenAPI document
    let mut openapi = json!({
        "openapi": "3.1.0",
        "info": info_obj,
        "paths": Value::Object(paths_obj)
    });

    if !input.servers.is_empty() {
        openapi["servers"] = json!(input.servers);
    }

    if !input.schemas.is_empty() {
        openapi["components"] = json!({ "schemas": input.schemas });
    }

    serde_json::to_string_pretty(&openapi).map_err(|e| GenerateError::Serialization(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_openapi() {
        let input = OpenApiInput {
            info: ApiInfo {
                title: "User API".to_string(),
                version: "1.0.0".to_string(),
                description: Some("API for user management".to_string()),
            },
            paths: vec![PathOperation {
                path: "/users".to_string(),
                method: HttpMethod::Get,
                summary: "List users".to_string(),
                description: None,
                operation_id: Some("listUsers".to_string()),
                tags: vec!["users".to_string()],
                parameters: vec![],
                request_body: None,
                responses: vec![ApiResponse {
                    status: "200".to_string(),
                    description: "List of users".to_string(),
                    content_type: None,
                    schema: Some(json!({"type": "array"})),
                }],
            }],
            servers: vec![],
            schemas: HashMap::new(),
        };

        let result = generate_openapi(&input).unwrap();
        assert!(result.contains("\"openapi\": \"3.1.0\""));
        assert!(result.contains("User API"));
        assert!(result.contains("/users"));
        assert!(result.contains("listUsers"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/specs/openapi.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete OpenAPI specification generation module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Nine serde shapes; mix of structs/enums; HashMap + Vec + Options.
- [schema] All well-formed; Option<bool>/Option<Value> via x-rust-type; rename for `in`.
- [changes] All nine in `replaces`; hand-written boundary preserves the generator function.
