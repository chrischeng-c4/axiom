// SPEC-MANAGED: projects/guard/tech-design/semantic/source/projects-guard-src-lib-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! guard — security posture gate for the cclab ecosystem.
//!
//! `guard` is a first-line static security scanner: it consumes `compass` for
//! AST/lint/data-flow primitives, then emits one agent-readable security report
//! per run. It does not integrate upward into vat/rig/meter/arena — those are
//! upper-layer tools that may consume guard, never the reverse.

pub mod baseline;
pub mod config;
pub mod report;
pub mod scan;

pub use baseline::Baseline;
pub use config::GuardConfig;
pub use report::{
    Completion, Finding, GuardReport, IntegrationMap, Location, OverallStatus, Severity, Summary,
    SCHEMA_VERSION,
};
pub use scan::{
    default_languages, scan_path, scan_paths, scan_paths_with_options, PolicyProfile, ScanOptions,
};
// CODEGEN-END
