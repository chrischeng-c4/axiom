// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! AST probe — run with `cargo test -p jet::tsx_to_rust --test ast_probe
//! -- --nocapture` to dump the tree-sitter parse tree for the Counter
//! fixture. Used to discover kind names while building the transpiler.

use tree_sitter::{Node, Parser};

const COUNTER_TSX: &str = include_str!("../fixtures/tsx_to_rust_counter.tsx");

fn print_tree(node: Node, source: &str, depth: usize) {
    let prefix = "  ".repeat(depth);
    let text = &source[node.byte_range()];
    let snippet = text.replace('\n', "⏎");
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
fn dump_counter_ast() {
    let mut parser = Parser::new();
    let language = tree_sitter_typescript::LANGUAGE_TSX.into();
    parser.set_language(&language).unwrap();
    let tree = parser.parse(COUNTER_TSX, None).unwrap();
    print_tree(tree.root_node(), COUNTER_TSX, 0);
}
// CODEGEN-END
