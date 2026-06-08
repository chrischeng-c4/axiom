---
id: projects-sdd-src-spec-alignment-annotations-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: traceability-closure-gate
    claim: traceability-closure-gate
    coverage: full
    rationale: "Spec alignment interfaces implement TD/source annotation and coverage checks used by the traceability closure gate."
---

# Standardized projects/agentic-workflow/src/spec_alignment/annotations.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/spec_alignment/annotations.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `scan_directories` | projects/agentic-workflow/src/spec_alignment/annotations.rs | function | pub | 51 | scan_directories(source_dirs: &[&Path]) -> Vec<SpecAnnotation> |
| `scan_file` | projects/agentic-workflow/src/spec_alignment/annotations.rs | function | pub | 24 | scan_file(source_file: &str, content: &str) -> Vec<SpecAnnotation> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:claim-code -->

<!-- source-snapshot: path=projects/agentic-workflow/src/spec_alignment/annotations.rs -->
```rust
//! @spec annotation parser.
//!
//! Language-agnostic scan for `@spec {path}#{id}` across comment syntaxes:
//! `//`, `#`, `--`, `<!-- -->`, `/* */`.
//!
//! Returns `Vec<SpecAnnotation>` with spec_path, requirement_id, source_file, line, comment_syntax.

use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

use super::models::SpecAnnotation;

/// Regex to capture `@spec {path}#{id}` where path ends with `.md` and id is `R\d+`.
static SPEC_ANNOTATION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"@spec\s+([\w/.\-]+\.md)#(R\d+)").unwrap());

/// Scan a single file for `@spec` annotations.
///
/// Returns all `SpecAnnotation` entries found, with 1-based line numbers.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/annotations.md#source
pub fn scan_file(source_file: &str, content: &str) -> Vec<SpecAnnotation> {
    let mut results = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        let line_number = idx + 1; // 1-based

        if let Some((comment_syntax, comment_body)) = extract_comment(line) {
            for cap in SPEC_ANNOTATION_RE.captures_iter(comment_body) {
                results.push(SpecAnnotation {
                    spec_path: cap[1].to_string(),
                    requirement_id: cap[2].to_string(),
                    source_file: source_file.to_string(),
                    line: line_number,
                    comment_syntax: comment_syntax.to_string(),
                });
            }
        }
    }

    results
}

/// Scan all source files in the given directories for `@spec` annotations.
///
/// Recursively collects source files (`.rs`, `.py`, `.sql`, `.md`, `.html`, `.js`, `.ts`,
/// `.go`, `.rb`, `.sh`, `.yaml`, `.yml`, `.toml`) and scans each.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/annotations.md#source
pub fn scan_directories(source_dirs: &[&Path]) -> Vec<SpecAnnotation> {
    let mut all_annotations = Vec::new();

    for dir in source_dirs {
        if !dir.is_dir() {
            continue;
        }
        let files = collect_source_files(dir);
        for file_path in files {
            let path_str = file_path.display().to_string();
            match std::fs::read_to_string(&file_path) {
                Ok(content) => {
                    let annotations = scan_file(&path_str, &content);
                    all_annotations.extend(annotations);
                }
                Err(_) => continue,
            }
        }
    }

    all_annotations
}

/// Extract comment body and syntax from a line.
///
/// Checks comment prefixes in order: `<!--`, `/*`, `//`, `#`, `--`.
/// Returns `(syntax, body)` where body is the content after the comment marker.
fn extract_comment(line: &str) -> Option<(&'static str, &str)> {
    let trimmed = line.trim();

    // HTML-style: <!-- ... -->
    if let Some(rest) = trimmed.strip_prefix("<!--") {
        return Some(("<!--", rest));
    }

    // C-style block: /* ... */
    if let Some(rest) = trimmed.strip_prefix("/*") {
        return Some(("/*", rest));
    }

    // C-style line: // ...
    if let Some(rest) = trimmed.strip_prefix("//") {
        return Some(("//", rest));
    }

    // SQL-style: -- ... (must check after <!-- to avoid false positives)
    // Disambiguate from `---` (YAML frontmatter) by requiring space after `--`
    if trimmed.starts_with("-- ")
        || (trimmed.starts_with("--") && trimmed.len() > 2 && !trimmed.starts_with("---"))
    {
        if let Some(rest) = trimmed.strip_prefix("--") {
            return Some(("--", rest));
        }
    }

    // Hash-style: # ... (must check after other prefixes to avoid conflicts)
    if let Some(rest) = trimmed.strip_prefix('#') {
        // Avoid matching markdown headings (## Heading)
        if !rest.starts_with('#') {
            return Some(("#", rest));
        }
    }

    None
}

/// Source file extensions to scan for annotations.
const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "py", "sql", "md", "html", "js", "ts", "tsx", "jsx", "go", "rb", "sh", "yaml", "yml",
    "toml", "css", "svelte", "vue",
];

/// Recursively collect source files under a directory.
fn collect_source_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    collect_source_files_recursive(dir, &mut files);
    files.sort();
    files
}

fn collect_source_files_recursive(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip hidden directories and common non-source directories
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }
            }
            collect_source_files_recursive(&path, files);
        } else if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if SOURCE_EXTENSIONS.contains(&ext) {
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
  - path: projects/agentic-workflow/src/spec_alignment/annotations.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:claim-code>"
    description: |
      Source template owns the complete spec-alignment annotation scanner.
```
