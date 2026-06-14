use std::process::Command;

/// Apply the local agent shell defaults needed for deterministic Rust gates.
pub(crate) fn apply_default_shell_env(command: &mut Command) {
    command.env("CC", "/usr/bin/cc");
    command.env("CXX", "/usr/bin/c++");
    command.env("PATH", default_path());
}

fn default_path() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let mut parts = Vec::new();
    if !home.is_empty() {
        parts.push(format!(
            "{home}/.rustup/toolchains/stable-aarch64-apple-darwin/bin"
        ));
    }
    parts.extend(
        ["/usr/bin", "/bin", "/usr/sbin", "/sbin"]
            .iter()
            .map(|path| path.to_string()),
    );
    if !home.is_empty() {
        parts.push(format!("{home}/.cargo/bin"));
    }
    if let Ok(existing) = std::env::var("PATH") {
        parts.push(existing);
    }
    parts.join(":")
}
