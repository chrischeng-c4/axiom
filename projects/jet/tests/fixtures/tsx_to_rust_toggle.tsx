// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests-fixtures.md#component
// CODEGEN-BEGIN
interface ToggleProps {
  initial: boolean;
}

export function Toggle({ initial }: ToggleProps) {
  const [on, setOn] = useState(initial);
  return (
    <div id="root">
      <button id="flip" onClick={() => setOn(!on)}>
        toggle
      </button>
      {on && <span id="indicator">on</span>}
      <img id="icon" />
    </div>
  );
}
// CODEGEN-END
