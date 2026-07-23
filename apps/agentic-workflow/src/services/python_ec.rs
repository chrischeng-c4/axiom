// HANDWRITE-BEGIN gap="missing-generator:python-ec-inventory" tracker="#2293" reason="Python EC projects are authored directly; this adapter validates their declarative inventory without generating Python source."
//! Hand-authored Python external-contract inventory discovery.
//!
//! Python EC projects remain ordinary Python projects.  The small
//! `pyproject.toml` table below is only the inventory boundary AW needs to
//! locate and structurally validate those hand-authored cases; it is neither a
//! test scaffold nor a Python specification framework.
//!
//! @spec apps/agentic-workflow/tech-design/core/logic/python-ec-inventory-check.md#logic

use crate::services::python_artifact::{discover_python_artifact_project, PythonArtifactInput};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
};

pub const PYTHON_EC_PROTOCOL: &str = "aw.python-ec.v1";

/// A normalized hand-authored Python EC case declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonEcCase {
    pub id: String,
    pub artifact_id: String,
    pub capability_id: String,
    pub use_case_id: String,
    pub dimension: String,
    pub applicability: String,
    pub test_path: String,
    pub promise: String,
    pub oracle: String,
    pub threshold: Option<String>,
    pub target: String,
    pub command: String,
    pub evidence_paths: Vec<String>,
    pub known_failure: Option<PythonEcKnownFailure>,
}

/// A deliberate EC-first red. It is an explicit contract exception, never a
/// blanket ignore: the case id supplies identity and both the reason and the
/// expected failure characteristic are required.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PythonEcKnownFailure {
    pub reason: String,
    pub expected: String,
}

/// The structural inventory read directly from an EC Python project's
/// `pyproject.toml`. Findings deliberately accumulate so authors can repair
/// all local inventory defects in one edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonEcInventory {
    pub inventory_path: PathBuf,
    pub source_digest: String,
    pub dependency_lock_digest: String,
    pub author: String,
    pub input_files: Vec<PythonArtifactInput>,
    pub efficiency_policy: String,
    pub cases: Vec<PythonEcCase>,
    pub findings: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PyprojectDocument {
    tool: PyprojectTool,
}

#[derive(Debug, Deserialize)]
struct PyprojectTool {
    aw: PyprojectAw,
}

#[derive(Debug, Deserialize)]
struct PyprojectAw {
    #[serde(rename = "python-ec")]
    python_ec: PythonEcConfig,
}

#[derive(Debug, Deserialize)]
struct PythonEcConfig {
    protocol: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    efficiency_policy: String,
    #[serde(default)]
    cases: Vec<PythonEcCaseConfig>,
}

#[derive(Debug, Default, Deserialize)]
struct PythonEcCaseConfig {
    #[serde(default)]
    id: String,
    #[serde(default)]
    artifact_id: String,
    #[serde(default)]
    capability_id: String,
    #[serde(default)]
    use_case_id: String,
    #[serde(default)]
    dimension: String,
    #[serde(default)]
    applicability: String,
    #[serde(default)]
    test_path: String,
    #[serde(default)]
    promise: String,
    #[serde(default)]
    oracle: String,
    #[serde(default)]
    threshold: Option<String>,
    #[serde(default)]
    target: String,
    #[serde(default)]
    command: String,
    #[serde(default)]
    evidence_paths: Vec<String>,
    #[serde(default)]
    known_failure: Option<PythonEcKnownFailure>,
}

/// Discover a Python-v1 EC inventory without importing or executing any EC
/// module. The shared artifact-project discovery proves this is a declared
/// CPython project before this adapter reads its EC-specific metadata.
pub fn discover_python_ec_inventory(ec_root: &Path) -> Result<PythonEcInventory> {
    let artifact = discover_python_artifact_project(ec_root)?;
    let inventory_path = artifact.root().join("pyproject.toml");
    let content = fs::read_to_string(&inventory_path)
        .with_context(|| format!("read Python EC inventory {}", inventory_path.display()))?;
    let document: PyprojectDocument = toml::from_str(&content)
        .with_context(|| format!("parse Python EC inventory {}", inventory_path.display()))?;
    let config = document.tool.aw.python_ec;

    let mut findings = Vec::new();
    if config.protocol != PYTHON_EC_PROTOCOL {
        findings.push(format!(
            "Python EC protocol `{}` is unsupported; set [tool.aw.python-ec].protocol = `{PYTHON_EC_PROTOCOL}`",
            config.protocol
        ));
    }
    if config.cases.is_empty() {
        findings.push(
            "Python EC inventory declares no cases; add at least one [[tool.aw.python-ec.cases]] entry"
                .to_string(),
        );
    }
    let author = config.author.trim().to_string();
    if author.is_empty() {
        findings.push(
            "Python EC inventory is missing `author`; declare the identity that authored this contract bundle so independent agent review can reject self-review"
                .to_string(),
        );
    }
    let efficiency_policy = config.efficiency_policy.trim().to_string();
    if !efficiency_policy.is_empty()
        && !matches!(
            efficiency_policy.as_str(),
            "required" | "optional" | "not-applicable"
        )
    {
        findings.push(format!(
            "Python EC efficiency_policy `{efficiency_policy}` is invalid; expected required|optional|not-applicable"
        ));
    }

    let mut case_ids = BTreeSet::new();
    let mut cases = Vec::with_capacity(config.cases.len());
    for (index, raw) in config.cases.into_iter().enumerate() {
        let position = index + 1;
        let case = PythonEcCase {
            id: raw.id.trim().to_string(),
            artifact_id: raw.artifact_id.trim().to_string(),
            capability_id: raw.capability_id.trim().to_string(),
            use_case_id: raw.use_case_id.trim().to_string(),
            dimension: raw.dimension.trim().to_string(),
            applicability: raw.applicability.trim().to_string(),
            test_path: raw.test_path.trim().to_string(),
            promise: raw.promise.trim().to_string(),
            oracle: raw.oracle.trim().to_string(),
            threshold: raw.threshold.map(|value| value.trim().to_string()),
            target: raw.target.trim().to_string(),
            command: raw.command.trim().to_string(),
            evidence_paths: raw
                .evidence_paths
                .into_iter()
                .map(|path| path.trim().to_string())
                .collect(),
            known_failure: raw.known_failure.map(|failure| PythonEcKnownFailure {
                reason: failure.reason.trim().to_string(),
                expected: failure.expected.trim().to_string(),
            }),
        };
        let label = if case.id.is_empty() {
            format!("Python EC case #{position}")
        } else {
            format!("Python EC case `{}`", case.id)
        };

        validate_stable_id(&case.id, "id", &label, &mut findings);
        validate_artifact_id(&case.artifact_id, &label, &mut findings);
        validate_stable_id(&case.capability_id, "capability_id", &label, &mut findings);
        validate_stable_id(&case.use_case_id, "use_case_id", &label, &mut findings);
        if !case.id.is_empty() && !case_ids.insert(case.id.clone()) {
            findings.push(format!(
                "Python EC inventory contains duplicate case id `{}`; use one stable id per capability/use-case/dimension case",
                case.id
            ));
        }
        validate_dimension_and_applicability(&case, &label, &mut findings);
        validate_test_path(artifact.root(), &case.test_path, &label, &mut findings);
        validate_execution_contract(&case, &label, &mut findings);
        validate_known_failure(&case, &label, &mut findings);
        cases.push(case);
    }
    cases.sort_by(|left, right| left.id.cmp(&right.id));
    findings.sort();
    findings.dedup();

    Ok(PythonEcInventory {
        inventory_path,
        source_digest: artifact.source_digest().to_string(),
        dependency_lock_digest: artifact.dependency_lock_digest().to_string(),
        author,
        input_files: artifact.input_files().to_vec(),
        efficiency_policy,
        cases,
        findings,
    })
}

fn validate_known_failure(case: &PythonEcCase, label: &str, findings: &mut Vec<String>) {
    let Some(failure) = case.known_failure.as_ref() else {
        return;
    };
    if failure.reason.is_empty() {
        findings.push(format!("{label} known_failure is missing `reason`"));
    }
    if failure.expected.is_empty() {
        findings.push(format!("{label} known_failure is missing `expected`; state the failure characteristic that makes the red intentional"));
    }
    if !matches!(case.dimension.as_str(), "behavior" | "security") {
        findings.push(format!("{label} known_failure is only allowed for behavior or security during EC-first/TD verification"));
    }
}

fn validate_execution_contract(case: &PythonEcCase, label: &str, findings: &mut Vec<String>) {
    for (field, value) in [
        ("promise", case.promise.as_str()),
        ("oracle", case.oracle.as_str()),
        ("target", case.target.as_str()),
        ("command", case.command.as_str()),
    ] {
        if value.is_empty() {
            findings.push(format!(
                "{label} is missing `{field}`; staged verification requires an explicit external contract"
            ));
        }
    }
    if !case.target.is_empty()
        && !matches!(
            case.target.as_str(),
            "python" | "rust" | "typescript" | "javascript"
        )
    {
        findings.push(format!(
            "{label} target `{}` is invalid; expected python|rust|typescript|javascript",
            case.target
        ));
    }
    if matches!(case.dimension.as_str(), "stability" | "efficiency")
        && case
            .threshold
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .is_empty()
    {
        findings.push(format!(
            "{label} is missing `threshold`; {} verification must declare its measurable limit",
            case.dimension
        ));
    }
    if case.evidence_paths.is_empty() {
        findings.push(format!(
            "{label} is missing `evidence_paths`; every staged result needs external evidence"
        ));
    }
    let mut seen = BTreeSet::new();
    for path in &case.evidence_paths {
        let valid = !path.is_empty()
            && !Path::new(path).is_absolute()
            && Path::new(path)
                .components()
                .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
            && Path::new(path).starts_with("evidence");
        if !valid {
            findings.push(format!(
                "{label} evidence path `{path}` must be a safe project-relative evidence/* path"
            ));
        } else if !seen.insert(path) {
            findings.push(format!("{label} repeats evidence path `{path}`"));
        }
    }
    if !case.test_path.is_empty() && case.command.contains(&case.test_path) {
        findings.push(format!(
            "{label} command directly executes its EC source `{}`; use an external target/oracle command instead",
            case.test_path
        ));
    }
}

fn validate_stable_id(value: &str, field: &str, label: &str, findings: &mut Vec<String>) {
    if value.is_empty() {
        findings.push(format!(
            "{label} is missing `{field}`; declare a stable lowercase hyphenated identifier"
        ));
        return;
    }
    let valid = value.bytes().enumerate().all(|(index, byte)| match byte {
        b'a'..=b'z' | b'0'..=b'9' => true,
        b'-' => index != 0 && index + 1 != value.len(),
        _ => false,
    }) && !value.contains("--");
    if !valid {
        findings.push(format!(
            "{label} `{field}` must be a stable lowercase hyphenated identifier, got `{value}`"
        ));
    }
}

fn validate_artifact_id(value: &str, label: &str, findings: &mut Vec<String>) {
    let Some((kind, body)) = value.split_once(':') else {
        findings.push(format!(
            "{label} is missing `artifact_id`; declare artifact:<context>/<name> to bind this EC case to the DDD artifact graph"
        ));
        return;
    };
    let parts = body.split('/').collect::<Vec<_>>();
    let valid = kind == "artifact"
        && parts.len() == 2
        && parts.iter().all(|part| {
            let bytes = part.as_bytes();
            !bytes.is_empty()
                && bytes[0].is_ascii_lowercase()
                && bytes.last().is_some_and(u8::is_ascii_alphanumeric)
                && bytes
                    .iter()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
        });
    if !valid {
        findings.push(format!(
            "{label} `artifact_id` must be artifact:<context>/<name> using lowercase kebab-case segments, got `{value}`"
        ));
    }
}

fn validate_dimension_and_applicability(
    case: &PythonEcCase,
    label: &str,
    findings: &mut Vec<String>,
) {
    let valid_dimension = matches!(
        case.dimension.as_str(),
        "behavior" | "security" | "stability" | "efficiency"
    );
    if !valid_dimension {
        findings.push(format!(
            "{label} has unknown dimension `{}`; expected behavior|security|stability|efficiency",
            case.dimension
        ));
    }

    match case.applicability.as_str() {
        "td" => {
            if valid_dimension && !matches!(case.dimension.as_str(), "behavior" | "security") {
                findings.push(format!(
                    "{label} uses applicability `td`, which only applies to behavior or security; use `post-gen` for {}",
                    case.dimension
                ));
            }
        }
        "post-gen" => {
            if valid_dimension && !matches!(case.dimension.as_str(), "stability" | "efficiency") {
                findings.push(format!(
                    "{label} uses applicability `post-gen`, which only applies to stability or efficiency; use `td` for {}",
                    case.dimension
                ));
            }
        }
        _ => findings.push(format!(
            "{label} has invalid applicability `{}`; expected `td` or `post-gen`",
            case.applicability
        )),
    }
}

fn validate_test_path(root: &Path, value: &str, label: &str, findings: &mut Vec<String>) {
    if value.is_empty() {
        findings.push(format!(
            "{label} is missing `test_path`; point to its hand-authored src/*.py contract module"
        ));
        return;
    }
    let path = Path::new(value);
    let safe = !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
        && path.starts_with("src")
        && path.extension().and_then(|extension| extension.to_str()) == Some("py");
    if !safe {
        findings.push(format!(
            "{label} `test_path` must be a safe project-relative src/*.py path, got `{value}`"
        ));
        return;
    }
    let absolute = root.join(path);
    match fs::symlink_metadata(&absolute) {
        Ok(metadata) if metadata.file_type().is_symlink() => findings.push(format!(
            "{label} `test_path` must not be a symlink: `{value}`"
        )),
        Ok(metadata) if !metadata.is_file() => findings.push(format!(
            "{label} `test_path` is not a regular file: `{value}`"
        )),
        Ok(_) => {}
        Err(_) => findings.push(format!("{label} `test_path` does not exist: `{value}`")),
    }
}
// HANDWRITE-END
