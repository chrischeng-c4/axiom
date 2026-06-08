// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/models/source_reference.md#source
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-source-first-reference-artifacts.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceReferenceKind {
    VisualReference,
    CliTranscript,
    ApiContract,
    DocOutline,
    ValidationInventory,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-source-first-reference-artifacts.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceReferenceAvailability {
    Available,
    Missing,
    SourceNotAvailable,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-source-first-reference-artifacts.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFailureMode {
    HardFail,
    HitlRequired,
    Advisory,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-source-first-reference-artifacts.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceReferenceRequirement {
    pub kind: SourceReferenceKind,
    pub required: bool,
    pub failure_mode: SourceFailureMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-source-first-reference-artifacts.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceReferencePolicy {
    pub profile_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirements: Vec<SourceReferenceRequirement>,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-source-first-reference-artifacts.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceReference {
    pub id: String,
    pub kind: SourceReferenceKind,
    pub availability: SourceReferenceAvailability,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_response: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outline: Option<String>,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-source-first-reference-artifacts.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceReviewSeverity {
    Hard,
    Advisory,
    Hitl,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-source-first-reference-artifacts.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceReviewFinding {
    pub code: String,
    pub severity: SourceReviewSeverity,
    pub message: String,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-source-first-reference-artifacts.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceReferenceReview {
    pub source_backed: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<SourceReviewFinding>,
}

/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-source-first-reference-artifacts.md#logic
pub fn evaluate_source_references(
    policy: &SourceReferencePolicy,
    references: &[SourceReference],
    implementation_citations: &[String],
) -> SourceReferenceReview {
    let mut findings = Vec::new();

    for requirement in &policy.requirements {
        if !requirement.required {
            continue;
        }

        let reference = references
            .iter()
            .find(|reference| reference.kind == requirement.kind);
        match reference {
            Some(reference) => evaluate_reference(
                requirement,
                reference,
                implementation_citations,
                &mut findings,
            ),
            None => push_missing_source(requirement, &mut findings),
        }
    }

    let source_backed = !findings.iter().any(|finding| {
        matches!(
            finding.severity,
            SourceReviewSeverity::Hard | SourceReviewSeverity::Hitl
        )
    });

    SourceReferenceReview {
        source_backed,
        findings,
    }
}

fn evaluate_reference(
    requirement: &SourceReferenceRequirement,
    reference: &SourceReference,
    implementation_citations: &[String],
    findings: &mut Vec<SourceReviewFinding>,
) {
    match reference.availability {
        SourceReferenceAvailability::Available => {
            if !reference_has_evidence(reference) {
                findings.push(finding(
                    "empty_source_reference",
                    severity_for(requirement.failure_mode),
                    format!(
                        "{:?} source reference has no evidence body",
                        requirement.kind
                    ),
                ));
                return;
            }
            if !implementation_cites(reference, implementation_citations) {
                findings.push(finding(
                    "uncited_source_reference",
                    severity_for(requirement.failure_mode),
                    format!(
                        "implementation must cite source reference '{}'",
                        reference.id
                    ),
                ));
            }
        }
        SourceReferenceAvailability::Missing => push_missing_source(requirement, findings),
        SourceReferenceAvailability::SourceNotAvailable => findings.push(finding(
            "source_not_available",
            SourceReviewSeverity::Hitl,
            format!("{:?} source reference requires HITL", requirement.kind),
        )),
    }
}

fn push_missing_source(
    requirement: &SourceReferenceRequirement,
    findings: &mut Vec<SourceReviewFinding>,
) {
    findings.push(finding(
        "missing_source_reference",
        severity_for(requirement.failure_mode),
        format!("{:?} source reference is required", requirement.kind),
    ));
}

fn severity_for(mode: SourceFailureMode) -> SourceReviewSeverity {
    match mode {
        SourceFailureMode::HardFail => SourceReviewSeverity::Hard,
        SourceFailureMode::HitlRequired => SourceReviewSeverity::Hitl,
        SourceFailureMode::Advisory => SourceReviewSeverity::Advisory,
    }
}

fn finding(code: &str, severity: SourceReviewSeverity, message: String) -> SourceReviewFinding {
    SourceReviewFinding {
        code: code.to_string(),
        severity,
        message,
    }
}

fn reference_has_evidence(reference: &SourceReference) -> bool {
    [
        reference.citation.as_deref(),
        reference.transcript.as_deref(),
        reference.request_response.as_deref(),
        reference.outline.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(|value| !value.trim().is_empty())
}

fn implementation_cites(reference: &SourceReference, citations: &[String]) -> bool {
    citations.iter().any(|citation| {
        citation == &reference.id
            || reference
                .citation
                .as_deref()
                .is_some_and(|source_citation| citation == source_citation)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_reference_missing_required_source() {
        let policy = policy(
            SourceReferenceKind::CliTranscript,
            SourceFailureMode::HardFail,
        );

        let review = evaluate_source_references(&policy, &[], &[]);

        assert!(!review.source_backed);
        assert_eq!(review.findings.len(), 1);
        assert_eq!(review.findings[0].code, "missing_source_reference");
        assert_eq!(review.findings[0].severity, SourceReviewSeverity::Hard);
    }

    #[test]
    fn source_reference_api_contract_source_backed() {
        let policy = policy(
            SourceReferenceKind::ApiContract,
            SourceFailureMode::HardFail,
        );
        let reference = SourceReference {
            id: "api-contract:v1".to_string(),
            kind: SourceReferenceKind::ApiContract,
            availability: SourceReferenceAvailability::Available,
            citation: Some("contract://api/v1".to_string()),
            transcript: None,
            request_response: Some("GET /items -> 200 []".to_string()),
            outline: None,
        };

        let review =
            evaluate_source_references(&policy, &[reference], &["contract://api/v1".to_string()]);

        assert!(review.source_backed);
        assert!(review.findings.is_empty());
    }

    #[test]
    fn source_reference_not_available_requires_hitl() {
        let policy = policy(
            SourceReferenceKind::VisualReference,
            SourceFailureMode::HitlRequired,
        );
        let reference = SourceReference {
            id: "visual:missing".to_string(),
            kind: SourceReferenceKind::VisualReference,
            availability: SourceReferenceAvailability::SourceNotAvailable,
            citation: None,
            transcript: None,
            request_response: None,
            outline: None,
        };

        let review = evaluate_source_references(&policy, &[reference], &[]);

        assert!(!review.source_backed);
        assert_eq!(review.findings[0].code, "source_not_available");
        assert_eq!(review.findings[0].severity, SourceReviewSeverity::Hitl);
    }

    fn policy(kind: SourceReferenceKind, failure_mode: SourceFailureMode) -> SourceReferencePolicy {
        SourceReferencePolicy {
            profile_name: "test-profile".to_string(),
            requirements: vec![SourceReferenceRequirement {
                kind,
                required: true,
                failure_mode,
                rationale: Some("test requirement".to_string()),
            }],
        }
    }
}
// CODEGEN-END
