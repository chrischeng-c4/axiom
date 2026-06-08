// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-commands.md#schema
// CODEGEN-BEGIN
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

    match source.as_deref() {
        Some("runner") => {
            if let Some(runner) = test_run.runner {
                print_pair(&runner.stdout_log, &runner.stderr_log)?;
            }
        }
        Some(service_id) => {
            let service = test_run
                .services
                .iter()
                .find(|s| s.id == service_id)
                .with_context(|| format!("no log source `{service_id}` in vat {id}"))?;
            print_pair(&service.stdout_log, &service.stderr_log)?;
        }
        None => {
            for service in &test_run.services {
                println!("== service:{} stdout ==", service.id);
                print_file(&service.stdout_log)?;
                println!("== service:{} stderr ==", service.id);
                print_file(&service.stderr_log)?;
            }
            if let Some(runner) = test_run.runner {
                println!("== runner stdout ==");
                print_file(&runner.stdout_log)?;
                println!("== runner stderr ==");
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
// CODEGEN-END
