interface FooterProps {
  remaining: number;
}

export function Footer(props: FooterProps) {
  const label = props.remaining === 1 ? 'item left' : 'items left';
  return (
    <footer className="footer">
      <span>{props.remaining} {label}</span>
    </footer>
  );
}
