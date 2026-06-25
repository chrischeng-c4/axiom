//! CLI integration tests for `mamba pip`.

use std::path::{Path, PathBuf};
use std::process::Command;

use mamba::pkgmanage::pkgmgr::wheel_build::{
    CoreMetadata, WheelBuilder, WheelMetadata, compose_filename,
};
use sha2::{Digest, Sha256};

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

fn write_dist(site: &Path, dir_name: &str, metadata: &str) {
    let dist = site.join(dir_name);
    std::fs::create_dir_all(&dist).unwrap();
    std::fs::write(dist.join("METADATA"), metadata).unwrap();
}

fn build_wheel(root: &Path, name: &str, version: &str, requires: &[&str]) -> PathBuf {
    let filename = compose_filename(name, version, "py3", "none", "any");
    let mut wheel_meta = WheelMetadata::new("mamba-pkgmgr-test");
    wheel_meta.tags.push("py3-none-any".into());
    let mut core_meta = CoreMetadata::new(name, version);
    core_meta.requires_dist = requires.iter().map(|r| r.to_string()).collect();
    let mut builder = WheelBuilder::new(filename, wheel_meta, core_meta);
    let module = name.replace(['-', '.'], "_").to_ascii_lowercase();
    builder.add_file(
        format!("{module}/__init__.py"),
        format!("__version__ = {version:?}\n"),
    );
    builder.build_to_dir(root).unwrap()
}

fn build_wheel_index(wheels: &[PathBuf]) -> (tempfile::TempDir, std::process::Output) {
    let index = tempfile::tempdir().unwrap();
    let mut args = vec!["index", "build", "--out", index.path().to_str().unwrap()];
    for wheel in wheels {
        args.push(wheel.to_str().unwrap());
    }
    let tmp = tempfile::tempdir().unwrap();
    let out = run(tmp.path(), &args);
    (index, out)
}

fn sha256_file(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn fixture_site() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    write_dist(
        tmp.path(),
        "Requests-2.31.0.dist-info",
        "\
Name: Requests
Version: 2.31.0
Summary: Python HTTP for Humans.
Home-page: https://requests.readthedocs.io
Requires-Dist: urllib3>=2
",
    );
    write_dist(
        tmp.path(),
        "urllib3-2.1.0.dist-info",
        "\
Name: urllib3
Version: 2.1.0
",
    );
    tmp
}

#[test]
fn pip_list_reads_site_packages_inventory() {
    let site = fixture_site();
    let tmp = tempfile::tempdir().unwrap();
    let out = run(
        tmp.path(),
        &[
            "pip",
            "list",
            "--site-packages",
            site.path().to_str().unwrap(),
            "--no-header",
        ],
    );
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Requests"), "{stdout}");
    assert!(stdout.contains("urllib3"), "{stdout}");
}

#[test]
fn pip_freeze_emits_sorted_requirements_pins() {
    let site = fixture_site();
    let tmp = tempfile::tempdir().unwrap();
    let out = run(
        tmp.path(),
        &[
            "pip",
            "freeze",
            "--site-packages",
            site.path().to_str().unwrap(),
        ],
    );
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "Requests==2.31.0\nurllib3==2.1.0\n"
    );
}

#[test]
fn pip_show_renders_metadata_for_one_package() {
    let site = fixture_site();
    let tmp = tempfile::tempdir().unwrap();
    let out = run(
        tmp.path(),
        &[
            "pip",
            "show",
            "requests",
            "--site-packages",
            site.path().to_str().unwrap(),
        ],
    );
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Name: Requests"), "{stdout}");
    assert!(stdout.contains("Version: 2.31.0"), "{stdout}");
    assert!(stdout.contains("Requires: urllib3"), "{stdout}");
}

#[test]
fn pip_check_reports_success_for_consistent_inventory() {
    let site = fixture_site();
    let tmp = tempfile::tempdir().unwrap();
    let out = run(
        tmp.path(),
        &[
            "pip",
            "check",
            "--site-packages",
            site.path().to_str().unwrap(),
        ],
    );
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("No broken requirements found."));
}

#[test]
fn pip_compile_frozen_index_writes_pins_hashes_and_annotations() {
    let wheel_dir = tempfile::tempdir().unwrap();
    let app = build_wheel(
        wheel_dir.path(),
        "compile-app",
        "1.0.0",
        &["compile-dep==0.2.0"],
    );
    let dep = build_wheel(wheel_dir.path(), "compile-dep", "0.2.0", &[]);
    let (index, index_out) = build_wheel_index(&[app, dep]);
    assert!(
        index_out.status.success(),
        "index stderr: {}",
        String::from_utf8_lossy(&index_out.stderr)
    );

    let tmp = tempfile::tempdir().unwrap();
    let input = tmp.path().join("requirements.in");
    let output = tmp.path().join("compiled.txt");
    std::fs::write(&input, "compile-app>=1\n").unwrap();
    let compile = run(
        tmp.path(),
        &[
            "pip",
            "compile",
            input.to_str().unwrap(),
            "--index",
            index.path().to_str().unwrap(),
            "--output-file",
            output.to_str().unwrap(),
            "--generate-hashes",
            "--no-header",
        ],
    );
    assert!(
        compile.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let body = std::fs::read_to_string(output).unwrap();
    assert!(body.contains("compile-app==1.0.0"), "{body}");
    assert!(body.contains("compile-dep==0.2.0"), "{body}");
    assert!(body.contains("--hash=sha256:"), "{body}");
    assert!(body.contains("# via compile-app"), "{body}");
}

#[test]
fn pip_compile_no_deps_omits_transitive_requirements() {
    let wheel_dir = tempfile::tempdir().unwrap();
    let app = build_wheel(
        wheel_dir.path(),
        "nodeps-app",
        "1.0.0",
        &["nodeps-dep==0.2.0"],
    );
    let dep = build_wheel(wheel_dir.path(), "nodeps-dep", "0.2.0", &[]);
    let (index, index_out) = build_wheel_index(&[app, dep]);
    assert!(
        index_out.status.success(),
        "index stderr: {}",
        String::from_utf8_lossy(&index_out.stderr)
    );

    let tmp = tempfile::tempdir().unwrap();
    let input = tmp.path().join("requirements.in");
    std::fs::write(&input, "nodeps-app==1.0.0\n").unwrap();
    let compile = run(
        tmp.path(),
        &[
            "pip",
            "compile",
            input.to_str().unwrap(),
            "--index",
            index.path().to_str().unwrap(),
            "--no-header",
            "--no-deps",
        ],
    );
    assert!(
        compile.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let body = String::from_utf8_lossy(&compile.stdout);
    assert!(body.contains("nodeps-app==1.0.0"), "{body}");
    assert!(!body.contains("nodeps-dep"), "{body}");
}

#[test]
fn pip_compile_index_url_resolves_transitive_requirements() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let server_url = rt.block_on(async {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let root_meta = serde_json::json!({
            "info": { "name": "live_app", "version": "1.0.0" },
            "releases": {
                "1.0.0": [{
                    "filename": "live_app-1.0.0-py3-none-any.whl",
                    "url": "https://example.invalid/live_app-1.0.0-py3-none-any.whl",
                    "digests": { "sha256": &"a".repeat(64) },
                    "yanked": false
                }]
            }
        });
        Mock::given(method("GET"))
            .and(path("/pypi/live-app/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(root_meta))
            .mount(&server)
            .await;
        let root_version = serde_json::json!({
            "info": {
                "name": "live_app",
                "version": "1.0.0",
                "requires_dist": ["live_dep==0.2.0"]
            }
        });
        Mock::given(method("GET"))
            .and(path("/pypi/live-app/1.0.0/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(root_version))
            .mount(&server)
            .await;
        let dep_meta = serde_json::json!({
            "info": { "name": "live_dep", "version": "0.2.0" },
            "releases": {
                "0.2.0": [{
                    "filename": "live_dep-0.2.0-py3-none-any.whl",
                    "url": "https://example.invalid/live_dep-0.2.0-py3-none-any.whl",
                    "digests": { "sha256": &"b".repeat(64) },
                    "yanked": false
                }]
            }
        });
        Mock::given(method("GET"))
            .and(path("/pypi/live-dep/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(dep_meta))
            .mount(&server)
            .await;
        let dep_version = serde_json::json!({
            "info": {
                "name": "live_dep",
                "version": "0.2.0",
                "requires_dist": []
            }
        });
        Mock::given(method("GET"))
            .and(path("/pypi/live-dep/0.2.0/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(dep_version))
            .mount(&server)
            .await;
        let url = server.uri();
        std::mem::forget(server);
        url
    });

    let tmp = tempfile::tempdir().unwrap();
    let input = tmp.path().join("requirements.in");
    std::fs::write(&input, "live_app>=1\n").unwrap();
    let compile = run(
        tmp.path(),
        &[
            "pip",
            "compile",
            input.to_str().unwrap(),
            "--index-url",
            &server_url,
            "--generate-hashes",
            "--no-header",
        ],
    );
    assert!(
        compile.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let body = String::from_utf8_lossy(&compile.stdout);
    assert!(body.contains("live-app==1.0.0"), "{body}");
    assert!(body.contains("live-dep==0.2.0"), "{body}");
    assert!(body.contains("--hash=sha256:"), "{body}");
    assert!(body.contains("# via live-app"), "{body}");
}

#[test]
fn pip_install_direct_wheel_and_uninstall_use_record() {
    let wheel_dir = tempfile::tempdir().unwrap();
    let wheel = build_wheel(wheel_dir.path(), "demo-pkg", "1.0.0", &[]);
    let site = tempfile::tempdir().unwrap();
    let tmp = tempfile::tempdir().unwrap();

    let install = run(
        tmp.path(),
        &[
            "pip",
            "install",
            wheel.to_str().unwrap(),
            "--site-packages",
            site.path().to_str().unwrap(),
            "--python",
            "python3",
        ],
    );
    assert!(
        install.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&install.stderr)
    );
    assert!(
        site.path().join("demo_pkg").join("__init__.py").exists(),
        "wheel module should be installed"
    );

    let list = run(
        tmp.path(),
        &[
            "pip",
            "list",
            "--site-packages",
            site.path().to_str().unwrap(),
            "--no-header",
        ],
    );
    assert!(list.status.success());
    assert!(
        String::from_utf8_lossy(&list.stdout).contains("demo-pkg"),
        "{}",
        String::from_utf8_lossy(&list.stdout)
    );

    let uninstall = run(
        tmp.path(),
        &[
            "pip",
            "uninstall",
            "demo-pkg",
            "--site-packages",
            site.path().to_str().unwrap(),
        ],
    );
    assert!(
        uninstall.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&uninstall.stderr)
    );
    assert!(
        !site.path().join("demo_pkg").join("__init__.py").exists(),
        "RECORD uninstall should remove installed package files"
    );
}

#[test]
fn pip_install_from_frozen_index_installs_dependencies() {
    let wheel_dir = tempfile::tempdir().unwrap();
    let app = build_wheel(wheel_dir.path(), "demo-app", "1.0.0", &["demo-dep==0.2.0"]);
    let dep = build_wheel(wheel_dir.path(), "demo-dep", "0.2.0", &[]);
    let (index, index_out) = build_wheel_index(&[app, dep]);
    assert!(
        index_out.status.success(),
        "index stderr: {}",
        String::from_utf8_lossy(&index_out.stderr)
    );

    let site = tempfile::tempdir().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let install = run(
        tmp.path(),
        &[
            "pip",
            "install",
            "demo-app==1.0.0",
            "--index",
            index.path().to_str().unwrap(),
            "--site-packages",
            site.path().to_str().unwrap(),
            "--python",
            "python3",
        ],
    );
    assert!(
        install.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    let tree = run(
        tmp.path(),
        &[
            "pip",
            "tree",
            "--site-packages",
            site.path().to_str().unwrap(),
            "--package",
            "demo-app",
        ],
    );
    assert!(tree.status.success());
    let stdout = String::from_utf8_lossy(&tree.stdout);
    assert!(stdout.contains("demo-app v1.0.0"), "{stdout}");
    assert!(stdout.contains("demo-dep v0.2.0"), "{stdout}");
}

#[test]
fn pip_install_index_url_downloads_and_installs_dependencies() {
    let wheel_dir = tempfile::tempdir().unwrap();
    let app = build_wheel(
        wheel_dir.path(),
        "live-install-app",
        "1.0.0",
        &["live-install-dep==0.2.0"],
    );
    let dep = build_wheel(wheel_dir.path(), "live-install-dep", "0.2.0", &[]);
    let app_sha = sha256_file(&app);
    let dep_sha = sha256_file(&dep);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let server_url = rt.block_on(async {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let base = server.uri();
        let app_name = app.file_name().unwrap().to_str().unwrap().to_string();
        let dep_name = dep.file_name().unwrap().to_str().unwrap().to_string();
        let app_meta = serde_json::json!({
            "info": { "name": "live-install-app", "version": "1.0.0" },
            "releases": {
                "1.0.0": [{
                    "filename": &app_name,
                    "url": format!("{base}/files/{app_name}"),
                    "digests": { "sha256": &app_sha },
                    "yanked": false
                }]
            }
        });
        Mock::given(method("GET"))
            .and(path("/pypi/live-install-app/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(app_meta))
            .mount(&server)
            .await;
        let app_version = serde_json::json!({
            "info": {
                "name": "live-install-app",
                "version": "1.0.0",
                "requires_dist": ["live-install-dep==0.2.0"]
            }
        });
        Mock::given(method("GET"))
            .and(path("/pypi/live-install-app/1.0.0/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(app_version))
            .mount(&server)
            .await;
        let dep_meta = serde_json::json!({
            "info": { "name": "live-install-dep", "version": "0.2.0" },
            "releases": {
                "0.2.0": [{
                    "filename": &dep_name,
                    "url": format!("{base}/files/{dep_name}"),
                    "digests": { "sha256": &dep_sha },
                    "yanked": false
                }]
            }
        });
        Mock::given(method("GET"))
            .and(path("/pypi/live-install-dep/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(dep_meta))
            .mount(&server)
            .await;
        let dep_version = serde_json::json!({
            "info": {
                "name": "live-install-dep",
                "version": "0.2.0",
                "requires_dist": []
            }
        });
        Mock::given(method("GET"))
            .and(path("/pypi/live-install-dep/0.2.0/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(dep_version))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path(format!("/files/{app_name}")))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(std::fs::read(&app).unwrap()))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path(format!("/files/{dep_name}")))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(std::fs::read(&dep).unwrap()))
            .mount(&server)
            .await;
        let url = server.uri();
        std::mem::forget(server);
        url
    });

    let site = tempfile::tempdir().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().join("cache");
    std::fs::create_dir_all(&cache).unwrap();
    let install = Command::new(mamba_bin())
        .args([
            "pip",
            "install",
            "live-install-app==1.0.0",
            "--index-url",
            &server_url,
            "--site-packages",
            site.path().to_str().unwrap(),
            "--python",
            "python3",
        ])
        .env("MAMBA_CACHE_DIR", &cache)
        .current_dir(tmp.path())
        .output()
        .expect("spawn mamba");
    assert!(
        install.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    let tree = run(
        tmp.path(),
        &[
            "pip",
            "tree",
            "--site-packages",
            site.path().to_str().unwrap(),
            "--package",
            "live-install-app",
        ],
    );
    assert!(tree.status.success());
    let stdout = String::from_utf8_lossy(&tree.stdout);
    assert!(stdout.contains("live-install-app v1.0.0"), "{stdout}");
    assert!(stdout.contains("live-install-dep v0.2.0"), "{stdout}");
}

#[test]
fn pip_sync_installs_requirements_and_prunes_extras() {
    let wheel_dir = tempfile::tempdir().unwrap();
    let app = build_wheel(wheel_dir.path(), "sync-app", "1.0.0", &["sync-dep==0.2.0"]);
    let dep = build_wheel(wheel_dir.path(), "sync-dep", "0.2.0", &[]);
    let extra = build_wheel(wheel_dir.path(), "sync-extra", "9.9.9", &[]);
    let (index, index_out) = build_wheel_index(&[app, dep]);
    assert!(
        index_out.status.success(),
        "index stderr: {}",
        String::from_utf8_lossy(&index_out.stderr)
    );

    let site = tempfile::tempdir().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let preinstall = run(
        tmp.path(),
        &[
            "pip",
            "install",
            extra.to_str().unwrap(),
            "--site-packages",
            site.path().to_str().unwrap(),
            "--python",
            "python3",
        ],
    );
    assert!(preinstall.status.success());

    let requirements = tmp.path().join("requirements.txt");
    std::fs::write(&requirements, "sync-app==1.0.0\n").unwrap();
    let sync = run(
        tmp.path(),
        &[
            "pip",
            "sync",
            requirements.to_str().unwrap(),
            "--index",
            index.path().to_str().unwrap(),
            "--site-packages",
            site.path().to_str().unwrap(),
            "--python",
            "python3",
        ],
    );
    assert!(
        sync.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&sync.stderr)
    );

    let freeze = run(
        tmp.path(),
        &[
            "pip",
            "freeze",
            "--site-packages",
            site.path().to_str().unwrap(),
        ],
    );
    assert!(freeze.status.success());
    let stdout = String::from_utf8_lossy(&freeze.stdout);
    assert!(stdout.contains("sync-app==1.0.0"), "{stdout}");
    assert!(stdout.contains("sync-dep==0.2.0"), "{stdout}");
    assert!(!stdout.contains("sync-extra"), "{stdout}");
}

#[test]
fn pip_sync_index_url_downloads_and_prunes_extras() {
    let wheel_dir = tempfile::tempdir().unwrap();
    let app = build_wheel(
        wheel_dir.path(),
        "live-sync-app",
        "1.0.0",
        &["live-sync-dep==0.2.0"],
    );
    let dep = build_wheel(wheel_dir.path(), "live-sync-dep", "0.2.0", &[]);
    let extra = build_wheel(wheel_dir.path(), "live-sync-extra", "9.9.9", &[]);
    let app_sha = sha256_file(&app);
    let dep_sha = sha256_file(&dep);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let server_url = rt.block_on(async {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let base = server.uri();
        let app_name = app.file_name().unwrap().to_str().unwrap().to_string();
        let dep_name = dep.file_name().unwrap().to_str().unwrap().to_string();
        let app_meta = serde_json::json!({
            "info": { "name": "live-sync-app", "version": "1.0.0" },
            "releases": {
                "1.0.0": [{
                    "filename": &app_name,
                    "url": format!("{base}/files/{app_name}"),
                    "digests": { "sha256": &app_sha },
                    "yanked": false
                }]
            }
        });
        Mock::given(method("GET"))
            .and(path("/pypi/live-sync-app/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(app_meta))
            .mount(&server)
            .await;
        let app_version = serde_json::json!({
            "info": {
                "name": "live-sync-app",
                "version": "1.0.0",
                "requires_dist": ["live-sync-dep==0.2.0"]
            }
        });
        Mock::given(method("GET"))
            .and(path("/pypi/live-sync-app/1.0.0/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(app_version))
            .mount(&server)
            .await;
        let dep_meta = serde_json::json!({
            "info": { "name": "live-sync-dep", "version": "0.2.0" },
            "releases": {
                "0.2.0": [{
                    "filename": &dep_name,
                    "url": format!("{base}/files/{dep_name}"),
                    "digests": { "sha256": &dep_sha },
                    "yanked": false
                }]
            }
        });
        Mock::given(method("GET"))
            .and(path("/pypi/live-sync-dep/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(dep_meta))
            .mount(&server)
            .await;
        let dep_version = serde_json::json!({
            "info": {
                "name": "live-sync-dep",
                "version": "0.2.0",
                "requires_dist": []
            }
        });
        Mock::given(method("GET"))
            .and(path("/pypi/live-sync-dep/0.2.0/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(dep_version))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path(format!("/files/{app_name}")))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(std::fs::read(&app).unwrap()))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path(format!("/files/{dep_name}")))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(std::fs::read(&dep).unwrap()))
            .mount(&server)
            .await;
        let url = server.uri();
        std::mem::forget(server);
        url
    });

    let site = tempfile::tempdir().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let cache = tmp.path().join("cache");
    std::fs::create_dir_all(&cache).unwrap();
    let preinstall = run(
        tmp.path(),
        &[
            "pip",
            "install",
            extra.to_str().unwrap(),
            "--site-packages",
            site.path().to_str().unwrap(),
            "--python",
            "python3",
        ],
    );
    assert!(preinstall.status.success());

    let requirements = tmp.path().join("requirements.txt");
    std::fs::write(&requirements, "live-sync-app==1.0.0\n").unwrap();
    let sync = Command::new(mamba_bin())
        .args([
            "pip",
            "sync",
            requirements.to_str().unwrap(),
            "--index-url",
            &server_url,
            "--site-packages",
            site.path().to_str().unwrap(),
            "--python",
            "python3",
        ])
        .env("MAMBA_CACHE_DIR", &cache)
        .current_dir(tmp.path())
        .output()
        .expect("spawn mamba");
    assert!(
        sync.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&sync.stderr)
    );

    let freeze = run(
        tmp.path(),
        &[
            "pip",
            "freeze",
            "--site-packages",
            site.path().to_str().unwrap(),
        ],
    );
    assert!(freeze.status.success());
    let stdout = String::from_utf8_lossy(&freeze.stdout);
    assert!(stdout.contains("live-sync-app==1.0.0"), "{stdout}");
    assert!(stdout.contains("live-sync-dep==0.2.0"), "{stdout}");
    assert!(!stdout.contains("live-sync-extra"), "{stdout}");
}

#[test]
fn pip_sync_keeps_dependencies_when_root_is_already_installed() {
    let wheel_dir = tempfile::tempdir().unwrap();
    let app = build_wheel(
        wheel_dir.path(),
        "ready-app",
        "1.0.0",
        &["ready-dep==0.2.0"],
    );
    let dep = build_wheel(wheel_dir.path(), "ready-dep", "0.2.0", &[]);
    let extra = build_wheel(wheel_dir.path(), "ready-extra", "9.9.9", &[]);
    let (index, index_out) = build_wheel_index(&[app.clone(), dep]);
    assert!(
        index_out.status.success(),
        "index stderr: {}",
        String::from_utf8_lossy(&index_out.stderr)
    );

    let site = tempfile::tempdir().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let install_root = run(
        tmp.path(),
        &[
            "pip",
            "install",
            app.to_str().unwrap(),
            "--index",
            index.path().to_str().unwrap(),
            "--site-packages",
            site.path().to_str().unwrap(),
            "--python",
            "python3",
        ],
    );
    assert!(
        install_root.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&install_root.stderr)
    );
    let install_extra = run(
        tmp.path(),
        &[
            "pip",
            "install",
            extra.to_str().unwrap(),
            "--site-packages",
            site.path().to_str().unwrap(),
            "--python",
            "python3",
        ],
    );
    assert!(install_extra.status.success());

    let requirements = tmp.path().join("requirements.txt");
    std::fs::write(&requirements, "ready-app==1.0.0\n").unwrap();
    let sync = run(
        tmp.path(),
        &[
            "pip",
            "sync",
            requirements.to_str().unwrap(),
            "--index",
            index.path().to_str().unwrap(),
            "--site-packages",
            site.path().to_str().unwrap(),
            "--python",
            "python3",
        ],
    );
    assert!(
        sync.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&sync.stderr)
    );

    let freeze = run(
        tmp.path(),
        &[
            "pip",
            "freeze",
            "--site-packages",
            site.path().to_str().unwrap(),
        ],
    );
    assert!(freeze.status.success());
    let stdout = String::from_utf8_lossy(&freeze.stdout);
    assert!(stdout.contains("ready-app==1.0.0"), "{stdout}");
    assert!(stdout.contains("ready-dep==0.2.0"), "{stdout}");
    assert!(!stdout.contains("ready-extra"), "{stdout}");
}

#[test]
fn pip_tree_renders_installed_dependency_graph() {
    let site = fixture_site();
    let tmp = tempfile::tempdir().unwrap();
    let out = run(
        tmp.path(),
        &[
            "pip",
            "tree",
            "--site-packages",
            site.path().to_str().unwrap(),
        ],
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Requests v2.31.0"), "{stdout}");
    assert!(stdout.contains("└── urllib3 v2.1.0"), "{stdout}");
}

#[test]
fn pip_tree_supports_focus_invert_prune_and_depth() {
    let site = fixture_site();
    let tmp = tempfile::tempdir().unwrap();

    let depth = run(
        tmp.path(),
        &[
            "pip",
            "tree",
            "--site-packages",
            site.path().to_str().unwrap(),
            "--package",
            "Requests",
            "--depth",
            "0",
        ],
    );
    assert!(depth.status.success());
    let depth_stdout = String::from_utf8_lossy(&depth.stdout);
    assert!(depth_stdout.contains("Requests v2.31.0"), "{depth_stdout}");
    assert!(!depth_stdout.contains("urllib3"), "{depth_stdout}");

    let inverted = run(
        tmp.path(),
        &[
            "pip",
            "tree",
            "--site-packages",
            site.path().to_str().unwrap(),
            "--package",
            "urllib3",
            "--invert",
        ],
    );
    assert!(inverted.status.success());
    let inverted_stdout = String::from_utf8_lossy(&inverted.stdout);
    assert!(
        inverted_stdout.contains("urllib3 v2.1.0"),
        "{inverted_stdout}"
    );
    assert!(
        inverted_stdout.contains("Requests v2.31.0"),
        "{inverted_stdout}"
    );

    let pruned = run(
        tmp.path(),
        &[
            "pip",
            "tree",
            "--site-packages",
            site.path().to_str().unwrap(),
            "--prune",
            "urllib3",
        ],
    );
    assert!(pruned.status.success());
    let pruned_stdout = String::from_utf8_lossy(&pruned.stdout);
    assert!(!pruned_stdout.contains("urllib3"), "{pruned_stdout}");
}

#[test]
fn pip_check_fails_when_required_dist_is_missing() {
    let site = tempfile::tempdir().unwrap();
    write_dist(
        site.path(),
        "Requests-2.31.0.dist-info",
        "\
Name: Requests
Version: 2.31.0
Requires-Dist: urllib3>=2
",
    );
    let tmp = tempfile::tempdir().unwrap();
    let out = run(
        tmp.path(),
        &[
            "pip",
            "check",
            "--site-packages",
            site.path().to_str().unwrap(),
        ],
    );
    assert!(!out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Requests requires urllib3"), "{stdout}");
}
