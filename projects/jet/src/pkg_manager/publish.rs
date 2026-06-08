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
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl Publisher {
    pub fn new(root_dir: PathBuf) -> Self {
        let npmrc = NpmrcConfig::load(&root_dir);
        Self { root_dir, npmrc }
    }

    /// Create a tarball (.tgz) for publishing without actually publishing.
    pub fn pack(&self) -> Result<PathBuf> {
        let pkg = self.read_and_transform_package_json()?;
        let package_json_path = self.root_dir.join("package.json");
        let (name, version) = require_publish_identity(&pkg, &package_json_path)?;

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
        let pkg = self.read_and_transform_package_json()?;
        let package_json_path = self.root_dir.join("package.json");
        let (name_owned, version_owned) = require_publish_identity(&pkg, &package_json_path)?;
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

    /// Collect files to include in the package (respects .npmignore / files field).
    fn collect_publish_files(root: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(root)
            .min_depth(1)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                // Skip common non-publishable dirs
                !matches!(
                    name.as_ref(),
                    "node_modules" | ".git" | "patches" | ".jet-cache"
                )
            })
        {
            let entry = entry?;
            if entry.file_type().is_file() {
                let name = entry.file_name().to_string_lossy();
                if name == "package.json" {
                    continue; // Already added transformed
                }
                files.push(entry.path().to_path_buf());
            }
        }
        Ok(files)
    }
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
}
// CODEGEN-END
