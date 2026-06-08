//! Per-language import extraction and resolution

use regex_lite::Regex;
use std::path::{Path, PathBuf};

/// An extracted import from source code
#[derive(Debug, Clone)]
pub struct ExtractedImport {
    pub path: String,
    pub line: u32,
    pub language: &'static str,
}

/// Extract import statements from source code based on file extension
pub fn extract_imports(source: &str, file_path: &Path) -> Vec<ExtractedImport> {
    match file_path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "py" | "pyi" => extract_python_imports(source),
        "ts" | "tsx" | "js" | "jsx" => extract_js_imports(source),
        "rs" => extract_rust_imports(source),
        "go" => extract_go_imports(source),
        _ => Vec::new(),
    }
}

/// Resolve an import path to an absolute file path
pub fn resolve_import(
    import_path: &str,
    from_file: &Path,
    project_root: &Path,
    language: &str,
) -> Option<PathBuf> {
    match language {
        "python" => resolve_python_import(import_path, from_file, project_root),
        "javascript" | "typescript" => resolve_js_import(import_path, from_file, project_root),
        "rust" => resolve_rust_import(import_path, from_file, project_root),
        "go" => resolve_go_import(import_path, from_file, project_root),
        _ => None,
    }
}

// -- Python ------------------------------------------------------------------

fn extract_python_imports(source: &str) -> Vec<ExtractedImport> {
    let re_from = Regex::new(r"^\s*from\s+(\.{0,3}[a-zA-Z0-9_.]*)\s+import\b").unwrap();
    let re_import = Regex::new(r"^\s*import\s+([a-zA-Z0-9_.]+)").unwrap();
    let mut out = Vec::new();
    for (i, line) in source.lines().enumerate() {
        let cap = re_from.captures(line).or_else(|| re_import.captures(line));
        if let Some(c) = cap {
            out.push(ExtractedImport {
                path: c[1].to_string(),
                line: (i + 1) as u32,
                language: "python",
            });
        }
    }
    out
}

fn resolve_python_import(imp: &str, from: &Path, root: &Path) -> Option<PathBuf> {
    let dots = imp.chars().take_while(|c| *c == '.').count();
    if dots > 0 {
        let mut base = from.parent()?.to_path_buf();
        for _ in 1..dots {
            base = base.parent()?.to_path_buf();
        }
        return resolve_py_module(&imp[dots..], &base);
    }
    resolve_py_module(imp, root)
}

fn resolve_py_module(module: &str, base: &Path) -> Option<PathBuf> {
    if module.is_empty() {
        let init = base.join("__init__.py");
        return if init.exists() { Some(init) } else { None };
    }
    let rel: PathBuf = module.split('.').collect();
    for (ext, is_pkg) in [("py", false), ("pyi", false)] {
        let p = base.join(&rel).with_extension(ext);
        if p.exists() && !is_pkg {
            return Some(p);
        }
    }
    let pkg = base.join(&rel).join("__init__.py");
    if pkg.exists() {
        return Some(pkg);
    }
    let stub = base.join(&rel).with_extension("pyi");
    if stub.exists() {
        return Some(stub);
    }
    None
}

// -- JavaScript / TypeScript -------------------------------------------------

fn extract_js_imports(source: &str) -> Vec<ExtractedImport> {
    let re_imp = Regex::new(r#"(?:import\s+.*?\s+from\s+|import\s+)['"]([^'"]+)['"]"#).unwrap();
    let re_req = Regex::new(r#"require\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap();
    let re_dyn = Regex::new(r#"import\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap();
    let mut out = Vec::new();
    for (i, line) in source.lines().enumerate() {
        let ln = (i + 1) as u32;
        for re in [&re_imp, &re_req, &re_dyn] {
            for caps in re.captures_iter(line) {
                let p = caps[1].to_string();
                if p.starts_with('.') || p.starts_with('/') {
                    out.push(ExtractedImport {
                        path: p,
                        line: ln,
                        language: "typescript",
                    });
                }
            }
        }
    }
    out
}

fn resolve_js_import(imp: &str, from: &Path, _root: &Path) -> Option<PathBuf> {
    let base = from.parent()?.join(imp);
    if let Some(ext) = base.extension().and_then(|e| e.to_str()) {
        if ["ts", "tsx", "js", "jsx"].contains(&ext) && base.exists() {
            return Some(base);
        }
    }
    for ext in ["ts", "tsx", "js", "jsx"] {
        let c = base.with_extension(ext);
        if c.exists() {
            return Some(c);
        }
    }
    for ext in ["ts", "tsx", "js", "jsx"] {
        let c = base.join("index").with_extension(ext);
        if c.exists() {
            return Some(c);
        }
    }
    None
}

// -- Rust --------------------------------------------------------------------

fn extract_rust_imports(source: &str) -> Vec<ExtractedImport> {
    let re_mod = Regex::new(r"^\s*(?:pub\s+)?mod\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*;").unwrap();
    let re_use =
        Regex::new(r"^\s*(?:pub\s+)?use\s+(crate|super|self)(?:::([a-zA-Z0-9_:]+))?").unwrap();
    let mut out = Vec::new();
    for (i, line) in source.lines().enumerate() {
        let ln = (i + 1) as u32;
        if let Some(c) = re_mod.captures(line) {
            out.push(ExtractedImport {
                path: format!("mod:{}", &c[1]),
                line: ln,
                language: "rust",
            });
        }
        if let Some(c) = re_use.captures(line) {
            let rest = c.get(2).map(|m| m.as_str()).unwrap_or("");
            let full = if rest.is_empty() {
                c[1].to_string()
            } else {
                format!("{}::{rest}", &c[1])
            };
            out.push(ExtractedImport {
                path: full,
                line: ln,
                language: "rust",
            });
        }
    }
    out
}

fn resolve_rust_import(imp: &str, from: &Path, root: &Path) -> Option<PathBuf> {
    if let Some(name) = imp.strip_prefix("mod:") {
        let dir = from.parent()?;
        let stem = from.file_stem()?.to_str()?;
        let base = if ["mod", "lib", "main"].contains(&stem) {
            dir.to_path_buf()
        } else {
            dir.join(stem)
        };
        let f = base.join(format!("{name}.rs"));
        if f.exists() {
            return Some(f);
        }
        let m = base.join(name).join("mod.rs");
        if m.exists() {
            return Some(m);
        }
        return None;
    }
    if let Some(rest) = imp.strip_prefix("crate::") {
        let src = find_rust_crate_src(from, root)?;
        return resolve_rs_chain(&rest.split("::").collect::<Vec<_>>(), &src);
    }
    if let Some(rest) = imp.strip_prefix("super::") {
        let dir = from.parent()?;
        let stem = from.file_stem()?.to_str()?;
        let base = if ["mod", "lib", "main"].contains(&stem) {
            dir.parent()?.to_path_buf()
        } else {
            dir.to_path_buf()
        };
        return resolve_rs_chain(&rest.split("::").collect::<Vec<_>>(), &base);
    }
    None
}

fn find_rust_crate_src(file: &Path, root: &Path) -> Option<PathBuf> {
    let mut dir = file.parent()?;
    loop {
        if dir.join("Cargo.toml").exists() {
            let src = dir.join("src");
            return Some(if src.is_dir() { src } else { dir.to_path_buf() });
        }
        if dir == root || dir.parent().is_none() {
            break;
        }
        dir = dir.parent()?;
    }
    None
}

fn resolve_rs_chain(parts: &[&str], base: &Path) -> Option<PathBuf> {
    if parts.is_empty() {
        return None;
    }
    let mut dir = base.to_path_buf();
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            let f = dir.join(format!("{part}.rs"));
            if f.exists() {
                return Some(f);
            }
            let m = dir.join(part).join("mod.rs");
            if m.exists() {
                return Some(m);
            }
            return None;
        }
        let next = dir.join(part);
        if next.is_dir() {
            dir = next;
        } else {
            return None;
        }
    }
    None
}

// -- Go ----------------------------------------------------------------------

fn extract_go_imports(source: &str) -> Vec<ExtractedImport> {
    let re_single = Regex::new(r#"^\s*import\s+"([^"]+)""#).unwrap();
    let re_block = Regex::new(r"^\s*import\s*\(").unwrap();
    let re_line = Regex::new(r#"^\s*(?:[a-zA-Z_]\w*\s+)?"([^"]+)""#).unwrap();
    let mut out = Vec::new();
    let mut in_block = false;
    for (i, line) in source.lines().enumerate() {
        let ln = (i + 1) as u32;
        if in_block {
            if line.trim() == ")" {
                in_block = false;
                continue;
            }
            if let Some(c) = re_line.captures(line) {
                out.push(ExtractedImport {
                    path: c[1].to_string(),
                    line: ln,
                    language: "go",
                });
            }
            continue;
        }
        if re_block.is_match(line) {
            in_block = true;
            continue;
        }
        if let Some(c) = re_single.captures(line) {
            out.push(ExtractedImport {
                path: c[1].to_string(),
                line: ln,
                language: "go",
            });
        }
    }
    out
}

fn resolve_go_import(imp: &str, _from: &Path, root: &Path) -> Option<PathBuf> {
    let candidate = root.join(imp);
    if candidate.is_dir() {
        return Some(candidate);
    }
    let go_mod = root.join("go.mod");
    if go_mod.exists() {
        if let Ok(content) = std::fs::read_to_string(&go_mod) {
            let re = Regex::new(r"^\s*module\s+(\S+)").unwrap();
            for line in content.lines() {
                if let Some(c) = re.captures(line) {
                    if let Some(rest) = imp.strip_prefix(&c[1]) {
                        let rest = rest.strip_prefix('/').unwrap_or(rest);
                        let local = root.join(rest);
                        if local.is_dir() {
                            return Some(local);
                        }
                    }
                }
            }
        }
    }
    None
}

// -- Tests -------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_extract_python_imports() {
        let src = "import os\nfrom collections import OrderedDict\nfrom .utils import helper\nfrom ..base import Base\n";
        let imps = extract_python_imports(src);
        assert_eq!(imps.len(), 4);
        assert_eq!(imps[0].path, "os");
        assert_eq!(imps[1].path, "collections");
        assert_eq!(imps[2].path, ".utils");
        assert_eq!(imps[3].path, "..base");
    }

    #[test]
    fn test_extract_js_imports() {
        let src =
            "import React from 'react';\nimport foo from './foo';\nconst baz = require('./baz');\n";
        let imps = extract_js_imports(src);
        assert_eq!(imps.len(), 2);
        assert_eq!(imps[0].path, "./foo");
        assert_eq!(imps[1].path, "./baz");
    }

    #[test]
    fn test_extract_rust_imports() {
        let src = "mod utils;\npub mod config;\nuse crate::server::handler;\nuse super::types;\n";
        let imps = extract_rust_imports(src);
        assert_eq!(imps.len(), 4);
        assert_eq!(imps[0].path, "mod:utils");
        assert_eq!(imps[1].path, "mod:config");
        assert_eq!(imps[2].path, "crate::server::handler");
        assert_eq!(imps[3].path, "super::types");
    }

    #[test]
    fn test_extract_go_imports() {
        let src = "package main\n\nimport \"fmt\"\n\nimport (\n    \"os\"\n    myalias \"github.com/user/pkg\"\n)\n";
        let imps = extract_go_imports(src);
        assert_eq!(imps.len(), 3);
        assert_eq!(imps[0].path, "fmt");
        assert_eq!(imps[1].path, "os");
        assert_eq!(imps[2].path, "github.com/user/pkg");
    }

    #[test]
    fn test_resolve_relative_python_import() {
        let tmp = TempDir::new().unwrap();
        let pkg = tmp.path().join("pkg");
        fs::create_dir_all(&pkg).unwrap();
        fs::write(pkg.join("a.py"), "from .utils import helper").unwrap();
        fs::write(pkg.join("utils.py"), "def helper(): pass").unwrap();
        assert_eq!(
            resolve_python_import(".utils", &pkg.join("a.py"), tmp.path()),
            Some(pkg.join("utils.py")),
        );
    }

    #[test]
    fn test_resolve_relative_js_import() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("app.ts"), "").unwrap();
        fs::write(src.join("utils.ts"), "").unwrap();
        assert_eq!(
            resolve_js_import("./utils", &src.join("app.ts"), tmp.path()),
            Some(src.join("utils.ts")),
        );
    }
}
