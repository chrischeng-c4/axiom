// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-generate-gen-python.md#schema
// CODEGEN-BEGIN
// Pydantic model emitter (R2 of projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md). Hand-written
// until the schema section emitter can lower `PydanticModelIr` into
// Pydantic v2 BaseModel sources.
use super::ast::{
    pydantic_model_ast, pydantic_module_ast, render_python_module, PythonImportStmt,
    PythonModuleAst, PythonModuleStmt,
};
use super::types::{EmittedPythonFile, PydanticModelIr, PythonBackendSpec, PythonBodyKind};

/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#py-emit-pydantic-model
pub fn emit_pydantic_model(spec_id: &str, ir: &PydanticModelIr) -> EmittedPythonFile {
    let module = PythonModuleAst {
        docstring: None,
        imports: vec![PythonImportStmt {
            module: "pydantic".to_string(),
            names: vec!["BaseModel".to_string()],
        }],
        body: vec![PythonModuleStmt::Class(pydantic_model_ast(ir))],
    };

    EmittedPythonFile {
        path: format!("models/{}.py", snake_case(&ir.name)),
        kind: PythonBodyKind::PydanticModel,
        ir_source: format!("{}#pydantic_model:{}", spec_id, ir.name),
        content: render_python_module(&module),
    }
}

/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#logic
///
/// Render a complete pydantic-model module: module docstring, imports,
/// blank line separator, then one or more classes. Caller supplies the
/// output `path`.
pub fn emit_pydantic_module(
    spec_id: &str,
    spec: &PythonBackendSpec,
    path: &str,
) -> EmittedPythonFile {
    let module = pydantic_module_ast(spec);

    EmittedPythonFile {
        path: path.to_string(),
        kind: PythonBodyKind::PydanticModel,
        ir_source: format!("{}#pydantic_module", spec_id),
        content: render_python_module(&module),
    }
}

fn snake_case(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for (idx, ch) in name.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if idx != 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}
// CODEGEN-END
