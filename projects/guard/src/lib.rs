// SPEC-MANAGED: projects/guard/tech-design/semantic/source/projects-guard-src-lib-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! guard — security posture gate for the cclab ecosystem.
//!
//! `guard` owns security policy and report semantics. It consumes `compass`
//! for AST/lint/data-flow primitives, then emits one agent-readable security
//! report per run.

pub mod evidence;
pub mod report;
pub mod scan;

pub use evidence::{EvidenceCommand, EvidenceStatus, ExternalEvidence};
pub use report::{
    Completion, Finding, GuardReport, IntegrationMap, Location, OverallStatus, Severity, Summary,
    SCHEMA_VERSION,
};
pub use scan::{default_languages, scan_path, PolicyProfile, ScanOptions};
// CODEGEN-END
