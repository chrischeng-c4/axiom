---
id: libs-compass-src-semantic-symbols-mod-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/semantic/symbols/mod.rs`.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [overview, source, changes]
---

# Standardized libs/compass/src/semantic/symbols/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/semantic/symbols/mod.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SymbolId` | libs/compass/src/semantic/symbols/mod.rs | struct | pub | 29 | pub struct SymbolId(pub usize); |
| `SymbolKind` | libs/compass/src/semantic/symbols/mod.rs | enum | pub | 33 | pub enum SymbolKind { |
| `display_name` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 72 | pub fn display_name(&self) -> &'static str { |
| `TypeInfo` | libs/compass/src/semantic/symbols/mod.rs | enum | pub | 105 | pub enum TypeInfo { |
| `display` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 138 | pub fn display(&self) -> String { |
| `from_python_annotation` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 174 | pub fn from_python_annotation(annotation: &str) -> Self { |
| `from_rust_type` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 228 | pub fn from_rust_type(type_str: &str) -> Self { |
| `Symbol` | libs/compass/src/semantic/symbols/mod.rs | struct | pub | 293 | pub struct Symbol { |
| `hover_content` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 305 | pub fn hover_content(&self, language: Language) -> String { |
| `SymbolReference` | libs/compass/src/semantic/symbols/mod.rs | struct | pub | 388 | pub struct SymbolReference { |
| `SymbolTable` | libs/compass/src/semantic/symbols/mod.rs | struct | pub | 396 | pub struct SymbolTable { |
| `new` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 404 | pub fn new() -> Self { |
| `add_symbol` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 409 | pub fn add_symbol( |
| `add_reference` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 445 | pub fn add_reference(&mut self, symbol_id: SymbolId, location: Range) { |
| `get` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 454 | pub fn get(&self, id: SymbolId) -> Option<&Symbol> { |
| `find_by_name` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 459 | pub fn find_by_name(&self, name: &str) -> Vec<&Symbol> { |
| `find_at_position` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 467 | pub fn find_at_position(&self, line: u32, character: u32) -> Option<&Symbol> { |
| `find_definition_at` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 486 | pub fn find_definition_at(&self, line: u32, character: u32) -> Option<&Symbol> { |
| `find_references_at` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 497 | pub fn find_references_at( |
| `all_symbols` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 523 | pub fn all_symbols(&self) -> &[Symbol] { |
| `all_references` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 528 | pub fn all_references(&self) -> &[SymbolReference] { |
| `SymbolTableBuilder` | libs/compass/src/semantic/symbols/mod.rs | struct | pub | 534 | pub struct SymbolTableBuilder { |
| `build_python` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 552 | pub fn build_python(mut self, file: &ParsedFile) -> SymbolTable { |
| `build_rust` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 558 | pub fn build_rust(mut self, file: &ParsedFile) -> SymbolTable { |
| `build_javascript` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 564 | pub fn build_javascript(self, file: &ParsedFile) -> SymbolTable { |
| `build_typescript` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 569 | pub fn build_typescript(mut self, file: &ParsedFile) -> SymbolTable { |
| `build_go` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 575 | pub fn build_go(mut self, file: &ParsedFile) -> SymbolTable { |
| `build_dockerfile` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 581 | pub fn build_dockerfile(mut self, file: &ParsedFile) -> SymbolTable { |
| `build_dockerfile_from_source` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 588 | pub fn build_dockerfile_from_source(mut self, source: &str) -> SymbolTable { |
| `build_terraform` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 594 | pub fn build_terraform(mut self, file: &ParsedFile) -> SymbolTable { |
| `build_kubernetes` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 600 | pub fn build_kubernetes(mut self, file: &ParsedFile) -> SymbolTable { |
| `build_gitlab_ci` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 606 | pub fn build_gitlab_ci(mut self, file: &ParsedFile) -> SymbolTable { |
| `build_gitlab_ci_from_source` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 613 | pub fn build_gitlab_ci_from_source(mut self, source: &str) -> SymbolTable { |
| `build_markdown` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 619 | pub fn build_markdown(mut self, file: &ParsedFile) -> SymbolTable { |
| `build_markdown_from_source` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 626 | pub fn build_markdown_from_source(mut self, source: &str) -> SymbolTable { |
| `build_mermaid` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 632 | pub fn build_mermaid(mut self, file: &ParsedFile) -> SymbolTable { |
| `build_mermaid_from_source` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 639 | pub fn build_mermaid_from_source(mut self, source: &str) -> SymbolTable { |
| `push_scope` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 644 | pub(crate) fn push_scope(&mut self) { |
| `pop_scope` | libs/compass/src/semantic/symbols/mod.rs | function | pub | 650 | pub(crate) fn pop_scope(&mut self) { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Unified Symbol Table for cross-language semantic analysis
//!
//! Provides a common symbol representation for Python, TypeScript, and Rust.

mod css;
mod dockerfile;
mod gitlab_ci;
mod go;
mod graphql_sym;
mod html;
mod javascript;
mod kubernetes;
mod markdown;
mod mermaid;
mod proto_sym;
mod python;
mod rust;
mod sql_sym;
mod terraform;
mod toml_sym;
mod typescript;

use crate::diagnostic::Range;
use crate::syntax::{Language, ParsedFile};
use std::collections::HashMap;

/// Unique identifier for a symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

/// Kind of symbol (cross-language)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    // Common
    Variable,
    Function,
    Class,
    Parameter,
    Import,
    Module,

    // Python-specific
    TypeAlias,
    Decorator,

    // TypeScript-specific
    Interface,
    TypeParameter,
    Enum,
    EnumMember,

    // Rust-specific
    Struct,
    Trait,
    Impl,
    Macro,
    Const,
    Static,

    // Infrastructure-specific (Dockerfile, Terraform, K8s, CI)
    Resource,
    Stage,
    Job,
    Port,
    Label,
    Selector,
    Template,
}

impl SymbolKind {
    /// Get LSP symbol kind for hover display
    pub fn display_name(&self) -> &'static str {
        match self {
            SymbolKind::Variable => "variable",
            SymbolKind::Function => "function",
            SymbolKind::Class => "class",
            SymbolKind::Parameter => "parameter",
            SymbolKind::Import => "import",
            SymbolKind::Module => "module",
            SymbolKind::TypeAlias => "type alias",
            SymbolKind::Decorator => "decorator",
            SymbolKind::Interface => "interface",
            SymbolKind::TypeParameter => "type parameter",
            SymbolKind::Enum => "enum",
            SymbolKind::EnumMember => "enum member",
            SymbolKind::Struct => "struct",
            SymbolKind::Trait => "trait",
            SymbolKind::Impl => "impl",
            SymbolKind::Macro => "macro",
            SymbolKind::Const => "const",
            SymbolKind::Static => "static",
            SymbolKind::Resource => "resource",
            SymbolKind::Stage => "stage",
            SymbolKind::Job => "job",
            SymbolKind::Port => "port",
            SymbolKind::Label => "label",
            SymbolKind::Selector => "selector",
            SymbolKind::Template => "template",
        }
    }
}

/// Type information (basic)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeInfo {
    /// Primitive types (int, str, bool, etc.)
    Primitive(String),
    /// List/Array type
    List(Box<TypeInfo>),
    /// Dict/Map type
    Dict(Box<TypeInfo>, Box<TypeInfo>),
    /// Optional type
    Optional(Box<TypeInfo>),
    /// Union type
    Union(Vec<TypeInfo>),
    /// Callable/Function type
    Callable {
        params: Vec<TypeInfo>,
        ret: Box<TypeInfo>,
    },
    /// Named type (class, interface, etc.)
    Named(String),
    /// Generic type with parameters
    Generic(String, Vec<TypeInfo>),
    /// Reference type (Rust &T, &mut T)
    Reference(Box<TypeInfo>),
    /// Unknown type
    Unknown,
    /// Any type
    Any,
    /// Error type - placeholder for unresolved expressions in error contexts
    /// Used to prevent cascading errors when the parser encounters syntax errors
    Error,
}

impl TypeInfo {
    /// Format type for display
    pub fn display(&self) -> String {
        match self {
            TypeInfo::Primitive(name) => name.clone(),
            TypeInfo::List(inner) => format!("list[{}]", inner.display()),
            TypeInfo::Dict(key, value) => format!("dict[{}, {}]", key.display(), value.display()),
            TypeInfo::Optional(inner) => format!("{}?", inner.display()),
            TypeInfo::Union(types) => types
                .iter()
                .map(|t| t.display())
                .collect::<Vec<_>>()
                .join(" | "),
            TypeInfo::Callable { params, ret } => {
                let params_str = params
                    .iter()
                    .map(|t| t.display())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({}) -> {}", params_str, ret.display())
            }
            TypeInfo::Named(name) => name.clone(),
            TypeInfo::Generic(name, args) => {
                let args_str = args
                    .iter()
                    .map(|t| t.display())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", name, args_str)
            }
            TypeInfo::Reference(inner) => format!("&{}", inner.display()),
            TypeInfo::Unknown => "unknown".to_string(),
            TypeInfo::Any => "any".to_string(),
            TypeInfo::Error => "<error>".to_string(),
        }
    }

    /// Parse from Python type annotation string
    pub fn from_python_annotation(annotation: &str) -> Self {
        let annotation = annotation.trim();

        // Handle Optional
        if annotation.starts_with("Optional[") && annotation.ends_with(']') {
            let inner = &annotation[9..annotation.len() - 1];
            return TypeInfo::Optional(Box::new(Self::from_python_annotation(inner)));
        }

        // Handle List
        if annotation.starts_with("List[") && annotation.ends_with(']') {
            let inner = &annotation[5..annotation.len() - 1];
            return TypeInfo::List(Box::new(Self::from_python_annotation(inner)));
        }
        if annotation.starts_with("list[") && annotation.ends_with(']') {
            let inner = &annotation[5..annotation.len() - 1];
            return TypeInfo::List(Box::new(Self::from_python_annotation(inner)));
        }

        // Handle Dict
        if (annotation.starts_with("Dict[") || annotation.starts_with("dict["))
            && annotation.ends_with(']')
        {
            let inner = &annotation[5..annotation.len() - 1];
            if let Some((key, value)) = inner.split_once(',') {
                return TypeInfo::Dict(
                    Box::new(Self::from_python_annotation(key.trim())),
                    Box::new(Self::from_python_annotation(value.trim())),
                );
            }
        }

        // Handle Union with |
        if annotation.contains(" | ") {
            let types: Vec<_> = annotation
                .split(" | ")
                .map(|t| Self::from_python_annotation(t.trim()))
                .collect();
            return TypeInfo::Union(types);
        }

        // Handle primitives
        match annotation {
            "int" => TypeInfo::Primitive("int".to_string()),
            "str" => TypeInfo::Primitive("str".to_string()),
            "bool" => TypeInfo::Primitive("bool".to_string()),
            "float" => TypeInfo::Primitive("float".to_string()),
            "None" => TypeInfo::Primitive("None".to_string()),
            "Any" => TypeInfo::Any,
            _ => TypeInfo::Named(annotation.to_string()),
        }
    }

    /// Parse from Rust type annotation string
    pub fn from_rust_type(type_str: &str) -> Self {
        let type_str = type_str.trim();

        if type_str.is_empty() {
            return TypeInfo::Unknown;
        }

        // Handle references
        if let Some(inner) = type_str.strip_prefix("&mut ") {
            return TypeInfo::Reference(Box::new(Self::from_rust_type(inner)));
        }
        if let Some(inner) = type_str.strip_prefix('&') {
            return TypeInfo::Reference(Box::new(Self::from_rust_type(inner)));
        }

        // Handle Option<T>
        if type_str.starts_with("Option<") && type_str.ends_with('>') {
            let inner = &type_str[7..type_str.len() - 1];
            return TypeInfo::Optional(Box::new(Self::from_rust_type(inner)));
        }

        // Handle Vec<T>
        if type_str.starts_with("Vec<") && type_str.ends_with('>') {
            let inner = &type_str[4..type_str.len() - 1];
            return TypeInfo::List(Box::new(Self::from_rust_type(inner)));
        }

        // Handle HashMap<K, V>
        if type_str.starts_with("HashMap<") && type_str.ends_with('>') {
            let inner = &type_str[8..type_str.len() - 1];
            if let Some((key, value)) = inner.split_once(',') {
                return TypeInfo::Dict(
                    Box::new(Self::from_rust_type(key.trim())),
                    Box::new(Self::from_rust_type(value.trim())),
                );
            }
        }

        // Handle Result<T, E> and other generics with <>
        if let Some(lt_pos) = type_str.find('<') {
            if type_str.ends_with('>') {
                let name = &type_str[..lt_pos];
                let inner = &type_str[lt_pos + 1..type_str.len() - 1];
                let args: Vec<TypeInfo> = inner
                    .split(',')
                    .map(|t| Self::from_rust_type(t.trim()))
                    .collect();
                return TypeInfo::Generic(name.to_string(), args);
            }
        }

        // Handle Rust primitives
        match type_str {
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
            | "u128" | "usize" | "f32" | "f64" | "bool" | "char" | "str" | "()" => {
                TypeInfo::Primitive(type_str.to_string())
            }
            "String" => TypeInfo::Named("String".to_string()),
            _ => TypeInfo::Named(type_str.to_string()),
        }
    }
}

/// A symbol in the symbol table
#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub location: Range,
    pub type_info: Option<TypeInfo>,
    pub doc: Option<String>,
    pub scope_id: usize,
}

impl Symbol {
    /// Generate hover content for this symbol
    pub fn hover_content(&self, language: Language) -> String {
        let mut content = String::new();

        // Add code block with symbol signature
        let lang_str = language.as_str();

        content.push_str(&format!("```{}\n", lang_str));

        match self.kind {
            SymbolKind::Function => {
                if let Some(ref type_info) = self.type_info {
                    if language == Language::Rust {
                        content.push_str(&format!(
                            "fn {}(...) -> {}\n",
                            self.name,
                            type_info.display()
                        ));
                    } else {
                        content.push_str(&format!(
                            "def {}(...) -> {}\n",
                            self.name,
                            type_info.display()
                        ));
                    }
                } else if language == Language::Rust {
                    content.push_str(&format!("fn {}(...)\n", self.name));
                } else {
                    content.push_str(&format!("def {}(...)\n", self.name));
                }
            }
            SymbolKind::Struct => {
                content.push_str(&format!("struct {}\n", self.name));
            }
            SymbolKind::Trait => {
                content.push_str(&format!("trait {}\n", self.name));
            }
            SymbolKind::Impl => {
                content.push_str(&format!("impl {}\n", self.name));
            }
            SymbolKind::Enum => {
                content.push_str(&format!("enum {}\n", self.name));
            }
            SymbolKind::Class => {
                content.push_str(&format!("class {}\n", self.name));
            }
            SymbolKind::Variable | SymbolKind::Parameter => {
                if let Some(ref type_info) = self.type_info {
                    content.push_str(&format!("{}: {}\n", self.name, type_info.display()));
                } else {
                    content.push_str(&format!("{}\n", self.name));
                }
            }
            SymbolKind::Const | SymbolKind::Static => {
                if let Some(ref type_info) = self.type_info {
                    content.push_str(&format!(
                        "{} {}: {}\n",
                        self.kind.display_name(),
                        self.name,
                        type_info.display()
                    ));
                } else {
                    content.push_str(&format!("{} {}\n", self.kind.display_name(), self.name));
                }
            }
            _ => {
                content.push_str(&format!("{} {}\n", self.kind.display_name(), self.name));
            }
        }

        content.push_str("```\n");

        // Add documentation if available
        if let Some(ref doc) = self.doc {
            content.push_str("\n---\n\n");
            content.push_str(doc);
        }

        content
    }
}

/// Reference to a symbol
#[derive(Debug, Clone)]
pub struct SymbolReference {
    pub symbol_id: SymbolId,
    pub location: Range,
    pub is_definition: bool,
}

/// Symbol table for a file
#[derive(Debug, Default)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
    by_name: HashMap<String, Vec<SymbolId>>,
    references: Vec<SymbolReference>,
    next_id: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a symbol to the table
    pub fn add_symbol(
        &mut self,
        name: String,
        kind: SymbolKind,
        location: Range,
        type_info: Option<TypeInfo>,
        doc: Option<String>,
        scope_id: usize,
    ) -> SymbolId {
        let id = SymbolId(self.next_id);
        self.next_id += 1;

        let symbol = Symbol {
            id,
            name: name.clone(),
            kind,
            location: location.clone(),
            type_info,
            doc,
            scope_id,
        };

        self.symbols.push(symbol);
        self.by_name.entry(name).or_default().push(id);

        // Add definition reference
        self.references.push(SymbolReference {
            symbol_id: id,
            location,
            is_definition: true,
        });

        id
    }

    /// Add a reference to a symbol
    pub fn add_reference(&mut self, symbol_id: SymbolId, location: Range) {
        self.references.push(SymbolReference {
            symbol_id,
            location,
            is_definition: false,
        });
    }

    /// Get symbol by ID
    pub fn get(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id.0)
    }

    /// Find symbols by name
    pub fn find_by_name(&self, name: &str) -> Vec<&Symbol> {
        self.by_name
            .get(name)
            .map(|ids| ids.iter().filter_map(|id| self.get(*id)).collect())
            .unwrap_or_default()
    }

    /// Find symbol at position
    pub fn find_at_position(&self, line: u32, character: u32) -> Option<&Symbol> {
        // First check references (more precise)
        for reference in &self.references {
            if reference.location.contains(line, character) {
                return self.get(reference.symbol_id);
            }
        }

        // Then check symbol definitions
        for symbol in &self.symbols {
            if symbol.location.contains(line, character) {
                return Some(symbol);
            }
        }

        None
    }

    /// Find definition of symbol at position
    pub fn find_definition_at(&self, line: u32, character: u32) -> Option<&Symbol> {
        // Find what's at position
        for reference in &self.references {
            if reference.location.contains(line, character) {
                return self.get(reference.symbol_id);
            }
        }
        None
    }

    /// Find all references to symbol at position
    pub fn find_references_at(
        &self,
        line: u32,
        character: u32,
        include_definition: bool,
    ) -> Vec<Range> {
        // Find the symbol at position
        let symbol_id = self
            .references
            .iter()
            .find(|r| r.location.contains(line, character))
            .map(|r| r.symbol_id);

        let Some(id) = symbol_id else {
            return Vec::new();
        };

        // Find all references to this symbol
        self.references
            .iter()
            .filter(|r| r.symbol_id == id && (include_definition || !r.is_definition))
            .map(|r| r.location.clone())
            .collect()
    }

    /// Get all symbols
    pub fn all_symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    /// Get all references (definitions + usages)
    pub fn all_references(&self) -> &[SymbolReference] {
        &self.references
    }
}

/// Build symbol table from parsed file
pub struct SymbolTableBuilder {
    pub(crate) table: SymbolTable,
    pub(crate) current_scope: usize,
    pub(crate) scope_stack: Vec<usize>,
    pub(crate) next_scope: usize,
}

impl SymbolTableBuilder {
    pub fn new() -> Self {
        Self {
            table: SymbolTable::new(),
            current_scope: 0,
            scope_stack: vec![0],
            next_scope: 1,
        }
    }

    /// Build symbol table for a Python file
    pub fn build_python(mut self, file: &ParsedFile) -> SymbolTable {
        self.visit_python_node(&file.root_node(), file);
        self.table
    }

    /// Build symbol table for a Rust file
    pub fn build_rust(mut self, file: &ParsedFile) -> SymbolTable {
        self.visit_rust_node(&file.root_node(), file);
        self.table
    }

    /// Build symbol table for a JavaScript file (delegates to TypeScript)
    pub fn build_javascript(self, file: &ParsedFile) -> SymbolTable {
        self.build_typescript(file)
    }

    /// Build symbol table for a TypeScript file
    pub fn build_typescript(mut self, file: &ParsedFile) -> SymbolTable {
        self.visit_typescript_node(&file.root_node(), file);
        self.table
    }

    /// Build symbol table for a Go file
    pub fn build_go(mut self, file: &ParsedFile) -> SymbolTable {
        self.visit_go_node(&file.root_node(), file);
        self.table
    }

    /// Build symbol table for a Dockerfile (line-based)
    pub fn build_dockerfile(mut self, file: &ParsedFile) -> SymbolTable {
        self.visit_dockerfile_lines(&file.source);
        self.table
    }

    /// Build symbol table for Dockerfile from raw source (test helper)
    #[cfg(test)]
    pub fn build_dockerfile_from_source(mut self, source: &str) -> SymbolTable {
        self.visit_dockerfile_lines(source);
        self.table
    }

    /// Build symbol table for Terraform/HCL files
    pub fn build_terraform(mut self, file: &ParsedFile) -> SymbolTable {
        self.visit_hcl_node(&file.root_node(), file);
        self.table
    }

    /// Build symbol table for Kubernetes YAML manifests
    pub fn build_kubernetes(mut self, file: &ParsedFile) -> SymbolTable {
        self.visit_k8s_node(&file.root_node(), file);
        self.table
    }

    /// Build symbol table for GitLab CI YAML
    pub fn build_gitlab_ci(mut self, file: &ParsedFile) -> SymbolTable {
        self.visit_gitlab_ci_lines(&file.source);
        self.table
    }

    /// Build symbol table for GitLab CI from raw source (test helper)
    #[cfg(test)]
    pub fn build_gitlab_ci_from_source(mut self, source: &str) -> SymbolTable {
        self.visit_gitlab_ci_lines(source);
        self.table
    }

    /// Build symbol table for a Markdown file (line-based)
    pub fn build_markdown(mut self, file: &ParsedFile) -> SymbolTable {
        self.visit_markdown_lines(&file.source);
        self.table
    }

    /// Build symbol table for Markdown from raw source (test helper)
    #[cfg(test)]
    pub fn build_markdown_from_source(mut self, source: &str) -> SymbolTable {
        self.visit_markdown_lines(source);
        self.table
    }

    /// Build symbol table for a Mermaid diagram file (line-based)
    pub fn build_mermaid(mut self, file: &ParsedFile) -> SymbolTable {
        self.visit_mermaid_lines(&file.source);
        self.table
    }

    /// Build symbol table for Mermaid from raw source (test helper)
    #[cfg(test)]
    pub fn build_mermaid_from_source(mut self, source: &str) -> SymbolTable {
        self.visit_mermaid_lines(source);
        self.table
    }

    pub(crate) fn push_scope(&mut self) {
        self.scope_stack.push(self.current_scope);
        self.current_scope = self.next_scope;
        self.next_scope += 1;
    }

    pub(crate) fn pop_scope(&mut self) {
        if let Some(parent) = self.scope_stack.pop() {
            self.current_scope = parent;
        }
    }
}

impl Default for SymbolTableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_info_display() {
        assert_eq!(TypeInfo::Primitive("int".to_string()).display(), "int");
        assert_eq!(
            TypeInfo::List(Box::new(TypeInfo::Primitive("str".to_string()))).display(),
            "list[str]"
        );
        assert_eq!(
            TypeInfo::Optional(Box::new(TypeInfo::Primitive("int".to_string()))).display(),
            "int?"
        );
    }

    #[test]
    fn test_type_info_from_annotation() {
        assert_eq!(
            TypeInfo::from_python_annotation("int"),
            TypeInfo::Primitive("int".to_string())
        );
        assert_eq!(
            TypeInfo::from_python_annotation("List[str]"),
            TypeInfo::List(Box::new(TypeInfo::Primitive("str".to_string())))
        );
        assert_eq!(
            TypeInfo::from_python_annotation("Optional[int]"),
            TypeInfo::Optional(Box::new(TypeInfo::Primitive("int".to_string())))
        );
    }

    #[test]
    fn test_rust_type_parsing() {
        assert_eq!(
            TypeInfo::from_rust_type("i32"),
            TypeInfo::Primitive("i32".to_string())
        );
        assert_eq!(
            TypeInfo::from_rust_type("&str"),
            TypeInfo::Reference(Box::new(TypeInfo::Primitive("str".to_string())))
        );
        assert_eq!(
            TypeInfo::from_rust_type("Option<String>"),
            TypeInfo::Optional(Box::new(TypeInfo::Named("String".to_string())))
        );
        assert_eq!(
            TypeInfo::from_rust_type("Vec<i32>"),
            TypeInfo::List(Box::new(TypeInfo::Primitive("i32".to_string())))
        );
        assert_eq!(
            TypeInfo::from_rust_type("Result<String, Error>"),
            TypeInfo::Generic(
                "Result".to_string(),
                vec![
                    TypeInfo::Named("String".to_string()),
                    TypeInfo::Named("Error".to_string()),
                ]
            )
        );
    }

    #[test]
    fn test_generic_display_uses_angle_brackets() {
        let ty = TypeInfo::Generic(
            "Result".to_string(),
            vec![
                TypeInfo::Named("String".to_string()),
                TypeInfo::Named("Error".to_string()),
            ],
        );
        assert_eq!(ty.display(), "Result<String, Error>");
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/semantic/symbols/mod.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/semantic/symbols/mod.rs` captured during libs codegen standardization.
```
