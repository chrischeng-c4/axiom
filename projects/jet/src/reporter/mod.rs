// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-reporter.md#schema
// CODEGEN-BEGIN
//! HTML reporter module for the jet native test runner.
//!
//! Provides:
//! - [`html::HtmlReporter`] — consumes `TestReport` events, renders
//!   a deterministic self-contained `index.html`.
//! - [`parser`] — parses NDJSON wire-protocol event streams back into
//!   `TestReport` rows for offline processing.
//! - [`merge`] — merges N per-shard report directories into a unified report.
//!
// @spec enhancement-html-reporter-for-native-test-runner-spec#R1
// @spec enhancement-html-reporter-for-native-test-runner-spec#R4
// @spec enhancement-html-reporter-for-native-test-runner-spec#R7

pub mod html;
pub mod merge;
pub mod parser;

pub use html::HtmlReporter;
pub use merge::merge_reports;
// CODEGEN-END
