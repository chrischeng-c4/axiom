// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Regression coverage for #1404: the TSX→Rust transpiler used to
//! lower `useState(false)` as `use_state::<i64>(false)`, which broke
//! the WASM build. The fix maps boolean / numeric / string literal
//! initializers to their matching Rust primitive turbofish.
//!
//! Spec: projects/jet/docs/wasm-transpiler-boolean-usestate-literals.md

use jet::tsx_to_rust::transpile;

const FIXTURE: &str = include_str!("fixtures/tsx_to_rust_boolean_literal_state.tsx");

#[test]
fn boolean_literal_initializer_yields_bool_turbofish() {
    let out = transpile(FIXTURE).expect("transpile");
    assert!(
        out.contains("use_state::<bool>(false)"),
        "useState(false) should yield bool turbofish.\nGENERATED:\n{out}"
    );
}

#[test]
fn numeric_literal_initializer_preserves_i64_default() {
    let out = transpile(FIXTURE).unwrap();
    assert!(
        out.contains("use_state::<i64>(0)"),
        "useState(0) should keep the i64 default.\nGENERATED:\n{out}"
    );
}

#[test]
fn string_literal_initializer_yields_string_turbofish() {
    let out = transpile(FIXTURE).unwrap();
    assert!(
        out.contains("use_state::<String>(\"anon\".to_string())"),
        "useState(\"anon\") should yield String turbofish.\nGENERATED:\n{out}"
    );
}

#[test]
fn boolean_state_drives_conditional_render() {
    let out = transpile(FIXTURE).unwrap();
    // `{on && <span id="indicator">on</span>}` must lower to
    // `if on { ... } else { Element::Empty }`. This is what would
    // have been a type error under the old `i64` lowering.
    assert!(
        out.contains("if on"),
        "conditional should emit `if on`.\nGENERATED:\n{out}"
    );
    assert!(
        out.contains("Element::Empty"),
        "conditional else branch must be Element::Empty.\nGENERATED:\n{out}"
    );
}

#[test]
fn boolean_state_setter_lowers_unary_not() {
    let out = transpile(FIXTURE).unwrap();
    assert!(
        out.contains("setOn.set((!on))"),
        "setOn(!on) should lower to `setOn.set((!on))`.\nGENERATED:\n{out}"
    );
}
// CODEGEN-END
