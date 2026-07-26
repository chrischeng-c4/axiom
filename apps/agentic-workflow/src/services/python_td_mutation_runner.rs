//! Isolated re-emission and complete gate execution for typed Python TD mutants.
//!
//! @spec apps/agentic-workflow/tech-design/logic/aw-python-td-mutation-execution.md#logic

use super::{
    python_td_mutation::{PythonTdMutant, PythonTdMutationScope},
    python_td_rust_target::emit_python_td_rust_target,
    python_td_target::emit_python_td_target,
    python_td_typescript_target::emit_python_td_typescript_target,
};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

pub const PYTHON_TD_MUTATION_RUN_SCHEMA: &str = "aw.python-td-mutation-run.v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PythonTdNativeTarget {
    Python,
    Rust,
    TypeScript,
}

impl PythonTdNativeTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Python => "python",
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MutationGateKind {
    Unit,
    ExternalContract,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MutationGate {
    pub id: String,
    pub kind: MutationGateKind,
    pub command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compiled_target_marker: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MutationRunOptions {
    pub timeout: Duration,
}

impl Default for MutationRunOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30 * 60),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MutationGateStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MutationGateResult {
    pub gate_id: String,
    pub kind: MutationGateKind,
    pub command: String,
    pub status: MutationGateStatus,
    pub exit_code: i32,
    pub executed_tests: Option<usize>,
    pub compiled_target_marker: Option<String>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MutationVerdict {
    Killed,
    Survived,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MutationRunResult {
    pub schema_version: String,
    pub mutant_id: String,
    pub target: PythonTdNativeTarget,
    pub target_digest: String,
    pub verdict: MutationVerdict,
    pub gates: Vec<MutationGateResult>,
}

pub fn run_python_td_mutant(
    mutant: &PythonTdMutant,
    target: PythonTdNativeTarget,
    output_root: &Path,
    gates: &[MutationGate],
    options: &MutationRunOptions,
) -> Result<MutationRunResult> {
    validate_scope(mutant, target)?;
    validate_gates(gates, target)?;
    if options.timeout.is_zero() {
        bail!("Python TD mutation gate timeout must be greater than zero");
    }
    if output_root.exists() {
        bail!(
            "Python TD mutation output root must not exist before isolated re-emission: {}",
            output_root.display()
        );
    }

    let target_digest = match target {
        PythonTdNativeTarget::Python => emit_python_td_target(&mutant.ir, output_root)?.digest,
        PythonTdNativeTarget::Rust => emit_python_td_rust_target(&mutant.ir, output_root)?.digest,
        PythonTdNativeTarget::TypeScript => {
            emit_python_td_typescript_target(&mutant.ir, output_root)?.digest
        }
    };

    let mut results = Vec::with_capacity(gates.len());
    for gate in gates {
        results.push(run_gate(
            mutant,
            target,
            output_root,
            gate,
            options.timeout,
        )?);
    }
    let verdict = if results
        .iter()
        .any(|result| result.status == MutationGateStatus::Failed)
    {
        MutationVerdict::Killed
    } else {
        MutationVerdict::Survived
    };
    Ok(MutationRunResult {
        schema_version: PYTHON_TD_MUTATION_RUN_SCHEMA.to_string(),
        mutant_id: mutant.descriptor.id.clone(),
        target,
        target_digest,
        verdict,
        gates: results,
    })
}

fn validate_scope(mutant: &PythonTdMutant, target: PythonTdNativeTarget) -> Result<()> {
    let expected = match mutant.descriptor.scope {
        PythonTdMutationScope::Semantic => return Ok(()),
        PythonTdMutationScope::Python => PythonTdNativeTarget::Python,
        PythonTdMutationScope::Rust => PythonTdNativeTarget::Rust,
        PythonTdMutationScope::TypeScript => PythonTdNativeTarget::TypeScript,
    };
    if expected != target {
        bail!(
            "Python TD mutant `{}` is scoped to `{}` and cannot run against `{}`",
            mutant.descriptor.id,
            expected.as_str(),
            target.as_str()
        );
    }
    Ok(())
}

fn validate_gates(gates: &[MutationGate], target: PythonTdNativeTarget) -> Result<()> {
    if gates.is_empty() {
        bail!("Python TD mutation run requires configured unit and external-contract gates");
    }
    let mut ids = BTreeSet::new();
    let mut unit_count = 0;
    let mut ec_count = 0;
    for gate in gates {
        if gate.id.trim().is_empty() || gate.command.trim().is_empty() {
            bail!("Python TD mutation gates require non-empty ids and commands");
        }
        if !ids.insert(gate.id.as_str()) {
            bail!("duplicate Python TD mutation gate id `{}`", gate.id);
        }
        match gate.kind {
            MutationGateKind::Unit => {
                unit_count += 1;
                if gate
                    .compiled_target_marker
                    .as_ref()
                    .is_none_or(|marker| marker.trim().is_empty())
                {
                    bail!(
                        "unit mutation gate `{}` must declare a compiled-target marker for `{}`",
                        gate.id,
                        target.as_str()
                    );
                }
            }
            MutationGateKind::ExternalContract => ec_count += 1,
        }
    }
    if unit_count == 0 || ec_count == 0 {
        bail!("Python TD mutation run requires at least one unit and one external-contract gate");
    }
    Ok(())
}

fn run_gate(
    mutant: &PythonTdMutant,
    target: PythonTdNativeTarget,
    output_root: &Path,
    gate: &MutationGate,
    timeout: Duration,
) -> Result<MutationGateResult> {
    let stdout_file = tempfile::NamedTempFile::new()
        .with_context(|| format!("create stdout capture for mutation gate `{}`", gate.id))?;
    let stderr_file = tempfile::NamedTempFile::new()
        .with_context(|| format!("create stderr capture for mutation gate `{}`", gate.id))?;
    let stdout = stdout_file.reopen()?;
    let stderr = stderr_file.reopen()?;
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&gate.command)
        .current_dir(output_root)
        .env("AW_MUTATION_ID", &mutant.descriptor.id)
        .env("AW_MUTATION_TARGET", target.as_str())
        .env("AW_MUTATION_TARGET_ROOT", output_root)
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .with_context(|| format!("execute mutation gate `{}`", gate.id))?;
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .with_context(|| format!("poll mutation gate `{}`", gate.id))?
        {
            break status;
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            bail!(
                "Python TD mutation gate `{}` timed out after {}s",
                gate.id,
                timeout.as_secs()
            );
        }
        thread::sleep(Duration::from_millis(10));
    };
    let stdout = fs::read_to_string(stdout_file.path())
        .with_context(|| format!("read stdout for mutation gate `{}`", gate.id))?;
    let stderr = fs::read_to_string(stderr_file.path())
        .with_context(|| format!("read stderr for mutation gate `{}`", gate.id))?;
    let combined = format!("{stdout}\n{stderr}");
    let executed_tests = parse_executed_test_count(&combined);
    let requires_count =
        gate.kind == MutationGateKind::Unit || recognized_test_command(&gate.command);
    if status.success() && requires_count && executed_tests.unwrap_or(0) == 0 {
        bail!(
            "Python TD mutation gate `{}` passed but executed zero tests",
            gate.id
        );
    }
    if status.success() {
        if let Some(marker) = &gate.compiled_target_marker {
            if !combined.contains(marker) {
                bail!(
                    "Python TD mutation gate `{}` passed without compiled-target marker `{marker}`",
                    gate.id
                );
            }
        }
    }
    Ok(MutationGateResult {
        gate_id: gate.id.clone(),
        kind: gate.kind,
        command: gate.command.clone(),
        status: if status.success() {
            MutationGateStatus::Passed
        } else {
            MutationGateStatus::Failed
        },
        exit_code: status.code().unwrap_or(-1),
        executed_tests,
        compiled_target_marker: gate.compiled_target_marker.clone(),
        stdout,
        stderr,
    })
}

fn recognized_test_command(command: &str) -> bool {
    [
        "cargo test",
        "unittest",
        "pytest",
        "node --test",
        "npm test",
        "pnpm test",
    ]
    .iter()
    .any(|needle| command.contains(needle))
}

fn parse_executed_test_count(output: &str) -> Option<usize> {
    let mut total = 0usize;
    let mut saw = false;
    for line in output.lines() {
        let line = line.trim();
        let parsed = line
            .strip_prefix("running ")
            .and_then(|rest| {
                rest.strip_suffix(" tests")
                    .or_else(|| rest.strip_suffix(" test"))
            })
            .or_else(|| {
                line.strip_prefix("Ran ")
                    .and_then(|rest| rest.split_whitespace().next())
            })
            .or_else(|| line.strip_prefix("# tests "));
        if let Some(count) = parsed.and_then(|count| count.trim().parse::<usize>().ok()) {
            total = total.saturating_add(count);
            saw = true;
        }
    }
    saw.then_some(total)
}
