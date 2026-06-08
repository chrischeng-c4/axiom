// Shell integration primitives (Tick 126).
//
// `mamba python install` and `mamba tool install` drop binaries into
// directories that need to be on `$PATH`. uv emits a shell-specific
// snippet for the user to paste into their rc file. This module is the
// shell-kind classifier + snippet emitter.
//
// Each shell variant ships:
//   * the canonical lowercase identifier used in `uv generate-shell-...`
//   * the default rc filename (relative to `$HOME`)
//   * the PATH-prepend syntax (so the shim takes priority over system
//     Python)
//   * a stable marker comment used when an installer wants to insert
//     and later remove a self-managed block idempotently
//
// Detection: pip / uv read `$SHELL` (POSIX) or `$PSModulePath`
// (PowerShell heuristic). On Windows, `$ComSpec` ending in cmd.exe
// implies cmd; the rest are detected from `$SHELL` basename.
//
// Pure string library — no I/O. Filesystem rc-file mutation is left
// to the caller, who already has venv/install-aware logic for picking
// the right rc-file path.

const MARKER_BEGIN: &str = "# >>> mamba initialize >>>";
const MARKER_END: &str = "# <<< mamba initialize <<<";

/// Shell variants mamba (and uv) ship snippets for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
    Nushell,
    Elvish,
}

impl Shell {
    /// Canonical lower-case identifier — matches `--shell` flag values
    /// used by uv and pip-completion.
    pub fn as_str(self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::PowerShell => "powershell",
            Shell::Cmd => "cmd",
            Shell::Nushell => "nushell",
            Shell::Elvish => "elvish",
        }
    }

    /// Parse `--shell` flag value into a Shell variant.
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "bash" => Some(Shell::Bash),
            "zsh" => Some(Shell::Zsh),
            "fish" => Some(Shell::Fish),
            "powershell" | "pwsh" => Some(Shell::PowerShell),
            "cmd" | "cmd.exe" => Some(Shell::Cmd),
            "nu" | "nushell" => Some(Shell::Nushell),
            "elvish" => Some(Shell::Elvish),
            _ => None,
        }
    }

    /// Default rc-file path *relative to $HOME* (or $USERPROFILE on
    /// Windows). PowerShell uses `$PROFILE`; we return the relative
    /// path under `Documents\PowerShell\` per the PS7 convention.
    pub fn default_rc_path(self) -> &'static str {
        match self {
            Shell::Bash => ".bashrc",
            Shell::Zsh => ".zshrc",
            Shell::Fish => ".config/fish/config.fish",
            Shell::PowerShell => "Documents/PowerShell/Microsoft.PowerShell_profile.ps1",
            // cmd.exe has no rc file; AutoRun via registry is the only
            // option. Return an empty string so callers can branch on it.
            Shell::Cmd => "",
            Shell::Nushell => ".config/nushell/env.nu",
            Shell::Elvish => ".config/elvish/rc.elv",
        }
    }

    /// Emit a snippet that prepends `dir` to the shell's PATH.
    /// `dir` should already be quoted / escaped if it contains spaces;
    /// this function does NOT add additional escaping.
    pub fn prepend_path_snippet(self, dir: &str) -> String {
        match self {
            Shell::Bash | Shell::Zsh => format!("export PATH=\"{dir}:$PATH\""),
            Shell::Fish => format!("set -gx PATH \"{dir}\" $PATH"),
            Shell::PowerShell => format!("$env:Path = \"{dir};\" + $env:Path"),
            Shell::Cmd => format!("set PATH={dir};%PATH%"),
            Shell::Nushell => format!("$env.PATH = ($env.PATH | prepend \"{dir}\")"),
            Shell::Elvish => format!("set paths = [\"{dir}\" $@paths]"),
        }
    }

    /// Stable comment string a tool can use to mark the start of a
    /// self-managed block in the user's rc file. Paired with
    /// [`Self::marker_end_for`] for idempotent insert/remove.
    pub fn marker_begin_for(self) -> &'static str {
        match self {
            Shell::PowerShell | Shell::Cmd => "# >>> mamba initialize >>>",
            // Fallback: every other shell uses `#` for comments.
            _ => MARKER_BEGIN,
        }
    }

    /// Stable comment string for the end of a self-managed block.
    pub fn marker_end_for(self) -> &'static str {
        match self {
            Shell::PowerShell | Shell::Cmd => "# <<< mamba initialize <<<",
            _ => MARKER_END,
        }
    }

    /// Wrap `body` in begin/end markers for idempotent rc-file edits.
    /// The wrapped block always ends with a trailing newline so it
    /// reads cleanly when concatenated with the rest of the rc file.
    pub fn wrap_managed_block(self, body: &str) -> String {
        format!(
            "{begin}\n{body}\n{end}\n",
            begin = self.marker_begin_for(),
            body = body.trim_end_matches('\n'),
            end = self.marker_end_for(),
        )
    }
}

/// Detect the user's shell from the `$SHELL` env-var value (POSIX) or
/// a sentinel string (Windows). Caller is responsible for reading the
/// env var; we keep this pure.
///
/// On POSIX, `$SHELL` is the absolute path to the shell binary
/// (`/bin/bash`, `/usr/local/bin/zsh`, etc.). We classify by the
/// trailing basename only.
pub fn detect_from_shell_env(shell_env: &str) -> Option<Shell> {
    if shell_env.is_empty() {
        return None;
    }
    // Accept both `/` (POSIX) and `\` (Windows) separators. Take the
    // tail after the last separator on either system.
    let basename = match shell_env.rfind(['/', '\\']) {
        Some(idx) => &shell_env[idx + 1..],
        None => shell_env,
    };
    // Trim trailing `.exe` for Windows-style values where it survives.
    let basename = basename.strip_suffix(".exe").unwrap_or(basename);
    Shell::parse(basename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_known_shells() {
        for s in [
            Shell::Bash,
            Shell::Zsh,
            Shell::Fish,
            Shell::PowerShell,
            Shell::Cmd,
            Shell::Nushell,
            Shell::Elvish,
        ] {
            assert_eq!(Shell::parse(s.as_str()), Some(s));
        }
    }

    #[test]
    fn parse_is_case_insensitive() {
        assert_eq!(Shell::parse("BASH"), Some(Shell::Bash));
        assert_eq!(Shell::parse("Zsh"), Some(Shell::Zsh));
        assert_eq!(Shell::parse("PowerShell"), Some(Shell::PowerShell));
    }

    #[test]
    fn parse_recognizes_aliases() {
        assert_eq!(Shell::parse("pwsh"), Some(Shell::PowerShell));
        assert_eq!(Shell::parse("nu"), Some(Shell::Nushell));
        assert_eq!(Shell::parse("cmd.exe"), Some(Shell::Cmd));
    }

    #[test]
    fn parse_returns_none_for_unknown_shell() {
        assert!(Shell::parse("tcsh").is_none());
        assert!(Shell::parse("xonsh").is_none());
        assert!(Shell::parse("").is_none());
    }

    #[test]
    fn rc_paths_are_stable() {
        assert_eq!(Shell::Bash.default_rc_path(), ".bashrc");
        assert_eq!(Shell::Zsh.default_rc_path(), ".zshrc");
        assert_eq!(Shell::Fish.default_rc_path(), ".config/fish/config.fish");
        assert_eq!(
            Shell::PowerShell.default_rc_path(),
            "Documents/PowerShell/Microsoft.PowerShell_profile.ps1"
        );
        assert_eq!(Shell::Cmd.default_rc_path(), "");
        assert_eq!(Shell::Nushell.default_rc_path(), ".config/nushell/env.nu");
        assert_eq!(Shell::Elvish.default_rc_path(), ".config/elvish/rc.elv");
    }

    #[test]
    fn bash_zsh_path_snippet_uses_export_form() {
        let snippet = Shell::Bash.prepend_path_snippet("/opt/mamba/bin");
        assert_eq!(snippet, r#"export PATH="/opt/mamba/bin:$PATH""#);
        let zsh = Shell::Zsh.prepend_path_snippet("/opt/mamba/bin");
        assert_eq!(zsh, snippet);
    }

    #[test]
    fn fish_path_snippet_uses_set_gx() {
        assert_eq!(
            Shell::Fish.prepend_path_snippet("/opt/mamba/bin"),
            r#"set -gx PATH "/opt/mamba/bin" $PATH"#
        );
    }

    #[test]
    fn powershell_path_snippet_uses_env_path_concat() {
        assert_eq!(
            Shell::PowerShell.prepend_path_snippet("C:\\mamba\\bin"),
            r#"$env:Path = "C:\mamba\bin;" + $env:Path"#
        );
    }

    #[test]
    fn cmd_path_snippet_uses_set_with_percent_path() {
        assert_eq!(
            Shell::Cmd.prepend_path_snippet("C:\\mamba\\bin"),
            r#"set PATH=C:\mamba\bin;%PATH%"#
        );
    }

    #[test]
    fn nushell_path_snippet_uses_prepend_form() {
        assert_eq!(
            Shell::Nushell.prepend_path_snippet("/opt/mamba/bin"),
            r#"$env.PATH = ($env.PATH | prepend "/opt/mamba/bin")"#
        );
    }

    #[test]
    fn elvish_path_snippet_uses_set_paths_form() {
        assert_eq!(
            Shell::Elvish.prepend_path_snippet("/opt/mamba/bin"),
            r#"set paths = ["/opt/mamba/bin" $@paths]"#
        );
    }

    #[test]
    fn detect_from_shell_env_parses_basename() {
        assert_eq!(detect_from_shell_env("/bin/bash"), Some(Shell::Bash));
        assert_eq!(
            detect_from_shell_env("/usr/local/bin/zsh"),
            Some(Shell::Zsh)
        );
        assert_eq!(
            detect_from_shell_env("/opt/homebrew/bin/fish"),
            Some(Shell::Fish)
        );
    }

    #[test]
    fn detect_strips_exe_suffix_on_windows() {
        // Windows backslash paths are recognized; .exe is stripped.
        assert_eq!(
            detect_from_shell_env("C:\\Windows\\System32\\cmd.exe"),
            Some(Shell::Cmd),
        );
        assert_eq!(
            detect_from_shell_env("C:\\Program Files\\PowerShell\\7\\pwsh.exe"),
            Some(Shell::PowerShell),
        );
        // Bare basename (no separator) also accepted.
        assert_eq!(detect_from_shell_env("bash"), Some(Shell::Bash));
    }

    #[test]
    fn detect_returns_none_for_unknown_shell() {
        assert!(detect_from_shell_env("/bin/tcsh").is_none());
        assert!(detect_from_shell_env("").is_none());
    }

    #[test]
    fn wrap_managed_block_is_idempotent_friendly() {
        let body = "export PATH=\"/opt/mamba/bin:$PATH\"";
        let wrapped = Shell::Bash.wrap_managed_block(body);
        assert!(wrapped.starts_with(MARKER_BEGIN));
        assert!(wrapped.ends_with(&format!("{MARKER_END}\n")));
        // Body in the middle survives unchanged.
        assert!(wrapped.contains(body));
    }

    #[test]
    fn wrap_strips_trailing_newlines_from_body() {
        let body = "snippet\n\n\n";
        let wrapped = Shell::Bash.wrap_managed_block(body);
        // No double-newline before the closing marker.
        assert!(!wrapped.contains("snippet\n\n"));
        assert!(wrapped.contains("snippet\n"));
    }

    #[test]
    fn markers_are_stable_across_shells() {
        // POSIX shells share the same markers; PS / cmd use their
        // visually-equivalent comment form.
        assert_eq!(
            Shell::Bash.marker_begin_for(),
            Shell::Zsh.marker_begin_for()
        );
        assert_eq!(
            Shell::Bash.marker_begin_for(),
            Shell::Fish.marker_begin_for()
        );
        assert!(Shell::Cmd.marker_begin_for().starts_with('#'));
        assert!(Shell::PowerShell.marker_begin_for().starts_with('#'));
    }

    #[test]
    fn round_trip_as_str_then_parse() {
        for s in [
            Shell::Bash,
            Shell::Zsh,
            Shell::Fish,
            Shell::PowerShell,
            Shell::Cmd,
            Shell::Nushell,
            Shell::Elvish,
        ] {
            assert_eq!(Shell::parse(s.as_str()), Some(s));
        }
    }
}
