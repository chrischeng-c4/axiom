// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-report-mod-rs.md#source
// CODEGEN-BEGIN
//! Agent-first report layer — the center of the `meter` agent surface.
//!
//! Every `meter` verb funnels its result through [`ReportBuilder::finalize`] into a
//! single self-describing [`MeterReport`], which [`emit`] prints as exactly one JSON
//! document on stdout (diagnostics go to stderr). Populator verbs persist their
//! report to `.meter/last-report.json` via [`persist::write_last_report`] so the
//! read-only `report`/`state` verb can re-project with zero engine work.
//!
//! This module is ALWAYS compiled (not feature-gated): it is a pure data +
//! serialization layer with no spawn/IO side effects beyond `emit`/`persist`,
//! and the `meter` crate stays a clean rlib for its mamba + pgkit consumers.

pub mod builder;
pub mod emit;
pub mod env;
pub mod envelope;
pub mod finding;
pub mod persist;
pub mod producer;
pub mod schema;

// Public surface re-exports.
pub use builder::ReportBuilder;
pub use emit::{diag, emit, render};
pub use envelope::{
    Completion, EnvBlock, FindingsSummary, MeterReport, OverallStatus, RunnerRecord, SCHEMA_VERSION,
};
pub use finding::{finding_id, Finding, Invoke, Kind, Location, Severity};
pub use persist::{read_last_report, write_last_report};
pub use producer::IntoFindings;
pub use schema::{catalog, json_schema};
// CODEGEN-END
