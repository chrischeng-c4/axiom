---
id: projects-meter-src-capture-bench-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: runtime-resource-attribution
    role: primary
    gap: benchmark-regression-api
    claim: benchmark-regression-api
    coverage: full
    rationale: "Source template implements meter performance measurement and regression reporting surfaces."
---

# Standardized projects/meter/src/capture/bench.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/capture/bench.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `BenchOutcome` | projects/meter/src/capture/bench.rs | struct | pub | 27 |  |
| `delegate_bench` | projects/meter/src/capture/bench.rs | function | pub | 45 | delegate_bench(target: impl AsRef<Path>) -> std::io::Result<BenchOutcome> |
| `load_regression_report` | projects/meter/src/capture/bench.rs | function | pub | 102 | load_regression_report(path: impl AsRef<Path>) -> Result<RegressionReport, String> |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `meter bench` capture — delegate `cargo bench` and load a regression baseline.
//!
//! `meter` does not re-implement a benchmark harness for the CLI path: it
//! DELEGATES to `cargo bench` for the target (capturing argv + timing into a
//! [`RunnerRecord`]), and — when an agent supplies a baseline file — loads a
//! previously serialized [`RegressionReport`](crate::baseline::RegressionReport)
//! so the exit-2 regression path is reachable from the CLI.
//!
//! Live "run benchmarks + auto-detect regression" needs a baseline store that
//! is out of this wave's scope; the deterministic exit-2 proof lives in the
//! `producer`/`builder` unit tests. This module ships the two CLI-reachable
//! pieces: the `cargo bench` delegate and the baseline loader.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

use chrono::Utc;

use crate::baseline::RegressionReport;
use crate::report::envelope::RunnerRecord;

/// Outcome of a delegated `cargo bench` run.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-bench-rs.md#source
pub struct BenchOutcome {
    /// The recorded child invocation (argv, timing, exit code).
    pub record: RunnerRecord,
    /// The child's exit code (clamped to `0..=255`).
    pub child_exit_code: i32,
}

/// Delegate a `cargo bench` run for the crate at `target`.
///
/// Stderr is INHERITED so the live benchmark output streams to the terminal;
/// stdout is CAPTURED (not inherited) so it cannot pollute `meter`'s own
/// JSON-on-stdout document — the report is the only thing on stdout. The
/// captured bench output is human-oriented and not parsed here. Returns a
/// [`BenchOutcome`] whose `record.kind == "cargo-bench"` and `record.delegated
/// == true`. A spawn failure (cargo missing / target unbuildable to the point of
/// not launching) surfaces as `Err`, which the dispatch layer maps to a
/// `ToolError(5)` report.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-bench-rs.md#source
pub fn delegate_bench(target: impl AsRef<Path>) -> std::io::Result<BenchOutcome> {
    let target = target.as_ref();
    let argv: Vec<String> = vec![
        "cargo".into(),
        "bench".into(),
        "--manifest-path".into(),
        manifest_path(target),
    ];

    let started_at = Utc::now();
    let started = Instant::now();

    let mut cmd = Command::new("cargo");
    cmd.args(&argv[1..]);
    // Inherit stderr so live bench progress streams to the terminal; CAPTURE
    // stdout so it never corrupts meter's single JSON stdout document.
    cmd.stderr(Stdio::inherit());
    cmd.stdout(Stdio::piped());

    let output = cmd.output()?;
    let duration_ms = started.elapsed().as_millis() as u64;
    let finished_at = Utc::now();

    let child_exit_code = output.status.code().unwrap_or(-1).clamp(0, 255);

    let record = RunnerRecord {
        command: argv,
        kind: "cargo-bench".to_string(),
        started_at,
        finished_at: Some(finished_at),
        exit_code: Some(child_exit_code),
        duration_ms: Some(duration_ms),
        delegated: true,
    };

    Ok(BenchOutcome {
        record,
        child_exit_code,
    })
}

/// Build the `--manifest-path` argument for `target`. If `target` already points
/// at a `Cargo.toml`, use it verbatim; otherwise append `Cargo.toml` to the dir.
fn manifest_path(target: &Path) -> String {
    if target.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml") {
        target.display().to_string()
    } else {
        target.join("Cargo.toml").display().to_string()
    }
}

/// Load a serialized [`RegressionReport`] from `path`.
///
/// The file is the JSON form produced by serializing a `RegressionReport` (the
/// derives added in Wave 4). `Err(msg)` on a read or parse failure so the
/// dispatch layer can surface a `ToolError` rather than a fake-clean report.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-bench-rs.md#source
pub fn load_regression_report(path: impl AsRef<Path>) -> Result<RegressionReport, String> {
    let path = path.as_ref();
    let raw = std::fs::read_to_string(path)
        .map_err(|e| format!("could not read baseline `{}`: {e}", path.display()))?;
    serde_json::from_str::<RegressionReport>(&raw).map_err(|e| {
        format!(
            "could not parse baseline `{}` as a RegressionReport: {e}",
            path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::baseline::{PercentileType, Regression, RegressionSeverity, RegressionSummary};

    #[test]
    fn manifest_path_appends_cargo_toml_for_dir() {
        assert_eq!(manifest_path(Path::new("/x/crate")), "/x/crate/Cargo.toml");
    }

    #[test]
    fn manifest_path_uses_explicit_manifest() {
        assert_eq!(
            manifest_path(Path::new("/x/crate/Cargo.toml")),
            "/x/crate/Cargo.toml"
        );
    }

    #[test]
    fn load_missing_baseline_is_err() {
        let r = load_regression_report("/nonexistent/meter/bench/baseline.json");
        assert!(r.is_err());
    }

    #[test]
    fn load_roundtrips_a_serialized_report() {
        // A serialized RegressionReport loads back into findings-producing form.
        let report = RegressionReport {
            baseline_timestamp: "2026-01-01T00:00:00Z".to_string(),
            current_timestamp: "2026-01-02T00:00:00Z".to_string(),
            regressions: vec![Regression {
                name: "bench_x".to_string(),
                percentile_type: PercentileType::Mean,
                baseline_value_ms: 10.0,
                current_value_ms: 12.0,
                percent_change: 20.0,
                ci_overlap: false,
                severity: RegressionSeverity::Severe,
            }],
            improvements: Vec::new(),
            summary: RegressionSummary {
                total_benchmarks: 1,
                regressions_found: 1,
                improvements_found: 0,
                unchanged: 0,
            },
        };
        let json = serde_json::to_string(&report).unwrap();
        let mut path = std::env::temp_dir();
        path.push(format!("meter-bench-baseline-{}.json", std::process::id()));
        std::fs::write(&path, &json).unwrap();

        let loaded = load_regression_report(&path).unwrap();
        assert_eq!(loaded.regressions.len(), 1);
        assert_eq!(loaded.regressions[0].name, "bench_x");
        assert_eq!(loaded.regressions[0].severity, RegressionSeverity::Severe);

        let _ = std::fs::remove_file(&path);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/capture/bench.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/capture/bench.rs` captured during meter full-codegen standardization.
```
