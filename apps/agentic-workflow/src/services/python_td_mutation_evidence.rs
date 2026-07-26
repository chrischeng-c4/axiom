//! Reproducible digest-bound evidence for Python TD mutation execution.
//!
//! @spec apps/agentic-workflow/tech-design/logic/aw-python-td-mutation-evidence.md#logic

use super::{
    python_td_mutation::{PythonTdMutant, PythonTdMutationDescriptor},
    python_td_mutation_runner::{
        MutationGateKind, MutationGateResult, MutationGateStatus, MutationRunResult,
        MutationVerdict, PythonTdNativeTarget,
    },
};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path};

pub const PYTHON_TD_MUTATION_EVIDENCE_SCHEMA: &str = "aw.python-td-mutation-evidence.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MutationEvidenceBindings {
    pub td_digest: String,
    pub ec_digest: String,
    pub source_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MutationGateEvidence {
    pub gate_id: String,
    pub kind: MutationGateKind,
    pub command: String,
    pub command_digest: String,
    pub status: MutationGateStatus,
    pub exit_code: i32,
    pub executed_tests: Option<usize>,
    pub compiled_target_marker: Option<String>,
    pub stdout: String,
    pub stdout_digest: String,
    pub stderr: String,
    pub stderr_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PythonTdMutationEvidence {
    pub schema_version: String,
    pub report_digest: String,
    pub bindings: MutationEvidenceBindings,
    pub mutant: PythonTdMutationDescriptor,
    pub mutated_semantic_digest: String,
    pub target: PythonTdNativeTarget,
    pub target_digest: String,
    pub gate_inventory_digest: String,
    pub gates: Vec<MutationGateEvidence>,
    pub verdict: MutationVerdict,
}

pub fn build_python_td_mutation_evidence(
    bindings: MutationEvidenceBindings,
    mutant: &PythonTdMutant,
    run: &MutationRunResult,
) -> Result<PythonTdMutationEvidence> {
    validate_digest("td", &bindings.td_digest)?;
    validate_digest("EC", &bindings.ec_digest)?;
    validate_digest("source", &bindings.source_digest)?;
    if run.mutant_id != mutant.descriptor.id {
        bail!(
            "mutation run `{}` does not belong to mutant `{}`",
            run.mutant_id,
            mutant.descriptor.id
        );
    }
    if run.target_digest.trim().is_empty() || run.gates.is_empty() {
        bail!("mutation run must contain a target digest and executed gates");
    }

    let gates = run.gates.iter().map(gate_evidence).collect::<Vec<_>>();
    let gate_inventory_digest = gate_inventory_digest(&gates)?;
    let mut evidence = PythonTdMutationEvidence {
        schema_version: PYTHON_TD_MUTATION_EVIDENCE_SCHEMA.to_string(),
        report_digest: String::new(),
        bindings,
        mutant: mutant.descriptor.clone(),
        mutated_semantic_digest: mutant.mutated_semantic_digest.clone(),
        target: run.target,
        target_digest: run.target_digest.clone(),
        gate_inventory_digest,
        gates,
        verdict: run.verdict,
    };
    evidence.report_digest = report_digest(&evidence)?;
    verify_python_td_mutation_evidence(&evidence, &evidence.bindings)?;
    Ok(evidence)
}

pub fn render_python_td_mutation_evidence(evidence: &PythonTdMutationEvidence) -> Result<Vec<u8>> {
    verify_python_td_mutation_evidence(evidence, &evidence.bindings)?;
    let mut bytes = serde_json::to_vec_pretty(evidence).context("serialize mutation evidence")?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn write_python_td_mutation_evidence(
    path: &Path,
    evidence: &PythonTdMutationEvidence,
) -> Result<()> {
    let bytes = render_python_td_mutation_evidence(evidence)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create mutation evidence directory {}", parent.display()))?;
    }
    fs::write(path, bytes).with_context(|| format!("write mutation evidence {}", path.display()))
}

pub fn read_python_td_mutation_evidence(
    path: &Path,
    current: &MutationEvidenceBindings,
) -> Result<PythonTdMutationEvidence> {
    let bytes =
        fs::read(path).with_context(|| format!("read mutation evidence {}", path.display()))?;
    let evidence: PythonTdMutationEvidence = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse mutation evidence {}", path.display()))?;
    verify_python_td_mutation_evidence(&evidence, current)?;
    Ok(evidence)
}

pub fn verify_python_td_mutation_evidence(
    evidence: &PythonTdMutationEvidence,
    current: &MutationEvidenceBindings,
) -> Result<()> {
    if evidence.schema_version != PYTHON_TD_MUTATION_EVIDENCE_SCHEMA {
        bail!(
            "unsupported mutation evidence schema `{}`",
            evidence.schema_version
        );
    }
    for (name, recorded, actual) in [
        ("TD", &evidence.bindings.td_digest, &current.td_digest),
        ("EC", &evidence.bindings.ec_digest, &current.ec_digest),
        (
            "source",
            &evidence.bindings.source_digest,
            &current.source_digest,
        ),
    ] {
        validate_digest(name, actual)?;
        if recorded != actual {
            bail!(
                "mutation evidence is stale: recorded {name} digest `{recorded}` does not match current `{actual}`"
            );
        }
    }
    if evidence.mutant.input_semantic_digest != evidence.bindings.td_digest {
        bail!("mutation evidence mutant input is not bound to the recorded TD digest");
    }
    if evidence.mutated_semantic_digest.trim().is_empty()
        || evidence.target_digest.trim().is_empty()
        || evidence.gates.is_empty()
    {
        bail!("mutation evidence is missing mutant, target, or gate results");
    }
    let mut gate_ids = BTreeSet::new();
    for gate in &evidence.gates {
        if !gate_ids.insert(gate.gate_id.as_str()) {
            bail!(
                "mutation evidence contains duplicate gate `{}`",
                gate.gate_id
            );
        }
        if gate.command_digest != digest(gate.command.as_bytes()) {
            bail!(
                "mutation evidence gate `{}` command digest mismatch",
                gate.gate_id
            );
        }
        if gate.stdout_digest != digest(gate.stdout.as_bytes()) {
            bail!(
                "mutation evidence gate `{}` stdout digest mismatch",
                gate.gate_id
            );
        }
        if gate.stderr_digest != digest(gate.stderr.as_bytes()) {
            bail!(
                "mutation evidence gate `{}` stderr digest mismatch",
                gate.gate_id
            );
        }
    }
    let expected_inventory = gate_inventory_digest(&evidence.gates)?;
    if evidence.gate_inventory_digest != expected_inventory {
        bail!("mutation evidence gate inventory digest mismatch: expected `{expected_inventory}`");
    }
    let expected_report = report_digest(evidence)?;
    if evidence.report_digest != expected_report {
        bail!("mutation evidence report digest mismatch: expected `{expected_report}`");
    }
    Ok(())
}

fn gate_evidence(result: &MutationGateResult) -> MutationGateEvidence {
    MutationGateEvidence {
        gate_id: result.gate_id.clone(),
        kind: result.kind,
        command: result.command.clone(),
        command_digest: digest(result.command.as_bytes()),
        status: result.status,
        exit_code: result.exit_code,
        executed_tests: result.executed_tests,
        compiled_target_marker: result.compiled_target_marker.clone(),
        stdout: result.stdout.clone(),
        stdout_digest: digest(result.stdout.as_bytes()),
        stderr: result.stderr.clone(),
        stderr_digest: digest(result.stderr.as_bytes()),
    }
}

fn gate_inventory_digest(gates: &[MutationGateEvidence]) -> Result<String> {
    let inventory = gates
        .iter()
        .map(|gate| {
            (
                gate.gate_id.as_str(),
                gate.kind,
                gate.command.as_str(),
                gate.compiled_target_marker.as_deref(),
            )
        })
        .collect::<Vec<_>>();
    Ok(digest(
        &serde_json::to_vec(&inventory).context("serialize mutation gate inventory")?,
    ))
}

fn report_digest(evidence: &PythonTdMutationEvidence) -> Result<String> {
    Ok(digest(
        &serde_json::to_vec(&(
            evidence.schema_version.as_str(),
            &evidence.bindings,
            &evidence.mutant,
            evidence.mutated_semantic_digest.as_str(),
            evidence.target,
            evidence.target_digest.as_str(),
            evidence.gate_inventory_digest.as_str(),
            &evidence.gates,
            evidence.verdict,
        ))
        .context("serialize canonical mutation evidence payload")?,
    ))
}

fn validate_digest(name: &str, value: &str) -> Result<()> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        bail!("mutation evidence {name} digest must use sha256");
    };
    if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("mutation evidence {name} digest is malformed");
    }
    Ok(())
}

fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
