interface StringStateProps {
  initial: string;
}

export function StringState({ initial }: StringStateProps) {
  const [s, setS] = useState(initial);
  return (
    <button id="view" onClick={() => setS(s)}>
      value: {s}
    </button>
  );
}
