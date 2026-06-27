// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
// CODEGEN-BEGIN
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use super::nx::NxWorkspaceManager;

/// pnpm-workspace.yaml format (third-priority config source).
#[derive(Debug, Clone, Default, Deserialize)]
struct PnpmWorkspaceYaml {
    #[serde(default)]
    packages: Vec<String>,
    #[serde(default)]
    catalog: HashMap<String, String>,
    #[serde(default)]
    catalogs: HashMap<String, HashMap<String, String>>,
}

/// Workspace configuration from package.json or jet-workspace.yaml.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub catalog: HashMap<String, String>,
    #[serde(default)]
    pub hoisting: HoistingConfig,
}

/// Hoisting configuration for node_modules layout.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoistingConfig {
    #[serde(default)]
    pub shamefully_hoist: bool,
    #[serde(default = "default_hoist_patterns")]
    pub public_hoist_pattern: Vec<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl Default for HoistingConfig {
    fn default() -> Self {
        Self {
            shamefully_hoist: false,
            public_hoist_pattern: default_hoist_patterns(),
        }
    }
}

fn default_hoist_patterns() -> Vec<String> {
    vec!["*eslint*".to_string(), "*prettier*".to_string()]
}

/// A discovered workspace package.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone)]
pub struct WorkspacePackage {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
    pub deps_on_workspace: Vec<String>,
}

/// Manages workspace discovery, dependency graph, and protocol resolution.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub struct WorkspaceManager {
    pub root: PathBuf,
    pub config: WorkspaceConfig,
    pub packages: Vec<WorkspacePackage>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl WorkspaceManager {
    /// Discover workspace from project root.
    pub fn discover(root: &Path) -> Result<Option<Self>> {
        let config = Self::load_config(root)?;
        let Some(config) = config else {
            return Ok(None);
        };

        let packages = Self::expand_packages(root, &config)?;
        Ok(Some(Self {
            root: root.to_path_buf(),
            config,
            packages,
        }))
    }

    /// Load workspace config from jet-workspace.yaml, package.json.workspaces,
    /// or pnpm-workspace.yaml (in that priority order).
    fn load_config(root: &Path) -> Result<Option<WorkspaceConfig>> {
        // 1. Try jet-workspace.yaml first (highest priority)
        let yaml_path = root.join("jet-workspace.yaml");
        if yaml_path.exists() {
            let content = std::fs::read_to_string(&yaml_path)
                .with_context(|| workspace_config_io_ctx(&yaml_path))?;
            let config: WorkspaceConfig = serde_yaml::from_str(&content)
                .with_context(|| workspace_config_parse_ctx(&yaml_path))?;
            return Ok(Some(config));
        }

        // 2. Fall back to package.json.workspaces
        let pkg_path = root.join("package.json");
        if pkg_path.exists() {
            let content = std::fs::read_to_string(&pkg_path)
                .with_context(|| workspace_config_io_ctx(&pkg_path))?;
            let content = strip_aw_claim_wrapper_lines(&content);
            let pkg: serde_json::Value = serde_json::from_str(content.as_ref())
                .with_context(|| workspace_config_parse_ctx(&pkg_path))?;
            if let Some(workspaces) = pkg.get("workspaces") {
                let patterns = parse_workspaces_field(workspaces).map_err(|shape| {
                    anyhow::anyhow!(
                        "Invalid package.json workspaces field at {}: {}",
                        pkg_path.display(),
                        shape
                    )
                })?;
                if !patterns.is_empty() {
                    return Ok(Some(WorkspaceConfig {
                        packages: patterns,
                        ..Default::default()
                    }));
                }
            }
        }

        // 3. Fall back to pnpm-workspace.yaml (lowest priority)
        let pnpm_yaml_path = root.join("pnpm-workspace.yaml");
        if pnpm_yaml_path.exists() {
            let content = std::fs::read_to_string(&pnpm_yaml_path)
                .with_context(|| workspace_config_io_ctx(&pnpm_yaml_path))?;
            let pnpm: PnpmWorkspaceYaml = serde_yaml::from_str(&content)
                .with_context(|| workspace_config_parse_ctx(&pnpm_yaml_path))?;
            let mut catalog = pnpm.catalog;
            // Merge named catalogs with key prefix "<catalog_name>:"
            for (catalog_name, entries) in pnpm.catalogs {
                for (dep_name, version) in entries {
                    catalog.insert(format!("{}:{}", catalog_name, dep_name), version);
                }
            }
            return Ok(Some(WorkspaceConfig {
                packages: pnpm.packages,
                catalog,
                ..Default::default()
            }));
        }

        Ok(None)
    }

    /// Expand glob patterns to find workspace packages.
    fn expand_packages(root: &Path, config: &WorkspaceConfig) -> Result<Vec<WorkspacePackage>> {
        let mut packages = Vec::new();

        for pattern in &config.packages {
            let full_pattern = root.join(pattern).join("package.json");
            let pattern_str = full_pattern.to_string_lossy().to_string();

            for entry in glob::glob(&pattern_str)
                .with_context(|| format!("Invalid glob pattern: {}", pattern))?
            {
                let pkg_json_path = entry?;
                let pkg_dir = pkg_json_path.parent().unwrap();
                // GH #3524 — surface workspace package.json read /
                // parse failures instead of dropping them silently.
                // A malformed package.json (trailing comma, unclosed
                // brace, EACCES) used to make the affected workspace
                // member silently disappear from the resolved
                // workspace, with the resolver later complaining
                // "workspace:* not found" — and the dev chasing the
                // wrong code.
                match Self::read_workspace_package(root, pkg_dir) {
                    Ok(pkg) => packages.push(pkg),
                    Err(err) => {
                        tracing::warn!(
                            target: "jet::pkg_manager::workspace",
                            package_json = %pkg_json_path.display(),
                            error = %err,
                            "{}",
                            format_workspace_pkg_warn(&pkg_json_path, &err)
                        );
                    }
                }
            }
        }

        Ok(packages)
    }

    /// Read a single workspace package.
    fn read_workspace_package(root: &Path, pkg_dir: &Path) -> Result<WorkspacePackage> {
        let pkg_path = pkg_dir.join("package.json");
        let content = std::fs::read_to_string(&pkg_path)?;
        let pkg: serde_json::Value = serde_json::from_str(&content)?;

        // GH #3580 — the prior `as_str().unwrap_or("unnamed" / "0.0.0")`
        // silently substituted placeholders for missing/non-string
        // `name` and `version` fields. Multiple packages with a bad
        // `name` then collapsed onto the same "unnamed" key in
        // `WorkspaceManager::topological_order`'s HashSet of names,
        // so distinct packages were treated as one node and the
        // workspace dependency graph was silently corrupted. Refuse
        // to load the package instead, naming the offending file,
        // the missing field, and the observed JSON kind.
        let name = require_workspace_string_field(&pkg, "name", &pkg_path)?;
        let version = require_workspace_string_field(&pkg, "version", &pkg_path)?;
        // GH #3628 — prior `pkg_dir.strip_prefix(root).unwrap_or(pkg_dir)`
        // silently leaked an absolute `pkg_dir` into `WorkspacePackage::path`
        // when the package directory was outside `root` (symlink escape,
        // future canonicalize step). The abs path then propagated to
        // `ws_root.join(...)` (which silently discards `ws_root` when the
        // RHS is absolute) and to the on-disk `jet-lock.yaml`'s
        // `local_path` field.
        let (rel_path, warn) = safe_workspace_relative_path(pkg_dir, root);
        if let Some(msg) = warn {
            tracing::warn!(
                target: "jet::pkg_manager::workspace",
                "{}",
                msg
            );
        }

        let dependencies = Self::extract_deps(&pkg, "dependencies", &pkg_path);
        let dev_dependencies = Self::extract_deps(&pkg, "devDependencies", &pkg_path);

        Ok(WorkspacePackage {
            name,
            version,
            path: rel_path,
            dependencies,
            dev_dependencies,
            deps_on_workspace: Vec::new(),
        })
    }

    /// GH #3747 — extract a `dependencies` / `devDependencies` map from
    /// a workspace `package.json`.
    ///
    /// The prior body was `pkg.get(field).and_then(|v|
    /// serde_json::from_value(v.clone()).ok()).unwrap_or_default()`,
    /// which silently swallowed every shape mismatch. That conflated:
    ///
    /// 1. Field absent — legitimate; package has no dependencies.
    /// 2. Field present but the wrong shape — e.g. `"dependencies":
    ///    "latest"` (string), `"dependencies": []` (array), or
    ///    `"dependencies": { "foo": 1 }` (non-string version). In that
    ///    case the package now looks like it has no deps, so the
    ///    workspace dependency graph in `topological_order` is silently
    ///    corrupted, `jet-lock.yaml` will be missing entries, and
    ///    downstream installs/builds fail in obscure ways far from the
    ///    parse site.
    ///
    /// Case 1 stays silent; case 2 emits `tracing::warn!` and still
    /// returns an empty map (the dep load is not allowed to abort the
    /// whole workspace scan — corrupt deps for one package shouldn't
    /// block discovering siblings).
    fn extract_deps(
        pkg: &serde_json::Value,
        field: &str,
        pkg_path: &Path,
    ) -> HashMap<String, String> {
        let Some(value) = pkg.get(field) else {
            return HashMap::new();
        };
        match serde_json::from_value::<HashMap<String, String>>(value.clone()) {
            Ok(map) => map,
            Err(err) => {
                let observed = describe_json_kind(value);
                tracing::warn!(
                    target: "jet::pkg_manager::workspace",
                    package = %pkg_path.display(),
                    field = %field,
                    observed_type = %observed,
                    error = %err,
                    "{}",
                    format_workspace_deps_shape_warn(pkg_path, field, observed)
                );
                HashMap::new()
            }
        }
    }

    /// Build topological order of workspace packages based on inter-dependencies.
    pub fn topological_order(&mut self) -> Result<Vec<String>> {
        let names: HashSet<String> = self.packages.iter().map(|p| p.name.clone()).collect();

        // Mark workspace deps
        for pkg in &mut self.packages {
            let mut ws_deps = Vec::new();
            for dep_name in pkg.dependencies.keys().chain(pkg.dev_dependencies.keys()) {
                if names.contains(dep_name) {
                    ws_deps.push(dep_name.clone());
                }
            }
            pkg.deps_on_workspace = ws_deps;
        }

        // Kahn's algorithm
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut dependents: HashMap<String, Vec<String>> = HashMap::new();

        for pkg in &self.packages {
            in_degree.entry(pkg.name.clone()).or_insert(0);
            for dep in &pkg.deps_on_workspace {
                *in_degree.entry(pkg.name.clone()).or_insert(0) += 1;
                dependents
                    .entry(dep.clone())
                    .or_default()
                    .push(pkg.name.clone());
            }
        }

        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(name, _)| name.clone())
            .collect();

        let mut order = Vec::new();
        while let Some(name) = queue.pop_front() {
            order.push(name.clone());
            if let Some(deps) = dependents.get(&name) {
                for dep in deps {
                    if let Some(deg) = in_degree.get_mut(dep) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(dep.clone());
                        }
                    }
                }
            }
        }

        if order.len() != self.packages.len() {
            anyhow::bail!("Circular dependency detected in workspace packages");
        }

        Ok(order)
    }

    /// Resolve workspace:* protocol to actual version.
    pub fn resolve_workspace_protocol(&self, spec: &str, target_name: &str) -> Option<String> {
        let pkg = self.packages.iter().find(|p| p.name == target_name)?;
        let version = &pkg.version;

        match spec {
            "workspace:*" => Some(version.clone()),
            "workspace:^" => Some(format!("^{}", version)),
            "workspace:~" => Some(format!("~{}", version)),
            _ => None,
        }
    }

    /// Get a package by name.
    pub fn get_package(&self, name: &str) -> Option<&WorkspacePackage> {
        self.packages.iter().find(|p| p.name == name)
    }

    /// Get catalog version for a dependency, if defined.
    pub fn catalog_version(&self, dep_name: &str) -> Option<&str> {
        self.config.catalog.get(dep_name).map(|s| s.as_str())
    }

    /// Check if a dependency spec is a workspace protocol.
    pub fn is_workspace_protocol(spec: &str) -> bool {
        spec.starts_with("workspace:")
    }
}

// ------------------------------------------------------------------
// Workspace mode detection
// ------------------------------------------------------------------

/// The workspace mode that Jet is operating in.
///
/// Detection order (matches the spec flowchart):
/// 1. `nx.json` present → [`WorkspaceMode::Nx`]
/// 2. `package.json workspaces` or `jet-workspace.yaml` → [`WorkspaceMode::Jet`]
/// 3. Neither → [`WorkspaceMode::Single`]
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub enum WorkspaceMode {
    /// Nx monorepo: `nx.json` was found and parsed successfully.
    Nx(NxWorkspaceManager),
    /// Jet / npm workspaces: `jet-workspace.yaml` or `package.json.workspaces` found.
    Jet(WorkspaceManager),
    /// Plain single-project repository; no workspace configuration detected.
    Single,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl WorkspaceMode {
    /// Detect the workspace mode for the given repository root.
    ///
    /// # Errors
    /// Returns an error if `nx.json` or `jet-workspace.yaml` is present but
    /// malformed.
    pub fn detect(root: &Path) -> Result<Self> {
        // 1. Nx takes highest priority.
        if let Some(nx_mgr) = NxWorkspaceManager::discover(root)? {
            return Ok(WorkspaceMode::Nx(nx_mgr));
        }

        // 2. Jet / npm workspaces.
        if let Some(ws_mgr) = WorkspaceManager::discover(root)? {
            return Ok(WorkspaceMode::Jet(ws_mgr));
        }

        // 3. Single project.
        Ok(WorkspaceMode::Single)
    }
}

/// GH #3524 — build the warn message for a workspace `package.json`
/// read / parse failure during glob expansion. Extracted so the
/// wording is unit-testable without provoking a real malformed
/// `package.json` in the integration path.
///
/// Names the exact `package.json` path so the dev can `cat` it,
/// preserves the underlying error (serde_json line/column, io::Error),
/// and explains the "silently disappears from workspace" symptom that
/// makes this bug class hard to debug from the resolver side.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_workspace_pkg_warn(pkg_json_path: &Path, err: &anyhow::Error) -> String {
    format!(
        "GH #3524 workspace package.json read failed at {}: {err}; \
         this package will silently disappear from the workspace and \
         later `workspace:` references will fail to resolve. Common \
         causes: malformed JSON (trailing comma / unclosed brace), \
         EACCES, UTF-8 decoding error.",
        pkg_json_path.display()
    )
}

/// GH #3747 — build the warn message for a workspace `package.json`
/// `dependencies` / `devDependencies` field whose JSON shape doesn't
/// match `HashMap<String, String>`. Extracted so the wording (issue
/// tag + field name + observed-shape + downstream consequence) is
/// unit-testable without provoking a malformed `package.json` in the
/// integration path.
///
/// `field` is the package.json field that was malformed (typically
/// `"dependencies"` or `"devDependencies"`). `observed` is one of the
/// `describe_json_kind` labels (e.g. `"string"`, `"array"`, `"null"`).
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_workspace_deps_shape_warn(
    pkg_path: &Path,
    field: &str,
    observed: &str,
) -> String {
    format!(
        "GH #3747 workspace package.json at {} has a malformed `{field}` \
         field (observed JSON kind `{observed}`, expected object of \
         string → string); jet will treat this package as having no \
         {field} for workspace topology purposes. The prior silent \
         `.ok()` swallowed this, causing wrong `topological_order` \
         output and missing entries in `jet-lock.yaml`. Fix the \
         package.json shape to restore graph correctness.",
        pkg_path.display()
    )
}

/// Parse the `workspaces` field of a `package.json` into a list of glob
/// patterns, supporting the three forms seen in the wild (GH #3560):
///
/// 1. **Array of strings** — `"workspaces": ["packages/*", "apps/*"]`
///    (npm / yarn modern). Patterns returned verbatim.
/// 2. **Object form** — `"workspaces": { "packages": ["pkg/*"], "nohoist": [...] }`
///    (yarn 1; still in the wild in projects migrating from yarn 1 or
///    keeping the legacy shape). Inner `packages` array is used.
/// 3. **String form** — `"workspaces": "pkg/*"` (single glob). Wrapped
///    into a one-element vec for graceful upgrade.
///
/// Anything else returns `Err(shape_description)` where `shape_description`
/// names the JSON kind that was encountered (e.g. "number", "null",
/// "object without `packages` array"). The caller turns that into a warn.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn parse_workspaces_field(
    workspaces: &serde_json::Value,
) -> Result<Vec<String>, String> {
    match workspaces {
        // 1. Array of strings → verbatim.
        serde_json::Value::Array(arr) => {
            let mut patterns = Vec::with_capacity(arr.len());
            for (i, v) in arr.iter().enumerate() {
                match v.as_str() {
                    Some(s) => patterns.push(s.to_string()),
                    None => {
                        return Err(format!(
                            "array element at index {i} is not a string (got {})",
                            describe_json_kind(v)
                        ));
                    }
                }
            }
            Ok(patterns)
        }
        // 2. Object form → take `packages` array.
        serde_json::Value::Object(obj) => {
            match obj.get("packages") {
                Some(serde_json::Value::Array(arr)) => {
                    let mut patterns = Vec::with_capacity(arr.len());
                    for (i, v) in arr.iter().enumerate() {
                        match v.as_str() {
                            Some(s) => patterns.push(s.to_string()),
                            None => {
                                return Err(format!(
                                    "object form `packages` array element at index {i} is not a string (got {})",
                                    describe_json_kind(v)
                                ));
                            }
                        }
                    }
                    Ok(patterns)
                }
                Some(other) => Err(format!(
                    "object form `packages` field is not an array (got {})",
                    describe_json_kind(other)
                )),
                None => Err(
                    "object without `packages` array (yarn-1 object form requires a `packages` field)"
                        .to_string(),
                ),
            }
        }
        // 3. String → wrap.
        serde_json::Value::String(s) => Ok(vec![s.clone()]),
        // Anything else (number, bool, null) is unsupported.
        other => Err(format!(
            "unsupported `workspaces` JSON kind: {} (expected array of strings, object with `packages`, or a single string)",
            describe_json_kind(other)
        )),
    }
}

/// Human-readable JSON-kind label for use in `parse_workspaces_field`
/// error strings — picks one of `"null" | "bool" | "number" | "string"
/// | "array" | "object"` so the warn message is grep-friendly.
fn describe_json_kind(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

/// Context string for IO failures while reading a workspace-config file
/// (`jet-workspace.yaml`, `package.json`, or `pnpm-workspace.yaml`).
/// Tagged `GH #3560`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn workspace_config_io_ctx(path: &Path) -> String {
    format!(
        "GH #3560 reading workspace config from {} failed (e.g. EACCES, EIO); jet will fall through to the next workspace-config candidate or treat the project as non-workspace",
        path.display()
    )
}

/// Context string for YAML/JSON parse failures while loading a
/// workspace-config file. Tagged `GH #3560`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn workspace_config_parse_ctx(path: &Path) -> String {
    format!(
        "GH #3560 parsing workspace config at {} failed; the file is on disk but malformed (typical cause: stray merge markers, trailing commas, or unclosed brace/bracket). Fix the syntax to re-enable workspace discovery",
        path.display()
    )
}

/// Strip AW source-ownership wrapper lines from JSON fixtures before parsing.
///
/// This intentionally accepts only the exact line-comment markers emitted by
/// standardization (`// <HANDWRITE ...>` and `// </HANDWRITE>`). General JSON
/// comments remain invalid so real malformed `package.json` files still fail.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn strip_aw_claim_wrapper_lines(content: &str) -> std::borrow::Cow<'_, str> {
    if !content.contains("// <HANDWRITE") && !content.contains("// </HANDWRITE>") {
        return std::borrow::Cow::Borrowed(content);
    }
    let mut stripped = String::with_capacity(content.len());
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("// <HANDWRITE ") || trimmed == "// </HANDWRITE>" {
            continue;
        }
        stripped.push_str(line);
        stripped.push('\n');
    }
    if !content.ends_with('\n') {
        stripped.pop();
    }
    std::borrow::Cow::Owned(stripped)
}

/// GH #3580 — require the workspace package's `name`/`version` to be a
/// non-empty string. Returns the value on success; on failure returns
/// an `anyhow::Error` whose Display names the offending package.json
/// path, the field, and the observed JSON kind so the user can grep
/// for the cause.
///
/// Replaces the prior `pkg["{field}"].as_str().unwrap_or("unnamed" /
/// "0.0.0")` silent fallback, which collapsed distinct workspace
/// packages onto the same name key and silently corrupted the
/// `topological_order` dep graph.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn require_workspace_string_field(
    pkg: &serde_json::Value,
    field: &str,
    pkg_path: &Path,
) -> Result<String> {
    let observed_kind = match pkg.get(field) {
        Some(serde_json::Value::String(s)) if s.is_empty() => "empty-string",
        Some(serde_json::Value::String(s)) => return Ok(s.clone()),
        Some(other) => describe_workspace_field_kind(other),
        None => "missing",
    };
    Err(anyhow::anyhow!(
        "{}",
        format_workspace_identity_err(pkg_path, field, observed_kind)
    ))
}

/// GH #3580 — build the error message for a workspace `name`/`version`
/// field that is missing, non-string, or empty. Extracted so the
/// wording (path + field + kind + tag) is unit-testable without
/// provoking the actual filesystem case.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_workspace_identity_err(
    pkg_path: &Path,
    field: &str,
    observed_kind: &str,
) -> String {
    format!(
        "GH #3580 cannot load workspace package: package.json at {} has \
         no non-empty string `{field}` field (observed: {observed_kind}). \
         A missing or placeholder `{field}` silently collides distinct \
         workspace packages onto the same dependency-graph node. Add a \
         valid `\"{field}\": \"…\"` entry and retry.",
        pkg_path.display()
    )
}

/// GH #3628 — `read_workspace_package` previously did
/// `pkg_dir.strip_prefix(root).unwrap_or(pkg_dir)`. When the package
/// directory was outside the workspace root (symlink escape, future
/// canonicalize step), the fallback silently leaked the absolute path
/// into `WorkspacePackage::path` — which is later concatenated via
/// `ws_root.join(&pkg.path)`. `Path::join` with an absolute RHS
/// discards the LHS, so all on-disk reads, symlink targets, and the
/// `local_path` field shipped in `jet-lock.yaml` end up referring to
/// the outside-root directory verbatim.
///
/// Returns the relative path on success. On failure (outside-root)
/// falls back to the bare file name and returns a warning string so
/// the caller can route it through its own tracing target.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn safe_workspace_relative_path(
    pkg_dir: &Path,
    root: &Path,
) -> (PathBuf, Option<String>) {
    match pkg_dir.strip_prefix(root) {
        Ok(rel) => (rel.to_path_buf(), None),
        Err(err) => {
            let warn = format_safe_workspace_relative_path_warn(pkg_dir, root, &err);
            match pkg_dir.file_name() {
                Some(name) => (PathBuf::from(name), Some(warn)),
                None => (pkg_dir.to_path_buf(), Some(warn)),
            }
        }
    }
}

/// GH #3628 — format the warn line for a workspace package whose
/// directory is outside the workspace root. Names the offending
/// directory, the workspace root, the underlying `StripPrefixError`,
/// the GH #3628 tag, and the operator-visible consequence (lockfile
/// `local_path` would have leaked, symlink target would have escaped).
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_safe_workspace_relative_path_warn(
    pkg_dir: &Path,
    root: &Path,
    err: &std::path::StripPrefixError,
) -> String {
    format!(
        "GH #3628 workspace package directory {pkg_dir:?} is not under workspace root {root:?} ({err}); \
         falling back to the bare file name as `WorkspacePackage::path` to avoid leaking the absolute path \
         into `jet-lock.yaml` `local_path` and into `ws_root.join(...)` (which would silently discard the \
         workspace root). Move the package under the workspace root, or remove the offending pattern from \
         your `workspaces` config."
    )
}

/// GH #3580 — describe the JSON kind of a value for the workspace
/// identity error message so the dev can tell from the message what
/// was actually observed at the field slot.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn describe_workspace_field_kind(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // ─── GH #3524: workspace expander silent skip ────────────────────────

    /// GH #3524 — the warn message must name the exact package.json
    /// path, preserve the underlying error verbatim, AND include the
    /// GH #3524 tag so the dev has a direct breadcrumb to the bad file.
    #[test]
    fn gh3524_workspace_pkg_warn_names_path_error_and_issue() {
        let path = Path::new("/proj/packages/internal/package.json");
        let err = anyhow::anyhow!("trailing comma at line 12 column 5");
        let msg = format_workspace_pkg_warn(path, &err);

        assert!(
            msg.contains("/proj/packages/internal/package.json"),
            "must name exact package.json path, got: {msg}"
        );
        assert!(
            msg.contains("trailing comma at line 12 column 5"),
            "must preserve underlying error verbatim, got: {msg}"
        );
        assert!(
            msg.contains("GH #3524"),
            "must include searchable issue tag, got: {msg}"
        );
        assert!(
            msg.contains("silently disappear"),
            "must explain user-visible symptom, got: {msg}"
        );
    }

    /// GH #3524 — the message must list common causes so the dev does
    /// not have to guess. Malformed JSON, EACCES, and UTF-8 are the
    /// three failure shapes; at least one must appear.
    #[test]
    fn gh3524_workspace_pkg_warn_hints_common_causes() {
        let path = Path::new("/proj/packages/a/package.json");
        let err = anyhow::anyhow!("io error");
        let msg = format_workspace_pkg_warn(path, &err);

        assert!(
            msg.contains("JSON") || msg.contains("EACCES") || msg.contains("UTF-8"),
            "must list a common cause, got: {msg}"
        );
    }

    /// GH #3524 — drive the full pipeline: create a temp workspace
    /// with one valid and one malformed package.json, expand globs,
    /// confirm only the valid one survives. (Behavioural test for the
    /// fix; the warn was emitted but tracing capture isn't wired up
    /// here — the message contract is pinned by the formatter tests
    /// above.)
    #[test]
    fn gh3524_expand_packages_skips_malformed_package_json() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("packages/good")).unwrap();
        std::fs::create_dir_all(root.join("packages/bad")).unwrap();
        std::fs::write(
            root.join("packages/good/package.json"),
            r#"{"name":"good","version":"1.0.0"}"#,
        )
        .unwrap();
        // Trailing comma → serde_json fails.
        std::fs::write(
            root.join("packages/bad/package.json"),
            r#"{"name":"bad","version":"1.0.0",}"#,
        )
        .unwrap();

        let config = WorkspaceConfig {
            packages: vec!["packages/*".to_string()],
            ..Default::default()
        };
        let pkgs = WorkspaceManager::expand_packages(root, &config)
            .expect("expand_packages must succeed even with bad package");
        let names: Vec<&str> = pkgs.iter().map(|p| p.name.as_str()).collect();
        assert!(
            names.contains(&"good"),
            "valid workspace member must survive, got: {names:?}"
        );
        assert!(
            !names.contains(&"bad"),
            "malformed package must be skipped, got: {names:?}"
        );
    }

    #[test]
    fn test_workspace_protocol_resolution() {
        let ws = WorkspaceManager {
            root: PathBuf::from("/tmp"),
            config: WorkspaceConfig::default(),
            packages: vec![WorkspacePackage {
                name: "pkg-a".to_string(),
                version: "1.2.3".to_string(),
                path: PathBuf::from("packages/pkg-a"),
                dependencies: HashMap::new(),
                dev_dependencies: HashMap::new(),
                deps_on_workspace: Vec::new(),
            }],
        };

        assert_eq!(
            ws.resolve_workspace_protocol("workspace:*", "pkg-a"),
            Some("1.2.3".to_string())
        );
        assert_eq!(
            ws.resolve_workspace_protocol("workspace:^", "pkg-a"),
            Some("^1.2.3".to_string())
        );
        assert_eq!(
            ws.resolve_workspace_protocol("workspace:~", "pkg-a"),
            Some("~1.2.3".to_string())
        );
    }

    #[test]
    fn test_is_workspace_protocol() {
        assert!(WorkspaceManager::is_workspace_protocol("workspace:*"));
        assert!(WorkspaceManager::is_workspace_protocol("workspace:^"));
        assert!(!WorkspaceManager::is_workspace_protocol("^1.0.0"));
    }

    #[test]
    fn test_discover_no_workspace() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"{"name": "test", "version": "1.0.0"}"#,
        )
        .unwrap();

        let result = WorkspaceManager::discover(dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn aw_wrapped_package_json_workspaces_are_detected() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("package.json"),
            r#"// <HANDWRITE gap="standardize:claim-code" tracker="pkg-json">
{
  "name": "wrapped-root",
  "version": "1.0.0",
  "workspaces": ["packages/*"]
}
// </HANDWRITE>
"#,
        )
        .unwrap();
        std::fs::create_dir_all(dir.path().join("packages/ui")).unwrap();
        std::fs::write(
            dir.path().join("packages/ui/package.json"),
            r#"{"name": "ui", "version": "1.0.0"}"#,
        )
        .unwrap();

        let result = WorkspaceManager::discover(dir.path()).unwrap();
        let wm = result.expect("AW-wrapped package.json workspaces should parse");
        assert_eq!(wm.packages.len(), 1);
        assert_eq!(wm.packages[0].name, "ui");
    }

    #[test]
    fn test_topological_order() {
        let mut ws = WorkspaceManager {
            root: PathBuf::from("/tmp"),
            config: WorkspaceConfig::default(),
            packages: vec![
                WorkspacePackage {
                    name: "pkg-b".to_string(),
                    version: "1.0.0".to_string(),
                    path: PathBuf::from("packages/pkg-b"),
                    dependencies: HashMap::from([("pkg-a".to_string(), "workspace:*".to_string())]),
                    dev_dependencies: HashMap::new(),
                    deps_on_workspace: Vec::new(),
                },
                WorkspacePackage {
                    name: "pkg-a".to_string(),
                    version: "1.0.0".to_string(),
                    path: PathBuf::from("packages/pkg-a"),
                    dependencies: HashMap::new(),
                    dev_dependencies: HashMap::new(),
                    deps_on_workspace: Vec::new(),
                },
            ],
        };

        let order = ws.topological_order().unwrap();
        assert_eq!(order, vec!["pkg-a", "pkg-b"]);
    }

    #[test]
    fn test_hoisting_defaults() {
        let config = HoistingConfig::default();
        assert!(!config.shamefully_hoist);
        assert_eq!(config.public_hoist_pattern.len(), 2);
    }

    // ------------------------------------------------------------------
    // Tests for pnpm-workspace.yaml detection (R1, R2, S1, S2, S3)
    // ------------------------------------------------------------------

    #[test]
    fn test_pnpm_workspace_yaml_discovery() {
        let dir = tempdir().unwrap();

        // Only pnpm-workspace.yaml — no jet-workspace.yaml, no package.json.workspaces
        std::fs::write(
            dir.path().join("pnpm-workspace.yaml"),
            "packages:\n  - packages/*\n",
        )
        .unwrap();

        // Create workspace package for glob to find
        std::fs::create_dir_all(dir.path().join("packages/ui")).unwrap();
        std::fs::write(
            dir.path().join("packages/ui/package.json"),
            r#"{"name": "ui", "version": "1.0.0"}"#,
        )
        .unwrap();

        let result = WorkspaceManager::discover(dir.path()).unwrap();
        assert!(
            result.is_some(),
            "WorkspaceManager should discover pnpm-workspace.yaml"
        );
        let wm = result.unwrap();
        assert_eq!(wm.packages.len(), 1);
        assert_eq!(wm.packages[0].name, "ui");
    }

    #[test]
    fn test_jet_workspace_yaml_priority() {
        let dir = tempdir().unwrap();

        // jet-workspace.yaml lists apps/*, pnpm-workspace.yaml lists packages/*
        std::fs::write(
            dir.path().join("jet-workspace.yaml"),
            "packages:\n  - apps/*\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("pnpm-workspace.yaml"),
            "packages:\n  - packages/*\n",
        )
        .unwrap();

        // Create a package in apps/ (should be found)
        std::fs::create_dir_all(dir.path().join("apps/web")).unwrap();
        std::fs::write(
            dir.path().join("apps/web/package.json"),
            r#"{"name": "web", "version": "1.0.0"}"#,
        )
        .unwrap();

        // Create a package in packages/ (should NOT be found; pnpm yaml ignored)
        std::fs::create_dir_all(dir.path().join("packages/lib")).unwrap();
        std::fs::write(
            dir.path().join("packages/lib/package.json"),
            r#"{"name": "lib", "version": "2.0.0"}"#,
        )
        .unwrap();

        let result = WorkspaceManager::discover(dir.path()).unwrap();
        assert!(result.is_some());
        let wm = result.unwrap();
        // Only the app from jet-workspace.yaml should be discovered
        assert_eq!(
            wm.packages.len(),
            1,
            "jet-workspace.yaml should win over pnpm-workspace.yaml"
        );
        assert_eq!(wm.packages[0].name, "web");
    }

    #[test]
    fn test_pnpm_catalog_default() {
        let dir = tempdir().unwrap();

        let yaml =
            "packages:\n  - packages/*\ncatalog:\n  react: \"^18.0.0\"\n  typescript: \"^5.0.0\"\n";
        std::fs::write(dir.path().join("pnpm-workspace.yaml"), yaml).unwrap();

        let result = WorkspaceManager::discover(dir.path()).unwrap();
        assert!(result.is_some());
        let wm = result.unwrap();

        assert_eq!(
            wm.catalog_version("react"),
            Some("^18.0.0"),
            "Default catalog entry should be accessible via catalog_version()"
        );
        assert_eq!(
            wm.catalog_version("typescript"),
            Some("^5.0.0"),
            "Second catalog entry should be accessible"
        );
        // Unknown entry returns None
        assert_eq!(wm.catalog_version("vue"), None);
    }

    #[test]
    fn test_pnpm_catalogs_named() {
        let dir = tempdir().unwrap();

        // Named catalogs are merged into catalog map with prefix "<catalog_name>:"
        let yaml =
            "packages:\n  - packages/*\ncatalogs:\n  default:\n    react: \"^18\"\n    vue: \"^3\"\n  legacy:\n    react: \"^16\"\n";
        std::fs::write(dir.path().join("pnpm-workspace.yaml"), yaml).unwrap();

        let result = WorkspaceManager::discover(dir.path()).unwrap();
        assert!(result.is_some());
        let wm = result.unwrap();

        assert_eq!(
            wm.catalog_version("default:react"),
            Some("^18"),
            "Named catalog entries should be prefixed with '<catalog_name>:'"
        );
        assert_eq!(wm.catalog_version("default:vue"), Some("^3"));
        assert_eq!(wm.catalog_version("legacy:react"), Some("^16"));
        // Bare name is not present (only prefixed entries)
        assert_eq!(wm.catalog_version("react"), None);
    }

    #[test]
    fn test_pnpm_workspace_yaml_with_catalog_and_catalogs() {
        let dir = tempdir().unwrap();

        // Both catalog: and catalogs: present — both should be merged
        let yaml =
            "packages:\n  - packages/*\ncatalog:\n  shared: \"^1.0.0\"\ncatalogs:\n  v2:\n    shared: \"^2.0.0\"\n";
        std::fs::write(dir.path().join("pnpm-workspace.yaml"), yaml).unwrap();

        let result = WorkspaceManager::discover(dir.path()).unwrap();
        assert!(result.is_some());
        let wm = result.unwrap();

        // Default catalog entry accessible directly
        assert_eq!(wm.catalog_version("shared"), Some("^1.0.0"));
        // Named catalog entry accessible with prefix
        assert_eq!(wm.catalog_version("v2:shared"), Some("^2.0.0"));
    }

    // ─── GH #3560: workspaces field shape parser ──────────────────────

    /// GH #3560 — npm / yarn-modern array form is the canonical happy path.
    #[test]
    fn gh3560_parse_workspaces_array_form() {
        let v = serde_json::json!(["packages/*", "apps/*"]);
        let patterns = parse_workspaces_field(&v).expect("array form must parse");
        assert_eq!(
            patterns,
            vec!["packages/*".to_string(), "apps/*".to_string()]
        );
    }

    /// GH #3560 — yarn-1 object form `{ packages: [...], nohoist: [...] }`
    /// must produce the inner `packages` array. The prior
    /// `.unwrap_or_default()` silently turned this into "no workspace".
    #[test]
    fn gh3560_parse_workspaces_object_form_yarn1() {
        let v = serde_json::json!({
            "packages": ["pkg/*", "tools/*"],
            "nohoist": ["**/react-native"],
        });
        let patterns = parse_workspaces_field(&v).expect("yarn-1 object form must parse");
        assert_eq!(patterns, vec!["pkg/*".to_string(), "tools/*".to_string()]);
    }

    /// GH #3560 — single-string form `"workspaces": "pkg/*"` is a common
    /// typo / single-pattern shortcut. Wrap into a one-element vec rather
    /// than silently dropping it.
    #[test]
    fn gh3560_parse_workspaces_string_form_wraps() {
        let v = serde_json::json!("pkg/*");
        let patterns = parse_workspaces_field(&v).expect("string form must parse");
        assert_eq!(patterns, vec!["pkg/*".to_string()]);
    }

    /// GH #3560 — unsupported shapes (number, null, object without
    /// `packages`, array with non-string elements) must surface a `Err`
    /// whose body names the actual JSON kind so the warn message is
    /// actionable.
    #[test]
    fn gh3560_parse_workspaces_unsupported_shapes_describe_actual_kind() {
        // Number.
        let v = serde_json::json!(42);
        let err = parse_workspaces_field(&v).unwrap_err();
        assert!(
            err.contains("number"),
            "number form err must name 'number', got: {err}"
        );

        // Null.
        let v = serde_json::Value::Null;
        let err = parse_workspaces_field(&v).unwrap_err();
        assert!(
            err.contains("null"),
            "null form err must name 'null', got: {err}"
        );

        // Object without `packages`.
        let v = serde_json::json!({ "nohoist": ["foo"] });
        let err = parse_workspaces_field(&v).unwrap_err();
        assert!(
            err.contains("packages"),
            "object-without-packages err must name 'packages', got: {err}"
        );

        // Object whose `packages` is not an array.
        let v = serde_json::json!({ "packages": "pkg/*" });
        let err = parse_workspaces_field(&v).unwrap_err();
        assert!(
            err.contains("packages") && (err.contains("not an array") || err.contains("string")),
            "object with non-array packages err must say so, got: {err}"
        );

        // Array with a non-string element.
        let v = serde_json::json!(["pkg/*", 42]);
        let err = parse_workspaces_field(&v).unwrap_err();
        assert!(
            err.contains("index 1") && err.contains("number"),
            "array-with-non-string err must name index 1 and 'number', got: {err}"
        );
    }

    // ─── GH #3580: workspace name/version silent fallback ────────────

    /// GH #3580 — `format_workspace_identity_err` must include the
    /// issue tag, the offending package.json path, the field name,
    /// and the observed JSON kind.
    #[test]
    fn gh3580_format_workspace_identity_err_names_tag_path_field_and_kind() {
        let p = std::path::Path::new("/proj/packages/foo/package.json");
        for kind in [
            "missing",
            "null",
            "number",
            "bool",
            "array",
            "object",
            "empty-string",
        ] {
            let msg = format_workspace_identity_err(p, "name", kind);
            assert!(
                msg.contains("GH #3580"),
                "must include tag (kind={kind}), got: {msg}"
            );
            assert!(
                msg.contains("/proj/packages/foo/package.json"),
                "must name the offending path (kind={kind}), got: {msg}"
            );
            assert!(
                msg.contains("name"),
                "must name the field (kind={kind}), got: {msg}"
            );
            assert!(
                msg.contains(kind),
                "must name the kind (kind={kind}), got: {msg}"
            );
        }
    }

    /// GH #3580 — `describe_workspace_field_kind` must distinguish all
    /// JSON shapes so the workspace identity error is precise about
    /// what was observed at the `name`/`version` slot.
    #[test]
    fn gh3580_describe_workspace_field_kind_distinguishes_json_shapes() {
        assert_eq!(
            describe_workspace_field_kind(&serde_json::Value::Null),
            "null"
        );
        assert_eq!(
            describe_workspace_field_kind(&serde_json::Value::Bool(true)),
            "bool"
        );
        assert_eq!(
            describe_workspace_field_kind(&serde_json::json!(42)),
            "number"
        );
        assert_eq!(
            describe_workspace_field_kind(&serde_json::json!("v")),
            "string"
        );
        assert_eq!(
            describe_workspace_field_kind(&serde_json::json!([])),
            "array"
        );
        assert_eq!(
            describe_workspace_field_kind(&serde_json::json!({})),
            "object"
        );
    }

    /// GH #3580 — `require_workspace_string_field` on a `{"name": null}`
    /// package.json must surface an error whose Display contains the
    /// GH #3580 tag, the path, the field, and the observed kind
    /// `null` — NOT a silent "unnamed" placeholder.
    #[test]
    fn gh3580_require_workspace_string_field_surfaces_null_value() {
        let p = std::path::Path::new("/proj/packages/foo/package.json");
        let pkg = serde_json::json!({"name": null, "version": "1.0.0"});
        let err =
            require_workspace_string_field(&pkg, "name", p).expect_err("null name must error");
        let chain = format!("{err:#}");
        assert!(chain.contains("GH #3580"), "must include tag, got: {chain}");
        assert!(chain.contains("name"), "must name the field, got: {chain}");
        assert!(chain.contains("null"), "must name the kind, got: {chain}");
        assert!(
            !chain.contains("unnamed"),
            "must NOT silently fall back to 'unnamed', got: {chain}"
        );
    }

    /// GH #3580 — happy path: a valid `{"name": "foo", "version": "1.0.0"}`
    /// passes through `require_workspace_string_field` unchanged. Pins
    /// that the new error path does not regress the common case.
    #[test]
    fn gh3580_require_workspace_string_field_happy_path() {
        let p = std::path::Path::new("/proj/packages/foo/package.json");
        let pkg = serde_json::json!({"name": "foo", "version": "1.0.0"});
        assert_eq!(
            require_workspace_string_field(&pkg, "name", p).unwrap(),
            "foo"
        );
        assert_eq!(
            require_workspace_string_field(&pkg, "version", p).unwrap(),
            "1.0.0"
        );
    }

    // ─── GH #3628: workspace package path abs-path leak ──────────────

    /// GH #3628 — happy path: a package directory under the workspace
    /// root produces a relative path with no warning.
    #[test]
    fn gh3628_safe_workspace_relative_under_root_returns_relative_no_warn() {
        let root = Path::new("/work/repo");
        let pkg_dir = Path::new("/work/repo/packages/ui");
        let (rel, warn) = safe_workspace_relative_path(pkg_dir, root);
        assert_eq!(rel, PathBuf::from("packages/ui"));
        assert!(warn.is_none(), "no warn under root, got: {warn:?}");
    }

    /// GH #3628 — bug-of-record: a package directory OUTSIDE the
    /// workspace root must NOT leak the absolute path into the
    /// returned PathBuf (previously `unwrap_or(pkg_dir)` returned the
    /// absolute path verbatim). Helper falls back to the bare file
    /// name and surfaces a warning.
    #[test]
    fn gh3628_safe_workspace_relative_outside_root_no_abs_leak() {
        let root = Path::new("/work/repo");
        let pkg_dir = Path::new("/some/other/place/escaped-pkg");
        let (rel, warn) = safe_workspace_relative_path(pkg_dir, root);
        assert_eq!(
            rel,
            PathBuf::from("escaped-pkg"),
            "MUST NOT be the absolute /some/other/place/escaped-pkg"
        );
        assert!(
            !rel.is_absolute(),
            "outside-root fallback MUST NOT be absolute, got: {rel:?}"
        );
        let msg = warn.expect("outside-root must produce a warning");
        assert!(msg.contains("GH #3628"), "msg: {msg}");
        assert!(msg.contains("escaped-pkg"), "msg must name offender: {msg}");
        assert!(
            msg.contains("/work/repo"),
            "msg must name workspace root: {msg}"
        );
    }

    /// GH #3628 — outside-root with no file_name (e.g. "/" or "..")
    /// falls back to the bare PathBuf but still emits the warning so
    /// the operator is alerted to the misconfiguration.
    #[test]
    fn gh3628_safe_workspace_relative_outside_root_no_filename_still_warns() {
        let root = Path::new("/work/repo");
        let pkg_dir = Path::new("/");
        let (_rel, warn) = safe_workspace_relative_path(pkg_dir, root);
        assert!(
            warn.is_some(),
            "outside-root without file_name still must warn"
        );
    }

    /// GH #3628 — formatter shape pins the issue tag, both paths,
    /// and the consequence string.
    #[test]
    fn gh3628_format_safe_workspace_relative_path_warn_shape() {
        let pkg_dir = Path::new("/elsewhere/pkg");
        let root = Path::new("/repo");
        let err = pkg_dir.strip_prefix(root).expect_err("disjoint paths");
        let msg = format_safe_workspace_relative_path_warn(pkg_dir, root, &err);
        assert!(msg.contains("GH #3628"), "msg: {msg}");
        assert!(msg.contains("/elsewhere/pkg"), "msg: {msg}");
        assert!(msg.contains("/repo"), "msg: {msg}");
        assert!(
            msg.contains("jet-lock.yaml") || msg.contains("local_path"),
            "msg must name the leaked artifact: {msg}"
        );
    }

    /// GH #3628 — end-to-end via `read_workspace_package` with an
    /// out-of-root package directory: the resulting `WorkspacePackage`
    /// must carry a non-absolute `path`.
    #[test]
    fn gh3628_read_workspace_package_outside_root_keeps_path_non_abs() {
        let pkg_holder = tempdir().unwrap();
        let root_holder = tempdir().unwrap();
        let pkg_dir = pkg_holder.path();
        let root = root_holder.path();

        std::fs::write(
            pkg_dir.join("package.json"),
            r#"{"name": "stray", "version": "1.0.0"}"#,
        )
        .unwrap();

        let pkg = WorkspaceManager::read_workspace_package(root, pkg_dir).unwrap();
        assert!(
            !pkg.path.is_absolute(),
            "WorkspacePackage.path must NOT be absolute for out-of-root package, got: {:?}",
            pkg.path
        );
    }
}

#[cfg(test)]
mod gh3747_extract_deps_warn_tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_pkg(json: &str) -> (tempfile::TempDir, PathBuf) {
        let tmp = tempdir().unwrap();
        let pkg_dir = tmp.path().join("pkg");
        fs::create_dir_all(&pkg_dir).unwrap();
        let pkg_path = pkg_dir.join("package.json");
        fs::write(&pkg_path, json).unwrap();
        (tmp, pkg_path)
    }

    /// Absent dependencies field → empty map, silent (no warn).
    #[test]
    fn gh3747_absent_field_returns_empty_silently() {
        let (_g, pkg_path) = write_pkg(r#"{"name":"a","version":"1.0.0"}"#);
        let p = WorkspaceManager::read_workspace_package(
            pkg_path.parent().unwrap().parent().unwrap(),
            pkg_path.parent().unwrap(),
        )
        .unwrap();
        assert!(p.dependencies.is_empty());
        assert!(p.dev_dependencies.is_empty());
    }

    /// Empty object dependencies → empty map, silent (no warn).
    #[test]
    fn gh3747_empty_object_field_returns_empty_silently() {
        let (_g, pkg_path) =
            write_pkg(r#"{"name":"a","version":"1.0.0","dependencies":{},"devDependencies":{}}"#);
        let p = WorkspaceManager::read_workspace_package(
            pkg_path.parent().unwrap().parent().unwrap(),
            pkg_path.parent().unwrap(),
        )
        .unwrap();
        assert!(p.dependencies.is_empty());
        assert!(p.dev_dependencies.is_empty());
    }

    /// Well-formed dependencies object → round-trips intact.
    #[test]
    fn gh3747_well_formed_dependencies_round_trip() {
        let (_g, pkg_path) = write_pkg(
            r#"{"name":"a","version":"1.0.0","dependencies":{"foo":"^1.0.0","bar":"2.x"}}"#,
        );
        let p = WorkspaceManager::read_workspace_package(
            pkg_path.parent().unwrap().parent().unwrap(),
            pkg_path.parent().unwrap(),
        )
        .unwrap();
        assert_eq!(
            p.dependencies.get("foo").map(String::as_str),
            Some("^1.0.0")
        );
        assert_eq!(p.dependencies.get("bar").map(String::as_str), Some("2.x"));
    }

    /// Wrong shape: dependencies is a string ("latest") instead of an
    /// object. Must NOT panic and must NOT abort the package load —
    /// the dep map degrades to empty (with a warn emitted to logs).
    #[test]
    fn gh3747_string_shape_does_not_panic_or_abort_load() {
        let (_g, pkg_path) = write_pkg(r#"{"name":"a","version":"1.0.0","dependencies":"latest"}"#);
        let p = WorkspaceManager::read_workspace_package(
            pkg_path.parent().unwrap().parent().unwrap(),
            pkg_path.parent().unwrap(),
        )
        .unwrap();
        assert!(p.dependencies.is_empty());
    }

    /// Wrong shape: dependencies is an array.
    #[test]
    fn gh3747_array_shape_does_not_panic_or_abort_load() {
        let (_g, pkg_path) =
            write_pkg(r#"{"name":"a","version":"1.0.0","dependencies":["foo","bar"]}"#);
        let p = WorkspaceManager::read_workspace_package(
            pkg_path.parent().unwrap().parent().unwrap(),
            pkg_path.parent().unwrap(),
        )
        .unwrap();
        assert!(p.dependencies.is_empty());
    }

    /// Wrong shape: object with non-string version values.
    #[test]
    fn gh3747_object_with_non_string_values_does_not_panic() {
        let (_g, pkg_path) =
            write_pkg(r#"{"name":"a","version":"1.0.0","dependencies":{"foo":1,"bar":true}}"#);
        let p = WorkspaceManager::read_workspace_package(
            pkg_path.parent().unwrap().parent().unwrap(),
            pkg_path.parent().unwrap(),
        )
        .unwrap();
        assert!(p.dependencies.is_empty());
    }

    /// Helper output carries the issue tag and field name so the warn
    /// is greppable during incident triage.
    #[test]
    fn gh3747_helper_message_contains_issue_tag_and_field() {
        let p = PathBuf::from("/tmp/x/pkg/package.json");
        let msg = format_workspace_deps_shape_warn(&p, "dependencies", "string");
        assert!(
            msg.contains("GH #3747"),
            "msg must contain issue tag: {msg}"
        );
        assert!(
            msg.contains("dependencies"),
            "msg must name the field: {msg}"
        );
        assert!(msg.contains("string"), "msg must name observed kind: {msg}");
        assert!(
            msg.contains("topological_order") || msg.contains("topology"),
            "msg must call out the graph-corruption consequence: {msg}"
        );
    }

    /// Deterministic — same input → byte-identical message.
    #[test]
    fn gh3747_helper_message_is_deterministic() {
        let p = PathBuf::from("/tmp/x/pkg/package.json");
        let a = format_workspace_deps_shape_warn(&p, "dependencies", "string");
        let b = format_workspace_deps_shape_warn(&p, "dependencies", "string");
        assert_eq!(a, b);
    }

    /// Sibling-distinctness vs related warn-tags emitted by this
    /// module / family (#3524 workspace pkg read; #3580 identity err;
    /// #3628 relative path; #3743 history field; #3745 unknown page
    /// request). The #3747 warn must NOT collide.
    #[test]
    fn gh3747_warn_is_distinct_from_sibling_warn_helpers() {
        let p = PathBuf::from("/tmp/x/pkg/package.json");
        let m_3747 = format_workspace_deps_shape_warn(&p, "dependencies", "string");
        let m_3524 = format_workspace_pkg_warn(&p, &anyhow::anyhow!("boom"));
        assert_ne!(m_3747, m_3524);
        assert!(!m_3747.contains("#3524"));
        assert!(!m_3747.contains("#3580"));
        assert!(!m_3747.contains("#3628"));
        assert!(!m_3747.contains("#3743"));
        assert!(!m_3747.contains("#3745"));
        assert!(!m_3524.contains("#3747"));
    }

    /// Naming convention discoverability — keeps the warn-helper
    /// family uniformly named so future authors find them via
    /// `format_*_warn`.
    #[test]
    fn gh3747_helper_name_follows_family_convention() {
        let name = "format_workspace_deps_shape_warn";
        assert!(name.starts_with("format_"));
        assert!(name.ends_with("_warn"));
        assert!(name.contains("workspace"));
    }

    /// Field name varies between `dependencies` and `devDependencies`;
    /// the same shape-error wording must surface BOTH variants
    /// distinctly so triage can tell which one is broken.
    #[test]
    fn gh3747_dependencies_vs_dev_dependencies_messages_are_distinct() {
        let p = PathBuf::from("/tmp/x/pkg/package.json");
        let m_deps = format_workspace_deps_shape_warn(&p, "dependencies", "string");
        let m_dev = format_workspace_deps_shape_warn(&p, "devDependencies", "string");
        assert_ne!(m_deps, m_dev);
        assert!(m_deps.contains("dependencies") && !m_deps.contains("devDependencies"));
        assert!(m_dev.contains("devDependencies"));
    }
}
// CODEGEN-END
