// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! AST probe for i18n copy constants (#1409). Ignored by default —
//! run with `cargo test -p jet --test tsx_to_rust_i18n_probe -- --ignored --nocapture`
//! to dump the parse tree for a fixture that uses top-level + in-component
//! readonly copy dictionaries.

use tree_sitter::{Node, Parser};

const TSX: &str = r#"const COPY = {
  title: "title",
  desc: "desc",
};

const GREETING = "hi";

interface AppProps { name: string }

export function App({ name }: AppProps) {
  const HEADER = "welcome";
  const items = { a: "1", b: "2" };
  return <div id="root">root</div>;
}
"#;

fn print_tree(node: Node, source: &str, depth: usize) {
    let prefix = "  ".repeat(depth);
    let text = &source[node.byte_range()];
    let snippet = text.replace('\n', " ");
    let snippet = if snippet.len() > 70 {
        format!("{}…", &snippet[..70])
    } else {
        snippet
    };
    let kind_kind = if node.is_named() { "N" } else { "a" };
    eprintln!("{prefix}{} [{}]  {:?}", node.kind(), kind_kind, snippet);
    let mut walker = node.walk();
    for child in node.children(&mut walker) {
        print_tree(child, source, depth + 1);
    }
}

#[test]
#[ignore]
fn dump_i18n_ast() {
    let mut parser = Parser::new();
    let language = tree_sitter_typescript::LANGUAGE_TSX.into();
    parser.set_language(&language).unwrap();
    let tree = parser.parse(TSX, None).unwrap();
    print_tree(tree.root_node(), TSX, 0);
}
// CODEGEN-END
