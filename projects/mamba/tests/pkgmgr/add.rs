//! CLI integration tests for `mamba add` — closes the runtime side of
//! tests/governance/gates/pkgmgr/add/manifest.toml (#2681).
//!
//! Pinned acceptance:
//!
//!   1. Add records the requested dependency deterministically (manifest
//!      and lockfile both byte-identical on replay).
//!   2. Missing package against a configured frozen index fails exit 1
//!      with stderr containing "not found", and does NOT mutate the
//!      manifest or lockfile.
//!   3. Offline — no $HOME / global cache state is touched.

use std::path::{Path, PathBuf};
use std::process::Command;

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

fn run_init(workdir: &Path) -> std::process::Output {
    Command::new(mamba_bin())
        .arg("init")
        .current_dir(workdir)
        .output()
        .expect("spawn mamba init")
}

fn run_add(workdir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(mamba_bin())
        .arg("add")
        .args(args)
        .current_dir(workdir)
        .output()
        .expect("spawn mamba add")
}

/// PEP 503 normalize used by `mamba add` when keying the index dir.
fn normalize_pep503(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_sep = false;
    for c in name.chars() {
        let is_sep = c == '-' || c == '_' || c == '.';
        if is_sep {
            if !prev_sep && !out.is_empty() {
                out.push('-');
            }
            prev_sep = true;
        } else {
            out.push(c.to_ascii_lowercase());
            prev_sep = false;
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    out
}

fn frozen_index_with(pkg: &str, versions: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let pkg_dir = dir.path().join(normalize_pep503(pkg));
    std::fs::create_dir(&pkg_dir).unwrap();
    for v in versions {
        std::fs::create_dir(pkg_dir.join(v)).unwrap();
    }
    dir
}

#[test]
fn add_records_dep_in_manifest_and_lockfile() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(run_init(tmp.path()).status.success());

    let out = run_add(tmp.path(), &["frozen_demo_pkg==0.1.0", "--offline"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let manifest = std::fs::read_to_string(tmp.path().join("mamba.toml")).unwrap();
    assert!(
        manifest.contains("\"frozen_demo_pkg==0.1.0\""),
        "manifest must record dep: {manifest}"
    );

    let lock = std::fs::read_to_string(tmp.path().join("mamba.lock")).unwrap();
    assert!(
        lock.contains("name = \"frozen_demo_pkg\""),
        "lock name: {lock}"
    );
    assert!(lock.contains("version = \"0.1.0\""), "lock version: {lock}");
    assert!(lock.contains("format_version = 1"), "lock fmt: {lock}");
    assert!(lock.contains("input_hash = "), "lock hash: {lock}");
}

#[test]
fn add_is_byte_identical_on_replay() {
    let tmp = tempfile::tempdir().unwrap();
    let project_dir = tmp.path().join("demo");
    std::fs::create_dir(&project_dir).unwrap();
    assert!(run_init(&project_dir).status.success());
    assert!(
        run_add(&project_dir, &["frozen_demo_pkg==0.1.0", "--offline"])
            .status
            .success()
    );
    let m_after_first = std::fs::read(project_dir.join("mamba.toml")).unwrap();
    let l_after_first = std::fs::read(project_dir.join("mamba.lock")).unwrap();

    // Replay: identical args, identical project state already on disk.
    assert!(
        run_add(&project_dir, &["frozen_demo_pkg==0.1.0", "--offline"])
            .status
            .success()
    );
    let m_after_second = std::fs::read(project_dir.join("mamba.toml")).unwrap();
    let l_after_second = std::fs::read(project_dir.join("mamba.lock")).unwrap();

    assert_eq!(
        m_after_first, m_after_second,
        "mamba.toml replay must be byte-identical"
    );
    assert_eq!(
        l_after_first, l_after_second,
        "mamba.lock replay must be byte-identical"
    );
}

#[test]
fn add_missing_package_against_frozen_index_fails_cleanly() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(run_init(tmp.path()).status.success());

    let index = frozen_index_with("frozen_demo_pkg", &["0.1.0"]);

    let manifest_before = std::fs::read(tmp.path().join("mamba.toml")).unwrap();
    let lock_path = tmp.path().join("mamba.lock");
    assert!(!lock_path.exists(), "no lockfile before");

    let out = run_add(
        tmp.path(),
        &[
            "package_that_does_not_exist",
            "--index",
            index.path().to_str().unwrap(),
        ],
    );

    assert!(!out.status.success(), "missing package must exit non-zero");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("not found"),
        "stderr must contain 'not found', got: {stderr:?}"
    );

    let manifest_after = std::fs::read(tmp.path().join("mamba.toml")).unwrap();
    assert_eq!(
        manifest_before, manifest_after,
        "manifest must not mutate on missing-package failure"
    );
    assert!(
        !lock_path.exists(),
        "lockfile must not be created on missing-package failure"
    );
}

#[test]
fn add_resolves_against_frozen_index_when_version_omitted() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(run_init(tmp.path()).status.success());

    let index = frozen_index_with("frozen_demo_pkg", &["0.1.0", "0.2.0"]);
    let out = run_add(
        tmp.path(),
        &["frozen_demo_pkg", "--index", index.path().to_str().unwrap()],
    );
    assert!(out.status.success());

    let manifest = std::fs::read_to_string(tmp.path().join("mamba.toml")).unwrap();
    assert!(
        manifest.contains("\"frozen_demo_pkg==0.2.0\""),
        "must pick highest version: {manifest}"
    );
}

#[test]
fn add_upserts_existing_dep_in_place() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(run_init(tmp.path()).status.success());

    assert!(run_add(tmp.path(), &["foo==1.0.0", "--offline"])
        .status
        .success());
    assert!(run_add(tmp.path(), &["foo==1.1.0", "--offline"])
        .status
        .success());

    let manifest = std::fs::read_to_string(tmp.path().join("mamba.toml")).unwrap();
    assert!(
        manifest.contains("\"foo==1.1.0\""),
        "must record upgraded version: {manifest}"
    );
    assert!(
        !manifest.contains("\"foo==1.0.0\""),
        "must not retain old version: {manifest}"
    );
}

#[test]
fn add_requires_initialized_project() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run_add(tmp.path(), &["foo==1.0.0", "--offline"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("mamba init"),
        "stderr should point to init: {stderr:?}"
    );
}

#[test]
fn add_offline_requires_explicit_version() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(run_init(tmp.path()).status.success());
    let out = run_add(tmp.path(), &["bare_name", "--offline"]);
    assert!(
        !out.status.success(),
        "offline + bare name must bail; stdout: {} stderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--offline") || stderr.contains("offline"),
        "stderr must point at --offline mode: {stderr:?}"
    );
}

#[test]
fn add_against_pypi_mock_records_real_sha256() {
    // wiremock stakes a fake PyPI JSON endpoint; mamba add must:
    //   1. Hit /pypi/<name>/json
    //   2. Pick the highest stable version
    //   3. Carry the wheel's sha256 into mamba.lock
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (server_url, expected_sha) = rt.block_on(async {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        let sha = "d".repeat(64);
        let body = serde_json::json!({
            "info": { "name": "mock_pkg", "version": "1.2.3" },
            "releases": {
                "1.2.3": [
                    {
                        "filename": "mock_pkg-1.2.3-py3-none-any.whl",
                        "url": "https://example.invalid/mock_pkg-1.2.3-py3-none-any.whl",
                        "digests": { "sha256": &sha },
                        "yanked": false
                    }
                ],
                "1.0.0": [
                    {
                        "filename": "mock_pkg-1.0.0-py3-none-any.whl",
                        "url": "https://example.invalid/mock_pkg-1.0.0-py3-none-any.whl",
                        "digests": { "sha256": "0".repeat(64) },
                        "yanked": false
                    }
                ]
            }
        });
        // PEP 503 normalize: `mock_pkg` -> `mock-pkg`, the index client
        // hits `/pypi/mock-pkg/json`.
        Mock::given(method("GET"))
            .and(path("/pypi/mock-pkg/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;
        // Leak server so it lives past the runtime drop.
        let url = server.uri();
        std::mem::forget(server);
        (url, sha)
    });

    let tmp = tempfile::tempdir().unwrap();
    let isolated_cache = tmp.path().join("cache");
    std::fs::create_dir_all(&isolated_cache).unwrap();
    assert!(run_init(tmp.path()).status.success());

    let out = Command::new(mamba_bin())
        .args(["add", "mock_pkg", "--index-url", &server_url])
        .env("MAMBA_CACHE_DIR", &isolated_cache)
        .current_dir(tmp.path())
        .output()
        .expect("spawn mamba");
    assert!(
        out.status.success(),
        "wiremock add must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let manifest = std::fs::read_to_string(tmp.path().join("mamba.toml")).unwrap();
    assert!(
        manifest.contains("\"mock_pkg==1.2.3\""),
        "must record highest version: {manifest}"
    );
    let lock = std::fs::read_to_string(tmp.path().join("mamba.lock")).unwrap();
    assert!(
        lock.contains(&format!("sha256 = \"{}\"", expected_sha)),
        "lock must carry wheel sha256: {lock}"
    );
}

#[test]
fn add_prefers_native_wheel_over_purepy_via_tags() {
    // Release ships both a native cp312 wheel for the running host AND a
    // generic `py3-none-any.whl`. PEP 425 selection (`pkgmgr::tags`) must
    // score the native wheel higher and that's the sha we should see in
    // the lockfile. Skipped on hosts we don't yet emit native tags for.
    let native_filename = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Some("tagged_pkg-1.0.0-cp312-cp312-macosx_11_0_arm64.whl")
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Some("tagged_pkg-1.0.0-cp312-cp312-macosx_10_9_x86_64.whl")
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Some("tagged_pkg-1.0.0-cp312-cp312-manylinux_2_17_x86_64.whl")
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        Some("tagged_pkg-1.0.0-cp312-cp312-manylinux_2_17_aarch64.whl")
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        Some("tagged_pkg-1.0.0-cp312-cp312-win_amd64.whl")
    } else {
        None
    };
    let Some(native_filename) = native_filename else {
        return;
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    let (server_url, native_sha) = rt.block_on(async {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        let native_sha = "a".repeat(64);
        let purepy_sha = "b".repeat(64);
        let body = serde_json::json!({
            "info": { "name": "tagged_pkg", "version": "1.0.0" },
            "releases": {
                "1.0.0": [
                    {
                        "filename": "tagged_pkg-1.0.0-py3-none-any.whl",
                        "url": "https://example.invalid/tagged_pkg-1.0.0-py3-none-any.whl",
                        "digests": { "sha256": &purepy_sha },
                        "yanked": false
                    },
                    {
                        "filename": native_filename,
                        "url": format!("https://example.invalid/{native_filename}"),
                        "digests": { "sha256": &native_sha },
                        "yanked": false
                    }
                ]
            }
        });
        Mock::given(method("GET"))
            .and(path("/pypi/tagged-pkg/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;
        let url = server.uri();
        std::mem::forget(server);
        (url, native_sha)
    });

    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().join("cache");
    std::fs::create_dir_all(&cache).unwrap();
    assert!(run_init(tmp.path()).status.success());
    let out = Command::new(mamba_bin())
        .args(["add", "tagged_pkg", "--index-url", &server_url])
        .env("MAMBA_CACHE_DIR", &cache)
        .current_dir(tmp.path())
        .output()
        .expect("spawn mamba");
    assert!(
        out.status.success(),
        "tagged add must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let lock = std::fs::read_to_string(tmp.path().join("mamba.lock")).unwrap();
    assert!(
        lock.contains(&format!("sha256 = \"{}\"", native_sha)),
        "lock must carry the NATIVE wheel sha (tag-aware selection), got: {lock}"
    );
}
