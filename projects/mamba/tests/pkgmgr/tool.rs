//! CLI integration tests for `mamba tool`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

fn run(tools_dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(mamba_bin())
        .args(args)
        .env("MAMBA_TOOLS_DIR", tools_dir)
        .output()
        .expect("spawn mamba")
}

fn normalize_pep503(name: &str) -> String {
    name.chars()
        .map(|c| if matches!(c, '_' | '.') { '-' } else { c })
        .collect::<String>()
        .to_ascii_lowercase()
}

fn stake_pkg(index: &Path, name: &str, version: &str) {
    let ver_dir = index.join(normalize_pep503(name)).join(version);
    std::fs::create_dir_all(&ver_dir).unwrap();
    std::fs::write(ver_dir.join("metadata.toml"), "requires = []\n").unwrap();
}

fn build_index() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    stake_pkg(dir.path(), "frozen_demo_pkg", "0.1.0");
    stake_pkg(dir.path(), "frozen_demo_pkg", "0.2.0");
    dir
}

#[test]
fn tool_install_list_and_uninstall_wrap_existing_tool_install() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let tools = tmp.path().join("mamba-tools");

    let install = run(
        &tools,
        &[
            "tool",
            "install",
            "frozen_demo_pkg",
            "--version",
            "0.1.0",
            "--index",
            index.path().to_str().unwrap(),
        ],
    );
    assert!(
        install.status.success(),
        "tool install must succeed; stderr: {}",
        String::from_utf8_lossy(&install.stderr)
    );
    assert!(tools.join("frozen-demo-pkg/manifest.toml").exists());

    let list = run(&tools, &["tool", "list"]);
    assert!(
        list.status.success(),
        "tool list must succeed; stderr: {}",
        String::from_utf8_lossy(&list.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&list.stdout),
        "frozen_demo_pkg==0.1.0\n"
    );

    let uninstall = run(&tools, &["tool", "uninstall", "frozen_demo_pkg"]);
    assert!(
        uninstall.status.success(),
        "tool uninstall must succeed; stderr: {}",
        String::from_utf8_lossy(&uninstall.stderr)
    );
    assert!(!tools.join("frozen-demo-pkg").exists());
}

#[test]
fn tool_upgrade_reinstalls_latest_available_version() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let tools = tmp.path().join("mamba-tools");

    assert!(
        run(
            &tools,
            &[
                "tool",
                "install",
                "frozen_demo_pkg",
                "--version",
                "0.1.0",
                "--index",
                index.path().to_str().unwrap(),
            ],
        )
        .status
        .success()
    );

    let upgrade = run(
        &tools,
        &[
            "tool",
            "upgrade",
            "frozen_demo_pkg",
            "--index",
            index.path().to_str().unwrap(),
        ],
    );
    assert!(
        upgrade.status.success(),
        "tool upgrade must succeed; stderr: {}",
        String::from_utf8_lossy(&upgrade.stderr)
    );
    let manifest = std::fs::read_to_string(tools.join("frozen-demo-pkg/manifest.toml")).unwrap();
    assert!(manifest.contains("version = \"0.2.0\""), "{manifest}");
}

#[test]
fn tool_run_auto_installs_from_index_and_executes_stub() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let tools = tmp.path().join("mamba-tools");

    let out = run(
        &tools,
        &[
            "tool",
            "run",
            "--version",
            "0.1.0",
            "--index",
            index.path().to_str().unwrap(),
            "frozen_demo_pkg",
        ],
    );
    assert!(
        out.status.success(),
        "tool run must install and execute; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let manifest = std::fs::read_to_string(tools.join("frozen-demo-pkg/manifest.toml")).unwrap();
    assert!(manifest.contains("version = \"0.1.0\""), "{manifest}");
}

#[test]
fn tool_dir_and_update_shell_are_scriptable() {
    let tmp = tempfile::tempdir().unwrap();
    let tools = tmp.path().join("mamba-tools");

    let dir = run(&tools, &["tool", "dir"]);
    assert!(
        dir.status.success(),
        "tool dir must succeed; stderr: {}",
        String::from_utf8_lossy(&dir.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&dir.stdout).trim_end(),
        tools.display().to_string()
    );

    let shell = run(
        &tools,
        &[
            "tool",
            "update-shell",
            "--shell",
            "bash",
            "--bin-dir",
            "/opt/mamba/bin",
        ],
    );
    assert!(
        shell.status.success(),
        "tool update-shell must succeed; stderr: {}",
        String::from_utf8_lossy(&shell.stderr)
    );
    let stdout = String::from_utf8_lossy(&shell.stdout);
    assert!(
        stdout.contains("# >>> mamba initialize >>>"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("export PATH=\"/opt/mamba/bin:$PATH\""),
        "stdout: {stdout}"
    );
}
