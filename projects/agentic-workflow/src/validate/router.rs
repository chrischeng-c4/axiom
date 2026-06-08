// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-router-rs.md#source
// CODEGEN-BEGIN
//! Path-shape classifier + spec file walker.
//!
//! `aw td validate <target>` accepts three shapes for `<target>`:
//!
//! - **Slug** — an issue identifier (no `/`, no `.md` suffix). Activates the
//!   CRRR commit-gate path. Owned by `projects/agentic-workflow/cli/src/td.rs`; this
//!   module just reports the shape.
//! - **Prefix** — a directory under `.aw/tech-design/`. Walk every `.md`
//!   file underneath and validate in read-only mode.
//! - **File** — a single spec file. Validate just that file in read-only mode.

use std::path::{Path, PathBuf};

/// What kind of argument the caller passed to aw td validate.
/// Three tuple variants; no unit variants, no serde.
/// @spec projects/agentic-workflow/tech-design/core/validate/router.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathShape {
    /// Issue identifier — no slashes, no .md.
    Slug(String),
    /// Directory under .aw/tech-design/.
    Prefix(PathBuf),
    /// Single .md spec file.
    File(PathBuf),
}
/// Classify `target` by text shape. Does NOT check the filesystem — callers
/// that need existence-checking should call `resolve_spec_files` afterward.
/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-router-rs.md#source
pub fn classify(target: &str, project_root: &Path) -> PathShape {
    // File: ends in `.md`
    if target.ends_with(".md") {
        return PathShape::File(join_under_root(target, project_root));
    }
    // Prefix: contains a path separator
    if target.contains('/') || target.contains(std::path::MAIN_SEPARATOR) {
        return PathShape::Prefix(join_under_root(target, project_root));
    }
    // Otherwise: slug
    PathShape::Slug(target.to_string())
}

fn join_under_root(target: &str, project_root: &Path) -> PathBuf {
    let p = Path::new(target);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        project_root.join(p)
    }
}

/// Walk the filesystem for a `PathShape::Prefix` or `File`, return every
/// `.md` spec file the walker finds. For `Slug` returns empty.
/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-router-rs.md#source
pub fn resolve_spec_files(shape: &PathShape) -> std::io::Result<Vec<PathBuf>> {
    match shape {
        PathShape::Slug(_) => Ok(Vec::new()),
        PathShape::File(p) => {
            if p.is_file() {
                Ok(vec![p.clone()])
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("spec file not found: {}", p.display()),
                ))
            }
        }
        PathShape::Prefix(dir) => {
            if !dir.is_dir() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("spec directory not found: {}", dir.display()),
                ));
            }
            let mut out = Vec::new();
            walk_markdown(dir, &mut out)?;
            out.sort();
            Ok(out)
        }
    }
}

/// Depth-first collect every `.md` file under `dir` (recursive). Skips
/// symlinks + hidden entries.
fn walk_markdown(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        let ft = entry.file_type()?;
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            walk_markdown(&path, out)?;
        } else if ft.is_file()
            && path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e == "md")
        {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn slug_no_slash_no_dot_md() {
        let root = PathBuf::from("/repo");
        assert_eq!(
            classify("my-issue-slug", &root),
            PathShape::Slug("my-issue-slug".to_string())
        );
    }

    #[test]
    fn prefix_with_slash() {
        let root = PathBuf::from("/repo");
        match classify("projects/agentic-workflow/tech-design/core", &root) {
            PathShape::Prefix(p) => {
                assert_eq!(
                    p,
                    PathBuf::from("/repo/projects/agentic-workflow/tech-design/core")
                );
            }
            other => panic!("expected Prefix, got {:?}", other),
        }
    }

    #[test]
    fn file_ends_in_md() {
        let root = PathBuf::from("/repo");
        match classify("foo/bar.md", &root) {
            PathShape::File(p) => assert_eq!(p, PathBuf::from("/repo/foo/bar.md")),
            other => panic!("expected File, got {:?}", other),
        }
    }

    #[test]
    fn absolute_path_preserved() {
        let root = PathBuf::from("/repo");
        match classify("/abs/spec.md", &root) {
            PathShape::File(p) => assert_eq!(p, PathBuf::from("/abs/spec.md")),
            other => panic!("expected File, got {:?}", other),
        }
    }

    #[test]
    fn prefix_single_name_with_trailing_slash() {
        let root = PathBuf::from("/repo");
        match classify("crates/", &root) {
            PathShape::Prefix(p) => assert_eq!(p, PathBuf::from("/repo/crates/")),
            other => panic!("expected Prefix, got {:?}", other),
        }
    }

    #[test]
    fn walk_finds_md_files_recursively() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join("a/b")).unwrap();
        std::fs::write(root.join("top.md"), "x").unwrap();
        std::fs::write(root.join("a/mid.md"), "x").unwrap();
        std::fs::write(root.join("a/b/deep.md"), "x").unwrap();
        std::fs::write(root.join("a/skip.txt"), "x").unwrap();

        let shape = PathShape::Prefix(root.to_path_buf());
        let mut got = resolve_spec_files(&shape).unwrap();
        got.sort();
        let expected = vec![
            root.join("a/b/deep.md"),
            root.join("a/mid.md"),
            root.join("top.md"),
        ];
        assert_eq!(got, expected);
    }

    #[test]
    fn walk_skips_dotfiles() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".hidden")).unwrap();
        std::fs::write(root.join(".hidden/secret.md"), "x").unwrap();
        std::fs::write(root.join(".dotfile.md"), "x").unwrap();
        std::fs::write(root.join("visible.md"), "x").unwrap();
        let got = resolve_spec_files(&PathShape::Prefix(root.to_path_buf())).unwrap();
        assert_eq!(got, vec![root.join("visible.md")]);
    }

    #[test]
    fn resolve_missing_file_errors() {
        let shape = PathShape::File(PathBuf::from("/nonexistent/a.md"));
        assert!(resolve_spec_files(&shape).is_err());
    }

    #[test]
    fn resolve_missing_prefix_errors() {
        let shape = PathShape::Prefix(PathBuf::from("/nonexistent/"));
        assert!(resolve_spec_files(&shape).is_err());
    }

    #[test]
    fn resolve_slug_returns_empty() {
        let shape = PathShape::Slug("foo".to_string());
        assert!(resolve_spec_files(&shape).unwrap().is_empty());
    }
}

// CODEGEN-END
