//! CLI integration tests for shell integration and completion output.

use std::path::PathBuf;
use std::process::Command;

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

#[test]
fn shell_path_prints_bash_path_prepend_snippet() {
    let out = Command::new(mamba_bin())
        .args([
            "shell",
            "path",
            "--shell",
            "bash",
            "--bin-dir",
            "/opt/mamba/bin",
        ])
        .output()
        .expect("spawn mamba");
    assert!(
        out.status.success(),
        "shell path must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "export PATH=\"/opt/mamba/bin:$PATH\"\n"
    );
}

#[test]
fn shell_init_wraps_nushell_snippet_in_managed_block() {
    let out = Command::new(mamba_bin())
        .args([
            "shell",
            "init",
            "--shell",
            "nushell",
            "--bin-dir",
            "/opt/mamba/bin",
        ])
        .output()
        .expect("spawn mamba");
    assert!(
        out.status.success(),
        "shell init must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("# >>> mamba initialize >>>"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("$env.PATH = ($env.PATH | prepend \"/opt/mamba/bin\")"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("# <<< mamba initialize <<<"),
        "stdout: {stdout}"
    );
}

#[test]
fn generate_shell_completion_bash_includes_current_command_tree() {
    let out = Command::new(mamba_bin())
        .args(["generate-shell-completion", "bash"])
        .output()
        .expect("spawn mamba");
    assert!(
        out.status.success(),
        "completion must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("workspace"), "stdout missing workspace");
    assert!(
        stdout.contains("pkgmgr-validate"),
        "stdout missing pkgmgr-validate"
    );
    assert!(
        stdout.contains("generate-shell-completion"),
        "stdout missing generate-shell-completion"
    );
}
