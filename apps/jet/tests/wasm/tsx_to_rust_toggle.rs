// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Transpiler coverage for the v1 rules added on top of Counter:
//! - Nested JSX elements as children.
//! - Self-closing elements (`<img id="icon" />`).
//! - Conditional rendering (`{on && <span>...</span>}`).
//! - `bool` prop type → `use_state::<bool>` turbofish.
//! - Unary `!` in expression context.

use jet::tsx_to_rust::transpile;

const TOGGLE_TSX: &str = include_str!("../fixtures/tsx_to_rust_toggle.tsx");

#[test]
fn toggle_transpiles_without_error() {
    let out = transpile(TOGGLE_TSX).expect("transpile");
    eprintln!("\n=== GENERATED ===\n{out}=================\n");
}

#[test]
fn bool_prop_yields_bool_turbofish() {
    let out = transpile(TOGGLE_TSX).unwrap();
    assert!(
        out.contains("use_state::<bool>(props.initial)"),
        "boolean prop should yield bool turbofish.\nGENERATED:\n{out}"
    );
    assert!(
        out.contains("pub initial: bool"),
        "props struct should use `bool` not `i64`.\nGENERATED:\n{out}"
    );
    assert!(
        out.contains("pub fn toggle(initial: bool)"),
        "factory arg type should be bool.\nGENERATED:\n{out}"
    );
}

#[test]
fn unary_not_in_setter_call() {
    let out = transpile(TOGGLE_TSX).unwrap();
    assert!(
        out.contains("setOn.set((!on))"),
        "unary ! should lower to Rust ! with parens.\nGENERATED:\n{out}"
    );
}

#[test]
fn nested_jsx_produces_nested_intrinsics() {
    let out = transpile(TOGGLE_TSX).unwrap();
    // <div> should contain Element::intrinsic children.
    assert!(
        out.contains("Element::intrinsic(\"div\""),
        "root div missing.\nGENERATED:\n{out}"
    );
    assert!(
        out.contains("Element::intrinsic(\"button\""),
        "nested button missing.\nGENERATED:\n{out}"
    );
}

#[test]
fn self_closing_emits_empty_children() {
    let out = transpile(TOGGLE_TSX).unwrap();
    assert!(
        out.contains("Element::intrinsic(\"img\""),
        "<img/> should lower to Element::intrinsic.\nGENERATED:\n{out}"
    );
    // The img has no children.
    assert!(
        out.contains("Element::intrinsic(\"img\", Props { id: Some(\"icon\".to_string()), ..Default::default() }, vec![])"),
        "self-closing img should emit empty children vec.\nGENERATED:\n{out}"
    );
}

#[test]
fn conditional_render_uses_if_else() {
    let out = transpile(TOGGLE_TSX).unwrap();
    // `{on && <span id="indicator">on</span>}` should lower to
    // `if on { Element::intrinsic(...) } else { Element::Empty }`.
    assert!(
        out.contains("if on"),
        "conditional should emit `if cond`.\nGENERATED:\n{out}"
    );
    assert!(
        out.contains("Element::Empty"),
        "conditional else branch should be Element::Empty.\nGENERATED:\n{out}"
    );
    assert!(
        out.contains("Element::intrinsic(\"span\""),
        "then-branch span should be present.\nGENERATED:\n{out}"
    );
}
// CODEGEN-END
