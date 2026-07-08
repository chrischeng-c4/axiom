---
id: libs-compass-src-refactoring-rename-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/refactoring/rename.rs`.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [overview, source, changes]
---

# Standardized libs/compass/src/refactoring/rename.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/refactoring/rename.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `RenameEngine` | libs/compass/src/refactoring/rename.rs | struct | pub | 21 | pub struct RenameEngine; |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Cross-file rename refactoring
//!
//! Finds all references to a symbol (definitions, usages, imports) across
//! the project and generates `TextEdit`s to rename each occurrence.

use std::path::PathBuf;

use crate::lens_error::{ArgusError, Result};
use crate::semantic::symbols::{SymbolKind, SymbolTable};
use crate::type_inference::{
    DiagnosticLevel, ImportChange, RefactorKind, RefactorRequest, RefactorResult, Span, TextEdit,
};

use super::{validate_identifier, FileContext, ProjectContext, RefactoringOp};

// ============================================================================
// Rename engine
// ============================================================================

/// Engine for renaming symbols across files.
pub struct RenameEngine;

impl RefactoringOp for RenameEngine {
    fn apply(
        &self,
        request: &RefactorRequest,
        file: &FileContext<'_>,
        project: Option<&ProjectContext<'_>>,
    ) -> Result<RefactorResult> {
        let new_name = match &request.kind {
            RefactorKind::Rename { new_name } => new_name,
            _ => {
                return Err(ArgusError::other(
                    "RenameEngine received non-Rename request",
                ))
            }
        };

        validate_identifier(new_name, file.language)?;

        // Find the symbol at the target span
        let symbol = file
            .symbols
            .find_at_position(
                request.span.start_line as u32,
                request.span.start_col as u32,
            )
            .ok_or_else(|| {
                ArgusError::definition_not_found("No symbol found at the given position")
            })?;

        let old_name = symbol.name.clone();
        let _symbol_id = symbol.id;
        let symbol_kind = symbol.kind;

        if old_name == *new_name {
            return Ok(RefactorResult::empty());
        }

        let mut result = RefactorResult::empty();

        // Collect same-file references (definition + usages)
        let refs = file.symbols.find_references_at(
            request.span.start_line as u32,
            request.span.start_col as u32,
            true, // include definition
        );

        for r in &refs {
            let span = range_to_span(r, file.source);
            result.add_edit(
                request.file.clone(),
                TextEdit {
                    span,
                    new_text: new_name.clone(),
                },
            );
        }

        // Cross-file rename when project context is available
        if let Some(ctx) = project {
            self.rename_across_files(
                &old_name,
                new_name,
                symbol_kind,
                &request.file,
                ctx,
                &mut result,
            );
        }

        result.add_diagnostic(
            DiagnosticLevel::Info,
            format!(
                "Renamed '{}' -> '{}' ({} occurrence(s) in primary file)",
                old_name,
                new_name,
                refs.len()
            ),
            Some(request.file.clone()),
            Some(request.span),
        );

        Ok(result)
    }
}

impl RenameEngine {
    /// Walk every other file in the project and update references + imports.
    fn rename_across_files(
        &self,
        old_name: &str,
        new_name: &str,
        kind: SymbolKind,
        origin: &PathBuf,
        project: &ProjectContext<'_>,
        result: &mut RefactorResult,
    ) {
        for (path, (_source, _parsed, symbols)) in project.files.iter() {
            if path == origin {
                continue;
            }

            // Find references to the same name in other files
            let matches = symbols.find_by_name(old_name);
            for sym in &matches {
                // Only rename imports / usages of matching kind
                if sym.kind == SymbolKind::Import || sym.kind == kind {
                    let span = Span::with_lines(
                        0,
                        0,
                        sym.location.start.line as usize,
                        sym.location.start.character as usize,
                        sym.location.end.line as usize,
                        sym.location.end.character as usize,
                    );
                    result.add_edit(
                        path.clone(),
                        TextEdit {
                            span,
                            new_text: new_name.to_string(),
                        },
                    );
                }
            }

            // Update import statements that reference the old name
            self.update_imports_in_file(old_name, new_name, path, symbols, result);
        }
    }

    /// Rewrite import entries that mention `old_name`.
    fn update_imports_in_file(
        &self,
        old_name: &str,
        new_name: &str,
        path: &PathBuf,
        symbols: &SymbolTable,
        result: &mut RefactorResult,
    ) {
        let import_syms = symbols.find_by_name(old_name);
        let has_import = import_syms.iter().any(|s| s.kind == SymbolKind::Import);

        if has_import {
            result
                .import_changes
                .entry(path.clone())
                .or_default()
                .push(ImportChange::Update {
                    module: String::new(), // module path resolved at apply time
                    old_names: vec![old_name.to_string()],
                    new_names: vec![new_name.to_string()],
                });
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Convert a `diagnostic::Range` to a byte-offset `Span`.
///
/// This is a best-effort conversion: we walk lines of `source` to translate
/// line/column positions into byte offsets.
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

/// Translate 0-indexed line/column to byte offset.
fn line_col_to_byte(source: &str, line: u32, col: u32) -> usize {
    let mut current_line = 0u32;
    let mut byte_offset = 0usize;

    for text_line in source.split('\n') {
        if current_line == line {
            return byte_offset + (col as usize).min(text_line.len());
        }
        byte_offset += text_line.len() + 1; // +1 for '\n'
        current_line += 1;
    }
    source.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_col_to_byte() {
        let src = "abc\ndef\nghi";
        assert_eq!(line_col_to_byte(src, 0, 0), 0);
        assert_eq!(line_col_to_byte(src, 0, 2), 2);
        assert_eq!(line_col_to_byte(src, 1, 0), 4);
        assert_eq!(line_col_to_byte(src, 2, 1), 9);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/refactoring/rename.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/refactoring/rename.rs` captured during libs codegen standardization.
```
