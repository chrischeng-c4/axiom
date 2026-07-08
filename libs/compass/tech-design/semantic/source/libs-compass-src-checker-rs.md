---
id: libs-compass-src-checker-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/checker.rs`.
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

# Standardized libs/compass/src/checker.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/checker.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `check_paths` | libs/compass/src/checker.rs | function | pub | 17 | pub fn check_paths(paths: &[&Path], config: &LintConfig) -> Vec<FileResult> { |
| `check_paths_with_propagation` | libs/compass/src/checker.rs | function | pub | 52 | pub fn check_paths_with_propagation( |
| `FileResult` | libs/compass/src/checker.rs | struct | pub | 169 | pub struct FileResult { |
| `has_errors` | libs/compass/src/checker.rs | function | pub | 176 | pub fn has_errors(&self) -> bool { |
| `error_count` | libs/compass/src/checker.rs | function | pub | 182 | pub fn error_count(&self) -> usize { |
| `warning_count` | libs/compass/src/checker.rs | function | pub | 189 | pub fn warning_count(&self) -> usize { |
| `LintConfig` | libs/compass/src/checker.rs | struct | pub | 199 | pub struct LintConfig { |
| `is_language_enabled` | libs/compass/src/checker.rs | function | pub | 222 | pub fn is_language_enabled(&self, lang: Language) -> bool { |
| `is_excluded` | libs/compass/src/checker.rs | function | pub | 226 | pub fn is_excluded(&self, path: &Path) -> bool { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Top-level file checking orchestrator
//!
//! Provides the public `check_paths` API and supporting types (`FileResult`,
//! `LintConfig`) that were formerly in `lens/mod.rs`.

use crate::diagnostic::{Diagnostic, DiagnosticSeverity};
use crate::graph::ImportGraph;
use crate::lint::CheckerRegistry;
use crate::syntax::{Language, MultiParser, ParsedFile};
use crate::type_inference::{
    DeepTypeInferencer, PropagationPipeline, PropagationRequest, PropagationResult,
};

use std::path::{Path, PathBuf};

/// Check files and return diagnostics
pub fn check_paths(paths: &[&Path], config: &LintConfig) -> Vec<FileResult> {
    let registry = CheckerRegistry::new();

    // Initialize parser, return empty results on failure
    let mut parser = match MultiParser::new() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to initialize parser: {}", e);
            return Vec::new();
        }
    };

    let mut results = Vec::new();

    for path in paths {
        if path.is_file() {
            if let Some(result) = check_file(&mut parser, &registry, path, config) {
                results.push(result);
            }
        } else if path.is_dir() {
            results.extend(check_directory(&mut parser, &registry, path, config));
        }
    }

    results
}

/// Check files with cross-file type propagation (R10).
///
/// After running per-file checks, builds an ImportGraph across all checked
/// files, runs `PropagationPipeline` in topological order, and returns both
/// the lint results and the propagation summary.
///
/// This is the preferred entry point when cross-file type resolution is
/// desired (e.g., for `type_at` / `hover` accuracy).
pub fn check_paths_with_propagation(
    paths: &[&Path],
    config: &LintConfig,
    project_root: &Path,
) -> (Vec<FileResult>, PropagationResult) {
    // Phase 1: per-file lint + analysis (unchanged).
    let results = check_paths(paths, config);

    // Phase 2: collect sources for import graph + propagation.
    let mut file_sources: Vec<(PathBuf, String)> = Vec::new();
    for r in &results {
        if let Ok(src) = std::fs::read_to_string(&r.path) {
            file_sources.push((r.path.clone(), src));
        }
    }

    if file_sources.is_empty() {
        return (
            results,
            PropagationResult {
                propagated: Default::default(),
                cycles: Vec::new(),
                stats: Default::default(),
            },
        );
    }

    // Phase 3: build ImportGraph.
    let import_graph = ImportGraph::build(&file_sources, project_root);

    // Phase 4: run propagation pipeline.
    let mut inferencer = DeepTypeInferencer::new();
    let all_files: Vec<PathBuf> = file_sources.iter().map(|(p, _)| p.clone()).collect();

    let request = PropagationRequest {
        files: all_files,
        changed_files: Vec::new(), // full propagation
    };

    let propagation = PropagationPipeline::run(&request, &mut inferencer, &import_graph);

    (results, propagation)
}

/// Check a single file
fn check_file(
    parser: &mut MultiParser,
    registry: &CheckerRegistry,
    path: &Path,
    config: &LintConfig,
) -> Option<FileResult> {
    let language = MultiParser::detect_language(path)?;

    if !config.is_language_enabled(language) {
        return None;
    }

    let source = std::fs::read_to_string(path).ok()?;

    // Some languages (Dockerfile, Markdown, Mermaid) use line-based analysis without tree-sitter.
    // SQL, Proto, GraphQL, TOML now have real AST grammars (R3) so parser.parse() handles them;
    // we fall back to line_based only for the remaining line-only languages.
    let parsed = if let Some(p) = parser.parse(&source, language) {
        p
    } else if matches!(
        language,
        Language::Dockerfile | Language::Markdown | Language::Mdx | Language::Mermaid
    ) {
        // Create a minimal ParsedFile for line-based checkers
        ParsedFile::line_based(source, language)
    } else {
        return None;
    };

    let checker = registry.get(language)?;
    let diagnostics = checker.check(&parsed, config);

    Some(FileResult {
        path: path.to_path_buf(),
        language,
        diagnostics,
    })
}

/// Check all files in a directory
fn check_directory(
    parser: &mut MultiParser,
    registry: &CheckerRegistry,
    dir: &Path,
    config: &LintConfig,
) -> Vec<FileResult> {
    use jwalk::WalkDir;

    let mut results = Vec::new();

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Skip excluded patterns
        if config.is_excluded(&path) {
            continue;
        }

        if let Some(result) = check_file(parser, registry, &path, config) {
            results.push(result);
        }
    }

    results
}

/// Result of checking a single file
#[derive(Debug)]
pub struct FileResult {
    pub path: std::path::PathBuf,
    pub language: Language,
    pub diagnostics: Vec<Diagnostic>,
}

impl FileResult {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Error)
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Warning)
            .count()
    }
}

/// Lint configuration
#[derive(Debug, Clone)]
pub struct LintConfig {
    pub languages: Vec<Language>,
    pub exclude_patterns: Vec<String>,
    pub min_severity: DiagnosticSeverity,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            languages: vec![Language::Python, Language::TypeScript, Language::Rust],
            exclude_patterns: vec![
                "__pycache__".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
                ".venv".to_string(),
            ],
            min_severity: DiagnosticSeverity::Warning,
        }
    }
}

impl LintConfig {
    pub fn is_language_enabled(&self, lang: Language) -> bool {
        self.languages.contains(&lang)
    }

    pub fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.exclude_patterns.iter().any(|p| path_str.contains(p))
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/checker.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/checker.rs` captured during libs codegen standardization.
```
