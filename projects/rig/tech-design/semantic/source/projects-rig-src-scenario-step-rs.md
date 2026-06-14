---
id: projects-rig-src-scenario-step-rs
capability_refs:
  - id: scenario-engine
    role: primary
    claim: scenario-step-dsl-execution
    coverage: partial
    rationale: "This source unit implements rig scenario discovery, execution, verdict, or report behavior used by the scenario engine."
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/src/scenario/step.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/src/scenario/step.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AssertStep` | projects/rig/src/scenario/step.rs | struct | pub | 109 |  |
| `ExecStep` | projects/rig/src/scenario/step.rs | struct | pub | 148 |  |
| `HttpExpect` | projects/rig/src/scenario/step.rs | struct | pub | 15 |  |
| `HttpRequest` | projects/rig/src/scenario/step.rs | struct | pub | 66 |  |
| `HttpStep` | projects/rig/src/scenario/step.rs | struct | pub | 78 |  |
| `MeasureRssStep` | projects/rig/src/scenario/step.rs | struct | pub | 132 |  |
| `SampleStep` | projects/rig/src/scenario/step.rs | struct | pub | 91 |  |
| `Step` | projects/rig/src/scenario/step.rs | enum | pub | 169 |  |
| `WaitUntilStep` | projects/rig/src/scenario/step.rs | struct | pub | 117 |  |
| `name` | projects/rig/src/scenario/step.rs | function | pub | 181 | name(&self) -> &str |
| `status_ok` | projects/rig/src/scenario/step.rs | function | pub | 34 | status_ok(&self, status: u16) -> bool |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! The step DSL — `[[steps]]` entries, serde-tagged by `type`.
//!
//! Steps are deliberately few and declarative; `exec` is the escape hatch.
//! Anything reachable over HTTP (including toxiproxy control) is an `http`
//! step — rig carries no chaos-specific machinery.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Expectations for one HTTP exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpExpect {
    /// Required response status (default 200).
    #[serde(default = "default_status")]
    pub status: u16,
    /// Accept ANY of these statuses instead of `status` — for
    /// create-if-absent idioms like `statuses = [200, 409]`.
    #[serde(default)]
    pub statuses: Vec<u16>,
    /// Per-request timeout (default 5000).
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    /// Dot-path assertions over the response JSON, e.g. `"$.total" = ">= 1"`.
    #[serde(default)]
    pub jsonpath: BTreeMap<String, String>,
}

impl HttpExpect {
    /// Does `status` satisfy this expectation's status contract?
    pub fn status_ok(&self, status: u16) -> bool {
        if self.statuses.is_empty() {
            status == self.status
        } else {
            self.statuses.contains(&status)
        }
    }
}

impl Default for HttpExpect {
    fn default() -> Self {
        Self {
            status: default_status(),
            statuses: Vec::new(),
            timeout_ms: default_timeout_ms(),
            jsonpath: BTreeMap::new(),
        }
    }
}

fn default_status() -> u16 {
    200
}
fn default_timeout_ms() -> u64 {
    5000
}

/// One HTTP request template (used by `http`, `sample.request`,
/// `wait_until.probe`, and `[load.request]`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub expect: HttpExpect,
}

/// `type = "http"` — one request; optional jsonpath captures into vars.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpStep {
    pub name: String,
    #[serde(flatten)]
    pub request: HttpRequest,
    /// var name -> response dot-path (`$.total`) or response meta
    /// (`status`, `latency_ms`).
    #[serde(default)]
    pub capture: BTreeMap<String, String>,
}

/// `type = "sample"` — repeat one request N times, fold latency stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleStep {
    pub name: String,
    pub samples: u32,
    /// Don't fail the step on per-request expect misses (the miss count is
    /// still available as `fail_count`).
    #[serde(default)]
    pub allow_failures: bool,
    pub request: HttpRequest,
    /// var name -> stat key: `p50_ms` | `p90_ms` | `p99_ms` | `mean_ms` |
    /// `fail_count` | `ok_count`.
    #[serde(default)]
    pub capture: BTreeMap<String, String>,
}

/// `type = "assert"` — expressions over captured vars.
/// Grammar: `IDENT OP (NUMBER ['*' IDENT] | IDENT)`, OP ∈ {== != < <= > >=}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertStep {
    pub name: String,
    pub exprs: Vec<String>,
}

/// `type = "wait_until"` — poll a probe until it passes or the budget ends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitUntilStep {
    pub name: String,
    pub budget_secs: u64,
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,
    pub probe: HttpRequest,
}

fn default_interval_ms() -> u64 {
    500
}

/// `type = "measure_rss"` — sample a process's resident set size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasureRssStep {
    pub name: String,
    /// Process name for `pgrep -n` (racy; prefer `pid_var`).
    #[serde(default)]
    pub process: Option<String>,
    /// Var holding a pid captured earlier (e.g. from vat export or exec).
    #[serde(default)]
    pub pid_var: Option<String>,
    /// var name -> `rss_kb`.
    #[serde(default)]
    pub capture: BTreeMap<String, String>,
}

/// `type = "exec"` — escape hatch: run a command under a timeout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecStep {
    pub name: String,
    pub cmd: Vec<String>,
    #[serde(default = "default_exec_timeout")]
    pub timeout_secs: u64,
    /// Required exit code (default 0).
    #[serde(default)]
    pub expect_exit_code: i32,
    /// var name -> `stdout` (trimmed) | `exit_code`.
    #[serde(default)]
    pub capture: BTreeMap<String, String>,
}

fn default_exec_timeout() -> u64 {
    120
}

/// One scenario step, tagged by `type`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Step {
    Http(HttpStep),
    Sample(SampleStep),
    Assert(AssertStep),
    WaitUntil(WaitUntilStep),
    MeasureRss(MeasureRssStep),
    Exec(ExecStep),
    Sleep { name: String, secs: u64 },
}

impl Step {
    pub fn name(&self) -> &str {
        match self {
            Step::Http(s) => &s.name,
            Step::Sample(s) => &s.name,
            Step::Assert(s) => &s.name,
            Step::WaitUntil(s) => &s.name,
            Step::MeasureRss(s) => &s.name,
            Step::Exec(s) => &s.name,
            Step::Sleep { name, .. } => name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_step_deserializes_with_defaults() {
        let s: Step = toml::from_str(
            r#"
type = "http"
name = "create_proxy"
method = "POST"
url = "http://{{toxiproxy}}/proxies"
body = '{"name":"lumen"}'
"#,
        )
        .unwrap();
        match s {
            Step::Http(h) => {
                assert_eq!(h.request.expect.status, 200);
                assert_eq!(h.request.expect.timeout_ms, 5000);
            }
            _ => panic!("expected http"),
        }
    }

    #[test]
    fn sample_step_with_nested_request() {
        let s: Step = toml::from_str(
            r#"
type = "sample"
name = "baseline"
samples = 100
capture = { baseline_p99 = "p99_ms" }
[request]
method = "POST"
url = "http://{{proxy}}/search"
expect = { status = 200, timeout_ms = 2000 }
"#,
        )
        .unwrap();
        match s {
            Step::Sample(s) => {
                assert_eq!(s.samples, 100);
                assert!(!s.allow_failures);
                assert_eq!(s.capture["baseline_p99"], "p99_ms");
                assert_eq!(s.request.expect.timeout_ms, 2000);
            }
            _ => panic!("expected sample"),
        }
    }

    #[test]
    fn unknown_type_is_an_error() {
        let r: Result<Step, _> = toml::from_str(
            r#"
type = "teleport"
name = "x"
"#,
        );
        assert!(r.is_err());
    }

    #[test]
    fn sleep_and_assert_parse() {
        let s: Step = toml::from_str(r#"type = "sleep"
name = "settle"
secs = 2"#)
            .unwrap();
        assert_eq!(s.name(), "settle");
        let s: Step = toml::from_str(
            r#"
type = "assert"
name = "recovered"
exprs = ["recovery_p99 <= 2 * baseline_p99"]
"#,
        )
        .unwrap();
        assert_eq!(s.name(), "recovered");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/src/scenario/step.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/src/scenario/step.rs` captured during rig
      standardization onto the codegen ladder.
```
