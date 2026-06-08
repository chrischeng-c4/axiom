// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
// CODEGEN-BEGIN
//! React Fast Refresh injection for HMR support.
//!
//! Detects React component declarations in the AST and injects:
//! 1. Preamble: import RefreshRuntime + create $RefreshSig$
//! 2. `$RefreshReg$(Component, "Component")` after each component
//! 3. `$RefreshSig$()` for hooks signature tracking
//! 4. Footer: `RefreshRuntime.enqueueUpdate()`

use tree_sitter::Node;

/// Inject React Fast Refresh instrumentation into transformed code.
///
/// `transformed` — the already-JSX-transformed JavaScript code
/// `source` — the original source (for AST name extraction)
/// `root` — the parsed tree-sitter AST root node
/// @spec .aw/tech-design/projects/jet/semantic/jet-transform.md#schema
pub fn inject_react_fast_refresh(transformed: &str, source: &str, root: &Node) -> String {
    let components = detect_react_components(source, root);

    if components.is_empty() {
        return transformed.to_string();
    }

    let mut result = String::new();

    // 1. Preamble
    result.push_str(
        "import RefreshRuntime from '/@react-refresh';\n\
         const $RefreshReg$ = RefreshRuntime.register;\n\
         const $RefreshSig$ = RefreshRuntime.createSignatureFunctionForTransform;\n",
    );

    result.push_str(transformed);

    // 2. Register each component with $RefreshReg$
    result.push('\n');
    for comp in &components {
        result.push_str(&format!(
            "$RefreshReg$({name}, \"{name}\");\n",
            name = comp.name
        ));

        // 3. If the component uses hooks, inject signature tracking
        if comp.uses_hooks {
            result.push_str(&format!(
                "$RefreshSig$()({name}, \"{hooks}\");\n",
                name = comp.name,
                hooks = comp.hooks_signature
            ));
        }
    }

    // 4. Footer
    result.push_str("\nRefreshRuntime.enqueueUpdate();\n");

    result
}

/// Information about a detected React component.
struct ReactComponent {
    /// Component function name (e.g. "App", "Counter").
    name: String,
    /// Whether the component uses React hooks.
    uses_hooks: bool,
    /// Fingerprint of hook call order (e.g. "useState{} useEffect{}").
    hooks_signature: String,
}

/// Detect React component declarations in the source AST.
///
/// A function is considered a React component if:
/// - Its name starts with an uppercase letter
/// - It returns JSX (contains jsx_element, jsx_self_closing_element, or jsx_fragment)
/// - OR it's wrapped in React.memo() / React.forwardRef()
fn detect_react_components(source: &str, root: &Node) -> Vec<ReactComponent> {
    let mut components = Vec::new();
    collect_components(source, root, &mut components);
    components
}

fn collect_components(source: &str, node: &Node, components: &mut Vec<ReactComponent>) {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            // function App() { return <div/> }
            "function_declaration" => {
                if let Some(comp) = check_function_component(source, &child) {
                    components.push(comp);
                }
            }
            // export function App() { ... }
            "export_statement" => {
                let mut inner_cursor = child.walk();
                for inner in child.children(&mut inner_cursor) {
                    if inner.kind() == "function_declaration" {
                        if let Some(comp) = check_function_component(source, &inner) {
                            components.push(comp);
                        }
                    }
                }
            }
            // const App = () => <div/>  or  const App = function() { return <div/> }
            // const App = React.memo(...)  or  const App = React.forwardRef(...)
            "lexical_declaration" => {
                if let Some(comp) = check_variable_component(source, &child) {
                    components.push(comp);
                }
            }
            _ => {
                // Recurse into other nodes
                collect_components(source, &child, components);
            }
        }
    }
}

/// Check if a function declaration is a React component.
fn check_function_component(source: &str, node: &Node) -> Option<ReactComponent> {
    let name = get_function_name(source, node)?;

    // Must start with uppercase (React component convention)
    if !name.starts_with(|c: char| c.is_uppercase()) {
        return None;
    }

    // Must contain JSX in its body
    if !function_body_has_jsx(node) {
        return None;
    }

    let (uses_hooks, hooks_sig) = detect_hooks_usage(source, node);

    Some(ReactComponent {
        name,
        uses_hooks,
        hooks_signature: hooks_sig,
    })
}

/// Check if a variable declaration contains a React component
/// (arrow function or function expression returning JSX, or React.memo/forwardRef).
fn check_variable_component(source: &str, node: &Node) -> Option<ReactComponent> {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "variable_declarator" {
            let name = get_declarator_name(source, &child)?;

            if !name.starts_with(|c: char| c.is_uppercase()) {
                return None;
            }

            // Get the initializer (value being assigned)
            let mut dc = child.walk();
            for dchild in child.children(&mut dc) {
                match dchild.kind() {
                    "arrow_function" | "function" | "function_expression" => {
                        if function_body_has_jsx(&dchild) {
                            let (uses_hooks, hooks_sig) = detect_hooks_usage(source, &dchild);
                            return Some(ReactComponent {
                                name,
                                uses_hooks,
                                hooks_signature: hooks_sig,
                            });
                        }
                    }
                    "call_expression" => {
                        // Check for React.memo(...) or React.forwardRef(...)
                        let call_text = &source[dchild.byte_range()];
                        if call_text.starts_with("React.memo")
                            || call_text.starts_with("React.forwardRef")
                            || call_text.starts_with("memo(")
                            || call_text.starts_with("forwardRef(")
                        {
                            if subtree_has_jsx(&dchild) {
                                let (uses_hooks, hooks_sig) = detect_hooks_usage(source, &dchild);
                                return Some(ReactComponent {
                                    name,
                                    uses_hooks,
                                    hooks_signature: hooks_sig,
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    None
}

/// Get the name of a function declaration.
fn get_function_name(source: &str, node: &Node) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" {
            return Some(source[child.byte_range()].to_string());
        }
    }
    None
}

/// Get the name from a variable declarator.
fn get_declarator_name(source: &str, node: &Node) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" {
            return Some(source[child.byte_range()].to_string());
        }
    }
    None
}

/// Check if a function's body contains JSX.
fn function_body_has_jsx(node: &Node) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "statement_block" || child.kind() == "parenthesized_expression" {
            if subtree_has_jsx(&child) {
                return true;
            }
        }
        // Arrow function with expression body: () => <div/>
        if matches!(
            child.kind(),
            "jsx_element" | "jsx_self_closing_element" | "jsx_fragment"
        ) {
            return true;
        }
    }
    false
}

/// Recursively check if any node in the subtree is a JSX element.
fn subtree_has_jsx(node: &Node) -> bool {
    if matches!(
        node.kind(),
        "jsx_element" | "jsx_self_closing_element" | "jsx_fragment"
    ) {
        return true;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if subtree_has_jsx(&child) {
            return true;
        }
    }
    false
}

/// Detect React hooks usage in a function body.
///
/// Returns (uses_hooks, hooks_signature_string).
fn detect_hooks_usage(source: &str, node: &Node) -> (bool, String) {
    let mut hooks = Vec::new();
    collect_hook_calls(source, node, &mut hooks);

    if hooks.is_empty() {
        (false, String::new())
    } else {
        let sig = hooks.join(" ");
        (true, sig)
    }
}

/// Collect all React hook calls (functions starting with "use") from a subtree.
fn collect_hook_calls(source: &str, node: &Node, hooks: &mut Vec<String>) {
    if node.kind() == "call_expression" {
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();
        if let Some(first_child) = children.first() {
            if first_child.kind() == "identifier" {
                let name = &source[first_child.byte_range()];
                // React hooks convention: useXxx
                if name.starts_with("use") && name.len() > 3 {
                    let next_char = name.chars().nth(3).unwrap_or('a');
                    if next_char.is_uppercase() {
                        hooks.push(format!("{}{{}}", name));
                    }
                }
            }
        }
    }

    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();
    for child in children {
        collect_hook_calls(source, &child, hooks);
    }
}
// CODEGEN-END
