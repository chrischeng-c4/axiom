interface MultiHandlerProps {
  initial: number;
}

export function MultiHandler({ initial }: MultiHandlerProps) {
  const [a, setA] = useState(initial);
  const [b, setB] = useState(initial);
  return (
    <div id="root">
      <button id="bump-a" onClick={() => setA(a + 1)}>
        a: {a}
      </button>
      <button id="bump-b" onClick={() => setB(b + 10)}>
        b: {b}
      </button>
    </div>
  );
}
