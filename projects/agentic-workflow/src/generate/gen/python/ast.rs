// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-generate-gen-python.md#schema
// CODEGEN-BEGIN
//! Small Python codegen AST used by the Python backend emitter.
//!
//! This is an emitter AST, not a parsed CPython AST. It gives TD lowering a
//! stable, language-shaped target before bytes are rendered.

use super::determinism::normalize;
use super::types::{
    ImportIr, PydanticField, PydanticModelIr, PythonBackendSpec, PythonClassItemIr, PythonModuleIr,
    PythonModuleItemIr,
};

/// A complete Python module.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonModuleAst {
    pub docstring: Option<String>,
    pub imports: Vec<PythonImportStmt>,
    pub body: Vec<PythonModuleStmt>,
}

/// `import module` or `from module import Name`.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonImportStmt {
    pub module: String,
    pub names: Vec<String>,
}

/// A Python class definition.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonClassDef {
    pub name: String,
    pub bases: Vec<String>,
    pub docstring: Option<String>,
    pub body: Vec<PythonClassStmt>,
}

/// Top-level module statements currently supported by the shared renderer.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PythonModuleStmt {
    Assign { target: String, value: String },
    Class(PythonClassDef),
    Function(PythonFunctionDef),
    RawBlock(Vec<String>),
}

/// Statements currently needed inside generated classes.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PythonClassStmt {
    AnnotatedAssign {
        target: String,
        annotation: String,
        value: Option<String>,
    },
    Function(PythonFunctionDef),
    RawBlock(Vec<String>),
    Pass,
}

/// A Python function definition.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonFunctionDef {
    pub decorators: Vec<String>,
    pub is_async: bool,
    pub name: String,
    pub args: Vec<String>,
    pub returns: Option<String>,
    pub body: Vec<PythonFunctionStmt>,
}

/// Statements currently needed inside generated functions.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PythonFunctionStmt {
    Raise(String),
    RawLine(String),
    RawBlock(Vec<String>),
    Pass,
}

/// Lower the Pydantic-specific backend IR into the generic Python module AST.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#logic
pub fn pydantic_module_ast(spec: &PythonBackendSpec) -> PythonModuleAst {
    PythonModuleAst {
        docstring: spec.module_docstring.clone(),
        imports: spec.imports.iter().map(import_ast).collect(),
        body: spec
            .pydantic_models
            .iter()
            .map(pydantic_model_ast)
            .map(PythonModuleStmt::Class)
            .collect(),
    }
}

/// Lower one Pydantic model into a Python class AST.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#py-emit-pydantic-model
pub fn pydantic_model_ast(model: &PydanticModelIr) -> PythonClassDef {
    let mut body: Vec<PythonClassStmt> = model.fields.iter().map(field_stmt).collect();
    if body.is_empty() && model.docstring.is_none() {
        body.push(PythonClassStmt::Pass);
    }

    PythonClassDef {
        name: model.name.clone(),
        bases: vec![model.base.clone()],
        docstring: model.docstring.clone(),
        body,
    }
}

/// Lower a TD-described Python module into the generic Python module AST.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
pub fn python_module_ir_ast(module: &PythonModuleIr) -> PythonModuleAst {
    PythonModuleAst {
        docstring: module.docstring.clone(),
        imports: module.imports.iter().map(import_ast).collect(),
        body: module.body.iter().map(module_item_stmt).collect(),
    }
}

/// Build a TD-storable Python module IR from an already-adopted source file.
///
/// The first version preserves the source as a module-level raw AST node after
/// removing Score ownership wrappers. More specific lowerers can replace raw
/// nodes with class/function nodes without changing the TD schema contract.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#schema
pub fn python_module_ir_from_source(path: &str, source: &str) -> PythonModuleIr {
    let body = strip_score_handwrite_envelope(source);
    PythonModuleIr {
        path: path.to_string(),
        docstring: None,
        imports: Vec::new(),
        body: vec![PythonModuleItemIr::Raw {
            lines: body.lines().map(|line| line.to_string()).collect(),
        }],
    }
}

fn import_ast(import: &ImportIr) -> PythonImportStmt {
    PythonImportStmt {
        module: import.module.clone(),
        names: import.names.clone(),
    }
}

fn module_item_stmt(item: &PythonModuleItemIr) -> PythonModuleStmt {
    match item {
        PythonModuleItemIr::Raw { lines } => PythonModuleStmt::RawBlock(lines.clone()),
        PythonModuleItemIr::Assign { target, value } => PythonModuleStmt::Assign {
            target: target.clone(),
            value: value.clone(),
        },
        PythonModuleItemIr::Class {
            name,
            bases,
            docstring,
            body,
        } => PythonModuleStmt::Class(PythonClassDef {
            name: name.clone(),
            bases: bases.clone(),
            docstring: docstring.clone(),
            body: body.iter().map(class_item_stmt).collect(),
        }),
        PythonModuleItemIr::Function {
            decorators,
            is_async,
            name,
            args,
            returns,
            body,
        } => PythonModuleStmt::Function(function_def(
            decorators.clone(),
            *is_async,
            name.clone(),
            args.clone(),
            returns.clone(),
            body.clone(),
        )),
    }
}

fn class_item_stmt(item: &PythonClassItemIr) -> PythonClassStmt {
    match item {
        PythonClassItemIr::Raw { lines } => PythonClassStmt::RawBlock(lines.clone()),
        PythonClassItemIr::AnnotatedAssign {
            target,
            annotation,
            value,
        } => PythonClassStmt::AnnotatedAssign {
            target: target.clone(),
            annotation: annotation.clone(),
            value: value.clone(),
        },
        PythonClassItemIr::Function {
            decorators,
            is_async,
            name,
            args,
            returns,
            body,
        } => PythonClassStmt::Function(function_def(
            decorators.clone(),
            *is_async,
            name.clone(),
            args.clone(),
            returns.clone(),
            body.clone(),
        )),
        PythonClassItemIr::Pass => PythonClassStmt::Pass,
    }
}

fn function_def(
    decorators: Vec<String>,
    is_async: bool,
    name: String,
    args: Vec<String>,
    returns: Option<String>,
    body: Vec<String>,
) -> PythonFunctionDef {
    PythonFunctionDef {
        decorators,
        is_async,
        name,
        args,
        returns,
        body: if body.is_empty() {
            Vec::new()
        } else {
            vec![PythonFunctionStmt::RawBlock(body)]
        },
    }
}

fn field_stmt(field: &PydanticField) -> PythonClassStmt {
    PythonClassStmt::AnnotatedAssign {
        target: field.name.clone(),
        annotation: field.py_type.clone(),
        value: field.default.clone(),
    }
}

/// Render a Python module AST to normalized source.
/// @spec projects/agentic-workflow/tech-design/core/specs/python-backend-emitter.md#py-normalize
pub fn render_python_module(module: &PythonModuleAst) -> String {
    let mut body = String::new();

    if let Some(doc) = &module.docstring {
        body.push_str("\"\"\"");
        body.push_str(doc);
        body.push_str("\"\"\"\n");
    }

    if !module.imports.is_empty() {
        let mut prev_kind: Option<u8> = None;
        for import in &module.imports {
            let kind = import_kind(&import.module);
            if let Some(prev) = prev_kind {
                if prev != kind {
                    body.push('\n');
                }
            }
            body.push_str(&render_import(import));
            body.push('\n');
            prev_kind = Some(kind);
        }
    }

    for (idx, stmt) in module.body.iter().enumerate() {
        if idx == 0 && (!module.imports.is_empty() || module.docstring.is_some()) {
            body.push_str("\n\n");
        } else if idx > 0 {
            body.push_str("\n\n");
        }
        render_module_stmt(&mut body, stmt);
    }

    normalize(&body)
}

fn render_import(import: &PythonImportStmt) -> String {
    if import.names.is_empty() {
        format!("import {}", import.module)
    } else {
        format!("from {} import {}", import.module, import.names.join(", "))
    }
}

fn import_kind(module: &str) -> u8 {
    if module == "src" || module.starts_with("src.") {
        2
    } else {
        1
    }
}

fn render_module_stmt(body: &mut String, stmt: &PythonModuleStmt) {
    match stmt {
        PythonModuleStmt::Assign { target, value } => {
            body.push_str(&format!("{} = {}\n", target, value));
        }
        PythonModuleStmt::Class(class) => render_class(body, class),
        PythonModuleStmt::Function(func) => render_function(body, func),
        PythonModuleStmt::RawBlock(lines) => render_raw_lines(body, "", lines),
    }
}

fn render_class(body: &mut String, class: &PythonClassDef) {
    let bases = if class.bases.is_empty() {
        String::new()
    } else {
        format!("({})", class.bases.join(", "))
    };
    body.push_str(&format!("class {}{}:\n", class.name, bases));

    if let Some(doc) = &class.docstring {
        body.push_str("    \"\"\"");
        let mut first = true;
        for line in doc.split('\n') {
            if first {
                body.push_str(line);
                first = false;
            } else {
                body.push('\n');
                if !line.is_empty() {
                    body.push_str("    ");
                    body.push_str(line);
                }
            }
        }
        body.push_str("\n    \"\"\"\n");
    }

    for stmt in &class.body {
        match stmt {
            PythonClassStmt::AnnotatedAssign {
                target,
                annotation,
                value,
            } => {
                if let Some(value) = value {
                    body.push_str(&format!("    {}: {} = {}\n", target, annotation, value));
                } else {
                    body.push_str(&format!("    {}: {}\n", target, annotation));
                }
            }
            PythonClassStmt::Function(func) => render_function_with_indent(body, func, "    "),
            PythonClassStmt::RawBlock(lines) => render_raw_lines(body, "    ", lines),
            PythonClassStmt::Pass => body.push_str("    pass\n"),
        }
    }
}

fn render_function(body: &mut String, func: &PythonFunctionDef) {
    render_function_with_indent(body, func, "");
}

fn render_function_with_indent(body: &mut String, func: &PythonFunctionDef, indent: &str) {
    for decorator in &func.decorators {
        body.push_str(indent);
        body.push_str(&format!("@{}\n", decorator));
    }
    let async_prefix = if func.is_async { "async " } else { "" };
    let returns = func
        .returns
        .as_ref()
        .map(|value| format!(" -> {}", value))
        .unwrap_or_default();
    body.push_str(indent);
    body.push_str(&format!(
        "{}def {}({}){}:\n",
        async_prefix,
        func.name,
        func.args.join(", "),
        returns
    ));
    if func.body.is_empty() {
        body.push_str(indent);
        body.push_str("    pass\n");
        return;
    }
    for stmt in &func.body {
        match stmt {
            PythonFunctionStmt::Raise(expr) => {
                body.push_str(indent);
                body.push_str(&format!("    raise {}\n", expr));
            }
            PythonFunctionStmt::RawLine(line) => {
                body.push_str(indent);
                body.push_str("    ");
                body.push_str(line);
                body.push('\n');
            }
            PythonFunctionStmt::RawBlock(lines) => {
                let block_indent = format!("{indent}    ");
                render_raw_lines(body, &block_indent, lines);
            }
            PythonFunctionStmt::Pass => {
                body.push_str(indent);
                body.push_str("    pass\n");
            }
        }
    }
}

fn render_raw_lines(body: &mut String, indent: &str, lines: &[String]) {
    for line in lines {
        if !line.is_empty() {
            body.push_str(indent);
            body.push_str(line);
        }
        body.push('\n');
    }
}

fn strip_score_handwrite_envelope(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    let start = lines
        .iter()
        .position(|line| {
            let body = strip_comment_lead(line.trim_start());
            body.starts_with("<HANDWRITE") || body.starts_with("HANDWRITE-BEGIN")
        })
        .unwrap_or(usize::MAX);
    let end = lines
        .iter()
        .rposition(|line| {
            let body = strip_comment_lead(line.trim_start());
            body.starts_with("</HANDWRITE") || body.starts_with("HANDWRITE-END")
        })
        .unwrap_or(usize::MAX);

    if start == usize::MAX || end == usize::MAX || start >= end {
        return normalize(source);
    }

    let mut out = Vec::new();
    out.extend(lines[..start].iter().copied());
    out.extend(lines[(start + 1)..end].iter().copied());
    out.extend(lines[(end + 1)..].iter().copied());
    normalize(&out.join("\n"))
}

fn strip_comment_lead(line: &str) -> &str {
    let s = line.trim_start();
    if let Some(rest) = s.strip_prefix("///") {
        return rest.trim_start();
    }
    if let Some(rest) = s.strip_prefix("//!") {
        return rest.trim_start();
    }
    if let Some(rest) = s.strip_prefix("//") {
        return rest.trim_start();
    }
    if let Some(rest) = s.strip_prefix('#') {
        return rest.trim_start();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::gen::python::{
        ImportIr, PydanticField, PydanticModelIr, PythonBackendSpec,
    };

    #[test]
    fn pydantic_module_ast_renders_imports_classes_and_fields() {
        let spec = PythonBackendSpec {
            spec_id: "fixture".to_string(),
            routers: Vec::new(),
            python_modules: Vec::new(),
            imports: vec![ImportIr {
                module: "pydantic".to_string(),
                names: Vec::new(),
            }],
            module_docstring: Some("Widget API models.\n".to_string()),
            pydantic_models: vec![PydanticModelIr {
                name: "WidgetRequest".to_string(),
                base: "pydantic.BaseModel".to_string(),
                docstring: Some("WidgetRequest.".to_string()),
                fields: vec![
                    PydanticField {
                        name: "name".to_string(),
                        py_type: "str".to_string(),
                        default: None,
                    },
                    PydanticField {
                        name: "active".to_string(),
                        py_type: "bool".to_string(),
                        default: Some("True".to_string()),
                    },
                ],
            }],
        };

        let rendered = render_python_module(&pydantic_module_ast(&spec));

        assert!(rendered.starts_with("\"\"\"Widget API models."));
        assert!(rendered.contains("import pydantic"));
        assert!(rendered.contains("class WidgetRequest(pydantic.BaseModel):"));
        assert!(rendered.contains("    \"\"\"WidgetRequest.\n    \"\"\"\n"));
        assert!(rendered.contains("    active: bool = True"));
    }

    #[test]
    fn python_module_ir_ast_renders_class_methods_and_raw_body() {
        let module = PythonModuleIr {
            path: "migrations/015.py".to_string(),
            docstring: Some("Migration docs.".to_string()),
            imports: vec![ImportIr {
                module: "beanie".to_string(),
                names: vec!["free_fall_migration".to_string()],
            }],
            body: vec![PythonModuleItemIr::Class {
                name: "Forward".to_string(),
                bases: Vec::new(),
                docstring: None,
                body: vec![PythonClassItemIr::Function {
                    decorators: vec!["free_fall_migration(document_models=[])".to_string()],
                    is_async: true,
                    name: "run".to_string(),
                    args: vec!["self".to_string(), "session".to_string()],
                    returns: None,
                    body: vec![
                        "\"\"\"Run migration.\"\"\"".to_string(),
                        "".to_string(),
                        "value = 1".to_string(),
                        "if value:".to_string(),
                        "    print(value)".to_string(),
                    ],
                }],
            }],
        };

        let rendered = render_python_module(&python_module_ir_ast(&module));

        assert!(rendered.contains("from beanie import free_fall_migration"));
        assert!(rendered.contains("class Forward:"));
        assert!(rendered.contains("    @free_fall_migration(document_models=[])"));
        assert!(rendered.contains("    async def run(self, session):"));
        assert!(rendered.contains("        if value:\n            print(value)"));
    }

    #[test]
    fn python_module_ir_from_source_strips_score_handwrite_envelope() {
        let source = "# <HANDWRITE gap=\"g\" tracker=\"t\" reason=\"r\">\n\
\"\"\"Module docs.\"\"\"\n\n\
def handle():\n\
    return 1\n\
# </HANDWRITE>\n";

        let module = python_module_ir_from_source("src/app.py", source);
        let rendered = render_python_module(&python_module_ir_ast(&module));

        assert!(rendered.starts_with("\"\"\"Module docs.\"\"\""));
        assert!(rendered.contains("def handle():"));
        assert!(rendered.contains("return 1"));
        assert!(!rendered.contains("HANDWRITE"));
    }
}
// CODEGEN-END
