// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
// CODEGEN-BEGIN
//! Persistent browser-session state.
//!
//! `jet browser launch` boots Chromium, attaches a CDP target, and
//! writes a session file. Every other `jet browser *` command reads
//! that file and reattaches to the same tab so the user sees exactly
//! the state the launch command left behind.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub ws_endpoint: String,
    pub target_id: String,
    pub url: String,
    pub pid: u32,
    /// Unix seconds when `launch` wrote the file.
    pub started_at: u64,
}

/// Session file path for a given project root. We intentionally bind
/// to the project dir rather than a global location so two jet
/// projects can debug in parallel without stepping on each other.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub fn session_path(root_dir: &Path) -> PathBuf {
    root_dir.join(".jet").join("browser-session.json")
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub fn shutdown_request_path(root_dir: &Path) -> PathBuf {
    root_dir.join(".jet").join("browser-shutdown-request")
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub fn write(root_dir: &Path, session: &Session) -> Result<()> {
    let path = session_path(root_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let body = serde_json::to_string_pretty(session).context("serializing browser session")?;
    std::fs::write(&path, body).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub fn request_shutdown(root_dir: &Path) -> Result<()> {
    let path = shutdown_request_path(root_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    std::fs::write(&path, now_unix().to_string())
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub fn shutdown_requested(root_dir: &Path) -> bool {
    shutdown_request_path(root_dir).exists()
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub fn clear_shutdown_request(root_dir: &Path) {
    let path = shutdown_request_path(root_dir);
    match std::fs::remove_file(&path) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            tracing::warn!(
                target: "jet::browser::session",
                path = %path.display(),
                error = %err,
                "failed to clear browser shutdown request file"
            );
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub fn read(root_dir: &Path) -> Result<Session> {
    let path = session_path(root_dir);
    let body = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "no browser session at {} — run `jet browser launch <url>` first",
            path.display()
        )
    })?;
    serde_json::from_str(&body).with_context(|| format!("parsing {}", path.display()))
}

/// GH #3252 — Remove the persisted session file. The prior
/// implementation collapsed every failure mode into `let _ = ...`,
/// so a permission-denied or EIO on the session file left a stale
/// session that the next `read()` happily deserialised — and the
/// caller silently reattached to a long-dead WebSocket endpoint.
///
/// Now: `NotFound` is silent (the file is already gone — that *is*
/// the success path), every other IO error emits a `tracing::warn!`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub fn clear(root_dir: &Path) {
    let path = session_path(root_dir);
    match std::fs::remove_file(&path) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            tracing::warn!(
                target: "jet::browser::session",
                path = %path.display(),
                error = %err,
                "GH #3252 failed to clear browser session file; \
                 next `jet browser` command may reattach to a stale or dead session"
            );
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub fn now_unix() -> u64 {
    // GH #3677 — was `.unwrap_or(0)` which silently collapsed any
    // clock-before-epoch failure onto zero. `now_unix()` populates
    // `Session.started_at` for the persisted browser-session file —
    // broken-clock hosts wrote sessions with `started_at = 0`, and
    // back-to-back sessions all collided on the same zero value with
    // no breadcrumb pointing at the host clock. Helper preserves the
    // historical zero on the broken-clock branch but returns a tagged
    // warn the caller emits via `tracing::warn!` against the
    // static-target macro. Sibling of #3669 (e2e::now_ms) and #3673
    // (trace::buffer).
    let (secs, warn) = safe_session_now_unix(SystemTime::now());
    if let Some(msg) = warn {
        tracing::warn!(target: "jet::browser::session", "{}", msg);
    }
    secs
}

/// GH #3677 — convert `SystemTime` to epoch-seconds with an observable
/// error branch. Happy path returns wall-clock seconds. Error branch
/// (clock before UNIX_EPOCH) returns `0` (preserving historical
/// behaviour so the on-disk Session.started_at schema doesn't shift)
/// plus a tagged warn message the caller is expected to emit via
/// `tracing::warn!` against its own static-target macro.
///
/// Sibling of `safe_e2e_now_ms` (#3669) and `safe_trace_now_ms` (#3673)
/// — same shape, but returns seconds via `as_secs()` rather than
/// milliseconds via `as_millis()` because `Session.started_at` is
/// declared as Unix-seconds in the on-disk schema.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub(crate) fn safe_session_now_unix(now: SystemTime) -> (u64, Option<String>) {
    match now.duration_since(UNIX_EPOCH) {
        Ok(dur) => (dur.as_secs(), None),
        Err(err) => {
            let warn = format_safe_session_now_unix_warn(&err);
            (0, Some(warn))
        }
    }
}

/// GH #3677 — build the warn wording for the clock-before-epoch branch.
/// Extracted so the issue tag, error visibility, and operator guidance
/// are unit-testable without provoking the actual broken-clock platform
/// case.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub(crate) fn format_safe_session_now_unix_warn(err: &std::time::SystemTimeError) -> String {
    format!(
        "GH #3677 jet::browser::session: SystemTime::now() reports a wall \
         clock before UNIX_EPOCH ({err}); falling back to secs=0. The \
         persisted browser-session file (`.jet/browser-session.json`) will \
         record `started_at = 0` (1970-01-01), and back-to-back `jet \
         browser launch` invocations will produce sessions with identical \
         started_at values — any staleness heuristic that depends on \
         started_at will treat them all as equally stale. Fix the host \
         clock (NTP / container --rtc / RTC battery) before trusting \
         session-file timestamps."
    )
}

/// Read a session. Staleness is detected at connect time — the
/// WebSocket handshake against `ws_endpoint` fails if Chromium is
/// gone, and the caller reports + clears the file then. Kept as its
/// own fn so future probes (port-check, PID probe) can land here.
/// @spec .aw/tech-design/projects/jet/semantic/jet-browser-cli.md#schema
pub fn read_live(root_dir: &Path) -> Result<Session> {
    read(root_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn roundtrips_session_file() {
        let dir = tempdir().unwrap();
        let s = Session {
            ws_endpoint: "ws://127.0.0.1:9222/devtools/browser/abc".into(),
            target_id: "ABC123".into(),
            url: "http://localhost:3131/".into(),
            pid: 42,
            started_at: now_unix(),
        };
        write(dir.path(), &s).unwrap();
        let back = read(dir.path()).unwrap();
        assert_eq!(back.target_id, s.target_id);
        assert_eq!(back.url, s.url);
        assert_eq!(back.pid, s.pid);
    }

    #[test]
    fn read_errors_cleanly_when_missing() {
        let dir = tempdir().unwrap();
        let err = read(dir.path()).unwrap_err().to_string();
        assert!(
            err.contains("no browser session"),
            "expected stale hint, got {err:?}"
        );
    }

    #[test]
    fn clear_removes_file() {
        let dir = tempdir().unwrap();
        let s = Session {
            ws_endpoint: "ws://127.0.0.1:9222".into(),
            target_id: "X".into(),
            url: "about:blank".into(),
            pid: 42,
            started_at: now_unix(),
        };
        write(dir.path(), &s).unwrap();
        clear(dir.path());
        assert!(read(dir.path()).is_err());
    }

    #[test]
    fn shutdown_request_roundtrips_and_clears() {
        let dir = tempdir().unwrap();
        assert!(!shutdown_requested(dir.path()));
        request_shutdown(dir.path()).unwrap();
        assert!(shutdown_requested(dir.path()));
        clear_shutdown_request(dir.path());
        assert!(!shutdown_requested(dir.path()));
    }

    /// GH #3252 — Already-absent file: `clear()` must be a silent
    /// no-op (NotFound is the success path) and must not panic.
    #[test]
    fn clear_when_session_file_absent_is_silent_noop() {
        let dir = tempdir().unwrap();
        // No file written.
        clear(dir.path());
        // Calling twice must also be a silent no-op (idempotency).
        clear(dir.path());
        assert!(read(dir.path()).is_err());
    }

    /// GH #3252 — Unix-only: when the *parent directory* is locked
    /// down so `remove_file` can't unlink, `clear()` must still not
    /// panic. The session file remains (the underlying perms bug is
    /// surfaced via tracing::warn, which we can't capture here), but
    /// callers that wrap `clear()` in higher-level cleanup paths
    /// continue running without aborting the session.
    #[cfg(unix)]
    #[test]
    fn clear_does_not_panic_when_remove_fails() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let s = Session {
            ws_endpoint: "ws://127.0.0.1:9222".into(),
            target_id: "X".into(),
            url: "about:blank".into(),
            pid: 42,
            started_at: now_unix(),
        };
        write(dir.path(), &s).unwrap();

        // Lock the parent of the session file: remove_file requires
        // write+execute on the parent dir. 0o500 = r-x for owner.
        let jet_dir = dir.path().join(".jet");
        let original = std::fs::metadata(&jet_dir).unwrap().permissions();
        std::fs::set_permissions(&jet_dir, std::fs::Permissions::from_mode(0o500)).unwrap();

        // Root may bypass the perms — skip cleanly in that case.
        let still_present = jet_dir.join("browser-session.json").exists();
        if !still_present {
            std::fs::set_permissions(&jet_dir, original).unwrap();
            return;
        }

        // Must not panic. The session file remains; the warn fired.
        clear(dir.path());

        // Restore perms so tempdir cleanup succeeds.
        std::fs::set_permissions(&jet_dir, original).unwrap();
    }
}

#[cfg(test)]
mod gh3677_safe_session_now_unix_tests {
    //! GH #3677 — `browser_cli::session::now_unix()` used to call
    //! `SystemTime::now().duration_since(UNIX_EPOCH).map(...).unwrap_or(0)`,
    //! silently collapsing any clock-before-epoch failure onto zero.
    //! Persisted browser-session files on broken-clock hosts recorded
    //! `started_at = 0` with no breadcrumb — multiple sessions then
    //! collided on the same zero value, defeating any staleness
    //! gating that uses started_at. Sibling of #3669 (e2e::now_ms)
    //! and #3673 (trace::buffer); same shape but returns seconds.
    use super::*;
    use std::time::Duration;

    #[test]
    fn happy_path_returns_seconds_and_no_warn() {
        let t = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let (secs, warn) = safe_session_now_unix(t);
        assert_eq!(secs, 1_700_000_000);
        assert!(warn.is_none(), "happy path must not warn");
    }

    #[test]
    fn epoch_itself_returns_zero_and_no_warn() {
        // UNIX_EPOCH must be Ok(0s), not the broken-clock branch.
        let (secs, warn) = safe_session_now_unix(UNIX_EPOCH);
        assert_eq!(secs, 0);
        assert!(warn.is_none());
    }

    #[test]
    fn happy_path_truncates_subsecond_to_floor_seconds() {
        // `as_secs()` truncates, not rounds. Pin this so a future
        // refactor that swaps to `as_secs_f64() as u64` doesn't
        // silently change the on-disk schema rounding.
        let t = UNIX_EPOCH + Duration::from_millis(1_999);
        let (secs, _) = safe_session_now_unix(t);
        assert_eq!(secs, 1, "must truncate to 1, not round to 2");
    }

    #[test]
    fn clock_before_epoch_returns_zero_and_warns() {
        let before = UNIX_EPOCH - Duration::from_secs(1);
        let (secs, warn) = safe_session_now_unix(before);
        assert_eq!(secs, 0, "broken-clock branch must preserve historical zero");
        let msg = warn.expect("broken-clock branch must emit a warn");
        assert!(
            msg.contains("GH #3677"),
            "warn must carry issue tag, got: {msg}"
        );
    }

    #[test]
    fn warn_message_names_session_file_and_started_at() {
        // The warn must point at the persisted session file and the
        // started_at field — otherwise the operator inspecting the
        // 1970-01-01 value in browser-session.json can't connect it
        // to clock skew.
        let before = UNIX_EPOCH - Duration::from_secs(1);
        let (_, warn) = safe_session_now_unix(before);
        let msg = warn.unwrap();
        assert!(
            msg.contains("started_at"),
            "warn must name Session.started_at field, got: {msg}"
        );
        assert!(
            msg.contains("browser-session") || msg.contains(".jet"),
            "warn must point at the persisted session file, got: {msg}"
        );
    }

    #[test]
    fn warn_message_points_at_the_host_clock_fix_not_jet_code() {
        let before = UNIX_EPOCH - Duration::from_secs(1);
        let (_, warn) = safe_session_now_unix(before);
        let msg = warn.unwrap();
        assert!(
            msg.contains("clock") || msg.contains("NTP") || msg.contains("RTC"),
            "warn must point at host clock as fix surface, got: {msg}"
        );
    }

    #[test]
    fn format_helper_round_trip_carries_observed_error_text() {
        let err = (UNIX_EPOCH - Duration::from_secs(11))
            .duration_since(UNIX_EPOCH)
            .unwrap_err();
        let msg = format_safe_session_now_unix_warn(&err);
        assert!(msg.contains("GH #3677"));
        assert!(
            msg.contains("11") || msg.contains("seconds") || msg.contains("UNIX_EPOCH"),
            "warn must forward error detail, got: {msg}"
        );
    }

    #[test]
    fn helper_output_is_deterministic_across_calls() {
        let before = UNIX_EPOCH - Duration::from_millis(789);
        let (_, w1) = safe_session_now_unix(before);
        let (_, w2) = safe_session_now_unix(before);
        assert_eq!(w1, w2);
    }
}
// CODEGEN-END
