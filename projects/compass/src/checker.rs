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
