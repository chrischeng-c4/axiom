//! CLI integration tests for `mamba pip`.

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

fn write_dist(site: &Path, dir_name: &str, metadata: &str) {
    let dist = site.join(dir_name);
    std::fs::create_dir_all(&dist).unwrap();
    std::fs::write(dist.join("METADATA"), metadata).unwrap();
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
