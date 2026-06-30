---
id: vat-source-projects-vat-src-commands-logs-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/src/commands/logs.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/src/commands/logs.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/commands/logs.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `exec` | projects/vat/src/commands/logs.rs | function | pub | 12 | exec(id: String, source: Option<String>) -> Result<ExitCode> |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! `vat logs` — print captured logs from a vat.toml runner invocation.

use std::process::ExitCode;

use anyhow::{bail, Context, Result};

use crate::store;

/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#cli
pub fn exec(id: String, source: Option<String>) -> Result<ExitCode> {
    let vat = store::load(&id)?;
    let Some(test_run) = vat.meta.test_run else {
        bail!("vat {id} has no vat.toml runner evidence");
    };

    // Concurrent sets fill `runners`; legacy metadata only has `runner`.
    let runner_records: Vec<_> = if test_run.runners.is_empty() {
        test_run.runner.iter().cloned().collect()
    } else {
        test_run.runners.clone()
    };
    match source.as_deref() {
        Some("runner") => {
            for runner in &runner_records {
                print_pair(&runner.stdout_log, &runner.stderr_log)?;
            }
        }
        Some(source_id) => {
            if let Some(service) = test_run.services.iter().find(|s| s.id == source_id) {
                print_pair(&service.stdout_log, &service.stderr_log)?;
            } else if let Some(runner) = runner_records.iter().find(|r| r.id == source_id) {
                print_pair(&runner.stdout_log, &runner.stderr_log)?;
            } else {
                bail!("no log source `{source_id}` in vat {id}");
            }
        }
        None => {
            for service in &test_run.services {
                println!("== service:{} stdout ==", service.id);
                print_file(&service.stdout_log)?;
                println!("== service:{} stderr ==", service.id);
                print_file(&service.stderr_log)?;
            }
            for runner in &runner_records {
                println!("== runner:{} stdout ==", runner.id);
                print_file(&runner.stdout_log)?;
                println!("== runner:{} stderr ==", runner.id);
                print_file(&runner.stderr_log)?;
            }
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn print_pair(stdout: &str, stderr: &str) -> Result<()> {
    print_file(stdout)?;
    print_file(stderr)
}

fn print_file(path: &str) -> Result<()> {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            print!("{content}");
            Ok(())
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err).with_context(|| format!("read log {path}")),
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/commands/logs.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/src/commands/logs.rs` captured during #39 vat standardization.
```
