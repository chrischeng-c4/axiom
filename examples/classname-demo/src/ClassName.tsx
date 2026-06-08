interface ClassNameProps {
  initial: number;
}

export function ClassName({ initial }: ClassNameProps) {
  const [n, setN] = useState(initial);
  return (
    <button className="primary" id="cta" onClick={() => setN(n + 1)}>
      click me: {n}
    </button>
  );
}
