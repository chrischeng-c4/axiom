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
//! @issue #722
//! @issue #784
//! @issue #795
//! @issue #796

use jet::bundler::types::OutputFormat;
use jet::bundler::types::SourceMapOption;
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
        css_merge: Vec::new(),
        raw_copy: Vec::new(),
        sourcemap: SourceMapOption::None,
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

#[test]
fn plain_object_literal_const_export_emits_dts() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{ "name": "object-literal-lib", "version": "1.0.0", "module": "./src/index.ts" }"#,
    );
    write_file(
        root,
        "src/index.ts",
        r#"export const UPLOAD_ACCEPT_TYPE = {
    JPG: "image/jpeg",
    PNG: "image/png",
    PDF: "application/pdf",
};
"#,
    );

    let result = build_library(lib_options(root)).expect("object literal export must build");
    let dts = std::fs::read_to_string(&result.types[0].path).unwrap();

    assert!(
        dts.contains("export declare const UPLOAD_ACCEPT_TYPE: {"),
        "plain object literal export must synthesize an object type, got:\n{dts}"
    );
    for expected in ["JPG: string;", "PNG: string;", "PDF: string;"] {
        assert!(
            dts.contains(expected),
            "object property {expected:?} should be emitted, got:\n{dts}"
        );
    }
}

#[test]
fn stale_default_dist_index_does_not_control_dts_emit_with_custom_out_dir() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{ "name": "stale-dist-lib", "version": "1.0.0", "module": "./dist/index.js" }"#,
    );
    write_file(
        root,
        "src/index.ts",
        r#"export const SOURCE_VALUE: string = "from-source";
"#,
    );
    write_file(root, "dist/index.js", "not valid js from stale dist\n");

    let mut options = lib_options(root);
    options.out_dir = root.join("custom-out");
    let result = build_library(options).expect("stale default dist must not affect dts emit");

    let dts_path = root.join("custom-out/index.d.ts");
    assert_eq!(result.types[0].path, dts_path);
    assert!(
        dts_path.is_file(),
        "declaration should be written to the explicit output directory"
    );
    let dts = std::fs::read_to_string(&dts_path).unwrap();
    assert!(
        dts.contains("export declare const SOURCE_VALUE: string;"),
        "declaration must come from src/index.ts, got:\n{dts}"
    );
    assert!(
        !dts.is_empty() && !root.join("dist/index.d.ts").exists(),
        "stale default dist must not become declaration input/output"
    );

    let js = result
        .entries
        .iter()
        .find(|entry| entry.format == OutputFormat::Esm)
        .expect("ESM output present");
    assert!(
        js.code.contains("SOURCE_VALUE") && !js.code.contains("not valid js"),
        "runtime output must also be source-derived, got:\n{}",
        js.code
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

#[test]
fn barrel_reexports_emit_sibling_declaration_files() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{
            "name": "barrel-lib",
            "version": "1.0.0",
            "module": "./src/index.ts"
        }"#,
    );
    write_file(
        root,
        "src/index.ts",
        r#"export * from "./math";
export * from "./greeter";
"#,
    );
    write_file(
        root,
        "src/math.ts",
        "export function add(a: number, b: number) { return a + b; }\n",
    );
    write_file(
        root,
        "src/greeter.ts",
        r#"export class Greeter {
    greet(name: string) { return `hi ${name}`; }
}
"#,
    );

    let result = build_library(lib_options(root)).expect("library build must succeed");

    let index_dts = root.join("dist/index.d.ts");
    let math_dts = root.join("dist/math.d.ts");
    let greeter_dts = root.join("dist/greeter.d.ts");

    assert_eq!(
        result.types.len(),
        1,
        "public type outputs still track entry declarations"
    );
    assert_eq!(result.types[0].path, index_dts);
    assert!(index_dts.is_file(), "entry declaration must exist");
    assert!(
        math_dts.is_file(),
        "barrel re-export target declaration must exist"
    );
    assert!(
        greeter_dts.is_file(),
        "barrel re-export target declaration must exist"
    );

    let index_text = std::fs::read_to_string(index_dts).unwrap();
    assert!(
        index_text.contains("export * from \"./math\"")
            && index_text.contains("export * from \"./greeter\""),
        "entry declaration must preserve barrel re-exports, got:\n{index_text}"
    );

    let math_text = std::fs::read_to_string(math_dts).unwrap();
    assert!(
        math_text.contains("export declare function add(a: number, b: number): number;"),
        "sibling declaration must preserve typed function signature, got:\n{math_text}"
    );

    let greeter_text = std::fs::read_to_string(greeter_dts).unwrap();
    assert!(
        greeter_text.contains("export declare class Greeter")
            && greeter_text.contains("greet(name: string): string;"),
        "sibling declaration must preserve typed class member, got:\n{greeter_text}"
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
    write_file(root, "src/index.ts", "export const N: number = 1;\n");

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
// (e) Untyped exported values still fail, but locally inferable exported
//     function/member returns emit tsc-like declarations.
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
    write_file(root, "src/index.ts", "export const VERSION = \"1.0.0\";\n");

    let err = build_library(lib_options(root)).expect_err("untyped export must fail the build");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("isolatedDeclarations"),
        "error must explain the isolatedDeclarations requirement, got: {msg}"
    );
}

#[test]
fn dts_isolated_declarations_errors_are_aggregated() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{ "name": "aggregate-errors-lib", "version": "1.0.0", "module": "./src/index.ts" }"#,
    );
    write_file(
        root,
        "src/index.ts",
        r#"export const VERSION = "1.0.0";

export function makeThing() {
    return createThing();
}

export { Widget } from "./widget";
"#,
    );
    write_file(
        root,
        "src/widget.ts",
        r#"export class Widget {
    load() {
        return fetchWidget();
    }

    count = 1;
}
"#,
    );

    let err = build_library(lib_options(root))
        .expect_err("all isolatedDeclarations errors should be reported together");
    let msg = format!("{err:#}");

    for expected in [
        "src/index.ts",
        "src/widget.ts",
        "VERSION",
        "makeThing",
        "load",
        "count",
    ] {
        assert!(
            msg.contains(expected),
            "aggregated error must include {expected:?}, got:\n{msg}"
        );
    }
    assert!(
        msg.contains("4 error(s)"),
        "error count should include every invalid export, got:\n{msg}"
    );
    assert!(
        !root.join("dist/index.d.ts").exists(),
        "entry declaration must not be written after aggregate failure"
    );
    assert!(
        !root.join("dist/widget.d.ts").exists(),
        "re-export target declaration must not be written after aggregate failure"
    );
}

#[test]
fn exported_function_infers_number_return_type() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{ "name": "bad-function-lib", "version": "1.0.0", "module": "./src/index.ts" }"#,
    );
    write_file(
        root,
        "src/index.ts",
        "export function add(a: number, b: number) { return a + b; }\n",
    );

    let result = build_library(lib_options(root)).expect("library build must succeed");
    let dts = std::fs::read_to_string(&result.types[0].path).unwrap();
    assert!(
        dts.contains("export declare function add(a: number, b: number): number;"),
        "inferred numeric return type must be emitted, got:\n{dts}"
    );
}

#[test]
fn exported_class_member_infers_string_return_type() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    write_file(
        root,
        "package.json",
        r#"{ "name": "bad-class-lib", "version": "1.0.0", "module": "./src/index.ts" }"#,
    );
    write_file(
        root,
        "src/index.ts",
        r#"export class Greeter {
    greet(name: string) { return `hi ${name}`; }
}
"#,
    );

    let result = build_library(lib_options(root)).expect("library build must succeed");
    let dts = std::fs::read_to_string(&result.types[0].path).unwrap();
    assert!(
        dts.contains("export declare class Greeter")
            && dts.contains("greet(name: string): string;"),
        "inferred string member return type must be emitted, got:\n{dts}"
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
