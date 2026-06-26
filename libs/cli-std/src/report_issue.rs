//! Deprecated compatibility alias of [`crate::issue`].
//!
//! The `<tool> report-issue` command is being replaced by the `<tool> issue
//! <verb>` group (`search` / `view` / `create`), so the logic now lives in
//! [`crate::issue`]. This shim keeps tools that have not yet migrated their CLI
//! surface (keep / loom / lumen) building unchanged — they call
//! `cli_std::report_issue::run(&tool, Options { .. })`, which forwards to
//! [`crate::issue::create`]. Drop this module once those tools adopt `issue`.

pub use crate::issue::{create as run, CreateOptions as Options};
