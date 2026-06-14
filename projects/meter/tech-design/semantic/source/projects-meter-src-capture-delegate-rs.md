---
id: projects-meter-src-capture-delegate-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: delegated-runner-exit-code-contract
    claim: delegated-runner-exit-code-contract
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/capture/delegate.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/capture/delegate.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DelegateOutcome` | projects/meter/src/capture/delegate.rs | struct | pub | 24 |  |
| `delegate_test` | projects/meter/src/capture/delegate.rs | function | pub | 42 | delegate_test(     passthrough: &[String],     nextest_present: bool, ) -> std::io::Result<DelegateOutcome> |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `meter test` delegate — run the real test runner and FORWARD its exit code.
//!
//! `meter` is NOT a test framework: it delegates to `cargo nextest run` (when
//! present) or falls back to `cargo test`. The child's stderr is INHERITED so
//! the live runner output reaches the terminal unchanged; stdout is captured so
//! `meter` can best-effort-parse the `test result:` summary lines into
//! [`Finding`]s of kind `TestFailure`. The child's exit code is captured into a
//! [`RunnerRecord`] and forwarded verbatim by the caller — meter imposes no 0/1/2
//! verdict here.

use std::process::{Command, Stdio};
use std::time::Instant;

use chrono::Utc;

use crate::report::envelope::RunnerRecord;
use crate::report::finding::Finding;
use crate::report::producer::generic_test_failure;

/// Outcome of a delegated test run.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-delegate-rs.md#source
pub struct DelegateOutcome {
    /// The recorded child invocation (argv, timing, forwarded exit).
    pub record: RunnerRecord,
    /// Findings parsed from the runner output (best-effort).
    pub findings: Vec<Finding>,
    /// The child's exit code, clamped to `0..=255` for forwarding.
    pub child_exit_code: i32,
}

/// Delegate a test run for `target`-args, forwarding the child exit code.
///
/// `passthrough` is appended after the runner's own args (e.g. a filter, or
/// `--test-threads=1`); `nextest_present` selects the runner (else cargo test).
///
/// Stderr is inherited (live output); stdout is captured for parsing. Returns a
/// [`DelegateOutcome`] whose `record.delegated == true` and whose
/// `record.exit_code == Some(child_exit_code)`.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-delegate-rs.md#source
pub fn delegate_test(
    passthrough: &[String],
    nextest_present: bool,
) -> std::io::Result<DelegateOutcome> {
    let (kind, mut argv): (&str, Vec<String>) = if nextest_present {
        (
            "nextest",
            vec!["cargo".into(), "nextest".into(), "run".into()],
        )
    } else {
        ("cargo-test", vec!["cargo".into(), "test".into()])
    };
    argv.extend(passthrough.iter().cloned());

    let started_at = Utc::now();
    let started = Instant::now();

    let mut cmd = Command::new("cargo");
    // Skip the leading "cargo" element of argv when building the Command.
    cmd.args(&argv[1..]);
    // Inherit stderr so the live runner output streams to the terminal.
    cmd.stderr(Stdio::inherit());
    // Capture stdout so we can parse the `test result:` summary lines.
    cmd.stdout(Stdio::piped());

    let output = cmd.output()?;
    let duration_ms = started.elapsed().as_millis() as u64;
    let finished_at = Utc::now();

    let raw_code = output.status.code().unwrap_or(-1);
    let child_exit_code = raw_code.clamp(0, 255);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let findings = parse_findings(&stdout, child_exit_code, &argv);

    let record = RunnerRecord {
        command: argv,
        kind: kind.to_string(),
        started_at,
        finished_at: Some(finished_at),
        exit_code: Some(child_exit_code),
        duration_ms: Some(duration_ms),
        delegated: true,
    };

    Ok(DelegateOutcome {
        record,
        findings,
        child_exit_code,
    })
}

/// Best-effort parse of runner stdout into `TestFailure` findings.
///
/// Strategy: scan for `cargo test`-style `test <name> ... FAILED` lines (the
/// most reliable per-test signal). If the run exited non-zero but no per-test
/// failure was parsed, emit ONE generic `TestFailure` so a non-zero outcome is
/// never silently finding-free.
fn parse_findings(stdout: &str, child_exit_code: i32, argv: &[String]) -> Vec<Finding> {
    use crate::report::producer::test_failure_finding;

    let mut findings = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        // libtest line shape: `test some::path ... FAILED`
        if let Some(rest) = line.strip_prefix("test ") {
            if rest.ends_with("FAILED") || rest.contains("... FAILED") {
                if let Some(name) = rest.split_whitespace().next() {
                    findings.push(test_failure_finding(name, "", None, None));
                }
            }
        }
    }

    if findings.is_empty() && child_exit_code != 0 {
        let target = argv.join(" ");
        findings.push(generic_test_failure(&target, child_exit_code));
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_failed_test_lines() {
        let out = "\
running 2 tests
test mymod::passes ... ok
test mymod::breaks ... FAILED

test result: FAILED. 1 passed; 1 failed; 0 ignored
";
        let f = parse_findings(out, 101, &["cargo".into(), "test".into()]);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].id, "test_failure:mymod::breaks");
    }

    #[test]
    fn nonzero_without_parse_emits_generic() {
        let out = "compile error: something exploded\n";
        let f = parse_findings(out, 101, &["cargo".into(), "test".into()]);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].id, "test_failure:delegated-run");
    }

    #[test]
    fn clean_run_emits_no_findings() {
        let out = "\
test mymod::a ... ok
test result: ok. 1 passed; 0 failed
";
        let f = parse_findings(out, 0, &["cargo".into(), "test".into()]);
        assert!(f.is_empty());
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/capture/delegate.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/capture/delegate.rs` captured during meter full-codegen standardization.
```
