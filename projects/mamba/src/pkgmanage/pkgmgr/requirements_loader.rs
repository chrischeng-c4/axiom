// Recursive `requirements.txt` loader (Tick 51).
//
// Tick 47 (`requirements_parse`) handles one body. This module is the
// file-tree driver that `uv pip install -r foo.txt` calls: read a root
// `requirements.txt` from disk, follow every `-r other.txt` and
// `-c constraints.txt` directive (relative to the including file's
// parent dir), detect cycles, and return a flat `LoadedRequirements`
// with primary requirements / constraints / editables / unknown lines.
//
// What this module does NOT do (deferred):
//   * Resolve package versions — that's the resolver.
//   * Apply markers / environment evaluation — kept as opaque strings.
//   * Fetch URLs — `-r` payloads are only resolved against the local
//     filesystem here.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::pkgmanage::pkgmgr::requirements_parse::{
    parse_requirements_txt, EditableSpec, IndexFlag, PackageRequirement, RequirementLine,
};
use crate::pkgmanage::pkgmgr::types::IndexError;

/// Flat output of the recursive load: every requirement aggregated, in
/// source order (root file first, then each include in the order it was
/// referenced).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoadedRequirements {
    /// Normal package requirements (non-constraint, non-editable).
    pub primary: Vec<PackageRequirement>,
    /// Constraints — pins coming from any `-c constraints.txt` chain.
    /// They never imply installation; the resolver uses them only to
    /// narrow candidates already selected by `primary`.
    pub constraints: Vec<PackageRequirement>,
    /// Editable installs (`-e ...`) in source order.
    pub editables: Vec<EditableSpec>,
    /// Typed pip index/binary flags (`--index-url`, `--no-binary`, etc.)
    /// captured in the order they were encountered. The resolver applies
    /// them as a stream.
    pub index_flags: Vec<IndexFlag>,
    /// Lines that this layer did not understand (e.g. uv-only flags not
    /// yet wired up), preserved verbatim so the caller may forward them
    /// to the resolver or warn.
    pub unknown: Vec<String>,
    /// Files visited, in load order. Useful for `# via X` annotations and
    /// for filesystem-watch invalidation.
    pub visited: Vec<PathBuf>,
}

/// Read `path` from disk and recursively follow `-r` / `-c` directives.
pub fn load_requirements_file(path: &Path) -> Result<LoadedRequirements, IndexError> {
    let canonical = canonicalize_path(path)?;
    let mut state = LoadState::default();
    load_one(&canonical, &mut state, IncludeMode::Primary)?;
    Ok(state.into_output())
}

/// In-memory entry point: parse `src` as if it lived at `base_dir`,
/// resolving any `-r` / `-c` includes relative to `base_dir`. The
/// pseudo-path is reported in `visited` so callers can distinguish
/// in-memory roots.
pub fn load_requirements_text(
    src: &str,
    base_dir: &Path,
) -> Result<LoadedRequirements, IndexError> {
    let mut state = LoadState::default();
    let pseudo = base_dir.join("<inline>");
    state.visited.insert(pseudo.clone());
    state.order.push(pseudo);
    process_body(src, base_dir, &mut state, IncludeMode::Primary)?;
    Ok(state.into_output())
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum IncludeMode {
    /// `-r foo.txt` — every package becomes a `primary` requirement.
    Primary,
    /// `-c foo.txt` — every package becomes a `constraint`. Editables and
    /// nested `-r` inside a constraint file are an error in pip; we mirror
    /// that by rejecting them.
    Constraint,
}

#[derive(Default)]
struct LoadState {
    primary: Vec<PackageRequirement>,
    constraints: Vec<PackageRequirement>,
    editables: Vec<EditableSpec>,
    index_flags: Vec<IndexFlag>,
    unknown: Vec<String>,
    visited: BTreeSet<PathBuf>,
    order: Vec<PathBuf>,
}

impl LoadState {
    fn into_output(self) -> LoadedRequirements {
        LoadedRequirements {
            primary: self.primary,
            constraints: self.constraints,
            editables: self.editables,
            index_flags: self.index_flags,
            unknown: self.unknown,
            visited: self.order,
        }
    }
}

fn load_one(
    path: &Path,
    state: &mut LoadState,
    mode: IncludeMode,
) -> Result<(), IndexError> {
    if !state.visited.insert(path.to_path_buf()) {
        // Cycle: already visited. Pip silently skips re-includes.
        return Ok(());
    }
    state.order.push(path.to_path_buf());

    let body = fs::read_to_string(path).map_err(|e| IndexError::CacheIo {
        path: path.display().to_string(),
        detail: e.to_string(),
    })?;

    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
    process_body(&body, base_dir, state, mode)
}

fn process_body(
    body: &str,
    base_dir: &Path,
    state: &mut LoadState,
    mode: IncludeMode,
) -> Result<(), IndexError> {
    let lines = parse_requirements_txt(body)?;
    for line in lines {
        match line {
            RequirementLine::Package(p) => match mode {
                IncludeMode::Primary => state.primary.push(p),
                IncludeMode::Constraint => state.constraints.push(p),
            },
            RequirementLine::Editable(e) => match mode {
                IncludeMode::Primary => state.editables.push(e),
                IncludeMode::Constraint => {
                    return parse_err(
                        "editable requirements are not allowed inside a constraints file",
                    );
                }
            },
            RequirementLine::Include(rel) => match mode {
                IncludeMode::Primary => {
                    let target = resolve_include(base_dir, &rel)?;
                    load_one(&target, state, IncludeMode::Primary)?;
                }
                IncludeMode::Constraint => {
                    return parse_err(
                        "-r is not allowed inside a constraints file; use -c instead",
                    );
                }
            },
            RequirementLine::Constraint(rel) => {
                let target = resolve_include(base_dir, &rel)?;
                load_one(&target, state, IncludeMode::Constraint)?;
            }
            RequirementLine::IndexFlag(f) => state.index_flags.push(f),
            RequirementLine::Unknown(raw) => state.unknown.push(raw),
        }
    }
    Ok(())
}

fn resolve_include(base_dir: &Path, rel: &str) -> Result<PathBuf, IndexError> {
    let joined = base_dir.join(rel);
    canonicalize_path(&joined)
}

fn canonicalize_path(path: &Path) -> Result<PathBuf, IndexError> {
    path.canonicalize().map_err(|e| IndexError::CacheIo {
        path: path.display().to_string(),
        detail: e.to_string(),
    })
}

fn parse_err<T>(detail: &str) -> Result<T, IndexError> {
    Err(IndexError::ParseError {
        url: "<requirements-loader>".into(),
        detail: detail.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    #[test]
    fn loads_single_file_no_includes() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("requirements.txt");
        write(&p, "click==8.1.7\nrequests==2.31.0\n");
        let out = load_requirements_file(&p).unwrap();
        assert_eq!(out.primary.len(), 2);
        assert_eq!(out.primary[0].name, "click");
        assert_eq!(out.primary[1].name, "requests");
        assert!(out.constraints.is_empty());
    }

    #[test]
    fn follows_relative_include() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("requirements.txt");
        let base = dir.path().join("base.txt");
        write(&root, "-r base.txt\nrich==13\n");
        write(&base, "click==8.1.7\n");
        let out = load_requirements_file(&root).unwrap();
        let names: Vec<&str> = out.primary.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["click", "rich"]);
    }

    #[test]
    fn follows_nested_include() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root.txt");
        let a = dir.path().join("a.txt");
        let b = dir.path().join("nested").join("b.txt");
        write(&root, "-r a.txt\nrich==13\n");
        write(&a, "-r nested/b.txt\nclick==8.1.7\n");
        write(&b, "tomli==2.0.1\n");
        let out = load_requirements_file(&root).unwrap();
        let names: Vec<&str> = out.primary.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["tomli", "click", "rich"]);
        assert_eq!(out.visited.len(), 3);
    }

    #[test]
    fn detects_cycle_silently_via_revisit_skip() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root.txt");
        let a = dir.path().join("a.txt");
        // root -> a -> root (cycle)
        write(&root, "-r a.txt\nrich==13\n");
        write(&a, "-r root.txt\nclick==8.1.7\n");
        let out = load_requirements_file(&root).unwrap();
        // root visited first; a includes root which is already visited
        // so the second visit is skipped.
        let names: Vec<&str> = out.primary.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["click", "rich"]);
    }

    #[test]
    fn constraint_directive_populates_constraints() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root.txt");
        let c = dir.path().join("constraints.txt");
        write(&root, "-c constraints.txt\nrequests\n");
        write(&c, "urllib3==2.0.7\n");
        let out = load_requirements_file(&root).unwrap();
        assert_eq!(out.primary.len(), 1);
        assert_eq!(out.primary[0].name, "requests");
        assert_eq!(out.constraints.len(), 1);
        assert_eq!(out.constraints[0].name, "urllib3");
    }

    #[test]
    fn rejects_editable_inside_constraints_file() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root.txt");
        let c = dir.path().join("constraints.txt");
        write(&root, "-c constraints.txt\n");
        write(&c, "-e .\n");
        let err = load_requirements_file(&root).unwrap_err();
        match err {
            IndexError::ParseError { detail, .. } => {
                assert!(detail.contains("editable"));
            }
            _ => panic!("wrong error"),
        }
    }

    #[test]
    fn rejects_dash_r_inside_constraints_file() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root.txt");
        let c = dir.path().join("constraints.txt");
        let inner = dir.path().join("inner.txt");
        write(&root, "-c constraints.txt\n");
        write(&c, "-r inner.txt\n");
        write(&inner, "click\n");
        let err = load_requirements_file(&root).unwrap_err();
        match err {
            IndexError::ParseError { detail, .. } => {
                assert!(detail.contains("-r is not allowed"));
            }
            _ => panic!("wrong error"),
        }
    }

    #[test]
    fn editables_collected_in_source_order() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("requirements.txt");
        write(&root, "-e .\n-e ./pkg-b\nclick==8.1.7\n");
        let out = load_requirements_file(&root).unwrap();
        assert_eq!(out.editables.len(), 2);
        assert_eq!(out.editables[0].target, ".");
        assert_eq!(out.editables[1].target, "./pkg-b");
        assert_eq!(out.primary.len(), 1);
    }

    #[test]
    fn unknown_lines_are_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("requirements.txt");
        // `--config-settings` is not a known directive at this layer; it
        // must round-trip through `unknown` so callers can warn/forward.
        write(&root, "--config-settings foo=bar\nclick\n");
        let out = load_requirements_file(&root).unwrap();
        assert_eq!(out.unknown.len(), 1);
        assert!(out.unknown[0].contains("--config-settings"));
        assert_eq!(out.primary.len(), 1);
    }

    #[test]
    fn index_flags_are_captured() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("requirements.txt");
        write(
            &root,
            "--index-url https://example.com/simple/\n--no-binary pillow\nclick\n",
        );
        let out = load_requirements_file(&root).unwrap();
        assert_eq!(out.index_flags.len(), 2);
        assert!(matches!(out.index_flags[0], IndexFlag::IndexUrl(_)));
        assert!(matches!(out.index_flags[1], IndexFlag::NoBinary(_)));
        assert!(out.unknown.is_empty());
        assert_eq!(out.primary.len(), 1);
    }

    #[test]
    fn missing_include_target_surfaces_cacheio() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("requirements.txt");
        write(&root, "-r missing.txt\n");
        let err = load_requirements_file(&root).unwrap_err();
        assert!(matches!(err, IndexError::CacheIo { .. }));
    }

    #[test]
    fn missing_root_file_surfaces_cacheio() {
        let p = PathBuf::from("/definitely/does/not/exist/req.txt");
        let err = load_requirements_file(&p).unwrap_err();
        assert!(matches!(err, IndexError::CacheIo { .. }));
    }

    #[test]
    fn inline_text_loader_resolves_relative_includes() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("base.txt");
        write(&base, "click==8.1.7\n");
        let out = load_requirements_text("-r base.txt\nrich==13\n", dir.path()).unwrap();
        let names: Vec<&str> = out.primary.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["click", "rich"]);
    }

    #[test]
    fn visited_order_root_then_each_include() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root.txt");
        let a = dir.path().join("a.txt");
        let b = dir.path().join("b.txt");
        write(&root, "-r a.txt\n-r b.txt\n");
        write(&a, "");
        write(&b, "");
        let out = load_requirements_file(&root).unwrap();
        // visited should list root first, then a, then b.
        assert_eq!(out.visited.len(), 3);
        assert!(out.visited[0].ends_with("root.txt"));
        assert!(out.visited[1].ends_with("a.txt"));
        assert!(out.visited[2].ends_with("b.txt"));
    }

    #[test]
    fn duplicate_includes_visited_once() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root.txt");
        let a = dir.path().join("a.txt");
        write(&root, "-r a.txt\n-r a.txt\n");
        write(&a, "click\n");
        let out = load_requirements_file(&root).unwrap();
        assert_eq!(out.primary.len(), 1);
        // visited = root + a (a is included twice but deduped).
        assert_eq!(out.visited.len(), 2);
    }
}
