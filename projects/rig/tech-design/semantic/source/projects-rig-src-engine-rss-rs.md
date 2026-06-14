---
id: projects-rig-src-engine-rss-rs
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/src/engine/rss.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/rig/src/engine/rss.rs`, captured as a rust-source-unit (td_ast) item-tree
during rig standardization onto the codegen ladder.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `measure_rss`: sample a process's resident set size into a var.
//!
//! Source of the pid, in preference order: `pid_var` (a var captured from
//! vat export or an exec step), else `pgrep -n <process>` (racy; last
//! resort). RSS read via `ps -o rss=` (KB) — portable across macOS/Linux.

use serde_json::json;
use std::process::Command;

use crate::scenario::interp::VarStore;
use crate::scenario::step::MeasureRssStep;

pub fn execute(step: &MeasureRssStep, vars: &mut VarStore) -> Result<(), String> {
    let pid = resolve_pid(step, vars)?;
    let rss_kb = rss_kb_of(pid)?;
    for (var, key) in &step.capture {
        match key.as_str() {
            "rss_kb" => vars.set(var, json!(rss_kb)),
            other => return Err(format!("unknown measure_rss capture `{other}` (only rss_kb)")),
        }
    }
    Ok(())
}

fn resolve_pid(step: &MeasureRssStep, vars: &VarStore) -> Result<u32, String> {
    if let Some(pid_var) = &step.pid_var {
        let pid = vars
            .get_f64(pid_var)
            .ok_or_else(|| format!("pid_var `{pid_var}` is unset or non-numeric"))?;
        return Ok(pid as u32);
    }
    let Some(process) = &step.process else {
        return Err("measure_rss needs `pid_var` or `process`".into());
    };
    let out = Command::new("pgrep")
        .args(["-n", process])
        .output()
        .map_err(|e| format!("pgrep failed to run: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout);
    text.trim()
        .parse()
        .map_err(|_| format!("no live process matches `{process}`"))
}

fn rss_kb_of(pid: u32) -> Result<u64, String> {
    let out = Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .map_err(|e| format!("ps failed to run: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout);
    text.trim()
        .parse()
        .map_err(|_| format!("pid {pid} is not running (ps returned `{}`)", text.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn measures_own_process_via_pid_var() {
        let mut vars = VarStore::new();
        vars.set("self_pid", json!(std::process::id()));
        let step = MeasureRssStep {
            name: "rss".into(),
            process: None,
            pid_var: Some("self_pid".into()),
            capture: BTreeMap::from([("my_rss".to_string(), "rss_kb".to_string())]),
        };
        execute(&step, &mut vars).unwrap();
        assert!(vars.get_f64("my_rss").unwrap() > 0.0);
    }

    #[test]
    fn missing_pid_sources_is_error() {
        let step = MeasureRssStep {
            name: "rss".into(),
            process: None,
            pid_var: None,
            capture: BTreeMap::new(),
        };
        assert!(execute(&step, &mut VarStore::new()).is_err());
    }

    #[test]
    fn dead_pid_is_error() {
        let mut vars = VarStore::new();
        vars.set("pid", json!(999_999_999));
        let step = MeasureRssStep {
            name: "rss".into(),
            process: None,
            pid_var: Some("pid".into()),
            capture: BTreeMap::new(),
        };
        assert!(execute(&step, &mut vars).is_err());
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/src/engine/rss.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/src/engine/rss.rs` captured during rig
      standardization onto the codegen ladder.
```
