//! Stable release-promotion oracle. Keep it independent from `release_candidate`.

use base64::Engine as _;
use serde_json::json;
use serde_yaml::{Mapping, Value};
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

const TAG: &str = "lumen@0.4.28";
const VERSION: &str = "0.4.28";
const COMMIT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const RUN_ID: &str = "123";
const RELEASE_WORKFLOW_SHA256: &str =
    "14cbd1d902c2961ff2efae64a11a68fe82aeb9ee767e56b338acd8ea76b6a2b4";
const PROMOTION_VERIFIER_BYTES_SHA256: &str =
    "08a21bb4b62db5b09d0371e52eb0f2ea88edbd5ed99dbe170355e2af32e8526a";
const CHECKOUT: &str = "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1";
const COSIGN_INSTALLER: &str = "sigstore/cosign-installer@6f9f17788090df1f26f669e9d70d6ae9567deba6";
const SETUP_BUILDX: &str = "docker/setup-buildx-action@8d2750c68a42422c14e847fe6c8ac0403b4cbd6f";
const DOCKER_LOGIN: &str = "docker/login-action@c94ce9fb468520275223c153574b00df6fe4bcc9";
const PUBLIC_RELEASE_NOTE_STATEMENTS: &[&str] = &[
    "- Release path: landed main -> untagged candidate verification -> protected annotated tag -> promotion of the same candidate digest.",
    "- Placement path: a non-empty nodeSelector with the default initialMachineType skips the legacy capacity catalog.",
    "- Legacy placement path: an empty selector, tolerations-only placement, or a non-default initialMachineType still requires lumen-system/lumen-capacity-catalog.",
];
const PUBLIC_COMPATIBILITY_NOTE: &str = "- Compatibility: shipped Docker images default to durable segment storage at /var/lib/lumen/data; bare lumen serve stays ephemeral without --data-dir or LUMEN_DATA_DIR. A 0.4.28 segment volume upgrades one way on first 0.4.29 start; in-place downgrade is unsupported.";
const PUBLIC_RECEIPT_NOTE: &str = "- Standalone GKE receipt SHA-256: ${{ needs.verify-inputs.outputs.standalone_gke_receipt_sha256 }}";
const PUBLIC_RECEIPT_INPUTS: &[(&str, &str)] = &[
    (
        "standalone_gke_receipt_b64",
        "Base64 of the sanitized lumen-standalone-gke-receipt.json bytes.",
    ),
    (
        "standalone_gke_receipt_sha256",
        "Lowercase SHA-256 of the sanitized lumen-standalone-gke-receipt.json bytes.",
    ),
    (
        "standalone_gke_receipt_sidecar_b64",
        "Base64 of the exact lumen-standalone-gke-receipt.json.sha256 sidecar bytes.",
    ),
    (
        "standalone_gke_receipt_sidecar_sha256",
        "Lowercase SHA-256 of the exact lumen-standalone-gke-receipt.json.sha256 sidecar bytes.",
    ),
];
const PUBLIC_RECEIPT_DECODE_RUN: &str = r#"set -euo pipefail
mkdir -p gke-receipt
decode_exact() {
  local encoded="$1" expected="$2" output="$3" actual
  [[ "$encoded" =~ ^[A-Za-z0-9+/]+={0,2}$ && $((${#encoded} % 4)) -eq 0 ]] || { echo "receipt input is not canonical base64" >&2; exit 1; }
  [[ "$expected" =~ ^[0-9a-f]{64}$ ]] || { echo "receipt input SHA-256 is invalid" >&2; exit 1; }
  printf '%s' "$encoded" | base64 --decode > "$output"
  actual="$(sha256sum "$output" | awk '{print $1}')"
  [[ "$actual" == "$expected" ]] || { echo "receipt input SHA-256 does not bind decoded bytes" >&2; exit 1; }
}
decode_exact "$STANDALONE_GKE_RECEIPT_B64" "$STANDALONE_GKE_RECEIPT_SHA256" gke-receipt/lumen-standalone-gke-receipt.json
decode_exact "$STANDALONE_GKE_RECEIPT_SIDECAR_B64" "$STANDALONE_GKE_RECEIPT_SIDECAR_SHA256" gke-receipt/lumen-standalone-gke-receipt.json.sha256
{
  echo "receipt=$PWD/gke-receipt/lumen-standalone-gke-receipt.json"
  echo "sidecar=$PWD/gke-receipt/lumen-standalone-gke-receipt.json.sha256"
} >> "$GITHUB_OUTPUT"
"#;
const OLD_PUBLIC_COMPATIBILITY_NOTE: &str =
    "- Compatibility: no API, CRD, or runtime-default migration.";
const RECOVERY_TAG_OBJECT: &str = "5b7b4d9a8b8b5596cb9fcfa149008a8f1313da82";
const RECOVERY_ROOT: &str =
    "sha256:59a85c96d807428c424ec8889ac830b14e02869da49c4b44ae12dcce3786d03d";
const RECOVERY_AMD64: &str =
    "sha256:754b3c53b849f1a6de94897fe11d796d1ede0120ac9aa2e3d226f1edf08e7b00";
const RECOVERY_ARM64: &str =
    "sha256:0b15b56db206ce2a7d9e832b829fceb88acaa28656d21f0e28f731b41bb5580f";
const RECOVERY_OLD_LATEST: &str =
    "sha256:4a5748848d384b2fa56b130b48976c309942dcd9e613e000da7f89a7c858cff4";
const RECOVERY_COMMIT: &str = "b1cbee3fcee0bfff54c425fe0f605b54125f4740";
const RECOVERY_RUN: &str = "32974297012";
const RECOVERY_WORKFLOW_SHA256: &str =
    "4fadd9fdae2f0db6f408424db19982f837274a549b0af22cce226e2a78839ca2";
static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut child = Command::new("shasum")
        .args(["-a", "256"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "shasum failed");
    String::from_utf8(output.stdout)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned()
}

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "lumen-release-artifacts-{name}-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed),
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn shell(command: &mut Command) -> String {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn sha256(path: &Path) -> String {
    let mut command = Command::new("shasum");
    command.arg("-a").arg("256").arg(path);
    let output = command.output().unwrap();
    if output.status.success() {
        return String::from_utf8(output.stdout)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .to_owned();
    }
    shell(Command::new("sha256sum").arg(path))
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned()
}

fn write_archive_pair(root: &Path, target: &str) -> (String, String, String, String) {
    write_archive_pair_for_version(root, target, VERSION)
}

fn write_archive_pair_for_version(
    root: &Path,
    target: &str,
    version: &str,
) -> (String, String, String, String) {
    let stage = root.join("stage").join(format!("lumen-{target}"));
    fs::create_dir_all(&stage).unwrap();
    fs::write(stage.join("README.md"), "fixture\n").unwrap();
    let binary = stage.join("lumen");
    fs::write(
        &binary,
        format!("#!/bin/sh\n# target={target}\necho 'lumen {version}'\n"),
    )
    .unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();

    let archive = format!("lumen-{target}.tar.gz");
    let archive_path = root.join(&archive);
    shell(
        Command::new("tar")
            .arg("-C")
            .arg(root.join("stage"))
            .arg("-czf")
            .arg(&archive_path)
            .arg(format!("lumen-{target}")),
    );
    let archive_sha = sha256(&archive_path);
    let sidecar = format!("{archive}.sha256");
    let sidecar_path = root.join(&sidecar);
    fs::write(&sidecar_path, format!("{archive_sha}  {archive}\n")).unwrap();
    let sidecar_sha = sha256(&sidecar_path);
    (archive, archive_sha, sidecar, sidecar_sha)
}

fn release_script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/verify-release-artifacts.sh")
}

struct ReleaseFixture {
    _temp: TempDir,
    candidate: PathBuf,
    public: PathBuf,
}

fn release_fixture() -> ReleaseFixture {
    let temp = TempDir::new("fixture");
    let candidate = temp.0.join("candidate");
    let public = temp.0.join("public");
    fs::create_dir_all(&candidate).unwrap();
    fs::create_dir_all(&public).unwrap();

    let targets = [
        "aarch64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-musl",
    ];
    let mut artifacts = Vec::new();
    for target in targets {
        let (archive, archive_sha, sidecar, sidecar_sha) = write_archive_pair(&candidate, target);
        fs::copy(candidate.join(&archive), public.join(&archive)).unwrap();
        fs::copy(candidate.join(&sidecar), public.join(&sidecar)).unwrap();
        artifacts.push(format!(
            r#"{{"target":"{target}","archive":"{archive}","archive_sha256":"{archive_sha}","sidecar":"{sidecar}","sidecar_sha256":"{sidecar_sha}"}}"#
        ));
    }
    let spdx = r#"{"spdxVersion":"SPDX-2.3","name":"fixture"}"#;
    for arch in ["amd64", "arm64"] {
        fs::write(candidate.join(format!("spdx-{arch}.json")), spdx).unwrap();
        fs::copy(
            candidate.join(format!("spdx-{arch}.json")),
            public.join(format!("spdx-{arch}.json")),
        )
        .unwrap();
    }
    let manifest = format!(
        r#"{{"schema":"cclab.lumen.candidate-manifest.v3","repository":"chrischeng-c4/axiom","workflow_path":".github/workflows/lumen-release-candidate.yml","workflow_id":1,"run_id":"{RUN_ID}","run_attempt":"1","run_url":"https://github.com/chrischeng-c4/axiom/actions/runs/{RUN_ID}/attempts/1","source_ref":"refs/heads/main","workflow_ref":"chrischeng-c4/axiom/.github/workflows/lumen-release-candidate.yml@refs/heads/main","commit":"{COMMIT}","version":"{VERSION}","tag":"{TAG}","candidate_tag":"release-candidate-{RUN_ID}-1","pr":{{"number":1,"url":"https://github.com/chrischeng-c4/axiom/pull/1"}},"image":{{"repository":"ghcr.io/chrischeng-c4/lumen","root_digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","amd64_digest":"sha256:2222222222222222222222222222222222222222222222222222222222222222","arm64_digest":"sha256:3333333333333333333333333333333333333333333333333333333333333333"}},"artifacts":[{}],"sboms":{{"amd64":{{"file":"spdx-amd64.json","sha256":"{}"}},"arm64":{{"file":"spdx-arm64.json","sha256":"{}"}}}},"jobs":{{"identity":"success","build":"success","manifest":"success","ghcr-image-and-attest":"success","verify-candidate":"success","verify-libraries":"success","kind-amd64":"success","kind-arm64":"success","result":"success"}}}}"#,
        artifacts.join(","),
        sha256(&candidate.join("spdx-amd64.json")),
        sha256(&candidate.join("spdx-arm64.json")),
    );
    fs::write(candidate.join("final-candidate-manifest.json"), &manifest).unwrap();
    let manifest_sha = sha256(&candidate.join("final-candidate-manifest.json"));
    fs::write(
        candidate.join("final-candidate-manifest.json.sha256"),
        format!("{manifest_sha}  final-candidate-manifest.json\n"),
    )
    .unwrap();
    ReleaseFixture {
        _temp: temp,
        candidate,
        public,
    }
}

fn run_fixture(fixture: &ReleaseFixture) -> std::process::Output {
    Command::new("bash")
        .arg(release_script())
        .args([
            "--repo",
            "chrischeng-c4/axiom",
            "--tag",
            TAG,
            "--commit",
            COMMIT,
            "--candidate-run-id",
            RUN_ID,
            "--mode",
            "fixture",
            "--candidate-receipt-dir",
        ])
        .arg(&fixture.candidate)
        .arg("--release-assets-dir")
        .arg(&fixture.public)
        .output()
        .unwrap()
}

fn refresh_sha256_sidecar(path: &Path) {
    fs::write(
        path.with_extension("json.sha256"),
        format!(
            "{}  {}\n",
            sha256(path),
            path.file_name().unwrap().to_string_lossy()
        ),
    )
    .unwrap();
}

fn archive_member_sha256(root: &Path, target: &str) -> String {
    let output = Command::new("tar")
        .args(["-xOzf"])
        .arg(root.join(format!("lumen-{target}.tar.gz")))
        .arg(format!("lumen-{target}/lumen"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "cannot stream fixture controller CLI: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    sha256_bytes(&output.stdout)
}

struct GkeReceiptFixture {
    release: ReleaseFixture,
    receipt: PathBuf,
    sidecar: PathBuf,
}

fn gke_receipt_fixture() -> GkeReceiptFixture {
    let release = release_fixture();
    let manifest_path = release.candidate.join("final-candidate-manifest.json");
    let mut manifest =
        serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&manifest_path).unwrap())
            .unwrap();
    manifest["version"] = serde_json::Value::String("0.4.29".to_owned());
    manifest["tag"] = serde_json::Value::String("lumen@0.4.29".to_owned());
    for target in [
        "aarch64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-musl",
    ] {
        let (archive, archive_sha, sidecar, sidecar_sha) =
            write_archive_pair_for_version(&release.candidate, target, "0.4.29");
        fs::copy(
            &release.candidate.join(&archive),
            release.public.join(&archive),
        )
        .unwrap();
        fs::copy(
            &release.candidate.join(&sidecar),
            release.public.join(&sidecar),
        )
        .unwrap();
        let entry = manifest["artifacts"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|entry| entry["target"] == target)
            .unwrap();
        entry["archive_sha256"] = serde_json::Value::String(archive_sha);
        entry["sidecar_sha256"] = serde_json::Value::String(sidecar_sha);
    }
    fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    refresh_sha256_sidecar(&manifest_path);

    let target = "x86_64-unknown-linux-gnu";
    let root = format!("sha256:{}", "1".repeat(64));
    let amd64 = format!("sha256:{}", "2".repeat(64));
    let arm64 = format!("sha256:{}", "3".repeat(64));
    let receipt = release.candidate.join("lumen-standalone-gke-receipt.json");
    let body = json!({
        "schema": "lumen.standalone-gke-receipt/v2",
        "stage": "slice-b-live",
        "complete": true,
        "candidate": {
            "repository": "chrischeng-c4/axiom",
            "version": "0.4.29",
            "commit": COMMIT,
            "workflow_ref": "chrischeng-c4/axiom/.github/workflows/lumen-release-candidate.yml@refs/heads/main",
            "run_id": RUN_ID,
            "run_attempt": "1",
            "manifest_sha256": sha256(&manifest_path),
            "root_digest": root.clone(),
            "amd64_digest": amd64.clone(),
            "arm64_digest": arm64.clone(),
            "controller_cli": {"target": target, "sha256": archive_member_sha256(&release.candidate, target)},
        },
        "matrix": {
            "clusterip_only": "passed",
            "network_policy": "passed",
            "allowed_ksa": "passed",
            "unlisted_ksa": "passed",
            "missing_token": "passed",
            "bad_token": "passed",
            "tokenreview": "passed",
            "subjectaccessreview": "passed",
            "application_admin_403": "passed",
            "admin_backup_restore": "passed",
            "pod_replacement": "passed",
            "pvc_recovery": "passed",
            "vertical_resize": "passed",
            "cleanup": "passed",
            "required_continuity": {
                "profile": "LUMEN_AUTH=required",
                "audience": "lumen.axiom.dev",
                "observed_runtime_image_digest": root.clone(),
                "scheduled_node_arch": "amd64",
                "scheduled_runtime_child_digest": amd64.clone(),
                "projected_allowed_2xx": "passed",
                "same_ksa_default_token_401": "passed",
                "projected_unlisted_403": "passed",
                "tokenreview_delta": 1,
                "subjectaccessreview_delta": 2,
                "allowed_delta": 3,
                "denied_delta": 4,
            }
        },
        "redaction": {
            "kubeconfig_retained": false,
            "token_retained": false,
            "authorization_retained": false,
            "secret_retained": false,
            "cluster_identity_retained": false,
            "command_output_retained": false,
            "canary_scan": true,
        }
    });
    fs::write(&receipt, serde_json::to_vec(&body).unwrap()).unwrap();
    let sidecar = receipt.with_extension("json.sha256");
    fs::write(
        &sidecar,
        format!("{}  lumen-standalone-gke-receipt.json\n", sha256(&receipt)),
    )
    .unwrap();
    fs::copy(
        &receipt,
        release.public.join("lumen-standalone-gke-receipt.json"),
    )
    .unwrap();
    fs::copy(
        &sidecar,
        release
            .public
            .join("lumen-standalone-gke-receipt.json.sha256"),
    )
    .unwrap();
    GkeReceiptFixture {
        release,
        receipt,
        sidecar,
    }
}

fn run_gke_receipt_fixture(fixture: &GkeReceiptFixture) -> Output {
    Command::new("bash")
        .arg(release_script())
        .args([
            "--repo",
            "chrischeng-c4/axiom",
            "--tag",
            "lumen@0.4.29",
            "--commit",
            COMMIT,
            "--candidate-run-id",
            RUN_ID,
            "--mode",
            "fixture",
            "--candidate-receipt-dir",
        ])
        .arg(&fixture.release.candidate)
        .arg("--release-assets-dir")
        .arg(&fixture.release.public)
        .arg("--standalone-gke-receipt")
        .arg(&fixture.receipt)
        .arg("--standalone-gke-receipt-sidecar")
        .arg(&fixture.sidecar)
        .output()
        .unwrap()
}

fn run_public_receipt_decoder(
    receipt_b64: String,
    receipt_sha256: String,
    sidecar_b64: String,
    sidecar_sha256: String,
) -> (TempDir, Output) {
    let temp = TempDir::new("public-receipt-decoder");
    let output = temp.0.join("github-output");
    let result = Command::new("bash")
        .args(["-c", PUBLIC_RECEIPT_DECODE_RUN, "bash"])
        .current_dir(&temp.0)
        .env("GITHUB_OUTPUT", &output)
        .env("STANDALONE_GKE_RECEIPT_B64", receipt_b64)
        .env("STANDALONE_GKE_RECEIPT_SHA256", receipt_sha256)
        .env("STANDALONE_GKE_RECEIPT_SIDECAR_B64", sidecar_b64)
        .env("STANDALONE_GKE_RECEIPT_SIDECAR_SHA256", sidecar_sha256)
        .output()
        .unwrap();
    (temp, result)
}

fn rewrite_receipt(fixture: &GkeReceiptFixture, receipt: &serde_json::Value) {
    fs::write(&fixture.receipt, serde_json::to_vec(receipt).unwrap()).unwrap();
    fs::write(
        &fixture.sidecar,
        format!(
            "{}  lumen-standalone-gke-receipt.json\n",
            sha256(&fixture.receipt)
        ),
    )
    .unwrap();
    fs::copy(
        &fixture.receipt,
        fixture
            .release
            .public
            .join("lumen-standalone-gke-receipt.json"),
    )
    .unwrap();
    fs::copy(
        &fixture.sidecar,
        fixture
            .release
            .public
            .join("lumen-standalone-gke-receipt.json.sha256"),
    )
    .unwrap();
}

fn workflow_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/lumen-release.yml")
}

const CANDIDATE_EXECUTION_NAMES: &[&str] = &[
    "bind candidate inputs",
    "build (aarch64-apple-darwin)",
    "build (aarch64-unknown-linux-gnu)",
    "build (aarch64-unknown-linux-musl)",
    "build (x86_64-unknown-linux-gnu)",
    "build (x86_64-unknown-linux-musl)",
    "build candidate image and attest",
    "candidate identity",
    "final candidate receipt",
    "kind e2e (amd64)",
    "kind e2e (arm64)",
    "verify exact candidate gates",
    "verify service and Raft library gates",
];

fn shell_function_lines<'a>(source: &'a str, name: &str) -> Result<Vec<&'a str>, String> {
    let header = format!("{name}() {{");
    let starts = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| (line.trim() == header).then_some(index))
        .collect::<Vec<_>>();
    if starts.len() != 1 {
        return Err(format!("{name} definition count changed: {}", starts.len()));
    }
    let lines = source.lines().collect::<Vec<_>>();
    let next_function = lines
        .iter()
        .enumerate()
        .skip(starts[0] + 1)
        .find_map(|(index, line)| {
            let trimmed = line.trim();
            (line.len() == line.trim_start().len()
                && trimmed.ends_with("() {")
                && trimmed[..trimmed.len() - 4]
                    .chars()
                    .all(|ch| ch == '_' || ch.is_ascii_alphanumeric()))
            .then_some(index)
        })
        .unwrap_or(lines.len());
    let closing = lines[starts[0] + 1..next_function]
        .iter()
        .rposition(|line| line.trim() == "}")
        .map(|offset| starts[0] + 1 + offset)
        .ok_or_else(|| format!("{name} closing brace is missing"))?;
    if lines[closing + 1..next_function]
        .iter()
        .any(|line| !line.trim().is_empty())
    {
        return Err(format!(
            "{name} has executable text after its closing brace"
        ));
    }
    Ok(lines[starts[0] + 1..closing]
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect())
}

fn validate_candidate_execution_bindings(verifier: &str) -> Result<(), String> {
    let lines = shell_function_lines(verifier, "fetch_candidate_receipt")?;
    for forbidden in ["if ", "case ", "for ", "while ", "until ", "{", "}"] {
        if lines
            .iter()
            .any(|line| *line == forbidden.trim() || line.starts_with(forbidden))
        {
            return Err(format!(
                "fetch_candidate_receipt gained unsupported control flow: {forbidden}"
            ));
        }
    }
    for assignment in ["jobs=", "artifacts="] {
        if lines
            .iter()
            .filter(|line| line.starts_with(assignment))
            .count()
            != 1
        {
            return Err(format!(
                "fetch_candidate_receipt assignment count changed: {assignment}"
            ));
        }
    }
    for marker in [
        "jobs?filter=latest&per_page=100\" | flatten_paginated_jobs)",
        "validate_candidate_job_inventory <<<\"$jobs\"",
        "artifacts?per_page=100\" | flatten_paginated_artifacts)",
    ] {
        if lines.iter().filter(|line| line.contains(marker)).count() != 1 {
            return Err(format!("candidate execution binding changed: {marker}"));
        }
    }
    for (label, jobs, expect_download) in [
        (
            "valid",
            candidate_jobs(CANDIDATE_EXECUTION_NAMES, None),
            true,
        ),
        (
            "missing",
            candidate_jobs(&CANDIDATE_EXECUTION_NAMES[..12], None),
            false,
        ),
        (
            "failed",
            candidate_jobs(CANDIDATE_EXECUTION_NAMES, Some(0)),
            false,
        ),
        (
            "extra",
            candidate_jobs(
                &[CANDIDATE_EXECUTION_NAMES, &["unexpected execution"]].concat(),
                None,
            ),
            false,
        ),
        (
            "duplicate",
            candidate_jobs(
                &CANDIDATE_EXECUTION_NAMES
                    .iter()
                    .enumerate()
                    .map(|(index, name)| {
                        if index == 12 {
                            CANDIDATE_EXECUTION_NAMES[0]
                        } else {
                            name
                        }
                    })
                    .collect::<Vec<_>>(),
                None,
            ),
            false,
        ),
    ] {
        let downloaded = execute_candidate_fetch(verifier, &jobs);
        if downloaded != expect_download {
            return Err(format!(
                "{label} inventory did not produce expected artifact-download behavior"
            ));
        }
    }
    if sha256_bytes(verifier.as_bytes()) != PROMOTION_VERIFIER_BYTES_SHA256 {
        return Err("PROMOTION_VERIFIER_BYTES".to_owned());
    }
    Ok(())
}

fn execute_candidate_fetch(verifier: &str, jobs: &str) -> bool {
    let temp = TempDir::new("candidate-fetch");
    let bin = temp.0.join("bin");
    let receipt = temp.0.join("receipt");
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(&receipt).unwrap();
    let pages = serde_json::from_str::<serde_json::Value>(jobs).unwrap();
    let split = pages.as_array().unwrap().len() / 2;
    let first = serde_json::to_string(&json!({"total_count": pages.as_array().unwrap().len(), "jobs": &pages.as_array().unwrap()[..split]})).unwrap();
    let second = serde_json::to_string(&json!({"total_count": pages.as_array().unwrap().len(), "jobs": &pages.as_array().unwrap()[split..]})).unwrap();
    let gh = format!(
        "#!/usr/bin/env bash\nset -e\ncase \"$*\" in\n  *'/actions/runs/123'*) printf '%s\\n' '{{\"run_attempt\":\"1\",\"event\":\"workflow_dispatch\",\"status\":\"completed\",\"conclusion\":\"success\",\"head_branch\":\"main\",\"head_sha\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"workflow_id\":99,\"head_repository\":{{\"full_name\":\"chrischeng-c4/axiom\"}}}}' ;;\n  *'/actions/workflows/lumen-release-candidate.yml'*) printf '99\\n' ;;\n  *'/attempts/1/jobs'*) printf '%s\\n%s\\n' '{first}' '{second}' ;;\n  *'/artifacts?per_page=100'*) printf '%s\\n' '{{\"total_count\":1,\"artifacts\":[{{\"name\":\"lumen-release-candidate-123-1\",\"expired\":false,\"id\":7}}]}}' ;;\n  *'/artifacts/7/zip'*) touch \"$ARTIFACT_DOWNLOAD\"; exit 1 ;;\n  *) exit 2 ;;\nesac\n"
    );
    let gh = gh.replace(
        "case \"$*\" in\n",
        &format!(
            "if [[ \"$*\" == */attempts/1/jobs* ]]; then printf '%s\\n%s\\n' '{first}' '{second}'; exit 0; fi\nif [[ \"$*\" == */artifacts\\?per_page=100* ]]; then printf '%s\\n' '{{\"total_count\":1,\"artifacts\":[{{\"name\":\"lumen-release-candidate-123-1\",\"expired\":false,\"id\":7}}]}}'; exit 0; fi\nif [[ \"$*\" == */artifacts/7/zip* ]]; then touch \"$ARTIFACT_DOWNLOAD\"; exit 1; fi\ncase \"$*\" in\n"
        ),
    );
    let gh_path = bin.join("gh");
    fs::write(&gh_path, gh).unwrap();
    fs::set_permissions(&gh_path, fs::Permissions::from_mode(0o755)).unwrap();
    let script = temp.0.join("verifier.sh");
    fs::write(&script, verifier).unwrap();
    let marker = temp.0.join("downloaded");
    let output = Command::new("bash")
        .arg("-c")
        .arg("source \"$1\"; fetch_candidate_receipt")
        .arg("bash")
        .arg(&script)
        .env(
            "PATH",
            format!("{}:{}", bin.display(), std::env::var("PATH").unwrap()),
        )
        .env("REPO", "chrischeng-c4/axiom")
        .env("TAG", "lumen@0.4.28")
        .env("COMMIT", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        .env("CANDIDATE_RUN_ID", "123")
        .env("CANDIDATE_RECEIPT_DIR", &receipt)
        .env("ARTIFACT_DOWNLOAD", &marker)
        .output()
        .unwrap();
    let _ = output;
    marker.exists()
}

fn execute_verifier_function(verifier: &str, function: &str, input: &str) -> Output {
    let temp = TempDir::new("pagination");
    let script_path = temp.0.join("verifier.sh");
    let input_path = temp.0.join("input.json");
    fs::write(&script_path, verifier).unwrap();
    fs::write(&input_path, input).unwrap();
    Command::new("bash")
        .args(["-c", "source \"$1\"; \"$2\" < \"$3\""])
        .arg("bash")
        .arg(&script_path)
        .arg(function)
        .arg(&input_path)
        .output()
        .unwrap()
}

fn execute_verifier_function_with_arg(
    verifier: &str,
    function: &str,
    argument: &str,
    input: &str,
) -> Output {
    let temp = TempDir::new("pagination-arg");
    let script_path = temp.0.join("verifier.sh");
    let input_path = temp.0.join("input.json");
    fs::write(&script_path, verifier).unwrap();
    fs::write(&input_path, input).unwrap();
    Command::new("bash")
        .args(["-c", "source \"$1\"; \"$2\" \"$3\" < \"$4\""])
        .arg("bash")
        .arg(&script_path)
        .arg(function)
        .arg(argument)
        .arg(&input_path)
        .output()
        .unwrap()
}

fn execute_page_flattener(function: &str, pages: &str) -> Value {
    let output = execute_verifier_function(
        include_str!("../scripts/verify-release-artifacts.sh"),
        function,
        pages,
    );
    assert!(
        output.status.success(),
        "{function} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_yaml::from_slice(&output.stdout).unwrap()
}

fn candidate_jobs(names: &[&str], failed: Option<usize>) -> String {
    serde_json::to_string(
        &names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                serde_json::json!({
                    "name": name,
                    "status": "completed",
                    "conclusion": if failed == Some(index) { "failure" } else { "success" },
                })
            })
            .collect::<Vec<_>>(),
    )
    .unwrap()
}

fn yaml_mapping<'a>(value: &'a Value, path: &str) -> Result<&'a Mapping, String> {
    value
        .as_mapping()
        .ok_or_else(|| format!("{path} must be a mapping"))
}

fn yaml_sequence<'a>(value: &'a Value, path: &str) -> Result<&'a Vec<Value>, String> {
    value
        .as_sequence()
        .ok_or_else(|| format!("{path} must be a sequence"))
}

fn yaml_field<'a>(map: &'a Mapping, key: &str, path: &str) -> Result<&'a Value, String> {
    map.get(Value::String(key.to_owned()))
        .ok_or_else(|| format!("{path}.{key} is required"))
}

fn yaml_text<'a>(value: &'a Value, path: &str) -> Result<&'a str, String> {
    value
        .as_str()
        .ok_or_else(|| format!("{path} must be a string"))
}

fn exact_keys(map: &Mapping, expected: &[&str], path: &str) -> Result<(), String> {
    let mut actual = Vec::new();
    for key in map.keys() {
        actual.push(
            key.as_str()
                .ok_or_else(|| format!("{path} has a non-string key"))?,
        );
    }
    actual.sort_unstable();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{path} keys changed: {actual:?}"))
    }
}

fn expect_text(map: &Mapping, key: &str, expected: &str, path: &str) -> Result<(), String> {
    let actual = yaml_text(yaml_field(map, key, path)?, &format!("{path}.{key}"))?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{path}.{key} changed"))
    }
}

fn expect_bool(map: &Mapping, key: &str, expected: bool, path: &str) -> Result<(), String> {
    let actual = yaml_field(map, key, path)?
        .as_bool()
        .ok_or_else(|| format!("{path}.{key} must be a boolean"))?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{path}.{key} changed"))
    }
}

fn expect_i64(map: &Mapping, key: &str, expected: i64, path: &str) -> Result<(), String> {
    let actual = yaml_field(map, key, path)?
        .as_i64()
        .ok_or_else(|| format!("{path}.{key} must be an integer"))?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{path}.{key} changed"))
    }
}

fn validate_permissions(
    job: &Mapping,
    expected: &[(&str, &str)],
    path: &str,
) -> Result<(), String> {
    let permissions = yaml_mapping(
        yaml_field(job, "permissions", path)?,
        &format!("{path}.permissions"),
    )?;
    let keys = expected.iter().map(|(key, _)| *key).collect::<Vec<_>>();
    exact_keys(permissions, &keys, &format!("{path}.permissions"))?;
    for (key, value) in expected {
        expect_text(permissions, key, value, &format!("{path}.permissions"))?;
    }
    Ok(())
}

fn validate_action_step(
    step: &Value,
    path: &str,
    expected_uses: &str,
    expected_with: &[(&str, &str)],
    expected_integer_with: Option<(&str, i64)>,
) -> Result<(), String> {
    let step = yaml_mapping(step, path)?;
    let mut keys = vec!["uses"];
    if !expected_with.is_empty() || expected_integer_with.is_some() {
        keys.push("with");
    }
    exact_keys(step, &keys, path)?;
    expect_text(step, "uses", expected_uses, path)?;
    let (_, pin) = expected_uses
        .rsplit_once('@')
        .ok_or_else(|| format!("{path}.uses is missing a pin"))?;
    if pin.len() != 40 || !pin.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{path}.uses is not pinned to a 40-hex revision"));
    }
    if !expected_with.is_empty() || expected_integer_with.is_some() {
        let with = yaml_mapping(yaml_field(step, "with", path)?, &format!("{path}.with"))?;
        let mut with_keys = expected_with
            .iter()
            .map(|(key, _)| *key)
            .collect::<Vec<_>>();
        if let Some((key, _)) = expected_integer_with {
            with_keys.push(key);
        }
        exact_keys(with, &with_keys, &format!("{path}.with"))?;
        for (key, value) in expected_with {
            expect_text(with, key, value, &format!("{path}.with"))?;
        }
        if let Some((key, value)) = expected_integer_with {
            expect_i64(with, key, value, &format!("{path}.with"))?;
        }
    }
    Ok(())
}

fn validate_run_step(
    step: &Value,
    path: &str,
    name: &str,
    expected_keys: &[&str],
    required_fragments: &[&str],
    expected_if: Option<&str>,
    expected_id: Option<&str>,
    needs_gh_token: bool,
) -> Result<(), String> {
    let step = yaml_mapping(step, path)?;
    exact_keys(step, expected_keys, path)?;
    expect_text(step, "name", name, path)?;
    expect_text(step, "shell", "bash", path)?;
    if let Some(expected_if) = expected_if {
        expect_text(step, "if", expected_if, path)?;
    }
    if let Some(expected_id) = expected_id {
        expect_text(step, "id", expected_id, path)?;
    }
    if needs_gh_token {
        let env = yaml_mapping(yaml_field(step, "env", path)?, &format!("{path}.env"))?;
        exact_keys(env, &["GH_TOKEN"], &format!("{path}.env"))?;
        expect_text(
            env,
            "GH_TOKEN",
            "${{ github.token }}",
            &format!("{path}.env"),
        )?;
    }
    let run = yaml_text(yaml_field(step, "run", path)?, &format!("{path}.run"))?;
    for fragment in required_fragments {
        if !run.contains(fragment) {
            return Err(format!(
                "{path}.run is missing required control: {fragment}"
            ));
        }
    }
    Ok(())
}

fn validate_release_creation_step(step: &Value, path: &str) -> Result<(), String> {
    validate_run_step(
        step,
        path,
        "Create exact GitHub Release with candidate bytes",
        &["name", "if", "env", "shell", "run"],
        &[],
        Some("steps.existing_release.outputs.exists == 'false'"),
        None,
        true,
    )?;
    let step = yaml_mapping(step, path)?;
    let run = yaml_text(yaml_field(step, "run", path)?, &format!("{path}.run"))?;
    let lines = run.lines().collect::<Vec<_>>();
    let expected = [
        "set -euo pipefail",
        "cat > release-notes.md <<EOF",
        "# lumen@${{ inputs.version }}",
        "",
        "- Source commit: $GITHUB_SHA",
        "- Pull request: ${{ needs.verify-inputs.outputs.pr_url }}",
        "- Candidate run: ${{ needs.verify-inputs.outputs.candidate_url }}",
        "- Promotion run: https://github.com/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID/attempts/$GITHUB_RUN_ATTEMPT",
        "- Root index digest: ${{ needs.verify-inputs.outputs.root_digest }}",
        "- linux/amd64 digest: ${{ needs.verify-inputs.outputs.amd64_digest }}",
        "- linux/arm64 digest: ${{ needs.verify-inputs.outputs.arm64_digest }}",
        PUBLIC_RECEIPT_NOTE,
        PUBLIC_RELEASE_NOTE_STATEMENTS[0],
        PUBLIC_RELEASE_NOTE_STATEMENTS[1],
        PUBLIC_RELEASE_NOTE_STATEMENTS[2],
        PUBLIC_COMPATIBILITY_NOTE,
        "EOF",
        "gh release create \"lumen@${{ inputs.version }}\" --repo \"$GITHUB_REPOSITORY\" --target \"$GITHUB_SHA\" --title \"lumen@${{ inputs.version }}\" --notes-file release-notes.md \\",
        "  candidate/lumen-*.tar.gz candidate/lumen-*.tar.gz.sha256 candidate/spdx-amd64.json candidate/spdx-arm64.json gke-receipt/lumen-standalone-gke-receipt.json gke-receipt/lumen-standalone-gke-receipt.json.sha256",
    ];
    if lines.as_slice() != expected {
        return Err(format!(
            "{path}.run release-note heredoc or create command changed"
        ));
    }
    let body = &lines[2..16];
    if body
        .iter()
        .filter(|line| **line == PUBLIC_COMPATIBILITY_NOTE)
        .count()
        != 1
    {
        return Err(format!(
            "{path}.run compatibility note is not one raw heredoc body line"
        ));
    }
    Ok(())
}

fn validate_public_receipt_decoder(step: &Value, path: &str) -> Result<(), String> {
    let step = yaml_mapping(step, path)?;
    exact_keys(step, &["name", "id", "env", "shell", "run"], path)?;
    expect_text(
        step,
        "name",
        "Decode and bind sanitized standalone GKE receipt",
        path,
    )?;
    expect_text(step, "id", "gke_receipt", path)?;
    expect_text(step, "shell", "bash", path)?;
    let env = yaml_mapping(yaml_field(step, "env", path)?, &format!("{path}.env"))?;
    let expected_env = PUBLIC_RECEIPT_INPUTS
        .iter()
        .map(|(name, _)| name.to_ascii_uppercase())
        .collect::<Vec<_>>();
    let expected_env_refs = PUBLIC_RECEIPT_INPUTS
        .iter()
        .map(|(name, _)| format!("${{{{ inputs.{name} }}}}"))
        .collect::<Vec<_>>();
    exact_keys(
        env,
        &expected_env.iter().map(String::as_str).collect::<Vec<_>>(),
        &format!("{path}.env"),
    )?;
    for (name, value) in expected_env.iter().zip(expected_env_refs.iter()) {
        expect_text(env, name, value, &format!("{path}.env"))?;
    }
    let run = yaml_text(yaml_field(step, "run", path)?, &format!("{path}.run"))?;
    if run != PUBLIC_RECEIPT_DECODE_RUN {
        return Err(format!(
            "{path}.run receipt decode and hash binding changed"
        ));
    }
    Ok(())
}

fn count(text: &str, needle: &str) -> usize {
    text.match_indices(needle).count()
}

fn validate_promotion_workflow(workflow: &str) -> Result<(), String> {
    let document: Value = serde_yaml::from_str(workflow)
        .map_err(|error| format!("workflow is not valid YAML: {error}"))?;
    let top = yaml_mapping(&document, "workflow")?;
    exact_keys(top, &["name", "on", "concurrency", "jobs"], "workflow")?;
    expect_text(top, "name", "lumen-release", "workflow")?;

    let trigger = yaml_mapping(yaml_field(top, "on", "workflow")?, "workflow.on")?;
    exact_keys(trigger, &["workflow_dispatch"], "workflow.on")?;
    let dispatch = yaml_mapping(
        yaml_field(trigger, "workflow_dispatch", "workflow.on")?,
        "workflow.on.workflow_dispatch",
    )?;
    exact_keys(dispatch, &["inputs"], "workflow.on.workflow_dispatch")?;
    let inputs = yaml_mapping(
        yaml_field(dispatch, "inputs", "workflow.on.workflow_dispatch")?,
        "workflow.on.workflow_dispatch.inputs",
    )?;
    exact_keys(
        inputs,
        &[
            "version",
            "candidate_run_id",
            "standalone_gke_receipt_b64",
            "standalone_gke_receipt_sha256",
            "standalone_gke_receipt_sidecar_b64",
            "standalone_gke_receipt_sidecar_sha256",
        ],
        "workflow.on.workflow_dispatch.inputs",
    )?;
    for (name, description) in [
        ("version", "Exact Lumen semver without the lumen@ prefix."),
        (
            "candidate_run_id",
            "Exact successful lumen-release-candidate run ID.",
        ),
    ]
    .into_iter()
    .chain(
        PUBLIC_RECEIPT_INPUTS
            .iter()
            .map(|(name, description)| (*name, *description)),
    ) {
        let input_path = format!("workflow.on.workflow_dispatch.inputs.{name}");
        let input = yaml_mapping(
            yaml_field(inputs, name, "workflow.on.workflow_dispatch.inputs")?,
            &input_path,
        )?;
        exact_keys(input, &["description", "required", "type"], &input_path)?;
        expect_text(input, "description", description, &input_path)?;
        expect_bool(input, "required", true, &input_path)?;
        expect_text(input, "type", "string", &input_path)?;
    }

    let concurrency = yaml_mapping(
        yaml_field(top, "concurrency", "workflow")?,
        "workflow.concurrency",
    )?;
    exact_keys(
        concurrency,
        &["group", "cancel-in-progress"],
        "workflow.concurrency",
    )?;
    expect_text(
        concurrency,
        "group",
        "lumen-release-promotion-${{ inputs.version }}",
        "workflow.concurrency",
    )?;
    expect_bool(
        concurrency,
        "cancel-in-progress",
        false,
        "workflow.concurrency",
    )?;

    let jobs = yaml_mapping(yaml_field(top, "jobs", "workflow")?, "workflow.jobs")?;
    exact_keys(jobs, &["verify-inputs", "publish-release"], "workflow.jobs")?;
    let verify = yaml_mapping(
        yaml_field(jobs, "verify-inputs", "workflow.jobs")?,
        "workflow.jobs.verify-inputs",
    )?;
    exact_keys(
        verify,
        &["name", "runs-on", "permissions", "outputs", "steps"],
        "workflow.jobs.verify-inputs",
    )?;
    expect_text(
        verify,
        "name",
        "prove immutable tag and candidate receipt",
        "workflow.jobs.verify-inputs",
    )?;
    expect_text(
        verify,
        "runs-on",
        "ubuntu-latest",
        "workflow.jobs.verify-inputs",
    )?;
    validate_permissions(
        verify,
        &[
            ("actions", "read"),
            ("attestations", "read"),
            ("contents", "read"),
            ("packages", "read"),
            ("pull-requests", "read"),
        ],
        "workflow.jobs.verify-inputs",
    )?;
    let outputs = yaml_mapping(
        yaml_field(verify, "outputs", "workflow.jobs.verify-inputs")?,
        "workflow.jobs.verify-inputs.outputs",
    )?;
    exact_keys(
        outputs,
        &[
            "root_digest",
            "amd64_digest",
            "arm64_digest",
            "candidate_attempt",
            "candidate_url",
            "pr_url",
            "standalone_gke_receipt_sha256",
        ],
        "workflow.jobs.verify-inputs.outputs",
    )?;
    for key in [
        "root_digest",
        "amd64_digest",
        "arm64_digest",
        "candidate_attempt",
        "candidate_url",
        "pr_url",
        "standalone_gke_receipt_sha256",
    ] {
        expect_text(
            outputs,
            key,
            &format!("${{{{ steps.contract.outputs.{key} }}}}"),
            "workflow.jobs.verify-inputs.outputs",
        )?;
    }
    let verify_steps = yaml_sequence(
        yaml_field(verify, "steps", "workflow.jobs.verify-inputs")?,
        "workflow.jobs.verify-inputs.steps",
    )?;
    if verify_steps.len() != 8 {
        return Err("verify-inputs step count changed".to_owned());
    }
    validate_action_step(
        &verify_steps[0],
        "workflow.jobs.verify-inputs.steps[0]",
        CHECKOUT,
        &[("ref", "${{ github.sha }}")],
        Some(("fetch-depth", 0)),
    )?;
    validate_run_step(
        &verify_steps[1],
        "workflow.jobs.verify-inputs.steps[1]",
        "Refuse the retired tag-first route",
        &["name", "shell", "run"],
        &[
            "$GITHUB_EVENT_NAME\" == workflow_dispatch",
            "refs/tags/lumen@${{ inputs.version }}",
            "$GITHUB_WORKFLOW_REF",
        ],
        None,
        None,
        false,
    )?;
    validate_action_step(
        &verify_steps[2],
        "workflow.jobs.verify-inputs.steps[2]",
        COSIGN_INSTALLER,
        &[("cosign-release", "v3.1.3")],
        None,
    )?;
    validate_action_step(
        &verify_steps[3],
        "workflow.jobs.verify-inputs.steps[3]",
        SETUP_BUILDX,
        &[],
        None,
    )?;
    validate_action_step(
        &verify_steps[4],
        "workflow.jobs.verify-inputs.steps[4]",
        DOCKER_LOGIN,
        &[
            ("registry", "ghcr.io"),
            ("username", "${{ github.actor }}"),
            ("password", "${{ github.token }}"),
        ],
        None,
    )?;
    validate_public_receipt_decoder(&verify_steps[5], "workflow.jobs.verify-inputs.steps[5]")?;
    validate_run_step(
        &verify_steps[6],
        "workflow.jobs.verify-inputs.steps[6]",
        "Verify tag ruleset, candidate receipt, and supply chain",
        &["name", "env", "shell", "run"],
        &[
            "verify-release-artifacts.sh",
            "--mode candidate",
            "--output promotion-contract.json",
            "--standalone-gke-receipt \"${{ steps.gke_receipt.outputs.receipt }}\"",
            "--standalone-gke-receipt-sidecar \"${{ steps.gke_receipt.outputs.sidecar }}\"",
        ],
        None,
        None,
        true,
    )?;
    validate_run_step(
        &verify_steps[7],
        "workflow.jobs.verify-inputs.steps[7]",
        "Export immutable candidate contract",
        &["name", "id", "shell", "run"],
        &[
            ".commit == $commit",
            "candidate_run_id",
            ".standalone_gke_receipt_sha256 == $receipt",
            "root_digest=$(jq -r",
        ],
        None,
        Some("contract"),
        false,
    )?;

    let publish = yaml_mapping(
        yaml_field(jobs, "publish-release", "workflow.jobs")?,
        "workflow.jobs.publish-release",
    )?;
    exact_keys(
        publish,
        &["name", "needs", "runs-on", "permissions", "steps"],
        "workflow.jobs.publish-release",
    )?;
    expect_text(
        publish,
        "name",
        "promote exact candidate root and publish release",
        "workflow.jobs.publish-release",
    )?;
    expect_text(
        publish,
        "needs",
        "verify-inputs",
        "workflow.jobs.publish-release",
    )?;
    expect_text(
        publish,
        "runs-on",
        "ubuntu-latest",
        "workflow.jobs.publish-release",
    )?;
    validate_permissions(
        publish,
        &[
            ("actions", "read"),
            ("attestations", "read"),
            ("contents", "write"),
            ("packages", "write"),
            ("pull-requests", "read"),
        ],
        "workflow.jobs.publish-release",
    )?;
    let publish_steps = yaml_sequence(
        yaml_field(publish, "steps", "workflow.jobs.publish-release")?,
        "workflow.jobs.publish-release.steps",
    )?;
    if publish_steps.len() != 12 {
        return Err("publish-release step count changed".to_owned());
    }
    validate_action_step(
        &publish_steps[0],
        "workflow.jobs.publish-release.steps[0]",
        CHECKOUT,
        &[("ref", "${{ github.sha }}")],
        Some(("fetch-depth", 0)),
    )?;
    validate_action_step(
        &publish_steps[1],
        "workflow.jobs.publish-release.steps[1]",
        COSIGN_INSTALLER,
        &[("cosign-release", "v3.1.3")],
        None,
    )?;
    validate_action_step(
        &publish_steps[2],
        "workflow.jobs.publish-release.steps[2]",
        SETUP_BUILDX,
        &[],
        None,
    )?;
    validate_action_step(
        &publish_steps[3],
        "workflow.jobs.publish-release.steps[3]",
        DOCKER_LOGIN,
        &[
            ("registry", "ghcr.io"),
            ("username", "${{ github.actor }}"),
            ("password", "${{ github.token }}"),
        ],
        None,
    )?;
    validate_public_receipt_decoder(&publish_steps[4], "workflow.jobs.publish-release.steps[4]")?;
    validate_run_step(
        &publish_steps[5],
        "workflow.jobs.publish-release.steps[5]",
        "Re-query exact candidate immediately before stable writes",
        &["name", "env", "shell", "run"],
        &[
            "verify-release-artifacts.sh",
            "--mode candidate",
            ".root_digest == $root",
            ".standalone_gke_receipt_sha256 == $receipt",
            "--standalone-gke-receipt \"${{ steps.gke_receipt.outputs.receipt }}\"",
            "--standalone-gke-receipt-sidecar \"${{ steps.gke_receipt.outputs.sidecar }}\"",
        ],
        None,
        None,
        true,
    )?;
    validate_run_step(
        &publish_steps[6],
        "workflow.jobs.publish-release.steps[6]",
        "Exit safely if this exact public release already exists",
        &["name", "id", "env", "shell", "run"],
        &["gh api graphql", "release(tagName", "exists=true"],
        None,
        Some("existing_release"),
        true,
    )?;
    validate_run_step(
        &publish_steps[7],
        "workflow.jobs.publish-release.steps[7]",
        "Verify existing public release without moving latest",
        &["name", "if", "env", "shell", "run"],
        &[
            "verify-release-artifacts.sh",
            "--mode public",
            "--standalone-gke-receipt \"${{ steps.gke_receipt.outputs.receipt }}\"",
            "--standalone-gke-receipt-sidecar \"${{ steps.gke_receipt.outputs.sidecar }}\"",
        ],
        Some("steps.existing_release.outputs.exists == 'true'"),
        None,
        true,
    )?;
    validate_run_step(
        &publish_steps[8],
        "workflow.jobs.publish-release.steps[8]",
        "Download exact candidate release bytes",
        &["name", "if", "env", "shell", "run"],
        &["gh run download", "final-candidate-manifest.json"],
        Some("steps.existing_release.outputs.exists == 'false'"),
        None,
        true,
    )?;
    validate_run_step(
        &publish_steps[9],
        "workflow.jobs.publish-release.steps[9]",
        "Promote exact root digest to semver and safe latest",
        &["name", "if", "env", "shell", "run"],
        &[
            "semver_current",
            "latest_current",
            "docker buildx imagetools create",
        ],
        Some("steps.existing_release.outputs.exists == 'false'"),
        None,
        true,
    )?;
    validate_release_creation_step(
        &publish_steps[10],
        "workflow.jobs.publish-release.steps[10]",
    )?;
    validate_run_step(
        &publish_steps[11],
        "workflow.jobs.publish-release.steps[11]",
        "Publicly verify immutable tag, release bytes, and promoted image",
        &["name", "if", "env", "shell", "run"],
        &[
            "verify-release-artifacts.sh",
            "--mode public",
            "--standalone-gke-receipt \"${{ steps.gke_receipt.outputs.receipt }}\"",
            "--standalone-gke-receipt-sidecar \"${{ steps.gke_receipt.outputs.sidecar }}\"",
        ],
        Some("steps.existing_release.outputs.exists == 'false'"),
        None,
        true,
    )?;

    let lower = workflow.to_ascii_lowercase();
    for forbidden in [
        "id-token: write",
        "cargo build",
        "docker/build-push-action",
        "cosign sign",
        "cosign attest",
        "actions/attest",
        "anchore/sbom-action",
        "git tag",
        "git push",
        "git update-ref",
        "/git/refs/tags",
        "deleteref",
        "updateref",
        "gcloud",
        "kubectl",
        "kind create",
        "gh run upload",
        "actions/upload-artifact",
    ] {
        if lower.contains(forbidden) {
            return Err(format!(
                "workflow contains forbidden release mutation or environment: {forbidden}"
            ));
        }
    }
    if count(workflow, "docker buildx imagetools create") != 2
        || count(
            workflow,
            "docker buildx imagetools create --tag \"$semver_ref\" \"$root_ref\"",
        ) != 1
        || count(
            workflow,
            "docker buildx imagetools create --tag \"${image_repo}:latest\" \"$root_ref\"",
        ) != 1
        || count(workflow, "gh release create") != 1
        || count(workflow, "--mode candidate") != 2
        || count(workflow, "--mode public") != 2
    {
        return Err("workflow stable write or verifier inventory changed".to_owned());
    }
    let requery = workflow
        .find("Re-query exact candidate immediately before stable writes")
        .ok_or_else(|| "immediate candidate re-query is absent".to_owned())?;
    let promotion = workflow
        .find("Promote exact root digest to semver and safe latest")
        .ok_or_else(|| "digest promotion is absent".to_owned())?;
    let release_create = workflow
        .find("gh release create")
        .ok_or_else(|| "GitHub Release creation is absent".to_owned())?;
    let public_verify = workflow
        .find("Publicly verify immutable tag, release bytes, and promoted image")
        .ok_or_else(|| "post-create public verifier is absent".to_owned())?;
    if !(requery < promotion && promotion < release_create && release_create < public_verify) {
        return Err("candidate re-query, digest promotion, release creation, and public verification are out of order".to_owned());
    }
    Ok(())
}

fn validate_recovery_workflow(workflow: &str) -> Result<(), String> {
    let document: Value = serde_yaml::from_str(workflow)
        .map_err(|error| format!("recovery workflow is not valid YAML: {error}"))?;
    let top = yaml_mapping(&document, "recovery workflow")?;
    exact_keys(
        top,
        &["name", "on", "concurrency", "permissions", "jobs"],
        "recovery workflow",
    )?;
    expect_text(top, "name", "lumen-release-recovery", "recovery workflow")?;
    let trigger = yaml_mapping(
        yaml_field(top, "on", "recovery workflow")?,
        "recovery workflow.on",
    )?;
    exact_keys(trigger, &["workflow_dispatch"], "recovery workflow.on")?;
    let dispatch = yaml_field(trigger, "workflow_dispatch", "recovery workflow.on")?;
    if !dispatch.is_null() {
        return Err("recovery workflow_dispatch must have no inputs".to_owned());
    }
    let concurrency = yaml_mapping(
        yaml_field(top, "concurrency", "recovery workflow")?,
        "recovery workflow.concurrency",
    )?;
    exact_keys(
        concurrency,
        &["group", "cancel-in-progress"],
        "recovery workflow.concurrency",
    )?;
    expect_text(
        concurrency,
        "group",
        "lumen-release-promotion-0.4.28",
        "recovery workflow.concurrency",
    )?;
    expect_bool(
        concurrency,
        "cancel-in-progress",
        false,
        "recovery workflow.concurrency",
    )?;
    validate_permissions(
        top,
        &[
            ("actions", "read"),
            ("attestations", "read"),
            ("contents", "write"),
            ("packages", "write"),
            ("pull-requests", "read"),
        ],
        "recovery workflow",
    )?;
    let jobs = yaml_mapping(
        yaml_field(top, "jobs", "recovery workflow")?,
        "recovery workflow.jobs",
    )?;
    exact_keys(jobs, &["recover"], "recovery workflow.jobs")?;
    let job = yaml_mapping(
        yaml_field(jobs, "recover", "recovery workflow.jobs")?,
        "recovery workflow.jobs.recover",
    )?;
    exact_keys(
        job,
        &["name", "runs-on", "steps"],
        "recovery workflow.jobs.recover",
    )?;
    expect_text(
        job,
        "name",
        "recover lumen@0.4.28 from frozen candidate",
        "recovery workflow.jobs.recover",
    )?;
    expect_text(
        job,
        "runs-on",
        "ubuntu-latest",
        "recovery workflow.jobs.recover",
    )?;
    let steps = yaml_sequence(
        yaml_field(job, "steps", "recovery workflow.jobs.recover")?,
        "recovery workflow.jobs.recover.steps",
    )?;
    if steps.len() != 12 {
        return Err(format!(
            "recovery workflow step count changed: {}",
            steps.len()
        ));
    }
    let source = workflow.to_ascii_lowercase();
    for forbidden in [
        "id-token:",
        "cargo build",
        "docker/build-push-action",
        "cosign sign",
        "cosign attest",
        "actions/attest",
        "git tag",
        "git push",
        "update-ref",
        "delete-ref",
        "git/refs",
        "--method patch",
        "force",
        "if: false",
        "kind",
        "gke",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "recovery workflow contains forbidden control: {forbidden}"
            ));
        }
    }
    for exact in [
        RECOVERY_TAG_OBJECT,
        RECOVERY_COMMIT,
        RECOVERY_RUN,
        RECOVERY_ROOT,
        RECOVERY_AMD64,
        RECOVERY_ARM64,
        RECOVERY_OLD_LATEST,
        "lumen-release-candidate-32974297012-1",
        "lumen@0.4.28",
    ] {
        if count(workflow, exact) == 0 {
            return Err(format!("recovery workflow lost frozen binding: {exact}"));
        }
    }
    for required in [
        "verify-release-artifacts.sh",
        "--mode candidate",
        "--mode public",
        "immediately before stable writes",
        "docker buildx imagetools create",
        "gh release create",
        "32980139617",
        "Recovery note",
        "candidate/lumen-aarch64-apple-darwin.tar.gz",
        "candidate/spdx-amd64.json",
        "candidate/spdx-arm64.json",
        "gh api repos/chrischeng-c4/axiom/git/ref/tags/lumen@0.4.28",
        "tag_object=",
    ] {
        if !workflow.contains(required) {
            return Err(format!(
                "recovery workflow missing required control: {required}"
            ));
        }
    }
    if !workflow
        .contains("lumen@sha256:59a85c96d807428c424ec8889ac830b14e02869da49c4b44ae12dcce3786d03d")
    {
        return Err("recovery promotion source is not the frozen digest reference".to_owned());
    }
    if count(workflow, "test -f candidate/") != 12
        || count(workflow, "docker buildx imagetools create") != 2
        || count(workflow, "gh release create") != 1
        || count(workflow, "--mode candidate") != 2
        || count(workflow, "--mode public") != 2
    {
        return Err("recovery workflow write or verifier inventory changed".to_owned());
    }
    for forbidden in ["steps.state.outputs.semver", "steps.state.outputs.latest"] {
        if workflow.contains(forbidden) {
            return Err(format!(
                "recovery workflow uses stale state output: {forbidden}"
            ));
        }
    }
    for required in [
        "GITHUB_WORKFLOW_REF",
        "chrischeng-c4/axiom/.github/workflows/lumen-release-recovery.yml@refs/heads/main",
        "[[ \"$GITHUB_REF\" == refs/heads/main ]]",
        "origin/main",
        "candidate_attempt == \"1\"",
        "semver tag did not bind frozen root",
        "latest tag did not bind frozen root",
        "published 0.4.27 root changed",
        "latest changed before write",
    ] {
        if !workflow.contains(required) {
            return Err(format!(
                "recovery workflow missing race control: {required}"
            ));
        }
    }
    if !workflow.contains("- name: Refuse non-main or workflow-identity dispatch") {
        return Err("recovery workflow lacks explicit main guard".to_owned());
    }
    let recheck = workflow
        .find("immediately before stable writes")
        .ok_or_else(|| "recovery recheck is absent".to_owned())?;
    let image = workflow
        .find("Promote exact frozen digest")
        .ok_or_else(|| "recovery digest promotion is absent".to_owned())?;
    let release = workflow
        .find("Create exact GitHub Release")
        .ok_or_else(|| "recovery release creation is absent".to_owned())?;
    let public = workflow
        .find("Publicly verify recovered release")
        .ok_or_else(|| "recovery public verifier is absent".to_owned())?;
    if !(recheck < image && image < release && release < public) {
        return Err("recovery controls are out of order".to_owned());
    }
    Ok(())
}

fn replace_last(text: &str, needle: &str, replacement: &str) -> String {
    let index = text.rfind(needle).expect("mutation needle must exist");
    format!(
        "{}{}{}",
        &text[..index],
        replacement,
        &text[index + needle.len()..]
    )
}

#[test]
fn promotion_workflow_is_semantically_frozen_and_exactly_hashed() {
    let workflow = include_str!("../../../.github/workflows/lumen-release.yml");
    validate_promotion_workflow(workflow)
        .expect("promotion workflow must satisfy the fail-closed contract");
    assert_eq!(sha256(&workflow_path()), RELEASE_WORKFLOW_SHA256, "promotion workflow bytes changed; review and update the semantic validator before changing this digest");
}

#[test]
fn promotion_workflow_rejects_high_risk_source_mutations() {
    let workflow = include_str!("../../../.github/workflows/lumen-release.yml");
    let yaml_comment = workflow.replacen(
        "        run: |\n          set -euo pipefail\n          cat > release-notes.md <<EOF",
        "        # run: |\n          set -euo pipefail\n          cat > release-notes.md <<EOF",
        1,
    );
    let shell_comment = workflow.replacen(
        "          cat > release-notes.md <<EOF",
        "          # cat > release-notes.md <<EOF",
        1,
    );
    let dead_heredoc = workflow
        .replacen(
            "          cat > release-notes.md <<EOF",
            "          if false; then\n          cat > release-notes.md <<EOF",
            1,
        )
        .replacen("          EOF\n", "          EOF\n          fi\n", 1);
    let quoted_opener = workflow.replacen(
        "          cat > release-notes.md <<EOF",
        "          printf '%s\\n' 'cat > release-notes.md <<EOF'",
        1,
    );
    let quoted_bullet = workflow.replacen(
        &format!("          {PUBLIC_COMPATIBILITY_NOTE}"),
        &format!("          printf '%s\\n' '{PUBLIC_COMPATIBILITY_NOTE}'"),
        1,
    );
    let quoted_create = workflow.replacen(
        "          gh release create ",
        "          printf '%s\\n' 'gh release create ",
        1,
    );
    let create_line = "          gh release create \"lumen@${{ inputs.version }}\" --repo \"$GITHUB_REPOSITORY\" --target \"$GITHUB_SHA\" --title \"lumen@${{ inputs.version }}\" --notes-file release-notes.md \\\n";
    let reordered = workflow.replacen(
        "          cat > release-notes.md <<EOF\n",
        &format!("{create_line}          cat > release-notes.md <<EOF\n"),
        1,
    );
    let generated_notes = workflow.replacen("--notes-file release-notes.md", "--generate-notes", 1);
    let mutations = vec![
        ("yaml comment", yaml_comment),
        ("shell comment", shell_comment),
        ("dead heredoc", dead_heredoc),
        ("quoted opener", quoted_opener),
        ("quoted bullet", quoted_bullet),
        ("quoted create", quoted_create),
        ("create before heredoc", reordered),
        ("generate-notes substitution", generated_notes),
        (
            "extra trigger",
            workflow.replacen(
                "on:\n  workflow_dispatch:",
                "on:\n  push:\n    branches: [main]\n  workflow_dispatch:",
                1,
            ),
        ),
        (
            "extra job",
            format!("{workflow}\n  unauthorized:\n    runs-on: ubuntu-latest\n    steps: []\n"),
        ),
        (
            "id token write",
            workflow.replacen("contents: write", "contents: write\n      id-token: write", 1),
        ),
        (
            "rebuild",
            workflow.replacen(
                "set -euo pipefail\n          image_repo=ghcr.io/chrischeng-c4/lumen",
                "set -euo pipefail\n          cargo build --release\n          image_repo=ghcr.io/chrischeng-c4/lumen",
                1,
            ),
        ),
        (
            "re-sign",
            workflow.replacen(
                "set -euo pipefail\n          image_repo=ghcr.io/chrischeng-c4/lumen",
                "set -euo pipefail\n          cosign sign ghcr.io/chrischeng-c4/lumen@sha256:deadbeef\n          image_repo=ghcr.io/chrischeng-c4/lumen",
                1,
            ),
        ),
        (
            "skipped immediate re-query",
            replace_last(workflow, "--mode candidate", "--mode public"),
        ),
        (
            "mutable candidate image promotion",
            workflow.replacen(
                "docker buildx imagetools create --tag \"$semver_ref\" \"$root_ref\"",
                "docker buildx imagetools create --tag \"$semver_ref\" \"${image_repo}:release-candidate\"",
                1,
            ),
        ),
        (
            "commented receipt decode",
            workflow.replacen(
                "          decode_exact \"$STANDALONE_GKE_RECEIPT_B64\"",
                "          # decode_exact \"$STANDALONE_GKE_RECEIPT_B64\"",
                1,
            ),
        ),
        (
            "quoted receipt decode",
            workflow.replacen(
                "          decode_exact \"$STANDALONE_GKE_RECEIPT_B64\"",
                "          printf '%s\\n' 'decode_exact \"$STANDALONE_GKE_RECEIPT_B64\"",
                1,
            ),
        ),
        (
            "no-op receipt decoder",
            workflow.replacen(
                "            printf '%s' \"$encoded\" | base64 --decode > \"$output\"",
                "            true # base64 --decode",
                1,
            ),
        ),
        (
            "divergent receipt sidecar",
            workflow.replacen(
                "--standalone-gke-receipt-sidecar \"${{ steps.gke_receipt.outputs.sidecar }}\"",
                "--standalone-gke-receipt-sidecar \"candidate/final-candidate-manifest.json.sha256\"",
                1,
            ),
        ),
        (
            "candidate artifact rewrite",
            workflow.replacen(
                "          gh run download",
                "          gh run upload\n          gh run download",
                1,
            ),
        ),
    ];
    for (name, mutation) in mutations {
        assert!(
            validate_promotion_workflow(&mutation).is_err(),
            "validator accepted forbidden mutation: {name}"
        );
    }
}

#[test]
fn recovery_workflow_is_frozen_and_rejects_high_risk_mutations() {
    let workflow = include_str!("../../../.github/workflows/lumen-release-recovery.yml");
    validate_recovery_workflow(workflow)
        .expect("recovery workflow must satisfy its frozen contract");
    assert_eq!(sha256(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/lumen-release-recovery.yml").as_path()), RECOVERY_WORKFLOW_SHA256, "recovery workflow bytes changed; review the semantic validator before changing this digest");
    let mutations = [
        ("extra trigger", workflow.replace("workflow_dispatch:\n", "push:\n    branches: [main]\n  workflow_dispatch:\n")),
        ("caller input", workflow.replace("workflow_dispatch:\n", "workflow_dispatch:\n    inputs:\n      version:\n        required: true\n        type: string\n")),
        ("non-main", workflow.replace("$GITHUB_REF\" == refs/heads/main", "$GITHUB_REF\" == refs/heads/release")),
        ("tag drift", workflow.replace(RECOVERY_TAG_OBJECT, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
        ("commit drift", workflow.replace(RECOVERY_COMMIT, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
        ("rebuild", workflow.replace("Promote exact frozen digest", "cargo build --release\n      - name: Promote exact frozen digest")),
        ("re-sign", workflow.replace("Promote exact frozen digest", "cosign sign image\n      - name: Promote exact frozen digest")),
        ("mutable source", workflow.replace("lumen@sha256:59a85c96d807428c424ec8889ac830b14e02869da49c4b44ae12dcce3786d03d", "lumen:release-candidate")),
        ("skipped recheck", workflow.replace("--mode candidate", "--mode public")),
        ("unsafe latest", workflow.replace(RECOVERY_OLD_LATEST, "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
        ("asset drift", workflow.replace("candidate/spdx-arm64.json", "candidate/extra.json")),
        ("note drift", workflow.replace("Recovery note:", "Changed note:")),
        ("missing public verifier", workflow.replace("Publicly verify recovered release", "Verify recovered release")),
        ("run drift", workflow.replace("32974297012", "32974297013")),
        ("attempt drift", workflow.replace("candidate_attempt == \"1\"", "candidate_attempt == \"2\"")),
        ("root drift", workflow.replace(RECOVERY_ROOT, "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
        ("amd64 drift", workflow.replace(RECOVERY_AMD64, "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
        ("arm64 drift", workflow.replace(RECOVERY_ARM64, "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
        ("workflow identity drift", workflow.replace("chrischeng-c4/axiom/.github/workflows/lumen-release-recovery.yml@refs/heads/main", "chrischeng-c4/axiom/.github/workflows/other.yml@refs/heads/main")),
        ("job skip fail-open", workflow.replace("Refuse non-main or workflow-identity dispatch", "if: false\n      - name: Refuse non-main or workflow-identity dispatch")),
        ("unsafe tag api write", workflow.replace("git/ref/tags/lumen@0.4.28 --jq", "git/refs/tags/lumen@0.4.28 --method PATCH --jq")),
        ("force tag mutation", workflow.replace("git merge-base --is-ancestor", "git update-ref refs/tags/lumen@0.4.28\n          git merge-base --is-ancestor")),
        ("omit semver post-write check", workflow.replace("semver tag did not bind frozen root", "semver check omitted")),
        ("omit latest post-write check", workflow.replace("latest tag did not bind frozen root", "latest check omitted")),
    ];
    for (name, mutation) in mutations {
        assert!(
            validate_recovery_workflow(&mutation).is_err(),
            "validator accepted recovery mutation: {name}"
        );
    }
}

#[test]
fn public_release_notes_are_bound_to_candidate_and_promotion_evidence() {
    let workflow = include_str!("../../../.github/workflows/lumen-release.yml");
    let verifier = include_str!("../scripts/verify-release-artifacts.sh");
    for note_line in [
        "- Source commit: $GITHUB_SHA",
        "- Pull request: ${{ needs.verify-inputs.outputs.pr_url }}",
        "- Candidate run: ${{ needs.verify-inputs.outputs.candidate_url }}",
        "- Promotion run: https://github.com/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID/attempts/$GITHUB_RUN_ATTEMPT",
        "- Root index digest: ${{ needs.verify-inputs.outputs.root_digest }}",
        "- linux/amd64 digest: ${{ needs.verify-inputs.outputs.amd64_digest }}",
        "- linux/arm64 digest: ${{ needs.verify-inputs.outputs.arm64_digest }}",
        PUBLIC_RELEASE_NOTE_STATEMENTS[0],
        PUBLIC_RELEASE_NOTE_STATEMENTS[1],
        PUBLIC_RELEASE_NOTE_STATEMENTS[2],
        PUBLIC_COMPATIBILITY_NOTE,
    ] {
        assert!(workflow.contains(note_line), "missing public release-note field: {note_line}");
    }
    for verifier_control in [
        "--json assets,isDraft,tagName,targetCommitish,url,body",
        "index(\"- Source commit: \" + $commit) != null",
        "index(\"- Pull request: \" + $pr_url) != null",
        "index(\"- Candidate run: \" + $candidate_url) != null",
        "index(\"- Root index digest: \" + $root) != null",
        "index(\"- linux/amd64 digest: \" + $amd64) != null",
        "index(\"- linux/arm64 digest: \" + $arm64) != null",
        "index(\"- Standalone GKE receipt SHA-256: \" + $receipt_sha256) != null",
        "index(\"- Release path: landed main -> untagged candidate verification -> protected annotated tag -> promotion of the same candidate digest.\") != null",
        "index(\"- Placement path: a non-empty nodeSelector with the default initialMachineType skips the legacy capacity catalog.\") != null",
        "index(\"- Legacy placement path: an empty selector, tolerations-only placement, or a non-default initialMachineType still requires lumen-system/lumen-capacity-catalog.\") != null",
        "index(\"- Compatibility: shipped Docker images default to durable segment storage at /var/lib/lumen/data; bare lumen serve stays ephemeral without --data-dir or LUMEN_DATA_DIR. A 0.4.28 segment volume upgrades one way on first 0.4.29 start; in-place downgrade is unsupported.\") != null",
        "^- Promotion run: https://github\\\\.com/",
        "public GitHub Release notes do not bind exact promotion evidence",
    ] {
        assert!(
            verifier.contains(verifier_control),
            "missing public release-note verifier control: {verifier_control}"
        );
    }
    for statement in PUBLIC_RELEASE_NOTE_STATEMENTS {
        let drifted = workflow.replacen(statement, "- Drifted release-note statement.", 1);
        assert_ne!(
            drifted, workflow,
            "release-note drift fixture must change bytes"
        );
        assert!(
            validate_promotion_workflow(&drifted).is_err(),
            "promotion validator accepted release-note drift: {statement}"
        );
    }
    let drifted = workflow.replacen(
        PUBLIC_RECEIPT_NOTE,
        "- Standalone GKE receipt SHA-256: mutable",
        1,
    );
    assert_ne!(
        drifted, workflow,
        "receipt-note drift fixture must change bytes"
    );
    assert!(
        validate_promotion_workflow(&drifted).is_err(),
        "promotion validator accepted a mutable receipt hash note"
    );
}

#[test]
fn public_release_note_body_helper_is_hermetic_and_fail_closed() {
    let verifier = include_str!("../scripts/verify-release-artifacts.sh");
    assert!(verifier.contains("verify_public_release_notes <<<\"$release_json\""));
    let valid = format!(r#"{{"body":"{PUBLIC_COMPATIBILITY_NOTE}"}}"#);
    assert!(
        execute_verifier_function(verifier, "verify_public_release_notes", &valid)
            .status
            .success(),
        "exact compatibility bullet was rejected"
    );
    let receipt_sha256 = "a".repeat(64);
    let receipt_note = format!("- Standalone GKE receipt SHA-256: {receipt_sha256}");
    let valid_with_receipt = format!(
        r#"{{"body":{}}}"#,
        serde_json::to_string(&format!("{PUBLIC_COMPATIBILITY_NOTE}\n{receipt_note}")).unwrap()
    );
    assert!(
        execute_verifier_function_with_arg(
            verifier,
            "verify_public_release_notes",
            &receipt_sha256,
            &valid_with_receipt,
        )
        .status
        .success(),
        "exact standalone GKE receipt hash note was rejected"
    );
    for (name, body) in [
        ("missing", PUBLIC_COMPATIBILITY_NOTE.to_owned()),
        (
            "commented",
            format!("{PUBLIC_COMPATIBILITY_NOTE}\n<!-- {receipt_note} -->"),
        ),
        (
            "quoted",
            format!("{PUBLIC_COMPATIBILITY_NOTE}\n> {receipt_note}"),
        ),
        (
            "divergent",
            format!(
                "{PUBLIC_COMPATIBILITY_NOTE}\n- Standalone GKE receipt SHA-256: {}",
                "b".repeat(64)
            ),
        ),
    ] {
        let input = format!(r#"{{"body":{}}}"#, serde_json::to_string(&body).unwrap());
        assert!(
            !execute_verifier_function_with_arg(
                verifier,
                "verify_public_release_notes",
                &receipt_sha256,
                &input,
            )
            .status
            .success(),
            "receipt-note helper accepted {name} body"
        );
    }
    let variants = [
        ("missing", "release notes".to_owned()),
        ("old", OLD_PUBLIC_COMPATIBILITY_NOTE.to_owned()),
        (
            "changed",
            PUBLIC_COMPATIBILITY_NOTE.replace("durable segment storage", "durable data storage"),
        ),
        (
            "generalized",
            PUBLIC_COMPATIBILITY_NOTE.replace("A 0.4.28 segment volume", "Every 0.4.28 volume"),
        ),
        (
            "additive generalized",
            format!("{PUBLIC_COMPATIBILITY_NOTE}\n- Compatibility: Every 0.4.28 volume upgrades one way."),
        ),
        (
            "duplicate exact note",
            format!("{PUBLIC_COMPATIBILITY_NOTE}\n{PUBLIC_COMPATIBILITY_NOTE}"),
        ),
        ("blockquote", format!("> {PUBLIC_COMPATIBILITY_NOTE}")),
        ("fenced", format!("```\n{PUBLIC_COMPATIBILITY_NOTE}\n```")),
        ("tilde fenced", format!("~~~\n{PUBLIC_COMPATIBILITY_NOTE}\n~~~")),
        (
            "html comment",
            format!("<!-- {PUBLIC_COMPATIBILITY_NOTE} -->"),
        ),
        (
            "indented blockquote",
            format!("    > {PUBLIC_COMPATIBILITY_NOTE}"),
        ),
        (
            "indented backtick fence",
            format!("    ```\n{PUBLIC_COMPATIBILITY_NOTE}\n    ```"),
        ),
        (
            "indented tilde fence",
            format!("\t~~~\n{PUBLIC_COMPATIBILITY_NOTE}\n\t~~~"),
        ),
        (
            "indented html comment",
            format!("    <!-- {PUBLIC_COMPATIBILITY_NOTE} -->"),
        ),
        (
            "embedded html comment",
            format!("<div><!--\n{PUBLIC_COMPATIBILITY_NOTE}\n--></div>"),
        ),
    ];
    for (name, body) in variants {
        let input = format!(r#"{{"body":{}}}"#, serde_json::to_string(&body).unwrap());
        assert!(
            !execute_verifier_function(verifier, "verify_public_release_notes", &input)
                .status
                .success(),
            "compatibility body helper accepted {name} variant"
        );
    }
}

#[test]
fn candidate_execution_inventory_is_exact_and_fail_closed() {
    let verifier = include_str!("../scripts/verify-release-artifacts.sh");
    validate_candidate_execution_bindings(verifier)
        .expect("candidate verifier must call the exact pagination and inventory helpers");
    let bypassed_candidate_check = verifier.replacen(
        "\nverify_candidate_supply_chain\n",
        "\ntrue # verify_candidate_supply_chain\n",
        1,
    );
    assert_eq!(
        validate_candidate_execution_bindings(&bypassed_candidate_check),
        Err("PROMOTION_VERIFIER_BYTES".to_owned())
    );

    let valid = candidate_jobs(CANDIDATE_EXECUTION_NAMES, None);
    assert!(
        execute_verifier_function(verifier, "validate_candidate_job_inventory", &valid)
            .status
            .success(),
        "exact successful execution inventory was rejected"
    );

    let mut fewer = CANDIDATE_EXECUTION_NAMES.to_vec();
    fewer.pop();
    let mut more = CANDIDATE_EXECUTION_NAMES.to_vec();
    more.push("extra execution");
    let mut duplicate = CANDIDATE_EXECUTION_NAMES.to_vec();
    *duplicate.last_mut().unwrap() = CANDIDATE_EXECUTION_NAMES[0];
    for (name, jobs) in [
        ("fewer executions", candidate_jobs(&fewer, None)),
        ("more executions", candidate_jobs(&more, None)),
        ("duplicate execution", candidate_jobs(&duplicate, None)),
        (
            "failed execution",
            candidate_jobs(CANDIDATE_EXECUTION_NAMES, Some(0)),
        ),
    ] {
        assert!(
            !execute_verifier_function(verifier, "validate_candidate_job_inventory", &jobs)
                .status
                .success(),
            "candidate execution inventory accepted {name} fixture"
        );
    }

    let binding = "validate_candidate_job_inventory <<<\"$jobs\"";
    let comment_shadow = format!("# {binding}\n{}", verifier.replacen(binding, "true", 1));
    assert!(
        validate_candidate_execution_bindings(&comment_shadow).is_err(),
        "comment shadow hid a disabled operative inventory check"
    );

    let fetch_lines = shell_function_lines(verifier, "fetch_candidate_receipt").unwrap();
    let job_assignment = fetch_lines
        .iter()
        .find(|line| line.starts_with("jobs=") && line.contains("flatten_paginated_jobs"))
        .unwrap();
    let dead_group =
        format!("if false; then\n    {job_assignment}\n    {binding}\n  fi\n  jobs='[]'");
    let dead_group = verifier.replacen(*job_assignment, &dead_group, 1);
    assert!(
        validate_candidate_execution_bindings(&dead_group).is_err(),
        "dead grouped bindings hid weakened operative fetch code"
    );

    let quoted = verifier.replacen(
        "validate_candidate_job_inventory <<<\"$jobs\"",
        "printf '%s\\n' 'validate_candidate_job_inventory <<<\"$jobs\"'",
        1,
    );
    assert!(
        validate_candidate_execution_bindings(&quoted).is_err(),
        "quoted inventory prose passed executable oracle"
    );
    let nested = verifier.replacen(
        "validate_candidate_job_inventory <<<\"$jobs\"",
        "noop_inventory() { :; }; noop_inventory # validate_candidate_job_inventory <<<\"$jobs\"",
        1,
    );
    assert!(
        validate_candidate_execution_bindings(&nested).is_err(),
        "nested no-op inventory passed executable oracle"
    );
}

#[test]
fn candidate_pagination_helpers_flatten_page_items_not_object_values() {
    let jobs = execute_page_flattener(
        "flatten_paginated_jobs",
        "{\"total_count\":3,\"jobs\":[{\"name\":\"a\"},{\"name\":\"b\"}]}\n{\"total_count\":3,\"jobs\":[{\"name\":\"c\"}]}\n",
    );
    let job_names = jobs
        .as_sequence()
        .unwrap()
        .iter()
        .map(|job| job["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(job_names, ["a", "b", "c"]);

    let artifacts = execute_page_flattener(
        "flatten_paginated_artifacts",
        "{\"total_count\":2,\"artifacts\":[{\"name\":\"one\"}]}\n{\"total_count\":2,\"artifacts\":[{\"name\":\"two\"}]}\n",
    );
    let artifact_names = artifacts
        .as_sequence()
        .unwrap()
        .iter()
        .map(|artifact| artifact["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(artifact_names, ["one", "two"]);

    let verifier = include_str!("../scripts/verify-release-artifacts.sh");
    let old_jobs = verifier.replacen(
        "flatten_paginated_jobs() { jq -cs '[.[] | .jobs[]]'; }",
        "flatten_paginated_jobs() { jq -cs '[.[][]]'; }",
        1,
    );
    let output = execute_verifier_function(
        &old_jobs,
        "flatten_paginated_jobs",
        "{\"total_count\":1,\"jobs\":[{\"name\":\"a\"}]}\n",
    );
    assert!(output.status.success());
    let wrong_shape: Value = serde_yaml::from_slice(&output.stdout).unwrap();
    assert!(
        wrong_shape
            .as_sequence()
            .unwrap()
            .iter()
            .any(|item| !item.is_mapping()),
        "object-value job flatten unexpectedly returned only job objects"
    );

    let old_artifacts = verifier.replacen(
        "flatten_paginated_artifacts() { jq -cs '[.[] | .artifacts[]]'; }",
        "flatten_paginated_artifacts() { jq -cs '[.[][].artifacts[]]'; }",
        1,
    );
    assert!(
        !execute_verifier_function(
            &old_artifacts,
            "flatten_paginated_artifacts",
            "{\"total_count\":1,\"artifacts\":[{\"name\":\"one\"}]}\n",
        )
        .status
        .success(),
        "object-value artifact flatten unexpectedly succeeded"
    );
}

#[test]
fn promotion_oracle_freezes_the_exact_tag_ruleset_shape() {
    let verifier = include_str!("../scripts/verify-release-artifacts.sh");
    for needle in [
        ".target == \"tag\" and .enforcement == \"active\"",
        ".conditions.ref_name.include == [\"refs/tags/lumen@*\"]",
        "[.rules[].type] | sort == [\"deletion\",\"update\"]",
        "(.bypass_actors // []) | length == 0",
        "exact active immutable lumen tag ruleset is absent",
    ] {
        assert!(
            verifier.contains(needle),
            "missing ruleset control: {needle}"
        );
    }
    assert!(!verifier.contains("\"creation\""));
}

#[test]
fn fixture_mode_requires_exact_candidate_bytes_and_private_home_binary() {
    let fixture = release_fixture();
    assert_eq!(
        sha256(
            &fixture
                .candidate
                .join("lumen-x86_64-unknown-linux-gnu.tar.gz"),
        ),
        sha256(&fixture.public.join("lumen-x86_64-unknown-linux-gnu.tar.gz")),
    );
    let output = run_fixture(&fixture);
    assert!(
        output.status.success(),
        "fixture verifier failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("LOCAL FIXTURE ONLY"));
}

#[test]
fn fixture_mode_accepts_root_or_matching_scheduled_child_in_the_0_4_29_receipt() {
    let fixture = gke_receipt_fixture();
    let output = run_gke_receipt_fixture(&fixture);
    assert!(
        output.status.success(),
        "root image receipt fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let mut amd64_receipt =
        serde_json::from_slice::<serde_json::Value>(&fs::read(&fixture.receipt).unwrap()).unwrap();
    let amd64 = amd64_receipt["candidate"]["amd64_digest"].clone();
    amd64_receipt["matrix"]["required_continuity"]["observed_runtime_image_digest"] = amd64;
    rewrite_receipt(&fixture, &amd64_receipt);
    let amd64_output = run_gke_receipt_fixture(&fixture);
    assert!(
        amd64_output.status.success(),
        "matching amd64 child receipt was rejected: {}",
        String::from_utf8_lossy(&amd64_output.stderr)
    );

    let arm64_fixture = gke_receipt_fixture();
    let mut arm64_receipt = serde_json::from_slice::<serde_json::Value>(
        &fs::read(&arm64_fixture.receipt).unwrap(),
    )
    .unwrap();
    let arm64 = arm64_receipt["candidate"]["arm64_digest"].clone();
    arm64_receipt["matrix"]["required_continuity"]["scheduled_node_arch"] = json!("arm64");
    arm64_receipt["matrix"]["required_continuity"]["scheduled_runtime_child_digest"] =
        arm64.clone();
    arm64_receipt["matrix"]["required_continuity"]["observed_runtime_image_digest"] = arm64;
    rewrite_receipt(&arm64_fixture, &arm64_receipt);
    let arm64_output = run_gke_receipt_fixture(&arm64_fixture);
    assert!(
        arm64_output.status.success(),
        "matching arm64 child receipt was rejected: {}",
        String::from_utf8_lossy(&arm64_output.stderr)
    );
}

#[test]
fn public_receipt_dispatch_decode_is_executable_and_fail_closed() {
    let fixture = gke_receipt_fixture();
    let receipt = fs::read(&fixture.receipt).unwrap();
    let sidecar = fs::read(&fixture.sidecar).unwrap();
    let receipt_b64 = base64::engine::general_purpose::STANDARD.encode(&receipt);
    let sidecar_b64 = base64::engine::general_purpose::STANDARD.encode(&sidecar);
    let receipt_sha256 = sha256(&fixture.receipt);
    let sidecar_sha256 = sha256(&fixture.sidecar);

    let (temp, output) = run_public_receipt_decoder(
        receipt_b64.clone(),
        receipt_sha256.clone(),
        sidecar_b64.clone(),
        sidecar_sha256.clone(),
    );
    assert!(
        output.status.success(),
        "valid Base64 receipt dispatch input failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read(temp.0.join("gke-receipt/lumen-standalone-gke-receipt.json")).unwrap(),
        receipt
    );
    assert_eq!(
        fs::read(
            temp.0
                .join("gke-receipt/lumen-standalone-gke-receipt.json.sha256"),
        )
        .unwrap(),
        sidecar
    );

    for (name, b64, sha, sidecar_b64_input, sidecar_sha) in [
        (
            "non-base64 receipt",
            "not!base64".to_owned(),
            receipt_sha256.clone(),
            sidecar_b64.clone(),
            sidecar_sha256.clone(),
        ),
        (
            "wrong receipt hash",
            receipt_b64.clone(),
            "0".repeat(64),
            sidecar_b64.clone(),
            sidecar_sha256.clone(),
        ),
        (
            "wrong sidecar hash",
            receipt_b64.clone(),
            receipt_sha256.clone(),
            sidecar_b64.clone(),
            "0".repeat(64),
        ),
        (
            "quoted base64 newline",
            format!("{}\n", receipt_b64),
            receipt_sha256.clone(),
            sidecar_b64.clone(),
            sidecar_sha256.clone(),
        ),
    ] {
        let (_, output) = run_public_receipt_decoder(b64, sha, sidecar_b64_input, sidecar_sha);
        assert!(
            !output.status.success(),
            "receipt decoder accepted {name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn fixture_mode_rejects_standalone_gke_receipt_schema_and_candidate_binding_drift() {
    macro_rules! assert_receipt_rejected {
        ($name:literal, $mutate:expr) => {{
            let fixture = gke_receipt_fixture();
            let mut receipt =
                serde_json::from_slice::<serde_json::Value>(&fs::read(&fixture.receipt).unwrap())
                    .unwrap();
            ($mutate)(&mut receipt);
            rewrite_receipt(&fixture, &receipt);
            let output = run_gke_receipt_fixture(&fixture);
            assert!(
                !output.status.success(),
                "standalone GKE receipt mutation passed: {}\n{}",
                $name,
                String::from_utf8_lossy(&output.stderr)
            );
        }};
    }

    assert_receipt_rejected!("v1 schema", |receipt: &mut serde_json::Value| {
        receipt["schema"] = json!("lumen.standalone-gke-receipt/v1");
    });
    assert_receipt_rejected!("missing top key", |receipt: &mut serde_json::Value| {
        receipt.as_object_mut().unwrap().remove("stage");
    });
    assert_receipt_rejected!("extra top key", |receipt: &mut serde_json::Value| {
        receipt["extra"] = json!(true);
    });
    assert_receipt_rejected!(
        "missing candidate key",
        |receipt: &mut serde_json::Value| {
            receipt["candidate"]
                .as_object_mut()
                .unwrap()
                .remove("manifest_sha256");
        }
    );
    assert_receipt_rejected!("extra candidate key", |receipt: &mut serde_json::Value| {
        receipt["candidate"]["timestamp"] = json!(0);
    });
    assert_receipt_rejected!(
        "missing controller CLI key",
        |receipt: &mut serde_json::Value| {
            receipt["candidate"]["controller_cli"]
                .as_object_mut()
                .unwrap()
                .remove("sha256");
        }
    );
    assert_receipt_rejected!(
        "extra controller CLI key",
        |receipt: &mut serde_json::Value| {
            receipt["candidate"]["controller_cli"]["archive"] = json!("unbound");
        }
    );
    assert_receipt_rejected!(
        "missing required continuity key",
        |receipt: &mut serde_json::Value| {
            receipt["matrix"]["required_continuity"]
                .as_object_mut()
                .unwrap()
                .remove("audience");
        }
    );
    assert_receipt_rejected!(
        "extra required continuity key",
        |receipt: &mut serde_json::Value| {
            receipt["matrix"]["required_continuity"]["timestamp"] = json!(0);
        }
    );
    assert_receipt_rejected!(
        "required continuity scalar",
        |receipt: &mut serde_json::Value| {
            receipt["matrix"]["required_continuity"] = json!("passed");
        }
    );
    assert_receipt_rejected!(
        "zero required continuity delta",
        |receipt: &mut serde_json::Value| {
            receipt["matrix"]["required_continuity"]["tokenreview_delta"] = json!(0);
        }
    );
    assert_receipt_rejected!(
        "fractional required continuity delta",
        |receipt: &mut serde_json::Value| {
            receipt["matrix"]["required_continuity"]["allowed_delta"] = json!(1.5);
        }
    );
    assert_receipt_rejected!(
        "mismatched observed runtime image",
        |receipt: &mut serde_json::Value| {
            receipt["matrix"]["required_continuity"]["observed_runtime_image_digest"] =
                json!(format!("sha256:{}", "4".repeat(64)));
        }
    );
    assert_receipt_rejected!("amd64 observed arm64 child", |receipt: &mut serde_json::Value| {
        let arm64 = receipt["candidate"]["arm64_digest"].clone();
        receipt["matrix"]["required_continuity"]["observed_runtime_image_digest"] = arm64;
    });
    assert_receipt_rejected!("wrong scheduled child", |receipt: &mut serde_json::Value| {
        let arm64 = receipt["candidate"]["arm64_digest"].clone();
        receipt["matrix"]["required_continuity"]["scheduled_runtime_child_digest"] = arm64;
    });
    assert_receipt_rejected!("unknown scheduled arch", |receipt: &mut serde_json::Value| {
        receipt["matrix"]["required_continuity"]["scheduled_node_arch"] = json!("s390x");
    });

    let arm64_fixture = gke_receipt_fixture();
    let mut arm64_receipt = serde_json::from_slice::<serde_json::Value>(
        &fs::read(&arm64_fixture.receipt).unwrap(),
    )
    .unwrap();
    let arm64 = arm64_receipt["candidate"]["arm64_digest"].clone();
    let amd64 = arm64_receipt["candidate"]["amd64_digest"].clone();
    arm64_receipt["matrix"]["required_continuity"]["scheduled_node_arch"] = json!("arm64");
    arm64_receipt["matrix"]["required_continuity"]["scheduled_runtime_child_digest"] =
        arm64.clone();
    arm64_receipt["matrix"]["required_continuity"]["observed_runtime_image_digest"] =
        arm64;
    rewrite_receipt(&arm64_fixture, &arm64_receipt);
    let arm64_output = run_gke_receipt_fixture(&arm64_fixture);
    assert!(
        arm64_output.status.success(),
        "valid arm64 scheduled child receipt was rejected: {}",
        String::from_utf8_lossy(&arm64_output.stderr)
    );
    arm64_receipt["matrix"]["required_continuity"]["observed_runtime_image_digest"] =
        amd64.clone();
    rewrite_receipt(&arm64_fixture, &arm64_receipt);
    assert!(
        !run_gke_receipt_fixture(&arm64_fixture).status.success(),
        "arm64 receipt accepted the amd64 observed child"
    );
    arm64_receipt["matrix"]["required_continuity"]["observed_runtime_image_digest"] =
        arm64_receipt["candidate"]["arm64_digest"].clone();
    arm64_receipt["matrix"]["required_continuity"]["scheduled_runtime_child_digest"] = amd64;
    rewrite_receipt(&arm64_fixture, &arm64_receipt);
    assert!(
        !run_gke_receipt_fixture(&arm64_fixture).status.success(),
        "arm64 receipt accepted the amd64 scheduled child"
    );

    assert_receipt_rejected!(
        "missing redaction key",
        |receipt: &mut serde_json::Value| {
            receipt["redaction"]
                .as_object_mut()
                .unwrap()
                .remove("canary_scan");
        }
    );
    assert_receipt_rejected!("extra redaction key", |receipt: &mut serde_json::Value| {
        receipt["redaction"]["cluster_name"] = json!("sensitive");
    });
    assert_receipt_rejected!(
        "redaction value drift",
        |receipt: &mut serde_json::Value| {
            receipt["redaction"]["token_retained"] = json!(true);
        }
    );
    for key in [
        "manifest_sha256",
        "root_digest",
        "amd64_digest",
        "arm64_digest",
        "run_id",
    ] {
        let fixture = gke_receipt_fixture();
        let mut receipt =
            serde_json::from_slice::<serde_json::Value>(&fs::read(&fixture.receipt).unwrap())
                .unwrap();
        receipt["candidate"][key] = json!(format!("mismatch-{key}"));
        rewrite_receipt(&fixture, &receipt);
        assert!(
            !run_gke_receipt_fixture(&fixture).status.success(),
            "candidate {key} drift passed"
        );
    }
    assert_receipt_rejected!(
        "wrong but listed controller target",
        |receipt: &mut serde_json::Value| {
            receipt["candidate"]["controller_cli"]["target"] = json!("aarch64-apple-darwin");
        }
    );
    assert_receipt_rejected!(
        "wrong controller hash",
        |receipt: &mut serde_json::Value| {
            receipt["candidate"]["controller_cli"]["sha256"] = json!("0".repeat(64));
        }
    );
}

#[test]
fn fixture_mode_rejects_standalone_gke_receipt_sidecar_size_and_version_bypasses() {
    for (name, rewrite) in [
        (
            "wrong filename",
            format!("{}  other.json\n", "0".repeat(64)),
        ),
        (
            "extra sidecar line",
            format!(
                "{}  lumen-standalone-gke-receipt.json\nextra\n",
                "0".repeat(64)
            ),
        ),
    ] {
        let fixture = gke_receipt_fixture();
        fs::write(&fixture.sidecar, rewrite).unwrap();
        let output = run_gke_receipt_fixture(&fixture);
        assert!(
            !output.status.success(),
            "standalone GKE receipt {name} sidecar passed"
        );
    }
    let fixture = gke_receipt_fixture();
    let mut oversized = fs::read(&fixture.receipt).unwrap();
    oversized.extend(std::iter::repeat_n(b' ', 16385));
    fs::write(&fixture.receipt, oversized).unwrap();
    fs::write(
        &fixture.sidecar,
        format!(
            "{}  lumen-standalone-gke-receipt.json\n",
            sha256(&fixture.receipt)
        ),
    )
    .unwrap();
    assert!(
        !run_gke_receipt_fixture(&fixture).status.success(),
        "oversized standalone GKE receipt passed"
    );

    let fixture = gke_receipt_fixture();
    let output = Command::new("bash")
        .args([
            "-c",
            "source \"$1\"; TAG=lumen@0.4.28; STANDALONE_GKE_RECEIPT=\"$2\"; STANDALONE_GKE_RECEIPT_SIDECAR=\"$3\"; validate_standalone_gke_receipt",
            "bash",
        ])
        .arg(release_script())
        .arg(&fixture.receipt)
        .arg(&fixture.sidecar)
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "a standalone GKE receipt was accepted for a pre-0.4.29 tag"
    );
}

#[test]
fn fixture_mode_rejects_changed_public_standalone_gke_receipt_bytes() {
    for (name, path) in [
        ("receipt", "lumen-standalone-gke-receipt.json"),
        ("sidecar", "lumen-standalone-gke-receipt.json.sha256"),
    ] {
        let fixture = gke_receipt_fixture();
        fs::write(
            fixture.release.public.join(path),
            format!("changed-{name}\n"),
        )
        .unwrap();
        let output = run_gke_receipt_fixture(&fixture);
        assert!(
            !output.status.success(),
            "changed public standalone GKE {name} bytes passed"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("public standalone GKE"),
            "changed public standalone GKE {name} did not reach the exact byte check: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn fixture_mode_rejects_v2_and_job_map_drift() {
    for (name, from, to) in [
        (
            "v2 schema",
            "cclab.lumen.candidate-manifest.v3",
            "cclab.lumen.candidate-manifest.v2",
        ),
        ("missing job key", "\"verify-libraries\":\"success\",", ""),
        (
            "extra job key",
            "\"verify-libraries\":\"success\",",
            "\"verify-libraries\":\"success\",\"extra\":\"success\",",
        ),
        (
            "failed library job",
            "\"verify-libraries\":\"success\"",
            "\"verify-libraries\":\"failure\"",
        ),
    ] {
        let fixture = release_fixture();
        let path = fixture.candidate.join("final-candidate-manifest.json");
        let manifest = fs::read_to_string(&path).unwrap();
        assert_eq!(manifest.matches(from).count(), 1, "{name} fixture target");
        let mutated = manifest.replacen(from, to, 1);
        assert_ne!(mutated, manifest, "{name} fixture must change bytes");
        fs::write(&path, mutated).unwrap();
        let digest = sha256(&path);
        fs::write(
            path.with_extension("json.sha256"),
            format!("{digest}  final-candidate-manifest.json\n"),
        )
        .unwrap();
        let output = run_fixture(&fixture);
        assert!(!output.status.success(), "{name} fixture passed");
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("candidate final receipt contract changed"),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn fixture_mode_rejects_a_public_archive_even_when_its_public_sidecar_matches() {
    let fixture = release_fixture();
    let archive = fixture.public.join("lumen-x86_64-unknown-linux-gnu.tar.gz");
    let sidecar = fixture
        .public
        .join("lumen-x86_64-unknown-linux-gnu.tar.gz.sha256");
    let stage = fixture
        .public
        .join("mutated-stage/lumen-x86_64-unknown-linux-gnu");
    fs::create_dir_all(&stage).unwrap();
    fs::write(stage.join("README.md"), "tampered public archive\n").unwrap();
    let binary = stage.join("lumen");
    fs::write(&binary, format!("#!/bin/sh\necho 'lumen {VERSION}'\n")).unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
    shell(
        Command::new("tar")
            .arg("-C")
            .arg(fixture.public.join("mutated-stage"))
            .arg("-czf")
            .arg(&archive)
            .arg("lumen-x86_64-unknown-linux-gnu"),
    );
    fs::write(
        &sidecar,
        format!(
            "{}  lumen-x86_64-unknown-linux-gnu.tar.gz\n",
            sha256(&archive)
        ),
    )
    .unwrap();
    let output = run_fixture(&fixture);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("public archive hash differs"));
}

#[test]
fn fixture_mode_rejects_a_public_checksum_sidecar_that_differs_from_the_receipt() {
    let fixture = release_fixture();
    fs::write(
        fixture
            .public
            .join("lumen-x86_64-unknown-linux-gnu.tar.gz.sha256"),
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  lumen-x86_64-unknown-linux-gnu.tar.gz\n",
    )
    .unwrap();
    let output = run_fixture(&fixture);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("release checksum mismatch"));
}
