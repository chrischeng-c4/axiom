// Build-time constraint file reader (Tick 129).
//
// uv distinguishes between two kinds of constraint files:
//
//   * `--constraint <file>` / `UV_CONSTRAINT` / `PIP_CONSTRAINT`:
//     pins the *runtime* dependency resolver (what version of
//     each transitive runtime dep is allowed). Handled by
//     `constraints.rs`.
//
//   * `--build-constraint <file>` / `UV_BUILD_CONSTRAINT` /
//     `PIP_BUILD_CONSTRAINT`: pins the *build-time* environment
//     used to invoke each PEP 517 sdist build backend. Same file
//     format, but applied to a different graph (the build-system
//     `requires` from each package's pyproject.toml).
//
// The file format is a subset of pip's requirements format:
//
//   * One PEP 508 requirement per line.
//   * `#` to end-of-line is a comment.
//   * Blank lines ignored.
//   * Line continuation via trailing `\` (joins physical lines).
//   * `-r <file>` / `--requirement <file>` includes another constraint
//     file (recorded as `Include` so the loader can recurse).
//
// Like pip, constraints differ from requirements in that:
//
//   * Each constraint must name a package and may carry a specifier
//     set or marker, but MUST NOT carry extras (`pkg[foo]` is an
//     error in constraint files).
//   * Each constraint must NOT be a direct URL or VCS reference.
//
// This module is a pure parser. Resolver integration is in
// `resolver.rs`; CLI plumbing is in the caller.

use crate::pkgmanage::pkgmgr::requirement_string::Requirement;
use crate::pkgmanage::pkgmgr::types::IndexError;

const DETAIL: &str = "<build-constraints>";

/// One parsed line of a build-constraints file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildConstraintLine {
    /// A normal constraint (`pkg>=1.2`).
    Constraint(Requirement),
    /// A nested include (`-r other-constraints.txt`).
    Include(String),
}

/// A parsed build-constraints file body.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildConstraints {
    pub lines: Vec<BuildConstraintLine>,
}

impl BuildConstraints {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Project to only the typed constraints, discarding `-r` includes.
    /// Useful when the include graph has already been flattened by the
    /// loader.
    pub fn constraints(&self) -> Vec<&Requirement> {
        self.lines
            .iter()
            .filter_map(|l| match l {
                BuildConstraintLine::Constraint(r) => Some(r),
                BuildConstraintLine::Include(_) => None,
            })
            .collect()
    }

    /// Project to only the include directives.
    pub fn includes(&self) -> Vec<&str> {
        self.lines
            .iter()
            .filter_map(|l| match l {
                BuildConstraintLine::Include(p) => Some(p.as_str()),
                BuildConstraintLine::Constraint(_) => None,
            })
            .collect()
    }
}

/// Parse the body of a build-constraints file.
pub fn parse_build_constraints(src: &str) -> Result<BuildConstraints, IndexError> {
    let logical_lines = join_continuations(src);
    let mut out = BuildConstraints::default();

    for (lineno, raw_line) in logical_lines {
        let line = strip_comment(&raw_line);
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Handle `-r <file>` / `--requirement <file>` includes. The
        // bare flag with no path is still recognized so we can emit a
        // targeted error rather than the generic "unsupported flag".
        if trimmed == "--requirement" || trimmed == "-r" {
            return Err(IndexError::ParseError {
                url: DETAIL.into(),
                detail: format!("line {lineno}: {trimmed} without a path"),
            });
        }
        if let Some(rest) = trimmed
            .strip_prefix("--requirement ")
            .or_else(|| trimmed.strip_prefix("-r "))
        {
            let path = rest.trim();
            if path.is_empty() {
                return Err(IndexError::ParseError {
                    url: DETAIL.into(),
                    detail: format!("line {lineno}: --requirement without a path"),
                });
            }
            out.lines
                .push(BuildConstraintLine::Include(path.to_string()));
            continue;
        }

        // Reject other flags that don't make sense in constraint files.
        if trimmed.starts_with('-') {
            return Err(IndexError::ParseError {
                url: DETAIL.into(),
                detail: format!(
                    "line {lineno}: unsupported flag `{trimmed}` in build-constraints file"
                ),
            });
        }

        let req = Requirement::parse(trimmed).map_err(|e| IndexError::ParseError {
            url: DETAIL.into(),
            detail: format!("line {lineno}: {e:?}"),
        })?;

        validate_constraint(&req, lineno)?;
        out.lines.push(BuildConstraintLine::Constraint(req));
    }

    Ok(out)
}

/// pip + uv reject constraints that carry extras or that are direct
/// URL / VCS references — those constrain the *identity* of the
/// candidate rather than its version, which constraint files aren't
/// allowed to do.
fn validate_constraint(req: &Requirement, lineno: usize) -> Result<(), IndexError> {
    if !req.extras.extras.is_empty() {
        return Err(IndexError::ParseError {
            url: DETAIL.into(),
            detail: format!(
                "line {lineno}: constraint `{name}` must not carry extras",
                name = req.name
            ),
        });
    }
    if req.url.is_some() {
        return Err(IndexError::ParseError {
            url: DETAIL.into(),
            detail: format!(
                "line {lineno}: constraint `{name}` must not be a direct URL reference",
                name = req.name
            ),
        });
    }
    Ok(())
}

/// Strip a `#`-comment that starts a comment outside the (unsupported
/// for constraints) URL portion. Constraint lines don't contain `#`
/// fragments — uv's constraint reader strips all `#`-to-EOL the same
/// way pip does.
fn strip_comment(line: &str) -> &str {
    match line.find('#') {
        Some(idx) => &line[..idx],
        None => line,
    }
}

/// Resolve trailing-backslash line continuations. Returns the joined
/// logical lines paired with the *first* physical-line number that
/// contributed to each logical line — that's the diagnostic anchor
/// pip uses.
fn join_continuations(src: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut current: Option<(usize, String)> = None;
    for (idx, raw) in src.lines().enumerate() {
        let lineno = idx + 1;
        let trimmed_end = raw.trim_end_matches(['\r']);
        if let Some(stripped) = trimmed_end.strip_suffix('\\') {
            match &mut current {
                Some((_, buf)) => buf.push_str(stripped),
                None => current = Some((lineno, stripped.to_string())),
            }
        } else {
            let logical = match current.take() {
                Some((start, mut buf)) => {
                    buf.push_str(trimmed_end);
                    (start, buf)
                }
                None => (lineno, trimmed_end.to_string()),
            };
            out.push(logical);
        }
    }
    // Trailing backslash on the last line (no following EOL) — flush
    // whatever we accumulated.
    if let Some((start, buf)) = current.take() {
        out.push((start, buf));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_empty_output() {
        let r = parse_build_constraints("").unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn parses_simple_constraints() {
        let src = "setuptools>=65\nwheel>=0.40\nCython>=3.0\n";
        let r = parse_build_constraints(src).unwrap();
        let names: Vec<_> = r.constraints().iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["setuptools", "wheel", "cython"]);
    }

    #[test]
    fn comments_and_blank_lines_ignored() {
        let src = "\n# header comment\n  \nsetuptools>=65 # inline\n";
        let r = parse_build_constraints(src).unwrap();
        assert_eq!(r.constraints().len(), 1);
        assert_eq!(r.constraints()[0].name, "setuptools");
    }

    #[test]
    fn parses_nested_requirement_includes() {
        let src = "-r common.txt\n--requirement extras.txt\nsetuptools>=65\n";
        let r = parse_build_constraints(src).unwrap();
        assert_eq!(r.includes(), vec!["common.txt", "extras.txt"]);
        assert_eq!(r.constraints().len(), 1);
    }

    #[test]
    fn line_continuation_joins_logical_lines() {
        let src = "setuptools \\\n  >=65\n";
        let r = parse_build_constraints(src).unwrap();
        let names: Vec<_> = r.constraints().iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["setuptools"]);
    }

    #[test]
    fn rejects_constraint_with_extras() {
        let src = "setuptools[ssl]>=65\n";
        let err = parse_build_constraints(src).unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("must not carry extras"));
    }

    #[test]
    fn rejects_unsupported_flag() {
        let src = "--no-binary :all:\n";
        let err = parse_build_constraints(src).unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("unsupported flag"));
    }

    #[test]
    fn rejects_include_without_path() {
        let src = "-r\n";
        // Note: the input has no trailing space, so the `strip_prefix`
        // for `-r ` won't match — it falls through to the "unsupported
        // flag" branch instead. That's accurate; a bare `-r` is
        // unsupported too.
        let err = parse_build_constraints(src).unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("unsupported flag") || msg.contains("without a path"));
    }

    #[test]
    fn rejects_requirement_flag_without_path() {
        let src = "--requirement   \n";
        let err = parse_build_constraints(src).unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("--requirement without a path"));
    }

    #[test]
    fn marker_is_preserved_on_constraint() {
        let src = "setuptools>=65 ; python_version<'3.12'\n";
        let r = parse_build_constraints(src).unwrap();
        let req = r.constraints()[0];
        assert_eq!(req.name, "setuptools");
        assert!(req.marker.is_some());
    }

    #[test]
    fn crlf_line_endings_handled() {
        let src = "setuptools>=65\r\nwheel>=0.40\r\n";
        let r = parse_build_constraints(src).unwrap();
        let names: Vec<_> = r.constraints().iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["setuptools", "wheel"]);
    }

    #[test]
    fn realistic_uv_build_constraints_corpus() {
        // Pattern from a real uv build-constraint file used to pin
        // a corporate-mirrored build environment for sdists.
        let src = r#"
# Build-time pins for corp PyPI mirror.
-r common-build-pins.txt
setuptools>=65,<80
wheel>=0.40
Cython>=3.0,<4 ; python_version >= "3.9"
hatchling>=1.18
"#;
        let r = parse_build_constraints(src).unwrap();
        assert_eq!(r.includes(), vec!["common-build-pins.txt"]);
        let names: Vec<_> = r.constraints().iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, vec!["setuptools", "wheel", "cython", "hatchling"]);
    }

    #[test]
    fn empty_after_comment_strip_is_skipped() {
        let r = parse_build_constraints("setuptools>=65#trailing\n").unwrap();
        assert_eq!(r.constraints().len(), 1);
        assert_eq!(r.constraints()[0].name, "setuptools");
    }

    #[test]
    fn includes_strip_whitespace() {
        let r = parse_build_constraints("-r   ./pinned/build.txt   \n").unwrap();
        assert_eq!(r.includes(), vec!["./pinned/build.txt"]);
    }
}
