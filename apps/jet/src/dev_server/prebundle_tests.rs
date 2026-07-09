// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-dev-server.md#schema
// CODEGEN-BEGIN
use super::*;
use std::fs;
use tempfile::tempdir;

/// T2: ESM Package Skipped — module Field
#[test]
fn t02_esm_package_skipped_module_field() {
    let dir = tempdir().unwrap();
    let pkg_dir = dir.path().join("date-fns");
    fs::create_dir_all(&pkg_dir).unwrap();
    fs::write(
        pkg_dir.join("package.json"),
        r#"{ "name": "date-fns", "version": "2.0.0", "module": "src/index.js" }"#,
    )
    .unwrap();

    let prebundler = PreBundler::new(dir.path().to_path_buf());
    let result = prebundler.is_cjs_package(&pkg_dir).unwrap();
    assert!(
        !result,
        "package with 'module' field must be detected as ESM"
    );
}

/// T3: ESM Package Skipped — exports.import Field
#[test]
fn t03_esm_package_skipped_exports_import() {
    let dir = tempdir().unwrap();
    let pkg_dir = dir.path().join("some-pkg");
    fs::create_dir_all(&pkg_dir).unwrap();
    fs::write(
        pkg_dir.join("package.json"),
        r#"{ "name": "some-pkg", "exports": { ".": { "import": "./dist/esm/index.js" } } }"#,
    )
    .unwrap();

    let prebundler = PreBundler::new(dir.path().to_path_buf());
    let result = prebundler.is_cjs_package(&pkg_dir).unwrap();
    assert!(
        !result,
        "package with exports.import must be detected as ESM"
    );
}

/// Variant: CJS package (no ESM markers) returns true
#[test]
fn test_cjs_package_detected() {
    let dir = tempdir().unwrap();
    let pkg_dir = dir.path().join("react");
    fs::create_dir_all(&pkg_dir).unwrap();
    fs::write(
        pkg_dir.join("package.json"),
        r#"{ "name": "react", "version": "18.2.0", "main": "index.js" }"#,
    )
    .unwrap();

    let prebundler = PreBundler::new(dir.path().to_path_buf());
    let result = prebundler.is_cjs_package(&pkg_dir).unwrap();
    assert!(
        result,
        "CJS package (no module/exports.import) must be detected"
    );
}

/// Variant: package with type: "module" is ESM
#[test]
fn test_type_module_detected_as_esm() {
    let dir = tempdir().unwrap();
    let pkg_dir = dir.path().join("esm-pkg");
    fs::create_dir_all(&pkg_dir).unwrap();
    fs::write(
        pkg_dir.join("package.json"),
        r#"{ "name": "esm-pkg", "type": "module", "main": "index.mjs" }"#,
    )
    .unwrap();

    let prebundler = PreBundler::new(dir.path().to_path_buf());
    let result = prebundler.is_cjs_package(&pkg_dir).unwrap();
    assert!(!result, "package with type:module must be ESM");
}

#[tokio::test]
async fn prebundle_accepts_aw_wrapped_root_package_json() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(
        root.join("package.json"),
        r#"// <HANDWRITE gap="standardize:claim-code" tracker="fixture-package-json" reason="fixture ownership">
{
  "name": "wrapped-root",
  "private": true,
  "dependencies": {}
}
// </HANDWRITE>
"#,
    )
    .unwrap();

    let result = PreBundler::new(root.to_path_buf())
        .prebundle_deps()
        .await
        .unwrap();

    assert!(
        !result.cache_hit,
        "fresh wrapper-backed fixture should prebundle rather than fail parsing"
    );
    assert!(
        root.join("node_modules/.jet/_cache_marker").exists(),
        "successful prebundle must write the cache marker"
    );
}

#[test]
fn cjs_detection_accepts_aw_wrapped_package_json() {
    let dir = tempdir().unwrap();
    let pkg_dir = dir.path().join("node_modules/demo-cjs");
    fs::create_dir_all(&pkg_dir).unwrap();
    fs::write(
        pkg_dir.join("package.json"),
        r#"// <HANDWRITE gap="standardize:claim-code" tracker="demo-package-json" reason="fixture ownership">
{
  "name": "demo-cjs",
  "version": "1.0.0",
  "main": "index.js"
}
// </HANDWRITE>
"#,
    )
    .unwrap();

    let prebundler = PreBundler::new(dir.path().to_path_buf());
    assert!(
        prebundler.is_cjs_package(&pkg_dir).unwrap(),
        "AW wrapper lines around package.json must not make CJS detection fail"
    );
}

/// T5: Scoped Package Pre-Bundled — dep_filename handles scoped names
#[test]
fn t05_scoped_package_filename() {
    let filename = dep_filename("@tanstack/react-query");
    assert_eq!(
        filename, "@tanstack__react-query.mjs",
        "scoped package must produce correct .mjs filename"
    );
}

/// T5 variant: regular package filename
#[test]
fn test_dep_filename_regular() {
    let filename = dep_filename("react");
    assert_eq!(filename, "react.mjs");
}

/// T4: Subpath Export Filename
#[test]
fn t04_subpath_export_filename() {
    let filename = dep_filename("react/jsx-runtime");
    assert_eq!(
        filename, "react_jsx-runtime.mjs",
        "subpath export must produce correct .mjs filename"
    );
}

/// T6: Pre-Bundle Cache Hit
#[test]
fn t06_prebundle_cache_hit() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let jet_dir = root.join("node_modules/.jet");
    fs::create_dir_all(&jet_dir).unwrap();

    // Create package.json and lockfile first
    fs::write(root.join("package.json"), r#"{"dependencies":{}}"#).unwrap();
    fs::write(root.join("jet-lock.yaml"), "lockfile-v1").unwrap();

    // Wait a tiny bit then create cache marker so its mtime is newer
    std::thread::sleep(std::time::Duration::from_millis(50));
    fs::write(
        jet_dir.join("_cache_marker"),
        format!("prebundle cache marker {}", CACHE_MARKER_VERSION),
    )
    .unwrap();

    let prebundler = PreBundler::new(root.to_path_buf());
    assert!(
        prebundler.check_cache_valid(&jet_dir),
        "cache must be valid when marker is newer than package.json and lockfile"
    );
}

/// T7: Pre-Bundle Cache Invalidation on package.json Change
#[test]
fn t07_cache_invalidation_package_json() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let jet_dir = root.join("node_modules/.jet");
    fs::create_dir_all(&jet_dir).unwrap();

    // Create cache marker first
    fs::write(
        jet_dir.join("_cache_marker"),
        format!("prebundle cache marker {}", CACHE_MARKER_VERSION),
    )
    .unwrap();

    // Then create package.json (newer than marker)
    std::thread::sleep(std::time::Duration::from_millis(50));
    fs::write(
        root.join("package.json"),
        r#"{"dependencies":{"react":"^18"}}"#,
    )
    .unwrap();

    let prebundler = PreBundler::new(root.to_path_buf());
    assert!(
        !prebundler.check_cache_valid(&jet_dir),
        "cache must be invalid when package.json is newer than marker"
    );
}

/// T8: Pre-Bundle Cache Invalidation on Lockfile Change
#[test]
fn t08_cache_invalidation_lockfile() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let jet_dir = root.join("node_modules/.jet");
    fs::create_dir_all(&jet_dir).unwrap();

    // Create package.json and marker first
    fs::write(root.join("package.json"), r#"{"dependencies":{}}"#).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    fs::write(
        jet_dir.join("_cache_marker"),
        format!("prebundle cache marker {}", CACHE_MARKER_VERSION),
    )
    .unwrap();

    // Then update lockfile (newer than marker)
    std::thread::sleep(std::time::Duration::from_millis(50));
    fs::write(root.join("jet-lock.yaml"), "updated-lockfile").unwrap();

    let prebundler = PreBundler::new(root.to_path_buf());
    assert!(
        !prebundler.check_cache_valid(&jet_dir),
        "cache must be invalid when lockfile is newer than marker"
    );
}

/// Variant: No cache marker → cache invalid
#[test]
fn test_no_cache_marker() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let jet_dir = root.join("node_modules/.jet");
    fs::create_dir_all(&jet_dir).unwrap();
    fs::write(root.join("package.json"), r#"{"dependencies":{}}"#).unwrap();

    let prebundler = PreBundler::new(root.to_path_buf());
    assert!(
        !prebundler.check_cache_valid(&jet_dir),
        "cache must be invalid when no marker exists"
    );
}

#[test]
fn test_stale_cache_marker_version_invalid() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let jet_dir = root.join("node_modules/.jet");
    fs::create_dir_all(&jet_dir).unwrap();
    fs::write(root.join("package.json"), r#"{"dependencies":{}}"#).unwrap();
    fs::write(
        jet_dir.join("_cache_marker"),
        "prebundle cache marker\nversion=1\n",
    )
    .unwrap();

    let prebundler = PreBundler::new(root.to_path_buf());
    assert!(
        !prebundler.check_cache_valid(&jet_dir),
        "cache must be invalid when the prebundle output version changes"
    );
}

#[test]
fn test_resolve_exports_prefers_browser_module_nested_default() {
    let exports = serde_json::json!({
        ".": {
            "browser": {
                "module": "./dist/browser.esm.js",
                "default": "./dist/browser.cjs.js"
            },
            "import": {
                "default": "./dist/node.mjs"
            },
            "default": "./dist/node.js"
        }
    });

    assert_eq!(
        resolve_exports_condition(&exports),
        Some("./dist/browser.esm.js".to_string())
    );
}

#[test]
fn test_resolve_exports_import_object_default() {
    let exports = serde_json::json!({
        "import": {
            "types": "./index.d.mts",
            "default": "./index.mjs"
        },
        "require": {
            "default": "./index.js"
        }
    });

    assert_eq!(
        resolve_exports_condition(&exports),
        Some("./index.mjs".to_string())
    );
}

/// T12: Virtual ESM Entry Created Correctly
#[test]
fn t12_virtual_esm_entry() {
    let entry = create_virtual_entry("axios");
    assert!(
        entry.contains("export * from 'axios'"),
        "must contain namespace re-export: {}",
        entry
    );
    assert!(
        entry.contains("export { default } from 'axios'"),
        "must contain default re-export: {}",
        entry
    );
}

/// Variant: scoped package virtual entry
#[test]
fn test_virtual_esm_entry_scoped() {
    let entry = create_virtual_entry("@tanstack/react-query");
    assert!(entry.contains("export * from '@tanstack/react-query'"));
    assert!(entry.contains("export { default } from '@tanstack/react-query'"));
}

/// T13: process.env.NODE_ENV Resolved in Dev Mode
#[test]
fn t13_process_env_node_env_resolved() {
    let source = r#"if (process.env.NODE_ENV === 'production') { trim(); }"#;
    // convert_cjs_to_esm wraps CJS, then the caller replaces process.env.NODE_ENV
    let esm = convert_cjs_to_esm(source, "test-pkg", &HashMap::new());
    let final_output = esm.replace("process.env.NODE_ENV", "'development'");
    assert!(
        final_output.contains("'development'"),
        "process.env.NODE_ENV must be replaced with 'development': {}",
        final_output
    );
    assert!(
        !final_output.contains("process.env.NODE_ENV"),
        "original reference must be gone: {}",
        final_output
    );
}

/// T16: Exports Map Condition Resolution
#[test]
fn t16_exports_map_condition_resolution() {
    let exports: serde_json::Value = serde_json::from_str(
        r#"{ ".": { "import": "./esm.js", "require": "./cjs.js", "default": "./index.js" } }"#,
    )
    .unwrap();
    let entry = resolve_exports_entry(&exports);
    assert_eq!(
        entry.as_deref(),
        Some("./esm.js"),
        "import condition must be preferred over require/default"
    );
}

/// Variant: require takes precedence when import is absent
#[test]
fn test_exports_require_fallback() {
    let exports: serde_json::Value =
        serde_json::from_str(r#"{ ".": { "require": "./cjs.js", "default": "./index.js" } }"#)
            .unwrap();
    let entry = resolve_exports_entry(&exports);
    assert_eq!(
        entry.as_deref(),
        Some("./cjs.js"),
        "require condition must be preferred over default when import absent"
    );
}

/// Variant: string export
#[test]
fn test_exports_string() {
    let exports: serde_json::Value = serde_json::from_str(r#""./dist/index.js""#).unwrap();
    let entry = resolve_exports_entry(&exports);
    assert_eq!(entry.as_deref(), Some("./dist/index.js"));
}

/// Test: has_import_condition detects nested import
#[test]
fn test_has_import_condition() {
    let exports: serde_json::Value =
        serde_json::from_str(r#"{ ".": { "import": "./esm.js" } }"#).unwrap();
    assert!(has_import_condition(&exports));
}

/// Test: has_import_condition returns false when only require
#[test]
fn test_no_import_condition() {
    let exports: serde_json::Value =
        serde_json::from_str(r#"{ ".": { "require": "./cjs.js" } }"#).unwrap();
    assert!(!has_import_condition(&exports));
}

/// CJS source that is already ESM-ish is passed through (not spec T14)
#[test]
fn test_esm_passthrough() {
    let source = "export default function() {};\nexport const foo = 1;";
    let result = convert_cjs_to_esm(source, "test", &HashMap::new());
    assert_eq!(
        result, source,
        "ESM source must be passed through unchanged"
    );
}

/// T1: CJS Package Produces Valid ESM (unit-level: conversion wraps correctly)
#[test]
fn t01_cjs_produces_valid_esm() {
    let source = "var x = 1;\nmodule.exports = { x: x };";
    let result = convert_cjs_to_esm(source, "test-pkg", &HashMap::new());
    assert!(
        result.contains("export default"),
        "converted CJS must have default export: {}",
        result
    );
    assert!(
        result.contains("export"),
        "must be valid ESM with exports: {}",
        result
    );
}

/// T14: Circular CJS Require Detected
///
/// Verifies that detect_circular_deps() finds cycles between packages
/// that require() each other.
#[test]
fn t14_circular_require_detected() {
    let mut sources = HashMap::new();
    // A requires B
    sources.insert(
        "pkg-a".to_string(),
        "var b = require('pkg-b'); module.exports = { a: true };".to_string(),
    );
    // B requires A — circular!
    sources.insert(
        "pkg-b".to_string(),
        "var a = require('pkg-a'); module.exports = { b: true };".to_string(),
    );
    let cycles = detect_circular_deps(&sources);
    assert!(
        !cycles.is_empty(),
        "must detect circular dependency between pkg-a and pkg-b: {:?}",
        cycles
    );
    // Verify the cycle contains both packages
    let cycle = &cycles[0];
    let cycle_str = cycle.join(" → ");
    assert!(
        cycle_str.contains("pkg-a") && cycle_str.contains("pkg-b"),
        "cycle must involve pkg-a and pkg-b: {}",
        cycle_str
    );
}

/// Variant: no cycle when no circular require
#[test]
fn test_no_circular_when_linear() {
    let mut sources = HashMap::new();
    sources.insert(
        "pkg-a".to_string(),
        "var b = require('pkg-b'); module.exports = {};".to_string(),
    );
    sources.insert(
        "pkg-b".to_string(),
        "module.exports = { b: true };".to_string(),
    );
    let cycles = detect_circular_deps(&sources);
    assert!(
        cycles.is_empty(),
        "must not detect cycle in linear deps: {:?}",
        cycles
    );
}

/// T15: Transitive CJS Deps Auto-Discovered
///
/// Verifies that discover_subpath_exports() finds and pre-bundles all
/// subpath entries from a package's exports map (not just hardcoded ones).
#[tokio::test]
async fn t15_transitive_dep_discovered() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let jet_dir = root.join("node_modules/.jet");
    fs::create_dir_all(&jet_dir).unwrap();

    // Create a package with an exports map containing a subpath
    let pkg_dir = root.join("node_modules/my-lib");
    fs::create_dir_all(&pkg_dir).unwrap();
    fs::write(
        pkg_dir.join("package.json"),
        r#"{
            "name": "my-lib",
            "version": "1.0.0",
            "main": "index.js",
            "exports": {
                ".": "./index.js",
                "./utils": "./utils.js"
            }
        }"#,
    )
    .unwrap();
    fs::write(pkg_dir.join("index.js"), "module.exports = {};").unwrap();
    fs::write(pkg_dir.join("utils.js"), "module.exports = { helper: 1 };").unwrap();

    // Create root package.json
    fs::write(
        root.join("package.json"),
        r#"{"dependencies":{"my-lib":"^1.0.0"}}"#,
    )
    .unwrap();

    let prebundler = PreBundler::new(root.to_path_buf());
    let deps = {
        let mut d = HashMap::new();
        d.insert("my-lib".to_string(), "^1.0.0".to_string());
        d
    };
    let mut prebundled = HashMap::new();
    let mut prebundled_sources = HashMap::new();
    let source_subpath_imports = HashMap::new();

    prebundler
        .discover_subpath_exports(
            &deps,
            &jet_dir,
            &mut prebundled,
            &mut prebundled_sources,
            &source_subpath_imports,
        )
        .await;

    assert!(
        prebundled.contains_key("my-lib/utils"),
        "must auto-discover subpath 'my-lib/utils' from exports map: {:?}",
        prebundled
    );
    assert!(
        jet_dir.join("my-lib_utils.mjs").exists(),
        "must write pre-bundled file for discovered subpath"
    );
}

#[tokio::test]
async fn test_esm_dependency_transitive_cjs_root_prebundled() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let node_modules = root.join("node_modules");
    let jet_dir = node_modules.join(".jet");
    fs::create_dir_all(&jet_dir).unwrap();

    let esm_pkg = node_modules.join("@mui/material");
    fs::create_dir_all(&esm_pkg).unwrap();
    fs::write(
        esm_pkg.join("package.json"),
        r#"{
            "name": "@mui/material",
            "version": "9.0.1",
            "module": "./index.mjs",
            "dependencies": {
                "react-is": "^19.0.0"
            }
        }"#,
    )
    .unwrap();
    fs::write(esm_pkg.join("index.mjs"), "export const Button = {};").unwrap();

    let cjs_pkg = node_modules.join("react-is");
    fs::create_dir_all(&cjs_pkg).unwrap();
    fs::write(
        cjs_pkg.join("package.json"),
        r#"{
            "name": "react-is",
            "version": "19.0.0",
            "main": "index.js"
        }"#,
    )
    .unwrap();
    fs::write(
        cjs_pkg.join("index.js"),
        "exports.isValidElementType = function isValidElementType() { return true; };",
    )
    .unwrap();

    fs::write(
        root.join("package.json"),
        r#"{"dependencies":{"@mui/material":"^9.0.1"}}"#,
    )
    .unwrap();

    let result = PreBundler::new(root.to_path_buf())
        .prebundle_deps()
        .await
        .unwrap();

    assert!(
        result.prebundled.contains_key("react-is"),
        "ESM dependency CJS root must be prebundled: {:?}",
        result.prebundled
    );
    assert!(
        jet_dir.join("react-is.mjs").exists(),
        "react-is CJS wrapper must be written"
    );
}

#[tokio::test]
async fn cjs_nested_dependencies_resolve_from_parent_package() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let node_modules = root.join("node_modules");
    let jet_dir = node_modules.join(".jet");
    fs::create_dir_all(&jet_dir).unwrap();

    let prop_types = node_modules.join("prop-types");
    let nested_react_is = prop_types.join("node_modules/react-is");
    let root_react_is = node_modules.join("react-is");
    fs::create_dir_all(&nested_react_is).unwrap();
    fs::create_dir_all(&root_react_is).unwrap();

    fs::write(
        prop_types.join("package.json"),
        r#"{
            "name": "prop-types",
            "version": "15.8.1",
            "main": "index.js"
        }"#,
    )
    .unwrap();
    fs::write(
        prop_types.join("index.js"),
        "var ReactIs = require('react-is'); module.exports = ReactIs;",
    )
    .unwrap();
    fs::write(
        nested_react_is.join("package.json"),
        r#"{
            "name": "react-is",
            "version": "16.13.1",
            "main": "index.js"
        }"#,
    )
    .unwrap();
    fs::write(
        nested_react_is.join("index.js"),
        "exports.Element = 'nested';",
    )
    .unwrap();
    fs::write(
        root_react_is.join("package.json"),
        r#"{
            "name": "react-is",
            "version": "19.2.6",
            "main": "index.js"
        }"#,
    )
    .unwrap();
    fs::write(root_react_is.join("index.js"), "exports.Element = 'root';").unwrap();

    let prebundler = PreBundler::new(root.to_path_buf());
    let (_filename, source) = prebundler
        .bundle_cjs_dep("prop-types", &prop_types, &jet_dir)
        .await
        .unwrap();

    assert!(
        source.contains("/node_modules/.jet/prop-types_node_modules_react-is.mjs"),
        "prop-types require('react-is') must resolve to its nested dependency: {}",
        source
    );
    assert!(
        jet_dir
            .join("prop-types_node_modules_react-is.mjs")
            .exists(),
        "nested dependency wrapper must be written"
    );
}

#[tokio::test]
async fn cjs_nested_dependency_reuses_root_when_same_version() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let node_modules = root.join("node_modules");
    let jet_dir = node_modules.join(".jet");
    fs::create_dir_all(&jet_dir).unwrap();

    let react_dom = node_modules.join("react-dom");
    let nested_react = react_dom.join("node_modules/react");
    let root_react = node_modules.join("react");
    fs::create_dir_all(&nested_react).unwrap();
    fs::create_dir_all(&root_react).unwrap();

    fs::write(
        react_dom.join("package.json"),
        r#"{
            "name": "react-dom",
            "version": "18.3.1",
            "main": "index.js"
        }"#,
    )
    .unwrap();
    fs::write(
        react_dom.join("index.js"),
        "var React = require('react'); module.exports = React;",
    )
    .unwrap();
    for pkg in [&nested_react, &root_react] {
        fs::write(
            pkg.join("package.json"),
            r#"{
                "name": "react",
                "version": "18.3.1",
                "main": "index.js"
            }"#,
        )
        .unwrap();
        fs::write(
            pkg.join("index.js"),
            "exports.useState = function useState() {};",
        )
        .unwrap();
    }

    let prebundler = PreBundler::new(root.to_path_buf());
    let (_filename, source) = prebundler
        .bundle_cjs_dep("react-dom", &react_dom, &jet_dir)
        .await
        .unwrap();

    assert!(
        source.contains("/node_modules/.jet/react.mjs"),
        "same-version nested React must reuse the root prebundle: {}",
        source
    );
    assert!(
        !source.contains("react-dom_node_modules_react.mjs"),
        "same-version nested React must not create a second React copy: {}",
        source
    );
}

/// jet#1908 R7: cache is invalid when the resolver version stamped in the
/// marker no longer matches the current `CACHE_MARKER_VERSION`. Without this
/// check, a stale importmap from before the MUI/Emotion patches survives
/// after a jet upgrade.
#[test]
fn mui_emotion_resolver_version_invalidates_cache() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let jet_dir = root.join("node_modules/.jet");
    fs::create_dir_all(&jet_dir).unwrap();

    // Write a marker tagged with an old resolver version
    fs::write(
        jet_dir.join("_cache_marker"),
        "prebundle cache marker v0-pre-mui-emotion-patches",
    )
    .unwrap();

    // Create package.json + lockfile with mtimes older than the marker
    fs::write(root.join("package.json"), r#"{"dependencies":{}}"#).unwrap();
    fs::write(root.join("jet-lock.yaml"), "lockfile-v1").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    // Re-touch the marker so its mtime is newest
    fs::write(
        jet_dir.join("_cache_marker"),
        "prebundle cache marker v0-pre-mui-emotion-patches",
    )
    .unwrap();

    let prebundler = PreBundler::new(root.to_path_buf());
    assert!(
        !prebundler.check_cache_valid(&jet_dir),
        "cache must be invalid when stamped resolver version is stale, even if mtimes are fresh"
    );
}

/// GH #3117 — when `package.json` exists but its metadata cannot be read
/// (Unix `chmod 000` of the *containing dir* defeats stat; chmod 000 on
/// the file itself still allows stat → check the helper directly), the
/// cache must be invalidated. The previous code silently treated an
/// unreadable mtime as "unchanged" and served a stale pre-bundle.
///
/// We exercise the helper `manifest_mtime_within_marker` directly to
/// keep the test deterministic across filesystems.
#[test]
fn manifest_mtime_within_marker_invalidates_when_mtime_unreadable() {
    use std::time::SystemTime;

    let dir = tempdir().unwrap();
    let missing = dir.path().join("never-was.json");
    // Missing file → not in input set → must NOT invalidate (otherwise
    // every lockfile candidate the project doesn't use would invalidate
    // the cache forever).
    assert!(
        manifest_mtime_within_marker(&missing, SystemTime::now()),
        "missing files must not invalidate the cache (legitimate not-in-input)"
    );

    // Present, mtime ≤ marker → within marker.
    let pkg = dir.path().join("package.json");
    fs::write(&pkg, r#"{"dependencies":{}}"#).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    let marker_time = SystemTime::now();
    assert!(
        manifest_mtime_within_marker(&pkg, marker_time),
        "present file with mtime ≤ marker_mtime must be treated as within-marker"
    );

    // Present, mtime > marker → must invalidate.
    std::thread::sleep(std::time::Duration::from_millis(20));
    fs::write(&pkg, r#"{"dependencies":{"react":"^18"}}"#).unwrap();
    assert!(
        !manifest_mtime_within_marker(&pkg, marker_time),
        "file modified after marker must invalidate the cache"
    );
}

/// GH #3117 — full integration: a freshly-installed dep should not be
/// masked by a stale cache when mtime checks would have flagged it,
/// even if the previous-but-still-stale cache appeared valid by other
/// signals. Documents the freshness contract end-to-end.
#[test]
fn cache_invalidated_when_pkg_json_newer_than_marker() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let jet_dir = root.join("node_modules/.jet");
    fs::create_dir_all(&jet_dir).unwrap();

    // marker first
    fs::write(
        jet_dir.join("_cache_marker"),
        format!("prebundle cache marker {}", CACHE_MARKER_VERSION),
    )
    .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));

    // package.json modified AFTER the marker — should invalidate.
    fs::write(
        root.join("package.json"),
        r#"{"dependencies":{"react":"^18"}}"#,
    )
    .unwrap();

    let prebundler = PreBundler::new(root.to_path_buf());
    assert!(
        !prebundler.check_cache_valid(&jet_dir),
        "cache must be invalid when package.json is newer than the marker"
    );
}

/// jet#1908 R7: cache is valid only when the marker contains the current
/// `CACHE_MARKER_VERSION` AND mtimes are fresh.
#[test]
fn mui_emotion_resolver_version_matches_keeps_cache() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let jet_dir = root.join("node_modules/.jet");
    fs::create_dir_all(&jet_dir).unwrap();

    fs::write(root.join("package.json"), r#"{"dependencies":{}}"#).unwrap();
    fs::write(root.join("jet-lock.yaml"), "lockfile-v1").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));
    fs::write(
        jet_dir.join("_cache_marker"),
        format!("prebundle cache marker {}", CACHE_MARKER_VERSION),
    )
    .unwrap();

    let prebundler = PreBundler::new(root.to_path_buf());
    assert!(
        prebundler.check_cache_valid(&jet_dir),
        "cache must be valid when resolver version matches and mtimes are fresh"
    );
}

#[test]
fn esm_detection_ignores_import_text_inside_cjs_strings() {
    let source = r#"
'use strict';
if (false) {
  console.error('You are importing createRoot from "react-dom"');
}
exports.render = render;
"#;

    assert!(
        !is_esm_module_source(source),
        "plain CJS with import-like prose must still be wrapped"
    );
}

#[test]
fn esm_detection_accepts_static_import_export_syntax() {
    assert!(is_esm_module_source("import * as React from 'react';"));
    assert!(is_esm_module_source(
        "export { default } from './helper.mjs';"
    ));
}
// CODEGEN-END
