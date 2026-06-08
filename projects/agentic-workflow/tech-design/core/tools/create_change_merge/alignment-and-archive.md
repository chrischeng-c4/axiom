---
id: sdd-tools-create-change-merge-alignment-and-archive
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools create change merge alignment and archive

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/create_change_merge.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `execute_workflow` | projects/agentic-workflow/src/tools/create_change_merge.rs | function | pub | 69 | execute_workflow(args: &Value, project_root: &Path) -> Result<String> |
| `workflow_definition` | projects/agentic-workflow/src/tools/create_change_merge.rs | function | pub | 29 | workflow_definition() -> ToolDefinition |
## Source
<!-- type: source lang: rust -->

````rust
// ─── Alignment Check Helpers ─────────────────────────────────────────────────

/// Alignment warning from post-merge spec check.
struct AlignmentWarning {
    file: String,
    kind: String,
    message: String,
    heading: Option<String>,
    line: Option<usize>,
}

/// Run alignment checks on merged spec target paths.
///
/// Returns `(warnings, summary)` where summary is `Some` if there are violations,
/// e.g. "3 violation(s) in 2 file(s)".
fn run_alignment_checks(
    target_paths: &[std::path::PathBuf],
) -> (Vec<AlignmentWarning>, Option<String>) {
    let mut warnings = Vec::new();
    let mut files_with_violations = 0_usize;

    for path in target_paths {
        let check_result = crate::spec_alignment::check(path);
        let mut has_violations = false;
        for file_result in &check_result.files {
            for violation in &file_result.violations {
                has_violations = true;
                warnings.push(AlignmentWarning {
                    file: file_result.path.clone(),
                    kind: violation.kind.to_string(),
                    message: violation.message.clone(),
                    heading: violation.heading.clone(),
                    line: violation.line,
                });
            }
        }
        if has_violations {
            files_with_violations += 1;
        }
    }

    let summary = if warnings.is_empty() {
        None
    } else {
        Some(format!(
            "{} violation(s) in {} file(s)",
            warnings.len(),
            files_with_violations
        ))
    };

    (warnings, summary)
}

/// Append alignment warnings table to `implementation.md` in the archive.
///
/// Creates the file if it doesn't exist; appends if it does.
fn append_alignment_to_impl(archive_path: &Path, warnings: &[AlignmentWarning]) {
    use std::io::Write;
    let impl_path = archive_path.join("implementation.md");
    let mut file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&impl_path)
    {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!(
                path = %impl_path.display(),
                error = %e,
                "failed to open implementation.md for alignment warnings"
            );
            return;
        }
    };

    let files_checked = {
        let mut seen = std::collections::HashSet::new();
        for w in warnings {
            seen.insert(&w.file);
        }
        seen.len()
    };
    let mut content = String::from("\n\n## Alignment Warnings\n\n");
    content.push_str(&format!(
        "{} violation(s) found across {} spec(s).\n\n",
        warnings.len(),
        files_checked
    ));
    content.push_str("| File | Kind | Message |\n|------|------|---------|");
    for w in warnings {
        content.push_str(&format!("\n| {} | {} | {} |", w.file, w.kind, w.message));
    }
    content.push('\n');

    if let Err(e) = file.write_all(content.as_bytes()) {
        tracing::warn!(
            path = %impl_path.display(),
            error = %e,
            "failed to write alignment warnings to implementation.md"
        );
    }
}

// ─── Archive ─────────────────────────────────────────────────────────────────

/// Build archive path for a change.
fn build_archive_path(change_id: &str) -> String {
    format!(
        ".aw/archive/{}-{}",
        chrono::Utc::now().format("%Y%m%d"),
        change_id
    )
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/create_change_merge.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "AlignmentWarning"
      - "run_alignment_checks"
      - "append_alignment_to_impl"
      - "build_archive_path"
    description: "Post-merge alignment warnings and archive path construction."
```
