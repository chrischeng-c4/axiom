// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
// CODEGEN-BEGIN
//! Package-manager (`pm`) report — IA, navigation, states, metadata, loader,
//! redaction, deep links. Rendered to HTML / JSON by the reporter pipeline.

pub mod deep_links;
pub mod ia;
pub mod loader;
pub mod metadata;
pub mod nav;
pub mod redaction;
pub mod states;
// CODEGEN-END
