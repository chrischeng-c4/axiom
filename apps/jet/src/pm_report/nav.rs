// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
// CODEGEN-BEGIN
//! Read-only PM report navigation state (#2733).
//!
//! The PM web shell drives navigation through three observable
//! transitions: pick a case from the run summary, pick a step inside
//! that case, drill into the failure detail for the selected step.
//! This module owns the *state machine* — the navigation cursor and
//! the legal transitions — so the renderer (and tests) can react to
//! state changes without re-deriving them from URL fragments.
//!
//! The navigation surface is read-only by construction: every
//! transition leaves run controls absent. Compose the cursor with
//! [`crate::pm_report_ia::PmReportControls`] to confirm that
//! invariant; this module never exposes run/pause/replay hooks.

use crate::pm_report_ia::PmReportSection;
use serde::{Deserialize, Serialize};

/// Stable schema tag for the navigation cursor + its event log.
pub const PM_REPORT_NAV_SCHEMA_VERSION: &str = "jet.pm.report-nav.v1";

/// Lightweight summary of one case as the PM nav sees it. The full
/// case payload lives in the evidence bundle; this is the projection
/// the cursor needs to validate transitions and render an artifact
/// state.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseNavInfo {
    pub id: String,
    pub title: String,
    pub steps: Vec<StepNavInfo>,
    /// `true` if this case ended in a failure, so the cursor can
    /// auto-select the failed step when drilling down.
    pub has_failure: bool,
}

/// One step inside a case nav. `is_failure` distinguishes the failed
/// step that anchors the drilldown; `has_artifacts` controls whether
/// the artifacts tab renders content or the missing-artifacts state.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepNavInfo {
    pub id: String,
    pub title: String,
    pub is_failure: bool,
    pub has_artifacts: bool,
    /// `true` when the step ran but reported "skipped" (e.g. gated by
    /// a previous failure). Rendered as a dedicated read-only state.
    pub is_skipped: bool,
}

/// The pages the cursor can target. `Section` re-exports
/// [`PmReportSection`] for consistency but the cursor only navigates
/// between summary/timeline/failure/artifacts pages internally.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NavPage {
    Summary,
    CaseTimeline,
    FailureDetail,
    Artifacts,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
impl NavPage {
    fn section(self) -> PmReportSection {
        match self {
            Self::Summary => PmReportSection::RunSummary,
            Self::CaseTimeline => PmReportSection::CaseTimeline,
            Self::FailureDetail => PmReportSection::FailureDetail,
            Self::Artifacts => PmReportSection::Artifacts,
        }
    }
}

/// The cursor: which page, which case, which step. The struct is the
/// observable state; transitions go through methods so the legal
/// machine is enforced.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PmReportCursor {
    pub schema_version: String,
    pub page: NavPage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
    /// Append-only event log for analytics/tests. Renderers can ignore.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<NavEvent>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
impl Default for PmReportCursor {
    fn default() -> Self {
        Self {
            schema_version: PM_REPORT_NAV_SCHEMA_VERSION.to_string(),
            page: NavPage::Summary,
            case_id: None,
            step_id: None,
            events: Vec::new(),
        }
    }
}

/// Why the cursor refused a transition. Renderers can show a toast or
/// keep the previous state.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum NavRejection {
    UnknownCase { id: String },
    UnknownStep { case_id: String, step_id: String },
    NoFailureInCase { case_id: String },
}

/// One transition event. The log lets tests assert exact sequences
/// without comparing whole cursor snapshots.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NavEvent {
    SelectCase { case_id: String },
    SelectStep { case_id: String, step_id: String },
    OpenFailure { case_id: String },
    OpenArtifacts { case_id: String, step_id: String },
    ReturnToSummary,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
impl PmReportCursor {
    /// Bring the cursor onto a case's timeline. Validates that the
    /// case exists in the supplied nav set.
    pub fn select_case(
        &mut self,
        cases: &[CaseNavInfo],
        case_id: &str,
    ) -> Result<(), NavRejection> {
        if !cases.iter().any(|c| c.id == case_id) {
            return Err(NavRejection::UnknownCase {
                id: case_id.to_string(),
            });
        }
        self.page = NavPage::CaseTimeline;
        self.case_id = Some(case_id.to_string());
        self.step_id = None;
        self.events.push(NavEvent::SelectCase {
            case_id: case_id.to_string(),
        });
        Ok(())
    }

    /// Highlight a step inside the current case. Stays on the
    /// timeline page; drilldown into failure/artifacts is a separate
    /// transition.
    pub fn select_step(
        &mut self,
        cases: &[CaseNavInfo],
        case_id: &str,
        step_id: &str,
    ) -> Result<(), NavRejection> {
        let Some(case) = cases.iter().find(|c| c.id == case_id) else {
            return Err(NavRejection::UnknownCase {
                id: case_id.to_string(),
            });
        };
        if !case.steps.iter().any(|s| s.id == step_id) {
            return Err(NavRejection::UnknownStep {
                case_id: case_id.to_string(),
                step_id: step_id.to_string(),
            });
        }
        self.page = NavPage::CaseTimeline;
        self.case_id = Some(case_id.to_string());
        self.step_id = Some(step_id.to_string());
        self.events.push(NavEvent::SelectStep {
            case_id: case_id.to_string(),
            step_id: step_id.to_string(),
        });
        Ok(())
    }

    /// Drill into the failure detail for the selected case. Picks the
    /// case's failed step automatically.
    pub fn open_failure(
        &mut self,
        cases: &[CaseNavInfo],
        case_id: &str,
    ) -> Result<(), NavRejection> {
        let Some(case) = cases.iter().find(|c| c.id == case_id) else {
            return Err(NavRejection::UnknownCase {
                id: case_id.to_string(),
            });
        };
        let Some(step) = case.steps.iter().find(|s| s.is_failure) else {
            return Err(NavRejection::NoFailureInCase {
                case_id: case_id.to_string(),
            });
        };
        self.page = NavPage::FailureDetail;
        self.case_id = Some(case_id.to_string());
        self.step_id = Some(step.id.clone());
        self.events.push(NavEvent::OpenFailure {
            case_id: case_id.to_string(),
        });
        Ok(())
    }

    /// Move to the artifact tab for a specific step.
    pub fn open_artifacts(
        &mut self,
        cases: &[CaseNavInfo],
        case_id: &str,
        step_id: &str,
    ) -> Result<(), NavRejection> {
        let Some(case) = cases.iter().find(|c| c.id == case_id) else {
            return Err(NavRejection::UnknownCase {
                id: case_id.to_string(),
            });
        };
        if !case.steps.iter().any(|s| s.id == step_id) {
            return Err(NavRejection::UnknownStep {
                case_id: case_id.to_string(),
                step_id: step_id.to_string(),
            });
        }
        self.page = NavPage::Artifacts;
        self.case_id = Some(case_id.to_string());
        self.step_id = Some(step_id.to_string());
        self.events.push(NavEvent::OpenArtifacts {
            case_id: case_id.to_string(),
            step_id: step_id.to_string(),
        });
        Ok(())
    }

    /// Clear case/step selection and return to the run summary page.
    pub fn return_to_summary(&mut self) {
        self.page = NavPage::Summary;
        self.case_id = None;
        self.step_id = None;
        self.events.push(NavEvent::ReturnToSummary);
    }

    /// Snapshot of which top-level section is currently active. Lets
    /// the renderer highlight the left rail without inspecting the
    /// enum variant.
    pub fn active_section(&self) -> PmReportSection {
        self.page.section()
    }

    /// Render state hint for the artifacts panel. `MissingArtifact`
    /// means the bundle reported the step but no artifacts attach.
    pub fn artifact_state<'a>(&self, cases: &'a [CaseNavInfo]) -> Option<ArtifactPanelState<'a>> {
        let case = cases
            .iter()
            .find(|c| Some(&c.id) == self.case_id.as_ref())?;
        let step = case
            .steps
            .iter()
            .find(|s| Some(&s.id) == self.step_id.as_ref())?;
        Some(if step.is_skipped {
            ArtifactPanelState::SkippedStep { step }
        } else if step.has_artifacts {
            ArtifactPanelState::Available { step }
        } else {
            ArtifactPanelState::MissingArtifact { step }
        })
    }
}

/// Discriminated view of how the artifacts panel renders for the
/// currently selected step.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactPanelState<'a> {
    Available { step: &'a StepNavInfo },
    MissingArtifact { step: &'a StepNavInfo },
    SkippedStep { step: &'a StepNavInfo },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn case(id: &str, steps: Vec<StepNavInfo>) -> CaseNavInfo {
        let has_failure = steps.iter().any(|s| s.is_failure);
        CaseNavInfo {
            id: id.into(),
            title: id.into(),
            steps,
            has_failure,
        }
    }

    fn step(id: &str, failure: bool, artifacts: bool, skipped: bool) -> StepNavInfo {
        StepNavInfo {
            id: id.into(),
            title: id.into(),
            is_failure: failure,
            has_artifacts: artifacts,
            is_skipped: skipped,
        }
    }

    #[test]
    fn select_case_moves_cursor_to_timeline_and_logs_event() {
        // Stop condition (#2733): selecting a case changes read-only
        // report state.
        let cases = vec![case(
            "flows/buy",
            vec![
                step("s1", false, true, false),
                step("s2", true, true, false),
            ],
        )];
        let mut cursor = PmReportCursor::default();
        cursor.select_case(&cases, "flows/buy").unwrap();
        assert_eq!(cursor.page, NavPage::CaseTimeline);
        assert_eq!(cursor.case_id.as_deref(), Some("flows/buy"));
        assert!(cursor.step_id.is_none());
        assert_eq!(
            cursor.events.last(),
            Some(&NavEvent::SelectCase {
                case_id: "flows/buy".into()
            })
        );
    }

    #[test]
    fn select_step_records_step_without_changing_page() {
        let cases = vec![case(
            "flows/buy",
            vec![
                step("s1", false, true, false),
                step("s2", true, true, false),
            ],
        )];
        let mut cursor = PmReportCursor::default();
        cursor.select_case(&cases, "flows/buy").unwrap();
        cursor.select_step(&cases, "flows/buy", "s2").unwrap();
        assert_eq!(cursor.page, NavPage::CaseTimeline);
        assert_eq!(cursor.step_id.as_deref(), Some("s2"));
    }

    #[test]
    fn open_failure_auto_selects_failed_step() {
        let cases = vec![case(
            "flows/buy",
            vec![
                step("s1", false, true, false),
                step("boom", true, true, false),
            ],
        )];
        let mut cursor = PmReportCursor::default();
        cursor.open_failure(&cases, "flows/buy").unwrap();
        assert_eq!(cursor.page, NavPage::FailureDetail);
        assert_eq!(cursor.step_id.as_deref(), Some("boom"));
    }

    #[test]
    fn open_failure_rejects_clean_case() {
        let cases = vec![case("flows/ok", vec![step("s1", false, true, false)])];
        let mut cursor = PmReportCursor::default();
        let err = cursor.open_failure(&cases, "flows/ok").unwrap_err();
        assert_eq!(
            err,
            NavRejection::NoFailureInCase {
                case_id: "flows/ok".into()
            }
        );
    }

    #[test]
    fn unknown_case_or_step_returns_rejection() {
        let cases = vec![case("a", vec![step("s1", false, true, false)])];
        let mut cursor = PmReportCursor::default();
        assert_eq!(
            cursor.select_case(&cases, "missing").unwrap_err(),
            NavRejection::UnknownCase {
                id: "missing".into()
            }
        );
        cursor.select_case(&cases, "a").unwrap();
        assert_eq!(
            cursor.select_step(&cases, "a", "ghost").unwrap_err(),
            NavRejection::UnknownStep {
                case_id: "a".into(),
                step_id: "ghost".into()
            }
        );
    }

    #[test]
    fn artifact_state_distinguishes_missing_and_skipped() {
        // Stop condition (#2733): missing artifacts + skipped steps
        // expose clear read-only states.
        let cases = vec![case(
            "flows/buy",
            vec![
                step("with", true, true, false),
                step("without", false, false, false),
                step("skipped", false, false, true),
            ],
        )];
        let mut cursor = PmReportCursor::default();
        cursor.open_artifacts(&cases, "flows/buy", "with").unwrap();
        assert!(matches!(
            cursor.artifact_state(&cases),
            Some(ArtifactPanelState::Available { .. })
        ));
        cursor
            .open_artifacts(&cases, "flows/buy", "without")
            .unwrap();
        assert!(matches!(
            cursor.artifact_state(&cases),
            Some(ArtifactPanelState::MissingArtifact { .. })
        ));
        cursor
            .open_artifacts(&cases, "flows/buy", "skipped")
            .unwrap();
        assert!(matches!(
            cursor.artifact_state(&cases),
            Some(ArtifactPanelState::SkippedStep { .. })
        ));
    }

    #[test]
    fn return_to_summary_clears_case_step_and_logs() {
        let cases = vec![case("a", vec![step("s1", true, true, false)])];
        let mut cursor = PmReportCursor::default();
        cursor.open_failure(&cases, "a").unwrap();
        cursor.return_to_summary();
        assert_eq!(cursor.page, NavPage::Summary);
        assert!(cursor.case_id.is_none());
        assert!(cursor.step_id.is_none());
        assert_eq!(cursor.events.last(), Some(&NavEvent::ReturnToSummary));
    }

    #[test]
    fn active_section_tracks_page() {
        let mut cursor = PmReportCursor::default();
        assert_eq!(cursor.active_section(), PmReportSection::RunSummary);
        let cases = vec![case("a", vec![step("s1", true, false, false)])];
        cursor.open_failure(&cases, "a").unwrap();
        assert_eq!(cursor.active_section(), PmReportSection::FailureDetail);
    }

    #[test]
    fn cursor_round_trips_through_json() {
        let cases = vec![case("a", vec![step("s1", true, true, false)])];
        let mut cursor = PmReportCursor::default();
        cursor.open_failure(&cases, "a").unwrap();
        let json = serde_json::to_string(&cursor).unwrap();
        let back: PmReportCursor = serde_json::from_str(&json).unwrap();
        assert_eq!(back, cursor);
        assert!(json.contains("\"page\":\"failure-detail\""), "{json}");
    }
}
// CODEGEN-END
