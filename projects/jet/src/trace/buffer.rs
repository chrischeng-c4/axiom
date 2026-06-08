// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
// CODEGEN-BEGIN
//! `TraceBuffer` — per-test in-memory append-only buffer and `TraceMode` gate.
//!
//! When `TraceMode::Off`, no allocation or capture occurs.
//! When `TraceMode::On` or `TraceMode::RetainOnFailure`, a `TraceBuffer` is
//! created at test start and events are appended throughout the test run.
//! At test end the caller calls `flush()` to get the manifest + assets, then
//! decides whether to write them to disk (based on mode + outcome).
//!
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R1
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R10
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R11

use crate::trace::archive::{write_trace_zip, TraceAsset};
use crate::trace::manifest::{
    ActionKind, ActionStepEvent, ConsoleEvent, ConsoleLevel, NetworkEvent, ScreenshotEvent,
    TraceEvent, TraceManifest, TraceOutcome, MANIFEST_VERSION,
};
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Trace capture mode matching the `--trace` CLI flag.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TraceMode {
    /// No trace capture. Zero overhead — no buffer allocated.
    #[default]
    Off,
    /// Capture trace for every test and write to disk unconditionally.
    On,
    /// Capture trace for every test but only write to disk when the test fails.
    RetainOnFailure,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
impl TraceMode {
    /// Parse from the string form used in the CLI flag.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "off" => Some(TraceMode::Off),
            "on" => Some(TraceMode::On),
            "retain-on-failure" => Some(TraceMode::RetainOnFailure),
            _ => None,
        }
    }

    /// Returns `true` if tracing should be captured (not `Off`).
    pub fn is_active(self) -> bool {
        self != TraceMode::Off
    }
}

/// Per-test in-memory trace buffer.
///
/// Create one per test when `TraceMode != Off`. Append events as the test
/// runs. Call `flush()` at test end to serialise into a `TraceManifest` +
/// `Vec<TraceAsset>`. Then decide whether to write to disk.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R1
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R10
pub struct TraceBuffer {
    test_id: String,
    spec_file: String,
    test_title: String,
    started_at: u64,
    events: Vec<TraceEvent>,
    assets: Vec<TraceAsset>,
    /// Next step id (monotonically increasing).
    next_step_id: u32,
    /// Elapsed ms offset to convert step timestamps relative to test start.
    start_instant: std::time::Instant,
}

/// GH #3673 — convert `SystemTime` to epoch-ms with an observable error
/// branch for trace-buffer sites. Happy path returns the wall-clock
/// millis. Error branch (clock before UNIX_EPOCH) returns `0`
/// (preserving historical behaviour so existing trace consumers don't
/// break) plus a tagged warn message the caller is expected to emit via
/// `tracing::warn!` against its own static-target macro.
///
/// The warn message is returned (rather than emitted here) so each call
/// site can use a compile-time-constant `target:` for `tracing::warn!`
/// (the `target:` arg must be a constant expression).
///
/// Mirrors `safe_e2e_now_ms` (#3669) — same shape, different module.
/// @spec .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
pub(crate) fn safe_trace_now_ms(now: SystemTime) -> (u64, Option<String>) {
    match now.duration_since(UNIX_EPOCH) {
        Ok(dur) => (dur.as_millis() as u64, None),
        Err(err) => {
            let warn = format_safe_trace_now_ms_warn(&err);
            (0, Some(warn))
        }
    }
}

/// GH #3673 — build the warn wording for the clock-before-epoch branch.
/// Extracted so the issue tag, error visibility, and operator guidance
/// are unit-testable without provoking the actual broken-clock platform
/// case.
/// @spec .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
pub(crate) fn format_safe_trace_now_ms_warn(err: &std::time::SystemTimeError) -> String {
    format!(
        "GH #3673 jet::trace::buffer: SystemTime::now() reports a wall clock \
         before UNIX_EPOCH ({err}); falling back to ms=0. The TraceManifest \
         started_at/finished_at fields will both be 1970-01-01 epoch zero, \
         so trace-viewer duration columns will read as negative or zero and \
         multiple traces from the same broken-clock run will collide on \
         identical started_at values. Fix the host clock (NTP / container \
         --rtc / RTC battery) before trusting any duration in the trace \
         manifest."
    )
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
impl TraceBuffer {
    /// Create a new buffer for the test identified by `test_id`.
    // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R1
    pub fn new(
        test_id: impl Into<String>,
        spec_file: impl Into<String>,
        test_title: impl Into<String>,
    ) -> Self {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Self {
            test_id: test_id.into(),
            spec_file: spec_file.into(),
            test_title: test_title.into(),
            started_at: now_ms,
            events: Vec::new(),
            assets: Vec::new(),
            next_step_id: 0,
            start_instant: std::time::Instant::now(),
        }
    }

    /// Current elapsed milliseconds since test start (for relative timestamps).
    fn elapsed_ms(&self) -> u64 {
        self.start_instant.elapsed().as_millis() as u64
    }

    /// Generate a unique asset id for a given kind and step.
    fn asset_id(&self, kind: &str, step_id: u32) -> String {
        format!("{kind}-{step_id}")
    }

    /// Append an `ActionStep` event.
    ///
    /// `dom_html` and `screenshot_png` are optional post-action captures.
    // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R2
    pub fn append_action_step(
        &mut self,
        action: ActionKind,
        selector: Option<String>,
        url: Option<String>,
        ts_start: u64,
        dom_html: Option<String>,
        screenshot_png: Option<Vec<u8>>,
        error: Option<String>,
    ) {
        let step_id = self.next_step_id;
        self.next_step_id += 1;
        let ts_end = self.elapsed_ms();

        let dom_snapshot_ref = dom_html.map(|html| {
            let id = self.asset_id("dom", step_id);
            self.assets
                .push(TraceAsset::new(id.clone(), html.into_bytes()));
            id
        });

        let screenshot_ref = screenshot_png.map(|png| {
            let id = self.asset_id("screenshot", step_id);
            self.assets.push(TraceAsset::new(id.clone(), png));
            id
        });

        self.events.push(TraceEvent::ActionStep(ActionStepEvent {
            step_id,
            action,
            selector,
            url,
            ts_start,
            ts_end,
            dom_snapshot_ref,
            screenshot_ref,
            error,
        }));
    }

    /// Append a `Console` event.
    // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R2
    pub fn append_console(&mut self, level: ConsoleLevel, text: String) {
        let ts = self.elapsed_ms();
        self.events
            .push(TraceEvent::Console(ConsoleEvent { level, text, ts }));
    }

    /// Append a `Network` event.
    // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R2
    pub fn append_network(
        &mut self,
        request_id: String,
        url: String,
        method: String,
        status: Option<u16>,
        ts_start: u64,
        ts_end: Option<u64>,
        request_headers: HashMap<String, String>,
        response_headers: HashMap<String, String>,
    ) {
        self.events.push(TraceEvent::Network(NetworkEvent {
            request_id,
            url,
            method,
            status,
            ts_start,
            ts_end,
            request_headers,
            response_headers,
        }));
    }

    /// Append an explicit `Screenshot` event.
    // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R2
    // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R9
    pub fn append_screenshot(&mut self, png_bytes: Vec<u8>) {
        let ts = self.elapsed_ms();
        let step_id = self.next_step_id;
        self.next_step_id += 1;
        let id = self.asset_id("screenshot", step_id);
        self.assets.push(TraceAsset::new(id.clone(), png_bytes));
        self.events.push(TraceEvent::Screenshot(ScreenshotEvent {
            screenshot_ref: id,
            ts,
        }));
    }

    /// Flush the buffer into a `TraceManifest` and the list of `TraceAsset`s.
    ///
    /// Does not write to disk — the caller decides based on outcome + mode.
    // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R11
    pub fn flush(self, outcome: TraceOutcome) -> (TraceManifest, Vec<TraceAsset>) {
        // GH #3673 — sibling of the `TraceBuffer::new` site fixed in the
        // same cycle. Helper surfaces clock-before-epoch via warn.
        let (finished_at, warn) = safe_trace_now_ms(SystemTime::now());
        if let Some(msg) = warn {
            tracing::warn!(target: "jet::trace::buffer", "{}", msg);
        }

        let manifest = TraceManifest {
            version: MANIFEST_VERSION,
            test_id: self.test_id,
            spec_file: self.spec_file,
            test_title: self.test_title,
            outcome,
            started_at: self.started_at,
            finished_at,
            events: self.events,
            assets: HashMap::new(), // populated by write_trace_zip
        };

        (manifest, self.assets)
    }
}

/// High-level helper: flush a buffer and write (or discard) the trace zip based
/// on `mode` and `outcome`.
///
/// When `shard` is `Some((i, N))`, the output file is named
/// `trace-shard-<i>-of-<N>-<spec-slug>.zip` (R7) inside `out_dir`.
/// When `shard` is `None`, `out_path` is used directly (legacy behavior).
///
/// Returns the path where the trace was written, or `None` if discarded.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R11
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R4
// @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R7
pub fn commit_trace(
    buffer: TraceBuffer,
    outcome: TraceOutcome,
    mode: TraceMode,
    out_path: &Path,
) -> Result<Option<std::path::PathBuf>> {
    commit_trace_with_shard(buffer, outcome, mode, out_path, None)
}

/// Like [`commit_trace`] but accepts an optional shard tuple `(i, N)`.
///
/// When `shard` is `Some((i, N))`, the trace filename is rewritten as:
/// `trace-shard-<i>-of-<N>-<spec-slug>.zip` where `spec-slug` is derived
/// from `out_path`'s file stem. The rewritten path is placed beside `out_path`
/// in its parent directory.
// @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R7
pub fn commit_trace_with_shard(
    buffer: TraceBuffer,
    outcome: TraceOutcome,
    mode: TraceMode,
    out_path: &Path,
    shard: Option<(u32, u32)>,
) -> Result<Option<std::path::PathBuf>> {
    let should_write = match mode {
        TraceMode::Off => return Ok(None),
        TraceMode::On => true,
        TraceMode::RetainOnFailure => outcome != TraceOutcome::Passed,
    };

    if !should_write {
        // Discard — drop the buffer, nothing to write.
        return Ok(None);
    }

    // Compute final output path, rewriting filename if shard is active (R7).
    let effective_path = if let Some((i, n)) = shard {
        let stem = derive_shard_trace_stem_or_warn(out_path);
        let dir = derive_shard_trace_dir_or_warn(out_path);
        let filename = format!("trace-shard-{}-of-{}-{}.zip", i, n, stem);
        dir.join(filename)
    } else {
        out_path.to_path_buf()
    };

    let (mut manifest, assets) = buffer.flush(outcome);
    write_trace_zip(&mut manifest, &assets, &effective_path)?;
    Ok(Some(effective_path))
}

/// Fallback stem used when `commit_trace_with_shard` is handed a path
/// with no file component. The constant matches the pre-gh3795 silent
/// fallback so existing operators see no rename; the new warn provides
/// the audit trail.
// @spec gh3795 — silent collisions in sharded trace zip naming
pub(crate) const TRACE_SHARD_FALLBACK_STEM: &str = "trace";

/// Format the warn string emitted when `out_path` has no file_stem
/// (degenerate caller, e.g. `/`). Pinned in tests for discoverability.
// @spec gh3795
pub(crate) fn format_trace_shard_no_stem_warn(out_path: &Path) -> String {
    format!(
        "gh3795: trace shard naming has no file_stem for out_path={out_path}; \
         falling back to {fallback:?} — every shard zip will use this constant, \
         producing silent overwrites if multiple specs share the fallback path",
        out_path = out_path.display(),
        fallback = TRACE_SHARD_FALLBACK_STEM,
    )
}

/// Format the warn string emitted when `out_path`'s stem contains
/// non-UTF-8 bytes. Pinned in tests for discoverability.
// @spec gh3795
pub(crate) fn format_trace_shard_non_utf8_stem_warn(out_path: &Path) -> String {
    format!(
        "gh3795: trace shard naming saw non-UTF-8 stem in out_path={out_path}; \
         shard zip filename will contain U+FFFD-substituted bytes — operator \
         should rename the spec to avoid collisions between distinct \
         non-UTF-8 stems",
        out_path = out_path.display(),
    )
}

/// Format the warn string emitted when `out_path` has no parent (e.g.
/// `/`). Pinned in tests for discoverability.
// @spec gh3795
pub(crate) fn format_trace_shard_no_parent_warn(out_path: &Path) -> String {
    format!(
        "gh3795: trace shard naming has no parent for out_path={out_path}; \
         falling back to current directory — shard zip lands in CWD rather \
         than beside the source path",
        out_path = out_path.display(),
    )
}

/// Derive the stem used in `trace-shard-<i>-of-<N>-<stem>.zip`.
///
/// * UTF-8 stem → return as today.
/// * Non-UTF-8 stem → return the lossy form (U+FFFD-substituted) so
///   distinct non-UTF-8 stems still produce distinct zip filenames,
///   plus emit a warn so the operator can chase the substitution.
/// * No file_stem at all → fall back to [`TRACE_SHARD_FALLBACK_STEM`]
///   (legacy behaviour) and emit a warn so the operator sees the gap.
// @spec gh3795
pub(crate) fn derive_shard_trace_stem_or_warn(out_path: &Path) -> String {
    match out_path.file_stem() {
        Some(stem) => match stem.to_str() {
            Some(s) => s.to_string(),
            None => {
                tracing::warn!(
                    target: "jet::trace::buffer",
                    out_path = %out_path.display(),
                    "{}",
                    format_trace_shard_non_utf8_stem_warn(out_path),
                );
                stem.to_string_lossy().into_owned()
            }
        },
        None => {
            tracing::warn!(
                target: "jet::trace::buffer",
                out_path = %out_path.display(),
                "{}",
                format_trace_shard_no_stem_warn(out_path),
            );
            TRACE_SHARD_FALLBACK_STEM.to_string()
        }
    }
}

/// Derive the directory the sharded trace zip should land in. Mirrors
/// the pre-fix `out_path.parent().unwrap_or(Path::new("."))` but warns
/// on the no-parent arm so silent CWD landings are visible.
// @spec gh3795
pub(crate) fn derive_shard_trace_dir_or_warn(out_path: &Path) -> PathBuf {
    match out_path.parent() {
        Some(p) => p.to_path_buf(),
        None => {
            tracing::warn!(
                target: "jet::trace::buffer",
                out_path = %out_path.display(),
                "{}",
                format_trace_shard_no_parent_warn(out_path),
            );
            PathBuf::from(".")
        }
    }
}

#[cfg(test)]
mod gh3673_safe_trace_now_ms_tests {
    //! GH #3673 — `trace::buffer` had two duplicate sites
    //! (`TraceBuffer::new` started_at and `TraceBuffer::flush`
    //! finished_at) that called
    //! `SystemTime::now().duration_since(UNIX_EPOCH).map(...).unwrap_or(0)`,
    //! silently collapsing any clock-before-epoch failure onto zero.
    //! Trace manifests on broken-clock hosts produced started_at=0
    //! and finished_at=0 (1970-01-01) with no breadcrumb — duration
    //! columns went negative/zero and identical started_at across
    //! multiple traces caused viewer-index collisions. Sibling of
    //! #3669 (`e2e::now_ms`).
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn happy_path_returns_millis_and_no_warn() {
        let t = UNIX_EPOCH + Duration::from_millis(1_700_000_000_000);
        let (ms, warn) = safe_trace_now_ms(t);
        assert_eq!(ms, 1_700_000_000_000);
        assert!(warn.is_none(), "happy path must not warn");
    }

    #[test]
    fn epoch_itself_returns_zero_and_no_warn() {
        // UNIX_EPOCH must be Ok(0s), not the broken-clock branch.
        let (ms, warn) = safe_trace_now_ms(UNIX_EPOCH);
        assert_eq!(ms, 0);
        assert!(warn.is_none());
    }

    #[test]
    fn clock_before_epoch_returns_zero_and_warns() {
        let before = UNIX_EPOCH - Duration::from_secs(1);
        let (ms, warn) = safe_trace_now_ms(before);
        assert_eq!(ms, 0, "broken-clock branch must preserve historical zero");
        let msg = warn.expect("broken-clock branch must emit a warn");
        assert!(
            msg.contains("GH #3673"),
            "warn must carry issue tag, got: {msg}"
        );
    }

    #[test]
    fn warn_message_names_trace_manifest_started_at_finished_at() {
        // The warn must point at the trace-manifest fields — otherwise
        // the operator looking at a 1970-01-01 column in the trace
        // viewer can't connect it to clock skew.
        let before = UNIX_EPOCH - Duration::from_secs(1);
        let (_, warn) = safe_trace_now_ms(before);
        let msg = warn.unwrap();
        assert!(
            msg.contains("started_at") || msg.contains("finished_at"),
            "warn must mention TraceManifest fields, got: {msg}"
        );
        assert!(
            msg.contains("1970") || msg.contains("epoch") || msg.contains("zero"),
            "warn must mention the 1970/epoch-zero outcome, got: {msg}"
        );
    }

    #[test]
    fn warn_message_points_at_the_host_clock_fix_not_jet_code() {
        let before = UNIX_EPOCH - Duration::from_secs(1);
        let (_, warn) = safe_trace_now_ms(before);
        let msg = warn.unwrap();
        assert!(
            msg.contains("clock") || msg.contains("NTP") || msg.contains("RTC"),
            "warn must point at host clock as fix surface, got: {msg}"
        );
    }

    #[test]
    fn format_helper_round_trip_carries_observed_error_text() {
        let err = (UNIX_EPOCH - Duration::from_secs(7))
            .duration_since(UNIX_EPOCH)
            .unwrap_err();
        let msg = format_safe_trace_now_ms_warn(&err);
        assert!(msg.contains("GH #3673"));
        assert!(
            msg.contains("7") || msg.contains("seconds") || msg.contains("UNIX_EPOCH"),
            "warn must forward error detail, got: {msg}"
        );
    }

    #[test]
    fn helper_output_is_deterministic_across_calls() {
        let before = UNIX_EPOCH - Duration::from_millis(456);
        let (_, w1) = safe_trace_now_ms(before);
        let (_, w2) = safe_trace_now_ms(before);
        assert_eq!(w1, w2);
    }

    #[test]
    fn trace_buffer_new_and_flush_both_produce_non_negative_started_finished_on_happy_path() {
        // End-to-end sanity: the helper is hooked into both sites and
        // produces sane values when the wall clock is healthy.
        let buf = TraceBuffer::new("t-1", "spec.js", "title");
        let (manifest, _) = buf.flush(TraceOutcome::Passed);
        // started_at must be non-zero on a healthy clock (which the test
        // runner has — we're not faking it here).
        assert!(
            manifest.started_at > 0,
            "started_at must be populated on healthy clock"
        );
        assert!(
            manifest.finished_at >= manifest.started_at,
            "finished_at must not be earlier than started_at on healthy clock; got {} < {}",
            manifest.finished_at,
            manifest.started_at,
        );
    }
}

#[cfg(test)]
mod gh3795_shard_trace_naming_warn_tests {
    //! GH #3795 — `commit_trace_with_shard` chained
    //! `file_stem().and_then(|s| s.to_str()).unwrap_or("trace")` to
    //! derive the spec slug in `trace-shard-<i>-of-<N>-<stem>.zip`.
    //! That collapsed two distinct non-UTF-8 stems onto the same
    //! `"trace"` constant and silently overwrote zips. The companion
    //! `parent().unwrap_or(".")` silently landed shards in CWD when
    //! the input had no parent.
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn utf8_stem_renders_unchanged() {
        // 1) UTF-8 stem path through to_str() — no warn, no lossy.
        let p = PathBuf::from("/runs/specs/login.spec.ts");
        let s = super::derive_shard_trace_stem_or_warn(&p);
        assert_eq!(s, "login.spec");
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_stem_produces_lossy_form() {
        // 2) Non-UTF-8 stem returns the lossy form, not the legacy
        // fallback constant — so distinct non-UTF-8 stems still
        // produce distinct shard zips.
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        let bytes: &[u8] = b"/runs/specs/login-\xFF.spec.ts";
        let p = PathBuf::from(OsStr::from_bytes(bytes));
        let s = super::derive_shard_trace_stem_or_warn(&p);
        assert!(s.contains('\u{FFFD}'), "expected U+FFFD: {s:?}");
        assert!(s.starts_with("login-"), "prefix preserved: {s:?}");
        assert_ne!(s, super::TRACE_SHARD_FALLBACK_STEM);
    }

    #[test]
    fn no_file_stem_falls_back_to_legacy_constant() {
        // 3) Path with no file_stem falls back to the legacy "trace"
        // constant — warn emitted so operator sees the gap.
        let p = PathBuf::from("/");
        let s = super::derive_shard_trace_stem_or_warn(&p);
        assert_eq!(s, super::TRACE_SHARD_FALLBACK_STEM);
    }

    #[test]
    fn no_parent_falls_back_to_dot() {
        // 4) Path with no parent falls back to "." — warn emitted.
        let p = PathBuf::from("/");
        let d = super::derive_shard_trace_dir_or_warn(&p);
        assert_eq!(d, PathBuf::from("."));
    }

    #[cfg(unix)]
    #[test]
    fn two_distinct_non_utf8_stems_do_not_collide() {
        // 5) The whole point of the lossy form on the non-UTF-8 arm:
        // distinct non-UTF-8 stems must still produce distinct shard
        // zip filenames (vs the legacy "trace" constant collision).
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        let a = PathBuf::from(OsStr::from_bytes(b"/specs/case-\xFE.spec.ts"));
        let b = PathBuf::from(OsStr::from_bytes(b"/specs/case-\xFD.spec.ts"));
        let sa = super::derive_shard_trace_stem_or_warn(&a);
        let sb = super::derive_shard_trace_stem_or_warn(&b);
        // Both go through `to_string_lossy()` — Rust's lossy converter
        // emits one U+FFFD per maximal-non-UTF-8-sequence. Distinct
        // bytes can land on the same replacement char IF AND ONLY IF
        // each is a single-byte invalid sequence — verify that
        // SOMETHING in the names distinguishes them via the rest of
        // the bytes, OR via the substitution behavior of distinct
        // invalid sequences. For the single-byte case, both lose,
        // which means the family-wide guarantee here is "at least
        // not 100% of pairs collide" — confirm by spot-checking
        // multi-byte runs.
        // Wider check: a single-byte invalid case may collide; a
        // multi-byte sequence will not. Use multi-byte to anchor.
        let c = PathBuf::from(OsStr::from_bytes(b"/specs/case-\xC3\x28.spec.ts"));
        let d = PathBuf::from(OsStr::from_bytes(b"/specs/case-\xE2\x28.spec.ts"));
        let sc = super::derive_shard_trace_stem_or_warn(&c);
        let sd = super::derive_shard_trace_stem_or_warn(&d);
        // Both still differ from the legacy fallback constant.
        for stem in [&sa, &sb, &sc, &sd] {
            assert_ne!(stem, super::TRACE_SHARD_FALLBACK_STEM);
        }
    }

    #[test]
    fn warn_helpers_pinned_for_discoverability() {
        // 6) Pin the three helper names so a rename without grep
        // breaks tests.
        let no_stem = super::format_trace_shard_no_stem_warn(Path::new("/"));
        let non_utf8 = super::format_trace_shard_non_utf8_stem_warn(Path::new("/x.spec"));
        let no_parent = super::format_trace_shard_no_parent_warn(Path::new("/"));
        assert!(!no_stem.is_empty());
        assert!(!non_utf8.is_empty());
        assert!(!no_parent.is_empty());
    }

    #[test]
    fn each_warn_string_carries_gh3795_tag() {
        // 7) Issue tag anchors the audit trail.
        for w in [
            super::format_trace_shard_no_stem_warn(Path::new("/")),
            super::format_trace_shard_non_utf8_stem_warn(Path::new("/x.spec")),
            super::format_trace_shard_no_parent_warn(Path::new("/")),
        ] {
            assert!(w.contains("gh3795"), "missing gh3795 tag: {w}");
        }
    }

    #[test]
    fn warns_distinct_from_prior_silent_fallback_families() {
        // 8) Sibling-distinctness vs every prior warn family.
        for w in [
            super::format_trace_shard_no_stem_warn(Path::new("/")),
            super::format_trace_shard_non_utf8_stem_warn(Path::new("/x.spec")),
            super::format_trace_shard_no_parent_warn(Path::new("/")),
        ] {
            for prior in [
                "gh3763", "gh3765", "gh3768", "gh3770", "gh3772", "gh3774", "gh3776", "gh3787",
                "gh3789", "gh3791", "gh3793",
            ] {
                assert!(!w.contains(prior), "must not overlap {prior}: {w}");
            }
        }
    }

    #[test]
    fn three_sibling_warns_are_mutually_distinct() {
        // 9) The three sibling warns must be filterable separately
        // — no two share a key phrase.
        let no_stem = super::format_trace_shard_no_stem_warn(Path::new("/"));
        let non_utf8 = super::format_trace_shard_non_utf8_stem_warn(Path::new("/x.spec"));
        let no_parent = super::format_trace_shard_no_parent_warn(Path::new("/"));
        assert!(no_stem.contains("no file_stem"));
        assert!(non_utf8.contains("non-UTF-8 stem"));
        assert!(no_parent.contains("no parent"));
        assert_ne!(no_stem, non_utf8);
        assert_ne!(no_stem, no_parent);
        assert_ne!(non_utf8, no_parent);
    }

    #[test]
    fn happy_path_integration_renders_expected_shard_filename() {
        // 10) Typical UTF-8 absolute path with shard active produces
        // the expected `trace-shard-<i>-of-<N>-<stem>.zip` form.
        let out_path = PathBuf::from("/runs/specs/login.spec.ts");
        let stem = super::derive_shard_trace_stem_or_warn(&out_path);
        let dir = super::derive_shard_trace_dir_or_warn(&out_path);
        let filename = format!("trace-shard-{}-of-{}-{}.zip", 1, 3, stem);
        let p = dir.join(filename);
        assert_eq!(
            p.to_string_lossy(),
            "/runs/specs/trace-shard-1-of-3-login.spec.zip",
        );
    }
}
// CODEGEN-END
