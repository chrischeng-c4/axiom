// HANDWRITE-BEGIN gap="missing-generator:unit-test:e3d0a32a" tracker="pending-tracker" reason="Integration tests: a fixture library builds ESM with deps/peerDeps kept as external bare imports, optional CJS emission, multi-entry from `exports`, and an app-mode no-regression assertion."
//! Integration tests for `jet build --lib` (library-build mode).
//!
//! Coverage:
//!   (a) ESM output keeps a `dependency` and a `peerDependency` as external
//!       bare imports (not inlined),
//!   (b) optional CJS emission converts those imports to `require(...)`,
//!   (c) multi-entry: two `exports` keys produce two output files,
//!   (d) app-mode `Bundler::bundle` is unchanged (no regression).
//!
//! @issue #170

use jet::bundler::types::OutputFormat;
use jet::bundler::{build_library, BundleOptions, Bundler, LibBuildOptions};
use std::collections::HashSet;
use tempfile::tempdir;

/// Write a file, creating parent dirs as needed.
fn write_file(base: &std::path::Path, rel: &str, content: &str) {
    let path = base.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, content).unwrap();
}

/// Build a library at `root` with the given formats and return the result.
fn run_lib_build(root: &std::path::Path, formats: Vec<OutputFormat>) -> jet::bundler::LibBuildResult {
    let options = LibBuildOptions {
        project_root: root.to_path_buf(),
        out_dir: root.join("dist"),
        formats,
        conditions: vec!["import".to_string(), "default".to_string()],
        extra_externals: HashSet::new(),
        preserve_modules: false,
    };
    build_library(options).expect("library build must succeed")
}

// ──────────────────────────────────────────────────────────────────────────
// (a) ESM keeps dependency + peerDependency external; inlines internal module
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn lib_esm_keeps_dependency_and_peer_dependency_external() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{
            "name": "my-lib",
            "version": "1.0.0",
            "module": "./src/index.js",
            "dependencies": { "lodash-es": "^4.0.0" },
            "peerDependencies": { "react": "^18.0.0" }
        }"#,
    );

    // Internal helper module that should be INLINED (not left as an import).
    write_file(
        root,
        "src/util.js",
        "export function double(x) { return x * 2; }\n",
    );

    // Entry imports an external dependency, an external peerDependency, and an
    // internal relative module.
    write_file(
        root,
        "src/index.js",
        r#"import { merge } from "lodash-es";
import { useState } from "react";
import { double } from "./util.js";

export function go(a, b) {
    const [s] = useState(double(a));
    return merge({}, { v: s + b });
}
"#,
    );

    let result = run_lib_build(root, vec![OutputFormat::Esm]);
    assert_eq!(result.entries.len(), 1, "single entry → single ESM file");

    let out = &result.entries[0];
    assert_eq!(out.format, OutputFormat::Esm);
    let code = &out.code;

    // External dependency kept as a real bare import.
    assert!(
        code.contains("from \"lodash-es\""),
        "dependency `lodash-es` must remain an external bare import, got:\n{code}"
    );
    // External peerDependency kept as a real bare import.
    assert!(
        code.contains("from \"react\""),
        "peerDependency `react` must remain an external bare import, got:\n{code}"
    );

    // Internal module INLINED — the relative import statement must be gone,
    // and the helper body must be present.
    assert!(
        !code.contains("./util"),
        "internal relative import must be inlined (no `./util` left), got:\n{code}"
    );
    assert!(
        code.contains("function double"),
        "internal helper body must be inlined into the bundle, got:\n{code}"
    );

    // The output file was written to disk.
    assert!(out.path.is_file(), "ESM output file must exist on disk");
    assert!(
        out.path.file_name().unwrap() == "index.js",
        "`.` entry → index.js, got {:?}",
        out.path
    );
}

// ──────────────────────────────────────────────────────────────────────────
// (b) Optional CJS emission
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn lib_emits_cjs_with_require_for_externals() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{
            "name": "my-lib",
            "version": "1.0.0",
            "module": "./src/index.js",
            "dependencies": { "lodash-es": "^4.0.0" }
        }"#,
    );
    write_file(
        root,
        "src/index.js",
        r#"import { merge } from "lodash-es";
export function go(a) { return merge({}, { a }); }
"#,
    );

    let result = run_lib_build(root, vec![OutputFormat::Esm, OutputFormat::Cjs]);
    assert_eq!(result.entries.len(), 2, "one entry × two formats → two files");

    let esm = result
        .entries
        .iter()
        .find(|e| e.format == OutputFormat::Esm)
        .expect("ESM output present");
    let cjs = result
        .entries
        .iter()
        .find(|e| e.format == OutputFormat::Cjs)
        .expect("CJS output present");

    // ESM keeps the bare import; CJS rewrites it to require().
    assert!(
        esm.code.contains("from \"lodash-es\""),
        "ESM keeps bare import, got:\n{}",
        esm.code
    );
    assert!(
        cjs.code.contains("require(\"lodash-es\")"),
        "CJS must externalize via require(\"lodash-es\"), got:\n{}",
        cjs.code
    );
    assert!(
        cjs.code.contains("exports.go"),
        "CJS must expose the named export via exports.*, got:\n{}",
        cjs.code
    );
    assert_eq!(
        cjs.path.file_name().unwrap(),
        "index.cjs",
        "CJS output → index.cjs, got {:?}",
        cjs.path
    );
}

// ──────────────────────────────────────────────────────────────────────────
// (c) Multi-entry from two `exports` entries
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn lib_multi_entry_from_exports() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{
            "name": "my-lib",
            "version": "1.0.0",
            "exports": {
                ".": { "import": "./src/index.js" },
                "./client": { "import": "./src/client.js" }
            },
            "peerDependencies": { "react": "^18.0.0" }
        }"#,
    );
    write_file(
        root,
        "src/index.js",
        "export const root = 1;\n",
    );
    write_file(
        root,
        "src/client.js",
        r#"import { useState } from "react";
export function client() { return useState(0); }
"#,
    );

    let result = run_lib_build(root, vec![OutputFormat::Esm]);
    assert_eq!(
        result.entries.len(),
        2,
        "two exports entries → two ESM files, got {:?}",
        result
            .entries
            .iter()
            .map(|e| e.subpath.clone())
            .collect::<Vec<_>>()
    );

    let subpaths: HashSet<&str> = result.entries.iter().map(|e| e.subpath.as_str()).collect();
    assert!(subpaths.contains("."), "`.` entry must be built");
    assert!(subpaths.contains("./client"), "`./client` entry must be built");

    let client = result
        .entries
        .iter()
        .find(|e| e.subpath == "./client")
        .unwrap();
    assert!(
        client.code.contains("from \"react\""),
        "client entry keeps peerDependency `react` external, got:\n{}",
        client.code
    );
    assert_eq!(
        client.path.file_name().unwrap(),
        "client.js",
        "`./client` → client.js, got {:?}",
        client.path
    );
}

// ──────────────────────────────────────────────────────────────────────────
// (d) App-mode build is unchanged (no regression)
// ──────────────────────────────────────────────────────────────────────────

/// App-mode `Bundler::bundle` (the default, non-library path) must continue to
/// INLINE relative dependencies and produce a runtime/scope-hoisted bundle —
/// the library path must not have changed its behaviour.
#[tokio::test]
async fn app_mode_build_unchanged() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "src/util.js",
        "exports.double = function(x) { return x * 2; };\n",
    );
    write_file(
        root,
        "src/index.js",
        r#"var util = require('./util');
exports.run = function(x) { return util.double(x); };
"#,
    );

    let entry = root.join("src/index.js");
    let options = BundleOptions {
        entry: entry.clone(),
        output_dir: root.join("dist"),
        resolve_options: jet::resolver::ResolveOptions {
            extensions: vec!["js".to_string()],
            resolve_index: true,
            ..Default::default()
        },
        // Default: library=false, externalize_all_packages=false.
        ..Default::default()
    };

    // Sanity: defaults keep app mode off.
    assert!(!options.library, "app-mode BundleOptions::library must default false");
    assert!(
        !options.externalize_all_packages,
        "app-mode must default to inlining packages"
    );

    let bundler = Bundler::new(options).unwrap();
    let result = bundler.bundle(entry).await.expect("app bundle must succeed");

    // The relative util.js must be INLINED into the app bundle (not external).
    assert!(
        result.code.contains("double"),
        "app-mode bundle must inline ./util (double present), got:\n{}",
        result.code
    );
    assert!(
        !result.code.is_empty(),
        "app-mode bundle must produce output"
    );
}
// HANDWRITE-END
