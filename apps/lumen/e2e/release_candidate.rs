//! Static and local-fixture oracle for the run-scoped release candidate.
use serde_json::{json, Value};
use serde_yaml::Value as Yaml;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[derive(Debug, PartialEq, Eq)]
struct Finding(&'static str);

const ACTIONS: &[&str] = &[
    "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1",
    "dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c",
    "Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6",
    "actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02",
    "actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093",
    "docker/setup-qemu-action@c7c53464625b32c7a7e944ae62b3e17d2b600130",
    "docker/setup-buildx-action@8d2750c68a42422c14e847fe6c8ac0403b4cbd6f",
    "docker/login-action@c94ce9fb468520275223c153574b00df6fe4bcc9",
    "docker/build-push-action@53b7df96c91f9c12dcc8a07bcb9ccacbed38856a",
    "sigstore/cosign-installer@6f9f17788090df1f26f669e9d70d6ae9567deba6",
    "actions/attest@1e69f48acb82d1966a394da916b4c1698aa569d6",
    "anchore/sbom-action@e22c389904149dbc22b58101806040fa8d37a610",
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .into()
}
fn key(name: &str) -> Yaml {
    Yaml::String(name.into())
}
fn field<'a>(value: &'a Yaml, name: &str) -> Option<&'a Yaml> {
    value.as_mapping()?.get(&key(name))
}
fn job<'a>(workflow: &'a Yaml, name: &str) -> Option<&'a Yaml> {
    field(field(workflow, "jobs")?, name)
}
fn strings(value: Option<&Yaml>) -> Vec<&str> {
    match value {
        Some(Yaml::String(value)) => vec![value],
        Some(Yaml::Sequence(values)) => values.iter().filter_map(Yaml::as_str).collect(),
        _ => Vec::new(),
    }
}
fn require(ok: bool, code: &'static str) -> Result<(), Finding> {
    ok.then_some(()).ok_or(Finding(code))
}
fn replace_once(source: &str, from: &str, to: &str) -> String {
    assert_eq!(
        source.matches(from).count(),
        1,
        "mutation target {from:?} is not unique"
    );
    let changed = source.replacen(from, to, 1);
    assert_ne!(changed, source, "mutation did not change bytes");
    changed
}

fn validate_workflow(source: &str, dockerfile: &str) -> Result<(), Finding> {
    let workflow: Yaml = serde_yaml::from_str(source).map_err(|_| Finding("YAML"))?;
    let events = field(&workflow, "on")
        .and_then(Yaml::as_mapping)
        .ok_or(Finding("TRIGGER"))?;
    require(events.len() == 1, "TRIGGER")?;
    let dispatch = events
        .get(&key("workflow_dispatch"))
        .ok_or(Finding("TRIGGER"))?;
    let inputs = field(dispatch, "inputs").ok_or(Finding("INPUTS"))?;
    for name in ["version", "commit"] {
        let input = field(inputs, name).ok_or(Finding("INPUTS"))?;
        require(
            field(input, "required").and_then(Yaml::as_bool) == Some(true),
            "INPUTS",
        )?;
        require(
            field(input, "type").and_then(Yaml::as_str) == Some("string"),
            "INPUTS",
        )?;
    }
    let concurrency = field(&workflow, "concurrency").ok_or(Finding("CONCURRENCY"))?;
    require(
        field(concurrency, "group").and_then(Yaml::as_str)
            == Some("lumen-release-candidate-${{ inputs.version }}"),
        "CONCURRENCY",
    )?;
    require(
        field(concurrency, "cancel-in-progress").and_then(Yaml::as_bool) == Some(false),
        "CONCURRENCY",
    )?;
    let names = [
        "identity",
        "build",
        "ghcr-image-and-attest",
        "manifest",
        "verify-candidate",
        "kind-amd64",
        "kind-arm64",
        "result",
    ];
    let jobs = field(&workflow, "jobs")
        .and_then(Yaml::as_mapping)
        .ok_or(Finding("JOBS"))?;
    require(
        jobs.len() == names.len() && names.iter().all(|name| job(&workflow, name).is_some()),
        "JOBS",
    )?;
    let graph = [
        ("identity", &[][..]),
        ("build", &["identity"][..]),
        ("ghcr-image-and-attest", &["identity", "build"][..]),
        (
            "manifest",
            &["identity", "build", "ghcr-image-and-attest"][..],
        ),
        (
            "verify-candidate",
            &["identity", "manifest", "ghcr-image-and-attest"][..],
        ),
        (
            "kind-amd64",
            &["identity", "verify-candidate", "ghcr-image-and-attest"][..],
        ),
        (
            "kind-arm64",
            &["identity", "verify-candidate", "ghcr-image-and-attest"][..],
        ),
        (
            "result",
            &[
                "identity",
                "build",
                "manifest",
                "ghcr-image-and-attest",
                "verify-candidate",
                "kind-amd64",
                "kind-arm64",
            ][..],
        ),
    ];
    for (name, expected) in graph {
        require(
            strings(field(job(&workflow, name).unwrap(), "needs")) == expected,
            "GRAPH",
        )?;
    }
    let permissions = [
        (
            "identity",
            "actions: read\ncontents: read\npull-requests: read",
        ),
        ("build", "contents: read"),
        (
            "ghcr-image-and-attest",
            "attestations: write\ncontents: read\nid-token: write\npackages: write",
        ),
        ("manifest", "contents: read"),
        (
            "verify-candidate",
            "attestations: read\ncontents: read\npackages: read",
        ),
        ("kind-amd64", "contents: read\npackages: read"),
        ("kind-arm64", "contents: read\npackages: read"),
        ("result", "contents: read"),
    ];
    for (name, expected) in permissions {
        let actual = field(job(&workflow, name).unwrap(), "permissions")
            .and_then(Yaml::as_mapping)
            .map(|map| {
                map.iter()
                    .filter_map(|(k, v)| Some(format!("{}: {}", k.as_str()?, v.as_str()?)))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut actual = actual;
        actual.sort();
        let mut expected = expected.split('\n').map(str::to_owned).collect::<Vec<_>>();
        expected.sort();
        require(actual == expected, "PERMISSIONS")?;
    }
    for line in source.lines().filter_map(|line| {
        let line = line.trim();
        line.strip_prefix("- uses: ")
            .or_else(|| line.strip_prefix("uses: "))
    }) {
        let action = line.split_whitespace().next().unwrap_or("");
        require(ACTIONS.contains(&action), "ACTION_PIN")?;
        let (_, sha) = action.rsplit_once('@').ok_or(Finding("ACTION_PIN"))?;
        require(
            sha.len() == 40 && sha.chars().all(|c| c.is_ascii_hexdigit()),
            "ACTION_PIN",
        )?;
    }
    for needle in [
        "candidate must dispatch main", "REQUESTED_COMMIT", "REQUESTED_VERSION",
        "git merge-base --is-ancestor", "expected one merged main PR",
        "git ls-remote --exit-code --tags origin", "the exact release tag already exists",
        "release_query=\"query", "the exact GitHub Release already exists",
        "cargo test -p lumen --features operator --test capacity_catalog_client",
        "cargo test -p lumen --features operator --test capacity_catalog_contract",
        "cargo test -p lumen --test cli_convention", "cargo test -p lumen --test release_artifacts",
        "cargo test -p lumen --test release_candidate", "cargo test -p lumen",
        "cargo test -p lumen --features \"operator delegated-auth\"",
        "cargo test -p lumen --locked --features release --test release_feature_set",
        "bash apps/lumen/scripts/standalone-container-smoke.sh bind",
        "cargo test -p service-k8s", "cargo test -p raft-runtime",
        "bash scripts/raft-implementor-build.sh", "git -c core.fsmonitor=false diff --check",
        "python3 scripts/meta/test_readme_contract.py", "project_docs_contract.py check apps/lumen libs/service-k8s",
        "--image \"${{ needs.ghcr-image-and-attest.outputs.image_repo }}@${{ needs.ghcr-image-and-attest.outputs.root_digest }}\"",
        "LUMEN_E2E_EXPECTED_RUNTIME_DIGEST", "final-candidate-manifest.json",
        "Verify final receipt as local fixture only", "--mode local",
    ] {
        require(source.contains(needle), "GATES")?;
    }
    for forbidden in [
        "gh release create",
        "gh release publish",
        "gh release upload",
        "imagetools create",
        ":latest",
        "gke-gcloud-auth-plugin",
        "gcloud container clusters",
        "git tag ",
    ] {
        require(!source.contains(forbidden), "CANDIDATE_ONLY")?;
    }
    require(
        source.contains(
            "candidate_tag=release-candidate-${{ github.run_id }}-${{ github.run_attempt }}",
        ),
        "IMAGE",
    )?;
    require(source.contains("org.opencontainers.image.url=https://github.com/${{ github.repository }}/actions/runs/${{ github.run_id }}/attempts/${{ github.run_attempt }}"), "IMAGE")?;
    require(
        source.contains(".manifests | type == \"array\" and length == 2"),
        "DIGESTS",
    )?;
    require(
        source.contains("[.manifests[].digest] | unique | length == 2"),
        "DIGESTS",
    )?;
    require(source.contains("sort == [\"amd64\",\"arm64\"]"), "DIGESTS")?;
    require(
        source.contains(
            "\"$root\" != \"$amd64\" && \"$root\" != \"$arm64\" && \"$amd64\" != \"$arm64\"",
        ),
        "DIGESTS",
    )?;
    require(source.matches("cosign sign --yes").count() == 1, "ATTEST")?;
    require(
        source.matches("uses: actions/attest@").count() == 3,
        "ATTEST",
    )?;
    require(
        source.matches("uses: anchore/sbom-action@").count() == 2,
        "ATTEST",
    )?;
    require(source.contains("name: lumen-candidate-${{ matrix.target }}-${{ github.run_id }}-${{ github.run_attempt }}"), "ARTIFACTS")?;
    for target in [
        "aarch64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-musl",
    ] {
        require(source.contains(&format!("target: {target}")), "ARTIFACTS")?;
    }
    require(
        source.contains("schema:\"cclab.lumen.candidate-manifest.v2\""),
        "MANIFEST",
    )?;
    for binding in [
        "--arg run_id",
        "--arg run_attempt",
        "--arg run_url",
        "--arg source_ref",
        "--arg workflow_ref",
        "--argjson pr_number",
        "--arg pr_url",
    ] {
        require(source.contains(binding), "MANIFEST")?;
    }
    require(source.contains(". + {jobs:{identity:"), "MANIFEST")?;
    require(source.contains("LUMEN_E2E_EXPECTED_RUNTIME_DIGEST=\"${{ needs.ghcr-image-and-attest.outputs.amd64_digest }}\""), "KIND")?;
    require(source.contains("LUMEN_E2E_EXPECTED_RUNTIME_DIGEST=\"${{ needs.ghcr-image-and-attest.outputs.arm64_digest }}\""), "KIND")?;
    require(
        source.contains("--manifest-sidecar candidate/final-candidate-manifest.json.sha256"),
        "MANIFEST",
    )?;
    validate_dockerfile(dockerfile)
}

fn validate_dockerfile(source: &str) -> Result<(), Finding> {
    const DEBIAN: &str = "debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171";
    const DISTROLESS: &str = "gcr.io/distroless/static-debian12:nonroot@sha256:afa5c872c891853ca7fcf1f12c3edb23f7eeef36189728842dd51042ff57f7ab";
    let froms = source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("FROM "))
        .map(|line| {
            line.split_whitespace()
                .take(3)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>();
    require(
        froms
            == vec![
                format!("{DEBIAN} AS seed"),
                format!("{DEBIAN} AS binary-source-fetch"),
                format!("{DEBIAN} AS binary-source-staged"),
                "binary-source-${SOURCE} AS binary-source".to_string(),
                DISTROLESS.to_string(),
            ],
        "BASE_IMAGE",
    )
}

fn validate_verifier(source: &str, mode: u32) -> Result<(), Finding> {
    require(mode & 0o777 == 0o755, "MODE")?;
    for needle in [
        "`local` validates synthetic files only. It is not candidate acceptance.",
        "`full` also validates the run-scoped GHCR image", "cclab.lumen.candidate-manifest.v2",
        "run_url == (\"https://github.com/\" + $repo + \"/actions/runs/\" + $run_id + \"/attempts/\" + $attempt)",
        ".source_ref == \"refs/heads/main\"", ".workflow_ref == $workflow_ref",
        ".jobs == {identity:\"success\",build:\"success\",manifest:\"success\"",
        "archive members changed", "archive binary is not executable", "invalid SPDX 2.3 SBOM",
        "predicate == $sbom[0]", "--certificate-identity \"$EXPECTED_CERT_ID\"",
        "--cert-oidc-issuer https://token.actions.githubusercontent.com",
        "expected_run_url=\"https://github.com/${REPO}/actions/runs/${RUN_ID}/attempts/${RUN_ATTEMPT}\"",
        "org.opencontainers.image.url", "archive_sha256", "sidecar_sha256", "spdx-${sbom}.json",
        "candidate tag is not scoped to this run attempt", "LOCAL FIXTURE ONLY: artifacts verified; this is not candidate acceptance.",
    ] {
        require(source.contains(needle), "VERIFIER")?;
    }
    for forbidden in [
        "gh release create",
        "gh release publish",
        "cosign sign",
        "imagetools create",
    ] {
        require(!source.contains(forbidden), "VERIFIER_SIDE_EFFECT")?;
    }
    Ok(())
}

fn expect_workflow(source: &str, from: &str, to: &str, code: &'static str) {
    let changed = replace_once(source, from, to);
    assert_eq!(
        validate_workflow(&changed, &dockerfile()).unwrap_err(),
        Finding(code)
    );
}
fn expect_verifier(source: &str, from: &str, to: &str, code: &'static str) {
    let changed = replace_once(source, from, to);
    assert_eq!(
        validate_verifier(&changed, 0o755).unwrap_err(),
        Finding(code)
    );
}
fn workflow() -> String {
    fs::read_to_string(root().join(".github/workflows/lumen-release-candidate.yml")).unwrap()
}
fn verifier() -> (String, u32) {
    let path = root().join("apps/lumen/scripts/verify-release-candidate.sh");
    (
        fs::read_to_string(&path).unwrap(),
        fs::metadata(path).unwrap().permissions().mode(),
    )
}
fn dockerfile() -> String {
    fs::read_to_string(root().join("apps/lumen/Dockerfile.release")).unwrap()
}

#[test]
fn live_candidate_contract_is_fail_closed() {
    let source = workflow();
    validate_workflow(&source, &dockerfile()).expect("candidate workflow contract");
    let (script, mode) = verifier();
    validate_verifier(&script, mode).expect("candidate verifier contract");
    assert_eq!(validate_verifier(&script, 0o644), Err(Finding("MODE")));
}

#[test]
fn candidate_source_mutations_fail_with_stable_categories() {
    let source = workflow();
    for (from, to, code) in [
        (
            "description: Exact Lumen semver without the lumen@ prefix.\n        required: true",
            "description: Exact Lumen semver without the lumen@ prefix.\n        required: false",
            "INPUTS",
        ),
        (
            "needs: [identity, manifest, ghcr-image-and-attest]",
            "needs: [identity, ghcr-image-and-attest]",
            "GRAPH",
        ),
        (
            "attestations: read\n      contents: read\n      packages: read",
            "attestations: read\n      contents: read",
            "PERMISSIONS",
        ),
        (
            "candidate_tag=release-candidate-${{ github.run_id }}-${{ github.run_attempt }}",
            "candidate_tag=lumen@${{ needs.identity.outputs.version }}",
            "IMAGE",
        ),
        ("--mode local", "--mode full", "GATES"),
        (
            "cargo test -p lumen --test release_candidate",
            "true",
            "GATES",
        ),
        (
            "dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c",
            "dtolnay/rust-toolchain@stable",
            "ACTION_PIN",
        ),
        (
            "imagetools inspect --raw",
            "imagetools create",
            "CANDIDATE_ONLY",
        ),
    ] {
        expect_workflow(&source, from, to, code);
    }
    let (script, _) = verifier();
    for (from, to, code) in [
        (
            "LOCAL FIXTURE ONLY: artifacts verified; this is not candidate acceptance.",
            "CANDIDATE ACCEPTED",
            "VERIFIER",
        ),
        (
            ".source_ref == \"refs/heads/main\"",
            ".source_ref == \"refs/heads/dev\"",
            "VERIFIER",
        ),
        (
            "candidate tag is not scoped to this run attempt",
            "candidate tag accepted",
            "VERIFIER",
        ),
        ("cosign verify", "cosign sign", "VERIFIER_SIDE_EFFECT"),
    ] {
        expect_verifier(&script, from, to, code);
    }
    let docker = dockerfile();
    let changed = docker.replacen("FROM debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171", "FROM debian:bookworm-slim@sha256:0000000000000000000000000000000000000000000000000000000000000000", 1);
    assert_ne!(changed, docker);
    assert_eq!(validate_dockerfile(&changed), Err(Finding("BASE_IMAGE")));
    assert!(validate_dockerfile(&docker).is_ok());
}

fn sha(path: &Path) -> String {
    let output = Command::new("shasum")
        .args(["-a", "256"])
        .arg(path)
        .output()
        .unwrap();
    String::from_utf8(output.stdout)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .into()
}
fn write_manifest(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
    let digest = sha(path);
    fs::write(
        path.with_extension("json.sha256"),
        format!(
            "{digest}  {}\n",
            path.file_name().unwrap().to_string_lossy()
        ),
    )
    .unwrap();
}
fn local_fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let artifacts = dir.path();
    let stage = artifacts.join("stage");
    for target in [
        "aarch64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-musl",
    ] {
        let package = stage.join(format!("lumen-{target}"));
        fs::create_dir_all(&package).unwrap();
        let binary = package.join("lumen");
        fs::write(&binary, "#!/bin/sh\nprintf 'lumen 0.4.27\\n'\n").unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
        fs::write(package.join("README.md"), "fixture\n").unwrap();
        let archive = artifacts.join(format!("lumen-{target}.tar.gz"));
        assert!(Command::new("tar")
            .args([
                "-C",
                stage.to_str().unwrap(),
                "-czf",
                archive.to_str().unwrap(),
                &format!("lumen-{target}")
            ])
            .status()
            .unwrap()
            .success());
        let archive_sha = sha(&archive);
        fs::write(
            artifacts.join(format!("lumen-{target}.tar.gz.sha256")),
            format!("{archive_sha}  lumen-{target}.tar.gz\n"),
        )
        .unwrap();
    }
    for arch in ["amd64", "arm64"] {
        fs::write(
            artifacts.join(format!("spdx-{arch}.json")),
            r#"{"spdxVersion":"SPDX-2.3"}"#,
        )
        .unwrap();
    }
    let targets = [
        "aarch64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-musl",
    ];
    let artifacts_json: Vec<_> = targets.iter().map(|target| json!({"target":target,"archive":format!("lumen-{target}.tar.gz"),"archive_sha256":sha(&artifacts.join(format!("lumen-{target}.tar.gz"))),"sidecar":format!("lumen-{target}.tar.gz.sha256"),"sidecar_sha256":sha(&artifacts.join(format!("lumen-{target}.tar.gz.sha256")))})).collect();
    let manifest = json!({"schema":"cclab.lumen.candidate-manifest.v2","repository":"chrischeng-c4/axiom","workflow_path":".github/workflows/lumen-release-candidate.yml","workflow_id":42,"run_id":"7","run_attempt":"2","run_url":"https://github.com/chrischeng-c4/axiom/actions/runs/7/attempts/2","source_ref":"refs/heads/main","workflow_ref":"chrischeng-c4/axiom/.github/workflows/lumen-release-candidate.yml@refs/heads/main","commit":"0123456789012345678901234567890123456789","version":"0.4.27","tag":"lumen@0.4.27","candidate_tag":"release-candidate-7-2","pr":{"number":42,"url":"https://github.com/chrischeng-c4/axiom/pull/42"},"image":{"repository":"ghcr.io/chrischeng-c4/lumen","root_digest":format!("sha256:{}", "1".repeat(64)),"amd64_digest":format!("sha256:{}", "2".repeat(64)),"arm64_digest":format!("sha256:{}", "3".repeat(64))},"artifacts":artifacts_json,"sboms":{"amd64":{"file":"spdx-amd64.json","sha256":sha(&artifacts.join("spdx-amd64.json"))},"arm64":{"file":"spdx-arm64.json","sha256":sha(&artifacts.join("spdx-arm64.json"))}},"jobs":{"identity":"success","build":"success","manifest":"success","ghcr-image-and-attest":"success","verify-candidate":"success","kind-amd64":"success","kind-arm64":"success","result":"success"}});
    write_manifest(&artifacts.join("final-candidate-manifest.json"), &manifest);
    dir
}

fn replace_host_archive(dir: &Path, stage_name: &str, readme: bool, mode: u32, version: &str) {
    let target = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        _ => panic!("unsupported verifier host"),
    };
    let stage_root = dir.join(stage_name);
    let package = stage_root.join(format!("lumen-{target}"));
    fs::create_dir_all(&package).unwrap();
    let binary = package.join("lumen");
    fs::write(&binary, format!("#!/bin/sh\nprintf 'lumen {version}\\n'\n")).unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(mode)).unwrap();
    if readme {
        fs::write(package.join("README.md"), "fixture\n").unwrap();
    }
    let archive = dir.join(format!("lumen-{target}.tar.gz"));
    assert!(Command::new("tar")
        .args([
            "-C",
            stage_root.to_str().unwrap(),
            "-czf",
            archive.to_str().unwrap(),
            &format!("lumen-{target}")
        ])
        .status()
        .unwrap()
        .success());
    let archive_sidecar = dir.join(format!("lumen-{target}.tar.gz.sha256"));
    let archive_digest = sha(&archive);
    fs::write(
        &archive_sidecar,
        format!("{archive_digest}  lumen-{target}.tar.gz\n"),
    )
    .unwrap();
    let path = dir.join("final-candidate-manifest.json");
    let mut manifest: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let index = manifest["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .position(|item| item["target"] == target)
        .unwrap();
    manifest["artifacts"][index]["archive_sha256"] = json!(archive_digest);
    manifest["artifacts"][index]["sidecar_sha256"] = json!(sha(&archive_sidecar));
    write_manifest(&path, &manifest);
}

fn run_local(dir: &Path) -> Output {
    Command::new("bash")
        .arg(root().join("apps/lumen/scripts/verify-release-candidate.sh"))
        .args([
            "--repo",
            "chrischeng-c4/axiom",
            "--version",
            "0.4.27",
            "--commit",
            "0123456789012345678901234567890123456789",
            "--run-id",
            "7",
            "--run-attempt",
            "2",
            "--manifest",
        ])
        .arg(dir.join("final-candidate-manifest.json"))
        .args(["--manifest-sidecar"])
        .arg(dir.join("final-candidate-manifest.json.sha256"))
        .args(["--artifacts-dir"])
        .arg(dir)
        .args(["--mode", "local"])
        .output()
        .unwrap()
}

#[test]
fn local_final_receipt_and_negative_fixtures_are_executable() {
    assert!(run_local(local_fixture().path()).status.success());
    fn negative(name: &str, mutate: fn(&Path), needle: &str) {
        let dir = local_fixture();
        mutate(dir.path());
        let output = run_local(dir.path());
        assert!(!output.status.success(), "{name} mutation passed");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(needle),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    negative(
        "checksum",
        |dir| {
            let path = dir.join("final-candidate-manifest.json");
            let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            value["artifacts"][0]["archive_sha256"] = json!("0".repeat(64));
            write_manifest(&path, &value);
        },
        "archive checksum mismatch",
    );
    negative(
        "manifest sidecar",
        |dir| {
            fs::write(
                dir.join("final-candidate-manifest.json.sha256"),
                "0  final-candidate-manifest.json\n",
            )
            .unwrap();
        },
        "manifest sidecar does not bind",
    );
    negative(
        "SBOM",
        |dir| {
            fs::write(dir.join("spdx-amd64.json"), "{}\n").unwrap();
        },
        "SBOM checksum mismatch",
    );
    negative(
        "job result",
        |dir| {
            let path = dir.join("final-candidate-manifest.json");
            let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            value["jobs"]["kind-amd64"] = json!("failure");
            write_manifest(&path, &value);
        },
        "final candidate manifest does not bind",
    );
    negative(
        "run/ref",
        |dir| {
            let path = dir.join("final-candidate-manifest.json");
            let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            value["source_ref"] = json!("refs/heads/dev");
            write_manifest(&path, &value);
        },
        "candidate manifest bindings changed",
    );
    negative(
        "archive member",
        |dir| replace_host_archive(dir, "bad-stage", false, 0o755, "0.4.27"),
        "archive members changed",
    );
    negative(
        "executable mode",
        |dir| replace_host_archive(dir, "mode-stage", true, 0o644, "0.4.27"),
        "archive binary is not executable",
    );
    negative(
        "wrong version",
        |dir| replace_host_archive(dir, "version-stage", true, 0o755, "0.4.26"),
        "candidate binary version mismatch",
    );
}
