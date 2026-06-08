// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/quality_primitives.md#source
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

use crate::models::artifact_quality::ArtifactKind;

/// Primitive dial compatibility support level.
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-quality-primitive-metadata.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimitiveDialSupport {
    Required,
    Supported,
    Unsupported,
}

/// Review check severity for primitive metadata.
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-quality-primitive-metadata.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimitiveReviewSeverity {
    Hard,
    Advisory,
}

/// Evidence kind expected by a primitive profile.
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-quality-primitive-metadata.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimitiveEvidenceKind {
    Test,
    Screenshot,
    Transcript,
    LinkCheck,
    SourceAnnotation,
    ReviewNote,
}

/// Dial compatibility entry for one quality primitive.
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-quality-primitive-metadata.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimitiveDialCompatibility {
    pub dial: String,
    pub support: PrimitiveDialSupport,
    pub rationale: String,
}

/// Review check attached to a quality primitive profile.
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-quality-primitive-metadata.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimitiveReviewCheck {
    pub id: String,
    pub severity: PrimitiveReviewSeverity,
    pub description: String,
}

/// Evidence example attached to a quality primitive profile.
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-quality-primitive-metadata.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimitiveEvidenceExample {
    pub kind: PrimitiveEvidenceKind,
    pub description: String,
}

/// Inspectable metadata for generator primitive selection and review.
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-quality-primitive-metadata.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityPrimitiveProfile {
    pub name: String,
    pub artifact_kind: ArtifactKind,
    pub dial_compatibility: Vec<PrimitiveDialCompatibility>,
    pub when_to_use: Vec<String>,
    pub not_for: Vec<String>,
    pub required_inputs: Vec<String>,
    pub required_fallbacks: Vec<String>,
    pub anti_patterns: Vec<String>,
    pub review_checks: Vec<PrimitiveReviewCheck>,
    pub evidence_examples: Vec<PrimitiveEvidenceExample>,
}

/// Request used to explain whether a primitive profile applies.
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-quality-primitive-metadata.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimitiveSelectionRequest {
    pub primitive_name: String,
    pub artifact_kind: ArtifactKind,
    pub requested_dials: Vec<String>,
}

/// Metadata-backed explanation for a primitive selection decision.
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-quality-primitive-metadata.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimitiveSelectionCitation {
    pub primitive_name: String,
    pub applicable: bool,
    pub matched_fields: Vec<String>,
    pub rejected_fields: Vec<String>,
    pub evidence_expectations: Vec<String>,
}

/// Review finding emitted from primitive profile anti-pattern checks.
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-quality-primitive-metadata.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimitiveReviewFinding {
    pub check_id: String,
    pub severity: PrimitiveReviewSeverity,
    pub message: String,
}

/// Returns the built-in quality primitive metadata profiles.
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-quality-primitive-metadata.md#logic
pub fn default_quality_primitive_profiles() -> Vec<QualityPrimitiveProfile> {
    vec![
        QualityPrimitiveProfile {
            name: "frontend_page_responsive_shell".to_string(),
            artifact_kind: ArtifactKind::FrontendPage,
            dial_compatibility: vec![dial(
                "responsive_layout",
                PrimitiveDialSupport::Required,
                "page primitives must preserve mobile and desktop layouts",
            )],
            when_to_use: strings(&["a frontend page needs reusable layout scaffolding"]),
            not_for: strings(&["non-visual CLI or documentation artifacts"]),
            required_inputs: strings(&[
                "viewport targets",
                "navigation shape",
                "primary content regions",
            ]),
            required_fallbacks: strings(&["mobile layout fallback"]),
            anti_patterns: strings(&["text overlap", "nested cards", "unverified empty viewport"]),
            review_checks: vec![review_check(
                "viewport-proof",
                PrimitiveReviewSeverity::Hard,
                "artifact includes desktop and mobile render evidence",
            )],
            evidence_examples: vec![evidence(
                PrimitiveEvidenceKind::Screenshot,
                "desktop and mobile screenshot proof",
            )],
        },
        QualityPrimitiveProfile {
            name: "cli_help_command_tree".to_string(),
            artifact_kind: ArtifactKind::CliSurface,
            dial_compatibility: vec![dial(
                "command_discoverability",
                PrimitiveDialSupport::Required,
                "help output must expose command purpose and args",
            )],
            when_to_use: strings(&["a CLI command surface needs inspectable help text"]),
            not_for: strings(&["non-interactive data model generation"]),
            required_inputs: strings(&["command name", "argument list", "exit behavior"]),
            required_fallbacks: strings(&["plain text output fallback"]),
            anti_patterns: strings(&["hidden required args", "help text that omits side effects"]),
            review_checks: vec![review_check(
                "help-output-proof",
                PrimitiveReviewSeverity::Hard,
                "artifact includes captured help output",
            )],
            evidence_examples: vec![evidence(
                PrimitiveEvidenceKind::Transcript,
                "CLI help transcript",
            )],
        },
        QualityPrimitiveProfile {
            name: "documentation_capability_contract".to_string(),
            artifact_kind: ArtifactKind::Documentation,
            dial_compatibility: vec![dial(
                "evidence_backed_claims",
                PrimitiveDialSupport::Required,
                "documentation claims must cite verification or source refs",
            )],
            when_to_use: strings(&["a capability or workflow contract is documented"]),
            not_for: strings(&["source modules without public explanation surface"]),
            required_inputs: strings(&["capability root", "contract table", "verification refs"]),
            required_fallbacks: strings(&["review note when verification is manual"]),
            anti_patterns: strings(&["unverified completion claim", "stale issue reference"]),
            review_checks: vec![review_check(
                "docs-evidence-proof",
                PrimitiveReviewSeverity::Advisory,
                "documentation links claims to evidence",
            )],
            evidence_examples: vec![evidence(
                PrimitiveEvidenceKind::LinkCheck,
                "link or reference validation output",
            )],
        },
    ]
}

/// Finds a built-in quality primitive profile by name.
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-quality-primitive-metadata.md#logic
pub fn find_quality_primitive_profile(name: &str) -> Option<QualityPrimitiveProfile> {
    default_quality_primitive_profiles()
        .into_iter()
        .find(|profile| profile.name == name)
}

/// Validates required fields for quality primitive profiles.
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-quality-primitive-metadata.md#logic
pub fn validate_quality_primitive_profiles(profiles: &[QualityPrimitiveProfile]) -> Vec<String> {
    let mut errors = Vec::new();

    if profiles.len() < 3 {
        errors.push("at least three quality primitive profiles are required".to_string());
    }

    for profile in profiles {
        require_text(&mut errors, &profile.name, "name", &profile.name);
        require_vec(
            &mut errors,
            &profile.name,
            "dial_compatibility",
            &profile.dial_compatibility,
        );
        require_vec(
            &mut errors,
            &profile.name,
            "when_to_use",
            &profile.when_to_use,
        );
        require_vec(&mut errors, &profile.name, "not_for", &profile.not_for);
        require_vec(
            &mut errors,
            &profile.name,
            "required_inputs",
            &profile.required_inputs,
        );
        require_vec(
            &mut errors,
            &profile.name,
            "required_fallbacks",
            &profile.required_fallbacks,
        );
        require_vec(
            &mut errors,
            &profile.name,
            "anti_patterns",
            &profile.anti_patterns,
        );
        require_vec(
            &mut errors,
            &profile.name,
            "review_checks",
            &profile.review_checks,
        );
        require_vec(
            &mut errors,
            &profile.name,
            "evidence_examples",
            &profile.evidence_examples,
        );
    }

    errors
}

/// Explains primitive applicability without changing generator selection behavior.
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-quality-primitive-metadata.md#logic
pub fn explain_primitive_selection(
    profiles: &[QualityPrimitiveProfile],
    request: &PrimitiveSelectionRequest,
) -> PrimitiveSelectionCitation {
    let Some(profile) = profiles
        .iter()
        .find(|profile| profile.name == request.primitive_name)
    else {
        return PrimitiveSelectionCitation {
            primitive_name: request.primitive_name.clone(),
            applicable: false,
            matched_fields: Vec::new(),
            rejected_fields: vec!["name".to_string()],
            evidence_expectations: Vec::new(),
        };
    };

    let mut matched_fields = vec!["name".to_string()];
    let mut rejected_fields = Vec::new();

    if profile.artifact_kind == request.artifact_kind {
        matched_fields.push("artifact_kind".to_string());
    } else {
        rejected_fields.push("artifact_kind".to_string());
    }

    for dial_name in &request.requested_dials {
        match profile
            .dial_compatibility
            .iter()
            .find(|dial| dial.dial == *dial_name)
        {
            Some(dial) if dial.support != PrimitiveDialSupport::Unsupported => {
                matched_fields.push(format!("dial_compatibility:{dial_name}"));
            }
            _ => rejected_fields.push(format!("dial_compatibility:{dial_name}")),
        }
    }

    PrimitiveSelectionCitation {
        primitive_name: profile.name.clone(),
        applicable: rejected_fields.is_empty(),
        matched_fields,
        rejected_fields,
        evidence_expectations: profile
            .evidence_examples
            .iter()
            .map(|example| format!("{:?}: {}", example.kind, example.description))
            .collect(),
    }
}

/// Evaluates primitive anti-pattern checks against artifact review text.
/// @spec projects/agentic-workflow/tech-design/surface/specs/aw-quality-primitive-metadata.md#logic
pub fn evaluate_primitive_review_checks(
    profile: &QualityPrimitiveProfile,
    artifact_text: &str,
) -> Vec<PrimitiveReviewFinding> {
    let artifact_text = artifact_text.to_ascii_lowercase();
    let fallback_check = profile.review_checks.first();

    profile
        .anti_patterns
        .iter()
        .filter(|anti_pattern| artifact_text.contains(&anti_pattern.to_ascii_lowercase()))
        .map(|anti_pattern| {
            let (check_id, severity) = fallback_check
                .map(|check| (check.id.clone(), check.severity))
                .unwrap_or_else(|| ("anti-pattern".to_string(), PrimitiveReviewSeverity::Hard));
            PrimitiveReviewFinding {
                check_id,
                severity,
                message: format!(
                    "artifact matches quality primitive anti-pattern '{}'",
                    anti_pattern
                ),
            }
        })
        .collect()
}

fn dial(dial: &str, support: PrimitiveDialSupport, rationale: &str) -> PrimitiveDialCompatibility {
    PrimitiveDialCompatibility {
        dial: dial.to_string(),
        support,
        rationale: rationale.to_string(),
    }
}

fn review_check(
    id: &str,
    severity: PrimitiveReviewSeverity,
    description: &str,
) -> PrimitiveReviewCheck {
    PrimitiveReviewCheck {
        id: id.to_string(),
        severity,
        description: description.to_string(),
    }
}

fn evidence(kind: PrimitiveEvidenceKind, description: &str) -> PrimitiveEvidenceExample {
    PrimitiveEvidenceExample {
        kind,
        description: description.to_string(),
    }
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

fn require_text(errors: &mut Vec<String>, profile_name: &str, field: &str, value: &str) {
    if value.trim().is_empty() {
        errors.push(format!(
            "quality primitive profile '{profile_name}' missing {field}"
        ));
    }
}

fn require_vec<T>(errors: &mut Vec<String>, profile_name: &str, field: &str, value: &[T]) {
    if value.is_empty() {
        errors.push(format!(
            "quality primitive profile '{profile_name}' missing {field}"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_quality_primitive_profiles_validate() {
        let profiles = default_quality_primitive_profiles();

        assert_eq!(profiles.len(), 3);
        assert!(profiles
            .iter()
            .any(|profile| profile.name == "frontend_page_responsive_shell"));
        assert!(profiles
            .iter()
            .any(|profile| profile.name == "cli_help_command_tree"));
        assert!(profiles
            .iter()
            .any(|profile| profile.name == "documentation_capability_contract"));
        assert_eq!(
            validate_quality_primitive_profiles(&profiles),
            Vec::<String>::new()
        );
    }

    #[test]
    fn selection_citation_records_matched_fields_and_evidence() {
        let profiles = default_quality_primitive_profiles();
        let citation = explain_primitive_selection(
            &profiles,
            &PrimitiveSelectionRequest {
                primitive_name: "frontend_page_responsive_shell".to_string(),
                artifact_kind: ArtifactKind::FrontendPage,
                requested_dials: vec!["responsive_layout".to_string()],
            },
        );

        assert!(citation.applicable);
        assert!(citation
            .matched_fields
            .contains(&"artifact_kind".to_string()));
        assert!(citation
            .matched_fields
            .contains(&"dial_compatibility:responsive_layout".to_string()));
        assert_eq!(citation.rejected_fields, Vec::<String>::new());
        assert!(citation
            .evidence_expectations
            .iter()
            .any(|expectation| { expectation.contains("desktop and mobile screenshot proof") }));
    }

    #[test]
    fn selection_citation_records_rejected_dial() {
        let profiles = default_quality_primitive_profiles();
        let citation = explain_primitive_selection(
            &profiles,
            &PrimitiveSelectionRequest {
                primitive_name: "cli_help_command_tree".to_string(),
                artifact_kind: ArtifactKind::CliSurface,
                requested_dials: vec!["responsive_layout".to_string()],
            },
        );

        assert!(!citation.applicable);
        assert!(citation
            .rejected_fields
            .contains(&"dial_compatibility:responsive_layout".to_string()));
    }

    #[test]
    fn review_check_reports_matching_anti_pattern() {
        let profile = find_quality_primitive_profile("frontend_page_responsive_shell").unwrap();
        let findings = evaluate_primitive_review_checks(
            &profile,
            "The generated page has text overlap in the primary panel.",
        );

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].check_id, "viewport-proof");
        assert_eq!(findings[0].severity, PrimitiveReviewSeverity::Hard);
        assert!(findings[0].message.contains("text overlap"));
    }
}
// CODEGEN-END
