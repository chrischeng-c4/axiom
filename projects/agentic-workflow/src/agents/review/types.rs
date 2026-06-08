// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/agents/review/types.md#source
// CODEGEN-BEGIN
//! Review types used by [`ReviewAgent`] and [`CRRCycle`].

use serde::{Deserialize, Serialize};

/// What kind of artifact is being reviewed.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/types.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewType {
    Spec,
    Code,
}

/// Severity of a review issue.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/types.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    High,
    Medium,
    Low,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/types.md#schema.trait-impls.Display
impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::High => write!(f, "High"),
            Severity::Medium => write!(f, "Medium"),
            Severity::Low => write!(f, "Low"),
        }
    }
}

/// A single issue found during review.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewIssue {
    /// Issue severity.
    pub severity: Severity,
    /// Human-readable description of the issue.
    pub description: String,
    /// Suggested fix or follow-up action.
    pub suggestion: String,
    /// Optional spec/file location anchor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

/// Verdict returned by a reviewer. Internally-tagged enum with three variants — Approved (unit), NeedsRevision { issues } (struct), Rejected { reason } (struct).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "verdict", rename_all = "snake_case")]
pub enum ReviewVerdict {
    /// Reviewer approved the artifact.
    Approved,
    /// Reviewer flagged issues; artifact must be revised.
    NeedsRevision { issues: Vec<ReviewIssue> },
    /// Reviewer rejected the artifact outright.
    Rejected { reason: String },
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/review/types.md#source
impl ReviewVerdict {
    pub fn is_approved(&self) -> bool {
        matches!(self, ReviewVerdict::Approved)
    }

    pub fn is_needs_revision(&self) -> bool {
        matches!(self, ReviewVerdict::NeedsRevision { .. })
    }

    pub fn is_rejected(&self) -> bool {
        matches!(self, ReviewVerdict::Rejected { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verdict_serializes_with_tag() {
        let v = ReviewVerdict::NeedsRevision {
            issues: vec![ReviewIssue {
                severity: Severity::High,
                description: "Missing section".to_string(),
                suggestion: "Add overview".to_string(),
                location: Some("spec.md:1".to_string()),
            }],
        };
        let json = serde_json::to_value(&v).unwrap();
        assert_eq!(json["verdict"], "needs_revision");
        assert_eq!(json["issues"][0]["severity"], "high");
    }

    #[test]
    fn test_verdict_approved_serializes() {
        let v = ReviewVerdict::Approved;
        let json = serde_json::to_value(&v).unwrap();
        assert_eq!(json["verdict"], "approved");
    }

    #[test]
    fn test_verdict_helpers() {
        assert!(ReviewVerdict::Approved.is_approved());
        assert!(!ReviewVerdict::Approved.is_needs_revision());
        assert!(ReviewVerdict::NeedsRevision { issues: vec![] }.is_needs_revision());
        assert!(ReviewVerdict::Rejected { reason: "x".into() }.is_rejected());
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::High), "High");
        assert_eq!(format!("{}", Severity::Medium), "Medium");
        assert_eq!(format!("{}", Severity::Low), "Low");
    }
}

// CODEGEN-END
