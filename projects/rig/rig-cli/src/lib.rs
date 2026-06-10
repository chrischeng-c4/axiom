//! Shared verb parse + dispatch for the `rig` agent-first CLI.
//!
//! Every verb produces a single `RigReport`; `print_report` emits it as
//! exactly one JSON document on stdout (diagnostics go to stderr).
//! JSON-on-stdout is the UNFLAGGED default; `--human` and `--compact` are
//! the only opt-ins.

pub mod dispatch;
