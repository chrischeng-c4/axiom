// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-commands.md#schema
// CODEGEN-BEGIN
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

- If the project has `vat.toml`, prefer plain `vat run`.
- Use `vat run <runner-id>` only when you need a non-default runner.
- If you only need one ad-hoc command, use `vat run -- <command>`.
- `vat run` prints sparse JSONL checkpoints; the final line has
  `"type":"result"`.
- After a retained run, inspect `vat state <id>`, `vat diff <id>`, and
  `vat logs <id> [runner|service-id]`.
- Use `vat --help` for flag syntax and `vat <command> --help` for command flags.

## vat.toml Contract

```toml
version = 1
default_runner = "e2e"

[workspace]
base = "."
workdir = "."
keep = "failed" # failed | always | never

[[services]]
id = "pg"
preset = "postgres"
seed = ["schema.sql", "fixtures.sql"]
export = { DATABASE_URL = "DATABASE_URL" }

[[runners]]
id = "e2e"
requires = ["pg"]
cmd = ["pnpm", "run", "test:e2e"]
artifacts = ["test-results/**", "playwright-report/**"]
```

## Command Patterns

- `vat run`: select the default runner, prepare or clone service images, start
  required services, wait for readiness, run the runner, capture evidence, stop
  services, and return the runner exit code.
- `vat run e2e`: explicitly run the `e2e` runner.
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
// CODEGEN-END
