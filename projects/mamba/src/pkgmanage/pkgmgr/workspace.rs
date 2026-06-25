// Workspace member support — `[tool.uv.workspace]` parity (Tick 20).
//
// uv lets a single `pyproject.toml` declare a workspace whose members are
// other local Python packages, e.g.:
//
//     [tool.uv.workspace]
//     members = ["packages/*", "extras/foo"]
//     exclude = ["packages/legacy"]
//
// This module owns the *data layer*: parse the table out of a pyproject TOML
// string, glob-expand the member patterns against the filesystem rooted at
// the workspace `pyproject.toml`, and read each member's own `[project]`
// table to produce a typed `WorkspaceMember`.
//
// Resolver/sync wire-up (treating workspace members as local editable sources
// that share one lockfile) lives in a follow-up tick — this module is pure
// data so it can be unit-tested without a registry or a virtual env.
//
// Pattern syntax (subset of uv's globber):
// - Literal path:               `packages/foo`        — match exactly this dir
// - Single `*` per segment:     `packages/*`          — every immediate child
// - Mixed literal + `*`:        `packages/foo-*`      — every child prefixed `foo-`
// - Multi-segment globbing (`**`) is *not* supported; uv documents `**` but
//   for first-cut parity we cover the dominant `packages/*` shape and reject
//   `**` early with a clear error so callers know to upgrade later.
//
// All paths in the returned `WorkspaceMember` are absolute (joined onto the
// workspace root), so downstream resolvers/installers don't have to track
// the relative-to-workspace context.

use std::fs;
use std::path::{Path, PathBuf};

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Parsed `[tool.uv.workspace]` table.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceConfig {
    /// Member patterns — literal paths or `*`-globs relative to the workspace
    /// `pyproject.toml` directory. uv requires at least one entry; we mirror
    /// that: `parse_workspace_config` returns `Err(ParseError)` if `members`
    /// is missing or empty when the table is present.
    pub members: Vec<String>,
    /// Patterns to skip after globbing. Same syntax as `members`.
    pub exclude: Vec<String>,
}

/// A discovered workspace member with its `[project]` identity resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceMember {
    /// PEP 503-normalized package name from the member's `[project] name`.
    pub name: String,
    /// Version string from the member's `[project] version`. Left as-is
    /// (no PEP 440 normalization here — that's the resolver's job).
    pub version: String,
    /// Absolute path to the member's source root (directory containing its
    /// own `pyproject.toml`).
    pub root: PathBuf,
    /// Absolute path to the member's `pyproject.toml`.
    pub pyproject: PathBuf,
}

const WORKSPACE_TOML_URL: &str = "<workspace pyproject.toml>";

/// Parse `[tool.uv.workspace]` out of a `pyproject.toml` source string.
///
/// Returns `Ok(None)` when the table is absent (the project is not a
/// workspace). Returns `Err(ParseError)` for:
/// - malformed TOML
/// - `members` missing or not a list of strings
/// - `members` empty (uv considers this an authoring error)
/// - any pattern containing `**` (unsupported for now — see module comment)
pub fn parse_workspace_config(toml_src: &str) -> Result<Option<WorkspaceConfig>, IndexError> {
    let doc: toml::Value = toml_src.parse().map_err(|err| IndexError::ParseError {
        url: WORKSPACE_TOML_URL.into(),
        detail: format!("malformed TOML: {err}"),
    })?;

    let ws_table = doc
        .get("tool")
        .and_then(|t| t.get("uv"))
        .and_then(|u| u.get("workspace"));

    let Some(ws) = ws_table else {
        return Ok(None);
    };

    let members = ws
        .get("members")
        .ok_or_else(|| IndexError::ParseError {
            url: WORKSPACE_TOML_URL.into(),
            detail: "[tool.uv.workspace] missing required `members` array".into(),
        })?
        .as_array()
        .ok_or_else(|| IndexError::ParseError {
            url: WORKSPACE_TOML_URL.into(),
            detail: "[tool.uv.workspace].members must be an array of strings".into(),
        })?;

    if members.is_empty() {
        return Err(IndexError::ParseError {
            url: WORKSPACE_TOML_URL.into(),
            detail: "[tool.uv.workspace].members is empty — declare at least one member".into(),
        });
    }

    let members = collect_string_array(members, "members")?;
    let exclude = match ws.get("exclude") {
        Some(arr) => {
            let arr = arr.as_array().ok_or_else(|| IndexError::ParseError {
                url: WORKSPACE_TOML_URL.into(),
                detail: "[tool.uv.workspace].exclude must be an array of strings".into(),
            })?;
            collect_string_array(arr, "exclude")?
        }
        None => Vec::new(),
    };

    for pat in members.iter().chain(exclude.iter()) {
        if pat.contains("**") {
            return Err(IndexError::ParseError {
                url: WORKSPACE_TOML_URL.into(),
                detail: format!("recursive `**` patterns are not yet supported: {pat:?}"),
            });
        }
    }

    Ok(Some(WorkspaceConfig { members, exclude }))
}

fn collect_string_array(arr: &[toml::Value], field_name: &str) -> Result<Vec<String>, IndexError> {
    arr.iter()
        .map(|v| {
            v.as_str()
                .map(String::from)
                .ok_or_else(|| IndexError::ParseError {
                    url: WORKSPACE_TOML_URL.into(),
                    detail: format!(
                        "[tool.uv.workspace].{field_name} entries must be strings, got {v:?}"
                    ),
                })
        })
        .collect()
}

/// Discover workspace members under `workspace_root` (a directory containing
/// the workspace `pyproject.toml`).
///
/// Steps:
/// 1. Read `<workspace_root>/pyproject.toml`.
/// 2. Parse `[tool.uv.workspace]`. If absent, return an empty vec.
/// 3. Expand each `members` pattern to one-or-more directory paths.
/// 4. Drop anything matched by an `exclude` pattern.
/// 5. For each remaining dir, read its `pyproject.toml` and pull
///    `[project] name` + `version`.
pub fn discover_workspace_members(
    workspace_root: &Path,
) -> Result<Vec<WorkspaceMember>, IndexError> {
    let cfg = match read_workspace_config(workspace_root)? {
        Some(cfg) => cfg,
        None => return Ok(Vec::new()),
    };

    let mut matched: Vec<PathBuf> = Vec::new();
    for pat in &cfg.members {
        let hits = expand_pattern(workspace_root, pat)?;
        if hits.is_empty() {
            return Err(IndexError::ParseError {
                url: WORKSPACE_TOML_URL.into(),
                detail: format!("workspace member pattern matched no directories: {pat:?}"),
            });
        }
        for hit in hits {
            if !matched.contains(&hit) {
                matched.push(hit);
            }
        }
    }

    if !cfg.exclude.is_empty() {
        let mut excluded: Vec<PathBuf> = Vec::new();
        for pat in &cfg.exclude {
            for hit in expand_pattern(workspace_root, pat)? {
                if !excluded.contains(&hit) {
                    excluded.push(hit);
                }
            }
        }
        matched.retain(|m| !excluded.contains(m));
    }

    matched.sort();

    matched.into_iter().map(|root| read_member(&root)).collect()
}

/// Read and parse `[tool.uv.workspace]` from `<workspace_root>/pyproject.toml`.
pub fn read_workspace_config(workspace_root: &Path) -> Result<Option<WorkspaceConfig>, IndexError> {
    let ws_toml = workspace_root.join("pyproject.toml");
    let src = fs::read_to_string(&ws_toml).map_err(|err| IndexError::CacheIo {
        path: ws_toml.display().to_string(),
        detail: format!("reading workspace pyproject.toml: {err}"),
    })?;
    parse_workspace_config(&src)
}

/// Expand one member pattern. Supports literal segments and single `*` per
/// segment; uses depth-first directory walking, one segment at a time.
fn expand_pattern(workspace_root: &Path, pattern: &str) -> Result<Vec<PathBuf>, IndexError> {
    if pattern.is_empty() {
        return Err(IndexError::ParseError {
            url: WORKSPACE_TOML_URL.into(),
            detail: "workspace member pattern is empty".into(),
        });
    }
    if Path::new(pattern).is_absolute() {
        return Err(IndexError::ParseError {
            url: WORKSPACE_TOML_URL.into(),
            detail: format!("workspace member pattern must be relative: {pattern:?}"),
        });
    }

    let segments: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return Err(IndexError::ParseError {
            url: WORKSPACE_TOML_URL.into(),
            detail: format!("workspace member pattern has no segments: {pattern:?}"),
        });
    }

    let mut frontier: Vec<PathBuf> = vec![workspace_root.to_path_buf()];
    for seg in segments {
        let mut next: Vec<PathBuf> = Vec::new();
        for base in &frontier {
            if seg.contains('*') {
                let read_dir = match fs::read_dir(base) {
                    Ok(r) => r,
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(err) => {
                        return Err(IndexError::CacheIo {
                            path: base.display().to_string(),
                            detail: format!("listing workspace candidate directory: {err}"),
                        });
                    }
                };
                for entry in read_dir {
                    let entry = entry.map_err(|err| IndexError::CacheIo {
                        path: base.display().to_string(),
                        detail: format!("reading directory entry: {err}"),
                    })?;
                    let name = entry.file_name();
                    let name = match name.to_str() {
                        Some(n) => n,
                        None => continue,
                    };
                    if matches_segment(seg, name) {
                        let path = entry.path();
                        if path.is_dir() {
                            next.push(path);
                        }
                    }
                }
            } else {
                let candidate = base.join(seg);
                if candidate.is_dir() {
                    next.push(candidate);
                }
            }
        }
        frontier = next;
    }

    Ok(frontier)
}

/// Match one path segment against one pattern segment.
/// Supports a single `*` wildcard (matches any non-empty run of characters
/// that does not cross `/`, which is implicit since we operate one segment
/// at a time). Multiple `*`s are also fine — handled via prefix/suffix walk.
fn matches_segment(pat: &str, name: &str) -> bool {
    // Fast path: literal.
    if !pat.contains('*') {
        return pat == name;
    }
    // General glob with `*` wildcards. We use a simple two-pointer matcher.
    let pat_bytes = pat.as_bytes();
    let name_bytes = name.as_bytes();
    let mut p_idx = 0usize;
    let mut n_idx = 0usize;
    let mut last_star: Option<usize> = None;
    let mut match_idx: usize = 0;
    while n_idx < name_bytes.len() {
        if p_idx < pat_bytes.len() && pat_bytes[p_idx] == b'*' {
            last_star = Some(p_idx);
            match_idx = n_idx;
            p_idx += 1;
        } else if p_idx < pat_bytes.len() && pat_bytes[p_idx] == name_bytes[n_idx] {
            p_idx += 1;
            n_idx += 1;
        } else if let Some(star_pos) = last_star {
            p_idx = star_pos + 1;
            match_idx += 1;
            n_idx = match_idx;
        } else {
            return false;
        }
    }
    while p_idx < pat_bytes.len() && pat_bytes[p_idx] == b'*' {
        p_idx += 1;
    }
    p_idx == pat_bytes.len()
}

fn read_member(root: &Path) -> Result<WorkspaceMember, IndexError> {
    let pyproject = root.join("pyproject.toml");
    let src = fs::read_to_string(&pyproject).map_err(|err| IndexError::CacheIo {
        path: pyproject.display().to_string(),
        detail: format!("reading workspace member pyproject.toml: {err}"),
    })?;
    let doc: toml::Value = src.parse().map_err(|err| IndexError::ParseError {
        url: pyproject.display().to_string(),
        detail: format!("malformed member pyproject.toml: {err}"),
    })?;

    let project = doc.get("project").ok_or_else(|| IndexError::ParseError {
        url: pyproject.display().to_string(),
        detail: "workspace member missing [project] table".into(),
    })?;

    let name = project
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| IndexError::ParseError {
            url: pyproject.display().to_string(),
            detail: "workspace member [project] missing string `name`".into(),
        })?;

    let version = project
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| IndexError::ParseError {
            url: pyproject.display().to_string(),
            detail: "workspace member [project] missing string `version`".into(),
        })?;

    Ok(WorkspaceMember {
        name: normalize_workspace_package_name(name),
        version: version.to_string(),
        root: root.to_path_buf(),
        pyproject,
    })
}

/// PEP 503 normalize: lowercase + collapse runs of `[-_.]` to single `-`.
/// Duplicates the helper in `cache.rs` deliberately — keeping `workspace.rs`
/// dependency-free of sibling modules makes it easier to test in isolation.
pub fn normalize_workspace_package_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut last_was_sep = false;
    for ch in name.chars() {
        let normalized = ch.to_ascii_lowercase();
        let is_sep = matches!(normalized, '-' | '_' | '.');
        if is_sep {
            if !last_was_sep {
                out.push('-');
                last_was_sep = true;
            }
        } else {
            out.push(normalized);
            last_was_sep = false;
        }
    }
    // Trim any trailing `-` produced by collapsing a separator at end-of-string.
    while out.ends_with('-') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write(dir: &Path, rel: &str, contents: &str) {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn parse_returns_none_when_table_absent() {
        let src = r#"
[project]
name = "outer"
version = "0.1.0"
"#;
        let cfg = parse_workspace_config(src).unwrap();
        assert!(cfg.is_none());
    }

    #[test]
    fn parse_picks_up_members_and_exclude() {
        let src = r#"
[tool.uv.workspace]
members = ["packages/*", "extras/foo"]
exclude = ["packages/legacy"]
"#;
        let cfg = parse_workspace_config(src).unwrap().unwrap();
        assert_eq!(cfg.members, vec!["packages/*", "extras/foo"]);
        assert_eq!(cfg.exclude, vec!["packages/legacy"]);
    }

    #[test]
    fn parse_rejects_missing_members() {
        let src = r#"
[tool.uv.workspace]
exclude = ["foo"]
"#;
        let err = parse_workspace_config(src).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("missing required `members`"), "got: {msg}");
    }

    #[test]
    fn parse_rejects_empty_members() {
        let src = r#"
[tool.uv.workspace]
members = []
"#;
        let err = parse_workspace_config(src).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("empty"), "got: {msg}");
    }

    #[test]
    fn parse_rejects_double_star_pattern() {
        let src = r#"
[tool.uv.workspace]
members = ["packages/**/foo"]
"#;
        let err = parse_workspace_config(src).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("recursive `**`"), "got: {msg}");
    }

    #[test]
    fn parse_rejects_non_string_members() {
        let src = r#"
[tool.uv.workspace]
members = [42]
"#;
        let err = parse_workspace_config(src).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("entries must be strings"), "got: {msg}");
    }

    #[test]
    fn matches_segment_literal_and_star() {
        assert!(matches_segment("foo", "foo"));
        assert!(!matches_segment("foo", "bar"));
        assert!(matches_segment("*", "anything"));
        assert!(matches_segment("foo-*", "foo-bar"));
        assert!(!matches_segment("foo-*", "bar"));
        assert!(matches_segment("*-foo", "bar-foo"));
        assert!(matches_segment("*a*", "banana"));
    }

    #[test]
    fn discover_returns_empty_when_no_workspace_table() {
        let dir = TempDir::new().unwrap();
        write(
            dir.path(),
            "pyproject.toml",
            r#"
[project]
name = "loner"
version = "0.1.0"
"#,
        );
        let members = discover_workspace_members(dir.path()).unwrap();
        assert!(members.is_empty());
    }

    #[test]
    fn discover_expands_star_glob_and_reads_member_identity() {
        let dir = TempDir::new().unwrap();
        write(
            dir.path(),
            "pyproject.toml",
            r#"
[tool.uv.workspace]
members = ["packages/*"]
"#,
        );
        write(
            dir.path(),
            "packages/alpha/pyproject.toml",
            r#"
[project]
name = "Alpha_Pkg"
version = "0.1.0"
"#,
        );
        write(
            dir.path(),
            "packages/beta/pyproject.toml",
            r#"
[project]
name = "beta-pkg"
version = "0.2.0"
"#,
        );

        let mut members = discover_workspace_members(dir.path()).unwrap();
        members.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(members.len(), 2);
        assert_eq!(members[0].name, "alpha-pkg"); // normalized
        assert_eq!(members[0].version, "0.1.0");
        assert!(members[0].root.ends_with("packages/alpha"));
        assert!(
            members[0]
                .pyproject
                .ends_with("packages/alpha/pyproject.toml")
        );
        assert_eq!(members[1].name, "beta-pkg");
    }

    #[test]
    fn discover_filters_excluded_patterns() {
        let dir = TempDir::new().unwrap();
        write(
            dir.path(),
            "pyproject.toml",
            r#"
[tool.uv.workspace]
members = ["packages/*"]
exclude = ["packages/legacy"]
"#,
        );
        write(
            dir.path(),
            "packages/keep/pyproject.toml",
            r#"
[project]
name = "keep"
version = "1.0.0"
"#,
        );
        write(
            dir.path(),
            "packages/legacy/pyproject.toml",
            r#"
[project]
name = "legacy"
version = "0.0.1"
"#,
        );

        let members = discover_workspace_members(dir.path()).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].name, "keep");
    }

    #[test]
    fn discover_supports_literal_member_path() {
        let dir = TempDir::new().unwrap();
        write(
            dir.path(),
            "pyproject.toml",
            r#"
[tool.uv.workspace]
members = ["extras/special"]
"#,
        );
        write(
            dir.path(),
            "extras/special/pyproject.toml",
            r#"
[project]
name = "special"
version = "0.3.0"
"#,
        );

        let members = discover_workspace_members(dir.path()).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].name, "special");
    }

    #[test]
    fn discover_errors_when_member_pyproject_missing_project_table() {
        let dir = TempDir::new().unwrap();
        write(
            dir.path(),
            "pyproject.toml",
            r#"
[tool.uv.workspace]
members = ["packages/orphan"]
"#,
        );
        // No [project] table — only a [build-system] table.
        write(
            dir.path(),
            "packages/orphan/pyproject.toml",
            r#"
[build-system]
requires = ["setuptools"]
"#,
        );

        let err = discover_workspace_members(dir.path()).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("missing [project]"), "got: {msg}");
    }

    #[test]
    fn discover_errors_when_pattern_matches_nothing() {
        let dir = TempDir::new().unwrap();
        write(
            dir.path(),
            "pyproject.toml",
            r#"
[tool.uv.workspace]
members = ["packages/*"]
"#,
        );
        // No packages/ directory at all.

        let err = discover_workspace_members(dir.path()).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("matched no directories"), "got: {msg}");
    }

    #[test]
    fn discover_deduplicates_overlapping_patterns() {
        let dir = TempDir::new().unwrap();
        write(
            dir.path(),
            "pyproject.toml",
            r#"
[tool.uv.workspace]
members = ["packages/*", "packages/alpha"]
"#,
        );
        write(
            dir.path(),
            "packages/alpha/pyproject.toml",
            r#"
[project]
name = "alpha"
version = "0.1.0"
"#,
        );

        let members = discover_workspace_members(dir.path()).unwrap();
        // Both patterns match `packages/alpha` — must appear once.
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].name, "alpha");
    }
}
