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
    fn parse_http_method_rejects_unknown_methods() {
        assert_eq!(parse_http_method("GET"), Some(HttpMethod::Get));
        assert_eq!(parse_http_method("trace"), None);
    }
}
// CODEGEN-END
