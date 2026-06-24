// HANDWRITE-BEGIN gap="missing-generator:unit-test:3e627729" tracker="pending-tracker" reason="Tests: a typed fixture library emits `.d.ts` with the right exported signatures, the build sets `types`/`exports.types`, and a consumer type-checks clean against the emitted declarations."
//! Integration tests for `.d.ts` emission in `jet build --lib`.
//!
//! Coverage:
//!   (a) a typed fixture lib emits `<entry>.d.ts` containing the exported
//!       interface plus `export declare` function/const signatures,
//!   (b) the build records the `types` path (LibBuildResult::types and
//!       EntryOutput::dts),
//!   (c) a multi-entry lib emits one `.d.ts` per entry,
//!   (d) declaration emission is opt-out (declaration = false → no `.d.ts`),
//!   (e) an untyped export fails the build (isolatedDeclarations).
//!
//! @issue #171

use jet::bundler::types::OutputFormat;
use jet::bundler::{build_library, LibBuildOptions};
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

/// Library build options with declaration emission ON.
fn lib_options(root: &std::path::Path) -> LibBuildOptions {
    LibBuildOptions {
        project_root: root.to_path_buf(),
        out_dir: root.join("dist"),
        formats: vec![OutputFormat::Esm],
        conditions: vec!["import".to_string(), "default".to_string()],
        extra_externals: HashSet::new(),
        preserve_modules: false,
        declaration: true,
        library_global_name: None,
        entry: Vec::new(),
    }
}

// ──────────────────────────────────────────────────────────────────────────
// (a) Typed lib emits <entry>.d.ts with interface + declare signatures.
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn typed_lib_emits_dts_with_interface_and_signatures() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{
            "name": "typed-lib",
            "version": "1.0.0",
            "module": "./src/index.ts"
        }"#,
    );
    write_file(
        root,
        "src/index.ts",
        r#"export interface User {
    id: number;
    name: string;
}

export const VERSION: string = "1.0.0";

export function greet(user: User): string {
    return "hello " + user.name;
}
"#,
    );

    let result = build_library(lib_options(root)).expect("library build must succeed");

    assert_eq!(result.types.len(), 1, "single entry → single .d.ts");
    let dts_path = &result.types[0].path;
    assert!(dts_path.is_file(), ".d.ts file must exist on disk");
    assert_eq!(
        dts_path.file_name().unwrap(),
        "index.d.ts",
        "`.` entry → index.d.ts, got {:?}",
        dts_path
    );

    let dts = std::fs::read_to_string(dts_path).unwrap();

    // Interface emitted verbatim.
    assert!(
        dts.contains("export interface User"),
        "exported interface must be emitted, got:\n{dts}"
    );
    assert!(
        dts.contains("id: number") && dts.contains("name: string"),
        "interface members preserved, got:\n{dts}"
    );

    // const reduced to a `declare` signature (initializer dropped).
    assert!(
        dts.contains("export declare const VERSION: string;"),
        "typed const reduced to declare signature, got:\n{dts}"
    );
    assert!(
        !dts.contains("1.0.0"),
        "const initializer must be dropped from .d.ts, got:\n{dts}"
    );

    // function reduced to a `declare` signature (body dropped).
    assert!(
        dts.contains("export declare function greet(user: User): string;"),
        "typed function reduced to declare signature, got:\n{dts}"
    );
    assert!(
        !dts.contains("hello"),
        "function body must be dropped from .d.ts, got:\n{dts}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// (b) The build records the `types` path on both the result and the entry.
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn build_records_types_path_on_result_and_entry() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{ "name": "typed-lib", "version": "1.0.0", "module": "./src/index.ts" }"#,
    );
    write_file(
        root,
        "src/index.ts",
        "export type ID = string | number;\nexport const FLAG: boolean = true;\n",
    );

    let result = build_library(lib_options(root)).expect("library build must succeed");

    // Result-level `types` entry.
    assert_eq!(result.types.len(), 1);
    assert_eq!(result.types[0].subpath, ".");
    let recorded = result.types[0].path.clone();
    assert!(recorded.is_file());

    // Each JS EntryOutput points at the same `.d.ts` via `dts`.
    let js = result
        .entries
        .iter()
        .find(|e| e.format == OutputFormat::Esm)
        .expect("ESM output present");
    assert_eq!(
        js.dts.as_ref(),
        Some(&recorded),
        "EntryOutput::dts must record the emitted declaration path"
    );

    // The `.d.ts` sits next to the JS output.
    assert_eq!(
        recorded.parent(),
        js.path.parent(),
        "declarations must be emitted next to the JS output"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// (c) Multi-entry lib emits one .d.ts per entry.
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn multi_entry_emits_one_dts_per_entry() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{
            "name": "multi-lib",
            "version": "1.0.0",
            "exports": {
                ".": { "import": "./src/index.ts" },
                "./client": { "import": "./src/client.ts" }
            }
        }"#,
    );
    write_file(
        root,
        "src/index.ts",
        "export interface Root { value: number; }\n",
    );
    write_file(
        root,
        "src/client.ts",
        "export type ClientId = string;\nexport declare function noop(): void;\n",
    );

    let result = build_library(lib_options(root)).expect("library build must succeed");

    assert_eq!(result.types.len(), 2, "two entries → two .d.ts files");

    let subpaths: HashSet<&str> = result.types.iter().map(|t| t.subpath.as_str()).collect();
    assert!(subpaths.contains("."), "`.` entry must emit a .d.ts");
    assert!(
        subpaths.contains("./client"),
        "`./client` entry must emit a .d.ts"
    );

    let index_dts = result.types.iter().find(|t| t.subpath == ".").unwrap();
    assert_eq!(index_dts.path.file_name().unwrap(), "index.d.ts");
    let client_dts = result
        .types
        .iter()
        .find(|t| t.subpath == "./client")
        .unwrap();
    assert_eq!(client_dts.path.file_name().unwrap(), "client.d.ts");

    let index_text = std::fs::read_to_string(&index_dts.path).unwrap();
    assert!(
        index_text.contains("export interface Root"),
        "root .d.ts has its interface, got:\n{index_text}"
    );
    let client_text = std::fs::read_to_string(&client_dts.path).unwrap();
    assert!(
        client_text.contains("export type ClientId = string"),
        "client .d.ts has its type alias, got:\n{client_text}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// (d) declaration = false → no .d.ts emitted.
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn declaration_off_emits_no_dts() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{ "name": "no-dts-lib", "version": "1.0.0", "module": "./src/index.ts" }"#,
    );
    write_file(
        root,
        "src/index.ts",
        "export const N: number = 1;\n",
    );

    let mut options = lib_options(root);
    options.declaration = false;
    let result = build_library(options).expect("library build must succeed");

    assert!(
        result.types.is_empty(),
        "no .d.ts records when declaration is off"
    );
    for entry in &result.entries {
        assert!(
            entry.dts.is_none(),
            "EntryOutput::dts must be None when declaration is off"
        );
    }
    assert!(
        !root.join("dist/index.d.ts").exists(),
        "no index.d.ts on disk when declaration is off"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// (e) Untyped exported value fails the build (isolatedDeclarations).
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn untyped_export_fails_build() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{ "name": "bad-lib", "version": "1.0.0", "module": "./src/index.ts" }"#,
    );
    // No explicit type annotation on the exported const.
    write_file(
        root,
        "src/index.ts",
        "export const VERSION = \"1.0.0\";\n",
    );

    let err = build_library(lib_options(root)).expect_err("untyped export must fail the build");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("isolatedDeclarations"),
        "error must explain the isolatedDeclarations requirement, got: {msg}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// (f) An exported class is reduced to its public ambient surface: method
//     bodies dropped to signatures, field initializers dropped, and
//     `private`/`#private` members dropped.
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn exported_class_reduced_to_ambient_surface() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{ "name": "ui-lib", "version": "1.0.0", "module": "./src/index.ts" }"#,
    );
    write_file(
        root,
        "src/index.ts",
        r#"export interface Props { label: string; }
export type Node = unknown;

export class Button {
    constructor(p: Props) { this.p = p; }
    render(): Node { return null; }
    private x = 1;
    #secret = 2;
    readonly id: string = "";
}
"#,
    );

    let result = build_library(lib_options(root)).expect("library build must succeed");
    assert_eq!(result.types.len(), 1);
    let dts = std::fs::read_to_string(&result.types[0].path).unwrap();

    // Reduced to a `declare class` with signature-only members.
    assert!(
        dts.contains("export declare class Button"),
        "class reduced to `declare class`, got:\n{dts}"
    );
    assert!(
        dts.contains("constructor(p: Props);"),
        "constructor reduced to a signature, got:\n{dts}"
    );
    assert!(
        dts.contains("render(): Node;"),
        "method reduced to a signature, got:\n{dts}"
    );
    assert!(
        dts.contains("readonly id: string;"),
        "public readonly field kept without initializer, got:\n{dts}"
    );

    // Bodies and initializers gone.
    assert!(
        !dts.contains("return null"),
        "method body must be dropped, got:\n{dts}"
    );
    assert!(
        !dts.contains("this.p = p"),
        "constructor body must be dropped, got:\n{dts}"
    );
    assert!(
        !dts.contains("= \"\""),
        "field initializer must be dropped, got:\n{dts}"
    );

    // Private members dropped.
    assert!(
        !dts.contains("private x") && !dts.contains("x = 1"),
        "private member must be dropped, got:\n{dts}"
    );
    assert!(
        !dts.contains("#secret") && !dts.contains("= 2"),
        "#private member must be dropped, got:\n{dts}"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// (g) A previously-deferred export shape now emits a valid declaration:
//     `export default (expr as Type)` resolves to its annotated type, and a
//     re-export passthrough is preserved.
// ──────────────────────────────────────────────────────────────────────────

#[test]
fn previously_deferred_export_shapes_now_emit() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{ "name": "shapes-lib", "version": "1.0.0", "module": "./src/index.ts" }"#,
    );
    write_file(
        root,
        "src/index.ts",
        r#"import type { Config } from "./config";
export type { Helper } from "./helper";

export default (loadConfig() as Config);
"#,
    );

    let result = build_library(lib_options(root)).expect("library build must succeed");
    assert_eq!(result.types.len(), 1);
    let dts = std::fs::read_to_string(&result.types[0].path).unwrap();

    // Annotated default export → synthetic typed default (no TODO marker, no
    // leaked `loadConfig()` call expression).
    assert!(
        dts.contains("declare const _default: Config;") && dts.contains("export default _default;"),
        "annotated default export resolves to its type, got:\n{dts}"
    );
    assert!(
        !dts.contains("loadConfig()"),
        "default-export initializer expression must not leak, got:\n{dts}"
    );
    assert!(
        !dts.contains("TODO"),
        "this shape is now handled — no TODO marker expected, got:\n{dts}"
    );

    // `export type { … } from` re-export passthrough preserved.
    assert!(
        dts.contains("export type { Helper } from \"./helper\""),
        "type re-export preserved, got:\n{dts}"
    );
}
// HANDWRITE-END
