// SPEC-MANAGED: libs/cli-std/tech-design/semantic/source/libs-cli-std-src-chainable-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Chainable-output conformance — the reusable check a project's
//! `chainable_output` baseline capability cites as its gate
//! (`CONTRIBUTING.md` § "CLI convention: stdout tells the agent the next
//! step" — anchor `chainable-output-conformance`).
//!
//! The convention recognizes two real shapes, both accepted here:
//!
//! - **The full `aw.cli.v1` envelope** — `aw`'s reference implementation,
//!   split across two real call sites: `apps/agentic-workflow/src/runtime/envelope.rs`'s
//!   `Envelope::Dispatch` carries a runnable step at `invoke.command`;
//!   `apps/agentic-workflow/src/cli/run.rs`'s `WorkflowEnvelope` (the
//!   `aw run` loop-driver output) carries it at `next.command` instead, and
//!   its sole terminal marker is `completion.workflow_complete == true` (a
//!   terminal envelope omits `next.command` entirely — see
//!   `workflow_envelope_serializes_optional_artifact_quality_profile` in
//!   `run.rs` for the real serialized shape).
//! - **The lightweight form** every other CLI may use instead per
//!   CONTRIBUTING's "Simple CLIs without a full envelope MAY use a lighter
//!   conforming form": either a single top-level JSON `next` field (a
//!   command string, or the literal `"done"`), or — for CLIs that emit plain
//!   text — a fixed trailing stdout line `next: <cmd>` / `next: done`.
//!
//! An output matching none of these is a chainable-output defect: the agent
//! has no way to know what happens next. [`assert_chainable`] is the check;
//! [`assert_command_chainable`] wraps invoking a binary under test around it.

use serde_json::Value;
use std::fmt;

/// A CLI's output carries neither a runnable next command nor an explicit
/// terminal marker — a chainable-output defect. [`fmt::Display`] gives the
/// specific shapes that were checked and not found, so a failing test tells
/// you what to add.
#[derive(Debug, Clone, PartialEq, Eq)]
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-chainable-rs.md#source
pub struct ChainableViolation(String);

/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-chainable-rs.md#source
impl ChainableViolation {
    fn new(reason: impl Into<String>) -> Self {
        Self(reason.into())
    }
}

/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-chainable-rs.md#source
impl fmt::Display for ChainableViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "chainable-output violation: {}", self.0)
    }
}

/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-chainable-rs.md#source
impl std::error::Error for ChainableViolation {}

/// Assert that `output` (a CLI's captured stdout) carries the
/// chainable-output contract: a runnable `next`/`invoke.command` string, or
/// an explicit terminal marker. See the module docs for the exact shapes
/// recognized.
///
/// ```
/// // aw's runtime/envelope.rs Dispatch shape: `invoke.command` is the step.
/// let dispatch = r#"{"action":"dispatch","agent":"score-issue-author","slug":"foo",
///     "invoke":{"command":"aw wi author","args":{"slug":"foo"}}}"#;
/// cli_std::chainable::assert_chainable(dispatch).unwrap();
///
/// // aw run's WorkflowEnvelope shape: `next.command` is the step.
/// let continuing = r#"{"schema_version":"aw.cli.v1","status":"continue",
///     "completion":{"workflow_complete":false},
///     "next":{"kind":"run_command","command":"aw td gen"}}"#;
/// cli_std::chainable::assert_chainable(continuing).unwrap();
///
/// // Same envelope family, terminal: no `next.command`, only the marker.
/// let done = r#"{"schema_version":"aw.cli.v1","status":"done",
///     "completion":{"workflow_complete":true},
///     "next":{"kind":"inspect_parent"}}"#;
/// cli_std::chainable::assert_chainable(done).unwrap();
///
/// // Missing next step entirely: a chainable-output defect.
/// assert!(cli_std::chainable::assert_chainable(r#"{"result":"ok"}"#).is_err());
/// ```
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-chainable-rs.md#source
pub fn assert_chainable(output: &str) -> Result<(), ChainableViolation> {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return Err(ChainableViolation::new(
            "output is empty — no JSON payload and no `next:` line",
        ));
    }

    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        if has_terminal_marker(&value) {
            return Ok(());
        }
        if has_runnable_command(&value) {
            return Ok(());
        }
        return Err(ChainableViolation::new(format!(
            "JSON output has neither a runnable `next`/`invoke.command` string \
             nor a terminal marker (`completion.workflow_complete`, `done`, \
             `status:\"done\"`, or `next:\"done\"`): {trimmed}"
        )));
    }

    // Not JSON — fall back to the lightweight trailing-line form.
    match trailing_next_line(trimmed) {
        Some(NextLine::Done) | Some(NextLine::Command) => Ok(()),
        None => Err(ChainableViolation::new(format!(
            "output is not valid JSON and its last line is not a `next: <cmd>` \
             / `next: done` marker: {:?}",
            last_non_empty_line(trimmed)
        ))),
    }
}

/// Run `command`, then assert its captured stdout is chainable per
/// [`assert_chainable`], returning the captured stdout so the caller can
/// chain further assertions. Plain `std::process::Command` — no extra
/// dev-deps — build it exactly as you'd invoke the binary under test. This
/// is the full adoption recipe (AC1: under 10 lines), using the standard
/// `CARGO_BIN_EXE_<name>` env var Cargo sets for integration tests of a
/// binary named `mytool` in the same crate:
///
/// ```ignore
/// #[test]
/// fn status_output_is_chainable() {
///     let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_mytool"));
///     cmd.arg("status");
///     cli_std::chainable::assert_command_chainable(&mut cmd).unwrap();
/// }
/// ```
/// @spec libs/cli-std/tech-design/semantic/source/libs-cli-std-src-chainable-rs.md#source
pub fn assert_command_chainable(command: &mut std::process::Command) -> anyhow::Result<String> {
    use anyhow::Context;
    let output = command.output().context("run command under test")?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert_chainable(&stdout)
        .map_err(|violation| anyhow::anyhow!("{violation}\n--- stdout ---\n{stdout}"))?;
    Ok(stdout)
}

fn has_terminal_marker(value: &Value) -> bool {
    if value
        .pointer("/completion/workflow_complete")
        .and_then(Value::as_bool)
        == Some(true)
    {
        return true;
    }
    for key in ["done", "workflow_complete", "complete"] {
        if value.get(key).and_then(Value::as_bool) == Some(true) {
            return true;
        }
    }
    if matches!(
        value.get("status").and_then(Value::as_str),
        Some("done") | Some("complete")
    ) {
        return true;
    }
    if matches!(value.get("next"), Some(Value::String(next)) if next.eq_ignore_ascii_case("done")) {
        return true;
    }
    if matches!(
        value.pointer("/next/kind").and_then(Value::as_str),
        Some("done")
    ) {
        return true;
    }
    false
}

fn has_runnable_command(value: &Value) -> bool {
    for pointer in ["/invoke/command", "/next/command"] {
        if let Some(command) = value.pointer(pointer).and_then(Value::as_str) {
            if !command.trim().is_empty() {
                return true;
            }
        }
    }
    if let Some(Value::String(next)) = value.get("next") {
        if !next.trim().is_empty() && !next.eq_ignore_ascii_case("done") {
            return true;
        }
    }
    false
}

enum NextLine {
    Done,
    Command,
}

fn last_non_empty_line(output: &str) -> Option<&str> {
    output.lines().rev().find(|line| !line.trim().is_empty())
}

/// The lightweight form's trailing-line marker: the last non-blank line must
/// be exactly `next: <cmd>` (a runnable command) or `next: done` (terminal).
fn trailing_next_line(output: &str) -> Option<NextLine> {
    let last = last_non_empty_line(output)?;
    let rest = last.trim().strip_prefix("next:")?.trim();
    if rest.is_empty() {
        return None;
    }
    if rest.eq_ignore_ascii_case("done") {
        Some(NextLine::Done)
    } else {
        Some(NextLine::Command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real sample: `runtime/envelope.rs`'s `parses_dispatch_author` test
    /// literal — the `Envelope::Dispatch` shape, `invoke.command` runnable.
    const AW_DISPATCH_ENVELOPE: &str = r#"{"action":"dispatch","agent":"score-issue-author","slug":"foo","invoke":{"command":"aw wi author","args":{"slug":"foo","section":"requirements"}}}"#;

    /// Real shape: `run.rs`'s `WorkflowEnvelope` custom `Serialize` impl for
    /// an in-progress root — `next.command` runnable, no top-level `invoke`
    /// key (confirmed by `run.rs`'s own
    /// `workflow_envelope_serializes_optional_artifact_quality_profile` test:
    /// `assert!(json.get("invoke").is_none())`).
    const AW_RUN_CONTINUE_ENVELOPE: &str = r#"{
        "schema_version": "aw.cli.v1",
        "status": "continue",
        "action": "dispatch",
        "root": {"kind": "wi", "id": "3903"},
        "current": {"kind": "td", "id": "3903"},
        "completion": {
            "root_complete": false,
            "workflow_complete": false,
            "requires_hitl": false,
            "criteria": [],
            "missing": []
        },
        "next": {"kind": "run_command", "command": "aw td create 3903", "reason": "td not yet created"},
        "agent_prompt": "run `aw td create 3903`"
    }"#;

    /// Real shape: `run.rs`'s `WorkflowEnvelope` for a completed root —
    /// `next.command` is absent (skipped when `None`); the only marker is
    /// `completion.workflow_complete == true`.
    const AW_RUN_DONE_ENVELOPE: &str = r#"{
        "schema_version": "aw.cli.v1",
        "status": "done",
        "action": "done",
        "root": {"kind": "project", "id": "jet"},
        "current": {"kind": "project", "id": "jet"},
        "completion": {
            "root_complete": true,
            "workflow_complete": true,
            "requires_hitl": false,
            "criteria": ["project health clean"],
            "missing": []
        },
        "next": {"kind": "inspect_parent", "reason": "root complete; inspect the parent"},
        "agent_prompt": "workflow complete"
    }"#;

    #[test]
    fn real_aw_dispatch_envelope_is_chainable() {
        assert_chainable(AW_DISPATCH_ENVELOPE).expect("real aw dispatch envelope should pass");
    }

    #[test]
    fn real_aw_run_continue_envelope_is_chainable() {
        assert_chainable(AW_RUN_CONTINUE_ENVELOPE).expect("real aw run envelope should pass");
    }

    #[test]
    fn real_aw_run_done_envelope_is_chainable_via_terminal_marker() {
        assert_chainable(AW_RUN_DONE_ENVELOPE)
            .expect("terminal aw run envelope should pass via completion.workflow_complete");
    }

    #[test]
    fn next_less_json_fails_with_useful_message() {
        let err = assert_chainable(r#"{"result": "ok", "count": 3}"#)
            .expect_err("next-less JSON must fail");
        let message = err.to_string();
        assert!(
            message.contains("next"),
            "message should mention `next`: {message}"
        );
        assert!(
            message.contains("invoke.command"),
            "message should name the recognized shapes: {message}"
        );
    }

    #[test]
    fn lightweight_json_next_field_is_chainable() {
        assert_chainable(r#"{"data": "x", "next": "aw td gen"}"#).unwrap();
        assert_chainable(r#"{"data": "x", "next": "done"}"#).unwrap();
    }

    #[test]
    fn lightweight_json_next_missing_fails() {
        assert!(assert_chainable(r#"{"data": "x"}"#).is_err());
    }

    #[test]
    fn trailing_stdout_line_is_chainable() {
        assert_chainable("built ok\nnext: aw td gen").unwrap();
        assert_chainable("built ok\nnext: done").unwrap();
    }

    #[test]
    fn plain_text_without_next_line_fails() {
        let err = assert_chainable("just some log output\nno marker here")
            .expect_err("plain text with no marker must fail");
        assert!(err.to_string().contains("next:"));
    }

    #[test]
    fn empty_output_fails() {
        assert!(assert_chainable("   \n  ").is_err());
    }

    #[test]
    fn assert_command_chainable_wraps_process_output() {
        // `echo` doubles as the "binary under test": it emits the
        // lightweight trailing-line form, proving the harness without
        // spawning a real ecosystem CLI.
        let mut cmd = std::process::Command::new("echo");
        cmd.arg("next: done");
        let stdout = assert_command_chainable(&mut cmd).expect("echo output should be chainable");
        assert!(stdout.trim().ends_with("next: done"));
    }

    #[test]
    fn assert_command_chainable_surfaces_violation() {
        let mut cmd = std::process::Command::new("echo");
        cmd.arg("no marker here");
        let err = assert_command_chainable(&mut cmd).expect_err("should surface violation");
        assert!(err.to_string().contains("chainable-output violation"));
    }
}
// CODEGEN-END
