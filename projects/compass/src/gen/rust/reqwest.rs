//! reqwest HTTP client generator
//!
//! Generates async HTTP client from OpenAPI specs.

use crate::gen::rust::type_to_rust;
use crate::gen::traits::{CodeGenerator, GenContext, GenResult, GeneratedCode, Language};
use crate::spec::ir::{DataModelSpec, EndpointDef, HttpMethod, RestApiSpec};

/// Reqwest (HTTP client) code generator
pub struct ReqwestGenerator;

impl ReqwestGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate client struct from RestApiSpec
    pub fn generate_client(&self, spec: &RestApiSpec, ctx: &GenContext) -> String {
        let mut lines = Vec::new();

        let struct_name = to_pascal_case(&spec.title.replace(" ", "").replace("-", ""));

        // Struct definition
        if ctx.generate_docs {
            if let Some(desc) = &spec.description {
                lines.push(format!("/// {}", desc));
            } else {
                lines.push(format!("/// HTTP client for {} API", spec.title));
            }
        }

        lines.push("#[derive(Clone)]".to_string());
        lines.push(format!("pub struct {}Client {{", struct_name));
        lines.push(format!("{}client: reqwest::Client,", ctx.indent));
        lines.push(format!("{}base_url: String,", ctx.indent));
        lines.push("}".to_string());

        lines.push(String::new());

        // Impl block
        lines.push(format!("impl {}Client {{", struct_name));

        // Constructor
        lines.push(format!("{}/// Create a new client instance", ctx.indent));
        lines.push(format!(
            "{}pub fn new(base_url: impl Into<String>) -> Self {{",
            ctx.indent
        ));
        lines.push(format!("{}{}Self {{", ctx.indent, ctx.indent));
        lines.push(format!(
            "{}{}{}client: reqwest::Client::new(),",
            ctx.indent, ctx.indent, ctx.indent
        ));
        lines.push(format!(
            "{}{}{}base_url: base_url.into(),",
            ctx.indent, ctx.indent, ctx.indent
        ));
        lines.push(format!("{}{}}}", ctx.indent, ctx.indent));
        lines.push(format!("{}}}", ctx.indent));

        lines.push(String::new());

        // Constructor with custom client
        lines.push(format!(
            "{}/// Create with a custom reqwest client",
            ctx.indent
        ));
        lines.push(format!(
            "{}pub fn with_client(client: reqwest::Client, base_url: impl Into<String>) -> Self {{",
            ctx.indent
        ));
        lines.push(format!("{}{}Self {{", ctx.indent, ctx.indent));
        lines.push(format!("{}{}{}client,", ctx.indent, ctx.indent, ctx.indent));
        lines.push(format!(
            "{}{}{}base_url: base_url.into(),",
            ctx.indent, ctx.indent, ctx.indent
        ));
        lines.push(format!("{}{}}}", ctx.indent, ctx.indent));
        lines.push(format!("{}}}", ctx.indent));

        lines.push(String::new());

        // Generate methods for each endpoint
        for endpoint in &spec.endpoints {
            lines.push(self.generate_method(endpoint, ctx));
            lines.push(String::new());
        }

        lines.push("}".to_string());

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

        // Doc comment
        if ctx.generate_docs {
            if let Some(summary) = &endpoint.summary {
                lines.push(format!("{}/// {}", ctx.indent, summary));
            }
        }

        // Build parameters
        let mut params = vec!["&self".to_string()];

        // Path parameters
        for p in &endpoint.path_params {
            let ty = type_to_rust(&p.ty);
            params.push(format!("{}: {}", to_snake_case(&p.name), param_type(&ty)));
        }

        // Query parameters (required first)
        for p in endpoint.query_params.iter().filter(|p| p.required) {
            let ty = type_to_rust(&p.ty);
            params.push(format!("{}: {}", to_snake_case(&p.name), param_type(&ty)));
        }

        // Request body
        if let Some(body) = &endpoint.request_body {
            if body.required {
                let ty = type_to_rust(&body.schema);
                params.push(format!("body: &{}", ty));
            }
        }

        // Optional query params
        for p in endpoint.query_params.iter().filter(|p| !p.required) {
            let ty = type_to_rust(&p.ty);
            params.push(format!("{}: Option<{}>", to_snake_case(&p.name), ty));
        }

        // Optional request body
        if let Some(body) = &endpoint.request_body {
            if !body.required {
                let ty = type_to_rust(&body.schema);
                params.push(format!("body: Option<&{}>", ty));
            }
        }

        // Method signature
        let params_str = params.join(", ");
        let return_type = self.get_return_type(endpoint);

        lines.push(format!(
            "{}pub async fn {}({}) -> Result<{}, reqwest::Error> {{",
            ctx.indent, method_name, params_str, return_type
        ));

        // Build URL
        let url_build = self.build_url_code(endpoint, ctx);
        lines.push(url_build);

        // Build request
        let method_lower = format!("{:?}", endpoint.method).to_lowercase();
        lines.push(format!(
            "{}{}let mut request = self.client.{}(&url);",
            ctx.indent, ctx.indent, method_lower
        ));

        // Add query params
        for p in &endpoint.query_params {
            let name = to_snake_case(&p.name);
            if p.required {
                lines.push(format!(
                    "{}{}request = request.query(&[(\"{}\", &{})]);",
                    ctx.indent, ctx.indent, p.name, name
                ));
            } else {
                lines.push(format!(
                    "{}{}if let Some(ref v) = {} {{",
                    ctx.indent, ctx.indent, name
                ));
                lines.push(format!(
                    "{}{}{}request = request.query(&[(\"{}\", v)]);",
                    ctx.indent, ctx.indent, ctx.indent, p.name
                ));
                lines.push(format!("{}{}}}", ctx.indent, ctx.indent));
            }
        }

        // Add body
        if let Some(body) = &endpoint.request_body {
            if body.required {
                lines.push(format!(
                    "{}{}request = request.json(body);",
                    ctx.indent, ctx.indent
                ));
            } else {
                lines.push(format!(
                    "{}{}if let Some(b) = body {{",
                    ctx.indent, ctx.indent
                ));
                lines.push(format!(
                    "{}{}{}request = request.json(b);",
                    ctx.indent, ctx.indent, ctx.indent
                ));
                lines.push(format!("{}{}}}", ctx.indent, ctx.indent));
            }
        }

        // Send and parse response
        lines.push(format!(
            "{}{}let response = request.send().await?;",
            ctx.indent, ctx.indent
        ));
        lines.push(format!("{}{}response.json().await", ctx.indent, ctx.indent));

        lines.push(format!("{}}}", ctx.indent));

        lines.join("\n")
    }

    /// Build URL construction code
    fn build_url_code(&self, endpoint: &EndpointDef, ctx: &GenContext) -> String {
        if endpoint.path_params.is_empty() {
            format!(
                "{}{}let url = format!(\"{{}}{}\", self.base_url);",
                ctx.indent, ctx.indent, endpoint.path
            )
        } else {
            // Replace {param} with {} for format!
            let mut path = endpoint.path.clone();
            let mut format_args = vec!["self.base_url".to_string()];

            for p in &endpoint.path_params {
                path = path.replace(&format!("{{{}}}", p.name), "{}");
                format_args.push(to_snake_case(&p.name));
            }

            format!(
                "{}{}let url = format!(\"{{}}{}\", {});",
                ctx.indent,
                ctx.indent,
                path,
                format_args.join(", ")
            )
        }
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
        for resp in &endpoint.responses {
            if resp.status_code >= 200 && resp.status_code < 300 {
                if let Some(schema) = &resp.schema {
                    return type_to_rust(schema);
                }
            }
        }
        "serde_json::Value".to_string()
    }

    /// Generate imports
    fn generate_imports(&self) -> Vec<String> {
        vec![]
    }
}

impl Default for ReqwestGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for ReqwestGenerator {
    fn name(&self) -> &str {
        "reqwest"
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
        let content = self.generate_client(spec, ctx);
        let imports = self.generate_imports();

        let name = ctx
            .module_name
            .clone()
            .unwrap_or_else(|| "client".to_string());

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

/// Get parameter type (impl Into for strings, direct for others)
fn param_type(ty: &str) -> String {
    if ty == "String" {
        "impl AsRef<str>".to_string()
    } else {
        ty.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::ir::{ParamDef, ResponseDef};
    use crate::type_inference::Type;

    #[test]
    fn test_generate_client() {
        let spec = RestApiSpec {
            title: "Pet Store".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A sample pet store API".to_string()),
            servers: vec![],
            endpoints: vec![EndpointDef {
                path: "/pets/{pet_id}".to_string(),
                method: HttpMethod::Get,
                operation_id: Some("getPet".to_string()),
                summary: Some("Get a pet by ID".to_string()),
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
            }],
            schemas: DataModelSpec::default(),
            security_schemes: vec![],
        };

        let gen = ReqwestGenerator::new();
        let ctx = GenContext::default();
        let result = gen.generate_rest_api(&spec, &ctx).unwrap();

        assert_eq!(result.len(), 1);
        let code = &result[0].content;

        assert!(code.contains("pub struct PetStoreClient"));
        assert!(code.contains("pub async fn get_pet"));
        assert!(code.contains("pet_id: impl AsRef<str>"));
    }
}
