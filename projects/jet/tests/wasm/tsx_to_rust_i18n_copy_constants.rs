// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Regression coverage for #1409: the TSX→Rust transpiler used to
//! reject readonly copy constants at the top level (`lexical_declaration
//! outside the spike subset`) and inside components (`binding RHS must
//! be a hook call`). Cue's i18n boundary requires both forms so the
//! WASM shell can share the regular React shell's copy dictionary
//! instead of inlining literals.
//!
//! @issue #1409

use jet::tsx_to_rust::transpile;

const FIXTURE: &str = include_str!("../fixtures/tsx_to_rust_i18n_copy_constants.tsx");

#[test]
fn top_level_string_const_passes_through() {
    let out = transpile(FIXTURE).expect("transpile");
    assert!(
        out.contains("const GREETING: &str = \"hello\";"),
        "top-level string const should become a Rust `const`.\nGENERATED:\n{out}"
    );
}

#[test]
fn top_level_object_const_passes_through() {
    let out = transpile(FIXTURE).expect("transpile");
    // Object-literal copy dicts lower to a generated struct + const
    // instance so that JSX member-expression access (`COPY.title`)
    // keeps the same dot syntax.
    assert!(
        out.contains("const COPY:"),
        "top-level object const should bind `const COPY: …`.\nGENERATED:\n{out}"
    );
    assert!(
        out.contains("title: \"title\""),
        "object const should preserve string-keyed entries.\nGENERATED:\n{out}"
    );
    assert!(
        out.contains("description: \"description\""),
        "object const should preserve every entry.\nGENERATED:\n{out}"
    );
}

#[test]
fn in_component_string_const_lowers_to_let() {
    let out = transpile(FIXTURE).expect("transpile");
    // In-component `const X = "literal"` should lower to an owned Rust
    // `String` inside the render fn — NOT trip the "binding RHS must be
    // a hook call" guard, and closures can capture it without no-op
    // cloning a borrowed `&str`.
    assert!(
        out.contains("let HEADER = \"welcome\".to_string();"),
        "in-component string const should lower to a Rust let.\nGENERATED:\n{out}"
    );
}

#[test]
fn member_access_in_jsx_resolves() {
    let out = transpile(FIXTURE).expect("transpile");
    // `{COPY.title}` is a member expression on a const struct;
    // we already lower it via `transpile_expr`, but the regression
    // here is that the surrounding top-level const must be accepted
    // so the program compiles at all.
    assert!(
        out.contains("COPY.title"),
        "JSX interp of `COPY.title` should survive lowering.\nGENERATED:\n{out}"
    );
}
// CODEGEN-END
