use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

/// Apply the local agent shell defaults needed for deterministic Rust gates.
pub(crate) fn apply_default_shell_env(command: &mut Command) {
    if let Some(home) = default_home() {
        command.env("HOME", home);
    }
    command.env("CC", "/usr/bin/cc");
    command.env("CXX", "/usr/bin/c++");
    command.env("PATH", default_path());
}

/// Build a shell command, wrapped by cap when the repository has a local cap
/// binary available. Cap is intentionally optional: command identity stays the
/// original shell string, and missing cap keeps the old `sh -c` behavior.
pub(crate) fn protected_shell_command(project_root: &Path, shell_command: &str) -> Command {
    if let Some(cap_bin) = resolve_cap_binary(project_root, shell_command) {
        let mut command = Command::new(cap_bin);
        apply_default_shell_env(&mut command);
        command
            .arg("run")
            .arg("--")
            .arg("sh")
            .arg("-c")
            .arg(shell_command);
        command
    } else {
        let mut command = Command::new("sh");
        apply_default_shell_env(&mut command);
        command.arg("-c").arg(shell_command);
        command
    }
}

fn resolve_cap_binary(project_root: &Path, shell_command: &str) -> Option<PathBuf> {
    resolve_cap_binary_from(
        project_root,
        shell_command,
        std::env::var_os("AW_CAP_BIN"),
        std::env::var_os("AW_DISABLE_CAP").is_some(),
    )
}

fn resolve_cap_binary_from(
    project_root: &Path,
    shell_command: &str,
    env_bin: Option<OsString>,
    disabled: bool,
) -> Option<PathBuf> {
    if disabled || command_is_cap_wrapped(shell_command) {
        return None;
    }
    if let Some(path) = env_bin.map(PathBuf::from).filter(|path| path.is_file()) {
        return Some(path);
    }
    let local = project_root.join("target/debug/cap");
    if local.is_file() {
        return Some(local);
    }
    #[cfg(not(test))]
    {
        return find_executable_in_path("cap");
    }
    #[cfg(test)]
    None
}

#[cfg(not(test))]
fn find_executable_in_path(program: &str) -> Option<PathBuf> {
    std::env::split_paths(&default_path()).find_map(|dir| {
        let candidate = dir.join(program);
        candidate.is_file().then_some(candidate)
    })
}

fn command_is_cap_wrapped(shell_command: &str) -> bool {
    let trimmed = shell_command.trim_start();
    trimmed.starts_with("cap ")
        || trimmed.starts_with("./target/debug/cap ")
        || trimmed.contains("/cap run ")
}

fn default_path() -> String {
    let home = default_home();
    let existing = std::env::var("PATH").ok();
    default_path_for(home.as_deref(), existing.as_deref())
}

fn default_home() -> Option<String> {
    default_home_from(std::env::var("HOME").ok(), dirs::home_dir())
}

fn default_home_from(home_env: Option<String>, detected_home: Option<PathBuf>) -> Option<String> {
    home_env
        .filter(|home| !home.is_empty())
        .or_else(|| detected_home.map(|path| path.display().to_string()))
        .filter(|home| !home.is_empty())
}

fn default_path_for(home: Option<&str>, existing_path: Option<&str>) -> String {
    let mut parts = Vec::new();
    if let Some(home) = home {
        parts.push(format!(
            "{home}/.rustup/toolchains/stable-aarch64-apple-darwin/bin"
        ));
    }
    parts.extend(
        [
            "/opt/homebrew/bin",
            "/usr/local/bin",
            "/usr/bin",
            "/bin",
            "/usr/sbin",
            "/sbin",
        ]
        .iter()
        .map(|path| path.to_string()),
    );
    if let Some(home) = home {
        parts.push(format!("{home}/.cargo/bin"));
    }
    if let Some(existing) = existing_path {
        parts.push(existing.to_string());
    }
    parts.join(":")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_home_falls_back_to_detected_home_when_env_is_empty() {
        assert_eq!(
            default_home_from(Some(String::new()), Some(PathBuf::from("/Users/example"))),
            Some("/Users/example".to_string())
        );
    }

    #[test]
    fn default_path_uses_detected_home_for_rustup_and_cargo_bins() {
        let path = default_path_for(Some("/Users/example"), Some("/opt/tool/bin"));

        assert!(
            path.starts_with("/Users/example/.rustup/toolchains/stable-aarch64-apple-darwin/bin:")
        );
        assert!(path.contains(":/opt/homebrew/bin:/usr/local/bin:"));
        assert!(path.contains(":/Users/example/.cargo/bin:"));
        assert!(path.ends_with(":/opt/tool/bin"));
    }

    #[test]
    fn cap_protection_uses_local_debug_cap_when_present() {
        let root = tempfile::tempdir().unwrap();
        let cap = root.path().join("target/debug/cap");
        std::fs::create_dir_all(cap.parent().unwrap()).unwrap();
        std::fs::write(&cap, "").unwrap();

        assert_eq!(
            resolve_cap_binary_from(root.path(), "cargo test", None, false),
            Some(cap)
        );
    }

    #[test]
    fn protected_shell_command_wraps_shell_syntax_as_sh_c_under_cap() {
        let root = tempfile::tempdir().unwrap();
        let cap = root.path().join("target/debug/cap");
        std::fs::create_dir_all(cap.parent().unwrap()).unwrap();
        std::fs::write(&cap, "").unwrap();

        let command = protected_shell_command(root.path(), "cd app && cargo test");
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert_eq!(command.get_program(), cap.as_os_str());
        assert_eq!(args, vec!["run", "--", "sh", "-c", "cd app && cargo test"]);
    }

    #[test]
    fn cap_protection_skips_already_wrapped_commands() {
        let root = tempfile::tempdir().unwrap();
        let cap = root.path().join("target/debug/cap");
        std::fs::create_dir_all(cap.parent().unwrap()).unwrap();
        std::fs::write(&cap, "").unwrap();

        assert_eq!(
            resolve_cap_binary_from(root.path(), "cap run 'cargo test'", None, false),
            None
        );
    }
}
