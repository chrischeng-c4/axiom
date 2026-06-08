// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
// CODEGEN-BEGIN
//! Report metadata projection for PM web reports (#2734).
//!
//! The PM summary card needs commit/branch/runner/build-time context.
//! That data already lives in [`BundleManifest`] — this module owns
//! the projection that converts the raw manifest into the
//! human-readable strings the report renders, with two safety rules:
//!
//! 1. Missing fields render as the explicit "unavailable" sentinel,
//!    never as guesses or blanks. The PM shell uses
//!    [`UNAVAILABLE_LABEL`] verbatim.
//! 2. Environment variables are filtered through the redaction policy
//!    from [`crate::pm_report_redaction`] before display, so a
//!    shareable bundle never leaks secrets into the metadata card.

use crate::evidence_bundle::{BundleCommand, BundleManifest};
use crate::pm_report_redaction::RedactionPolicy;
use serde::{Deserialize, Serialize};

/// Stable schema tag for [`PmReportMetadata`].
pub const PM_REPORT_METADATA_SCHEMA_VERSION: &str = "jet.pm.report-metadata.v1";

/// Sentinel string the PM shell renders when a manifest field is
/// missing. Centralised so renderers don't invent variants.
pub const UNAVAILABLE_LABEL: &str = "unavailable";

/// Display projection for the PM summary card. Every field is a
/// `String` so the renderer can pass values verbatim; missing data is
/// represented by [`UNAVAILABLE_LABEL`].
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PmReportMetadata {
    pub schema_version: String,
    pub run_id: String,
    pub command: String,
    pub project: String,
    pub commit: String,
    pub os: String,
    pub runner_version: String,
    pub ci: String,
    pub node_version: String,
    /// Optional generated-at timestamp pulled from a bundle adapter.
    /// Independent of the manifest because some packagers add it
    /// later.
    pub generated_at: String,
    /// Optional branch name, also added post-manifest by packagers
    /// that record git context.
    pub branch: String,
    /// Filtered environment list — redaction policy removes secrets;
    /// nothing here is ever the raw env block from the host.
    pub redacted_env: Vec<EnvDisplayPair>,
}

/// One row in the env display table. Values matching the policy
/// secret list are rewritten to the redaction placeholder before they
/// reach this struct.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvDisplayPair {
    pub key: String,
    pub value: String,
}

/// Extra context the bundle adapter knows but the on-disk manifest
/// does not directly carry (generation timestamp, source branch).
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, Default)]
pub struct AdapterContext<'a> {
    pub generated_at: Option<&'a str>,
    pub branch: Option<&'a str>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
impl PmReportMetadata {
    /// Build the projection from a manifest + adapter context +
    /// redaction policy. Use [`RedactionPolicy::shareable_baseline`]
    /// for any shareable bundle.
    pub fn from_manifest(
        manifest: &BundleManifest,
        adapter: AdapterContext<'_>,
        env_pairs: &[(&str, &str)],
        policy: &RedactionPolicy,
    ) -> Self {
        let redacted_env = env_pairs
            .iter()
            .map(|(k, v)| EnvDisplayPair {
                key: (*k).to_string(),
                value: policy.redact_env(k, v),
            })
            .collect();
        Self {
            schema_version: PM_REPORT_METADATA_SCHEMA_VERSION.to_string(),
            run_id: present_or_unavailable(&manifest.run_id),
            command: command_label(manifest.command),
            project: present_or_unavailable(&manifest.project),
            commit: present_or_unavailable(&manifest.commit),
            os: present_or_unavailable(&manifest.environment.os),
            runner_version: present_or_unavailable(&manifest.environment.runner_version),
            ci: manifest
                .environment
                .ci
                .as_deref()
                .map(present_or_unavailable)
                .unwrap_or_else(|| UNAVAILABLE_LABEL.to_string()),
            node_version: manifest
                .environment
                .node_version
                .as_deref()
                .map(present_or_unavailable)
                .unwrap_or_else(|| UNAVAILABLE_LABEL.to_string()),
            generated_at: adapter
                .generated_at
                .map(present_or_unavailable)
                .unwrap_or_else(|| UNAVAILABLE_LABEL.to_string()),
            branch: adapter
                .branch
                .map(present_or_unavailable)
                .unwrap_or_else(|| UNAVAILABLE_LABEL.to_string()),
            redacted_env,
        }
    }
}

fn present_or_unavailable(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        UNAVAILABLE_LABEL.to_string()
    } else {
        trimmed.to_string()
    }
}

fn command_label(c: BundleCommand) -> String {
    match c {
        BundleCommand::Test => "test".into(),
        BundleCommand::E2e => "e2e".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence_bundle::{BundleEnvironment, BUNDLE_SCHEMA_VERSION};

    fn manifest(env_ci: Option<&str>, node: Option<&str>) -> BundleManifest {
        BundleManifest {
            schema_version: BUNDLE_SCHEMA_VERSION.into(),
            run_id: "run-42".into(),
            command: BundleCommand::E2e,
            project: "demo".into(),
            commit: "cafef00d".into(),
            environment: BundleEnvironment {
                os: "darwin".into(),
                runner_version: "0.3.60".into(),
                ci: env_ci.map(|s| s.into()),
                node_version: node.map(|s| s.into()),
            },
            artifacts: Vec::new(),
        }
    }

    #[test]
    fn present_fields_pass_through() {
        // Stop condition (#2734): present metadata renders.
        let m = manifest(Some("github-actions"), Some("20.11.0"));
        let meta = PmReportMetadata::from_manifest(
            &m,
            AdapterContext {
                generated_at: Some("2026-05-20T03:14:15Z"),
                branch: Some("project-jet"),
            },
            &[],
            &RedactionPolicy::shareable_baseline(),
        );
        assert_eq!(meta.run_id, "run-42");
        assert_eq!(meta.command, "e2e");
        assert_eq!(meta.commit, "cafef00d");
        assert_eq!(meta.runner_version, "0.3.60");
        assert_eq!(meta.ci, "github-actions");
        assert_eq!(meta.node_version, "20.11.0");
        assert_eq!(meta.generated_at, "2026-05-20T03:14:15Z");
        assert_eq!(meta.branch, "project-jet");
    }

    #[test]
    fn missing_fields_render_as_unavailable_sentinel() {
        // Stop condition (#2734): missing metadata shows unavailable.
        let m = manifest(None, None);
        let meta = PmReportMetadata::from_manifest(
            &m,
            AdapterContext::default(),
            &[],
            &RedactionPolicy::shareable_baseline(),
        );
        assert_eq!(meta.ci, UNAVAILABLE_LABEL);
        assert_eq!(meta.node_version, UNAVAILABLE_LABEL);
        assert_eq!(meta.generated_at, UNAVAILABLE_LABEL);
        assert_eq!(meta.branch, UNAVAILABLE_LABEL);
    }

    #[test]
    fn empty_or_whitespace_manifest_strings_become_unavailable() {
        let mut m = manifest(Some("   "), None);
        m.commit = "".into();
        let meta = PmReportMetadata::from_manifest(
            &m,
            AdapterContext::default(),
            &[],
            &RedactionPolicy::shareable_baseline(),
        );
        assert_eq!(meta.commit, UNAVAILABLE_LABEL);
        assert_eq!(meta.ci, UNAVAILABLE_LABEL);
    }

    #[test]
    fn secret_env_values_are_redacted_before_display() {
        // Stop condition (#2734): secret-like env values aren't shown.
        let m = manifest(Some("github-actions"), None);
        let env = vec![
            ("PATH", "/usr/bin"),
            ("STRIPE_SECRET_KEY", "sk_live_abc"),
            ("DEPLOY_TOKEN", "xyz"),
        ];
        let meta = PmReportMetadata::from_manifest(
            &m,
            AdapterContext::default(),
            &env,
            &RedactionPolicy::shareable_baseline(),
        );
        let stripe = meta
            .redacted_env
            .iter()
            .find(|p| p.key == "STRIPE_SECRET_KEY")
            .unwrap();
        let deploy = meta
            .redacted_env
            .iter()
            .find(|p| p.key == "DEPLOY_TOKEN")
            .unwrap();
        let path = meta.redacted_env.iter().find(|p| p.key == "PATH").unwrap();
        assert_eq!(stripe.value, "[REDACTED]");
        assert_eq!(deploy.value, "[REDACTED]");
        assert_eq!(path.value, "/usr/bin");
    }

    #[test]
    fn command_label_covers_known_variants() {
        let m = manifest(None, None);
        let m2 = BundleManifest {
            command: BundleCommand::Test,
            ..m.clone()
        };
        let p = RedactionPolicy::shareable_baseline();
        assert_eq!(
            PmReportMetadata::from_manifest(&m, AdapterContext::default(), &[], &p).command,
            "e2e",
        );
        assert_eq!(
            PmReportMetadata::from_manifest(&m2, AdapterContext::default(), &[], &p).command,
            "test",
        );
    }

    #[test]
    fn metadata_round_trips_through_json() {
        let m = manifest(Some("github-actions"), Some("20"));
        let meta = PmReportMetadata::from_manifest(
            &m,
            AdapterContext {
                generated_at: Some("2026-05-20T00:00:00Z"),
                branch: Some("main"),
            },
            &[("PATH", "/x")],
            &RedactionPolicy::shareable_baseline(),
        );
        let json = serde_json::to_string(&meta).unwrap();
        let back: PmReportMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back, meta);
    }
}
// CODEGEN-END
