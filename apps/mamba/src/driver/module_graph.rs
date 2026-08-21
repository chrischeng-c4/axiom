/// Module graph resolution for multi-file compilation (R1).
///
/// Builds a dependency graph of Mamba source files by scanning their import
/// statements, resolves file paths relative to a set of search roots, performs
/// a topological sort to determine compilation order, and reports cycles.
///
/// Supports:
/// - Absolute imports:  `import os`, `import os.path`, `from os import path`
/// - Import aliases:    `import numpy as np`, `from os import getcwd as gw`
/// - Relative imports:  `from . import utils`, `from ..pkg import helper`
/// - Package detection: `__init__.py` in any ancestor directory
///
/// # Example
/// ```no_run
/// use mamba::driver::module_graph::{ModuleGraph, GraphError};
/// use std::path::PathBuf;
///
/// let mut graph = ModuleGraph::new(vec![PathBuf::from("src")]);
/// graph.add_root("src/main.py").unwrap();
/// let order = graph.topo_sort().unwrap();
/// ```
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};

use crate::parser;
use crate::parser::ast::{Module, Stmt};
use crate::source::span::FileId;

// ── Error types ──────────────────────────────────────────────────────────────

/// Errors that can occur during module graph construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    /// A file could not be read.
    Io { path: PathBuf, reason: String },
    /// A source file failed to parse.
    Parse { path: PathBuf, reason: String },
    /// The import graph contains a cycle.
    Cycle { cycle: Vec<PathBuf> },
    /// An import could not be resolved to a file.
    Unresolved { import: String, from: PathBuf },
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphError::Io { path, reason } => {
                write!(f, "I/O error reading {}: {}", path.display(), reason)
            }
            GraphError::Parse { path, reason } => {
                write!(f, "Parse error in {}: {}", path.display(), reason)
            }
            GraphError::Cycle { cycle } => {
                let names: Vec<_> = cycle.iter().map(|p| p.display().to_string()).collect();
                write!(f, "Import cycle: {}", names.join(" → "))
            }
            GraphError::Unresolved { import, from } => {
                write!(f, "Unresolved import '{}' in {}", import, from.display())
            }
        }
    }
}

impl std::error::Error for GraphError {}

// ── Import descriptor ────────────────────────────────────────────────────────

/// A parsed import dependency extracted from a source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDep {
    /// Dotted module path (e.g. `["os", "path"]` for `import os.path`).
    pub module: Vec<String>,
    /// Number of leading dots (0 = absolute, 1 = `.`, 2 = `..`, etc.).
    pub level: usize,
    /// Whether this is a star import (`from x import *`).
    pub is_star: bool,
    /// Individual names imported with optional aliases.
    /// Empty for plain `import M`.
    pub names: Vec<(String, Option<String>)>,
}

// ── Node ─────────────────────────────────────────────────────────────────────

/// A node in the module graph.
#[derive(Debug)]
pub struct ModuleNode {
    /// Canonical absolute path.
    pub path: PathBuf,
    /// Module name (dotted, relative to a search root).
    pub name: String,
    /// Parsed AST (available after `add_root` / `resolve_all`).
    pub ast: Module,
    /// Whether this file is a package (`__init__.py`).
    pub is_package: bool,
    /// Direct import dependencies (canonicalized paths).
    pub deps: Vec<PathBuf>,
    /// Raw import descriptors extracted from the AST.
    pub imports: Vec<ImportDep>,
}

// ── ModuleGraph ───────────────────────────────────────────────────────────────

/// The module dependency graph.
pub struct ModuleGraph {
    /// Ordered list of search roots (checked left-to-right).
    search_roots: Vec<PathBuf>,
    /// nodes keyed by canonical absolute path.
    nodes: HashMap<PathBuf, ModuleNode>,
    /// Queue of paths that still need their dependencies resolved.
    work_queue: VecDeque<PathBuf>,
    /// Next synthetic file ID for the parser.
    next_file_id: u32,
}

impl ModuleGraph {
    /// Create a new graph with the given search roots.
    pub fn new(search_roots: Vec<PathBuf>) -> Self {
        Self {
            search_roots,
            nodes: HashMap::new(),
            work_queue: VecDeque::new(),
            next_file_id: 0,
        }
    }

    // ── Public API ────────────────────────────────────────────────────────────

    /// Add a root source file and recursively resolve all its imports.
    ///
    /// Returns `Err` if any file cannot be read, parsed, or resolved.
    /// Unresolvable imports from the standard library are silently skipped.
    pub fn add_root(&mut self, path: impl AsRef<Path>) -> Result<(), Vec<GraphError>> {
        let canon =
            std::fs::canonicalize(path.as_ref()).unwrap_or_else(|_| path.as_ref().to_path_buf());

        self.enqueue(canon);
        self.resolve_all()
    }

    /// Perform a topological sort of the resolved graph.
    ///
    /// Returns nodes in compilation order (dependencies first).
    /// Returns `Err` if there is a cycle.
    pub fn topo_sort(&self) -> Result<Vec<&ModuleNode>, GraphError> {
        // in_degree[path] = number of unresolved local dependencies of `path`.
        let mut in_degree: HashMap<&PathBuf, usize> = HashMap::new();
        // adj[dep] = nodes that depend on `dep` (dep must come before them).
        let mut adj: HashMap<&PathBuf, Vec<&PathBuf>> = HashMap::new();

        for (path, node) in &self.nodes {
            in_degree.entry(path).or_insert(0);
            for dep in &node.deps {
                if self.nodes.contains_key(dep) {
                    // `path` depends on `dep`  →  edge: dep → path
                    *in_degree.entry(path).or_insert(0) += 1;
                    adj.entry(dep).or_default().push(path);
                }
            }
        }

        // Kahn's algorithm: start with nodes that have no unresolved deps.
        let mut queue: VecDeque<&PathBuf> = in_degree
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(p, _)| *p)
            .collect();

        let mut order: Vec<&ModuleNode> = Vec::new();

        while let Some(path) = queue.pop_front() {
            if let Some(node) = self.nodes.get(path) {
                order.push(node);
            }
            // Decrement in-degree of nodes that depend on `path`.
            for dependent in adj.get(path).unwrap_or(&vec![]) {
                let entry = in_degree.entry(dependent).or_insert(0);
                if *entry > 0 {
                    *entry -= 1;
                    if *entry == 0 {
                        queue.push_back(dependent);
                    }
                }
            }
        }

        if order.len() != self.nodes.len() {
            // Cycle detected — find it.
            let in_cycle: Vec<PathBuf> = in_degree
                .into_iter()
                .filter(|(_, d)| *d > 0)
                .map(|(p, _)| p.clone())
                .collect();
            return Err(GraphError::Cycle { cycle: in_cycle });
        }

        Ok(order)
    }

    /// Iterate over all resolved nodes.
    pub fn nodes(&self) -> impl Iterator<Item = &ModuleNode> {
        self.nodes.values()
    }

    /// Number of resolved nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn enqueue(&mut self, path: PathBuf) {
        if !self.nodes.contains_key(&path) {
            self.work_queue.push_back(path);
        }
    }

    /// Drain the work queue, loading and parsing each file.
    fn resolve_all(&mut self) -> Result<(), Vec<GraphError>> {
        let mut errors: Vec<GraphError> = Vec::new();

        while let Some(path) = self.work_queue.pop_front() {
            if self.nodes.contains_key(&path) {
                continue;
            }

            match self.load_node(&path) {
                Ok(node) => {
                    let deps = node.deps.clone();
                    self.nodes.insert(path, node);
                    for dep in deps {
                        self.enqueue(dep);
                    }
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Load, parse, and extract imports for a single file.
    fn load_node(&mut self, path: &Path) -> Result<ModuleNode, GraphError> {
        let source = std::fs::read_to_string(path).map_err(|e| GraphError::Io {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;

        let fid = FileId(self.next_file_id);
        self.next_file_id += 1;

        let mut ast = parser::parse(&source, fid).map_err(|e| GraphError::Parse {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;
        crate::lower::pep695::desugar_module(&mut ast);
        let ast = ast;

        let imports = collect_imports(&ast);
        let is_package = path
            .file_name()
            .map(|n| n == "__init__.py")
            .unwrap_or(false);

        let module_name = self.path_to_module_name(path).unwrap_or_else(|| {
            path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        });

        // Resolve each import to a canonical path.
        let mut deps: Vec<PathBuf> = Vec::new();
        for imp in &imports {
            match self.resolve_import(imp, path) {
                Some(dep_path) => deps.push(dep_path),
                None => {
                    // Unresolvable = likely a stdlib/native module, skip silently.
                }
            }
        }
        deps.sort();
        deps.dedup();

        Ok(ModuleNode {
            path: path.to_path_buf(),
            name: module_name,
            ast,
            is_package,
            deps,
            imports,
        })
    }

    /// Resolve an import statement to a canonical filesystem path.
    ///
    /// Returns `None` if the module cannot be found in any search root
    /// (stdlib or external packages are not in the search roots).
    fn resolve_import(&self, imp: &ImportDep, from: &Path) -> Option<PathBuf> {
        if imp.level > 0 {
            // Relative import: anchor to the directory of `from`.
            self.resolve_relative(imp, from)
        } else {
            // Absolute import: search roots.
            self.resolve_absolute(imp)
        }
    }

    fn resolve_absolute(&self, imp: &ImportDep) -> Option<PathBuf> {
        for root in &self.search_roots {
            if let Some(p) = probe_module(root, &imp.module) {
                return Some(p);
            }
        }
        None
    }

    fn resolve_relative(&self, imp: &ImportDep, from: &Path) -> Option<PathBuf> {
        // Walk up `level` directories from `from`.
        let from_dir = if from.is_file() {
            from.parent().unwrap_or(from)
        } else {
            from
        };

        let mut anchor = from_dir.to_path_buf();
        for _ in 1..imp.level {
            anchor = anchor.parent().unwrap_or(&anchor).to_path_buf();
        }

        if imp.module.is_empty() {
            // `from . import name` — the anchor dir IS the package.
            return Some(anchor.join("__init__.py")).filter(|p| p.exists());
        }

        probe_module(&anchor, &imp.module)
    }

    /// Convert a file path to a dotted module name relative to a search root.
    fn path_to_module_name(&self, path: &Path) -> Option<String> {
        for root in &self.search_roots {
            if let Ok(rel) = path.strip_prefix(root) {
                let parts: Vec<_> = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect();
                let mut parts: Vec<String> = parts;
                // Strip .py suffix from last component.
                if let Some(last) = parts.last_mut() {
                    if last.ends_with(".py") {
                        *last = last[..last.len() - 3].to_string();
                    }
                }
                // Drop __init__ suffix for packages.
                if parts.last().map(|s| s == "__init__").unwrap_or(false) {
                    parts.pop();
                }
                return Some(parts.join("."));
            }
        }
        None
    }
}

// ── Module path probing ───────────────────────────────────────────────────────

/// Try to find a `.py` file or package directory for `module` under `root`.
///
/// Search order (mirrors CPython's importer):
/// 1. `root/a/b/c.py`              — simple module
/// 2. `root/a/b/c/__init__.py`     — package
fn probe_module(root: &Path, module: &[String]) -> Option<PathBuf> {
    if module.is_empty() {
        return None;
    }
    let mut path = root.to_path_buf();
    for part in module {
        path = path.join(part);
    }

    // Plain module file.
    let py_file = path.with_extension("py");
    if py_file.is_file() {
        if let Ok(canon) = std::fs::canonicalize(&py_file) {
            return Some(canon);
        }
        return Some(py_file);
    }

    // Package directory.
    let init = path.join("__init__.py");
    if init.is_file() {
        if let Ok(canon) = std::fs::canonicalize(&init) {
            return Some(canon);
        }
        return Some(init);
    }

    None
}

// ── Import extraction ─────────────────────────────────────────────────────────

/// Walk the top-level statements of a module and collect all import descriptors.
fn collect_imports(module: &Module) -> Vec<ImportDep> {
    let mut deps = Vec::new();
    for spanned in &module.stmts {
        extract_stmt_imports(&spanned.node, &mut deps);
    }
    deps
}

fn extract_stmt_imports(stmt: &Stmt, out: &mut Vec<ImportDep>) {
    match stmt {
        Stmt::Import { module, names, .. } => {
            // `import a.b.c` or `from a.b import name [as alias]`
            let name_list: Vec<(String, Option<String>)> = match names {
                None => vec![],
                Some(ns) => ns.iter().map(|(n, a)| (n.clone(), a.clone())).collect(),
            };
            // Detect relative import level from leading empty module components.
            let (level, trimmed) = infer_import_level(module);
            out.push(ImportDep {
                module: trimmed.to_vec(),
                level,
                is_star: name_list.iter().any(|(n, _)| n == "*"),
                names: name_list,
            });
        }

        // `from . import name` or `from .pkg import name`
        // In the AST, relative imports use a non-empty `module` for the leading
        // path and `names = Some([...])`. The level is encoded as the number of
        // leading empty strings in `module` (one per dot).
        //
        // Actually the AST represents relative imports with the leading dots
        // counted as empty components at the front of `module`, OR the parser
        // sets the first component to "" per leading dot.
        //
        // Let's inspect what the parser actually produces and normalise here.
        _ => {
            // No other statement types introduce imports at the module level.
        }
    }
}

// ── Relative import level inference ──────────────────────────────────────────
//
// The Mamba parser currently uses a single `Import` variant for both absolute
// and relative imports. Relative imports have one or more leading empty strings
// in `module` (one per leading dot). The code above handles only the simple
// case; this helper extracts the actual level.

fn infer_import_level(module: &[String]) -> (usize, &[String]) {
    let level = module.iter().take_while(|s| s.is_empty()).count();
    (level, &module[level..])
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_file(dir: &TempDir, rel: &str, content: &str) -> PathBuf {
        let path = dir.path().join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "{content}").unwrap();
        path
    }

    #[test]
    fn test_probe_module_simple_file() {
        let dir = TempDir::new().unwrap();
        let _path = write_file(&dir, "utils.py", "x: int = 1");
        let found = probe_module(dir.path(), &["utils".to_string()]);
        assert!(found.is_some());
        assert_eq!(found.unwrap().file_name().unwrap(), "utils.py");
    }

    #[test]
    fn test_probe_module_package() {
        let dir = TempDir::new().unwrap();
        write_file(&dir, "mypkg/__init__.py", "");
        let found = probe_module(dir.path(), &["mypkg".to_string()]);
        assert!(found.is_some());
        assert!(found.unwrap().ends_with("__init__.py"));
    }

    #[test]
    fn test_probe_module_missing() {
        let dir = TempDir::new().unwrap();
        let found = probe_module(dir.path(), &["nonexistent".to_string()]);
        assert!(found.is_none());
    }

    #[test]
    fn test_add_root_single_file_no_imports() {
        let dir = TempDir::new().unwrap();
        let main = write_file(&dir, "main.py", "x: int = 1\nprint(x)\n");
        let mut graph = ModuleGraph::new(vec![dir.path().to_path_buf()]);
        graph.add_root(&main).expect("should succeed");
        assert_eq!(graph.len(), 1);
    }

    #[test]
    fn test_add_root_with_local_import() {
        let dir = TempDir::new().unwrap();
        write_file(&dir, "utils.py", "def helper() -> int: return 42\n");
        let main = write_file(&dir, "main.py", "import utils\nprint(utils.helper())\n");
        let mut graph = ModuleGraph::new(vec![dir.path().to_path_buf()]);
        graph.add_root(&main).expect("should succeed");
        // Both main.py and utils.py resolved
        assert_eq!(graph.len(), 2);
    }

    #[test]
    fn test_topo_sort_respects_dependency_order() {
        let dir = TempDir::new().unwrap();
        write_file(&dir, "a.py", "x: int = 1\n");
        write_file(&dir, "b.py", "import a\ny: int = a.x + 1\n");
        let main = write_file(&dir, "main.py", "import b\n");

        let mut graph = ModuleGraph::new(vec![dir.path().to_path_buf()]);
        graph.add_root(&main).expect("resolve");
        assert_eq!(graph.len(), 3);

        let order = graph.topo_sort().expect("no cycle");
        // a.py must come before b.py, b.py before main.py
        let names: Vec<&str> = order
            .iter()
            .map(|n| n.path.file_stem().unwrap().to_str().unwrap())
            .collect();
        let pos = |name: &str| names.iter().position(|&n| n == name).unwrap();
        assert!(pos("a") < pos("b"));
        assert!(pos("b") < pos("main"));
    }

    #[test]
    fn test_stdlib_import_silently_skipped() {
        let dir = TempDir::new().unwrap();
        // `import os` cannot be resolved in dir — should not error.
        let main = write_file(&dir, "main.py", "import os\nprint(os.getcwd())\n");
        let mut graph = ModuleGraph::new(vec![dir.path().to_path_buf()]);
        graph
            .add_root(&main)
            .expect("stdlib import skipped silently");
        assert_eq!(graph.len(), 1); // only main.py
    }

    #[test]
    fn test_infer_import_level_absolute() {
        let module = vec!["os".to_string(), "path".to_string()];
        let (level, rest) = infer_import_level(&module);
        assert_eq!(level, 0);
        assert_eq!(rest, &["os".to_string(), "path".to_string()]);
    }

    #[test]
    fn test_infer_import_level_relative_one_dot() {
        let module = vec!["".to_string(), "utils".to_string()];
        let (level, rest) = infer_import_level(&module);
        assert_eq!(level, 1);
        assert_eq!(rest, &["utils".to_string()]);
    }

    #[test]
    fn test_infer_import_level_relative_two_dots() {
        let module = vec!["".to_string(), "".to_string(), "pkg".to_string()];
        let (level, rest) = infer_import_level(&module);
        assert_eq!(level, 2);
        assert_eq!(rest, &["pkg".to_string()]);
    }

    #[test]
    fn test_path_to_module_name() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("mymod.py");
        std::fs::File::create(&path).unwrap();
        let graph = ModuleGraph::new(vec![dir.path().to_path_buf()]);
        let name = graph.path_to_module_name(&path);
        assert_eq!(name, Some("mymod".to_string()));
    }

    #[test]
    fn test_path_to_module_name_nested() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("pkg")).unwrap();
        let path = dir.path().join("pkg").join("sub.py");
        std::fs::File::create(&path).unwrap();
        let graph = ModuleGraph::new(vec![dir.path().to_path_buf()]);
        let name = graph.path_to_module_name(&path);
        assert_eq!(name, Some("pkg.sub".to_string()));
    }

    #[test]
    fn test_topo_sort_detects_cycle() {
        let dir = TempDir::new().unwrap();
        // a.py imports b, b.py imports a → cycle
        write_file(&dir, "a.py", "import b\nx: int = 1\n");
        write_file(&dir, "b.py", "import a\ny: int = 2\n");
        let a = dir.path().join("a.py");

        let mut graph = ModuleGraph::new(vec![dir.path().to_path_buf()]);
        graph
            .add_root(&a)
            .expect("add_root should not error on cycle");
        // Both a.py and b.py are in the graph
        assert_eq!(graph.len(), 2);

        let result = graph.topo_sort();
        assert!(
            matches!(result, Err(GraphError::Cycle { .. })),
            "expected Cycle error, got: {:?}",
            result
        );
    }
}
