interface UnicodeProps {
  greeting: string;
}

export function Unicode({ greeting }: UnicodeProps) {
  const [msg, setMsg] = useState(greeting);
  return (
    <button id="show" onClick={() => setMsg(msg)}>
      {msg}
    </button>
  );
}
