//! AST Analysis Module
//!
//! Provides tree-sitter based parsing for supported languages.
//! Extracts modules, functions, structs, and imports from source files.

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/fillback/ast_imports_source.md#source
// CODEGEN-BEGIN
use crate::generate::diagrams::content::logic::{FlowEdge, FlowNode, FlowNodeKind, LogicContent};
use crate::Result;
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Language, Parser, Tree};
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/fillback/ast.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Result of analyzing a codebase.
/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/ast.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisContext {
    /// Parsed modules.
    pub modules: Vec<ModuleInfo>,
    /// Files that were skipped.
    pub skipped_files: Vec<String>,
    /// Language → file-count map.
    pub language_counts: HashMap<String, usize>,
}

/// AST Analyzer using tree-sitter.
/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/ast.md#schema
pub struct AstAnalyzer {
    /// Per-language parsers.
    parsers: HashMap<SupportedLanguage, Parser>,
}

/// An import or dependency relationship.
/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/ast.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    /// Import path.
    pub path: String,
    /// Imported items.
    pub items: Vec<String>,
    /// External dep vs internal.
    pub is_external: bool,
}

/// Parsed module information.
/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/ast.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    /// Module name.
    pub name: String,
    /// Module path.
    pub path: String,
    /// Module language.
    pub language: SupportedLanguage,
    /// Symbols in this module.
    pub symbols: Vec<Symbol>,
    /// Imports.
    pub imports: Vec<Import>,
}

/// Parse error information.
/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/ast.md#schema
#[derive(Debug, Clone)]
pub struct ParseError {
    /// File path.
    pub path: String,
    /// Failure reason.
    pub reason: String,
}

/// A field on a struct symbol.
/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/ast.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StructField {
    /// Field name.
    pub name: String,
    /// Rendered type text (verbatim source slice).
    pub rust_type: String,
    /// Whether the field is pub.
    pub is_public: bool,
}

/// Supported programming languages for AST analysis.
/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/ast.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SupportedLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
}

/// A symbol extracted from source code.
/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/ast.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Symbol {
    /// Symbol name.
    pub name: String,
    /// Symbol kind.
    pub kind: SymbolKind,
    /// Optional signature text.
    pub signature: Option<String>,
    /// Optional doc comment.
    pub doc: Option<String>,
    /// Line number.
    pub line: usize,
    /// Whether the symbol is pub.
    pub is_public: bool,
    /// Populated for SymbolKind::Struct.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<StructField>,
    /// Populated for SymbolKind::Enum.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<String>,
    /// Top-level control flow for functions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logic: Option<LogicContent>,
}

/// Kind of symbol extracted from source code.
/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/ast.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SymbolKind {
    /// Function symbol (default).
    #[default]
    Function,
    /// Struct symbol.
    Struct,
    /// Enum symbol.
    Enum,
    /// Interface symbol.
    Interface,
    /// Class symbol.
    Class,
    /// Module symbol.
    Module,
    /// Constant symbol.
    Constant,
    /// Type alias symbol.
    Type,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/fillback/ast-standardization.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/ast-standardization.md#source
impl SupportedLanguage {
    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(Self::Rust),
            "py" => Some(Self::Python),
            "js" | "jsx" | "mjs" | "cjs" => Some(Self::JavaScript),
            "ts" | "tsx" => Some(Self::TypeScript),
            "go" => Some(Self::Go),
            _ => None,
        }
    }

    /// Get the tree-sitter language for this language
    fn tree_sitter_language(&self) -> Language {
        match self {
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Go => tree_sitter_go::LANGUAGE.into(),
        }
    }

    /// Get the display name for this language
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::Python => "Python",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Go => "Go",
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/ast-standardization.md#source
impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function => write!(f, "function"),
            Self::Struct => write!(f, "struct"),
            Self::Enum => write!(f, "enum"),
            Self::Interface => write!(f, "interface"),
            Self::Class => write!(f, "class"),
            Self::Module => write!(f, "module"),
            Self::Constant => write!(f, "constant"),
            Self::Type => write!(f, "type"),
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/ast-standardization.md#source
impl AnalysisContext {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            skipped_files: Vec::new(),
            language_counts: HashMap::new(),
        }
    }

    /// Get total symbol count across all modules
    pub fn total_symbols(&self) -> usize {
        self.modules.iter().map(|m| m.symbols.len()).sum()
    }

    /// Get all unique external dependencies
    pub fn external_dependencies(&self) -> Vec<String> {
        let mut deps: Vec<String> = self
            .modules
            .iter()
            .flat_map(|m| m.imports.iter())
            .filter(|i| i.is_external)
            .map(|i| i.path.clone())
            .collect();
        deps.sort();
        deps.dedup();
        deps
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/ast-standardization.md#source
impl Default for AnalysisContext {
    fn default() -> Self {
        Self::new()
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/ast-standardization.md#source
impl AstAnalyzer {
    /// Create a new AST analyzer with parsers for all supported languages
    pub fn new() -> Result<Self> {
        let mut parsers = HashMap::new();

        for lang in [
            SupportedLanguage::Rust,
            SupportedLanguage::Python,
            SupportedLanguage::JavaScript,
            SupportedLanguage::TypeScript,
            SupportedLanguage::Go,
        ] {
            let mut parser = Parser::new();
            parser.set_language(&lang.tree_sitter_language())?;
            parsers.insert(lang, parser);
        }

        Ok(Self { parsers })
    }

    /// Parse a single file and extract module information
    pub fn parse_file(
        &mut self,
        path: &Path,
        content: &str,
    ) -> std::result::Result<ModuleInfo, ParseError> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        let language = SupportedLanguage::from_extension(ext).ok_or_else(|| ParseError {
            path: path.display().to_string(),
            reason: format!("Unsupported file extension: {}", ext),
        })?;

        let parser = self.parsers.get_mut(&language).ok_or_else(|| ParseError {
            path: path.display().to_string(),
            reason: format!("No parser for language: {:?}", language),
        })?;

        let tree = parser.parse(content, None).ok_or_else(|| ParseError {
            path: path.display().to_string(),
            reason: "Failed to parse file".to_string(),
        })?;

        let module_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let (symbols, imports) = self.extract_symbols_and_imports(&tree, content, language);

        Ok(ModuleInfo {
            name: module_name,
            path: path.display().to_string(),
            language,
            symbols,
            imports,
        })
    }

    /// Extract symbols and imports from a parsed tree
    fn extract_symbols_and_imports(
        &self,
        tree: &Tree,
        source: &str,
        language: SupportedLanguage,
    ) -> (Vec<Symbol>, Vec<Import>) {
        let mut symbols = Vec::new();
        let mut imports = Vec::new();

        let root = tree.root_node();
        let mut cursor = root.walk();

        // Walk through top-level nodes
        for node in root.children(&mut cursor) {
            match language {
                SupportedLanguage::Rust => {
                    self.extract_rust_node(&node, source, &mut symbols, &mut imports);
                }
                SupportedLanguage::Python => {
                    self.extract_python_node(&node, source, &mut symbols, &mut imports);
                }
                SupportedLanguage::JavaScript | SupportedLanguage::TypeScript => {
                    self.extract_js_node(&node, source, &mut symbols, &mut imports);
                }
                SupportedLanguage::Go => {
                    self.extract_go_node(&node, source, &mut symbols, &mut imports);
                }
            }
        }

        (symbols, imports)
    }

    /// Extract symbols and imports from Rust AST nodes
    fn extract_rust_node(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<Symbol>,
        imports: &mut Vec<Import>,
    ) {
        let kind = node.kind();

        match kind {
            "function_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = self.has_visibility_modifier(node, source);
                    let signature = self.extract_rust_function_signature(node, source);
                    let doc = self.extract_rust_doc_comment(node, source);
                    let logic = self.extract_rust_fn_logic(node, source, &name);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        signature: Some(signature),
                        doc,
                        line: node.start_position().row + 1,
                        is_public,
                        logic,
                        ..Default::default()
                    });
                }
            }
            "struct_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = self.has_visibility_modifier(node, source);
                    let doc = self.extract_rust_doc_comment(node, source);
                    let fields = self.extract_rust_struct_fields(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Struct,
                        signature: None,
                        doc,
                        line: node.start_position().row + 1,
                        is_public,
                        fields,
                        ..Default::default()
                    });
                }
            }
            "enum_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = self.has_visibility_modifier(node, source);
                    let doc = self.extract_rust_doc_comment(node, source);
                    let variants = self.extract_rust_enum_variants(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Enum,
                        signature: None,
                        doc,
                        line: node.start_position().row + 1,
                        is_public,
                        variants,
                        ..Default::default()
                    });
                }
            }
            "impl_item" => {
                // Extract methods from impl blocks
                self.extract_rust_impl_methods(node, source, symbols);
            }
            "use_declaration" => {
                let import = self.extract_rust_use(node, source);
                if let Some(import) = import {
                    imports.push(import);
                }
            }
            "mod_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = self.has_visibility_modifier(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Module,
                        signature: None,
                        doc: None,
                        line: node.start_position().row + 1,
                        is_public,
                        ..Default::default()
                    });
                }
            }
            "type_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = self.has_visibility_modifier(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Type,
                        signature: None,
                        doc: None,
                        line: node.start_position().row + 1,
                        is_public,
                        ..Default::default()
                    });
                }
            }
            "const_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = self.has_visibility_modifier(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Constant,
                        signature: None,
                        doc: None,
                        line: node.start_position().row + 1,
                        is_public,
                        ..Default::default()
                    });
                }
            }
            _ => {}
        }
    }

    /// Extract methods from Rust impl block
    fn extract_rust_impl_methods(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<Symbol>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "declaration_list" {
                let mut inner_cursor = child.walk();
                for item in child.children(&mut inner_cursor) {
                    if item.kind() == "function_item" {
                        if let Some(name_node) = item.child_by_field_name("name") {
                            let name = self.node_text(&name_node, source);
                            let is_public = self.has_visibility_modifier(&item, source);
                            let signature = self.extract_rust_function_signature(&item, source);
                            let doc = self.extract_rust_doc_comment(&item, source);

                            symbols.push(Symbol {
                                name,
                                kind: SymbolKind::Function,
                                signature: Some(signature),
                                doc,
                                line: item.start_position().row + 1,
                                is_public,
                                ..Default::default()
                            });
                        }
                    }
                }
            }
        }
    }

    /// Extract Rust function signature
    /// Scan a `function_item` body for top-level control flow and, if it
    /// contains at least one `if_expression` or `match_expression`,
    /// return a shallow `LogicContent` that captures the branching.
    ///
    /// "Shallow" means nested ifs/matches are represented as a single
    /// Decision node each — deeper recursion would produce noisy graphs
    /// that mostly pick up lexical artifacts. Trivial linear bodies
    /// (no if/match) return `None` so no Logic section is emitted and
    /// the module spec stays clean.
    ///
    /// Returned graph shape:
    /// - `start` (Start): entry point
    /// - one `Decision` node per top-level if/match with the condition
    ///   text as label
    /// - `ok`/`err` Terminal nodes when the branch body contains a
    ///   bare `return` with an obvious ok/error expression
    /// - `process` nodes for non-return branches
    fn extract_rust_fn_logic(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        fn_name: &str,
    ) -> Option<LogicContent> {
        let body = node.child_by_field_name("body")?;
        // Walk one level deep looking for if_expression / match_expression.
        let mut ifs = Vec::new();
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            // Descend past expression_statement wrapper.
            let candidate = if child.kind() == "expression_statement" {
                child.child(0).unwrap_or(child)
            } else {
                child
            };
            if candidate.kind() == "if_expression" || candidate.kind() == "match_expression" {
                ifs.push(candidate);
            }
        }
        if ifs.is_empty() {
            return None;
        }

        let mut nodes: HashMap<String, FlowNode> = HashMap::new();
        let mut edges: Vec<FlowEdge> = Vec::new();
        let logic_id = snake_sanitise(fn_name);

        nodes.insert(
            "start".to_string(),
            FlowNode {
                kind: FlowNodeKind::Start,
                ..Default::default()
            },
        );

        let mut prev = "start".to_string();
        for (idx, if_node) in ifs.iter().enumerate() {
            let dec_id = format!("decision_{}", idx);
            let cond_label = self.if_condition_label(if_node, source);
            nodes.insert(
                dec_id.clone(),
                FlowNode {
                    kind: FlowNodeKind::Decision,
                    label: cond_label,
                    ..Default::default()
                },
            );
            edges.push(FlowEdge {
                from: prev.clone(),
                to: dec_id.clone(),
                label: None,
            });

            // Inspect consequence / alternative to decide terminal vs process.
            let (then_id, then_edge_label) = self.classify_branch(
                if_node.child_by_field_name("consequence").as_ref(),
                source,
                &format!("then_{}", idx),
                &mut nodes,
            );
            let (else_id, else_edge_label) = self.classify_branch(
                if_node.child_by_field_name("alternative").as_ref(),
                source,
                &format!("else_{}", idx),
                &mut nodes,
            );
            edges.push(FlowEdge {
                from: dec_id.clone(),
                to: then_id,
                label: Some(then_edge_label),
            });
            edges.push(FlowEdge {
                from: dec_id.clone(),
                to: else_id,
                label: Some(else_edge_label),
            });

            prev = dec_id;
        }

        Some(LogicContent {
            id: logic_id,
            entry: "start".to_string(),
            nodes,
            edges,
            title: Some(fn_name.to_string()),
        })
    }

    /// Extract the condition expression of an if_expression as a free-form
    /// label, or `None` for match_expression (labelled at the caller).
    fn if_condition_label(&self, if_node: &tree_sitter::Node, source: &str) -> Option<String> {
        if if_node.kind() == "match_expression" {
            return Some("match".to_string());
        }
        let cond = if_node.child_by_field_name("condition")?;
        Some(self.node_text(&cond, source).trim().to_string())
    }

    /// Inspect a branch block (consequence / alternative). If it contains a
    /// top-level `return` with an obvious Ok/Err literal, emit a Terminal
    /// node; otherwise a Process node. The caller has already decided the
    /// node id prefix (`then_<n>` / `else_<n>`).
    ///
    /// Returns `(node_id, edge_label)`.
    fn classify_branch(
        &self,
        branch: Option<&tree_sitter::Node>,
        source: &str,
        id: &str,
        nodes: &mut HashMap<String, FlowNode>,
    ) -> (String, String) {
        let Some(branch) = branch else {
            // Missing else — synthesize a passthrough process.
            nodes.insert(
                id.to_string(),
                FlowNode {
                    kind: FlowNodeKind::Process,
                    label: Some("continue".to_string()),
                    ..Default::default()
                },
            );
            return (id.to_string(), "else".to_string());
        };

        // Walk the branch body (could be a block or a single expression).
        // We only look one level deep for `return_expression` hints.
        let mut has_ok_return = false;
        let mut has_err_return = false;
        let mut first_call_label: Option<String> = None;
        let mut cursor = branch.walk();
        for child in branch.children(&mut cursor) {
            let stmt = if child.kind() == "expression_statement" {
                child.child(0).unwrap_or(child)
            } else {
                child
            };
            if stmt.kind() == "return_expression" {
                let txt = self.node_text(&stmt, source);
                if txt.contains("Ok(") || txt.trim_end_matches(';').trim() == "return" {
                    has_ok_return = true;
                } else if txt.contains("Err(") || txt.contains("panic!") {
                    has_err_return = true;
                } else {
                    has_ok_return = true;
                }
            } else if stmt.kind() == "call_expression" && first_call_label.is_none() {
                let txt = self.node_text(&stmt, source);
                first_call_label = Some(txt.trim().to_string());
            }
        }

        let (kind, label, edge_label) = if has_err_return {
            (FlowNodeKind::Terminal, "error".to_string(), "err")
        } else if has_ok_return {
            (FlowNodeKind::Terminal, "ok".to_string(), "ok")
        } else if let Some(call) = first_call_label {
            (FlowNodeKind::Process, call, "branch")
        } else {
            (FlowNodeKind::Process, "continue".to_string(), "branch")
        };

        nodes.insert(
            id.to_string(),
            FlowNode {
                kind,
                label: Some(label),
                ..Default::default()
            },
        );
        (id.to_string(), edge_label.to_string())
    }

    /// Walk a `struct_item` node's body and collect its named fields. Tuple
    /// struct variants (e.g. `struct Foo(u32)`) and unit structs produce
    /// no fields — that's correct: TD schemas are built on named fields.
    fn extract_rust_struct_fields(
        &self,
        node: &tree_sitter::Node,
        source: &str,
    ) -> Vec<StructField> {
        let mut out = Vec::new();
        let Some(body) = node.child_by_field_name("body") else {
            return out;
        };
        // Body is a `field_declaration_list`; fields are `field_declaration`.
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            if child.kind() != "field_declaration" {
                continue;
            }
            let Some(name_node) = child.child_by_field_name("name") else {
                continue;
            };
            let Some(type_node) = child.child_by_field_name("type") else {
                continue;
            };
            let is_public = child
                .children(&mut child.walk())
                .any(|c| c.kind() == "visibility_modifier");
            out.push(StructField {
                name: self.node_text(&name_node, source),
                rust_type: self.node_text(&type_node, source),
                is_public,
            });
        }
        out
    }

    /// Walk an `enum_item` node's body and collect its variant names. Tuple
    /// and struct variants contribute the bare identifier — payload types
    /// are not represented on `EnumContent` yet and would overshoot TD's
    /// enum shape.
    fn extract_rust_enum_variants(&self, node: &tree_sitter::Node, source: &str) -> Vec<String> {
        let mut out = Vec::new();
        let Some(body) = node.child_by_field_name("body") else {
            return out;
        };
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            if child.kind() != "enum_variant" {
                continue;
            }
            if let Some(name_node) = child.child_by_field_name("name") {
                out.push(self.node_text(&name_node, source));
            }
        }
        out
    }

    fn extract_rust_function_signature(&self, node: &tree_sitter::Node, source: &str) -> String {
        let mut signature = String::new();

        // Get function name
        if let Some(name_node) = node.child_by_field_name("name") {
            signature.push_str(&self.node_text(&name_node, source));
        }

        // Get parameters
        if let Some(params_node) = node.child_by_field_name("parameters") {
            signature.push_str(&self.node_text(&params_node, source));
        }

        // Get return type
        if let Some(return_node) = node.child_by_field_name("return_type") {
            signature.push_str(" -> ");
            signature.push_str(&self.node_text(&return_node, source));
        }

        signature
    }

    /// Extract Rust doc comment (/// or //!)
    fn extract_rust_doc_comment(&self, node: &tree_sitter::Node, source: &str) -> Option<String> {
        // Look for preceding line comments that are doc comments
        let start_row = node.start_position().row;
        if start_row == 0 {
            return None;
        }

        let lines: Vec<&str> = source.lines().collect();
        let mut doc_lines = Vec::new();

        // Look backwards from the node for doc comments
        let mut row = start_row.saturating_sub(1);
        while row < lines.len() {
            let line = lines[row].trim();
            if line.starts_with("///") {
                doc_lines.insert(0, line.trim_start_matches("///").trim());
            } else if line.starts_with("//!") {
                doc_lines.insert(0, line.trim_start_matches("//!").trim());
            } else if !line.is_empty() && !line.starts_with("//") {
                break;
            }
            if row == 0 {
                break;
            }
            row -= 1;
        }

        if doc_lines.is_empty() {
            None
        } else {
            Some(doc_lines.join(" "))
        }
    }

    /// Check if node has visibility modifier (pub)
    fn has_visibility_modifier(&self, node: &tree_sitter::Node, source: &str) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "visibility_modifier" {
                return true;
            }
            // Also check if line starts with pub
            let text = self.node_text(node, source);
            if text.starts_with("pub ") || text.starts_with("pub(") {
                return true;
            }
        }
        false
    }

    /// Extract Rust use statement
    fn extract_rust_use(&self, node: &tree_sitter::Node, source: &str) -> Option<Import> {
        let text = self.node_text(node, source);

        // Parse use statement (simplified)
        let path = text
            .trim_start_matches("use ")
            .trim_end_matches(';')
            .to_string();

        // Determine if external (doesn't start with crate::, self::, super::)
        let is_external = !path.starts_with("crate::")
            && !path.starts_with("self::")
            && !path.starts_with("super::");

        // Extract the base path (before any braces or ::*)
        let base_path = path
            .split("::{")
            .next()
            .unwrap_or(&path)
            .split("::*")
            .next()
            .unwrap_or(&path)
            .to_string();

        Some(Import {
            path: base_path,
            items: vec![],
            is_external,
        })
    }

    /// Extract symbols and imports from Python AST nodes
    fn extract_python_node(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<Symbol>,
        imports: &mut Vec<Import>,
    ) {
        self.extract_python_node_in(node, source, symbols, imports, "", None);
    }

    /// Recursive worker for python extraction. `name_prefix` qualifies
    /// nested function names (e.g. `ClassName.` for methods);
    /// `pending_decorators` threads decorator text from a surrounding
    /// `decorated_definition` down to the inner def/class.
    /// @spec projects/agentic-workflow/tech-design/core/specs/python-ast-recursive-extraction.md#logic
    fn extract_python_node_in(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<Symbol>,
        imports: &mut Vec<Import>,
        name_prefix: &str,
        pending_decorators: Option<&[String]>,
    ) {
        let kind = node.kind();

        match kind {
            "function_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let raw_name = self.node_text(&name_node, source);
                    let is_public = !raw_name.starts_with('_');
                    let qualified_name = format!("{name_prefix}{raw_name}");
                    let mut signature = self.extract_python_function_signature(node, source);
                    let is_async = (0..node.child_count())
                        .any(|i| node.child(i).map(|c| c.kind() == "async").unwrap_or(false));
                    if is_async {
                        signature = format!("async {signature}");
                    }
                    if let Some(decos) = pending_decorators {
                        let joined: String = decos.iter().map(|d| format!("@{d}\n")).collect();
                        signature = format!("{joined}{signature}");
                    }
                    let doc = self.extract_python_docstring(node, source);

                    symbols.push(Symbol {
                        name: qualified_name,
                        kind: SymbolKind::Function,
                        signature: Some(signature),
                        doc,
                        line: node.start_position().row + 1,
                        is_public,
                        ..Default::default()
                    });
                }
            }
            "class_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let raw_name = self.node_text(&name_node, source);
                    let is_public = !raw_name.starts_with('_');
                    let qualified_name = format!("{name_prefix}{raw_name}");
                    let doc = self.extract_python_docstring(node, source);

                    let mut signature = format!("class {raw_name}");
                    if let Some(supers) = node.child_by_field_name("superclasses") {
                        signature.push_str(&self.node_text(&supers, source));
                    }
                    if let Some(decos) = pending_decorators {
                        let joined: String = decos.iter().map(|d| format!("@{d}\n")).collect();
                        signature = format!("{joined}{signature}");
                    }

                    symbols.push(Symbol {
                        name: qualified_name.clone(),
                        kind: SymbolKind::Class,
                        signature: Some(signature),
                        doc,
                        line: node.start_position().row + 1,
                        is_public,
                        ..Default::default()
                    });

                    if let Some(body) = node.child_by_field_name("body") {
                        let mut cursor = body.walk();
                        let inner_prefix = format!("{qualified_name}.");
                        for child in body.children(&mut cursor) {
                            self.extract_python_node_in(
                                &child,
                                source,
                                symbols,
                                imports,
                                &inner_prefix,
                                None,
                            );
                        }
                    }
                }
            }
            "import_statement" | "import_from_statement" => {
                let import = self.extract_python_import(node, source);
                if let Some(import) = import {
                    imports.push(import);
                }
            }
            "decorated_definition" => {
                let mut decos: Vec<String> = Vec::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "decorator" {
                        let raw = self.node_text(&child, source);
                        let trimmed = raw.trim().trim_start_matches('@').to_string();
                        decos.push(trimmed);
                    }
                }
                if let Some(inner) = node.child_by_field_name("definition") {
                    self.extract_python_node_in(
                        &inner,
                        source,
                        symbols,
                        imports,
                        name_prefix,
                        Some(&decos),
                    );
                }
            }
            "if_statement" | "elif_clause" | "else_clause" | "try_statement" | "except_clause"
            | "finally_clause" | "with_statement" | "block" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.extract_python_node_in(
                        &child,
                        source,
                        symbols,
                        imports,
                        name_prefix,
                        None,
                    );
                }
            }
            _ => {}
        }
    }

    /// Extract Python function signature
    fn extract_python_function_signature(&self, node: &tree_sitter::Node, source: &str) -> String {
        let mut signature = String::new();

        if let Some(name_node) = node.child_by_field_name("name") {
            signature.push_str(&self.node_text(&name_node, source));
        }

        if let Some(params_node) = node.child_by_field_name("parameters") {
            signature.push_str(&self.node_text(&params_node, source));
        }

        if let Some(return_node) = node.child_by_field_name("return_type") {
            signature.push_str(" -> ");
            signature.push_str(&self.node_text(&return_node, source));
        }

        signature
    }

    /// Extract Python docstring
    fn extract_python_docstring(&self, node: &tree_sitter::Node, source: &str) -> Option<String> {
        // Look for string expression as first statement in body
        let body = node.child_by_field_name("body")?;
        let mut cursor = body.walk();
        // Only check the first child for docstring
        let first_child = body.children(&mut cursor).next()?;

        if first_child.kind() == "expression_statement" {
            let mut inner_cursor = first_child.walk();
            for expr in first_child.children(&mut inner_cursor) {
                if expr.kind() == "string" {
                    let text = self.node_text(&expr, source);
                    // Remove quotes
                    let doc = text
                        .trim_start_matches("\"\"\"")
                        .trim_end_matches("\"\"\"")
                        .trim_start_matches("'''")
                        .trim_end_matches("'''")
                        .trim_start_matches('"')
                        .trim_end_matches('"')
                        .trim()
                        .to_string();
                    return Some(doc);
                }
            }
        }
        None
    }

    /// Extract Python import
    fn extract_python_import(&self, node: &tree_sitter::Node, source: &str) -> Option<Import> {
        let text = self.node_text(node, source);

        // Parse import statement (simplified)
        let (path, items) = if text.starts_with("from ") {
            let parts: Vec<&str> = text.split(" import ").collect();
            let path = parts
                .first()?
                .trim_start_matches("from ")
                .trim()
                .to_string();
            let items: Vec<String> = parts
                .get(1)
                .map(|s| s.split(',').map(|i| i.trim().to_string()).collect())
                .unwrap_or_default();
            (path, items)
        } else {
            let path = text.trim_start_matches("import ").trim().to_string();
            (path, vec![])
        };

        // Determine if external (doesn't start with .)
        let is_external = !path.starts_with('.');

        Some(Import {
            path,
            items,
            is_external,
        })
    }

    /// Extract symbols and imports from JavaScript/TypeScript AST nodes
    fn extract_js_node(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<Symbol>,
        imports: &mut Vec<Import>,
    ) {
        let kind = node.kind();

        match kind {
            "function_declaration" | "generator_function_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let signature = self.extract_js_function_signature(node, source);
                    let doc = self.extract_jsdoc(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        signature: Some(signature),
                        doc,
                        line: node.start_position().row + 1,
                        is_public: true, // JS exports determine visibility
                        ..Default::default()
                    });
                }
            }
            "class_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let doc = self.extract_jsdoc(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Class,
                        signature: None,
                        doc,
                        line: node.start_position().row + 1,
                        is_public: true,
                        ..Default::default()
                    });
                }
            }
            "interface_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Interface,
                        signature: None,
                        doc: None,
                        line: node.start_position().row + 1,
                        is_public: true,
                        ..Default::default()
                    });
                }
            }
            "type_alias_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Type,
                        signature: None,
                        doc: None,
                        line: node.start_position().row + 1,
                        is_public: true,
                        ..Default::default()
                    });
                }
            }
            "lexical_declaration" => {
                // const/let declarations
                self.extract_js_variable_declaration(node, source, symbols);
            }
            "import_statement" => {
                let import = self.extract_js_import(node, source);
                if let Some(import) = import {
                    imports.push(import);
                }
            }
            "export_statement" => {
                // Handle export default function, export const, etc.
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.extract_js_node(&child, source, symbols, imports);
                }
            }
            _ => {}
        }
    }

    /// Extract JS function signature
    fn extract_js_function_signature(&self, node: &tree_sitter::Node, source: &str) -> String {
        let mut signature = String::new();

        if let Some(name_node) = node.child_by_field_name("name") {
            signature.push_str(&self.node_text(&name_node, source));
        }

        if let Some(params_node) = node.child_by_field_name("parameters") {
            signature.push_str(&self.node_text(&params_node, source));
        }

        signature
    }

    /// Extract JS variable declaration
    fn extract_js_variable_declaration(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<Symbol>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_declarator" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);

                    // Check if it's a function expression
                    if let Some(value_node) = child.child_by_field_name("value") {
                        let kind = match value_node.kind() {
                            "arrow_function" | "function" => SymbolKind::Function,
                            _ => SymbolKind::Constant,
                        };

                        symbols.push(Symbol {
                            name,
                            kind,
                            signature: None,
                            doc: None,
                            line: node.start_position().row + 1,
                            is_public: true,
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    /// Extract JSDoc comment
    fn extract_jsdoc(&self, node: &tree_sitter::Node, source: &str) -> Option<String> {
        let start_row = node.start_position().row;
        if start_row == 0 {
            return None;
        }

        let lines: Vec<&str> = source.lines().collect();
        let mut doc_lines = Vec::new();
        let mut in_jsdoc = false;

        let mut row = start_row.saturating_sub(1);
        while row < lines.len() {
            let line = lines[row].trim();

            if line.ends_with("*/") {
                in_jsdoc = true;
            } else if line.starts_with("/**") {
                if in_jsdoc {
                    doc_lines.insert(
                        0,
                        line.trim_start_matches("/**").trim_end_matches("*/").trim(),
                    );
                }
                break;
            } else if in_jsdoc {
                doc_lines.insert(0, line.trim_start_matches('*').trim());
            } else if !line.is_empty() {
                break;
            }

            if row == 0 {
                break;
            }
            row -= 1;
        }

        if doc_lines.is_empty() {
            None
        } else {
            Some(doc_lines.join(" "))
        }
    }

    /// Extract JS import
    fn extract_js_import(&self, node: &tree_sitter::Node, source: &str) -> Option<Import> {
        let text = self.node_text(node, source);

        // Parse import statement (simplified)
        // Examples: import { a, b } from 'module'
        //           import default from 'module'
        //           import * as name from 'module'

        let path = text
            .split(" from ")
            .last()?
            .trim()
            .trim_matches(|c| c == '\'' || c == '"' || c == ';')
            .to_string();

        // Determine if external (doesn't start with . or /)
        let is_external = !path.starts_with('.') && !path.starts_with('/');

        Some(Import {
            path,
            items: vec![],
            is_external,
        })
    }

    /// Extract symbols and imports from Go AST nodes
    fn extract_go_node(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<Symbol>,
        imports: &mut Vec<Import>,
    ) {
        let kind = node.kind();

        match kind {
            "function_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false);
                    let signature = self.extract_go_function_signature(node, source);
                    let doc = self.extract_go_doc(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        signature: Some(signature),
                        doc,
                        line: node.start_position().row + 1,
                        is_public,
                        ..Default::default()
                    });
                }
            }
            "method_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false);
                    let signature = self.extract_go_function_signature(node, source);
                    let doc = self.extract_go_doc(node, source);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Function,
                        signature: Some(signature),
                        doc,
                        line: node.start_position().row + 1,
                        is_public,
                        ..Default::default()
                    });
                }
            }
            "type_declaration" => {
                self.extract_go_type_declaration(node, source, symbols);
            }
            "import_declaration" => {
                let import = self.extract_go_import(node, source);
                for imp in import {
                    imports.push(imp);
                }
            }
            "const_declaration" | "var_declaration" => {
                self.extract_go_const_var(node, source, symbols);
            }
            _ => {}
        }
    }

    /// Extract Go function signature
    fn extract_go_function_signature(&self, node: &tree_sitter::Node, source: &str) -> String {
        let mut signature = String::new();

        if let Some(name_node) = node.child_by_field_name("name") {
            signature.push_str(&self.node_text(&name_node, source));
        }

        if let Some(params_node) = node.child_by_field_name("parameters") {
            signature.push_str(&self.node_text(&params_node, source));
        }

        if let Some(result_node) = node.child_by_field_name("result") {
            signature.push_str(" ");
            signature.push_str(&self.node_text(&result_node, source));
        }

        signature
    }

    /// Extract Go doc comment
    fn extract_go_doc(&self, node: &tree_sitter::Node, source: &str) -> Option<String> {
        let start_row = node.start_position().row;
        if start_row == 0 {
            return None;
        }

        let lines: Vec<&str> = source.lines().collect();
        let mut doc_lines = Vec::new();

        let mut row = start_row.saturating_sub(1);
        while row < lines.len() {
            let line = lines[row].trim();

            if line.starts_with("//") {
                doc_lines.insert(0, line.trim_start_matches("//").trim());
            } else if !line.is_empty() {
                break;
            }

            if row == 0 {
                break;
            }
            row -= 1;
        }

        if doc_lines.is_empty() {
            None
        } else {
            Some(doc_lines.join(" "))
        }
    }

    /// Extract Go type declaration
    fn extract_go_type_declaration(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<Symbol>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "type_spec" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false);

                    // Determine kind based on type
                    let kind = if let Some(type_node) = child.child_by_field_name("type") {
                        match type_node.kind() {
                            "struct_type" => SymbolKind::Struct,
                            "interface_type" => SymbolKind::Interface,
                            _ => SymbolKind::Type,
                        }
                    } else {
                        SymbolKind::Type
                    };

                    symbols.push(Symbol {
                        name,
                        kind,
                        signature: None,
                        doc: None,
                        line: node.start_position().row + 1,
                        is_public,
                        ..Default::default()
                    });
                }
            }
        }
    }

    /// Extract Go const/var declaration
    fn extract_go_const_var(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        symbols: &mut Vec<Symbol>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "const_spec" || child.kind() == "var_spec" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = self.node_text(&name_node, source);
                    let is_public = name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false);

                    symbols.push(Symbol {
                        name,
                        kind: SymbolKind::Constant,
                        signature: None,
                        doc: None,
                        line: node.start_position().row + 1,
                        is_public,
                        ..Default::default()
                    });
                }
            }
        }
    }

    /// Extract Go import
    fn extract_go_import(&self, node: &tree_sitter::Node, source: &str) -> Vec<Import> {
        let mut imports = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "import_spec" {
                let text = self.node_text(&child, source);
                let path = text.trim().trim_matches('"').to_string();

                // Determine if external (doesn't contain the current module path)
                // Simplified: assume external if it contains a dot
                let is_external = path.contains('.');

                imports.push(Import {
                    path,
                    items: vec![],
                    is_external,
                });
            } else if child.kind() == "import_spec_list" {
                let mut inner_cursor = child.walk();
                for spec in child.children(&mut inner_cursor) {
                    if spec.kind() == "import_spec" {
                        let text = self.node_text(&spec, source);
                        let path = text.trim().trim_matches('"').to_string();
                        let is_external = path.contains('.');

                        imports.push(Import {
                            path,
                            items: vec![],
                            is_external,
                        });
                    }
                }
            }
        }

        imports
    }

    /// Get text content of a node
    fn node_text(&self, node: &tree_sitter::Node, source: &str) -> String {
        let start = node.start_byte();
        let end = node.end_byte();
        source[start..end].to_string()
    }
}

/// Identifier-safe slug used as `LogicContent.id`. Lowercased ASCII with
/// non-alnum → `_`, leading digits prefixed with `_`, never empty.
fn snake_sanitise(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('_') && !out.is_empty() {
            out.push('_');
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    if out.is_empty() {
        "fn_logic".to_string()
    } else if out.chars().next().unwrap().is_ascii_digit() {
        format!("_{}", out)
    } else {
        out
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/fillback/ast-standardization.md#source
impl Default for AstAnalyzer {
    fn default() -> Self {
        Self::new().expect("Failed to initialize AST analyzer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_language_from_extension() {
        assert_eq!(
            SupportedLanguage::from_extension("rs"),
            Some(SupportedLanguage::Rust)
        );
        assert_eq!(
            SupportedLanguage::from_extension("py"),
            Some(SupportedLanguage::Python)
        );
        assert_eq!(
            SupportedLanguage::from_extension("js"),
            Some(SupportedLanguage::JavaScript)
        );
        assert_eq!(
            SupportedLanguage::from_extension("ts"),
            Some(SupportedLanguage::TypeScript)
        );
        assert_eq!(
            SupportedLanguage::from_extension("go"),
            Some(SupportedLanguage::Go)
        );
        assert_eq!(SupportedLanguage::from_extension("txt"), None);
    }

    #[test]
    fn test_parse_rust_file() {
        let mut analyzer = AstAnalyzer::new().unwrap();
        let content = r#"
use std::path::Path;
use crate::models::Foo;

/// A test function
pub fn test_function(x: i32) -> String {
    x.to_string()
}

struct TestStruct {
    field: String,
}

pub enum TestEnum {
    A,
    B,
}
"#;

        let result = analyzer.parse_file(&PathBuf::from("test.rs"), content);
        assert!(result.is_ok());

        let module = result.unwrap();
        assert_eq!(module.name, "test");
        assert_eq!(module.language, SupportedLanguage::Rust);

        // Check symbols
        let function = module
            .symbols
            .iter()
            .find(|s| s.name == "test_function")
            .expect("Should find test_function");
        assert_eq!(function.kind, SymbolKind::Function);
        assert!(function.is_public);
        assert!(function.signature.as_ref().unwrap().contains("i32"));

        let struct_symbol = module
            .symbols
            .iter()
            .find(|s| s.name == "TestStruct")
            .expect("Should find TestStruct");
        assert_eq!(struct_symbol.kind, SymbolKind::Struct);

        let enum_symbol = module
            .symbols
            .iter()
            .find(|s| s.name == "TestEnum")
            .expect("Should find TestEnum");
        assert_eq!(enum_symbol.kind, SymbolKind::Enum);

        // Check imports
        assert!(module.imports.len() >= 2);
        let external_import = module.imports.iter().find(|i| i.path.contains("std"));
        assert!(external_import.is_some());
    }

    #[test]
    fn test_parse_python_file() {
        let mut analyzer = AstAnalyzer::new().unwrap();
        let content = r#"
import os
from typing import List

def public_function(x: int) -> str:
    """This is a docstring."""
    return str(x)

def _private_function():
    pass

class TestClass:
    """A test class."""
    def method(self):
        pass
"#;

        let result = analyzer.parse_file(&PathBuf::from("test.py"), content);
        assert!(result.is_ok());

        let module = result.unwrap();
        assert_eq!(module.language, SupportedLanguage::Python);

        let public_fn = module
            .symbols
            .iter()
            .find(|s| s.name == "public_function")
            .expect("Should find public_function");
        assert!(public_fn.is_public);
        assert!(public_fn.doc.is_some());

        let private_fn = module
            .symbols
            .iter()
            .find(|s| s.name == "_private_function")
            .expect("Should find _private_function");
        assert!(!private_fn.is_public);

        let class = module
            .symbols
            .iter()
            .find(|s| s.name == "TestClass")
            .expect("Should find TestClass");
        assert_eq!(class.kind, SymbolKind::Class);
    }

    #[test]
    fn test_parse_python_decorated_definitions() {
        // Regression: tree-sitter-python wraps `@decorator\ndef f():` in a
        // `decorated_definition` node. Without recursion the function /
        // class is silently dropped — exactly what was hiding FastAPI
        // routes and @contextmanager defs from fixture_platform claim_code
        // output before this fix.
        let mut analyzer = AstAnalyzer::new().unwrap();
        let content = r#"
import contextlib

@some_router.get(path="/health")
async def health_check() -> None:
    return None

@contextlib.asynccontextmanager
async def lifespan(app):
    yield

@dataclass
class Config:
    name: str
"#;
        let module = analyzer
            .parse_file(&PathBuf::from("decorated.py"), content)
            .expect("parse decorated.py");
        let names: Vec<&str> = module.symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(
            names.contains(&"health_check"),
            "decorated async def missing: {names:?}"
        );
        assert!(
            names.contains(&"lifespan"),
            "@asynccontextmanager-decorated def missing: {names:?}"
        );
        assert!(
            names.contains(&"Config"),
            "@dataclass-decorated class missing: {names:?}"
        );
        let health = module
            .symbols
            .iter()
            .find(|s| s.name == "health_check")
            .unwrap();
        assert_eq!(health.kind, SymbolKind::Function);
        let cfg = module.symbols.iter().find(|s| s.name == "Config").unwrap();
        assert_eq!(cfg.kind, SymbolKind::Class);
    }

    #[test]
    fn test_python_extracts_class_methods() {
        // R1: methods inside a class body must surface as
        // `ClassName.method_name` symbols.
        let mut analyzer = AstAnalyzer::new().unwrap();
        let content = r#"
class TestUserFlow:
    def test_login(self):
        pass
    async def test_logout(self):
        pass

class Helper:
    @staticmethod
    def util(x):
        return x
"#;
        let module = analyzer
            .parse_file(&PathBuf::from("test_methods.py"), content)
            .expect("parse test_methods.py");
        let names: Vec<&str> = module.symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"TestUserFlow"), "class missing: {names:?}");
        assert!(
            names.contains(&"TestUserFlow.test_login"),
            "qualified method missing: {names:?}"
        );
        assert!(
            names.contains(&"TestUserFlow.test_logout"),
            "qualified async method missing: {names:?}"
        );
        assert!(
            names.contains(&"Helper.util"),
            "decorated method missing: {names:?}"
        );
    }

    #[test]
    fn test_python_extracts_env_gated_def() {
        // R2: defs inside `if FLAG:` must reach the symbol list.
        let mut analyzer = AstAnalyzer::new().unwrap();
        let content = r#"
DEV = True

if DEV:
    def dev_only_route():
        return None

    @some_router.post(path="/echo")
    async def echo(data):
        return data
"#;
        let module = analyzer
            .parse_file(&PathBuf::from("env_gated.py"), content)
            .expect("parse env_gated.py");
        let names: Vec<&str> = module.symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(
            names.contains(&"dev_only_route"),
            "env-gated def missing: {names:?}"
        );
        assert!(
            names.contains(&"echo"),
            "env-gated decorated def missing: {names:?}"
        );
    }

    #[test]
    fn test_python_decorator_embedded_in_signature() {
        // R3: decorator text on `decorated_definition` is prepended to
        // the inner symbol's signature, one `@<call>\n` per line.
        let mut analyzer = AstAnalyzer::new().unwrap();
        let content = r#"
@router.get(path="/health")
@auth_required
async def health() -> None:
    return None
"#;
        let module = analyzer
            .parse_file(&PathBuf::from("multi_deco.py"), content)
            .expect("parse multi_deco.py");
        let sym = module
            .symbols
            .iter()
            .find(|s| s.name == "health")
            .expect("health symbol present");
        let sig = sym.signature.as_deref().unwrap_or("");
        assert!(
            sig.contains("@router.get(path=\"/health\")"),
            "first decorator missing from signature: {sig:?}"
        );
        assert!(
            sig.contains("@auth_required"),
            "second decorator missing from signature: {sig:?}"
        );
    }

    #[test]
    fn test_python_async_def_signature_prefix() {
        // R4: `async def` must surface as `async ` prefix on signature.
        let mut analyzer = AstAnalyzer::new().unwrap();
        let content = r#"
async def fetch(url: str) -> str:
    return url

def sync_fetch(url: str) -> str:
    return url
"#;
        let module = analyzer
            .parse_file(&PathBuf::from("async_def.py"), content)
            .expect("parse async_def.py");
        let fetch = module
            .symbols
            .iter()
            .find(|s| s.name == "fetch")
            .expect("fetch present");
        let sync = module
            .symbols
            .iter()
            .find(|s| s.name == "sync_fetch")
            .expect("sync_fetch present");
        assert!(
            fetch
                .signature
                .as_deref()
                .unwrap_or("")
                .starts_with("async "),
            "async prefix missing: {:?}",
            fetch.signature
        );
        assert!(
            !sync
                .signature
                .as_deref()
                .unwrap_or("")
                .starts_with("async "),
            "sync def should not have async prefix: {:?}",
            sync.signature
        );
    }

    #[test]
    fn test_python_class_signature_includes_bases() {
        // R5: base classes must survive into Symbol.signature.
        let mut analyzer = AstAnalyzer::new().unwrap();
        let content = r#"
class Foo(BaseModel, GenericMixin):
    name: str

class Bare:
    pass
"#;
        let module = analyzer
            .parse_file(&PathBuf::from("bases.py"), content)
            .expect("parse bases.py");
        let foo = module
            .symbols
            .iter()
            .find(|s| s.name == "Foo")
            .expect("Foo present");
        let bare = module
            .symbols
            .iter()
            .find(|s| s.name == "Bare")
            .expect("Bare present");
        let foo_sig = foo.signature.as_deref().unwrap_or("");
        assert!(
            foo_sig.contains("BaseModel") && foo_sig.contains("GenericMixin"),
            "bases missing from signature: {foo_sig:?}"
        );
        let bare_sig = bare.signature.as_deref().unwrap_or("");
        assert_eq!(
            bare_sig, "class Bare",
            "bare class signature drift: {bare_sig:?}"
        );
    }

    #[test]
    fn test_parse_javascript_file() {
        let mut analyzer = AstAnalyzer::new().unwrap();
        let content = r#"
import { something } from './module';
import external from 'external-package';

function regularFunction(x) {
    return x;
}

const arrowFunction = (y) => y * 2;

class TestClass {
    constructor() {}
}
"#;

        let result = analyzer.parse_file(&PathBuf::from("test.js"), content);
        assert!(result.is_ok());

        let module = result.unwrap();
        assert_eq!(module.language, SupportedLanguage::JavaScript);

        // Check function was found
        assert!(module.symbols.iter().any(|s| s.name == "regularFunction"));
        assert!(module.symbols.iter().any(|s| s.name == "TestClass"));

        // Check imports
        let internal_import = module.imports.iter().find(|i| i.path == "./module");
        assert!(internal_import.is_some());
        assert!(!internal_import.unwrap().is_external);

        let external_import = module.imports.iter().find(|i| i.path == "external-package");
        assert!(external_import.is_some());
        assert!(external_import.unwrap().is_external);
    }

    #[test]
    fn test_parse_go_file() {
        let mut analyzer = AstAnalyzer::new().unwrap();
        let content = r#"
package main

import (
    "fmt"
    "github.com/example/pkg"
)

// PublicFunction is exported
func PublicFunction(x int) string {
    return fmt.Sprintf("%d", x)
}

func privateFunction() {
}

type PublicStruct struct {
    Field string
}

type PublicInterface interface {
    Method()
}
"#;

        let result = analyzer.parse_file(&PathBuf::from("test.go"), content);
        assert!(result.is_ok());

        let module = result.unwrap();
        assert_eq!(module.language, SupportedLanguage::Go);

        let public_fn = module
            .symbols
            .iter()
            .find(|s| s.name == "PublicFunction")
            .expect("Should find PublicFunction");
        assert!(public_fn.is_public);

        let private_fn = module
            .symbols
            .iter()
            .find(|s| s.name == "privateFunction")
            .expect("Should find privateFunction");
        assert!(!private_fn.is_public);

        let public_struct = module
            .symbols
            .iter()
            .find(|s| s.name == "PublicStruct")
            .expect("Should find PublicStruct");
        assert_eq!(public_struct.kind, SymbolKind::Struct);
        assert!(public_struct.is_public);

        let public_interface = module
            .symbols
            .iter()
            .find(|s| s.name == "PublicInterface")
            .expect("Should find PublicInterface");
        assert_eq!(public_interface.kind, SymbolKind::Interface);
    }

    #[test]
    fn test_unsupported_extension() {
        let mut analyzer = AstAnalyzer::new().unwrap();
        let result = analyzer.parse_file(&PathBuf::from("test.txt"), "some text");
        assert!(result.is_err());
    }

    #[test]
    fn test_analysis_context() {
        let mut context = AnalysisContext::new();

        context.modules.push(ModuleInfo {
            name: "test".to_string(),
            path: "test.rs".to_string(),
            language: SupportedLanguage::Rust,
            symbols: vec![
                Symbol {
                    name: "fn1".to_string(),
                    kind: SymbolKind::Function,
                    signature: None,
                    doc: None,
                    line: 1,
                    is_public: true,
                    ..Default::default()
                },
                Symbol {
                    name: "fn2".to_string(),
                    kind: SymbolKind::Function,
                    signature: None,
                    doc: None,
                    line: 5,
                    is_public: true,
                    ..Default::default()
                },
            ],
            imports: vec![
                Import {
                    path: "std::path".to_string(),
                    items: vec![],
                    is_external: true,
                },
                Import {
                    path: "crate::models".to_string(),
                    items: vec![],
                    is_external: false,
                },
            ],
        });

        assert_eq!(context.total_symbols(), 2);
        assert_eq!(context.external_dependencies(), vec!["std::path"]);
    }
}
// CODEGEN-END
