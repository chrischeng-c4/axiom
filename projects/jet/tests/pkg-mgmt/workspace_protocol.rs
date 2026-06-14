// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests for the workspace protocol (pnpm-style) implementation.
//!
//! These tests exercise the end-to-end workspace-protocol install flow:
//! - pnpm-workspace.yaml discovery and catalog parsing
//! - workspace:* / workspace:^ / workspace:~ symlink creation
//! - Recursive workspace install across all packages in topological order
//! - Lockfile entries for workspace packages (workspace=true, localPath, version)
//!
//! All test fixtures use tempdir monorepos with **only** workspace-protocol
//! dependencies so that no network access is needed.

use jet::pkg_manager::lockfile::Lockfile;
use jet::pkg_manager::workspace::{WorkspaceManager, WorkspaceMode};
use jet::pkg_manager::PackageManager;
use std::collections::HashMap;
use std::path::Path;
use tempfile::tempdir;

// ------------------------------------------------------------------
// Fixture helpers
// ------------------------------------------------------------------

/// Write a file, creating parent directories as needed.
fn write_file(base: &Path, rel: &str, content: &str) {
    let full = base.join(rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(full, content).unwrap();
}

/// Build a `package.json` string.
fn pkg_json(name: &str, version: &str, deps: &[(&str, &str)]) -> String {
    let dep_entries: Vec<String> = deps
        .iter()
        .map(|(n, v)| format!("\"{}\":\"{}\"", n, v))
        .collect();
    let deps_obj = if dep_entries.is_empty() {
        "{}".to_string()
    } else {
        format!("{{{}}}", dep_entries.join(","))
    };
    format!(
        r#"{{"name":"{}","version":"{}","dependencies":{}}}"#,
        name, version, deps_obj
    )
}

// ------------------------------------------------------------------
// WorkspaceManager unit-level integration tests
// ------------------------------------------------------------------

#[test]
fn test_pnpm_workspace_yaml_discovery() {
    let dir = tempdir().unwrap();

    write_file(
        dir.path(),
        "pnpm-workspace.yaml",
        "packages:\n  - packages/*\n",
    );

    // Create a workspace package
    write_file(
        dir.path(),
        "packages/ui/package.json",
        r#"{"name":"ui","version":"1.5.0"}"#,
    );

    let wm = WorkspaceManager::discover(dir.path())
        .expect("discover should not error")
        .expect("should discover workspace from pnpm-workspace.yaml");

    assert_eq!(wm.packages.len(), 1, "one package expected");
    assert_eq!(wm.packages[0].name, "ui");
    assert_eq!(wm.packages[0].version, "1.5.0");
}

#[test]
fn test_jet_workspace_yaml_priority() {
    let dir = tempdir().unwrap();

    // jet-workspace.yaml lists apps/*, pnpm-workspace.yaml lists packages/*
    write_file(dir.path(), "jet-workspace.yaml", "packages:\n  - apps/*\n");
    write_file(
        dir.path(),
        "pnpm-workspace.yaml",
        "packages:\n  - packages/*\n",
    );

    write_file(
        dir.path(),
        "apps/web/package.json",
        r#"{"name":"web","version":"0.1.0"}"#,
    );
    write_file(
        dir.path(),
        "packages/lib/package.json",
        r#"{"name":"lib","version":"9.9.9"}"#,
    );

    let wm = WorkspaceManager::discover(dir.path())
        .expect("discover should succeed")
        .expect("workspace should be found");

    // Only app from jet-workspace.yaml should be present
    assert_eq!(
        wm.packages.len(),
        1,
        "jet-workspace.yaml should take priority over pnpm-workspace.yaml"
    );
    assert_eq!(wm.packages[0].name, "web");
}

#[test]
fn test_catalog_resolution() {
    let dir = tempdir().unwrap();

    let yaml = "packages:\n  - packages/*\ncatalog:\n  react: \"^18.0.0\"\ncatalogs:\n  legacy:\n    react: \"^16\"\n";
    write_file(dir.path(), "pnpm-workspace.yaml", yaml);

    let wm = WorkspaceManager::discover(dir.path())
        .expect("discover ok")
        .expect("workspace found");

    assert_eq!(
        wm.catalog_version("react"),
        Some("^18.0.0"),
        "default catalog entry should be accessible"
    );
    assert_eq!(
        wm.catalog_version("legacy:react"),
        Some("^16"),
        "named catalog entry should be prefixed with catalog_name:"
    );
    assert_eq!(
        wm.catalog_version("vue"),
        None,
        "unknown entry should return None"
    );
}

// ------------------------------------------------------------------
// WorkspaceMode detection
// ------------------------------------------------------------------

#[test]
fn test_workspace_mode_jet_detected_for_pnpm_yaml() {
    let dir = tempdir().unwrap();
    write_file(
        dir.path(),
        "pnpm-workspace.yaml",
        "packages:\n  - packages/*\n",
    );

    let mode = WorkspaceMode::detect(dir.path()).expect("detect ok");
    assert!(
        matches!(mode, WorkspaceMode::Jet(_)),
        "pnpm-workspace.yaml should produce WorkspaceMode::Jet"
    );
}

// ------------------------------------------------------------------
// Symlink creation integration tests (no network — workspace: deps only)
// ------------------------------------------------------------------

/// Build a minimal workspace fixture:
///
/// ```text
/// root/
///   pnpm-workspace.yaml  (packages: [packages/*])
///   packages/
///     ui/package.json    (name: ui, version: 1.5.0)
///     web/package.json   (name: web, version: 1.0.0, deps: {ui: workspace:*})
/// ```
fn make_two_package_workspace(root: &Path) {
    write_file(root, "pnpm-workspace.yaml", "packages:\n  - packages/*\n");
    write_file(
        root,
        "packages/ui/package.json",
        &pkg_json("ui", "1.5.0", &[]),
    );
    write_file(
        root,
        "packages/web/package.json",
        &pkg_json("web", "1.0.0", &[("ui", "workspace:*")]),
    );
}

#[tokio::test]
async fn test_workspace_star_symlink() {
    let dir = tempdir().unwrap();
    make_two_package_workspace(dir.path());

    let pm =
        PackageManager::new(dir.path().to_path_buf()).expect("PackageManager::new should succeed");

    pm.install_with_options(false)
        .await
        .expect("workspace install should succeed");

    // `packages/web/node_modules/ui` must be a symlink
    let symlink_path = dir.path().join("packages/web/node_modules/ui");
    assert!(
        symlink_path.is_symlink(),
        "node_modules/ui should be a relative symlink, got: {:?}",
        symlink_path.symlink_metadata()
    );

    // The symlink must resolve to packages/ui
    let resolved = symlink_path
        .canonicalize()
        .expect("symlink should be resolvable");
    let expected = dir
        .path()
        .join("packages/ui")
        .canonicalize()
        .expect("packages/ui should exist");
    assert_eq!(resolved, expected, "symlink should resolve to packages/ui");
}

#[tokio::test]
async fn test_workspace_caret_resolution() {
    let dir = tempdir().unwrap();

    write_file(
        dir.path(),
        "pnpm-workspace.yaml",
        "packages:\n  - packages/*\n",
    );
    // shared is at version 2.3.1
    write_file(
        dir.path(),
        "packages/shared/package.json",
        &pkg_json("@acme/shared", "2.3.1", &[]),
    );
    // server depends on shared via workspace:^
    write_file(
        dir.path(),
        "packages/server/package.json",
        &pkg_json("server", "1.0.0", &[("@acme/shared", "workspace:^")]),
    );

    let pm = PackageManager::new(dir.path().to_path_buf()).unwrap();
    pm.install_with_options(false).await.unwrap();

    // Symlink should exist
    let link = dir.path().join("packages/server/node_modules/@acme/shared");
    assert!(link.is_symlink(), "workspace:^ dep should create a symlink");

    // Lockfile should record version ^2.3.1
    let lf = Lockfile::read(&dir.path().join("jet-lock.yaml")).expect("jet-lock.yaml should exist");

    let ws_entry = lf
        .packages
        .iter()
        .find(|(k, e)| k.contains("@acme/shared") && e.workspace)
        .map(|(_, e)| e)
        .expect("lockfile should have a workspace entry for @acme/shared");

    assert_eq!(
        ws_entry.version, "^2.3.1",
        "workspace:^ should resolve to ^<actual_version>"
    );
    assert!(ws_entry.workspace, "entry should have workspace: true");
}

#[tokio::test]
async fn test_recursive_workspace_install() {
    let dir = tempdir().unwrap();

    write_file(
        dir.path(),
        "pnpm-workspace.yaml",
        "packages:\n  - packages/*\n  - apps/*\n",
    );

    // utils: no deps
    write_file(
        dir.path(),
        "packages/utils/package.json",
        &pkg_json("utils", "1.0.0", &[]),
    );
    // ui: depends on utils
    write_file(
        dir.path(),
        "packages/ui/package.json",
        &pkg_json("ui", "2.0.0", &[("utils", "workspace:*")]),
    );
    // web: depends on both ui and utils
    write_file(
        dir.path(),
        "apps/web/package.json",
        &pkg_json(
            "web",
            "0.1.0",
            &[("ui", "workspace:*"), ("utils", "workspace:*")],
        ),
    );

    let pm = PackageManager::new(dir.path().to_path_buf()).unwrap();
    pm.install_with_options(false).await.unwrap();

    // ui/node_modules/utils symlink
    assert!(
        dir.path()
            .join("packages/ui/node_modules/utils")
            .is_symlink(),
        "ui → utils symlink should exist"
    );

    // web/node_modules/ui symlink
    assert!(
        dir.path().join("apps/web/node_modules/ui").is_symlink(),
        "web → ui symlink should exist"
    );

    // web/node_modules/utils symlink
    assert!(
        dir.path().join("apps/web/node_modules/utils").is_symlink(),
        "web → utils symlink should exist"
    );
}

#[tokio::test]
async fn test_no_registry_call_for_workspace_dep() {
    // All dependencies are workspace: — no tarball should be downloaded.
    // We verify this indirectly: node_modules entries are symlinks (not real dirs).
    let dir = tempdir().unwrap();
    make_two_package_workspace(dir.path());

    let pm = PackageManager::new(dir.path().to_path_buf()).unwrap();
    pm.install_with_options(false).await.unwrap();

    let link = dir.path().join("packages/web/node_modules/ui");

    // Must be a symlink, not a real extracted directory
    assert!(
        link.is_symlink(),
        "workspace dep should be a symlink, not an extracted tarball"
    );

    // Real extracted packages contain a package.json; symlinks point to the
    // source dir (which also has package.json, but the symlink itself is not
    // a regular directory at this path).
    let meta = std::fs::symlink_metadata(&link).unwrap();
    assert!(
        meta.file_type().is_symlink(),
        "file type must be symlink, not regular dir"
    );
}

#[tokio::test]
async fn test_lockfile_workspace_fields() {
    let dir = tempdir().unwrap();
    make_two_package_workspace(dir.path());

    let pm = PackageManager::new(dir.path().to_path_buf()).unwrap();
    pm.install_with_options(false).await.unwrap();

    let lf_path = dir.path().join("jet-lock.yaml");
    assert!(lf_path.exists(), "jet-lock.yaml should be written");

    let lf = Lockfile::read(&lf_path).expect("should parse jet-lock.yaml");

    // Find the workspace entry for ui@1.5.0
    let ui_entry = lf
        .packages
        .iter()
        .find(|(k, e)| k.contains("ui") && e.workspace)
        .map(|(_, e)| e)
        .expect("lockfile should contain a workspace entry for ui");

    assert!(ui_entry.workspace, "workspace field must be true");
    assert_eq!(
        ui_entry.version, "1.5.0",
        "version should match workspace package version"
    );
    assert_eq!(
        ui_entry.local_path.as_deref(),
        Some("packages/ui"),
        "localPath should be relative path from workspace root to package dir"
    );
}

#[tokio::test]
async fn test_idempotent_symlink_creation() {
    // Running install twice should not fail — symlinks are idempotent.
    let dir = tempdir().unwrap();
    make_two_package_workspace(dir.path());

    let pm = PackageManager::new(dir.path().to_path_buf()).unwrap();
    pm.install_with_options(false)
        .await
        .expect("first install should succeed");
    pm.install_with_options(false)
        .await
        .expect("second install should be idempotent");

    assert!(dir.path().join("packages/web/node_modules/ui").is_symlink());
}

#[test]
fn test_workspace_protocol_resolution_variants() {
    // Verify resolve_workspace_protocol handles *, ^, ~ correctly
    let ws = WorkspaceManager {
        root: std::path::PathBuf::from("/tmp"),
        config: jet::pkg_manager::workspace::WorkspaceConfig::default(),
        packages: vec![jet::pkg_manager::workspace::WorkspacePackage {
            name: "pkg-a".to_string(),
            version: "2.3.1".to_string(),
            path: std::path::PathBuf::from("packages/pkg-a"),
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            deps_on_workspace: Vec::new(),
        }],
    };

    assert_eq!(
        ws.resolve_workspace_protocol("workspace:*", "pkg-a"),
        Some("2.3.1".to_string()),
        "workspace:* → exact version"
    );
    assert_eq!(
        ws.resolve_workspace_protocol("workspace:^", "pkg-a"),
        Some("^2.3.1".to_string()),
        "workspace:^ → caret range"
    );
    assert_eq!(
        ws.resolve_workspace_protocol("workspace:~", "pkg-a"),
        Some("~2.3.1".to_string()),
        "workspace:~ → tilde range"
    );
    assert_eq!(
        ws.resolve_workspace_protocol("workspace:*", "nonexistent"),
        None,
        "unknown package → None"
    );
}
// CODEGEN-END
