// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-generate-gen-python.md#schema
// CODEGEN-BEGIN
// Python backend emitter module root (R3 of projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md).
// Hand-written until the codegen pipeline can scaffold module roots for
// new emitter backends.
pub mod ast;
pub mod determinism;
pub mod lower;
pub mod module;
pub mod pydantic_model;
pub mod router;
pub mod types;

pub use ast::{
    pydantic_model_ast, pydantic_module_ast, python_module_ir_ast, python_module_ir_from_source,
    render_python_module, PythonClassDef, PythonClassStmt, PythonFunctionDef, PythonFunctionStmt,
    PythonImportStmt, PythonModuleAst, PythonModuleStmt,
};
pub use lower::{
    lower_backend_spec_value, lower_backend_spec_yaml, lower_openapi_payload, parse_http_method,
};
pub use module::emit_python_module;
pub use pydantic_model::{emit_pydantic_model, emit_pydantic_module};
pub use router::emit_router;
pub use types::{
    EmittedPythonFile, HttpMethod, ImportIr, PydanticField, PydanticModelIr, PythonBackendSpec,
    PythonBodyKind, PythonClassItemIr, PythonModuleIr, PythonModuleItemIr, RouteRecord, RouterIr,
};
// CODEGEN-END
