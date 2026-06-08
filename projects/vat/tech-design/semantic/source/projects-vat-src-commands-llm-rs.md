---
id: vat-source-projects-vat-src-commands-llm-rs
summary: Source replay payload for projects/vat/src/commands/llm.rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: agent-legible-state-and-diff-surface
    claim: agent-legible-state-and-diff-surface
    coverage: partial
    rationale: "This source replay TD preserves the agent-facing vat llm usage guide."
---

# Source TD: projects/vat/src/commands/llm.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/commands/llm.rs` generated from AST during source replay standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `exec` | projects/vat/src/commands/llm.rs | function | pub | 68 | exec() -> Result<ExitCode> |
## Source
<!-- type: source lang: rust -->

`````rust
//! `vat llm` — compact agent-facing usage contract.

use std::process::ExitCode;

use anyhow::Result;

/// Stable guide text intended for LLM/tool agents.
/// @spec projects/vat/tech-design/logic/llm-agent-usage-guide.md#cli
const GUIDE: &str = r#"# vat LLM Guide

vat is a local, ephemeral agent test runner. Use it to prepare a real local
workspace, run one command or one named vat.toml runner, and inspect structured
evidence afterward.

## First Choice

- If the project has `vat.toml`, prefer `vat run <runner-id> --json`.
- If you only need one ad-hoc command, use `vat run -- <command>`.
- After a retained run, inspect `vat state <id>`, `vat diff <id>`, and
  `vat logs <id> [runner|service-id]`.
- Use `vat --help` for flag syntax and `vat <command> --help` for command flags.

## vat.toml Contract

```toml
version = 1

[workspace]
base = "."
workdir = "."
keep = "failed" # failed | always | never

[[services]]
id = "web"
cmd = ["pnpm", "run", "dev", "--host", "127.0.0.1"]
ready_http = "http://127.0.0.1:5173/"
timeout_s = 60

[[runners]]
id = "e2e"
requires = ["web"]
cmd = ["pnpm", "run", "test:e2e"]
artifacts = ["test-results/**", "playwright-report/**"]
```

## Command Patterns

- `vat run e2e --json`: create a copy-on-write workspace, run setup, start
  required services, wait for readiness, run the runner, capture evidence, stop
  services, and return the runner exit code.
- `vat run -- cargo test -p app`: run one direct command without requiring
  vat.toml; the child exit code is forwarded.
- `vat logs <id> runner`: print retained runner stdout/stderr.
- `vat logs <id> <service-id>`: print retained service stdout/stderr.
- `vat state <id>`: read the agent-legible JSON state.
- `vat diff <id> --json`: read filesystem changes vs. the vat base.

## Retention

Default `keep = "failed"` means successful configured runs clean up after
emitting JSON, while failed runs keep workspace state and logs for inspection.

## Boundaries

- vat is not Docker, OCI, Compose, a Linux runtime, a VM, a daemon, or a
  long-lived process manager.
- Services in `vat.toml` are run-scoped dependencies of one runner invocation.
- vat does not schedule production work or manage restart policy.
"#;

/// @spec projects/vat/tech-design/logic/llm-agent-usage-guide.md#cli
pub fn exec() -> Result<ExitCode> {
    print!("{GUIDE}");
    Ok(ExitCode::SUCCESS)
}
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/vat/src/commands/llm.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/vat/tech-design/semantic/vat-commands.md#schema.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-vat-src-commands-llm-rs-source-replay-superseded>"
```
