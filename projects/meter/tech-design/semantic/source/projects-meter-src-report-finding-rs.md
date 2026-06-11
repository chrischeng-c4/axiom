---
id: projects-meter-src-report-finding-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/report/finding.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/report/finding.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Finding` | projects/meter/src/report/finding.rs | struct | pub | 20 |  |
| `Invoke` | projects/meter/src/report/finding.rs | struct | pub | 49 |  |
| `Kind` | projects/meter/src/report/finding.rs | enum | pub | 133 |  |
| `Location` | projects/meter/src/report/finding.rs | struct | pub | 71 |  |
| `Severity` | projects/meter/src/report/finding.rs | enum | pub | 84 |  |
| `all` | projects/meter/src/report/finding.rs | function | pub | 117 | all() -> [Severity; 5] |
| `all` | projects/meter/src/report/finding.rs | function | pub | 183 | all() -> [Kind; 8] |
| `as_str` | projects/meter/src/report/finding.rs | function | pub | 106 | as_str(&self) -> &'static str |
| `as_str` | projects/meter/src/report/finding.rs | function | pub | 155 | as_str(&self) -> &'static str |
| `command` | projects/meter/src/report/finding.rs | function | pub | 60 | command(command: impl Into<String>) -> Self |
| `finding_id` | projects/meter/src/report/finding.rs | function | pub | 200 | finding_id(kind: Kind, slug: impl AsRef<str>) -> String |
| `id_prefix` | projects/meter/src/report/finding.rs | function | pub | 169 | id_prefix(&self) -> &'static str |
| `rank` | projects/meter/src/report/finding.rs | function | pub | 95 | rank(&self) -> u8 |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/src/report/finding.rs -->
````rust
//! Finding schema — the per-issue unit inside a [`MeterReport`](super::MeterReport).
//!
//! A [`Finding`] is the machine-actionable record of one thing `meter` found: a
//! hot spot, a boundary cost, a regression, or a delegated test failure. Every
//! finding carries a STABLE/DETERMINISTIC `id`, a `severity`, a closed public
//! `kind`, human prose (`title`/`detail`/`remediation`), a literally runnable
//! next step (`invoke`), per-kind structured `evidence`, and an optional source
//! [`Location`].
//!
//! Field order = declaration order = byte-stable JSON. Enums serialize
//! `snake_case`.

use serde::{Deserialize, Serialize};

/// A single machine-actionable issue surfaced by a `meter` verb.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-finding-rs.md#source
pub struct Finding {
    /// STABLE/DETERMINISTIC `"{kind}:{slug}"` identifier. Examples:
    /// `"rust_vuln:RUSTSEC-2021-0001"`, `"hotspot:mb_release"`,
    /// `"boundary:RustConvert"`, `"regression:bench_sha256"`,
    /// `"fuzz_crash:parse_header:<blake3(input)[..8]>"`,
    /// `"test_failure:mod::name"`.
    pub id: String,
    /// Severity bucket; drives `FindingsSummary` tallies and sort order.
    pub severity: Severity,
    /// Closed-set classification of what was found.
    pub kind: Kind,
    /// Short human title.
    pub title: String,
    /// What was found, in prose.
    pub detail: String,
    /// What to do next, in prose.
    pub remediation: String,
    /// Literally-runnable next step.
    pub invoke: Invoke,
    /// Per-kind structured proof (see `Kind` doc comments for the shape).
    pub evidence: serde_json::Value,
    /// Optional source location.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
}

/// A literally-runnable next step the agent can execute verbatim.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-finding-rs.md#source
pub struct Invoke {
    /// The command to run (e.g. `"cargo test -p meter mod::name"`).
    pub command: String,
    /// Structured args, free-form JSON.
    #[serde(default)]
    pub args: serde_json::Value,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-finding-rs.md#source
impl Invoke {
    /// Construct a command-only invoke with a null args payload.
    pub fn command(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: serde_json::Value::Null,
        }
    }
}

/// Optional source location for a finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-finding-rs.md#source
pub struct Location {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

/// Severity bucket. Sorted critical -> info when ordering findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-finding-rs.md#source
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-finding-rs.md#source
impl Severity {
    /// Descending rank: higher number = more severe (for sort-desc).
    pub fn rank(&self) -> u8 {
        match self {
            Severity::Critical => 4,
            Severity::High => 3,
            Severity::Medium => 2,
            Severity::Low => 1,
            Severity::Info => 0,
        }
    }

    /// Lowercase snake_case label, matching the serde representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::High => "high",
            Severity::Medium => "medium",
            Severity::Low => "low",
            Severity::Info => "info",
        }
    }

    /// All severities, most-severe first.
    pub fn all() -> [Severity; 5] {
        [
            Severity::Critical,
            Severity::High,
            Severity::Medium,
            Severity::Low,
            Severity::Info,
        ]
    }
}

/// Closed-set classification of a finding. Each variant documents the
/// `evidence` JSON shape it carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-finding-rs.md#source
pub enum Kind {
    /// `meter profile` capture — `{symbol,self_ns,total_ns,pct,samples,rank}` (C1 contract).
    Hotspot,
    /// `meter profile` embed (BoundaryTracer) — `{phase,self_ns,total_ns,pct,samples}`.
    BoundaryCost,
    /// `meter bench` — `{bench,baseline_ms,current_ms,percent_change,severity,ci_overlap}` => exit 2.
    Regression,
    /// Legacy carried audit internals — `{advisory_id,package,version,cvss,title}`.
    RustVuln,
    /// Legacy carried audit internals — `{package,kind}` (yanked/unmaintained).
    RustWarning,
    /// Legacy carried fuzz internals — `{input_b64,panic_msg,strategy,seed}`.
    FuzzCrash,
    /// Legacy carried fuzz internals — `{payload,category,reflected}`.
    Injection,
    /// `meter test` — `{name,stdout_tail}` (delegated, informational).
    TestFailure,
    /// `meter profile`/`run` capture vitals — `{cpu_time_ms,wall_time_ms,peak_rss_bytes}`
    /// (Info), or `{gate,limit,observed,unit}` for a breached meter.toml `[gate]`
    /// ceiling (High => exit 1).
    Vital,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-finding-rs.md#source
impl Kind {
    /// Lowercase snake_case label, matching the serde representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Kind::Hotspot => "hotspot",
            Kind::BoundaryCost => "boundary_cost",
            Kind::Regression => "regression",
            Kind::RustVuln => "rust_vuln",
            Kind::RustWarning => "rust_warning",
            Kind::FuzzCrash => "fuzz_crash",
            Kind::Injection => "injection",
            Kind::TestFailure => "test_failure",
            Kind::Vital => "vital",
        }
    }

    /// The conventional `id` prefix for this kind (e.g. `"test_failure"`).
    pub fn id_prefix(&self) -> &'static str {
        match self {
            Kind::Hotspot => "hotspot",
            Kind::BoundaryCost => "boundary",
            Kind::Regression => "regression",
            Kind::RustVuln => "rust_vuln",
            Kind::RustWarning => "rust_warning",
            Kind::FuzzCrash => "fuzz_crash",
            Kind::Injection => "injection",
            Kind::TestFailure => "test_failure",
            Kind::Vital => "vital",
        }
    }

    /// Public meter kinds, in declaration order. Legacy carried variants remain
    /// serializable for internal modules but are intentionally absent from the
    /// public `meter spec` schema/catalog.
    pub fn all() -> [Kind; 5] {
        [
            Kind::Hotspot,
            Kind::BoundaryCost,
            Kind::Regression,
            Kind::TestFailure,
            Kind::Vital,
        ]
    }
}

/// Build a deterministic finding id as `"{prefix}:{slug}"`. The slug is taken
/// verbatim (callers are responsible for stable slugs).
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-finding-rs.md#source
pub fn finding_id(kind: Kind, slug: impl AsRef<str>) -> String {
    format!("{}:{}", kind.id_prefix(), slug.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_serializes_snake_case() {
        let j = serde_json::to_string(&Severity::Critical).unwrap();
        assert_eq!(j, "\"critical\"");
    }

    #[test]
    fn kind_serializes_snake_case() {
        let j = serde_json::to_string(&Kind::TestFailure).unwrap();
        assert_eq!(j, "\"test_failure\"");
        assert_eq!(
            serde_json::to_string(&Kind::RustVuln).unwrap(),
            "\"rust_vuln\""
        );
    }

    #[test]
    fn finding_id_uses_prefix() {
        assert_eq!(
            finding_id(Kind::TestFailure, "mod::name"),
            "test_failure:mod::name"
        );
        assert_eq!(
            finding_id(Kind::Hotspot, "mb_release"),
            "hotspot:mb_release"
        );
    }

    #[test]
    fn severity_rank_orders_desc() {
        assert!(Severity::Critical.rank() > Severity::High.rank());
        assert!(Severity::High.rank() > Severity::Info.rank());
    }

    #[test]
    fn finding_roundtrips() {
        let f = Finding {
            id: finding_id(Kind::TestFailure, "a::b"),
            severity: Severity::High,
            kind: Kind::TestFailure,
            title: "t".into(),
            detail: "d".into(),
            remediation: "r".into(),
            invoke: Invoke::command("cargo test"),
            evidence: serde_json::json!({"name":"a::b"}),
            location: None,
        };
        let s = serde_json::to_string(&f).unwrap();
        let back: Finding = serde_json::from_str(&s).unwrap();
        assert_eq!(back.id, f.id);
        assert_eq!(back.kind, Kind::TestFailure);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/report/finding.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/report/finding.rs` captured during meter full-codegen standardization.
```
