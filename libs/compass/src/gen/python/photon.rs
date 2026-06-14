//! cclab.photon HTTP client generator
//!
//! Generates async HTTP client classes from OpenAPI specs.

use crate::gen::python::type_to_python;
use crate::gen::traits::{CodeGenerator, GenContext, GenResult, GeneratedCode, Language};
use crate::spec::ir::{DataModelSpec, EndpointDef, HttpMethod, RestApiSpec};

/// Photon (HTTP client) code generator
pub struct PhotonGenerator;

impl PhotonGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate client class from RestApiSpec
    pub fn generate_client(&self, spec: &RestApiSpec, ctx: &GenContext) -> String {
        let mut lines = Vec::new();

        let class_name = to_pascal_case(&spec.title.replace(" ", "").replace("-", ""));

        // Class docstring
        if ctx.generate_docs {
            if let Some(desc) = &spec.description {
                lines.push(format!("class {}Client:", class_name));
                lines.push(format!("{}\"\"\"{}\"\"\"", ctx.indent, desc));
            } else {
                lines.push(format!("class {}Client:", class_name));
                lines.push(format!(
                    "{}\"\"\"Generated HTTP client for {} API.\"\"\"",
                    ctx.indent, spec.title
                ));
            }
        } else {
            lines.push(format!("class {}Client:", class_name));
        }

        lines.push(String::new());

        // Constructor
        lines.push(format!("{}def __init__(", ctx.indent));
        lines.push(format!("{}{}self,", ctx.indent, ctx.indent));
        lines.push(format!("{}{}base_url: str,", ctx.indent, ctx.indent));
        lines.push(format!(
            "{}{}timeout: float = 30.0,",
            ctx.indent, ctx.indent
        ));
        lines.push(format!(
            "{}{}headers: Optional[dict[str, str]] = None,",
            ctx.indent, ctx.indent
        ));
        lines.push(format!("{}{}) -> None:", ctx.indent, ctx.indent));
        lines.push(format!(
            "{}{}self._client = HttpClient(",
            ctx.indent, ctx.indent
        ));
        lines.push(format!(
            "{}{}{}base_url=base_url,",
            ctx.indent, ctx.indent, ctx.indent
        ));
        lines.push(format!(
            "{}{}{}timeout=timeout,",
            ctx.indent, ctx.indent, ctx.indent
        ));
        lines.push(format!(
            "{}{}{}headers=headers or {{}},",
            ctx.indent, ctx.indent, ctx.indent
        ));
        lines.push(format!("{}{})", ctx.indent, ctx.indent));

        lines.push(String::new());

        // Generate methods for each endpoint
        for endpoint in &spec.endpoints {
            lines.push(self.generate_method(endpoint, ctx));
            lines.push(String::new());
        }

        // Close context manager methods
        lines.push(format!(
            "{}async def __aenter__(self) -> \"{}Client\":",
            ctx.indent, class_name
        ));
        lines.push(format!("{}{}return self", ctx.indent, ctx.indent));
        lines.push(String::new());

        lines.push(format!(
            "{}async def __aexit__(self, *args) -> None:",
            ctx.indent
        ));
        lines.push(format!(
            "{}{}await self._client.close()",
            ctx.indent, ctx.indent
        ));

        lines.join("\n")
    }

    /// Generate a method for an endpoint
    fn generate_method(&self, endpoint: &EndpointDef, ctx: &GenContext) -> String {
        let mut lines = Vec::new();

        // Method name from operation_id or path
        let method_name = endpoint
            .operation_id
            .clone()
            .map(|s| to_snake_case(&s))
            .unwrap_or_else(|| self.path_to_method_name(&endpoint.path, endpoint.method));

        // Build parameters
        let mut params = vec!["self".to_string()];

        // Path parameters
        for p in &endpoint.path_params {
            let ty = type_to_python(&p.ty);
            params.push(format!("{}: {}", p.name, ty));
        }

        // Query parameters (required first)
        let required_query: Vec<_> = endpoint
            .query_params
            .iter()
            .filter(|p| p.required)
            .collect();
        let optional_query: Vec<_> = endpoint
            .query_params
            .iter()
            .filter(|p| !p.required)
            .collect();

        for p in &required_query {
            let ty = type_to_python(&p.ty);
            params.push(format!("{}: {}", p.name, ty));
        }

        // Request body
        if let Some(body) = &endpoint.request_body {
            if body.required {
                let ty = type_to_python(&body.schema);
                params.push(format!("body: {}", ty));
            }
        }

        // Optional query params
        for p in &optional_query {
            let ty = type_to_python(&p.ty);
            if let Some(default) = &p.default {
                params.push(format!("{}: {} = {}", p.name, ty, default));
            } else {
                params.push(format!("{}: Optional[{}] = None", p.name, ty));
            }
        }

        // Optional request body
        if let Some(body) = &endpoint.request_body {
            if !body.required {
                let ty = type_to_python(&body.schema);
                params.push(format!("body: Optional[{}] = None", ty));
            }
        }

        // Method signature
        let params_str = params.join(", ");
        let return_type = self.get_return_type(endpoint);

        lines.push(format!(
            "{}async def {}({}) -> {}:",
            ctx.indent, method_name, params_str, return_type
        ));

        // Docstring
        if ctx.generate_docs {
            if let Some(summary) = &endpoint.summary {
                lines.push(format!(
                    "{}{}\"\"\"{}\"\"\"",
                    ctx.indent, ctx.indent, summary
                ));
            }
        }

        // Build URL
        let path = endpoint.path.replace("{", "{").replace("}", "}");
        lines.push(format!(
            "{}{}url = f\"{{self._client.base_url}}{}\"",
            ctx.indent, ctx.indent, path
        ));

        // Build query params
        if !endpoint.query_params.is_empty() {
            lines.push(format!("{}{}params = {{}}", ctx.indent, ctx.indent));
            for p in &endpoint.query_params {
                if p.required {
                    lines.push(format!(
                        "{}{}params[\"{}\"] = {}",
                        ctx.indent, ctx.indent, p.name, p.name
                    ));
                } else {
                    lines.push(format!(
                        "{}{}if {} is not None:",
                        ctx.indent, ctx.indent, p.name
                    ));
                    lines.push(format!(
                        "{}{}{}params[\"{}\"] = {}",
                        ctx.indent, ctx.indent, ctx.indent, p.name, p.name
                    ));
                }
            }
        }

        // Make request
        let method_lower = format!("{:?}", endpoint.method).to_lowercase();
        let has_params = !endpoint.query_params.is_empty();
        let has_body = endpoint.request_body.is_some();

        let mut call_args = vec!["url".to_string()];
        if has_params {
            call_args.push("params=params".to_string());
        }
        if has_body {
            call_args.push("json=body".to_string());
        }

        lines.push(format!(
            "{}{}response = await self._client.{}({})",
            ctx.indent,
            ctx.indent,
            method_lower,
            call_args.join(", ")
        ));

        lines.push(format!(
            "{}{}return response.json()",
            ctx.indent, ctx.indent
        ));

        lines.join("\n")
    }

    /// Generate method name from path
    fn path_to_method_name(&self, path: &str, method: HttpMethod) -> String {
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
                    return type_to_python(schema);
                }
            }
        }
        "Any".to_string()
    }

    /// Generate imports
    fn generate_imports(&self) -> Vec<String> {
        vec![
            "from typing import Optional, Any".to_string(),
            "from cclab.photon import HttpClient".to_string(),
        ]
    }
}

impl Default for PhotonGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for PhotonGenerator {
    fn name(&self) -> &str {
        "photon"
    }

    fn generate_data_models(
        &self,
        _spec: &DataModelSpec,
        _ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        // PhotonGenerator primarily generates from RestApiSpec
        Ok(vec![])
    }

    fn generate_rest_api(
        &self,
        spec: &RestApiSpec,
        ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        let content = self.generate_client(spec, ctx);
        let imports = self.generate_imports();

        let name = ctx
            .module_name
            .clone()
            .unwrap_or_else(|| "client".to_string());

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
    use crate::spec::ir::{ParamDef, QueryParam, ResponseDef, ServerDef};
    use crate::type_inference::Type;

    #[test]
    fn test_generate_client() {
        let spec = RestApiSpec {
            title: "Pet Store".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A sample pet store API".to_string()),
            servers: vec![ServerDef {
                url: "https://api.example.com".to_string(),
                description: None,
            }],
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
                    path: "/pets/{petId}".to_string(),
                    method: HttpMethod::Get,
                    operation_id: Some("getPet".to_string()),
                    summary: Some("Get a pet by ID".to_string()),
                    description: None,
                    tags: vec!["pets".to_string()],
                    path_params: vec![ParamDef {
                        name: "petId".to_string(),
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

        let gen = PhotonGenerator::new();
        let ctx = GenContext::default();
        let result = gen.generate_rest_api(&spec, &ctx).unwrap();

        assert_eq!(result.len(), 1);
        let code = &result[0].content;

        assert!(code.contains("class PetStoreClient:"));
        assert!(code.contains("async def list_pets"));
        assert!(code.contains("async def get_pet"));
        assert!(code.contains("petId: str"));
    }
}
