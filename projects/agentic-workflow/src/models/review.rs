// Re-export IssueSeverity from challenge module for consistency
pub use super::challenge::IssueSeverity;

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/models/review.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Category classifying the type of review issue.
/// Display impl is hand-written outside CODEGEN — external trait impl emission is a future generator feature.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/review.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IssueCategory {
    /// Security-related issue.
    #[serde(rename = "Security")]
    Security,
    /// Performance-related issue.
    #[serde(rename = "Performance")]
    Performance,
    /// Code style issue.
    #[serde(rename = "Style")]
    Style,
    /// Missing feature.
    #[serde(rename = "MissingFeature")]
    MissingFeature,
    /// Incorrect behavior.
    #[serde(rename = "WrongBehavior")]
    WrongBehavior,
    /// Consistency issue.
    #[serde(rename = "Consistency")]
    Consistency,
    /// Missing or insufficient test coverage.
    #[serde(rename = "TestCoverage")]
    TestCoverage,
    /// Unhandled edge case.
    #[serde(rename = "EdgeCase")]
    EdgeCase,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/review.md#schema.trait-impls.Display
impl std::fmt::Display for IssueCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueCategory::Security => write!(f, "Security"),
            IssueCategory::Performance => write!(f, "Performance"),
            IssueCategory::Style => write!(f, "Style"),
            IssueCategory::MissingFeature => write!(f, "Missing Feature"),
            IssueCategory::WrongBehavior => write!(f, "Wrong Behavior"),
            IssueCategory::Consistency => write!(f, "Consistency"),
            IssueCategory::TestCoverage => write!(f, "Test Coverage"),
            IssueCategory::EdgeCase => write!(f, "Edge Case"),
        }
    }
}

/// Represents a single issue found during review.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/review.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    /// Issue title.
    pub title: String,
    /// Severity level of the issue. Re-exported from models::challenge.
    pub severity: IssueSeverity,
    /// Category of the issue.
    pub category: IssueCategory,
    /// Location in the reviewed artifact (e.g., 'src/models/review.rs:42').
    pub location: String,
    /// Description of the issue.
    pub description: String,
    /// Recommended fix or improvement.
    pub recommendation: String,
}

/// Verdict from a review.
/// Approved: review is approved and ready for implementation.
/// Reviewed: review needs refinement to address issues.
/// Rejected: review is rejected due to major issues.
/// Unknown: verdict could not be determined (parsing error or not filled).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/review.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReviewVerdict {
    /// Review is approved and ready for implementation.
    #[serde(rename = "Approved")]
    Approved,
    /// Review needs refinement to address issues.
    #[serde(rename = "Reviewed")]
    Reviewed,
    /// Review is rejected due to major issues.
    #[serde(rename = "Rejected")]
    Rejected,
    /// Verdict could not be determined (parsing error or not filled).
    #[serde(rename = "Unknown")]
    Unknown,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/review.md#schema.impls
impl ReviewVerdict {
    /// True iff verdict is Approved.
    pub fn is_approved(&self) -> bool {
        matches!(self, Self::Approved)
    }

    /// True iff verdict is Reviewed (needs refinement).
    pub fn needs_refinement(&self) -> bool {
        matches!(self, Self::Reviewed)
    }

    /// True iff verdict is Rejected (has major issues).
    pub fn has_major_issues(&self) -> bool {
        matches!(self, Self::Rejected)
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/models/review.md#schema.trait-impls.Display
impl std::fmt::Display for ReviewVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewVerdict::Approved => write!(f, "APPROVED"),
            ReviewVerdict::Reviewed => write!(f, "REVIEWED"),
            ReviewVerdict::Rejected => write!(f, "REJECTED"),
            ReviewVerdict::Unknown => write!(f, "UNKNOWN"),
        }
    }
}
// CODEGEN-END
