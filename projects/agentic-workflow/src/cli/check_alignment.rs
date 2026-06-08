// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/interfaces/src/check_alignment.md#source
// CODEGEN-BEGIN
//! check-alignment command handler.
//!
//! Validates spec files for format compliance and logical consistency.
//! Calls `spec_alignment::check()` and formats output (text or JSON).

use crate::spec_alignment;
use crate::Result;
use colored::Colorize;
use std::path::PathBuf;

// Run check-alignment for the given path (or configured tech-design root).
///
// Prints results in text or JSON format, exits non-zero if any violations found.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/check_alignment.md#source
pub fn run(path: Option<&str>, json: bool) -> Result<()> {
    let project_root = crate::find_project_root()?;

    let target_path = match path {
        Some(p) => PathBuf::from(p),
        None => crate::shared::workspace::tech_design_path(&project_root),
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

// CODEGEN-END
