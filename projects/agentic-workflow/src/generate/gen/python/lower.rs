// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-generate-gen-python.md#schema
// CODEGEN-BEGIN
//! Lower TD section YAML into Python backend IR.
//!
//! This keeps `apply.rs` as dispatch glue only. Python-specific section
//! shapes live here and lower into the shared Python codegen IR before the
//! emitter renders a Python AST to source.

use super::types::{
    HttpMethod, ImportIr, PydanticField, PydanticModelIr, PythonBackendSpec, PythonModuleIr,
    RouteRecord, RouterIr,
};
use crate::td_ast::payloads::{OpenApiOperation, OpenApiPathItem, OpenApiPayload};

#[derive(Debug, serde::Deserialize)]
struct PythonBackendSpecYaml {
    #[serde(default)]
    spec_id: String,
    #[serde(default)]
    routers: Vec<PythonRouterYaml>,
    #[serde(default)]
    pydantic_models: Vec<PythonPydanticModelYaml>,
    #[serde(default)]
    python_modules: Vec<PythonModuleIr>,
    #[serde(default)]
    imports: Vec<PythonImportYaml>,
    #[serde(default)]
    module_docstring: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct PythonRouterYaml {
    #[serde(default)]
    name: String,
    #[serde(default)]
    prefix: String,
    #[serde(default)]
    tag: String,
    #[serde(default)]
    routes: Vec<PythonRouteYaml>,
}

#[derive(Debug, serde::Deserialize)]
struct PythonRouteYaml {
    #[serde(default)]
    method: String,
    #[serde(default)]
    path: String,
    #[serde(default)]
    handler_symbol: String,
    #[serde(default)]
    request_model: Option<String>,
    #[serde(default)]
    response_model: String,
}

#[derive(Debug, serde::Deserialize)]
struct PythonPydanticModelYaml {
    #[serde(default)]
    name: String,
    #[serde(default = "default_pydantic_base")]
    base: String,
    #[serde(default)]
    docstring: Option<String>,
    #[serde(default)]
    fields: Vec<PythonPydanticFieldYaml>,
}

#[derive(Debug, serde::Deserialize)]
struct PythonPydanticFieldYaml {
    #[serde(default)]
    name: String,
    #[serde(default)]
    py_type: String,
    #[serde(default)]
    default: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct PythonImportYaml {
    #[serde(default)]
    module: String,
    #[serde(default)]
    names: Vec<String>,
}

fn default_pydantic_base() -> String {
    "BaseModel".to_string()
}

/// Parse and lower a Python backend section YAML body.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
pub fn lower_backend_spec_yaml(yaml: &str, spec_path: &str) -> Option<PythonBackendSpec> {
    let parsed: PythonBackendSpecYaml = serde_yaml::from_str(yaml).ok()?;
    Some(lower_backend_spec(parsed, spec_path))
}

/// Lower a TD AST payload value into Python backend IR.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
pub fn lower_backend_spec_value(
    value: serde_yaml::Value,
    spec_path: &str,
) -> Option<PythonBackendSpec> {
    let parsed: PythonBackendSpecYaml = serde_yaml::from_value(value).ok()?;
    Some(lower_backend_spec(parsed, spec_path))
}

/// Lower an OpenAPI 3.1 TD payload into the shared Python backend IR.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#py-emit-router
pub fn lower_openapi_payload(
    payload: &OpenApiPayload,
    spec_path: &str,
    target_path: Option<&str>,
) -> Option<PythonBackendSpec> {
    let routes: Vec<RouteRecord> = payload
        .paths
        .iter()
        .flat_map(|(path, item)| openapi_routes_for_path(path, item))
        .collect();
    if routes.is_empty() {
        return None;
    }

    let router_name = target_path
        .map(python_router_name_from_target)
        .unwrap_or_else(|| "api".to_string());
    Some(PythonBackendSpec {
        spec_id: default_spec_id(spec_path),
        routers: vec![RouterIr {
            name: router_name.clone(),
            prefix: String::new(),
            tag: openapi_tag(payload).unwrap_or(router_name),
            routes,
        }],
        pydantic_models: Vec::new(),
        python_modules: Vec::new(),
        imports: Vec::new(),
        module_docstring: None,
    })
}

fn lower_backend_spec(parsed: PythonBackendSpecYaml, spec_path: &str) -> PythonBackendSpec {
    let spec_id = if parsed.spec_id.trim().is_empty() {
        default_spec_id(spec_path)
    } else {
        parsed.spec_id
    };

    PythonBackendSpec {
        spec_id,
        routers: parsed
            .routers
            .into_iter()
            .map(|router| RouterIr {
                name: router.name,
                prefix: router.prefix,
                tag: router.tag,
                routes: router
                    .routes
                    .into_iter()
                    .filter_map(|route| {
                        Some(RouteRecord {
                            method: parse_http_method(&route.method)?,
                            path: route.path,
                            handler_symbol: route.handler_symbol,
                            request_model: route.request_model,
                            response_model: route.response_model,
                        })
                    })
                    .collect(),
            })
            .collect(),
        pydantic_models: parsed
            .pydantic_models
            .into_iter()
            .filter(|model| !model.name.trim().is_empty())
            .map(|model| PydanticModelIr {
                name: model.name,
                base: model.base,
                docstring: model.docstring.map(trim_trailing_newlines),
                fields: model
                    .fields
                    .into_iter()
                    .filter(|field| !field.name.trim().is_empty())
                    .map(|field| PydanticField {
                        name: field.name,
                        py_type: field.py_type,
                        default: field.default,
                    })
                    .collect(),
            })
            .collect(),
        python_modules: parsed.python_modules,
        imports: parsed
            .imports
            .into_iter()
            .filter(|import| !import.module.trim().is_empty())
            .map(|import| ImportIr {
                module: import.module,
                names: import.names,
            })
            .collect(),
        module_docstring: parsed.module_docstring,
    }
}

fn default_spec_id(spec_path: &str) -> String {
    spec_path
        .rsplit('/')
        .next()
        .and_then(|name| name.strip_suffix(".md"))
        .unwrap_or(spec_path)
        .to_string()
}

fn trim_trailing_newlines(value: String) -> String {
    value.trim_end_matches(['\r', '\n']).to_string()
}

/// Parse the stable HTTP method vocabulary used by Python router IR.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
pub fn parse_http_method(method: &str) -> Option<HttpMethod> {
    match method.to_ascii_lowercase().as_str() {
        "get" => Some(HttpMethod::Get),
        "post" => Some(HttpMethod::Post),
        "put" => Some(HttpMethod::Put),
        "delete" => Some(HttpMethod::Delete),
        "patch" => Some(HttpMethod::Patch),
        _ => None,
    }
}

fn openapi_routes_for_path(path: &str, item: &OpenApiPathItem) -> Vec<RouteRecord> {
    let mut routes = Vec::new();
    if let Some(operation) = item.get.as_ref() {
        routes.push(openapi_route_record(HttpMethod::Get, path, operation));
    }
    if let Some(operation) = item.post.as_ref() {
        routes.push(openapi_route_record(HttpMethod::Post, path, operation));
    }
    if let Some(operation) = item.put.as_ref() {
        routes.push(openapi_route_record(HttpMethod::Put, path, operation));
    }
    if let Some(operation) = item.delete.as_ref() {
        routes.push(openapi_route_record(HttpMethod::Delete, path, operation));
    }
    if let Some(operation) = item.patch.as_ref() {
        routes.push(openapi_route_record(HttpMethod::Patch, path, operation));
    }
    routes
}

fn openapi_route_record(
    method: HttpMethod,
    path: &str,
    operation: &OpenApiOperation,
) -> RouteRecord {
    RouteRecord {
        method,
        path: path.to_string(),
        handler_symbol: operation
            .operation_id
            .clone()
            .unwrap_or_else(|| default_openapi_handler_symbol(method, path)),
        request_model: openapi_request_model(operation),
        response_model: openapi_response_model(operation).unwrap_or_else(|| "dict".to_string()),
    }
}

fn openapi_tag(payload: &OpenApiPayload) -> Option<String> {
    yaml_mapping_get(payload.info.as_ref()?, "title")
        .and_then(|value| value.as_str())
        .map(|title| title.trim().to_string())
        .filter(|title| !title.is_empty())
}

fn openapi_request_model(operation: &OpenApiOperation) -> Option<String> {
    yaml_mapping_get(&operation.extra, "requestBody").and_then(openapi_content_schema_model)
}

fn openapi_response_model(operation: &OpenApiOperation) -> Option<String> {
    let responses = yaml_mapping_get(&operation.extra, "responses")?.as_mapping()?;
    for status in ["200", "201", "202", "203", "default"] {
        let Some(response) = yaml_mapping_get_in_mapping(responses, status) else {
            continue;
        };
        if let Some(model) = openapi_content_schema_model(response) {
            return Some(model);
        }
    }
    responses
        .iter()
        .filter_map(|(status, response)| {
            status
                .as_str()
                .filter(|status| status.starts_with('2'))
                .and_then(|_| openapi_content_schema_model(response))
        })
        .next()
}

fn openapi_content_schema_model(value: &serde_yaml::Value) -> Option<String> {
    let content = yaml_mapping_get(value, "content")?.as_mapping()?;
    if let Some(json) = yaml_mapping_get_in_mapping(content, "application/json") {
        if let Some(model) = yaml_mapping_get(json, "schema").and_then(openapi_schema_model) {
            return Some(model);
        }
    }
    content
        .values()
        .filter_map(|media| yaml_mapping_get(media, "schema").and_then(openapi_schema_model))
        .next()
}

fn openapi_schema_model(value: &serde_yaml::Value) -> Option<String> {
    yaml_mapping_get(value, "$ref")
        .and_then(|value| value.as_str())
        .and_then(|reference| reference.rsplit('/').next())
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .or_else(|| {
            value
                .as_mapping()
                .and_then(|mapping| yaml_mapping_get_in_mapping(mapping, "type"))
                .and_then(|value| value.as_str())
                .map(|ty| match ty {
                    "array" => "list",
                    "boolean" => "bool",
                    "integer" => "int",
                    "number" => "float",
                    "object" => "dict",
                    "string" => "str",
                    _ => "dict",
                })
                .map(str::to_string)
        })
}

fn yaml_mapping_get<'a>(value: &'a serde_yaml::Value, key: &str) -> Option<&'a serde_yaml::Value> {
    yaml_mapping_get_in_mapping(value.as_mapping()?, key)
}

fn yaml_mapping_get_in_mapping<'a>(
    mapping: &'a serde_yaml::Mapping,
    key: &str,
) -> Option<&'a serde_yaml::Value> {
    mapping.get(serde_yaml::Value::String(key.to_string()))
}

fn default_openapi_handler_symbol(method: HttpMethod, path: &str) -> String {
    let mut symbol = method.decorator().to_string();
    for segment in path.split('/') {
        let cleaned = segment
            .trim_matches(|ch| ch == '{' || ch == '}')
            .chars()
            .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
            .collect::<String>()
            .trim_matches('_')
            .to_ascii_lowercase();
        if cleaned.is_empty() {
            continue;
        }
        symbol.push('_');
        symbol.push_str(&cleaned);
    }
    symbol
}

fn python_router_name_from_target(target_path: &str) -> String {
    let path = std::path::Path::new(target_path);
    if path.file_stem().and_then(|name| name.to_str()) != Some("router") {
        return path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("api")
            .to_string();
    }
    path.parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("api")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lower_backend_spec_yaml_builds_python_ir() {
        let yaml = r#"
module_docstring: |
  Widget API models.
imports:
  - { module: pydantic, names: [] }
pydantic_models:
  - name: WidgetRequest
    base: pydantic.BaseModel
    docstring: |
      WidgetRequest.
    fields:
      - { name: name, py_type: str }
      - { name: active, py_type: bool, default: "True" }
"#;

        let spec = lower_backend_spec_yaml(yaml, "widget/api_models.md").expect("lower spec");

        assert_eq!(spec.spec_id, "api_models");
        assert_eq!(spec.imports[0].module, "pydantic");
        assert_eq!(spec.pydantic_models[0].name, "WidgetRequest");
        assert_eq!(
            spec.pydantic_models[0].docstring.as_deref(),
            Some("WidgetRequest.")
        );
        assert_eq!(
            spec.pydantic_models[0].fields[1].default.as_deref(),
            Some("True")
        );
        assert!(spec.python_modules.is_empty());
    }

    #[test]
    fn lower_backend_spec_value_accepts_typed_td_payload_shape() {
        let value: serde_yaml::Value = serde_yaml::from_str(
            r#"
spec_id: typed-fixture
pydantic_models:
  - name: TypedWidget
    fields:
      - { name: name, py_type: str }
"#,
        )
        .expect("yaml value");

        let spec = lower_backend_spec_value(value, "typed-fixture.md").expect("lower value");

        assert_eq!(spec.spec_id, "typed-fixture");
        assert_eq!(spec.pydantic_models[0].name, "TypedWidget");
        assert_eq!(spec.pydantic_models[0].base, "BaseModel");
    }

    #[test]
    fn lower_backend_spec_yaml_accepts_python_modules() {
        let yaml = r#"
python_modules:
  - path: src/app.py
    body:
      - kind: function
        name: handle
        args: []
        body:
          - "return 1"
"#;

        let spec = lower_backend_spec_yaml(yaml, "app.md").expect("lower value");

        assert_eq!(spec.python_modules[0].path, "src/app.py");
    }

    #[test]
    fn lower_openapi_payload_builds_router_ir() {
        let payload = crate::td_ast::payloads::OpenApiPayload::from_yaml_str(
            r##"
openapi: 3.1.0
info:
  title: Project API
  version: "0.1.0"
paths:
  /projects:
    post:
      operationId: create_project
      requestBody:
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/ProjectRequest"
      responses:
        "201":
          description: Created
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/ProjectResponse"
components:
  schemas:
    ProjectRequest:
      type: object
    ProjectResponse:
      type: object
"##,
        )
        .expect("openapi parses");

        let spec = lower_openapi_payload(&payload, "api.md", Some("src/api/router.py"))
            .expect("lower openapi");

        assert_eq!(spec.spec_id, "api");
        assert_eq!(spec.routers[0].name, "api");
        assert_eq!(spec.routers[0].tag, "Project API");
        assert_eq!(spec.routers[0].routes[0].handler_symbol, "create_project");
        assert_eq!(
            spec.routers[0].routes[0].request_model.as_deref(),
            Some("ProjectRequest")
        );
        assert_eq!(spec.routers[0].routes[0].response_model, "ProjectResponse");
    }

    #[test]
    fn parse_http_method_rejects_unknown_methods() {
        assert_eq!(parse_http_method("GET"), Some(HttpMethod::Get));
        assert_eq!(parse_http_method("trace"), None);
    }
}
// CODEGEN-END
