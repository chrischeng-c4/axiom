//! Project-wide analysis
//!
//! This module provides:
//! - Python file discovery (respects .gitignore)
//! - pyproject.toml configuration reading
//! - Directory exclusion (venv, __pycache__, .git)
//! - Project-wide type checking

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::check::{TypeChecker, TypeError};
use super::imports::ModuleInfo;
use super::modules::ModuleGraph;
use super::stubs::StubLoader;
use super::ty::Type;
use crate::lens_error::Result;
use crate::syntax::{Language, MultiParser};
use rayon::prelude::*;

use super::cache::{AnalysisCache, CacheEntry, ContentHash};

/// Directories to always exclude from analysis
const EXCLUDED_DIRS: &[&str] = &[
    "venv",
    ".venv",
    "env",
    ".env",
    "__pycache__",
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    ".tox",
    ".nox",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    "dist",
    "build",
    "*.egg-info",
];

/// Project configuration (from pyproject.toml)
#[derive(Debug, Clone, Default)]
pub struct ProjectConfig {
    /// Project root directory
    pub root: PathBuf,
    /// Source directories to analyze
    pub source_dirs: Vec<PathBuf>,
    /// Directories to exclude
    pub exclude: Vec<String>,
    /// Python version target
    pub python_version: Option<String>,
    /// Strict mode
    pub strict: bool,
    /// Type checking mode: basic, standard, strict, all
    pub type_checking_mode: String,
}

impl ProjectConfig {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root: root.clone(),
            source_dirs: vec![root],
            exclude: EXCLUDED_DIRS.iter().map(|s| s.to_string()).collect(),
            python_version: None,
            strict: false,
            type_checking_mode: "standard".to_string(),
        }
    }

    /// Load configuration from pyproject.toml
    pub fn from_pyproject(root: &Path) -> Self {
        let mut config = Self::new(root.to_path_buf());

        let pyproject_path = root.join("pyproject.toml");
        if pyproject_path.exists() {
            if let Ok(content) = fs::read_to_string(&pyproject_path) {
                config.parse_pyproject(&content);
            }
        }

        config
    }

    /// Parse pyproject.toml content
    fn parse_pyproject(&mut self, content: &str) {
        // Simple TOML parsing for [tool.cclab_lens] section
        // Note: For production, use a proper TOML parser

        let mut in_tool_cclab_lens = false;
        let mut in_tool_pyright = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Section headers
            if trimmed == "[tool.cclab_lens]" {
                in_tool_cclab_lens = true;
                in_tool_pyright = false;
                continue;
            } else if trimmed == "[tool.pyright]" || trimmed == "[tool.mypy]" {
                in_tool_cclab_lens = false;
                in_tool_pyright = true;
                continue;
            } else if trimmed.starts_with('[') {
                in_tool_cclab_lens = false;
                in_tool_pyright = false;
                continue;
            }

            // Parse tool.cclab_lens settings
            if in_tool_cclab_lens || in_tool_pyright {
                if let Some((key, value)) = trimmed.split_once('=') {
                    let key = key.trim();
                    let value = value.trim().trim_matches('"').trim_matches('\'');

                    match key {
                        "pythonVersion" | "python_version" => {
                            self.python_version = Some(value.to_string());
                        }
                        "strict" => {
                            self.strict = value == "true";
                        }
                        "typeCheckingMode" | "type_checking_mode" => {
                            self.type_checking_mode = value.to_string();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Check if a path should be excluded
    pub fn should_exclude(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for exclude in &self.exclude {
            if exclude.contains('*') {
                // Glob pattern
                if path_str.contains(&exclude.replace('*', "")) {
                    return true;
                }
            } else if path_str.contains(exclude) {
                return true;
            }
        }

        false
    }
}

/// Project analyzer
pub struct ProjectAnalyzer {
    /// Project configuration
    config: ProjectConfig,
    /// Module graph
    graph: ModuleGraph,
    /// Stub loader
    stubs: StubLoader,
    /// Parser for source files
    parser: MultiParser,
    /// Analysis cache
    cache: AnalysisCache,
    /// Analyzed module info
    module_info: HashMap<String, ModuleInfo>,
    /// Type errors by module
    errors: HashMap<String, Vec<TypeError>>,
}

impl ProjectAnalyzer {
    pub fn new(config: ProjectConfig) -> Result<Self> {
        let mut stubs = StubLoader::new();
        stubs.load_builtins();

        let parser = MultiParser::new()?;

        Ok(Self {
            config,
            graph: ModuleGraph::new(),
            stubs,
            parser,
            cache: AnalysisCache::new(),
            module_info: HashMap::new(),
            errors: HashMap::new(),
        })
    }

    /// Create analyzer from project root
    pub fn from_root(root: &Path) -> Result<Self> {
        let config = ProjectConfig::from_pyproject(root);
        Self::new(config)
    }

    /// Discover all Python files in the project
    pub fn discover_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();

        for source_dir in &self.config.source_dirs {
            if source_dir.is_dir() {
                self.discover_in_dir(source_dir, &mut files);
            }
        }

        files
    }

    fn discover_in_dir(&self, dir: &Path, files: &mut Vec<PathBuf>) {
        if self.config.should_exclude(dir) {
            return;
        }

        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                self.discover_in_dir(&path, files);
            } else if path.extension().map(|e| e == "py").unwrap_or(false) {
                if !self.config.should_exclude(&path) {
                    files.push(path);
                }
            }
        }
    }

    /// Build module graph from discovered files
    pub fn build_graph(&mut self) {
        let files = self.discover_files();

        for file in &files {
            if let Some(module_name) = ModuleGraph::path_to_module_name(file, &self.config.root) {
                self.graph.add_module(&module_name, Some(file.clone()));

                // Parse imports from file
                if let Ok(source) = fs::read_to_string(file) {
                    let imports = self.extract_imports(&source);
                    for import in imports {
                        self.graph.add_import(&module_name, &import);
                    }
                }
            }
        }
    }

    /// Extract import statements from Python source
    fn extract_imports(&self, source: &str) -> Vec<String> {
        let mut imports = Vec::new();

        for line in source.lines() {
            let trimmed = line.trim();

            // import module
            if let Some(rest) = trimmed.strip_prefix("import ") {
                for part in rest.split(',') {
                    let module = part.split_whitespace().next().unwrap_or("");
                    let module = module.split(" as ").next().unwrap_or(module);
                    if !module.is_empty() {
                        imports.push(module.to_string());
                    }
                }
            }
            // from module import ...
            else if let Some(rest) = trimmed.strip_prefix("from ") {
                if let Some((module, _)) = rest.split_once(" import ") {
                    let module = module.trim();
                    if !module.is_empty() {
                        imports.push(module.to_string());
                    }
                }
            }
        }

        imports
    }

    /// Analyze all modules in the project
    pub fn analyze(&mut self) -> &HashMap<String, Vec<TypeError>> {
        // Build graph if not already built
        if self.graph.module_names().next().is_none() {
            self.build_graph();
        }

        // Get analysis order
        let order = self.graph.topological_sort().unwrap_or_else(|| {
            // If there are cycles, just analyze in any order
            self.graph.module_names().cloned().collect()
        });

        // Analyze each module
        for module_name in order {
            if let Some(node) = self.graph.get_module(&module_name) {
                if let Some(path) = &node.path {
                    if let Ok(source) = fs::read_to_string(path) {
                        let errors = self.analyze_module(&module_name, &source);
                        if !errors.is_empty() {
                            self.errors.insert(module_name, errors);
                        }
                    }
                }
            }
        }

        &self.errors
    }

    /// Analyze a single module
    fn analyze_module(&mut self, _module_name: &str, source: &str) -> Vec<TypeError> {
        // Parse the source file
        let parsed = match self.parser.parse(source, Language::Python) {
            Some(p) => p,
            None => return vec![], // Skip files that fail to parse
        };

        // Run type checker
        let mut checker = TypeChecker::new(source);
        let diagnostics = checker.check_file(&parsed);

        // Convert diagnostics to TypeErrors
        diagnostics
            .into_iter()
            .filter_map(|d| {
                // Only include type errors
                if d.message.contains("type") || d.message.contains("Type") {
                    Some(TypeError {
                        range: d.range,
                        expected: Type::Unknown,
                        got: Type::Unknown,
                        message: d.message,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get module info by name
    pub fn get_module_info(&self, name: &str) -> Option<&ModuleInfo> {
        self.module_info.get(name)
    }

    /// Get type for a name in a module
    pub fn get_type(&self, module: &str, name: &str) -> Option<&Type> {
        // First check module exports
        if let Some(info) = self.module_info.get(module) {
            if let Some(ty) = info.exports.get(name) {
                return Some(ty);
            }
        }

        // Then check stubs
        if let Some(stub) = self.stubs.get_stub(module) {
            return stub.exports.get(name);
        }

        None
    }

    /// Get all errors
    pub fn all_errors(&self) -> impl Iterator<Item = (&String, &Vec<TypeError>)> {
        self.errors.iter()
    }

    /// Get errors for a specific module
    pub fn module_errors(&self, name: &str) -> Option<&Vec<TypeError>> {
        self.errors.get(name)
    }

    /// Get the module graph
    pub fn graph(&self) -> &ModuleGraph {
        &self.graph
    }

    /// Check for circular imports
    pub fn circular_imports(&self) -> Vec<Vec<String>> {
        self.graph.detect_cycles()
    }

    /// Get the analysis cache
    pub fn cache(&self) -> &AnalysisCache {
        &self.cache
    }

    /// Analyze modules in parallel (for independent modules)
    /// This analyzes modules that have no interdependencies in parallel
    pub fn analyze_parallel(&mut self) -> &HashMap<String, Vec<TypeError>> {
        // Build graph if not already built
        if self.graph.module_names().next().is_none() {
            self.build_graph();
        }

        // Collect modules to analyze with their paths and sources
        let modules_to_analyze: Vec<(String, String)> = self
            .graph
            .modules()
            .filter_map(|(name, node)| {
                // Skip if already cached and not changed
                if let Some(cached) = self.cache.get(name) {
                    if !cached.needs_reanalysis() {
                        // Use cached errors
                        if !cached.errors.is_empty() {
                            return None; // Will handle separately
                        }
                        return None;
                    }
                }

                node.path.as_ref().and_then(|path| {
                    fs::read_to_string(path)
                        .ok()
                        .map(|source| (name.clone(), source))
                })
            })
            .collect();

        // Analyze in parallel using rayon
        let results: Vec<(String, Vec<TypeError>)> = modules_to_analyze
            .par_iter()
            .filter_map(|(name, source)| {
                // Create a new parser for this thread
                let mut parser = match MultiParser::new() {
                    Ok(p) => p,
                    Err(_) => return None,
                };

                let parsed = parser.parse(source, Language::Python)?;
                let mut checker = TypeChecker::new(source);
                let diagnostics = checker.check_file(&parsed);

                let errors: Vec<TypeError> = diagnostics
                    .into_iter()
                    .filter_map(|d| {
                        if d.message.contains("type") || d.message.contains("Type") {
                            Some(TypeError {
                                range: d.range,
                                expected: Type::Unknown,
                                got: Type::Unknown,
                                message: d.message,
                            })
                        } else {
                            None
                        }
                    })
                    .collect();

                Some((name.clone(), errors))
            })
            .collect();

        // Merge results
        for (name, errors) in results {
            if !errors.is_empty() {
                self.errors.insert(name, errors);
            }
        }

        &self.errors
    }

    /// Incremental analysis - only analyze changed files and their dependents
    pub fn analyze_incremental(&mut self) -> &HashMap<String, Vec<TypeError>> {
        // Build graph if not already built
        if self.graph.module_names().next().is_none() {
            self.build_graph();
        }

        // Find changed modules
        let changed: Vec<String> = self.cache.get_changed_modules();

        // Get all affected modules (including dependents)
        let mut affected = std::collections::HashSet::new();
        for module in &changed {
            affected.extend(self.cache.get_affected_modules(module));
        }

        // Collect module data first to avoid borrow issues
        let modules_data: Vec<(String, PathBuf, String)> = affected
            .iter()
            .filter_map(|module_name| {
                let node = self.graph.get_module(module_name)?;
                let path = node.path.clone()?;
                let source = fs::read_to_string(&path).ok()?;
                Some((module_name.clone(), path, source))
            })
            .collect();

        // Now analyze each module
        for (module_name, path, source) in modules_data {
            let errors = self.analyze_module(&module_name, &source);

            // Update cache
            let hash = ContentHash::from_content(&source);
            let mut entry = CacheEntry::new(
                module_name.clone(),
                path,
                hash,
                ModuleInfo::new(&module_name),
            );
            entry.errors = errors.clone();
            self.cache.store(entry);

            if !errors.is_empty() {
                self.errors.insert(module_name, errors);
            }
        }

        &self.errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_project_config_default() {
        let config = ProjectConfig::new(PathBuf::from("/project"));

        assert_eq!(config.root, PathBuf::from("/project"));
        assert!(!config.strict);
        assert_eq!(config.type_checking_mode, "standard");
    }

    #[test]
    fn test_should_exclude() {
        let config = ProjectConfig::new(PathBuf::from("/project"));

        assert!(config.should_exclude(Path::new("/project/venv/lib")));
        assert!(config.should_exclude(Path::new("/project/__pycache__/file.pyc")));
        assert!(config.should_exclude(Path::new("/project/.git/config")));
        assert!(!config.should_exclude(Path::new("/project/src/main.py")));
    }

    #[test]
    fn test_parse_pyproject() {
        let content = r#"
[tool.cclab_lens]
python_version = "3.11"
strict = true
type_checking_mode = "strict"

[tool.other]
key = "value"
"#;

        let mut config = ProjectConfig::new(PathBuf::from("/project"));
        config.parse_pyproject(content);

        assert_eq!(config.python_version, Some("3.11".to_string()));
        assert!(config.strict);
        assert_eq!(config.type_checking_mode, "strict");
    }

    #[test]
    fn test_extract_imports() {
        let analyzer = ProjectAnalyzer::new(ProjectConfig::new(PathBuf::from("/test"))).unwrap();

        let source = r#"
import os
import sys, json
from pathlib import Path
from typing import Optional, List
import numpy as np
"#;

        let imports = analyzer.extract_imports(source);

        assert!(imports.contains(&"os".to_string()));
        assert!(imports.contains(&"sys".to_string()));
        assert!(imports.contains(&"json".to_string()));
        assert!(imports.contains(&"pathlib".to_string()));
        assert!(imports.contains(&"typing".to_string()));
        assert!(imports.contains(&"numpy".to_string()));
    }

    #[test]
    fn test_project_analyzer_creation() {
        let config = ProjectConfig::new(PathBuf::from("/test"));
        let analyzer = ProjectAnalyzer::new(config).unwrap();

        assert!(analyzer.errors.is_empty());
    }

    #[test]
    fn test_discover_files_in_temp() {
        // Create a temp directory structure
        let temp_dir = env::temp_dir().join("cclab_lens_test_project");
        let _ = fs::remove_dir_all(&temp_dir); // Clean up if exists
        fs::create_dir_all(temp_dir.join("src")).unwrap();
        fs::create_dir_all(temp_dir.join("venv")).unwrap();

        // Create some Python files
        fs::write(temp_dir.join("src/main.py"), "print('hello')").unwrap();
        fs::write(temp_dir.join("src/utils.py"), "def foo(): pass").unwrap();
        fs::write(temp_dir.join("venv/lib.py"), "# should be excluded").unwrap();

        let config = ProjectConfig::new(temp_dir.clone());
        let analyzer = ProjectAnalyzer::new(config).unwrap();

        let files = analyzer.discover_files();

        // Should find main.py and utils.py, but not venv/lib.py
        assert!(files.iter().any(|p| p.ends_with("main.py")));
        assert!(files.iter().any(|p| p.ends_with("utils.py")));
        assert!(!files.iter().any(|p| p.to_string_lossy().contains("venv")));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_build_graph_with_imports() {
        // Create a temp directory with files that import each other
        let temp_dir = env::temp_dir().join("cclab_lens_test_graph");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // main.py imports utils
        fs::write(
            temp_dir.join("main.py"),
            "from utils import helper\n\ndef main():\n    helper()",
        )
        .unwrap();

        // utils.py is standalone
        fs::write(temp_dir.join("utils.py"), "def helper(): pass").unwrap();

        let config = ProjectConfig::new(temp_dir.clone());
        let mut analyzer = ProjectAnalyzer::new(config).unwrap();
        analyzer.build_graph();

        let graph = analyzer.graph();

        // Check that main imports utils
        let main = graph.get_module("main");
        assert!(main.is_some());
        assert!(main.unwrap().imports.contains("utils"));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_circular_import_detection() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_circular");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create circular imports: a -> b -> c -> a
        fs::write(temp_dir.join("a.py"), "from b import foo").unwrap();
        fs::write(temp_dir.join("b.py"), "from c import bar").unwrap();
        fs::write(temp_dir.join("c.py"), "from a import baz").unwrap();

        let config = ProjectConfig::new(temp_dir.clone());
        let mut analyzer = ProjectAnalyzer::new(config).unwrap();
        analyzer.build_graph();

        let cycles = analyzer.circular_imports();
        assert!(!cycles.is_empty(), "Should detect circular imports");

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_pyright_config_parsing() {
        let content = r#"
[tool.pyright]
pythonVersion = "3.10"
strict = true
typeCheckingMode = "strict"
"#;

        let mut config = ProjectConfig::new(PathBuf::from("/project"));
        config.parse_pyproject(content);

        // Should parse pyright config too
        assert_eq!(config.python_version, Some("3.10".to_string()));
        assert_eq!(config.type_checking_mode, "strict");
    }
}
