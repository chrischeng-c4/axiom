// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
// CODEGEN-BEGIN
//! Trace capture, format, and viewer for `jet test --trace`.
//!
//! Three sub-modules:
//! - `manifest` — `TraceManifest` + event types + NDJSON helpers
//! - `archive` — zip archive writer/reader
//! - `buffer` — `TraceBuffer` in-memory append buffer + `TraceMode` gating
//! - `server` — HTTP handler for the embedded viewer
//! - `view` — `jet trace view` entry point

// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R1

pub mod archive;
pub mod buffer;
pub mod manifest;
pub mod server;
pub mod view;

// Convenient re-exports
pub use archive::{write_trace_zip, TraceAsset};
pub use buffer::{TraceBuffer, TraceMode};
pub use manifest::{
    ActionKind, ActionStepEvent, ConsoleEvent, ConsoleLevel, NetworkEvent, ScreenshotEvent,
    TraceEvent, TraceManifest, TraceOutcome, MANIFEST_VERSION,
};
// CODEGEN-END
