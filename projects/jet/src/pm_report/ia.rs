// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
// CODEGEN-BEGIN
//! Read-only PM report information architecture (#2730).
//!
//! The PM web report is a read-only viewer over a finished evidence
//! bundle. Reviewers want to understand *what happened* and *why it
//! failed* — not to drive live execution. This module owns the IA:
//! the canonical section list, the navigation routes that surface
//! each section, and the projection from evidence-bundle fields to
//! the data each section needs.
//!
//! Run / pause / next-step / replay controls are excluded by
//! construction here: [`PmReportControls::is_live_disabled`] is the
//! single source of truth and is hard-wired to `true`. The PM report
//! renderer reads this flag to suppress the open-mode shortcut bar.
//!
//! Information architecture only. The actual HTML/CSS comes from
//! `report_package` / `e2e::render_pm_report_html`.

use serde::{Deserialize, Serialize};

/// Stable schema tag for [`PmReportIa`].
pub const PM_REPORT_IA_SCHEMA_VERSION: &str = "jet.pm.report-ia.v1";

/// Top-level navigation sections. Order matches the left-rail order in
/// the report; renderers should iterate this list, not enumerate the
/// variants themselves, so adding a section is a one-line change.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PmReportSection {
    /// Run-level summary: totals, duration, pass/fail/skipped counts.
    RunSummary,
    /// Flat list of cases with status badges and quick filters.
    CaseList,
    /// Per-case timeline of steps + their outcomes.
    CaseTimeline,
    /// Drilldown for a single failed step: assertion diff, console,
    /// network, artifacts. Reuses inspector data shapes.
    FailureDetail,
    /// Per-step artifact panel: screenshots, traces, videos.
    Artifacts,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
impl PmReportSection {
    /// Stable URL slug — used by [`PmReportRoute::for_section`] to
    /// build the static-report deep link.
    pub fn slug(self) -> &'static str {
        match self {
            Self::RunSummary => "summary",
            Self::CaseList => "cases",
            Self::CaseTimeline => "timeline",
            Self::FailureDetail => "failure",
            Self::Artifacts => "artifacts",
        }
    }
}

/// Read-only feature flags. Renderers must consult these instead of
/// hard-coding control visibility so the IA contract is honoured.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PmReportControls {
    /// Always `true`: PM report never exposes live run controls.
    pub is_live_disabled: bool,
    /// Always `false`: PM report never edits the underlying case
    /// source.
    pub allows_case_edit: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
impl Default for PmReportControls {
    fn default() -> Self {
        Self {
            is_live_disabled: true,
            allows_case_edit: false,
        }
    }
}

/// One navigation route. Generated from a section + optional case /
/// step ids so renderers can deep-link into the static bundle.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PmReportRoute {
    pub section: PmReportSection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
impl PmReportRoute {
    pub fn for_section(section: PmReportSection) -> Self {
        Self {
            section,
            case_id: None,
            step_id: None,
        }
    }

    pub fn for_case(section: PmReportSection, case_id: impl Into<String>) -> Self {
        Self {
            section,
            case_id: Some(case_id.into()),
            step_id: None,
        }
    }

    pub fn for_step(
        section: PmReportSection,
        case_id: impl Into<String>,
        step_id: impl Into<String>,
    ) -> Self {
        Self {
            section,
            case_id: Some(case_id.into()),
            step_id: Some(step_id.into()),
        }
    }

    /// Path fragment used in the static report URL (e.g.
    /// `#/cases/<case>/timeline`). Renderers may prefix with a static
    /// base path.
    pub fn path(&self) -> String {
        match (&self.case_id, &self.step_id) {
            (Some(c), Some(s)) => format!("#/cases/{c}/{}/{s}", self.section.slug()),
            (Some(c), None) => format!("#/cases/{c}/{}", self.section.slug()),
            (None, _) => format!("#/{}", self.section.slug()),
        }
    }
}

/// Maps evidence-bundle fields to the section that surfaces them.
/// Each entry names the section, the source field on
/// [`crate::evidence::EvidenceBundle`] or [`crate::e2e::E2eEvidenceBundle`],
/// and a one-line description for the IA spec.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PmReportFieldMapping {
    pub section: PmReportSection,
    /// Dot-path of the evidence bundle field (e.g.
    /// `cases[].steps[].assertion.diff`).
    pub evidence_field: String,
    pub description: String,
}

/// Canonical IA bundle. Renderers and tests load this struct rather
/// than re-deriving the section list per surface.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PmReportIa {
    pub schema_version: String,
    pub sections: Vec<PmReportSection>,
    pub controls: PmReportControls,
    pub field_mappings: Vec<PmReportFieldMapping>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
impl Default for PmReportIa {
    fn default() -> Self {
        Self {
            schema_version: PM_REPORT_IA_SCHEMA_VERSION.to_string(),
            sections: default_sections(),
            controls: PmReportControls::default(),
            field_mappings: default_field_mappings(),
        }
    }
}

/// Canonical section ordering. Public so renderers can iterate
/// without depending on the struct default.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
pub fn default_sections() -> Vec<PmReportSection> {
    vec![
        PmReportSection::RunSummary,
        PmReportSection::CaseList,
        PmReportSection::CaseTimeline,
        PmReportSection::FailureDetail,
        PmReportSection::Artifacts,
    ]
}

/// Canonical evidence-field -> section mapping. The strings are the
/// IA contract; renderers must not invent new sections.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
pub fn default_field_mappings() -> Vec<PmReportFieldMapping> {
    vec![
        PmReportFieldMapping {
            section: PmReportSection::RunSummary,
            evidence_field: "envelope.totals".into(),
            description: "Pass/fail/skipped counts plus wall duration".into(),
        },
        PmReportFieldMapping {
            section: PmReportSection::RunSummary,
            evidence_field: "envelope.run.started_at".into(),
            description: "Run start timestamp + runner version".into(),
        },
        PmReportFieldMapping {
            section: PmReportSection::CaseList,
            evidence_field: "envelope.cases[].status".into(),
            description: "Case-level status badges and filter source".into(),
        },
        PmReportFieldMapping {
            section: PmReportSection::CaseTimeline,
            evidence_field: "envelope.cases[].steps[]".into(),
            description: "Ordered step timeline with per-step outcomes".into(),
        },
        PmReportFieldMapping {
            section: PmReportSection::FailureDetail,
            evidence_field: "envelope.cases[].failure.diff".into(),
            description: "Assertion diff for the failed step".into(),
        },
        PmReportFieldMapping {
            section: PmReportSection::FailureDetail,
            evidence_field: "envelope.cases[].failure.console".into(),
            description: "Captured console output around the failure".into(),
        },
        PmReportFieldMapping {
            section: PmReportSection::Artifacts,
            evidence_field: "envelope.cases[].steps[].artifacts[]".into(),
            description: "Screenshots, traces, and videos per step".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ia_default_lists_four_top_level_sections_in_canonical_order() {
        // Stop condition (#2730): IA distinguishes run summary, case
        // list, timeline, and failure detail.
        let ia = PmReportIa::default();
        assert_eq!(
            ia.sections,
            vec![
                PmReportSection::RunSummary,
                PmReportSection::CaseList,
                PmReportSection::CaseTimeline,
                PmReportSection::FailureDetail,
                PmReportSection::Artifacts,
            ],
        );
    }

    #[test]
    fn pm_report_controls_are_read_only_by_construction() {
        // Stop condition (#2730): no live run/pause/replay in PM mode.
        let c = PmReportControls::default();
        assert!(c.is_live_disabled);
        assert!(!c.allows_case_edit);
    }

    #[test]
    fn every_section_has_at_least_one_evidence_mapping() {
        // Stop condition (#2730): IA implementable from evidence bundle.
        let ia = PmReportIa::default();
        for section in &ia.sections {
            let hit = ia.field_mappings.iter().any(|m| m.section == *section);
            assert!(hit, "no mapping for section {section:?}");
        }
    }

    #[test]
    fn route_paths_deep_link_into_cases_and_steps() {
        let summary = PmReportRoute::for_section(PmReportSection::RunSummary);
        assert_eq!(summary.path(), "#/summary");

        let case = PmReportRoute::for_case(PmReportSection::CaseTimeline, "flows/buy::buyer");
        assert_eq!(case.path(), "#/cases/flows/buy::buyer/timeline");

        let step =
            PmReportRoute::for_step(PmReportSection::FailureDetail, "flows/buy::buyer", "step-3");
        assert_eq!(step.path(), "#/cases/flows/buy::buyer/failure/step-3");
    }

    #[test]
    fn section_slugs_are_stable_kebab_case() {
        assert_eq!(PmReportSection::RunSummary.slug(), "summary");
        assert_eq!(PmReportSection::CaseList.slug(), "cases");
        assert_eq!(PmReportSection::CaseTimeline.slug(), "timeline");
        assert_eq!(PmReportSection::FailureDetail.slug(), "failure");
        assert_eq!(PmReportSection::Artifacts.slug(), "artifacts");
    }

    #[test]
    fn ia_round_trips_through_json() {
        let ia = PmReportIa::default();
        let json = serde_json::to_string(&ia).unwrap();
        let back: PmReportIa = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ia);
        assert!(json.contains("\"run-summary\""), "{json}");
        assert!(json.contains("\"is_live_disabled\":true"), "{json}");
    }

    #[test]
    fn unset_case_step_ids_skip_serialise() {
        let r = PmReportRoute::for_section(PmReportSection::RunSummary);
        let json = serde_json::to_string(&r).unwrap();
        assert!(!json.contains("case_id"), "{json}");
        assert!(!json.contains("step_id"), "{json}");
    }
}
// CODEGEN-END
