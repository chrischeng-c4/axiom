// SPEC-MANAGED: projects/rig/tech-design/semantic/source/projects-rig-src-lib-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! rig — declarative test-scenario harness engine.
//!
//! Runs declarative SCENARIOS (e2e behavior steps) and LOAD profiles
//! (open-loop QPS) against a real serving process, judged by assertions and
//! declarative pins (floors/ratchets), emitting ONE agent-readable JSON
//! report per verb.
//!
//! Division of labor in the ecosystem: vat owns the environment (services,
//! workspace, readiness), rig owns case orchestration + assertions + gates,
//! meter owns resource attribution.

pub mod config;
pub mod discovery;
pub mod engine;
pub mod pins;
pub mod report;
pub mod scenario;
pub mod vat;
pub mod verdict;
// CODEGEN-END
