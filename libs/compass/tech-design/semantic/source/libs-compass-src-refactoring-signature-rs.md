---
id: libs-compass-src-refactoring-signature-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/refactoring/signature.rs`.
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

# Standardized libs/compass/src/refactoring/signature.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/refactoring/signature.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SignatureEngine` | libs/compass/src/refactoring/signature.rs | struct | pub | 19 | pub struct SignatureEngine; |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/refactoring/signature.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/refactoring/signature.rs` captured during libs codegen standardization.
```
