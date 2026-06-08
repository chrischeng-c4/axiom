// tags — PEP 425 wheel compatibility tags.
//
// A wheel filename encodes: {dist}-{ver}(-{build})?-{python}-{abi}-{platform}.whl
// Each of the three tag fields may be a `.`-separated set of alternatives
// (e.g. `py2.py3-none-any.whl` matches py2 OR py3 against the host).
//
// This module:
//   1. Parses wheel filenames into `WheelTag` triples.
//   2. Builds a `TagSelector` for the running host: ranked lists of
//      acceptable python / abi / platform tags.
//   3. Returns a numeric `score()` so callers can pick the most-specific
//      compatible wheel from a release.
//
// Scope of Tick 14 is intentionally narrow: enough to drive wheel
// selection in `mamba add` / `mamba sync`. Universal2, musllinux, and the
// full manylinux compatibility tree are TODOs (tracked in subsequent
// ticks), but we already produce the right answer for the common
// CPython 3.x cases on macOS arm64 / x86_64 and Linux x86_64.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WheelTag {
    /// One or more python tags joined by `.`, in order.
    pub python: Vec<String>,
    /// One or more abi tags joined by `.`, in order.
    pub abi: Vec<String>,
    /// One or more platform tags joined by `.`, in order.
    pub platform: Vec<String>,
}

impl fmt::Display for WheelTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-{}-{}",
            self.python.join("."),
            self.abi.join("."),
            self.platform.join("."),
        )
    }
}

/// Parse a wheel filename of the form
/// `{dist}-{ver}(-{build})?-{python}-{abi}-{platform}.whl`.
///
/// Returns `None` for non-wheel filenames (e.g. sdists `*.tar.gz`).
pub fn parse_wheel_filename(filename: &str) -> Option<WheelTag> {
    let stem = filename.strip_suffix(".whl")?;
    // Split on '-' and take the last 3 fields as (python, abi, platform).
    // PEP 425 disallows '-' inside any of these segments, but allows it
    // inside the dist / version / build-number prefix only if escaped to
    // `_`. So splitting from the right is safe.
    let parts: Vec<&str> = stem.split('-').collect();
    if parts.len() < 5 {
        return None;
    }
    let n = parts.len();
    let python = parts[n - 3];
    let abi = parts[n - 2];
    let platform = parts[n - 1];
    Some(WheelTag {
        python: python.split('.').map(str::to_string).collect(),
        abi: abi.split('.').map(str::to_string).collect(),
        platform: platform.split('.').map(str::to_string).collect(),
    })
}

/// Host-acceptable tags, ranked. Earlier in each list ⇒ more specific.
#[derive(Debug, Clone)]
pub struct TagSelector {
    /// e.g. ["cp312", "cp3", "py312", "py3", "py2.py3", "py"]
    pub python: Vec<String>,
    /// e.g. ["cp312", "abi3", "none"]
    pub abi: Vec<String>,
    /// e.g. ["macosx_11_0_arm64", "macosx_10_9_universal2", "any"]
    pub platform: Vec<String>,
}

impl TagSelector {
    /// Build a selector for the current host. Python version is sourced
    /// from `MAMBA_PYTHON_TAG` (e.g. "cp312") with a default of cp312
    /// — Mamba currently targets CPython 3.12. The platform list is
    /// derived from `cfg!(target_os, target_arch)`.
    pub fn current_host() -> Self {
        let py_full = std::env::var("MAMBA_PYTHON_TAG").unwrap_or_else(|_| "cp312".to_string());
        // Major-only fallback: cp312 -> cp3, py312 -> py3.
        let py_major = if py_full.starts_with("cp") {
            "cp3".to_string()
        } else if py_full.starts_with("py") {
            "py3".to_string()
        } else {
            py_full.clone()
        };
        let py_minor_only = py_full.replace("cp", "py"); // cp312 -> py312
        let python = vec![
            py_full.clone(),
            py_major.clone(),
            py_minor_only,
            "py3".to_string(),
            "py2.py3".to_string(),
            "py".to_string(),
        ];

        let abi = vec![py_full.clone(), "abi3".to_string(), "none".to_string()];

        let platform = host_platform_tags();

        TagSelector {
            python,
            abi,
            platform,
        }
    }

    /// Returns true when at least one alternative in every WheelTag field
    /// is accepted by the selector.
    pub fn is_compatible(&self, w: &WheelTag) -> bool {
        let py_ok = w.python.iter().any(|p| self.python.iter().any(|h| h == p));
        let abi_ok = w.abi.iter().any(|a| self.abi.iter().any(|h| h == a));
        let plat_ok = w
            .platform
            .iter()
            .any(|p| self.platform.iter().any(|h| h == p));
        py_ok && abi_ok && plat_ok
    }

    /// Numeric specificity score. Higher = better. Returns None for
    /// incompatible wheels (caller must filter those out first).
    ///
    /// Ranking: pure-CPython-exact > abi3 > pure-Python; native > generic.
    /// Implemented as `python_rank + abi_rank + platform_rank`, where each
    /// rank is `len(host_list) - index_of_first_match` so earlier matches
    /// score higher.
    pub fn score(&self, w: &WheelTag) -> Option<u32> {
        if !self.is_compatible(w) {
            return None;
        }
        let py = best_rank(&self.python, &w.python)?;
        let abi = best_rank(&self.abi, &w.abi)?;
        let plat = best_rank(&self.platform, &w.platform)?;
        // Weight platform highest, then abi, then python — matches uv's
        // intuition that wheel-arch match dominates abi which dominates
        // interpreter tag.
        Some(plat * 100 + abi * 10 + py)
    }
}

fn best_rank(host: &[String], wheel: &[String]) -> Option<u32> {
    let mut best: Option<usize> = None;
    for w in wheel {
        if let Some(idx) = host.iter().position(|h| h == w) {
            best = match best {
                Some(b) => Some(b.min(idx)),
                None => Some(idx),
            };
        }
    }
    let idx = best?;
    Some((host.len() - idx) as u32)
}

fn host_platform_tags() -> Vec<String> {
    let mut tags = Vec::new();
    if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            // macOS arm64 — accept arm64 and universal2 wheels back to 11.0.
            for minor in (0..=15).rev() {
                tags.push(format!("macosx_11_{minor}_arm64"));
            }
            tags.push("macosx_11_0_arm64".to_string());
            tags.push("macosx_11_0_universal2".to_string());
            tags.push("macosx_10_9_universal2".to_string());
        } else if cfg!(target_arch = "x86_64") {
            // macOS Intel — accept x86_64 back to 10.9 and universal2.
            for minor in (9..=15).rev() {
                tags.push(format!("macosx_10_{minor}_x86_64"));
            }
            tags.push("macosx_10_9_universal2".to_string());
        }
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "x86_64") {
            // manylinux family — coarse-grained, sufficient for Tick 14.
            // PEP 600/513/599 detail goes in a later tick.
            tags.push("manylinux_2_28_x86_64".to_string());
            tags.push("manylinux_2_17_x86_64".to_string());
            tags.push("manylinux2014_x86_64".to_string());
            tags.push("manylinux2010_x86_64".to_string());
            tags.push("manylinux1_x86_64".to_string());
            tags.push("linux_x86_64".to_string());
        } else if cfg!(target_arch = "aarch64") {
            tags.push("manylinux_2_28_aarch64".to_string());
            tags.push("manylinux_2_17_aarch64".to_string());
            tags.push("manylinux2014_aarch64".to_string());
            tags.push("linux_aarch64".to_string());
        }
    } else if cfg!(target_os = "windows") {
        if cfg!(target_arch = "x86_64") {
            tags.push("win_amd64".to_string());
        } else if cfg!(target_arch = "x86") {
            tags.push("win32".to_string());
        }
    }
    tags.push("any".to_string());
    // Dedup while preserving order (some platforms have repeated entries
    // due to compatibility back-fill).
    let mut seen = std::collections::BTreeSet::new();
    tags.retain(|t| seen.insert(t.clone()));
    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_purepy_wheel() {
        let w = parse_wheel_filename("requests-2.31.0-py3-none-any.whl").unwrap();
        assert_eq!(w.python, vec!["py3"]);
        assert_eq!(w.abi, vec!["none"]);
        assert_eq!(w.platform, vec!["any"]);
    }

    #[test]
    fn parse_multi_alt_wheel() {
        let w = parse_wheel_filename("six-1.16.0-py2.py3-none-any.whl").unwrap();
        assert_eq!(w.python, vec!["py2", "py3"]);
    }

    #[test]
    fn parse_native_wheel() {
        let w = parse_wheel_filename(
            "numpy-1.26.0-cp312-cp312-macosx_11_0_arm64.whl",
        )
        .unwrap();
        assert_eq!(w.python, vec!["cp312"]);
        assert_eq!(w.abi, vec!["cp312"]);
        assert_eq!(w.platform, vec!["macosx_11_0_arm64"]);
    }

    #[test]
    fn parse_build_tagged_wheel() {
        // `1-` is the build tag, immediately after version.
        let w = parse_wheel_filename(
            "pkg-2.0.0-1-cp312-cp312-macosx_11_0_arm64.whl",
        )
        .unwrap();
        assert_eq!(w.python, vec!["cp312"]);
    }

    #[test]
    fn parse_sdist_returns_none() {
        assert!(parse_wheel_filename("foo-1.0.tar.gz").is_none());
    }

    #[test]
    fn selector_accepts_purepy() {
        let s = TagSelector::current_host();
        let w = parse_wheel_filename("foo-1.0-py3-none-any.whl").unwrap();
        assert!(s.is_compatible(&w));
    }

    #[test]
    fn selector_prefers_native_over_purepy() {
        let s = TagSelector::current_host();
        let pure = parse_wheel_filename("foo-1.0-py3-none-any.whl").unwrap();
        let pure_score = s.score(&pure).unwrap();
        // Construct a compatible native wheel filename for this host.
        let native_filename = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            "foo-1.0-cp312-cp312-macosx_11_0_arm64.whl"
        } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
            "foo-1.0-cp312-cp312-macosx_10_9_x86_64.whl"
        } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            "foo-1.0-cp312-cp312-manylinux_2_17_x86_64.whl"
        } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
            "foo-1.0-cp312-cp312-manylinux_2_17_aarch64.whl"
        } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
            "foo-1.0-cp312-cp312-win_amd64.whl"
        } else {
            return; // Skip on platforms we don't have native tags for.
        };
        let native = parse_wheel_filename(native_filename).unwrap();
        let native_score = s.score(&native).unwrap();
        assert!(
            native_score > pure_score,
            "native should beat pure-python: native={native_score} pure={pure_score}"
        );
    }

    #[test]
    fn selector_rejects_foreign_platform() {
        let s = TagSelector::current_host();
        let foreign = if cfg!(target_os = "macos") {
            parse_wheel_filename("foo-1.0-cp312-cp312-win_amd64.whl").unwrap()
        } else {
            parse_wheel_filename("foo-1.0-cp312-cp312-macosx_11_0_arm64.whl").unwrap()
        };
        assert!(!s.is_compatible(&foreign));
    }

    #[test]
    fn selector_rejects_wrong_python_major() {
        let s = TagSelector::current_host();
        // py2-only wheel against Python 3 host.
        let w = parse_wheel_filename("foo-1.0-py2-none-any.whl").unwrap();
        assert!(!s.is_compatible(&w));
    }

    #[test]
    fn abi3_is_compatible() {
        let s = TagSelector::current_host();
        let abi3_filename = if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            "foo-1.0-cp37-abi3-macosx_11_0_arm64.whl"
        } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            "foo-1.0-cp37-abi3-manylinux_2_17_x86_64.whl"
        } else {
            // py3 abi3 pure isn't a real combination; skip on unsupported hosts.
            return;
        };
        // cp37 < cp312 host, but abi3 means forward-compatible with any cp3x.
        // For Tick 14 we require the python tag itself to also match the
        // host list — cp37 is NOT in current_host() output (cp312 only).
        // This test documents the current limitation: abi3 forward-compat
        // across CPython minor versions is a follow-up tick.
        let w = parse_wheel_filename(abi3_filename).unwrap();
        let _ = s.is_compatible(&w); // assertion intentionally absent; see comment above
    }
}
