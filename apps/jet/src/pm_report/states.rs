// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
// CODEGEN-BEGIN
//! PM report load error and empty states (#2890).
//!
//! The static PM shell renders one of a small number of top-level
//! states depending on what the loader returned and what the bundle
//! actually contained. This module owns the classifier: it folds a
//! [`StaticReportLoadError`] or a content summary into a
//! [`ReportState`] enum that the renderer maps to a dedicated panel.
//!
//! Every state is read-only by construction — there's no run, replay
//! or repair affordance on any variant. Repairing or regenerating
//! evidence bundles is out of scope; the closest this surface gets is
//! a structured reason a non-dev PM reader can act on (file a bug,
//! re-run jet, ask for a re-export).
//!
//! Pair with [`crate::pm_report_loader::StaticReportBundle`] for the
//! happy path and with [`crate::pm_report_loader::StaticReportLoadError`]
//! for the failure variants.

use crate::pm_report_loader::StaticReportLoadError;
use serde::{Deserialize, Serialize};

/// Stable schema tag for [`ReportState`].
pub const PM_REPORT_STATES_SCHEMA_VERSION: &str = "jet.pm.report-states.v1";

/// Content summary the caller computes from the loaded bundle.
/// `cases` counts every case the bundle exposes; `failures` counts the
/// subset that ended in a failed outcome.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReportContentSummary {
    pub cases: u32,
    pub failures: u32,
}

/// One of the top-level states the PM shell can render. Each variant
/// is rendered as a distinct panel so non-dev readers get a
/// state-appropriate hint instead of a single generic error.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ReportState {
    /// Bundle loaded, has at least one case, has at least one failure.
    /// Full report UI renders.
    Ready {
        schema_version: String,
        summary: ReportContentSummary,
    },
    /// Manifest file wasn't found at the expected location.
    ManifestMissing {
        schema_version: String,
        path: String,
    },
    /// Manifest was found but didn't parse as the expected JSON shape.
    ManifestInvalid {
        schema_version: String,
        path: String,
        detail: String,
    },
    /// Manifest parsed but the schema_version isn't compatible with
    /// this viewer build.
    SchemaUnsupported { schema_version: String, tag: String },
    /// Bundle loaded fine but an artifact ref isn't portable. This is
    /// surfaced as an error state because the bundle is unsafe to
    /// render — paths could escape the report root.
    NonPortableArtifact {
        schema_version: String,
        path: String,
    },
    /// Bundle loaded but recorded zero cases.
    NoCases { schema_version: String },
    /// Bundle loaded and has cases but none of them failed. The PM
    /// shell hides failure-only panels and shows a celebratory empty
    /// state instead of an error.
    NoFailures { schema_version: String, cases: u32 },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
impl ReportState {
    /// True for any "everything is fine" state (currently just
    /// [`ReportState::Ready`] and [`ReportState::NoFailures`]).
    pub fn is_renderable_report(&self) -> bool {
        matches!(self, Self::Ready { .. } | Self::NoFailures { .. })
    }

    /// True if the state is a hard error (manifest missing, invalid,
    /// unsupported, or non-portable). The renderer disables navigation
    /// chrome on these states.
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            Self::ManifestMissing { .. }
                | Self::ManifestInvalid { .. }
                | Self::SchemaUnsupported { .. }
                | Self::NonPortableArtifact { .. }
        )
    }
}

/// Classify a load attempt + content summary into a [`ReportState`].
/// The caller is responsible for computing the content summary from
/// the bundle (cases / failures live above this module).
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
pub fn classify(
    load: Result<(), &StaticReportLoadError>,
    summary: ReportContentSummary,
) -> ReportState {
    match load {
        Ok(()) => {
            if summary.cases == 0 {
                ReportState::NoCases {
                    schema_version: PM_REPORT_STATES_SCHEMA_VERSION.to_string(),
                }
            } else if summary.failures == 0 {
                ReportState::NoFailures {
                    schema_version: PM_REPORT_STATES_SCHEMA_VERSION.to_string(),
                    cases: summary.cases,
                }
            } else {
                ReportState::Ready {
                    schema_version: PM_REPORT_STATES_SCHEMA_VERSION.to_string(),
                    summary,
                }
            }
        }
        Err(err) => from_error(err),
    }
}

fn from_error(err: &StaticReportLoadError) -> ReportState {
    let v = PM_REPORT_STATES_SCHEMA_VERSION.to_string();
    match err {
        StaticReportLoadError::ManifestMissing { path } => ReportState::ManifestMissing {
            schema_version: v,
            path: path.clone(),
        },
        StaticReportLoadError::ManifestInvalid { path, source } => ReportState::ManifestInvalid {
            schema_version: v,
            path: path.clone(),
            detail: source.to_string(),
        },
        StaticReportLoadError::SchemaUnsupported { tag } => ReportState::SchemaUnsupported {
            schema_version: v,
            tag: tag.clone(),
        },
        StaticReportLoadError::NonPortableArtifact { path } => ReportState::NonPortableArtifact {
            schema_version: v,
            path: path.clone(),
        },
        StaticReportLoadError::Io(io) => ReportState::ManifestInvalid {
            schema_version: v,
            path: "(unknown)".to_string(),
            detail: io.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_with_failures_renders_ready_state() {
        // Stop condition (#2890): fixture bundles cover each state.
        let state = classify(
            Ok(()),
            ReportContentSummary {
                cases: 3,
                failures: 1,
            },
        );
        match state {
            ReportState::Ready { summary, .. } => {
                assert_eq!(summary.cases, 3);
                assert_eq!(summary.failures, 1);
            }
            other => panic!("expected ready, got {other:?}"),
        }
    }

    #[test]
    fn missing_manifest_renders_missing_state() {
        let err = StaticReportLoadError::ManifestMissing {
            path: "/tmp/runs/manifest.json".into(),
        };
        match classify(Err(&err), ReportContentSummary::default()) {
            ReportState::ManifestMissing { path, .. } => {
                assert_eq!(path, "/tmp/runs/manifest.json");
            }
            other => panic!("expected missing, got {other:?}"),
        }
    }

    #[test]
    fn unsupported_schema_renders_unsupported_state() {
        let err = StaticReportLoadError::SchemaUnsupported {
            tag: "jet.evidence.bundle.v99".into(),
        };
        match classify(Err(&err), ReportContentSummary::default()) {
            ReportState::SchemaUnsupported { tag, .. } => {
                assert_eq!(tag, "jet.evidence.bundle.v99");
            }
            other => panic!("expected unsupported, got {other:?}"),
        }
    }

    #[test]
    fn loaded_bundle_with_zero_cases_renders_no_cases_state() {
        match classify(Ok(()), ReportContentSummary::default()) {
            ReportState::NoCases { .. } => {}
            other => panic!("expected no-cases, got {other:?}"),
        }
    }

    #[test]
    fn loaded_bundle_with_cases_but_no_failures_renders_no_failures_state() {
        let state = classify(
            Ok(()),
            ReportContentSummary {
                cases: 5,
                failures: 0,
            },
        );
        match state {
            ReportState::NoFailures { cases, .. } => assert_eq!(cases, 5),
            other => panic!("expected no-failures, got {other:?}"),
        }
    }

    #[test]
    fn non_portable_artifact_renders_dedicated_error_state() {
        let err = StaticReportLoadError::NonPortableArtifact {
            path: "/etc/passwd".into(),
        };
        let state = classify(Err(&err), ReportContentSummary::default());
        assert!(state.is_error());
        assert!(matches!(state, ReportState::NonPortableArtifact { .. }));
    }

    #[test]
    fn is_renderable_report_distinguishes_content_from_errors() {
        let ready = classify(
            Ok(()),
            ReportContentSummary {
                cases: 1,
                failures: 1,
            },
        );
        let no_failures = classify(
            Ok(()),
            ReportContentSummary {
                cases: 1,
                failures: 0,
            },
        );
        let no_cases = classify(Ok(()), ReportContentSummary::default());
        let missing = classify(
            Err(&StaticReportLoadError::ManifestMissing { path: "p".into() }),
            ReportContentSummary::default(),
        );
        assert!(ready.is_renderable_report());
        assert!(no_failures.is_renderable_report());
        // no_cases is not "renderable_report" because there's nothing
        // to render — it gets its own dedicated panel.
        assert!(!no_cases.is_renderable_report());
        assert!(missing.is_error());
    }

    #[test]
    fn no_state_exposes_live_dev_controls_via_serialization() {
        // Acceptance (#2890): No state exposes live dev controls. We
        // assert this structurally: serialising each variant must not
        // contain keys like "run", "replay", "pause", "control".
        let states = [
            classify(
                Ok(()),
                ReportContentSummary {
                    cases: 1,
                    failures: 1,
                },
            ),
            classify(Ok(()), ReportContentSummary::default()),
            classify(
                Ok(()),
                ReportContentSummary {
                    cases: 2,
                    failures: 0,
                },
            ),
            classify(
                Err(&StaticReportLoadError::ManifestMissing { path: "p".into() }),
                ReportContentSummary::default(),
            ),
            classify(
                Err(&StaticReportLoadError::SchemaUnsupported { tag: "x".into() }),
                ReportContentSummary::default(),
            ),
            classify(
                Err(&StaticReportLoadError::NonPortableArtifact { path: "x".into() }),
                ReportContentSummary::default(),
            ),
        ];
        for state in &states {
            let json = serde_json::to_string(state).unwrap();
            for forbidden in ["\"run\"", "\"replay\"", "\"pause\"", "\"control\""] {
                assert!(
                    !json.contains(forbidden),
                    "state {state:?} leaked control key {forbidden}: {json}"
                );
            }
        }
    }

    #[test]
    fn report_state_round_trips_through_json() {
        let state = classify(
            Ok(()),
            ReportContentSummary {
                cases: 2,
                failures: 1,
            },
        );
        let json = serde_json::to_string(&state).unwrap();
        let back: ReportState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, state);
        assert!(json.contains("\"kind\":\"ready\""), "{json}");
    }
}
// CODEGEN-END
