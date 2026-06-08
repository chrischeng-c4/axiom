---
id: projects-sdd-src-spec-alignment-check-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: traceability-closure-gate
    claim: traceability-closure-gate
    coverage: full
    rationale: "Spec alignment interfaces implement TD/source annotation and coverage checks used by the traceability closure gate."
---

# Standardized projects/agentic-workflow/src/spec_alignment/check.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/spec_alignment/check.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `check` | projects/agentic-workflow/src/spec_alignment/check.rs | function | pub | 22 | check(path: &Path) -> CheckResult |
| `check_with_coverage` | projects/agentic-workflow/src/spec_alignment/check.rs | function | pub | 49 | check_with_coverage(spec_dir: &Path, source_dirs: &[&Path]) -> CheckResult |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:claim-code -->

<!-- source-snapshot: path=projects/agentic-workflow/src/spec_alignment/check.rs -->
```rust
//! Entry point for spec alignment checking.
//!
//! Orchestrates: parse -> format checks -> logical checks -> aggregate results.
//! Phase 2 adds `check_with_coverage()` for annotation + requirement coverage analysis.

use std::path::Path;

use super::format_rules;
use super::logical_rules;
use super::models::{CheckResult, FileResult};
use super::parser;

/// Check a single file or directory for spec alignment violations.
///
/// If `path` is a file, checks that single file.
/// If `path` is a directory, recursively checks all `.md` files.
///
/// Returns a `CheckResult` with per-file results and aggregate statistics.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/check.md#source
pub fn check(path: &Path) -> CheckResult {
    let files = collect_files(path);
    let mut results = Vec::new();

    for file_path in &files {
        let result = check_single_file(file_path);
        results.push(result);
    }

    let total_violations: usize = results.iter().map(|r| r.violations.len()).sum();

    CheckResult {
        files: results,
        total_violations,
        passed: total_violations == 0,
        coverage: None,
    }
}

/// Check spec files and produce a coverage report.
///
/// Runs all Phase 1 checks (format + logical) plus Phase 2 analysis:
/// - Requirement↔scenario cross-reference (orphan detection)
/// - `@spec` annotation coverage analysis
/// - Schema↔struct validation stubs (daemon-dependent)
/// - Nested schema conflict detection (via logical rules)
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/check.md#source
pub fn check_with_coverage(spec_dir: &Path, source_dirs: &[&Path]) -> CheckResult {
    let files = collect_files(spec_dir);
    let mut results = Vec::new();
    let mut all_orphans = Vec::new();
    let mut all_requirements = std::collections::HashMap::new();

    for file_path in &files {
        let path_str = file_path.display().to_string();

        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                results.push(FileResult {
                    path: path_str,
                    status: "fail".to_string(),
                    violations: vec![super::models::Violation {
                        kind: super::models::ViolationKind::IoError,
                        message: format!("Failed to read file: {}", e),
                        heading: None,
                        line: None,
                        lines: None,
                        name: None,
                        expected_lang: None,
                        field: None,
                        details: None,
                    }],
                });
                continue;
            }
        };

        let doc = parser::parse(&path_str, &content);

        // Phase 1: format + logical checks
        let mut violations = format_rules::check(&doc);
        violations.extend(logical_rules::check(&doc));

        // Phase 2: requirement↔scenario cross-reference
        let (req_violations, orphans) =
            super::requirement_coverage::check_with_content(&doc, &content);
        violations.extend(req_violations);
        all_orphans.extend(orphans);

        // Collect requirement IDs for coverage analysis
        let reqs = super::requirement_coverage::extract_requirement_ids_from_content(&content);
        if !reqs.is_empty() {
            all_requirements.insert(path_str.clone(), reqs);
        }

        let status = if violations.is_empty() {
            "ok".to_string()
        } else {
            "fail".to_string()
        };

        results.push(FileResult {
            path: path_str,
            status,
            violations,
        });
    }

    // Phase 2: schema↔struct validation (daemon-dependent stub)
    let daemon_ready = false;
    let (schema_violations, schema_mismatches) =
        super::schema_struct::check(spec_dir, daemon_ready);
    // TODO: integrate schema_violations into file results when daemon is ready
    let _ = schema_violations;

    // Phase 2: coverage analysis with pre-computed requirements
    let report = super::coverage::analyze_with_precomputed(
        spec_dir,
        source_dirs,
        all_orphans,
        all_requirements,
        schema_mismatches,
        daemon_ready,
    );

    let total_violations: usize = results.iter().map(|r| r.violations.len()).sum();
    let passed = total_violations == 0 && report.uncovered_requirements.is_empty();

    CheckResult {
        files: results,
        total_violations,
        passed,
        coverage: Some(report),
    }
}

/// Check a single file and return a `FileResult`.
fn check_single_file(path: &Path) -> FileResult {
    let path_str = path.display().to_string();

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return FileResult {
                path: path_str,
                status: "fail".to_string(),
                violations: vec![super::models::Violation {
                    kind: super::models::ViolationKind::IoError,
                    message: format!("Failed to read file: {}", e),
                    heading: None,
                    line: None,
                    lines: None,
                    name: None,
                    expected_lang: None,
                    field: None,
                    details: None,
                }],
            };
        }
    };

    let doc = parser::parse(&path_str, &content);

    // Run format checks
    let mut violations = format_rules::check(&doc);

    // Run logical checks
    violations.extend(logical_rules::check(&doc));

    let status = if violations.is_empty() {
        "ok".to_string()
    } else {
        "fail".to_string()
    };

    FileResult {
        path: path_str,
        status,
        violations,
    }
}

/// Collect files to check.
///
/// If `path` is a file, returns it directly.
/// If `path` is a directory, recursively collects all `.md` files.
fn collect_files(path: &Path) -> Vec<std::path::PathBuf> {
    if path.is_file() {
        return vec![path.to_path_buf()];
    }

    if !path.is_dir() {
        return Vec::new();
    }

    let mut files = Vec::new();
    collect_md_files_recursive(path, &mut files);
    files.sort();
    files
}

/// Recursively collect all `.md` files under a directory.
fn collect_md_files_recursive(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_md_files_recursive(&path, files);
        } else if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "md" {
                    files.push(path);
                }
            }
        }
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/spec_alignment/check.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:claim-code>"
    description: |
      Source template owns the complete spec-alignment check module.
```
