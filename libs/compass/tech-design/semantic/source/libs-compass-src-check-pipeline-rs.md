---
id: libs-compass-src-check-pipeline-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/check_pipeline.rs`.
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

# Standardized libs/compass/src/check_pipeline.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/check_pipeline.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `TypeAtResult` | libs/compass/src/check_pipeline.rs | struct | pub | 26 | pub struct TypeAtResult { |
| `type_at` | libs/compass/src/check_pipeline.rs | function | pub | 42 | pub fn type_at(inferencer: &DeepTypeInferencer, file: &Path, symbol: &str) -> Option<TypeAtResult> { |
| `HoverResult` | libs/compass/src/check_pipeline.rs | struct | pub | 61 | pub struct HoverResult { |
| `hover` | libs/compass/src/check_pipeline.rs | function | pub | 72 | pub fn hover(inferencer: &DeepTypeInferencer, file: &Path, symbol: &str) -> Option<HoverResult> { |
| `run_check_pipeline` | libs/compass/src/check_pipeline.rs | function | pub | 97 | pub fn run_check_pipeline( |
| `recheck_after_change` | libs/compass/src/check_pipeline.rs | function | pub | 114 | pub fn recheck_after_change( |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Check pipeline with cross-file type propagation (R4, R5, R10).
//!
//! Wires `PropagationPipeline` into the analysis flow so that:
//! - `type_at` queries on imported symbols return the propagated type (R4)
//! - `hover` queries on imported symbols return the propagated type signature (R5)
//! - `check_paths` diagnostics benefit from cross-file types (R10)
//!
//! This is the integration layer between per-file type inference and
//! cross-file propagation.  After lens dissolution this replaces the
//! former `lens/mod.rs` check pipeline.

use std::path::{Path, PathBuf};

use crate::graph::ImportGraph;
use crate::type_inference::Type;
use crate::type_inference::{
    DeepTypeInferencer, PropagationPipeline, PropagationRequest, PropagationResult,
};

// ---------------------------------------------------------------------------
// Type-at query (R4)
// ---------------------------------------------------------------------------

/// Result of a `type_at` query.
#[derive(Debug, Clone)]
pub struct TypeAtResult {
    /// The resolved type.
    pub ty: Type,
    /// Source file where the type is originally defined.
    pub source_file: PathBuf,
    /// Symbol name.
    pub symbol: String,
    /// Whether the result came from cross-file propagation.
    pub is_propagated: bool,
}

/// Query the type of a symbol at a given file, returning propagated types for
/// imported symbols instead of `Type::Unknown` (R4).
///
/// If the symbol was propagated from another file, returns the resolved type
/// from the source module.
pub fn type_at(inferencer: &DeepTypeInferencer, file: &Path, symbol: &str) -> Option<TypeAtResult> {
    let file_buf = file.to_path_buf();
    let fa = inferencer.file_analysis(&file_buf)?;
    let binding = fa.symbols.get(symbol)?;

    Some(TypeAtResult {
        ty: binding.ty.clone(),
        source_file: binding.source_file.clone(),
        symbol: binding.symbol.clone(),
        is_propagated: binding.is_propagated,
    })
}

// ---------------------------------------------------------------------------
// Hover query (R5)
// ---------------------------------------------------------------------------

/// Result of a `hover` query.
#[derive(Debug, Clone)]
pub struct HoverResult {
    /// Human-readable type signature string.
    pub type_signature: String,
    /// Source file where the symbol is defined.
    pub source_file: PathBuf,
    /// Whether the type came from cross-file propagation.
    pub is_propagated: bool,
}

/// Query hover information for a symbol, returning the propagated type
/// signature for imported symbols (R5).
pub fn hover(inferencer: &DeepTypeInferencer, file: &Path, symbol: &str) -> Option<HoverResult> {
    let file_buf = file.to_path_buf();
    let fa = inferencer.file_analysis(&file_buf)?;
    let binding = fa.symbols.get(symbol)?;

    Some(HoverResult {
        type_signature: format!("{}: {:?}", binding.symbol, binding.ty),
        source_file: binding.source_file.clone(),
        is_propagated: binding.is_propagated,
    })
}

// ---------------------------------------------------------------------------
// Pipeline: check with propagation (R10)
// ---------------------------------------------------------------------------

/// Run the full check pipeline with cross-file type propagation.
///
/// 1. Ensure all files are registered in the inferencer.
/// 2. Build an `ImportGraph` and run `PropagationPipeline`.
/// 3. Return the propagation result so callers can inspect diagnostics
///    and propagated types.
///
/// This is meant to be called *after* per-file inference has populated
/// `FileAnalysis` entries in the inferencer.
pub fn run_check_pipeline(
    files: &[PathBuf],
    inferencer: &mut DeepTypeInferencer,
    file_import_graph: &ImportGraph,
) -> PropagationResult {
    let request = PropagationRequest {
        files: files.to_vec(),
        changed_files: Vec::new(), // full propagation
    };

    PropagationPipeline::run(&request, inferencer, file_import_graph)
}

/// Incremental re-check after a file change.
///
/// Invalidates propagated types from the changed file and re-propagates
/// through reverse import edges.
pub fn recheck_after_change(
    changed_file: &Path,
    inferencer: &mut DeepTypeInferencer,
    file_import_graph: &ImportGraph,
) -> PropagationResult {
    PropagationPipeline::invalidate_and_repropagate(changed_file, inferencer, file_import_graph)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_inference::TypeBinding;

    fn make_binding(symbol: &str, ty: Type, source: &str, propagated: bool) -> TypeBinding {
        TypeBinding {
            ty,
            source_file: PathBuf::from(source),
            symbol: symbol.to_string(),
            line: 1,
            is_exported: true,
            dependencies: vec![],
            is_propagated: propagated,
        }
    }

    #[test]
    fn test_type_at_local_symbol() {
        let mut inf = DeepTypeInferencer::new();
        let file = PathBuf::from("handler.py");
        inf.add_file(file.clone());
        inf.add_file_symbol(
            &file,
            "process".to_string(),
            make_binding("process", Type::Int, "handler.py", false),
        );

        let result = type_at(&inf, Path::new("handler.py"), "process").unwrap();
        assert_eq!(result.ty, Type::Int);
        assert!(!result.is_propagated);
    }

    #[test]
    fn test_type_at_propagated_symbol() {
        let mut inf = DeepTypeInferencer::new();
        let db = PathBuf::from("db.py");
        let handler = PathBuf::from("handler.py");
        inf.add_file(db.clone());
        inf.add_file(handler.clone());

        // Simulate propagation: get_user propagated from db.py to handler.py
        let callable_ty = Type::Callable {
            params: vec![],
            ret: Box::new(Type::Instance {
                name: "User".to_string(),
                module: None,
                type_args: vec![],
            }),
        };
        inf.add_file_symbol(
            &handler,
            "get_user".to_string(),
            make_binding("get_user", callable_ty.clone(), "db.py", true),
        );

        let result = type_at(&inf, Path::new("handler.py"), "get_user").unwrap();
        assert!(result.is_propagated);
        assert_eq!(result.source_file, PathBuf::from("db.py"));
        match &result.ty {
            Type::Callable { ret, .. } => match ret.as_ref() {
                Type::Instance { name, .. } => assert_eq!(name, "User"),
                other => panic!("Expected Instance, got {:?}", other),
            },
            other => panic!("Expected Callable, got {:?}", other),
        }
    }

    #[test]
    fn test_type_at_missing_symbol() {
        let inf = DeepTypeInferencer::new();
        assert!(type_at(&inf, Path::new("nonexistent.py"), "foo").is_none());
    }

    #[test]
    fn test_hover_propagated_symbol() {
        let mut inf = DeepTypeInferencer::new();
        let handler = PathBuf::from("handler.py");
        inf.add_file(handler.clone());
        inf.add_file_symbol(
            &handler,
            "get_user".to_string(),
            make_binding("get_user", Type::Int, "db.py", true),
        );

        let result = hover(&inf, Path::new("handler.py"), "get_user").unwrap();
        assert!(result.is_propagated);
        assert!(result.type_signature.contains("get_user"));
        assert!(result.type_signature.contains("Int"));
    }

    #[test]
    fn test_hover_missing_symbol() {
        let inf = DeepTypeInferencer::new();
        assert!(hover(&inf, Path::new("x.py"), "missing").is_none());
    }

    #[test]
    fn test_run_check_pipeline_empty() {
        let mut inf = DeepTypeInferencer::new();
        let ig = ImportGraph::new();
        let result = run_check_pipeline(&[], &mut inf, &ig);
        assert_eq!(result.stats.files_analyzed, 0);
        assert_eq!(result.stats.symbols_propagated, 0);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/check_pipeline.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/check_pipeline.rs` captured during libs codegen standardization.
```
