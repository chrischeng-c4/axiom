//! Code outline: enumerate the functions/methods a source file defines.
//!
//! This is a language-agnostic code-intelligence primitive — "what callable
//! definitions live in this file, and where" — built directly on the
//! tree-sitter parse. It deliberately knows nothing about *why* a caller wants
//! the list (instrumentation, navigation, coverage, doc generation); consumers
//! layer their own policy on top. `meter`, for example, maps each
//! [`FunctionDef`] to a probe point.
//!
//! The per-language tree-sitter node-kind knowledge lives here (the compass
//! domain), so consumers never re-implement grammar details.

use crate::lens_error::{ArgusError, Result};
use crate::syntax::{Language, MultiParser, ParsedFile};
use serde::{Deserialize, Serialize};

/// Whether a definition is a free function or a method the grammar distinguishes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FunctionKind {
    /// Free function (Rust `fn`, Python `def`, TS/JS `function`, Go `func`).
    /// Rust methods are reported as `Function` because tree-sitter-rust models
    /// them as the same `function_item` node.
    Function,
    /// A method node the grammar distinguishes from a free function
    /// (TS/JS `method_definition`, Go `method_declaration`).
    Method,
}

/// A single callable definition discovered in a source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionDef {
    /// Definition name (`<anonymous>` when the grammar node has no `name` field).
    pub name: String,
    /// Free function vs method.
    pub kind: FunctionKind,
    /// 1-based first line of the definition (inclusive).
    pub start_line: usize,
    /// 1-based last line of the definition (inclusive).
    pub end_line: usize,
}

/// tree-sitter node kinds that name a function/method, paired with the
/// [`FunctionKind`] they map to, per language. Returns `&[]` for languages
/// compass does not enumerate callables for.
fn function_kinds(language: Language) -> &'static [(&'static str, FunctionKind)] {
    match language {
        Language::Rust => &[("function_item", FunctionKind::Function)],
        Language::Python => &[("function_definition", FunctionKind::Function)],
        Language::TypeScript | Language::JavaScript => &[
            ("function_declaration", FunctionKind::Function),
            ("generator_function_declaration", FunctionKind::Function),
            ("method_definition", FunctionKind::Method),
        ],
        Language::Go => &[
            ("function_declaration", FunctionKind::Function),
            ("method_declaration", FunctionKind::Method),
        ],
        _ => &[],
    }
}

/// Enumerate the callable definitions in an already-parsed file.
///
/// Pure (no parsing/IO); returns the definitions ordered by `start_line`. Use
/// this when you already hold a [`ParsedFile`]; otherwise see [`outline`].
pub fn outline_parsed(parsed: &ParsedFile) -> Vec<FunctionDef> {
    let kinds = function_kinds(parsed.language);
    if kinds.is_empty() {
        return Vec::new();
    }

    let mut defs = Vec::new();
    parsed.walk(|node, _depth| {
        if let Some((_, kind)) = kinds.iter().find(|(k, _)| *k == node.kind()) {
            let name = node
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(parsed.source.as_bytes()).ok())
                .unwrap_or("<anonymous>")
                .to_string();
            defs.push(FunctionDef {
                name,
                kind: *kind,
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
            });
        }
        true
    });

    defs.sort_by_key(|d| (d.start_line, d.end_line));
    defs
}

/// Parse `source` as `language` and enumerate its callable definitions.
///
/// Convenience over [`outline_parsed`] that owns the parse. Errors if the
/// parser cannot initialize or the grammar produces no tree.
pub fn outline(source: &str, language: Language) -> Result<Vec<FunctionDef>> {
    let mut parser = MultiParser::new()?;
    let parsed = parser.parse(source, language).ok_or_else(|| {
        ArgusError::parser(format!(
            "outline: grammar produced no tree for {}",
            language.as_str()
        ))
    })?;
    Ok(outline_parsed(&parsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_functions_and_methods_have_precise_spans() {
        // 1: fn alpha() -> i32 {
        // 2:     1
        // 3: }
        // 4:
        // 5: struct S;
        // 6: impl S {
        // 7:     fn beta(&self) -> i32 {
        // 8:         2
        // 9:     }
        // 10: }
        let src = "fn alpha() -> i32 {\n    1\n}\n\nstruct S;\nimpl S {\n    fn beta(&self) -> i32 {\n        2\n    }\n}\n";
        let defs = outline(src, Language::Rust).unwrap();

        let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "beta"]);

        assert_eq!(defs[0].start_line, 1);
        assert_eq!(defs[0].end_line, 3);
        // tree-sitter-rust models impl methods as `function_item` too.
        assert_eq!(defs[0].kind, FunctionKind::Function);
        assert_eq!(defs[1].start_line, 7);
        assert_eq!(defs[1].end_line, 9);
    }

    #[test]
    fn python_functions_are_discovered() {
        let src = "def gamma():\n    return 1\n\ndef delta(x):\n    return x\n";
        let defs = outline(src, Language::Python).unwrap();
        let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, vec!["gamma", "delta"]);
        assert_eq!(defs[0].start_line, 1);
        assert_eq!(defs[1].start_line, 4);
    }

    #[test]
    fn typescript_distinguishes_methods() {
        let src = "function top() {}\nclass C {\n  m() {}\n}\n";
        let defs = outline(src, Language::TypeScript).unwrap();
        let by_name: std::collections::HashMap<&str, FunctionKind> =
            defs.iter().map(|d| (d.name.as_str(), d.kind)).collect();
        assert_eq!(by_name.get("top"), Some(&FunctionKind::Function));
        assert_eq!(by_name.get("m"), Some(&FunctionKind::Method));
    }
}
