// Python toolchain — version model, system discovery, `.python-version`
// pin file handling (Tick 21).
//
// uv exposes a `python` verb with sub-commands `list`, `install`,
// `uninstall`, `find`, `pin`, `dir`. This module is the *foundation* layer
// — the parts that don't need to download a new interpreter:
//
//   - `PythonVersion` with PEP 440-ish parse/display
//   - `PythonRequest` matching against installed interpreters
//   - `discover_system_pythons()` — walk `PATH` and identify python
//     candidates by filename, then ask each one for its `sys.version_info`
//   - `read_python_pin` / `write_python_pin` — the `.python-version` file
//     used by `uv python pin`
//
// Managed Python download from remote python-build-standalone artifacts lands
// above this foundation layer. The local/offline command surface can still
// register an existing source interpreter into the managed Python dir.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use crate::pkgmanage::pkgmgr::types::IndexError;

/// `(major, minor, patch)` triple. We don't carry pre/post/dev tags here —
/// CPython releases are simple integer triples and uv treats every other
/// segment as ignorable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PythonVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl PythonVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl std::fmt::Display for PythonVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for PythonVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim().trim_start_matches("cpython-");
        let parts: Vec<&str> = trimmed
            .split(|c: char| c == '.' || c == '-' || c == '_' || c == '+')
            .filter(|p| !p.is_empty())
            .collect();

        if parts.is_empty() {
            return Err(format!("empty python version string: {s:?}"));
        }

        // We want the first 1..=3 numeric segments. Everything after that is
        // tag noise (rc1, +tag, etc.) — silently dropped.
        let mut numeric: Vec<u32> = Vec::with_capacity(3);
        for p in parts {
            if numeric.len() == 3 {
                break;
            }
            match p.parse::<u32>() {
                Ok(n) => numeric.push(n),
                Err(_) => break,
            }
        }

        if numeric.is_empty() {
            return Err(format!("no numeric components in python version: {s:?}"));
        }

        let major = numeric[0];
        let minor = numeric.get(1).copied().unwrap_or(0);
        let patch = numeric.get(2).copied().unwrap_or(0);
        Ok(PythonVersion {
            major,
            minor,
            patch,
        })
    }
}

/// A user-supplied Python request. uv accepts requests at three granularity
/// levels, plus a wildcard. Matching semantics mirror uv:
///   `3` → any installed 3.x
///   `3.12` → any installed 3.12.x
///   `3.12.7` → exactly 3.12.7
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonRequest {
    /// `python` / unspecified — match any installed interpreter.
    Any,
    /// `3` — match any patch+minor of this major.
    Major(u32),
    /// `3.12` — match any patch of this minor.
    MajorMinor(u32, u32),
    /// `3.12.7` — exact.
    Exact(PythonVersion),
}

impl PythonRequest {
    pub fn matches(&self, version: &PythonVersion) -> bool {
        match self {
            PythonRequest::Any => true,
            PythonRequest::Major(maj) => version.major == *maj,
            PythonRequest::MajorMinor(maj, min) => version.major == *maj && version.minor == *min,
            PythonRequest::Exact(want) => *want == *version,
        }
    }
}

impl FromStr for PythonRequest {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() || trimmed == "*" || trimmed == "any" || trimmed == "python" {
            return Ok(PythonRequest::Any);
        }
        let parts: Vec<&str> = trimmed.split('.').collect();
        match parts.as_slice() {
            [maj] => {
                let n = maj
                    .parse::<u32>()
                    .map_err(|_| format!("bad python major: {maj:?}"))?;
                Ok(PythonRequest::Major(n))
            }
            [maj, min] => {
                let maj = maj
                    .parse::<u32>()
                    .map_err(|_| format!("bad python major: {maj:?}"))?;
                let min = min
                    .parse::<u32>()
                    .map_err(|_| format!("bad python minor: {min:?}"))?;
                Ok(PythonRequest::MajorMinor(maj, min))
            }
            [maj, min, patch] => {
                let v = PythonVersion::from_str(&format!("{maj}.{min}.{patch}"))?;
                Ok(PythonRequest::Exact(v))
            }
            _ => Err(format!("unrecognized python request: {s:?}")),
        }
    }
}

/// A discovered Python interpreter on the local system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredPython {
    pub version: PythonVersion,
    pub path: PathBuf,
}

/// Filenames that *might* be a Python interpreter. We scan PATH for these
/// and probe each match. The list mirrors uv's discovery heuristic — the
/// versioned forms first (more informative if multiple exist), then the
/// generic ones.
const PYTHON_EXECUTABLE_NAMES: &[&str] = &[
    "python3.13",
    "python3.12",
    "python3.11",
    "python3.10",
    "python3.9",
    "python3",
    "python",
];

/// Walk every directory on `PATH` and return discovered Python interpreters.
///
/// Each candidate is probed with `python -c "import sys; print(sys.version_info.major, ..."`
/// so only interpreters that actually run on this host are returned. Results
/// are deduped by canonical path (multiple aliases pointing to the same
/// executable are collapsed). Order is stable: by descending version, then
/// by first-found path.
pub fn discover_system_pythons() -> Result<Vec<DiscoveredPython>, IndexError> {
    let path_var = env::var_os("PATH").unwrap_or_default();
    let mut seen_canonical: Vec<PathBuf> = Vec::new();
    let mut discovered: Vec<DiscoveredPython> = Vec::new();

    for dir in env::split_paths(&path_var) {
        for name in PYTHON_EXECUTABLE_NAMES {
            let candidate = dir.join(name);
            if !candidate.is_file() {
                continue;
            }
            let canonical = fs::canonicalize(&candidate).unwrap_or(candidate.clone());
            if seen_canonical.contains(&canonical) {
                continue;
            }
            match probe_python_version(&candidate) {
                Ok(version) => {
                    seen_canonical.push(canonical);
                    discovered.push(DiscoveredPython {
                        version,
                        path: candidate,
                    });
                }
                Err(_) => {
                    // Not a working interpreter — skip silently. e.g. broken
                    // venv shim, wrong architecture, missing shared lib.
                }
            }
        }
    }

    discovered.sort_by(|a, b| b.version.cmp(&a.version).then(a.path.cmp(&b.path)));
    Ok(discovered)
}

/// Run `<python> -c "import sys; print(*sys.version_info[:3])"` and parse
/// the response. Public so callers can probe a specific path (e.g. a venv
/// python) on demand.
pub fn probe_python_version(python: &Path) -> Result<PythonVersion, IndexError> {
    let output = Command::new(python)
        .args([
            "-I", // ignore env (PYTHON*), like uv
            "-c",
            "import sys; print(sys.version_info[0], sys.version_info[1], sys.version_info[2])",
        ])
        .output()
        .map_err(|err| IndexError::NetworkError {
            url: python.display().to_string(),
            detail: format!("spawning python: {err}"),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(IndexError::NetworkError {
            url: python.display().to_string(),
            detail: format!(
                "python exited with status {:?}: {}",
                output.status.code(),
                stderr.trim()
            ),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next().unwrap_or("").trim();
    let nums: Vec<&str> = line.split_whitespace().collect();
    if nums.len() < 3 {
        return Err(IndexError::ParseError {
            url: python.display().to_string(),
            detail: format!("unexpected sys.version_info output: {line:?}"),
        });
    }
    let parse = |s: &str| -> Result<u32, IndexError> {
        s.parse::<u32>().map_err(|err| IndexError::ParseError {
            url: python.display().to_string(),
            detail: format!("non-integer version component {s:?}: {err}"),
        })
    };
    Ok(PythonVersion {
        major: parse(nums[0])?,
        minor: parse(nums[1])?,
        patch: parse(nums[2])?,
    })
}

/// Pick the highest-versioned discovered Python that satisfies `request`.
/// Returns `None` if nothing matches.
pub fn select_python<'a>(
    request: &PythonRequest,
    candidates: &'a [DiscoveredPython],
) -> Option<&'a DiscoveredPython> {
    candidates
        .iter()
        .filter(|c| request.matches(&c.version))
        .max_by_key(|c| c.version)
}

/// Read the `.python-version` file at `<project_root>/.python-version`.
/// Returns `Ok(None)` when the file doesn't exist. Trims surrounding
/// whitespace and ignores blank lines / lines starting with `#`.
pub fn read_python_pin(project_root: &Path) -> Result<Option<PythonRequest>, IndexError> {
    let pin = project_root.join(".python-version");
    let contents = match fs::read_to_string(&pin) {
        Ok(s) => s,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(IndexError::CacheIo {
                path: pin.display().to_string(),
                detail: format!("reading .python-version: {err}"),
            });
        }
    };

    // Pick the first non-blank, non-comment line. uv supports multiple pins
    // (one per line) but always tries them in order — the first one wins for
    // our purposes here.
    let first = contents
        .lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty() && !l.starts_with('#'));

    let Some(line) = first else {
        return Ok(None);
    };

    PythonRequest::from_str(line)
        .map(Some)
        .map_err(|err| IndexError::ParseError {
            url: pin.display().to_string(),
            detail: err,
        })
}

/// Write a `.python-version` file pinning the project to `request`.
/// Overwrites any existing pin. `Any` is rejected — a bare `*` pin would
/// be a no-op that's almost certainly an authoring mistake.
pub fn write_python_pin(project_root: &Path, request: &PythonRequest) -> Result<(), IndexError> {
    let body = match request {
        PythonRequest::Any => {
            return Err(IndexError::ParseError {
                url: "<.python-version>".into(),
                detail: "refusing to write an unconstrained `Any` python pin".into(),
            });
        }
        PythonRequest::Major(m) => format!("{m}\n"),
        PythonRequest::MajorMinor(maj, min) => format!("{maj}.{min}\n"),
        PythonRequest::Exact(v) => format!("{v}\n"),
    };

    let pin = project_root.join(".python-version");
    fs::write(&pin, body).map_err(|err| IndexError::CacheIo {
        path: pin.display().to_string(),
        detail: format!("writing .python-version: {err}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn parse_python_version_full_triple() {
        assert_eq!(
            "3.12.7".parse::<PythonVersion>().unwrap(),
            PythonVersion::new(3, 12, 7)
        );
    }

    #[test]
    fn parse_python_version_strips_cpython_prefix() {
        assert_eq!(
            "cpython-3.13.0".parse::<PythonVersion>().unwrap(),
            PythonVersion::new(3, 13, 0)
        );
    }

    #[test]
    fn parse_python_version_drops_tag_suffix() {
        assert_eq!(
            "3.13.0rc1".parse::<PythonVersion>().unwrap(),
            PythonVersion::new(3, 13, 0)
        );
        assert_eq!(
            "3.12+local".parse::<PythonVersion>().unwrap(),
            PythonVersion::new(3, 12, 0)
        );
    }

    #[test]
    fn parse_python_version_two_segments_zero_patch() {
        assert_eq!(
            "3.12".parse::<PythonVersion>().unwrap(),
            PythonVersion::new(3, 12, 0)
        );
    }

    #[test]
    fn parse_python_version_rejects_empty_and_non_numeric() {
        assert!("".parse::<PythonVersion>().is_err());
        assert!("rc1".parse::<PythonVersion>().is_err());
    }

    #[test]
    fn python_version_orders_by_components() {
        assert!(PythonVersion::new(3, 12, 0) < PythonVersion::new(3, 13, 0));
        assert!(PythonVersion::new(3, 12, 7) > PythonVersion::new(3, 12, 6));
        assert!(PythonVersion::new(4, 0, 0) > PythonVersion::new(3, 99, 99));
    }

    #[test]
    fn parse_python_request_all_shapes() {
        assert_eq!("".parse::<PythonRequest>().unwrap(), PythonRequest::Any);
        assert_eq!("any".parse::<PythonRequest>().unwrap(), PythonRequest::Any);
        assert_eq!("*".parse::<PythonRequest>().unwrap(), PythonRequest::Any);
        assert_eq!(
            "3".parse::<PythonRequest>().unwrap(),
            PythonRequest::Major(3)
        );
        assert_eq!(
            "3.12".parse::<PythonRequest>().unwrap(),
            PythonRequest::MajorMinor(3, 12)
        );
        assert_eq!(
            "3.12.7".parse::<PythonRequest>().unwrap(),
            PythonRequest::Exact(PythonVersion::new(3, 12, 7))
        );
    }

    #[test]
    fn python_request_matches() {
        let v = PythonVersion::new(3, 12, 7);
        assert!(PythonRequest::Any.matches(&v));
        assert!(PythonRequest::Major(3).matches(&v));
        assert!(!PythonRequest::Major(4).matches(&v));
        assert!(PythonRequest::MajorMinor(3, 12).matches(&v));
        assert!(!PythonRequest::MajorMinor(3, 11).matches(&v));
        assert!(PythonRequest::Exact(v).matches(&v));
        assert!(!PythonRequest::Exact(PythonVersion::new(3, 12, 6)).matches(&v));
    }

    #[test]
    fn select_python_picks_highest_match() {
        let candidates = vec![
            DiscoveredPython {
                version: PythonVersion::new(3, 11, 8),
                path: PathBuf::from("/usr/bin/python3.11"),
            },
            DiscoveredPython {
                version: PythonVersion::new(3, 12, 7),
                path: PathBuf::from("/usr/bin/python3.12"),
            },
            DiscoveredPython {
                version: PythonVersion::new(3, 12, 9),
                path: PathBuf::from("/opt/local/bin/python3.12"),
            },
        ];

        let pick = select_python(&PythonRequest::MajorMinor(3, 12), &candidates).unwrap();
        assert_eq!(pick.version, PythonVersion::new(3, 12, 9));

        let pick = select_python(&PythonRequest::Major(3), &candidates).unwrap();
        assert_eq!(pick.version, PythonVersion::new(3, 12, 9));

        assert!(select_python(&PythonRequest::Major(4), &candidates).is_none());
    }

    #[test]
    fn read_pin_returns_none_when_missing() {
        let dir = TempDir::new().unwrap();
        let pin = read_python_pin(dir.path()).unwrap();
        assert!(pin.is_none());
    }

    #[test]
    fn read_pin_strips_comments_and_blanks() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join(".python-version"),
            "# top-level comment\n\n   3.12  \n3.11\n",
        )
        .unwrap();
        let pin = read_python_pin(dir.path()).unwrap().unwrap();
        assert_eq!(pin, PythonRequest::MajorMinor(3, 12));
    }

    #[test]
    fn read_pin_exact_triple() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(".python-version"), "3.12.7\n").unwrap();
        let pin = read_python_pin(dir.path()).unwrap().unwrap();
        assert_eq!(pin, PythonRequest::Exact(PythonVersion::new(3, 12, 7)));
    }

    #[test]
    fn write_pin_round_trips_through_read() {
        let dir = TempDir::new().unwrap();
        let original = PythonRequest::MajorMinor(3, 12);
        write_python_pin(dir.path(), &original).unwrap();
        let read_back = read_python_pin(dir.path()).unwrap().unwrap();
        assert_eq!(read_back, original);
    }

    #[test]
    fn write_pin_refuses_any() {
        let dir = TempDir::new().unwrap();
        let err = write_python_pin(dir.path(), &PythonRequest::Any).unwrap_err();
        assert!(format!("{err}").contains("unconstrained"));
    }

    #[test]
    fn discover_system_pythons_returns_running_python() {
        // This test is best-effort: most dev machines have at least one
        // python3 on PATH. If not, we soft-skip rather than fail CI.
        let pythons = discover_system_pythons().expect("discovery should not error");
        if pythons.is_empty() {
            eprintln!("(no python3 on PATH — skipping discover_system_pythons body check)");
            return;
        }
        for py in &pythons {
            assert!(py.version.major >= 2, "absurd major: {}", py.version);
            assert!(py.path.is_file(), "{} should be a file", py.path.display());
        }
        // Sorted by descending version.
        for w in pythons.windows(2) {
            assert!(
                w[0].version >= w[1].version,
                "discovery not sorted: {} then {}",
                w[0].version,
                w[1].version
            );
        }
    }

    #[test]
    fn probe_python_version_matches_running_python() {
        // Same soft-skip discipline as above.
        let Some(python_path) = first_python_on_path() else {
            eprintln!("(no python3 on PATH — skipping probe)");
            return;
        };
        let v = probe_python_version(&python_path).expect("running python should probe");
        assert!(v.major >= 3, "got {v}");
    }

    fn first_python_on_path() -> Option<PathBuf> {
        let path_var = std::env::var_os("PATH")?;
        for dir in std::env::split_paths(&path_var) {
            for name in ["python3", "python"] {
                let p = dir.join(name);
                if p.is_file() {
                    return Some(p);
                }
            }
        }
        None
    }
}
