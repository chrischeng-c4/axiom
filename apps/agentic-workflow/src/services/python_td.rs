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
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};
use tree_sitter::{Node, Parser};
use walkdir::WalkDir;

pub const PYTHON_TD_IR_SCHEMA: &str = "aw.python-td-ir.v1";
pub(crate) const NATIVE_TARGET_OWNER: &str = "aw.python-td-native-target.v1";

/// Atomically materialize one target-native package into an owned root.
///
/// Every candidate carries [`NATIVE_TARGET_OWNER`]. Existing files may change
/// only when their bytes already carry the same owner; byte-identical files
/// are also admitted. The complete collision set is checked before staging,
/// then the existing tree is copied into a sibling stage and swapped with a
/// rollback directory so unrelated files survive without partial mutation.
pub(crate) fn materialize_owned_target(
    output_root: &Path,
    target: &str,
    files: &BTreeMap<String, String>,
) -> Result<()> {
    if files.is_empty() {
        bail!("{target} target candidate write set is empty");
    }
    for relative in files.keys() {
        let path = Path::new(relative);
        if relative.is_empty()
            || path.is_absolute()
            || path.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            bail!("unsafe {target} target candidate path `{relative}`");
        }
        if !files[relative].contains(NATIVE_TARGET_OWNER) {
            bail!(
                "{target} target candidate `{relative}` is missing ownership sentinel `{NATIVE_TARGET_OWNER}`"
            );
        }
    }

    let output_existed = match fs::symlink_metadata(output_root) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                bail!(
                    "refusing to materialize greenfield {target} target at `{}`: output root must be a new or empty directory",
                    output_root.display()
                );
            }
            let mut collisions = Vec::new();
            for (relative, candidate) in files {
                let existing_path = output_root.join(relative);
                match fs::symlink_metadata(&existing_path) {
                    Ok(existing_metadata)
                        if existing_metadata.file_type().is_symlink()
                            || !existing_metadata.is_file() =>
                    {
                        collisions.push(relative.clone());
                    }
                    Ok(_) => {
                        let existing = fs::read_to_string(&existing_path).with_context(|| {
                            format!("read existing target {}", existing_path.display())
                        })?;
                        if existing != *candidate && !existing.contains(NATIVE_TARGET_OWNER) {
                            collisions.push(relative.clone());
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => {
                        return Err(error).with_context(|| {
                            format!("inspect existing target {}", existing_path.display())
                        });
                    }
                }
            }
            if !collisions.is_empty() {
                let candidates = files.keys().cloned().collect::<Vec<_>>().join(", ");
                bail!(
                    "refusing to overwrite unowned {target} target files in existing project `{}`; collisions: {}; candidate write set: {candidates}. Existing-project materialization requires byte-identical content or ownership sentinel `{NATIVE_TARGET_OWNER}`; otherwise use a bounded HANDWRITE or patch workflow",
                    output_root.display(),
                    collisions.join(", ")
                );
            }
            true
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => {
            return Err(error)
                .with_context(|| format!("inspect output root {}", output_root.display()));
        }
    };

    let parent = output_root
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .with_context(|| format!("create target parent {}", parent.display()))?;
    let staging = tempfile::Builder::new()
        .prefix(".aw-python-td-target-")
        .tempdir_in(parent)
        .with_context(|| format!("create {target} target staging directory"))?;
    if output_existed {
        copy_owned_target_tree(output_root, staging.path())?;
    }
    for (relative, content) in files {
        let staged = staging.path().join(relative);
        fs::create_dir_all(staged.parent().expect("validated candidate has parent"))
            .with_context(|| format!("create staged parent for {relative}"))?;
        fs::write(&staged, content.as_bytes())
            .with_context(|| format!("write staged {target} target {relative}"))?;
    }

    if output_existed {
        let backup = tempfile::Builder::new()
            .prefix(".aw-python-td-backup-")
            .tempdir_in(parent)
            .with_context(|| format!("create {target} target rollback directory"))?;
        let backup_path = backup.path().to_path_buf();
        fs::remove_dir(&backup_path)
            .with_context(|| format!("prepare rollback path {}", backup_path.display()))?;
        fs::rename(output_root, &backup_path).with_context(|| {
            format!(
                "move existing {target} target {} to rollback path",
                output_root.display()
            )
        })?;
        if let Err(error) = fs::rename(staging.path(), output_root) {
            let rollback = fs::rename(&backup_path, output_root);
            return match rollback {
                Ok(()) => Err(error).with_context(|| {
                    format!(
                        "atomically install staged {target} target at {}",
                        output_root.display()
                    )
                }),
                Err(rollback_error) => bail!(
                    "failed to install staged {target} target at {} ({error}); rollback from {} also failed ({rollback_error})",
                    output_root.display(),
                    backup_path.display()
                ),
            };
        }
        fs::remove_dir_all(&backup_path)
            .with_context(|| format!("remove {target} target rollback directory"))?;
    } else if let Err(error) = fs::rename(staging.path(), output_root) {
        return Err(error).with_context(|| {
            format!(
                "atomically install staged {target} target at {}",
                output_root.display()
            )
        });
    }
    Ok(())
}

fn copy_owned_target_tree(source: &Path, destination: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(source).min_depth(1) {
        let entry = entry.with_context(|| format!("walk existing target {}", source.display()))?;
        let relative = entry
            .path()
            .strip_prefix(source)
            .expect("walk entry is below source");
        let target = destination.join(relative);
        if entry.file_type().is_symlink() {
            bail!(
                "refusing preservation-aware target update with symlink `{}`",
                entry.path().display()
            );
        }
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)
                .with_context(|| format!("copy target directory {}", target.display()))?;
        } else if entry.file_type().is_file() {
            fs::create_dir_all(target.parent().expect("copied file has parent"))?;
            fs::copy(entry.path(), &target).with_context(|| {
                format!("copy existing target {} to staging", entry.path().display())
            })?;
        } else {
            bail!(
                "refusing preservation-aware target update with special file `{}`",
                entry.path().display()
            );
        }
    }
    Ok(())
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codegen: Option<PythonTdCodegen>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum PythonTdCodegen {
    OpenApi(PythonTdOpenApiCodegen),
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonTdOpenApiCodegen {
    pub document_path: String,
    pub document: String,
    pub client_name: String,
    pub python_target: String,
    pub typescript_target: String,
    pub rust_target: String,
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
    validate_unique_artifact_ids(&modules)?;
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

/// @spec apps/agentic-workflow/tech-design/src/agentic_workflow/health/python_td_global_artifact_identity.py
fn validate_unique_artifact_ids(modules: &[PythonTdModule]) -> Result<()> {
    let mut paths_by_id = BTreeMap::<&str, Vec<&str>>::new();
    for module in modules {
        if let Some(artifact_id) = module.artifact_id.as_deref() {
            paths_by_id
                .entry(artifact_id)
                .or_default()
                .push(module.path.as_str());
        }
    }
    let mut collisions = paths_by_id
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(artifact_id, mut paths)| {
            paths.sort_unstable();
            format!("`{artifact_id}`: {}", paths.join(", "))
        })
        .collect::<Vec<_>>();
    collisions.sort();
    if !collisions.is_empty() {
        bail!(
            "Python TD diagnostic [duplicate-project-artifact-id]: every __aw_artifact_id__ must be globally unique; conflicts: {}",
            collisions.join("; ")
        );
    }
    Ok(())
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
    let id = artifact_id
        .clone()
        .unwrap_or_else(|| format!("module:{}", rel.trim_end_matches(".py").replace('/', ".")));
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
    let codegen = compile_codegen_contract(root, path, &declarations)?;
    Ok(PythonTdModule {
        id,
        artifact_id,
        path: rel.clone(),
        role: role_for_path(&rel),
        imports,
        declarations,
        codegen,
    })
}

fn compile_codegen_contract(
    root: &Path,
    module_path: &Path,
    declarations: &[PythonTdDeclaration],
) -> Result<Option<PythonTdCodegen>> {
    let candidates = declarations
        .iter()
        .flat_map(|declaration| {
            declaration
                .decorators
                .iter()
                .filter(|decorator| decorator.starts_with("@openapi_client("))
                .map(move |decorator| (declaration, decorator.as_str()))
        })
        .collect::<Vec<_>>();
    let Some((declaration, decorator)) = candidates.first().copied() else {
        return Ok(None);
    };
    if candidates.len() != 1 {
        bail!(
            "Python TD diagnostic [duplicate-openapi-codegen] {}: declare exactly one @openapi_client contract per module",
            module_path.display()
        );
    }
    if declaration.kind != PythonTdDeclarationKind::Class {
        bail!(
            "Python TD diagnostic [invalid-openapi-codegen] {}: @openapi_client must decorate one class",
            module_path.display()
        );
    }
    let body = decorator
        .strip_prefix("@openapi_client(")
        .and_then(|value| value.strip_suffix(')'))
        .with_context(|| {
            format!(
                "Python TD diagnostic [invalid-openapi-codegen] {}: malformed @openapi_client decorator",
                module_path.display()
            )
        })?;
    let mut values = BTreeMap::new();
    for raw in body
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let (key, value) = raw.split_once('=').with_context(|| {
            format!(
                "Python TD diagnostic [invalid-openapi-codegen] {}: every @openapi_client argument must be key=\"value\"",
                module_path.display()
            )
        })?;
        if !matches!(key, "source" | "python" | "typescript" | "rust") {
            bail!(
                "Python TD diagnostic [invalid-openapi-codegen] {}: unknown @openapi_client argument `{key}`",
                module_path.display()
            );
        }
        let value = quoted_literal(value).with_context(|| {
            format!(
                "Python TD diagnostic [invalid-openapi-codegen] {}: @openapi_client `{key}` must be one quoted string",
                module_path.display()
            )
        })?;
        if values.insert(key, value).is_some() {
            bail!(
                "Python TD diagnostic [invalid-openapi-codegen] {}: duplicate @openapi_client argument `{key}`",
                module_path.display()
            );
        }
    }
    let required = |key| {
        values.get(key).copied().with_context(|| {
            format!(
                "Python TD diagnostic [invalid-openapi-codegen] {}: @openapi_client is missing `{key}`",
                module_path.display()
            )
        })
    };
    let document_path = required("source")?;
    let source_path = Path::new(document_path);
    if source_path.is_absolute()
        || source_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        bail!(
            "Python TD diagnostic [invalid-openapi-source] {}: OpenAPI source must stay below the TD root",
            module_path.display()
        );
    }
    let canonical_root = root
        .canonicalize()
        .with_context(|| format!("resolve Python TD root {}", root.display()))?;
    let resolved = canonical_root
        .join(source_path)
        .canonicalize()
        .with_context(|| {
            format!(
                "Python TD diagnostic [missing-openapi-source] {}: cannot resolve `{document_path}`",
                module_path.display()
            )
        })?;
    if !resolved.starts_with(&canonical_root) {
        bail!(
            "Python TD diagnostic [invalid-openapi-source] {}: OpenAPI source escapes the TD root",
            module_path.display()
        );
    }
    let document = fs::read_to_string(&resolved).with_context(|| {
        format!(
            "Python TD diagnostic [missing-openapi-source] {}: read `{document_path}`",
            module_path.display()
        )
    })?;
    let json: serde_json::Value = serde_json::from_str(&document).with_context(|| {
        format!(
            "Python TD diagnostic [invalid-openapi-source] {}: `{document_path}` must be JSON",
            module_path.display()
        )
    })?;
    if json
        .get("openapi")
        .and_then(serde_json::Value::as_str)
        .is_none()
    {
        bail!(
            "Python TD diagnostic [invalid-openapi-source] {}: `{document_path}` is missing an OpenAPI version",
            module_path.display()
        );
    }
    let document = serde_json::to_string(&json).context("normalize OpenAPI JSON document")?;
    let python_target =
        validate_openapi_target(required("python")?, openapi_codegen::Lang::Py, module_path)?;
    let typescript_target = validate_openapi_target(
        required("typescript")?,
        openapi_codegen::Lang::Ts,
        module_path,
    )?;
    let rust_target =
        validate_openapi_target(required("rust")?, openapi_codegen::Lang::Rust, module_path)?;
    Ok(Some(PythonTdCodegen::OpenApi(PythonTdOpenApiCodegen {
        document_path: document_path.to_string(),
        document,
        client_name: declaration.name.clone(),
        python_target,
        typescript_target,
        rust_target,
    })))
}

fn quoted_literal(value: &str) -> Option<&str> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
}

fn validate_openapi_target(
    value: &str,
    expected: openapi_codegen::Lang,
    module_path: &Path,
) -> Result<String> {
    let target = openapi_codegen::TargetProfile::from_id(value).with_context(|| {
        format!(
            "Python TD diagnostic [invalid-openapi-target] {}: invalid target `{value}`",
            module_path.display()
        )
    })?;
    if target.lang() != expected {
        bail!(
            "Python TD diagnostic [invalid-openapi-target] {}: target `{value}` does not match {:?}",
            module_path.display(),
            expected
        );
    }
    Ok(value.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_project_artifact_id_reports_every_conflicting_path() {
        let root = tempfile::tempdir().unwrap();
        let src = root.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("z_projection.py"),
            "__aw_artifact_id__ = \"artifact:health/contract\"\n\ndef projection() -> str:\n    return \"z\"\n",
        )
        .unwrap();
        fs::write(
            src.join("a_design.py"),
            "__aw_artifact_id__ = \"artifact:health/contract\"\n\ndef design() -> str:\n    return \"a\"\n",
        )
        .unwrap();

        let error = compile_python_td_project(root.path())
            .expect_err("duplicate global artifact identity must fail")
            .to_string();

        assert!(error.contains("[duplicate-project-artifact-id]"), "{error}");
        assert!(error.contains("artifact:health/contract"), "{error}");
        let a = error.find("src/a_design.py").expect("first path");
        let z = error.find("src/z_projection.py").expect("second path");
        assert!(a < z, "diagnostic paths must be sorted: {error}");
    }
}
