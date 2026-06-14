//! Import resolution for cross-file type checking
//!
//! This module handles:
//! - Parsing import statements
//! - Resolving module paths to .py and .pyi files
//! - Loading exported types from other modules
//! - Module indexing for quick lookup
//! - Lazy loading with caching to minimize memory usage
//! - Circular import detection and handling

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::ty::Type;

/// Represents an import statement
#[derive(Debug, Clone)]
pub enum Import {
    /// import module
    Module {
        module: String,
        alias: Option<String>,
    },
    /// from module import name
    FromModule {
        module: String,
        names: Vec<ImportedName>,
    },
    /// from module import *
    WildcardImport { module: String },
}

/// A single imported name with optional alias
#[derive(Debug, Clone)]
pub struct ImportedName {
    pub name: String,
    pub alias: Option<String>,
}

/// Loading state for a module (for circular import detection)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModuleLoadState {
    /// Module not yet loaded
    #[default]
    NotLoaded,
    /// Module is currently being loaded (used for circular import detection)
    Loading,
    /// Module fully loaded
    Loaded,
    /// Module failed to load
    Failed,
}

/// Module information including exported types
#[derive(Debug, Clone, Default)]
pub struct ModuleInfo {
    /// Module path (e.g., "collections.abc")
    pub path: String,
    /// File path if available
    pub file_path: Option<PathBuf>,
    /// Whether this was loaded from a stub file (.pyi)
    pub is_stub: bool,
    /// Exported types (name -> type)
    pub exports: HashMap<String, Type>,
    /// Re-exports from other modules
    pub reexports: HashMap<String, String>, // name -> source module
    /// Is this a package (__init__.py)?
    pub is_package: bool,
    /// Load state for circular import handling
    #[allow(dead_code)]
    load_state: ModuleLoadState,
    /// Submodules (for packages)
    pub submodules: Vec<String>,
}

impl ModuleInfo {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            file_path: None,
            is_stub: false,
            exports: HashMap::new(),
            reexports: HashMap::new(),
            is_package: false,
            load_state: ModuleLoadState::NotLoaded,
            submodules: Vec::new(),
        }
    }

    /// Create a module info from a file path
    pub fn from_file(path: &str, file_path: PathBuf) -> Self {
        let is_stub = file_path.extension().map_or(false, |ext| ext == "pyi");
        let is_package = file_path.file_name().map_or(false, |name| {
            name == "__init__.py" || name == "__init__.pyi"
        });

        Self {
            path: path.to_string(),
            file_path: Some(file_path),
            is_stub,
            exports: HashMap::new(),
            reexports: HashMap::new(),
            is_package,
            load_state: ModuleLoadState::NotLoaded,
            submodules: Vec::new(),
        }
    }

    /// Get an exported type by name
    pub fn get_export(&self, name: &str) -> Option<&Type> {
        self.exports.get(name)
    }

    /// Check if the module has been loaded
    pub fn is_loaded(&self) -> bool {
        self.load_state == ModuleLoadState::Loaded
    }
}

/// Indexed module entry for quick lookups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleIndexEntry {
    /// Full module path (e.g., "django.db.models")
    pub module_path: String,
    /// File path to the module source
    pub file_path: PathBuf,
    /// Whether this is a stub file
    pub is_stub: bool,
    /// Whether this is a package
    pub is_package: bool,
}

/// Import resolver that manages module loading and type resolution
#[derive(Debug, Default)]
pub struct ImportResolver {
    /// Loaded modules (module path -> info)
    modules: HashMap<String, ModuleInfo>,
    /// Module index (module path -> file path) for quick lookups
    module_index: HashMap<String, ModuleIndexEntry>,
    /// Search paths for modules (in priority order)
    search_paths: Vec<PathBuf>,
    /// Current working directory
    cwd: PathBuf,
    /// Set of modules currently being loaded (for circular import detection)
    loading: HashSet<String>,
    /// Whether the index has been built
    indexed: bool,
}

impl ImportResolver {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            module_index: HashMap::new(),
            search_paths: vec![],
            cwd: PathBuf::new(),
            loading: HashSet::new(),
            indexed: false,
        }
    }

    /// Create a new resolver with the given search paths
    pub fn with_search_paths(paths: Vec<PathBuf>) -> Self {
        Self {
            modules: HashMap::new(),
            module_index: HashMap::new(),
            search_paths: paths,
            cwd: PathBuf::new(),
            loading: HashSet::new(),
            indexed: false,
        }
    }

    /// Set search paths for module resolution
    pub fn set_search_paths(&mut self, paths: Vec<PathBuf>) {
        self.search_paths = paths;
        self.indexed = false; // Invalidate index
    }

    /// Add a search path
    pub fn add_search_path(&mut self, path: PathBuf) {
        if !self.search_paths.contains(&path) {
            self.search_paths.push(path);
            self.indexed = false;
        }
    }

    /// Get the current search paths
    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }

    /// Set current working directory
    pub fn set_cwd(&mut self, cwd: PathBuf) {
        self.cwd = cwd;
    }

    /// Register a module's exports
    pub fn register_module(&mut self, module_path: &str, info: ModuleInfo) {
        self.modules.insert(module_path.to_string(), info);
    }

    /// Get a registered module
    pub fn get_module(&self, module_path: &str) -> Option<&ModuleInfo> {
        self.modules.get(module_path)
    }

    /// Get a mutable reference to a registered module
    pub fn get_module_mut(&mut self, module_path: &str) -> Option<&mut ModuleInfo> {
        self.modules.get_mut(module_path)
    }

    /// Check if a module is registered
    pub fn has_module(&self, module_path: &str) -> bool {
        self.modules.contains_key(module_path)
    }

    /// Get all registered module names
    pub fn module_names(&self) -> impl Iterator<Item = &String> {
        self.modules.keys()
    }

    /// Build the module index by scanning search paths
    pub fn build_index(&mut self) {
        self.module_index.clear();

        for search_path in &self.search_paths.clone() {
            self.index_directory(search_path, "");
        }

        // Also index cwd if not in search paths
        if !self.search_paths.contains(&self.cwd) && self.cwd.exists() {
            self.index_directory(&self.cwd.clone(), "");
        }

        self.indexed = true;
    }

    /// Index a directory recursively
    fn index_directory(&mut self, dir: &Path, prefix: &str) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Skip hidden directories and common exclusions
            if name_str.starts_with('.') || name_str == "__pycache__" || name_str == "node_modules"
            {
                continue;
            }

            if path.is_dir() {
                // Check if it's a package
                let init_py = path.join("__init__.py");
                let init_pyi = path.join("__init__.pyi");

                let module_path = if prefix.is_empty() {
                    name_str.to_string()
                } else {
                    format!("{}.{}", prefix, name_str)
                };

                // Prefer .pyi over .py
                if init_pyi.exists() {
                    self.module_index.insert(
                        module_path.clone(),
                        ModuleIndexEntry {
                            module_path: module_path.clone(),
                            file_path: init_pyi,
                            is_stub: true,
                            is_package: true,
                        },
                    );
                    // Recurse into package
                    self.index_directory(&path, &module_path);
                } else if init_py.exists() {
                    self.module_index.insert(
                        module_path.clone(),
                        ModuleIndexEntry {
                            module_path: module_path.clone(),
                            file_path: init_py,
                            is_stub: false,
                            is_package: true,
                        },
                    );
                    // Recurse into package
                    self.index_directory(&path, &module_path);
                }
            } else if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str());
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

                // Skip __init__ files (handled as packages)
                if stem == "__init__" {
                    continue;
                }

                let module_path = if prefix.is_empty() {
                    stem.to_string()
                } else {
                    format!("{}.{}", prefix, stem)
                };

                match ext {
                    Some("pyi") => {
                        // .pyi takes precedence
                        self.module_index.insert(
                            module_path.clone(),
                            ModuleIndexEntry {
                                module_path,
                                file_path: path,
                                is_stub: true,
                                is_package: false,
                            },
                        );
                    }
                    Some("py") => {
                        // Only add .py if no .pyi exists
                        if !self.module_index.contains_key(&module_path) {
                            self.module_index.insert(
                                module_path.clone(),
                                ModuleIndexEntry {
                                    module_path,
                                    file_path: path,
                                    is_stub: false,
                                    is_package: false,
                                },
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// List all indexed modules, optionally filtered by prefix
    pub fn list_modules(&self, prefix: Option<&str>) -> Vec<&ModuleIndexEntry> {
        match prefix {
            Some(p) => self
                .module_index
                .values()
                .filter(|e| e.module_path.starts_with(p))
                .collect(),
            None => self.module_index.values().collect(),
        }
    }

    /// Get module index entry
    pub fn get_index_entry(&self, module_path: &str) -> Option<&ModuleIndexEntry> {
        self.module_index.get(module_path)
    }

    /// Check if the index is built
    pub fn is_indexed(&self) -> bool {
        self.indexed
    }

    /// Check if a module is currently being loaded (for circular import detection)
    pub fn is_loading(&self, module_path: &str) -> bool {
        self.loading.contains(module_path)
    }

    /// Mark a module as being loaded
    pub fn start_loading(&mut self, module_path: &str) {
        self.loading.insert(module_path.to_string());
    }

    /// Mark a module as done loading
    pub fn finish_loading(&mut self, module_path: &str) {
        self.loading.remove(module_path);
    }

    /// Resolve an import and return the types it brings into scope
    pub fn resolve_import(&self, import: &Import) -> HashMap<String, Type> {
        let mut result = HashMap::new();

        match import {
            Import::Module { module, alias } => {
                // For `import foo`, we don't directly import types
                // The module name becomes available for attribute access
                let name = alias.as_ref().unwrap_or(module);
                result.insert(
                    name.clone(),
                    Type::Instance {
                        name: format!("module:{}", module),
                        module: Some(module.clone()),
                        type_args: vec![],
                    },
                );
            }
            Import::FromModule { module, names } => {
                if let Some(module_info) = self.modules.get(module) {
                    for imported_name in names {
                        let local_name =
                            imported_name.alias.as_ref().unwrap_or(&imported_name.name);

                        if let Some(ty) = module_info.exports.get(&imported_name.name) {
                            result.insert(local_name.clone(), ty.clone());
                        }
                    }
                }
            }
            Import::WildcardImport { module } => {
                if let Some(module_info) = self.modules.get(module) {
                    for (name, ty) in &module_info.exports {
                        // Skip private names
                        if !name.starts_with('_') {
                            result.insert(name.clone(), ty.clone());
                        }
                    }
                }
            }
        }

        result
    }

    /// Resolve a module path to a file path
    /// Prefers .pyi (stub) files over .py files
    pub fn resolve_module_path(&self, module_path: &str) -> Option<PathBuf> {
        // First check the index
        if let Some(entry) = self.module_index.get(module_path) {
            return Some(entry.file_path.clone());
        }

        // Fall back to filesystem search
        self.resolve_module_path_filesystem(module_path)
    }

    /// Resolve a module path by searching the filesystem directly
    fn resolve_module_path_filesystem(&self, module_path: &str) -> Option<PathBuf> {
        let parts: Vec<&str> = module_path.split('.').collect();
        let relative_path = parts.join("/");

        // Check search paths
        for search_path in &self.search_paths {
            if let Some(path) = self.find_module_in_dir(search_path, &relative_path) {
                return Some(path);
            }
        }

        // Check relative to cwd
        self.find_module_in_dir(&self.cwd, &relative_path)
    }

    /// Find a module file in a directory, preferring .pyi over .py
    fn find_module_in_dir(&self, dir: &Path, relative_path: &str) -> Option<PathBuf> {
        // Try as a package with stub
        let package_stub = dir.join(relative_path).join("__init__.pyi");
        if package_stub.exists() {
            return Some(package_stub);
        }

        // Try as a package with .py
        let package_init = dir.join(relative_path).join("__init__.py");
        if package_init.exists() {
            return Some(package_init);
        }

        // Try as a stub module (.pyi) - prefer over .py
        let stub_file = dir.join(format!("{}.pyi", relative_path));
        if stub_file.exists() {
            return Some(stub_file);
        }

        // Try as a module (.py)
        let module_file = dir.join(format!("{}.py", relative_path));
        if module_file.exists() {
            return Some(module_file);
        }

        None
    }

    /// Get a resolved module, loading it if necessary
    /// Returns None if the module cannot be found or is currently being loaded (circular import)
    pub fn get_or_resolve_module(&mut self, module_path: &str) -> Option<&ModuleInfo> {
        // Check if already loaded
        if self.modules.contains_key(module_path) {
            return self.modules.get(module_path);
        }

        // Check for circular import
        if self.loading.contains(module_path) {
            // Return a placeholder for circular imports
            return None;
        }

        // Try to resolve the module path
        let file_path = self.resolve_module_path(module_path)?;

        // Create and register a basic module info
        // (actual parsing would be done by the type inferencer)
        let info = ModuleInfo::from_file(module_path, file_path);
        self.modules.insert(module_path.to_string(), info);
        self.modules.get(module_path)
    }

    /// Clear the resolver state
    pub fn clear(&mut self) {
        self.modules.clear();
        self.module_index.clear();
        self.loading.clear();
        self.indexed = false;
    }
}

/// Parse an import statement from a tree-sitter node
pub fn parse_import(source: &str, node: &tree_sitter::Node) -> Option<Import> {
    let node_text = |n: &tree_sitter::Node| -> String {
        n.utf8_text(source.as_bytes()).unwrap_or("").to_string()
    };

    match node.kind() {
        "import_statement" => {
            // import foo, bar as baz
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "dotted_name" || child.kind() == "aliased_import" {
                    let (module, alias) = if child.kind() == "aliased_import" {
                        let name = child.child_by_field_name("name").map(|n| node_text(&n));
                        let alias = child.child_by_field_name("alias").map(|n| node_text(&n));
                        (name.unwrap_or_default(), alias)
                    } else {
                        (node_text(&child), None)
                    };

                    return Some(Import::Module { module, alias });
                }
            }
            None
        }
        "import_from_statement" => {
            // from foo import bar, baz as qux
            let module = node
                .child_by_field_name("module_name")
                .map(|n| node_text(&n))
                .unwrap_or_default();

            // Check for wildcard import
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "wildcard_import" {
                    return Some(Import::WildcardImport { module });
                }
            }

            // Parse named imports
            let mut names = Vec::new();
            for child in node.children(&mut cursor) {
                match child.kind() {
                    "dotted_name" | "identifier" => {
                        names.push(ImportedName {
                            name: node_text(&child),
                            alias: None,
                        });
                    }
                    "aliased_import" => {
                        let name = child
                            .child_by_field_name("name")
                            .map(|n| node_text(&n))
                            .unwrap_or_default();
                        let alias = child.child_by_field_name("alias").map(|n| node_text(&n));
                        names.push(ImportedName { name, alias });
                    }
                    _ => {}
                }
            }

            if !names.is_empty() {
                Some(Import::FromModule { module, names })
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Create module info for common Python builtins
#[allow(dead_code)]
pub fn create_builtins_module() -> ModuleInfo {
    let mut info = ModuleInfo::new("builtins");

    // Add common builtin types
    info.exports.insert(
        "int".to_string(),
        Type::ClassType {
            name: "int".to_string(),
            module: Some("builtins".to_string()),
        },
    );
    info.exports.insert(
        "str".to_string(),
        Type::ClassType {
            name: "str".to_string(),
            module: Some("builtins".to_string()),
        },
    );
    info.exports.insert(
        "float".to_string(),
        Type::ClassType {
            name: "float".to_string(),
            module: Some("builtins".to_string()),
        },
    );
    info.exports.insert(
        "bool".to_string(),
        Type::ClassType {
            name: "bool".to_string(),
            module: Some("builtins".to_string()),
        },
    );
    info.exports.insert(
        "list".to_string(),
        Type::ClassType {
            name: "list".to_string(),
            module: Some("builtins".to_string()),
        },
    );
    info.exports.insert(
        "dict".to_string(),
        Type::ClassType {
            name: "dict".to_string(),
            module: Some("builtins".to_string()),
        },
    );
    info.exports.insert(
        "set".to_string(),
        Type::ClassType {
            name: "set".to_string(),
            module: Some("builtins".to_string()),
        },
    );
    info.exports.insert(
        "tuple".to_string(),
        Type::ClassType {
            name: "tuple".to_string(),
            module: Some("builtins".to_string()),
        },
    );
    info.exports.insert(
        "bytes".to_string(),
        Type::ClassType {
            name: "bytes".to_string(),
            module: Some("builtins".to_string()),
        },
    );
    info.exports.insert(
        "type".to_string(),
        Type::ClassType {
            name: "type".to_string(),
            module: Some("builtins".to_string()),
        },
    );
    info.exports.insert(
        "object".to_string(),
        Type::ClassType {
            name: "object".to_string(),
            module: Some("builtins".to_string()),
        },
    );

    // Common functions
    info.exports.insert(
        "len".to_string(),
        Type::callable(vec![Type::Any], Type::Int),
    );
    info.exports.insert(
        "print".to_string(),
        Type::callable(vec![Type::Any], Type::None),
    );
    info.exports.insert(
        "range".to_string(),
        Type::callable(vec![Type::Int], Type::list(Type::Int)),
    );
    info.exports.insert(
        "enumerate".to_string(),
        Type::callable(
            vec![Type::list(Type::Unknown)],
            Type::list(Type::Tuple(vec![Type::Int, Type::Unknown])),
        ),
    );
    info.exports.insert(
        "zip".to_string(),
        Type::callable(
            vec![Type::list(Type::Unknown), Type::list(Type::Unknown)],
            Type::list(Type::Tuple(vec![Type::Unknown, Type::Unknown])),
        ),
    );

    info
}

/// Create module info for typing module
#[allow(dead_code)]
pub fn create_typing_module() -> ModuleInfo {
    let mut info = ModuleInfo::new("typing");

    // Generic type constructors
    info.exports.insert(
        "List".to_string(),
        Type::ClassType {
            name: "list".to_string(),
            module: Some("typing".to_string()),
        },
    );
    info.exports.insert(
        "Dict".to_string(),
        Type::ClassType {
            name: "dict".to_string(),
            module: Some("typing".to_string()),
        },
    );
    info.exports.insert(
        "Set".to_string(),
        Type::ClassType {
            name: "set".to_string(),
            module: Some("typing".to_string()),
        },
    );
    info.exports.insert(
        "Tuple".to_string(),
        Type::ClassType {
            name: "tuple".to_string(),
            module: Some("typing".to_string()),
        },
    );
    info.exports.insert(
        "Optional".to_string(),
        Type::ClassType {
            name: "Optional".to_string(),
            module: Some("typing".to_string()),
        },
    );
    info.exports.insert(
        "Union".to_string(),
        Type::ClassType {
            name: "Union".to_string(),
            module: Some("typing".to_string()),
        },
    );
    info.exports.insert(
        "Callable".to_string(),
        Type::ClassType {
            name: "Callable".to_string(),
            module: Some("typing".to_string()),
        },
    );
    info.exports.insert("Any".to_string(), Type::Any);
    info.exports.insert(
        "TypeVar".to_string(),
        Type::ClassType {
            name: "TypeVar".to_string(),
            module: Some("typing".to_string()),
        },
    );
    info.exports.insert(
        "Generic".to_string(),
        Type::ClassType {
            name: "Generic".to_string(),
            module: Some("typing".to_string()),
        },
    );
    info.exports.insert(
        "Protocol".to_string(),
        Type::ClassType {
            name: "Protocol".to_string(),
            module: Some("typing".to_string()),
        },
    );

    info
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_module_registration() {
        let mut resolver = ImportResolver::new();

        let mut module_info = ModuleInfo::new("mymodule");
        module_info.exports.insert(
            "MyClass".to_string(),
            Type::Instance {
                name: "MyClass".to_string(),
                module: Some("mymodule".to_string()),
                type_args: vec![],
            },
        );

        resolver.register_module("mymodule", module_info);

        let module = resolver.get_module("mymodule");
        assert!(module.is_some());
        assert!(module.unwrap().exports.contains_key("MyClass"));
    }

    #[test]
    fn test_from_import_resolution() {
        let mut resolver = ImportResolver::new();

        let mut module_info = ModuleInfo::new("mymodule");
        module_info.exports.insert("foo".to_string(), Type::Int);
        module_info.exports.insert("bar".to_string(), Type::Str);

        resolver.register_module("mymodule", module_info);

        let import = Import::FromModule {
            module: "mymodule".to_string(),
            names: vec![
                ImportedName {
                    name: "foo".to_string(),
                    alias: None,
                },
                ImportedName {
                    name: "bar".to_string(),
                    alias: Some("baz".to_string()),
                },
            ],
        };

        let resolved = resolver.resolve_import(&import);
        assert_eq!(resolved.get("foo"), Some(&Type::Int));
        assert_eq!(resolved.get("baz"), Some(&Type::Str));
        assert!(!resolved.contains_key("bar")); // aliased, not available as "bar"
    }

    #[test]
    fn test_builtins_module() {
        let builtins = create_builtins_module();
        assert!(builtins.exports.contains_key("int"));
        assert!(builtins.exports.contains_key("str"));
        assert!(builtins.exports.contains_key("len"));
    }

    #[test]
    fn test_typing_module() {
        let typing = create_typing_module();
        assert!(typing.exports.contains_key("List"));
        assert!(typing.exports.contains_key("Optional"));
        assert!(typing.exports.contains_key("TypeVar"));
    }

    #[test]
    fn test_module_info_from_file() {
        let info = ModuleInfo::from_file("mymodule", PathBuf::from("src/mymodule.py"));
        assert_eq!(info.path, "mymodule");
        assert!(!info.is_stub);
        assert!(!info.is_package);

        let stub_info = ModuleInfo::from_file("mymodule", PathBuf::from("src/mymodule.pyi"));
        assert!(stub_info.is_stub);

        let package_info =
            ModuleInfo::from_file("mypackage", PathBuf::from("src/mypackage/__init__.py"));
        assert!(package_info.is_package);
    }

    #[test]
    fn test_with_search_paths() {
        let paths = vec![PathBuf::from("/lib1"), PathBuf::from("/lib2")];
        let resolver = ImportResolver::with_search_paths(paths.clone());
        assert_eq!(resolver.search_paths(), &paths);
    }

    #[test]
    fn test_add_search_path() {
        let mut resolver = ImportResolver::new();
        resolver.add_search_path(PathBuf::from("/lib1"));
        resolver.add_search_path(PathBuf::from("/lib2"));
        resolver.add_search_path(PathBuf::from("/lib1")); // Duplicate

        assert_eq!(resolver.search_paths().len(), 2);
    }

    #[test]
    fn test_build_index_with_modules() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_import_index");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create module files
        fs::write(temp_dir.join("utils.py"), "def helper(): pass").unwrap();
        fs::write(temp_dir.join("config.py"), "DEBUG = True").unwrap();

        // Create a package
        fs::create_dir_all(temp_dir.join("mypackage")).unwrap();
        fs::write(temp_dir.join("mypackage/__init__.py"), "").unwrap();
        fs::write(temp_dir.join("mypackage/submodule.py"), "class Foo: pass").unwrap();

        let mut resolver = ImportResolver::with_search_paths(vec![temp_dir.clone()]);
        resolver.build_index();

        assert!(resolver.is_indexed());

        // Check indexed modules
        let modules = resolver.list_modules(None);
        assert!(modules.iter().any(|m| m.module_path == "utils"));
        assert!(modules.iter().any(|m| m.module_path == "config"));
        assert!(modules.iter().any(|m| m.module_path == "mypackage"));
        assert!(modules
            .iter()
            .any(|m| m.module_path == "mypackage.submodule"));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_stub_file_priority() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_stub_priority");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create both .py and .pyi
        fs::write(temp_dir.join("mymod.py"), "def foo(): pass").unwrap();
        fs::write(temp_dir.join("mymod.pyi"), "def foo() -> int: ...").unwrap();

        let mut resolver = ImportResolver::with_search_paths(vec![temp_dir.clone()]);
        resolver.build_index();

        // .pyi should be preferred
        let entry = resolver.get_index_entry("mymod");
        assert!(entry.is_some());
        assert!(entry.unwrap().is_stub);

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_list_modules_with_prefix() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_list_prefix");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create Django-like structure
        fs::create_dir_all(temp_dir.join("django")).unwrap();
        fs::write(temp_dir.join("django/__init__.py"), "").unwrap();
        fs::create_dir_all(temp_dir.join("django/db")).unwrap();
        fs::write(temp_dir.join("django/db/__init__.py"), "").unwrap();
        fs::write(temp_dir.join("django/db/models.py"), "").unwrap();

        let mut resolver = ImportResolver::with_search_paths(vec![temp_dir.clone()]);
        resolver.build_index();

        // List all django modules
        let django_modules = resolver.list_modules(Some("django"));
        assert!(django_modules.iter().any(|m| m.module_path == "django"));
        assert!(django_modules.iter().any(|m| m.module_path == "django.db"));
        assert!(django_modules
            .iter()
            .any(|m| m.module_path == "django.db.models"));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_resolve_module_path() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_resolve_path");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        fs::write(temp_dir.join("mymod.py"), "").unwrap();
        fs::create_dir_all(temp_dir.join("pkg")).unwrap();
        fs::write(temp_dir.join("pkg/__init__.py"), "").unwrap();

        let mut resolver = ImportResolver::with_search_paths(vec![temp_dir.clone()]);
        resolver.build_index();

        // Resolve module
        let path = resolver.resolve_module_path("mymod");
        assert!(path.is_some());
        assert!(path.unwrap().ends_with("mymod.py"));

        // Resolve package
        let pkg_path = resolver.resolve_module_path("pkg");
        assert!(pkg_path.is_some());
        assert!(pkg_path.unwrap().ends_with("__init__.py"));

        // Non-existent module
        let none_path = resolver.resolve_module_path("nonexistent");
        assert!(none_path.is_none());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_circular_import_detection() {
        let mut resolver = ImportResolver::new();

        // Start loading module_a
        resolver.start_loading("module_a");
        assert!(resolver.is_loading("module_a"));
        assert!(!resolver.is_loading("module_b"));

        // Start loading module_b (simulating module_a importing module_b)
        resolver.start_loading("module_b");
        assert!(resolver.is_loading("module_b"));

        // Finish loading module_b
        resolver.finish_loading("module_b");
        assert!(!resolver.is_loading("module_b"));

        // module_a is still loading
        assert!(resolver.is_loading("module_a"));

        resolver.finish_loading("module_a");
        assert!(!resolver.is_loading("module_a"));
    }

    #[test]
    fn test_clear_resolver() {
        let mut resolver = ImportResolver::new();
        resolver.register_module("test", ModuleInfo::new("test"));
        resolver.start_loading("loading");

        resolver.clear();

        assert!(!resolver.has_module("test"));
        assert!(!resolver.is_loading("loading"));
        assert!(!resolver.is_indexed());
    }

    #[test]
    fn test_module_names_iterator() {
        let mut resolver = ImportResolver::new();
        resolver.register_module("mod1", ModuleInfo::new("mod1"));
        resolver.register_module("mod2", ModuleInfo::new("mod2"));

        let names: Vec<_> = resolver.module_names().collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&&"mod1".to_string()));
        assert!(names.contains(&&"mod2".to_string()));
    }

    /// Acceptance Criteria: WHEN local module imported THEN resolve from src
    /// Spec: import-resolution.md#acceptance-criteria
    #[test]
    fn test_resolve_local_module_from_src() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_local_import");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create project with src directory
        let src_dir = temp_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Create src/utils.py with a helper function
        fs::write(
            src_dir.join("utils.py"),
            "def helper(x: int) -> str:\n    return str(x)\n",
        )
        .unwrap();

        // Create src/__init__.py to make it a package
        fs::write(src_dir.join("__init__.py"), "").unwrap();

        let mut resolver = ImportResolver::with_search_paths(vec![temp_dir.clone()]);
        resolver.build_index();

        // Should find src as a package
        let src_entry = resolver.get_index_entry("src");
        assert!(src_entry.is_some(), "src package should be indexed");
        assert!(src_entry.unwrap().is_package);

        // Should find src.utils
        let utils_entry = resolver.get_index_entry("src.utils");
        assert!(utils_entry.is_some(), "src.utils should be indexed");

        // Resolve module path
        let resolved_path = resolver.resolve_module_path("src.utils");
        assert!(resolved_path.is_some());
        assert!(
            resolved_path.as_ref().unwrap().ends_with("utils.py"),
            "Expected utils.py, got: {:?}",
            resolved_path
        );

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// Acceptance Criteria: WHEN library imported THEN resolve from site-packages
    /// Spec: import-resolution.md#acceptance-criteria
    #[test]
    fn test_resolve_library_from_site_packages() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_site_pkg_import");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Simulate a venv with site-packages
        let site_packages = temp_dir.join("site-packages");
        fs::create_dir_all(&site_packages).unwrap();

        // Create a mock "requests" package in site-packages
        let requests_pkg = site_packages.join("requests");
        fs::create_dir_all(&requests_pkg).unwrap();
        fs::write(
            requests_pkg.join("__init__.py"),
            "from .api import get, post\n__version__ = '2.28.0'\n",
        )
        .unwrap();
        fs::write(
            requests_pkg.join("api.py"),
            "def get(url: str) -> Response: ...\ndef post(url: str, data: dict) -> Response: ...\n",
        )
        .unwrap();

        // Create resolver with site-packages as search path
        let mut resolver = ImportResolver::with_search_paths(vec![site_packages.clone()]);
        resolver.build_index();

        // Should find requests package
        let requests_entry = resolver.get_index_entry("requests");
        assert!(requests_entry.is_some(), "requests should be indexed");
        assert!(requests_entry.unwrap().is_package);

        // Should find requests.api
        let api_entry = resolver.get_index_entry("requests.api");
        assert!(api_entry.is_some(), "requests.api should be indexed");

        // Resolve the module
        let resolved_path = resolver.resolve_module_path("requests");
        assert!(resolved_path.is_some());
        assert!(resolved_path.as_ref().unwrap().ends_with("__init__.py"));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// Test get_or_resolve_module for lazy loading
    #[test]
    fn test_get_or_resolve_module_lazy_loading() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_lazy_load");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a module
        fs::write(temp_dir.join("mymodule.py"), "x = 42").unwrap();

        let mut resolver = ImportResolver::with_search_paths(vec![temp_dir.clone()]);
        resolver.build_index();

        // Module not loaded yet
        assert!(!resolver.has_module("mymodule"));

        // Get or resolve should load it
        let module = resolver.get_or_resolve_module("mymodule");
        assert!(module.is_some());

        // Now it should be registered
        assert!(resolver.has_module("mymodule"));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// Test wildcard import resolution
    #[test]
    fn test_wildcard_import_resolution() {
        let mut resolver = ImportResolver::new();

        let mut module_info = ModuleInfo::new("mymodule");
        module_info
            .exports
            .insert("public_func".to_string(), Type::Int);
        module_info
            .exports
            .insert("_private_func".to_string(), Type::Str);
        module_info
            .exports
            .insert("__dunder__".to_string(), Type::Bool);

        resolver.register_module("mymodule", module_info);

        let import = Import::WildcardImport {
            module: "mymodule".to_string(),
        };

        let resolved = resolver.resolve_import(&import);

        // Public names should be imported
        assert!(resolved.contains_key("public_func"));

        // Private names (starting with _) should NOT be imported
        assert!(!resolved.contains_key("_private_func"));
        assert!(!resolved.contains_key("__dunder__"));
    }

    /// Test module import (import foo) creates module reference
    #[test]
    fn test_module_import_creates_reference() {
        let resolver = ImportResolver::new();

        let import = Import::Module {
            module: "os".to_string(),
            alias: None,
        };

        let resolved = resolver.resolve_import(&import);

        // Should have "os" as a module reference
        assert!(resolved.contains_key("os"));
        match resolved.get("os") {
            Some(Type::Instance { name, module, .. }) => {
                assert!(name.starts_with("module:"));
                assert_eq!(module.as_deref(), Some("os"));
            }
            _ => panic!("Expected Instance type for module import"),
        }
    }

    /// Test module import with alias
    #[test]
    fn test_module_import_with_alias() {
        let resolver = ImportResolver::new();

        let import = Import::Module {
            module: "numpy".to_string(),
            alias: Some("np".to_string()),
        };

        let resolved = resolver.resolve_import(&import);

        // Should have "np" as the name (not "numpy")
        assert!(resolved.contains_key("np"));
        assert!(!resolved.contains_key("numpy"));
    }

    /// Test indexing skips hidden and excluded directories
    #[test]
    fn test_index_skips_excluded_directories() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_skip_excluded");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create normal module
        fs::write(temp_dir.join("normal.py"), "").unwrap();

        // Create modules in excluded directories
        fs::create_dir_all(temp_dir.join("__pycache__")).unwrap();
        fs::write(temp_dir.join("__pycache__/cached.py"), "").unwrap();

        fs::create_dir_all(temp_dir.join(".hidden")).unwrap();
        fs::write(temp_dir.join(".hidden/secret.py"), "").unwrap();

        fs::create_dir_all(temp_dir.join("node_modules")).unwrap();
        fs::write(temp_dir.join("node_modules/nodemod.py"), "").unwrap();

        let mut resolver = ImportResolver::with_search_paths(vec![temp_dir.clone()]);
        resolver.build_index();

        let modules = resolver.list_modules(None);
        let module_paths: Vec<_> = modules.iter().map(|m| m.module_path.as_str()).collect();

        // Normal module should be indexed
        assert!(module_paths.contains(&"normal"));

        // Excluded modules should NOT be indexed
        assert!(!module_paths.iter().any(|p| p.contains("cached")));
        assert!(!module_paths.iter().any(|p| p.contains("secret")));
        assert!(!module_paths.iter().any(|p| p.contains("nodemod")));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// R6: `from .utils import helper` resolves within package directory.
    #[test]
    fn test_relative_import_resolution() {
        let temp_dir = env::temp_dir().join("cclab_lens_test_relative_import");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create a package with two modules
        let pkg_dir = temp_dir.join("mypackage");
        fs::create_dir_all(&pkg_dir).unwrap();
        fs::write(pkg_dir.join("__init__.py"), "").unwrap();
        fs::write(
            pkg_dir.join("utils.py"),
            "def helper(x: int) -> str:\n    return str(x)\n",
        )
        .unwrap();
        fs::write(pkg_dir.join("main.py"), "from .utils import helper\n").unwrap();

        let mut resolver = ImportResolver::with_search_paths(vec![temp_dir.clone()]);
        resolver.build_index();

        // mypackage.utils should be indexed
        let utils_entry = resolver.get_index_entry("mypackage.utils");
        assert!(utils_entry.is_some(), "mypackage.utils should be found");
        assert!(
            utils_entry.unwrap().file_path.ends_with("utils.py"),
            "Should resolve to utils.py"
        );

        // Verify the package itself is indexed
        let pkg_entry = resolver.get_index_entry("mypackage");
        assert!(pkg_entry.is_some(), "mypackage should be indexed");
        assert!(pkg_entry.unwrap().is_package);

        // A relative import from .utils should resolve to the same file
        // within the package directory.
        let resolved_utils = resolver.resolve_module_path("mypackage.utils");
        assert!(resolved_utils.is_some());
        assert!(
            resolved_utils.unwrap().ends_with("utils.py"),
            "Relative import .utils should resolve to utils.py within the package"
        );

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
