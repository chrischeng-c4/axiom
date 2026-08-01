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
    /// True only when the module explicitly declares
    /// `__aw_public_contract__ = True`. Public artifacts must have a matching
    /// EC identity; internal implementation artifacts remain TD-only.
    pub public_contract: bool,
    /// Optional work-item binding for an explicitly declared native HANDWRITE
    /// target. The binding stays separate from the artifact identity because a
    /// release/acceptance harness can be a bounded implementation witness
    /// without being a DDD artifact itself.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_item: Option<String>,
    /// Repository-relative native HANDWRITE paths owned by `work_item`.
    /// These are explicit TD declarations, not inferred from the target
    /// project's language or a whole-worktree marker scan.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub native_handwrite_targets: Vec<String>,
    /// Executable public behavior identities derived from top-level Python
    /// functions. Python uses snake_case while EC use_case_id uses kebab-case.
    /// Internal modules always project an empty list.
    pub public_behaviors: Vec<String>,
    pub path: String,
    pub role: PythonTdRole,
    pub imports: Vec<String>,
    pub declarations: Vec<PythonTdDeclaration>,
    /// Target-neutral CLI grammar compiled from the constrained Typer
    /// authoring subset. Compilation is syntax-only and never imports Typer or
    /// executes module registration code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli: Option<PythonTdCli>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codegen: Option<PythonTdCodegen>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonTdCli {
    pub framework: &'static str,
    pub app_symbol: String,
    pub configuration: String,
    pub commands: Vec<PythonTdCliCommand>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonTdCliCommand {
    pub function: String,
    pub name: String,
    pub kind: PythonTdCliCommandKind,
    pub configuration: String,
    pub parameters: Vec<PythonTdCliParameter>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PythonTdCliCommandKind {
    Callback,
    Command,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonTdCliParameter {
    pub name: String,
    pub kind: PythonTdCliParameterKind,
    pub annotation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    pub binding: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PythonTdCliParameterKind {
    Argument,
    Option,
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
    /// Digest of the executable class/function body. Python TD is a runnable
    /// reference implementation, so behavior changes are semantic changes
    /// even when the public signature stays fixed.
    pub implementation_digest: String,
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
    validate_unique_native_handwrite_targets(&modules)?;
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
    let public_contract = explicit_public_contract(&source, path)?;
    let work_item = explicit_work_item(tree.root_node(), &source, path)?;
    let native_handwrite_targets =
        explicit_native_handwrite_targets(tree.root_node(), &source, path)?;
    if !native_handwrite_targets.is_empty() && work_item.is_none() {
        bail!(
            "Python TD diagnostic [native-handwrite-without-work-item] {}: __aw_native_handwrite_targets__ requires one quoted __aw_work_item__ binding",
            path.display()
        );
    }
    if public_contract && artifact_id.is_none() {
        bail!(
            "Python TD diagnostic [public-contract-without-artifact-id] {}: __aw_public_contract__ = True requires one explicit __aw_artifact_id__",
            path.display()
        );
    }
    let id = artifact_id
        .clone()
        .unwrap_or_else(|| module_id_for_path(&rel));
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
                        decorators.push(normalize_python_expression(node_text(item, &source)));
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
    let public_behaviors = if public_contract {
        let declared_functions = declarations
            .iter()
            .filter(|declaration| {
                declaration.kind == PythonTdDeclarationKind::Function
                    && !declaration.name.starts_with('_')
            })
            .map(|declaration| declaration.name.as_str())
            .collect::<BTreeSet<_>>();
        let explicit =
            explicit_public_behaviors(tree.root_node(), &source, path)?.unwrap_or_default();
        let behaviors = if explicit.is_empty() {
            declared_functions
                .iter()
                .map(|name| name.replace('_', "-"))
                .collect::<Vec<_>>()
        } else {
            let missing = explicit
                .iter()
                .filter(|name| !declared_functions.contains(name.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            if !missing.is_empty() {
                bail!(
                    "Python TD diagnostic [unknown-public-behavior] {}: __aw_public_behaviors__ references missing public functions: {}",
                    path.display(),
                    missing.join(", ")
                );
            }
            explicit
                .into_iter()
                .map(|name| name.replace('_', "-"))
                .collect::<Vec<_>>()
        };
        if behaviors.is_empty() {
            bail!(
                "Python TD diagnostic [public-contract-without-behavior] {}: __aw_public_contract__ = True requires at least one public top-level function",
                path.display()
            );
        }
        if behaviors.iter().any(|behavior| {
            let parts = behavior.split('/').collect::<Vec<_>>();
            parts.len() != 1 || !identity_slug(&parts[0])
        }) {
            bail!(
                "Python TD diagnostic [invalid-public-behavior] {}: public top-level function names must normalize to lowercase kebab-case EC use_case_id values",
                path.display()
            );
        }
        behaviors
    } else {
        Vec::new()
    };
    let cli = compile_typer_cli(path, tree.root_node(), &source, &imports, &declarations)?;
    let codegen = compile_codegen_contract(root, path, &declarations)?;
    Ok(PythonTdModule {
        id,
        artifact_id,
        public_contract,
        work_item,
        native_handwrite_targets,
        public_behaviors,
        path: rel.clone(),
        role: role_for_path(&rel),
        imports,
        declarations,
        cli,
        codegen,
    })
}

fn explicit_work_item(root: Node<'_>, source: &str, path: &Path) -> Result<Option<String>> {
    let mut values = Vec::new();
    let mut cursor = root.walk();
    for child in root.named_children(&mut cursor) {
        let Some(assignment) = direct_named_child(child, "assignment") else {
            continue;
        };
        let Some(left) = assignment.child_by_field_name("left") else {
            continue;
        };
        if node_text(left, source) != "__aw_work_item__" {
            continue;
        }
        let Some(right) = assignment.child_by_field_name("right") else {
            continue;
        };
        if right.kind() != "string" {
            return diagnostic(
                path,
                right,
                "invalid-work-item",
                "declare __aw_work_item__ as one quoted issue id",
            );
        }
        let value = quoted_literal(node_text(right, source)).with_context(|| {
            format!(
                "Python TD diagnostic [invalid-work-item] {}: __aw_work_item__ must be one simple quoted issue id",
                path.display()
            )
        })?;
        values.push(value.to_string());
    }
    if values.is_empty() {
        return Ok(None);
    }
    if values.len() != 1 || values[0].trim().is_empty() {
        bail!(
            "Python TD diagnostic [invalid-work-item] {}: declare one non-empty __aw_work_item__ per module",
            path.display()
        );
    }
    Ok(Some(values.remove(0)))
}

fn explicit_native_handwrite_targets(
    root: Node<'_>,
    source: &str,
    path: &Path,
) -> Result<Vec<String>> {
    let mut declarations = Vec::new();
    let mut cursor = root.walk();
    for child in root.named_children(&mut cursor) {
        let Some(assignment) = direct_named_child(child, "assignment") else {
            continue;
        };
        let Some(left) = assignment.child_by_field_name("left") else {
            continue;
        };
        if node_text(left, source) != "__aw_native_handwrite_targets__" {
            continue;
        }
        let Some(right) = assignment.child_by_field_name("right") else {
            continue;
        };
        if !matches!(right.kind(), "tuple" | "list") {
            return diagnostic(
                path,
                right,
                "invalid-native-handwrite-targets",
                "declare __aw_native_handwrite_targets__ as one literal tuple/list of repository-relative paths",
            );
        }
        let mut targets = Vec::new();
        let mut items = right.walk();
        for item in right.named_children(&mut items) {
            if item.kind() != "string" {
                return diagnostic(
                    path,
                    item,
                    "invalid-native-handwrite-targets",
                    "use only quoted repository-relative paths in __aw_native_handwrite_targets__",
                );
            }
            let value = quoted_literal(node_text(item, source)).with_context(|| {
                format!(
                    "Python TD diagnostic [invalid-native-handwrite-targets] {}: target paths must be simple quoted strings",
                    path.display()
                )
            })?;
            let candidate = Path::new(value);
            if value.is_empty()
                || candidate.is_absolute()
                || candidate.components().any(|component| {
                    matches!(
                        component,
                        Component::ParentDir | Component::RootDir | Component::Prefix(_)
                    )
                })
            {
                bail!(
                    "Python TD diagnostic [unsafe-native-handwrite-target] {}: `{value}` must be a repository-relative path without `..`",
                    path.display()
                );
            }
            let normalized = candidate
                .components()
                .filter_map(|component| match component {
                    Component::Normal(component) => Some(component.to_string_lossy().into_owned()),
                    Component::CurDir => None,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_) => None,
                })
                .collect::<Vec<_>>()
                .join("/");
            if normalized.is_empty() {
                bail!(
                    "Python TD diagnostic [unsafe-native-handwrite-target] {}: `{value}` must name a repository file",
                    path.display()
                );
            }
            targets.push(normalized);
        }
        declarations.push(targets);
    }
    if declarations.is_empty() {
        return Ok(Vec::new());
    }
    if declarations.len() != 1 || declarations[0].is_empty() {
        bail!(
            "Python TD diagnostic [invalid-native-handwrite-targets] {}: declare one non-empty __aw_native_handwrite_targets__ tuple/list per module",
            path.display()
        );
    }
    let mut targets = declarations.remove(0);
    targets.sort();
    if targets.windows(2).any(|pair| pair[0] == pair[1]) {
        bail!(
            "Python TD diagnostic [duplicate-native-handwrite-target] {}: __aw_native_handwrite_targets__ must not repeat paths",
            path.display()
        );
    }
    Ok(targets)
}

fn validate_unique_native_handwrite_targets(modules: &[PythonTdModule]) -> Result<()> {
    let mut owners = BTreeMap::<(String, String), &str>::new();
    for module in modules {
        let Some(work_item) = module.work_item.as_deref() else {
            continue;
        };
        for target in &module.native_handwrite_targets {
            let key = (work_item.to_string(), target.clone());
            if let Some(existing) = owners.insert(key.clone(), module.path.as_str()) {
                bail!(
                    "Python TD diagnostic [duplicate-native-handwrite-target] work item `{}` target `{}` is declared by both {} and {}",
                    key.0,
                    key.1,
                    existing,
                    module.path,
                );
            }
        }
    }
    Ok(())
}

fn explicit_public_behaviors(
    root: Node<'_>,
    source: &str,
    path: &Path,
) -> Result<Option<Vec<String>>> {
    let mut values = Vec::new();
    let mut cursor = root.walk();
    for child in root.named_children(&mut cursor) {
        let Some(assignment) = direct_named_child(child, "assignment") else {
            continue;
        };
        let Some(left) = assignment.child_by_field_name("left") else {
            continue;
        };
        if node_text(left, source) != "__aw_public_behaviors__" {
            continue;
        }
        let Some(right) = assignment.child_by_field_name("right") else {
            continue;
        };
        if !matches!(right.kind(), "tuple" | "list") {
            return diagnostic(
                path,
                right,
                "invalid-public-behaviors",
                "declare __aw_public_behaviors__ as one literal tuple/list of public function names",
            );
        }
        let mut items = right.walk();
        for item in right.named_children(&mut items) {
            if item.kind() != "string" {
                return diagnostic(
                    path,
                    item,
                    "invalid-public-behaviors",
                    "use only quoted public function names in __aw_public_behaviors__",
                );
            }
            let value = quoted_literal(node_text(item, source)).with_context(|| {
                format!(
                    "Python TD diagnostic [invalid-public-behaviors] {}: behavior names must be simple quoted strings",
                    path.display()
                )
            })?;
            values.push(value.to_string());
        }
    }
    if values.is_empty() {
        return Ok(None);
    }
    values.sort();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        bail!(
            "Python TD diagnostic [duplicate-public-behavior] {}: __aw_public_behaviors__ must not repeat function names",
            path.display()
        );
    }
    Ok(Some(values))
}

fn compile_typer_cli(
    path: &Path,
    root: Node<'_>,
    source: &str,
    imports: &[String],
    declarations: &[PythonTdDeclaration],
) -> Result<Option<PythonTdCli>> {
    let mut app_assignments = Vec::new();
    let mut cursor = root.walk();
    for child in root.named_children(&mut cursor) {
        let Some(assignment) = direct_named_child(child, "assignment") else {
            continue;
        };
        let Some(right) = assignment.child_by_field_name("right") else {
            continue;
        };
        let right_text = normalize_python_expression(node_text(right, source));
        if !right_text.starts_with("typer.Typer(") || !right_text.ends_with(')') {
            continue;
        }
        let Some(left) = assignment.child_by_field_name("left") else {
            return diagnostic(
                path,
                assignment,
                "invalid-typer-app",
                "assign typer.Typer(...) to one plain top-level identifier",
            );
        };
        if left.kind() != "identifier" {
            return diagnostic(
                path,
                left,
                "invalid-typer-app",
                "assign typer.Typer(...) to one plain top-level identifier",
            );
        }
        app_assignments.push((
            node_text(left, source).to_string(),
            canonical_argument_list(
                right_text
                    .trim_start_matches("typer.Typer(")
                    .trim_end_matches(')'),
            ),
            assignment,
        ));
    }
    if app_assignments.is_empty() {
        if imports.iter().any(|import| import == "importtyper")
            || normalize(source).contains("typer.Typer(")
        {
            reject_dynamic_typer_registration(path, root, source, None)?;
        }
        return Ok(None);
    }
    if app_assignments.len() != 1 {
        return diagnostic(
            path,
            app_assignments[1].2,
            "multiple-typer-apps",
            "declare exactly one top-level `app = typer.Typer(...)` per CLI TD module",
        );
    }
    if !imports.iter().any(|import| import == "importtyper") {
        return diagnostic(
            path,
            app_assignments[0].2,
            "invalid-typer-import",
            "use the explicit `import typer` form for the static CLI authoring subset",
        );
    }

    let (app_symbol, configuration, _) = &app_assignments[0];
    reject_dynamic_typer_registration(path, root, source, Some(app_symbol))?;
    let mut commands = Vec::new();
    for declaration in declarations
        .iter()
        .filter(|declaration| declaration.kind == PythonTdDeclarationKind::Function)
    {
        for decorator in &declaration.decorators {
            let Some((kind, binding_configuration)) =
                parse_typer_decorator(path, decorator, app_symbol)?
            else {
                continue;
            };
            let name = match kind {
                PythonTdCliCommandKind::Callback => declaration.name.clone(),
                PythonTdCliCommandKind::Command => first_quoted_positional(&binding_configuration)
                    .unwrap_or_else(|| declaration.name.replace('_', "-")),
            };
            if !identity_slug(&name.as_str()) {
                bail!(
                    "Python TD diagnostic [invalid-typer-command-name] {}:{}:{}: command `{name}` must be lowercase kebab-case",
                    path.display(),
                    declaration.span.line,
                    declaration.span.column
                );
            }
            let signature = declaration
                .annotations
                .iter()
                .find(|annotation| annotation.starts_with('('))
                .map(String::as_str)
                .unwrap_or("()");
            validate_typer_command_body(path, root, source, declaration)?;
            commands.push(PythonTdCliCommand {
                function: declaration.name.clone(),
                name,
                kind,
                configuration: binding_configuration,
                parameters: parse_typer_parameters(path, declaration, signature)?,
            });
        }
    }
    if commands.is_empty() {
        bail!(
            "Python TD diagnostic [typer-app-without-command] {}: one `typer.Typer(...)` app requires at least one `@{app_symbol}.command(...)` binding",
            path.display()
        );
    }
    commands.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then(left.function.cmp(&right.function))
    });
    let duplicates = commands
        .windows(2)
        .filter(|pair| pair[0].name == pair[1].name && pair[0].kind == pair[1].kind)
        .map(|pair| pair[0].name.clone())
        .collect::<Vec<_>>();
    if !duplicates.is_empty() {
        bail!(
            "Python TD diagnostic [duplicate-typer-command] {}: duplicate command identities: {}",
            path.display(),
            duplicates.join(", ")
        );
    }
    Ok(Some(PythonTdCli {
        framework: "typer",
        app_symbol: app_symbol.clone(),
        configuration: configuration.clone(),
        commands,
    }))
}

fn validate_typer_command_body(
    path: &Path,
    root: Node<'_>,
    source: &str,
    declaration: &PythonTdDeclaration,
) -> Result<()> {
    let Some(function) =
        root.descendant_for_byte_range(declaration.span.byte_start, declaration.span.byte_end)
    else {
        bail!(
            "Python TD diagnostic [missing-typer-command-body] {}:{}:{}: cannot resolve executable body for `{}`",
            path.display(),
            declaration.span.line,
            declaration.span.column,
            declaration.name
        );
    };
    let Some(body) = function.child_by_field_name("body") else {
        bail!(
            "Python TD diagnostic [missing-typer-command-body] {}:{}:{}: command `{}` has no executable body",
            path.display(),
            declaration.span.line,
            declaration.span.column,
            declaration.name
        );
    };
    let mut cursor = body.walk();
    let statements = body.named_children(&mut cursor).collect::<Vec<_>>();
    let executable = statements.iter().enumerate().any(|(index, statement)| {
        let text = normalize_python_expression(node_text(*statement, source));
        let docstring = index == 0
            && statement.kind() == "expression_statement"
            && statement
                .named_child(0)
                .is_some_and(|child| child.kind() == "string");
        !docstring
            && statement.kind() != "pass_statement"
            && text != "..."
            && text != "return"
            && text != "returnNone"
    });
    if !executable {
        bail!(
            "Python TD diagnostic [non-executable-typer-command] {}:{}:{}: command `{}` is only a placeholder; a TD command must execute the reference-product behavior",
            path.display(),
            declaration.span.line,
            declaration.span.column,
            declaration.name
        );
    }
    Ok(())
}

fn direct_named_child<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    if node.kind() == kind {
        return Some(node);
    }
    let mut cursor = node.walk();
    let found = node
        .named_children(&mut cursor)
        .find(|child| child.kind() == kind);
    found
}

fn reject_dynamic_typer_registration(
    path: &Path,
    root: Node<'_>,
    source: &str,
    app_symbol: Option<&str>,
) -> Result<()> {
    fn walk(
        path: &Path,
        node: Node<'_>,
        source: &str,
        app_symbol: Option<&str>,
        inside_decorator: bool,
    ) -> Result<()> {
        let inside_decorator = inside_decorator || node.kind() == "decorator";
        if node.kind() == "call" && !inside_decorator {
            if let Some(function) = node.child_by_field_name("function") {
                let function = normalize(node_text(function, source));
                let dynamic = function.ends_with(".add_typer")
                    || function.ends_with(".command")
                    || function.ends_with(".callback");
                if dynamic
                    && app_symbol.is_none_or(|app| {
                        function == format!("{app}.add_typer")
                            || function == format!("{app}.command")
                            || function == format!("{app}.callback")
                    })
                {
                    return diagnostic(
                        path,
                        node,
                        "dynamic-typer-registration",
                        "use only top-level `@app.command(...)` and `@app.callback(...)` decorators; runtime registration is outside the static subset",
                    );
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            walk(path, child, source, app_symbol, inside_decorator)?;
        }
        Ok(())
    }
    walk(path, root, source, app_symbol, false)
}

fn parse_typer_decorator(
    path: &Path,
    decorator: &str,
    app_symbol: &str,
) -> Result<Option<(PythonTdCliCommandKind, String)>> {
    let prefix = format!("@{app_symbol}.");
    if !decorator.starts_with(&prefix) {
        if decorator.contains(".command") || decorator.contains(".callback") {
            bail!(
                "Python TD diagnostic [ambiguous-typer-app] {}: bind every Typer command to the single `{app_symbol}` app",
                path.display()
            );
        }
        return Ok(None);
    }
    let tail = &decorator[prefix.len()..];
    let (kind, arguments) = if let Some(arguments) = tail
        .strip_prefix("command(")
        .and_then(|value| value.strip_suffix(')'))
    {
        (PythonTdCliCommandKind::Command, arguments)
    } else if let Some(arguments) = tail
        .strip_prefix("callback(")
        .and_then(|value| value.strip_suffix(')'))
    {
        (PythonTdCliCommandKind::Callback, arguments)
    } else {
        bail!(
            "Python TD diagnostic [invalid-typer-decorator] {}: use exactly `@{app_symbol}.command(...)` or `@{app_symbol}.callback(...)`",
            path.display()
        );
    };
    Ok(Some((kind, canonical_argument_list(arguments))))
}

fn parse_typer_parameters(
    path: &Path,
    declaration: &PythonTdDeclaration,
    signature: &str,
) -> Result<Vec<PythonTdCliParameter>> {
    let inner = signature
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .unwrap_or(signature);
    let mut parameters = Vec::new();
    for raw in split_top_level(inner, ',') {
        if raw.is_empty() || raw == "/" || raw == "*" {
            continue;
        }
        if raw.starts_with('*') {
            bail!(
                "Python TD diagnostic [unsupported-typer-parameter] {}:{}:{}: variadic CLI parameters are outside the static subset",
                path.display(),
                declaration.span.line,
                declaration.span.column
            );
        }
        let colon = top_level_character(&raw, ':').with_context(|| {
            format!(
                "Python TD diagnostic [untyped-typer-parameter] {}:{}:{}: every CLI parameter requires an explicit annotation",
                path.display(),
                declaration.span.line,
                declaration.span.column
            )
        })?;
        let equals = top_level_character(&raw, '=');
        let name = raw[..colon].to_string();
        let annotation_end = equals.unwrap_or(raw.len());
        let annotation = raw[colon + 1..annotation_end].to_string();
        let default = equals.map(|index| raw[index + 1..].to_string());
        let (kind, binding) = if let Some(binding) =
            typer_binding_expression(&annotation, "typer.Argument").or_else(|| {
                default
                    .as_deref()
                    .and_then(|value| typer_binding_expression(value, "typer.Argument"))
            }) {
            (PythonTdCliParameterKind::Argument, binding)
        } else if let Some(binding) =
            typer_binding_expression(&annotation, "typer.Option").or_else(|| {
                default
                    .as_deref()
                    .and_then(|value| typer_binding_expression(value, "typer.Option"))
            })
        {
            (PythonTdCliParameterKind::Option, binding)
        } else {
            bail!(
                "Python TD diagnostic [implicit-typer-parameter] {}:{}:{}: parameter `{name}` must declare `typer.Argument(...)` or `typer.Option(...)` explicitly",
                path.display(),
                declaration.span.line,
                declaration.span.column
            );
        };
        parameters.push(PythonTdCliParameter {
            name,
            kind,
            annotation,
            default,
            binding,
        });
    }
    Ok(parameters)
}

fn typer_binding_expression(value: &str, binding: &str) -> Option<String> {
    let start = value.find(binding)?;
    let suffix = &value[start..];
    let open = suffix.find('(')?;
    let end = matching_delimiter(suffix, open, '(', ')')?;
    Some(suffix[..=end].to_string())
}

fn split_top_level(value: &str, separator: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut stack = Vec::new();
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if let Some(active) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == active {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '(' | '[' | '{' => stack.push(character),
            ')' | ']' | '}' => {
                stack.pop();
            }
            _ if character == separator && stack.is_empty() => {
                parts.push(value[start..index].to_string());
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(value[start..].to_string());
    parts
}

fn canonical_argument_list(value: &str) -> String {
    split_top_level(value, ',')
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(",")
}

fn top_level_character(value: &str, wanted: char) -> Option<usize> {
    let mut stack = Vec::new();
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if let Some(active) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == active {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '(' | '[' | '{' => stack.push(character),
            ')' | ']' | '}' => {
                stack.pop();
            }
            _ if character == wanted && stack.is_empty() => return Some(index),
            _ => {}
        }
    }
    None
}

fn matching_delimiter(value: &str, open_index: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0;
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in value
        .char_indices()
        .filter(|(index, _)| *index >= open_index)
    {
        if let Some(active) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == active {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            _ if character == open => depth += 1,
            _ if character == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn first_quoted_positional(value: &str) -> Option<String> {
    let first = split_top_level(value, ',').into_iter().next()?;
    if first.contains('=') {
        return None;
    }
    quoted_literal(&first).map(str::to_string)
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

fn explicit_public_contract(source: &str, path: &Path) -> Result<bool> {
    let values = source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("__aw_public_contract__"))
        .filter_map(|tail| tail.trim_start().strip_prefix('='))
        .map(str::trim)
        .collect::<Vec<_>>();
    if values.is_empty() {
        return Ok(false);
    }
    if values.len() != 1 {
        bail!(
            "Python TD diagnostic [duplicate-public-contract] {}: declare __aw_public_contract__ at most once per module",
            path.display()
        );
    }
    match values[0] {
        "True" => Ok(true),
        "False" => Ok(false),
        _ => bail!(
            "Python TD diagnostic [invalid-public-contract] {}: __aw_public_contract__ must be the literal True or False",
            path.display()
        ),
    }
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
        let normalized = normalize_python_expression(node_text(parameters, source));
        let inner = normalized
            .strip_prefix('(')
            .and_then(|value| value.strip_suffix(')'))
            .unwrap_or(&normalized);
        annotations.push(format!("({})", canonical_argument_list(inner)));
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
        source,
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
        source,
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
    source: &str,
) -> PythonTdDeclaration {
    annotations.sort();
    decorators.sort();
    let point = node.start_position();
    let implementation = node
        .child_by_field_name("body")
        .map(|body| normalize_python_expression(node_text(body, source)))
        .unwrap_or_default();
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
        implementation_digest: format!("sha256:{:x}", Sha256::digest(implementation.as_bytes())),
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
        .map(|module| module_id_for_path(&module.path))
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
            if !known_modules.contains(&candidate) && !known_modules.contains(&package_init) {
                bail!(
                    "Python TD diagnostic [unresolved-local-import] {}:1:1: `{path}` is not a project module; add the module or correct the import",
                    module.path
                );
            }
        }
    }
    Ok(())
}

fn module_id_for_path(path: &str) -> String {
    format!("module:{}", path.trim_end_matches(".py").replace('/', "."))
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
fn normalize_python_expression(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut quote = None;
    let mut escaped = false;
    for character in value.chars() {
        if let Some(active) = quote {
            output.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == active {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => {
                quote = Some(character);
                output.push(character);
            }
            _ if character.is_whitespace() => {}
            _ => output.push(character),
        }
    }
    output
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

    #[test]
    fn public_contract_requires_explicit_artifact_identity() {
        let root = tempfile::tempdir().unwrap();
        let src = root.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("contract.py"),
            "__aw_public_contract__ = True\n\ndef contract() -> str:\n    return \"public\"\n",
        )
        .unwrap();

        let error = compile_python_td_project(root.path())
            .expect_err("public contract without identity must fail")
            .to_string();
        assert!(
            error.contains("[public-contract-without-artifact-id]"),
            "{error}"
        );
    }

    #[test]
    fn public_contract_marker_is_projected_into_ir() {
        let root = tempfile::tempdir().unwrap();
        let src = root.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("contract.py"),
            "__aw_artifact_id__ = \"artifact:health/contract\"\n__aw_public_contract__ = True\n\ndef contract() -> str:\n    return \"public\"\n",
        )
        .unwrap();

        let ir = compile_python_td_project(root.path()).unwrap();
        assert!(ir.modules[0].public_contract);
        assert_eq!(ir.modules[0].public_behaviors, vec!["contract"]);
    }

    #[test]
    fn native_handwrite_targets_are_projected_with_their_work_item_binding() {
        let root = tempfile::tempdir().unwrap();
        let src = root.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("acceptance.py"),
            "__aw_work_item__ = \"42\"\n__aw_native_handwrite_targets__ = (\"acceptance/verify-demo.sh\",)\n",
        )
        .unwrap();

        let ir = compile_python_td_project(root.path()).unwrap();
        assert_eq!(ir.modules[0].work_item.as_deref(), Some("42"));
        assert_eq!(
            ir.modules[0].native_handwrite_targets,
            vec!["acceptance/verify-demo.sh"]
        );
    }

    #[test]
    fn native_handwrite_targets_require_a_work_item_binding() {
        let root = tempfile::tempdir().unwrap();
        let src = root.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("acceptance.py"),
            "__aw_native_handwrite_targets__ = (\"acceptance/verify-demo.sh\",)\n",
        )
        .unwrap();

        let error = compile_python_td_project(root.path())
            .expect_err("native target without a work-item binding must fail")
            .to_string();
        assert!(
            error.contains("[native-handwrite-without-work-item]"),
            "{error}"
        );
    }

    #[test]
    fn duplicate_native_handwrite_target_across_modules_is_rejected_after_normalization() {
        let root = tempfile::tempdir().unwrap();
        let src = root.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("first.py"),
            "__aw_work_item__ = \"42\"\n__aw_native_handwrite_targets__ = (\"acceptance/verify-demo.sh\",)\n",
        )
        .unwrap();
        fs::write(
            src.join("second.py"),
            "__aw_work_item__ = \"42\"\n__aw_native_handwrite_targets__ = (\"./acceptance/verify-demo.sh\",)\n",
        )
        .unwrap();

        let error = compile_python_td_project(root.path())
            .expect_err("the same normalized target cannot be owned twice")
            .to_string();
        assert!(
            error.contains("[duplicate-native-handwrite-target]"),
            "{error}"
        );
    }

    #[test]
    fn public_contract_requires_an_executable_behavior() {
        let temp = tempfile::tempdir().expect("tempdir");
        let src = temp.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(
            src.join("contract.py"),
            "__aw_artifact_id__ = \"artifact:health/contract\"\n__aw_public_contract__ = True\n",
        )
        .expect("write");

        let error = compile_python_td_project(temp.path()).expect_err("must fail");
        assert!(error
            .to_string()
            .contains("[public-contract-without-behavior]"));
    }

    #[test]
    fn test_module_can_import_a_module_with_explicit_artifact_identity() {
        let root = tempfile::tempdir().unwrap();
        let src = root.path().join("src/demo");
        let tests = root.path().join("tests/unit");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&tests).unwrap();
        fs::write(src.join("__init__.py"), "").unwrap();
        fs::write(
            src.join("contract.py"),
            "__aw_artifact_id__ = \"artifact:demo/contract\"\n\ndef contract() -> bool:\n    return True\n",
        )
        .unwrap();
        fs::write(
            tests.join("test_contract.py"),
            "from demo.contract import contract\n\ndef test_contract() -> None:\n    assert contract()\n",
        )
        .unwrap();

        compile_python_td_project(root.path())
            .expect("module path remains importable even when identity is artifact-based");
    }

    #[test]
    fn typer_cli_compiles_into_target_neutral_ir_without_importing_module() {
        let root = tempfile::tempdir().unwrap();
        let src = root.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("cli.py"),
            r#"from typing import Annotated
import typer

__aw_artifact_id__ = "artifact:demo/cli"

raise RuntimeError("the static compiler must never import this module")

app = typer.Typer(name="demo", no_args_is_help=True)

@app.callback()
def root(verbose: Annotated[bool, typer.Option("--verbose", "-v")] = False) -> None:
    typer.echo("verbose" if verbose else "normal")

@app.command("scan", help="Scan one path.")
def scan(
    path: Annotated[str, typer.Argument(help="Path to scan")],
    strict: Annotated[bool, typer.Option("--strict")] = False,
) -> None:
    typer.echo(f"{path}:{strict}")
"#,
        )
        .unwrap();

        let ir = compile_python_td_project(root.path()).unwrap();
        let cli = ir.modules[0].cli.as_ref().expect("Typer CLI IR");
        assert_eq!(cli.framework, "typer");
        assert_eq!(cli.app_symbol, "app");
        assert_eq!(cli.configuration, "name=\"demo\",no_args_is_help=True");
        assert_eq!(cli.commands.len(), 2);
        let scan = cli
            .commands
            .iter()
            .find(|command| command.name == "scan")
            .unwrap();
        assert_eq!(scan.kind, PythonTdCliCommandKind::Command);
        assert!(scan.configuration.contains("help=\"Scan one path.\""));
        assert_eq!(scan.parameters.len(), 2);
        assert_eq!(scan.parameters[0].kind, PythonTdCliParameterKind::Argument);
        assert_eq!(scan.parameters[0].name, "path");
        assert!(scan.parameters[0].binding.contains("Path to scan"));
        assert_eq!(scan.parameters[1].kind, PythonTdCliParameterKind::Option);
        assert_eq!(scan.parameters[1].default.as_deref(), Some("False"));
    }

    #[test]
    fn typer_cli_formatting_does_not_change_semantic_digest() {
        let write_fixture = |root: &Path, source: &str| {
            let src = root.join("src");
            fs::create_dir_all(&src).unwrap();
            fs::write(src.join("cli.py"), source).unwrap();
        };
        let compact = tempfile::tempdir().unwrap();
        write_fixture(
            compact.path(),
            "import typer\napp=typer.Typer(name=\"demo\")\n@app.command()\ndef run(path:typer.Argument(...))->None:\n typer.echo(path)\n",
        );
        let formatted = tempfile::tempdir().unwrap();
        write_fixture(
            formatted.path(),
            "import typer\n\napp = typer.Typer(\n    name = \"demo\",\n)\n\n@app.command()\ndef run(\n    path: typer.Argument(...),\n) -> None:\n    typer.echo(path)\n",
        );

        let compact = compile_python_td_project(compact.path()).unwrap();
        let formatted = compile_python_td_project(formatted.path()).unwrap();
        assert_eq!(compact.semantic_digest, formatted.semantic_digest);
    }

    #[test]
    fn typer_cli_rejects_dynamic_registration() {
        let root = tempfile::tempdir().unwrap();
        let src = root.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("cli.py"),
            "import typer\napp = typer.Typer(name=\"demo\")\ndef run() -> None:\n    pass\napp.command()(run)\n",
        )
        .unwrap();

        let error = compile_python_td_project(root.path())
            .expect_err("runtime command registration must fail")
            .to_string();
        assert!(error.contains("[dynamic-typer-registration]"), "{error}");
    }

    #[test]
    fn typer_cli_requires_explicit_argument_or_option_binding() {
        let root = tempfile::tempdir().unwrap();
        let src = root.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("cli.py"),
            "import typer\napp = typer.Typer(name=\"demo\")\n@app.command()\ndef run(path: str) -> None:\n    typer.echo(path)\n",
        )
        .unwrap();

        let error = compile_python_td_project(root.path())
            .expect_err("implicit parameter semantics must fail")
            .to_string();
        assert!(error.contains("[implicit-typer-parameter]"), "{error}");
    }

    #[test]
    fn typer_cli_rejects_placeholder_command_bodies() {
        let root = tempfile::tempdir().unwrap();
        let src = root.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("cli.py"),
            "import typer\napp = typer.Typer(name=\"demo\")\n@app.command()\ndef run() -> None:\n    \"\"\"Only documentation, not product behavior.\"\"\"\n    pass\n",
        )
        .unwrap();

        let error = compile_python_td_project(root.path())
            .expect_err("placeholder commands must fail")
            .to_string();
        assert!(error.contains("[non-executable-typer-command]"), "{error}");
    }

    #[test]
    fn executable_body_changes_semantic_digest() {
        let write_fixture = |root: &Path, value: &str| {
            let src = root.join("src");
            fs::create_dir_all(&src).unwrap();
            fs::write(
                src.join("contract.py"),
                format!("def decision() -> str:\n    return \"{value}\"\n"),
            )
            .unwrap();
        };
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        write_fixture(first.path(), "clean");
        write_fixture(second.path(), "findings");

        let first = compile_python_td_project(first.path()).unwrap();
        let second = compile_python_td_project(second.path()).unwrap();
        assert_ne!(first.semantic_digest, second.semantic_digest);
    }

    #[test]
    fn public_behavior_inventory_can_exclude_internal_product_helpers() {
        let root = tempfile::tempdir().unwrap();
        let src = root.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("contract.py"),
            r#"__aw_artifact_id__ = "artifact:demo/contract"
__aw_public_contract__ = True
__aw_public_behaviors__ = ("external_journey",)

def external_journey() -> bool:
    return helper()

def helper() -> bool:
    return True
"#,
        )
        .unwrap();

        let ir = compile_python_td_project(root.path()).unwrap();
        assert_eq!(ir.modules[0].public_behaviors, vec!["external-journey"]);
        assert_eq!(ir.modules[0].declarations.len(), 2);
    }
}
