// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests for bundler fixes:
//!   1. Circular dependency handling — bundle completes instead of erroring
//!   2. Monorepo node_modules walk-up — packages at workspace root are bundled

use jet::bundler::{BundleOptions, Bundler};
use jet::resolver::ResolveOptions;
use std::collections::HashSet;
use tempfile::tempdir;

// Helper: write a file, creating parent dirs as needed.
fn write_file(base: &std::path::Path, rel: &str, content: &str) {
    let path = base.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, content).unwrap();
}

// ──────────────────────────────────────────────────────────────────────────
// Test 1: Circular dependency — bundle must complete without error
// ──────────────────────────────────────────────────────────────────────────

/// A → B → A forms a cycle (shared-ui-form-inputs pattern).
/// Previously the bundler bailed with "Circular dependency detected" error.
/// After the fix, the bundle must succeed using the runtime module system.
#[tokio::test]
async fn test_bundler_circular_dependency_completes() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // a.js → imports b.js
    write_file(
        root,
        "src/a.js",
        r#"var b = require('./b');
exports.hello = function() { return 'hello from a, b says: ' + b.world(); };
"#,
    );

    // b.js → imports a.js (creates cycle a → b → a)
    write_file(
        root,
        "src/b.js",
        r#"var a = require('./a');
exports.world = function() { return 'world from b'; };
"#,
    );

    let entry = root.join("src/a.js");
    let options = BundleOptions {
        entry: entry.clone(),
        output_dir: root.join("dist"),
        resolve_options: ResolveOptions {
            extensions: vec!["js".to_string()],
            resolve_index: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let bundler = Bundler::new(options).unwrap();
    let result = bundler.bundle(entry).await;

    assert!(
        result.is_ok(),
        "Bundler must not bail on circular dependencies, got error: {:?}",
        result.err()
    );

    let output = result.unwrap();
    assert!(
        !output.code.is_empty(),
        "Bundle output must not be empty for circular dependency graph"
    );

    // The bundle must contain both modules
    assert!(
        output.code.contains("hello from a"),
        "Module a content must be present in bundle"
    );
    assert!(
        output.code.contains("world from b"),
        "Module b content must be present in bundle"
    );

    // The runtime module system must be used (has __jet__.define and __jet__.require)
    assert!(
        output.code.contains("__jet__"),
        "Cyclic bundle must use the __jet__ runtime module system"
    );
}

/// Three-module cycle: A → B → C → A.
/// Verifies the fallback handles larger cycles correctly.
#[tokio::test]
async fn test_bundler_three_module_cycle_completes() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(root, "src/a.js", "var b = require('./b'); exports.a = 1;");
    write_file(root, "src/b.js", "var c = require('./c'); exports.b = 2;");
    write_file(root, "src/c.js", "var a = require('./a'); exports.c = 3;");

    let entry = root.join("src/a.js");
    let options = BundleOptions {
        entry: entry.clone(),
        output_dir: root.join("dist"),
        resolve_options: ResolveOptions {
            extensions: vec!["js".to_string()],
            resolve_index: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let bundler = Bundler::new(options).unwrap();
    let result = bundler.bundle(entry).await;

    assert!(
        result.is_ok(),
        "Three-module cycle must not cause an error: {:?}",
        result.err()
    );
}

// ──────────────────────────────────────────────────────────────────────────
// Test 2: Monorepo node_modules walk-up — package at workspace root bundled
// ──────────────────────────────────────────────────────────────────────────

/// Simulates an Nx monorepo where React is installed at the workspace root
/// but the app is in a subdirectory apps/demo/src/.
///
/// Layout:
///   workspace_root/
///     node_modules/react/     ← React installed here
///     apps/demo/src/index.js  ← entry point, imports react
///
/// The bundler must resolve 'react' by walking up from apps/demo/src/
/// and find it at workspace_root/node_modules/react.
/// Acceptance: bundle output > 100 bytes (React code is included).
#[tokio::test]
async fn test_bundler_resolves_monorepo_workspace_root_package() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create a minimal React-like package at workspace root node_modules.
    // Use a trimmed fixture (not real React) to keep the test fast.
    write_file(
        root,
        "node_modules/react/package.json",
        r#"{"name":"react","version":"18.0.0","main":"index.js"}"#,
    );
    write_file(
        root,
        "node_modules/react/index.js",
        r#"// React trimmed fixture
exports.createElement = function(type, props) { return { type: type, props: props }; };
exports.useState = function(initial) { return [initial, function() {}]; };
exports.useEffect = function(fn) {};
exports.Component = function Component() {};
exports.version = '18.0.0';
"#,
    );

    // Entry app in a deeply nested project directory
    write_file(
        root,
        "apps/demo/src/index.js",
        r#"var React = require('react');
exports.render = function(el) {
    return React.createElement(el, {});
};
exports.version = React.version;
"#,
    );

    let entry = root.join("apps/demo/src/index.js");
    let options = BundleOptions {
        entry: entry.clone(),
        output_dir: root.join("dist"),
        resolve_options: ResolveOptions {
            extensions: vec!["js".to_string()],
            resolve_index: true,
            externals: HashSet::new(), // React must NOT be external
            ..Default::default()
        },
        ..Default::default()
    };

    let bundler = Bundler::new(options).unwrap();
    let result = bundler.bundle(entry).await;

    assert!(
        result.is_ok(),
        "Bundler must resolve React from workspace root node_modules: {:?}",
        result.err()
    );

    let output = result.unwrap();

    // React code must be inlined (not treated as external)
    assert!(
        output.code.contains("createElement"),
        "React.createElement must be present in the bundle (not treated as external)"
    );
    assert!(
        output.code.contains("useState"),
        "React.useState must be present in the bundle"
    );
    assert!(
        output.code.contains("18.0.0"),
        "React version string from fixture must appear in bundle"
    );

    // Bundle must be substantially larger than just the app code (React is included)
    // The fixture React is ~200 bytes; combined bundle must exceed 100 bytes.
    assert!(
        output.code.len() > 100,
        "Bundle ({} bytes) must include React code from workspace root (expected > 100 bytes)",
        output.code.len()
    );
}

/// Packages in project-level node_modules take priority over workspace root.
/// This mirrors Node.js resolution: closest node_modules wins.
#[tokio::test]
async fn test_bundler_project_node_modules_takes_priority() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Workspace root has lodash v4
    write_file(
        root,
        "node_modules/lodash/package.json",
        r#"{"name":"lodash","version":"4.0.0","main":"lodash.js"}"#,
    );
    write_file(
        root,
        "node_modules/lodash/lodash.js",
        "exports.version = '4.0.0'; exports.merge = function() {};",
    );

    // Project-level node_modules has lodash v3 (closer, must win)
    write_file(
        root,
        "apps/demo/node_modules/lodash/package.json",
        r#"{"name":"lodash","version":"3.0.0","main":"lodash.js"}"#,
    );
    write_file(
        root,
        "apps/demo/node_modules/lodash/lodash.js",
        "exports.version = '3.0.0'; exports.merge = function() {};",
    );

    write_file(
        root,
        "apps/demo/src/index.js",
        "var _ = require('lodash'); exports.v = _.version;",
    );

    let entry = root.join("apps/demo/src/index.js");
    let options = BundleOptions {
        entry: entry.clone(),
        output_dir: root.join("dist"),
        resolve_options: ResolveOptions {
            extensions: vec!["js".to_string()],
            resolve_index: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let bundler = Bundler::new(options).unwrap();
    let result = bundler.bundle(entry).await.unwrap();

    // Project-level lodash v3 must be picked (it's closer to the importing file)
    assert!(
        result.code.contains("3.0.0"),
        "Project-level lodash v3 must take priority over workspace-root v4: {}",
        result.code
    );
    assert!(
        !result.code.contains("4.0.0"),
        "Workspace-root lodash v4 must NOT appear when project-level v3 is present"
    );
}
// CODEGEN-END
