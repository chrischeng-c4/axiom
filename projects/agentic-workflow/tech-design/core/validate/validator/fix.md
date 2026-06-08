---
id: sdd-validator-fix
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Validation TDs implement managed and semantic production gates for standardization readiness."
---

# Auto-Fixer Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/validator/fix.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AutoFixer` | projects/agentic-workflow/src/validator/fix.rs | struct | pub | 15 |  |
| `FixResult` | projects/agentic-workflow/src/validator/fix.rs | struct | pub | 23 |  |
| `fix_errors` | projects/agentic-workflow/src/validator/fix.rs | function | pub | 44 | fix_errors(&self, errors: &[ValidationError]) -> Result<FixResult> |
| `new` | projects/agentic-workflow/src/validator/fix.rs | function | pub | 37 | new(project_root: impl Into<PathBuf>) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  AutoFixer:
    type: object
    required: [project_root]
    description: Auto-fixer for validation errors.
    properties:
      project_root:
        type: string
        x-rust-type: "PathBuf"
        x-rust-visibility: private
        description: "Project root."
    x-rust-struct:
      derive: []

  FixResult:
    type: object
    required: [files_modified, errors_fixed, unfixable_errors, fix_details]
    description: Result of a fix attempt.
    properties:
      files_modified:
        type: integer
        x-rust-type: "usize"
        description: "Number of files modified."
      errors_fixed:
        type: integer
        x-rust-type: "usize"
        description: "Number of errors fixed."
      unfixable_errors:
        type: array
        items: { type: object }
        x-rust-type: "Vec<ValidationError>"
        description: "Errors that could not be fixed."
      fix_details:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Details of what was fixed."
    x-rust-struct:
      derive: [Debug]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/validator/fix.rs -->
```rust
//! Auto-fixer for validation errors
//!
//! This module provides automatic fixes for common validation errors that can be
//! mechanically corrected without AI intervention.

use crate::models::{ErrorCategory, ValidationError};
use crate::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Auto-fixer for validation errors.
/// @spec projects/agentic-workflow/tech-design/core/validate/validator/fix.md#schema
pub struct AutoFixer {
    /// Project root.
    project_root: PathBuf,
}

/// Result of a fix attempt.
/// @spec projects/agentic-workflow/tech-design/core/validate/validator/fix.md#schema
#[derive(Debug)]
pub struct FixResult {
    /// Number of files modified.
    pub files_modified: usize,
    /// Number of errors fixed.
    pub errors_fixed: usize,
    /// Errors that could not be fixed.
    pub unfixable_errors: Vec<ValidationError>,
    /// Details of what was fixed.
    pub fix_details: Vec<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/validate/validator/fix.md#changes
impl AutoFixer {
    /// Create a new auto-fixer
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
        }
    }

    /// Attempt to fix validation errors
    pub fn fix_errors(&self, errors: &[ValidationError]) -> Result<FixResult> {
        let mut files_modified = 0;
        let mut errors_fixed = 0;
        let mut unfixable_errors = Vec::new();
        let mut fix_details = Vec::new();

        // Group errors by file
        let mut errors_by_file: HashMap<PathBuf, Vec<&ValidationError>> = HashMap::new();
        for error in errors {
            errors_by_file
                .entry(error.file.clone())
                .or_default()
                .push(error);
        }

        // Process each file
        for (file_path, file_errors) in errors_by_file {
            let full_path = if file_path.is_absolute() {
                file_path.clone()
            } else {
                self.project_root.join(&file_path)
            };

            if !full_path.exists() {
                for error in file_errors {
                    unfixable_errors.push(error.clone());
                }
                continue;
            }

            let content = std::fs::read_to_string(&full_path)?;
            let mut modified_content = content.clone();
            let mut file_fixed = false;

            for error in file_errors {
                if !error.category.is_fixable() {
                    unfixable_errors.push(error.clone());
                    continue;
                }

                match self.apply_fix(&mut modified_content, error) {
                    Some(detail) => {
                        fix_details.push(detail);
                        errors_fixed += 1;
                        file_fixed = true;
                    }
                    None => {
                        unfixable_errors.push(error.clone());
                    }
                }
            }

            if file_fixed && modified_content != content {
                std::fs::write(&full_path, &modified_content)?;
                files_modified += 1;
            }
        }

        Ok(FixResult {
            files_modified,
            errors_fixed,
            unfixable_errors,
            fix_details,
        })
    }

    /// Apply a single fix and return a description of what was done
    fn apply_fix(&self, content: &mut String, error: &ValidationError) -> Option<String> {
        match error.category {
            ErrorCategory::MissingHeading => self.fix_missing_heading(content, error),
            ErrorCategory::MissingWhenThen => self.fix_missing_when_then(content, error),
            ErrorCategory::MissingScenario => self.fix_missing_scenario(content, error),
            _ => None,
        }
    }

    /// Check if content has Acceptance Criteria with proper WHEN/THEN
    fn has_valid_acceptance_criteria(content: &str) -> bool {
        if let Some(ac_pos) = content.find("## Acceptance Criteria") {
            let after_ac = &content[ac_pos..];
            // Check if there's at least one scenario with WHEN and THEN
            after_ac.contains("WHEN") && after_ac.contains("THEN")
        } else {
            false
        }
    }

    /// Fix missing heading by appending it to the file
    fn fix_missing_heading(&self, content: &mut String, error: &ValidationError) -> Option<String> {
        // Extract heading name from error message
        // Format: "Missing required heading: Overview"
        let heading_name = error.message.strip_prefix("Missing required heading: ")?;

        // Check if heading already exists (case-insensitive)
        let heading_lower = heading_name.to_lowercase();
        if content
            .to_lowercase()
            .contains(&format!("## {}", heading_lower))
        {
            return None;
        }

        // Append the missing heading
        let heading_content = match heading_name {
            "Overview" => "\n\n## Overview\n\n<!-- Brief description of this feature -->\n",
            "Acceptance Criteria" => {
                "\n\n## Acceptance Criteria\n\n### Scenario: Basic Usage\n- **WHEN** the feature is used\n- **THEN** it should work correctly\n"
            }
            "Requirements" => "\n\n## Requirements\n\n### R1: Basic Requirement\nDescription of the requirement.\n",
            _ => return None, // Unknown heading, can't auto-fix
        };

        content.push_str(heading_content);
        Some(format!(
            "{}: Added missing '## {}' heading",
            error.file.display(),
            heading_name
        ))
    }

    /// Fix missing WHEN/THEN by adding Acceptance Criteria section with placeholder scenario
    fn fix_missing_when_then(
        &self,
        content: &mut String,
        error: &ValidationError,
    ) -> Option<String> {
        // Check if we already have valid Acceptance Criteria
        if Self::has_valid_acceptance_criteria(content) {
            return None;
        }

        // If there's no Acceptance Criteria section, add one with a placeholder scenario
        if !content.contains("## Acceptance Criteria") {
            let ac_section = "\n\n## Acceptance Criteria\n\n### Scenario: Basic Usage\n- **WHEN** the feature is used\n- **THEN** it should work correctly\n";
            content.push_str(ac_section);
            return Some(format!(
                "{}: Added Acceptance Criteria with WHEN/THEN",
                error.file.display()
            ));
        }

        // There's an AC section but no WHEN/THEN - add a placeholder scenario
        if let Some(ac_pos) = content.find("## Acceptance Criteria") {
            let ac_end = ac_pos + "## Acceptance Criteria".len();
            let scenario = "\n\n### Scenario: Basic Usage\n- **WHEN** the feature is used\n- **THEN** it should work correctly\n";
            content.insert_str(ac_end, scenario);
            return Some(format!(
                "{}: Added placeholder scenario with WHEN/THEN",
                error.file.display()
            ));
        }

        // Fallback: try to find scenario without WHEN/THEN
        // Error format: "Scenario 'X' is missing WHEN clause" (old format)
        let scenario_name = if error.message.contains("WHEN") {
            error
                .message
                .strip_prefix("Scenario '")?
                .split("' is missing")
                .next()?
        } else if error.message.contains("THEN") {
            error
                .message
                .strip_prefix("Scenario '")?
                .split("' is missing")
                .next()?
        } else {
            return None;
        };

        // Find the scenario in content
        let scenario_marker = format!("Scenario: {}", scenario_name);
        if let Some(pos) = content.find(&scenario_marker) {
            // Find the end of this scenario (next ### or end of file)
            let after_scenario = &content[pos..];
            let scenario_end = after_scenario
                .find("\n### ")
                .or_else(|| after_scenario.find("\n## "))
                .unwrap_or(after_scenario.len());

            let scenario_content = &content[pos..pos + scenario_end];

            // Check what's missing and add it
            let missing_when = !scenario_content.contains("WHEN");
            let missing_then = !scenario_content.contains("THEN");

            if missing_when || missing_then {
                let insert_pos = pos + scenario_end;
                let mut to_insert = String::new();

                if missing_when {
                    to_insert.push_str("\n- **WHEN** [condition]");
                }
                if missing_then {
                    to_insert.push_str("\n- **THEN** [expected result]");
                }

                content.insert_str(insert_pos, &to_insert);
                return Some(format!(
                    "{}: Added WHEN/THEN to scenario '{}'",
                    error.file.display(),
                    scenario_name
                ));
            }
        }

        None
    }

    /// Fix missing scenario by adding a placeholder
    fn fix_missing_scenario(
        &self,
        content: &mut String,
        error: &ValidationError,
    ) -> Option<String> {
        // Check if we already have valid Acceptance Criteria
        if Self::has_valid_acceptance_criteria(content) {
            return None;
        }

        // If there's no Acceptance Criteria section, add one with a placeholder scenario
        if !content.contains("## Acceptance Criteria") {
            let ac_section = "\n\n## Acceptance Criteria\n\n### Scenario: Basic Usage\n- **WHEN** the feature is used\n- **THEN** it should work correctly\n";
            content.push_str(ac_section);
            return Some(format!(
                "{}: Added Acceptance Criteria with scenario",
                error.file.display()
            ));
        }

        // Find Acceptance Criteria section
        let ac_marker = "## Acceptance Criteria";
        if let Some(ac_pos) = content.find(ac_marker) {
            // Check if there are any scenarios with WHEN/THEN
            let after_ac = &content[ac_pos..];
            let has_scenario =
                after_ac.contains("### Scenario:") || after_ac.contains("#### Scenario:");
            let has_when_then = after_ac.contains("WHEN") && after_ac.contains("THEN");

            if !has_scenario || !has_when_then {
                // Add a placeholder scenario
                let ac_end = ac_pos + ac_marker.len();
                let scenario = "\n\n### Scenario: Basic Usage\n- **WHEN** the feature is used\n- **THEN** it should work correctly\n";
                content.insert_str(ac_end, scenario);
                return Some(format!(
                    "{}: Added placeholder scenario to Acceptance Criteria",
                    error.file.display()
                ));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Severity;
    use tempfile::TempDir;

    // ── Helpers ─────────────────────────────────────────────────────────────

    fn make_error(msg: &str, file: impl Into<PathBuf>, category: ErrorCategory) -> ValidationError {
        ValidationError::new(msg, file, None, Severity::High, category)
    }

    fn write_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let path = dir.path().join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    // ── REQ: R5 — fix_missing_heading ──────────────────────────────────────

    #[test]
    fn test_fix_missing_overview() {
        let dir = TempDir::new().unwrap();
        let fp = write_file(
            &dir,
            "test.md",
            "# Spec: Test\n\n## Requirements\n\nSome content",
        );
        let fixer = AutoFixer::new(dir.path());
        let err = make_error(
            "Missing required heading: Overview",
            fp.clone(),
            ErrorCategory::MissingHeading,
        );

        let result = fixer.fix_errors(&[err]).unwrap();
        assert_eq!(result.errors_fixed, 1);
        assert_eq!(result.files_modified, 1);
        let content = std::fs::read_to_string(&fp).unwrap();
        assert!(content.contains("## Overview"));
    }

    #[test]
    fn test_fix_missing_acceptance_criteria() {
        let dir = TempDir::new().unwrap();
        let fp = write_file(
            &dir,
            "test.md",
            "# Spec: Test\n\n## Overview\n\nSome content",
        );
        let fixer = AutoFixer::new(dir.path());
        let err = make_error(
            "Missing required heading: Acceptance Criteria",
            fp.clone(),
            ErrorCategory::MissingHeading,
        );

        let result = fixer.fix_errors(&[err]).unwrap();
        assert_eq!(result.errors_fixed, 1);
        let content = std::fs::read_to_string(&fp).unwrap();
        assert!(content.contains("## Acceptance Criteria"));
        assert!(content.contains("WHEN"));
        assert!(content.contains("THEN"));
    }

    // REQ: R5 — S18
    #[test]
    fn test_fix_missing_heading_requirements() {
        let dir = TempDir::new().unwrap();
        let fp = write_file(&dir, "test.md", "# Spec\n\n## Overview\n\nBody");
        let fixer = AutoFixer::new(dir.path());
        let err = make_error(
            "Missing required heading: Requirements",
            fp.clone(),
            ErrorCategory::MissingHeading,
        );

        let result = fixer.fix_errors(&[err]).unwrap();
        assert_eq!(result.errors_fixed, 1);
        let content = std::fs::read_to_string(&fp).unwrap();
        assert!(content.contains("## Requirements"));
        assert!(content.contains("R1:"));
    }

    // REQ: R5 — S19
    #[test]
    fn test_fix_missing_heading_duplicate_detection() {
        let dir = TempDir::new().unwrap();
        let fp = write_file(&dir, "test.md", "# Spec\n\n## overview\n\nAlready has it");
        let fixer = AutoFixer::new(dir.path());
        let err = make_error(
            "Missing required heading: Overview",
            fp.clone(),
            ErrorCategory::MissingHeading,
        );

        let result = fixer.fix_errors(&[err]).unwrap();
        // Duplicate detected — heading not added, counted as unfixable
        assert_eq!(result.errors_fixed, 0);
        assert_eq!(result.unfixable_errors.len(), 1);
    }

    // REQ: R5 — S5n (unknown heading)
    #[test]
    fn test_fix_missing_heading_unknown() {
        let dir = TempDir::new().unwrap();
        let fp = write_file(&dir, "test.md", "# Spec\n\nBody");
        let fixer = AutoFixer::new(dir.path());
        let err = make_error(
            "Missing required heading: FooBar",
            fp.clone(),
            ErrorCategory::MissingHeading,
        );

        let result = fixer.fix_errors(&[err]).unwrap();
        assert_eq!(result.errors_fixed, 0);
        assert_eq!(result.unfixable_errors.len(), 1);
    }

    // ── REQ: R5 — fix_missing_when_then ────────────────────────────────────

    // REQ: R5 — S11
    #[test]
    fn test_fix_missing_when_then_ac_exists_without_when_then() {
        let dir = TempDir::new().unwrap();
        let fp = write_file(
            &dir,
            "test.md",
            "# Spec\n\n## Acceptance Criteria\n\nSome text",
        );
        let fixer = AutoFixer::new(dir.path());
        let err = make_error(
            "Missing WHEN/THEN in Acceptance Criteria",
            fp.clone(),
            ErrorCategory::MissingWhenThen,
        );

        let result = fixer.fix_errors(&[err]).unwrap();
        assert_eq!(result.errors_fixed, 1);
        let content = std::fs::read_to_string(&fp).unwrap();
        assert!(content.contains("WHEN"));
        assert!(content.contains("THEN"));
    }

    // REQ: R5 — S12
    #[test]
    fn test_fix_missing_when_then_ac_already_valid() {
        let dir = TempDir::new().unwrap();
        let content = "# Spec\n\n## Acceptance Criteria\n\n### Scenario: X\n- **WHEN** user does Y\n- **THEN** Z happens\n";
        let fp = write_file(&dir, "test.md", content);
        let fixer = AutoFixer::new(dir.path());
        let err = make_error(
            "Missing WHEN/THEN in Acceptance Criteria",
            fp.clone(),
            ErrorCategory::MissingWhenThen,
        );

        let result = fixer.fix_errors(&[err]).unwrap();
        assert_eq!(result.errors_fixed, 0);
        assert_eq!(result.unfixable_errors.len(), 1);
    }

    // REQ: R5 — S11 variant (no AC section at all)
    #[test]
    fn test_fix_missing_when_then_no_ac_section() {
        let dir = TempDir::new().unwrap();
        let fp = write_file(&dir, "test.md", "# Spec\n\n## Overview\n\nBody only");
        let fixer = AutoFixer::new(dir.path());
        let err = make_error(
            "Missing WHEN/THEN in Acceptance Criteria",
            fp.clone(),
            ErrorCategory::MissingWhenThen,
        );

        let result = fixer.fix_errors(&[err]).unwrap();
        assert_eq!(result.errors_fixed, 1);
        let content = std::fs::read_to_string(&fp).unwrap();
        assert!(content.contains("## Acceptance Criteria"));
        assert!(content.contains("WHEN"));
        assert!(content.contains("THEN"));
    }

    // REQ: R5 — S14 (named scenario missing WHEN)
    #[test]
    fn test_fix_missing_when_then_named_scenario_missing_when() {
        let dir = TempDir::new().unwrap();
        let content = "# Spec\n\n## Acceptance Criteria\n\n### Scenario: Login\n- **THEN** user sees dashboard\n";
        let fp = write_file(&dir, "test.md", content);
        let fixer = AutoFixer::new(dir.path());
        // Note: has_valid_acceptance_criteria checks for WHEN AND THEN globally.
        // This scenario has THEN but no WHEN in the AC section, so it's not valid.
        let err = make_error(
            "Scenario 'Login' is missing WHEN clause",
            fp.clone(),
            ErrorCategory::MissingWhenThen,
        );

        let result = fixer.fix_errors(&[err]).unwrap();
        assert_eq!(result.errors_fixed, 1);
        let content = std::fs::read_to_string(&fp).unwrap();
        assert!(content.contains("WHEN"));
    }

    // REQ: R5 — S14 variant (named scenario missing THEN)
    #[test]
    fn test_fix_missing_when_then_named_scenario_missing_then() {
        let dir = TempDir::new().unwrap();
        let content = "# Spec\n\n## Acceptance Criteria\n\n### Scenario: Login\n- **WHEN** user enters creds\n";
        let fp = write_file(&dir, "test.md", content);
        let fixer = AutoFixer::new(dir.path());
        let err = make_error(
            "Scenario 'Login' is missing THEN clause",
            fp.clone(),
            ErrorCategory::MissingWhenThen,
        );

        let result = fixer.fix_errors(&[err]).unwrap();
        assert_eq!(result.errors_fixed, 1);
        let content = std::fs::read_to_string(&fp).unwrap();
        assert!(content.contains("THEN"));
    }

    // ── REQ: R5 — fix_missing_scenario ─────────────────────────────────────

    // REQ: R5 — S13
    #[test]
    fn test_fix_missing_scenario_no_ac() {
        let dir = TempDir::new().unwrap();
        let fp = write_file(&dir, "test.md", "# Spec\n\n## Overview\n\nBody");
        let fixer = AutoFixer::new(dir.path());
        let err = make_error(
            "Missing scenario in Acceptance Criteria",
            fp.clone(),
            ErrorCategory::MissingScenario,
        );

        let result = fixer.fix_errors(&[err]).unwrap();
        assert_eq!(result.errors_fixed, 1);
        let content = std::fs::read_to_string(&fp).unwrap();
        assert!(content.contains("## Acceptance Criteria"));
        assert!(content.contains("### Scenario:"));
        assert!(content.contains("WHEN"));
        assert!(content.contains("THEN"));
    }

    // REQ: R5 — S13 variant (AC exists but no scenario heading)
    #[test]
    fn test_fix_missing_scenario_ac_without_scenario() {
        let dir = TempDir::new().unwrap();
        let fp = write_file(
            &dir,
            "test.md",
            "# Spec\n\n## Acceptance Criteria\n\nJust text, no scenarios",
        );
        let fixer = AutoFixer::new(dir.path());
        let err = make_error(
            "Missing scenario in Acceptance Criteria",
            fp.clone(),
            ErrorCategory::MissingScenario,
        );

        let result = fixer.fix_errors(&[err]).unwrap();
        assert_eq!(result.errors_fixed, 1);
        let content = std::fs::read_to_string(&fp).unwrap();
        assert!(content.contains("### Scenario:"));
        assert!(content.contains("WHEN"));
    }

    // REQ: R5 — S13 variant (already valid)
    #[test]
    fn test_fix_missing_scenario_already_valid() {
        let dir = TempDir::new().unwrap();
        let content =
            "# Spec\n\n## Acceptance Criteria\n\n### Scenario: X\n- **WHEN** A\n- **THEN** B\n";
        let fp = write_file(&dir, "test.md", content);
        let fixer = AutoFixer::new(dir.path());
        let err = make_error(
            "Missing scenario in Acceptance Criteria",
            fp.clone(),
            ErrorCategory::MissingScenario,
        );

        let result = fixer.fix_errors(&[err]).unwrap();
        assert_eq!(result.errors_fixed, 0);
        assert_eq!(result.unfixable_errors.len(), 1);
    }

    // ── REQ: R5 — fix_errors grouping / dispatch ──────────────────────────

    // REQ: R5 — S15
    #[test]
    fn test_fix_errors_multi_file_grouping() {
        let dir = TempDir::new().unwrap();
        let fp1 = write_file(&dir, "a.md", "# A\n\nBody");
        let fp2 = write_file(&dir, "b.md", "# B\n\nBody");
        let fp3 = write_file(&dir, "c.md", "# C\n\nBody");

        let fixer = AutoFixer::new(dir.path());
        let errors = vec![
            make_error(
                "Missing required heading: Overview",
                fp1.clone(),
                ErrorCategory::MissingHeading,
            ),
            make_error(
                "Missing required heading: Overview",
                fp2.clone(),
                ErrorCategory::MissingHeading,
            ),
            make_error(
                "Missing required heading: Overview",
                fp3.clone(),
                ErrorCategory::MissingHeading,
            ),
        ];

        let result = fixer.fix_errors(&errors).unwrap();
        assert_eq!(result.errors_fixed, 3);
        assert_eq!(result.files_modified, 3);
    }

    // REQ: R5 — S16
    #[test]
    fn test_fix_errors_unfixable_category() {
        let dir = TempDir::new().unwrap();
        let fp = write_file(&dir, "test.md", "# Spec\n\nBody");
        let fixer = AutoFixer::new(dir.path());
        let err = make_error("Bad structure", fp.clone(), ErrorCategory::InvalidStructure);

        let result = fixer.fix_errors(&[err]).unwrap();
        assert_eq!(result.errors_fixed, 0);
        assert_eq!(result.unfixable_errors.len(), 1);
    }

    // REQ: R5 — S17
    #[test]
    fn test_fix_errors_nonexistent_file() {
        let dir = TempDir::new().unwrap();
        let fake = dir.path().join("nonexistent.md");
        let fixer = AutoFixer::new(dir.path());
        let err = make_error(
            "Missing required heading: Overview",
            fake.clone(),
            ErrorCategory::MissingHeading,
        );

        let result = fixer.fix_errors(&[err]).unwrap();
        assert_eq!(result.errors_fixed, 0);
        assert_eq!(result.unfixable_errors.len(), 1);
    }

    // REQ: R5 — relative path resolution
    #[test]
    fn test_fix_errors_relative_path() {
        let dir = TempDir::new().unwrap();
        let _fp = write_file(&dir, "rel.md", "# Spec\n\nBody");
        let fixer = AutoFixer::new(dir.path());
        // Use a relative path — fixer should join with project_root
        let err = make_error(
            "Missing required heading: Overview",
            "rel.md",
            ErrorCategory::MissingHeading,
        );

        let result = fixer.fix_errors(&[err]).unwrap();
        assert_eq!(result.errors_fixed, 1);
        let content = std::fs::read_to_string(dir.path().join("rel.md")).unwrap();
        assert!(content.contains("## Overview"));
    }

    // ── REQ: R5 — has_valid_acceptance_criteria ────────────────────────────

    // REQ: R5 — S5o
    #[test]
    fn test_has_valid_acceptance_criteria_true() {
        let content = "## Acceptance Criteria\n\n### Scenario: X\n- **WHEN** A\n- **THEN** B";
        assert!(AutoFixer::has_valid_acceptance_criteria(content));
    }

    // REQ: R5 — S5p
    #[test]
    fn test_has_valid_acceptance_criteria_false_no_section() {
        let content = "## Overview\n\nNo AC section here";
        assert!(!AutoFixer::has_valid_acceptance_criteria(content));
    }

    #[test]
    fn test_has_valid_acceptance_criteria_false_no_when_then() {
        let content = "## Acceptance Criteria\n\nJust some text, no structured scenarios";
        assert!(!AutoFixer::has_valid_acceptance_criteria(content));
    }

    // ── REQ: R5 — apply_fix dispatch ───────────────────────────────────────

    #[test]
    fn test_apply_fix_dispatches_to_correct_handler() {
        let dir = TempDir::new().unwrap();
        let fixer = AutoFixer::new(dir.path());

        // MissingHeading
        let mut content = "# Spec\n\nBody".to_string();
        let err = make_error(
            "Missing required heading: Overview",
            "t.md",
            ErrorCategory::MissingHeading,
        );
        let detail = fixer.apply_fix(&mut content, &err);
        assert!(detail.is_some());
        assert!(content.contains("## Overview"));

        // Unsupported category returns None
        let mut content2 = "# Spec".to_string();
        let err2 = make_error("broken", "t.md", ErrorCategory::BrokenReference);
        assert!(fixer.apply_fix(&mut content2, &err2).is_none());
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/validator/fix.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Codegen owns the complete source file through a source template.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: review lang: markdown -->

**Verdict:** approved

- [overview] Two structs.
- [schema] Standard pattern; private project_root, public FixResult.
- [changes] Standard split.
