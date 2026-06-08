---
id: projects-score-src-validate-spec-structure-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: traceability-closure-gate
    claim: traceability-closure-gate
    coverage: full
    rationale: "Validation, migration, fillback, and alignment CLI surfaces support standardization and traceability gates."
---

# Standardized projects/agentic-workflow/src/cli/validate_spec_structure.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/cli/validate_spec_structure.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `run` | projects/agentic-workflow/src/cli/validate_spec_structure.rs | function | pub | 43 | run(path: Option<&str>, json: bool) -> Result<()> |
| `run_all` | projects/agentic-workflow/src/cli/validate_spec_structure.rs | function | pub | 131 | run_all() -> Result<()> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/cli/validate_spec_structure.rs -->
```rust
//! validate-spec-structure command
//!
//! Lints a spec root against canonical structure rules (defined in
//! logic/spec-structure.md). Hard error — exits non-zero on any violation.
//! No lenient/warning mode.

use colored::Colorize;
use agentic_workflow::Result;
use std::path::{Path, PathBuf};

// Only README.md is allowed as a loose file directly at the spec root.
const ALLOWED_ROOT_FILES: &[&str] = &["README.md"];

// Allowed top-level subdirectory names inside a spec root.
const ALLOWED_TOP_DIRS: &[&str] = &[
    "interfaces",
    "logic",
    "config",
    "tools",
    "skills",
    "generate",
    "semantic",
];

// A single structure violation found during validation.
struct Violation {
    /// Violation kind (e.g. "loose_root_file", "unexpected_subdir").
    kind: &'static str,
    /// Relative path (from spec root) of the offending entry.
    path: String,
    /// Human-readable description of the rule that was broken.
    message: String,
}

// Run validate-spec-structure for the given path (or all crate roots).
///
// Prints each violation and exits with code 1 if any are found.
// When `json` is true, violations are emitted as a JSON array instead of
// text lines.
// @spec projects/agentic-workflow/tech-design/surface/interfaces/src/validate_spec_structure.md#source
pub fn run(path: Option<&str>, json: bool) -> Result<()> {
    let project_root = crate::find_project_root()?;

    let roots: Vec<PathBuf> = match path {
        Some(p) => vec![PathBuf::from(p)],
        None => discover_spec_roots(&project_root),
    };

    if roots.is_empty() {
        if json {
            println!("[]");
        } else {
            println!("{}", "No spec roots found to validate.".yellow());
        }
        return Ok(());
    }

    let mut all_violations: Vec<(PathBuf, Vec<Violation>)> = Vec::new();

    for root in &roots {
        let violations = validate_root(root)?;
        if !violations.is_empty() {
            all_violations.push((root.clone(), violations));
        }
    }

    if all_violations.is_empty() {
        if json {
            println!("[]");
        } else {
            println!("{}", "All spec roots valid.".green().bold());
        }
        return Ok(());
    }

    if json {
        // Emit all violations as a flat JSON array.
        let entries: Vec<String> = all_violations
            .iter()
            .flat_map(|(_, violations)| violations.iter())
            .map(|v| format!("{{\"kind\":\"{}\",\"path\":\"{}\"}}", v.kind, v.path))
            .collect();
        println!("[{}]", entries.join(","));
    } else {
        // Print violations grouped by spec root.
        for (root, violations) in &all_violations {
            println!(
                "\n{}",
                format!("Violations in {}:", root.display()).red().bold()
            );
            for v in violations {
                if v.kind == "loose_root_file" {
                    println!("loose_root_file: {}", v.path);
                } else {
                    println!("  {} {}: {}", "✗".red(), v.path, v.message);
                }
            }
        }

        let total: usize = all_violations.iter().map(|(_, v)| v.len()).sum();
        eprintln!(
            "\n{}",
            format!("{} violation(s) found — spec structure invalid.", total)
                .red()
                .bold()
        );
    }

    // Hard error: exit non-zero.
    std::process::exit(1);
}

// Read-only batch mode (R7): walk every `.md` file under
// `.aw/tech-design/` and run the rule registry against each.
///
// Output format (one line per finding):
//   `{file}:{line}: [{rule_short}] {message}`
///
// where `{file}` is path-from-project-root with forward slashes, `{line}`
// is 1-indexed (or `0` when unknown), and `{rule_short}` is the
// `RuleId::short()` label (e.g. `R3h:section-format`).
///
// Exit code is always 0 on success regardless of whether findings exist
// — the caller is meant to grep this stream into a backlog. Non-zero
// exits are reserved for runner-internal errors (R8 still applies at
// the apply gates; this is the read-only walk).
///
// @spec projects/agentic-workflow/tech-design/core/validate/section-format-rule.md#requirements
pub fn run_all() -> Result<()> {
    let project_root = crate::find_project_root()?;
    let target = agentic_workflow::shared::workspace::tech_design_path(&project_root);

    let shape = agentic_workflow::validate::PathShape::Prefix(target.clone());
    let files = match agentic_workflow::validate::resolve_spec_files(&shape) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("validate --all: failed to walk {}: {}", target.display(), e);
            std::process::exit(2);
        }
    };

    if files.is_empty() {
        return Ok(());
    }

    let report = agentic_workflow::validate::run_rules(&files);
    for f in &report.findings {
        let rel: PathBuf = f
            .file
            .strip_prefix(&project_root)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| f.file.clone());
        let rel_str = rel
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        let line = f.line.unwrap_or(0);
        println!("{}:{}: [{}] {}", rel_str, line, f.rule.short(), f.message,);
    }

    Ok(())
}

// Discover configured project spec roots, with legacy crate roots as fallback.
fn discover_spec_roots(project_root: &Path) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = agentic_workflow::shared::workspace::project_tech_design_paths(project_root)
        .into_iter()
        .map(|(_, path)| path)
        .filter(|path| path.exists())
        .collect();

    if roots.is_empty() {
        let specs_dir = agentic_workflow::shared::workspace::tech_design_path(project_root).join("crates");
        if let Ok(entries) = std::fs::read_dir(&specs_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    roots.push(p);
                }
            }
        }
    }

    roots.sort();
    roots.dedup();
    roots
}

// Validate a single spec root and return all violations found.
fn validate_root(root: &Path) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();

    if !root.exists() {
        violations.push(Violation {
            kind: "io_error",
            path: root.display().to_string(),
            message: "spec root does not exist".to_string(),
        });
        return Ok(violations);
    }

    let entries = match std::fs::read_dir(root) {
        Ok(e) => e,
        Err(err) => {
            violations.push(Violation {
                kind: "io_error",
                path: root.display().to_string(),
                message: format!("cannot read spec root: {}", err),
            });
            return Ok(violations);
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if path.is_file() {
            // Rule: only README.md is allowed as a loose .md file at the spec root.
            if name.ends_with(".md") && !ALLOWED_ROOT_FILES.contains(&name.as_str()) {
                violations.push(Violation {
                    kind: "loose_root_file",
                    path: name.clone(),
                    message: "loose file at spec root — only README.md is allowed; \
                              move to interfaces/, logic/, or another canonical subdir"
                        .to_string(),
                });
            }
        } else if path.is_dir() {
            // Rule: only canonical top-level subdirectory names are allowed.
            if !ALLOWED_TOP_DIRS.contains(&name.as_str()) {
                violations.push(Violation {
                    kind: "unexpected_subdir",
                    path: name.clone(),
                    message: format!(
                        "unexpected subdirectory '{}'; allowed top-level dirs: {}",
                        name,
                        ALLOWED_TOP_DIRS.join(", ")
                    ),
                });
            }

            // Rule: no loose files directly inside interfaces/ — they must be
            // in a protocol subdir (mcp/, cli/, rest/, etc.).
            if name == "interfaces" {
                if let Ok(iface_entries) = std::fs::read_dir(&path) {
                    for iface_entry in iface_entries.flatten() {
                        let iface_path = iface_entry.path();
                        let iface_name = iface_path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        if iface_path.is_file() {
                            violations.push(Violation {
                                kind: "loose_interfaces_file",
                                path: format!("interfaces/{}", iface_name),
                                message:
                                    "loose file inside interfaces/ — files must be placed in a \
                                     protocol subdir (mcp/, cli/, rest/, etc.)"
                                        .to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(violations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ── validate_root ────────────────────────────────────────────────────────

    #[test]
    fn valid_structure_produces_no_violations() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("interfaces/mcp")).unwrap();
        std::fs::create_dir_all(root.join("logic")).unwrap();
        std::fs::write(root.join("README.md"), "# test\n").unwrap();

        let violations = validate_root(root).unwrap();
        assert!(
            violations.is_empty(),
            "expected no violations but got: {:?}",
            violations.iter().map(|v| &v.path).collect::<Vec<_>>()
        );
    }

    #[test]
    fn nonexistent_root_reports_violation() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("does-not-exist");

        let violations = validate_root(&root).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("does not exist"));
    }

    #[test]
    fn readme_at_root_is_allowed() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        std::fs::write(root.join("README.md"), "# ok\n").unwrap();

        let violations = validate_root(root).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn loose_file_at_root_reports_violation() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        std::fs::write(root.join("commands.md"), "# commands\n").unwrap();

        let violations = validate_root(root).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].path, "commands.md");
        assert!(violations[0].message.contains("loose file at spec root"));
    }

    #[test]
    fn canonical_top_level_subdirs_are_allowed() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("interfaces")).unwrap();
        std::fs::create_dir_all(root.join("logic")).unwrap();
        std::fs::create_dir_all(root.join("config")).unwrap();
        std::fs::create_dir_all(root.join("semantic")).unwrap();

        let violations = validate_root(root).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn unexpected_subdir_reports_violation() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("docs")).unwrap();

        let violations = validate_root(root).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].path, "docs");
        assert!(violations[0].message.contains("unexpected subdirectory"));
    }

    #[test]
    fn loose_file_inside_interfaces_reports_violation() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("interfaces")).unwrap();
        std::fs::write(root.join("interfaces/commands.md"), "# cmd\n").unwrap();

        let violations = validate_root(root).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].path, "interfaces/commands.md");
        assert!(violations[0]
            .message
            .contains("loose file inside interfaces/"));
    }

    #[test]
    fn file_inside_interfaces_protocol_subdir_is_valid() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("interfaces/mcp")).unwrap();
        std::fs::write(root.join("interfaces/mcp/tools.md"), "# tools\n").unwrap();

        let violations = validate_root(root).unwrap();
        assert!(violations.is_empty());
    }

    #[test]
    fn multiple_violations_all_reported() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        // Two loose files at root (neither is README.md)
        std::fs::write(root.join("state.md"), "# state\n").unwrap();
        std::fs::write(root.join("logic.md"), "# logic\n").unwrap();

        let violations = validate_root(root).unwrap();
        assert_eq!(violations.len(), 2);
    }

    // ── discover_spec_roots ──────────────────────────────────────────────────

    #[test]
    fn discover_returns_empty_when_specs_dir_absent() {
        let tmp = TempDir::new().unwrap();
        let roots = discover_spec_roots(tmp.path());
        assert!(roots.is_empty());
    }

    #[test]
    fn discover_returns_crate_directories() {
        let tmp = TempDir::new().unwrap();
        let specs_dir = tmp.path().join(".aw/tech-design/crates");
        std::fs::create_dir_all(specs_dir.join("sdd")).unwrap();
        std::fs::create_dir_all(specs_dir.join("jet")).unwrap();

        let roots = discover_spec_roots(tmp.path());
        assert_eq!(roots.len(), 2);
        // Returned results are sorted
        assert!(roots[0].ends_with("jet"));
        assert!(roots[1].ends_with("sdd"));
    }

    #[test]
    fn discover_prefers_configured_project_td_paths() {
        let tmp = TempDir::new().unwrap();
        let score_dir = tmp.path().join(".aw");
        std::fs::create_dir_all(&score_dir).unwrap();
        std::fs::create_dir_all(tmp.path().join("projects/cgdb/tech_design")).unwrap();
        std::fs::write(
            score_dir.join("config.toml"),
            r#"
[agentic_workflow.tech_design_platform]
path = ".aw/tech-design"

[[projects]]
name = "cgdb"
path = "projects/cgdb"
td_path = "projects/cgdb/tech_design"
"#,
        )
        .unwrap();

        let roots = discover_spec_roots(tmp.path());

        assert_eq!(roots, vec![tmp.path().join("projects/cgdb/tech_design")]);
    }

    #[test]
    fn discover_ignores_files_in_crates_dir() {
        let tmp = TempDir::new().unwrap();
        let specs_dir = tmp.path().join(".aw/tech-design/crates");
        std::fs::create_dir_all(&specs_dir).unwrap();
        // A plain file alongside crate dirs must not be returned
        std::fs::write(specs_dir.join("README.md"), "# index\n").unwrap();
        std::fs::create_dir_all(specs_dir.join("sdd")).unwrap();

        let roots = discover_spec_roots(tmp.path());
        assert_eq!(roots.len(), 1);
        assert!(roots[0].ends_with("sdd"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/cli/validate_spec_structure.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
```
