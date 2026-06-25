//! CLI integration tests for `mamba sync` — closes the runtime side of
//! tests/governance/gates/pkgmgr/sync/manifest.toml (#2683).
//!
//! Pinned acceptance:
//!
//!   1. First run creates `.venv` + materializes every locked package.
//!   2. Second run is a clean no-op (no env mutation, stderr signals
//!      `no_op`).
//!   3. mamba.lock is byte-identical across both runs.
//!   4. Import probe: `<pkg>/__init__.py` exists after both runs.
//!   5. Sync without a lockfile fails cleanly.

use std::path::{Path, PathBuf};
use std::process::Command;

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

fn run(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(mamba_bin())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("spawn mamba")
}

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

fn stake_pkg(index: &Path, name: &str, version: &str, requires: &[&str]) {
    let ver_dir = index.join(normalize_pep503(name)).join(version);
    std::fs::create_dir_all(&ver_dir).unwrap();
    let meta = if requires.is_empty() {
        "requires = []\n".to_string()
    } else {
        let arr = requires
            .iter()
            .map(|r| format!("\"{r}\""))
            .collect::<Vec<_>>()
            .join(", ");
        format!("requires = [{arr}]\n")
    };
    std::fs::write(ver_dir.join("metadata.toml"), meta).unwrap();
}

fn build_index() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    stake_pkg(
        dir.path(),
        "frozen_demo_pkg",
        "0.1.0",
        &["frozen_demo_transitive==0.2.0"],
    );
    stake_pkg(dir.path(), "frozen_demo_transitive", "0.2.0", &[]);
    dir
}

fn setup_locked_project(proj: &Path, index: &Path) {
    assert!(run(proj, &["init"]).status.success());
    assert!(
        run(
            proj,
            &[
                "add",
                "frozen_demo_pkg==0.1.0",
                "--index",
                index.to_str().unwrap()
            ]
        )
        .status
        .success()
    );
    assert!(
        run(proj, &["lock", "--index", index.to_str().unwrap()])
            .status
            .success()
    );
}

#[test]
fn sync_first_run_creates_env_and_installs_locked_deps() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    setup_locked_project(&proj, index.path());

    let out = run(&proj, &["sync"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let venv = proj.join(".venv");
    assert!(venv.exists(), ".venv must be created");
    assert!(
        venv.join("pyvenv.cfg").exists(),
        "pyvenv.cfg must be written"
    );
    let site = venv.join("site-packages");
    for pkg in ["frozen_demo_pkg", "frozen_demo_transitive"] {
        let dir = site.join(pkg);
        assert!(
            dir.join("__init__.py").exists(),
            "{pkg} __init__.py missing"
        );
        assert!(dir.join("INSTALLER").exists(), "{pkg} INSTALLER missing");
        assert!(dir.join("VERSION").exists(), "{pkg} VERSION missing");
    }
}

#[test]
fn sync_second_run_is_a_clean_noop() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    setup_locked_project(&proj, index.path());

    assert!(run(&proj, &["sync"]).status.success());
    let lock_a = std::fs::read(proj.join("mamba.lock")).unwrap();
    let init_a =
        std::fs::read(proj.join(".venv/site-packages/frozen_demo_pkg/__init__.py")).unwrap();

    let out = run(&proj, &["sync"]);
    assert!(out.status.success(), "second sync must succeed");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("no_op")
            || stderr.contains("unchanged")
            || stderr.contains("already_synced")
            || stderr.contains("up_to_date"),
        "second sync must signal no-op, got: {stderr:?}"
    );

    let lock_b = std::fs::read(proj.join("mamba.lock")).unwrap();
    let init_b =
        std::fs::read(proj.join(".venv/site-packages/frozen_demo_pkg/__init__.py")).unwrap();
    assert_eq!(lock_a, lock_b, "lockfile byte-identical across syncs");
    assert_eq!(init_a, init_b, "package init.py untouched on no-op");
}

#[test]
fn sync_check_reports_environment_drift_without_mutation() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    setup_locked_project(&proj, index.path());

    let before = std::fs::read(proj.join("mamba.lock")).unwrap();
    let out = run(&proj, &["sync", "--check"]);
    assert!(!out.status.success(), "sync --check must fail before sync");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("not synchronized"),
        "stderr names drift: {stderr:?}"
    );
    assert!(
        !proj.join(".venv").exists(),
        "failed sync --check must not create .venv"
    );
    assert_eq!(
        before,
        std::fs::read(proj.join("mamba.lock")).unwrap(),
        "sync --check must not rewrite the lockfile"
    );

    assert!(run(&proj, &["sync"]).status.success());
    let synced = run(&proj, &["sync", "--check"]);
    assert!(
        synced.status.success(),
        "sync --check must pass after sync; stderr: {}",
        String::from_utf8_lossy(&synced.stderr)
    );
    assert!(
        String::from_utf8_lossy(&synced.stdout).contains("synchronized"),
        "stdout names synchronized env: {}",
        String::from_utf8_lossy(&synced.stdout)
    );
}

#[test]
fn sync_import_probe_holds_after_both_runs() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    setup_locked_project(&proj, index.path());

    assert!(run(&proj, &["sync"]).status.success());
    let probe = proj.join(".venv/site-packages/frozen_demo_pkg/__init__.py");
    assert!(probe.exists(), "import probe present after first sync");
    let body = std::fs::read_to_string(&probe).unwrap();
    assert!(body.contains("__mamba_pkg__"), "stub marks itself: {body}");

    assert!(run(&proj, &["sync"]).status.success());
    assert!(probe.exists(), "import probe present after second sync");
}

#[test]
fn sync_requires_lockfile() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(run(tmp.path(), &["init"]).status.success());

    let out = run(tmp.path(), &["sync"]);
    assert!(!out.status.success(), "sync without lockfile must fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("mamba.lock"),
        "stderr names the missing file: {stderr:?}"
    );
}

#[test]
fn sync_requires_initialized_project() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run(tmp.path(), &["sync"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("mamba init"),
        "stderr hints at init: {stderr:?}"
    );
}

#[test]
fn sync_downloads_and_verifies_wheel_when_url_and_sha_present() {
    // Tick 15: when the lockfile carries `url` + `sha256` for a package,
    // `mamba sync` must stream the artifact through the sha-verifying
    // download path (uv-style). The wheel lands in the cache directory
    // alongside its .sha256 sidecar; sha256 mismatch would abort.
    use sha2::{Digest, Sha256};
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (server_url, body_sha) = rt.block_on(async {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        let body = b"tick15-fake-wheel-bytes-for-sync-sha256-test";
        let mut hasher = Sha256::new();
        hasher.update(body);
        let digest = format!("{:x}", hasher.finalize());
        Mock::given(method("GET"))
            .and(path("/files/tick15_sync_pkg-1.0.0-py3-none-any.whl"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(body.to_vec()))
            .mount(&server)
            .await;
        let url = server.uri();
        std::mem::forget(server);
        (url, digest)
    });

    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().join("cache");
    std::fs::create_dir_all(&cache).unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    assert!(run(&proj, &["init"]).status.success());

    // Stake a lockfile by hand so the test is scoped to `mamba sync`.
    let wheel_url = format!("{server_url}/files/tick15_sync_pkg-1.0.0-py3-none-any.whl");
    let lock = format!(
        "format_version = 1\ninput_hash = \"x\"\n\n[[package]]\nname = \"tick15_sync_pkg\"\nversion = \"1.0.0\"\nsha256 = \"{body_sha}\"\nurl = \"{wheel_url}\"\nsource = \"pypi://tick15_sync_pkg/1.0.0\"\ndependencies = []\n"
    );
    std::fs::write(proj.join("mamba.lock"), lock).unwrap();

    let out = Command::new(mamba_bin())
        .args(["sync"])
        .env("MAMBA_CACHE_DIR", &cache)
        .current_dir(&proj)
        .output()
        .expect("spawn mamba");
    assert!(
        out.status.success(),
        "sync must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Cache must contain the downloaded wheel + sha sidecar.
    // IndexClient normalizes name to PEP 503 (`_` -> `-`) when laying out the
    // artifacts directory, matching uv and the on-disk cache layout.
    let artifacts = cache.join("artifacts/tick15-sync-pkg");
    let wheel = artifacts.join("tick15_sync_pkg-1.0.0-py3-none-any.whl");
    let sidecar = artifacts.join("tick15_sync_pkg-1.0.0-py3-none-any.whl.sha256");
    assert!(
        wheel.exists(),
        "downloaded wheel must land in cache at {}",
        wheel.display()
    );
    assert!(
        sidecar.exists(),
        "sha sidecar must be written next to the wheel"
    );
    let sidecar_body = std::fs::read_to_string(&sidecar).unwrap();
    assert_eq!(
        sidecar_body.trim(),
        body_sha,
        "sidecar must record the verified sha"
    );
}

#[test]
fn sync_downloads_many_packages_in_parallel() {
    // Tick 16: stage N wheels on wiremock and verify `mamba sync` fetches
    // every one of them when the lockfile names them all. This validates the
    // semaphore + JoinSet fan-out path — the test does not assert wall-time
    // ratios (flaky on CI) but does assert every artifact lands on disk.
    use sha2::{Digest, Sha256};
    let rt = tokio::runtime::Runtime::new().unwrap();
    const N: usize = 6;
    let (server_url, pkgs) = rt.block_on(async {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        let mut pkgs: Vec<(String, String, String)> = Vec::with_capacity(N);
        for i in 0..N {
            let name = format!("tick16_par_{i}");
            let version = "1.0.0".to_string();
            let filename = format!("{name}-{version}-py3-none-any.whl");
            let body = format!("tick16-parallel-body-{i}").into_bytes();
            let mut h = Sha256::new();
            h.update(&body);
            let digest = format!("{:x}", h.finalize());
            Mock::given(method("GET"))
                .and(path(format!("/files/{filename}")))
                .respond_with(ResponseTemplate::new(200).set_body_bytes(body))
                .mount(&server)
                .await;
            pkgs.push((name, version, digest));
        }
        let url = server.uri();
        std::mem::forget(server);
        (url, pkgs)
    });

    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().join("cache");
    std::fs::create_dir_all(&cache).unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    assert!(run(&proj, &["init"]).status.success());

    let mut lock = String::from("format_version = 1\ninput_hash = \"x\"\n");
    for (name, version, digest) in &pkgs {
        let filename = format!("{name}-{version}-py3-none-any.whl");
        let url = format!("{server_url}/files/{filename}");
        lock.push_str(&format!(
            "\n[[package]]\nname = \"{name}\"\nversion = \"{version}\"\nsha256 = \"{digest}\"\nurl = \"{url}\"\nsource = \"pypi://{name}/{version}\"\ndependencies = []\n"
        ));
    }
    std::fs::write(proj.join("mamba.lock"), lock).unwrap();

    let out = Command::new(mamba_bin())
        .args(["sync", "--jobs", "4"])
        .env("MAMBA_CACHE_DIR", &cache)
        .current_dir(&proj)
        .output()
        .expect("spawn mamba");
    assert!(
        out.status.success(),
        "parallel sync must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Each wheel must be on disk in the per-package cache.
    for (name, version, _) in &pkgs {
        // PEP 503 normalize underscores to dashes for the cache key.
        let normalized = name.replace('_', "-");
        let filename = format!("{name}-{version}-py3-none-any.whl");
        let wheel = cache.join("artifacts").join(&normalized).join(&filename);
        assert!(
            wheel.exists(),
            "{name} wheel missing at {} after parallel sync",
            wheel.display()
        );
    }
}

#[test]
fn sync_jobs_one_runs_sequentially_but_still_completes() {
    // Tick 16: `--jobs 1` collapses to a serial download pass. The
    // semaphore caps at 1 permit, but the same JoinSet machinery applies.
    use sha2::{Digest, Sha256};
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (server_url, name, version, digest) = rt.block_on(async {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        let body = b"tick16-serial-body";
        let mut h = Sha256::new();
        h.update(body);
        let digest = format!("{:x}", h.finalize());
        Mock::given(method("GET"))
            .and(path("/files/tick16_serial-1.0.0-py3-none-any.whl"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(body.to_vec()))
            .mount(&server)
            .await;
        let url = server.uri();
        std::mem::forget(server);
        (
            url,
            "tick16_serial".to_string(),
            "1.0.0".to_string(),
            digest,
        )
    });

    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().join("cache");
    std::fs::create_dir_all(&cache).unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    assert!(run(&proj, &["init"]).status.success());

    let wheel_url = format!("{server_url}/files/tick16_serial-1.0.0-py3-none-any.whl");
    let lock = format!(
        "format_version = 1\ninput_hash = \"x\"\n\n[[package]]\nname = \"{name}\"\nversion = \"{version}\"\nsha256 = \"{digest}\"\nurl = \"{wheel_url}\"\nsource = \"pypi://{name}/{version}\"\ndependencies = []\n"
    );
    std::fs::write(proj.join("mamba.lock"), lock).unwrap();

    let out = Command::new(mamba_bin())
        .args(["sync", "-j", "1"])
        .env("MAMBA_CACHE_DIR", &cache)
        .current_dir(&proj)
        .output()
        .expect("spawn mamba");
    assert!(
        out.status.success(),
        "--jobs 1 must still succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let normalized = name.replace('_', "-");
    let wheel = cache
        .join("artifacts")
        .join(&normalized)
        .join("tick16_serial-1.0.0-py3-none-any.whl");
    assert!(wheel.exists(), "wheel missing at {}", wheel.display());
}

#[test]
fn sync_fails_on_sha256_mismatch() {
    // Tick 15: tamper with the lockfile sha so the streaming verify aborts.
    // The package must NOT be marked installed (no stub created), and stderr
    // must surface the mismatch — uv-style safety contract.
    let rt = tokio::runtime::Runtime::new().unwrap();
    let server_url = rt.block_on(async {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/files/tick15_sync_bad-1.0.0-py3-none-any.whl"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"different-bytes".to_vec()))
            .mount(&server)
            .await;
        let url = server.uri();
        std::mem::forget(server);
        url
    });

    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().join("cache");
    std::fs::create_dir_all(&cache).unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    assert!(run(&proj, &["init"]).status.success());

    let wrong_sha = "0".repeat(64);
    let wheel_url = format!("{server_url}/files/tick15_sync_bad-1.0.0-py3-none-any.whl");
    let lock = format!(
        "format_version = 1\ninput_hash = \"x\"\n\n[[package]]\nname = \"tick15_sync_bad\"\nversion = \"1.0.0\"\nsha256 = \"{wrong_sha}\"\nurl = \"{wheel_url}\"\nsource = \"pypi://tick15_sync_bad/1.0.0\"\ndependencies = []\n"
    );
    std::fs::write(proj.join("mamba.lock"), lock).unwrap();

    let out = Command::new(mamba_bin())
        .args(["sync"])
        .env("MAMBA_CACHE_DIR", &cache)
        .current_dir(&proj)
        .output()
        .expect("spawn mamba");
    assert!(
        !out.status.success(),
        "sync MUST fail when sha256 mismatches"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("hash mismatch") || stderr.contains("HashMismatch"),
        "stderr must say hash mismatch: {stderr:?}"
    );
    assert!(
        !proj
            .join(".venv/site-packages/tick15_sync_bad/__init__.py")
            .exists(),
        "stub must NOT be created when verification fails"
    );
}

#[test]
fn sync_reuses_content_addressed_cache_across_package_names() {
    // Tick 17: when the same wheel body (same sha256) is referenced from two
    // distinct package names in two distinct lockfiles, the second sync must
    // hit the content-addressed store populated by the first sync — the
    // wiremock GET endpoint for the second package URL receives zero hits.
    //
    // This is the production cache property that lets uv-style installs share
    // bytes across `extras`, sibling releases, and identical re-uploads.
    use sha2::{Digest, Sha256};
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (server_url, body_sha) = rt.block_on(async {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        let body = b"tick17-cas-shared-wheel-bytes-identical-across-pkgs";
        let mut hasher = Sha256::new();
        hasher.update(body);
        let digest = format!("{:x}", hasher.finalize());

        // Route A — first sync downloads from here.
        Mock::given(method("GET"))
            .and(path("/files/tick17_pkga-1.0.0-py3-none-any.whl"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(body.to_vec()))
            .expect(1)
            .mount(&server)
            .await;

        // Route B — the second sync's lockfile points here, but CAS must
        // short-circuit the request. `expect(0)` makes the test fail if the
        // CAS bypass regresses.
        Mock::given(method("GET"))
            .and(path("/files/tick17_pkgb-1.0.0-py3-none-any.whl"))
            .respond_with(
                ResponseTemplate::new(500).set_body_bytes(b"must-not-be-fetched".to_vec()),
            )
            .expect(0)
            .mount(&server)
            .await;

        let url = server.uri();
        std::mem::forget(server);
        (url, digest)
    });

    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().join("cache");
    std::fs::create_dir_all(&cache).unwrap();

    // First project — populates CAS by downloading pkga.
    let proj_a = tmp.path().join("demo_a");
    std::fs::create_dir(&proj_a).unwrap();
    assert!(run(&proj_a, &["init"]).status.success());
    let url_a = format!("{server_url}/files/tick17_pkga-1.0.0-py3-none-any.whl");
    let lock_a = format!(
        "format_version = 1\ninput_hash = \"x\"\n\n[[package]]\nname = \"tick17_pkga\"\nversion = \"1.0.0\"\nsha256 = \"{body_sha}\"\nurl = \"{url_a}\"\nsource = \"pypi://tick17_pkga/1.0.0\"\ndependencies = []\n"
    );
    std::fs::write(proj_a.join("mamba.lock"), lock_a).unwrap();
    let out_a = Command::new(mamba_bin())
        .args(["sync"])
        .env("MAMBA_CACHE_DIR", &cache)
        .current_dir(&proj_a)
        .output()
        .expect("spawn mamba");
    assert!(
        out_a.status.success(),
        "first sync must succeed; stderr: {}",
        String::from_utf8_lossy(&out_a.stderr)
    );

    // CAS entry must exist after the first sync.
    let prefix = &body_sha[..2];
    let cas_path = cache.join("content").join(prefix).join(&body_sha);
    assert!(
        cas_path.exists(),
        "CAS entry missing at {} after first sync — promote step regressed",
        cas_path.display()
    );

    // Second project — same wheel body, different package name + URL.
    // The URL response is a 500: if the cache short-circuits, we never hit it.
    let proj_b = tmp.path().join("demo_b");
    std::fs::create_dir(&proj_b).unwrap();
    assert!(run(&proj_b, &["init"]).status.success());
    let url_b = format!("{server_url}/files/tick17_pkgb-1.0.0-py3-none-any.whl");
    let lock_b = format!(
        "format_version = 1\ninput_hash = \"x\"\n\n[[package]]\nname = \"tick17_pkgb\"\nversion = \"1.0.0\"\nsha256 = \"{body_sha}\"\nurl = \"{url_b}\"\nsource = \"pypi://tick17_pkgb/1.0.0\"\ndependencies = []\n"
    );
    std::fs::write(proj_b.join("mamba.lock"), lock_b).unwrap();
    let out_b = Command::new(mamba_bin())
        .args(["sync"])
        .env("MAMBA_CACHE_DIR", &cache)
        .current_dir(&proj_b)
        .output()
        .expect("spawn mamba");
    assert!(
        out_b.status.success(),
        "second sync must succeed via CAS reuse; stderr: {}",
        String::from_utf8_lossy(&out_b.stderr)
    );

    // pkgb wheel must materialize in its name-addressed slot — proves the
    // CAS-hit path also wrote the per-package artifact + sidecar.
    let pkgb_artifacts = cache.join("artifacts/tick17-pkgb");
    let pkgb_wheel = pkgb_artifacts.join("tick17_pkgb-1.0.0-py3-none-any.whl");
    let pkgb_sidecar = pkgb_artifacts.join("tick17_pkgb-1.0.0-py3-none-any.whl.sha256");
    assert!(
        pkgb_wheel.exists(),
        "pkgb wheel must be materialized from CAS at {}",
        pkgb_wheel.display()
    );
    assert!(pkgb_sidecar.exists(), "pkgb sha sidecar must be written");
    let pkgb_sidecar_body = std::fs::read_to_string(&pkgb_sidecar).unwrap();
    assert_eq!(
        pkgb_sidecar_body.trim(),
        body_sha,
        "pkgb sidecar must record the shared sha"
    );
}
