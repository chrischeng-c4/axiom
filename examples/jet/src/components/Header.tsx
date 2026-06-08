interface HeaderProps {
  title: string;
  count: number;
}

export function Header(props: HeaderProps) {
  return (
    <header className="header">
      <h1>{props.title}</h1>
      <span className="badge">{props.count}</span>
    </header>
  );
}
