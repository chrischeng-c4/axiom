// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests-fixtures.md#component
// CODEGEN-BEGIN
interface AppProps {
  name: string;
}

const COPY = {
  title: "title",
  description: "description",
};

const GREETING = "hello";

export function App({ name }: AppProps) {
  const HEADER = "welcome";
  return (
    <div id="root">
      <span id="title">{COPY.title}</span>
      <span id="desc">{COPY.description}</span>
      <span id="greet">{GREETING}</span>
      <span id="header">{HEADER}</span>
      <span id="name">{name}</span>
    </div>
  );
}
// CODEGEN-END
