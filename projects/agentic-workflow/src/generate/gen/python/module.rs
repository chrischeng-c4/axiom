// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-generate-gen-python.md#schema
// CODEGEN-BEGIN
//! Generic Python module emitter.
//!
//! This path is intentionally separate from the Pydantic and router emitters:
//! semantic standardization can describe an existing Python module as a TD
//! schema payload, lower it into the shared Python AST, and render a complete
//! module without falling back to Rust-oriented marker stubs.

use super::ast::{python_module_ir_ast, render_python_module};
use super::types::{EmittedPythonFile, PythonBodyKind, PythonModuleIr};

/// Render a TD-described Python module into source text.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
pub fn emit_python_module(spec_id: &str, ir: &PythonModuleIr) -> EmittedPythonFile {
    let module = python_module_ir_ast(ir);

    EmittedPythonFile {
        path: ir.path.clone(),
        kind: PythonBodyKind::Module,
        ir_source: format!("{}#python_module:{}", spec_id, ir.path),
        content: render_python_module(&module),
    }
}
// CODEGEN-END
