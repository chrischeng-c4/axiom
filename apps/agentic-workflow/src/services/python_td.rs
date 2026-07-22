//! Syntax-only compiler for the restricted Python TD authoring subset.
//!
//! This is deliberately separate from the Python *emitter* IR. It never
//! imports or runs a project module: tree-sitter is the only parser used here.
//!
//! @spec apps/agentic-workflow/tech-design/logic/aw-python-tech-design-lowering.md#logic

use anyhow::{bail, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};
use tree_sitter::{Node, Parser};
use walkdir::WalkDir;

pub const PYTHON_TD_IR_SCHEMA: &str = "aw.python-td-ir.v1";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonTdIr {
    pub schema_version: &'static str,
    pub project_root: String,
    pub modules: Vec<PythonTdModule>,
    pub semantic_digest: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonTdModule {
    pub id: String,
    /// Explicit DDD artifact identity when the Python TD module declares
    /// `__aw_artifact_id__ = "artifact:<context>/<name>"`. The source path
    /// remains projection metadata and never replaces this identity.
    pub artifact_id: Option<String>,
    pub path: String,
    pub role: PythonTdRole,
    pub imports: Vec<String>,
    pub declarations: Vec<PythonTdDeclaration>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PythonTdRole {
    Domain,
    Application,
    Infrastructure,
    Interface,
    Test,
    ExternalContract,
    Module,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonTdDeclaration {
    pub id: String,
    pub kind: PythonTdDeclarationKind,
    pub name: String,
    pub is_async: bool,
    pub annotations: Vec<String>,
    pub decorators: Vec<String>,
    pub span: PythonTdSpan,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PythonTdDeclarationKind {
    Class,
    Function,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonTdSpan {
    pub line: usize,
    pub column: usize,
    pub byte_start: usize,
    pub byte_end: usize,
}

/// Compile a normal Python project into stable, target-neutral semantic data.
/// `src/` is preferred; fixture tests and EC roots are also included so their
/// native tests and independent contracts remain visible in the inventory.
pub fn compile_python_td_project(project_root: &Path) -> Result<PythonTdIr> {
    let root = project_root
        .canonicalize()
        .with_context(|| format!("canonicalize Python TD root {}", project_root.display()))?;
    let mut files = Vec::new();
    for rel in ["src", "tests", "external-contracts/tests"] {
        let directory = root.join(rel);
        if directory.is_dir() {
            files.extend(collect_python_files(&directory)?);
        }
    }
    if files.is_empty() {
        bail!("Python TD compiler found no .py files below src/, tests/, or external-contracts/tests/ in {}", root.display());
    }
    files.sort();

    let mut modules = files
        .into_iter()
        .map(|path| compile_module(&root, &path))
        .collect::<Result<Vec<_>>>()?;
    modules.sort_by(|left, right| left.id.cmp(&right.id));
    validate_local_imports(&modules)?;
    // Spans are diagnostic provenance, not semantic input. Preserve them in
    // the emitted IR while excluding them from the digest so whitespace-only
    // edits cannot produce a false semantic drift.
    let mut canonical_modules = modules.clone();
    for module in &mut canonical_modules {
        // An explicit artifact identity is the canonical address; paths are
        // projections and must not make a rename look like semantic drift.
        if module.artifact_id.is_some() {
            module.path.clear();
        }
        for declaration in &mut module.declarations {
            declaration.span = PythonTdSpan {
                line: 0,
                column: 0,
                byte_start: 0,
                byte_end: 0,
            };
        }
    }
    let canonical =
        serde_json::to_vec(&canonical_modules).context("serialize canonical Python TD IR")?;
    let semantic_digest = format!("sha256:{:x}", Sha256::digest(canonical));
    Ok(PythonTdIr {
        schema_version: PYTHON_TD_IR_SCHEMA,
        project_root: ".".to_string(),
        modules,
        semantic_digest,
    })
}

fn collect_python_files(root: &Path) -> Result<Vec<PathBuf>> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().is_file()
                && entry.path().extension().and_then(|ext| ext.to_str()) == Some("py")
        })
        .map(|entry| Ok(entry.into_path()))
        .collect()
}

fn compile_module(root: &Path, path: &Path) -> Result<PythonTdModule> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("read Python TD source {}", path.display()))?;
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_python::LANGUAGE.into())?;
    let tree = parser
        .parse(&source, None)
        .context("tree-sitter returned no Python syntax tree")?;
    if tree.root_node().has_error() {
        let error = first_error(tree.root_node()).unwrap_or(tree.root_node());
        return diagnostic(
            path,
            error,
            "syntax-error",
            "repair the Python syntax before compiling the TD inventory",
        );
    }
    if let Some(node) = find_kind(tree.root_node(), "lambda") {
        return diagnostic(
            path,
            node,
            "unsupported-syntax",
            "replace lambda with a named, annotated function in the restricted TD subset",
        );
    }
    let rel = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let artifact_id = explicit_artifact_id(&source, path)?;
    let id = artifact_id.clone().unwrap_or_else(|| {
        format!("module:{}", rel.trim_end_matches(".py").replace('/', "."))
    });
    let mut imports = Vec::new();
    let mut declarations = Vec::new();
    let mut cursor = tree.root_node().walk();
    for child in tree.root_node().named_children(&mut cursor) {
        match child.kind() {
            "import_statement" | "import_from_statement" => {
                imports.push(normalize(node_text(child, &source)))
            }
            "function_definition" | "async_function_definition" => {
                declarations.push(function_declaration(&id, child, &source, Vec::new()))
            }
            "class_definition" => {
                declarations.push(class_declaration(&id, child, &source, Vec::new()))
            }
            "decorated_definition" => {
                let mut decorators = Vec::new();
                let mut inner = child.walk();
                for item in child.named_children(&mut inner) {
                    if item.kind() == "decorator" {
                        decorators.push(normalize(node_text(item, &source)));
                    }
                    if item.kind() == "function_definition"
                        || item.kind() == "async_function_definition"
                    {
                        declarations.push(function_declaration(
                            &id,
                            item,
                            &source,
                            decorators.clone(),
                        ));
                    }
                    if item.kind() == "class_definition" {
                        declarations.push(class_declaration(
                            &id,
                            item,
                            &source,
                            decorators.clone(),
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    imports.sort();
    imports.dedup();
    declarations.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(PythonTdModule {
        id,
        artifact_id,
        path: rel.clone(),
        role: role_for_path(&rel),
        imports,
        declarations,
    })
}

fn explicit_artifact_id(source: &str, path: &Path) -> Result<Option<String>> {
    let values = source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("__aw_artifact_id__"))
        .filter_map(|tail| tail.trim_start().strip_prefix('='))
        .map(str::trim)
        .collect::<Vec<_>>();
    if values.is_empty() {
        return Ok(None);
    }
    if values.len() != 1 {
        bail!(
            "Python TD diagnostic [duplicate-artifact-id] {}: declare __aw_artifact_id__ exactly once per module",
            path.display()
        );
    }
    let value = values[0]
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            values[0]
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .context("Python TD diagnostic [invalid-artifact-id]: __aw_artifact_id__ must be one quoted artifact:<context>/<name> string")?;
    let Some((kind, body)) = value.split_once(':') else {
        bail!(
            "Python TD diagnostic [invalid-artifact-id] {}: `{value}` must be artifact:<context>/<name>",
            path.display()
        );
    };
    let parts = body.split('/').collect::<Vec<_>>();
    if kind != "artifact" || parts.len() != 2 || parts.iter().any(|part| !identity_slug(part)) {
        bail!(
            "Python TD diagnostic [invalid-artifact-id] {}: `{value}` must be artifact:<context>/<name> using lowercase kebab-case segments",
            path.display()
        );
    }
    Ok(Some(value.to_string()))
}

fn identity_slug(value: &&str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes[0].is_ascii_lowercase()
        && bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

fn function_declaration(
    module: &str,
    node: Node<'_>,
    source: &str,
    decorators: Vec<String>,
) -> PythonTdDeclaration {
    let name = node
        .child_by_field_name("name")
        .map(|node| node_text(node, source).to_string())
        .unwrap_or_else(|| "<anonymous>".to_string());
    let mut annotations = Vec::new();
    if let Some(parameters) = node.child_by_field_name("parameters") {
        annotations.push(normalize(node_text(parameters, source)));
    }
    if let Some(return_type) = node.child_by_field_name("return_type") {
        annotations.push(format!(
            "return:{}",
            normalize(node_text(return_type, source))
        ));
    }
    declaration(
        module,
        PythonTdDeclarationKind::Function,
        name,
        node.kind() == "async_function_definition",
        annotations,
        decorators,
        node,
    )
}

fn class_declaration(
    module: &str,
    node: Node<'_>,
    source: &str,
    decorators: Vec<String>,
) -> PythonTdDeclaration {
    let name = node
        .child_by_field_name("name")
        .map(|node| node_text(node, source).to_string())
        .unwrap_or_else(|| "<anonymous>".to_string());
    let mut annotations = Vec::new();
    if let Some(bases) = node.child_by_field_name("superclasses") {
        annotations.push(format!("bases:{}", normalize(node_text(bases, source))));
    }
    declaration(
        module,
        PythonTdDeclarationKind::Class,
        name,
        false,
        annotations,
        decorators,
        node,
    )
}

fn declaration(
    module: &str,
    kind: PythonTdDeclarationKind,
    name: String,
    is_async: bool,
    mut annotations: Vec<String>,
    mut decorators: Vec<String>,
    node: Node<'_>,
) -> PythonTdDeclaration {
    annotations.sort();
    decorators.sort();
    let point = node.start_position();
    PythonTdDeclaration {
        id: format!(
            "{module}:{}:{name}",
            match kind {
                PythonTdDeclarationKind::Class => "class",
                PythonTdDeclarationKind::Function => "function",
            }
        ),
        kind,
        name,
        is_async,
        annotations,
        decorators,
        span: PythonTdSpan {
            line: point.row + 1,
            column: point.column + 1,
            byte_start: node.start_byte(),
            byte_end: node.end_byte(),
        },
    }
}

fn role_for_path(path: &str) -> PythonTdRole {
    if path.contains("external-contracts/tests/") {
        PythonTdRole::ExternalContract
    } else if path.starts_with("tests/") {
        PythonTdRole::Test
    } else if path.contains("/domain/") {
        PythonTdRole::Domain
    } else if path.contains("/application/") {
        PythonTdRole::Application
    } else if path.contains("/infrastructure/") {
        PythonTdRole::Infrastructure
    } else if path.contains("/interface/") {
        PythonTdRole::Interface
    } else {
        PythonTdRole::Module
    }
}

fn validate_local_imports(modules: &[PythonTdModule]) -> Result<()> {
    let package_roots = modules
        .iter()
        .filter_map(|module| module.path.strip_prefix("src/"))
        .filter_map(|path| path.split('/').next())
        .collect::<BTreeSet<_>>();
    let known_modules = modules
        .iter()
        .map(|module| module.id.as_str())
        .collect::<BTreeSet<_>>();
    for module in modules {
        for import in &module.imports {
            let Some(path) = import
                .strip_prefix("from")
                .and_then(|value| value.split("import").next())
            else {
                continue;
            };
            let Some(root) = path.split('.').next() else {
                continue;
            };
            if !package_roots.contains(root) {
                continue;
            }
            let candidate = format!("module:src.{path}");
            let package_init = format!("{candidate}.__init__");
            if !known_modules.contains(candidate.as_str())
                && !known_modules.contains(package_init.as_str())
            {
                bail!(
                    "Python TD diagnostic [unresolved-local-import] {}:1:1: `{path}` is not a project module; add the module or correct the import",
                    module.path
                );
            }
        }
    }
    Ok(())
}
fn node_text<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}
fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}
fn first_error<'tree>(node: Node<'tree>) -> Option<Node<'tree>> {
    if node.kind() == "ERROR" {
        return Some(node);
    }
    let mut cursor = node.walk();
    let result = node.children(&mut cursor).find_map(first_error);
    result
}
fn find_kind<'tree>(node: Node<'tree>, wanted: &str) -> Option<Node<'tree>> {
    if node.kind() == wanted {
        return Some(node);
    }
    let mut cursor = node.walk();
    let result = node
        .children(&mut cursor)
        .find_map(|child| find_kind(child, wanted));
    result
}
fn diagnostic<T>(path: &Path, node: Node<'_>, code: &str, remediation: &str) -> Result<T> {
    let point = node.start_position();
    bail!(
        "Python TD diagnostic [{code}] {}:{}:{}: {remediation}",
        path.display(),
        point.row + 1,
        point.column + 1
    )
}
