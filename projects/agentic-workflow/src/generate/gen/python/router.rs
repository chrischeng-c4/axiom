// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-generate-gen-python.md#schema
// CODEGEN-BEGIN
// Router emitter (R3 of projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md). Hand-written until
// the section→emitter dispatcher can lower `interaction` sections into
// FastAPI router files.
use super::ast::{
    render_python_module, PythonFunctionDef, PythonFunctionStmt, PythonImportStmt, PythonModuleAst,
    PythonModuleStmt,
};
use super::types::{EmittedPythonFile, PythonBodyKind, RouteRecord, RouterIr};

/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#py-emit-router
pub fn emit_router(spec_id: &str, ir: &RouterIr) -> EmittedPythonFile {
    let mut body = Vec::with_capacity(ir.routes.len() + 1);
    body.push(PythonModuleStmt::Assign {
        target: "router".to_string(),
        value: format!("APIRouter(prefix=\"{}\", tags=[\"{}\"])", ir.prefix, ir.tag),
    });
    body.extend(ir.routes.iter().map(route_stmt));

    let module = PythonModuleAst {
        docstring: None,
        imports: router_imports(&ir.routes),
        body,
    };

    EmittedPythonFile {
        path: format!("{}/router.py", ir.name),
        kind: PythonBodyKind::Router,
        ir_source: format!("{}#router:{}", spec_id, ir.name),
        content: render_python_module(&module),
    }
}

fn route_stmt(route: &RouteRecord) -> PythonModuleStmt {
    let decorator = route.method.decorator();
    let args = route
        .request_model
        .as_ref()
        .map(|model| vec![format!("payload: {}", model)])
        .unwrap_or_default();

    PythonModuleStmt::Function(PythonFunctionDef {
        decorators: vec![format!(
            "router.{}(\"{}\", response_model={})",
            decorator, route.path, route.response_model
        )],
        is_async: true,
        name: route.handler_symbol.clone(),
        args,
        returns: Some(route.response_model.clone()),
        body: vec![PythonFunctionStmt::Raise("NotImplementedError".to_string())],
    })
}

fn router_imports(routes: &[RouteRecord]) -> Vec<PythonImportStmt> {
    let mut imports = vec![PythonImportStmt {
        module: "fastapi".to_string(),
        names: vec!["APIRouter".to_string()],
    }];
    for model in collect_models(routes) {
        imports.push(PythonImportStmt {
            module: ".models".to_string(),
            names: vec![model],
        });
    }
    imports.sort_by_key(|import| import_key(import));
    imports.dedup_by_key(|import| import_key(import));
    imports
}

fn import_key(import: &PythonImportStmt) -> String {
    if import.names.is_empty() {
        format!("import {}", import.module)
    } else {
        format!("from {} import {}", import.module, import.names.join(", "))
    }
}

fn collect_models(routes: &[RouteRecord]) -> Vec<String> {
    let mut models: Vec<String> = Vec::new();
    for route in routes {
        if let Some(req) = &route.request_model {
            models.push(req.clone());
        }
        models.push(route.response_model.clone());
    }
    models.sort();
    models.dedup();
    models
}
// CODEGEN-END
