// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-src.md#schema
// CODEGEN-BEGIN
// Cclab-Jet
//
// This crate provides the Jet build tool functionality.
// All commands are accessed through the unified CLI: `cclab jet <command>`

pub mod agent;
// Backward-compatibility aliases for pre-reorg paths.
pub use crate::agent::failures_text as agent_failures_text;
pub use crate::agent::jsonl as agent_jsonl;
pub mod asset;
pub mod browser;
pub mod browser_cli;
pub mod build_clean;
pub mod build_target;
pub mod bundler;
pub mod cdp_driver;
pub mod ci_summary;
pub mod cli;
pub mod codegen;
pub mod css;
pub mod dev_server;
pub mod e2e;
// Backward-compatibility aliases — `e2e_*` paths remain valid after the
// 2026-05-21 reorg moved the 16 e2e_* modules under `crate::e2e::*`.
pub use crate::e2e::actionability as e2e_actionability;
pub use crate::e2e::assertion_diff as e2e_assertion_diff;
pub use crate::e2e::browser_session as e2e_browser_session;
pub use crate::e2e::clock as e2e_clock;
pub use crate::e2e::discovery as e2e_discovery;
pub use crate::e2e::dom_snapshot as e2e_dom_snapshot;
pub use crate::e2e::explorer as e2e_explorer;
pub use crate::e2e::lifecycle as e2e_lifecycle;
pub use crate::e2e::network as e2e_network;
pub use crate::e2e::open_controls as e2e_open_controls;
pub use crate::e2e::open_replay as e2e_open_replay;
pub use crate::e2e::open_state as e2e_open_state;
pub use crate::e2e::permissions as e2e_permissions;
pub use crate::e2e::retry as e2e_retry;
pub use crate::e2e::screenshots as e2e_screenshots;
pub use crate::e2e::selectors as e2e_selectors;
pub use crate::e2e::step_artifacts as e2e_step_artifacts;
pub use crate::e2e::step_panels as e2e_step_panels;
pub use crate::e2e::storage as e2e_storage;
pub use crate::e2e::trace as e2e_trace;
pub use crate::e2e::video as e2e_video;
pub mod evidence;
// Backward-compatibility aliases for pre-reorg paths.
pub use crate::evidence::bundle as evidence_bundle;
pub use crate::evidence::writer as evidence_writer;
pub mod frontend;
pub mod pkg_manager;
pub mod pm_report;
// Backward-compatibility aliases for pre-reorg paths.
pub use crate::pm_report::deep_links as pm_report_deep_links;
pub use crate::pm_report::ia as pm_report_ia;
pub use crate::pm_report::loader as pm_report_loader;
pub use crate::pm_report::metadata as pm_report_metadata;
pub use crate::pm_report::nav as pm_report_nav;
pub use crate::pm_report::redaction as pm_report_redaction;
pub use crate::pm_report::states as pm_report_states;
pub mod report_package;
// playwright_shim moved under `crate::e2e::playwright_shim` (2026-05-21 reorg).
pub use crate::e2e::playwright_shim;
pub mod reporter;
pub mod rerun_manifest;
pub mod resolver;
pub mod result_envelope;
pub mod runner;
pub mod stories;
pub mod task_runner;
pub mod test_runner;
pub mod trace;
pub mod transform;
pub mod tsx_to_rust;
pub mod wasm_build;
pub mod wasm_dev;

// Re-export pnpm parity modules for convenience
pub use pkg_manager::{audit, gc, npmrc, nx, patch, publish, workspace};
// CODEGEN-END
