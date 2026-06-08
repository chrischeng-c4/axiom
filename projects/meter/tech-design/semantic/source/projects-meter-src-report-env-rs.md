---
id: projects-meter-src-report-env-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: json-default-report-envelope-and-findings
    claim: json-default-report-envelope-and-findings
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/report/env.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/report/env.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `detect` | projects/meter/src/report/env.rs | function | pub | 18 | detect() -> EnvBlock |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/src/report/env.rs -->
````rust
//! [`EnvBlock::detect`] — side-effect-free environment detection surfaced in
//! every report.
//!
//! Detection only PROBES (read-only `--version` invocations and `cfg!` checks);
//! it never builds, runs, or mutates anything. Tool absence is reported, never
//! fatal.

use std::process::Command;

use super::envelope::EnvBlock;

impl EnvBlock {
    /// Detect the current environment. Read-only: probes tool presence via
    /// `--version` invocations and compile-time `cfg!` for the sampler backend.
    pub fn detect() -> EnvBlock {
        let os = std::env::consts::OS.to_string();
        let arch = std::env::consts::ARCH.to_string();

        let nextest_present = probe(&["nextest", "--version"]);
        let rustc_version = rustc_version();
        let sampler_backend = sampler_backend().to_string();

        EnvBlock {
            os,
            arch,
            nextest_present,
            sampler_backend,
            rustc_version,
            note: String::new(),
        }
    }
}

/// Probe whether `cargo <args>` succeeds (read-only `--version`-style probe).
fn probe(args: &[&str]) -> bool {
    Command::new("cargo")
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// `rustc --version` short string, if available.
fn rustc_version() -> Option<String> {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

/// The platform stack sampler backend (compile-time; no spawn).
fn sampler_backend() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos-sample"
    } else if cfg!(target_os = "linux") {
        "linux-perf"
    } else {
        "none"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_populates_os_and_arch() {
        let env = EnvBlock::detect();
        assert!(!env.os.is_empty());
        assert!(!env.arch.is_empty());
    }

    #[test]
    fn sampler_backend_is_known() {
        let b = sampler_backend();
        assert!(b == "macos-sample" || b == "linux-perf" || b == "none");
    }

    #[test]
    fn note_defaults_empty() {
        let env = EnvBlock::detect();
        assert!(env.note.is_empty());
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/report/env.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/report/env.rs` captured during meter full-codegen standardization.
```
