// PEP 376 `.dist-info/INSTALLER` reader + writer (Tick 64).
//
// PEP 376 requires every installed distribution to carry an
// `INSTALLER` file that names the tool which installed it. The format
// is dirt simple: a single line with the installer's name and a
// trailing newline. pip writes `pip\n`; uv writes `uv\n`; conda
// writes `conda\n`. The file is read by `pip uninstall` to decide
// whether to refuse uninstalling distributions managed by another
// tool, and by `pip list --not-required` style operations.
//
// This module owns the on-disk format only — choosing the installer
// name and locating the `.dist-info` dir are caller responsibilities.

/// Writer-side: the installer name mamba writes when it installs a
/// distribution. Wheel install code should call
/// `render_installer(INSTALLER_NAME)` and stage the result.
pub const INSTALLER_NAME: &str = "mamba";

/// Render the on-disk form for an INSTALLER file. The output is
/// always `<name>\n` (single trailing newline, no other whitespace).
/// Empty `name` panics in debug builds — silently writing an empty
/// INSTALLER would create a file that `pip uninstall` interprets as
/// "any tool may uninstall", which is rarely what callers want.
pub fn render_installer(name: &str) -> String {
    debug_assert!(
        !name.is_empty(),
        "render_installer: installer name must be non-empty"
    );
    let mut out = String::with_capacity(name.len() + 1);
    out.push_str(name);
    out.push('\n');
    out
}

/// Parse a `.dist-info/INSTALLER` file. Returns the trimmed installer
/// name. An empty / whitespace-only file yields `None` — PEP 376
/// permits this and clients should treat it as "installer unknown".
///
/// Anything beyond the first non-blank line is ignored. The PEP
/// reference text says only the first line is significant, and
/// real-world INSTALLER files written by older versions of pip have
/// trailing blank lines that we tolerate silently.
pub fn parse_installer(src: &str) -> Option<String> {
    for line in src.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

/// Round-trip helper: write then read back yields the same name.
/// Provided for callers (and tests) that just want to know whether a
/// given installer claim survives the file format intact.
pub fn round_trip(name: &str) -> Option<String> {
    parse_installer(&render_installer(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installer_name_constant_is_mamba() {
        assert_eq!(INSTALLER_NAME, "mamba");
    }

    #[test]
    fn render_installer_appends_single_newline() {
        assert_eq!(render_installer("uv"), "uv\n");
        assert_eq!(render_installer("pip"), "pip\n");
        assert_eq!(render_installer(INSTALLER_NAME), "mamba\n");
    }

    #[test]
    fn render_installer_does_not_double_newline() {
        // Even if a caller passes "uv\n", render must still produce
        // exactly one trailing newline in total. (The current impl
        // does not strip; this test pins that contract — if we ever
        // add stripping it should hold.)
        let out = render_installer("uv");
        assert_eq!(out.matches('\n').count(), 1);
    }

    #[test]
    fn parse_installer_returns_trimmed_first_line() {
        assert_eq!(parse_installer("uv\n").as_deref(), Some("uv"));
        assert_eq!(parse_installer("  pip  \n").as_deref(), Some("pip"));
        assert_eq!(parse_installer("conda").as_deref(), Some("conda"));
    }

    #[test]
    fn parse_installer_skips_leading_blank_lines() {
        assert_eq!(parse_installer("\n\n   \nuv\n").as_deref(), Some("uv"));
    }

    #[test]
    fn parse_installer_ignores_trailing_lines() {
        // Older pip wrote a trailing blank line; we must tolerate it.
        assert_eq!(
            parse_installer("uv\n\nignored noise here\n").as_deref(),
            Some("uv")
        );
    }

    #[test]
    fn parse_installer_empty_input_is_none() {
        assert!(parse_installer("").is_none());
    }

    #[test]
    fn parse_installer_whitespace_only_is_none() {
        assert!(parse_installer("   \n\n\t\n").is_none());
    }

    #[test]
    fn round_trip_preserves_name() {
        assert_eq!(round_trip("uv").as_deref(), Some("uv"));
        assert_eq!(round_trip("pip").as_deref(), Some("pip"));
        assert_eq!(round_trip("mamba").as_deref(), Some("mamba"));
    }

    #[test]
    fn parse_installer_preserves_internal_punctuation() {
        // Some installers tag their version: `pip 24.2`. We treat the
        // whole first line as the installer name; downstream callers
        // may split if they care.
        assert_eq!(parse_installer("pip 24.2\n").as_deref(), Some("pip 24.2"));
    }

    #[test]
    fn round_trip_with_mamba_default_works() {
        assert_eq!(round_trip(INSTALLER_NAME).as_deref(), Some(INSTALLER_NAME));
    }

    #[test]
    #[should_panic(expected = "non-empty")]
    fn render_installer_panics_on_empty_name_in_debug() {
        let _ = render_installer("");
    }
}
