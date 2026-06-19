---
id: projects-meter-src-report-persist-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/report/persist.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/report/persist.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `LAST_REPORT_REL` | projects/meter/src/report/persist.rs | constant | pub | 15 |  |
| `last_report_path` | projects/meter/src/report/persist.rs | function | pub | 19 | last_report_path(base: impl AsRef<Path>) -> PathBuf |
| `read_last_report` | projects/meter/src/report/persist.rs | function | pub | 51 | read_last_report() -> Option<MeterReport> |
| `read_last_report_in` | projects/meter/src/report/persist.rs | function | pub | 58 | read_last_report_in(base: impl AsRef<Path>) -> Option<MeterReport> |
| `write_last_report` | projects/meter/src/report/persist.rs | function | pub | 28 | write_last_report(report: &MeterReport) -> std::io::Result<PathBuf> |
| `write_last_report_in` | projects/meter/src/report/persist.rs | function | pub | 35 | write_last_report_in(     base: impl AsRef<Path>,     report: &MeterReport, ) -> std::io::Result<PathBuf> |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Persistence of the last report to `.meter/last-report.json`.
//!
//! Every populator verb (anything that actually runs an engine — `test`,
//! `profile`, `bench`, `run`) writes its report here so the
//! read-only `report`/`state` verb can re-project it with zero engine work.
//! Offline self-describers (`spec`, `llm`) do NOT persist.

use std::path::{Path, PathBuf};

use super::envelope::MeterReport;

/// Relative path (under the working dir) the report cache lives at.
pub const LAST_REPORT_REL: &str = ".meter/last-report.json";

/// Resolve the absolute cache path under `base` (typically the cwd).
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-persist-rs.md#source
pub fn last_report_path(base: impl AsRef<Path>) -> PathBuf {
    base.as_ref().join(LAST_REPORT_REL)
}

/// Write `report` to `<cwd>/.meter/last-report.json`, creating `.meter/` as needed.
///
/// Best-effort: a write failure is surfaced as `Err` but callers generally log
/// it to stderr and continue — losing the cache never changes the verb's exit.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-persist-rs.md#source
pub fn write_last_report(report: &MeterReport) -> std::io::Result<PathBuf> {
    let base = std::env::current_dir()?;
    write_last_report_in(&base, report)
}

/// Write `report` to `<base>/.meter/last-report.json`.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-persist-rs.md#source
pub fn write_last_report_in(
    base: impl AsRef<Path>,
    report: &MeterReport,
) -> std::io::Result<PathBuf> {
    let path = last_report_path(&base);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(report)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(&path, json)?;
    Ok(path)
}

/// Read the persisted report from `<cwd>/.meter/last-report.json`, if present.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-persist-rs.md#source
pub fn read_last_report() -> Option<MeterReport> {
    let base = std::env::current_dir().ok()?;
    read_last_report_in(&base)
}

/// Read the persisted report from `<base>/.meter/last-report.json`, if present.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-persist-rs.md#source
pub fn read_last_report_in(base: impl AsRef<Path>) -> Option<MeterReport> {
    let path = last_report_path(&base);
    let data = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::builder::ReportBuilder;
    use crate::report::envelope::EnvBlock;

    fn env() -> EnvBlock {
        EnvBlock {
            os: "macos".into(),
            arch: "aarch64".into(),
            nextest_present: false,
            sampler_backend: "macos-sample".into(),
            rustc_version: None,
            note: String::new(),
        }
    }

    #[test]
    fn write_then_read_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        let mut b = ReportBuilder::new("test", "/x");
        b.with_environment(env());
        let report = b.finalize();
        let path = write_last_report_in(dir.path(), &report).unwrap();
        assert!(path.exists());
        let back = read_last_report_in(dir.path()).unwrap();
        assert_eq!(back.verb, "test");
        assert_eq!(back.schema_version, report.schema_version);
    }

    #[test]
    fn read_absent_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(read_last_report_in(dir.path()).is_none());
    }

    #[test]
    fn path_is_under_dot_meter() {
        let p = last_report_path("/base");
        assert!(p.ends_with(".meter/last-report.json"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/report/persist.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/report/persist.rs` captured during meter full-codegen standardization.
```
