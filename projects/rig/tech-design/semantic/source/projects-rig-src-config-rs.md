---
id: projects-rig-src-config-rs
capability_refs:
  - id: scenario-engine
    role: primary
    claim: scenario-step-dsl-execution
    coverage: partial
    rationale: "This source unit implements rig scenario configuration, case modeling, or execution behavior used by the scenario engine."
fill_sections: [overview, source, changes]
---

# Standardized projects/rig/src/config.rs

## Overview
<!-- type: overview lang: markdown -->

Rust source-unit TD for `projects/rig/src/config.rs`, captured during #39 rig traceability closure.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `rig.toml` launcher config — the agent-first knob home. rig's CLI stays
//! near-zero-args: everything that does NOT change per run (case dirs, pins,
//! the open-loop load schedule) lives here, so `rig test` is just `rig test`.
//!
//! Unknown keys are ignored, so launcher config coexists with an aw-managed
//! `AW-EC-TOOL` manifest block in the same `rig.toml` (aw owns `project` /
//! `scenarios_dir` / `source_contract`; rig reads `testpaths` / `pins` / `[load]`).

use serde::Deserialize;
use std::path::Path;

fn default_qps() -> u32 {
    200
}
fn default_workers() -> u32 {
    8
}
fn default_duration() -> u64 {
    30
}

/// The open-loop schedule injected into every `n>>1` (load) case. Lives in
/// config, never on the CLI — schedule is project policy, not a per-run knob.
#[derive(Debug, Clone, Deserialize)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-config-rs.md#source
pub struct LoadConfig {
    #[serde(default = "default_qps")]
    pub qps: u32,
    #[serde(default = "default_workers")]
    pub workers: u32,
    #[serde(default = "default_duration")]
    pub duration_secs: u64,
}

/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-config-rs.md#source
impl Default for LoadConfig {
    fn default() -> Self {
        Self {
            qps: default_qps(),
            workers: default_workers(),
            duration_secs: default_duration(),
        }
    }
}

/// Parsed `rig.toml` launcher config (the launcher sections only).
#[derive(Debug, Clone, Deserialize, Default)]
/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-config-rs.md#source
pub struct Config {
    /// Directories of lifecycle case TOMLs to discover.
    #[serde(default)]
    pub testpaths: Vec<String>,
    /// Pin directory for load-metric gating.
    #[serde(default)]
    pub pins: Option<String>,
    /// Open-loop schedule for `n>>1` cases.
    #[serde(default)]
    pub load: LoadConfig,
}

/// @spec projects/rig/tech-design/semantic/source/projects-rig-src-config-rs.md#source
impl Config {
    /// Load `rig.toml` from `dir`. Absent file or parse error => defaults.
    /// Unknown keys (an aw-generated `AW-EC-TOOL` block) are ignored.
    pub fn load_from(dir: &Path) -> Config {
        std::fs::read_to_string(dir.join("rig.toml"))
            .ok()
            .and_then(|t| toml::from_str(&t).ok())
            .unwrap_or_default()
    }

    /// The case directories to discover: explicit `testpaths`, else `fallback`.
    pub fn case_dirs(&self, fallback: &str) -> Vec<String> {
        if self.testpaths.is_empty() {
            vec![fallback.to_string()]
        } else {
            self.testpaths.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_when_absent() {
        let c = Config::load_from(Path::new("/nonexistent-xyz"));
        assert!(c.testpaths.is_empty());
        assert_eq!(c.load.qps, 200);
        assert_eq!(c.case_dirs("."), vec![".".to_string()]);
    }

    #[test]
    fn reads_launcher_sections_and_ignores_aw_block() {
        let dir = std::env::temp_dir().join(format!("rig-cfg-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("rig.toml"),
            // aw-managed keys + launcher sections coexist:
            "project = \"lumen\"\nscenarios_dir = \"x\"\ntestpaths = [\"tests/rig/cases\"]\npins = \"tests/rig/config/pins\"\n[load]\nqps = 600\nworkers = 16\n",
        )
        .unwrap();
        let c = Config::load_from(&dir);
        assert_eq!(c.testpaths, vec!["tests/rig/cases".to_string()]);
        assert_eq!(c.pins.as_deref(), Some("tests/rig/config/pins"));
        assert_eq!(c.load.qps, 600);
        assert_eq!(c.load.workers, 16);
        assert_eq!(c.load.duration_secs, 30); // default kept
        let _ = std::fs::remove_dir_all(&dir);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/rig/src/config.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/rig/src/config.rs` captured during #39 rig standardization.
```
