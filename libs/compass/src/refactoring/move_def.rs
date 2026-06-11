//! Move-definition refactoring
//!
//! Moves a function, class, or type definition from one file to another,
//! then rewrites imports across the entire project so that existing
//! consumers still resolve correctly.

use std::path::PathBuf;

use crate::lens_error::{ArgusError, Result};
use crate::semantic::symbols::SymbolKind;
use crate::syntax::Language;
use crate::type_inference::{
    DiagnosticLevel, ImportChange, RefactorKind, RefactorRequest, RefactorResult, Span, TextEdit,
};

use super::{FileContext, ProjectContext, RefactoringOp};

// ============================================================================
// Move-definition engine
// ============================================================================

/// Engine that moves a top-level definition between files.
pub struct MoveDefEngine;

impl RefactoringOp for MoveDefEngine {
    fn apply(
        &self,
        request: &RefactorRequest,
        file: &FileContext<'_>,
        project: Option<&ProjectContext<'_>>,
    ) -> Result<RefactorResult> {
        let target_file = match &request.kind {
            RefactorKind::MoveDefinition { target_file } => target_file,
            _ => {
                return Err(ArgusError::other(
                    "MoveDefEngine received non-MoveDefinition request",
                ))
            }
        };

        let source = file.source;
        let span = request.span;

        // Find the symbol at the given position
        let symbol = file
            .symbols
            .find_at_position(span.start_line as u32, span.start_col as u32)
            .ok_or_else(|| {
                ArgusError::definition_not_found("No definition found at the given position")
            })?;

        let sym_name = symbol.name.clone();
        let sym_kind = symbol.kind;

        // Only movable definition kinds
        if !matches!(
            sym_kind,
            SymbolKind::Function
                | SymbolKind::Class
                | SymbolKind::Struct
                | SymbolKind::Trait
                | SymbolKind::Enum
                | SymbolKind::Interface
                | SymbolKind::TypeAlias
        ) {
            return Err(ArgusError::other(format!(
                "Cannot move a {} definition",
                sym_kind.display_name()
            )));
        }

        // Extract the full text of the definition
        let def_span = find_full_definition_span(source, symbol, file.language);
        let def_text = source[def_span.start..def_span.end].to_string();

        let mut result = RefactorResult::empty();

        // 1. Remove definition from source file
        result.add_edit(
            request.file.clone(),
            TextEdit {
                span: def_span,
                new_text: String::new(),
            },
        );

        // 2. Add definition to target file
        result
            .new_files
            .entry(target_file.clone())
            .and_modify(|content| {
                content.push('\n');
                content.push_str(&def_text);
            })
            .or_insert_with(|| def_text.clone());

        // 3. Add import in the original file pointing to the new location
        let import_stmt =
            build_import_statement(&sym_name, target_file, &request.file, file.language);
        result.add_edit(
            request.file.clone(),
            TextEdit {
                span: Span::new(0, 0), // prepend at top of file
                new_text: import_stmt,
            },
        );

        // 4. Update imports in all other project files
        if let Some(ctx) = project {
            update_project_imports(
                &sym_name,
                &request.file,
                target_file,
                file.language,
                ctx,
                &mut result,
            );
        }

        result.add_diagnostic(
            DiagnosticLevel::Info,
            format!(
                "Moved {} '{}' to {}",
                sym_kind.display_name(),
                sym_name,
                target_file.display()
            ),
            Some(request.file.clone()),
            Some(span),
        );

        Ok(result)
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Find the full span of a top-level definition including decorators / doc
/// comments that precede it.
fn find_full_definition_span(
    source: &str,
    symbol: &crate::semantic::symbols::Symbol,
    language: Language,
) -> Span {
    let loc = &symbol.location;
    let def_line = loc.start.line as usize;

    // Walk backwards to include decorators / doc comments
    let start_line = scan_backwards_for_preamble(source, def_line, language);
    let start_byte = line_start_byte(source, start_line);

    // Walk forward to find the end of the definition
    let end_byte = find_definition_end(source, def_line, language);

    Span::with_lines(
        start_byte, end_byte, start_line, 0, 0, // end line/col filled approximately
        0,
    )
}

/// Scan upward from `line` to include decorators or doc comments.
fn scan_backwards_for_preamble(source: &str, line: usize, language: Language) -> usize {
    let lines: Vec<&str> = source.lines().collect();
    let mut start = line;

    match language {
        Language::Python => {
            while start > 0 {
                let prev = lines.get(start - 1).map(|l| l.trim()).unwrap_or("");
                if prev.starts_with('@') || prev.starts_with('#') {
                    start -= 1;
                } else {
                    break;
                }
            }
        }
        Language::Rust => {
            while start > 0 {
                let prev = lines.get(start - 1).map(|l| l.trim()).unwrap_or("");
                if prev.starts_with('#') || prev.starts_with("///") || prev.starts_with("//!") {
                    start -= 1;
                } else {
                    break;
                }
            }
        }
        Language::TypeScript | Language::JavaScript => {
            while start > 0 {
                let prev = lines.get(start - 1).map(|l| l.trim()).unwrap_or("");
                if prev.starts_with("//") || prev.starts_with('*') || prev.starts_with("/*") {
                    start -= 1;
                } else {
                    break;
                }
            }
        }
        _ => {}
    }

    start
}

/// Find the byte offset past the end of a definition starting at `line`.
fn find_definition_end(source: &str, line: usize, language: Language) -> usize {
    let start_byte = line_start_byte(source, line);

    match language {
        Language::Rust | Language::TypeScript | Language::JavaScript => {
            // Brace counting
            let mut depth: i32 = 0;
            let mut found_open = false;
            for (i, ch) in source[start_byte..].char_indices() {
                if ch == '{' {
                    depth += 1;
                    found_open = true;
                } else if ch == '}' {
                    depth -= 1;
                    if found_open && depth == 0 {
                        let end = start_byte + i + 1;
                        return if source.as_bytes().get(end) == Some(&b'\n') {
                            end + 1
                        } else {
                            end
                        };
                    }
                }
            }
            source.len()
        }
        _ => {
            // Python: indentation-based
            let first_line = source[start_byte..].lines().next().unwrap_or("");
            let base_indent = first_line.len() - first_line.trim_start().len();
            let mut end = start_byte + first_line.len() + 1;
            for text_line in source[end..].lines() {
                if text_line.trim().is_empty() {
                    end += text_line.len() + 1;
                    continue;
                }
                let indent = text_line.len() - text_line.trim_start().len();
                if indent <= base_indent {
                    break;
                }
                end += text_line.len() + 1;
            }
            end.min(source.len())
        }
    }
}

/// Build a language-appropriate import statement.
fn build_import_statement(
    name: &str,
    target: &PathBuf,
    origin: &PathBuf,
    language: Language,
) -> String {
    let module = path_to_module(target, origin);

    match language {
        Language::Python => format!("from {} import {}\n", module, name),
        Language::TypeScript | Language::JavaScript => {
            format!("import {{ {} }} from '{}';\n", name, module)
        }
        Language::Rust => format!("use {}::{};\n", module, name),
        _ => format!("// import {} from {}\n", name, module),
    }
}

/// Derive a module import path from a file path, relative to `origin`.
///
/// Computes the relative path from `origin`'s parent directory to `target`,
/// strips the file extension, and converts OS separators to the appropriate
/// module separator for the target language.
///
/// Examples (Python):
///   origin = `src/api/routes.py`, target = `src/models/user.py`
///   → `..models.user`
///
/// Examples (TypeScript/Rust):
///   origin = `src/api/routes.ts`, target = `src/models/user.ts`
///   → `../models/user`
fn path_to_module(target: &PathBuf, origin: &PathBuf) -> String {
    // Parent of the origin file is the "current" directory
    let origin_dir = origin.parent().unwrap_or_else(|| std::path::Path::new("."));

    // Compute a relative path from origin_dir to target
    let relative = relative_path(origin_dir, target);

    // Strip file extension from target component
    let without_ext = relative.with_extension("");

    // Convert to string
    let rel_str = without_ext.to_string_lossy().replace('\\', "/"); // normalise on Windows

    rel_str
}

/// Compute the relative path from `from_dir` to `to_path`.
///
/// Works by finding the longest common ancestor and then building
/// `../` segments to reach the common ancestor from `from_dir`,
/// followed by the remaining segments to `to_path`.
fn relative_path(from_dir: &std::path::Path, to_path: &std::path::Path) -> std::path::PathBuf {
    // Collect components as OS strings for comparison
    let from_parts: Vec<_> = from_dir.components().collect();
    let to_parts: Vec<_> = to_path.components().collect();

    // Find the common prefix length
    let common_len = from_parts
        .iter()
        .zip(to_parts.iter())
        .take_while(|(a, b)| a == b)
        .count();

    // Number of ".." needed to reach the common ancestor from from_dir
    let ups = from_parts.len() - common_len;

    let mut rel = std::path::PathBuf::new();
    for _ in 0..ups {
        rel.push("..");
    }
    for part in &to_parts[common_len..] {
        rel.push(part);
    }

    if rel.as_os_str().is_empty() {
        rel.push(".");
    }

    rel
}

/// Rewrite imports in every project file that previously imported `name`
/// from `old_file` so they now import from `new_file`.
fn update_project_imports(
    name: &str,
    old_file: &PathBuf,
    new_file: &PathBuf,
    _language: Language,
    project: &ProjectContext<'_>,
    result: &mut RefactorResult,
) {
    let new_module = path_to_module(new_file, old_file);

    for (path, (_src, _parsed, symbols)) in project.files.iter() {
        if path == old_file || path == new_file {
            continue;
        }

        let import_syms = symbols.find_by_name(name);
        let has_import = import_syms.iter().any(|s| s.kind == SymbolKind::Import);

        if has_import {
            result
                .import_changes
                .entry(path.clone())
                .or_default()
                .push(ImportChange::Update {
                    module: new_module.clone(),
                    old_names: vec![name.to_string()],
                    new_names: vec![name.to_string()],
                });
        }
    }
}

/// Byte offset of the beginning of the given 0-indexed line.
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
