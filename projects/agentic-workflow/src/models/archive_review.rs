// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#source
use super::IssueSeverity; // Re-use from challenge module

use serde::{Deserialize, Serialize};

/// Overall verdict from a Codex archive quality review.
/// Approved: all checks passed, safe to archive.
/// Reviewed: minor issues found, needs revision.
/// Rejected: major issues, requires manual intervention.
/// Unknown: could not parse verdict.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchiveReviewVerdict {
    /// All checks passed, safe to archive.
    #[serde(rename = "Approved")]
    Approved,
    /// Minor issues found, needs revision.
    #[serde(rename = "Reviewed")]
    Reviewed,
    /// Major issues, requires manual intervention.
    #[serde(rename = "Rejected")]
    Rejected,
    /// Could not parse verdict.
    #[serde(rename = "Unknown")]
    Unknown,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#schema.impls
impl ArchiveReviewVerdict {
    /// Human-readable display name for the verdict.
    pub fn name(&self) -> &'static str {
        match self {
            ArchiveReviewVerdict::Approved => "APPROVED",
            ArchiveReviewVerdict::Reviewed => "REVIEWED",
            ArchiveReviewVerdict::Rejected => "REJECTED",
            ArchiveReviewVerdict::Unknown => "UNKNOWN",
        }
    }

    /// Emoji symbol representing the verdict.
    pub fn emoji(&self) -> &'static str {
        match self {
            ArchiveReviewVerdict::Approved => "✅",
            ArchiveReviewVerdict::Reviewed => "⚠️",
            ArchiveReviewVerdict::Rejected => "❌",
            ArchiveReviewVerdict::Unknown => "❓",
        }
    }
}

/// Category of issue found during archive review.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchiveIssueCategory {
    /// Missing content from delta.
    #[serde(rename = "MissingContent")]
    MissingContent,
    /// Extra content not in delta (hallucination).
    #[serde(rename = "Hallucination")]
    Hallucination,
    /// Format violation.
    #[serde(rename = "FormatError")]
    FormatError,
    /// CHANGELOG inaccuracy.
    #[serde(rename = "ChangelogError")]
    ChangelogError,
    /// Cross-reference broken.
    #[serde(rename = "BrokenReference")]
    BrokenReference,
    /// Other issues.
    #[serde(rename = "Other")]
    Other,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#schema.impls
impl ArchiveIssueCategory {
    /// Human-readable display name for the category.
    pub fn name(&self) -> &'static str {
        match self {
            ArchiveIssueCategory::MissingContent => "Missing Content",
            ArchiveIssueCategory::Hallucination => "Hallucination",
            ArchiveIssueCategory::FormatError => "Format Error",
            ArchiveIssueCategory::ChangelogError => "CHANGELOG Error",
            ArchiveIssueCategory::BrokenReference => "Broken Reference",
            ArchiveIssueCategory::Other => "Other",
        }
    }
}

/// Issue found during archive review.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveReviewIssue {
    /// Severity of the issue. Imported from sibling challenge module; not redeclared here.
    pub severity: IssueSeverity,
    /// Category of the issue.
    pub category: ArchiveIssueCategory,
    /// Spec file where the issue was found.
    pub spec_file: String,
    /// Human-readable description of the issue.
    pub description: String,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#schema.impls
impl ArchiveReviewIssue {
    /// Create a new archive review issue.
    pub fn new(
        severity: IssueSeverity,
        category: ArchiveIssueCategory,
        spec_file: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            category,
            spec_file: spec_file.into(),
            description: description.into(),
        }
    }
}

/// Result of a Codex archive quality review.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveReview {
    /// Overall verdict.
    pub verdict: ArchiveReviewVerdict,
    /// List of issues found during review.
    pub issues: Vec<ArchiveReviewIssue>,
    /// Summary of review findings.
    pub summary: String,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#schema.impls
impl ArchiveReview {
    /// Create a new archive review result.
    pub fn new(
        verdict: ArchiveReviewVerdict,
        issues: Vec<ArchiveReviewIssue>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            verdict,
            issues,
            summary: summary.into(),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#source
impl ArchiveReviewIssue {
    /// Format issue for display
    pub fn format(&self) -> String {
        format!(
            "[{}] {}: {} - {}",
            self.severity.name(),
            self.spec_file,
            self.category.name(),
            self.description
        )
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/archive_review.md#source
impl ArchiveReview {
    /// Check if review passed (Approved verdict)
    pub fn passed(&self) -> bool {
        self.verdict == ArchiveReviewVerdict::Approved
    }

    /// Check if review should block archive
    pub fn blocks_archive(&self) -> bool {
        matches!(
            self.verdict,
            ArchiveReviewVerdict::Reviewed | ArchiveReviewVerdict::Rejected
        )
    }

    /// Count issues by severity
    pub fn count_by_severity(&self, severity: IssueSeverity) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == severity)
            .count()
    }

    /// Format review summary for display
    pub fn format_summary(&self) -> String {
        format!(
            r#"Archive Review: {} {}

Issues:
- High:   {}
- Medium: {}
- Low:    {}

Summary: {}
"#,
            self.verdict.emoji(),
            self.verdict.name(),
            self.count_by_severity(IssueSeverity::High),
            self.count_by_severity(IssueSeverity::Medium),
            self.count_by_severity(IssueSeverity::Low),
            self.summary
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verdict_display() {
        assert_eq!(ArchiveReviewVerdict::Approved.name(), "APPROVED");
        assert_eq!(ArchiveReviewVerdict::Approved.emoji(), "✅");
        assert_eq!(ArchiveReviewVerdict::Reviewed.name(), "REVIEWED");
        assert_eq!(ArchiveReviewVerdict::Reviewed.emoji(), "⚠️");
    }

    #[test]
    fn test_review_passed() {
        let review =
            ArchiveReview::new(ArchiveReviewVerdict::Approved, vec![], "All checks passed");
        assert!(review.passed());
        assert!(!review.blocks_archive());
    }

    #[test]
    fn test_review_blocks() {
        let review = ArchiveReview::new(
            ArchiveReviewVerdict::Reviewed,
            vec![ArchiveReviewIssue::new(
                IssueSeverity::High,
                ArchiveIssueCategory::MissingContent,
                "auth.md",
                "Missing R3",
            )],
            "Issues found",
        );
        assert!(!review.passed());
        assert!(review.blocks_archive());
        assert_eq!(review.count_by_severity(IssueSeverity::High), 1);
    }
}

// CODEGEN-END
