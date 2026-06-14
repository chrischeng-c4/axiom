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
