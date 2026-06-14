// SPEC-MANAGED: projects/rig/tech-design/semantic/source/projects-rig-src-report-mod-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! rig's report surface: one JSON document per verb on stdout.

pub mod builder;
pub mod envelope;
pub mod finding;

pub use builder::{persist, ReportBuilder};
pub use envelope::{OverallStatus, RigReport, SCHEMA_VERSION};
pub use finding::{finding_id, Finding, Invoke, Kind, Severity};
// CODEGEN-END
