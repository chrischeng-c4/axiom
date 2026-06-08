interface SelfClosingProps {
  initial: number;
}

export function SelfClosing({ initial }: SelfClosingProps) {
  const [n, setN] = useState(initial);
  return (
    <div id="root">
      <img id="icon" />
      <button id="bump" onClick={() => setN(n + 1)}>
        bump: {n}
      </button>
      <img id="after" />
    </div>
  );
}
