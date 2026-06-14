//! Stub file (.pyi) support for type information
//!
//! This module handles:
//! - Parsing .pyi stub files
//! - Loading standard library stubs
//! - Third-party stubs (typeshed)

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use tree_sitter::{Node, Parser};

use super::annotation::parse_type_annotation;
use super::imports::ModuleInfo;
use super::ty::{Param, ParamKind, Type};

/// Stub file loader and cache
#[derive(Debug, Default)]
pub struct StubLoader {
    /// Loaded stubs (module path -> module info)
    stubs: HashMap<String, ModuleInfo>,
    /// Stub search paths (e.g., typeshed location)
    stub_paths: Vec<PathBuf>,
    /// Built-in stubs (preloaded)
    builtins_loaded: bool,
}

impl StubLoader {
    pub fn new() -> Self {
        Self {
            stubs: HashMap::new(),
            stub_paths: vec![],
            builtins_loaded: false,
        }
    }

    /// Set stub search paths
    pub fn set_stub_paths(&mut self, paths: Vec<PathBuf>) {
        self.stub_paths = paths;
    }

    /// Add a stub search path
    pub fn add_stub_path(&mut self, path: PathBuf) {
        if !self.stub_paths.contains(&path) {
            self.stub_paths.push(path);
        }
    }

    /// Load builtin stubs
    pub fn load_builtins(&mut self) {
        if self.builtins_loaded {
            return;
        }

        // Load core builtin types
        self.stubs
            .insert("builtins".to_string(), create_builtins_stub());
        self.stubs
            .insert("typing".to_string(), create_typing_stub());
        self.stubs
            .insert("collections".to_string(), create_collections_stub());
        self.stubs
            .insert("collections.abc".to_string(), create_collections_abc_stub());

        // Load bundled typeshed stubs
        use super::typeshed::*;
        self.stubs.insert("os".to_string(), create_os_stub());
        self.stubs
            .insert("os.path".to_string(), create_os_path_stub());
        self.stubs.insert("sys".to_string(), create_sys_stub());
        self.stubs.insert("io".to_string(), create_io_stub());
        self.stubs.insert("re".to_string(), create_re_stub());
        self.stubs.insert("json".to_string(), create_json_stub());
        self.stubs
            .insert("pathlib".to_string(), create_pathlib_stub());
        self.stubs
            .insert("functools".to_string(), create_functools_stub());
        self.stubs
            .insert("itertools".to_string(), create_itertools_stub());
        self.stubs
            .insert("datetime".to_string(), create_datetime_stub());

        self.builtins_loaded = true;
    }

    /// Get or load a stub for a module (loads from .pyi files if not cached)
    /// Returns None if no stub exists for the module
    /// Caller should check inline types first for proper priority:
    /// Priority: inline types > .pyi stubs > typeshed > inferred
    pub fn get_or_load_stub(&mut self, module_path: &str) -> Option<&ModuleInfo> {
        // Check already loaded stubs
        if self.stubs.contains_key(module_path) {
            return self.stubs.get(module_path);
        }

        // Try to load from stub paths (.pyi files)
        if let Some(stub_path) = self.find_stub_file(module_path) {
            if let Some(info) = self.parse_stub_file(&stub_path) {
                self.stubs.insert(module_path.to_string(), info);
                return self.stubs.get(module_path);
            }
        }

        // Return bundled typeshed stub if available
        self.stubs.get(module_path)
    }

    /// Check if a package is typed (has py.typed marker)
    #[allow(dead_code)]
    pub fn is_typed_package(&self, package_path: &Path) -> bool {
        package_path.join("py.typed").exists()
    }

    /// Get stub for a module
    pub fn get_stub(&self, module_path: &str) -> Option<&ModuleInfo> {
        self.stubs.get(module_path)
    }

    /// Check if a stub exists for a module
    pub fn has_stub(&self, module_path: &str) -> bool {
        self.stubs.contains_key(module_path)
    }

    /// Try to load a stub file for a module
    #[allow(dead_code)]
    pub fn load_stub(&mut self, module_path: &str) -> Option<&ModuleInfo> {
        if self.stubs.contains_key(module_path) {
            return self.stubs.get(module_path);
        }

        // Try to find and load the stub
        if let Some(stub_path) = self.find_stub_file(module_path) {
            if let Some(info) = self.parse_stub_file(&stub_path) {
                self.stubs.insert(module_path.to_string(), info);
                return self.stubs.get(module_path);
            }
        }

        None
    }

    /// Find stub file for a module
    fn find_stub_file(&self, module_path: &str) -> Option<PathBuf> {
        let parts: Vec<&str> = module_path.split('.').collect();
        let relative_path = parts.join("/");

        for stub_path in &self.stub_paths {
            // Try as package stub
            let package_stub = stub_path.join(&relative_path).join("__init__.pyi");
            if package_stub.exists() {
                return Some(package_stub);
            }

            // Try as module stub
            let module_stub = stub_path.join(format!("{}.pyi", relative_path));
            if module_stub.exists() {
                return Some(module_stub);
            }
        }

        None
    }

    /// Parse a stub file using tree-sitter
    fn parse_stub_file(&self, path: &Path) -> Option<ModuleInfo> {
        let source = fs::read_to_string(path).ok()?;
        let module_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .ok()?;

        let tree = parser.parse(&source, None)?;
        let root = tree.root_node();

        let mut info = ModuleInfo::new(module_name);
        self.parse_stub_definitions(&source, &root, &mut info);

        Some(info)
    }

    /// Parse stub definitions from AST
    fn parse_stub_definitions(&self, source: &str, node: &Node, info: &mut ModuleInfo) {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "function_definition" => {
                    if let Some((name, ty)) = self.parse_function_stub(source, &child) {
                        // Check for @overload decorator
                        if self.has_overload_decorator(source, &child) {
                            // Add to overload signatures
                            if let Some(existing) = info.exports.get_mut(&name) {
                                if let Type::Overloaded { signatures } = existing {
                                    signatures.push(ty);
                                } else {
                                    // Convert to overloaded
                                    let old = existing.clone();
                                    *existing = Type::Overloaded {
                                        signatures: vec![old, ty],
                                    };
                                }
                            } else {
                                info.exports.insert(name, ty);
                            }
                        } else {
                            info.exports.insert(name, ty);
                        }
                    }
                }
                "class_definition" => {
                    if let Some((name, ty)) = self.parse_class_stub(source, &child) {
                        info.exports.insert(name, ty);
                    }
                }
                "expression_statement" => {
                    // Type alias: Name = Type or Name: TypeAlias = Type
                    if let Some((name, ty)) = self.parse_type_alias(source, &child) {
                        info.exports.insert(name, ty);
                    }
                }
                "import_from_statement" | "import_statement" => {
                    // Track re-exports for __init__.pyi files
                    self.parse_import_export(source, &child, info);
                }
                _ => {}
            }
        }
    }

    /// Check if function has @overload decorator
    fn has_overload_decorator(&self, source: &str, node: &Node) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "decorator" {
                let text = self.node_text(source, &child);
                if text.contains("overload") {
                    return true;
                }
            }
        }
        false
    }

    /// Parse a function stub
    fn parse_function_stub(&self, source: &str, node: &Node) -> Option<(String, Type)> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.node_text(source, &name_node).to_string();

        let params_node = node.child_by_field_name("parameters")?;
        let params = self.parse_parameters(source, &params_node);

        let return_type = node
            .child_by_field_name("return_type")
            .map(|n| parse_type_annotation(source, &n))
            .unwrap_or(Type::Any);

        Some((
            name,
            Type::Callable {
                params,
                ret: Box::new(return_type),
            },
        ))
    }

    /// Parse function parameters
    fn parse_parameters(&self, source: &str, node: &Node) -> Vec<Param> {
        let mut params = Vec::new();
        let mut cursor = node.walk();
        let mut positional_only = false;
        let mut keyword_only = false;

        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    // Simple parameter without annotation
                    let name = self.node_text(source, &child).to_string();
                    if name != "self" && name != "cls" {
                        let kind = if keyword_only {
                            ParamKind::KeywordOnly
                        } else if positional_only {
                            ParamKind::PositionalOnly
                        } else {
                            ParamKind::Positional
                        };
                        params.push(Param {
                            name,
                            ty: Type::Any,
                            has_default: false,
                            kind,
                        });
                    }
                }
                "typed_parameter" | "typed_default_parameter" => {
                    if let Some(param) =
                        self.parse_typed_param(source, &child, keyword_only, positional_only)
                    {
                        params.push(param);
                    }
                }
                "default_parameter" => {
                    if let Some(param) =
                        self.parse_default_param(source, &child, keyword_only, positional_only)
                    {
                        params.push(param);
                    }
                }
                "list_splat_pattern" => {
                    // *args
                    if let Some(name_node) = child.child(1) {
                        let name = self.node_text(source, &name_node).to_string();
                        params.push(Param {
                            name,
                            ty: Type::Any,
                            has_default: false,
                            kind: ParamKind::VarPositional,
                        });
                    }
                    keyword_only = true;
                }
                "dictionary_splat_pattern" => {
                    // **kwargs
                    if let Some(name_node) = child.child(1) {
                        let name = self.node_text(source, &name_node).to_string();
                        params.push(Param {
                            name,
                            ty: Type::Any,
                            has_default: false,
                            kind: ParamKind::VarKeyword,
                        });
                    }
                }
                "/" => {
                    positional_only = true;
                }
                "*" => {
                    keyword_only = true;
                }
                _ => {}
            }
        }

        params
    }

    /// Parse a typed parameter
    fn parse_typed_param(
        &self,
        source: &str,
        node: &Node,
        keyword_only: bool,
        positional_only: bool,
    ) -> Option<Param> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.node_text(source, &name_node).to_string();

        if name == "self" || name == "cls" {
            return None;
        }

        let ty = node
            .child_by_field_name("type")
            .map(|n| parse_type_annotation(source, &n))
            .unwrap_or(Type::Any);

        let has_default = node.child_by_field_name("value").is_some();

        let kind = if keyword_only {
            ParamKind::KeywordOnly
        } else if positional_only {
            ParamKind::PositionalOnly
        } else {
            ParamKind::Positional
        };

        Some(Param {
            name,
            ty,
            has_default,
            kind,
        })
    }

    /// Parse a default parameter (no type annotation)
    fn parse_default_param(
        &self,
        source: &str,
        node: &Node,
        keyword_only: bool,
        positional_only: bool,
    ) -> Option<Param> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.node_text(source, &name_node).to_string();

        if name == "self" || name == "cls" {
            return None;
        }

        let kind = if keyword_only {
            ParamKind::KeywordOnly
        } else if positional_only {
            ParamKind::PositionalOnly
        } else {
            ParamKind::Positional
        };

        Some(Param {
            name,
            ty: Type::Any,
            has_default: true,
            kind,
        })
    }

    /// Parse a class stub
    fn parse_class_stub(&self, source: &str, node: &Node) -> Option<(String, Type)> {
        let name_node = node.child_by_field_name("name")?;
        let name = self.node_text(source, &name_node).to_string();

        // For now, return a ClassType
        // In the future, we could parse methods and attributes
        Some((name.clone(), Type::ClassType { name, module: None }))
    }

    /// Parse a type alias
    fn parse_type_alias(&self, source: &str, node: &Node) -> Option<(String, Type)> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "assignment" {
                let left = child.child_by_field_name("left")?;
                let right = child.child_by_field_name("right")?;

                if left.kind() == "identifier" {
                    let name = self.node_text(source, &left).to_string();
                    let ty = parse_type_annotation(source, &right);
                    return Some((name, ty));
                }
            }
        }
        None
    }

    /// Parse import/export for re-exports
    fn parse_import_export(&self, source: &str, node: &Node, info: &mut ModuleInfo) {
        if node.kind() == "import_from_statement" {
            // from module import name -> re-export
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "dotted_name" || child.kind() == "identifier" {
                    let text = self.node_text(source, &child);
                    // Skip the module name, just track imported names
                    if child.prev_sibling().map(|n| n.kind()) == Some("import") {
                        info.exports.insert(
                            text.to_string(),
                            Type::Instance {
                                name: text.to_string(),
                                module: None,
                                type_args: vec![],
                            },
                        );
                    }
                }
                if child.kind() == "aliased_import" {
                    if let Some(alias) = child.child_by_field_name("alias") {
                        let name = self.node_text(source, &alias);
                        info.exports.insert(
                            name.to_string(),
                            Type::Instance {
                                name: name.to_string(),
                                module: None,
                                type_args: vec![],
                            },
                        );
                    }
                }
            }
        }
    }

    /// Get text of a node
    fn node_text<'a>(&self, source: &'a str, node: &Node) -> &'a str {
        node.utf8_text(source.as_bytes()).unwrap_or("")
    }

    /// Get all loaded modules
    pub fn modules(&self) -> impl Iterator<Item = (&String, &ModuleInfo)> {
        self.stubs.iter()
    }
}

/// Create builtins stub
fn create_builtins_stub() -> ModuleInfo {
    let mut info = ModuleInfo::new("builtins");

    // Primitive type constructors
    let primitives = [
        (
            "int",
            Type::ClassType {
                name: "int".to_string(),
                module: Some("builtins".to_string()),
            },
        ),
        (
            "float",
            Type::ClassType {
                name: "float".to_string(),
                module: Some("builtins".to_string()),
            },
        ),
        (
            "str",
            Type::ClassType {
                name: "str".to_string(),
                module: Some("builtins".to_string()),
            },
        ),
        (
            "bool",
            Type::ClassType {
                name: "bool".to_string(),
                module: Some("builtins".to_string()),
            },
        ),
        (
            "bytes",
            Type::ClassType {
                name: "bytes".to_string(),
                module: Some("builtins".to_string()),
            },
        ),
        (
            "bytearray",
            Type::ClassType {
                name: "bytearray".to_string(),
                module: Some("builtins".to_string()),
            },
        ),
        (
            "object",
            Type::ClassType {
                name: "object".to_string(),
                module: Some("builtins".to_string()),
            },
        ),
        (
            "type",
            Type::ClassType {
                name: "type".to_string(),
                module: Some("builtins".to_string()),
            },
        ),
    ];

    for (name, ty) in primitives {
        info.exports.insert(name.to_string(), ty);
    }

    // Container types
    let containers = [
        (
            "list",
            Type::ClassType {
                name: "list".to_string(),
                module: Some("builtins".to_string()),
            },
        ),
        (
            "dict",
            Type::ClassType {
                name: "dict".to_string(),
                module: Some("builtins".to_string()),
            },
        ),
        (
            "set",
            Type::ClassType {
                name: "set".to_string(),
                module: Some("builtins".to_string()),
            },
        ),
        (
            "frozenset",
            Type::ClassType {
                name: "frozenset".to_string(),
                module: Some("builtins".to_string()),
            },
        ),
        (
            "tuple",
            Type::ClassType {
                name: "tuple".to_string(),
                module: Some("builtins".to_string()),
            },
        ),
    ];

    for (name, ty) in containers {
        info.exports.insert(name.to_string(), ty);
    }

    // Common functions
    info.exports.insert(
        "len".to_string(),
        Type::callable(vec![Type::Any], Type::Int),
    );
    info.exports.insert(
        "abs".to_string(),
        Type::callable(vec![Type::Any], Type::Int),
    );
    info.exports.insert(
        "min".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );
    info.exports.insert(
        "max".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );
    info.exports.insert(
        "sum".to_string(),
        Type::callable(vec![Type::Any], Type::Int),
    );
    info.exports.insert(
        "sorted".to_string(),
        Type::callable(vec![Type::Any], Type::list(Type::Unknown)),
    );
    info.exports.insert(
        "reversed".to_string(),
        Type::callable(vec![Type::Any], Type::list(Type::Unknown)),
    );
    info.exports.insert(
        "enumerate".to_string(),
        Type::callable(
            vec![Type::Any],
            Type::list(Type::Tuple(vec![Type::Int, Type::Unknown])),
        ),
    );
    info.exports.insert(
        "zip".to_string(),
        Type::callable(
            vec![Type::Any, Type::Any],
            Type::list(Type::Tuple(vec![Type::Unknown, Type::Unknown])),
        ),
    );
    info.exports.insert(
        "map".to_string(),
        Type::callable(vec![Type::Any, Type::Any], Type::list(Type::Unknown)),
    );
    info.exports.insert(
        "filter".to_string(),
        Type::callable(vec![Type::Any, Type::Any], Type::list(Type::Unknown)),
    );
    info.exports.insert(
        "range".to_string(),
        Type::callable(vec![Type::Int], Type::list(Type::Int)),
    );
    info.exports.insert(
        "print".to_string(),
        Type::callable(vec![Type::Any], Type::None),
    );
    info.exports.insert(
        "input".to_string(),
        Type::callable(vec![Type::Str], Type::Str),
    );
    info.exports.insert(
        "open".to_string(),
        Type::callable(vec![Type::Str], Type::Any),
    ); // Simplified
    info.exports.insert(
        "isinstance".to_string(),
        Type::callable(vec![Type::Any, Type::Any], Type::Bool),
    );
    info.exports.insert(
        "issubclass".to_string(),
        Type::callable(vec![Type::Any, Type::Any], Type::Bool),
    );
    info.exports.insert(
        "hasattr".to_string(),
        Type::callable(vec![Type::Any, Type::Str], Type::Bool),
    );
    info.exports.insert(
        "getattr".to_string(),
        Type::callable(vec![Type::Any, Type::Str], Type::Any),
    );
    info.exports.insert(
        "setattr".to_string(),
        Type::callable(vec![Type::Any, Type::Str, Type::Any], Type::None),
    );
    info.exports.insert(
        "delattr".to_string(),
        Type::callable(vec![Type::Any, Type::Str], Type::None),
    );
    info.exports.insert(
        "callable".to_string(),
        Type::callable(vec![Type::Any], Type::Bool),
    );
    info.exports.insert(
        "repr".to_string(),
        Type::callable(vec![Type::Any], Type::Str),
    );
    info.exports.insert(
        "hash".to_string(),
        Type::callable(vec![Type::Any], Type::Int),
    );
    info.exports
        .insert("id".to_string(), Type::callable(vec![Type::Any], Type::Int));
    info.exports.insert(
        "iter".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );
    info.exports.insert(
        "next".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );

    // Exception types
    let exceptions = [
        "Exception",
        "BaseException",
        "ValueError",
        "TypeError",
        "KeyError",
        "IndexError",
        "AttributeError",
        "RuntimeError",
        "StopIteration",
        "ImportError",
        "ModuleNotFoundError",
        "FileNotFoundError",
        "IOError",
        "OSError",
        "AssertionError",
        "ZeroDivisionError",
        "OverflowError",
        "NameError",
        "UnboundLocalError",
        "NotImplementedError",
    ];

    for exc in exceptions {
        info.exports.insert(
            exc.to_string(),
            Type::ClassType {
                name: exc.to_string(),
                module: Some("builtins".to_string()),
            },
        );
    }

    // Constants
    info.exports.insert("None".to_string(), Type::None);
    info.exports.insert("True".to_string(), Type::Bool);
    info.exports.insert("False".to_string(), Type::Bool);
    info.exports.insert("Ellipsis".to_string(), Type::Any);
    info.exports.insert("NotImplemented".to_string(), Type::Any);

    info
}

/// Create typing module stub
fn create_typing_stub() -> ModuleInfo {
    let mut info = ModuleInfo::new("typing");

    // Type constructors
    let type_constructors = [
        "List",
        "Dict",
        "Set",
        "FrozenSet",
        "Tuple",
        "Optional",
        "Union",
        "Callable",
        "Type",
        "Sequence",
        "Mapping",
        "MutableMapping",
        "Iterable",
        "Iterator",
        "Generator",
        "Coroutine",
        "AsyncGenerator",
        "AsyncIterator",
        "AsyncIterable",
        "Awaitable",
        "ContextManager",
        "AsyncContextManager",
    ];

    for tc in type_constructors {
        info.exports.insert(
            tc.to_string(),
            Type::ClassType {
                name: tc.to_string(),
                module: Some("typing".to_string()),
            },
        );
    }

    // Special forms
    info.exports.insert("Any".to_string(), Type::Any);
    info.exports.insert("NoReturn".to_string(), Type::Never);
    info.exports.insert("Never".to_string(), Type::Never);

    // TypeVar and related
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

    // Literal and Final
    info.exports.insert(
        "Literal".to_string(),
        Type::ClassType {
            name: "Literal".to_string(),
            module: Some("typing".to_string()),
        },
    );
    info.exports.insert(
        "Final".to_string(),
        Type::ClassType {
            name: "Final".to_string(),
            module: Some("typing".to_string()),
        },
    );

    // TypedDict
    info.exports.insert(
        "TypedDict".to_string(),
        Type::ClassType {
            name: "TypedDict".to_string(),
            module: Some("typing".to_string()),
        },
    );

    // Other typing utilities
    info.exports.insert(
        "ClassVar".to_string(),
        Type::ClassType {
            name: "ClassVar".to_string(),
            module: Some("typing".to_string()),
        },
    );
    info.exports
        .insert("Self".to_string(), Type::SelfType { class_name: None });
    info.exports.insert(
        "TypeAlias".to_string(),
        Type::ClassType {
            name: "TypeAlias".to_string(),
            module: Some("typing".to_string()),
        },
    );

    // PEP 593: Annotated
    info.exports.insert(
        "Annotated".to_string(),
        Type::ClassType {
            name: "Annotated".to_string(),
            module: Some("typing".to_string()),
        },
    );

    // PEP 612: ParamSpec and Concatenate
    info.exports.insert(
        "ParamSpec".to_string(),
        Type::ClassType {
            name: "ParamSpec".to_string(),
            module: Some("typing".to_string()),
        },
    );
    info.exports.insert(
        "Concatenate".to_string(),
        Type::ClassType {
            name: "Concatenate".to_string(),
            module: Some("typing".to_string()),
        },
    );

    // PEP 646: TypeVarTuple and Unpack
    info.exports.insert(
        "TypeVarTuple".to_string(),
        Type::ClassType {
            name: "TypeVarTuple".to_string(),
            module: Some("typing".to_string()),
        },
    );
    info.exports.insert(
        "Unpack".to_string(),
        Type::ClassType {
            name: "Unpack".to_string(),
            module: Some("typing".to_string()),
        },
    );

    // PEP 675: LiteralString
    info.exports
        .insert("LiteralString".to_string(), Type::LiteralString);

    // PEP 647: TypeGuard
    info.exports.insert(
        "TypeGuard".to_string(),
        Type::ClassType {
            name: "TypeGuard".to_string(),
            module: Some("typing".to_string()),
        },
    );

    // PEP 742: TypeIs
    info.exports.insert(
        "TypeIs".to_string(),
        Type::ClassType {
            name: "TypeIs".to_string(),
            module: Some("typing".to_string()),
        },
    );

    // Functions
    info.exports.insert(
        "cast".to_string(),
        Type::callable(vec![Type::Any, Type::Any], Type::Any),
    );
    info.exports.insert(
        "overload".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );
    info.exports.insert(
        "no_type_check".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );
    info.exports.insert(
        "get_type_hints".to_string(),
        Type::callable(vec![Type::Any], Type::dict(Type::Str, Type::Any)),
    );

    info
}

/// Create collections module stub
fn create_collections_stub() -> ModuleInfo {
    let mut info = ModuleInfo::new("collections");

    let types = [
        (
            "deque",
            Type::ClassType {
                name: "deque".to_string(),
                module: Some("collections".to_string()),
            },
        ),
        (
            "defaultdict",
            Type::ClassType {
                name: "defaultdict".to_string(),
                module: Some("collections".to_string()),
            },
        ),
        (
            "OrderedDict",
            Type::ClassType {
                name: "OrderedDict".to_string(),
                module: Some("collections".to_string()),
            },
        ),
        (
            "Counter",
            Type::ClassType {
                name: "Counter".to_string(),
                module: Some("collections".to_string()),
            },
        ),
        (
            "ChainMap",
            Type::ClassType {
                name: "ChainMap".to_string(),
                module: Some("collections".to_string()),
            },
        ),
        (
            "namedtuple",
            Type::ClassType {
                name: "namedtuple".to_string(),
                module: Some("collections".to_string()),
            },
        ),
    ];

    for (name, ty) in types {
        info.exports.insert(name.to_string(), ty);
    }

    info
}

/// Create collections.abc module stub
fn create_collections_abc_stub() -> ModuleInfo {
    let mut info = ModuleInfo::new("collections.abc");

    let abstract_types = [
        "Awaitable",
        "Coroutine",
        "AsyncIterable",
        "AsyncIterator",
        "AsyncGenerator",
        "Hashable",
        "Iterable",
        "Iterator",
        "Generator",
        "Reversible",
        "Container",
        "Collection",
        "Callable",
        "Set",
        "MutableSet",
        "Mapping",
        "MutableMapping",
        "MappingView",
        "KeysView",
        "ItemsView",
        "ValuesView",
        "Sequence",
        "MutableSequence",
        "ByteString",
    ];

    for name in abstract_types {
        info.exports.insert(
            name.to_string(),
            Type::ClassType {
                name: name.to_string(),
                module: Some("collections.abc".to_string()),
            },
        );
    }

    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_loader_builtins() {
        let mut loader = StubLoader::new();
        loader.load_builtins();

        assert!(loader.has_stub("builtins"));
        assert!(loader.has_stub("typing"));

        let builtins = loader.get_stub("builtins").unwrap();
        assert!(builtins.exports.contains_key("int"));
        assert!(builtins.exports.contains_key("str"));
        assert!(builtins.exports.contains_key("len"));
        assert!(builtins.exports.contains_key("Exception"));
    }

    #[test]
    fn test_typing_stub() {
        let mut loader = StubLoader::new();
        loader.load_builtins();

        let typing = loader.get_stub("typing").unwrap();
        assert!(typing.exports.contains_key("List"));
        assert!(typing.exports.contains_key("Optional"));
        assert!(typing.exports.contains_key("TypeVar"));
        assert!(typing.exports.contains_key("Protocol"));
        assert!(typing.exports.contains_key("Any"));
    }

    #[test]
    fn test_collections_stub() {
        let mut loader = StubLoader::new();
        loader.load_builtins();

        let collections = loader.get_stub("collections").unwrap();
        assert!(collections.exports.contains_key("deque"));
        assert!(collections.exports.contains_key("defaultdict"));
        assert!(collections.exports.contains_key("Counter"));
    }

    #[test]
    fn test_stub_path_management() {
        let mut loader = StubLoader::new();

        loader.add_stub_path(PathBuf::from("/usr/lib/python3/stubs"));
        loader.add_stub_path(PathBuf::from("/home/user/.stubs"));

        assert_eq!(loader.stub_paths.len(), 2);

        // Adding same path again shouldn't duplicate
        loader.add_stub_path(PathBuf::from("/usr/lib/python3/stubs"));
        assert_eq!(loader.stub_paths.len(), 2);
    }

    // Phase C tests: Typeshed integration

    #[test]
    fn test_typeshed_os_stub() {
        let mut loader = StubLoader::new();
        loader.load_builtins();

        let os = loader.get_stub("os").unwrap();
        assert!(os.exports.contains_key("getcwd"));
        assert!(os.exports.contains_key("getenv"));
        assert!(os.exports.contains_key("listdir"));
        assert!(os.exports.contains_key("makedirs"));
        assert!(os.exports.contains_key("remove"));
    }

    #[test]
    fn test_typeshed_os_path_stub() {
        let mut loader = StubLoader::new();
        loader.load_builtins();

        let os_path = loader.get_stub("os.path").unwrap();
        assert!(os_path.exports.contains_key("exists"));
        assert!(os_path.exports.contains_key("join"));
        assert!(os_path.exports.contains_key("dirname"));
        assert!(os_path.exports.contains_key("basename"));
        assert!(os_path.exports.contains_key("isfile"));
        assert!(os_path.exports.contains_key("isdir"));
    }

    #[test]
    fn test_typeshed_sys_stub() {
        let mut loader = StubLoader::new();
        loader.load_builtins();

        let sys = loader.get_stub("sys").unwrap();
        assert!(sys.exports.contains_key("argv"));
        assert!(sys.exports.contains_key("path"));
        assert!(sys.exports.contains_key("version"));
        assert!(sys.exports.contains_key("exit"));
    }

    #[test]
    fn test_typeshed_io_stub() {
        let mut loader = StubLoader::new();
        loader.load_builtins();

        let io = loader.get_stub("io").unwrap();
        assert!(io.exports.contains_key("StringIO"));
        assert!(io.exports.contains_key("BytesIO"));
        assert!(io.exports.contains_key("open"));
    }

    #[test]
    fn test_typeshed_json_stub() {
        let mut loader = StubLoader::new();
        loader.load_builtins();

        let json = loader.get_stub("json").unwrap();
        assert!(json.exports.contains_key("loads"));
        assert!(json.exports.contains_key("dumps"));
        assert!(json.exports.contains_key("load"));
        assert!(json.exports.contains_key("dump"));
    }

    #[test]
    fn test_typeshed_pathlib_stub() {
        let mut loader = StubLoader::new();
        loader.load_builtins();

        let pathlib = loader.get_stub("pathlib").unwrap();
        assert!(pathlib.exports.contains_key("Path"));
        assert!(pathlib.exports.contains_key("PurePath"));
    }

    #[test]
    fn test_get_or_load_stub() {
        let mut loader = StubLoader::new();
        loader.load_builtins();

        // Should return existing stub
        let builtins = loader.get_or_load_stub("builtins");
        assert!(builtins.is_some());
        assert!(builtins.unwrap().exports.contains_key("int"));

        // Should return None for non-existent module (no stub paths configured)
        let nonexistent = loader.get_or_load_stub("nonexistent_module");
        assert!(nonexistent.is_none());

        // Should return typeshed stubs
        let os = loader.get_or_load_stub("os");
        assert!(os.is_some());
        assert!(os.unwrap().exports.contains_key("getcwd"));
    }

    #[test]
    fn test_all_typeshed_modules_loaded() {
        let mut loader = StubLoader::new();
        loader.load_builtins();

        // Verify all expected typeshed modules are available
        let expected_modules = [
            "builtins",
            "typing",
            "collections",
            "collections.abc",
            "os",
            "os.path",
            "sys",
            "io",
            "re",
            "json",
            "pathlib",
            "functools",
            "itertools",
            "datetime",
        ];

        for module in expected_modules {
            assert!(
                loader.has_stub(module),
                "Expected module '{}' to be available",
                module
            );
        }
    }

    #[test]
    fn test_typing_advanced_features() {
        let mut loader = StubLoader::new();
        loader.load_builtins();

        let typing = loader.get_stub("typing").unwrap();

        // PEP 612: ParamSpec
        assert!(typing.exports.contains_key("ParamSpec"));
        assert!(typing.exports.contains_key("Concatenate"));

        // PEP 646: TypeVarTuple
        assert!(typing.exports.contains_key("TypeVarTuple"));
        assert!(typing.exports.contains_key("Unpack"));

        // PEP 675: LiteralString
        assert!(typing.exports.contains_key("LiteralString"));

        // PEP 647: TypeGuard
        assert!(typing.exports.contains_key("TypeGuard"));

        // PEP 742: TypeIs
        assert!(typing.exports.contains_key("TypeIs"));

        // Self type
        assert!(typing.exports.contains_key("Self"));

        // Final and Annotated
        assert!(typing.exports.contains_key("Final"));
        assert!(typing.exports.contains_key("Annotated"));
    }
}
