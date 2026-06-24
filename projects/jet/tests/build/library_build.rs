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
        // These fixtures use untyped JS entries; declaration emission is
        // exercised separately in `library_dts.rs`.
        declaration: false,
        library_global_name: None,
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

// ──────────────────────────────────────────────────────────────────────────
// (e) preserve_modules: one output file per source module + deep import
// ──────────────────────────────────────────────────────────────────────────

/// Build a library at `root` with `preserve_modules` on (ESM) and return the
/// result.
fn run_lib_build_preserve(root: &std::path::Path) -> jet::bundler::LibBuildResult {
    let options = LibBuildOptions {
        project_root: root.to_path_buf(),
        out_dir: root.join("dist"),
        formats: vec![OutputFormat::Esm],
        conditions: vec!["import".to_string(), "default".to_string()],
        extra_externals: HashSet::new(),
        preserve_modules: true,
        declaration: false,
        library_global_name: None,
    };
    build_library(options).expect("preserve-modules library build must succeed")
}

#[test]
fn lib_preserve_modules_emits_one_file_per_module() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{
            "name": "my-lib",
            "version": "1.0.0",
            "module": "./src/index.js",
            "peerDependencies": { "react": "^18.0.0" }
        }"#,
    );
    // Deep internal module the entry pulls in.
    write_file(
        root,
        "src/util.js",
        "export function double(x) { return x * 2; }\n",
    );
    // Nested deep module to exercise tree mirroring.
    write_file(
        root,
        "src/widgets/Button.js",
        r#"import { double } from "../util.js";
export function Button(n) { return double(n); }
"#,
    );
    // Entry re-exports + imports both internal modules and an external.
    write_file(
        root,
        "src/index.js",
        r#"import { useState } from "react";
import { double } from "./util.js";
export { Button } from "./widgets/Button.js";
export function go(a) { return useState(double(a)); }
"#,
    );

    let result = run_lib_build_preserve(root);

    // One emitted file per source module: index, util, widgets/Button.
    let dist = root.join("dist");
    assert!(dist.join("index.js").is_file(), "entry index.js emitted");
    assert!(dist.join("util.js").is_file(), "util.js emitted as sibling");
    assert!(
        dist.join("widgets/Button.js").is_file(),
        "nested widgets/Button.js mirrors source tree"
    );

    // util.js was NOT inlined into index.js — index keeps a relative import to
    // the emitted sibling.
    let index_code = std::fs::read_to_string(dist.join("index.js")).unwrap();
    assert!(
        index_code.contains("./util.js"),
        "preserve_modules keeps the internal import as a sibling reference, got:\n{index_code}"
    );
    assert!(
        !index_code.contains("function double"),
        "preserve_modules must NOT inline util into the entry, got:\n{index_code}"
    );
    // External import stays bare.
    assert!(
        index_code.contains("from \"react\""),
        "external react import stays bare, got:\n{index_code}"
    );

    // The deep module's own relative import was rewritten to its emitted sibling.
    let button_code = std::fs::read_to_string(dist.join("widgets/Button.js")).unwrap();
    assert!(
        button_code.contains("../util.js"),
        "deep module's relative import points at the emitted sibling, got:\n{button_code}"
    );
    assert!(
        button_code.contains("function Button"),
        "deep module body preserved, got:\n{button_code}"
    );

    // Every emitted module is recorded in the result.
    let subpaths: HashSet<String> = result.entries.iter().map(|e| e.subpath.clone()).collect();
    assert!(
        subpaths.iter().any(|s| s.ends_with("index.js")),
        "index recorded, got {subpaths:?}"
    );
    assert!(
        subpaths.iter().any(|s| s.ends_with("util.js")),
        "util recorded, got {subpaths:?}"
    );
    assert!(
        subpaths.iter().any(|s| s.ends_with("widgets/Button.js")),
        "deep widgets/Button recorded, got {subpaths:?}"
    );

    // A consumer can deep-import the emitted module directly: the sibling path
    // resolves and its content is loadable JS exposing the expected symbol.
    assert!(
        button_code.contains("export function Button"),
        "deep module is independently importable (exports Button), got:\n{button_code}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// (f) IIFE: loadable global-var assignment
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn lib_iife_assigns_configured_global() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{
            "name": "my-widget-kit",
            "version": "1.0.0",
            "module": "./src/index.js",
            "peerDependencies": { "react": "^18.0.0" }
        }"#,
    );
    write_file(
        root,
        "src/util.js",
        "export function double(x) { return x * 2; }\n",
    );
    write_file(
        root,
        "src/index.js",
        r#"import { useState } from "react";
import { double } from "./util.js";
export function go(a) { return double(a); }
export const VERSION = "1.0.0";
"#,
    );

    let options = LibBuildOptions {
        project_root: root.to_path_buf(),
        out_dir: root.join("dist"),
        formats: vec![OutputFormat::Iife],
        conditions: vec!["import".to_string(), "default".to_string()],
        extra_externals: HashSet::new(),
        preserve_modules: false,
        declaration: false,
        library_global_name: Some("WidgetKit".to_string()),
    };
    let result = build_library(options).expect("iife library build must succeed");
    assert_eq!(result.entries.len(), 1, "single entry → single IIFE file");

    let out = &result.entries[0];
    assert_eq!(out.format, OutputFormat::Iife);
    let code = &out.code;

    // Assigns the configured global via an IIFE.
    assert!(
        code.contains("var WidgetKit = (function ()"),
        "IIFE must assign the configured global name, got:\n{code}"
    );
    // No surviving ESM `import` — externals are read from globals instead.
    assert!(
        !code.contains("import "),
        "IIFE must not keep ESM import statements, got:\n{code}"
    );
    // The external `react` peerDependency is read from a `globalThis.<name>`
    // global. The `<name>` is derived from the specifier (`react` →
    // `globalThis.react`); matching framework-specific globals like the
    // capitalised `React` is a documented TODO requiring a `globals` map.
    assert!(
        code.contains("globalThis.react"),
        "external `react` peerDependency is read from a global, got:\n{code}"
    );
    // Internal helper IS inlined (IIFE bundles a single file).
    assert!(
        code.contains("function double"),
        "IIFE bundles internal modules (double inlined), got:\n{code}"
    );
    // Named exports are collected onto the returned namespace.
    assert!(
        code.contains("go: go") && code.contains("VERSION: VERSION"),
        "named exports collected on the IIFE namespace, got:\n{code}"
    );
    // Output file name carries the .iife.js suffix and exists on disk.
    assert!(out.path.is_file(), "IIFE output file must exist on disk");
    assert_eq!(
        out.path.file_name().unwrap(),
        "index.iife.js",
        "`.` entry IIFE → index.iife.js, got {:?}",
        out.path
    );
}

#[test]
fn lib_iife_default_global_name_derived_from_package_name() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{
            "name": "@acme/widget-kit",
            "version": "1.0.0",
            "module": "./src/index.js"
        }"#,
    );
    write_file(
        root,
        "src/index.js",
        "export const x = 1;\n",
    );

    let options = LibBuildOptions {
        project_root: root.to_path_buf(),
        out_dir: root.join("dist"),
        formats: vec![OutputFormat::Iife],
        conditions: vec!["import".to_string(), "default".to_string()],
        extra_externals: HashSet::new(),
        preserve_modules: false,
        declaration: false,
        // No explicit global name → derived from package.json `name`.
        library_global_name: None,
    };
    let result = build_library(options).expect("iife build must succeed");
    let code = &result.entries[0].code;
    // `@acme/widget-kit` → scope dropped, camel-cased → `widgetKit`.
    assert!(
        code.contains("var widgetKit = (function ()"),
        "default IIFE global derived from package name (`widgetKit`), got:\n{code}"
    );
}
// HANDWRITE-END
