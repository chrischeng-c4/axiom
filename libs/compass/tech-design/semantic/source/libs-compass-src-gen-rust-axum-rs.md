---
id: libs-compass-src-gen-rust-axum-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/gen/rust/axum.rs`.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [overview, source, changes]
---

# Standardized libs/compass/src/gen/rust/axum.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/gen/rust/axum.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AxumGenerator` | libs/compass/src/gen/rust/axum.rs | struct | pub | 10 | pub struct AxumGenerator; |
| `new` | libs/compass/src/gen/rust/axum.rs | function | pub | 13 | pub fn new() -> Self { |
| `generate_router` | libs/compass/src/gen/rust/axum.rs | function | pub | 18 | pub fn generate_router(&self, spec: &RestApiSpec, ctx: &GenContext) -> String { |
| `create_router` | libs/compass/src/gen/rust/axum.rs | function | pub | 25 | lines.push("pub fn create_router() -> Router {".to_string()); |
| `list_pets` | libs/compass/src/gen/rust/axum.rs | function | pub | 398 | assert!(code.contains("pub async fn list_pets")); |
| `get_pet` | libs/compass/src/gen/rust/axum.rs | function | pub | 399 | assert!(code.contains("pub async fn get_pet")); |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Axum route handler generator
//!
//! Generates async route handlers from OpenAPI specs.

use crate::gen::rust::type_to_rust;
use crate::gen::traits::{CodeGenerator, GenContext, GenResult, GeneratedCode, Language};
use crate::spec::ir::{DataModelSpec, EndpointDef, HttpMethod, RestApiSpec};

/// Axum (route handlers) code generator
pub struct AxumGenerator;

impl AxumGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate router from RestApiSpec
    pub fn generate_router(&self, spec: &RestApiSpec, ctx: &GenContext) -> String {
        let mut lines = Vec::new();

        // Router function
        if ctx.generate_docs {
            lines.push("/// Create the API router".to_string());
        }
        lines.push("pub fn create_router() -> Router {".to_string());
        lines.push(format!("{}Router::new()", ctx.indent));

        // Add routes for each endpoint
        for endpoint in &spec.endpoints {
            let method_fn = match endpoint.method {
                HttpMethod::Get => "get",
                HttpMethod::Post => "post",
                HttpMethod::Put => "put",
                HttpMethod::Patch => "patch",
                HttpMethod::Delete => "delete",
                HttpMethod::Head => "head",
                HttpMethod::Options => "options",
            };

            let handler_name = endpoint
                .operation_id
                .clone()
                .map(|s| to_snake_case(&s))
                .unwrap_or_else(|| self.path_to_handler_name(&endpoint.path, endpoint.method));

            // Convert OpenAPI path params {id} to Axum path params :id
            let axum_path = endpoint.path.replace("{", ":").replace("}", "");

            lines.push(format!(
                "{}{}.route(\"{}\", {}({}))",
                ctx.indent, ctx.indent, axum_path, method_fn, handler_name
            ));
        }

        lines.push("}".to_string());

        lines.push(String::new());

        // Generate handler functions
        for endpoint in &spec.endpoints {
            lines.push(self.generate_handler(endpoint, ctx));
            lines.push(String::new());
        }

        lines.join("\n")
    }

    /// Generate a single handler
    fn generate_handler(&self, endpoint: &EndpointDef, ctx: &GenContext) -> String {
        let mut lines = Vec::new();

        let handler_name = endpoint
            .operation_id
            .clone()
            .map(|s| to_snake_case(&s))
            .unwrap_or_else(|| self.path_to_handler_name(&endpoint.path, endpoint.method));

        // Doc comment
        if ctx.generate_docs {
            if let Some(summary) = &endpoint.summary {
                lines.push(format!("/// {}", summary));
            }
        }

        // Build parameters
        let mut params = Vec::new();
        let mut extractors = Vec::new();

        // Path parameters
        if !endpoint.path_params.is_empty() {
            let path_types: Vec<String> = endpoint
                .path_params
                .iter()
                .map(|p| type_to_rust(&p.ty))
                .collect();

            if path_types.len() == 1 {
                params.push(format!(
                    "Path({}): Path<{}>",
                    to_snake_case(&endpoint.path_params[0].name),
                    path_types[0]
                ));
            } else {
                let names: Vec<String> = endpoint
                    .path_params
                    .iter()
                    .map(|p| to_snake_case(&p.name))
                    .collect();
                params.push(format!(
                    "Path(({})) : Path<({})>",
                    names.join(", "),
                    path_types.join(", ")
                ));
            }
        }

        // Query parameters
        if !endpoint.query_params.is_empty() {
            let query_struct = format!("{}Query", to_pascal_case(&handler_name));
            params.push(format!("Query(query): Query<{}>", query_struct));
            extractors.push(self.generate_query_struct(endpoint, &query_struct, ctx));
        }

        // Request body
        if let Some(body) = &endpoint.request_body {
            let ty = type_to_rust(&body.schema);
            params.push(format!("Json(body): Json<{}>", ty));
        }

        // Return type
        let return_type = self.get_return_type(endpoint);

        // Function signature
        let params_str = params.join(", ");
        lines.push(format!(
            "pub async fn {}({}) -> {} {{",
            handler_name, params_str, return_type
        ));

        // Stub body — replace with actual handler logic
        lines.push(format!(
            "{}todo!(\"Implement handler: {}\")",
            ctx.indent, handler_name
        ));

        lines.push("}".to_string());

        // Prepend query struct if needed
        if !extractors.is_empty() {
            let mut result = extractors.join("\n\n");
            result.push_str("\n\n");
            result.push_str(&lines.join("\n"));
            return result;
        }

        lines.join("\n")
    }

    /// Generate query parameter struct
    fn generate_query_struct(
        &self,
        endpoint: &EndpointDef,
        struct_name: &str,
        ctx: &GenContext,
    ) -> String {
        let mut lines = Vec::new();

        lines.push("#[derive(Debug, Deserialize)]".to_string());
        lines.push(format!("pub struct {} {{", struct_name));

        for p in &endpoint.query_params {
            let ty = type_to_rust(&p.ty);
            if p.required {
                lines.push(format!(
                    "{}pub {}: {},",
                    ctx.indent,
                    to_snake_case(&p.name),
                    ty
                ));
            } else {
                lines.push(format!(
                    "{}pub {}: Option<{}>,",
                    ctx.indent,
                    to_snake_case(&p.name),
                    ty
                ));
            }
        }

        lines.push("}".to_string());

        lines.join("\n")
    }

    /// Generate handler name from path
    fn path_to_handler_name(&self, path: &str, method: HttpMethod) -> String {
        let prefix = match method {
            HttpMethod::Get => "get",
            HttpMethod::Post => "create",
            HttpMethod::Put => "update",
            HttpMethod::Patch => "patch",
            HttpMethod::Delete => "delete",
            HttpMethod::Head => "head",
            HttpMethod::Options => "options",
        };

        let path_part = path
            .split('/')
            .filter(|s| !s.is_empty() && !s.starts_with('{'))
            .collect::<Vec<_>>()
            .join("_");

        format!("{}_{}", prefix, to_snake_case(&path_part))
    }

    /// Get return type for endpoint
    fn get_return_type(&self, endpoint: &EndpointDef) -> String {
        for resp in &endpoint.responses {
            if resp.status_code >= 200 && resp.status_code < 300 {
                if let Some(schema) = &resp.schema {
                    return format!("Json<{}>", type_to_rust(schema));
                }
            }
        }
        "impl IntoResponse".to_string()
    }

    /// Generate imports
    fn generate_imports(&self) -> Vec<String> {
        vec![
            "use axum::{Router, Json, extract::{Path, Query}, routing::{get, post, put, patch, delete}};".to_string(),
            "use axum::response::IntoResponse;".to_string(),
            "use serde::Deserialize;".to_string(),
        ]
    }
}

impl Default for AxumGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for AxumGenerator {
    fn name(&self) -> &str {
        "axum"
    }

    fn generate_data_models(
        &self,
        _spec: &DataModelSpec,
        _ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        Ok(vec![])
    }

    fn generate_rest_api(
        &self,
        spec: &RestApiSpec,
        ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        let content = self.generate_router(spec, ctx);
        let imports = self.generate_imports();

        let name = ctx
            .module_name
            .clone()
            .unwrap_or_else(|| "routes".to_string());

        Ok(vec![
            GeneratedCode::new(name, content, Language::Rust).with_imports(imports)
        ])
    }
}

/// Convert to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else if c == '-' || c == ' ' {
            result.push('_');
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert to PascalCase
fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::ir::{ParamDef, QueryParam, ResponseDef};
    use crate::type_inference::Type;

    #[test]
    fn test_generate_router() {
        let spec = RestApiSpec {
            title: "Pet Store".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            servers: vec![],
            endpoints: vec![
                EndpointDef {
                    path: "/pets".to_string(),
                    method: HttpMethod::Get,
                    operation_id: Some("listPets".to_string()),
                    summary: Some("List all pets".to_string()),
                    description: None,
                    tags: vec![],
                    path_params: vec![],
                    query_params: vec![QueryParam {
                        name: "limit".to_string(),
                        ty: Type::Int,
                        required: false,
                        description: None,
                        default: None,
                    }],
                    request_body: None,
                    responses: vec![ResponseDef {
                        status_code: 200,
                        description: "Success".to_string(),
                        schema: Some(Type::List(Box::new(Type::Instance {
                            name: "Pet".to_string(),
                            module: None,
                            type_args: vec![],
                        }))),
                        content_type: Some("application/json".to_string()),
                    }],
                    security: vec![],
                    deprecated: false,
                },
                EndpointDef {
                    path: "/pets/{pet_id}".to_string(),
                    method: HttpMethod::Get,
                    operation_id: Some("getPet".to_string()),
                    summary: Some("Get a pet".to_string()),
                    description: None,
                    tags: vec![],
                    path_params: vec![ParamDef {
                        name: "pet_id".to_string(),
                        ty: Type::Str,
                        default: None,
                    }],
                    query_params: vec![],
                    request_body: None,
                    responses: vec![ResponseDef {
                        status_code: 200,
                        description: "Success".to_string(),
                        schema: Some(Type::Instance {
                            name: "Pet".to_string(),
                            module: None,
                            type_args: vec![],
                        }),
                        content_type: Some("application/json".to_string()),
                    }],
                    security: vec![],
                    deprecated: false,
                },
            ],
            schemas: DataModelSpec::default(),
            security_schemes: vec![],
        };

        let gen = AxumGenerator::new();
        let ctx = GenContext::default();
        let result = gen.generate_rest_api(&spec, &ctx).unwrap();

        assert_eq!(result.len(), 1);
        let code = &result[0].content;

        assert!(code.contains("pub fn create_router() -> Router"));
        assert!(code.contains(".route(\"/pets\", get(list_pets))"));
        assert!(code.contains(".route(\"/pets/:pet_id\", get(get_pet))"));
        assert!(code.contains("pub async fn list_pets"));
        assert!(code.contains("pub async fn get_pet"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/gen/rust/axum.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/gen/rust/axum.rs` captured during libs codegen standardization.
```
