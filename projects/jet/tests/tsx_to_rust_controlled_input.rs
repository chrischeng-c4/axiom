// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Controlled input TSX lowering coverage for the WASM input surface.

use jet::tsx_to_rust::transpile;

const CONTROLLED_INPUT_TSX: &str = r#"
interface ControlledInputProps {
  initial: string;
}

export function ControlledInput({ initial }: ControlledInputProps) {
  const [name, setName] = useState(initial);
  return (
    <form id="form">
      <input
        id="name"
        value={name}
        placeholder="Name"
        onChange={(event) => setName(event.target.value)}
      />
      <span id="echo">hello {name}</span>
    </form>
  );
}
"#;

/// @spec .aw/tech-design/projects/jet/specs/4004.md#unit-test
#[test]
fn controlled_input_lowers_value_placeholder_and_on_change() {
    let out = transpile(CONTROLLED_INPUT_TSX).expect("transpile controlled input");

    assert!(
        out.contains("value: Some(name.clone()),"),
        "controlled input value must clone the state binding.\nGENERATED:\n{out}"
    );
    assert!(
        out.contains(r#"placeholder: Some("Name".to_string()),"#),
        "placeholder must lower to Props.placeholder.\nGENERATED:\n{out}"
    );
    assert!(
        out.contains(
            "on_change: Some(Callback::new({ let setName = setName.clone(); move |value| setName.set(value) })),"
        ),
        "onChange must lower event.target.value to the String payload callback.\nGENERATED:\n{out}"
    );
}
// CODEGEN-END
