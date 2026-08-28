// HANDWRITE-BEGIN gap="missing-generator:unit-test:dc8df8ae" tracker="standardize-gap-projects-lumen-tests-spec-gen-e2e-rs" reason="lumen spec gen e2e: drives the CLI to emit typed clients from lumen's own OpenAPI offline (py pydantic + sync/async h2c runtime, --lang emitter selection) plus plain `spec` OpenAPI passthrough. Not yet captured as unit-test units in lumen-tests.md; aw claim_code/fillback adoption hangs."
//! `lumen spec gen` — generate a typed client (ts/py/rust) from lumen's own
//! OpenAPI document, offline.
//!

use std::net::TcpListener;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

fn lumen() -> Command {
    Command::new(env!("CARGO_BIN_EXE_lumen"))
}

fn run_generated_python(
    manifest_or_dir: &Path,
    python_args: &[&str],
    uv_bin: Option<&Path>,
    envs: &[(&str, &str)],
) -> Result<std::process::Output, String> {
    let manifest_path = if manifest_or_dir.is_file() {
        manifest_or_dir.to_path_buf()
    } else {
        manifest_or_dir.join(".openapi-codegen.json")
    };

    let manifest_raw = std::fs::read_to_string(&manifest_path).map_err(|err| {
        format!(
            "generated Python toolchain: failed to read manifest at {}: {err}",
            manifest_path.display()
        )
    })?;

    let manifest: serde_json::Value = serde_json::from_str(&manifest_raw).map_err(|err| {
        format!(
            "generated Python toolchain: failed to parse manifest at {}: {err}",
            manifest_path.display()
        )
    })?;

    let target = manifest
        .get("target")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "generated Python toolchain: manifest missing 'target'".to_string())?;

    let language = manifest
        .get("language")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            format!("generated Python toolchain ({target}): manifest missing 'language'")
        })?;

    let compiler = manifest
        .get("compiler")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            format!("generated Python toolchain ({target}): manifest missing 'compiler'")
        })?;

    let minimum_version = manifest
        .get("minimum_version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            format!("generated Python toolchain ({target}): manifest missing 'minimum_version'")
        })?;

    let language_standard = manifest
        .get("language_standard")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            format!("generated Python toolchain ({target}): manifest missing 'language_standard'")
        })?;

    let runtime_dependencies = manifest
        .get("runtime_dependencies")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            format!(
                "generated Python toolchain ({target}): manifest missing 'runtime_dependencies'"
            )
        })?;

    if language != "python" {
        return Err(format!(
            "generated Python toolchain ({target}): invalid manifest language {language:?}"
        ));
    }
    if compiler != "python" {
        return Err(format!(
            "generated Python toolchain ({target}): invalid manifest compiler {compiler:?}"
        ));
    }
    let expected_target = format!("python-{minimum_version}");
    if target != expected_target {
        return Err(format!(
            "generated Python toolchain ({target}): manifest target does not match minimum_version {minimum_version:?}"
        ));
    }
    if language_standard != minimum_version {
        return Err(format!(
            "generated Python toolchain ({target}): manifest language_standard {language_standard:?} does not match minimum_version {minimum_version:?}"
        ));
    }

    let mut runtime_deps = Vec::new();
    for dep in runtime_dependencies {
        let dep_str = dep.as_str().ok_or_else(|| {
            format!(
                "generated Python toolchain ({target}): non-string runtime dependency in manifest: {dep:?}"
            )
        })?;
        runtime_deps.push(dep_str);
    }
    if runtime_deps.as_slice() != ["pydantic>=2"] {
        return Err(format!(
            "generated Python toolchain ({target}): unexpected runtime_dependencies {runtime_deps:?}"
        ));
    }

    let uv = uv_bin.unwrap_or_else(|| Path::new("uv"));
    let mut cmd = Command::new(uv);
    cmd.arg("run")
        .arg("--python")
        .arg(minimum_version)
        .arg("--no-project");

    for dep in &runtime_deps {
        cmd.arg("--with");
        cmd.arg(dep);
    }

    cmd.arg("python");
    for arg in python_args {
        cmd.arg(arg);
    }

    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.env("UV_MANAGED_PYTHON", "1");

    let output = match cmd.output() {
        Ok(output) => output,
        Err(err) => {
            return Err(format!(
                "generated Python toolchain ({target}): failed to start uv: {err}"
            ));
        }
    };

    if !output.status.success() {
        return Err(format!(
            "generated Python toolchain ({target}) failed with status: {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(output)
}

/// R1: `spec gen --lang py` writes pydantic + generated sync/async HTTP/2 runtime.
#[test]
fn gen_py_writes_pydantic_h2c_client() {
    let dir = tempfile::tempdir().unwrap();
    let status = lumen()
        .args(["spec", "gen", "--lang", "py", "--out"])
        .arg(dir.path())
        .status()
        .unwrap();
    assert!(status.success(), "spec gen --lang py failed");

    for f in [
        "models.py",
        "h2c_runtime.py",
        "client.py",
        "__init__.py",
        ".openapi-codegen.json",
    ] {
        assert!(dir.path().join(f).exists(), "missing {f}");
    }
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join(".openapi-codegen.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["target"], "python-3.14");
    assert_eq!(manifest["language"], "python");
    assert_eq!(manifest["minimum_version"], "3.14");
    let models = std::fs::read_to_string(dir.path().join("models.py")).unwrap();
    assert!(models.contains("BaseModel"), "models.py not pydantic");
    assert!(
        models.contains("RootModel"),
        "models.py missing pydantic RootModel for oneOf component unions"
    );
    assert!(models.contains("class "), "models.py has no model class");
    assert!(
        models.contains("class QueryNodeAnd(BaseModel):"),
        "QueryNode oneOf variants are not pydantic models"
    );
    assert!(
        models.contains("and_: list[QueryNode] = Field(alias=\"and\")"),
        "QueryNode keyword variant alias was not preserved"
    );
    let search_request_pos = models
        .find("class SearchRequest(BaseModel):")
        .expect("models.py missing SearchRequest");
    let batch_search_item_pos = models
        .find("BatchSearchItem = SearchRequest")
        .expect("models.py missing BatchSearchItem reference alias");
    assert!(
        search_request_pos < batch_search_item_pos,
        "reference alias must be emitted after its concrete model"
    );
    let model_path = dir.path().join("models.py");
    let model_path_str = model_path.to_str().unwrap();
    run_generated_python(
        dir.path(),
        &[
            "-c",
            "import pathlib, sys, pydantic; assert sys.version_info[:2] == (3, 14), f'expected Python 3.14, got {sys.version_info}'; assert int(pydantic.__version__.split('.')[0]) >= 2; path = pathlib.Path(sys.argv[1]); exec(compile(path.read_text(), str(path), 'exec'))",
            model_path_str,
        ],
        None,
        &[],
    )
    .expect("generated models.py failed execution under pinned Python toolchain");
    let runtime = std::fs::read_to_string(dir.path().join("h2c_runtime.py")).unwrap();
    assert!(
        runtime.contains("class H2CClient"),
        "runtime missing H2CClient"
    );
    assert!(
        runtime.contains("class H2CConnection"),
        "runtime missing connection/session layer"
    );
    assert!(
        runtime.contains("class AsyncH2CClient"),
        "runtime missing async client"
    );
    assert!(runtime.contains("TLS ALPN h2"), "runtime missing ALPN path");
    assert!(
        runtime.contains("def stream("),
        "runtime missing bidi stream surface"
    );
    let client = std::fs::read_to_string(dir.path().join("client.py")).unwrap();
    assert!(
        client.contains("H2CClient"),
        "client.py not wired to generated HTTP/2 runtime"
    );
    assert!(
        client.contains("class AsyncClient"),
        "client.py missing async typed client"
    );
    assert!(
        client.contains("class SupportsRequest"),
        "client.py missing httpx-compatible injection protocol"
    );
}

/// R2: `--lang` selects the emitter (ts → .ts set, rust → .rs set).
#[test]
fn gen_lang_selects_emitter() {
    for (lang, marker) in [("ts", "client.ts"), ("rust", "client.rs")] {
        let dir = tempfile::tempdir().unwrap();
        let status = lumen()
            .args(["spec", "gen", "--lang", lang, "--out"])
            .arg(dir.path())
            .status()
            .unwrap();
        assert!(status.success(), "spec gen --lang {lang} failed");
        assert!(dir.path().join(marker).exists(), "{lang}: missing {marker}");
    }
}

#[test]
fn gen_all_languages_opt_into_the_default_ksa_token_per_request() {
    for (lang, runtime, needles) in [
        (
            "ts",
            "runtime.ts",
            &[
                "node:fs/promises",
                "attachFileBearer",
                "unavailable in this runtime",
            ][..],
        ),
        (
            "py",
            "client.py",
            &[
                "Path(_FILE_BEARER_TOKEN_PATH).read_text",
                "_file_bearer_headers",
            ][..],
        ),
        (
            "rust",
            "client.rs",
            &["attach_file_bearer", "with_default_header"][..],
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let output = lumen()
            .args(["spec", "gen", "--lang", lang, "--out"])
            .arg(dir.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "spec gen --lang {lang} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(dir.path().join(".openapi-codegen.json").is_file());
        let source = std::fs::read_to_string(dir.path().join(runtime)).unwrap();
        assert!(
            source.contains("/var/run/secrets/kubernetes.io/serviceaccount/token"),
            "{lang} did not embed the Kubernetes default token path"
        );
        assert!(
            source.contains(".svc.cluster.local"),
            "{lang} did not bind automatic auth to cluster Service DNS"
        );
        assert!(
            !source.contains("/var/run/secrets/lumen.axiom.dev/token"),
            "{lang} mixed the Managed private-audience token contract into Standalone"
        );
        for needle in needles {
            assert!(source.contains(needle), "{lang} runtime missing {needle:?}");
        }
    }
}

/// An explicit target remains an auditable override of the project policy.
#[test]
fn gen_target_override_writes_the_requested_contract() {
    let dir = tempfile::tempdir().unwrap();
    let status = lumen()
        .args([
            "spec",
            "gen",
            "--lang",
            "py",
            "--target",
            "python-3.11",
            "--out",
        ])
        .arg(dir.path())
        .status()
        .unwrap();
    assert!(status.success(), "spec gen target override failed");

    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join(".openapi-codegen.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["target"], "python-3.11");
    assert_eq!(manifest["minimum_version"], "3.11");
}

/// R3: `lumen spec` (no subcommand) still prints the OpenAPI document unchanged.
#[test]
fn plain_spec_still_prints_openapi() {
    let out = lumen().arg("spec").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout, lumen::spec::openapi_json());
}

/// R4: generated Python h2c client drives a live Lumen public API journey.
#[test]
#[ignore = "AW EC gate: opens a local h2c listener; run explicitly with --ignored"]
fn generated_client_live_h2c_public_api_journey() {
    let dir = tempfile::tempdir().unwrap();
    let package_dir = dir.path().join("lumen_client");
    std::fs::create_dir(&package_dir).unwrap();

    let status = lumen()
        .args(["spec", "gen", "--lang", "py", "--out"])
        .arg(&package_dir)
        .status()
        .unwrap();
    assert!(status.success(), "spec gen --lang py failed");

    let port = free_local_port();
    let mut child = ChildGuard::spawn(port);

    let script = dir.path().join("generated_client_live_smoke.py");
    std::fs::write(&script, GENERATED_CLIENT_LIVE_SMOKE).unwrap();
    let endpoint = format!("http://127.0.0.1:{port}");
    let python_dir_str = dir.path().to_str().unwrap();
    let script_str = script.to_str().unwrap();

    let result = run_generated_python(
        &package_dir,
        &[script_str, &endpoint],
        None,
        &[("PYTHONPATH", python_dir_str)],
    );

    child.stop();
    result.expect("generated Python client live smoke failed");
}

#[test]
fn generated_python_toolchain_ignores_ambient_python3() {
    let dir = tempfile::tempdir().unwrap();
    let status = lumen()
        .args(["spec", "gen", "--lang", "py", "--out"])
        .arg(dir.path())
        .status()
        .unwrap();
    assert!(status.success(), "spec gen --lang py failed");

    let fake_bin_dir = tempfile::tempdir().unwrap();
    let sentinel = dir.path().join("ambient_python3_sentinel.txt");
    let fake_content = format!(
        "#!/bin/sh\necho ambient-python-invoked > \"{}\"\nexit 1\n",
        sentinel.display()
    );
    for name in ["python", "python3"] {
        let fake_python = fake_bin_dir.path().join(name);
        std::fs::write(&fake_python, &fake_content).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&fake_python).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&fake_python, perms).unwrap();
        }
    }

    let ambient_path = std::env::var("PATH").unwrap_or_default();
    let overridden_path = format!("{}:{}", fake_bin_dir.path().display(), ambient_path);

    let res = run_generated_python(
        dir.path(),
        &[
            "-c",
            "import sys, pydantic; assert sys.version_info[:2] == (3, 14)",
        ],
        None,
        &[("PATH", &overridden_path), ("UV_MANAGED_PYTHON", "0")],
    );

    assert!(
        res.is_ok(),
        "manifest-derived Python execution failed: {:?}",
        res.err()
    );
    assert!(
        !sentinel.exists(),
        "ambient python3 sentinel was created; ambient interpreter was executed instead of uv-managed Python"
    );
}

#[test]
fn generated_python_toolchain_reports_missing_uv() {
    let dir = tempfile::tempdir().unwrap();
    let status = lumen()
        .args(["spec", "gen", "--lang", "py", "--out"])
        .arg(dir.path())
        .status()
        .unwrap();
    assert!(status.success(), "spec gen --lang py failed");

    let nonexistent_uv = Path::new("/nonexistent/uv_binary_for_lumen_negative_test");
    let res = run_generated_python(
        dir.path(),
        &["-c", "print('hello')"],
        Some(nonexistent_uv),
        &[],
    );

    let err = res.expect_err("missing uv must report toolchain failure, not success");
    assert!(
        err.contains("failed to start uv"),
        "error missing 'failed to start uv': {err}"
    );
    assert!(
        err.contains("python-3.14"),
        "error missing target 'python-3.14': {err}"
    );
    assert!(
        err.contains("generated Python toolchain"),
        "error missing 'generated Python toolchain': {err}"
    );
}

#[test]
fn generated_python_toolchain_reports_missing_interpreter() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_json = r#"{
  "schema_version": 1,
  "generator": "openapi-codegen",
  "compiler": "python",
  "target": "python-3.99",
  "language": "python",
  "minimum_version": "3.99",
  "language_standard": "3.99",
  "module_system": null,
  "module_resolution": null,
  "strict": null,
  "transport": "generated-h2c-and-tls-alpn-h2",
  "runtime_dependencies": [
    "pydantic>=2"
  ]
}"#;
    std::fs::write(dir.path().join(".openapi-codegen.json"), manifest_json).unwrap();

    let res = run_generated_python(
        dir.path(),
        &["-c", "print('hello')"],
        None,
        &[("UV_PYTHON_DOWNLOADS", "never")],
    );

    let err = res.expect_err("missing interpreter 3.99 must report toolchain failure, not success");
    assert!(
        err.contains("3.99"),
        "error diagnostic missing version '3.99': {err}"
    );
    assert!(
        err.contains("No interpreter found for Python 3.99"),
        "error diagnostic does not prove the selected interpreter is missing: {err}"
    );
    assert!(
        err.contains("generated Python toolchain"),
        "error diagnostic missing 'generated Python toolchain': {err}"
    );
    assert!(
        err.contains("status"),
        "error diagnostic missing exit status: {err}"
    );
    assert!(
        err.contains("stderr:"),
        "error diagnostic missing captured stderr block: {err}"
    );
    assert!(
        !err.contains("SyntaxError"),
        "missing interpreter failure must not report SyntaxError: {err}"
    );
}

fn free_local_port() -> u16 {
    TcpListener::bind(("127.0.0.1", 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

struct ChildGuard {
    child: Child,
}

impl ChildGuard {
    fn spawn(port: u16) -> Self {
        let child = lumen()
            .args([
                "serve",
                "--host",
                "127.0.0.1",
                "--port",
                &port.to_string(),
                "--wal",
                "embedded",
                "--log-level",
                "warn",
            ])
            .env("LUMEN_AUTH", "off")
            .env("LUMEN_LOG_FORMAT", "json")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        thread::sleep(Duration::from_millis(100));
        Self { child }
    }

    fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.stop();
    }
}

const GENERATED_CLIENT_LIVE_SMOKE: &str = r#"
import sys
import time

from lumen_client import (
    Client,
    CreateCollectionRequest,
    DuplicatesRequest,
    FieldSpec,
    IndexItem,
    IndexRequest,
    QueryNode,
    SearchRequest,
)


def wait_ready(client: Client) -> None:
    last = None
    for _ in range(80):
        try:
            client.healthz()
            client.readyz()
            return
        except Exception as exc:
            last = exc
            time.sleep(0.1)
    raise RuntimeError(f"lumen did not become ready: {last!r}")


base_url = sys.argv[1]
collection_id = "generated_client_smoke"

with Client(base_url) as client:
    wait_ready(client)
    assert client.version() is not None
    client.metrics()

    try:
        client.drop_collection(collection_id=collection_id, force=True)
    except Exception:
        pass

    created = client.create_collection(
        collection_id=collection_id,
        body=CreateCollectionRequest(
            fields={
                "body": FieldSpec(type="text"),
                "email": FieldSpec(type="keyword"),
            }
        ),
    )
    assert created.collection_id == collection_id

    indexed = client.index(
        collection_id=collection_id,
        body=IndexRequest(
            items=[
                IndexItem(external_id="u1", field="body", value="blue search one"),
                IndexItem(external_id="u1", field="email", value="dup@example.test"),
                IndexItem(external_id="u2", field="body", value="green search two"),
                IndexItem(external_id="u2", field="email", value="dup@example.test"),
            ]
        ),
    )
    assert indexed.indexed == 4

    query = QueryNode.model_validate({"match": {"field": "body", "text": "blue", "op": "or"}})
    search = client.search(
        collection_id=collection_id,
        body=SearchRequest(query=query, limit=10, track_total=True),
    )
    assert [hit.external_id for hit in search.hits] == ["u1"], search

    dupes = client.duplicates(
        collection_id=collection_id,
        body=DuplicatesRequest(field="email", min_group_size=2),
    )
    assert dupes.groups and dupes.groups[0].external_ids == ["u1", "u2"], dupes

    stats = client.stats(collection_id=collection_id)
    assert stats.fields["body"].type == "text"

    client.drop_collection(collection_id=collection_id, force=True)
"#;
// HANDWRITE-END
