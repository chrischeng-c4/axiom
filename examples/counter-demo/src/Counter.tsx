interface CounterProps {
  start: number;
}

export function Counter({ start }: CounterProps) {
  const [n, setN] = useState(start);
  return (
    <button id="inc" onClick={() => setN(n + 1)}>
      count: {n}
    </button>
  );
}
