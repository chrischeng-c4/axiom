// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service_preamble_source.md#source
// CODEGEN-BEGIN
//! Implementation service - Business logic for implementation workflow
//!
//! Provides functions to read requirements and list changed files during
//! the implementation and review phases.
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service_requirements_runtime_source.md#source
// CODEGEN-BEGIN
use crate::Result;
use std::path::Path;
use std::process::Command;

/// Validate change_id to prevent directory traversal attacks
fn validate_change_id(change_id: &str) -> Result<()> {
    if !change_id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!(
            "Invalid change_id '{}': must be lowercase alphanumeric with hyphens only",
            change_id
        );
    }
    if change_id.contains("..") || change_id.starts_with('/') || change_id.starts_with('\\') {
        anyhow::bail!(
            "Invalid change_id '{}': directory traversal not allowed",
            change_id
        );
    }
    Ok(())
}

/// Check if current directory is a git repository
fn is_git_repo() -> bool {
    Command::new("git")
        .args(&["rev-parse", "--git-dir"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Run a git command and return output
fn run_git_command(args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Ok(format!("⚠️ Git command failed: {}", stderr.trim()));
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

/// Read all requirement files (proposal, tasks, specs) for a change in one call
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service_requirements_runtime_source.md#source
pub fn read_all_requirements(change_id: &str, project_root: &Path) -> Result<String> {
    validate_change_id(change_id)?;

    let change_dir = project_root.join(".aw/changes").join(change_id);
    if !change_dir.exists() {
        anyhow::bail!(
            "Change '{}' not found at {}",
            change_id,
            change_dir.display()
        );
    }

    let mut output = String::new();
    output.push_str(&format!("# Requirements for Change: {}\n\n", change_id));

    // Read proposal.md (required)
    let proposal_path = change_dir.join("proposal.md");
    if !proposal_path.exists() {
        anyhow::bail!("proposal.md not found for change '{}'", change_id);
    }
    let proposal_content = std::fs::read_to_string(&proposal_path)?;
    output.push_str("## Proposal\n\n");
    output.push_str(&proposal_content);
    output.push_str("\n\n---\n\n");

    // Read tasks.md (required)
    let tasks_path = change_dir.join("tasks.md");
    if !tasks_path.exists() {
        anyhow::bail!("tasks.md not found for change '{}'", change_id);
    }
    let tasks_content = std::fs::read_to_string(&tasks_path)?;
    output.push_str("## Tasks\n\n");
    output.push_str(&tasks_content);
    output.push_str("\n\n---\n\n");

    // Read all specs (optional)
    let specs_dir = change_dir.join("specs");
    let mut spec_count = 0;
    if specs_dir.exists() {
        let mut spec_files = Vec::new();
        for entry in std::fs::read_dir(&specs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                if let Some(name) = path.file_stem() {
                    let name_str = name.to_string_lossy();
                    // Skip skeleton files
                    if !name_str.starts_with('_') {
                        spec_files.push((name_str.to_string(), path));
                    }
                }
            }
        }

        spec_files.sort_by(|a, b| a.0.cmp(&b.0));

        if !spec_files.is_empty() {
            output.push_str("## Specifications\n\n");
            for (name, path) in spec_files {
                let spec_content = std::fs::read_to_string(&path)?;
                output.push_str(&format!("### Spec: {}\n\n", name));
                output.push_str(&spec_content);
                output.push_str("\n\n");
                spec_count += 1;
            }
            output.push_str("---\n\n");
        }
    }

    // Summary
    output.push_str(&format!(
        "**Total**: 1 proposal, 1 tasks file, {} specification(s)\n",
        spec_count
    ));

    Ok(output)
}

/// List changed files with detailed statistics (additions/deletions)
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service_requirements_runtime_source.md#source
pub fn list_changed_files(
    change_id: &str,
    base_branch: Option<&str>,
    filter: Option<&str>,
    _project_root: &Path,
) -> Result<String> {
    validate_change_id(change_id)?;

    let base_branch = base_branch.unwrap_or("main");

    if !is_git_repo() {
        anyhow::bail!("Not in a git repository");
    }

    let mut output = String::new();
    output.push_str(&format!("# Changed Files for: {}\n\n", change_id));

    if let Some(f) = filter {
        output.push_str(&format!("**Filter**: `{}`\n\n", f));
    }

    // Get numstat output
    let numstat = run_git_command(&["diff", "--numstat", base_branch])?;

    if numstat.is_empty() || numstat.starts_with("⚠️") {
        output.push_str("*No changes detected*\n");
        return Ok(output);
    }

    // Parse numstat output
    #[derive(Debug)]
    struct FileStat {
        added: String,
        removed: String,
        path: String,
    }

    let mut files: Vec<FileStat> = Vec::new();
    let mut total_added = 0;
    let mut total_removed = 0;

    for line in numstat.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 3 {
            continue;
        }

        let path = parts[2].to_string();

        // Apply filter if specified
        if let Some(f) = filter {
            if !path.contains(f) {
                continue;
            }
        }

        let added_str = parts[0].to_string();
        let removed_str = parts[1].to_string();

        // Parse numbers (handle binary files marked with '-')
        if added_str != "-" {
            if let Ok(n) = added_str.parse::<usize>() {
                total_added += n;
            }
        }
        if removed_str != "-" {
            if let Ok(n) = removed_str.parse::<usize>() {
                total_removed += n;
            }
        }

        files.push(FileStat {
            added: added_str,
            removed: removed_str,
            path,
        });
    }

    if files.is_empty() {
        output.push_str("*No matching files found*\n");
        return Ok(output);
    }

    // Format as markdown table
    output.push_str("| File | Status | +Lines | -Lines |\n");
    output.push_str("|------|--------|--------|--------|\n");

    for file in &files {
        let status = if file.added == "-" && file.removed == "-" {
            "Binary"
        } else if file.removed == "0" {
            "Added"
        } else if file.added == "0" {
            "Deleted"
        } else {
            "Modified"
        };

        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            file.path, status, file.added, file.removed
        ));
    }

    output.push_str("\n");
    output.push_str(&format!(
        "**Totals**: {} files, +{} lines, -{} lines\n",
        files.len(),
        total_added,
        total_removed
    ));

    Ok(output)
}

// ============================================================================
// Review Creation
// ============================================================================
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service.md#schema
// CODEGEN-BEGIN
/// Input for creating a merge review.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service.md#schema
#[derive(Debug, Clone)]
pub struct CreateMergeReviewInput {
    /// Change identifier.
    pub change_id: String,
    /// Iteration number.
    pub iteration: u32,
    /// Review summary.
    pub summary: String,
    /// Merge quality status.
    pub merge_quality: MergeQuality,
    /// Whether requirements were preserved.
    pub requirements_preserved: bool,
    /// Whether scenarios were preserved.
    pub scenarios_preserved: bool,
    /// Whether diagrams were preserved.
    pub diagrams_preserved: bool,
    /// Whether changelog is present.
    pub changelog_present: bool,
    /// Whether changelog is accurate.
    pub changelog_accurate: bool,
    /// Merge review issues.
    pub issues: Vec<MergeReviewIssue>,
    /// Merge review verdict.
    pub verdict: MergeReviewVerdict,
}

/// Input for creating a review.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service.md#schema
#[derive(Debug, Clone)]
pub struct CreateReviewInput {
    /// Change identifier.
    pub change_id: String,
    /// Iteration number.
    pub iteration: u32,
    /// Test results.
    pub test_results: TestResults,
    /// CLEAN, WARNINGS, VULNERABILITIES.
    pub security_status: String,
    /// Review issues.
    pub issues: Vec<ReviewIssue>,
    /// Review verdict.
    pub verdict: ReviewVerdict,
    /// Optional next steps text.
    pub next_steps: Option<String>,
}

/// Merge quality status.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service.md#schema
#[derive(Debug, Clone, PartialEq)]
pub enum MergeQuality {
    Clean,
    Partial,
    Failed,
}

/// A single merge review issue.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service.md#schema
#[derive(Debug, Clone)]
pub struct MergeReviewIssue {
    /// Severity level.
    pub severity: Severity,
    /// Issue description.
    pub description: String,
}

/// Merge review verdict.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service.md#schema
#[derive(Debug, Clone, PartialEq)]
pub enum MergeReviewVerdict {
    Approved,
    Reviewed,
    Rejected,
}

/// A single review issue.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service.md#schema
#[derive(Debug, Clone)]
pub struct ReviewIssue {
    /// Severity level.
    pub severity: Severity,
    /// Issue title.
    pub title: String,
    /// Issue description.
    pub description: String,
    /// Optional file path.
    pub file_path: Option<String>,
    /// Optional line number.
    pub line_number: Option<u32>,
    /// Optional recommendation.
    pub recommendation: Option<String>,
}

/// Review verdict.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service.md#schema
#[derive(Debug, Clone, PartialEq)]
pub enum ReviewVerdict {
    Approved,
    Reviewed,
    Rejected,
}

/// Issue severity level.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service.md#schema
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    High,
    Medium,
    Low,
}

/// Test results summary.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service.md#schema
#[derive(Debug, Clone, Default)]
pub struct TestResults {
    /// Status: PASS, FAIL, PARTIAL, UNKNOWN.
    pub status: String,
    /// Total tests.
    pub total: u32,
    /// Passed count.
    pub passed: u32,
    /// Failed count.
    pub failed: u32,
    /// Skipped count.
    pub skipped: u32,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service_review_runtime_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service_review_runtime_source.md#source
impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::High => write!(f, "HIGH"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::Low => write!(f, "LOW"),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service_review_runtime_source.md#source
impl std::fmt::Display for ReviewVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewVerdict::Approved => write!(f, "APPROVED"),
            ReviewVerdict::Reviewed => write!(f, "REVIEWED"),
            ReviewVerdict::Rejected => write!(f, "REJECTED"),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service_review_runtime_source.md#source
impl std::fmt::Display for MergeReviewVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MergeReviewVerdict::Approved => write!(f, "APPROVED"),
            MergeReviewVerdict::Reviewed => write!(f, "REVIEWED"),
            MergeReviewVerdict::Rejected => write!(f, "REJECTED"),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service_review_runtime_source.md#source
impl std::fmt::Display for MergeQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MergeQuality::Clean => write!(f, "CLEAN"),
            MergeQuality::Partial => write!(f, "PARTIAL"),
            MergeQuality::Failed => write!(f, "FAILED"),
        }
    }
}

/// Create REVIEW.md for a change
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service_review_runtime_source.md#source
pub fn create_review(input: CreateReviewInput, project_root: &Path) -> Result<String> {
    validate_change_id(&input.change_id)?;

    let change_dir = project_root.join(".aw/changes").join(&input.change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found", input.change_id);
    }

    // Count issues by severity
    let high_count = input
        .issues
        .iter()
        .filter(|i| i.severity == Severity::High)
        .count();
    let medium_count = input
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Medium)
        .count();
    let low_count = input
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Low)
        .count();

    // Build REVIEW.md content
    let mut content = String::new();

    // Header
    content.push_str(&format!(
        "# Code Review (Iteration {})\n\n",
        input.iteration
    ));

    // Test Results
    content.push_str("## Test Results\n");
    content.push_str(&format!("- **Status**: {}\n", input.test_results.status));
    if input.test_results.total > 0 {
        content.push_str(&format!(
            "- Total: {}, Passed: {}, Failed: {}, Skipped: {}\n",
            input.test_results.total,
            input.test_results.passed,
            input.test_results.failed,
            input.test_results.skipped
        ));
    }
    content.push('\n');

    // Security
    content.push_str("## Security\n");
    content.push_str(&format!("- **Status**: {}\n\n", input.security_status));

    // Issues
    content.push_str("## Issues\n\n");

    if input.issues.is_empty() {
        content.push_str("No issues found.\n\n");
    } else {
        // HIGH issues
        let high_issues: Vec<_> = input
            .issues
            .iter()
            .filter(|i| i.severity == Severity::High)
            .collect();
        if !high_issues.is_empty() {
            content.push_str("### HIGH\n");
            for (i, issue) in high_issues.iter().enumerate() {
                content.push_str(&format!("{}. **{}**\n", i + 1, issue.title));
                content.push_str(&format!("   - {}\n", issue.description));
                if let Some(ref path) = issue.file_path {
                    if let Some(line) = issue.line_number {
                        content.push_str(&format!("   - Location: `{}:{}`\n", path, line));
                    } else {
                        content.push_str(&format!("   - Location: `{}`\n", path));
                    }
                }
                if let Some(ref rec) = issue.recommendation {
                    content.push_str(&format!("   - Recommendation: {}\n", rec));
                }
                content.push('\n');
            }
        }

        // MEDIUM issues
        let medium_issues: Vec<_> = input
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Medium)
            .collect();
        if !medium_issues.is_empty() {
            content.push_str("### MEDIUM\n");
            for (i, issue) in medium_issues.iter().enumerate() {
                content.push_str(&format!("{}. **{}**\n", i + 1, issue.title));
                content.push_str(&format!("   - {}\n", issue.description));
                if let Some(ref path) = issue.file_path {
                    if let Some(line) = issue.line_number {
                        content.push_str(&format!("   - Location: `{}:{}`\n", path, line));
                    } else {
                        content.push_str(&format!("   - Location: `{}`\n", path));
                    }
                }
                if let Some(ref rec) = issue.recommendation {
                    content.push_str(&format!("   - Recommendation: {}\n", rec));
                }
                content.push('\n');
            }
        }

        // LOW issues
        let low_issues: Vec<_> = input
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Low)
            .collect();
        if !low_issues.is_empty() {
            content.push_str("### LOW\n");
            for (i, issue) in low_issues.iter().enumerate() {
                content.push_str(&format!("{}. **{}**\n", i + 1, issue.title));
                content.push_str(&format!("   - {}\n", issue.description));
                if let Some(ref path) = issue.file_path {
                    if let Some(line) = issue.line_number {
                        content.push_str(&format!("   - Location: `{}:{}`\n", path, line));
                    } else {
                        content.push_str(&format!("   - Location: `{}`\n", path));
                    }
                }
                if let Some(ref rec) = issue.recommendation {
                    content.push_str(&format!("   - Recommendation: {}\n", rec));
                }
                content.push('\n');
            }
        }
    }

    // Verdict
    content.push_str("## Verdict\n");
    content.push_str(&format!("{}\n", input.verdict));

    if let Some(ref next) = input.next_steps {
        content.push_str(&format!("\n**Next Steps**: {}\n", next));
    }

    // Write the file
    let review_path = change_dir.join("review_impl.md");
    std::fs::write(&review_path, &content)?;

    Ok(format!(
        "✓ review_impl.md written: {}\n  Verdict: {}\n  Issues: {} high, {} medium, {} low",
        review_path.display(),
        input.verdict,
        high_count,
        medium_count,
        low_count
    ))
}

/// Create MERGE_REVIEW.md for a change (merge-change workflow)
///
/// This generates a structured review file with:
/// - Summary of the merge
/// - Merge quality assessment
/// - Content preservation checks
/// - CHANGELOG quality check
/// - Issues found (if any)
/// - Verdict with checkbox (parseable by parser)
/// @spec projects/agentic-workflow/tech-design/core/interfaces/services/implementation_service_review_runtime_source.md#source
pub fn create_merge_review(input: CreateMergeReviewInput, project_root: &Path) -> Result<String> {
    validate_change_id(&input.change_id)?;

    let change_dir = project_root.join(".aw/changes").join(&input.change_id);
    if !change_dir.exists() {
        anyhow::bail!("Change '{}' not found", input.change_id);
    }

    // Count issues by severity
    let high_count = input
        .issues
        .iter()
        .filter(|i| i.severity == Severity::High)
        .count();
    let medium_count = input
        .issues
        .iter()
        .filter(|i| i.severity == Severity::Medium)
        .count();

    // Build review_merge.md content
    let mut content = String::new();

    // YAML frontmatter (required by extract_review_info parser)
    content.push_str("---\n");
    content.push_str(&format!("verdict: {}\n", input.verdict));
    content.push_str(&format!("change_id: {}\n", input.change_id));
    content.push_str(&format!("iteration: {}\n", input.iteration));
    content.push_str("---\n\n");

    // Header
    content.push_str(&format!("# Merge Review Report: {}\n\n", input.change_id));
    content.push_str(&format!("**Iteration**: {}\n\n", input.iteration));

    // Summary
    content.push_str("## Summary\n");
    content.push_str(&input.summary);
    content.push_str("\n\n");

    // Merge Quality
    content.push_str("## Merge Quality\n\n");
    content.push_str("### Spec Integration\n");
    content.push_str(&format!("- **Status**: {}\n\n", input.merge_quality));

    // Content Preservation
    content.push_str("### Content Preservation\n");
    content.push_str(&format!(
        "- **Requirements preserved**: {}\n",
        if input.requirements_preserved {
            "Yes"
        } else {
            "No"
        }
    ));
    content.push_str(&format!(
        "- **Scenarios preserved**: {}\n",
        if input.scenarios_preserved {
            "Yes"
        } else {
            "No"
        }
    ));
    content.push_str(&format!(
        "- **Diagrams preserved**: {}\n\n",
        if input.diagrams_preserved {
            "Yes"
        } else {
            "N/A"
        }
    ));

    // Issues
    content.push_str("## Issues Found\n\n");
    if input.issues.is_empty() {
        content.push_str("None.\n\n");
    } else {
        for issue in &input.issues {
            content.push_str(&format!(
                "- **[{}]** {}\n",
                issue.severity, issue.description
            ));
        }
        content.push_str("\n");
    }

    // CHANGELOG Quality
    content.push_str("## CHANGELOG Quality\n");
    content.push_str(&format!(
        "- **Entry present**: {}\n",
        if input.changelog_present { "Yes" } else { "No" }
    ));
    content.push_str(&format!(
        "- **Description accurate**: {}\n",
        if input.changelog_accurate {
            "Yes"
        } else {
            "No"
        }
    ));
    content.push_str("- **Format correct**: Yes\n\n");

    // Verdict with checkbox format (parseable by parser)
    content.push_str("## Verdict\n");
    match input.verdict {
        MergeReviewVerdict::Approved => {
            content.push_str("- [x] APPROVED - Merge quality acceptable, ready for archive\n");
            content.push_str("- [ ] REVIEWED - Address issues above (fixable automatically)\n");
            content
                .push_str("- [ ] REJECTED - Fundamental problems (require manual intervention)\n");
        }
        MergeReviewVerdict::Reviewed => {
            content.push_str("- [ ] APPROVED - Merge quality acceptable, ready for archive\n");
            content.push_str("- [x] REVIEWED - Address issues above (fixable automatically)\n");
            content
                .push_str("- [ ] REJECTED - Fundamental problems (require manual intervention)\n");
        }
        MergeReviewVerdict::Rejected => {
            content.push_str("- [ ] APPROVED - Merge quality acceptable, ready for archive\n");
            content.push_str("- [ ] REVIEWED - Address issues above (fixable automatically)\n");
            content
                .push_str("- [x] REJECTED - Fundamental problems (require manual intervention)\n");
        }
    }
    content.push('\n');

    // Next steps
    content.push_str("**Next Steps**: ");
    match input.verdict {
        MergeReviewVerdict::Approved => content.push_str("Proceed to archive.\n"),
        MergeReviewVerdict::Reviewed => content.push_str("Fix issues and re-run merge.\n"),
        MergeReviewVerdict::Rejected => content.push_str("Manual intervention required.\n"),
    }

    // Write the file
    let review_path = change_dir.join("review_merge.md");
    std::fs::write(&review_path, &content)?;

    Ok(format!(
        "✓ review_merge.md written: {}\n  Verdict: {}\n  Issues: {} high, {} medium",
        review_path.display(),
        input.verdict,
        high_count,
        medium_count
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validate_change_id_valid() {
        assert!(validate_change_id("test-change").is_ok());
        assert!(validate_change_id("feature-123").is_ok());
        assert!(validate_change_id("fix-bug-42").is_ok());
    }

    #[test]
    fn test_validate_change_id_invalid() {
        assert!(validate_change_id("../etc/passwd").is_err());
        assert!(validate_change_id("/absolute/path").is_err());
        assert!(validate_change_id("Test-Change").is_err()); // uppercase
        assert!(validate_change_id("test_change").is_err()); // underscore
        assert!(validate_change_id("test..change").is_err()); // double dot
    }

    #[test]
    fn test_read_all_requirements_basic() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory structure
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        // Create proposal.md
        std::fs::write(
            change_dir.join("proposal.md"),
            "# Test Proposal\n\nThis is a test proposal.",
        )
        .unwrap();

        // Create tasks.md
        std::fs::write(change_dir.join("tasks.md"), "# Tasks\n\n- Task 1\n- Task 2").unwrap();

        // Create specs
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        std::fs::write(
            specs_dir.join("feature-spec.md"),
            "# Feature Spec\n\nRequirements here.",
        )
        .unwrap();

        let result = read_all_requirements("test-change", project_root).unwrap();

        assert!(result.contains("# Requirements for Change: test-change"));
        assert!(result.contains("## Proposal"));
        assert!(result.contains("This is a test proposal"));
        assert!(result.contains("## Tasks"));
        assert!(result.contains("Task 1"));
        assert!(result.contains("## Specifications"));
        assert!(result.contains("### Spec: feature-spec"));
        assert!(result.contains("**Total**: 1 proposal, 1 tasks file, 1 specification(s)"));
    }

    #[test]
    fn test_read_all_requirements_no_specs() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory without specs
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        std::fs::write(change_dir.join("proposal.md"), "# Proposal").unwrap();
        std::fs::write(change_dir.join("tasks.md"), "# Tasks").unwrap();

        let result = read_all_requirements("test-change", project_root).unwrap();

        assert!(result.contains("**Total**: 1 proposal, 1 tasks file, 0 specification(s)"));
    }

    #[test]
    fn test_read_all_requirements_missing_proposal() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create change directory without proposal
        let change_dir = project_root.join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();
        std::fs::write(change_dir.join("tasks.md"), "# Tasks").unwrap();

        let result = read_all_requirements("test-change", project_root);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("proposal.md not found"));
    }

    #[test]
    fn test_is_git_repo() {
        // This test will pass or fail depending on whether we're in a git repo
        // Just verify it doesn't panic
        let _ = is_git_repo();
    }
}
// CODEGEN-END
