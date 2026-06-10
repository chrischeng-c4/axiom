//! vat integration: wrap a run in `vat run <runner>` and lift the inner
//! rig report back out.
//!
//! rig stays on the CLI seam (no vat crate dependency): it spawns the vat
//! binary, parses the JSONL checkpoints vat prints to stdout
//! (`{"type": "select|prepare|ready|runner|result|error", ...}`), and
//! retrieves the inner rig report from the runner's captured stdout via
//! `vat logs <id> runner`. The vat binary is overridable through
//! `RIG_VAT_BIN` (used by tests to fake the protocol).

use std::process::Command;

use serde_json::Value;

use crate::report::RigReport;

fn vat_bin() -> String {
    std::env::var("RIG_VAT_BIN").unwrap_or_else(|_| "vat".to_string())
}

/// The outcome of one `vat run <runner>` invocation.
#[derive(Debug)]
pub struct VatRun {
    pub vat_id: String,
    pub runner: String,
    pub ok: bool,
    pub exit_code: i32,
    /// Service readiness lines, for the report's environment evidence.
    pub ready_services: Vec<String>,
}

/// Spawn `vat run <runner>` and fold its checkpoint stream.
pub fn run_runner(runner: &str) -> Result<VatRun, String> {
    let output = Command::new(vat_bin())
        .args(["run", runner])
        .output()
        .map_err(|e| format!("could not spawn `{}` (install vat or set RIG_VAT_BIN): {e}", vat_bin()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut result: Option<VatRun> = None;
    let mut ready = Vec::new();
    for line in stdout.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line.trim()) else {
            continue; // non-checkpoint noise
        };
        match v.get("type").and_then(|t| t.as_str()) {
            Some("ready") => {
                if let Some(s) = v.get("service").and_then(|s| s.as_str()) {
                    ready.push(s.to_string());
                }
            }
            Some("result") => {
                result = Some(VatRun {
                    vat_id: v.get("id").and_then(|s| s.as_str()).unwrap_or("").to_string(),
                    runner: runner.to_string(),
                    ok: v.get("ok").and_then(|b| b.as_bool()).unwrap_or(false),
                    exit_code: v.get("exit_code").and_then(|c| c.as_i64()).unwrap_or(-1) as i32,
                    ready_services: Vec::new(),
                });
            }
            Some("error") => {
                let msg = v
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown vat error");
                return Err(format!("vat error for runner `{runner}`: {msg}"));
            }
            _ => {}
        }
    }
    let mut run = result.ok_or_else(|| {
        format!(
            "vat produced no result checkpoint for runner `{runner}` (exit {}); stderr: {}",
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stderr).trim()
        )
    })?;
    run.ready_services = ready;
    Ok(run)
}

/// Fetch the inner runner's captured stdout (`vat logs <id> runner`).
pub fn runner_log(vat_id: &str) -> Result<String, String> {
    let output = Command::new(vat_bin())
        .args(["logs", vat_id, "runner"])
        .output()
        .map_err(|e| format!("could not spawn `{} logs`: {e}", vat_bin()))?;
    if !output.status.success() {
        return Err(format!(
            "`vat logs {vat_id} runner` failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Best-effort removal once the report is lifted (the vat.toml uses
/// `keep = "always"` so the log survives success; rig owns the cleanup).
pub fn remove(vat_id: &str) {
    let _ = Command::new(vat_bin()).args(["rm", vat_id]).output();
}

/// Extract the inner rig report from a runner log: the LAST line (or
/// pretty-printed block) that parses as a `RigReport`.
pub fn extract_report(log: &str) -> Option<RigReport> {
    // Fast path: whole log is one pretty JSON document.
    if let Ok(report) = serde_json::from_str::<RigReport>(log.trim()) {
        return Some(report);
    }
    // Line-oriented: last parseable line wins (diagnostics may interleave).
    log.lines()
        .rev()
        .find_map(|line| serde_json::from_str::<RigReport>(line.trim()).ok())
        .or_else(|| {
            // Pretty-printed report embedded after other output: scan for the
            // first `{` of each trailing block.
            let bytes = log.as_bytes();
            (0..bytes.len())
                .filter(|&i| bytes[i] == b'{')
                .find_map(|i| serde_json::from_str::<RigReport>(log[i..].trim()).ok())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ReportBuilder;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::Mutex;

    /// RIG_VAT_BIN is process-global; serialize the tests that set it.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Write a fake `vat` script that prints the given stdout for any args.
    fn fake_vat(dir: &std::path::Path, run_stdout: &str, logs_stdout: &str) -> std::path::PathBuf {
        let path = dir.join("vat");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(
            f,
            "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then cat <<'RIGEOF'\n{run_stdout}\nRIGEOF\nelse cat <<'RIGEOF'\n{logs_stdout}\nRIGEOF\nfi"
        )
        .unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[test]
    fn parses_checkpoints_and_lifts_inner_report() {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let inner = ReportBuilder::new("run", "inner").finalize();
        let inner_json = serde_json::to_string(&inner).unwrap();
        let run_out = r#"{"type":"prepare","service":"lumen","mode":"direct_start"}
{"type":"ready","service":"lumen","ms":1200}
{"type":"runner","id":"rig-resilience","state":"started"}
{"type":"result","id":"vat-abc123","runner":"rig-resilience","ok":true,"exit_code":0,"state":"removed"}"#;
        let vat = fake_vat(tmp.path(), run_out, &inner_json);
        std::env::set_var("RIG_VAT_BIN", &vat);

        let run = run_runner("rig-resilience").unwrap();
        assert_eq!(run.vat_id, "vat-abc123");
        assert!(run.ok);
        assert_eq!(run.ready_services, vec!["lumen"]);

        let log = runner_log(&run.vat_id).unwrap();
        let report = extract_report(&log).expect("inner report lifted");
        assert_eq!(report.verb, "run");
        assert_eq!(report.target, "inner");
        std::env::remove_var("RIG_VAT_BIN");
    }

    #[test]
    fn error_checkpoint_is_an_err() {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let run_out = r#"{"type":"error","code":"runner_required","message":"no such runner"}"#;
        let vat = fake_vat(tmp.path(), run_out, "");
        std::env::set_var("RIG_VAT_BIN", &vat);
        let e = run_runner("missing").unwrap_err();
        assert!(e.contains("no such runner"));
        std::env::remove_var("RIG_VAT_BIN");
    }

    #[test]
    fn extract_report_finds_pretty_block_after_noise() {
        let inner = ReportBuilder::new("run", "x").finalize();
        let pretty = serde_json::to_string_pretty(&inner).unwrap();
        let log = format!("some diagnostic line\nanother\n{pretty}\n");
        assert!(extract_report(&log).is_some());
    }
}
