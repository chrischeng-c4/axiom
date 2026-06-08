interface ItemsListProps {
  items: number[];
}

export function ItemsList({ items }: ItemsListProps) {
  return (
    <div id="root">
      {items.map((x) => (
        <span id="item">item {x}</span>
      ))}
    </div>
  );
}
