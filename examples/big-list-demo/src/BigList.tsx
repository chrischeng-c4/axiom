interface BigListProps {
  items: number[];
}

export function BigList({ items }: BigListProps) {
  return (
    <div id="root">
      {items.map((x) => (
        <span id="item">item {x}</span>
      ))}
    </div>
  );
}
