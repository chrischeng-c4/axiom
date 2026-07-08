// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests-fixtures.md#component
// CODEGEN-BEGIN
interface SandboxProps {
  label: string;
}

export function Sandbox({ label }: SandboxProps) {
  const [on, setOn] = useState(false);
  const [count, setCount] = useState(0);
  const [name, setName] = useState("anon");
  return (
    <div id="root">
      <button id="flip" onClick={() => setOn(!on)}>
        toggle
      </button>
      {on && <span id="indicator">on</span>}
      <span id="count">count</span>
      <span id="name">name</span>
    </div>
  );
}
// CODEGEN-END
