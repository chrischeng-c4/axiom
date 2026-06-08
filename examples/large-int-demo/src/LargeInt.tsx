interface LargeIntProps {
  start: number;
}

export function LargeInt({ start }: LargeIntProps) {
  const [n, setN] = useState(start);
  return (
    <button id="bump" onClick={() => setN(n + 1)}>
      n={n}
    </button>
  );
}
