interface UseMemoProps {
  initial: number;
}

export function UseMemo({ initial }: UseMemoProps) {
  const [n, setN] = useState(initial);
  const doubled = useMemo(() => n * 2, [n]);
  return (
    <button id="bump" onClick={() => setN(n + 1)}>
      n={n} doubled={doubled}
    </button>
  );
}
