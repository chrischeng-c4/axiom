// `.python-version` reader/writer — pyenv-compatible (Tick 48).
//
// Mirrors uv's interpreter-selection helper: `uv venv` and `uv run` look for
// a `.python-version` file walking up from the workspace root, picking the
// first matching interpreter on disk.
//
// File format (pyenv convention):
//   * One version request per line.
//   * `# comments` and ` # trailing comments` ignored.
//   * Blank lines skipped.
//   * Both `3.12` and `3.12.4` (or any PEP 440-ish ident) are valid.
//   * `cpython-3.12` / `pypy3.10` style prefixed forms are preserved verbatim.
//
// The file is conceptually unordered, but order is preserved for round-trip
// fidelity. The first non-blank line is the *primary* request — what `uv run`
// hands to the interpreter discovery layer.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Filename pyenv / uv both look for in the workspace.
pub const FILENAME: &str = ".python-version";

/// Parsed `.python-version` contents.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PythonVersionFile {
    /// Version requests in source order. Each entry is a single non-blank,
    /// non-comment line (whitespace-trimmed).
    pub versions: Vec<String>,
}

impl PythonVersionFile {
    /// Primary (first) version request, if any.
    pub fn primary(&self) -> Option<&str> {
        self.versions.first().map(|s| s.as_str())
    }

    pub fn is_empty(&self) -> bool {
        self.versions.is_empty()
    }
}

/// Parse a `.python-version` body. Pure-text; never touches the filesystem.
pub fn parse_python_version_file(src: &str) -> PythonVersionFile {
    let mut out = PythonVersionFile::default();
    for raw in src.lines() {
        let stripped = strip_comment(raw);
        let line = stripped.trim();
        if line.is_empty() {
            continue;
        }
        out.versions.push(line.to_string());
    }
    out
}

/// Render a `PythonVersionFile` back to text. Output is deterministic:
/// one entry per line, single trailing newline. Empty bodies render as `""`.
pub fn render_python_version_file(file: &PythonVersionFile) -> String {
    if file.versions.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for v in &file.versions {
        out.push_str(v);
        out.push('\n');
    }
    out
}

/// Read `.python-version` from `path`. Returns an empty file (`versions:
/// vec![]`) if the file does not exist; surfaces other I/O errors as
/// `IndexError::CacheIo`.
pub fn read_python_version_file(path: &Path) -> Result<PythonVersionFile, IndexError> {
    match fs::read_to_string(path) {
        Ok(body) => Ok(parse_python_version_file(&body)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(PythonVersionFile::default()),
        Err(e) => Err(IndexError::CacheIo {
            path: path.display().to_string(),
            detail: e.to_string(),
        }),
    }
}

/// Write a `.python-version` to `path`, creating or overwriting it. Parent
/// directories are NOT created.
pub fn write_python_version_file(path: &Path, file: &PythonVersionFile) -> Result<(), IndexError> {
    let body = render_python_version_file(file);
    fs::write(path, body).map_err(|e| IndexError::CacheIo {
        path: path.display().to_string(),
        detail: e.to_string(),
    })
}

/// Walk from `start_dir` upward looking for `.python-version`. Returns the
/// containing directory and the parsed file, or `None` if no ancestor has
/// one. Stops at the filesystem root.
pub fn find_python_version_root(
    start_dir: &Path,
) -> Result<Option<(PathBuf, PythonVersionFile)>, IndexError> {
    let mut cur: Option<&Path> = Some(start_dir);
    while let Some(dir) = cur {
        let candidate = dir.join(FILENAME);
        if candidate.is_file() {
            let parsed = read_python_version_file(&candidate)?;
            return Ok(Some((dir.to_path_buf(), parsed)));
        }
        cur = dir.parent();
    }
    Ok(None)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn strip_comment(line: &str) -> String {
    match line.find('#') {
        Some(i) => line[..i].to_string(),
        None => line.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_single_line() {
        let f = parse_python_version_file("3.12\n");
        assert_eq!(f.versions, vec!["3.12".to_string()]);
        assert_eq!(f.primary(), Some("3.12"));
    }

    #[test]
    fn parses_multiple_lines_preserves_order() {
        let f = parse_python_version_file("3.12.4\n3.11.9\n3.10\n");
        assert_eq!(f.versions, vec!["3.12.4", "3.11.9", "3.10"]);
        assert_eq!(f.primary(), Some("3.12.4"));
    }

    #[test]
    fn skips_blank_lines() {
        let f = parse_python_version_file("\n\n3.12\n\n3.11\n\n");
        assert_eq!(f.versions, vec!["3.12", "3.11"]);
    }

    #[test]
    fn strips_full_line_comments() {
        let f = parse_python_version_file("# header\n3.12\n# trailing\n");
        assert_eq!(f.versions, vec!["3.12"]);
    }

    #[test]
    fn strips_trailing_comments() {
        let f = parse_python_version_file("3.12  # primary\n3.11 # backup\n");
        assert_eq!(f.versions, vec!["3.12", "3.11"]);
    }

    #[test]
    fn preserves_prefixed_forms() {
        let f = parse_python_version_file("cpython-3.12\npypy3.10\n");
        assert_eq!(f.versions, vec!["cpython-3.12", "pypy3.10"]);
    }

    #[test]
    fn empty_file_yields_no_versions() {
        let f = parse_python_version_file("");
        assert!(f.is_empty());
        assert!(f.primary().is_none());
    }

    #[test]
    fn whitespace_only_yields_no_versions() {
        let f = parse_python_version_file("   \n\t\n");
        assert!(f.is_empty());
    }

    #[test]
    fn trims_horizontal_whitespace() {
        let f = parse_python_version_file("   3.12   \n");
        assert_eq!(f.versions, vec!["3.12"]);
    }

    #[test]
    fn handles_crlf() {
        let f = parse_python_version_file("3.12\r\n3.11\r\n");
        assert_eq!(f.versions, vec!["3.12", "3.11"]);
    }

    #[test]
    fn render_round_trips() {
        let original = "3.12\n3.11\n";
        let parsed = parse_python_version_file(original);
        let rendered = render_python_version_file(&parsed);
        assert_eq!(rendered, original);
    }

    #[test]
    fn render_normalises_comment_only_body_to_empty() {
        let parsed = parse_python_version_file("# nothing\n# here\n");
        let rendered = render_python_version_file(&parsed);
        assert_eq!(rendered, "");
    }

    #[test]
    fn render_empty_file_is_empty_string() {
        assert_eq!(
            render_python_version_file(&PythonVersionFile::default()),
            ""
        );
    }

    #[test]
    fn read_missing_file_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join(FILENAME);
        let f = read_python_version_file(&p).unwrap();
        assert!(f.is_empty());
    }

    #[test]
    fn read_existing_file_returns_parsed() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join(FILENAME);
        fs::write(&p, "3.12.4\n3.11.9\n").unwrap();
        let f = read_python_version_file(&p).unwrap();
        assert_eq!(f.versions, vec!["3.12.4", "3.11.9"]);
    }

    #[test]
    fn write_then_read_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join(FILENAME);
        let original = PythonVersionFile {
            versions: vec!["3.12".into(), "3.11".into()],
        };
        write_python_version_file(&p, &original).unwrap();
        let read = read_python_version_file(&p).unwrap();
        assert_eq!(read, original);
    }

    #[test]
    fn find_returns_none_when_no_ancestor_has_file() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a").join("b");
        fs::create_dir_all(&nested).unwrap();
        let result = find_python_version_root(&nested).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn find_locates_file_in_current_dir() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(FILENAME), "3.12\n").unwrap();
        let (root, f) = find_python_version_root(dir.path()).unwrap().unwrap();
        assert_eq!(root, dir.path());
        assert_eq!(f.primary(), Some("3.12"));
    }

    #[test]
    fn find_walks_parents() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a").join("b").join("c");
        fs::create_dir_all(&nested).unwrap();
        fs::write(dir.path().join(FILENAME), "3.11\n").unwrap();
        let (root, f) = find_python_version_root(&nested).unwrap().unwrap();
        assert_eq!(root, dir.path());
        assert_eq!(f.primary(), Some("3.11"));
    }

    #[test]
    fn find_picks_nearest_ancestor() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a").join("b");
        fs::create_dir_all(&nested).unwrap();
        // Both root and `a` have a file; nested-most wins.
        fs::write(dir.path().join(FILENAME), "3.10\n").unwrap();
        fs::write(dir.path().join("a").join(FILENAME), "3.12\n").unwrap();
        let (root, f) = find_python_version_root(&nested).unwrap().unwrap();
        assert_eq!(root, dir.path().join("a"));
        assert_eq!(f.primary(), Some("3.12"));
    }

    #[test]
    fn primary_returns_first_entry_only() {
        let f = PythonVersionFile {
            versions: vec!["3.12".into(), "3.11".into()],
        };
        assert_eq!(f.primary(), Some("3.12"));
    }
}
