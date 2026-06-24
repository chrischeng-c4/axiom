// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
// CODEGEN-BEGIN
use anyhow::{Context, Result};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::npmrc::NpmrcConfig;
use super::workspace::WorkspaceManager;

/// Format the warning emitted when WorkspaceManager::discover fails.
///
/// The message names the project root verbatim, preserves the underlying
/// error, and hints at the downstream symptom (`workspace:*` deps will be
/// uploaded unresolved → registry rejects or downstream installs fail).
/// Tagged `GH #3530` so users grepping their publish logs for a confusing
/// "Invalid version" or "workspace:" error can land on this line.
///
/// Extracted as a free function so unit tests can pin the message shape
/// without inspecting log capture.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_workspace_discover_warn(root: &Path, err: &anyhow::Error) -> String {
    format!(
        "GH #3530 workspace discovery failed under {}: {}; `workspace:*` deps will NOT be resolved before upload — the publish will likely be rejected by the registry, or downstream installs will fail. Check jet-workspace.yaml / pnpm-workspace.yaml / package.json for parse errors.",
        root.display(),
        err
    )
}

/// GH #3570 — extract the `name` and `version` fields from a parsed
/// `package.json` and fail loudly if either is missing or not a string.
///
/// The prior code (`pkg["name"].as_str().unwrap_or("package")` and
/// `pkg["version"].as_str().unwrap_or("0.0.0")`) silently fell back to
/// placeholder values when the package.json was malformed, producing:
/// - the wrong tarball uploaded to npm as `package@0.0.0.tgz`
/// - the wrong registry/auth-token lookup (scoped `@company/foo` would
///   resolve to the public registry under the placeholder name — a
///   real security/leak signal)
/// - a publish_body the registry would either reject confusingly or
///   accept as a name the dev never intended
///
/// The Err naming the offending field, observed JSON kind, and the
/// package.json path lets the dev land on the actual cause from a
/// single error line.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn require_publish_identity(
    pkg: &serde_json::Value,
    package_json_path: &Path,
) -> Result<(String, String)> {
    let name = require_publish_string_field(pkg, "name", package_json_path)?;
    let version = require_publish_string_field(pkg, "version", package_json_path)?;
    Ok((name, version))
}

fn require_publish_string_field(
    pkg: &serde_json::Value,
    field: &str,
    package_json_path: &Path,
) -> Result<String> {
    match pkg.get(field) {
        Some(serde_json::Value::String(v)) if !v.is_empty() => Ok(v.clone()),
        Some(serde_json::Value::String(_)) => Err(anyhow::anyhow!(
            "{}",
            format_publish_identity_err(package_json_path, field, "empty-string",)
        )),
        Some(other) => Err(anyhow::anyhow!(
            "{}",
            format_publish_identity_err(
                package_json_path,
                field,
                describe_publish_field_kind(other),
            )
        )),
        None => Err(anyhow::anyhow!(
            "{}",
            format_publish_identity_err(package_json_path, field, "missing")
        )),
    }
}

/// GH #3570 — build the error message for a missing/non-string publish
/// identity field. Extracted so the wording (path + field + kind + tag
/// + consequence) is unit-testable.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_publish_identity_err(
    package_json_path: &Path,
    field: &str,
    observed_kind: &str,
) -> String {
    format!(
        "GH #3570 cannot publish: package.json at {} has no usable `{field}` \
         string (observed: {observed_kind}); refusing the silent fallback \
         that would have published the wrong tarball identity to npm and \
         mis-routed the registry/auth-token lookup. Add a valid \
         `\"{field}\": \"...\"` entry and retry.",
        package_json_path.display()
    )
}

/// GH #3570 — describe the JSON kind of a value for the publish-identity
/// error message so the dev can tell from the message what was actually
/// observed at the `name`/`version` slot.
fn describe_publish_field_kind(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

/// Publisher for npm registry.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub struct Publisher {
    root_dir: PathBuf,
    npmrc: NpmrcConfig,
    /// When set, run a library build (`jet build --lib`) before packing so the
    /// tarball contains freshly-built dist files. Opt-in — defaults to `false`
    /// so the historical `jet publish` / `jet pack` behaviour is unchanged.
    /// @issue #172
    build_first: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl Publisher {
    pub fn new(root_dir: PathBuf) -> Self {
        let npmrc = NpmrcConfig::load(&root_dir);
        Self {
            root_dir,
            npmrc,
            build_first: false,
        }
    }

    /// Enable the build-before-publish path: a library build runs and any
    /// missing `main`/`module`/`types` field is auto-filled from the build
    /// output before packing/publishing. Opt-in (default off).
    /// @issue #172
    pub fn with_build(mut self, build_first: bool) -> Self {
        self.build_first = build_first;
        self
    }

    /// Create a tarball (.tgz) for publishing without actually publishing.
    pub fn pack(&self) -> Result<PathBuf> {
        let mut pkg = self.read_and_transform_package_json()?;
        let package_json_path = self.root_dir.join("package.json");
        let (name, version) = require_publish_identity(&pkg, &package_json_path)?;

        // @issue #172 — optionally build first, then validate that every
        // metadata field (main/module/exports/types) points at a real file
        // that the tarball will contain.
        self.prepare_for_packing(&mut pkg)?;

        let tarball_name = format!(
            "{}-{}.tgz",
            name.replace('/', "-").trim_start_matches('@'),
            version
        );
        let tarball_path = self.root_dir.join(&tarball_name);

        self.create_tarball(&tarball_path, &pkg)?;
        tracing::info!("Created tarball: {}", tarball_name);
        Ok(tarball_path)
    }

    /// Publish to npm registry.
    pub async fn publish(&self, tag: &str, access: Option<&str>) -> Result<()> {
        let mut pkg = self.read_and_transform_package_json()?;
        let package_json_path = self.root_dir.join("package.json");
        let (name_owned, version_owned) = require_publish_identity(&pkg, &package_json_path)?;

        // @issue #172 — optionally build first, then validate the package
        // metadata before we contact the registry. Doing this before the
        // auth/registry lookup means a misconfigured package.json fails with a
        // clear filesystem error instead of a confusing registry rejection.
        self.prepare_for_packing(&mut pkg)?;

        let name = name_owned.as_str();
        let version = version_owned.as_str();

        let registry = self.npmrc.registry_for(name);
        let auth_token = self.npmrc.auth_token_for(registry).ok_or_else(|| {
            anyhow::anyhow!(
                "No auth token found for registry {}. Add to .npmrc.",
                registry
            )
        })?;

        // Create tarball in memory
        let tarball_bytes = self.create_tarball_bytes(&pkg)?;

        // Publish via npm registry PUT
        let url = format!("{}/{}", registry.trim_end_matches('/'), name);
        let client = reqwest::Client::new();

        let encoded = base64_encode(&tarball_bytes);
        let publish_body = serde_json::json!({
            "name": name,
            "description": pkg.get("description").and_then(|v| v.as_str()).unwrap_or(""),
            "dist-tags": { tag: version },
            "versions": {
                version: pkg
            },
            "access": access.unwrap_or("public"),
            "_attachments": {
                format!("{}-{}.tgz", name, version): {
                    "content_type": "application/octet-stream",
                    "data": encoded,
                    "length": tarball_bytes.len()
                }
            }
        });

        let response = client
            .put(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .json(&publish_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Publish failed ({}): {}", status, body);
        }

        tracing::info!("Published {}@{} with tag '{}'", name, version, tag);
        Ok(())
    }

    /// @issue #172 — the build-then-validate step shared by `pack` and
    /// `publish`. When `build_first` is set, run a library build and auto-fill
    /// any absent `main`/`module`/`types` field from the build output. Then
    /// validate that every metadata target (`main`/`module`/`exports`/`types`)
    /// resolves to a file that exists under the project root (i.e. will be in
    /// the tarball). A missing target is a hard error.
    fn prepare_for_packing(&self, pkg: &mut serde_json::Value) -> Result<()> {
        if self.build_first {
            let result = self.run_build()?;
            auto_fill_metadata(pkg, &result, &self.root_dir);
        }
        validate_package_metadata(pkg, &self.root_dir)?;
        Ok(())
    }

    /// @issue #172 — run `jet build --lib` for the project and return the
    /// build result so callers can auto-fill metadata fields from the emitted
    /// outputs. Reuses [`crate::bundler::build_library`].
    fn run_build(&self) -> Result<crate::bundler::LibBuildResult> {
        use crate::bundler::types::OutputFormat;
        use crate::bundler::LibBuildOptions;

        let options = LibBuildOptions {
            project_root: self.root_dir.clone(),
            out_dir: self.root_dir.join("dist"),
            // Emit both ESM and CJS so `module`/`main` can both be auto-filled.
            formats: vec![OutputFormat::Esm, OutputFormat::Cjs],
            ..Default::default()
        };
        crate::bundler::build_library(options)
            .context("jet publish --build: library build failed")
    }

    /// Read package.json and transform workspace:* protocols to real versions.
    fn read_and_transform_package_json(&self) -> Result<serde_json::Value> {
        let path = self.root_dir.join("package.json");
        let content =
            std::fs::read_to_string(&path).with_context(|| format!("Failed to read {:?}", path))?;
        let mut pkg: serde_json::Value = serde_json::from_str(&content)?;

        // GH #3530 — surface workspace discovery failures instead of
        // silently uploading a broken package.json. The prior
        // `if let Ok(Some(ws)) = ...` shortcut dropped any Err from
        // WorkspaceManager::discover (malformed jet-workspace.yaml /
        // pnpm-workspace.yaml / package.json), causing the publish to
        // proceed with `workspace:*` deps unresolved. The npm registry
        // then either rejects the publish with a confusing version error
        // or — worse — accepts it and downstream installs all fail with
        // no breadcrumb pointing at the actual malformed workspace config.
        match WorkspaceManager::discover(&self.root_dir) {
            Ok(Some(ws)) => {
                Self::transform_workspace_deps(&mut pkg, &ws, "dependencies");
                Self::transform_workspace_deps(&mut pkg, &ws, "devDependencies");
            }
            Ok(None) => {
                // Not a workspace — `workspace:*` deps, if any, will be
                // caught downstream by the registry. Nothing to do here.
            }
            Err(err) => {
                tracing::warn!(
                    target: "jet::pkg_manager::publish",
                    root = %self.root_dir.display(),
                    error = %err,
                    "{}",
                    format_workspace_discover_warn(&self.root_dir, &err)
                );
            }
        }

        Ok(pkg)
    }

    /// Replace workspace:* specs with real version ranges.
    fn transform_workspace_deps(pkg: &mut serde_json::Value, ws: &WorkspaceManager, field: &str) {
        if let Some(deps) = pkg.get_mut(field).and_then(|v| v.as_object_mut()) {
            for (name, version) in deps.iter_mut() {
                if let Some(spec) = version.as_str() {
                    if WorkspaceManager::is_workspace_protocol(spec) {
                        if let Some(resolved) = ws.resolve_workspace_protocol(spec, name) {
                            *version = serde_json::Value::String(resolved);
                        }
                    }
                }
            }
        }
    }

    fn create_tarball(&self, output: &Path, _pkg: &serde_json::Value) -> Result<()> {
        let bytes = self.create_tarball_bytes(_pkg)?;
        std::fs::write(output, bytes)?;
        Ok(())
    }

    fn create_tarball_bytes(&self, _pkg: &serde_json::Value) -> Result<Vec<u8>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let tar_buf = Vec::new();
        let mut builder = tar::Builder::new(tar_buf);

        // Add package.json (transformed)
        let pkg_content = serde_json::to_string_pretty(_pkg)?;
        let mut header = tar::Header::new_gnu();
        header.set_size(pkg_content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append_data(&mut header, "package/package.json", pkg_content.as_bytes())?;

        // Add other publishable files
        let publish_files = Self::collect_publish_files(&self.root_dir)?;
        for file in publish_files {
            let relative = file.strip_prefix(&self.root_dir)?;
            let content = std::fs::read(&file)?;
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(
                &mut header,
                Path::new("package").join(relative),
                content.as_slice(),
            )?;
        }

        let tar_bytes = builder.into_inner()?;
        let mut gz = GzEncoder::new(Vec::new(), Compression::default());
        gz.write_all(&tar_bytes)?;
        Ok(gz.finish()?)
    }

    /// Collect files to include in the package, matching npm pack semantics.
    ///
    /// Three modes, in priority order:
    ///   1. **`files` allowlist** — if `package.json` has a `files` array,
    ///      include ONLY paths matching those entries. A bare directory entry
    ///      (`"dist"`) means the whole directory; glob patterns
    ///      (`"dist/**"`, `"lib/*.js"`) are honoured.
    ///   2. **`.npmignore`** — else, if a `.npmignore` exists at the root,
    ///      walk the tree minus the gitignore-style patterns it lists.
    ///   3. **default** — else, walk the tree skipping the historical
    ///      non-publishable dirs.
    ///
    /// In every mode: `package.json` is never returned here (the transformed
    /// copy is appended by the caller), `node_modules`/`.git` are NEVER
    /// included, and `README*` / `LICENSE*` at the root are ALWAYS included
    /// if present (npm forces them in regardless of `files`/`.npmignore`).
    fn collect_publish_files(root: &Path) -> Result<Vec<PathBuf>> {
        let pkg_json = root.join("package.json");
        let files_field = std::fs::read_to_string(&pkg_json)
            .ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .and_then(|v| v.get("files").cloned())
            .and_then(|f| f.as_array().cloned())
            .map(|arr| {
                arr.into_iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            });

        let mut collected: Vec<PathBuf> = match files_field {
            Some(patterns) if !patterns.is_empty() => Self::collect_from_files_allowlist(root, &patterns)?,
            _ => {
                let npmignore = root.join(".npmignore");
                if npmignore.is_file() {
                    Self::collect_from_npmignore(root, &npmignore)?
                } else {
                    Self::collect_default(root)?
                }
            }
        };

        // npm always forces README* and LICENSE* in, regardless of the
        // `files` allowlist or `.npmignore`. Add them (deduped) if present.
        Self::ensure_always_included(root, &mut collected);

        Ok(collected)
    }

    /// Mode 1: include only files matching the `files` allowlist patterns.
    ///
    /// Each entry is matched two ways so both npm conventions work:
    ///   * as a directory prefix — `"dist"` includes everything under `dist/`;
    ///   * as a glob — `"dist/**"`, `"lib/*.js"`, `"*.md"`.
    fn collect_from_files_allowlist(root: &Path, patterns: &[String]) -> Result<Vec<PathBuf>> {
        use globset::{Glob, GlobSetBuilder};

        // Build a globset from the entries. For a bare entry like "dist" we
        // also register "dist/**" so the whole subtree matches, mirroring npm.
        let mut builder = GlobSetBuilder::new();
        let mut dir_prefixes: Vec<String> = Vec::new();
        for raw in patterns {
            let pat = raw.trim_start_matches("./").trim_end_matches('/');
            if pat.is_empty() {
                continue;
            }
            // The entry itself (a file path or a literal glob).
            if let Ok(g) = Glob::new(pat) {
                builder.add(g);
            }
            // Treat it as a directory too: include the whole subtree.
            if !pat.contains('*') {
                if let Ok(g) = Glob::new(&format!("{pat}/**")) {
                    builder.add(g);
                }
                dir_prefixes.push(pat.to_string());
            }
        }
        let set = builder.build().context("invalid pattern in package.json `files`")?;

        let mut files = Vec::new();
        for entry in Self::publishable_walk(root) {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.file_name().to_string_lossy() == "package.json" {
                continue; // Already added transformed.
            }
            let rel = match entry.path().strip_prefix(root) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            let in_dir = dir_prefixes
                .iter()
                .any(|d| rel_str == *d || rel_str.starts_with(&format!("{d}/")));
            if set.is_match(rel) || in_dir {
                files.push(entry.path().to_path_buf());
            }
        }
        Ok(files)
    }

    /// Mode 2: walk the tree, excluding paths matching `.npmignore` patterns
    /// (gitignore-style — bare names match anywhere, directory entries exclude
    /// the whole subtree, `*` globs are honoured).
    fn collect_from_npmignore(root: &Path, npmignore: &Path) -> Result<Vec<PathBuf>> {
        use globset::{Glob, GlobSetBuilder};

        let raw = std::fs::read_to_string(npmignore).unwrap_or_default();
        let mut builder = GlobSetBuilder::new();
        let mut names: Vec<String> = Vec::new();
        for line in raw.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let pat = line.trim_start_matches('/').trim_end_matches('/');
            if pat.is_empty() {
                continue;
            }
            // gitignore-style: a pattern with no slash matches at any depth.
            if !pat.contains('/') {
                if let Ok(g) = Glob::new(&format!("**/{pat}")) {
                    builder.add(g);
                }
                if let Ok(g) = Glob::new(&format!("**/{pat}/**")) {
                    builder.add(g);
                }
                names.push(pat.to_string());
            }
            if let Ok(g) = Glob::new(pat) {
                builder.add(g);
            }
            if !pat.contains('*') {
                if let Ok(g) = Glob::new(&format!("{pat}/**")) {
                    builder.add(g);
                }
            }
        }
        let set = builder.build().context("invalid pattern in .npmignore")?;

        let mut files = Vec::new();
        for entry in Self::publishable_walk(root) {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy();
            if name == "package.json" || name == ".npmignore" {
                continue;
            }
            let rel = match entry.path().strip_prefix(root) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let basename_excluded = names.iter().any(|n| {
                rel.components()
                    .any(|c| c.as_os_str().to_string_lossy() == *n)
            });
            if set.is_match(rel) || basename_excluded {
                continue;
            }
            files.push(entry.path().to_path_buf());
        }
        Ok(files)
    }

    /// Mode 3: the historical default — walk the tree skipping the known
    /// non-publishable dirs.
    fn collect_default(root: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for entry in Self::publishable_walk(root) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let name = entry.file_name().to_string_lossy();
                if name == "package.json" {
                    continue; // Already added transformed.
                }
                files.push(entry.path().to_path_buf());
            }
        }
        Ok(files)
    }

    /// Walk the project tree, always pruning `node_modules` / `.git` (and the
    /// historical `patches` / `.jet-cache`). Shared by all three collect modes
    /// so those dirs can NEVER leak into a tarball.
    fn publishable_walk(
        root: &Path,
    ) -> impl Iterator<Item = walkdir::Result<walkdir::DirEntry>> {
        walkdir::WalkDir::new(root)
            .min_depth(1)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !matches!(
                    name.as_ref(),
                    "node_modules" | ".git" | "patches" | ".jet-cache"
                )
            })
    }

    /// npm always ships the root README* and LICENSE* regardless of the
    /// `files` allowlist / `.npmignore`. Append any that exist and are not
    /// already in `collected`.
    fn ensure_always_included(root: &Path, collected: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(root) else {
            return;
        };
        for entry in rd.flatten() {
            if !entry.path().is_file() {
                continue;
            }
            let name = entry.file_name();
            let lower = name.to_string_lossy().to_lowercase();
            let forced = lower.starts_with("readme")
                || lower.starts_with("license")
                || lower.starts_with("licence");
            if forced && !collected.iter().any(|p| p == &entry.path()) {
                collected.push(entry.path());
            }
        }
    }
}

/// @issue #172 — auto-fill absent `main`/`module`/`types` package.json fields
/// from a fresh library build's output. Only *missing* fields are filled — an
/// already-declared field is left untouched so an author's explicit choice
/// always wins. Paths are emitted as `./`-relative POSIX paths against the
/// project root (npm convention).
///
/// Mapping:
///   * `main`   ← the first CJS entry output (`./dist/index.cjs`)
///   * `module` ← the first ESM entry output (`./dist/index.js`)
///   * `types`  ← the first emitted `.d.ts`     (`./dist/index.d.ts`)
pub(crate) fn auto_fill_metadata(
    pkg: &mut serde_json::Value,
    result: &crate::bundler::LibBuildResult,
    root_dir: &Path,
) {
    use crate::bundler::types::OutputFormat;

    let Some(obj) = pkg.as_object_mut() else {
        return;
    };

    let rel = |path: &Path| -> Option<String> {
        path.strip_prefix(root_dir)
            .ok()
            .map(|p| format!("./{}", p.to_string_lossy().replace('\\', "/")))
    };

    if !obj.contains_key("main") {
        if let Some(cjs) = result
            .entries
            .iter()
            .find(|e| e.subpath == "." && e.format == OutputFormat::Cjs)
            .or_else(|| result.entries.iter().find(|e| e.format == OutputFormat::Cjs))
        {
            if let Some(p) = rel(&cjs.path) {
                obj.insert("main".to_string(), serde_json::Value::String(p));
            }
        }
    }

    if !obj.contains_key("module") {
        if let Some(esm) = result
            .entries
            .iter()
            .find(|e| e.subpath == "." && e.format == OutputFormat::Esm)
            .or_else(|| result.entries.iter().find(|e| e.format == OutputFormat::Esm))
        {
            if let Some(p) = rel(&esm.path) {
                obj.insert("module".to_string(), serde_json::Value::String(p));
            }
        }
    }

    if !obj.contains_key("types") {
        if let Some(dts) = result
            .types
            .iter()
            .find(|t| t.subpath == ".")
            .or_else(|| result.types.first())
        {
            if let Some(p) = rel(&dts.path) {
                obj.insert("types".to_string(), serde_json::Value::String(p));
            }
        }
    }
}

/// @issue #172 — validate that every package.json metadata target points at a
/// file that exists under the project root (and will therefore be in the
/// published tarball). Validates, in order: `main`, `module`, `types`, and
/// every concrete string leaf reachable from `exports` (conditional and
/// subpath maps are walked recursively; `*` wildcard targets are skipped —
/// they need a filesystem glob and are out of scope here).
///
/// On the first missing target a clear error is returned naming the field, the
/// declared value, and the absolute path that was checked, so a publishing dev
/// lands on the actual cause from one line.
pub(crate) fn validate_package_metadata(
    pkg: &serde_json::Value,
    root_dir: &Path,
) -> Result<()> {
    for field in ["main", "module", "types"] {
        if let Some(serde_json::Value::String(rel)) = pkg.get(field) {
            check_metadata_target(field, rel, root_dir)?;
        }
    }

    if let Some(exports) = pkg.get("exports") {
        validate_exports_targets(exports, root_dir)?;
    }

    Ok(())
}

/// Recursively validate every concrete string leaf of an `exports` value.
fn validate_exports_targets(value: &serde_json::Value, root_dir: &Path) -> Result<()> {
    match value {
        serde_json::Value::String(rel) => check_metadata_target("exports", rel, root_dir),
        serde_json::Value::Object(map) => {
            for (key, v) in map.iter() {
                // Wildcard subpath patterns need a glob — out of scope here.
                if key.contains('*') {
                    continue;
                }
                validate_exports_targets(v, root_dir)?;
            }
            Ok(())
        }
        serde_json::Value::Array(items) => {
            // `exports` may use an array of fallbacks; validate each.
            for item in items {
                validate_exports_targets(item, root_dir)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Check one `./`-relative metadata target resolves to an existing file.
fn check_metadata_target(field: &str, rel: &str, root_dir: &Path) -> Result<()> {
    // Wildcard targets (`./dist/*.js`) need a glob — skip them here.
    if rel.contains('*') {
        return Ok(());
    }
    let cleaned = rel.trim_start_matches("./");
    let target = root_dir.join(cleaned);
    if target.is_file() {
        Ok(())
    } else {
        anyhow::bail!("{}", format_missing_metadata_err(field, rel, &target));
    }
}

/// @issue #172 — build the error message for a metadata field whose declared
/// path does not exist on disk. Extracted so the wording (field + declared
/// value + checked absolute path + consequence) is unit-testable.
pub(crate) fn format_missing_metadata_err(
    field: &str,
    declared: &str,
    checked: &Path,
) -> String {
    format!(
        "#172 cannot publish: package.json `{field}` points at `{declared}`, but no file \
         exists at {}. The published tarball would ship a dangling `{field}` and every \
         consumer's import/require of this package would fail with MODULE_NOT_FOUND. \
         Build the library first (`jet build --lib` or `jet publish --build`) or fix the \
         `{field}` path.",
        checked.display()
    )
}

/// Simple base64 encoding for tarball attachment.
fn base64_encode(data: &[u8]) -> String {
    use std::fmt::Write;
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        write!(
            result,
            "{}",
            CHARS[((triple >> 18) & 0x3F) as usize] as char
        )
        .ok();
        write!(
            result,
            "{}",
            CHARS[((triple >> 12) & 0x3F) as usize] as char
        )
        .ok();
        if chunk.len() > 1 {
            write!(result, "{}", CHARS[((triple >> 6) & 0x3F) as usize] as char).ok();
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            write!(result, "{}", CHARS[(triple & 0x3F) as usize] as char).ok();
        } else {
            result.push('=');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode(b"hello"), "aGVsbG8=");
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"ab"), "YWI=");
    }

    #[test]
    fn gh3530_format_workspace_discover_warn_names_root_error_and_issue() {
        let root = PathBuf::from("/path/to/monorepo");
        let err = anyhow::anyhow!("invalid YAML at line 5");
        let msg = format_workspace_discover_warn(&root, &err);
        assert!(
            msg.contains("/path/to/monorepo"),
            "warning must name the project root verbatim: {msg}"
        );
        assert!(
            msg.contains("invalid YAML at line 5"),
            "warning must preserve the underlying error verbatim: {msg}"
        );
        assert!(
            msg.contains("GH #3530"),
            "warning must carry the GH #3530 tag so users can grep their logs: {msg}"
        );
    }

    #[test]
    fn gh3530_format_workspace_discover_warn_hints_at_symptoms() {
        let root = PathBuf::from("/monorepo");
        let err = anyhow::anyhow!("bad yaml");
        let msg = format_workspace_discover_warn(&root, &err);
        // The point of this warning is to land in front of a user grepping
        // their publish log for "workspace:" or "Invalid version".
        assert!(
            msg.contains("workspace:"),
            "warning must mention 'workspace:' so a user grepping for the registry error keyword lands here: {msg}"
        );
        assert!(
            msg.contains("registry") || msg.contains("downstream") || msg.contains("install"),
            "warning must mention the registry/downstream-install downstream symptom: {msg}"
        );
        // The fix is always one of these three files — name them so the
        // user can grep their own repo for the typo.
        assert!(
            msg.contains("jet-workspace.yaml")
                || msg.contains("pnpm-workspace.yaml")
                || msg.contains("package.json"),
            "warning must name the candidate config files so the user knows what to check: {msg}"
        );
    }

    /// GH #3530 — end-to-end: a malformed pnpm-workspace.yaml at the
    /// project root must NOT abort the publish path. The publisher must
    /// still produce a transformed package.json (with workspace:*
    /// unresolved — that is intentional graceful degrade so the user can
    /// see the upstream warn + the downstream registry error
    /// side-by-side).
    #[test]
    fn gh3530_malformed_workspace_yaml_does_not_abort_publish_transform() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Minimal publishable package.json with a workspace:* dep.
        std::fs::write(
            root.join("package.json"),
            r#"{
  "name": "my-pkg",
  "version": "1.0.0",
  "dependencies": { "internal-lib": "workspace:*" }
}"#,
        )
        .unwrap();

        // Malformed pnpm-workspace.yaml — fundamental YAML error.
        std::fs::write(
            root.join("pnpm-workspace.yaml"),
            "packages:\n  - 'packages/*\n: : :\n",
        )
        .unwrap();

        let publisher = Publisher::new(root.to_path_buf());
        let result = publisher.read_and_transform_package_json();
        assert!(
            result.is_ok(),
            "malformed pnpm-workspace.yaml must not abort the publish transform — the publish must continue so the user can see the warn line alongside the downstream registry error: {:?}",
            result.err()
        );
    }

    // ─── GH #3570: silent name/version fallback on publish ─────────────────

    /// GH #3570 — happy path: a well-formed package.json yields the
    /// declared name and version verbatim. This pins the contract that
    /// the helper does NOT alter the identity on the success branch.
    #[test]
    fn gh3570_require_publish_identity_returns_name_and_version() {
        let pkg = serde_json::json!({
            "name": "@scope/foo",
            "version": "1.2.3"
        });
        let p = std::path::Path::new("/proj/package.json");
        let (name, version) = require_publish_identity(&pkg, p).expect("happy path");
        assert_eq!(name, "@scope/foo");
        assert_eq!(version, "1.2.3");
    }

    /// GH #3570 — the publish-identity err must name the offending
    /// package.json path, the missing field, the observed JSON kind,
    /// and include the GH #3570 tag.
    #[test]
    fn gh3570_publish_identity_err_names_path_field_kind_and_tag() {
        let p = std::path::Path::new("/proj/pkg-broken/package.json");

        for field in ["name", "version"] {
            for kind in [
                "missing",
                "null",
                "bool",
                "number",
                "array",
                "object",
                "empty-string",
            ] {
                let msg = format_publish_identity_err(p, field, kind);
                assert!(
                    msg.contains("GH #3570"),
                    "must include issue tag (field={field}, kind={kind}), got: {msg}"
                );
                assert!(
                    msg.contains("/proj/pkg-broken/package.json"),
                    "must name path (field={field}, kind={kind}), got: {msg}"
                );
                assert!(
                    msg.contains(field),
                    "must name field (field={field}, kind={kind}), got: {msg}"
                );
                assert!(
                    msg.contains(kind),
                    "must name observed kind (field={field}, kind={kind}), got: {msg}"
                );
            }
        }
    }

    /// GH #3570 — branching contract: missing `name` → Err naming
    /// `name` + `missing`. Non-string `version` → Err naming `version`
    /// + observed kind. Empty-string `name` → Err naming `name` +
    /// `empty-string`. Pin these because each branch was a separate
    /// silent-fallback in the prior code.
    #[test]
    fn gh3570_require_publish_identity_distinguishes_each_failure_branch() {
        let p = std::path::Path::new("/proj/package.json");

        // 1. Missing name.
        let pkg = serde_json::json!({ "version": "1.0.0" });
        let err = require_publish_identity(&pkg, p).unwrap_err();
        let chain = format!("{err:#}");
        assert!(
            chain.contains("name") && chain.contains("missing"),
            "missing-name branch must name field + kind, got: {chain}"
        );

        // 2. Non-string version (e.g. user typed 1.2.3 without quotes).
        let pkg = serde_json::json!({ "name": "foo", "version": 1.2 });
        let err = require_publish_identity(&pkg, p).unwrap_err();
        let chain = format!("{err:#}");
        assert!(
            chain.contains("version") && chain.contains("number"),
            "non-string-version branch must name field + kind, got: {chain}"
        );

        // 3. Empty-string name. The unwrap_or("package") prior code would
        // have published an empty name to npm — must NOT silently fall
        // through to "package".
        let pkg = serde_json::json!({ "name": "", "version": "1.0.0" });
        let err = require_publish_identity(&pkg, p).unwrap_err();
        let chain = format!("{err:#}");
        assert!(
            chain.contains("name") && chain.contains("empty-string"),
            "empty-string-name branch must name field + kind, got: {chain}"
        );
        assert!(
            !chain.contains("\"package\""),
            "must NOT echo the prior silent fallback \"package\", got: {chain}"
        );
    }

    /// GH #3570 — `describe_publish_field_kind` must distinguish all
    /// JSON shapes so the identity error is precise about what was
    /// observed. Mirrors gh3566_describe_version_kind for the patch path.
    #[test]
    fn gh3570_describe_publish_field_kind_distinguishes_json_shapes() {
        assert_eq!(
            describe_publish_field_kind(&serde_json::Value::Null),
            "null"
        );
        assert_eq!(
            describe_publish_field_kind(&serde_json::Value::Bool(false)),
            "bool"
        );
        assert_eq!(
            describe_publish_field_kind(&serde_json::json!(42)),
            "number"
        );
        assert_eq!(
            describe_publish_field_kind(&serde_json::json!("foo")),
            "string"
        );
        assert_eq!(
            describe_publish_field_kind(&serde_json::json!([1])),
            "array"
        );
        assert_eq!(
            describe_publish_field_kind(&serde_json::json!({"a": 1})),
            "object"
        );
    }

    // ─── #172: build-before-publish metadata validation + auto-fill ────────

    /// #172 — a declared `main`/`module`/`types`/`exports` target that does
    /// not exist on disk must be a hard error naming the field, the declared
    /// value, and the checked absolute path.
    #[test]
    fn issue172_missing_metadata_target_is_an_error() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // `main` points at a dist file that was never built.
        let pkg = serde_json::json!({
            "name": "my-lib",
            "version": "1.0.0",
            "main": "./dist/index.cjs"
        });

        let err = validate_package_metadata(&pkg, root).unwrap_err();
        let chain = format!("{err:#}");
        assert!(chain.contains("#172"), "must carry the issue tag: {chain}");
        assert!(chain.contains("main"), "must name the field: {chain}");
        assert!(
            chain.contains("./dist/index.cjs"),
            "must echo the declared value: {chain}"
        );
        assert!(
            chain.contains("MODULE_NOT_FOUND") || chain.contains("dangling"),
            "must explain the downstream consequence: {chain}"
        );
    }

    /// #172 — when every declared target exists on disk, validation passes.
    #[test]
    fn issue172_present_metadata_targets_validate_ok() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("dist")).unwrap();
        for f in ["index.cjs", "index.js", "index.d.ts", "client.js"] {
            std::fs::write(root.join("dist").join(f), "// built\n").unwrap();
        }

        let pkg = serde_json::json!({
            "name": "my-lib",
            "version": "1.0.0",
            "main": "./dist/index.cjs",
            "module": "./dist/index.js",
            "types": "./dist/index.d.ts",
            "exports": {
                ".": { "import": "./dist/index.js", "require": "./dist/index.cjs" },
                "./client": "./dist/client.js"
            }
        });

        assert!(
            validate_package_metadata(&pkg, root).is_ok(),
            "all targets exist on disk → validation must pass"
        );
    }

    /// #172 — a missing `exports` leaf (nested under a condition map) is an
    /// error, proving the recursive walk reaches conditional/subpath leaves.
    #[test]
    fn issue172_missing_exports_leaf_is_an_error() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("dist")).unwrap();
        std::fs::write(root.join("dist/index.js"), "// built\n").unwrap();
        // `require` target is intentionally absent.

        let pkg = serde_json::json!({
            "name": "my-lib",
            "version": "1.0.0",
            "exports": {
                ".": { "import": "./dist/index.js", "require": "./dist/index.cjs" }
            }
        });

        let err = validate_package_metadata(&pkg, root).unwrap_err();
        let chain = format!("{err:#}");
        assert!(chain.contains("exports"), "must name the exports field: {chain}");
        assert!(
            chain.contains("./dist/index.cjs"),
            "must echo the missing leaf: {chain}"
        );
    }

    /// #172 — wildcard targets and absent fields are skipped (no false error).
    #[test]
    fn issue172_wildcard_and_absent_fields_are_skipped() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // No main/module/types; exports uses a `*` wildcard that we don't glob.
        let pkg = serde_json::json!({
            "name": "my-lib",
            "version": "1.0.0",
            "exports": { "./features/*": "./dist/features/*.js" }
        });

        assert!(
            validate_package_metadata(&pkg, root).is_ok(),
            "wildcard exports + absent fields must not error"
        );
    }

    /// #172 — `auto_fill_metadata` fills *only* absent `main`/`module`/`types`
    /// from the build output, mapping CJS→main, ESM→module, .d.ts→types, as
    /// `./`-relative paths. An already-declared field is left untouched.
    #[test]
    fn issue172_auto_fill_metadata_from_build_output() {
        use crate::bundler::lib_build::{EntryOutput, LibBuildResult, TypesOutput};
        use crate::bundler::types::OutputFormat;

        let dir = tempdir().unwrap();
        let root = dir.path();

        let result = LibBuildResult {
            entries: vec![
                EntryOutput {
                    subpath: ".".to_string(),
                    format: OutputFormat::Esm,
                    path: root.join("dist/index.js"),
                    code: String::new(),
                    dts: None,
                },
                EntryOutput {
                    subpath: ".".to_string(),
                    format: OutputFormat::Cjs,
                    path: root.join("dist/index.cjs"),
                    code: String::new(),
                    dts: None,
                },
            ],
            types: vec![TypesOutput {
                subpath: ".".to_string(),
                path: root.join("dist/index.d.ts"),
            }],
            assets: Vec::new(),
        };

        // `main` is pre-declared; module/types are absent.
        let mut pkg = serde_json::json!({
            "name": "my-lib",
            "version": "1.0.0",
            "main": "./preexisting.cjs"
        });

        auto_fill_metadata(&mut pkg, &result, root);

        assert_eq!(
            pkg.get("main").and_then(|v| v.as_str()),
            Some("./preexisting.cjs"),
            "declared `main` must be left untouched"
        );
        assert_eq!(
            pkg.get("module").and_then(|v| v.as_str()),
            Some("./dist/index.js"),
            "absent `module` must be auto-filled from the ESM output"
        );
        assert_eq!(
            pkg.get("types").and_then(|v| v.as_str()),
            Some("./dist/index.d.ts"),
            "absent `types` must be auto-filled from the .d.ts output"
        );
    }

    /// #172 — the missing-metadata error helper names field, declared value,
    /// the checked absolute path, and the issue tag.
    #[test]
    fn issue172_format_missing_metadata_err_shape() {
        let checked = std::path::Path::new("/proj/dist/index.cjs");
        let msg = format_missing_metadata_err("main", "./dist/index.cjs", checked);
        assert!(msg.contains("#172"), "msg: {msg}");
        assert!(msg.contains("main"), "msg: {msg}");
        assert!(msg.contains("./dist/index.cjs"), "msg: {msg}");
        assert!(msg.contains("/proj/dist/index.cjs"), "msg: {msg}");
    }

    // ─── files allowlist / .npmignore honoured by collect_publish_files ────

    /// Collect the publishable file set as a sorted Vec of `/`-joined paths
    /// relative to `root`, so assertions read like the tarball listing.
    fn collected_rel(root: &Path) -> Vec<String> {
        let mut rels: Vec<String> = Publisher::collect_publish_files(root)
            .unwrap()
            .into_iter()
            .map(|p| {
                p.strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();
        rels.sort();
        rels
    }

    /// Lay down the fe-shared `utils`-shaped fixture: a built `dist/`, the full
    /// `src/**` tree, and the config files npm would otherwise leak.
    fn write_source_lib(root: &Path) {
        std::fs::create_dir_all(root.join("dist")).unwrap();
        std::fs::write(root.join("dist/index.js"), "// esm\n").unwrap();
        std::fs::write(root.join("dist/index.cjs"), "// cjs\n").unwrap();
        std::fs::write(root.join("dist/index.d.ts"), "// types\n").unwrap();

        std::fs::create_dir_all(root.join("src/lib")).unwrap();
        std::fs::write(root.join("src/index.ts"), "export {}\n").unwrap();
        std::fs::write(root.join("src/lib/string.ts"), "export {}\n").unwrap();
        std::fs::write(root.join("src/lib/string.test.ts"), "test\n").unwrap();

        std::fs::write(root.join("README.md"), "# readme\n").unwrap();
        std::fs::write(root.join("tsconfig.json"), "{}\n").unwrap();
        std::fs::write(root.join("tsconfig.lib.json"), "{}\n").unwrap();
        std::fs::write(root.join("vite.config.ts"), "export default {}\n").unwrap();
        std::fs::write(root.join(".eslintrc.json"), "{}\n").unwrap();
        std::fs::write(root.join("jest.config.ts"), "export default {}\n").unwrap();
    }

    /// (a) `files: ["dist"]` → the packed set is ONLY dist/** + README
    /// (package.json is added separately by the caller). Source and config
    /// files must NOT leak — this is the fe-shared `utils` bug.
    #[test]
    fn files_allowlist_includes_only_listed_dir_plus_readme() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_source_lib(root);
        std::fs::write(
            root.join("package.json"),
            r#"{ "name": "@tw/utils", "version": "1.0.0", "files": ["dist"] }"#,
        )
        .unwrap();

        let rels = collected_rel(root);

        // dist/** is in.
        assert!(rels.contains(&"dist/index.js".to_string()), "{rels:?}");
        assert!(rels.contains(&"dist/index.cjs".to_string()), "{rels:?}");
        assert!(rels.contains(&"dist/index.d.ts".to_string()), "{rels:?}");
        // README is always forced in.
        assert!(rels.contains(&"README.md".to_string()), "{rels:?}");

        // Source and config files must NOT leak.
        for leak in [
            "src/index.ts",
            "src/lib/string.ts",
            "src/lib/string.test.ts",
            "tsconfig.json",
            "tsconfig.lib.json",
            "vite.config.ts",
            ".eslintrc.json",
            "jest.config.ts",
        ] {
            assert!(
                !rels.contains(&leak.to_string()),
                "`{leak}` must NOT be packed when files=[\"dist\"]: {rels:?}"
            );
        }

        // package.json is never returned here (transformed copy added by caller).
        assert!(
            !rels.contains(&"package.json".to_string()),
            "package.json must not be double-collected: {rels:?}"
        );
    }

    /// (b) no `files` but a `.npmignore` excluding `*.test.ts` → the test files
    /// are excluded, the rest of the tree is still packed.
    #[test]
    fn npmignore_excludes_matching_patterns() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_source_lib(root);
        std::fs::write(
            root.join("package.json"),
            r#"{ "name": "@tw/utils", "version": "1.0.0" }"#,
        )
        .unwrap();
        std::fs::write(root.join(".npmignore"), "*.test.ts\n").unwrap();

        let rels = collected_rel(root);

        // The .test.ts file is excluded.
        assert!(
            !rels.contains(&"src/lib/string.test.ts".to_string()),
            "*.test.ts must be excluded by .npmignore: {rels:?}"
        );
        // Non-ignored files remain.
        assert!(rels.contains(&"src/lib/string.ts".to_string()), "{rels:?}");
        assert!(rels.contains(&"dist/index.js".to_string()), "{rels:?}");
        // README still forced in; package.json/.npmignore never collected.
        assert!(rels.contains(&"README.md".to_string()), "{rels:?}");
        assert!(!rels.contains(&"package.json".to_string()), "{rels:?}");
        assert!(!rels.contains(&".npmignore".to_string()), "{rels:?}");
    }

    /// (c) always-include: README + LICENSE are packed regardless of a `files`
    /// allowlist that does not mention them, and node_modules never leaks.
    #[test]
    fn readme_license_always_included_node_modules_never() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("dist")).unwrap();
        std::fs::write(root.join("dist/index.js"), "// esm\n").unwrap();
        std::fs::write(root.join("README.md"), "# readme\n").unwrap();
        std::fs::write(root.join("LICENSE"), "MIT\n").unwrap();
        // node_modules must never be packed even though it is not in `files`.
        std::fs::create_dir_all(root.join("node_modules/dep")).unwrap();
        std::fs::write(root.join("node_modules/dep/index.js"), "leak\n").unwrap();
        std::fs::write(
            root.join("package.json"),
            r#"{ "name": "@tw/utils", "version": "1.0.0", "files": ["dist"] }"#,
        )
        .unwrap();

        let rels = collected_rel(root);

        assert!(rels.contains(&"README.md".to_string()), "{rels:?}");
        assert!(rels.contains(&"LICENSE".to_string()), "{rels:?}");
        assert!(rels.contains(&"dist/index.js".to_string()), "{rels:?}");
        assert!(
            !rels.iter().any(|r| r.starts_with("node_modules/")),
            "node_modules must never be packed: {rels:?}"
        );
    }

    /// `files` glob entries (`"dist/**"`, `"*.md"`) are honoured, not just
    /// bare directory names.
    #[test]
    fn files_allowlist_supports_glob_entries() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_source_lib(root);
        std::fs::write(
            root.join("package.json"),
            r#"{ "name": "@tw/utils", "version": "1.0.0", "files": ["dist/**", "*.md"] }"#,
        )
        .unwrap();

        let rels = collected_rel(root);
        assert!(rels.contains(&"dist/index.js".to_string()), "{rels:?}");
        assert!(rels.contains(&"README.md".to_string()), "{rels:?}");
        assert!(!rels.contains(&"src/index.ts".to_string()), "{rels:?}");
        assert!(!rels.contains(&"vite.config.ts".to_string()), "{rels:?}");
    }

    /// No `files`, no `.npmignore` → the historical default walk still works
    /// and still prunes node_modules.
    #[test]
    fn default_mode_walks_tree_minus_node_modules() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_source_lib(root);
        std::fs::create_dir_all(root.join("node_modules/dep")).unwrap();
        std::fs::write(root.join("node_modules/dep/index.js"), "leak\n").unwrap();
        std::fs::write(
            root.join("package.json"),
            r#"{ "name": "@tw/utils", "version": "1.0.0" }"#,
        )
        .unwrap();

        let rels = collected_rel(root);
        // Default mode keeps source files (legacy behaviour).
        assert!(rels.contains(&"src/index.ts".to_string()), "{rels:?}");
        assert!(rels.contains(&"dist/index.js".to_string()), "{rels:?}");
        // But still prunes node_modules and never double-collects package.json.
        assert!(
            !rels.iter().any(|r| r.starts_with("node_modules/")),
            "{rels:?}"
        );
        assert!(!rels.contains(&"package.json".to_string()), "{rels:?}");
    }
}
// CODEGEN-END
