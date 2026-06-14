//! cclab.quasar route handler generator
//!
//! Generates API route handlers from OpenAPI specs.

use crate::gen::python::type_to_python;
use crate::gen::traits::{CodeGenerator, GenContext, GenResult, GeneratedCode, Language};
use crate::spec::ir::{DataModelSpec, EndpointDef, HttpMethod, RestApiSpec};

/// Quasar (route handlers) code generator
pub struct QuasarGenerator;

impl QuasarGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate router from RestApiSpec
    pub fn generate_router(&self, spec: &RestApiSpec, ctx: &GenContext) -> String {
        let mut lines = Vec::new();

        let router_name = to_snake_case(&spec.title.replace(" ", "").replace("-", ""));

        // Create router
        lines.push(format!(
            "{}_router = Router(prefix=\"/api/v{}\")",
            router_name,
            spec.version.split('.').next().unwrap_or("1")
        ));
        lines.push(String::new());

        // Generate handlers grouped by tags
        let mut handlers_by_tag: std::collections::HashMap<String, Vec<&EndpointDef>> =
            std::collections::HashMap::new();

        for endpoint in &spec.endpoints {
            let tag = endpoint
                .tags
                .first()
                .cloned()
                .unwrap_or_else(|| "default".to_string());
            handlers_by_tag.entry(tag).or_default().push(endpoint);
        }

        for (tag, endpoints) in handlers_by_tag {
            lines.push(format!("# {} endpoints", tag));
            for endpoint in endpoints {
                lines.push(self.generate_handler(endpoint, &router_name, ctx));
                lines.push(String::new());
            }
        }

        lines.join("\n")
    }

    /// Generate a single handler
    fn generate_handler(
        &self,
        endpoint: &EndpointDef,
        router_name: &str,
        ctx: &GenContext,
    ) -> String {
        let mut lines = Vec::new();

        // Handler name from operation_id or path
        let handler_name = endpoint
            .operation_id
            .clone()
            .map(|s| to_snake_case(&s))
            .unwrap_or_else(|| self.path_to_handler_name(&endpoint.path, endpoint.method));

        // Decorator
        let method_lower = format!("{:?}", endpoint.method).to_lowercase();
        lines.push(format!(
            "@{}_router.{}(\"{}\")",
            router_name, method_lower, endpoint.path
        ));

        // Build parameters
        let mut params = Vec::new();

        // Path parameters
        for p in &endpoint.path_params {
            let ty = type_to_python(&p.ty);
            params.push(format!("{}: {}", p.name, ty));
        }

        // Query parameters
        for p in &endpoint.query_params {
            let ty = type_to_python(&p.ty);
            if p.required {
                params.push(format!("{}: Query[{}]", p.name, ty));
            } else if let Some(default) = &p.default {
                params.push(format!("{}: Query[{}] = {}", p.name, ty, default));
            } else {
                params.push(format!("{}: Query[Optional[{}]] = None", p.name, ty));
            }
        }

        // Request body
        if let Some(body) = &endpoint.request_body {
            let ty = type_to_python(&body.schema);
            if body.required {
                params.push(format!("body: Body[{}]", ty));
            } else {
                params.push(format!("body: Body[Optional[{}]] = None", ty));
            }
        }

        // Function signature
        let return_type = self.get_return_type(endpoint);
        let params_str = params.join(", ");

        lines.push(format!(
            "async def {}({}) -> {}:",
            handler_name, params_str, return_type
        ));

        // Docstring
        if ctx.generate_docs {
            if let Some(summary) = &endpoint.summary {
                lines.push(format!("{}\"\"\"{}\"\"\"", ctx.indent, summary));
            }
        }

        // Stub body — replace with actual handler logic
        lines.push(format!(
            "{}raise NotImplementedError(\"Implement handler: {}\")",
            ctx.indent, handler_name
        ));

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
        // Find success response (2xx)
        for resp in &endpoint.responses {
            if resp.status_code >= 200 && resp.status_code < 300 {
                if let Some(schema) = &resp.schema {
                    return format!("Response[{}]", type_to_python(schema));
                }
            }
        }
        "Response[Any]".to_string()
    }

    /// Generate imports
    fn generate_imports(&self, spec: &RestApiSpec) -> Vec<String> {
        let mut imports = vec![
            "from typing import Optional, Any".to_string(),
            "from cclab.quasar import Router, Query, Body, Response".to_string(),
        ];

        // Add model imports if schemas exist
        if !spec.schemas.models.is_empty() {
            let model_names: Vec<_> = spec.schemas.models.iter().map(|m| m.name.clone()).collect();
            imports.push(format!("from .models import {}", model_names.join(", ")));
        }

        imports
    }
}

impl Default for QuasarGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for QuasarGenerator {
    fn name(&self) -> &str {
        "quasar"
    }

    fn generate_data_models(
        &self,
        _spec: &DataModelSpec,
        _ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        // QuasarGenerator primarily generates from RestApiSpec
        Ok(vec![])
    }

    fn generate_rest_api(
        &self,
        spec: &RestApiSpec,
        ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        let content = self.generate_router(spec, ctx);
        let imports = self.generate_imports(spec);

        let name = ctx
            .module_name
            .clone()
            .unwrap_or_else(|| "routes".to_string());

        Ok(vec![
            GeneratedCode::new(name, content, Language::Python).with_imports(imports)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::ir::{QueryParam, ResponseDef};
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
                    tags: vec!["pets".to_string()],
                    path_params: vec![],
                    query_params: vec![QueryParam {
                        name: "limit".to_string(),
                        ty: Type::Int,
                        required: false,
                        description: None,
                        default: Some("10".to_string()),
                    }],
                    request_body: None,
                    responses: vec![],
                    security: vec![],
                    deprecated: false,
                },
                EndpointDef {
                    path: "/pets".to_string(),
                    method: HttpMethod::Post,
                    operation_id: Some("createPet".to_string()),
                    summary: Some("Create a pet".to_string()),
                    description: None,
                    tags: vec!["pets".to_string()],
                    path_params: vec![],
                    query_params: vec![],
                    request_body: Some(crate::spec::ir::RequestBody {
                        content_type: "application/json".to_string(),
                        schema: Type::Instance {
                            name: "NewPet".to_string(),
                            module: None,
                            type_args: vec![],
                        },
                        required: true,
                        description: None,
                    }),
                    responses: vec![ResponseDef {
                        status_code: 201,
                        description: "Created".to_string(),
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

        let gen = QuasarGenerator::new();
        let ctx = GenContext::default();
        let result = gen.generate_rest_api(&spec, &ctx).unwrap();

        assert_eq!(result.len(), 1);
        let code = &result[0].content;

        assert!(code.contains("pet_store_router = Router"));
        assert!(code.contains("@pet_store_router.get"));
        assert!(code.contains("@pet_store_router.post"));
        assert!(code.contains("async def list_pets"));
        assert!(code.contains("async def create_pet"));
        assert!(code.contains("Query[int]"));
        assert!(code.contains("Body[NewPet]"));
    }
}
