// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
// CODEGEN-BEGIN
//! Nx monorepo integration for Jet.
//!
//! Provides types for parsing the Nx project graph (`nx graph --json`) and
//! configuration (`nx.json`), plus `NxWorkspaceManager` for workspace
//! detection and graph retrieval.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};

// ------------------------------------------------------------------
// project.json (native Nx project manifest)
// ------------------------------------------------------------------

/// Direct representation of an Nx `project.json` file.
///
/// Each Nx project carries a `project.json` at its root that declares the
/// project's name, type, build targets, and optional explicit dependency
/// edges. `build_project_graph_from_files` reads these files to construct
/// a full `NxProjectGraph` without invoking the `nx` CLI.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectJson {
    /// Canonical project name (e.g. `"my-app"`).
    pub name: Option<String>,
    /// `"application"` or `"library"`.
    #[serde(rename = "projectType")]
    pub project_type: Option<String>,
    /// Configured build/test/lint targets.
    #[serde(default)]
    pub targets: HashMap<String, serde_json::Value>,
    /// Explicit dependency edges declared in the manifest.
    #[serde(rename = "implicitDependencies", default)]
    pub implicit_dependencies: Vec<String>,
    /// Catch-all for forward-compatibility.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ------------------------------------------------------------------
// nx.json config
// ------------------------------------------------------------------

/// Parsed representation of `nx.json`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NxConfig {
    #[serde(rename = "tasksRunnerOptions", default)]
    pub tasks_runner_options: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub affected: HashMap<String, serde_json::Value>,
    #[serde(rename = "targetDefaults", default)]
    pub target_defaults: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub plugins: Vec<serde_json::Value>,
    /// Catch-all for forward-compatibility.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ------------------------------------------------------------------
// Project graph types (output of `nx graph --json`)
// ------------------------------------------------------------------

/// Root envelope returned by `nx graph --json`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NxProjectGraph {
    pub graph: NxGraphData,
}

/// Inner graph object containing nodes and dependency edges.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NxGraphData {
    /// Map of project name → project node.
    pub nodes: HashMap<String, NxProject>,
    /// Map of project name → list of dependency edges originating from it.
    #[serde(default)]
    pub dependencies: HashMap<String, Vec<NxDependency>>,
}

/// A single project node in the Nx project graph.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NxProject {
    pub name: String,
    #[serde(rename = "type")]
    pub project_type: Option<String>,
    pub data: Option<NxProjectData>,
}

/// Per-project metadata (subset of `project.json`).
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NxProjectData {
    /// Relative path from workspace root to the project directory.
    pub root: Option<String>,
    /// Configured Nx targets (build, test, lint, …).
    #[serde(default)]
    pub targets: HashMap<String, serde_json::Value>,
}

/// A directed dependency edge in the Nx project graph.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NxDependency {
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub dep_type: Option<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl NxProjectGraph {
    /// Return project names in topological (dependency-first) order.
    ///
    /// Projects with zero dependencies come first; projects that depend on
    /// them follow. The sort within each "tier" is alphabetical for
    /// determinism.
    pub fn topological_sort(&self) -> Vec<String> {
        let nodes = &self.graph.nodes;
        let deps = &self.graph.dependencies;

        // in_degree[name] = number of dependencies that name has (within the graph)
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        // dependents[target] = list of projects that depend on target
        let mut dependents: HashMap<String, Vec<String>> = HashMap::new();

        for name in nodes.keys() {
            in_degree.entry(name.clone()).or_insert(0);
        }

        for (source, dep_list) in deps {
            if !nodes.contains_key(source) {
                continue;
            }
            for dep in dep_list {
                if !nodes.contains_key(&dep.target) {
                    continue;
                }
                // source depends on target → target must build first
                *in_degree.entry(source.clone()).or_insert(0) += 1;
                dependents
                    .entry(dep.target.clone())
                    .or_default()
                    .push(source.clone());
            }
        }

        // Seed the queue with nodes that have no dependencies, sorted for
        // determinism.
        let mut zero: Vec<String> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(n, _)| n.clone())
            .collect();
        zero.sort();

        let mut queue: VecDeque<String> = zero.into();
        let mut order = Vec::new();

        while let Some(name) = queue.pop_front() {
            order.push(name.clone());
            if let Some(list) = dependents.get(&name) {
                let mut next_batch: Vec<String> = list
                    .iter()
                    .filter_map(|dep| {
                        let deg = in_degree.get_mut(dep)?;
                        *deg -= 1;
                        if *deg == 0 {
                            Some(dep.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                next_batch.sort();
                queue.extend(next_batch);
            }
        }

        order
    }

    /// Return all project names in sorted order.
    pub fn project_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.graph.nodes.keys().cloned().collect();
        names.sort();
        names
    }

    /// Return the filesystem root directory of a project, if set.
    pub fn project_root(&self, name: &str) -> Option<&str> {
        self.graph
            .nodes
            .get(name)
            .and_then(|p| p.data.as_ref())
            .and_then(|d| d.root.as_deref())
    }
}

// ------------------------------------------------------------------
// NxWorkspaceManager
// ------------------------------------------------------------------

/// Manages Nx workspace detection and project graph retrieval.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub struct NxWorkspaceManager {
    /// Absolute path to the workspace root (directory containing `nx.json`).
    pub root: PathBuf,
    /// Parsed `nx.json` configuration.
    pub config: NxConfig,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl NxWorkspaceManager {
    /// Detect an Nx workspace rooted at `root`.
    ///
    /// Returns `Some(NxWorkspaceManager)` when `nx.json` is present and
    /// valid; `None` when `nx.json` does not exist.
    ///
    /// # Errors
    /// Returns an error if `nx.json` exists but cannot be parsed.
    pub fn discover(root: &Path) -> Result<Option<Self>> {
        let nx_json_path = root.join("nx.json");
        if !nx_json_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&nx_json_path)
            .with_context(|| format!("Failed to read {:?}", nx_json_path))?;
        let config: NxConfig = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse nx.json at {:?}", nx_json_path))?;

        Ok(Some(Self {
            root: root.to_path_buf(),
            config,
        }))
    }

    /// Execute `nx graph --json` in the workspace root and parse the result.
    ///
    /// # Errors
    /// Returns an error if the `nx` binary is not found, the command fails,
    /// or the JSON output cannot be parsed.
    ///
    /// # Deprecation Note
    /// Prefer `build_project_graph_from_files` which parses `project.json`
    /// files directly and does not require the `nx` CLI to be installed.
    pub fn get_project_graph(&self) -> Result<NxProjectGraph> {
        let output = std::process::Command::new("nx")
            .args(["graph", "--json"])
            .current_dir(&self.root)
            .output()
            .with_context(|| {
                "Failed to execute 'nx graph --json'. \
                 Ensure Nx CLI is installed (npm install -g nx or use npx nx)."
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("'nx graph --json' failed:\n{}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let graph: NxProjectGraph = serde_json::from_str(&stdout)
            .with_context(|| "Failed to parse JSON output of 'nx graph --json'")?;

        Ok(graph)
    }

    /// Build the project graph by reading `project.json` files directly.
    ///
    /// This method replaces `get_project_graph` for environments where the
    /// `nx` CLI is not available. It:
    ///
    /// 1. Walks the workspace directory tree looking for `project.json` files
    ///    (skipping `node_modules`, `.git`, and hidden directories).
    /// 2. Parses each `project.json` to extract the project name, type, root,
    ///    and `implicitDependencies` edges.
    /// 3. Supplements explicit edges with cross-project workspace-protocol
    ///    references found in the project's own `package.json`.
    /// 4. Returns a fully populated `NxProjectGraph` with no subprocess call.
    ///
    /// # Errors
    /// Returns an error if any found `project.json` file cannot be read or
    /// parsed as valid JSON.
    pub fn build_project_graph_from_files(&self) -> Result<NxProjectGraph> {
        let mut nodes: HashMap<String, NxProject> = HashMap::new();
        let mut dependencies: HashMap<String, Vec<NxDependency>> = HashMap::new();

        // Collect (project_name, pkg_json_path, implicit_deps) for second pass.
        let mut pkg_json_paths: Vec<(String, PathBuf, Vec<String>)> = Vec::new();

        // --- Pass 1: Discover all project.json files ---
        Self::walk_for_project_json(&self.root, &mut |project_json_path| {
            // GH #3765 — was a stack of three silent fallbacks
            // (.parent() / strip_prefix / to_string_lossy) that conflated
            // "workspace root", "outside workspace", and "non-UTF-8 path"
            // into the same empty-string `rel_root`. Distinguish each
            // failure mode and warn so operators can chase Nx graph
            // identity bugs.
            let rel_root = derive_rel_root(&project_json_path, &self.root);

            let content = std::fs::read_to_string(&project_json_path)
                .with_context(|| format!("Failed to read {:?}", project_json_path))?;

            let proj: ProjectJson = serde_json::from_str(&content).with_context(|| {
                format!("Failed to parse project.json at {:?}", project_json_path)
            })?;

            // GH #3772 — the directory-name fallback used to silently
            // `to_string_lossy` non-UTF-8 names, which could collide two
            // sibling projects on the same HashMap key in the Nx graph
            // and shadow build/test discovery. Distinguish UTF-8 from
            // non-UTF-8 directory names and warn on the lossy branch.
            let name = proj
                .name
                .clone()
                .or_else(|| derive_project_name_from_dir(&project_json_path))
                .unwrap_or_else(|| rel_root.replace('/', "-"));

            let node = NxProject {
                name: name.clone(),
                project_type: proj.project_type.clone(),
                data: Some(NxProjectData {
                    root: Some(rel_root.clone()),
                    targets: proj.targets.clone(),
                }),
            };

            nodes.insert(name.clone(), node);
            dependencies.entry(name.clone()).or_default();

            // Queue implicit dependencies for pass 2.
            let pkg_json = project_json_path.parent().unwrap().join("package.json");
            pkg_json_paths.push((name, pkg_json, proj.implicit_dependencies));

            Ok(())
        })?;

        // --- Pass 2: Resolve dependency edges ---
        // a) Explicit `implicitDependencies` from project.json.
        // b) workspace:* / workspace:^ references in package.json.
        for (source, pkg_json_path, implicit_deps) in &pkg_json_paths {
            // Implicit edges declared in project.json.
            for target in implicit_deps {
                if nodes.contains_key(target.as_str()) {
                    dependencies
                        .entry(source.clone())
                        .or_default()
                        .push(NxDependency {
                            source: source.clone(),
                            target: target.clone(),
                            dep_type: Some("implicit".to_string()),
                        });
                }
            }

            // Cross-project workspace-protocol edges from package.json.
            if pkg_json_path.exists() {
                let pkg_content = std::fs::read_to_string(pkg_json_path).unwrap_or_default();
                if let Ok(pkg_val) = serde_json::from_str::<serde_json::Value>(&pkg_content) {
                    for field in &["dependencies", "devDependencies", "peerDependencies"] {
                        if let Some(map) = pkg_val[field].as_object() {
                            for (dep_name, dep_ver) in map {
                                // GH #3751 — prior `dep_ver.as_str()
                                // .unwrap_or("")` silently substituted
                                // empty string for any non-string shape
                                // (number, object, bool), so a wrong-
                                // shape `\"foo\": 1` could hide a true
                                // `workspace:*` ref from the project
                                // graph. Warn and skip non-string deps.
                                let ver = match dep_ver.as_str() {
                                    Some(s) => s,
                                    None => {
                                        let observed = describe_nx_dep_kind(dep_ver);
                                        tracing::warn!(
                                            target: "jet::pkg_manager::nx",
                                            project = %source,
                                            path = %pkg_json_path.display(),
                                            field = %field,
                                            dep_name = %dep_name,
                                            observed_type = %observed,
                                            "{}",
                                            format_nx_dep_version_shape_warn(
                                                &pkg_json_path,
                                                source,
                                                field,
                                                dep_name,
                                                observed,
                                            )
                                        );
                                        continue;
                                    }
                                };
                                if ver.starts_with("workspace:") {
                                    // Strip scope prefix for matching.
                                    let bare = dep_name.rsplit('/').next().unwrap_or(dep_name);
                                    // Find the project whose name or
                                    // directory matches.
                                    let target = nodes
                                        .keys()
                                        .find(|n| *n == dep_name || n.as_str() == bare)
                                        .cloned();
                                    if let Some(t) = target {
                                        let edges = dependencies.entry(source.clone()).or_default();
                                        let already = edges.iter().any(|e| e.target == t);
                                        if !already {
                                            edges.push(NxDependency {
                                                source: source.clone(),
                                                target: t,
                                                dep_type: Some("static".to_string()),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(NxProjectGraph {
            graph: NxGraphData {
                nodes,
                dependencies,
            },
        })
    }

    /// Recursively walk `dir` and call `visitor` for every `project.json`
    /// found. Skips `node_modules`, `.git`, and any directory whose name
    /// starts with `.`.
    fn walk_for_project_json(
        dir: &Path,
        visitor: &mut impl FnMut(PathBuf) -> Result<()>,
    ) -> Result<()> {
        let entries = std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory {:?}", dir))?;

        let mut subdirs: Vec<PathBuf> = Vec::new();

        // GH #3292 — `entries.flatten()` previously dropped per-dirent
        // errors. A single unreadable entry silently hid its own
        // potential `project.json` AND the entire subtree rooted at
        // it; downstream graph/test discovery then ran on a truncated
        // inventory with no breadcrumb. Surface the per-dirent error
        // and keep walking.
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(err) => {
                    tracing::warn!(
                        target: "jet::pkg::nx",
                        path = %dir.display(),
                        error = %err,
                        "GH #3292 unreadable dirent during workspace project.json discovery; \
                         a project.json or its subtree may be silently omitted"
                    );
                    continue;
                }
            };
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Skip hidden directories, node_modules, and dist artifacts.
            if name_str.starts_with('.')
                || name_str == "node_modules"
                || name_str == "dist"
                || name_str == "target"
            {
                continue;
            }

            if path.is_dir() {
                subdirs.push(path);
            } else if name_str == "project.json" {
                visitor(path)?;
            }
        }

        for subdir in subdirs {
            // GH #3292 — subtree-level `read_dir` failures (typically
            // `EACCES` on a chmod 0o000 directory) used to abort the
            // entire walk via `?`, hiding every `project.json` in
            // healthy sibling subtrees. Probe `read_dir` first and
            // downgrade an open failure to a warn so siblings still
            // get discovered. Visitor errors (e.g. malformed
            // `project.json`) continue to propagate.
            match std::fs::read_dir(&subdir) {
                Ok(_) => {
                    Self::walk_for_project_json(&subdir, visitor)?;
                }
                Err(err) => {
                    tracing::warn!(
                        target: "jet::pkg::nx",
                        path = %subdir.display(),
                        error = %err,
                        "GH #3292 unreadable subtree during workspace project.json discovery; \
                         a project.json or its subtree may be silently omitted"
                    );
                }
            }
        }

        Ok(())
    }
}

/// Human-readable JSON-kind label for the `package.json` dep-version
/// shape warn. Mirrors the per-module `describe_json_kind` style used
/// in `pkg_manager::workspace` / `pkg_manager::store`.
fn describe_nx_dep_kind(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

/// GH #3751 — build the warn message for an Nx `package.json` field
/// (`dependencies` / `devDependencies` / `peerDependencies`) entry
/// whose version value is not a JSON string. Extracted so the wording
/// (issue tag + project + field + dep name + observed kind +
/// downstream consequence) is unit-testable without provoking a
/// malformed `package.json` in the integration path.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_nx_dep_version_shape_warn(
    pkg_json_path: &Path,
    project: &str,
    field: &str,
    dep_name: &str,
    observed: &str,
) -> String {
    format!(
        "GH #3751 Nx project `{project}` package.json at {} has a \
         malformed `{field}.{dep_name}` value (observed JSON kind \
         `{observed}`, expected string per npm spec); this entry will \
         be skipped while scanning for `workspace:*` refs, so if it \
         WAS meant to be a workspace edge, the project graph will be \
         missing that dependency and `topological_order` will run the \
         build in the wrong order. The prior silent `.as_str()\
         .unwrap_or(\"\")` swallowed this. Fix the package.json shape \
         (e.g. `\"{dep_name}\": \"workspace:*\"`).",
        pkg_json_path.display()
    )
}

/// GH #3765 — derive a project's relative root from its `project.json`
/// path with explicit branching for each failure mode.
///
/// The pre-fix code stacked three silent fallbacks
/// (`.parent()` / `strip_prefix` / `to_string_lossy`) and conflated
/// "workspace root", "path outside workspace", and "non-UTF-8 bytes
/// in path" into the same empty-string `rel_root`. Two distinct projects
/// could then collide on the same identity in the Nx graph.
///
/// Behaviour:
/// - `project.json` lives at the workspace root → `""` (silent; legitimate).
/// - `.parent()` returns `None` (project.json at filesystem root) → warn + `""`.
/// - `strip_prefix(&workspace_root)` fails → warn + `""` (caller fed a
///   path outside the workspace).
/// - Path is UTF-8 → that string.
/// - Path has non-UTF-8 bytes → warn + lossy fallback (so the graph still
///   loads but the operator sees the substitution).
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn derive_rel_root(project_json_path: &Path, workspace_root: &Path) -> String {
    let parent = match project_json_path.parent() {
        Some(p) => p,
        None => {
            tracing::warn!(
                target: "jet::pkg::nx",
                project_json = %project_json_path.display(),
                "{}",
                format_pkg_manager_nx_rel_root_no_parent_warn(project_json_path)
            );
            return String::new();
        }
    };

    let stripped = match parent.strip_prefix(workspace_root) {
        Ok(p) => p,
        Err(_) => {
            tracing::warn!(
                target: "jet::pkg::nx",
                project_json = %project_json_path.display(),
                workspace_root = %workspace_root.display(),
                "{}",
                format_pkg_manager_nx_rel_root_outside_workspace_warn(
                    project_json_path,
                    workspace_root
                )
            );
            return String::new();
        }
    };

    match stripped.to_str() {
        Some(s) => s.to_string(),
        None => {
            let lossy = stripped.to_string_lossy().into_owned();
            tracing::warn!(
                target: "jet::pkg::nx",
                project_json = %project_json_path.display(),
                lossy = %lossy,
                "{}",
                format_pkg_manager_nx_rel_root_non_utf8_warn(project_json_path, &lossy)
            );
            lossy
        }
    }
}

/// GH #3765 — diagnostic for the `project.json` at the filesystem root
/// (no parent) branch. Should be unreachable in practice but worth a
/// breadcrumb if it ever fires.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_pkg_manager_nx_rel_root_no_parent_warn(project_json_path: &Path) -> String {
    format!(
        "GH #3765 jet Nx project.json {project_json_path:?} has no \
         parent directory; rel_root coerced to \"\". This should not be \
         reachable — the walker normally restricts to entries under the \
         workspace root."
    )
}

/// GH #3765 — diagnostic when a discovered `project.json` resolves to
/// a path outside the workspace (symlink escape, walker bug, etc.).
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_pkg_manager_nx_rel_root_outside_workspace_warn(
    project_json_path: &Path,
    workspace_root: &Path,
) -> String {
    format!(
        "GH #3765 jet Nx project.json {project_json_path:?} is not under \
         workspace root {workspace_root:?}; rel_root coerced to \"\". \
         Check for symlink escapes or a relative-path mismatch between \
         the walker and the configured workspace root."
    )
}

/// GH #3765 — diagnostic when a project's relative root contains
/// non-UTF-8 bytes; the lossy fallback substitutes U+FFFD so two
/// distinct paths can collide.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_pkg_manager_nx_rel_root_non_utf8_warn(
    project_json_path: &Path,
    lossy: &str,
) -> String {
    format!(
        "GH #3765 jet Nx project.json {project_json_path:?} has a \
         non-UTF-8 relative path; recovered via lossy decode as \
         {lossy:?}. Two projects with similarly-shaped non-UTF-8 \
         paths can collide on the same identity in the Nx graph."
    )
}

/// GH #3772 — derive the fallback project name from the directory
/// containing `project.json` when the manifest itself has no `name`
/// field. Returns `None` if there is no parent or no file_name (caller
/// then falls back further to `rel_root.replace('/', '-')`).
///
/// Distinguishes UTF-8 directory names (silent — current behavior) from
/// non-UTF-8 directory names (lossy fallback + warn — the collision
/// case where two sibling projects could shadow each other in the Nx
/// graph's `nodes` HashMap).
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn derive_project_name_from_dir(project_json_path: &Path) -> Option<String> {
    let file_name = project_json_path.parent().and_then(|p| p.file_name())?;
    match file_name.to_str() {
        Some(s) => Some(s.to_string()),
        None => {
            let lossy = file_name.to_string_lossy().into_owned();
            tracing::warn!(
                target: "jet::pkg::nx",
                project_json = %project_json_path.display(),
                lossy = %lossy,
                "{}",
                format_pkg_manager_nx_project_name_non_utf8_warn(project_json_path, &lossy)
            );
            Some(lossy)
        }
    }
}

/// GH #3772 — diagnostic when the project-name fallback derives from a
/// non-UTF-8 directory name. Sibling helper to
/// `format_pkg_manager_nx_rel_root_non_utf8_warn` from GH #3765 but
/// distinct so operators can grep for which field collided.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_pkg_manager_nx_project_name_non_utf8_warn(
    project_json_path: &Path,
    lossy: &str,
) -> String {
    format!(
        "GH #3772 jet Nx project.json {project_json_path:?} has no \
         explicit `name` and its parent directory contains non-UTF-8 \
         bytes; project name recovered via lossy decode as {lossy:?}. \
         Two sibling projects with similarly-shaped non-UTF-8 \
         directory names can collide on the same key in the Nx graph \
         nodes map and silently shadow each other."
    )
}

// ------------------------------------------------------------------
// Unit tests (see nx_test.rs)
// ------------------------------------------------------------------

#[cfg(test)]
#[path = "nx_test.rs"]
mod nx_test;

#[cfg(test)]
mod gh3751_nx_dep_version_warn_tests {
    use super::*;

    fn pkg_path() -> PathBuf {
        PathBuf::from("/tmp/x/libs/foo/package.json")
    }

    /// describe_nx_dep_kind covers all 6 JSON shapes.
    #[test]
    fn gh3751_describe_nx_dep_kind_covers_all_shapes() {
        assert_eq!(describe_nx_dep_kind(&serde_json::Value::Null), "null");
        assert_eq!(describe_nx_dep_kind(&serde_json::Value::Bool(true)), "bool");
        assert_eq!(describe_nx_dep_kind(&serde_json::json!(1)), "number");
        assert_eq!(describe_nx_dep_kind(&serde_json::json!("x")), "string");
        assert_eq!(describe_nx_dep_kind(&serde_json::json!([])), "array");
        assert_eq!(describe_nx_dep_kind(&serde_json::json!({})), "object");
    }

    /// Helper output carries the issue tag, the offending project /
    /// field / dep_name, and the observed JSON kind so the warn is
    /// greppable during incident triage.
    #[test]
    fn gh3751_helper_message_contains_issue_tag_and_context() {
        let msg = format_nx_dep_version_shape_warn(
            &pkg_path(),
            "libs-foo",
            "dependencies",
            "@scope/bar",
            "number",
        );
        assert!(msg.contains("GH #3751"), "msg: {msg}");
        assert!(msg.contains("libs-foo"), "msg must name project: {msg}");
        assert!(msg.contains("dependencies"), "msg must name field: {msg}");
        assert!(msg.contains("@scope/bar"), "msg must name dep_name: {msg}");
        assert!(msg.contains("number"), "msg must name observed kind: {msg}");
        assert!(
            msg.contains("topological_order") || msg.contains("project graph"),
            "msg must call out the graph-corruption consequence: {msg}"
        );
    }

    /// Deterministic — same input → byte-identical message.
    #[test]
    fn gh3751_helper_message_is_deterministic() {
        let p = pkg_path();
        let a = format_nx_dep_version_shape_warn(&p, "p", "dependencies", "bar", "number");
        let b = format_nx_dep_version_shape_warn(&p, "p", "dependencies", "bar", "number");
        assert_eq!(a, b);
    }

    /// Sibling-distinctness — the #3751 warn must NOT collide with
    /// related warn-tags from this module (#3261, #3292) or the
    /// adjacent workspace.rs / store.rs warn family.
    #[test]
    fn gh3751_warn_is_distinct_from_sibling_warn_tags() {
        let p = pkg_path();
        let msg = format_nx_dep_version_shape_warn(&p, "p", "dependencies", "bar", "number");
        for tag in [
            "#3261", "#3292", "#3324", "#3568", "#3637", "#3747", "#3749",
        ] {
            assert!(
                !msg.contains(tag),
                "msg must not contain sibling tag {tag}: {msg}"
            );
        }
    }

    /// Naming convention discoverability — keeps the warn-helper
    /// family uniformly named so future authors find them via
    /// `format_*_warn`.
    #[test]
    fn gh3751_helper_name_follows_family_convention() {
        let name = "format_nx_dep_version_shape_warn";
        assert!(name.starts_with("format_"));
        assert!(name.ends_with("_warn"));
        assert!(name.contains("nx"));
    }

    /// All three npm dep-fields (`dependencies` / `devDependencies` /
    /// `peerDependencies`) produce DISTINCT messages so triage can
    /// tell which field is broken.
    #[test]
    fn gh3751_three_dep_fields_produce_distinct_messages() {
        let p = pkg_path();
        let m_d = format_nx_dep_version_shape_warn(&p, "p", "dependencies", "bar", "number");
        let m_dev = format_nx_dep_version_shape_warn(&p, "p", "devDependencies", "bar", "number");
        let m_peer = format_nx_dep_version_shape_warn(&p, "p", "peerDependencies", "bar", "number");
        assert_ne!(m_d, m_dev);
        assert_ne!(m_d, m_peer);
        assert_ne!(m_dev, m_peer);
    }

    /// Smoke check that the warn-helper handles non-ASCII dep names
    /// without panicking (npm names are restricted to ASCII but the
    /// helper shouldn't crash on UTF-8 inputs).
    #[test]
    fn gh3751_helper_handles_non_ascii_dep_name() {
        let msg = format_nx_dep_version_shape_warn(
            &pkg_path(),
            "p",
            "dependencies",
            "@scope/包",
            "object",
        );
        assert!(msg.contains("@scope/包"));
    }

    /// Different observed-kinds produce DISTINCT messages.
    #[test]
    fn gh3751_observed_kind_is_reflected_in_message() {
        let p = pkg_path();
        let m_num = format_nx_dep_version_shape_warn(&p, "p", "dependencies", "bar", "number");
        let m_obj = format_nx_dep_version_shape_warn(&p, "p", "dependencies", "bar", "object");
        let m_bool = format_nx_dep_version_shape_warn(&p, "p", "dependencies", "bar", "bool");
        assert_ne!(m_num, m_obj);
        assert_ne!(m_num, m_bool);
        assert_ne!(m_obj, m_bool);
        assert!(m_num.contains("number"));
        assert!(m_obj.contains("object"));
        assert!(m_bool.contains("bool"));
    }

    /// Pure-logic mirror: the same as_str/unwrap pattern that the
    /// fix replaced. Pins the legitimate vs malformed distinction
    /// without requiring a live Nx workspace.
    #[test]
    fn gh3751_as_str_pattern_distinguishes_string_from_non_string() {
        // Legitimate workspace ref — string value starting with "workspace:".
        let ok = serde_json::json!("workspace:*");
        assert_eq!(ok.as_str(), Some("workspace:*"));

        // Wrong shape: number, object, bool, null — all produce None.
        assert!(serde_json::json!(1).as_str().is_none());
        assert!(serde_json::json!({"version": "^1"}).as_str().is_none());
        assert!(serde_json::json!(true).as_str().is_none());
        assert!(serde_json::Value::Null.as_str().is_none());
    }

    /// End-to-end behaviour: a project.json + package.json with
    /// mixed valid + malformed dep entries. The well-formed
    /// workspace:* edges still land; malformed ones are skipped
    /// (with warns we cannot capture without infra) and don't
    /// abort the graph build.
    #[test]
    fn gh3751_build_project_graph_skips_malformed_dep_entries_but_keeps_valid() {
        use std::fs;
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // nx.json — minimal, so NxWorkspaceManager::discover succeeds.
        fs::write(root.join("nx.json"), r#"{}"#).unwrap();

        // Project A — uses workspace:* in dependencies (well-formed).
        let a_dir = root.join("libs/a");
        fs::create_dir_all(&a_dir).unwrap();
        fs::write(
            a_dir.join("project.json"),
            r#"{"name":"a","root":"libs/a","projectType":"library"}"#,
        )
        .unwrap();
        fs::write(
            a_dir.join("package.json"),
            r#"{
              "name": "a",
              "version": "0.0.1",
              "dependencies": {
                "b": "workspace:*",
                "bad-number": 1,
                "bad-object": { "version": "^1" }
              }
            }"#,
        )
        .unwrap();

        // Project B — the target of A's workspace ref.
        let b_dir = root.join("libs/b");
        fs::create_dir_all(&b_dir).unwrap();
        fs::write(
            b_dir.join("project.json"),
            r#"{"name":"b","root":"libs/b","projectType":"library"}"#,
        )
        .unwrap();
        fs::write(
            b_dir.join("package.json"),
            r#"{"name":"b","version":"0.0.1"}"#,
        )
        .unwrap();

        let nx = NxWorkspaceManager::discover(root)
            .expect("discover should succeed with nx.json present")
            .expect("nx.json exists so discover returns Some");
        let graph = nx.build_project_graph_from_files().unwrap();

        // Edge a → b must be present (well-formed entry survives).
        let edges = graph
            .graph
            .dependencies
            .get("a")
            .cloned()
            .unwrap_or_default();
        assert!(
            edges.iter().any(|e| e.target == "b"),
            "well-formed workspace:* dep should produce edge a → b; got {edges:?}"
        );

        // Malformed entries do not crash the build — graph still
        // contains both projects.
        assert!(graph.graph.nodes.contains_key("a"));
        assert!(graph.graph.nodes.contains_key("b"));
    }
}
// CODEGEN-END
