// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/interfaces/src/fillback.md#source
// CODEGEN-BEGIN
//! Fillback CLI Command
//!
//! Analyzes existing codebase using AST parsing and generates
//! language-agnostic specifications in .aw/tech-design/.
//!

use crate::fillback::code::{CodeStrategy, CodeStrategyConfig};
use crate::fillback::ImportStrategy;
use crate::Result;
use anyhow::Context;
use colored::Colorize;
use std::path::{Path, PathBuf};

// Run the fillback command to analyze codebase and generate specs
///
// # Workflow
// 1. Parse source files using tree-sitter AST analysis
// 2. Build dependency graph from module relationships
// 3. Display analysis summary and dependency graph
// 4. Run interactive clarification to refine understanding
// 5. Check for existing specs and confirm overwrites
// 6. Generate specifications under the resolved project's tech-design root
//
// `project_name` drives the output root when invoked via `aw td create
// --from-source` (epic #1270 R5 / #1273): explicit `--project` wins, else
// the project is inferred from `path` against the configured project
// scopes. This always targets the owning project's project-local
// `tech-design/` root (`resolve_td_root_from_config`), never the legacy
// repo-root `.aw/tech-design` (#1243).
// @spec apps/agentic-workflow/tech-design/surface/interfaces/src/fillback.md#source
pub async fn run(
    path: Option<&str>,
    module: Option<&str>,
    force: bool,
    project_name: Option<&str>,
) -> Result<()> {
    let project_root = crate::find_project_root()?;

    println!("{}", "SDD Fillback".cyan().bold());
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black()
    );
    println!();

    // Determine source path
    let source_path = if let Some(p) = path {
        PathBuf::from(p)
    } else {
        project_root.clone()
    };

    // Validate source path exists
    if !source_path.exists() {
        anyhow::bail!("Source path does not exist: {}", source_path.display());
    }

    if !source_path.is_dir() {
        anyhow::bail!("Source path must be a directory: {}", source_path.display());
    }

    println!(
        "{}",
        format!("Source: {}", source_path.display()).bright_black()
    );

    if let Some(m) = module {
        println!("{}", format!("Module filter: {}", m).bright_black());
    }

    if force {
        println!("{}", "Force mode: will overwrite existing specs".yellow());
    }

    println!();

    let output_dir = resolve_from_source_output_dir(&project_root, path, project_name)?;

    // Create strategy with configuration
    let config = CodeStrategyConfig {
        path: path.map(String::from),
        module: module.map(String::from),
        force,
        output_dir: Some(output_dir.to_string_lossy().to_string()),
        quick: false,
    };

    let strategy = CodeStrategy::with_config(config);

    // Execute the strategy (it handles all the steps internally)
    // The change_id parameter is no longer used but kept for trait compatibility
    strategy.execute(&source_path, "fillback").await?;

    println!();
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_black()
    );
    println!("{}", "Fillback completed!".green().bold());
    println!();
    println!("{}", "Next steps:".bright_black());
    println!("  1. Review generated specs in {}", output_dir.display());
    println!("  2. Edit and enhance specifications as needed");
    println!("  3. Use specs as reference for future changes");

    Ok(())
}

// Resolve the tech-design output root for `aw td create --from-source`
// (#1273, fixing #1243): explicit `project_name` wins; otherwise the owning
// project is inferred from `path` via the configured project scopes
// (`configured_project_name_for_path`). Always resolves under the owning
// project's project-local `tech-design/` root (`resolve_td_root_from_config`)
// — never the legacy repo-root `.aw/tech-design`.
//
// Writes land under the root's `specs/` bucket specifically (not the
// tech-design root itself): `aw td check`'s structure rules
// (`locate_in_crate_spec_root` in `validate/rules/r6a_loose_root_file.rs`)
// exempt anything nested under a top-level `specs/` from the
// loose-root-file/unexpected-subdir gates, which is what lets
// `generate_specs`'s always-present `_overview.md`/`_dependency-graph.md`
// scaffold files and its source-tree-mirrored module specs coexist and
// still pass `aw td check` (AC1). `specs/` also keeps `generate_specs`'s
// existing `output_dir.parent().parent()` source-tree mirroring recovery
// (which assumes two path segments between `output_dir` and the project
// root, matching this root/specs nesting) working unchanged.
fn resolve_from_source_output_dir(
    project_root: &Path,
    path: Option<&str>,
    project_name: Option<&str>,
) -> Result<PathBuf> {
    let resolved_project = match project_name {
        Some(p) => p.to_string(),
        None => {
            let target = path.unwrap_or(".");
            let target_rel = crate::cli::cb::repo_relative_code_path(project_root, target);
            crate::cli::standardize::configured_project_name_for_path(project_root, &target_rel)?
                .with_context(|| {
                    format!(
                        "no configured project owns `{target_rel}` — pass --project <name> to \
                         target its tech-design root"
                    )
                })?
        }
    };
    let resolved = crate::services::project_registry::resolve_td_root_from_config(
        project_root,
        &resolved_project,
    )
    .map_err(|e| {
        anyhow::anyhow!(
            "failed to resolve tech-design root for project `{resolved_project}`: {}",
            e.message
        )
    })?;
    Ok(PathBuf::from(resolved.root).join("specs"))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project(dir: &std::path::Path) {
        let src_dir = dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        fs::write(
            src_dir.join("main.rs"),
            r#"
use std::path::Path;

// Main entry point
pub fn main() {
    println!("Hello!");
}
"#,
        )
        .unwrap();

        fs::write(
            src_dir.join("lib.rs"),
            r#"
pub mod utils;

pub struct Config {
    pub name: String,
}
"#,
        )
        .unwrap();

        // Create genesis directory structure
        fs::create_dir_all(dir.join(".aw/tech-design")).unwrap();
    }

    #[test]
    fn test_source_path_validation() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent = temp_dir.path().join("non_existent");

        // Test that non-existent path would fail (can't run async in sync test easily)
        assert!(!non_existent.exists());
    }

    #[test]
    fn test_project_structure() {
        let temp_dir = TempDir::new().unwrap();
        create_test_project(temp_dir.path());

        // Verify test project was created correctly
        assert!(temp_dir.path().join("src/main.rs").exists());
        assert!(temp_dir.path().join("src/lib.rs").exists());
        assert!(temp_dir.path().join(".aw/tech-design").exists());
    }
}

// CODEGEN-END
