// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
// CODEGEN-BEGIN
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::pkg_manager::workspace::strip_aw_claim_wrapper_lines;

use super::importmap;
use super::polyfills;

/// Version tag stamped into `_cache_marker`.
///
/// Bump this whenever the resolver, prebundle wrapper, or importmap patch
/// table changes in a way that should invalidate every project's cache.
/// The marker's textual contents must contain this tag for the cache to be
/// considered valid (see `check_cache_valid`). See jet#1908 AC R7.
pub(crate) const CACHE_MARKER_VERSION: &str = "v14-emotion-react-companion-fallback";

/// Pre-bundler for CJS→ESM conversion of npm dependencies.
///
/// Scans `package.json` dependencies, detects CJS packages, creates virtual
/// ESM entries, bundles them, and writes output to `node_modules/.jet/`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub struct PreBundler {
    root_dir: PathBuf,
}

/// Result of a pre-bundling run.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub struct PreBundleResult {
    /// Importmap JSON string ready for HTML injection.
    pub importmap_json: String,
    /// Map from bare specifier → `.jet/` filename.
    pub prebundled: HashMap<String, String>,
    /// Whether the cache was used (no bundling needed).
    pub cache_hit: bool,
}

/// Read the cached importmap JSON from `<jet_dir>/_importmap.json`.
///
/// Returns `None` when the file is absent (legitimate race between the
/// cache-valid check and this read), or after emitting a `tracing::warn!`
/// tagged `GH #3304` under target `jet::dev::prebundle` for any other IO
/// failure (EACCES, EIO, truncated file). In the latter case the caller
/// falls through to a full rebundle; the warn ensures the operator can
/// triage why a "cached" dev-server start did a cold rebuild.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub(crate) fn read_cached_importmap_or_warn(jet_dir: &Path) -> Option<String> {
    let importmap_path = jet_dir.join("_importmap.json");
    match std::fs::read_to_string(&importmap_path) {
        Ok(cached_json) => Some(cached_json),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
        Err(err) => {
            tracing::warn!(
                target: "jet::dev::prebundle",
                path = %importmap_path.display(),
                error = %err,
                "GH #3304 cache-valid importmap unreadable; falling through \
                 to a full rebundle. Subsequent dev-server starts will repeat \
                 the cold rebuild until the file becomes readable \
                 (chmod/ownership/corruption)."
            );
            None
        }
    }
}

/// Read a `package.json` file while tolerating AW source-ownership wrappers.
///
/// Standardized fixtures may wrap JSON bodies in exact `// <HANDWRITE ...>` /
/// `// </HANDWRITE>` ownership markers. Package-manager install already strips
/// those markers; prebundle must do the same or `jet install` succeeds and then
/// fails during the post-install prebundle step on the same project.
pub(crate) fn read_package_json_value(path: &Path) -> Result<serde_json::Value> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read package.json at {}", path.display()))?;
    let content = strip_aw_claim_wrapper_lines(&content);
    serde_json::from_str(content.as_ref())
        .with_context(|| format!("Failed to parse package.json at {}", path.display()))
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
impl PreBundler {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    /// Run the full pre-bundling pipeline.
    ///
    /// 1. Read `package.json` dependencies
    /// 2. Check cache validity (mtime of package.json + lockfile)
    /// 3. For each CJS dep, create virtual ESM entry → bundle → write .jet/
    /// 4. Detect Node.js builtin imports → generate polyfills
    /// 5. Build and return importmap
    pub async fn prebundle_deps(&self) -> Result<PreBundleResult> {
        let jet_dir = self.root_dir.join("node_modules/.jet");
        eprintln!("[jet] Pre-bundling dependencies...");

        // Check cache before doing any work
        if self.check_cache_valid(&jet_dir) {
            eprintln!("[jet] Pre-bundle cache valid, skipping");
            tracing::info!("Pre-bundle cache valid, skipping");
            if let Some(cached_json) = read_cached_importmap_or_warn(&jet_dir) {
                return Ok(PreBundleResult {
                    importmap_json: cached_json,
                    prebundled: HashMap::new(),
                    cache_hit: true,
                });
            }
        }

        std::fs::create_dir_all(&jet_dir)?;

        // Read package.json
        let pkg_json_path = self.root_dir.join("package.json");
        let pkg_json = read_package_json_value(&pkg_json_path)?;

        // Collect all dependencies (root + workspace transitive)
        let mut deps = self.collect_dependencies(&pkg_json);

        // Also collect transitive deps from workspace packages.
        // Workspace packages are served as source, but their CJS deps
        // still need pre-bundling.
        let node_modules = self.root_dir.join("node_modules");
        let workspace_names: Vec<String> = deps
            .iter()
            .filter(|(name, _)| is_workspace_symlink(&node_modules.join(name)))
            .map(|(name, _)| name.clone())
            .collect();
        self.merge_workspace_transitive_deps(&node_modules, &workspace_names, &mut deps);

        // Detect CJS packages and pre-bundle them
        let mut prebundled: HashMap<String, String> = HashMap::new();
        let mut prebundled_sources: HashMap<String, String> = HashMap::new();

        for (name, _version_range) in &deps {
            let node_modules = self.root_dir.join("node_modules");
            let pkg_dir = node_modules.join(name);
            if !pkg_dir.exists() {
                tracing::debug!("Skipping {} — not installed", name);
                continue;
            }

            // Skip workspace packages — local source, served on-the-fly
            if is_workspace_symlink(&pkg_dir) {
                tracing::debug!("Skipping {} — workspace package", name);
                continue;
            }

            if !self.is_cjs_package(&pkg_dir)? {
                tracing::debug!("Skipping {} — already ESM", name);
                continue;
            }

            // Create virtual ESM entry and bundle
            match self.bundle_cjs_dep(name, &pkg_dir, &jet_dir).await {
                Ok((filename, source)) => {
                    prebundled.insert(name.clone(), filename);
                    prebundled_sources.insert(name.clone(), source);
                }
                Err(e) => {
                    tracing::warn!("Failed to pre-bundle {}: {}", name, e);
                }
            }
        }

        // Detect circular require() between pre-bundled packages.
        // Under the CJS wrapper approach, circular deps yield empty module.exports
        // on the cycle-breaking edge — we warn so the user can investigate.
        let cycles = detect_circular_deps(&prebundled_sources);
        for cycle in &cycles {
            tracing::warn!(
                "Circular CJS dependency detected: {} — wrapped module.exports may be empty on cycle edge",
                cycle.join(" → ")
            );
        }

        // Auto-discover transitive CJS deps required by pre-bundled modules.
        // Scan pre-bundled source for require('...') calls and bundle missing deps.
        let mut transitive_queue: Vec<String> = Vec::new();
        let require_re =
            regex::Regex::new(r#"require\s*\(\s*['"]([^'"./][^'"]*)['"]\s*\)"#).unwrap();
        for source in prebundled_sources.values() {
            for cap in require_re.captures_iter(source) {
                let dep = cap[1].to_string();
                if !prebundled.contains_key(&dep) && !transitive_queue.contains(&dep) {
                    transitive_queue.push(dep);
                }
            }
        }
        // Resolve package dirs with parent nm support
        let node_modules = self.root_dir.join("node_modules");
        let mut nm_dirs_for_resolve = vec![node_modules.clone()];
        {
            let mut parent = self.root_dir.parent();
            while let Some(p) = parent {
                let parent_nm = p.join("node_modules");
                if parent_nm.exists() && parent_nm != node_modules {
                    nm_dirs_for_resolve.push(parent_nm);
                }
                if p.join(".git").exists() || p.join("pnpm-workspace.yaml").exists() {
                    break;
                }
                parent = p.parent();
            }
        }
        for dep_name in transitive_queue {
            if let Some(pkg_dir) = self.resolve_package_dir(&dep_name, &nm_dirs_for_resolve) {
                // Create symlink in local node_modules if not present.
                // GH #3282 — surface mkdir/symlink failures so a broken
                // symlink target or unwritable parent doesn't quietly
                // leave the dep absent from node_modules; later requests
                // for raw asset URLs under node_modules/<dep>/... would
                // then 404 with no log line linking them to the failure.
                let local_path = node_modules.join(&dep_name);
                if !local_path.exists() {
                    Self::symlink_into_node_modules(&pkg_dir, &local_path, &dep_name);
                }
                if self.is_cjs_package(&pkg_dir).unwrap_or(false) {
                    match self.bundle_cjs_dep(&dep_name, &pkg_dir, &jet_dir).await {
                        Ok((filename, source)) => {
                            prebundled.insert(dep_name.clone(), filename);
                            prebundled_sources.insert(dep_name, source);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to pre-bundle transitive dep {}: {}",
                                dep_name,
                                e
                            );
                        }
                    }
                }
            }
        }

        // Auto-discover transitive CJS deps via subpath scanning
        let mut subpath_deps = deps.clone();
        let mut source_subpath_imports = self.collect_source_subpath_imports();
        self.add_known_patch_subpath_imports(&mut subpath_deps, &mut source_subpath_imports);
        self.discover_subpath_exports(
            &subpath_deps,
            &jet_dir,
            &mut prebundled,
            &mut prebundled_sources,
            &source_subpath_imports,
        )
        .await;

        self.prebundle_known_cjs_patch_roots(&jet_dir, &mut prebundled, &mut prebundled_sources)
            .await;

        // Detect Node.js builtin imports in pre-bundled sources
        let detected_builtins = polyfills::detect_builtin_imports(&prebundled_sources);
        let polyfill_names = polyfills::write_polyfills(&detected_builtins, &jet_dir);

        // Scan ESM deps (not pre-bundled) and resolve their entry paths
        let esm_deps = self.scan_esm_deps(&deps, &prebundled);

        // Build importmap (includes pre-bundled CJS + ESM deps + polyfills)
        let importmap_json =
            importmap::build_importmap_full(&prebundled, &esm_deps, &polyfill_names);

        // Cache the importmap for next startup.
        write_importmap_cache(&jet_dir, &importmap_json);

        // Write cache marker
        self.write_cache_marker(&jet_dir)?;

        tracing::info!(
            "Pre-bundled {} dependencies, {} polyfills",
            prebundled.len(),
            polyfill_names.len()
        );

        Ok(PreBundleResult {
            importmap_json,
            prebundled,
            cache_hit: false,
        })
    }

    /// Merge transitive deps declared by workspace symlinks into `deps`.
    ///
    /// GH #3179 — the prior `if let Ok(content) ... if let Ok(ws_pkg)` chain
    /// silently dropped every workspace's transitive deps when its
    /// `package.json` was unreadable or malformed (trailing comma, broken
    /// quote, permission error). Symptom downstream was a browser error like
    /// `Failed to resolve module specifier <dep>` with no breadcrumb back to
    /// the offending file. The match arms below surface read/parse failures
    /// via `tracing::warn!` so the developer can trace it; `NotFound` stays
    /// silent because a workspace without a `package.json` is legitimate.
    pub(crate) fn merge_workspace_transitive_deps(
        &self,
        node_modules: &Path,
        workspace_names: &[String],
        deps: &mut HashMap<String, String>,
    ) {
        for ws_name in workspace_names {
            let ws_pkg_json_path = node_modules.join(ws_name).join("package.json");
            let content = match std::fs::read_to_string(&ws_pkg_json_path) {
                Ok(c) => c,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => {
                    tracing::warn!(
                        target: "jet::dev_server::prebundle",
                        "workspace package {} has unreadable package.json at {:?}: {e}; \
                         its transitive deps will NOT be added to the prebundle set (GH #3179)",
                        ws_name,
                        ws_pkg_json_path
                    );
                    continue;
                }
            };
            let content = strip_aw_claim_wrapper_lines(&content);
            let ws_pkg: serde_json::Value = match serde_json::from_str(content.as_ref()) {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(
                        target: "jet::dev_server::prebundle",
                        "workspace package {} has malformed package.json at {:?}: {e}; \
                         its transitive deps will NOT be added to the prebundle set (GH #3179)",
                        ws_name,
                        ws_pkg_json_path
                    );
                    continue;
                }
            };
            let ws_deps = self.collect_dependencies(&ws_pkg);
            for (name, ver) in ws_deps {
                deps.entry(name).or_insert(ver);
            }
        }
    }

    /// Collect all dependencies from package.json.
    fn collect_dependencies(&self, pkg_json: &serde_json::Value) -> HashMap<String, String> {
        let mut deps = HashMap::new();

        if let Some(obj) = pkg_json.get("dependencies").and_then(|v| v.as_object()) {
            for (name, ver) in obj {
                if let Some(v) = ver.as_str() {
                    deps.insert(name.clone(), v.to_string());
                }
            }
        }

        deps
    }

    /// Check if a package is CJS (has no `"module"` or `"exports"."import"` field).
    pub fn is_cjs_package(&self, pkg_dir: &Path) -> Result<bool> {
        let pkg_json_path = pkg_dir.join("package.json");
        if !pkg_json_path.exists() {
            return Ok(false);
        }

        let pkg = read_package_json_value(&pkg_json_path)?;

        // Has "module" field → ESM
        if pkg.get("module").and_then(|v| v.as_str()).is_some() {
            return Ok(false);
        }

        // Has "type": "module" → ESM
        if pkg.get("type").and_then(|v| v.as_str()) == Some("module") {
            return Ok(false);
        }

        // Has "exports" with "import" condition → ESM
        if let Some(exports) = pkg.get("exports") {
            if has_import_condition(exports) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Bundle a single CJS dependency into an ESM file.
    ///
    /// Creates a virtual entry that re-exports everything from the package,
    /// then uses the bundler to convert CJS → ESM.
    async fn bundle_cjs_dep(
        &self,
        name: &str,
        pkg_dir: &Path,
        jet_dir: &Path,
    ) -> Result<(String, String)> {
        let entry_source = create_virtual_entry(name);

        // Resolve the main entry point of the CJS package
        let main_entry = self.resolve_package_main(pkg_dir)?;

        // Read and transform the CJS source. GH #3374 — distinguish a
        // legitimate race (NotFound between resolve_package_main and read)
        // from real trouble (chmod, mid-read EIO) so a silently-stubbed
        // bundle doesn't mask the underlying problem.
        let source = match std::fs::read_to_string(&main_entry) {
            Ok(s) => s,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                let filename = dep_filename(name);
                std::fs::write(jet_dir.join(&filename), &entry_source)?;
                return Ok((filename, entry_source));
            }
            Err(err) => {
                tracing::warn!(
                    target: "jet::dev::prebundle",
                    package = %name,
                    main_entry = %main_entry.display(),
                    error_kind = ?err.kind(),
                    error = %err,
                    "GH #3374 CJS package main entry exists but failed to \
                     read; falling back to virtual-entry stub. The dev \
                     bundle will not contain the real package source — \
                     check filesystem permissions or aborted writes."
                );
                let filename = dep_filename(name);
                std::fs::write(jet_dir.join(&filename), &entry_source)?;
                return Ok((filename, entry_source));
            }
        };

        // Replace process.env.NODE_ENV BEFORE wrapping so conditionals resolve
        let source = source.replace("process.env.NODE_ENV", "'development'");

        // Inline relative require() calls — resolves the React pattern where
        // index.js does: module.exports = require('./cjs/react.development.js')
        let source = inline_requires(&source, main_entry.parent().unwrap());

        // Simple CJS → ESM conversion: wrap require/module.exports.
        let dep_imports = self.bundle_nested_cjs_deps(name, pkg_dir, &source, jet_dir);
        let esm_output = convert_cjs_to_esm(&source, name, &dep_imports);

        // Final pass: replace any remaining process.env.NODE_ENV in inlined code
        let esm_output = esm_output.replace("process.env.NODE_ENV", "'development'");

        let filename = dep_filename(name);
        std::fs::write(jet_dir.join(&filename), &esm_output)?;

        Ok((filename, esm_output))
    }

    fn bundle_nested_cjs_deps(
        &self,
        parent_specifier: &str,
        parent_pkg_dir: &Path,
        source: &str,
        jet_dir: &Path,
    ) -> HashMap<String, String> {
        let mut dep_imports = HashMap::new();

        for dep in collect_external_requires(source) {
            let Some(nested_pkg_dir) = resolve_nested_package_dir(parent_pkg_dir, &dep) else {
                continue;
            };
            if !self.is_cjs_package(&nested_pkg_dir).unwrap_or(false) {
                continue;
            }
            if let Some(root_pkg_dir) = resolve_root_package_dir(jet_dir, &dep) {
                if package_versions_match(&root_pkg_dir, &nested_pkg_dir)
                    && self.is_cjs_package(&root_pkg_dir).unwrap_or(false)
                {
                    dep_imports.insert(dep.clone(), dep_filename(&dep));
                    continue;
                }
            }

            let nested_specifier = format!("{}/node_modules/{}", parent_specifier, dep);
            match self.bundle_cjs_dep_without_nested(&nested_specifier, &nested_pkg_dir, jet_dir) {
                Ok((filename, _)) => {
                    dep_imports.insert(dep, filename);
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to pre-bundle nested CJS dependency {} for {}: {}",
                        dep,
                        parent_specifier,
                        e
                    );
                }
            }
        }

        dep_imports
    }

    fn bundle_cjs_dep_without_nested(
        &self,
        name: &str,
        pkg_dir: &Path,
        jet_dir: &Path,
    ) -> Result<(String, String)> {
        let entry_source = create_virtual_entry(name);
        let main_entry = self.resolve_package_main(pkg_dir)?;

        let source = match std::fs::read_to_string(&main_entry) {
            Ok(s) => s,
            Err(_) => {
                let filename = dep_filename(name);
                std::fs::write(jet_dir.join(&filename), &entry_source)?;
                return Ok((filename, entry_source));
            }
        };

        let source = source.replace("process.env.NODE_ENV", "'development'");
        let source = inline_requires(&source, main_entry.parent().unwrap());
        let esm_output = convert_cjs_to_esm(&source, name, &HashMap::new())
            .replace("process.env.NODE_ENV", "'development'");

        let filename = dep_filename(name);
        std::fs::write(jet_dir.join(&filename), &esm_output)?;
        Ok((filename, esm_output))
    }

    /// Discover subpath exports from package.json exports maps and pre-bundle them.
    ///
    /// Scans each installed dependency's `"exports"` field for subpath entries
    /// (e.g. `"./jsx-runtime"`, `"./client"`) and pre-bundles any that resolve to CJS.
    async fn discover_subpath_exports(
        &self,
        deps: &HashMap<String, String>,
        jet_dir: &Path,
        prebundled: &mut HashMap<String, String>,
        prebundled_sources: &mut HashMap<String, String>,
        source_subpath_imports: &HashMap<String, HashSet<String>>,
    ) {
        for (pkg, _) in deps {
            let pkg_dir = self.root_dir.join("node_modules").join(pkg);
            if is_workspace_symlink(&pkg_dir) || is_local_source_package(&pkg_dir) {
                continue; // workspace package — serve source files directly
            }
            let pkg_json_path = pkg_dir.join("package.json");
            let content = match std::fs::read_to_string(&pkg_json_path) {
                Ok(c) => c,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
                Err(err) => {
                    tracing::warn!(
                        target: "jet::dev::prebundle",
                        pkg = %pkg,
                        path = %pkg_json_path.display(),
                        error = %err,
                        "GH #3322 subpath-export discovery skipping package: \
                         package.json unreadable; the package's subpath \
                         exports will be missing from the importmap. Check \
                         node_modules permissions."
                    );
                    continue;
                }
            };
            let content = strip_aw_claim_wrapper_lines(&content);
            let pkg_json: serde_json::Value = match serde_json::from_str(content.as_ref()) {
                Ok(v) => v,
                Err(err) => {
                    tracing::warn!(
                        target: "jet::dev::prebundle",
                        pkg = %pkg,
                        path = %pkg_json_path.display(),
                        error = %err,
                        "GH #3322 subpath-export discovery skipping package: \
                         package.json failed to parse; the package's subpath \
                         exports will be missing from the importmap."
                    );
                    continue;
                }
            };

            // Extract subpath entries from exports map
            let all_subpaths = match pkg_json.get("exports").and_then(|e| e.as_object()) {
                Some(map) => map
                    .keys()
                    .filter(|k| k.starts_with("./") && *k != ".")
                    .map(|k| k.trim_start_matches("./").to_string())
                    .collect::<Vec<_>>(),
                None => continue,
            };
            let subpaths = match source_subpath_imports.get(pkg) {
                Some(used) => all_subpaths
                    .into_iter()
                    .filter(|subpath| used.contains(subpath))
                    .collect::<Vec<_>>(),
                None if all_subpaths.len() > 64 => Vec::new(),
                None => all_subpaths,
            };

            for subpath in subpaths {
                let specifier = format!("{}/{}", pkg, subpath);
                // Skip if already pre-bundled as main entry
                if prebundled.contains_key(&specifier) {
                    continue;
                }

                let entry = self.resolve_subpath_export(&pkg_dir, &subpath);
                if let Some(entry_path) = entry {
                    if let Ok(source) = std::fs::read_to_string(&entry_path) {
                        if is_esm_module_source(&source) {
                            continue;
                        }
                        let source = source.replace("process.env.NODE_ENV", "'development'");
                        let source = inline_requires(&source, entry_path.parent().unwrap());
                        let dep_imports =
                            self.bundle_nested_cjs_deps(&specifier, &pkg_dir, &source, jet_dir);
                        let esm = convert_cjs_to_esm(&source, &specifier, &dep_imports)
                            .replace("process.env.NODE_ENV", "'development'");
                        let filename = dep_filename(&specifier);
                        if std::fs::write(jet_dir.join(&filename), &esm).is_ok() {
                            prebundled.insert(specifier.clone(), filename);
                            prebundled_sources.insert(specifier, esm);
                        }
                    }
                }
            }
        }
    }

    fn collect_source_subpath_imports(&self) -> HashMap<String, HashSet<String>> {
        let mut imports: HashMap<String, HashSet<String>> = HashMap::new();
        imports
            .entry("react".to_string())
            .or_default()
            .extend(["jsx-runtime".to_string(), "jsx-dev-runtime".to_string()]);
        self.collect_source_subpath_imports_from_dir(&self.root_dir.join("src"), &mut imports);
        imports
    }

    fn add_known_patch_subpath_imports(
        &self,
        deps: &mut HashMap<String, String>,
        imports: &mut HashMap<String, HashSet<String>>,
    ) {
        for (specifier, target) in importmap::mui_emotion_patches() {
            if !target.starts_with("/node_modules/.jet/") {
                continue;
            }
            let Some((pkg, subpath)) = split_bare_subpath_import(specifier) else {
                continue;
            };
            deps.entry(pkg.clone()).or_default();
            imports.entry(pkg).or_default().insert(subpath);
        }
    }

    async fn prebundle_known_cjs_patch_roots(
        &self,
        jet_dir: &Path,
        prebundled: &mut HashMap<String, String>,
        prebundled_sources: &mut HashMap<String, String>,
    ) {
        let node_modules = self.root_dir.join("node_modules");
        let mut nm_dirs = vec![node_modules.clone()];
        let mut parent = self.root_dir.parent();
        while let Some(p) = parent {
            let parent_nm = p.join("node_modules");
            if parent_nm.exists() && parent_nm != node_modules {
                nm_dirs.push(parent_nm);
            }
            if p.join(".git").exists() || p.join("pnpm-workspace.yaml").exists() {
                break;
            }
            parent = p.parent();
        }

        for (specifier, target) in importmap::mui_emotion_patches() {
            if !target.starts_with("/node_modules/.jet/") || specifier.contains('/') {
                continue;
            }
            if prebundled.contains_key(*specifier) {
                continue;
            }
            let Some(pkg_dir) = self.resolve_package_dir(specifier, &nm_dirs) else {
                continue;
            };
            if !self.is_cjs_package(&pkg_dir).unwrap_or(false) {
                continue;
            }
            match self.bundle_cjs_dep(specifier, &pkg_dir, jet_dir).await {
                Ok((filename, source)) => {
                    prebundled.insert((*specifier).to_string(), filename);
                    prebundled_sources.insert((*specifier).to_string(), source);
                }
                Err(e) => {
                    tracing::warn!("Failed to pre-bundle known patch root {}: {}", specifier, e);
                }
            }
        }
    }

    fn collect_source_subpath_imports_from_dir(
        &self,
        dir: &Path,
        imports: &mut HashMap<String, HashSet<String>>,
    ) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.collect_source_subpath_imports_from_dir(&path, imports);
                continue;
            }
            let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
                continue;
            };
            if !matches!(ext, "js" | "jsx" | "ts" | "tsx") {
                continue;
            }
            let Ok(source) = fs::read_to_string(&path) else {
                continue;
            };
            let is_typescript = matches!(ext, "ts" | "tsx");
            let Ok(module_imports) =
                crate::bundler::imports::extract_imports(&source, is_typescript)
            else {
                continue;
            };
            for import in module_imports.static_imports {
                if let Some((pkg, subpath)) = split_bare_subpath_import(&import.source) {
                    imports.entry(pkg).or_default().insert(subpath);
                }
            }
        }
    }

    /// Resolve a subpath export from a package's exports map.
    ///
    /// GH #3182 — the prior `.ok()?` chain silently treated every failure
    /// mode (read error, parse error, missing `exports`) as "no subpath
    /// export". Symptom downstream: browser errors with
    /// `Failed to resolve module specifier <pkg>/<subpath>` and no
    /// breadcrumb back to the offending `package.json`. `NotFound` stays
    /// silent (caller's job — a dir without a `package.json` is not really
    /// a package), but other IO and JSON parse errors emit
    /// `tracing::warn!` so the developer can find the typo.
    pub(crate) fn resolve_subpath_export(&self, pkg_dir: &Path, subpath: &str) -> Option<PathBuf> {
        let pkg_json_path = pkg_dir.join("package.json");
        let content = match std::fs::read_to_string(&pkg_json_path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
            Err(e) => {
                tracing::warn!(
                    target: "jet::dev_server::prebundle",
                    "package at {:?} has unreadable package.json: {e}; subpath export \
                     './{subpath}' will NOT be resolved (GH #3182)",
                    pkg_dir
                );
                return None;
            }
        };
        let content = strip_aw_claim_wrapper_lines(&content);
        let pkg: serde_json::Value = match serde_json::from_str(content.as_ref()) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(
                    target: "jet::dev_server::prebundle",
                    "package at {:?} has malformed package.json: {e}; subpath export \
                     './{subpath}' will NOT be resolved (GH #3182)",
                    pkg_dir
                );
                return None;
            }
        };

        let exports = pkg.get("exports")?;
        let key = format!("./{}", subpath);

        if let Some(entry) = exports.get(&key).and_then(resolve_exports_entry) {
            return Some(pkg_dir.join(entry));
        }

        None
    }

    /// Resolve the main entry point of a package.
    fn resolve_package_main(&self, pkg_dir: &Path) -> Result<PathBuf> {
        let pkg_json_path = pkg_dir.join("package.json");
        let pkg = read_package_json_value(&pkg_json_path)?;

        // Priority: exports.import > exports.require > exports.default > module > main > index.js
        if let Some(exports) = pkg.get("exports") {
            if let Some(path) = resolve_exports_entry(exports) {
                let resolved = pkg_dir.join(path);
                if resolved.exists() {
                    return Ok(resolved);
                }
            }
        }

        if let Some(module) = pkg.get("module").and_then(|v| v.as_str()) {
            let resolved = pkg_dir.join(module);
            if resolved.exists() {
                return Ok(resolved);
            }
        }

        if let Some(main) = pkg.get("main").and_then(|v| v.as_str()) {
            let resolved = pkg_dir.join(main);
            if resolved.exists() {
                return Ok(resolved);
            }
        }

        // Default fallback
        let index = pkg_dir.join("index.js");
        if index.exists() {
            return Ok(index);
        }

        anyhow::bail!("Could not resolve main entry for {}", pkg_dir.display())
    }

    /// Check if the pre-bundle cache is still valid.
    ///
    /// Compares mtime of `package.json` and lockfile against cache marker,
    /// AND compares the marker's stamped resolver version against the current
    /// resolver. A resolver/patch-table change (e.g. new MUI/Emotion alias
    /// added) invalidates the cache so stale importmaps cannot survive a
    /// dependency-resolution fix. See jet#1908 AC R7.
    pub fn check_cache_valid(&self, jet_dir: &Path) -> bool {
        let marker = jet_dir.join("_cache_marker");
        if !marker.exists() {
            return false;
        }

        let marker_content = match std::fs::read_to_string(&marker) {
            Ok(c) => c,
            Err(err) => {
                tracing::warn!(
                    target: "jet::dev_server::prebundle",
                    path = %marker.display(),
                    error = %err,
                    "GH #3320 cache marker exists but is unreadable; \
                     conservatively invalidating the prebundle cache so the \
                     next startup re-bundles from scratch. Check jet_dir \
                     permissions if every startup re-bundles."
                );
                return false;
            }
        };
        if !marker_content.contains(CACHE_MARKER_VERSION) {
            return false;
        }

        let marker_mtime = match std::fs::metadata(&marker).and_then(|m| m.modified()) {
            Ok(t) => t,
            Err(err) => {
                tracing::warn!(
                    target: "jet::dev_server::prebundle",
                    path = %marker.display(),
                    error = %err,
                    "GH #3320 cache marker metadata/mtime unreadable; \
                     conservatively invalidating the prebundle cache."
                );
                return false;
            }
        };

        // GH #3117 — when an mtime probe fails for a file that EXISTS,
        // conservatively invalidate the cache. Previously the code
        // silently treated such files as "unchanged", so a transient
        // metadata error on `package.json` could serve a stale
        // pre-bundle that excluded a freshly-installed dep. Files that
        // are simply *missing* are still treated as not-in-input.
        if !manifest_mtime_within_marker(&self.root_dir.join("package.json"), marker_mtime) {
            return false;
        }

        for lockfile in [
            "jet-lock.yaml",
            "package-lock.json",
            "yarn.lock",
            "pnpm-lock.yaml",
        ] {
            if !manifest_mtime_within_marker(&self.root_dir.join(lockfile), marker_mtime) {
                return false;
            }
        }

        true
    }

    /// Scan ESM dependencies — packages in package.json that were NOT pre-bundled.
    /// Returns a map from bare specifier to entry file path (relative to node_modules/).
    /// For transitive deps only reachable via pnpm .pnpm/ store, creates symlinks in
    /// node_modules/ so the dev server can serve them.
    fn scan_esm_deps(
        &self,
        all_deps: &HashMap<String, String>,
        prebundled: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut esm = HashMap::new();
        let node_modules = self.root_dir.join("node_modules");

        // Collect all packages to scan: direct deps + all installed top-level packages
        let mut packages_to_scan: Vec<String> = all_deps.keys().cloned().collect();

        // Note: pnpm already creates symlinks for all accessible packages in node_modules/.
        // Deep scanning .pnpm/ virtual store is unnecessary and extremely slow (700+ dirs).
        // The top-level node_modules scan below picks up everything pnpm hoisted.

        // Scan ALL node_modules dirs (local + parent for monorepo hoisting)
        let mut nm_dirs = vec![node_modules.clone()];
        // Walk up to find parent node_modules (pnpm/yarn workspace hoisting)
        let mut parent = self.root_dir.parent();
        while let Some(p) = parent {
            let parent_nm = p.join("node_modules");
            if parent_nm.exists() && parent_nm != node_modules {
                nm_dirs.push(parent_nm);
            }
            // Stop at git root or filesystem root
            if p.join(".git").exists() || p.join("pnpm-workspace.yaml").exists() {
                break;
            }
            parent = p.parent();
        }

        for nm_dir in &nm_dirs {
            let entries = match std::fs::read_dir(nm_dir) {
                Ok(it) => it,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
                Err(err) => {
                    tracing::warn!(
                        target: "jet::dev::prebundle",
                        path = %nm_dir.display(),
                        error = %err,
                        "GH #3227 failed to read node_modules; importmap will be incomplete"
                    );
                    continue;
                }
            };
            for entry in entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(err) => {
                        tracing::warn!(
                            target: "jet::dev::prebundle",
                            path = %nm_dir.display(),
                            error = %err,
                            "GH #3227 unreadable dirent under node_modules; skipping"
                        );
                        continue;
                    }
                };
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                if name.starts_with('@') {
                    // Scoped packages
                    let scoped = match std::fs::read_dir(entry.path()) {
                        Ok(it) => it,
                        Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
                        Err(err) => {
                            tracing::warn!(
                                target: "jet::dev::prebundle",
                                scope = %name,
                                path = %entry.path().display(),
                                error = %err,
                                "GH #3227 failed to read scoped namespace; packages omitted from importmap"
                            );
                            continue;
                        }
                    };
                    for sub in scoped {
                        let sub = match sub {
                            Ok(s) => s,
                            Err(err) => {
                                tracing::warn!(
                                    target: "jet::dev::prebundle",
                                    scope = %name,
                                    error = %err,
                                    "GH #3227 unreadable dirent inside scoped namespace; skipping"
                                );
                                continue;
                            }
                        };
                        let full = format!("{}/{}", name, sub.file_name().to_string_lossy());
                        if !packages_to_scan.contains(&full) {
                            packages_to_scan.push(full);
                        }
                    }
                } else if !packages_to_scan.contains(&name) {
                    packages_to_scan.push(name);
                }
            }
        } // end for nm_dir

        // Process packages iteratively — discover transitive deps as we go
        let mut processed: HashSet<String> = HashSet::new();
        let mut queue: Vec<String> = packages_to_scan;

        while let Some(name) = queue.pop() {
            if processed.contains(&name) || prebundled.contains_key(&name) {
                continue;
            }
            processed.insert(name.clone());

            // Find package in any node_modules dir (local or hoisted parent)
            // Also check inside each direct dep's own node_modules (pnpm nesting)
            let pkg_dir = self.resolve_package_dir(&name, &nm_dirs);
            let Some(pkg_dir) = pkg_dir else { continue };

            // If the package is only accessible via .pnpm/, create a symlink in
            // local node_modules/ so the dev server can serve its files.
            // GH #3306 — route through the GH #3282 helper so mkdir/symlink
            // failures surface a `jet::dev::prebundle` warn instead of being
            // dropped silently (otherwise the dev server later 404s on every
            // raw asset URL under <name> with no log line linking it back).
            let local_nm_path = node_modules.join(&name);
            if !local_nm_path.exists() && pkg_dir.to_string_lossy().contains(".pnpm") {
                Self::symlink_into_node_modules(&pkg_dir, &local_nm_path, &name);
            }
            let pkg_json_path = pkg_dir.join("package.json");
            if !pkg_json_path.exists() {
                continue;
            }

            // GH #3275 — the prior `let Ok(...) else { continue };`
            // pair silently dropped this package's importmap entries
            // when its package.json was unreadable or malformed. The
            // browser then logged a confusing
            // `Failed to resolve module specifier "<name>"` at
            // runtime with no breadcrumb. Surface read/parse failures
            // via tracing::warn so operators can trace the missing
            // entries back to the offending file.
            let content = match std::fs::read_to_string(&pkg_json_path) {
                Ok(c) => c,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
                Err(err) => {
                    tracing::warn!(
                        target: "jet::dev::prebundle",
                        package = %name,
                        path = %pkg_json_path.display(),
                        error = %err,
                        "GH #3275 failed to read package.json; importmap entries for {name} will be missing"
                    );
                    continue;
                }
            };
            let content = strip_aw_claim_wrapper_lines(&content);
            let pkg: serde_json::Value = match serde_json::from_str(content.as_ref()) {
                Ok(v) => v,
                Err(err) => {
                    tracing::warn!(
                        target: "jet::dev::prebundle",
                        package = %name,
                        path = %pkg_json_path.display(),
                        error = %err,
                        "GH #3275 failed to parse package.json; importmap entries for {name} will be missing"
                    );
                    continue;
                }
            };

            // Resolve ESM entry: browser/module/import/default exports > module > main > index.js
            let entry = pkg
                .get("exports")
                .and_then(|e| {
                    if let Some(dot) = e.get(".") {
                        return resolve_exports_entry(dot);
                    }
                    resolve_exports_entry(e)
                })
                .or_else(|| pkg.get("module").and_then(|v| v.as_str()).map(String::from))
                .or_else(|| pkg.get("main").and_then(|v| v.as_str()).map(String::from))
                .unwrap_or_else(|| "index.js".to_string());

            let entry = entry.trim_start_matches("./");
            // Verify the resolved entry actually exists, fall back to index.js
            let entry = if pkg_dir.join(entry).exists() {
                entry.to_string()
            } else {
                // Try common alternatives
                ["index.js", "index.mjs", "lib/index.js"]
                    .iter()
                    .find(|f| pkg_dir.join(f).exists())
                    .map(|f| f.to_string())
                    .unwrap_or_else(|| entry.to_string())
            };
            // Add to importmap. CJS packages will be handled by the on-the-fly
            // wrapper in serve_root_file() when the browser requests them.
            esm.insert(name.clone(), format!("{}/{}", name, entry));

            if let Some(module) = pkg.get("module").and_then(|v| v.as_str()) {
                if let Some((module_dir, _)) = module.rsplit_once('/') {
                    esm.insert(format!("{}/", name), format!("{}/{}/", name, module_dir));
                }
            } else {
                esm.insert(format!("{}/", name), format!("{}/", name));
            }

            // Also add subpath exports (keys starting with "./")
            // Skip condition keys like "import", "require", "default", "types" etc.
            if let Some(exports) = pkg.get("exports").and_then(|e| e.as_object()) {
                for (key, val) in exports {
                    if !key.starts_with("./") || key == "./package.json" {
                        continue;
                    }
                    let subpath = key.trim_start_matches("./");
                    if let Some(target) = resolve_exports_entry(val) {
                        let target = target.trim_start_matches("./");
                        esm.insert(
                            format!("{}/{}", name, subpath),
                            format!("{}/{}", name, target),
                        );
                    }
                }
            }

            // GH #3288 — the prior `if let Ok(entries) = read_dir(&pkg_dir)`
            // + `entries.flatten()` silently dropped both the read_dir
            // error and every per-dirent error. Auto-discovered
            // `<pkg>/<sub>/index.mjs` subpath entries then disappeared
            // from the importmap with no breadcrumb; the browser later
            // logged a confusing `Failed to resolve module specifier`.
            let entries = match std::fs::read_dir(&pkg_dir) {
                Ok(it) => Some(it),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
                Err(err) => {
                    tracing::warn!(
                        target: "jet::dev::prebundle",
                        package = %name,
                        path = %pkg_dir.display(),
                        error = %err,
                        "GH #3288 failed to walk package dir for index.mjs subpaths; \
                         importmap entries under {name}/<sub> may be missing"
                    );
                    None
                }
            };
            if let Some(entries) = entries {
                for entry in entries {
                    let entry = match entry {
                        Ok(e) => e,
                        Err(err) => {
                            tracing::warn!(
                                target: "jet::dev::prebundle",
                                package = %name,
                                path = %pkg_dir.display(),
                                error = %err,
                                "GH #3288 unreadable dirent under package dir; subpath entry omitted"
                            );
                            continue;
                        }
                    };
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let index = path.join("index.mjs");
                    if !index.exists() {
                        continue;
                    }
                    let subpath = entry.file_name().to_string_lossy().to_string();
                    esm.entry(format!("{}/{}", name, subpath))
                        .or_insert_with(|| format!("{}/{}/index.mjs", name, subpath));
                }
            }

            // Queue transitive runtime dependencies for discovery
            if let Some(deps) = pkg.get("dependencies").and_then(|d| d.as_object()) {
                for dep_name in deps.keys() {
                    if !processed.contains(dep_name) {
                        queue.push(dep_name.clone());
                    }
                }
            }
        }

        esm
    }

    /// Materialize a `node_modules/<dep_name>` symlink pointing at `pkg_dir`.
    /// Both `create_dir_all` and `symlink` failures were previously dropped
    /// with `let _ = ...`; surface the underlying error so a broken
    /// symlink target or unwritable parent doesn't quietly leave the dep
    /// absent. `AlreadyExists` is benign — a concurrent prebundle or
    /// pre-existing entry created it between the `.exists()` check and
    /// this call.
    fn symlink_into_node_modules(pkg_dir: &Path, local_path: &Path, dep_name: &str) {
        if let Some(parent) = local_path.parent() {
            match std::fs::create_dir_all(parent) {
                Ok(()) => {}
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(err) => {
                    tracing::warn!(
                        target: "jet::dev::prebundle",
                        package = %dep_name,
                        path = %parent.display(),
                        error = %err,
                        "GH #3282 failed to create node_modules parent for {dep_name}; \
                         dev server may 404 on raw asset URLs under this package"
                    );
                    return;
                }
            }
        }
        match std::os::unix::fs::symlink(pkg_dir, local_path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(err) => {
                tracing::warn!(
                    target: "jet::dev::prebundle",
                    package = %dep_name,
                    target_path = %pkg_dir.display(),
                    link_path = %local_path.display(),
                    error = %err,
                    "GH #3282 failed to symlink {dep_name} into local node_modules; \
                     dev server may 404 on raw asset URLs under this package"
                );
            }
        }
    }

    /// Resolve a package directory by searching node_modules dirs and pnpm nested paths.
    fn resolve_package_dir(&self, name: &str, nm_dirs: &[PathBuf]) -> Option<PathBuf> {
        // Direct lookup in each node_modules dir
        for nm in nm_dirs {
            let candidate = nm.join(name);
            if candidate.join("package.json").exists() {
                return Some(candidate);
            }
        }
        // pnpm: check .pnpm/node_modules/<name> (hoisted transitive deps)
        for nm in nm_dirs {
            let candidate = nm.join(".pnpm/node_modules").join(name);
            if candidate.join("package.json").exists() {
                return Some(candidate);
            }
        }
        // pnpm nesting: check inside .pnpm/<pkg-version>/node_modules/<name>.
        //
        // GH #3296 — the prior implementation silently swallowed two IO
        // failure shapes here:
        //   * `let Ok(entries) = read_dir(.pnpm) else continue;` — an
        //     unreadable `.pnpm/` (EACCES, transient IO) silently
        //     downgraded the whole bucket to "package not found", so the
        //     dev server reported a bogus "Could not resolve foo" while
        //     the package was sitting in the virtual store. NotFound is
        //     impossible here because `pnpm_dir.exists()` was just
        //     checked, so any other read_dir error is unexpected.
        //   * `entries.flatten()` — per-dirent IO during the walk dropped
        //     individual pnpm buckets, again silently truncating the
        //     search.
        // Both now warn under `target = "jet::dev::prebundle"`.
        for nm in nm_dirs {
            let pnpm_dir = nm.join(".pnpm");
            if !pnpm_dir.exists() {
                continue;
            }
            let entries = match std::fs::read_dir(&pnpm_dir) {
                Ok(it) => it,
                Err(err) => {
                    tracing::warn!(
                        target: "jet::dev::prebundle",
                        path = %pnpm_dir.display(),
                        pkg = name,
                        error = %err,
                        "GH #3296 unreadable .pnpm/ during pnpm-nested package resolution; \
                         `{name}` may be misreported as unresolvable while it is in the virtual store"
                    );
                    continue;
                }
            };
            for entry in entries {
                let entry = match entry {
                    Ok(e) => e,
                    Err(err) => {
                        tracing::warn!(
                            target: "jet::dev::prebundle",
                            path = %pnpm_dir.display(),
                            pkg = name,
                            error = %err,
                            "GH #3296 unreadable dirent during pnpm-nested package resolution; \
                             a pnpm bucket for `{name}` may be silently skipped"
                        );
                        continue;
                    }
                };
                if entry.file_name() == "node_modules" {
                    continue;
                }
                let candidate = entry.path().join("node_modules").join(name);
                if candidate.join("package.json").exists() {
                    return Some(candidate);
                }
            }
        }
        None
    }

    /// Write a cache marker file to record the current pre-bundle time.
    ///
    /// The marker contents embed `CACHE_MARKER_VERSION` so a resolver or
    /// patch-table change automatically invalidates every project's cached
    /// importmap (jet#1908 AC R7).
    fn write_cache_marker(&self, jet_dir: &Path) -> Result<()> {
        let marker = jet_dir.join("_cache_marker");
        std::fs::write(
            &marker,
            format!("prebundle cache marker {}", CACHE_MARKER_VERSION),
        )?;
        Ok(())
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
impl PreBundler {
    /// Discover transitive deps from pnpm .pnpm/ virtual store.
    /// Creates symlinks in node_modules/ for packages that aren't directly accessible.
    #[allow(dead_code)]
    fn discover_pnpm_deps(node_modules: &std::path::Path, packages: &mut Vec<String>) {
        // Walk all parent node_modules for .pnpm dirs
        let mut search_dirs = vec![node_modules.to_path_buf()];
        if let Some(parent) = node_modules.parent() {
            let mut p = parent.parent();
            while let Some(pp) = p {
                let nm = pp.join("node_modules");
                if nm.join(".pnpm").exists() {
                    search_dirs.push(nm);
                }
                if pp.join(".git").exists() || pp.join("pnpm-workspace.yaml").exists() {
                    break;
                }
                p = pp.parent();
            }
        }

        for nm_dir in &search_dirs {
            let pnpm_dir = nm_dir.join(".pnpm");
            if !pnpm_dir.exists() {
                continue;
            }

            // Scan .pnpm/*/node_modules/*
            if let Ok(entries) = std::fs::read_dir(&pnpm_dir) {
                for entry in entries.flatten() {
                    let inner_nm = entry.path().join("node_modules");
                    if !inner_nm.exists() {
                        continue;
                    }
                    if let Ok(inner_entries) = std::fs::read_dir(&inner_nm) {
                        for pkg_entry in inner_entries.flatten() {
                            let name = pkg_entry.file_name().to_string_lossy().to_string();
                            if name.starts_with('.') {
                                continue;
                            }
                            if name.starts_with('@') {
                                if let Ok(scoped) = std::fs::read_dir(pkg_entry.path()) {
                                    for sub in scoped.flatten() {
                                        let full = format!(
                                            "{}/{}",
                                            name,
                                            sub.file_name().to_string_lossy()
                                        );
                                        if !packages.contains(&full) {
                                            // Create symlink so serve_root_file can find it
                                            let link = node_modules.join(&full);
                                            if !link.exists() {
                                                let _ =
                                                    std::fs::create_dir_all(link.parent().unwrap());
                                                let _ =
                                                    std::os::unix::fs::symlink(sub.path(), &link);
                                            }
                                            packages.push(full);
                                        }
                                    }
                                }
                            } else if !packages.contains(&name) {
                                let link = node_modules.join(&name);
                                if !link.exists() {
                                    let _ = std::os::unix::fs::symlink(pkg_entry.path(), &link);
                                }
                                packages.push(name);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn split_bare_subpath_import(specifier: &str) -> Option<(String, String)> {
    if specifier.starts_with('.') || specifier.starts_with('/') {
        return None;
    }
    let parts = specifier.split('/').collect::<Vec<_>>();
    if parts.is_empty() {
        return None;
    }
    if specifier.starts_with('@') {
        if parts.len() < 3 {
            return None;
        }
        return Some((format!("{}/{}", parts[0], parts[1]), parts[2..].join("/")));
    }
    if parts.len() < 2 {
        return None;
    }
    Some((parts[0].to_string(), parts[1..].join("/")))
}

/// Check if a package directory is a workspace symlink (points to local packages/).
/// pnpm symlinks ALL packages, but workspace packages point to the project's own
/// packages/ directory, not to .pnpm/ store.
fn is_workspace_symlink(pkg_dir: &std::path::Path) -> bool {
    match std::fs::read_link(pkg_dir) {
        Ok(target) => {
            let target_str = target.to_string_lossy();
            // Workspace symlinks point to packages/@cclab/*, packages/*, etc.
            // pnpm store symlinks point to .pnpm/
            !target_str.contains(".pnpm")
                && (target_str.contains("/packages/") || target_str.contains("\\packages\\"))
        }
        Err(_) => false, // not a symlink
    }
}

/// Check if a package directory contains local source (workspace package resolved via .pnpm).
/// These packages have TypeScript source that should be served directly, not pre-bundled.
fn is_local_source_package(pkg_dir: &std::path::Path) -> bool {
    // Follow symlink and check if target is within .pnpm/node_modules (workspace hoisted)
    if let Ok(target) = std::fs::read_link(pkg_dir) {
        let target_str = target.to_string_lossy();
        if target_str.contains(".pnpm/node_modules/") {
            // Check if the resolved package has TypeScript source (workspace indicator)
            let src_dir = pkg_dir.join("src");
            if src_dir.exists() {
                return true;
            }
        }
    }
    false
}

/// Create a virtual ESM entry for a CJS package.
///
/// The entry re-exports everything from the package, allowing the bundler
/// to convert CJS → ESM.
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn create_virtual_entry(package_name: &str) -> String {
    format!(
        "export * from '{pkg}';\nexport {{ default }} from '{pkg}';\n",
        pkg = package_name
    )
}

/// Generate the `.jet/` filename for a dependency.
///
/// Handles scoped packages: `@tanstack/react-query` → `@tanstack__react-query.mjs`
/// Handles subpaths: `react/jsx-runtime` → `react_jsx-runtime.mjs` (single underscore)
fn dep_filename(specifier: &str) -> String {
    // Scoped packages: @scope/name → @scope__name (double underscore for scope separator)
    // Subpath exports: react/jsx-runtime → react_jsx-runtime (single underscore)
    let sanitized = if specifier.starts_with('@') {
        // Split at first / (scope separator), then replace remaining / with _
        let parts: Vec<&str> = specifier.splitn(2, '/').collect();
        if parts.len() == 2 {
            let scope_and_name: Vec<&str> = parts[1].splitn(2, '/').collect();
            if scope_and_name.len() == 2 {
                // @scope/name/subpath → @scope__name_subpath
                format!("{}__{}_{}", parts[0], scope_and_name[0], scope_and_name[1])
            } else {
                // @scope/name → @scope__name
                format!("{}__{}", parts[0], parts[1])
            }
        } else {
            specifier.to_string()
        }
    } else {
        // Non-scoped: react/jsx-runtime → react_jsx-runtime (single underscore)
        specifier.replace('/', "_")
    };
    format!("{}.mjs", sanitized)
}

/// Check if an exports map contains an `"import"` condition (recursively).
fn has_import_condition(exports: &serde_json::Value) -> bool {
    match exports {
        serde_json::Value::Object(map) => {
            if map.contains_key("import") {
                return true;
            }
            // Check nested: { ".": { "import": ... } }
            for v in map.values() {
                if has_import_condition(v) {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

/// Resolve the best entry point from an exports map.
///
/// Priority: `browser` > `module` > `import` > `development` > `require` > `default` > `.`
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub(super) fn resolve_exports_entry(exports: &serde_json::Value) -> Option<String> {
    match exports {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Object(map) => {
            for key in [
                "browser",
                "module",
                "import",
                "development",
                "require",
                "default",
            ] {
                if let Some(value) = map.get(key) {
                    if let Some(resolved) = resolve_exports_entry(value) {
                        return Some(resolved);
                    }
                }
            }
            // Nested: { ".": { "import": ... } }
            if let Some(dot) = map.get(".") {
                return resolve_exports_entry(dot);
            }
            None
        }
        _ => None,
    }
}

/// Resolve the best exports entry with explicit condition priority.
///
/// Priority: `import` > `require` > `default`
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn resolve_exports_condition(exports: &serde_json::Value) -> Option<String> {
    resolve_exports_entry(exports)
}

/// Detect circular `require()` relationships between pre-bundled packages.
///
/// Builds a directed graph from `require('pkg')` calls found in each
/// pre-bundled source, then finds cycles using iterative DFS.
/// Returns a list of cycles, each represented as a Vec of package names
/// forming the cycle (e.g. `["A", "B", "A"]`).
/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub fn detect_circular_deps(prebundled_sources: &HashMap<String, String>) -> Vec<Vec<String>> {
    // Build adjacency list: package → set of required packages
    let pkg_names: HashSet<&str> = prebundled_sources.keys().map(|s| s.as_str()).collect();
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();

    for (pkg, source) in prebundled_sources {
        let mut edges = Vec::new();
        // Scan for require('dep') patterns
        let needle = "require(";
        let mut pos = 0;
        while let Some(idx) = source[pos..].find(needle) {
            let start = pos + idx + needle.len();
            if start >= source.len() {
                break;
            }
            let quote = source.as_bytes()[start];
            if quote == b'\'' || quote == b'"' {
                let inner_start = start + 1;
                if let Some(end) = source[inner_start..].find(quote as char) {
                    let dep = &source[inner_start..inner_start + end];
                    if pkg_names.contains(dep) && dep != pkg.as_str() {
                        edges.push(dep);
                    }
                }
            }
            pos = start + 1;
        }
        graph.insert(pkg.as_str(), edges);
    }

    // DFS-based cycle detection
    let mut cycles = Vec::new();
    let mut visited: HashSet<&str> = HashSet::new();
    let mut on_stack: HashSet<&str> = HashSet::new();
    let mut stack: Vec<&str> = Vec::new();

    for &start in &pkg_names {
        if visited.contains(start) {
            continue;
        }
        // Iterative DFS using explicit stack of (node, edge_index)
        let mut dfs: Vec<(&str, usize)> = vec![(start, 0)];
        on_stack.insert(start);
        stack.push(start);

        while let Some((node, idx)) = dfs.last_mut() {
            let edges = graph.get(*node).map(|v| v.as_slice()).unwrap_or(&[]);
            if *idx < edges.len() {
                let next = edges[*idx];
                *idx += 1;
                if on_stack.contains(next) {
                    // Found a cycle — extract it from the stack
                    let cycle_start = stack.iter().position(|&n| n == next).unwrap();
                    let mut cycle: Vec<String> =
                        stack[cycle_start..].iter().map(|s| s.to_string()).collect();
                    cycle.push(next.to_string()); // close the cycle
                    cycles.push(cycle);
                } else if !visited.contains(next) {
                    on_stack.insert(next);
                    stack.push(next);
                    dfs.push((next, 0));
                }
            } else {
                let (done, _) = dfs.pop().unwrap();
                on_stack.remove(done);
                stack.pop();
                visited.insert(done);
            }
        }
    }

    cycles
}

/// Simple CJS → ESM conversion.
///
/// Wraps the CJS source in an ESM-compatible format. Uses a lightweight
/// wrapper instead of `crate::bundler::Bundler` for faster dev startup.
/// The Bundler requires a full module graph (resolver + transformer + asset
/// processor) which is too heavy for per-dep pre-bundling. The wrapper
/// approach meets the <3s startup target for ~20 CJS deps.
// TODO(#1089): Migrate to crate::bundler::Bundler for proper tree-shaking
// and require→import conversion. Requires Bundler API extension to support
// single-dependency mode without a full project graph. Track in follow-up.
/// Inline relative `require('./...')` calls by replacing them with the file contents
/// wrapped in an IIFE. This resolves patterns like React's index.js which does:
///   module.exports = require('./cjs/react.development.js')
///
/// Only inlines relative requires (starting with `.`). External requires are left as-is.
/// Recurses up to 3 levels deep to handle chained requires.
fn inline_requires(source: &str, base_dir: &Path) -> String {
    inline_requires_depth(source, base_dir, 0)
}

fn inline_requires_depth(source: &str, base_dir: &Path, depth: usize) -> String {
    if depth > 3 {
        return source.to_string();
    }

    let mut result = source.to_string();
    // Match require('./relative/path') patterns
    let re = regex::Regex::new(r#"require\(\s*['"](\./[^'"]+)['"]\s*\)"#).unwrap();

    // Collect matches first to avoid borrow issues
    let matches: Vec<(String, String)> = re
        .captures_iter(&result)
        .map(|cap| (cap[0].to_string(), cap[1].to_string()))
        .collect();

    for (full_match, rel_path) in matches {
        // Resolve the path
        let mut target = base_dir.join(&rel_path);
        if !target.exists() {
            // Try adding .js extension
            target = base_dir.join(format!("{}.js", rel_path));
        }
        if !target.exists() {
            continue;
        }

        // GH #3312 — `target.exists()` already returned true, so a read
        // failure here is a real IO error. The prior `let Ok else continue`
        // silently left the `require(...)` un-inlined, which then blew up
        // at runtime as `ReferenceError: require is not defined` with no
        // log line linking the runtime error to the unreadable source.
        // NotFound is treated as a legitimate race (deletion/rename between
        // the `exists()` check and the read) and stays quiet.
        let content = match std::fs::read_to_string(&target) {
            Ok(c) => c,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(err) => {
                tracing::warn!(
                    target: "jet::dev::prebundle",
                    base_dir = %base_dir.display(),
                    target_path = %target.display(),
                    depth = depth,
                    error = %err,
                    "GH #3312 unreadable source during CJS require() inlining; \
                     the un-inlined require(...) call will reach the browser \
                     and fail at runtime"
                );
                continue;
            }
        };
        // Replace process.env.NODE_ENV in inlined content too
        let content = content.replace("process.env.NODE_ENV", "'development'");
        // Recursively inline nested requires
        let content = inline_requires_depth(&content, target.parent().unwrap(), depth + 1);

        // Replace the require() call with an IIFE that executes the inlined module
        let replacement = format!(
            "(function() {{ var module = {{ exports: {{}} }}; var exports = module.exports;\n{}\nreturn module.exports; }})()",
            content
        );
        result = result.replace(&full_match, &replacement);
    }

    result
}

fn convert_cjs_to_esm(source: &str, _name: &str, dep_imports: &HashMap<String, String>) -> String {
    // Check if it's already ESM-ish
    if is_esm_module_source(source) {
        return source.to_string();
    }

    // Extract named exports from CJS pattern (exports.X = ...)
    let named = super::extract_named_reexports(source);

    // Collect require('...') dependencies so the shim can resolve them
    let deps = collect_external_requires(source);

    // Generate import statements + require shim for collected dependencies
    let mut imports = String::new();
    let mut require_cases = String::new();
    for (i, dep) in deps.iter().enumerate() {
        let var_name = format!("__cjs_dep_{}__", i);
        let filename = dep_imports
            .get(dep)
            .cloned()
            .unwrap_or_else(|| dep_filename(dep));
        let dep_path = format!("/node_modules/.jet/{}", filename);
        imports.push_str(&format!("import {} from '{}';\n", var_name, dep_path));
        require_cases.push_str(&format!("    if (id === '{}') return {};\n", dep, var_name));
    }

    // Wrap CJS module in ESM
    let mut result = format!(
        r#"// CJS → ESM wrapper
{imports}var module = {{ exports: {{}} }};
var exports = module.exports;

(function(module, exports, require) {{
{source}
}})(module, exports, function require(id) {{
{require_cases}  console.warn('[jet] Dynamic require("' + id + '") — no pre-bundled module found');
  return {{}};
}});

export default module.exports;
"#,
    );
    if !named.is_empty() {
        result.push_str(&named);
        result.push('\n');
    }
    result
}

fn collect_external_requires(source: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let require_scan_source = strip_js_comments_for_require_scan(source);
    for cap in regex::Regex::new(r#"require\s*\(\s*['"]([^'"]+)['"]\s*\)"#)
        .unwrap()
        .captures_iter(&require_scan_source)
    {
        let dep = cap[1].to_string();
        if dep.starts_with('.') || dep.starts_with('/') {
            continue;
        }
        if !deps.contains(&dep) {
            deps.push(dep);
        }
    }
    deps
}

fn resolve_nested_package_dir(parent_pkg_dir: &Path, dep: &str) -> Option<PathBuf> {
    let candidate = parent_pkg_dir.join("node_modules").join(dep);
    if candidate.join("package.json").exists() {
        Some(candidate)
    } else {
        None
    }
}

fn resolve_root_package_dir(jet_dir: &Path, dep: &str) -> Option<PathBuf> {
    let node_modules = jet_dir.parent()?;
    let candidate = node_modules.join(dep);
    if candidate.join("package.json").exists() {
        Some(candidate)
    } else {
        None
    }
}

fn package_versions_match(left: &Path, right: &Path) -> bool {
    package_version(left).is_some_and(|left_version| {
        package_version(right).is_some_and(|right_version| left_version == right_version)
    })
}

fn package_version(pkg_dir: &Path) -> Option<String> {
    let parsed = read_package_json_value(&pkg_dir.join("package.json")).ok()?;
    parsed
        .get("version")
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

fn strip_js_comments_for_require_scan(source: &str) -> String {
    let block = regex::Regex::new(r"(?s)/\*.*?\*/").unwrap();
    let without_block = block.replace_all(source, "");
    let line = regex::Regex::new(r"(?m)//.*$").unwrap();
    line.replace_all(&without_block, "").into_owned()
}

fn is_esm_module_source(source: &str) -> bool {
    regex::Regex::new(
        r#"(?m)^\s*(?:import\s+(?:[\w*{]|\{|"|')|export\s+(?:default|\{|\*|function|class|const|var|let))"#,
    )
    .unwrap()
    .is_match(source)
}

fn manifest_mtime_within_marker(
    path: &std::path::Path,
    marker_mtime: std::time::SystemTime,
) -> bool {
    match std::fs::metadata(path) {
        Ok(meta) => match meta.modified() {
            Ok(mtime) => mtime <= marker_mtime,
            // File exists but mtime is unreadable — conservatively invalidate.
            Err(_) => false,
        },
        // Missing files are legitimately not in this run's input set.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => true,
        // Any other metadata error (EACCES, transient I/O) — conservatively invalidate.
        Err(_) => false,
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
pub(crate) fn write_importmap_cache(jet_dir: &Path, importmap_json: &str) {
    let importmap_cache = jet_dir.join("_importmap.json");
    if let Err(e) = std::fs::write(&importmap_cache, importmap_json) {
        tracing::warn!(
            target: "jet::dev_server::prebundle",
            "failed to cache importmap to {:?}: {e}; the next `jet dev` startup \
             will re-bundle every CJS dep from scratch instead of using the cache \
             (GH #3188)",
            importmap_cache
        );
    }
}

#[cfg(test)]
#[path = "prebundle_tests.rs"]
mod tests;
// CODEGEN-END
