//! Auto-discovery of project scopes from marker files (#1127)
//!
//! Scans a monorepo root for:
//! - `Cargo.toml` with `[workspace]` → Rust scope
//! - `pyproject.toml` → Python scope (detects .venv for site-packages)
//! - `tsconfig.json` + `package.json` → TypeScript scope

use crate::core::index_config::{ScopeConfig, ScopeLang};
use std::path::{Path, PathBuf};

/// Discover project scopes by scanning for marker files.
///
/// Returns scopes sorted by root path depth (deepest first) for
/// longest-prefix-match routing.
pub fn discover_scopes(project_root: &Path) -> Vec<ScopeConfig> {
    let mut scopes = Vec::new();

    // Walk up to 5 levels deep for marker files
    discover_recursive(project_root, project_root, 0, 5, &mut scopes);

    // Sort by root path length descending (longest prefix first for routing)
    scopes.sort_by(|a, b| b.root.as_os_str().len().cmp(&a.root.as_os_str().len()));

    scopes
}

fn discover_recursive(
    project_root: &Path,
    dir: &Path,
    depth: usize,
    max_depth: usize,
    scopes: &mut Vec<ScopeConfig>,
) {
    if depth > max_depth {
        return;
    }

    // Skip common non-project directories
    if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
        if matches!(
            name,
            "node_modules"
                | "target"
                | ".git"
                | ".venv"
                | "venv"
                | "__pycache__"
                | ".index"
                | "dist"
                | "build"
        ) {
            return;
        }
    }

    let rel = dir.strip_prefix(project_root).unwrap_or(dir);

    // Check for Cargo.toml with [workspace]
    let cargo_toml = dir.join("Cargo.toml");
    if cargo_toml.exists() {
        if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
            if content.contains("[workspace]") {
                let id = if rel.as_os_str().is_empty() {
                    "rust-workspace".to_string()
                } else {
                    format!("rust-{}", slugify(rel))
                };
                scopes.push(ScopeConfig {
                    id,
                    lang: ScopeLang::Rust,
                    root: rel.to_path_buf(),
                    interpreter: None,
                    search_paths: vec![],
                    marker: Some(cargo_toml),
                });
            }
        }
    }

    // Check for pyproject.toml
    let pyproject = dir.join("pyproject.toml");
    if pyproject.exists() {
        let id = if rel.as_os_str().is_empty() {
            "py-root".to_string()
        } else {
            format!("py-{}", slugify(rel))
        };

        let interpreter = detect_python_interpreter(dir);
        let mut search_paths = vec![dir.to_path_buf()];
        if let Some(ref interp) = interpreter {
            if let Some(site_packages) = detect_site_packages(interp) {
                search_paths.push(site_packages);
            }
        }

        scopes.push(ScopeConfig {
            id,
            lang: ScopeLang::Python,
            root: rel.to_path_buf(),
            interpreter,
            search_paths,
            marker: Some(pyproject),
        });
    }

    // Check for tsconfig.json (with package.json nearby)
    let tsconfig = dir.join("tsconfig.json");
    let package_json = dir.join("package.json");
    if tsconfig.exists() && package_json.exists() {
        let id = if rel.as_os_str().is_empty() {
            "ts-root".to_string()
        } else {
            format!("ts-{}", slugify(rel))
        };

        let mut search_paths = vec![dir.to_path_buf()];
        let node_modules = dir.join("node_modules");
        if node_modules.exists() {
            search_paths.push(node_modules);
        }

        scopes.push(ScopeConfig {
            id,
            lang: ScopeLang::Typescript,
            root: rel.to_path_buf(),
            interpreter: None,
            search_paths,
            marker: Some(tsconfig),
        });
    }

    // Recurse into subdirectories
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                discover_recursive(project_root, &entry.path(), depth + 1, max_depth, scopes);
            }
        }
    }
}

/// Detect Python interpreter from .venv in or near the given directory.
fn detect_python_interpreter(dir: &Path) -> Option<PathBuf> {
    // Check .venv/bin/python in the directory itself
    let venv = dir.join(".venv/bin/python");
    if venv.exists() {
        return Some(venv);
    }
    // Check venv/bin/python
    let venv2 = dir.join("venv/bin/python");
    if venv2.exists() {
        return Some(venv2);
    }
    None
}

/// Derive site-packages path from a Python interpreter.
fn detect_site_packages(interpreter: &Path) -> Option<PathBuf> {
    // .venv/bin/python → .venv/lib/python3.X/site-packages
    let venv_root = interpreter.parent()?.parent()?; // bin/ → .venv/
    let lib = venv_root.join("lib");
    if !lib.exists() {
        return None;
    }
    // Find python3.X directory
    let entries = std::fs::read_dir(&lib).ok()?;
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with("python3") {
            let site_packages = entry.path().join("site-packages");
            if site_packages.exists() {
                return Some(site_packages);
            }
        }
    }
    None
}

/// Convert a relative path to a URL-safe slug.
fn slugify(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "-")
        .replace('\\', "-")
        .to_lowercase()
}

/// Find the scope that best matches a file path (longest prefix match).
pub fn resolve_scope<'a>(file_path: &Path, scopes: &'a [ScopeConfig]) -> Option<&'a ScopeConfig> {
    // Scopes are pre-sorted by root length descending.
    // Empty root acts as catch-all fallback — skip during first pass.
    let mut fallback = None;
    for s in scopes {
        if s.root.as_os_str().is_empty() {
            fallback = Some(s);
            continue;
        }
        if file_path.starts_with(&s.root) {
            return Some(s);
        }
    }
    fallback
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_discover_rust_workspace() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]\n",
        )
        .unwrap();

        let scopes = discover_scopes(tmp.path());
        assert_eq!(scopes.len(), 1);
        assert_eq!(scopes[0].id, "rust-workspace");
        assert_eq!(scopes[0].lang, ScopeLang::Rust);
    }

    #[test]
    fn test_discover_python_project() {
        let tmp = TempDir::new().unwrap();
        let proj = tmp.path().join("projects/app");
        std::fs::create_dir_all(&proj).unwrap();
        std::fs::write(proj.join("pyproject.toml"), "[project]\nname = \"app\"\n").unwrap();

        let scopes = discover_scopes(tmp.path());
        assert_eq!(scopes.len(), 1);
        assert_eq!(scopes[0].id, "py-projects-app");
        assert_eq!(scopes[0].lang, ScopeLang::Python);
    }

    #[test]
    fn test_discover_typescript_project() {
        let tmp = TempDir::new().unwrap();
        let fe = tmp.path().join("fe");
        std::fs::create_dir_all(&fe).unwrap();
        std::fs::write(fe.join("tsconfig.json"), "{}").unwrap();
        std::fs::write(fe.join("package.json"), "{}").unwrap();

        let scopes = discover_scopes(tmp.path());
        assert_eq!(scopes.len(), 1);
        assert_eq!(scopes[0].id, "ts-fe");
        assert_eq!(scopes[0].lang, ScopeLang::Typescript);
    }

    #[test]
    fn test_discover_monorepo_multiple() {
        let tmp = TempDir::new().unwrap();
        // Rust workspace at root
        std::fs::write(tmp.path().join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
        // Python project
        let py = tmp.path().join("projects/mamba/mambalibs/httpkit");
        std::fs::create_dir_all(&py).unwrap();
        std::fs::write(py.join("pyproject.toml"), "[project]\n").unwrap();
        // TypeScript project
        let ts = tmp.path().join("projects/fe");
        std::fs::create_dir_all(&ts).unwrap();
        std::fs::write(ts.join("tsconfig.json"), "{}").unwrap();
        std::fs::write(ts.join("package.json"), "{}").unwrap();

        let scopes = discover_scopes(tmp.path());
        assert_eq!(scopes.len(), 3);
        // Sorted by root length descending
        assert!(scopes[0].root.as_os_str().len() >= scopes[1].root.as_os_str().len());
    }

    #[test]
    fn test_resolve_scope_longest_prefix() {
        let scopes = vec![
            ScopeConfig {
                id: "rust-workspace".into(),
                lang: ScopeLang::Rust,
                root: PathBuf::from(""),
                interpreter: None,
                search_paths: vec![],
                marker: None,
            },
            ScopeConfig {
                id: "py-api".into(),
                lang: ScopeLang::Python,
                root: PathBuf::from("projects/mamba/mambalibs/httpkit"),
                interpreter: None,
                search_paths: vec![],
                marker: None,
            },
        ];

        // File in Python project → py-api scope
        let py_file = Path::new("projects/mamba/mambalibs/httpkit/main.py");
        let scope = resolve_scope(py_file, &scopes).unwrap();
        assert_eq!(scope.id, "py-api");

        // File in Rust crate → rust-workspace (root scope)
        let rs_file = Path::new("crates/foo/src/lib.rs");
        let scope = resolve_scope(rs_file, &scopes).unwrap();
        assert_eq!(scope.id, "rust-workspace");
    }

    #[test]
    fn test_detect_site_packages() {
        let tmp = TempDir::new().unwrap();
        let site = tmp.path().join(".venv/lib/python3.11/site-packages");
        std::fs::create_dir_all(&site).unwrap();
        let interp = tmp.path().join(".venv/bin/python");
        std::fs::create_dir_all(interp.parent().unwrap()).unwrap();
        std::fs::write(&interp, "").unwrap();

        let result = detect_site_packages(&interp);
        assert!(result.is_some());
        assert!(result.unwrap().to_string_lossy().contains("site-packages"));
    }

    #[test]
    fn test_slugify() {
        assert_eq!(
            slugify(Path::new("projects/conductor/fe")),
            "projects-conductor-fe"
        );
        assert_eq!(slugify(Path::new(".")), ".");
        assert_eq!(slugify(Path::new("")), "");
    }
}
