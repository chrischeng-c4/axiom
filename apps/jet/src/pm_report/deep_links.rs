// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
// CODEGEN-BEGIN
//! PM report deep links for case and step selection (#2889).
//!
//! Static PM reports are served from a single HTML shell. To let a
//! reviewer share or bookmark a particular case/step, this module
//! defines a URL-fragment grammar and the round trip between a
//! [`crate::pm_report_nav::PmReportCursor`] and the fragment string.
//!
//! Grammar (URL-fragment, no leading `#`):
//! - `""` or `"/"` → summary page.
//! - `"/case/<case-id>"` → case timeline for the named case.
//! - `"/case/<case-id>/step/<step-id>"` → step highlighted inside the
//!   case timeline.
//! - `"/case/<case-id>/failure"` → failure-detail page for the case.
//! - `"/case/<case-id>/step/<step-id>/artifacts"` → artifacts tab for
//!   the selected step.
//!
//! Apply is read-only with respect to the nav state machine in
//! [`crate::pm_report_nav`]: every transition reuses the same
//! validation methods, so unknown cases / steps surface the same
//! [`NavRejection`] reasons the in-app clicks would. Auth, comments,
//! and live collaboration are out of scope.

use crate::pm_report_nav::{CaseNavInfo, NavPage, NavRejection, PmReportCursor};
use serde::{Deserialize, Serialize};

/// Stable schema tag for [`DeepLink`] serde shape.
pub const PM_REPORT_DEEP_LINK_SCHEMA_VERSION: &str = "jet.pm.report-deep-link.v1";

/// A deep link parsed from a URL fragment. Each variant maps to a
/// concrete cursor transition.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum DeepLink {
    Summary,
    Case { case_id: String },
    CaseStep { case_id: String, step_id: String },
    CaseFailure { case_id: String },
    CaseStepArtifacts { case_id: String, step_id: String },
}

/// Why a fragment string could not be parsed into a [`DeepLink`].
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
pub enum DeepLinkParseError {
    /// Fragment did not match any known shape.
    UnrecognizedShape { fragment: String },
    /// A path segment that should carry an id was empty (e.g.
    /// `"/case//step/foo"`).
    EmptySegment { fragment: String },
}

/// What happened when a deep link was applied to a cursor. Renderers
/// surface `NotFound` as a "clear not-found state" per the acceptance
/// criteria; `Applied` means the cursor moved.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum DeepLinkOutcome {
    Applied {
        page: NavPage,
        case_id: Option<String>,
        step_id: Option<String>,
    },
    NotFound {
        reason: DeepLinkNotFound,
    },
}

/// Why an otherwise-well-formed deep link could not select its target.
/// Mirrors [`NavRejection`] but kept distinct so renderers can show a
/// link-specific message ("This shared link points at a case that
/// isn't in the bundle") instead of the generic in-app one.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum DeepLinkNotFound {
    UnknownCase { case_id: String },
    UnknownStep { case_id: String, step_id: String },
    NoFailureInCase { case_id: String },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
impl From<NavRejection> for DeepLinkNotFound {
    fn from(value: NavRejection) -> Self {
        match value {
            NavRejection::UnknownCase { id } => Self::UnknownCase { case_id: id },
            NavRejection::UnknownStep { case_id, step_id } => {
                Self::UnknownStep { case_id, step_id }
            }
            NavRejection::NoFailureInCase { case_id } => Self::NoFailureInCase { case_id },
        }
    }
}

/// Parse a URL fragment string (without the leading `#`) into a
/// [`DeepLink`]. A leading `/` is tolerated.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
pub fn parse_fragment(fragment: &str) -> Result<DeepLink, DeepLinkParseError> {
    let trimmed = fragment.trim_start_matches('#');
    let trimmed = trimmed.trim_start_matches('/');
    if trimmed.is_empty() {
        return Ok(DeepLink::Summary);
    }
    let segments: Vec<&str> = trimmed.split('/').collect();
    let unrecognized = || DeepLinkParseError::UnrecognizedShape {
        fragment: fragment.to_string(),
    };
    let empty_segment = || DeepLinkParseError::EmptySegment {
        fragment: fragment.to_string(),
    };
    match segments.as_slice() {
        ["case", case_id] => {
            if case_id.is_empty() {
                return Err(empty_segment());
            }
            Ok(DeepLink::Case {
                case_id: (*case_id).to_string(),
            })
        }
        ["case", case_id, "failure"] => {
            if case_id.is_empty() {
                return Err(empty_segment());
            }
            Ok(DeepLink::CaseFailure {
                case_id: (*case_id).to_string(),
            })
        }
        ["case", case_id, "step", step_id] => {
            if case_id.is_empty() || step_id.is_empty() {
                return Err(empty_segment());
            }
            Ok(DeepLink::CaseStep {
                case_id: (*case_id).to_string(),
                step_id: (*step_id).to_string(),
            })
        }
        ["case", case_id, "step", step_id, "artifacts"] => {
            if case_id.is_empty() || step_id.is_empty() {
                return Err(empty_segment());
            }
            Ok(DeepLink::CaseStepArtifacts {
                case_id: (*case_id).to_string(),
                step_id: (*step_id).to_string(),
            })
        }
        _ => Err(unrecognized()),
    }
}

/// Encode a cursor's current selection back into a URL fragment. The
/// emitted fragment always round-trips through [`parse_fragment`].
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
pub fn cursor_to_fragment(cursor: &PmReportCursor) -> String {
    match (
        cursor.page,
        cursor.case_id.as_deref(),
        cursor.step_id.as_deref(),
    ) {
        (NavPage::Summary, _, _) => "/".to_string(),
        (NavPage::CaseTimeline, Some(c), None) => format!("/case/{c}"),
        (NavPage::CaseTimeline, Some(c), Some(s)) => format!("/case/{c}/step/{s}"),
        (NavPage::FailureDetail, Some(c), _) => format!("/case/{c}/failure"),
        (NavPage::Artifacts, Some(c), Some(s)) => {
            format!("/case/{c}/step/{s}/artifacts")
        }
        // Pages that need a case/step but the cursor lacks one fall
        // back to summary — invariant violation upstream, but we still
        // emit a usable link.
        _ => "/".to_string(),
    }
}

/// Apply a parsed deep link onto a cursor. Returns
/// [`DeepLinkOutcome::Applied`] with the resulting page on success and
/// [`DeepLinkOutcome::NotFound`] when the link names a case/step that
/// isn't in the supplied nav set.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
pub fn apply(
    link: &DeepLink,
    cursor: &mut PmReportCursor,
    cases: &[CaseNavInfo],
) -> DeepLinkOutcome {
    match link {
        DeepLink::Summary => {
            cursor.return_to_summary();
            DeepLinkOutcome::Applied {
                page: NavPage::Summary,
                case_id: None,
                step_id: None,
            }
        }
        DeepLink::Case { case_id } => match cursor.select_case(cases, case_id) {
            Ok(()) => DeepLinkOutcome::Applied {
                page: NavPage::CaseTimeline,
                case_id: Some(case_id.clone()),
                step_id: None,
            },
            Err(reason) => DeepLinkOutcome::NotFound {
                reason: reason.into(),
            },
        },
        DeepLink::CaseStep { case_id, step_id } => {
            match cursor.select_step(cases, case_id, step_id) {
                Ok(()) => DeepLinkOutcome::Applied {
                    page: NavPage::CaseTimeline,
                    case_id: Some(case_id.clone()),
                    step_id: Some(step_id.clone()),
                },
                Err(reason) => DeepLinkOutcome::NotFound {
                    reason: reason.into(),
                },
            }
        }
        DeepLink::CaseFailure { case_id } => match cursor.open_failure(cases, case_id) {
            Ok(()) => DeepLinkOutcome::Applied {
                page: NavPage::FailureDetail,
                case_id: Some(case_id.clone()),
                step_id: cursor.step_id.clone(),
            },
            Err(reason) => DeepLinkOutcome::NotFound {
                reason: reason.into(),
            },
        },
        DeepLink::CaseStepArtifacts { case_id, step_id } => {
            match cursor.open_artifacts(cases, case_id, step_id) {
                Ok(()) => DeepLinkOutcome::Applied {
                    page: NavPage::Artifacts,
                    case_id: Some(case_id.clone()),
                    step_id: Some(step_id.clone()),
                },
                Err(reason) => DeepLinkOutcome::NotFound {
                    reason: reason.into(),
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pm_report_nav::{CaseNavInfo, StepNavInfo};

    fn step(id: &str, is_failure: bool) -> StepNavInfo {
        StepNavInfo {
            id: id.into(),
            title: id.into(),
            is_failure,
            has_artifacts: true,
            is_skipped: false,
        }
    }

    fn cases() -> Vec<CaseNavInfo> {
        vec![CaseNavInfo {
            id: "case-checkout".into(),
            title: "Checkout flow".into(),
            steps: vec![step("nav", false), step("submit", true)],
            has_failure: true,
        }]
    }

    #[test]
    fn url_opens_intended_case_and_step_when_bundle_present() {
        // Stop condition (#2889): a static fixture supports one
        // case-step deep link; acceptance: URL opens intended case/step.
        let link = parse_fragment("/case/case-checkout/step/submit").unwrap();
        let mut cursor = PmReportCursor::default();
        let outcome = apply(&link, &mut cursor, &cases());
        match outcome {
            DeepLinkOutcome::Applied {
                page,
                case_id,
                step_id,
            } => {
                assert_eq!(page, NavPage::CaseTimeline);
                assert_eq!(case_id.as_deref(), Some("case-checkout"));
                assert_eq!(step_id.as_deref(), Some("submit"));
            }
            other => panic!("expected applied, got {other:?}"),
        }
        assert_eq!(cursor.page, NavPage::CaseTimeline);
        assert_eq!(cursor.case_id.as_deref(), Some("case-checkout"));
        assert_eq!(cursor.step_id.as_deref(), Some("submit"));
    }

    #[test]
    fn cursor_round_trips_through_fragment() {
        let mut cursor = PmReportCursor::default();
        cursor
            .select_step(&cases(), "case-checkout", "submit")
            .unwrap();
        let fragment = cursor_to_fragment(&cursor);
        assert_eq!(fragment, "/case/case-checkout/step/submit");

        let mut other = PmReportCursor::default();
        let link = parse_fragment(&fragment).unwrap();
        assert!(matches!(
            apply(&link, &mut other, &cases()),
            DeepLinkOutcome::Applied { .. }
        ));
        assert_eq!(other.case_id, cursor.case_id);
        assert_eq!(other.step_id, cursor.step_id);
        assert_eq!(other.page, cursor.page);
    }

    #[test]
    fn empty_and_root_fragments_map_to_summary() {
        for raw in ["", "/", "#/", "#"] {
            let link = parse_fragment(raw).unwrap();
            assert_eq!(link, DeepLink::Summary, "input was {raw:?}");
        }
    }

    #[test]
    fn case_failure_link_drills_into_failure_detail() {
        let link = parse_fragment("/case/case-checkout/failure").unwrap();
        let mut cursor = PmReportCursor::default();
        let outcome = apply(&link, &mut cursor, &cases());
        assert!(matches!(
            outcome,
            DeepLinkOutcome::Applied {
                page: NavPage::FailureDetail,
                ..
            }
        ));
        assert_eq!(cursor.page, NavPage::FailureDetail);
        assert_eq!(cursor.step_id.as_deref(), Some("submit"));
    }

    #[test]
    fn case_step_artifacts_link_lands_on_artifacts_page() {
        let link = parse_fragment("/case/case-checkout/step/submit/artifacts").unwrap();
        let mut cursor = PmReportCursor::default();
        let outcome = apply(&link, &mut cursor, &cases());
        assert!(matches!(
            outcome,
            DeepLinkOutcome::Applied {
                page: NavPage::Artifacts,
                ..
            }
        ));
        assert_eq!(cursor.page, NavPage::Artifacts);
    }

    #[test]
    fn unknown_case_link_falls_back_to_clear_not_found() {
        // Acceptance (#2889): invalid links fall back to a clear
        // not-found state.
        let link = parse_fragment("/case/case-missing").unwrap();
        let mut cursor = PmReportCursor::default();
        match apply(&link, &mut cursor, &cases()) {
            DeepLinkOutcome::NotFound {
                reason: DeepLinkNotFound::UnknownCase { case_id },
            } => {
                assert_eq!(case_id, "case-missing");
            }
            other => panic!("expected not-found, got {other:?}"),
        }
        // Cursor stays where it was (summary by default).
        assert_eq!(cursor.page, NavPage::Summary);
    }

    #[test]
    fn unknown_step_link_falls_back_with_step_in_reason() {
        let link = parse_fragment("/case/case-checkout/step/ghost").unwrap();
        let mut cursor = PmReportCursor::default();
        match apply(&link, &mut cursor, &cases()) {
            DeepLinkOutcome::NotFound {
                reason: DeepLinkNotFound::UnknownStep { case_id, step_id },
            } => {
                assert_eq!(case_id, "case-checkout");
                assert_eq!(step_id, "ghost");
            }
            other => panic!("expected unknown step, got {other:?}"),
        }
    }

    #[test]
    fn malformed_fragments_return_parse_errors() {
        match parse_fragment("/case//step/foo") {
            Err(DeepLinkParseError::EmptySegment { .. }) => {}
            other => panic!("expected empty segment, got {other:?}"),
        }
        match parse_fragment("/runs/123") {
            Err(DeepLinkParseError::UnrecognizedShape { .. }) => {}
            other => panic!("expected unrecognized shape, got {other:?}"),
        }
    }

    #[test]
    fn deep_link_round_trips_through_json() {
        let link = DeepLink::CaseStep {
            case_id: "case-x".into(),
            step_id: "step-y".into(),
        };
        let json = serde_json::to_string(&link).unwrap();
        let back: DeepLink = serde_json::from_str(&json).unwrap();
        assert_eq!(back, link);
        assert!(json.contains("\"kind\":\"case-step\""), "{json}");
    }
}
// CODEGEN-END
