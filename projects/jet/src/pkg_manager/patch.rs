// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
// CODEGEN-BEGIN
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Patch manager for creating and applying patches to installed packages.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub struct PatchManager {
    root_dir: PathBuf,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl PatchManager {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    /// Copy an installed package to `patches/{name}@{version}/` for editing.
    pub fn prepare_patch(&self, package: &str) -> Result<PathBuf> {
        let node_modules = self.root_dir.join("node_modules");
        let pkg_dir = node_modules.join(package);

        if !pkg_dir.exists() {
            anyhow::bail!(
                "Package '{}' not found in node_modules. Run 'jet install' first.",
                package
            );
        }

        // Read version from package.json
        let version = Self::read_package_version(&pkg_dir)?;
        let patch_dir = self
            .root_dir
            .join("patches")
            .join(format!("{}@{}", package, version));

        if patch_dir.exists() {
            std::fs::remove_dir_all(&patch_dir).with_context(|| {
                format!(
                    "GH #3566 failed to clear existing patch dir {}; the patch \
                     workspace could not be reset before re-copying",
                    patch_dir.display()
                )
            })?;
        }

        copy_dir_recursive(&pkg_dir, &patch_dir)
            .with_context(|| format!("Failed to copy {} to patches/", package))?;

        tracing::info!(
            "Prepared patch for {}@{} at {:?}",
            package,
            version,
            patch_dir
        );
        Ok(patch_dir)
    }

    /// Generate a .patch file by diffing the original package with the edited copy.
    pub fn commit_patch(&self, package: &str) -> Result<PathBuf> {
        let node_modules = self.root_dir.join("node_modules");
        let original = node_modules.join(package);
        let version = Self::read_package_version(&original)?;
        let edited = self
            .root_dir
            .join("patches")
            .join(format!("{}@{}", package, version));

        if !edited.exists() {
            anyhow::bail!(
                "No patch directory found. Run 'jet patch {}' first.",
                package
            );
        }

        // Generate diff using system diff command
        let patch_file = self
            .root_dir
            .join("patches")
            .join(format!("{}@{}.patch", package, version));

        let output = std::process::Command::new("diff")
            .args(["-ruN", "--strip-trailing-cr"])
            .arg(&original)
            .arg(&edited)
            .output()
            .context("Failed to run diff command")?;

        // diff returns exit code 1 when files differ (that's expected)
        let diff_content = String::from_utf8_lossy(&output.stdout);
        if diff_content.is_empty() {
            anyhow::bail!("No changes detected in patch for {}", package);
        }

        std::fs::write(&patch_file, diff_content.as_bytes()).with_context(|| {
            format!(
                "GH #3566 failed to write patch file {}; the diff was \
                 computed but could not be persisted",
                patch_file.display()
            )
        })?;

        // Clean up the edited directory
        std::fs::remove_dir_all(&edited).with_context(|| {
            format!(
                "GH #3566 failed to remove edited patch dir {} after writing \
                 the patch file; manual cleanup may be required",
                edited.display()
            )
        })?;

        tracing::info!("Patch file created: {:?}", patch_file);
        Ok(patch_file)
    }

    fn read_package_version(pkg_dir: &Path) -> Result<String> {
        let pkg_json_path = pkg_dir.join("package.json");
        let content = std::fs::read_to_string(&pkg_json_path).with_context(|| {
            format!(
                "GH #3566 failed to read package version from {}",
                pkg_json_path.display()
            )
        })?;
        let pkg: serde_json::Value = serde_json::from_str(&content).with_context(|| {
            format!(
                "GH #3566 failed to parse package metadata from {}",
                pkg_json_path.display()
            )
        })?;
        let version = pkg
            .get("version")
            .and_then(|value| value.as_str())
            .ok_or_else(|| {
                let kind = pkg
                    .get("version")
                    .map(describe_version_kind)
                    .unwrap_or("missing");
                anyhow::anyhow!("{}", format_patch_missing_version_err(&pkg_json_path, kind))
            })?;
        Ok(version.to_string())
    }
}

fn describe_version_kind(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

fn format_patch_missing_version_err(path: &Path, kind: &str) -> String {
    format!(
        "GH #3566 package.json {} has invalid version field kind {kind}; expected string and refusing silent default-version fallback",
        path.display()
    )
}

/// Recursively copy a directory.
fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in walkdir::WalkDir::new(src).min_depth(1) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(src)?;
        let dest_path = dest.join(relative);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_prepare_patch_missing_package() {
        let dir = tempdir().unwrap();
        let pm = PatchManager::new(dir.path().to_path_buf());
        let result = pm.prepare_patch("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_commit_patch_no_edit() {
        let dir = tempdir().unwrap();
        let pm = PatchManager::new(dir.path().to_path_buf());
        let result = pm.commit_patch("nonexistent");
        assert!(result.is_err());
    }

    // ─── GH #3566: silent version-fallback and missing-path context ────────

    /// GH #3566 — the missing-version err must name the offending
    /// package.json path, the observed JSON kind (so non-string is
    /// distinguished from missing), and the issue tag.
    #[test]
    fn gh3566_patch_missing_version_err_names_path_kind_and_tag() {
        let p = std::path::Path::new("/proj/node_modules/foo/package.json");

        for kind in ["missing", "number", "null", "object", "array", "bool"] {
            let msg = format_patch_missing_version_err(p, kind);
            assert!(
                msg.contains("GH #3566"),
                "must include issue tag (kind={kind}), got: {msg}"
            );
            assert!(
                msg.contains("/proj/node_modules/foo/package.json"),
                "must name the offending path (kind={kind}), got: {msg}"
            );
            assert!(
                msg.contains(kind),
                "must name the observed JSON kind (kind={kind}), got: {msg}"
            );
        }
    }

    /// GH #3566 — `describe_version_kind` must distinguish all JSON
    /// shapes so the version error is precise about what was observed.
    #[test]
    fn gh3566_describe_version_kind_distinguishes_json_shapes() {
        assert_eq!(describe_version_kind(&serde_json::Value::Null), "null");
        assert_eq!(
            describe_version_kind(&serde_json::Value::Bool(true)),
            "bool"
        );
        assert_eq!(describe_version_kind(&serde_json::json!(42)), "number");
        assert_eq!(describe_version_kind(&serde_json::json!("1.2.3")), "string");
        assert_eq!(
            describe_version_kind(&serde_json::json!([1, 2, 3])),
            "array"
        );
        assert_eq!(
            describe_version_kind(&serde_json::json!({"x": 1})),
            "object"
        );
    }

    /// GH #3566 — end-to-end: `read_package_version` on a package.json
    /// whose `version` is a number (e.g. user typed `1.2.3` without
    /// quotes) must produce an error whose Display contains the
    /// GH #3566 tag, the package.json path, and the observed kind
    /// `number` — NOT a silent "0.0.0" patch filename.
    #[test]
    fn gh3566_read_package_version_surfaces_path_and_kind_on_non_string_version() {
        let dir = tempdir().unwrap();
        let pkg_dir = dir.path().to_path_buf();
        std::fs::write(
            pkg_dir.join("package.json"),
            r#"{"name": "foo", "version": 1.2}"#,
        )
        .unwrap();

        let err = PatchManager::read_package_version(&pkg_dir)
            .expect_err("non-string version must error");
        let chain = format!("{err:#}");
        assert!(
            chain.contains("GH #3566"),
            "chained error must include issue tag, got: {chain}"
        );
        assert!(
            chain.contains("package.json"),
            "chained error must name the offending file, got: {chain}"
        );
        assert!(
            chain.contains("number"),
            "chained error must name the observed JSON kind, got: {chain}"
        );
        assert!(
            !chain.contains("0.0.0"),
            "must NOT silently fall back to 0.0.0, got: {chain}"
        );
    }

    /// GH #3566 — end-to-end: `read_package_version` on a missing
    /// package.json must surface a chained error whose Display
    /// contains the GH #3566 tag and names the offending path. This
    /// is the contract that the `with_context` on the `std::fs::read_to_string`
    /// site enforces.
    #[test]
    fn gh3566_read_package_version_surfaces_path_on_missing_file() {
        let dir = tempdir().unwrap();
        // No package.json written.
        let err = PatchManager::read_package_version(&dir.path().to_path_buf())
            .expect_err("missing file must error");
        let chain = format!("{err:#}");
        assert!(
            chain.contains("GH #3566"),
            "chained error must include issue tag, got: {chain}"
        );
        assert!(
            chain.contains("package.json"),
            "chained error must name the offending file, got: {chain}"
        );
    }
}
// CODEGEN-END
