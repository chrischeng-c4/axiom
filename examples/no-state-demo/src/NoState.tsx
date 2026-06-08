interface NoStateProps {
  value: number;
}

export function NoState({ value }: NoStateProps) {
  return (
    <div id="root">
      <span id="label">value: {value}</span>
    </div>
  );
}
