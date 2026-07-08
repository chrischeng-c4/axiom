---
id: libs-compass-src-refactoring-extract-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/refactoring/extract.rs`.
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

# Standardized libs/compass/src/refactoring/extract.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/refactoring/extract.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ExtractEngine` | libs/compass/src/refactoring/extract.rs | struct | pub | 21 | pub struct ExtractEngine; |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Extract function / method / variable refactoring
//!
//! - **Extract Function**: analyses data flow and lifts a selection into a
//!   standalone function.
//! - **Extract Method**: same but adds `self` and class-level indentation.
//! - **Extract Variable**: replaces an expression with a named variable.

use crate::lens_error::{ArgusError, Result};
use crate::syntax::Language;
use crate::type_inference::{
    DiagnosticLevel, RefactorKind, RefactorRequest, RefactorResult, Span, TextEdit,
};

use super::extract_helpers::{
    analyze_data_flow, build_call_expr, build_function_def, build_method_call, build_method_def,
    find_insertion_point, find_statement_start, leading_indent,
};
use super::{validate_identifier, FileContext, ProjectContext, RefactoringOp};

/// Unified engine for all extract-* operations.
pub struct ExtractEngine;

impl RefactoringOp for ExtractEngine {
    fn apply(
        &self,
        request: &RefactorRequest,
        file: &FileContext<'_>,
        _project: Option<&ProjectContext<'_>>,
    ) -> Result<RefactorResult> {
        match &request.kind {
            RefactorKind::ExtractFunction { name } => extract_function(request, name, file),
            RefactorKind::ExtractMethod { name } => extract_method(request, name, file),
            RefactorKind::ExtractVariable { name } => extract_variable(request, name, file),
            _ => Err(ArgusError::other(
                "ExtractEngine received non-Extract request",
            )),
        }
    }
}

// ============================================================================
// Extract Function
// ============================================================================

fn extract_function(
    request: &RefactorRequest,
    name: &str,
    file: &FileContext<'_>,
) -> Result<RefactorResult> {
    validate_identifier(name, file.language)?;

    let source = file.source;
    let span = request.span;
    let selected = &source[span.start..span.end];
    let flow = analyze_data_flow(selected);
    let mut result = RefactorResult::empty();

    let func_text = build_function_def(name, &flow, selected, file.language);
    let call_text = build_call_expr(name, &flow);
    let insert_pos = find_insertion_point(source, span);

    result.add_edit(
        request.file.clone(),
        TextEdit {
            span: Span::new(insert_pos, insert_pos),
            new_text: func_text,
        },
    );
    result.add_edit(
        request.file.clone(),
        TextEdit {
            span,
            new_text: call_text,
        },
    );
    result.add_diagnostic(
        DiagnosticLevel::Info,
        format!(
            "Extracted function '{}' ({} param(s), {} return(s))",
            name,
            flow.params.len(),
            flow.returns.len(),
        ),
        Some(request.file.clone()),
        Some(span),
    );
    Ok(result)
}

// ============================================================================
// Extract Method
// ============================================================================

fn extract_method(
    request: &RefactorRequest,
    name: &str,
    file: &FileContext<'_>,
) -> Result<RefactorResult> {
    validate_identifier(name, file.language)?;

    let source = file.source;
    let span = request.span;
    let selected = &source[span.start..span.end];
    let mut flow = analyze_data_flow(selected);
    flow.params.retain(|p| p != "self");

    let mut result = RefactorResult::empty();
    let method_text = build_method_def(name, &flow, selected, file.language);
    let call_text = build_method_call(name, &flow);
    let insert_pos = find_insertion_point(source, span);

    result.add_edit(
        request.file.clone(),
        TextEdit {
            span: Span::new(insert_pos, insert_pos),
            new_text: method_text,
        },
    );
    result.add_edit(
        request.file.clone(),
        TextEdit {
            span,
            new_text: call_text,
        },
    );
    result.add_diagnostic(
        DiagnosticLevel::Info,
        format!("Extracted method '{}'", name),
        Some(request.file.clone()),
        Some(span),
    );
    Ok(result)
}

// ============================================================================
// Extract Variable
// ============================================================================

fn extract_variable(
    request: &RefactorRequest,
    name: &str,
    file: &FileContext<'_>,
) -> Result<RefactorResult> {
    validate_identifier(name, file.language)?;

    let source = file.source;
    let span = request.span;
    let expr_text = &source[span.start..span.end];

    let (assign, replacement) = match file.language {
        Language::Rust => (format!("let {} = {};\n", name, expr_text), name.to_string()),
        Language::TypeScript | Language::JavaScript => (
            format!("const {} = {};\n", name, expr_text),
            name.to_string(),
        ),
        _ => (format!("{} = {}\n", name, expr_text), name.to_string()),
    };

    let stmt_start = find_statement_start(source, span.start);
    let indent = leading_indent(source, stmt_start);

    let mut result = RefactorResult::empty();
    result.add_edit(
        request.file.clone(),
        TextEdit {
            span: Span::new(stmt_start, stmt_start),
            new_text: format!("{}{}", indent, assign),
        },
    );
    result.add_edit(
        request.file.clone(),
        TextEdit {
            span,
            new_text: replacement,
        },
    );
    result.add_diagnostic(
        DiagnosticLevel::Info,
        format!("Extracted variable '{}'", name),
        Some(request.file.clone()),
        Some(span),
    );
    Ok(result)
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/refactoring/extract.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/refactoring/extract.rs` captured during libs codegen standardization.
```
