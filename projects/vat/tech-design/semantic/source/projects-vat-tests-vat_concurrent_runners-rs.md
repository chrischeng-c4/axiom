---
id: projects-vat-tests-vat_concurrent_runners-rs
fill_sections: [overview, source, changes]
---

# Standardized projects/vat/tests/vat_concurrent_runners.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/tests/vat_concurrent_runners.rs`, captured as a rust-source-unit (td_ast) item-tree
during vat standardization onto the codegen ladder.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `vat run a b` — concurrent runners share one workspace + service union,
//! run side by side, and fold into one result with worst-wins exit.

use std::process::Command;
use std::time::Instant;

use serde_json::Value;

fn vat_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vat")
}

fn jsonl(stdout: &[u8]) -> Vec<Value> {
    String::from_utf8_lossy(stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

fn result_event(events: &[Value]) -> &Value {
    events
        .iter()
        .find(|event| event["type"] == "result")
        .expect("missing result event")
}

fn write_config(dir: &std::path::Path) {
    std::fs::write(
        dir.join("vat.toml"),
        r#"
version = 1
name = "concurrent"

[workspace]
base = "."
keep = "never"

[[runners]]
id = "a"
cmd = ["/bin/sh", "-c", "echo from-a; sleep 1"]

[[runners]]
id = "b"
cmd = ["/bin/sh", "-c", "echo from-b; sleep 1"]

[[runners]]
id = "bad"
cmd = ["/bin/sh", "-c", "exit 7"]
"#,
    )
    .unwrap();
}

#[test]
fn concurrent_runners_overlap_and_report_each() {
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    write_config(project.path());

    let started = Instant::now();
    let output = Command::new(vat_bin())
        .args(["run", "a", "b"])
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .output()
        .unwrap();
    let wall = started.elapsed();
    assert!(output.status.success(), "vat run a b failed: {output:?}");

    let events = jsonl(&output.stdout);
    let result = result_event(&events);
    assert_eq!(result["ok"], true);
    assert_eq!(result["exit_code"], 0);
    assert_eq!(result["runner"], "a+b");
    let runners = result["runners"].as_array().unwrap();
    assert_eq!(runners.len(), 2);
    assert!(runners.iter().all(|r| r["exit_code"] == 0));

    // Two 1s runners side by side: well under the 2s a sequential run needs.
    // Generous bound (clone + spawn overhead) while still proving overlap.
    assert!(
        wall.as_secs_f64() < 1.9,
        "runners did not overlap: wall = {wall:?}"
    );

    // Both runner subtrees emitted started+exited events.
    for id in ["a", "b"] {
        assert!(events
            .iter()
            .any(|e| e["type"] == "runner" && e["id"] == id && e["state"] == "started"));
        assert!(events
            .iter()
            .any(|e| e["type"] == "runner" && e["id"] == id && e["state"] == "exited"));
    }
}

#[test]
fn worst_exit_code_wins_across_concurrent_runners() {
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    write_config(project.path());

    let output = Command::new(vat_bin())
        .args(["run", "a", "bad"])
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(7), "worst exit code forwarded");

    let events = jsonl(&output.stdout);
    let result = result_event(&events);
    assert_eq!(result["ok"], false);
    assert_eq!(result["exit_code"], 7);
    let runners = result["runners"].as_array().unwrap();
    let bad = runners.iter().find(|r| r["id"] == "bad").unwrap();
    assert_eq!(bad["exit_code"], 7);
    let a = runners.iter().find(|r| r["id"] == "a").unwrap();
    assert_eq!(a["exit_code"], 0);
}

#[test]
fn duplicate_runner_ids_are_rejected() {
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    write_config(project.path());

    let output = Command::new(vat_bin())
        .args(["run", "a", "a"])
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("listed twice"), "stderr: {stderr}");
}

#[test]
fn single_runner_keeps_legacy_log_names_and_result_shape() {
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    write_config(project.path());

    let output = Command::new(vat_bin())
        .args(["run", "a"])
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let events = jsonl(&output.stdout);
    let result = result_event(&events);
    assert_eq!(result["runner"], "a");
    let runners = result["runners"].as_array().unwrap();
    assert_eq!(runners.len(), 1);
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/tests/vat_concurrent_runners.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/tests/vat_concurrent_runners.rs` captured during vat
      standardization.
```
