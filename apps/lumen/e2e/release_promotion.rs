//! Stable release-promotion oracle. Keep it independent from `release_candidate`.

use serde_yaml::{Mapping, Value};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const TAG: &str = "lumen@0.4.28";
const VERSION: &str = "0.4.28";
const COMMIT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const RUN_ID: &str = "123";
const RELEASE_WORKFLOW_SHA256: &str =
    "00ea2d2d6e0ec181096a89fe2689b9d44ccd36bb154984d587df302007c5a738";
const CHECKOUT: &str = "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1";
const COSIGN_INSTALLER: &str = "sigstore/cosign-installer@6f9f17788090df1f26f669e9d70d6ae9567deba6";
const SETUP_BUILDX: &str = "docker/setup-buildx-action@8d2750c68a42422c14e847fe6c8ac0403b4cbd6f";
const DOCKER_LOGIN: &str = "docker/login-action@c94ce9fb468520275223c153574b00df6fe4bcc9";
const PUBLIC_RELEASE_NOTE_STATEMENTS: &[&str] = &[
    "- Release path: landed main -> untagged candidate verification -> protected annotated tag -> promotion of the same candidate digest.",
    "- Placement path: a non-empty nodeSelector with the default initialMachineType skips the legacy capacity catalog.",
    "- Legacy placement path: an empty selector, tolerations-only placement, or a non-default initialMachineType still requires lumen-system/lumen-capacity-catalog.",
];
static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

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
    let stage = root.join("stage").join(format!("lumen-{target}"));
    fs::create_dir_all(&stage).unwrap();
    fs::write(stage.join("README.md"), "fixture\n").unwrap();
    let binary = stage.join("lumen");
    fs::write(&binary, format!("#!/bin/sh\necho 'lumen {VERSION}'\n")).unwrap();
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
        r#"{{"schema":"cclab.lumen.candidate-manifest.v2","repository":"chrischeng-c4/axiom","workflow_path":".github/workflows/lumen-release-candidate.yml","workflow_id":1,"run_id":"{RUN_ID}","run_attempt":"1","run_url":"https://github.com/chrischeng-c4/axiom/actions/runs/{RUN_ID}/attempts/1","source_ref":"refs/heads/main","workflow_ref":"chrischeng-c4/axiom/.github/workflows/lumen-release-candidate.yml@refs/heads/main","commit":"{COMMIT}","version":"{VERSION}","tag":"{TAG}","candidate_tag":"release-candidate-{RUN_ID}-1","pr":{{"number":1,"url":"https://github.com/chrischeng-c4/axiom/pull/1"}},"image":{{"repository":"ghcr.io/chrischeng-c4/lumen","root_digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","amd64_digest":"sha256:2222222222222222222222222222222222222222222222222222222222222222","arm64_digest":"sha256:3333333333333333333333333333333333333333333333333333333333333333"}},"artifacts":[{}],"sboms":{{"amd64":{{"file":"spdx-amd64.json","sha256":"{}"}},"arm64":{{"file":"spdx-arm64.json","sha256":"{}"}}}},"jobs":{{"identity":"success","build":"success","manifest":"success","ghcr-image-and-attest":"success","verify-candidate":"success","kind-amd64":"success","kind-arm64":"success","result":"success"}}}}"#,
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

fn workflow_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/lumen-release.yml")
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
        &["version", "candidate_run_id"],
        "workflow.on.workflow_dispatch.inputs",
    )?;
    for (name, description) in [
        ("version", "Exact Lumen semver without the lumen@ prefix."),
        (
            "candidate_run_id",
            "Exact successful lumen-release-candidate run ID.",
        ),
    ] {
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
    if verify_steps.len() != 7 {
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
    validate_run_step(
        &verify_steps[5],
        "workflow.jobs.verify-inputs.steps[5]",
        "Verify tag ruleset, candidate receipt, and supply chain",
        &["name", "env", "shell", "run"],
        &[
            "verify-release-artifacts.sh",
            "--mode candidate",
            "--output promotion-contract.json",
        ],
        None,
        None,
        true,
    )?;
    validate_run_step(
        &verify_steps[6],
        "workflow.jobs.verify-inputs.steps[6]",
        "Export immutable candidate contract",
        &["name", "id", "shell", "run"],
        &[
            ".commit == $commit",
            "candidate_run_id",
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
    if publish_steps.len() != 11 {
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
    validate_run_step(
        &publish_steps[4],
        "workflow.jobs.publish-release.steps[4]",
        "Re-query exact candidate immediately before stable writes",
        &["name", "env", "shell", "run"],
        &[
            "verify-release-artifacts.sh",
            "--mode candidate",
            ".root_digest == $root",
        ],
        None,
        None,
        true,
    )?;
    validate_run_step(
        &publish_steps[5],
        "workflow.jobs.publish-release.steps[5]",
        "Exit safely if this exact public release already exists",
        &["name", "id", "env", "shell", "run"],
        &["gh api graphql", "release(tagName", "exists=true"],
        None,
        Some("existing_release"),
        true,
    )?;
    validate_run_step(
        &publish_steps[6],
        "workflow.jobs.publish-release.steps[6]",
        "Verify existing public release without moving latest",
        &["name", "if", "env", "shell", "run"],
        &["verify-release-artifacts.sh", "--mode public"],
        Some("steps.existing_release.outputs.exists == 'true'"),
        None,
        true,
    )?;
    validate_run_step(
        &publish_steps[7],
        "workflow.jobs.publish-release.steps[7]",
        "Download exact candidate release bytes",
        &["name", "if", "env", "shell", "run"],
        &["gh run download", "final-candidate-manifest.json"],
        Some("steps.existing_release.outputs.exists == 'false'"),
        None,
        true,
    )?;
    validate_run_step(
        &publish_steps[8],
        "workflow.jobs.publish-release.steps[8]",
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
    validate_run_step(
        &publish_steps[9],
        "workflow.jobs.publish-release.steps[9]",
        "Create exact GitHub Release with candidate bytes",
        &["name", "if", "env", "shell", "run"],
        &[
            "gh release create",
            "candidate/lumen-*.tar.gz",
            "Compatibility: no API, CRD, or runtime-default migration.",
            PUBLIC_RELEASE_NOTE_STATEMENTS[0],
            PUBLIC_RELEASE_NOTE_STATEMENTS[1],
            PUBLIC_RELEASE_NOTE_STATEMENTS[2],
        ],
        Some("steps.existing_release.outputs.exists == 'false'"),
        None,
        true,
    )?;
    validate_run_step(
        &publish_steps[10],
        "workflow.jobs.publish-release.steps[10]",
        "Publicly verify immutable tag, release bytes, and promoted image",
        &["name", "if", "env", "shell", "run"],
        &["verify-release-artifacts.sh", "--mode public"],
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
        "gke",
        "gcloud",
        "kubectl",
        "kind create",
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
    let mutations = [
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
    ];
    for (name, mutation) in mutations {
        assert!(
            validate_promotion_workflow(&mutation).is_err(),
            "validator accepted forbidden mutation: {name}"
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
        "- Compatibility: no API, CRD, or runtime-default migration.",
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
        "index(\"- Release path: landed main -> untagged candidate verification -> protected annotated tag -> promotion of the same candidate digest.\") != null",
        "index(\"- Placement path: a non-empty nodeSelector with the default initialMachineType skips the legacy capacity catalog.\") != null",
        "index(\"- Legacy placement path: an empty selector, tolerations-only placement, or a non-default initialMachineType still requires lumen-system/lumen-capacity-catalog.\") != null",
        "index(\"- Compatibility: no API, CRD, or runtime-default migration.\") != null",
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
