// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
// CODEGEN-BEGIN
//! TraceManifest and all TraceEvent variants with serde Serialize/Deserialize.
//! NDJSON serialization helpers.
//!
//! The manifest is the first NDJSON line in `manifest.ndjson` (inside
//! `trace.zip`). Each subsequent line is one serialised `TraceEvent`.

// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R3
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R2

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schema version — bump on breaking changes.
pub const MANIFEST_VERSION: u32 = 1;

/// Top-level trace manifest. First NDJSON line in `manifest.ndjson`.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceManifest {
    /// Schema version; always `1` for this release.
    pub version: u32,
    /// Stable slug derived from spec file path + test title.
    pub test_id: String,
    /// Workspace-relative path to the `.spec.ts` file.
    pub spec_file: String,
    /// Full test title including describe nesting (joined by " > ").
    pub test_title: String,
    /// Outcome of the test.
    pub outcome: TraceOutcome,
    /// Unix timestamp in milliseconds (wall-clock) when the test started.
    pub started_at: u64,
    /// Unix timestamp in milliseconds when the test finished.
    pub finished_at: u64,
    /// Ordered trace events.
    pub events: Vec<TraceEvent>,
    /// Map of asset_id -> zip entry path for all binary assets.
    #[serde(default)]
    pub assets: HashMap<String, String>,
}

/// Outcome stored in the trace manifest.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TraceOutcome {
    Passed,
    Failed,
    TimedOut,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
impl std::fmt::Display for TraceOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceOutcome::Passed => write!(f, "passed"),
            TraceOutcome::Failed => write!(f, "failed"),
            TraceOutcome::TimedOut => write!(f, "timed-out"),
        }
    }
}

/// Unified trace event — one JSON object per NDJSON line after the manifest header.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R2
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TraceEvent {
    /// A browser action (click, fill, goto, etc.).
    ActionStep(ActionStepEvent),
    /// A console message from the browser context.
    Console(ConsoleEvent),
    /// A network request/response pair.
    Network(NetworkEvent),
    /// An explicit screenshot taken during the test.
    Screenshot(ScreenshotEvent),
}

/// A browser action step.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionStepEvent {
    /// Monotonically increasing step index within the test.
    pub step_id: u32,
    /// Kind of action performed.
    pub action: ActionKind,
    /// CSS/ARIA/text selector, null for page-level actions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// Present for goto actions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Milliseconds since test start.
    pub ts_start: u64,
    /// Milliseconds since test start.
    pub ts_end: u64,
    /// Asset id for the post-action DOM snapshot HTML.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dom_snapshot_ref: Option<String>,
    /// Asset id for the post-action PNG screenshot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot_ref: Option<String>,
    /// Error message if the action threw; null on success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Kinds of browser actions traceable by the runner.
/// @spec .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    Click,
    Fill,
    Goto,
    Evaluate,
    Screenshot,
    WaitFor,
    Hover,
    Check,
    Uncheck,
    TypeText,
}

/// A console message from the browser context.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleEvent {
    pub level: ConsoleLevel,
    pub text: String,
    /// Milliseconds since test start.
    pub ts: u64,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConsoleLevel {
    Log,
    Info,
    Warn,
    Error,
    Debug,
}

/// A network request/response pair observed during the test.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    pub request_id: String,
    pub url: String,
    pub method: String,
    /// HTTP response status; null if request never completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    /// Milliseconds since test start.
    pub ts_start: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts_end: Option<u64>,
    #[serde(default)]
    pub request_headers: HashMap<String, String>,
    #[serde(default)]
    pub response_headers: HashMap<String, String>,
}

/// An explicit screenshot captured during the test.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R2
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R9
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotEvent {
    /// Asset id for the PNG screenshot.
    pub screenshot_ref: String,
    /// Milliseconds since test start.
    pub ts: u64,
}

// ── NDJSON helpers ────────────────────────────────────────────────────────────

/// Serialise a `TraceManifest` (without the `events` field for the header line)
/// followed by each event as separate NDJSON lines.
///
/// Format:
/// ```text
/// <manifest_json>\n
/// <event1_json>\n
/// <event2_json>\n
/// ...
/// ```
/// @spec .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
pub fn encode_ndjson(manifest: &TraceManifest) -> anyhow::Result<Vec<u8>> {
    use anyhow::Context;
    let mut out = Vec::new();

    // Write a "header" manifest that carries metadata only (events=[] to avoid
    // duplication — events follow as individual lines).
    let header = TraceManifestHeader {
        version: manifest.version,
        test_id: manifest.test_id.clone(),
        spec_file: manifest.spec_file.clone(),
        test_title: manifest.test_title.clone(),
        outcome: manifest.outcome,
        started_at: manifest.started_at,
        finished_at: manifest.finished_at,
        assets: manifest.assets.clone(),
    };
    let header_line =
        serde_json::to_string(&header).context("Failed to serialise trace manifest header")?;
    out.extend_from_slice(header_line.as_bytes());
    out.push(b'\n');

    for event in &manifest.events {
        let event_line = serde_json::to_string(event).context("Failed to serialise trace event")?;
        out.extend_from_slice(event_line.as_bytes());
        out.push(b'\n');
    }

    Ok(out)
}

/// Deserialise a `TraceManifest` from NDJSON bytes produced by `encode_ndjson`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
pub fn decode_ndjson(bytes: &[u8]) -> anyhow::Result<TraceManifest> {
    use anyhow::Context;
    let text = std::str::from_utf8(bytes).context("Trace NDJSON is not valid UTF-8")?;
    let mut lines = text.lines();

    let header_line = lines.next().context("Trace NDJSON is empty")?;
    let header: TraceManifestHeader =
        serde_json::from_str(header_line).context("Failed to parse trace manifest header")?;

    let mut events = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let event: TraceEvent =
            serde_json::from_str(trimmed).context("Failed to parse trace event line")?;
        events.push(event);
    }

    Ok(TraceManifest {
        version: header.version,
        test_id: header.test_id,
        spec_file: header.spec_file,
        test_title: header.test_title,
        outcome: header.outcome,
        started_at: header.started_at,
        finished_at: header.finished_at,
        events,
        assets: header.assets,
    })
}

/// Slim header struct used for the first NDJSON line (no `events` field).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TraceManifestHeader {
    version: u32,
    test_id: String,
    spec_file: String,
    test_title: String,
    outcome: TraceOutcome,
    started_at: u64,
    finished_at: u64,
    #[serde(default)]
    assets: HashMap<String, String>,
}
// CODEGEN-END
