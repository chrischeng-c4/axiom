// SPEC-MANAGED: projects/rig/tech-design/semantic/source/projects-rig-src-engine-exec-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! The `exec` escape-hatch step: run a command under a timeout, check its
//! exit code, optionally capture trimmed stdout.

use std::process::{Command, Stdio};
use std::time::Duration;

use super::timeout::{wait_with_timeout, TimeoutPolicy, WaitOutcome};
use crate::scenario::interp::VarStore;
use crate::scenario::step::ExecStep;

#[derive(Debug)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-engine-exec-rs.md#source
pub struct ExecOutcome {
    pub exit_code: i32,
    pub stdout: String,
    pub timed_out: bool,
    /// None = expectation met.
    pub violation: Option<String>,
}

/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-engine-exec-rs.md#source
pub fn execute(step: &ExecStep, vars: &VarStore) -> Result<ExecOutcome, String> {
    if step.cmd.is_empty() {
        return Err("exec step has an empty cmd".into());
    }
    let argv: Vec<String> = step
        .cmd
        .iter()
        .map(|a| vars.interpolate(a))
        .collect::<Result<_, _>>()?;

    let child = Command::new(&argv[0])
        .args(&argv[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not spawn `{}`: {e}", argv[0]))?;

    let policy = TimeoutPolicy::fixed(Duration::from_secs(step.timeout_secs));
    let outcome = wait_with_timeout(child, policy).map_err(|e| format!("wait failed: {e}"))?;

    let (output, timed_out) = match outcome {
        WaitOutcome::Finished(o) => (o, false),
        WaitOutcome::TimedOut(o) => (o, true),
    };
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let violation = if timed_out {
        Some(format!(
            "timed out after {}s: `{}`",
            step.timeout_secs,
            argv.join(" ")
        ))
    } else if exit_code != step.expect_exit_code {
        let stderr_tail: String = String::from_utf8_lossy(&output.stderr)
            .chars()
            .rev()
            .take(400)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        Some(format!(
            "exit {exit_code} != expected {} | stderr tail: {stderr_tail}",
            step.expect_exit_code
        ))
    } else {
        None
    };

    Ok(ExecOutcome {
        exit_code,
        stdout,
        timed_out,
        violation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn step(cmd: &[&str]) -> ExecStep {
        ExecStep {
            name: "t".into(),
            cmd: cmd.iter().map(|s| s.to_string()).collect(),
            timeout_secs: 5,
            expect_exit_code: 0,
            capture: BTreeMap::new(),
        }
    }

    #[test]
    fn captures_stdout_and_exit() {
        let o = execute(&step(&["echo", "hello"]), &VarStore::new()).unwrap();
        assert_eq!(o.exit_code, 0);
        assert_eq!(o.stdout, "hello");
        assert!(o.violation.is_none());
    }

    #[test]
    fn wrong_exit_code_is_violation() {
        let o = execute(&step(&["false"]), &VarStore::new()).unwrap();
        assert!(o.violation.is_some());
    }

    #[test]
    fn interpolates_argv() {
        let mut vars = VarStore::new();
        vars.set("word", serde_json::json!("rigged"));
        let o = execute(&step(&["echo", "{{word}}"]), &vars).unwrap();
        assert_eq!(o.stdout, "rigged");
    }
}
// CODEGEN-END
