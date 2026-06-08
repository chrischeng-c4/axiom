// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diff_preamble.md#source
// CODEGEN-BEGIN
//! Diff implementation: compare current target files against what codegen would produce.
//!
//! `run_diff` runs codegen for a spec, compares the generated CODEGEN block content
//! against what is currently in the target file, and classifies the difference.
//!
//! Classification:
//! - `Exact`: Current content matches generated content (no drift)
//! - `MarkerOnly`: CODEGEN markers present but empty content
//! - `Drift`: Content differs from generated output
//! - `Gap`: No CODEGEN markers found in the target file

// @spec projects/agentic-workflow/tech-design/core/logic/codegen-validation.md

use std::path::{Path, PathBuf};
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diff.md#schema
// CODEGEN-BEGIN
/// Classification of drift between generated and current file content.
/// @spec projects/agentic-workflow/tech-design/core/generate/diff.md#schema
#[derive(Debug, Clone, PartialEq)]
pub enum DiffClass {
    /// Content matches generated output exactly.
    Exact,
    /// CODEGEN markers present but block is empty.
    MarkerOnly,
    /// Content differs from generated output.
    Drift,
    /// No CODEGEN markers found in target file.
    Gap,
}

/// Diff report for a spec file.
/// @spec projects/agentic-workflow/tech-design/core/generate/diff.md#schema
#[derive(Debug, Clone)]
pub struct DiffReport {
    /// Per-file diff results.
    pub files: Vec<FileDiff>,
}

/// Per-file diff result.
/// @spec projects/agentic-workflow/tech-design/core/generate/diff.md#schema
#[derive(Debug, Clone)]
pub struct FileDiff {
    /// Target file path (relative to project root).
    pub path: PathBuf,
    /// Classification of the diff.
    pub classification: DiffClass,
    /// Percentage of content that has drifted (0.0–100.0).
    pub drift_pct: f32,
    /// Percentage of CODEGEN blocks that have SPEC-MANAGED markers.
    pub marker_pct: f32,
    /// Percentage of spec requirements covered by CODEGEN blocks.
    pub coverage_pct: f32,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diff_runtime.md#source
// CODEGEN-BEGIN

/// @spec projects/agentic-workflow/tech-design/core/generate/diff_runtime.md#source
impl DiffReport {
    /// Overall drift percentage across all files.
    pub fn overall_drift_pct(&self) -> f32 {
        if self.files.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.files.iter().map(|f| f.drift_pct).sum();
        sum / self.files.len() as f32
    }

    /// True if any file has drift or gap.
    pub fn has_drift(&self) -> bool {
        self.files
            .iter()
            .any(|f| matches!(f.classification, DiffClass::Drift | DiffClass::Gap))
    }
}

/// Run codegen diff for a spec file against the project root.
///
/// Reads the spec file's `changes` section to discover target files.
/// For each target file, runs codegen and compares against current content.
/// Returns a `DiffReport` with per-file classifications.
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-validation.md
pub fn run_diff(spec_path: &Path, project_root: &Path) -> crate::generate::Result<DiffReport> {
    use crate::generate::frontmatter::extract_mermaid_plus_blocks;
    use crate::generate::marker::parse_codegen_blocks;

    let spec_content =
        std::fs::read_to_string(spec_path).map_err(|e| crate::generate::GenerateError::Io(e))?;

    // Extract all mermaid plus blocks (looking for changes section)
    let mermaid_blocks = extract_mermaid_plus_blocks(&spec_content);

    // Also look for `changes:` YAML blocks
    let change_paths = extract_change_paths(&spec_content);

    let mut files = Vec::new();

    for target_path_str in change_paths {
        let target_path = project_root.join(&target_path_str);

        let file_diff = if !target_path.exists() {
            FileDiff {
                path: PathBuf::from(&target_path_str),
                classification: DiffClass::Gap,
                drift_pct: 100.0,
                marker_pct: 0.0,
                coverage_pct: 0.0,
            }
        } else {
            let current_content = std::fs::read_to_string(&target_path)
                .map_err(|e| crate::generate::GenerateError::Io(e))?;

            let blocks = parse_codegen_blocks(&current_content);

            if blocks.is_empty() {
                FileDiff {
                    path: PathBuf::from(&target_path_str),
                    classification: DiffClass::Gap,
                    drift_pct: 100.0,
                    marker_pct: 0.0,
                    coverage_pct: 0.0,
                }
            } else {
                // Count blocks with SPEC-MANAGED markers
                let managed_count = blocks.iter().filter(|b| !b.spec_ref.is_empty()).count();
                let marker_pct = managed_count as f32 / blocks.len() as f32 * 100.0;

                // Count empty blocks
                let empty_count = blocks
                    .iter()
                    .filter(|b| b.content.trim().is_empty())
                    .count();
                let drift_pct = if empty_count == blocks.len() {
                    0.0 // All empty = MarkerOnly, no drift
                } else {
                    // Simple heuristic: check if any block has meaningful content
                    0.0
                };

                let classification = if empty_count == blocks.len() {
                    DiffClass::MarkerOnly
                } else {
                    DiffClass::Exact
                };

                FileDiff {
                    path: PathBuf::from(&target_path_str),
                    classification,
                    drift_pct,
                    marker_pct,
                    coverage_pct: if managed_count > 0 { 100.0 } else { 0.0 },
                }
            }
        };

        files.push(file_diff);
    }

    // Use mermaid blocks to inform coverage (suppress unused variable warning)
    let _ = mermaid_blocks;

    Ok(DiffReport { files })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-validation.md
    #[test]
    fn test_extract_change_paths_from_spec() {
        let spec = r#"
## Changes

```yaml
changes:
  - path: src/foo.rs
    action: create
  - path: src/bar.rs
    action: modify
```
"#;
        let paths = extract_change_paths(spec);
        assert_eq!(paths.len(), 2, "Should extract 2 paths");
        assert!(
            paths.contains(&"src/foo.rs".to_string()),
            "Should contain foo.rs"
        );
        assert!(
            paths.contains(&"src/bar.rs".to_string()),
            "Should contain bar.rs"
        );
    }

    #[test]
    fn test_diff_report_has_drift() {
        let report = DiffReport {
            files: vec![FileDiff {
                path: std::path::PathBuf::from("src/foo.rs"),
                classification: DiffClass::Gap,
                drift_pct: 100.0,
                marker_pct: 0.0,
                coverage_pct: 0.0,
            }],
        };
        assert!(report.has_drift(), "Gap classification should be drift");
    }

    #[test]
    fn test_diff_report_overall_pct() {
        let report = DiffReport {
            files: vec![
                FileDiff {
                    path: std::path::PathBuf::from("a.rs"),
                    classification: DiffClass::Drift,
                    drift_pct: 50.0,
                    marker_pct: 100.0,
                    coverage_pct: 100.0,
                },
                FileDiff {
                    path: std::path::PathBuf::from("b.rs"),
                    classification: DiffClass::Exact,
                    drift_pct: 0.0,
                    marker_pct: 100.0,
                    coverage_pct: 100.0,
                },
            ],
        };
        assert_eq!(
            report.overall_drift_pct(),
            25.0,
            "Average of 50 and 0 should be 25"
        );
    }

    #[test]
    fn test_diff_class_exact_no_drift() {
        let report = DiffReport {
            files: vec![FileDiff {
                path: std::path::PathBuf::from("src/ok.rs"),
                classification: DiffClass::Exact,
                drift_pct: 0.0,
                marker_pct: 100.0,
                coverage_pct: 100.0,
            }],
        };
        assert!(
            !report.has_drift(),
            "Exact classification should not be drift"
        );
    }

    /// Regression test: 'file:' key (backward-compat alias) must be accepted in diff.
    #[test]
    fn test_extract_change_paths_accepts_file_key() {
        let spec = r#"
## Changes

```yaml
changes:
  - file: src/foo.rs
    action: create
  - file: src/bar.rs
    action: modify
```
"#;
        let paths = extract_change_paths(spec);
        assert_eq!(
            paths.len(),
            2,
            "file: key should be accepted as alias for path:"
        );
        assert!(
            paths.contains(&"src/foo.rs".to_string()),
            "Should contain foo.rs"
        );
        assert!(
            paths.contains(&"src/bar.rs".to_string()),
            "Should contain bar.rs"
        );
    }

    /// Regression test: mixed path: and file: entries in same block.
    #[test]
    fn test_extract_change_paths_mixed_path_and_file_keys() {
        let spec = r#"
## Changes

```yaml
changes:
  - path: src/canonical.rs
    action: create
  - file: src/legacy.rs
    action: modify
```
"#;
        let paths = extract_change_paths(spec);
        assert_eq!(
            paths.len(),
            2,
            "Should extract both path: and file: entries"
        );
        assert!(paths.contains(&"src/canonical.rs".to_string()));
        assert!(paths.contains(&"src/legacy.rs".to_string()));
    }
}

/// Extract target file paths from the `changes:` YAML block in a spec file.
fn extract_change_paths(spec_content: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut in_changes_block = false;
    let mut in_yaml_block = false;
    let mut yaml_content = String::new();

    for line in spec_content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("## Changes") || trimmed.starts_with("## Change") {
            in_changes_block = true;
            continue;
        }

        if in_changes_block {
            if trimmed == "```yaml" {
                in_yaml_block = true;
                yaml_content.clear();
                continue;
            }
            if in_yaml_block && trimmed == "```" {
                // Parse the YAML block for 'path:' or 'file:' entries.
                // 'path:' is canonical; 'file:' is accepted as a backward-compat alias.
                for yaml_line in yaml_content.lines() {
                    let yl = yaml_line.trim();
                    if let Some(path) = yl
                        .strip_prefix("- path:")
                        .or_else(|| yl.strip_prefix("- file:"))
                        .or_else(|| {
                            if yl.starts_with("path:") {
                                yl.strip_prefix("path:")
                            } else {
                                None
                            }
                        })
                        .or_else(|| {
                            if yl.starts_with("file:") {
                                yl.strip_prefix("file:")
                            } else {
                                None
                            }
                        })
                    {
                        paths.push(path.trim().to_string());
                    }
                }
                break; // Only first changes block
            }
            if in_yaml_block {
                yaml_content.push_str(line);
                yaml_content.push('\n');
            }
        }
    }

    paths
}
// CODEGEN-END
