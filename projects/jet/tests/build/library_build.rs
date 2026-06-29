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
use jet::bundler::types::SourceMapOption;
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
fn run_lib_build(
    root: &std::path::Path,
    formats: Vec<OutputFormat>,
) -> jet::bundler::LibBuildResult {
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
        entry: Vec::new(),
        css_merge: Vec::new(),
        raw_copy: Vec::new(),
        sourcemap: SourceMapOption::None,
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
    assert_eq!(
        result.entries.len(),
        2,
        "one entry × two formats → two files"
    );

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

#[test]
fn lib_cjs_multiline_class_export_assignment_stays_after_body_and_types_are_stripped() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{
            "name": "typed-class-lib",
            "version": "1.0.0",
            "module": "./src/index.ts"
        }"#,
    );
    write_file(
        root,
        "src/index.ts",
        r#"export class Greeter {
    greet(name: string): string {
        return `hello ${name}`;
    }
}

export const version: string = "1.0.0";
"#,
    );

    let result = run_lib_build(root, vec![OutputFormat::Esm, OutputFormat::Cjs]);
    let esm = result
        .entries
        .iter()
        .find(|e| e.format == OutputFormat::Esm)
        .expect("ESM output present");
    let cjs = cjs_code(&result);

    assert!(
        !esm.code.contains(": string") && !cjs.contains(": string"),
        "library JS outputs must strip TypeScript annotations.\nESM:\n{}\nCJS:\n{}",
        esm.code,
        cjs
    );
    assert!(
        cjs.contains("class Greeter") && cjs.contains("greet(name)"),
        "CJS output must preserve the class body as JavaScript, got:\n{cjs}"
    );
    let body_pos = cjs
        .find("return `hello ${name}`;")
        .expect("class method body present");
    let export_pos = cjs
        .find("exports.Greeter = Greeter;")
        .expect("Greeter export assignment present");
    assert!(
        export_pos > body_pos,
        "exports.Greeter assignment must be emitted after the class body, got:\n{cjs}"
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
    write_file(root, "src/index.js", "export const root = 1;\n");
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
    assert!(
        subpaths.contains("./client"),
        "`./client` entry must be built"
    );

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

#[test]
fn lib_build_fails_loudly_for_unsupported_local_asset_imports() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{
            "name": "asset-import-lib",
            "version": "1.0.0",
            "module": "./src/index.js"
        }"#,
    );
    write_file(root, "src/styles.scss", ".box { color: red; }\n");
    write_file(
        root,
        "src/index.js",
        "import './styles.scss';\nexport const Box = 'box';\n",
    );

    let options = LibBuildOptions {
        project_root: root.to_path_buf(),
        out_dir: root.join("dist"),
        formats: vec![OutputFormat::Esm],
        conditions: vec!["import".to_string(), "default".to_string()],
        extra_externals: HashSet::new(),
        preserve_modules: false,
        declaration: false,
        library_global_name: None,
        entry: Vec::new(),
        css_merge: Vec::new(),
        raw_copy: Vec::new(),
        sourcemap: SourceMapOption::None,
    };
    let err = build_library(options).expect_err("SCSS import must fail loudly");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("unsupported local import extension '.scss'")
            && msg.contains("css_merge/raw_copy"),
        "error should tell the operator why the asset import is unsupported, got:\n{msg}"
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
    assert!(
        !options.library,
        "app-mode BundleOptions::library must default false"
    );
    assert!(
        !options.externalize_all_packages,
        "app-mode must default to inlining packages"
    );

    let bundler = Bundler::new(options).unwrap();
    let result = bundler
        .bundle(entry)
        .await
        .expect("app bundle must succeed");

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
        entry: Vec::new(),
        css_merge: Vec::new(),
        raw_copy: Vec::new(),
        sourcemap: SourceMapOption::None,
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
        entry: Vec::new(),
        css_merge: Vec::new(),
        raw_copy: Vec::new(),
        sourcemap: SourceMapOption::None,
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
    write_file(root, "src/index.js", "export const x = 1;\n");

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
        entry: Vec::new(),
        css_merge: Vec::new(),
        raw_copy: Vec::new(),
        sourcemap: SourceMapOption::None,
    };
    let result = build_library(options).expect("iife build must succeed");
    let code = &result.entries[0].code;
    // `@acme/widget-kit` → scope dropped, camel-cased → `widgetKit`.
    assert!(
        code.contains("var widgetKit = (function ()"),
        "default IIFE global derived from package name (`widgetKit`), got:\n{code}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// (g) CJS re-export + renamed-alias edge cases
// ──────────────────────────────────────────────────────────────────────────

/// Find the CJS output among a build result's entries.
fn cjs_code(result: &jet::bundler::LibBuildResult) -> String {
    result
        .entries
        .iter()
        .find(|e| e.format == OutputFormat::Cjs)
        .expect("CJS output present")
        .code
        .clone()
}

/// `export { Foo as Bar } from "./foo"` (renamed, RELATIVE) is now INLINED in
/// single-file mode: `./foo`'s body lands in the bundle (private) and the
/// public `Bar` is bound to foo's `Foo` locally. The CJS lowering exposes
/// `exports.Bar = Foo` with NO dangling `require("./foo.js")`.
#[test]
fn lib_cjs_renamed_reexport_from_relative_inlines() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{
            "name": "my-lib",
            "version": "1.0.0",
            "module": "./src/index.js"
        }"#,
    );
    write_file(root, "src/foo.js", "export function Foo() { return 1; }\n");
    // Entry renames foo's `Foo` to the public `Bar` via a re-export.
    write_file(
        root,
        "src/index.js",
        "export { Foo as Bar } from \"./foo\";\n",
    );

    let result = run_lib_build(root, vec![OutputFormat::Esm, OutputFormat::Cjs]);
    let cjs = cjs_code(&result);

    // The target was inlined: foo's body is present and `Bar` binds the local
    // `Foo` — no dangling relative require.
    assert!(
        cjs.contains("function Foo"),
        "inlined foo body must be present, got:\n{cjs}"
    );
    assert!(
        cjs.contains("exports.Bar = Foo;"),
        "renamed relative re-export → exports.Bar = Foo (inlined), got:\n{cjs}"
    );
    assert!(
        !cjs.contains("require(\"./foo"),
        "inlined relative re-export must not leave a dangling require, got:\n{cjs}"
    );
    // The original ESM-only `export … from` shape must not survive into CJS.
    assert!(
        !cjs.contains("export {"),
        "no ESM `export {{` clause may leak into CJS, got:\n{cjs}"
    );
}

/// `export * from "./util"` (RELATIVE) is now INLINED in single-file mode:
/// util's body lands in the bundle keeping its `export` keywords, so each
/// export lowers to `exports.<name>` with NO dangling `require("./util.js")`.
#[test]
fn lib_cjs_star_reexport_from_relative_inlines() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{
            "name": "my-lib",
            "version": "1.0.0",
            "module": "./src/index.js"
        }"#,
    );
    write_file(
        root,
        "src/util.js",
        "export function alpha() {}\nexport function beta() {}\n",
    );
    write_file(root, "src/index.js", "export * from \"./util\";\n");

    let result = run_lib_build(root, vec![OutputFormat::Esm, OutputFormat::Cjs]);
    let cjs = cjs_code(&result);

    // util was inlined: its bodies + per-export `exports.<name>` are present.
    assert!(
        cjs.contains("function alpha") && cjs.contains("function beta"),
        "inlined util bodies must be present, got:\n{cjs}"
    );
    assert!(
        cjs.contains("exports.alpha = alpha;") && cjs.contains("exports.beta = beta;"),
        "each star-re-exported binding surfaces via exports.*, got:\n{cjs}"
    );
    // No dangling relative require — the module was inlined, not referenced.
    assert!(
        !cjs.contains("require(\"./util"),
        "inlined star re-export must not leave a dangling require, got:\n{cjs}"
    );
}

/// `export { useState } from "react"` (external) must keep `require("react")`
/// — externals are never rewritten to a relative path.
#[test]
fn lib_cjs_reexport_from_external_keeps_require_pkg() {
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
    write_file(
        root,
        "src/index.js",
        "export { useState } from \"react\";\n",
    );

    let result = run_lib_build(root, vec![OutputFormat::Esm, OutputFormat::Cjs]);
    let cjs = cjs_code(&result);

    // External re-export keeps the bare `require("react")` specifier.
    assert!(
        cjs.contains("exports.useState = require(\"react\").useState;"),
        "external re-export keeps require(\"react\"), got:\n{cjs}"
    );
    // The external specifier must not be rewritten to a relative `.js` path.
    assert!(
        !cjs.contains("require(\"./react"),
        "external specifier must stay bare (no relative rewrite), got:\n{cjs}"
    );
}

/// A local renamed export (`export { localA as renamedA };`, no `from`) maps the
/// public name to the in-module binding: `exports.renamedA = localA;`.
#[test]
fn lib_cjs_local_renamed_export() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{
            "name": "my-lib",
            "version": "1.0.0",
            "module": "./src/index.js"
        }"#,
    );
    write_file(
        root,
        "src/index.js",
        "function localA() { return 1; }\nexport { localA as renamedA };\n",
    );

    let result = run_lib_build(root, vec![OutputFormat::Esm, OutputFormat::Cjs]);
    let cjs = cjs_code(&result);

    assert!(
        cjs.contains("exports.renamedA = localA;"),
        "local renamed export → exports.renamedA = localA, got:\n{cjs}"
    );
    // The original function declaration is preserved (it is the value source).
    assert!(
        cjs.contains("function localA"),
        "the renamed export's source binding survives, got:\n{cjs}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// (h) Barrel re-export inlining (single-file mode)
// ──────────────────────────────────────────────────────────────────────────

/// A `src/index.ts` barrel of `export * from './lib/a'` + `export { Foo } from
/// './lib/b'` (RELATIVE specifiers) must INLINE the target modules into the
/// single ESM output — the bundle is self-contained, with no dangling
/// `from "./lib/a.js"` sibling reference.
#[test]
fn lib_barrel_reexport_inlines_relative_targets() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{
            "name": "barrel-lib",
            "version": "1.0.0",
            "module": "./src/index.js"
        }"#,
    );
    // `a` is star-re-exported: all of its named exports become public.
    write_file(
        root,
        "src/lib/a.js",
        "export function alpha(x) { return x + 1; }\nexport const A_CONST = 7;\n",
    );
    // `b` is named-re-exported: only `Foo` is published; `Hidden` stays private.
    write_file(
        root,
        "src/lib/b.js",
        "export function Foo() { return \"foo\"; }\nexport function Hidden() { return \"hidden\"; }\n",
    );
    write_file(
        root,
        "src/index.js",
        "export * from './lib/a';\nexport { Foo } from './lib/b';\n",
    );

    let result = run_lib_build(root, vec![OutputFormat::Esm]);
    assert_eq!(result.entries.len(), 1, "single entry → single ESM file");
    let code = &result.entries[0].code;

    // a.js and b.js code is INLINED (function bodies present).
    assert!(
        code.contains("function alpha"),
        "star-re-exported module a's body must be inlined, got:\n{code}"
    );
    assert!(
        code.contains("A_CONST"),
        "star-re-exported const must be inlined, got:\n{code}"
    );
    assert!(
        code.contains("function Foo"),
        "named-re-exported Foo's body must be inlined, got:\n{code}"
    );
    // `Hidden` body is inlined too (it lives in the same module) but stays
    // PRIVATE — only `Foo` is re-exported from the named clause.
    assert!(
        code.contains("function Hidden"),
        "b's other declaration is inlined (private), got:\n{code}"
    );

    // NO dangling sibling reference to the inlined modules.
    assert!(
        !code.contains("from \"./lib/a.js\"") && !code.contains("from './lib/a"),
        "no dangling re-export of ./lib/a may survive, got:\n{code}"
    );
    assert!(
        !code.contains("from \"./lib/b.js\"") && !code.contains("from './lib/b"),
        "no dangling re-export of ./lib/b may survive, got:\n{code}"
    );

    // The exports are surfaced: star keeps a's own `export` keywords, and the
    // named clause re-exports Foo (but NOT Hidden).
    assert!(
        code.contains("export function alpha") || code.contains("export { alpha"),
        "star re-export keeps a's exports public, got:\n{code}"
    );
    assert!(
        code.contains("export { Foo }") || code.contains("export {Foo}"),
        "named clause re-exports Foo, got:\n{code}"
    );
    assert!(
        !code.contains("export function Hidden") && !code.contains("export { Hidden"),
        "Hidden must NOT be re-exported (named clause only published Foo), got:\n{code}"
    );
}

/// Transitive barrel: the entry re-exports `./lib/a`, which itself re-exports
/// `./lib/c`. The chain is followed recursively and fully inlined — `c`'s code
/// lands in the single output with no `./lib/c` sibling reference.
#[test]
fn lib_barrel_reexport_is_transitive() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{
            "name": "transitive-barrel",
            "version": "1.0.0",
            "module": "./src/index.js"
        }"#,
    );
    // c is the deepest module.
    write_file(
        root,
        "src/lib/c.js",
        "export function gamma(n) { return n * 3; }\n",
    );
    // a re-exports c (and adds its own export).
    write_file(
        root,
        "src/lib/a.js",
        "export * from './c';\nexport function alpha() { return 1; }\n",
    );
    // entry re-exports a.
    write_file(root, "src/index.js", "export * from './lib/a';\n");

    let result = run_lib_build(root, vec![OutputFormat::Esm]);
    let code = &result.entries[0].code;

    assert!(
        code.contains("function gamma"),
        "transitively re-exported c's body must be inlined, got:\n{code}"
    );
    assert!(
        code.contains("function alpha"),
        "intermediate a's body must be inlined, got:\n{code}"
    );
    assert!(
        !code.contains("from \"./c") && !code.contains("from './c"),
        "no dangling re-export of ./c may survive, got:\n{code}"
    );
    assert!(
        !code.contains("from \"./lib/a") && !code.contains("from './lib/a"),
        "no dangling re-export of ./lib/a may survive, got:\n{code}"
    );
}

/// A barrel mixing internal (inlined) and external (bare) re-exports: the
/// relative `./lib/a` is inlined, while `export { x } from 'react'` stays a
/// bare external re-export.
#[test]
fn lib_barrel_keeps_external_reexport_bare() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{
            "name": "mixed-barrel",
            "version": "1.0.0",
            "module": "./src/index.js",
            "peerDependencies": { "react": "^18.0.0" }
        }"#,
    );
    write_file(
        root,
        "src/lib/a.js",
        "export function alpha() { return 1; }\n",
    );
    write_file(
        root,
        "src/index.js",
        "export * from './lib/a';\nexport { useState } from 'react';\n",
    );

    let result = run_lib_build(root, vec![OutputFormat::Esm]);
    let code = &result.entries[0].code;

    // Relative target inlined.
    assert!(
        code.contains("function alpha"),
        "relative ./lib/a must be inlined, got:\n{code}"
    );
    assert!(
        !code.contains("./lib/a"),
        "no dangling relative re-export, got:\n{code}"
    );
    // External re-export stays a bare `from "react"` re-export.
    assert!(
        code.contains("from \"react\"") || code.contains("from 'react'"),
        "external react re-export stays bare, got:\n{code}"
    );
}

/// CJS no-regression for inlined barrels: a `export * from './lib/a'` barrel,
/// once inlined into ESM, lowers to CJS that exposes a's symbol via `exports.*`
/// (NOT via a dangling `require("./lib/a.js")`).
#[test]
fn lib_barrel_inlined_cjs_exposes_symbol_without_dangling_require() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{
            "name": "barrel-cjs-lib",
            "version": "1.0.0",
            "module": "./src/index.js"
        }"#,
    );
    write_file(
        root,
        "src/lib/a.js",
        "export function alpha() { return 1; }\n",
    );
    write_file(root, "src/index.js", "export * from './lib/a';\n");

    let result = run_lib_build(root, vec![OutputFormat::Esm, OutputFormat::Cjs]);
    let cjs = cjs_code(&result);

    // The inlined ESM `export function alpha` lowers to a CJS `exports.alpha`.
    assert!(
        cjs.contains("exports.alpha = alpha;"),
        "inlined barrel export must surface as exports.alpha, got:\n{cjs}"
    );
    // No dangling relative require survives (the module was inlined, not kept).
    assert!(
        !cjs.contains("require(\"./lib/a"),
        "inlined barrel must not leave a dangling require(\"./lib/a.js\"), got:\n{cjs}"
    );
}

/// #170 entry resolution: package.json `exports`/`module`/`main` point at BUILD
/// OUTPUT (`./dist/index.js`), which does not exist as a source. The build must
/// fall back to the conventional `src/index.ts` rather than failing on the
/// not-yet-built dist path.
#[test]
fn lib_entry_falls_back_to_src_index_when_exports_point_at_dist() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    write_file(
        root,
        "package.json",
        r#"{
          "name": "conv-lib",
          "version": "1.0.0",
          "type": "module",
          "main": "./dist/index.cjs",
          "module": "./dist/index.js",
          "exports": { ".": { "import": "./dist/index.js", "require": "./dist/index.cjs" } }
        }"#,
    );
    write_file(
        root,
        "src/index.ts",
        "export const hello = (): string => \"hi\";\n",
    );

    let result = run_lib_build(root, vec![OutputFormat::Esm]);
    assert!(
        !result.entries.is_empty(),
        "convention fallback must yield an entry"
    );
    let esm = &result.entries[0].code;
    assert!(
        esm.contains("hello"),
        "built ESM must contain the source symbol, got:\n{esm}"
    );
}

/// #170 explicit `[lib].entry` (via `LibBuildOptions::entry`) selects the SOURCE
/// entry directly, overriding exports-based discovery.
#[test]
fn lib_explicit_entry_overrides_exports() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    write_file(
        root,
        "package.json",
        r#"{ "name": "explicit-lib", "version": "1.0.0", "type": "module",
            "exports": { ".": "./dist/index.js" } }"#,
    );
    write_file(
        root,
        "src/main.ts",
        "export const answer = (): number => 42;\n",
    );

    let options = LibBuildOptions {
        project_root: root.to_path_buf(),
        out_dir: root.join("dist-jet"),
        formats: vec![OutputFormat::Esm],
        conditions: vec!["import".to_string(), "default".to_string()],
        extra_externals: HashSet::new(),
        preserve_modules: false,
        declaration: false,
        library_global_name: None,
        entry: vec!["src/main.ts".to_string()],
        css_merge: Vec::new(),
        raw_copy: Vec::new(),
        sourcemap: SourceMapOption::None,
    };
    let result = build_library(options).expect("explicit-entry build must succeed");
    assert!(!result.entries.is_empty());
    assert!(
        result.entries[0].code.contains("answer"),
        "explicit entry src/main.ts must be the built source, got:\n{}",
        result.entries[0].code
    );
}

#[test]
fn lib_build_external_sourcemap_writes_map_and_url_comment() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    write_file(
        root,
        "package.json",
        r#"{ "name": "sourcemap-lib", "version": "1.0.0", "module": "./src/index.ts" }"#,
    );
    write_file(root, "src/index.ts", "export const answer: number = 42;\n");

    let options = LibBuildOptions {
        project_root: root.to_path_buf(),
        out_dir: root.join("dist"),
        formats: vec![OutputFormat::Esm],
        conditions: vec!["import".to_string(), "default".to_string()],
        extra_externals: HashSet::new(),
        preserve_modules: false,
        declaration: false,
        library_global_name: None,
        entry: Vec::new(),
        css_merge: Vec::new(),
        raw_copy: Vec::new(),
        sourcemap: SourceMapOption::External,
    };
    let result = build_library(options).expect("sourcemap library build must succeed");
    let js = root.join("dist/index.js");
    let map = root.join("dist/index.js.map");

    assert!(js.is_file(), "JS output exists");
    assert!(map.is_file(), "external source map must be written");
    assert!(
        result.entries[0]
            .code
            .contains("//# sourceMappingURL=index.js.map"),
        "JS output must point at the external map, got:\n{}",
        result.entries[0].code
    );
    let map_json = std::fs::read_to_string(map).unwrap();
    assert!(
        map_json.contains("\"sources\":[\"src/index.ts\"]"),
        "source map should point at the original library entry, got:\n{map_json}"
    );
}
// HANDWRITE-END
