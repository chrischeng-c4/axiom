// SPEC-MANAGED: projects/cap/tech-design/semantic/cap-src.md#schema
// CODEGEN-BEGIN
//! Per-command run log.
//!
//! Every command that actually ran through cap (reached the `Spawned`
//! stage) gets one JSON line appended to
//! `~/.cap/logs/events-YYYY-MM-DD.jsonl` when it finishes. The daily
//! file is chosen at write time, so a long-lived daemon rolls over at
//! midnight without any explicit rotation step.
//!
//! Writing is best-effort: a failure to log never affects the command's
//! exit status (the log is observability, not control). Concurrent
//! appends from different `cap run` connections are safe — each record
//! is a single sub-4 KB `write_all` to an `O_APPEND` handle, which the
//! kernel places at EOF atomically.

use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;

use crate::paths;
use crate::protocol::KillClassification;

/// One completed command. Field names are the JSONL schema — keep them
/// stable; downstream tooling/agents parse this.
/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Debug, Clone, Serialize)]
pub struct RunRecord {
    /// RFC 3339 timestamp when the command finished (record write time).
    pub ts: String,
    /// RFC 3339 timestamp when the command was submitted to cap (the
    /// `Acquire`, i.e. before any queue wait).
    pub started_at: String,
    pub lease: u64,
    /// Full command line as shown in `cap status` (program + args).
    pub command: String,
    /// argv[0] basename (`cargo`, `pytest`, …).
    pub program: String,
    pub cwd: String,
    pub client_pid: i32,
    pub child_pid: Option<i32>,
    /// Time the command waited between submission and actually starting
    /// — i.e. how long cap's Acquire backpressure held it back.
    pub queue_ms: u64,
    /// Wall-clock run time from spawn to exit.
    pub duration_ms: u64,
    /// Total time the command spent SIGSTOPped by cap during its run.
    pub paused_ms: u64,
    /// Peak resident memory of the lease *leader* process observed
    /// across the run. NOTE: leader only, not the whole process group —
    /// for `cargo`/`pytest` the heavy children (rustc, workers) are not
    /// summed in. Treat as a lower bound on true peak usage.
    pub peak_rss_gb: f64,
    /// System free memory at the moment the command started running.
    pub free_gb_at_start: Option<f64>,
    /// Child exit code, or null if it was terminated by a signal
    /// (including a cap SIGKILL).
    pub exit_code: Option<i32>,
    /// `"completed"` if the command exited on its own, `"killed"` if cap
    /// evicted it under memory pressure.
    pub outcome: &'static str,
    /// Why cap killed it, when `outcome == "killed"`.
    pub kill_classification: Option<KillClassification>,
}

/// Appends [`RunRecord`]s to the daily JSONL file. Cheap to clone-share
/// across daemon connection tasks (just an enable flag + dir path).
/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Debug, Clone)]
pub struct EventLog {
    enabled: bool,
    dir: Option<PathBuf>,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
impl EventLog {
    pub fn new(enabled: bool) -> Self {
        // Resolve the dir once; if it can't be resolved we just disable
        // logging rather than erroring on every command.
        let dir = paths::logs_dir().ok();
        Self {
            enabled: enabled && dir.is_some(),
            dir,
        }
    }

    /// Best-effort append. Logs a warning on failure but never returns
    /// an error — run logging must not be able to fail a command.
    pub fn append(&self, rec: &RunRecord) {
        if !self.enabled {
            return;
        }
        if let Err(e) = self.try_append(rec) {
            tracing::warn!(error = %e, "failed to write run log");
        }
    }

    fn try_append(&self, rec: &RunRecord) -> std::io::Result<()> {
        let dir = self
            .dir
            .as_ref()
            .ok_or_else(|| std::io::Error::other("no logs dir"))?;
        std::fs::create_dir_all(dir)?;
        let date = chrono::Local::now().format("%Y-%m-%d");
        let path = dir.join(format!("events-{date}.jsonl"));
        let mut line = serde_json::to_string(rec).map_err(std::io::Error::other)?;
        line.push('\n');
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        f.write_all(line.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> RunRecord {
        RunRecord {
            ts: "2026-05-29T14:23:01.123+08:00".into(),
            started_at: "2026-05-29T14:22:58.001+08:00".into(),
            lease: 7,
            command: "cargo test -p cap".into(),
            program: "cargo".into(),
            cwd: "/tmp/proj".into(),
            client_pid: 111,
            child_pid: Some(222),
            queue_ms: 350,
            duration_ms: 3122,
            paused_ms: 0,
            peak_rss_gb: 1.83,
            free_gb_at_start: Some(5.9),
            exit_code: Some(0),
            outcome: "completed",
            kill_classification: None,
        }
    }

    #[test]
    fn record_serializes_to_one_json_line() {
        let s = serde_json::to_string(&sample()).unwrap();
        assert!(!s.contains('\n'), "a record must fit on one JSONL line");
        // Spot-check the schema the way a downstream parser would.
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["command"], "cargo test -p cap");
        assert_eq!(v["queue_ms"], 350);
        assert_eq!(v["outcome"], "completed");
        assert_eq!(v["kill_classification"], serde_json::Value::Null);
    }

    #[test]
    fn killed_record_emits_snake_case_classification() {
        let mut r = sample();
        r.outcome = "killed";
        r.exit_code = None;
        r.kill_classification = Some(KillClassification::Oversize);
        let v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&r).unwrap()).unwrap();
        assert_eq!(v["outcome"], "killed");
        assert_eq!(v["kill_classification"], "oversize");
        assert_eq!(v["exit_code"], serde_json::Value::Null);
    }

    #[test]
    fn disabled_log_appends_nothing() {
        // enabled=false short-circuits before touching the filesystem.
        let log = EventLog {
            enabled: false,
            dir: None,
        };
        log.append(&sample()); // must not panic / must be a no-op
    }

    #[test]
    fn writes_and_reads_back_a_line() {
        let tmp = tempfile::tempdir().unwrap();
        let log = EventLog {
            enabled: true,
            dir: Some(tmp.path().to_path_buf()),
        };
        log.append(&sample());
        log.append(&sample());
        let date = chrono::Local::now().format("%Y-%m-%d");
        let path = tmp.path().join(format!("events-{date}.jsonl"));
        let text = std::fs::read_to_string(path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2, "two appends → two lines");
        for l in lines {
            let _: serde_json::Value = serde_json::from_str(l).unwrap();
        }
    }
}
// CODEGEN-END
