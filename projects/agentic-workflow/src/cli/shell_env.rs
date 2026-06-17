use std::{path::PathBuf, process::Command};

/// Apply the local agent shell defaults needed for deterministic Rust gates.
pub(crate) fn apply_default_shell_env(command: &mut Command) {
    if let Some(home) = default_home() {
        command.env("HOME", home);
    }
    command.env("CC", "/usr/bin/cc");
    command.env("CXX", "/usr/bin/c++");
    command.env("PATH", default_path());
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
        ["/usr/bin", "/bin", "/usr/sbin", "/sbin"]
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
        assert!(path.contains(":/Users/example/.cargo/bin:"));
        assert!(path.ends_with(":/opt/tool/bin"));
    }
}
