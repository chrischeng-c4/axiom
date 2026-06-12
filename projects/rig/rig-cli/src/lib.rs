// SPEC-MANAGED: projects/rig/tech-design/semantic/source/projects-rig-rig-cli-src-lib-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Shared verb parse + dispatch for the `rig` agent-first CLI.
//!
//! Every verb produces a single `RigReport`; `print_report` emits it as
//! exactly one JSON document on stdout (diagnostics go to stderr).
//! JSON-on-stdout is the UNFLAGGED default; `--human` and `--compact` are
//! the only opt-ins.

pub mod dispatch;
// CODEGEN-END
