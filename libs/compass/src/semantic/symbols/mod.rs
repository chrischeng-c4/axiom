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
