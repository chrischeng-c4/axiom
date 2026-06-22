//! CLI integration tests for `mamba lock` — closes the runtime side of
//! tests/governance/gates/pkgmgr/lock/manifest.toml (#2682).
//!
//! Pinned acceptance:
//!
//!   1. Lockfile contains direct AND transitive deps.
//!   2. Lockfile distinguishes direct vs transitive (per-package `direct`).
//!   3. Lock-only run does not create .venv / site-packages.
//!   4. Unresolvable dep => exit 1, stderr contains "no candidate" +
//!      failing dep name, NO partial lockfile written.
//!   5. Byte-identical on replay.

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
    let pkg_dir = index.join(normalize_pep503(name));
    let ver_dir = pkg_dir.join(version);
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
    // frozen_demo_pkg 0.1.0 -> frozen_demo_transitive 0.2.0
    stake_pkg(
        dir.path(),
        "frozen_demo_pkg",
        "0.1.0",
        &["frozen_demo_transitive==0.2.0"],
    );
    stake_pkg(dir.path(), "frozen_demo_transitive", "0.2.0", &[]);
    dir
}

#[test]
fn lock_records_direct_and_transitive_deps() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    assert!(run(&proj, &["init"]).status.success());
    assert!(run(
        &proj,
        &[
            "add",
            "frozen_demo_pkg==0.1.0",
            "--index",
            index.path().to_str().unwrap()
        ]
    )
    .status
    .success());

    let out = run(&proj, &["lock", "--index", index.path().to_str().unwrap()]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let lock = std::fs::read_to_string(proj.join("mamba.lock")).unwrap();
    assert!(
        lock.contains("name = \"frozen_demo_pkg\""),
        "direct: {lock}"
    );
    assert!(
        lock.contains("name = \"frozen_demo_transitive\""),
        "transitive: {lock}"
    );
    assert!(
        lock.contains("direct = true"),
        "direct flag present: {lock}"
    );
    assert!(
        lock.contains("direct = false"),
        "transitive flag present: {lock}"
    );
}

#[test]
fn lock_does_not_create_venv_or_site_packages() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    assert!(run(&proj, &["init"]).status.success());
    assert!(run(
        &proj,
        &[
            "add",
            "frozen_demo_pkg==0.1.0",
            "--index",
            index.path().to_str().unwrap()
        ]
    )
    .status
    .success());
    assert!(
        run(&proj, &["lock", "--index", index.path().to_str().unwrap()])
            .status
            .success()
    );

    assert!(!proj.join(".venv").exists(), "lock must not create .venv");
    assert!(
        !proj.join("site-packages").exists(),
        "lock must not create site-packages"
    );
}

#[test]
fn lock_unresolvable_dep_fails_cleanly() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    assert!(run(&proj, &["init"]).status.success());
    // Use --index that has frozen_demo_pkg but NOT package_that_does_not_exist.
    // We bypass the `add --index` validation by hand-editing manifest.
    let manifest = std::fs::read_to_string(proj.join("mamba.toml")).unwrap();
    let edited = manifest.replace(
        "dependencies = []",
        "dependencies = [\n    \"package_that_does_not_exist==0.0.1\",\n]",
    );
    std::fs::write(proj.join("mamba.toml"), edited).unwrap();

    let out = run(&proj, &["lock", "--index", index.path().to_str().unwrap()]);
    assert!(!out.status.success(), "must exit non-zero");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("no candidate"),
        "stderr must say 'no candidate': {stderr:?}"
    );
    assert!(
        stderr.contains("package_that_does_not_exist"),
        "stderr must name failing dep: {stderr:?}"
    );
    assert!(
        !proj.join("mamba.lock").exists(),
        "no partial lockfile on failure"
    );
}

#[test]
fn lock_is_byte_identical_on_replay() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    assert!(run(&proj, &["init"]).status.success());
    assert!(run(
        &proj,
        &[
            "add",
            "frozen_demo_pkg==0.1.0",
            "--index",
            index.path().to_str().unwrap()
        ]
    )
    .status
    .success());
    assert!(
        run(&proj, &["lock", "--index", index.path().to_str().unwrap()])
            .status
            .success()
    );
    let a = std::fs::read(proj.join("mamba.lock")).unwrap();
    assert!(
        run(&proj, &["lock", "--index", index.path().to_str().unwrap()])
            .status
            .success()
    );
    let b = std::fs::read(proj.join("mamba.lock")).unwrap();
    assert_eq!(a, b, "lockfile must be byte-identical on replay");
}

#[test]
fn lock_against_pypi_mock_records_sha256() {
    // wiremock stakes a fake PyPI JSON endpoint; `mamba lock` must:
    //   1. Hit /pypi/<pep503-name>/json for each dep in mamba.toml.
    //   2. Resolve through the real PubGrub-backed Resolver.
    //   3. Carry the wheel's sha256 into mamba.lock (no more empty placeholder).
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (server_url, expected_sha) = rt.block_on(async {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        let sha = "e".repeat(64);
        let body = serde_json::json!({
            "info": { "name": "lock_mock_pkg", "version": "2.0.0" },
            "releases": {
                "2.0.0": [
                    {
                        "filename": "lock_mock_pkg-2.0.0-py3-none-any.whl",
                        "url": "https://example.invalid/lock_mock_pkg-2.0.0-py3-none-any.whl",
                        "digests": { "sha256": &sha },
                        "yanked": false
                    }
                ]
            }
        });
        // PEP 503 normalize: `lock_mock_pkg` -> `lock-mock-pkg`.
        Mock::given(method("GET"))
            .and(path("/pypi/lock-mock-pkg/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;
        let url = server.uri();
        std::mem::forget(server);
        (url, sha)
    });

    let tmp = tempfile::tempdir().unwrap();
    let isolated_cache = tmp.path().join("cache");
    std::fs::create_dir_all(&isolated_cache).unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    assert!(run(&proj, &["init"]).status.success());

    // Edit manifest to declare the dep without running `add` (which would
    // also touch the index). This keeps the test scoped to `mamba lock`.
    let manifest = std::fs::read_to_string(proj.join("mamba.toml")).unwrap();
    let edited = manifest.replace(
        "dependencies = []",
        "dependencies = [\n    \"lock_mock_pkg==2.0.0\",\n]",
    );
    std::fs::write(proj.join("mamba.toml"), edited).unwrap();

    let out = Command::new(mamba_bin())
        .args(["lock", "--index-url", &server_url])
        .env("MAMBA_CACHE_DIR", &isolated_cache)
        .current_dir(&proj)
        .output()
        .expect("spawn mamba");
    assert!(
        out.status.success(),
        "wiremock lock must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let lock = std::fs::read_to_string(proj.join("mamba.lock")).unwrap();
    // Resolver canonicalizes names to PEP 503 form on the way out — this
    // matches uv. We just check the canonical form here.
    assert!(
        lock.contains("name = \"lock-mock-pkg\""),
        "lock names the dep (canonical): {lock}"
    );
    assert!(
        lock.contains(&format!("sha256 = \"{}\"", expected_sha)),
        "lock must carry wheel sha256: {lock}"
    );
}

#[test]
fn lock_resolves_transitive_requires_dist_against_pypi_mock() {
    // wiremock stakes a 2-level dependency chain:
    //   trans_root 1.0.0 requires_dist = ["trans_leaf>=2.0"]
    //   trans_leaf 2.0.0 requires_dist = []
    // `mamba lock` must walk the chain (Tick 13.5) so the lockfile
    // contains BOTH packages, distinguishes direct vs transitive, and
    // emits `dependencies = ["trans-leaf==2.0.0"]` on the root node.
    let rt = tokio::runtime::Runtime::new().unwrap();
    let server_url = rt.block_on(async {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;

        // Aggregate metadata for the root.
        let root_meta = serde_json::json!({
            "info": { "name": "trans_root", "version": "1.0.0" },
            "releases": {
                "1.0.0": [{
                    "filename": "trans_root-1.0.0-py3-none-any.whl",
                    "url": "https://example.invalid/trans_root-1.0.0-py3-none-any.whl",
                    "digests": { "sha256": &"1".repeat(64) },
                    "yanked": false
                }]
            }
        });
        Mock::given(method("GET"))
            .and(path("/pypi/trans-root/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(root_meta))
            .mount(&server)
            .await;
        // Per-version metadata for the root carries requires_dist.
        let root_version_meta = serde_json::json!({
            "info": {
                "name": "trans_root",
                "version": "1.0.0",
                "requires_dist": ["trans_leaf>=2.0"]
            }
        });
        Mock::given(method("GET"))
            .and(path("/pypi/trans-root/1.0.0/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(root_version_meta))
            .mount(&server)
            .await;

        // Aggregate metadata for the leaf.
        let leaf_meta = serde_json::json!({
            "info": { "name": "trans_leaf", "version": "2.0.0" },
            "releases": {
                "2.0.0": [{
                    "filename": "trans_leaf-2.0.0-py3-none-any.whl",
                    "url": "https://example.invalid/trans_leaf-2.0.0-py3-none-any.whl",
                    "digests": { "sha256": &"2".repeat(64) },
                    "yanked": false
                }]
            }
        });
        Mock::given(method("GET"))
            .and(path("/pypi/trans-leaf/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(leaf_meta))
            .mount(&server)
            .await;
        let leaf_version_meta = serde_json::json!({
            "info": {
                "name": "trans_leaf",
                "version": "2.0.0",
                "requires_dist": []
            }
        });
        Mock::given(method("GET"))
            .and(path("/pypi/trans-leaf/2.0.0/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(leaf_version_meta))
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
    let manifest = std::fs::read_to_string(proj.join("mamba.toml")).unwrap();
    let edited = manifest.replace(
        "dependencies = []",
        "dependencies = [\n    \"trans_root==1.0.0\",\n]",
    );
    std::fs::write(proj.join("mamba.toml"), edited).unwrap();

    let out = Command::new(mamba_bin())
        .args(["lock", "--index-url", &server_url])
        .env("MAMBA_CACHE_DIR", &cache)
        .current_dir(&proj)
        .output()
        .expect("spawn mamba");
    assert!(
        out.status.success(),
        "transitive lock must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let lock = std::fs::read_to_string(proj.join("mamba.lock")).unwrap();
    assert!(
        lock.contains("name = \"trans-root\""),
        "lock contains root: {lock}"
    );
    assert!(
        lock.contains("name = \"trans-leaf\""),
        "lock contains transitive leaf: {lock}"
    );
    // The root must distinguish itself as direct AND name its dep.
    assert!(lock.contains("direct = true"), "root marked direct: {lock}");
    assert!(
        lock.contains("direct = false"),
        "leaf marked transitive: {lock}"
    );
    assert!(
        lock.contains("dependencies = [\"trans-leaf==2.0.0\"]"),
        "root's dependencies pin the leaf: {lock}"
    );
}
