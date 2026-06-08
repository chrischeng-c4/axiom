interface ListRenderProps {
  initial: number;
}

export function ListRender({ initial }: ListRenderProps) {
  const [n, setN] = useState(initial);
  return (
    <div id="root">
      <button id="add" onClick={() => setN(n + 1)}>
        add
      </button>
      {[...Array(n)].map((_, i) => (
        <span id="item">item {i}</span>
      ))}
    </div>
  );
}
