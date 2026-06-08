---
id: projects-score-src-check-alignment-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: traceability-closure-gate
    claim: traceability-closure-gate
    coverage: full
    rationale: "Validation, migration, fillback, and alignment CLI surfaces support standardization and traceability gates."
---

# Standardized projects/agentic-workflow/src/cli/check_alignment.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/cli/check_alignment.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `run` | projects/agentic-workflow/src/cli/check_alignment.rs | function | pub | 17 | run(path: Option<&str>, json: bool) -> Result<()> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/cli/check_alignment.rs -->
```rust
//! check-alignment command handler.
//!
//! Validates spec files for format compliance and logical consistency.
//! Calls `spec_alignment::check()` and formats output (text or JSON).

use colored::Colorize;
use agentic_workflow::spec_alignment;
use agentic_workflow::Result;
use std::path::PathBuf;

// Run check-alignment for the given path (or configured tech-design root).
///
// Prints results in text or JSON format, exits non-zero if any violations found.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/check_alignment.md#source
pub fn run(path: Option<&str>, json: bool) -> Result<()> {
    let project_root = crate::find_project_root()?;

    let target_path = match path {
        Some(p) => PathBuf::from(p),
        None => agentic_workflow::shared::workspace::tech_design_path(&project_root),
    };

    if !target_path.exists() {
        if json {
            let result = spec_alignment::CheckResult {
                files: Vec::new(),
                total_violations: 0,
                passed: true,
                coverage: None,
            };
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{}",
                format!("Path not found: {}", target_path.display()).yellow()
            );
        }
        return Ok(());
    }

    let result = spec_alignment::check(&target_path);

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        for file_result in &result.files {
            if file_result.status == "ok" {
                println!("{}", format!("OK    {}", file_result.path).green());
            } else {
                println!("{}", format!("FAIL  {}", file_result.path).red().bold());
                for violation in &file_result.violations {
                    println!(
                        "  {}: {}",
                        format!("{}", violation.kind).yellow(),
                        violation.message
                    );
                }
            }
        }

        if result.passed {
            println!(
                "\n{}",
                format!("All {} file(s) passed.", result.files.len())
                    .green()
                    .bold()
            );
        } else {
            eprintln!(
                "\n{}",
                format!(
                    "{} violation(s) found across {} file(s).",
                    result.total_violations,
                    result.files.iter().filter(|f| f.status == "fail").count()
                )
                .red()
                .bold()
            );
        }
    }

    if !result.passed {
        std::process::exit(1);
    }

    Ok(())
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/check_alignment.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
```
