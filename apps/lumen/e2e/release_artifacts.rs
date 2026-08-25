//! Deterministic oracle for the Lumen release supply chain.
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub code: &'static str,
    pub detail: String,
}

const DEBIAN_BASE: &str =
    "debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171";
const DISTROLESS_BASE: &str =
    "gcr.io/distroless/static-debian12:nonroot@sha256:afa5c872c891853ca7fcf1f12c3edb23f7eeef36189728842dd51042ff57f7ab";
macro_rules! require {
    ($condition:expr, $code:expr, $detail:expr) => {
        if !$condition {
            return Err(Finding {
                code: $code,
                detail: $detail.into(),
            });
        }
    };
}
struct Inputs {
    kind: String,
    docs: String,
    dockerfile: String,
    installer: String,
    cargo: String,
    rendered: String,
}

#[rustfmt::skip]
fn root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().into() }
#[rustfmt::skip]
fn shell_fn<'a>(source: &'a str, name: &str) -> &'a str {
    let marker = format!("{name}() {{");
    source
        .split_once(&marker)
        .and_then(|(_, tail)| tail.split_once("\n}\n"))
        .map_or("", |v| v.0)
}
fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .and_then(|(_, tail)| tail.split_once(end))
        .map_or("", |v| v.0)
}

#[rustfmt::skip]
fn validate(input: &Inputs) -> Result<(), Finding> {
    let installer = &input.installer;
    for needle in ["|| die \"checksum download failed: ${sha_url}\"", "awk 'NR == 1 { print $1; exit }'", "[ \"${#expected}\" -eq 64 ]", "missing required checksum tool", "[ \"${actual}\" != \"${expected}\" ]", "actual_version=\"$(\"${bin}\" --version 2>/dev/null)\"", "[ \"${actual_version}\" = \"${expected_version}\" ]"] {
        require!(installer.contains(needle), "INSTALLER_INTEGRITY", format!("installer integrity proof missing: {needle}"));
    }
    require!(!installer.contains("Best-effort integrity check"), "INSTALLER_INTEGRITY", "installer must fail closed when the checksum is missing");
    let installer_checksum = between(installer, "# ---- download + verify", "# ---- extract + install");
    let installer_version = between(installer, "bin=\"${tmpdir}/lumen-${target}/lumen\"", "mkdir -p \"${INSTALL_DIR}\"");
    require!(!installer_checksum.contains("true ||") && !installer_checksum.contains("|| true") && !installer_version.contains("true ||") && !installer_version.contains("|| true"), "INSTALLER_INTEGRITY", "installer integrity control flow contains a bypass");

    let prebuilt = between(&input.kind, "if [[ \"$IMAGE_MODE\" == \"prebuilt\" ]]; then", "elif [[ \"$IMAGE_MODE\" != \"local\" ]]");
    for needle in ["requires LUMEN_E2E_MODE=operator", "^ghcr\\.io/chrischeng-c4/lumen@sha256:[0-9a-f]{64}$", "^sha256:[0-9a-f]{64}$", "^[0-9a-f]{8}$", "EXPECTED_RUNTIME_DIGEST\" != \"$ROOT_DIGEST", "cargo_ver=\"$(grep"] {
        require!(prebuilt.contains(needle), "KIND_INPUTS", format!("prebuilt input proof missing: {needle}"));
    }
    require!(!["docker build", "kind load docker-image", "docker login", "gh auth"].iter().any(|v| prebuilt.contains(v)), "KIND_PREBUILT_LOCAL", "prebuilt branch performs local or credential work");
    let loader = shell_fn(&input.kind, "build_and_load_image");
    let skipped = loader.find("return 0");
    let build = loader.rfind("\n  docker build -f");
    let load = loader.rfind("\n  kind load docker-image");
    require!(skipped.zip(build).is_some_and(|(a, b)| a < b) && skipped.zip(load).is_some_and(|(a, b)| a < b), "KIND_PREBUILT_LOCAL", "prebuilt does not return before build/load");
    let deploy = shell_fn(&input.kind, "deploy_via_operator");
    for needle in ["kubectl kustomize", "changed != 1", "old_images != 1", "mutable Lumen image remains", "kubectl apply -f \"$tmp_pinned\"", "spec:\n  image: ${IMAGE_TAG}"] {
        require!(deploy.contains(needle), "KIND_PINNING", format!("operator pinning proof missing: {needle}"));
    }
    require!(!deploy.contains("prepare_operator_capacity_fixture"), "KIND_CAPACITY", "kind must not install a fake capacity catalog");
    require!(!input.kind.contains("kind: ConfigMap") && !input.kind.contains("catalog.json") && !input.kind.contains("lumen.axiom.dev/capacity-profile"), "KIND_CAPACITY", "kind must not install a fake capacity catalog");
    require!(deploy.matches("      assert_native_placement\n").count() == 1 && deploy.matches("      assert_legacy_missing_catalog_event\n").count() == 1, "KIND_PLACEMENT", "operator deploy must run both placement proofs");
    let native = shell_fn(&input.kind, "assert_native_placement");
    for needle in ["kubectl -n \"$NAMESPACE\" get statefulset/\"${LUMEN_CR_NAME}\" -o json", ".spec.template.spec.nodeSelector[\"kubernetes.io/os\"]", "[[ \"$selector\" == \"linux\" ]]", "get configmap/lumen-capacity-catalog", "without a capacity catalog"] {
        require!(native.contains(needle), "KIND_PLACEMENT", format!("native placement proof missing: {needle}"));
    }
    let legacy = shell_fn(&input.kind, "assert_legacy_missing_catalog_event");
    for needle in ["local legacy_namespace=\"${NAMESPACE}-legacy\"", "--field-selector \"involvedObject.name=${legacy_name},reason=ReconcileFailed\"", "-n \"$legacy_namespace\" get events", "message // .note // \"\"", "[[ \"$message\" == *\"capacity catalog\"* ]]", "namespaced ReconcileFailed Event"] {
        require!(legacy.contains(needle), "KIND_PLACEMENT", format!("legacy catalog failure proof missing: {needle}"));
    }
    require!(input.kind.lines().any(|line| line == "BATCH_SIZE=1000") && input.kind.contains("MAX_INDEX_BATCH_SIZE=1000") && input.kind.contains("    --items-per-batch \"$BATCH_SIZE\" \\\n"), "KIND_INDEX_BATCH", "kind fixture generator must use the public HTTP index-batch cap");
    let fixture_cleanup = shell_fn(&input.kind, "cleanup");
    require!(fixture_cleanup.lines().any(|line| line.trim() == "cleanup_fixture_files \"$FIXTURE_FILE\""), "FIXTURE_CLEANUP_PATH", "cleanup must call the tested fixture cleanup helper");
    let fixture_flow = between(&input.kind, "# The fixture script emits one NDJSON doc per line", "index_all_batches() {");
    require!(fixture_flow.matches("\ndiscover_fixture_bodies\n").count() == 1, "FIXTURE_BATCH_PATH", "the index flow must call the tested fixture discovery helper once");
    let normalize = shell_fn(&input.kind, "normalize_runtime_image_id");
    for line in ["if [[ \"$raw\" =~ ^ghcr\\.io/chrischeng-c4/lumen@(sha256:[0-9a-f]{64})$ ]]; then", "elif [[ \"$raw\" =~ ^docker-pullable://ghcr\\.io/chrischeng-c4/lumen@(sha256:[0-9a-f]{64})$ ]]; then", "elif [[ \"$raw\" =~ ^(containerd|cri-o|docker)://(sha256:[0-9a-f]{64})$ ]]; then"] {
        require!(normalize.lines().any(|got| got.trim() == line), "KIND_RUNTIME_ID", format!("anchored runtime imageID form missing: {line}"));
    }
    require!(normalize.matches("if [[").count() == 3 && normalize.matches("elif [[").count() == 2 && normalize.matches("return 1").count() == 1, "KIND_RUNTIME_ID", "runtime imageID allowlist branch inventory changed");
    let runtime_digest = shell_fn(&input.kind, "runtime_digest_is_expected");
    require!(runtime_digest.contains("[[ \"$digest\" == \"$EXPECTED_RUNTIME_DIGEST\" ]]") && !runtime_digest.contains("$ROOT_DIGEST") && runtime_digest.matches("sha256").count() == 0, "KIND_RUNTIME_ID", "runtime digest must match only the exact platform child");
    let named_pods = shell_fn(&input.kind, "assert_named_pods");
    require!(named_pods.contains("normalized=\"$(normalize_runtime_image_id \"$runtime_id\")\" || die \"unrecognized $container runtime imageID: $runtime_id\"") && named_pods.contains("runtime_digest_is_expected \"$normalized\"") && named_pods.contains("not the expected platform child digest $EXPECTED_RUNTIME_DIGEST"), "KIND_RUNTIME_ID", "pod runtime identity must normalize fail closed and require the exact platform child");
    let identity_fn = shell_fn(&input.kind, "assert_cluster_identity");
    for needle in ["deploy/lumen-operator -o json", ".name == \"operator\" and .image == $image", "get lumen/", ".spec.image", "get statefulset/", ".name == \"server\" and .image == $image", "assert_named_pods \"$OPERATOR_NS\"", "assert_named_pods \"$NAMESPACE\"", "/version", "(.version | type) == \"string\"", ".version != \"unknown\""] {
        require!(identity_fn.contains(needle), "KIND_DESIRED_STATE", format!("identity surface missing: {needle}"));
    }
    let first = input.kind.find("step \"4a2. assert cluster identity and /version\" assert_cluster_identity");
    let mutation = input.kind.find("step \"4b. PUT /collections/users\" api_put_collection");
    require!(first.zip(mutation).is_some_and(|(a, b)| a < b), "KIND_ORDER", "identity must precede first API mutation");
    let second = input.kind.find("step \"6c. assert cluster identity and /version post-recovery\" assert_cluster_identity");
    let fresh = input.kind.find("step \"7a. PUT /collections/users after restart\" api_put_collection");
    require!(second.zip(fresh).is_some_and(|(a, b)| a < b), "KIND_POST_RESTART", "post-restart identity must precede fresh write");

    for needle in ["{{json .Manifest}}' | jq -er '.digest'", "[[ \"$RAW_DIGEST\" =~ ^sha256:[0-9a-f]{64}$ ]]", "IMAGE=\"ghcr.io/chrischeng-c4/lumen@${RAW_DIGEST}\"", "--candidate-run-id <id>", "--mode public", "--output /tmp/lumen-public-release.json", "protected annotated `lumen@<version>` tag", "discovery-only", "native amd64 and arm64 kind runs before publication"] {
        require!(input.docs.contains(needle), "DEPLOYMENT_DIGEST", format!("deployment proof missing: {needle}"));
    }
    let verify_docs = between(&input.docs, "Create one protected annotated `lumen@<version>` tag at the exact candidate\ncommit before promotion. Then verify the public release before deployment:", "Each release image carries");
    require!(verify_docs.contains("--candidate-run-id <id>") && verify_docs.contains("--mode public") && verify_docs.contains("--output /tmp/lumen-public-release.json"), "DEPLOYMENT_DIGEST", "published verifier must bind the candidate run and write its receipt");
    let fetch = between(&input.dockerfile, &format!("FROM {DEBIAN_BASE} AS binary-source-fetch"), &format!("FROM {DEBIAN_BASE} AS binary-source-staged"));
    let staged = between(&input.dockerfile, &format!("FROM {DEBIAN_BASE} AS binary-source-staged"), "FROM binary-source-${SOURCE} AS binary-source");
    let froms: Vec<_> = input.dockerfile.lines().filter_map(|line| line.trim().strip_prefix("FROM ")).map(|line| line.split_whitespace().take(3).collect::<Vec<_>>().join(" ")).collect();
    let expected_froms = vec![format!("{DEBIAN_BASE} AS seed"), format!("{DEBIAN_BASE} AS binary-source-fetch"), format!("{DEBIAN_BASE} AS binary-source-staged"), "binary-source-${SOURCE} AS binary-source".into(), DISTROLESS_BASE.into()];
    require!(froms == expected_froms, "DOCKERFILE_CONTRACT", "release Dockerfile base image inventory or digest changed");
    for needle in ["ARG SOURCE=fetch", "FROM binary-source-${SOURCE} AS binary-source", "ENV LUMEN_HOST=0.0.0.0", "ENTRYPOINT [\"/usr/local/bin/lumen\"]", "CMD [\"serve\"]"] {
        require!(input.dockerfile.contains(needle), "DOCKERFILE_CONTRACT", format!("Dockerfile release contract missing: {needle}"));
    }
    require!(input.dockerfile.matches("ARG TARGETARCH\n").count() == 2 && !input.dockerfile.contains("ARG TARGETARCH="), "TARGETARCH_SELECTION", "TARGETARCH must expose BuildKit's automatic target without a fallback");
    require!(fetch.contains("amd64) t=x86_64-unknown-linux-musl") && fetch.contains("arm64) t=aarch64-unknown-linux-musl") && staged.contains("COPY dist/linux/${TARGETARCH}/lumen /tmp/lumen"), "TARGETARCH_SELECTION", "TARGETARCH must select the matching fetched and staged binaries");
    require!(fetch.contains("releases/download/${LUMEN_VERSION}") && fetch.contains("sha256sum -c \"${asset}.sha256\"") && !["curl", "apt-get", "releases/download"].iter().any(|v| staged.contains(v)), "DOCKERFILE_CONTRACT", "fetch/staged sources are not isolated");
    let cargo: toml::Value = toml::from_str(&input.cargo).map_err(|e| Finding { code: "TOML_PARSE", detail: e.to_string() })?;
    let registered = cargo.get("test").and_then(toml::Value::as_array).map_or(0, |tests| tests.iter().filter(|t| t.get("name").and_then(toml::Value::as_str) == Some("release_artifacts") && t.get("path").and_then(toml::Value::as_str) == Some("e2e/release_artifacts.rs")).count());
    require!(registered == 1, "CARGO_REGISTRATION", "release oracle must be registered exactly once");
    for needle in ["ARG SOURCE=fetch", "ARG LUMEN_VERSION=lumen@9.9.9", "releases/download/${LUMEN_VERSION}", "sha256sum -c \"${asset}.sha256\"", "FROM binary-source-${SOURCE}", "--build-arg LUMEN_VERSION=lumen@9.9.9"] {
        require!(input.rendered.contains(needle), "PUBLIC_RENDER", format!("public render missing: {needle}"));
    }
    require!(!input.rendered.contains("SOURCE=staged"), "PUBLIC_RENDER", "public render selected staged source");
    Ok(())
}

#[rustfmt::skip]
fn render_release() -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_lumen")).args(["dockerfile", "render", "--variant", "release", "--version", "9.9.9"]).output().expect("run lumen dockerfile render");
    assert!(output.status.success(), "render failed: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).expect("render is UTF-8")
}
#[rustfmt::skip]
fn live() -> Inputs {
    let root = root();
    let read = |path: &str| fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("read {path}: {e}"));
    Inputs { kind: read("apps/lumen/scripts/kind-e2e.sh"), docs: read("apps/lumen/docs/deployment.md"), dockerfile: read("apps/lumen/Dockerfile.release"), installer: read("apps/lumen/install.sh"), cargo: read("apps/lumen/Cargo.toml"), rendered: render_release() }
}
#[rustfmt::skip]
fn replace_once(source: &str, from: &str, to: &str) -> String {
    assert_ne!(from, to, "mutation must change bytes");
    assert_eq!(source.matches(from).count(), 1, "mutation target {from:?} is not unique");
    let changed = source.replacen(from, to, 1); assert_ne!(changed, source); changed
}
#[rustfmt::skip]
fn replace_nth(source: &str, from: &str, to: &str, nth: usize) -> String {
    let offsets: Vec<_> = source.match_indices(from).map(|(at, _)| at).collect();
    let at = *offsets.get(nth).unwrap_or_else(|| panic!("mutation target {from:?} has only {} sites", offsets.len()));
    let changed = format!("{}{}{}", &source[..at], to, &source[at + from.len()..]); assert_ne!(changed, source); changed
}
fn function_replace(source: &str, name: &str, from: &str, to: &str) -> String {
    let marker = format!("{name}() {{");
    let start = source.find(&marker).expect("function exists");
    let tail = &source[start..];
    let end = start + tail.find("\n}\n").expect("function closes") + 3;
    let changed = replace_once(&source[start..end], from, to);
    format!("{}{}{}", &source[..start], changed, &source[end..])
}

fn run_runtime_image_id_fixture(kind: &str, raw: &str, root: &str, child: &str) -> Output {
    let normalize = shell_fn(kind, "normalize_runtime_image_id");
    let expected = shell_fn(kind, "runtime_digest_is_expected");
    let script = format!(
        "normalize_runtime_image_id() {{\n{normalize}\n}}\n\
         runtime_digest_is_expected() {{\n{expected}\n}}\n\
         ROOT_DIGEST=\"$2\"\n\
         EXPECTED_RUNTIME_DIGEST=\"$3\"\n\
         normalized=\"$(normalize_runtime_image_id \"$1\")\" && \
         runtime_digest_is_expected \"$normalized\""
    );
    Command::new("bash")
        .arg("-c")
        .arg(script)
        .arg("runtime-image-id-fixture")
        .arg(raw)
        .arg(root)
        .arg(child)
        .output()
        .expect("run runtime imageID fixture")
}

fn run_fixture_discovery(kind: &str, output: &Path) -> Result<Vec<PathBuf>, Finding> {
    let body = shell_fn(kind, "discover_fixture_bodies");
    require!(
        !body.is_empty(),
        "FIXTURE_BATCH_PATH",
        "discover_fixture_bodies is missing"
    );
    let script = format!(
        "discover_fixture_bodies() {{\n{body}\n}}\n\
         FIXTURE_FILE=\"$1\"\n\
         INDEX_BODIES=()\n\
         discover_fixture_bodies\n\
         if (( ${{#INDEX_BODIES[@]}} > 0 )); then printf '%s\\n' \"${{INDEX_BODIES[@]}}\"; fi"
    );
    let discovered = Command::new("bash")
        .arg("-c")
        .arg(script)
        .arg("fixture-discovery")
        .arg(output)
        .output()
        .map_err(|e| Finding {
            code: "FIXTURE_BATCH_PATH",
            detail: e.to_string(),
        })?;
    require!(
        discovered.status.success(),
        "FIXTURE_BATCH_PATH",
        String::from_utf8_lossy(&discovered.stderr)
    );
    Ok(String::from_utf8(discovered.stdout)
        .map_err(|e| Finding {
            code: "FIXTURE_BATCH_PATH",
            detail: e.to_string(),
        })?
        .lines()
        .map(PathBuf::from)
        .collect())
}

fn run_fixture_cleanup(kind: &str, output: &Path) -> Result<(), Finding> {
    let body = shell_fn(kind, "cleanup_fixture_files");
    require!(
        !body.is_empty(),
        "FIXTURE_CLEANUP_PATH",
        "cleanup_fixture_files is missing"
    );
    let script = format!("cleanup_fixture_files() {{\n{body}\n}}\ncleanup_fixture_files \"$1\"");
    let cleaned = Command::new("bash")
        .arg("-c")
        .arg(script)
        .arg("fixture-cleanup")
        .arg(output)
        .output()
        .map_err(|e| Finding {
            code: "FIXTURE_CLEANUP_PATH",
            detail: e.to_string(),
        })?;
    require!(
        cleaned.status.success(),
        "FIXTURE_CLEANUP_PATH",
        String::from_utf8_lossy(&cleaned.stderr)
    );
    let remaining = fs::read_dir(output.parent().expect("fixture output has a parent"))
        .map_err(|e| Finding {
            code: "FIXTURE_CLEANUP_PATH",
            detail: e.to_string(),
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path == output
                || path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.contains(".req.") && name.ends_with(".json"))
        })
        .collect::<Vec<_>>();
    require!(
        remaining.is_empty(),
        "FIXTURE_CLEANUP_PATH",
        format!("fixture cleanup left {remaining:?}")
    );
    Ok(())
}

fn verify_fixture_batch_paths(kind: &str, script: &Path, output: &Path) -> Result<(), Finding> {
    let generated = Command::new("python3")
        .arg(script)
        .args(["--count", "2", "--items-per-batch", "2", "--output"])
        .arg(output)
        .output()
        .map_err(|e| Finding {
            code: "FIXTURE_GENERATOR",
            detail: e.to_string(),
        })?;
    require!(
        generated.status.success(),
        "FIXTURE_GENERATOR",
        String::from_utf8_lossy(&generated.stderr)
    );

    let output_text = output.to_string_lossy();
    let stem = output_text.strip_suffix(".json").unwrap_or(&output_text);
    let mut expected = (0..2)
        .map(|batch| PathBuf::from(format!("{stem}.req.{batch:03}.json")))
        .collect::<Vec<_>>();
    expected.sort();

    let mut actual = run_fixture_discovery(kind, output)?;
    actual.sort();
    require!(
        actual == expected,
        "FIXTURE_BATCH_PATH",
        format!("expected {expected:?}, found {actual:?}")
    );

    let mut request_ids = Vec::new();
    for path in actual {
        let body: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).map_err(|e| Finding {
                code: "FIXTURE_BATCH_BODY",
                detail: format!("read {}: {e}", path.display()),
            })?)
            .map_err(|e| Finding {
                code: "FIXTURE_BATCH_BODY",
                detail: format!("parse {}: {e}", path.display()),
            })?;
        require!(
            body["items"]
                .as_array()
                .is_some_and(|items| items.len() == 2),
            "FIXTURE_BATCH_BODY",
            format!("{} does not contain two items", path.display())
        );
        request_ids.push(
            body["request_id"]
                .as_str()
                .ok_or_else(|| Finding {
                    code: "FIXTURE_BATCH_BODY",
                    detail: format!("{} has no request_id", path.display()),
                })?
                .to_string(),
        );
    }
    require!(
        request_ids.len() == 2 && request_ids[0] != request_ids[1],
        "FIXTURE_BATCH_BODY",
        "fixture batches must use distinct request IDs"
    );
    run_fixture_cleanup(kind, output)?;
    Ok(())
}

fn expect(mutated: Inputs, code: &'static str) {
    let finding = validate(&mutated).expect_err("negative mutation passed");
    assert_eq!(finding.code, code, "wrong finding: {finding:?}");
}

const RELEASE_TARGETS: [&str; 5] = [
    "aarch64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-musl",
];

fn release_host_target() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Some("aarch64-apple-darwin"),
        ("linux", "x86_64") => Some("x86_64-unknown-linux-gnu"),
        ("linux", "aarch64") => Some("aarch64-unknown-linux-gnu"),
        _ => None,
    }
}

fn sha256_file(path: &Path) -> String {
    for (program, args) in [
        ("sha256sum", vec![path.as_os_str()]),
        (
            "shasum",
            vec![
                std::ffi::OsStr::new("-a"),
                std::ffi::OsStr::new("256"),
                path.as_os_str(),
            ],
        ),
    ] {
        let Ok(output) = Command::new(program).args(args).output() else {
            continue;
        };
        if output.status.success() {
            return String::from_utf8(output.stdout)
                .expect("checksum output is UTF-8")
                .split_whitespace()
                .next()
                .expect("checksum output has a digest")
                .to_string();
        }
    }
    panic!("sha256sum or shasum is required for the release fixture");
}

fn write_checksum_sidecar(dir: &Path, target: &str) {
    let asset_name = format!("lumen-{target}.tar.gz");
    let digest = sha256_file(&dir.join(&asset_name));
    fs::write(
        dir.join(format!("{asset_name}.sha256")),
        format!("{digest}  {asset_name}\n"),
    )
    .expect("write checksum sidecar");
}

fn binary_fixture(version: &str, exit_code: i32) -> (tempfile::TempDir, &'static str) {
    let host = release_host_target().expect("release verifier runs on a supported host");
    let dir = tempfile::tempdir().expect("create release fixture");
    for target in RELEASE_TARGETS {
        let asset_name = format!("lumen-{target}.tar.gz");
        let asset = dir.path().join(&asset_name);
        let package = dir.path().join("stage").join(format!("lumen-{target}"));
        fs::create_dir_all(&package).expect("create release package");
        fs::write(package.join("README.md"), "release fixture\n").unwrap();
        let binary = package.join("lumen");
        fs::write(
            &binary,
            format!("#!/bin/sh\nprintf 'lumen {version}\\n'\nexit {exit_code}\n"),
        )
        .expect("write fixture binary");
        let mut permissions = fs::metadata(&binary).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&binary, permissions).unwrap();
        let status = Command::new("tar")
            .arg("-C")
            .arg(dir.path().join("stage"))
            .arg("-czf")
            .arg(&asset)
            .arg(format!("lumen-{target}"))
            .status()
            .expect("run tar");
        assert!(status.success(), "tar fixture failed");
        write_checksum_sidecar(dir.path(), target);
    }
    (dir, host)
}

fn binary_asset_names() -> Vec<String> {
    RELEASE_TARGETS
        .into_iter()
        .flat_map(|target| {
            let archive = format!("lumen-{target}.tar.gz");
            [archive.clone(), format!("{archive}.sha256")]
        })
        .collect()
}

fn write_executable(path: &Path, body: &str) {
    fs::write(path, body).expect("write executable fixture");
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

fn run_installer_fixture(assets: &Path, host: &str) -> (tempfile::TempDir, Output) {
    let run = tempfile::tempdir().expect("create installer run dir");
    let mock = run.path().join("mock");
    fs::create_dir(&mock).unwrap();
    write_executable(
        &mock.join("curl"),
        r#"#!/bin/sh
set -eu
out=""
url=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    -o) out="$2"; shift 2 ;;
    -H) shift 2 ;;
    -fsSL) shift ;;
    *) url="$1"; shift ;;
  esac
done
[ -n "$out" ]
cp "$LUMEN_TEST_ASSET_DIR/${url##*/}" "$out"
"#,
    );
    write_executable(&mock.join("gh"), "#!/bin/sh\nexit 1\n");
    let (os, arch) = match host {
        "aarch64-apple-darwin" => ("Darwin", "arm64"),
        "x86_64-unknown-linux-gnu" => ("Linux", "x86_64"),
        "aarch64-unknown-linux-gnu" => ("Linux", "aarch64"),
        other => panic!("unsupported installer fixture host: {other}"),
    };
    write_executable(
        &mock.join("uname"),
        &format!(
            "#!/bin/sh\ncase \"${{1:-}}\" in\n  -s) printf '{os}\\n' ;;\n  -m) printf '{arch}\\n' ;;\n  *) exit 2 ;;\nesac\n"
        ),
    );
    let path = format!(
        "{}:{}",
        mock.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let output = Command::new("sh")
        .arg(root().join("apps/lumen/install.sh"))
        .env("PATH", path)
        .env("LUMEN_VERSION", "lumen@9.9.9")
        .env("LUMEN_INSTALL", run.path().join("install"))
        .env("LUMEN_REPO", "chrischeng-c4/axiom")
        .env("LUMEN_TEST_ASSET_DIR", assets)
        .env_remove("GH_TOKEN")
        .env_remove("GITHUB_TOKEN")
        .output()
        .expect("run installer fixture");
    (run, output)
}

#[test]
fn installer_executes_checksum_and_version_failure_paths() {
    let (valid, host) = binary_fixture("9.9.9", 0);
    let (run, output) = run_installer_fixture(valid.path(), host);
    assert!(
        output.status.success(),
        "valid installer fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let installed = run.path().join("install/lumen");
    assert!(installed.is_file());
    let output = Command::new(&installed).arg("--version").output().unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "lumen 9.9.9\n");

    let (bad_checksum, host) = binary_fixture("9.9.9", 0);
    let sidecar = bad_checksum
        .path()
        .join(format!("lumen-{host}.tar.gz.sha256"));
    fs::write(
        &sidecar,
        format!("{}  lumen-{host}.tar.gz\n", "0".repeat(64)),
    )
    .unwrap();
    let (run, output) = run_installer_fixture(bad_checksum.path(), host);
    assert!(!output.status.success(), "bad installer checksum passed");
    assert!(!run.path().join("install/lumen").exists());

    let (missing_checksum, host) = binary_fixture("9.9.9", 0);
    fs::remove_file(
        missing_checksum
            .path()
            .join(format!("lumen-{host}.tar.gz.sha256")),
    )
    .unwrap();
    let (run, output) = run_installer_fixture(missing_checksum.path(), host);
    assert!(!output.status.success(), "missing checksum passed");
    assert!(!run.path().join("install/lumen").exists());

    let (wrong_version, host) = binary_fixture("9.9.8", 0);
    let (run, output) = run_installer_fixture(wrong_version.path(), host);
    assert!(!output.status.success(), "wrong installer version passed");
    assert!(!run.path().join("install/lumen").exists());

    let (nonzero_version, host) = binary_fixture("9.9.9", 17);
    let (run, output) = run_installer_fixture(nonzero_version.path(), host);
    assert!(
        !output.status.success(),
        "non-zero installer version passed"
    );
    assert!(!run.path().join("install/lumen").exists());
}

#[test]
fn release_asset_inventory_requires_exact_five_pairs() {
    let names = binary_asset_names();
    assert_eq!(names.len(), 10);
    assert!(RELEASE_TARGETS
        .iter()
        .all(|target| names.contains(&format!("lumen-{target}.tar.gz"))));

    let mut missing = names.clone();
    missing.pop();
    assert_ne!(missing.len(), 10);

    let mut duplicate = names.clone();
    duplicate.push(names[0].clone());
    assert_ne!(duplicate.len(), 10);

    let mut extra = names;
    extra.push("lumen-s390x-unknown-linux-gnu.tar.gz".into());
    extra.push("lumen-s390x-unknown-linux-gnu.tar.gz.sha256".into());
    assert_ne!(extra.len(), 10);
}

#[test]
fn live_release_artifacts_satisfy_contract() {
    validate(&live()).expect("live release artifacts satisfy the frozen contract");
}

#[test]
fn runtime_image_id_allowlist_and_digest_binding_are_executable() {
    let kind = live().kind;
    let root = format!("sha256:{}", "1".repeat(64));
    let child = format!("sha256:{}", "2".repeat(64));
    let third = format!("sha256:{}", "3".repeat(64));
    for raw in [
        format!("docker-pullable://ghcr.io/chrischeng-c4/lumen@{child}"),
        format!("cri-o://{child}"),
        format!("containerd://{child}"),
        format!("docker://{child}"),
    ] {
        assert!(
            run_runtime_image_id_fixture(&kind, &raw, &root, &child)
                .status
                .success(),
            "accepted runtime imageID failed: {raw}"
        );
    }
    for raw in [
        format!("ghcr.io/chrischeng-c4/lumen@{root}"),
        format!("containerd://{root}"),
        format!("docker://{root}"),
        format!("ghcr.io/other/lumen@{root}"),
        format!("unknown://{root}"),
        "ghcr.io/chrischeng-c4/lumen:0.4.27".into(),
        "ghcr.io/chrischeng-c4/lumen@sha256:1234".into(),
        format!("ghcr.io/chrischeng-c4/lumen@sha256:{}", "A".repeat(64)),
        format!("junk-ghcr.io/chrischeng-c4/lumen@{root}"),
        format!("ghcr.io/chrischeng-c4/lumen@{root}-junk"),
        format!("ghcr.io/chrischeng-c4/lumen@@{root}"),
        format!("ghcr.io/chrischeng-c4/lumen@{third}"),
    ] {
        assert!(
            !run_runtime_image_id_fixture(&kind, &raw, &root, &child)
                .status
                .success(),
            "rejected runtime imageID passed: {raw}"
        );
    }
}

#[test]
fn fixture_batch_paths_cover_gnu_and_bsd_mktemp_shapes() {
    let script = root().join("apps/lumen/scripts/load-fixture.py");
    let kind = fs::read_to_string(root().join("apps/lumen/scripts/kind-e2e.sh"))
        .expect("read kind e2e script");
    for name in [
        "lumen-fixture.ABCDEF.json",
        "lumen-fixture.XXXXXX.json.ABCDEF",
    ] {
        let dir = tempfile::tempdir().expect("create fixture path test dir");
        verify_fixture_batch_paths(&kind, &script, &dir.path().join(name))
            .unwrap_or_else(|finding| panic!("{name}: {finding:?}"));
    }

    let source = fs::read_to_string(&script).expect("read fixture generator");
    let mutated = replace_once(
        &source,
        "out_req = out_ndjson.with_suffix(\".req.json\")",
        "out_req = out_ndjson.with_suffix(\"\").with_suffix(\".req.json\")",
    );
    let dir = tempfile::tempdir().expect("create mutated fixture path test dir");
    let mutated_script = dir.path().join("load-fixture.py");
    fs::write(&mutated_script, mutated).expect("write mutated fixture generator");
    let finding = verify_fixture_batch_paths(
        &kind,
        &mutated_script,
        &dir.path().join("lumen-fixture.ABCDEF.json"),
    )
    .expect_err("old chained with_suffix path passed");
    assert_eq!(finding.code, "FIXTURE_BATCH_PATH", "{finding:?}");

    let mutated_discovery = function_replace(
        &kind,
        "discover_fixture_bodies",
        "\"${FIXTURE_FILE%.json}\".req.*.json",
        "\"$FIXTURE_FILE\".req.*.json",
    );
    let dir = tempfile::tempdir().expect("create mutated discovery test dir");
    let finding = verify_fixture_batch_paths(
        &mutated_discovery,
        &script,
        &dir.path().join("lumen-fixture.ABCDEF.json"),
    )
    .expect_err("wrong production discovery glob passed");
    assert_eq!(finding.code, "FIXTURE_BATCH_PATH", "{finding:?}");

    let mutated_cleanup = function_replace(
        &kind,
        "cleanup_fixture_files",
        "\"${fixture%.json}\".req.*.json",
        "\"$fixture\".req.*.json",
    );
    let dir = tempfile::tempdir().expect("create mutated cleanup test dir");
    let finding = verify_fixture_batch_paths(
        &mutated_cleanup,
        &script,
        &dir.path().join("lumen-fixture.ABCDEF.json"),
    )
    .expect_err("wrong production cleanup glob passed");
    assert_eq!(finding.code, "FIXTURE_CLEANUP_PATH", "{finding:?}");
}

#[test]
#[rustfmt::skip]
fn scoped_negative_mutations_fail_with_stable_findings() {
    let mut fixture = live(); fixture.kind = replace_once(&fixture.kind, "^ghcr\\.io/chrischeng-c4/lumen@sha256:[0-9a-f]{64}$", "^ghcr\\.io/chrischeng-c4/lumen:.+$"); expect(fixture, "KIND_INPUTS");
    let mut fixture = live(); fixture.kind = function_replace(&fixture.kind, "normalize_runtime_image_id", "^ghcr\\.io/chrischeng-c4/lumen@", "^ghcr\\.io/.+@"); expect(fixture, "KIND_RUNTIME_ID");
    let mut fixture = live(); fixture.kind = function_replace(&fixture.kind, "normalize_runtime_image_id", "^ghcr\\.io/chrischeng-c4/lumen@(sha256:[0-9a-f]{64})$", "^ghcr\\.io/chrischeng-c4/lumen@(sha256:[0-9a-f]{64})"); expect(fixture, "KIND_RUNTIME_ID");
    let mut fixture = live(); fixture.kind = function_replace(&fixture.kind, "runtime_digest_is_expected", "[[ \"$digest\" == \"$EXPECTED_RUNTIME_DIGEST\" ]]", "[[ \"$digest\" == \"$ROOT_DIGEST\" ]]"); expect(fixture, "KIND_RUNTIME_ID");
    let mut fixture = live(); fixture.kind = function_replace(&fixture.kind, "runtime_digest_is_expected", "[[ \"$digest\" == \"$EXPECTED_RUNTIME_DIGEST\" ]]", "[[ \"$digest\" == \"$ROOT_DIGEST\" || \"$digest\" == \"$EXPECTED_RUNTIME_DIGEST\" ]]"); expect(fixture, "KIND_RUNTIME_ID");
    let mut fixture = live(); fixture.kind = function_replace(&fixture.kind, "assert_named_pods", "normalized=\"$(normalize_runtime_image_id \"$runtime_id\")\"", "normalized=\"$ROOT_DIGEST\""); expect(fixture, "KIND_RUNTIME_ID");
    let mut fixture = live(); fixture.kind = replace_nth(&fixture.kind, "if [[ \"$IMAGE_MODE\" == \"prebuilt\" ]]; then\n", "if [[ \"$IMAGE_MODE\" == \"prebuilt\" ]]; then\n  docker login ghcr.io\n", 0); expect(fixture, "KIND_PREBUILT_LOCAL");
    let mut fixture = live(); fixture.kind = function_replace(&fixture.kind, "deploy_via_operator", "  echo \"   waiting for the Lumen CRD to be Established\"\n", "  prepare_operator_capacity_fixture\n  echo \"   waiting for the Lumen CRD to be Established\"\n"); expect(fixture, "KIND_CAPACITY");
    let mut fixture = live(); fixture.kind = function_replace(&fixture.kind, "deploy_via_operator", "      assert_native_placement\n", "      : native placement proof omitted\n"); expect(fixture, "KIND_PLACEMENT");
    let mut fixture = live(); fixture.kind = function_replace(&fixture.kind, "assert_native_placement", "[[ \"$selector\" == \"linux\" ]]", "[[ -n \"$selector\" ]]"); expect(fixture, "KIND_PLACEMENT");
    let mut fixture = live(); fixture.kind = function_replace(&fixture.kind, "deploy_via_operator", "      assert_legacy_missing_catalog_event\n", "      : legacy catalog proof omitted\n"); expect(fixture, "KIND_PLACEMENT");
    let mut fixture = live(); fixture.kind = function_replace(&fixture.kind, "assert_legacy_missing_catalog_event", "[[ \"$message\" == *\"capacity catalog\"* ]]", "[[ -n \"$message\" ]]"); expect(fixture, "KIND_PLACEMENT");
    let mut fixture = live(); fixture.kind = replace_once(&fixture.kind, "BATCH_SIZE=1000\n", "BATCH_SIZE=10000\n"); expect(fixture, "KIND_INDEX_BATCH");
    let mut fixture = live(); fixture.kind = replace_once(&fixture.kind, "    --items-per-batch \"$BATCH_SIZE\" \\\n", "    --items-per-batch 10000 \\\n"); expect(fixture, "KIND_INDEX_BATCH");
    let mut fixture = live(); fixture.kind = function_replace(&fixture.kind, "cleanup", "  cleanup_fixture_files \"$FIXTURE_FILE\"\n", "  : cleanup_fixture_files omitted\n"); expect(fixture, "FIXTURE_CLEANUP_PATH");
    let mut fixture = live(); fixture.kind = replace_once(&fixture.kind, "\ndiscover_fixture_bodies\nif [[ ${#INDEX_BODIES[@]} -eq 0 ]]; then", "\n: discover_fixture_bodies omitted\nif [[ ${#INDEX_BODIES[@]} -eq 0 ]]; then"); expect(fixture, "FIXTURE_BATCH_PATH");
    for (from, to) in [
        ("  op_json=\"$(kubectl -n \"$OPERATOR_NS\" get deploy/lumen-operator -o json)\"", "  op_json=\"$(jq -nc --arg image \"$IMAGE_TAG\" '{spec:{replicas:1,template:{spec:{containers:[{name:\"operator\",image:$image}]}}}}')\""),
        ("  cr_img=\"$(kubectl -n \"$NAMESPACE\" get lumen/\"${LUMEN_CR_NAME}\" -o jsonpath='{.spec.image}')\"", "  cr_img=\"$IMAGE_TAG\""),
        ("  sset_json=\"$(kubectl -n \"$NAMESPACE\" get statefulset/\"${LUMEN_CR_NAME}\" -o json)\"", "  sset_json=\"$(jq -nc --arg image \"$IMAGE_TAG\" '{spec:{replicas:1,template:{spec:{containers:[{name:\"server\",image:$image}]}}}}')\""),
        ("  assert_named_pods \"$OPERATOR_NS\" \"app.kubernetes.io/name=lumen-operator\" operator \"$op_replicas\"", "  : # operator Pod identity check omitted"),
        ("  assert_named_pods \"$NAMESPACE\" \"$APP_LABEL\" server \"$sset_replicas\"", "  : # serving Pod identity check omitted"),
    ] {
        let mut fixture = live(); fixture.kind = function_replace(&fixture.kind, "assert_cluster_identity", from, to); expect(fixture, "KIND_DESIRED_STATE");
    }
    let mut fixture = live(); fixture.kind = replace_once(&fixture.kind, "step \"4a2. assert cluster identity and /version\" assert_cluster_identity\nstep \"4b. PUT /collections/users\" api_put_collection", "step \"4b. PUT /collections/users\" api_put_collection\nstep \"4a2. assert cluster identity and /version\" assert_cluster_identity"); expect(fixture, "KIND_ORDER");
    let mut fixture = live(); fixture.kind = replace_once(&fixture.kind, "step \"6c. assert cluster identity and /version post-recovery\" assert_cluster_identity", "echo post-restart-identity-omitted"); expect(fixture, "KIND_POST_RESTART");
    let mut fixture = live(); fixture.cargo = replace_once(&fixture.cargo, "name = \"release_artifacts\"", "name = \"release_artifacts_disabled\""); expect(fixture, "CARGO_REGISTRATION");
    let mut fixture = live(); fixture.installer = replace_once(&fixture.installer, "|| die \"checksum download failed: ${sha_url}\"", "|| true"); expect(fixture, "INSTALLER_INTEGRITY");
    let mut fixture = live(); fixture.installer = replace_once(&fixture.installer, "expected=\"$(awk 'NR == 1 { print $1; exit }' \"${tmpdir}/${asset}.sha256\")\"", "expected=\"$(cat \"${tmpdir}/${asset}.sha256\")\""); expect(fixture, "INSTALLER_INTEGRITY");
    let mut fixture = live(); fixture.installer = replace_once(&fixture.installer, "[ \"${actual_version}\" = \"${expected_version}\" ]", "true"); expect(fixture, "INSTALLER_INTEGRITY");
    let mut fixture = live(); fixture.installer = replace_once(&fixture.installer, "[ \"${actual_version}\" = \"${expected_version}\" ]", "true || [ \"${actual_version}\" = \"${expected_version}\" ]"); expect(fixture, "INSTALLER_INTEGRITY");
    let mut fixture = live(); fixture.dockerfile = replace_once(&fixture.dockerfile, "ARG SOURCE=fetch", "ARG SOURCE=staged"); expect(fixture, "DOCKERFILE_CONTRACT");
    let mut fixture = live(); fixture.dockerfile = replace_once(&fixture.dockerfile, &format!("FROM {DEBIAN_BASE} AS seed"), "FROM debian:bookworm-slim AS seed"); expect(fixture, "DOCKERFILE_CONTRACT");
    let mut fixture = live(); fixture.dockerfile = replace_once(&fixture.dockerfile, &format!("FROM {DISTROLESS_BASE}"), "FROM gcr.io/distroless/static-debian12:nonroot@sha256:0000000000000000000000000000000000000000000000000000000000000000"); expect(fixture, "DOCKERFILE_CONTRACT");
    for nth in 0..2 {
        let mut fixture = live(); fixture.dockerfile = replace_nth(&fixture.dockerfile, "ARG TARGETARCH\n", "ARG TARGETARCH=amd64\n", nth); expect(fixture, "TARGETARCH_SELECTION");
    }
    let mut fixture = live(); fixture.dockerfile = replace_once(&fixture.dockerfile, "COPY dist/linux/${TARGETARCH}/lumen /tmp/lumen", "COPY dist/linux/amd64/lumen /tmp/lumen"); expect(fixture, "TARGETARCH_SELECTION");
    let mut fixture = live(); fixture.dockerfile = replace_once(&fixture.dockerfile, "arm64) t=aarch64-unknown-linux-musl", "arm64) t=x86_64-unknown-linux-musl"); expect(fixture, "TARGETARCH_SELECTION");
}
