// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-generate-gen-python.md#schema
// CODEGEN-BEGIN
// Closed-shape IR for the Python backend emitter (R1 of
// projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md). Hand-written until the schema-to-Rust
// type generator can emit Pydantic-flavoured records under
// `projects/agentic-workflow/src/generate/gen/python/`.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
use serde::{Deserialize, Serialize};

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-generate-gen-python.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PythonBackendSpec {
    pub spec_id: String,
    pub routers: Vec<RouterIr>,
    pub pydantic_models: Vec<PydanticModelIr>,
    /// Whole Python modules lowered from a TD schema section into the generic
    /// Python AST emitter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub python_modules: Vec<PythonModuleIr>,
    /// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
    pub imports: Vec<ImportIr>,
    /// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
    pub module_docstring: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouterIr {
    pub name: String,
    pub prefix: String,
    pub tag: String,
    pub routes: Vec<RouteRecord>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteRecord {
    pub method: HttpMethod,
    pub path: String,
    pub handler_symbol: String,
    pub request_model: Option<String>,
    pub response_model: String,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

/// @spec projects/agentic-workflow/tech-design/semantic/agentic-workflow-generate-gen-python.md#schema
impl HttpMethod {
    pub fn decorator(self) -> &'static str {
        match self {
            HttpMethod::Get => "get",
            HttpMethod::Post => "post",
            HttpMethod::Put => "put",
            HttpMethod::Delete => "delete",
            HttpMethod::Patch => "patch",
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PydanticModelIr {
    pub name: String,
    pub base: String,
    pub fields: Vec<PydanticField>,
    /// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
    pub docstring: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PydanticField {
    pub name: String,
    pub py_type: String,
    pub default: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportIr {
    pub module: String,
    pub names: Vec<String>,
}

/// A complete Python module described in TD schema YAML.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PythonModuleIr {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docstring: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<ImportIr>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub body: Vec<PythonModuleItemIr>,
}

/// Module-level Python statements supported by the TD-to-Python AST bridge.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PythonModuleItemIr {
    Raw {
        #[serde(default)]
        lines: Vec<String>,
    },
    Assign {
        target: String,
        value: String,
    },
    Class {
        name: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        bases: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        docstring: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        body: Vec<PythonClassItemIr>,
    },
    Function {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        decorators: Vec<String>,
        #[serde(default)]
        is_async: bool,
        name: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        returns: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        body: Vec<String>,
    },
}

/// Class-body statements supported by the TD-to-Python AST bridge.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PythonClassItemIr {
    Raw {
        #[serde(default)]
        lines: Vec<String>,
    },
    AnnotatedAssign {
        target: String,
        annotation: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        value: Option<String>,
    },
    Function {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        decorators: Vec<String>,
        #[serde(default)]
        is_async: bool,
        name: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        returns: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        body: Vec<String>,
    },
    Pass,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmittedPythonFile {
    pub path: String,
    pub kind: PythonBodyKind,
    pub ir_source: String,
    pub content: String,
}

/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonBodyKind {
    Router,
    PydanticModel,
    Module,
}
// CODEGEN-END
