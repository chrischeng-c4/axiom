// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! End-to-end spike test: TSX → transpile → inspect output → verify
//! the emitted Rust contains the expected API calls.
//!
//! We don't compile the emitted Rust into a child cargo (too heavy
//! for a spike) — we check structural content against known markers
//! from `jet-wasm`. A follow-up test will bundle the
//! generated code into an actual cargo target once the transpiler
//! covers enough surface to make that worthwhile.

use jet::tsx_to_rust::transpile;

const COUNTER_TSX: &str = include_str!("fixtures/tsx_to_rust_counter.tsx");

#[test]
fn counter_transpiles_without_error() {
    let out = transpile(COUNTER_TSX).expect("transpile failed");
    // Show the generated code on stderr so --nocapture displays it.
    eprintln!("\n=== GENERATED ===\n{out}=================\n");
}

#[test]
fn generated_has_props_struct() {
    let out = transpile(COUNTER_TSX).unwrap();
    assert!(
        out.contains("pub struct CounterProps {"),
        "missing CounterProps struct.\nGENERATED:\n{out}"
    );
    assert!(
        out.contains("pub start: i64"),
        "missing typed `start` field.\nGENERATED:\n{out}"
    );
}

#[test]
fn generated_has_render_fn() {
    let out = transpile(COUNTER_TSX).unwrap();
    assert!(
        out.contains("fn counter_render(props:"),
        "missing render fn.\nGENERATED:\n{out}"
    );
    assert!(
        out.contains("props.downcast_ref"),
        "render fn should downcast props.\nGENERATED:\n{out}"
    );
}

#[test]
fn generated_calls_use_state_with_i64() {
    let out = transpile(COUNTER_TSX).unwrap();
    assert!(
        out.contains("use_state::<i64>(props.start)"),
        "useState binding should resolve `start` prop and turbofish i64.\nGENERATED:\n{out}"
    );
    assert!(
        out.contains("let (n, setN) = use_state"),
        "useState destructure should preserve names.\nGENERATED:\n{out}"
    );
}

#[test]
fn generated_emits_jsx_as_element_intrinsic() {
    let out = transpile(COUNTER_TSX).unwrap();
    assert!(
        out.contains("Element::intrinsic(\"button\""),
        "JSX button should lower to Element::intrinsic.\nGENERATED:\n{out}"
    );
    assert!(
        out.contains("id: Some("),
        "id attribute should land on Props.id.\nGENERATED:\n{out}"
    );
    assert!(
        out.contains("on_click: Some(Callback::new("),
        "onClick should lower to Callback::new.\nGENERATED:\n{out}"
    );
    assert!(
        out.contains("setN.set("),
        "setter call should route through StateSetter::set.\nGENERATED:\n{out}"
    );
}

#[test]
fn generated_emits_text_and_interpolation() {
    let out = transpile(COUNTER_TSX).unwrap();
    assert!(
        out.contains("Element::text(\"count: \")"),
        "static text should land as Element::text.\nGENERATED:\n{out}"
    );
    assert!(
        out.contains("Element::text(n.to_string())"),
        "{{n}} interpolation should lower to text content.\nGENERATED:\n{out}"
    );
}

#[test]
fn generated_has_factory_fn() {
    let out = transpile(COUNTER_TSX).unwrap();
    assert!(
        out.contains("pub fn counter(start: i64) -> Component {"),
        "factory fn signature missing.\nGENERATED:\n{out}"
    );
    assert!(
        out.contains("Rc::new(CounterProps { start })"),
        "factory should build CounterProps.\nGENERATED:\n{out}"
    );
}

#[test]
fn out_of_subset_fails_loudly() {
    let bad_tsx = "export class Thing extends Component {}";
    let err = transpile(bad_tsx).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("outside the spike subset") || msg.contains("outside the spike"),
        "unclear error: {msg}"
    );
    assert!(
        msg.contains(":"),
        "error should include source position: {msg}"
    );
}
// CODEGEN-END
