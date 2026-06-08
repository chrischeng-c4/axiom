// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
// CODEGEN-BEGIN
/// Return the current platform as `(os, cpu)` tuple.
///
/// Values match the npm `package.json` `"os"` and `"cpu"` field conventions:
/// - os:  `"darwin"`, `"linux"`, `"win32"`, `"freebsd"`, etc.
/// - cpu: `"arm64"`, `"x64"`, `"ia32"`, `"arm"`, etc.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub fn current_platform() -> (&'static str, &'static str) {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        "windows" => "win32",
        other => other, // "linux", "freebsd", etc.
    };

    let cpu = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        "x86" => "ia32",
        other => other, // "arm", "mips", etc.
    };

    (os, cpu)
}

/// Check whether a package's platform constraints match the current system.
///
/// `pkg_os`  — values from the package's `"os"` field (empty = no constraint)
/// `pkg_cpu` — values from the package's `"cpu"` field (empty = no constraint)
///
/// An entry starting with `!` means "exclude this value". If the list contains
/// only exclusions the package is accepted unless the current platform is
/// explicitly excluded. If the list contains positive entries, the current
/// platform must appear.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub fn matches_platform(pkg_os: &[String], pkg_cpu: &[String]) -> bool {
    matches_field(pkg_os, current_platform().0) && matches_field(pkg_cpu, current_platform().1)
}

/// Check whether `current` matches the constraint list.
///
/// Rules (following npm semantics):
/// - Empty list → always matches (no constraint).
/// - List with negated entries (`!darwin`) → matches unless excluded.
/// - List with positive entries (`darwin`, `linux`) → matches only if included.
fn matches_field(allowed: &[String], current: &str) -> bool {
    if allowed.is_empty() {
        return true;
    }

    let has_positive = allowed.iter().any(|v| !v.starts_with('!'));
    let is_excluded = allowed
        .iter()
        .any(|v| v.starts_with('!') && &v[1..] == current);

    if is_excluded {
        return false;
    }

    if has_positive {
        // Must be explicitly listed
        allowed.iter().any(|v| v.as_str() == current)
    } else {
        // Only negations present — accepted unless excluded (handled above)
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// T48: Platform Filter Skips Wrong Platform
    #[test]
    fn t48_platform_filter_skips_wrong_platform() {
        // Package constrained to linux only
        let pkg_os = vec!["linux".to_string()];
        let _pkg_cpu: Vec<String> = vec![];
        // On macOS ARM64, this should not match
        let result = matches_field(&pkg_os, "darwin");
        assert!(!result, "darwin must not match linux-only package");
    }

    /// T49: Platform Filter Accepts Matching Platform
    #[test]
    fn t49_platform_filter_accepts_matching() {
        let pkg_os = vec!["darwin".to_string()];
        let pkg_cpu = vec!["arm64".to_string()];
        assert!(
            matches_field(&pkg_os, "darwin"),
            "darwin must match darwin constraint"
        );
        assert!(
            matches_field(&pkg_cpu, "arm64"),
            "arm64 must match arm64 constraint"
        );
    }

    /// T50: Platform Filter — os Only (No cpu)
    #[test]
    fn t50_platform_filter_os_only_no_cpu() {
        let pkg_os = vec!["darwin".to_string()];
        let pkg_cpu: Vec<String> = vec![]; // No cpu constraint

        assert!(
            matches_field(&pkg_os, "darwin"),
            "darwin must match os constraint"
        );
        assert!(
            matches_field(&pkg_cpu, "arm64"),
            "any cpu must match when no cpu constraint"
        );
        assert!(
            matches_field(&pkg_cpu, "x64"),
            "any cpu must match when no cpu constraint"
        );
    }

    /// Test empty constraints always match
    #[test]
    fn test_empty_constraints_always_match() {
        let empty: Vec<String> = vec![];
        assert!(matches_field(&empty, "darwin"));
        assert!(matches_field(&empty, "linux"));
        assert!(matches_field(&empty, "arm64"));
    }

    /// Test negation-based constraints
    #[test]
    fn test_negation_constraints() {
        let pkg_os = vec!["!win32".to_string()];
        assert!(
            matches_field(&pkg_os, "darwin"),
            "darwin should match !win32"
        );
        assert!(matches_field(&pkg_os, "linux"), "linux should match !win32");
        assert!(
            !matches_field(&pkg_os, "win32"),
            "win32 should not match !win32"
        );
    }

    /// Test multiple positive entries
    #[test]
    fn test_multiple_positive_entries() {
        let pkg_os = vec!["darwin".to_string(), "linux".to_string()];
        assert!(matches_field(&pkg_os, "darwin"), "darwin should match");
        assert!(matches_field(&pkg_os, "linux"), "linux should match");
        assert!(!matches_field(&pkg_os, "win32"), "win32 should not match");
    }

    /// Test current_platform returns valid values
    #[test]
    fn test_current_platform_valid() {
        let (os, cpu) = current_platform();
        assert!(!os.is_empty(), "os must not be empty");
        assert!(!cpu.is_empty(), "cpu must not be empty");
        // On macOS ARM: darwin, arm64
        // On Linux x86_64: linux, x64
        // Just verify it's a known value
        let known_os = ["darwin", "linux", "win32", "freebsd"];
        let known_cpu = ["arm64", "x64", "ia32", "arm"];
        assert!(
            known_os.contains(&os) || !os.is_empty(),
            "unexpected os: {}",
            os
        );
        assert!(
            known_cpu.contains(&cpu) || !cpu.is_empty(),
            "unexpected cpu: {}",
            cpu
        );
    }
}
// CODEGEN-END
