//! Inline refactoring
//!
//! Replaces every reference to a symbol with its definition body, then
//! removes the original definition.

use crate::lens_error::{ArgusError, Result};
use crate::semantic::symbols::SymbolKind;
use crate::syntax::Language;
use crate::type_inference::{
    DiagnosticLevel, RefactorKind, RefactorRequest, RefactorResult, Span, TextEdit,
};

use super::{FileContext, ProjectContext, RefactoringOp};

// ============================================================================
// Inline engine
// ============================================================================

/// Engine that inlines a variable or simple function.
pub struct InlineEngine;

impl RefactoringOp for InlineEngine {
    fn apply(
        &self,
        request: &RefactorRequest,
        file: &FileContext<'_>,
        _project: Option<&ProjectContext<'_>>,
    ) -> Result<RefactorResult> {
        if !matches!(request.kind, RefactorKind::Inline) {
            return Err(ArgusError::other(
                "InlineEngine received non-Inline request",
            ));
        }

        let source = file.source;
        let span = request.span;

        // Locate the symbol at the cursor
        let symbol = file
            .symbols
            .find_at_position(span.start_line as u32, span.start_col as u32)
            .ok_or_else(|| ArgusError::definition_not_found("No symbol at the given position"))?;

        let sym_name = symbol.name.clone();
        let _sym_kind = symbol.kind;

        // Extract the definition body
        let def_body = extract_definition_body(source, symbol, file.language)?;

        // Find all non-definition references in the same file
        let refs = file.symbols.find_references_at(
            span.start_line as u32,
            span.start_col as u32,
            false, // exclude definition
        );

        if refs.is_empty() {
            return Err(ArgusError::other(format!(
                "No references to '{}' found to inline",
                sym_name
            )));
        }

        let mut result = RefactorResult::empty();

        // Replace each reference with the definition body
        for r in &refs {
            let ref_span = range_to_span(r, source);
            result.add_edit(
                request.file.clone(),
                TextEdit {
                    span: ref_span,
                    new_text: def_body.clone(),
                },
            );
        }

        // Remove the original definition (entire statement)
        let def_span = find_definition_span(source, symbol, file.language);
        result.add_edit(
            request.file.clone(),
            TextEdit {
                span: def_span,
                new_text: String::new(),
            },
        );

        result.add_diagnostic(
            DiagnosticLevel::Info,
            format!("Inlined '{}' into {} site(s)", sym_name, refs.len()),
            Some(request.file.clone()),
            Some(span),
        );

        Ok(result)
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Extract the right-hand side of a variable assignment, or the body of a
/// simple single-expression function.
fn extract_definition_body(
    source: &str,
    symbol: &crate::semantic::symbols::Symbol,
    language: Language,
) -> Result<String> {
    let loc = &symbol.location;
    let def_line_start = line_start_byte(source, loc.start.line as usize);

    let line = source[def_line_start..].lines().next().unwrap_or("");

    match symbol.kind {
        SymbolKind::Variable | SymbolKind::Const | SymbolKind::Static => {
            // Look for `= <expr>` on the definition line
            if let Some(eq_idx) = line.find('=') {
                // Make sure it is `=` not `==`
                let after = line.as_bytes().get(eq_idx + 1).copied().unwrap_or(0);
                if after != b'=' {
                    let rhs = line[eq_idx + 1..].trim();
                    // Strip trailing semicolon (Rust/TS)
                    let rhs = rhs.strip_suffix(';').unwrap_or(rhs);
                    return Ok(rhs.to_string());
                }
            }
            Err(ArgusError::other(format!(
                "Could not extract definition body for '{}'",
                symbol.name
            )))
        }
        SymbolKind::Function => {
            // For simple functions we inline the body directly.
            // Single-expression functions return just the expression.
            // Multi-line function bodies are wrapped in a block expression.
            // Python: `def f(): return x` => `x`
            // Rust:   `fn f() -> T { stmt; expr }` => `{ stmt; expr }`
            extract_function_body(source, symbol, language)
        }
        _ => Err(ArgusError::other(format!(
            "Inlining {} symbols is not supported",
            symbol.kind.display_name()
        ))),
    }
}

/// Attempt to extract a single-expression function body.
fn extract_function_body(
    source: &str,
    symbol: &crate::semantic::symbols::Symbol,
    language: Language,
) -> Result<String> {
    let loc = &symbol.location;
    let start_byte = line_start_byte(source, loc.start.line as usize);

    // Collect lines that belong to this function definition
    let mut lines_iter = source[start_byte..].lines();
    let header = lines_iter.next().unwrap_or("");

    match language {
        Language::Python => {
            // `def f(x): return x + 1`  (single-line)
            if let Some(ret_idx) = header.find("return ") {
                let body = header[ret_idx + 7..].trim();
                return Ok(body.to_string());
            }
            // Multi-line: look for `return` on the next non-blank line
            for line in lines_iter {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Some(stripped) = trimmed.strip_prefix("return ") {
                    return Ok(stripped.trim().to_string());
                }
                break;
            }
        }
        Language::Rust => {
            // Collect lines until the closing `}` of the function body.
            let mut body_lines: Vec<String> = Vec::new();
            let mut depth: i32 = header.chars().filter(|&c| c == '{').count() as i32
                - header.chars().filter(|&c| c == '}').count() as i32;
            for line in lines_iter {
                depth += line.chars().filter(|&c| c == '{').count() as i32;
                depth -= line.chars().filter(|&c| c == '}').count() as i32;
                if depth <= 0 {
                    break;
                }
                body_lines.push(line.trim().to_string());
            }
            // Filter out blank lines that would be left over after stripping body markers
            let body_lines: Vec<&str> = body_lines
                .iter()
                .map(|l| l.as_str())
                .filter(|l| !l.is_empty())
                .collect();
            if body_lines.len() == 1 {
                // Single statement/expression: inline without wrapping braces
                let expr = body_lines[0].strip_suffix(';').unwrap_or(body_lines[0]);
                return Ok(expr.trim().to_string());
            } else if !body_lines.is_empty() {
                // Multi-line body: wrap in a block so the inlined expression is valid
                let indent = "    ";
                let inner = body_lines
                    .iter()
                    .map(|l| format!("{}{}", indent, l))
                    .collect::<Vec<_>>()
                    .join("\n");
                return Ok(format!("{{\n{}\n}}", inner));
            }
        }
        Language::TypeScript | Language::JavaScript => {
            // `function f() { return expr; }`
            if let Some(ret_idx) = header.find("return ") {
                let body = header[ret_idx + 7..].trim();
                let body = body
                    .strip_suffix("};")
                    .or_else(|| body.strip_suffix(';'))
                    .unwrap_or(body);
                let body = body.strip_suffix('}').unwrap_or(body).trim();
                return Ok(body.to_string());
            }
        }
        _ => {}
    }

    Err(ArgusError::other(format!(
        "Cannot inline multi-statement function '{}'",
        symbol.name
    )))
}

/// Return the full span of the definition statement so it can be deleted.
fn find_definition_span(
    source: &str,
    symbol: &crate::semantic::symbols::Symbol,
    language: Language,
) -> Span {
    let loc = &symbol.location;
    let line_start = line_start_byte(source, loc.start.line as usize);

    match symbol.kind {
        SymbolKind::Variable | SymbolKind::Const | SymbolKind::Static => {
            // Delete the whole line including trailing newline
            let line_end = source[line_start..]
                .find('\n')
                .map_or(source.len(), |i| line_start + i + 1);
            Span::new(line_start, line_end)
        }
        SymbolKind::Function => {
            // Walk forward to find the end of the function
            let end = find_function_end(source, line_start, language);
            Span::new(line_start, end)
        }
        _ => {
            let line_end = source[line_start..]
                .find('\n')
                .map_or(source.len(), |i| line_start + i + 1);
            Span::new(line_start, line_end)
        }
    }
}

/// Find the byte offset past the end of a function starting at `start`.
fn find_function_end(source: &str, start: usize, language: Language) -> usize {
    match language {
        Language::Rust | Language::TypeScript | Language::JavaScript => {
            // Brace-delimited: count `{` vs `}`
            let mut depth: i32 = 0;
            let mut found_open = false;
            for (i, ch) in source[start..].char_indices() {
                if ch == '{' {
                    depth += 1;
                    found_open = true;
                } else if ch == '}' {
                    depth -= 1;
                    if found_open && depth == 0 {
                        let end = start + i + 1;
                        // consume trailing newline
                        if source.as_bytes().get(end) == Some(&b'\n') {
                            return end + 1;
                        }
                        return end;
                    }
                }
            }
            source.len()
        }
        _ => {
            // Python: indentation-based
            let first_line = source[start..].lines().next().unwrap_or("");
            let base_indent = first_line.len() - first_line.trim_start().len();
            let mut end = start + first_line.len() + 1;
            for line in source[end..].lines() {
                if line.trim().is_empty() {
                    end += line.len() + 1;
                    continue;
                }
                let indent = line.len() - line.trim_start().len();
                if indent <= base_indent {
                    break;
                }
                end += line.len() + 1;
            }
            end.min(source.len())
        }
    }
}

/// Byte offset of the beginning of line number `line` (0-indexed).
fn line_start_byte(source: &str, line: usize) -> usize {
    let mut offset = 0;
    for (i, text_line) in source.split('\n').enumerate() {
        if i == line {
            return offset;
        }
        offset += text_line.len() + 1;
    }
    source.len()
}

/// Convert `diagnostic::Range` to byte-offset `Span`.
fn range_to_span(range: &crate::diagnostic::Range, source: &str) -> Span {
    let start = line_col_to_byte(source, range.start.line, range.start.character);
    let end = line_col_to_byte(source, range.end.line, range.end.character);
    Span::with_lines(
        start,
        end,
        range.start.line as usize,
        range.start.character as usize,
        range.end.line as usize,
        range.end.character as usize,
    )
}

fn line_col_to_byte(source: &str, line: u32, col: u32) -> usize {
    let mut cur_line = 0u32;
    let mut offset = 0usize;
    for text_line in source.split('\n') {
        if cur_line == line {
            return offset + (col as usize).min(text_line.len());
        }
        offset += text_line.len() + 1;
        cur_line += 1;
    }
    source.len()
}
