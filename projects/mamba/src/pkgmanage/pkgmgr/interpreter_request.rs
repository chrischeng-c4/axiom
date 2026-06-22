// Interpreter request parsing + matching (Tick 49).
//
// Pure-data layer behind `uv venv --python` / `uv run --python`. Translates
// user-visible request strings (`3.12`, `cpython@3.12`, `pypy-3.10.13`) and
// `python -V` style output (`Python 3.12.4`, `PyPy 7.3.13 (Python 3.10.13)`)
// into typed records, and picks the best match from a list of discovered
// interpreters.
//
// The actual discovery (walking PATH, spawning `python -V`) lives in a later
// tick. This module is fully synchronous and deterministic so it can be
// unit-tested without any host Python install.

use std::path::PathBuf;

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Implementation family (CPython / PyPy / etc).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Implementation {
    CPython,
    PyPy,
    GraalPy,
    Other(String),
}

impl Implementation {
    /// Canonical lowercase tag, suitable for matching against PEP 425
    /// compatibility tags and `.python-version` prefixes.
    pub fn tag(&self) -> String {
        match self {
            Implementation::CPython => "cpython".into(),
            Implementation::PyPy => "pypy".into(),
            Implementation::GraalPy => "graalpy".into(),
            Implementation::Other(s) => s.to_ascii_lowercase(),
        }
    }
}

impl Implementation {
    /// Parse the leading word of a request or version line. Case-insensitive.
    pub fn parse(s: &str) -> Implementation {
        match s.to_ascii_lowercase().as_str() {
            "cpython" | "python" => Implementation::CPython,
            "pypy" => Implementation::PyPy,
            "graalpy" => Implementation::GraalPy,
            other => Implementation::Other(other.to_string()),
        }
    }
}

/// Version slice expressed by a request: major mandatory, minor / patch
/// optional. `3` matches any 3.x.y; `3.12` matches any 3.12.y; `3.12.4`
/// matches exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionRequest {
    pub major: u64,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
}

impl VersionRequest {
    /// True if the concrete `(major, minor, patch)` triple satisfies this
    /// prefix-style request.
    pub fn matches(&self, v: (u64, u64, u64)) -> bool {
        if v.0 != self.major {
            return false;
        }
        if let Some(m) = self.minor {
            if v.1 != m {
                return false;
            }
        }
        if let Some(p) = self.patch {
            if v.2 != p {
                return false;
            }
        }
        true
    }
}

/// A typed interpreter request as parsed from a `.python-version` line or
/// CLI `--python` argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterpreterRequest {
    /// Implementation family if specified; `None` means "any".
    pub implementation: Option<Implementation>,
    /// Version prefix if specified; `None` means "any version".
    pub version: Option<VersionRequest>,
    /// Raw source string for diagnostics; not consulted for matching.
    pub raw: String,
}

/// A discovered interpreter on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterpreterInfo {
    pub implementation: Implementation,
    /// `(major, minor, patch)` triple of the language version. For PyPy
    /// this is the *Python* level, not the PyPy release.
    pub version: (u64, u64, u64),
    pub executable: PathBuf,
}

impl InterpreterInfo {
    /// Pretty-print form for diagnostics (`cpython-3.12.4`).
    pub fn label(&self) -> String {
        format!(
            "{}-{}.{}.{}",
            self.implementation.tag(),
            self.version.0,
            self.version.1,
            self.version.2
        )
    }
}

// ---------------------------------------------------------------------------
// Request parsing
// ---------------------------------------------------------------------------

/// Parse a request string. Accepted shapes:
///
///   * `3`, `3.12`, `3.12.4` — bare version triple-prefix.
///   * `cpython` / `pypy` / `python3.12` — bare implementation.
///   * `cpython@3.12`, `pypy@3.10.13` — `@` separator.
///   * `cpython-3.12`, `pypy3.10`, `python-3.12.4` — pyenv-style.
///   * `python3` / `python3.12` — pyenv-style prefix.
pub fn parse_interpreter_request(src: &str) -> Result<InterpreterRequest, IndexError> {
    let trimmed = src.trim();
    if trimmed.is_empty() {
        return parse_err("empty interpreter request");
    }

    // Bare version: starts with a digit.
    if trimmed.chars().next().unwrap().is_ascii_digit() {
        let ver = parse_version_request(trimmed)?;
        return Ok(InterpreterRequest {
            implementation: None,
            version: Some(ver),
            raw: src.to_string(),
        });
    }

    // Find the impl/version boundary. Pick the first of `@`, `-`, or the
    // first digit after the leading word.
    let bytes = trimmed.as_bytes();
    let mut sep_idx: Option<usize> = None;
    let mut sep_consume = false;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'@' || b == b'-' {
            sep_idx = Some(i);
            sep_consume = true;
            break;
        }
        if b.is_ascii_digit() && i > 0 {
            sep_idx = Some(i);
            sep_consume = false;
            break;
        }
    }

    let (impl_part, ver_part) = match sep_idx {
        Some(i) => {
            let tail_start = if sep_consume { i + 1 } else { i };
            (&trimmed[..i], &trimmed[tail_start..])
        }
        None => (trimmed, ""),
    };

    let implementation = Some(Implementation::parse(impl_part));
    let version = if ver_part.is_empty() {
        None
    } else {
        Some(parse_version_request(ver_part)?)
    };

    Ok(InterpreterRequest {
        implementation,
        version,
        raw: src.to_string(),
    })
}

fn parse_version_request(src: &str) -> Result<VersionRequest, IndexError> {
    let parts: Vec<&str> = src.split('.').collect();
    if parts.is_empty() || parts.len() > 3 {
        return Err(IndexError::ParseError {
            url: "<interpreter>".into(),
            detail: format!("invalid version request '{}'", src),
        });
    }
    let parse_u = |s: &str| -> Result<u64, IndexError> {
        s.parse::<u64>().map_err(|_| IndexError::ParseError {
            url: "<interpreter>".into(),
            detail: format!("non-numeric version component '{}'", s),
        })
    };
    let major = parse_u(parts[0])?;
    let minor = match parts.get(1) {
        Some(s) if !s.is_empty() => Some(parse_u(s)?),
        _ => None,
    };
    let patch = match parts.get(2) {
        Some(s) if !s.is_empty() => Some(parse_u(s)?),
        _ => None,
    };
    Ok(VersionRequest {
        major,
        minor,
        patch,
    })
}

// ---------------------------------------------------------------------------
// `python -V` output parsing
// ---------------------------------------------------------------------------

/// Parse stdout from `python -V` or `python --version`. Handles:
///
///   * `Python 3.12.4`
///   * `PyPy 7.3.13 (Python 3.10.13)` — picks the trailing `(Python ...)`.
///   * `GraalPy 24.0.0 (Python 3.10.13)`
///
/// `executable` is stashed verbatim onto the returned record.
pub fn parse_python_version_output(out: &str, executable: PathBuf) -> Option<InterpreterInfo> {
    let line = out.lines().next()?.trim();
    if line.is_empty() {
        return None;
    }

    // Prefer the parenthesised `(Python X.Y.Z)` form when present so PyPy
    // and GraalPy reports yield their *language* version, not the
    // implementation release number.
    if let Some(open) = line.find("(Python ") {
        let after = &line[open + "(Python ".len()..];
        let close = after.find(')')?;
        let ver_str = after[..close].trim();
        let version = parse_version_triple(ver_str)?;
        let impl_word = line[..open].split_whitespace().next()?;
        return Some(InterpreterInfo {
            implementation: Implementation::parse(impl_word),
            version,
            executable,
        });
    }

    // Otherwise: `<Impl> X.Y.Z`.
    let mut iter = line.split_whitespace();
    let impl_word = iter.next()?;
    let ver_str = iter.next()?;
    let version = parse_version_triple(ver_str)?;
    Some(InterpreterInfo {
        implementation: Implementation::parse(impl_word),
        version,
        executable,
    })
}

fn parse_version_triple(s: &str) -> Option<(u64, u64, u64)> {
    let trimmed = s.split(|c: char| !c.is_ascii_digit() && c != '.').next()?;
    let parts: Vec<&str> = trimmed.split('.').collect();
    if parts.is_empty() {
        return None;
    }
    let parse = |s: &str| s.parse::<u64>().ok();
    let major = parse(parts[0])?;
    let minor = parts.get(1).and_then(|s| parse(s)).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| parse(s)).unwrap_or(0);
    Some((major, minor, patch))
}

// ---------------------------------------------------------------------------
// Matching
// ---------------------------------------------------------------------------

/// Pick the best interpreter satisfying `req` from `interpreters`, returning
/// a reference to it or `None` if no candidate matches.
///
/// Selection rules (in priority order):
///   1. Implementation must match if requested.
///   2. Version prefix must match if requested.
///   3. Among matching candidates, prefer the highest `(major, minor, patch)`
///      so callers can ask for `3.12` and get the latest patch.
pub fn match_request<'a>(
    interpreters: &'a [InterpreterInfo],
    req: &InterpreterRequest,
) -> Option<&'a InterpreterInfo> {
    interpreters
        .iter()
        .filter(|i| match &req.implementation {
            Some(want) => &i.implementation == want,
            None => true,
        })
        .filter(|i| match &req.version {
            Some(v) => v.matches(i.version),
            None => true,
        })
        .max_by_key(|i| i.version)
}

fn parse_err<T>(detail: &str) -> Result<T, IndexError> {
    Err(IndexError::ParseError {
        url: "<interpreter>".into(),
        detail: detail.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn pb(p: &str) -> PathBuf {
        PathBuf::from(p)
    }

    fn info(impl_: Implementation, v: (u64, u64, u64), exe: &str) -> InterpreterInfo {
        InterpreterInfo {
            implementation: impl_,
            version: v,
            executable: pb(exe),
        }
    }

    // ---- Request parsing -------------------------------------------------

    #[test]
    fn parses_bare_minor_version() {
        let r = parse_interpreter_request("3.12").unwrap();
        assert!(r.implementation.is_none());
        let v = r.version.unwrap();
        assert_eq!(v.major, 3);
        assert_eq!(v.minor, Some(12));
        assert_eq!(v.patch, None);
    }

    #[test]
    fn parses_bare_major_only() {
        let r = parse_interpreter_request("3").unwrap();
        let v = r.version.unwrap();
        assert_eq!((v.major, v.minor, v.patch), (3, None, None));
    }

    #[test]
    fn parses_bare_full_triple() {
        let r = parse_interpreter_request("3.12.4").unwrap();
        let v = r.version.unwrap();
        assert_eq!((v.major, v.minor, v.patch), (3, Some(12), Some(4)));
    }

    #[test]
    fn parses_at_separator() {
        let r = parse_interpreter_request("cpython@3.12").unwrap();
        assert_eq!(r.implementation, Some(Implementation::CPython));
        let v = r.version.unwrap();
        assert_eq!(v.minor, Some(12));
    }

    #[test]
    fn parses_dash_separator() {
        let r = parse_interpreter_request("pypy-3.10.13").unwrap();
        assert_eq!(r.implementation, Some(Implementation::PyPy));
        let v = r.version.unwrap();
        assert_eq!((v.major, v.minor, v.patch), (3, Some(10), Some(13)));
    }

    #[test]
    fn parses_pyenv_style_no_sep() {
        let r = parse_interpreter_request("python3.12").unwrap();
        assert_eq!(r.implementation, Some(Implementation::CPython));
        assert_eq!(r.version.unwrap().minor, Some(12));
    }

    #[test]
    fn parses_pypy_no_sep() {
        let r = parse_interpreter_request("pypy3.10").unwrap();
        assert_eq!(r.implementation, Some(Implementation::PyPy));
        assert_eq!(r.version.unwrap().minor, Some(10));
    }

    #[test]
    fn parses_bare_implementation_no_version() {
        let r = parse_interpreter_request("cpython").unwrap();
        assert_eq!(r.implementation, Some(Implementation::CPython));
        assert!(r.version.is_none());
    }

    #[test]
    fn rejects_empty_request() {
        assert!(parse_interpreter_request("").is_err());
        assert!(parse_interpreter_request("   ").is_err());
    }

    #[test]
    fn rejects_non_numeric_component() {
        assert!(parse_interpreter_request("3.x").is_err());
    }

    #[test]
    fn rejects_too_many_components() {
        assert!(parse_interpreter_request("3.12.4.1").is_err());
    }

    // ---- Implementation parse -------------------------------------------

    #[test]
    fn implementation_parse_case_insensitive() {
        assert_eq!(Implementation::parse("CPython"), Implementation::CPython);
        assert_eq!(Implementation::parse("PYPY"), Implementation::PyPy);
        assert_eq!(Implementation::parse("python"), Implementation::CPython);
        assert_eq!(
            Implementation::parse("Jython"),
            Implementation::Other("jython".into())
        );
    }

    #[test]
    fn implementation_tag_canonical() {
        assert_eq!(Implementation::CPython.tag(), "cpython");
        assert_eq!(Implementation::PyPy.tag(), "pypy");
        assert_eq!(Implementation::GraalPy.tag(), "graalpy");
        assert_eq!(Implementation::Other("Foo".into()).tag(), "foo");
    }

    // ---- VersionRequest matches -----------------------------------------

    #[test]
    fn version_request_prefix_matches() {
        let v = VersionRequest {
            major: 3,
            minor: Some(12),
            patch: None,
        };
        assert!(v.matches((3, 12, 0)));
        assert!(v.matches((3, 12, 4)));
        assert!(!v.matches((3, 11, 0)));
        assert!(!v.matches((2, 12, 0)));
    }

    #[test]
    fn version_request_exact_matches() {
        let v = VersionRequest {
            major: 3,
            minor: Some(12),
            patch: Some(4),
        };
        assert!(v.matches((3, 12, 4)));
        assert!(!v.matches((3, 12, 5)));
    }

    // ---- `python -V` output parsing -------------------------------------

    #[test]
    fn parses_cpython_v_output() {
        let i = parse_python_version_output("Python 3.12.4\n", pb("/u/bin/python3")).unwrap();
        assert_eq!(i.implementation, Implementation::CPython);
        assert_eq!(i.version, (3, 12, 4));
        assert_eq!(i.executable, pb("/u/bin/python3"));
    }

    #[test]
    fn parses_pypy_v_output_uses_python_level() {
        let i =
            parse_python_version_output("PyPy 7.3.13 (Python 3.10.13)", pb("/u/bin/pypy")).unwrap();
        assert_eq!(i.implementation, Implementation::PyPy);
        assert_eq!(i.version, (3, 10, 13));
    }

    #[test]
    fn parses_graalpy_v_output() {
        let i =
            parse_python_version_output("GraalPy 24.0.0 (Python 3.10.13)", pb("/u/bin/graalpy"))
                .unwrap();
        assert_eq!(i.implementation, Implementation::GraalPy);
        assert_eq!(i.version, (3, 10, 13));
    }

    #[test]
    fn parses_python_two_component_version() {
        let i = parse_python_version_output("Python 3.12", pb("/u/bin/p")).unwrap();
        assert_eq!(i.version, (3, 12, 0));
    }

    #[test]
    fn rejects_unparseable_v_output() {
        assert!(parse_python_version_output("", pb("/u/bin/p")).is_none());
        assert!(parse_python_version_output("garbage", pb("/u/bin/p")).is_none());
    }

    // ---- Label ----------------------------------------------------------

    #[test]
    fn label_is_canonical() {
        let i = info(Implementation::CPython, (3, 12, 4), "/u/bin/python");
        assert_eq!(i.label(), "cpython-3.12.4");
    }

    // ---- Matching -------------------------------------------------------

    fn fixture() -> Vec<InterpreterInfo> {
        vec![
            info(Implementation::CPython, (3, 11, 9), "/u/bin/python3.11"),
            info(Implementation::CPython, (3, 12, 4), "/u/bin/python3.12"),
            info(Implementation::CPython, (3, 12, 2), "/u/bin/p312-old"),
            info(Implementation::PyPy, (3, 10, 13), "/u/bin/pypy3.10"),
        ]
    }

    #[test]
    fn matches_exact_prefix_picks_latest_patch() {
        let req = parse_interpreter_request("3.12").unwrap();
        let pool = fixture();
        let m = match_request(&pool, &req).unwrap();
        assert_eq!(m.version, (3, 12, 4));
        assert_eq!(m.executable, pb("/u/bin/python3.12"));
    }

    #[test]
    fn matches_with_implementation_filter() {
        let req = parse_interpreter_request("pypy").unwrap();
        let pool = fixture();
        let m = match_request(&pool, &req).unwrap();
        assert_eq!(m.implementation, Implementation::PyPy);
    }

    #[test]
    fn matches_implementation_plus_version() {
        let req = parse_interpreter_request("cpython@3.11").unwrap();
        let pool = fixture();
        let m = match_request(&pool, &req).unwrap();
        assert_eq!(m.version, (3, 11, 9));
    }

    #[test]
    fn no_match_returns_none() {
        let req = parse_interpreter_request("3.9").unwrap();
        let pool = fixture();
        assert!(match_request(&pool, &req).is_none());
    }

    #[test]
    fn empty_list_returns_none() {
        let req = parse_interpreter_request("3.12").unwrap();
        assert!(match_request(&[], &req).is_none());
    }

    #[test]
    fn picks_highest_when_no_request_constraints() {
        let req = InterpreterRequest {
            implementation: None,
            version: None,
            raw: "any".into(),
        };
        let pool = fixture();
        let m = match_request(&pool, &req).unwrap();
        // 3.12.4 is the global max in the fixture.
        assert_eq!(m.version, (3, 12, 4));
    }
}
