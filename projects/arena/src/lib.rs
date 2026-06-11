//! arena — an N-target competitive comparison runner above rig/meter.
//!
//! arena runs the SAME logical "cell" against N targets, reduces each to one
//! comparable scalar, computes `ratio = peer/base`, classifies the cell as
//! WIN / EXEMPT / TARGET, gates WIN cells against a ratcheted per-host
//! baseline, and emits ONE comparison report.
//!
//! Measurement is delegated wholesale: service targets reuse
//! [`rig::engine::loadgen`]; the runtime flavor (deferred) shells out to
//! `meter profile`. Per-target workload TRANSLATION (lumen-JSON vs pg-SQL vs
//! OS-DSL) stays glue in the spec — arena never reads request bodies.
//!
//! Ecosystem layering: vat = outer container (provisions each target's env),
//! arena = middle compare layer, rig/meter = per-target measurement units.

pub mod compare;
pub mod engine;
pub mod measure;
pub mod report;
pub mod spec;

pub use engine::{run, RunOpts};
pub use report::ArenaReport;
pub use spec::Spec;
