---
id: projects-rig-tests-engine_e2e-rs
capability_refs:
  - id: scenario-engine
    role: primary
    claim: scenario-step-dsl-execution
    coverage: partial
    rationale: "This source unit implements rig scenario discovery, execution, verdict, or report behavior used by the scenario engine."
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/tests/engine_e2e.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/tests/engine_e2e.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Engine end-to-end: a stub HTTP server (std TcpListener, no framework)
//! and a multi-step scenario through `engine::run_scenario`.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;

use rig::engine::run_scenario;
use rig::report::Kind;
use rig::scenario::parse_scenario;

/// Serve `{"total": 7, "hits": [{"id": "a"}]}` to every request until the
/// listener drops. Returns the bound address.
fn spawn_stub() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind stub");
    let addr = listener.local_addr().unwrap().to_string();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            std::thread::spawn(move || {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let body = r#"{"total": 7, "hits": [{"id": "a"}]}"#;
                let resp = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(resp.as_bytes());
            });
        }
    });
    addr
}

fn scenario_text(addr: &str, tail: &str) -> String {
    format!(
        r#"
[record]
suite = "rig"
dimension = "demo"
case = "stub_roundtrip"
subject = "engine drives a stub server"
kind = "e2e"
expected = "pass"

[env]
upstream = "{addr}"

[[steps]]
type = "http"
name = "first_search"
method = "GET"
url = "http://{{{{upstream}}}}/search"
expect = {{ status = 200, jsonpath = {{ "$.total" = ">= 1" }} }}
capture = {{ total = "$.total", lat = "latency_ms" }}

[[steps]]
type = "sample"
name = "baseline"
samples = 20
capture = {{ p99 = "p99_ms", fails = "fail_count" }}
[steps.request]
method = "GET"
url = "http://{{{{upstream}}}}/search"

{tail}
"#
    )
}

#[test]
fn engine_runs_http_sample_assert_chain() {
    let addr = spawn_stub();
    let text = scenario_text(
        &addr,
        r#"
[[steps]]
type = "assert"
name = "sane"
exprs = ["total == 7", "fails == 0", "p99 > 0", "p99 <= 1000 * total"]
"#,
    );
    let path = PathBuf::from("scenarios/demo/stub_roundtrip.toml");
    let scenario = parse_scenario(&path, &text).expect("parses");
    let run = run_scenario(&scenario);
    assert!(
        run.raw_passed,
        "expected pass, findings: {:?}",
        run.findings.iter().map(|f| &f.detail).collect::<Vec<_>>()
    );
    assert_eq!(run.steps_run, 3);
    assert_eq!(run.vars.get_f64("total"), Some(7.0));
}

#[test]
fn failed_assertion_stops_the_scenario_with_operands() {
    let addr = spawn_stub();
    let text = scenario_text(
        &addr,
        r#"
[[steps]]
type = "assert"
name = "impossible"
exprs = ["total == 9999"]

[[steps]]
type = "sleep"
name = "never_reached"
secs = 0
"#,
    );
    let path = PathBuf::from("scenarios/demo/stub_roundtrip.toml");
    let scenario = parse_scenario(&path, &text).expect("parses");
    let run = run_scenario(&scenario);
    assert!(!run.raw_passed);
    assert_eq!(run.findings.len(), 1);
    let f = &run.findings[0];
    assert_eq!(f.kind, Kind::AssertionFailure);
    assert!(f.detail.contains("total=7"), "detail: {}", f.detail);
    // The step after the failed assert never ran.
    assert_eq!(run.steps_run, 2);
}

#[test]
fn transport_error_is_step_failure_with_status_zero() {
    // Nothing listens on this port (bind+drop reserves then frees it).
    let dead = {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().to_string()
    };
    let text = format!(
        r#"
[record]
suite = "rig"
dimension = "demo"
case = "stub_roundtrip"
subject = "transport failure surfaces"
kind = "e2e"
expected = "pass"

[env]
upstream = "{dead}"

[[steps]]
type = "http"
name = "unreachable"
method = "GET"
url = "http://{{{{upstream}}}}/x"
expect = {{ status = 200, timeout_ms = 500 }}
"#
    );
    let path = PathBuf::from("scenarios/demo/stub_roundtrip.toml");
    let scenario = parse_scenario(&path, &text).expect("parses");
    let run = run_scenario(&scenario);
    assert!(!run.raw_passed);
    assert_eq!(run.findings[0].kind, Kind::StepFailure);
    assert_eq!(run.findings[0].evidence["status"], 0);
}

#[test]
fn wait_until_recovers_and_records_duration() {
    let addr = spawn_stub();
    let text = format!(
        r#"
[record]
suite = "rig"
dimension = "demo"
case = "stub_roundtrip"
subject = "wait_until passes against a live stub"
kind = "e2e"
expected = "pass"

[env]
upstream = "{addr}"

[[steps]]
type = "wait_until"
name = "ready"
budget_secs = 5
interval_ms = 50
[steps.probe]
method = "GET"
url = "http://{{{{upstream}}}}/healthz"

[[steps]]
type = "assert"
name = "fast"
exprs = ["ready_recovered_secs < 5"]
"#
    );
    let path = PathBuf::from("scenarios/demo/stub_roundtrip.toml");
    let scenario = parse_scenario(&path, &text).expect("parses");
    let run = run_scenario(&scenario);
    assert!(
        run.raw_passed,
        "findings: {:?}",
        run.findings.iter().map(|f| &f.detail).collect::<Vec<_>>()
    );
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/tests/engine_e2e.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/tests/engine_e2e.rs` captured during rig
      standardization onto the codegen ladder.
```
