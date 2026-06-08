//! Helpers for extract refactoring: data-flow analysis, code generation,
//! and position utilities.

use std::collections::HashSet;

use crate::syntax::Language;
use crate::type_inference::Span;

// ============================================================================
// Data-flow analysis
// ============================================================================

/// Lightweight data-flow result for the selected code region.
pub(super) struct DataFlow {
    /// Variables used but not defined inside the selection (become params).
    pub params: Vec<String>,
    /// Variables defined inside the selection and used after it (returned).
    pub returns: Vec<String>,
}

/// Simple text-based data-flow analysis.
pub(super) fn analyze_data_flow(selected: &str) -> DataFlow {
    let mut defined: HashSet<String> = HashSet::new();
    let mut used: HashSet<String> = HashSet::new();

    for line in selected.lines() {
        let trimmed = line.trim();
        if let Some(eq_pos) = trimmed.find('=') {
            let before_eq = if eq_pos > 0 {
                trimmed.as_bytes()[eq_pos - 1]
            } else {
                0
            };
            let after_eq = trimmed.as_bytes().get(eq_pos + 1).copied().unwrap_or(0);
            if before_eq != b'!' && before_eq != b'<' && before_eq != b'>' && after_eq != b'=' {
                let lhs = &trimmed[..eq_pos].trim();
                if is_simple_ident(lhs) {
                    defined.insert(lhs.to_string());
                }
                collect_identifiers(&trimmed[eq_pos + 1..], &mut used);
            } else {
                collect_identifiers(trimmed, &mut used);
            }
        } else {
            collect_identifiers(trimmed, &mut used);
        }
    }

    let params: Vec<String> = used
        .iter()
        .filter(|v| !defined.contains(*v) && !is_builtin(v))
        .cloned()
        .collect();
    let returns: Vec<String> = defined.into_iter().collect();

    DataFlow { params, returns }
}

fn is_simple_ident(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    let first = s.chars().next().unwrap();
    (first.is_alphabetic() || first == '_') && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

fn collect_identifiers(text: &str, out: &mut HashSet<String>) {
    let mut buf = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            buf.push(ch);
        } else {
            flush_ident(&buf, out);
            buf.clear();
        }
    }
    flush_ident(&buf, out);
}

fn flush_ident(buf: &str, out: &mut HashSet<String>) {
    if buf.is_empty() {
        return;
    }
    if buf.chars().next().map_or(true, |c| c.is_numeric()) {
        return;
    }
    out.insert(buf.to_string());
}

fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "print"
            | "len"
            | "range"
            | "str"
            | "int"
            | "float"
            | "bool"
            | "list"
            | "dict"
            | "tuple"
            | "set"
            | "True"
            | "False"
            | "None"
            | "self"
            | "cls"
            | "super"
            | "type"
            | "isinstance"
            | "console"
            | "undefined"
            | "null"
            | "require"
    )
}

// ============================================================================
// Code generation
// ============================================================================

pub(super) fn build_function_def(
    name: &str,
    flow: &DataFlow,
    body_text: &str,
    lang: Language,
) -> String {
    let body_indented = indent_block(body_text, "    ");
    match lang {
        Language::Rust => {
            let params_str = flow
                .params
                .iter()
                .map(|p| format!("{p}: _"))
                .collect::<Vec<_>>()
                .join(", ");
            let ret = if flow.returns.is_empty() {
                String::new()
            } else {
                " -> _".into()
            };
            let ret_stmt = return_stmt(&flow.returns, lang);
            format!("\nfn {name}({params_str}){ret} {{\n{body_indented}{ret_stmt}\n}}\n")
        }
        Language::TypeScript | Language::JavaScript => {
            let params_str = flow.params.join(", ");
            let ret_stmt = return_stmt(&flow.returns, lang);
            format!("\nfunction {name}({params_str}) {{\n{body_indented}{ret_stmt}\n}}\n")
        }
        _ => {
            let params_str = flow.params.join(", ");
            let ret_stmt = return_stmt(&flow.returns, lang);
            format!("\ndef {name}({params_str}):\n{body_indented}{ret_stmt}\n")
        }
    }
}

pub(super) fn build_method_def(
    name: &str,
    flow: &DataFlow,
    body_text: &str,
    _lang: Language,
) -> String {
    let mut method_params = vec!["self".to_string()];
    method_params.extend(flow.params.clone());
    let params_str = method_params.join(", ");
    let body_indented = indent_block(body_text, "        ");
    let ret_stmt = return_stmt(&flow.returns, Language::Python);
    format!("\n    def {name}({params_str}):\n{body_indented}{ret_stmt}\n")
}

fn return_stmt(vars: &[String], lang: Language) -> String {
    if vars.is_empty() {
        return String::new();
    }
    let val = if vars.len() == 1 {
        vars[0].clone()
    } else {
        match lang {
            Language::Rust => format!("({})", vars.join(", ")),
            Language::TypeScript | Language::JavaScript => format!("[{}]", vars.join(", ")),
            _ => vars.join(", "),
        }
    };
    format!("\n    return {val}")
}

pub(super) fn build_call_expr(name: &str, flow: &DataFlow) -> String {
    let args = flow.params.join(", ");
    let call = format!("{name}({args})");
    if flow.returns.is_empty() {
        call
    } else if flow.returns.len() == 1 {
        format!("{} = {}", flow.returns[0], call)
    } else {
        format!("{} = {}", flow.returns.join(", "), call)
    }
}

pub(super) fn build_method_call(name: &str, flow: &DataFlow) -> String {
    let args = flow.params.join(", ");
    let call = format!("self.{name}({args})");
    if flow.returns.is_empty() {
        call
    } else if flow.returns.len() == 1 {
        format!("{} = {}", flow.returns[0], call)
    } else {
        format!("{} = {}", flow.returns.join(", "), call)
    }
}

fn indent_block(text: &str, prefix: &str) -> String {
    text.lines()
        .map(|l| format!("{prefix}{l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

// ============================================================================
// Position helpers
// ============================================================================

/// Walk backwards from `pos` to find the start of the enclosing line.
pub(super) fn find_statement_start(source: &str, pos: usize) -> usize {
    source[..pos].rfind('\n').map_or(0, |i| i + 1)
}

/// Return the leading whitespace of the line starting at `line_start`.
pub(super) fn leading_indent(source: &str, line_start: usize) -> String {
    let rest = &source[line_start..];
    let ws_len = rest.len() - rest.trim_start().len();
    rest[..ws_len].to_string()
}

/// Find a suitable insertion point after the enclosing top-level definition.
pub(super) fn find_insertion_point(source: &str, span: Span) -> usize {
    let after = &source[span.end..];
    for (offset, line) in after.split('\n').enumerate() {
        if offset == 0 {
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        let indent = line.len() - trimmed.len();
        if indent == 0
            && (trimmed.starts_with("def ")
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("function ")
                || trimmed.starts_with("pub ")
                || trimmed.starts_with("struct ")
                || trimmed.starts_with("impl "))
        {
            let byte_offset: usize = after.split('\n').take(offset).map(|l| l.len() + 1).sum();
            return span.end + byte_offset;
        }
    }
    source.len()
}
