---
id: libs-compass-src-type-inference-model-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/type_inference/model.rs`.
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

# Standardized libs/compass/src/type_inference/model.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/type_inference/model.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SymbolId` | libs/compass/src/type_inference/model.rs | struct | pub | 18 | pub struct SymbolId(pub u64); |
| `new` | libs/compass/src/type_inference/model.rs | function | pub | 22 | pub fn new(id: u64) -> Self { |
| `ScopeId` | libs/compass/src/type_inference/model.rs | struct | pub | 29 | pub struct ScopeId(pub u64); |
| `SemanticSymbolKind` | libs/compass/src/type_inference/model.rs | enum | pub | 40 | pub enum SemanticSymbolKind { |
| `display_name` | libs/compass/src/type_inference/model.rs | function | pub | 55 | pub fn display_name(&self) -> &'static str { |
| `SymbolData` | libs/compass/src/type_inference/model.rs | struct | pub | 73 | pub struct SymbolData { |
| `TypeInfo` | libs/compass/src/type_inference/model.rs | enum | pub | 94 | pub enum TypeInfo { |
| `from_type` | libs/compass/src/type_inference/model.rs | function | pub | 142 | pub fn from_type(ty: &Type) -> Self { |
| `display` | libs/compass/src/type_inference/model.rs | function | pub | 186 | pub fn display(&self) -> String { |
| `is_unknown` | libs/compass/src/type_inference/model.rs | function | pub | 250 | pub fn is_unknown(&self) -> bool { |
| `ParamInfo` | libs/compass/src/type_inference/model.rs | struct | pub | 257 | pub struct ParamInfo { |
| `from_param` | libs/compass/src/type_inference/model.rs | function | pub | 272 | pub fn from_param(param: &super::ty::Param) -> Self { |
| `LiteralInfo` | libs/compass/src/type_inference/model.rs | enum | pub | 285 | pub enum LiteralInfo { |
| `from_literal` | libs/compass/src/type_inference/model.rs | function | pub | 295 | pub fn from_literal(lit: &super::ty::LiteralValue) -> Self { |
| `SymbolReference` | libs/compass/src/type_inference/model.rs | struct | pub | 319 | pub struct SymbolReference { |
| `ScopeInfo` | libs/compass/src/type_inference/model.rs | struct | pub | 330 | pub struct ScopeInfo { |
| `TypedRange` | libs/compass/src/type_inference/model.rs | struct | pub | 343 | pub struct TypedRange { |
| `SemanticModel` | libs/compass/src/type_inference/model.rs | struct | pub | 357 | pub struct SemanticModel { |
| `alloc_symbol_id` | libs/compass/src/type_inference/model.rs | function | pub | 389 | pub fn alloc_symbol_id(&mut self) -> SymbolId { |
| `alloc_scope_id` | libs/compass/src/type_inference/model.rs | function | pub | 396 | pub fn alloc_scope_id(&mut self) -> ScopeId { |
| `add_symbol` | libs/compass/src/type_inference/model.rs | function | pub | 403 | pub fn add_symbol(&mut self, data: SymbolData) -> SymbolId { |
| `add_reference` | libs/compass/src/type_inference/model.rs | function | pub | 423 | pub fn add_reference(&mut self, symbol_id: SymbolId, range: Range) { |
| `add_scope` | libs/compass/src/type_inference/model.rs | function | pub | 432 | pub fn add_scope(&mut self, parent: Option<ScopeId>, range: Range) -> ScopeId { |
| `add_typed_range` | libs/compass/src/type_inference/model.rs | function | pub | 447 | pub fn add_typed_range( |
| `finalize` | libs/compass/src/type_inference/model.rs | function | pub | 461 | pub fn finalize(&mut self) { |
| `type_at` | libs/compass/src/type_inference/model.rs | function | pub | 473 | pub fn type_at(&self, line: u32, column: u32) -> Option<&TypeInfo> { |
| `symbol_at` | libs/compass/src/type_inference/model.rs | function | pub | 484 | pub fn symbol_at(&self, line: u32, column: u32) -> Option<&SymbolData> { |
| `definition_at` | libs/compass/src/type_inference/model.rs | function | pub | 503 | pub fn definition_at(&self, line: u32, column: u32) -> Option<&SymbolData> { |
| `references_at` | libs/compass/src/type_inference/model.rs | function | pub | 514 | pub fn references_at( |
| `symbols_by_name` | libs/compass/src/type_inference/model.rs | function | pub | 539 | pub fn symbols_by_name(&self, name: &str) -> Vec<&SymbolData> { |
| `all_symbols` | libs/compass/src/type_inference/model.rs | function | pub | 547 | pub fn all_symbols(&self) -> impl Iterator<Item = &SymbolData> { |
| `hover_at` | libs/compass/src/type_inference/model.rs | function | pub | 552 | pub fn hover_at(&self, line: u32, column: u32) -> Option<String> { |
| `symbol_count` | libs/compass/src/type_inference/model.rs | function | pub | 599 | pub fn symbol_count(&self) -> usize { |
| `merge` | libs/compass/src/type_inference/model.rs | function | pub | 604 | pub fn merge(&mut self, other: SemanticModel) { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Semantic Model - Owned, serializable type information independent of AST
//!
//! The SemanticModel provides a persistent representation of resolved types,
//! symbols, and references that can be cached and queried without access to
//! the original source code or AST.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::diagnostic::Range;

use super::ty::Type;

/// Unique identifier for a symbol within the semantic model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SymbolId(pub u64);

impl SymbolId {
    /// Create a new symbol ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Unique identifier for a scope within the semantic model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScopeId(pub u64);

impl ScopeId {
    /// Create a new scope ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Kind of symbol in the semantic model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticSymbolKind {
    Variable,
    Function,
    Class,
    Parameter,
    Import,
    Module,
    TypeAlias,
    Attribute,
    Method,
    Property,
}

impl SemanticSymbolKind {
    /// Get a display name for the symbol kind
    pub fn display_name(&self) -> &'static str {
        match self {
            SemanticSymbolKind::Variable => "variable",
            SemanticSymbolKind::Function => "function",
            SemanticSymbolKind::Class => "class",
            SemanticSymbolKind::Parameter => "parameter",
            SemanticSymbolKind::Import => "import",
            SemanticSymbolKind::Module => "module",
            SemanticSymbolKind::TypeAlias => "type alias",
            SemanticSymbolKind::Attribute => "attribute",
            SemanticSymbolKind::Method => "method",
            SemanticSymbolKind::Property => "property",
        }
    }
}

/// Symbol data stored in the semantic model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolData {
    /// Symbol name
    pub name: String,
    /// Kind of symbol
    pub kind: SemanticSymbolKind,
    /// Definition range (where the symbol is defined)
    pub def_range: Range,
    /// File path where the symbol is defined
    pub file_path: PathBuf,
    /// Type information for this symbol
    pub type_info: TypeInfo,
    /// Documentation string if available
    pub documentation: Option<String>,
    /// Scope this symbol belongs to
    pub scope_id: ScopeId,
    /// Parent symbol (for methods/attributes of a class)
    pub parent_id: Option<SymbolId>,
}

/// Owned type information that can be serialized
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeInfo {
    /// Unknown type (not yet inferred)
    Unknown,
    /// Any type
    Any,
    /// None type
    None,
    /// Boolean type
    Bool,
    /// Integer type
    Int,
    /// Float type
    Float,
    /// String type
    Str,
    /// Bytes type
    Bytes,
    /// List type with element type
    List(Box<TypeInfo>),
    /// Set type with element type
    Set(Box<TypeInfo>),
    /// Dict type with key and value types
    Dict(Box<TypeInfo>, Box<TypeInfo>),
    /// Tuple type with element types
    Tuple(Vec<TypeInfo>),
    /// Optional type (T | None)
    Optional(Box<TypeInfo>),
    /// Union of types
    Union(Vec<TypeInfo>),
    /// Callable type
    Callable {
        params: Vec<ParamInfo>,
        return_type: Box<TypeInfo>,
    },
    /// Instance of a class
    Instance {
        name: String,
        module: Option<String>,
        type_args: Vec<TypeInfo>,
    },
    /// Literal type
    Literal(LiteralInfo),
    /// Error type (for error recovery)
    Error,
}

impl TypeInfo {
    /// Convert from the type checker's Type to owned TypeInfo
    pub fn from_type(ty: &Type) -> Self {
        match ty {
            Type::Unknown => TypeInfo::Unknown,
            Type::Any => TypeInfo::Any,
            Type::None => TypeInfo::None,
            Type::Bool => TypeInfo::Bool,
            Type::Int => TypeInfo::Int,
            Type::Float => TypeInfo::Float,
            Type::Str => TypeInfo::Str,
            Type::Bytes => TypeInfo::Bytes,
            Type::List(inner) => TypeInfo::List(Box::new(TypeInfo::from_type(inner))),
            Type::Set(inner) => TypeInfo::Set(Box::new(TypeInfo::from_type(inner))),
            Type::Dict(key, value) => TypeInfo::Dict(
                Box::new(TypeInfo::from_type(key)),
                Box::new(TypeInfo::from_type(value)),
            ),
            Type::Tuple(elems) => TypeInfo::Tuple(elems.iter().map(TypeInfo::from_type).collect()),
            Type::Optional(inner) => TypeInfo::Optional(Box::new(TypeInfo::from_type(inner))),
            Type::Union(types) => TypeInfo::Union(types.iter().map(TypeInfo::from_type).collect()),
            Type::Callable { params, ret } => TypeInfo::Callable {
                params: params.iter().map(ParamInfo::from_param).collect(),
                return_type: Box::new(TypeInfo::from_type(ret)),
            },
            Type::Instance {
                name,
                module,
                type_args,
            } => TypeInfo::Instance {
                name: name.clone(),
                module: module.clone(),
                type_args: type_args.iter().map(TypeInfo::from_type).collect(),
            },
            Type::Literal(lit) => TypeInfo::Literal(LiteralInfo::from_literal(lit)),
            Type::Error => TypeInfo::Error,
            // Handle other types by converting to string representation
            _ => TypeInfo::Instance {
                name: format!("{}", ty),
                module: None,
                type_args: vec![],
            },
        }
    }

    /// Display the type as a string
    pub fn display(&self) -> String {
        match self {
            TypeInfo::Unknown => "Unknown".to_string(),
            TypeInfo::Any => "Any".to_string(),
            TypeInfo::None => "None".to_string(),
            TypeInfo::Bool => "bool".to_string(),
            TypeInfo::Int => "int".to_string(),
            TypeInfo::Float => "float".to_string(),
            TypeInfo::Str => "str".to_string(),
            TypeInfo::Bytes => "bytes".to_string(),
            TypeInfo::List(inner) => format!("list[{}]", inner.display()),
            TypeInfo::Set(inner) => format!("set[{}]", inner.display()),
            TypeInfo::Dict(key, value) => format!("dict[{}, {}]", key.display(), value.display()),
            TypeInfo::Tuple(elems) => {
                let inner = elems
                    .iter()
                    .map(|t| t.display())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("tuple[{}]", inner)
            }
            TypeInfo::Optional(inner) => format!("{} | None", inner.display()),
            TypeInfo::Union(types) => types
                .iter()
                .map(|t| t.display())
                .collect::<Vec<_>>()
                .join(" | "),
            TypeInfo::Callable {
                params,
                return_type,
            } => {
                let param_str = params
                    .iter()
                    .map(|p| {
                        if p.name.is_empty() {
                            p.type_info.display()
                        } else {
                            format!("{}: {}", p.name, p.type_info.display())
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({}) -> {}", param_str, return_type.display())
            }
            TypeInfo::Instance {
                name, type_args, ..
            } => {
                if type_args.is_empty() {
                    name.clone()
                } else {
                    let args = type_args
                        .iter()
                        .map(|t| t.display())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}[{}]", name, args)
                }
            }
            TypeInfo::Literal(lit) => lit.display(),
            TypeInfo::Error => "<error>".to_string(),
        }
    }

    /// Check if this is an unknown type
    pub fn is_unknown(&self) -> bool {
        matches!(self, TypeInfo::Unknown)
    }
}

/// Parameter information for callable types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParamInfo {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub type_info: TypeInfo,
    /// Whether the parameter has a default value
    pub has_default: bool,
    /// Whether this is a *args parameter
    pub is_variadic: bool,
    /// Whether this is a **kwargs parameter
    pub is_keyword: bool,
}

impl ParamInfo {
    /// Convert from the type checker's Param to owned ParamInfo
    pub fn from_param(param: &super::ty::Param) -> Self {
        Self {
            name: param.name.clone(),
            type_info: TypeInfo::from_type(&param.ty),
            has_default: param.has_default,
            is_variadic: matches!(param.kind, super::ty::ParamKind::VarPositional),
            is_keyword: matches!(param.kind, super::ty::ParamKind::VarKeyword),
        }
    }
}

/// Literal value information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiteralInfo {
    Int(i64),
    Float(String), // Store as string to preserve exact representation
    Str(String),
    Bool(bool),
    None,
}

impl LiteralInfo {
    /// Convert from the type checker's LiteralValue
    pub fn from_literal(lit: &super::ty::LiteralValue) -> Self {
        match lit {
            super::ty::LiteralValue::Int(i) => LiteralInfo::Int(*i),
            super::ty::LiteralValue::Float(f) => LiteralInfo::Float(f.to_string()),
            super::ty::LiteralValue::Str(s) => LiteralInfo::Str(s.clone()),
            super::ty::LiteralValue::Bool(b) => LiteralInfo::Bool(*b),
            super::ty::LiteralValue::None => LiteralInfo::None,
        }
    }

    /// Display the literal value
    pub fn display(&self) -> String {
        match self {
            LiteralInfo::Int(i) => format!("Literal[{}]", i),
            LiteralInfo::Float(f) => format!("Literal[{}]", f),
            LiteralInfo::Str(s) => format!("Literal[\"{}\"]", s),
            LiteralInfo::Bool(b) => format!("Literal[{}]", b),
            LiteralInfo::None => "Literal[None]".to_string(),
        }
    }
}

/// Reference to a symbol (usage site)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolReference {
    /// The symbol being referenced
    pub symbol_id: SymbolId,
    /// Range of the reference
    pub range: Range,
    /// Whether this reference is the definition
    pub is_definition: bool,
}

/// Scope information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeInfo {
    /// Scope ID
    pub id: ScopeId,
    /// Parent scope (None for module scope)
    pub parent: Option<ScopeId>,
    /// Range of the scope in source code
    pub range: Range,
    /// Symbols defined in this scope
    pub symbols: Vec<SymbolId>,
}

/// An interval in the source code that maps to type/symbol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedRange {
    /// Range in the source code
    pub range: Range,
    /// Type at this range
    pub type_info: TypeInfo,
    /// Symbol ID if this range corresponds to a symbol
    pub symbol_id: Option<SymbolId>,
}

/// The main Semantic Model structure
///
/// Stores resolved types, symbols, and references independent of the AST.
/// This can be serialized and cached for fast retrieval.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SemanticModel {
    /// All symbols indexed by their ID
    pub symbols: HashMap<SymbolId, SymbolData>,
    /// All references to symbols
    pub references: Vec<SymbolReference>,
    /// Scope information
    pub scopes: HashMap<ScopeId, ScopeInfo>,
    /// Type information for ranges (sorted by start position for binary search)
    pub typed_ranges: Vec<TypedRange>,
    /// Symbol lookup by name within scopes
    pub name_to_symbols: HashMap<String, Vec<SymbolId>>,
    /// Next available symbol ID
    next_symbol_id: u64,
    /// Next available scope ID
    next_scope_id: u64,
}

impl SemanticModel {
    /// Create a new empty semantic model
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            references: Vec::new(),
            scopes: HashMap::new(),
            typed_ranges: Vec::new(),
            name_to_symbols: HashMap::new(),
            next_symbol_id: 0,
            next_scope_id: 0,
        }
    }

    /// Allocate a new symbol ID
    pub fn alloc_symbol_id(&mut self) -> SymbolId {
        let id = SymbolId::new(self.next_symbol_id);
        self.next_symbol_id += 1;
        id
    }

    /// Allocate a new scope ID
    pub fn alloc_scope_id(&mut self) -> ScopeId {
        let id = ScopeId::new(self.next_scope_id);
        self.next_scope_id += 1;
        id
    }

    /// Add a symbol to the model
    pub fn add_symbol(&mut self, data: SymbolData) -> SymbolId {
        let id = self.alloc_symbol_id();
        let name = data.name.clone();

        self.symbols.insert(id, data);
        self.name_to_symbols.entry(name).or_default().push(id);

        // Add definition reference
        if let Some(symbol_data) = self.symbols.get(&id) {
            self.references.push(SymbolReference {
                symbol_id: id,
                range: symbol_data.def_range.clone(),
                is_definition: true,
            });
        }

        id
    }

    /// Add a reference to a symbol
    pub fn add_reference(&mut self, symbol_id: SymbolId, range: Range) {
        self.references.push(SymbolReference {
            symbol_id,
            range,
            is_definition: false,
        });
    }

    /// Add a scope to the model
    pub fn add_scope(&mut self, parent: Option<ScopeId>, range: Range) -> ScopeId {
        let id = self.alloc_scope_id();
        self.scopes.insert(
            id,
            ScopeInfo {
                id,
                parent,
                range,
                symbols: Vec::new(),
            },
        );
        id
    }

    /// Add a typed range to the model
    pub fn add_typed_range(
        &mut self,
        range: Range,
        type_info: TypeInfo,
        symbol_id: Option<SymbolId>,
    ) {
        self.typed_ranges.push(TypedRange {
            range,
            type_info,
            symbol_id,
        });
    }

    /// Sort typed ranges by start position for efficient lookup
    pub fn finalize(&mut self) {
        self.typed_ranges.sort_by(|a, b| {
            let line_cmp = a.range.start.line.cmp(&b.range.start.line);
            if line_cmp == std::cmp::Ordering::Equal {
                a.range.start.character.cmp(&b.range.start.character)
            } else {
                line_cmp
            }
        });
    }

    /// Get type information at a position (line, column)
    pub fn type_at(&self, line: u32, column: u32) -> Option<&TypeInfo> {
        // Binary search for the range containing this position
        for typed_range in &self.typed_ranges {
            if typed_range.range.contains(line, column) {
                return Some(&typed_range.type_info);
            }
        }
        None
    }

    /// Get symbol at a position
    pub fn symbol_at(&self, line: u32, column: u32) -> Option<&SymbolData> {
        // First check references
        for reference in &self.references {
            if reference.range.contains(line, column) {
                return self.symbols.get(&reference.symbol_id);
            }
        }

        // Then check symbol definitions
        for symbol in self.symbols.values() {
            if symbol.def_range.contains(line, column) {
                return Some(symbol);
            }
        }

        None
    }

    /// Get the definition of the symbol at a position
    pub fn definition_at(&self, line: u32, column: u32) -> Option<&SymbolData> {
        // Find reference at position
        for reference in &self.references {
            if reference.range.contains(line, column) {
                return self.symbols.get(&reference.symbol_id);
            }
        }
        None
    }

    /// Find all references to the symbol at a position
    pub fn references_at(
        &self,
        line: u32,
        column: u32,
        include_definition: bool,
    ) -> Vec<&SymbolReference> {
        // Find the symbol at position
        let symbol_id = self
            .references
            .iter()
            .find(|r| r.range.contains(line, column))
            .map(|r| r.symbol_id);

        let Some(id) = symbol_id else {
            return Vec::new();
        };

        // Find all references to this symbol
        self.references
            .iter()
            .filter(|r| r.symbol_id == id && (include_definition || !r.is_definition))
            .collect()
    }

    /// Get symbols by name
    pub fn symbols_by_name(&self, name: &str) -> Vec<&SymbolData> {
        self.name_to_symbols
            .get(name)
            .map(|ids| ids.iter().filter_map(|id| self.symbols.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all symbols
    pub fn all_symbols(&self) -> impl Iterator<Item = &SymbolData> {
        self.symbols.values()
    }

    /// Get hover content for symbol at position
    pub fn hover_at(&self, line: u32, column: u32) -> Option<String> {
        let symbol = self.symbol_at(line, column)?;

        let mut content = String::new();
        content.push_str("```python\n");

        match symbol.kind {
            SemanticSymbolKind::Function | SemanticSymbolKind::Method => {
                content.push_str(&format!(
                    "def {}(...) -> {}\n",
                    symbol.name,
                    symbol.type_info.display()
                ));
            }
            SemanticSymbolKind::Class => {
                content.push_str(&format!("class {}\n", symbol.name));
            }
            SemanticSymbolKind::Variable
            | SemanticSymbolKind::Parameter
            | SemanticSymbolKind::Attribute => {
                content.push_str(&format!(
                    "{}: {}\n",
                    symbol.name,
                    symbol.type_info.display()
                ));
            }
            _ => {
                content.push_str(&format!(
                    "{} {}: {}\n",
                    symbol.kind.display_name(),
                    symbol.name,
                    symbol.type_info.display()
                ));
            }
        }

        content.push_str("```");

        if let Some(ref doc) = symbol.documentation {
            content.push_str("\n\n---\n\n");
            content.push_str(doc);
        }

        Some(content)
    }

    /// Get the number of indexed files (symbols count as proxy)
    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    /// Merge another semantic model into this one
    pub fn merge(&mut self, other: SemanticModel) {
        // Remap IDs from the other model
        let symbol_id_offset = self.next_symbol_id;
        let scope_id_offset = self.next_scope_id;

        for (old_id, mut symbol) in other.symbols {
            let new_id = SymbolId::new(old_id.0 + symbol_id_offset);
            symbol.scope_id = ScopeId::new(symbol.scope_id.0 + scope_id_offset);
            if let Some(ref mut parent) = symbol.parent_id {
                *parent = SymbolId::new(parent.0 + symbol_id_offset);
            }
            self.symbols.insert(new_id, symbol);
        }

        for mut reference in other.references {
            reference.symbol_id = SymbolId::new(reference.symbol_id.0 + symbol_id_offset);
            self.references.push(reference);
        }

        for (old_id, mut scope) in other.scopes {
            let new_id = ScopeId::new(old_id.0 + scope_id_offset);
            scope.id = new_id;
            if let Some(ref mut parent) = scope.parent {
                *parent = ScopeId::new(parent.0 + scope_id_offset);
            }
            for symbol_id in &mut scope.symbols {
                *symbol_id = SymbolId::new(symbol_id.0 + symbol_id_offset);
            }
            self.scopes.insert(new_id, scope);
        }

        for mut typed_range in other.typed_ranges {
            if let Some(ref mut id) = typed_range.symbol_id {
                *id = SymbolId::new(id.0 + symbol_id_offset);
            }
            self.typed_ranges.push(typed_range);
        }

        for (name, ids) in other.name_to_symbols {
            let entry = self.name_to_symbols.entry(name).or_default();
            for id in ids {
                entry.push(SymbolId::new(id.0 + symbol_id_offset));
            }
        }

        self.next_symbol_id += other.next_symbol_id;
        self.next_scope_id += other.next_scope_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_model_basic() {
        let mut model = SemanticModel::new();

        let scope_id = model.add_scope(None, Range::default());
        let symbol_id = model.add_symbol(SymbolData {
            name: "foo".to_string(),
            kind: SemanticSymbolKind::Function,
            def_range: Range {
                start: crate::diagnostic::Position {
                    line: 0,
                    character: 4,
                },
                end: crate::diagnostic::Position {
                    line: 0,
                    character: 7,
                },
            },
            file_path: PathBuf::from("test.py"),
            type_info: TypeInfo::Callable {
                params: vec![],
                return_type: Box::new(TypeInfo::Int),
            },
            documentation: Some("A test function".to_string()),
            scope_id,
            parent_id: None,
        });

        assert!(model.symbols.contains_key(&symbol_id));
        assert_eq!(model.symbols_by_name("foo").len(), 1);
    }

    #[test]
    fn test_type_info_display() {
        assert_eq!(TypeInfo::Int.display(), "int");
        assert_eq!(TypeInfo::Str.display(), "str");
        assert_eq!(
            TypeInfo::List(Box::new(TypeInfo::Int)).display(),
            "list[int]"
        );
        assert_eq!(
            TypeInfo::Dict(Box::new(TypeInfo::Str), Box::new(TypeInfo::Int)).display(),
            "dict[str, int]"
        );
        assert_eq!(
            TypeInfo::Optional(Box::new(TypeInfo::Str)).display(),
            "str | None"
        );
        assert_eq!(
            TypeInfo::Union(vec![TypeInfo::Int, TypeInfo::Str]).display(),
            "int | str"
        );
    }

    #[test]
    fn test_type_at_lookup() {
        let mut model = SemanticModel::new();

        model.add_typed_range(
            Range {
                start: crate::diagnostic::Position {
                    line: 0,
                    character: 0,
                },
                end: crate::diagnostic::Position {
                    line: 0,
                    character: 5,
                },
            },
            TypeInfo::Int,
            None,
        );

        model.add_typed_range(
            Range {
                start: crate::diagnostic::Position {
                    line: 1,
                    character: 0,
                },
                end: crate::diagnostic::Position {
                    line: 1,
                    character: 10,
                },
            },
            TypeInfo::Str,
            None,
        );

        model.finalize();

        assert_eq!(model.type_at(0, 2), Some(&TypeInfo::Int));
        assert_eq!(model.type_at(1, 5), Some(&TypeInfo::Str));
        assert_eq!(model.type_at(2, 0), None);
    }

    #[test]
    fn test_symbol_reference_lookup() {
        let mut model = SemanticModel::new();

        let scope_id = model.add_scope(None, Range::default());
        let symbol_id = model.add_symbol(SymbolData {
            name: "x".to_string(),
            kind: SemanticSymbolKind::Variable,
            def_range: Range {
                start: crate::diagnostic::Position {
                    line: 0,
                    character: 0,
                },
                end: crate::diagnostic::Position {
                    line: 0,
                    character: 1,
                },
            },
            file_path: PathBuf::from("test.py"),
            type_info: TypeInfo::Int,
            documentation: None,
            scope_id,
            parent_id: None,
        });

        // Add a reference at line 2
        model.add_reference(
            symbol_id,
            Range {
                start: crate::diagnostic::Position {
                    line: 2,
                    character: 4,
                },
                end: crate::diagnostic::Position {
                    line: 2,
                    character: 5,
                },
            },
        );

        // Definition lookup should work at definition site
        let def = model.definition_at(0, 0);
        assert!(def.is_some());
        assert_eq!(def.unwrap().name, "x");

        // Definition lookup should work at reference site
        let def = model.definition_at(2, 4);
        assert!(def.is_some());
        assert_eq!(def.unwrap().name, "x");

        // References should be found
        let refs = model.references_at(0, 0, true);
        assert_eq!(refs.len(), 2); // Definition + 1 reference

        let refs = model.references_at(0, 0, false);
        assert_eq!(refs.len(), 1); // Only the reference, not the definition
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/type_inference/model.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/type_inference/model.rs` captured during libs codegen standardization.
```
