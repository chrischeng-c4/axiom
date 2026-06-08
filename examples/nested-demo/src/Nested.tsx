interface NestedProps {
  initial: number;
}

export function Nested({ initial }: NestedProps) {
  const [n, setN] = useState(initial);
  return (
    <div id="outer">
      <div id="middle">
        <button id="inner" onClick={() => setN(n + 1)}>
          <span id="label">count: {n}</span>
        </button>
      </div>
    </div>
  );
}
