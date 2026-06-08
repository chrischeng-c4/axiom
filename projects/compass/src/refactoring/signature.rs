//! Change-signature refactoring
//!
//! Modifies a function's parameter list (add, remove, reorder) and updates
//! every call site across the project to match the new signature.

use crate::lens_error::{ArgusError, Result};
use crate::semantic::symbols::SymbolKind;
use crate::type_inference::{
    DiagnosticLevel, RefactorKind, RefactorRequest, RefactorResult, TextEdit,
};

use super::signature_helpers::{
    build_new_params, find_param_list_span, find_return_type_span, format_params, line_start_byte,
    parse_params, update_call_sites,
};
use super::{FileContext, ProjectContext, RefactoringOp};

/// Engine for changing a function signature and updating call sites.
pub struct SignatureEngine;

impl RefactoringOp for SignatureEngine {
    fn apply(
        &self,
        request: &RefactorRequest,
        file: &FileContext<'_>,
        project: Option<&ProjectContext<'_>>,
    ) -> Result<RefactorResult> {
        let changes = match &request.kind {
            RefactorKind::ChangeSignature { changes } => changes,
            _ => {
                return Err(ArgusError::other(
                    "SignatureEngine received non-ChangeSignature request",
                ))
            }
        };

        let source = file.source;
        let span = request.span;

        // Find the function symbol
        let symbol = file
            .symbols
            .find_at_position(span.start_line as u32, span.start_col as u32)
            .ok_or_else(|| {
                ArgusError::definition_not_found("No function found at the given position")
            })?;

        if symbol.kind != SymbolKind::Function {
            return Err(ArgusError::other(format!(
                "Expected a function, found {}",
                symbol.kind.display_name()
            )));
        }

        let func_name = symbol.name.clone();
        let def_line_start = line_start_byte(source, symbol.location.start.line as usize);
        let existing_params = parse_params(source, def_line_start, file.language);
        let new_params = build_new_params(&existing_params, changes);

        let mut result = RefactorResult::empty();

        // 1. Rewrite the definition's parameter list
        if let Some(param_span) = find_param_list_span(source, def_line_start, file.language) {
            let new_param_str = format_params(&new_params, file.language);
            result.add_edit(
                request.file.clone(),
                TextEdit {
                    span: param_span,
                    new_text: new_param_str,
                },
            );
        }

        // 2. Optionally update return type
        if let Some(ref new_ret) = changes.new_return_type {
            if let Some(ret_span) = find_return_type_span(source, def_line_start, file.language) {
                result.add_edit(
                    request.file.clone(),
                    TextEdit {
                        span: ret_span,
                        new_text: new_ret.clone(),
                    },
                );
            }
        }

        // 3. Update call sites in the same file
        update_call_sites(
            &func_name,
            &existing_params,
            changes,
            source,
            &request.file,
            file.language,
            &mut result,
        );

        // 4. Update call sites in other project files
        if let Some(ctx) = project {
            for (path, (src, _parsed, _symbols)) in ctx.files.iter() {
                if path == &request.file {
                    continue;
                }
                update_call_sites(
                    &func_name,
                    &existing_params,
                    changes,
                    src,
                    path,
                    file.language,
                    &mut result,
                );
            }
        }

        result.add_diagnostic(
            DiagnosticLevel::Info,
            format!(
                "Changed signature of '{}': {} -> {} params",
                func_name,
                existing_params.len(),
                new_params.len(),
            ),
            Some(request.file.clone()),
            Some(span),
        );

        Ok(result)
    }
}
