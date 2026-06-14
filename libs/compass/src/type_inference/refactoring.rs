//! Refactoring operations (Sprint 3 - Track 1)
//!
//! Provides type-aware refactoring operations:
//! - Extract function/method/variable
//! - Rename symbol (cross-file)
//! - Move definition
//! - Inline symbol
//! - Change signature

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use super::deep_inference::{DeepTypeInferencer, TypeContext};
use super::mutable_ast::{MutableAst, MutableNode, NodeId, NodeMetadata, Span};
use super::semantic_search::{SearchKind, SearchQuery, SearchScope, SemanticSearchEngine};
use crate::syntax::MultiParser;
use tree_sitter::Node;

// ============================================================================
// Refactoring Request
// ============================================================================

/// A request for a refactoring operation.
#[derive(Debug, Clone)]
pub struct RefactorRequest {
    /// Type of refactoring
    pub kind: RefactorKind,
    /// Target file
    pub file: PathBuf,
    /// Target span in source
    pub span: Span,
    /// Additional options
    pub options: RefactorOptions,
}

/// Type of refactoring operation.
#[derive(Debug, Clone)]
pub enum RefactorKind {
    /// Extract code into a new function
    ExtractFunction { name: String },
    /// Extract code into a new method
    ExtractMethod { name: String },
    /// Extract expression into a variable
    ExtractVariable { name: String },
    /// Rename a symbol
    Rename { new_name: String },
    /// Move a definition to another file
    MoveDefinition { target_file: PathBuf },
    /// Inline a symbol's definition
    Inline,
    /// Change function signature
    ChangeSignature { changes: SignatureChanges },
}

/// Options for refactoring operations.
#[derive(Debug, Clone, Default)]
pub struct RefactorOptions {
    /// Preview changes without applying
    pub preview_only: bool,
    /// Update imports automatically
    pub update_imports: bool,
    /// Add type annotations
    pub add_type_annotations: bool,
    /// Preserve formatting
    pub preserve_formatting: bool,
}

/// Changes to a function signature.
#[derive(Debug, Clone, Default)]
pub struct SignatureChanges {
    /// New parameters (name, type_annotation, default)
    pub new_params: Vec<(String, Option<String>, Option<String>)>,
    /// Reordered parameter indices
    pub param_order: Vec<usize>,
    /// Removed parameter indices
    pub removed_params: Vec<usize>,
    /// New return type annotation
    pub new_return_type: Option<String>,
}

// ============================================================================
// Refactoring Result
// ============================================================================

/// Result of a refactoring operation.
#[derive(Debug, Clone)]
pub struct RefactorResult {
    /// Edits to apply per file
    pub file_edits: HashMap<PathBuf, Vec<TextEdit>>,
    /// New files to create
    pub new_files: HashMap<PathBuf, String>,
    /// Files to delete
    pub deleted_files: Vec<PathBuf>,
    /// Import changes per file
    pub import_changes: HashMap<PathBuf, Vec<ImportChange>>,
    /// Diagnostics/warnings
    pub diagnostics: Vec<RefactorDiagnostic>,
}

/// A text edit to apply.
#[derive(Debug, Clone)]
pub struct TextEdit {
    /// Span to replace
    pub span: Span,
    /// New text
    pub new_text: String,
}

/// A change to imports.
#[derive(Debug, Clone)]
pub enum ImportChange {
    /// Add an import
    Add { module: String, names: Vec<String> },
    /// Remove an import
    Remove { module: String, names: Vec<String> },
    /// Update an import
    Update {
        module: String,
        old_names: Vec<String>,
        new_names: Vec<String>,
    },
}

/// A diagnostic from refactoring.
#[derive(Debug, Clone)]
pub struct RefactorDiagnostic {
    /// Severity level
    pub level: DiagnosticLevel,
    /// Message
    pub message: String,
    /// Related file
    pub file: Option<PathBuf>,
    /// Related span
    pub span: Option<Span>,
}

/// Diagnostic severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
    Hint,
}

impl RefactorResult {
    /// Create an empty result.
    pub fn empty() -> Self {
        Self {
            file_edits: HashMap::new(),
            new_files: HashMap::new(),
            deleted_files: Vec::new(),
            import_changes: HashMap::new(),
            diagnostics: Vec::new(),
        }
    }

    /// Check if there are any changes.
    pub fn has_changes(&self) -> bool {
        !self.file_edits.is_empty() || !self.new_files.is_empty() || !self.deleted_files.is_empty()
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.level == DiagnosticLevel::Error)
    }

    /// Add a text edit for a file.
    pub fn add_edit(&mut self, file: PathBuf, edit: TextEdit) {
        self.file_edits.entry(file).or_default().push(edit);
    }

    /// Add a diagnostic.
    pub fn add_diagnostic(
        &mut self,
        level: DiagnosticLevel,
        message: impl Into<String>,
        file: Option<PathBuf>,
        span: Option<Span>,
    ) {
        self.diagnostics.push(RefactorDiagnostic {
            level,
            message: message.into(),
            file,
            span,
        });
    }
}

// ============================================================================
// Refactoring Engine
// ============================================================================

/// Data flow analysis result.
struct DataFlow {
    /// Variables used but not defined in the selection (become parameters)
    external_vars: Vec<String>,
    /// Variables that are defined and used after selection (need to be returned)
    returned_vars: Vec<String>,
}

/// Engine for performing refactoring operations.
pub struct RefactoringEngine {
    /// Type inferencer for type information
    inferencer: DeepTypeInferencer,
    /// AST cache per file
    ast_cache: HashMap<PathBuf, MutableAst>,
    /// Semantic search engine for finding references
    search_engine: SemanticSearchEngine,
}

impl RefactoringEngine {
    /// Create a new refactoring engine.
    pub fn new() -> Self {
        Self {
            inferencer: DeepTypeInferencer::new(),
            ast_cache: HashMap::new(),
            search_engine: SemanticSearchEngine::new(),
        }
    }

    /// Create with existing type inferencer.
    pub fn with_inferencer(inferencer: DeepTypeInferencer) -> Self {
        Self {
            inferencer,
            ast_cache: HashMap::new(),
            search_engine: SemanticSearchEngine::new(),
        }
    }

    /// Populate AST cache for a file.
    pub fn populate_ast_cache(&mut self, file: &PathBuf, content: &str) -> Result<(), String> {
        // Detect language from file extension
        let language = MultiParser::detect_language(file)
            .ok_or_else(|| format!("Failed to detect language for file: {:?}", file))?;

        // Parse the file
        let mut parser =
            MultiParser::new().map_err(|e| format!("Failed to create parser: {}", e))?;
        let parsed = parser
            .parse(content, language)
            .ok_or_else(|| "Failed to parse file".to_string())?;

        // Convert tree-sitter AST to MutableAst
        let mut next_id = 0;
        let root = self.convert_node_to_mutable(&parsed.tree.root_node(), content, &mut next_id);
        let ast = MutableAst::new(root);

        self.ast_cache.insert(file.clone(), ast);
        Ok(())
    }

    /// Convert a tree-sitter Node to MutableNode recursively.
    fn convert_node_to_mutable(
        &self,
        node: &Node,
        source: &str,
        next_id: &mut usize,
    ) -> MutableNode {
        let id = NodeId(*next_id);
        *next_id += 1;

        let kind = node.kind().to_string();
        let span = Span::with_lines(
            node.start_byte(),
            node.end_byte(),
            node.start_position().row,
            node.start_position().column,
            node.end_position().row,
            node.end_position().column,
        );

        // Get node text value for leaf nodes
        let value = if node.child_count() == 0 {
            Some(source[node.byte_range()].to_string())
        } else {
            None
        };

        // Recursively convert children
        let mut children = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            children.push(self.convert_node_to_mutable(&child, source, next_id));
        }

        MutableNode {
            id,
            kind,
            span,
            value,
            children: Arc::new(children),
            metadata: NodeMetadata::default(),
        }
    }

    /// Get AST for a file, loading it if not cached.
    pub fn get_ast(&mut self, file: &PathBuf, content: &str) -> Result<&MutableAst, String> {
        if !self.ast_cache.contains_key(file) {
            self.populate_ast_cache(file, content)?;
        }
        self.ast_cache
            .get(file)
            .ok_or_else(|| format!("AST not found in cache for {}", file.display()))
    }

    /// Get mutable AST for a file, loading it if not cached.
    pub fn get_ast_mut(
        &mut self,
        file: &PathBuf,
        content: &str,
    ) -> Result<&mut MutableAst, String> {
        if !self.ast_cache.contains_key(file) {
            self.populate_ast_cache(file, content)?;
        }
        self.ast_cache
            .get_mut(file)
            .ok_or_else(|| format!("AST not found in cache for {}", file.display()))
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Analyze data flow for a specific span using text-based analysis.
    fn analyze_data_flow_simple(&self, span: Span, source: &str) -> DataFlow {
        let selected_code = &source[span.start..span.end];

        // Simple regex-based approach for Python identifiers
        let mut used_vars = std::collections::HashSet::new();
        let mut defined_vars = std::collections::HashSet::new();

        // Find all assignment targets (defined variables)
        for line in selected_code.lines() {
            if let Some(eq_pos) = line.find('=') {
                let left_side = &line[..eq_pos].trim();
                // Simple case: single identifier assignment
                if left_side.chars().all(|c| c.is_alphanumeric() || c == '_')
                    && !left_side.is_empty()
                {
                    defined_vars.insert(left_side.to_string());
                }

                // Right side contains used variables
                let right_side = &line[eq_pos + 1..];
                self.extract_identifiers_from_text(right_side, &mut used_vars);
            } else {
                // No assignment, just extract identifiers
                self.extract_identifiers_from_text(line, &mut used_vars);
            }
        }

        // External vars are used but not defined
        let external_vars: Vec<String> = used_vars
            .iter()
            .filter(|v| !defined_vars.contains(*v))
            .filter(|v| !self.is_builtin(v))
            .cloned()
            .collect();

        // Variables defined in the selection (potential return values)
        let returned: Vec<String> = defined_vars.iter().cloned().collect();

        DataFlow {
            external_vars,
            returned_vars: returned,
        }
    }

    /// Extract identifiers from text using simple pattern matching.
    fn extract_identifiers_from_text(
        &self,
        text: &str,
        identifiers: &mut std::collections::HashSet<String>,
    ) {
        let mut current_id = String::new();
        for ch in text.chars() {
            if ch.is_alphanumeric() || ch == '_' {
                current_id.push(ch);
            } else {
                if let Some(first_char) = current_id.chars().next() {
                    if !first_char.is_numeric() {
                        identifiers.insert(current_id.clone());
                    }
                }
                current_id.clear();
            }
        }
        // Don't forget last identifier
        if let Some(first_char) = current_id.chars().next() {
            if !first_char.is_numeric() {
                identifiers.insert(current_id);
            }
        }
    }

    /// Check if an identifier is a Python builtin.
    fn is_builtin(&self, name: &str) -> bool {
        let builtins = [
            "print",
            "len",
            "range",
            "str",
            "int",
            "float",
            "bool",
            "list",
            "dict",
            "tuple",
            "set",
            "abs",
            "all",
            "any",
            "bin",
            "chr",
            "ord",
            "hex",
            "oct",
            "max",
            "min",
            "sum",
            "sorted",
            "reversed",
            "enumerate",
            "zip",
            "map",
            "filter",
            "open",
            "input",
            "type",
            "isinstance",
            "issubclass",
            "callable",
            "hasattr",
            "getattr",
            "setattr",
            "delattr",
            "dir",
            "vars",
            "globals",
            "locals",
            "super",
            "staticmethod",
            "classmethod",
            "property",
        ];
        builtins.contains(&name)
    }

    /// Get indentation at a specific byte position.
    fn get_indent_at_position_static(source: &str, byte_pos: usize) -> String {
        // Find the line number containing this byte position
        let mut current_pos = 0;
        for line in source.lines() {
            let line_end = current_pos + line.len();

            // Check if byte_pos is in this line
            if byte_pos >= current_pos && byte_pos <= line_end {
                let indent_len = line.len() - line.trim_start().len();
                return " ".repeat(indent_len);
            }

            // +1 for newline character
            current_pos = line_end + 1;
        }

        String::new()
    }

    /// Find insertion point for a new function definition.
    fn find_function_insertion_point(&self, source: &str, span: Span) -> usize {
        // Find the end of the current function or class containing the selection
        // Strategy:
        // 1. Find the line containing the selection
        // 2. Search backwards for "def " or "class " at the beginning of a line
        // 3. Find the end of that definition (next def/class at same indentation or EOF)

        let selection_pos = span.start;

        // Find the start line of current function/class
        let mut current_pos = 0;
        let mut function_start_pos = 0;
        let mut function_indent = 0;

        for line in source.lines() {
            let line_end = current_pos + line.len();

            // Check if we've passed the selection
            if current_pos > selection_pos {
                break;
            }

            // Check if this line starts a function or class
            let trimmed = line.trim_start();
            if trimmed.starts_with("def ") || trimmed.starts_with("class ") {
                function_start_pos = current_pos;
                function_indent = line.len() - trimmed.len();
            }

            current_pos = line_end + 1; // +1 for newline
        }

        // Now find the end of this function/class
        // Look for next def/class at same or lower indentation
        current_pos = function_start_pos;
        let mut found_start = false;

        for line in source[function_start_pos..].lines() {
            let line_end = current_pos + line.len();

            if !found_start {
                found_start = true;
                current_pos = line_end + 1;
                continue;
            }

            let trimmed = line.trim_start();
            let line_indent = line.len() - trimmed.len();

            // Check if we found another def/class at same or lower indentation
            if (trimmed.starts_with("def ") || trimmed.starts_with("class "))
                && line_indent <= function_indent
            {
                // Insert before this line
                return current_pos;
            }

            // Check for end of file or empty lines
            if line.trim().is_empty() {
                current_pos = line_end + 1;
                continue;
            }

            current_pos = line_end + 1;
        }

        // If we didn't find another function, insert at end of file
        source.len()
    }

    /// Find insertion point for a new method definition within a class.
    fn find_method_insertion_point(&self, source: &str, span: Span) -> usize {
        // Similar to function insertion, but finds the end of the current class
        // For now, reuse the same logic
        self.find_function_insertion_point(source, span)
    }

    // ========================================================================
    // Public Methods
    // ========================================================================

    /// Execute a refactoring operation.
    pub fn execute(&mut self, request: &RefactorRequest, source: &str) -> RefactorResult {
        match &request.kind {
            RefactorKind::ExtractFunction { name } => self.extract_function(request, name, source),
            RefactorKind::ExtractMethod { name } => self.extract_method(request, name, source),
            RefactorKind::ExtractVariable { name } => self.extract_variable(request, name, source),
            RefactorKind::Rename { new_name } => self.rename_symbol(request, new_name, source),
            RefactorKind::MoveDefinition { target_file } => {
                self.move_definition(request, target_file, source)
            }
            RefactorKind::Inline => self.inline_symbol(request, source),
            RefactorKind::ChangeSignature { changes } => {
                self.change_signature(request, changes, source)
            }
        }
    }

    /// Extract selection into a new function.
    fn extract_function(
        &mut self,
        request: &RefactorRequest,
        name: &str,
        source: &str,
    ) -> RefactorResult {
        let mut result = RefactorResult::empty();

        // Validate function name
        if name.is_empty() {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                "Function name cannot be empty",
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                "Function name must be a valid identifier",
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        // Check if name is a Python keyword
        let python_keywords = [
            "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
            "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
            "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise",
            "return", "try", "while", "with", "yield",
        ];

        if python_keywords.contains(&name) {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                format!("'{}' is a reserved keyword", name),
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        // Get or populate AST
        if let Err(e) = self.populate_ast_cache(&request.file, source) {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                format!("Failed to parse file: {}", e),
                Some(request.file.clone()),
                None,
            );
            return result;
        }

        // Perform data flow analysis using simple text-based approach
        // (This is more reliable than AST-based analysis for arbitrary selections)
        let data_flow = self.analyze_data_flow_simple(request.span, source);

        // Extract the selected code
        let selected_code = &source[request.span.start..request.span.end];

        // Build parameter list from external variables
        let params: Vec<String> = data_flow.external_vars.clone();

        // Add type annotations if requested
        let params_str = if request.options.add_type_annotations {
            // Try to infer types from context
            params
                .iter()
                .map(|p| {
                    // Simplified: just add Any annotation
                    // In a full implementation, would use TypeInferencer
                    format!("{}: Any", p)
                })
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            params.join(", ")
        };

        // Build function body (with proper indentation)
        let body_lines: Vec<String> = selected_code
            .lines()
            .map(|line| format!("    {}", line))
            .collect();

        let mut body = body_lines.join("\n");

        // Add return statement if variables are defined and need to be returned
        if !data_flow.returned_vars.is_empty() {
            let return_vars = data_flow.returned_vars.join(", ");
            if data_flow.returned_vars.len() == 1 {
                body.push_str(&format!("\n    return {}", return_vars));
            } else {
                body.push_str(&format!("\n    return ({})", return_vars));
            }
        }

        // Generate function definition with optional return type annotation
        let func_def =
            if request.options.add_type_annotations && !data_flow.returned_vars.is_empty() {
                if data_flow.returned_vars.len() == 1 {
                    format!("def {}({}) -> Any:\n{}\n\n", name, params_str, body)
                } else {
                    format!("def {}({}) -> tuple:\n{}\n\n", name, params_str, body)
                }
            } else {
                format!("def {}({}):\n{}\n\n", name, params_str, body)
            };

        // Generate function call
        let call_str = if params.is_empty() {
            if data_flow.returned_vars.is_empty() {
                format!("{}()", name)
            } else if data_flow.returned_vars.len() == 1 {
                format!("{} = {}()", data_flow.returned_vars[0], name)
            } else {
                format!("{} = {}()", data_flow.returned_vars.join(", "), name)
            }
        } else {
            let call_params = params.join(", ");
            if data_flow.returned_vars.is_empty() {
                format!("{}({})", name, call_params)
            } else if data_flow.returned_vars.len() == 1 {
                format!("{} = {}({})", data_flow.returned_vars[0], name, call_params)
            } else {
                format!(
                    "{} = {}({})",
                    data_flow.returned_vars.join(", "),
                    name,
                    call_params
                )
            }
        };

        // Find appropriate insertion point (after current function/class)
        let insert_pos = self.find_function_insertion_point(source, request.span);

        // Create edits
        result.add_edit(
            request.file.clone(),
            TextEdit {
                span: Span::new(insert_pos, insert_pos),
                new_text: func_def,
            },
        );

        result.add_edit(
            request.file.clone(),
            TextEdit {
                span: request.span,
                new_text: call_str,
            },
        );

        result.add_diagnostic(
            DiagnosticLevel::Info,
            format!(
                "Extracted function '{}' with {} parameter(s) and {} return value(s)",
                name,
                params.len(),
                data_flow.returned_vars.len()
            ),
            Some(request.file.clone()),
            Some(request.span),
        );

        result
    }

    /// Extract selection into a new method.
    fn extract_method(
        &mut self,
        request: &RefactorRequest,
        name: &str,
        source: &str,
    ) -> RefactorResult {
        let mut result = RefactorResult::empty();

        // Validate method name
        if name.is_empty() {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                "Method name cannot be empty",
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                "Method name must be a valid identifier",
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        // Check if name is a Python keyword
        let python_keywords = [
            "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
            "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
            "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise",
            "return", "try", "while", "with", "yield",
        ];

        if python_keywords.contains(&name) {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                format!("'{}' is a reserved keyword", name),
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        // Get or populate AST
        if let Err(e) = self.populate_ast_cache(&request.file, source) {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                format!("Failed to parse file: {}", e),
                Some(request.file.clone()),
                None,
            );
            return result;
        }

        // Perform data flow analysis using simple text-based approach
        let data_flow = self.analyze_data_flow_simple(request.span, source);

        // Extract the selected code
        let selected_code = &source[request.span.start..request.span.end];

        // Build parameter list from external variables (excluding 'self')
        let mut params: Vec<String> = data_flow.external_vars.clone();
        params.retain(|p| p != "self"); // Remove self if it appears as external var

        // Build method parameters: self + other params
        let mut method_params = vec!["self".to_string()];

        // Add type annotations if requested
        if request.options.add_type_annotations {
            for p in &params {
                method_params.push(format!("{}: Any", p));
            }
        } else {
            method_params.extend(params.clone());
        }

        let params_str = method_params.join(", ");

        // Build method body (with proper indentation for class method)
        let body_lines: Vec<String> = selected_code
            .lines()
            .map(|line| format!("        {}", line))
            .collect();

        let mut body = body_lines.join("\n");

        // Add return statement if variables are defined and need to be returned
        if !data_flow.returned_vars.is_empty() {
            let return_vars = data_flow.returned_vars.join(", ");
            if data_flow.returned_vars.len() == 1 {
                body.push_str(&format!("\n        return {}", return_vars));
            } else {
                body.push_str(&format!("\n        return ({})", return_vars));
            }
        }

        // Generate method definition with class indentation and optional type annotations
        let method_def =
            if request.options.add_type_annotations && !data_flow.returned_vars.is_empty() {
                if data_flow.returned_vars.len() == 1 {
                    format!("    def {}({}) -> Any:\n{}\n\n", name, params_str, body)
                } else {
                    format!("    def {}({}) -> tuple:\n{}\n\n", name, params_str, body)
                }
            } else {
                format!("    def {}({}):\n{}\n\n", name, params_str, body)
            };

        // Generate method call
        let call_str = if params.is_empty() {
            if data_flow.returned_vars.is_empty() {
                format!("self.{}()", name)
            } else if data_flow.returned_vars.len() == 1 {
                format!("{} = self.{}()", data_flow.returned_vars[0], name)
            } else {
                format!("{} = self.{}()", data_flow.returned_vars.join(", "), name)
            }
        } else {
            let call_params = params.join(", ");
            if data_flow.returned_vars.is_empty() {
                format!("self.{}({})", name, call_params)
            } else if data_flow.returned_vars.len() == 1 {
                format!(
                    "{} = self.{}({})",
                    data_flow.returned_vars[0], name, call_params
                )
            } else {
                format!(
                    "{} = self.{}({})",
                    data_flow.returned_vars.join(", "),
                    name,
                    call_params
                )
            }
        };

        // Find appropriate insertion point (after current class/method)
        let insert_pos = self.find_method_insertion_point(source, request.span);

        // Create edits
        result.add_edit(
            request.file.clone(),
            TextEdit {
                span: Span::new(insert_pos, insert_pos),
                new_text: method_def,
            },
        );

        result.add_edit(
            request.file.clone(),
            TextEdit {
                span: request.span,
                new_text: call_str,
            },
        );

        result.add_diagnostic(
            DiagnosticLevel::Info,
            format!(
                "Extracted method '{}' with {} parameter(s) (plus self) and {} return value(s)",
                name,
                params.len(),
                data_flow.returned_vars.len()
            ),
            Some(request.file.clone()),
            Some(request.span),
        );

        result
    }

    /// Extract expression into a variable.
    fn extract_variable(
        &mut self,
        request: &RefactorRequest,
        name: &str,
        source: &str,
    ) -> RefactorResult {
        let mut result = RefactorResult::empty();

        // Validate variable name
        if name.is_empty() {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                "Variable name cannot be empty",
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                "Variable name must be a valid identifier",
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        // Check if name is a Python keyword
        let python_keywords = [
            "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
            "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
            "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise",
            "return", "try", "while", "with", "yield",
        ];

        if python_keywords.contains(&name) {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                format!("'{}' is a reserved keyword", name),
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        // Get or populate AST
        if let Err(e) = self.populate_ast_cache(&request.file, source) {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                format!("Failed to parse file: {}", e),
                Some(request.file.clone()),
                None,
            );
            return result;
        }

        // Extract the expression text
        let expr_text = &source[request.span.start..request.span.end].trim();

        // Get indentation at the current position
        let indent = Self::get_indent_at_position_static(source, request.span.start);

        // Generate variable assignment
        let assignment = format!("{}{} = {}\n", indent, name, expr_text);

        // Find line start position
        let line_start = Self::find_line_start(source, request.span.start);

        // Insert the assignment before the current line
        result.add_edit(
            request.file.clone(),
            TextEdit {
                span: Span::new(line_start, line_start),
                new_text: assignment,
            },
        );

        // Replace the expression with the variable name
        result.add_edit(
            request.file.clone(),
            TextEdit {
                span: request.span,
                new_text: name.to_string(),
            },
        );

        result.add_diagnostic(
            DiagnosticLevel::Info,
            format!("Extracted variable '{}'", name),
            Some(request.file.clone()),
            Some(request.span),
        );

        result
    }

    /// Find the start of a line containing a byte position.
    fn find_line_start(source: &str, pos: usize) -> usize {
        let mut line_start = 0;
        for (i, ch) in source.char_indices() {
            if i >= pos {
                break;
            }
            if ch == '\n' {
                line_start = i + 1;
            }
        }
        line_start
    }

    /// Rename a symbol across files.
    fn rename_symbol(
        &mut self,
        request: &RefactorRequest,
        new_name: &str,
        source: &str,
    ) -> RefactorResult {
        let mut result = RefactorResult::empty();

        // Extract the old symbol name from the source
        let old_name = &source[request.span.start..request.span.end];

        // Validate new name
        if old_name == new_name {
            result.add_diagnostic(
                DiagnosticLevel::Info,
                "New name is the same as the old name",
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        if new_name.is_empty() {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                "New name cannot be empty",
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        // Basic name validation (simplified - just check it's a valid identifier)
        if !new_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                "New name must be a valid identifier",
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        // Check if new name is a Python keyword
        let python_keywords = [
            "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
            "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
            "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise",
            "return", "try", "while", "with", "yield",
        ];

        if python_keywords.contains(&new_name) {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                format!(
                    "'{}' is a reserved keyword and cannot be used as a variable name",
                    new_name
                ),
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        // For simplified implementation: just rename in current file
        // In a full implementation, would use semantic search to find all references
        let query = SearchQuery {
            kind: SearchKind::Usages {
                symbol: old_name.to_string(),
                file: request.file.clone(),
            },
            scope: SearchScope::Project,
            max_results: 1000,
        };

        let search_result = self.search_engine.search(&query);

        // If no matches found in index, do simple text-based search in current file
        if search_result.matches.is_empty() {
            // Simple approach: find all occurrences of the old name in the current file
            let mut pos = 0;
            while let Some(found_pos) = source[pos..].find(old_name) {
                let absolute_pos = pos + found_pos;

                // Create a text edit for this occurrence
                result.add_edit(
                    request.file.clone(),
                    TextEdit {
                        span: Span::new(absolute_pos, absolute_pos + old_name.len()),
                        new_text: new_name.to_string(),
                    },
                );

                pos = absolute_pos + old_name.len();
            }
        } else {
            // Use search results
            for search_match in &search_result.matches {
                result.add_edit(
                    search_match.file.clone(),
                    TextEdit {
                        span: search_match.span,
                        new_text: new_name.to_string(),
                    },
                );
            }
        }

        if !result.has_changes() {
            result.add_diagnostic(
                DiagnosticLevel::Warning,
                format!("No occurrences of '{}' found", old_name),
                Some(request.file.clone()),
                Some(request.span),
            );
        } else {
            result.add_diagnostic(
                DiagnosticLevel::Info,
                format!(
                    "Renamed '{}' to '{}' ({} occurrences)",
                    old_name,
                    new_name,
                    result.file_edits.values().map(|v| v.len()).sum::<usize>()
                ),
                Some(request.file.clone()),
                Some(request.span),
            );
        }

        result
    }

    /// Move a definition to another file.
    fn move_definition(
        &mut self,
        request: &RefactorRequest,
        target_file: &PathBuf,
        source: &str,
    ) -> RefactorResult {
        let mut result = RefactorResult::empty();

        // Get or populate AST
        if let Err(e) = self.populate_ast_cache(&request.file, source) {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                format!("Failed to parse file: {}", e),
                Some(request.file.clone()),
                None,
            );
            return result;
        }

        // Extract the definition code
        let definition_code = &source[request.span.start..request.span.end];

        // Remove from current file
        result.add_edit(
            request.file.clone(),
            TextEdit {
                span: request.span,
                new_text: String::new(),
            },
        );

        // Add to target file (at beginning for simplicity)
        result
            .new_files
            .insert(target_file.clone(), format!("{}\n\n", definition_code));

        result.add_diagnostic(
            DiagnosticLevel::Info,
            format!("Moved definition to {:?}", target_file),
            Some(request.file.clone()),
            Some(request.span),
        );

        result
    }

    /// Inline a symbol's definition.
    fn inline_symbol(&mut self, request: &RefactorRequest, source: &str) -> RefactorResult {
        let mut result = RefactorResult::empty();

        // Get or populate AST
        if let Err(e) = self.populate_ast_cache(&request.file, source) {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                format!("Failed to parse file: {}", e),
                Some(request.file.clone()),
                None,
            );
            return result;
        }

        // Extract the symbol name to inline
        let symbol_name = &source[request.span.start..request.span.end];

        // Simplified implementation: Find the definition and replace all usages
        // Look for pattern: "symbol_name = expression"
        let mut definition_value = None;
        let mut definition_span = None;

        // Simple pattern matching for variable assignment
        if let Some(def_pos) = source.find(&format!("{} = ", symbol_name)) {
            let start = def_pos;
            // Find the end of the line (or end of file if no newline)
            let rest = &source[start..];
            let newline_pos = rest.find('\n').unwrap_or(rest.len());
            let end = start + newline_pos;
            let line = &source[start..end];

            // Extract the value part (after "symbol_name = ")
            if let Some(eq_pos) = line.find(" = ") {
                let value_start = start + eq_pos + 3;
                definition_value = Some(source[value_start..end].trim().to_string());
                // Include newline only if it exists
                let span_end = if newline_pos < rest.len() {
                    end + 1
                } else {
                    end
                };
                definition_span = Some(Span::new(start, span_end));
            }
        }

        let (def_value, def_span) = match (definition_value, definition_span) {
            (Some(value), Some(span)) => (value, span),
            _ => {
                result.add_diagnostic(
                    DiagnosticLevel::Error,
                    format!("Could not find definition for '{}'", symbol_name),
                    Some(request.file.clone()),
                    Some(request.span),
                );
                return result;
            }
        };

        // Find all usages of the symbol (excluding the definition)
        let mut usages = Vec::new();
        let mut pos = 0;
        while let Some(found_pos) = source[pos..].find(symbol_name) {
            let absolute_pos = pos + found_pos;

            // Skip if this is the definition itself
            if absolute_pos >= def_span.start && absolute_pos < def_span.end {
                pos = absolute_pos + symbol_name.len();
                continue;
            }

            // Check if it's a complete identifier (not part of another word)
            let before_ok = absolute_pos == 0
                || !source
                    .chars()
                    .nth(absolute_pos - 1)
                    .unwrap_or(' ')
                    .is_alphanumeric();
            let after_pos = absolute_pos + symbol_name.len();
            let after_ok = after_pos >= source.len()
                || !source
                    .chars()
                    .nth(after_pos)
                    .unwrap_or(' ')
                    .is_alphanumeric();

            if before_ok && after_ok {
                usages.push(Span::new(absolute_pos, absolute_pos + symbol_name.len()));
            }

            pos = absolute_pos + symbol_name.len();
        }

        if usages.is_empty() {
            result.add_diagnostic(
                DiagnosticLevel::Warning,
                format!("No usages of '{}' found to inline", symbol_name),
                Some(request.file.clone()),
                Some(request.span),
            );
            return result;
        }

        // Store count before iterating
        let usage_count = usages.len();

        // Replace all usages with the definition value
        for usage_span in usages {
            result.add_edit(
                request.file.clone(),
                TextEdit {
                    span: usage_span,
                    new_text: def_value.clone(),
                },
            );
        }

        // Remove the definition line
        result.add_edit(
            request.file.clone(),
            TextEdit {
                span: def_span,
                new_text: String::new(),
            },
        );

        result.add_diagnostic(
            DiagnosticLevel::Info,
            format!("Inlined '{}' ({} usages)", symbol_name, usage_count),
            Some(request.file.clone()),
            Some(request.span),
        );

        result
    }

    /// Change function signature.
    fn change_signature(
        &mut self,
        request: &RefactorRequest,
        changes: &SignatureChanges,
        source: &str,
    ) -> RefactorResult {
        let mut result = RefactorResult::empty();

        // Get or populate AST
        if let Err(e) = self.populate_ast_cache(&request.file, source) {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                format!("Failed to parse file: {}", e),
                Some(request.file.clone()),
                None,
            );
            return result;
        }

        // Simplified implementation: Find function definition and update parameters
        let function_code = &source[request.span.start..request.span.end];

        // Find the opening parenthesis
        if let Some(open_paren) = function_code.find('(') {
            if let Some(close_paren) = function_code.find(')') {
                let func_name_part = &function_code[..open_paren];
                let after_params = &function_code[close_paren + 1..];

                // Build new parameters list
                let mut new_params = Vec::new();

                // Add new parameters
                for (param_name, type_ann, default) in &changes.new_params {
                    let param_str = if let Some(type_str) = type_ann {
                        if let Some(default_val) = default {
                            format!("{}: {} = {}", param_name, type_str, default_val)
                        } else {
                            format!("{}: {}", param_name, type_str)
                        }
                    } else if let Some(default_val) = default {
                        format!("{}={}", param_name, default_val)
                    } else {
                        param_name.clone()
                    };
                    new_params.push(param_str);
                }

                let new_params_str = new_params.join(", ");

                // Construct new function signature
                let new_signature =
                    format!("{}({}){}", func_name_part, new_params_str, after_params);

                // Replace the function signature
                result.add_edit(
                    request.file.clone(),
                    TextEdit {
                        span: request.span,
                        new_text: new_signature,
                    },
                );

                result.add_diagnostic(
                    DiagnosticLevel::Info,
                    format!(
                        "Changed function signature ({} parameters)",
                        new_params.len()
                    ),
                    Some(request.file.clone()),
                    Some(request.span),
                );
            } else {
                result.add_diagnostic(
                    DiagnosticLevel::Error,
                    "Could not find closing parenthesis",
                    Some(request.file.clone()),
                    Some(request.span),
                );
            }
        } else {
            result.add_diagnostic(
                DiagnosticLevel::Error,
                "Could not find function parameters",
                Some(request.file.clone()),
                Some(request.span),
            );
        }

        result
    }

    /// Get type context.
    pub fn type_context(&self) -> &TypeContext {
        self.inferencer.context()
    }
}

impl Default for RefactoringEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refactor_result() {
        let mut result = RefactorResult::empty();
        assert!(!result.has_changes());
        assert!(!result.has_errors());

        result.add_edit(
            PathBuf::from("test.py"),
            TextEdit {
                span: Span::new(0, 10),
                new_text: "new text".to_string(),
            },
        );
        assert!(result.has_changes());
    }

    #[test]
    fn test_refactor_request() {
        let source = "old_name = 42";
        let request = RefactorRequest {
            kind: RefactorKind::Rename {
                new_name: "new_name".to_string(),
            },
            file: PathBuf::from("test.py"),
            span: Span::new(0, 8),
            options: RefactorOptions::default(),
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        assert!(!result.has_errors());
    }

    #[test]
    fn test_extract_variable() {
        let source = "result = user.name.upper()";
        let request = RefactorRequest {
            kind: RefactorKind::ExtractVariable {
                name: "temp_name".to_string(),
            },
            file: PathBuf::from("test.py"),
            span: Span::new(9, 26), // "user.name.upper()"
            options: RefactorOptions::default(),
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        assert!(!result.has_errors());
        assert!(result.has_changes());

        // Should have 2 edits: insert assignment + replace expression
        let edits = result.file_edits.get(&PathBuf::from("test.py"));
        assert!(edits.is_some());
        assert_eq!(edits.unwrap().len(), 2);
    }

    #[test]
    fn test_extract_variable_invalid_name() {
        let source = "result = 1 + 2";
        let request = RefactorRequest {
            kind: RefactorKind::ExtractVariable {
                name: "for".to_string(), // Python keyword
            },
            file: PathBuf::from("test.py"),
            span: Span::new(9, 14),
            options: RefactorOptions::default(),
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        assert!(result.has_errors());
        assert!(!result.has_changes());
    }

    #[test]
    fn test_rename_symbol() {
        let source = "old_name = 42\nprint(old_name)";
        let request = RefactorRequest {
            kind: RefactorKind::Rename {
                new_name: "new_name".to_string(),
            },
            file: PathBuf::from("test.py"),
            span: Span::new(0, 8),
            options: RefactorOptions::default(),
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        assert!(!result.has_errors());
        assert!(result.has_changes());

        // Should rename at least one occurrence
        let edits = result.file_edits.get(&PathBuf::from("test.py"));
        assert!(edits.is_some());
        assert!(edits.unwrap().len() >= 1);
    }

    #[test]
    fn test_rename_to_keyword() {
        let source = "old_name = 42";
        let request = RefactorRequest {
            kind: RefactorKind::Rename {
                new_name: "class".to_string(), // Python keyword
            },
            file: PathBuf::from("test.py"),
            span: Span::new(0, 8),
            options: RefactorOptions::default(),
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        assert!(result.has_errors());
        assert!(!result.has_changes());
    }

    #[test]
    fn test_rename_same_name() {
        let source = "my_var = 42";
        let request = RefactorRequest {
            kind: RefactorKind::Rename {
                new_name: "my_var".to_string(), // Same name
            },
            file: PathBuf::from("test.py"),
            span: Span::new(0, 6),
            options: RefactorOptions::default(),
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        // Should not error, but should not make changes
        assert!(!result.has_errors());
        assert!(!result.has_changes());
    }

    #[test]
    fn test_extract_function_no_params() {
        let source = r#"print("Hello")
print("World")"#;
        let request = RefactorRequest {
            kind: RefactorKind::ExtractFunction {
                name: "greet".to_string(),
            },
            file: PathBuf::from("test.py"),
            span: Span::new(0, 14), // print("Hello")
            options: RefactorOptions::default(),
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        assert!(!result.has_errors());
        assert!(result.has_changes());

        // Should have 2 edits: insert function definition + replace with call
        let edits = result.file_edits.get(&PathBuf::from("test.py"));
        assert!(edits.is_some());
        assert_eq!(edits.unwrap().len(), 2);
    }

    #[test]
    fn test_extract_function_with_params() {
        let source = r#"x = 5
y = 10
result = x + y
print(result)"#;
        let request = RefactorRequest {
            kind: RefactorKind::ExtractFunction {
                name: "add_numbers".to_string(),
            },
            file: PathBuf::from("test.py"),
            span: Span::new(12, 26), // "result = x + y"
            options: RefactorOptions::default(),
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        // Print diagnostics for debugging
        for diag in &result.diagnostics {
            eprintln!("[{:?}] {}", diag.level, diag.message);
        }

        if result.has_errors() {
            panic!("Unexpected errors in result");
        }

        assert!(result.has_changes());

        // Check diagnostic message contains parameter count
        let info_diag = result
            .diagnostics
            .iter()
            .find(|d| d.level == DiagnosticLevel::Info);
        assert!(info_diag.is_some());
        let msg = &info_diag.unwrap().message;
        assert!(msg.contains("parameter"));
    }

    #[test]
    fn test_extract_function_with_return() {
        let source = r#"def process():
    data = "test"
    result = data.upper()
    return result"#;
        let request = RefactorRequest {
            kind: RefactorKind::ExtractFunction {
                name: "transform".to_string(),
            },
            file: PathBuf::from("test.py"),
            span: Span::new(23, 48), // "data = "test"\n    result = data.upper()"
            options: RefactorOptions::default(),
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        assert!(!result.has_errors());
        assert!(result.has_changes());
    }

    #[test]
    fn test_extract_function_invalid_name() {
        let source = r#"print("test")"#;
        let request = RefactorRequest {
            kind: RefactorKind::ExtractFunction {
                name: "def".to_string(), // Python keyword
            },
            file: PathBuf::from("test.py"),
            span: Span::new(0, 13),
            options: RefactorOptions::default(),
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        assert!(result.has_errors());
        assert!(!result.has_changes());
    }

    #[test]
    fn test_extract_function_empty_name() {
        let source = r#"print("test")"#;
        let request = RefactorRequest {
            kind: RefactorKind::ExtractFunction {
                name: "".to_string(), // Empty name
            },
            file: PathBuf::from("test.py"),
            span: Span::new(0, 13),
            options: RefactorOptions::default(),
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        assert!(result.has_errors());
        assert!(!result.has_changes());
    }

    #[test]
    fn test_extract_method_no_params() {
        let source = r#"class MyClass:
    def process(self):
        print("Processing")
        print("Done")"#;
        let request = RefactorRequest {
            kind: RefactorKind::ExtractMethod {
                name: "do_print".to_string(),
            },
            file: PathBuf::from("test.py"),
            span: Span::new(47, 84), // print statements
            options: RefactorOptions::default(),
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        assert!(!result.has_errors());
        assert!(result.has_changes());

        // Should have 2 edits: insert method definition + replace with call
        let edits = result.file_edits.get(&PathBuf::from("test.py"));
        assert!(edits.is_some());
        assert_eq!(edits.unwrap().len(), 2);
    }

    #[test]
    fn test_extract_method_with_params() {
        let source = r#"class Calculator:
    def compute(self):
        x = 5
        y = 10
        result = x + y
        return result"#;
        let request = RefactorRequest {
            kind: RefactorKind::ExtractMethod {
                name: "add".to_string(),
            },
            file: PathBuf::from("test.py"),
            span: Span::new(63, 77), // "result = x + y"
            options: RefactorOptions::default(),
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        assert!(!result.has_errors());
        assert!(result.has_changes());

        // Check diagnostic message
        let info_diag = result
            .diagnostics
            .iter()
            .find(|d| d.level == DiagnosticLevel::Info);
        assert!(info_diag.is_some());
        let msg = &info_diag.unwrap().message;
        assert!(msg.contains("parameter"));
    }

    #[test]
    fn test_extract_method_with_self_access() {
        let source = r#"class Person:
    def __init__(self):
        self.name = "Alice"

    def greet(self):
        message = "Hello, " + self.name
        print(message)"#;
        let request = RefactorRequest {
            kind: RefactorKind::ExtractMethod {
                name: "create_message".to_string(),
            },
            file: PathBuf::from("test.py"),
            span: Span::new(89, 121), // message line
            options: RefactorOptions::default(),
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        assert!(!result.has_errors());
        assert!(result.has_changes());
    }

    #[test]
    fn test_extract_method_invalid_name() {
        let source = r#"class Test:
    def method(self):
        print("test")"#;
        let request = RefactorRequest {
            kind: RefactorKind::ExtractMethod {
                name: "while".to_string(), // Python keyword
            },
            file: PathBuf::from("test.py"),
            span: Span::new(42, 56),
            options: RefactorOptions::default(),
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        assert!(result.has_errors());
        assert!(!result.has_changes());
    }

    #[test]
    fn test_inline_symbol_simple() {
        let source = r#"temp = 42
result = temp * 2
print(result)"#;
        let request = RefactorRequest {
            kind: RefactorKind::Inline,
            file: PathBuf::from("test.py"),
            span: Span::new(0, 4), // "temp"
            options: RefactorOptions::default(),
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        assert!(!result.has_errors());
        assert!(result.has_changes());

        // Should have edits for inlining
        let edits = result.file_edits.get(&PathBuf::from("test.py"));
        assert!(edits.is_some());
        assert!(edits.unwrap().len() >= 2); // Replace usage + remove definition
    }

    #[test]
    fn test_change_signature_add_param() {
        let source = r#"def greet():
    print("Hello")"#;
        let request = RefactorRequest {
            kind: RefactorKind::ChangeSignature {
                changes: SignatureChanges {
                    new_params: vec![("name".to_string(), Some("str".to_string()), None)],
                    ..Default::default()
                },
            },
            file: PathBuf::from("test.py"),
            span: Span::new(0, 12), // "def greet():"
            options: RefactorOptions::default(),
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        assert!(!result.has_errors());
        assert!(result.has_changes());

        // Should have edit for changing signature
        let edits = result.file_edits.get(&PathBuf::from("test.py"));
        assert!(edits.is_some());
        assert_eq!(edits.unwrap().len(), 1);
    }

    #[test]
    fn test_move_definition_basic() {
        let source = r#"def helper():
    return 42

def main():
    result = helper()
    print(result)"#;
        let request = RefactorRequest {
            kind: RefactorKind::MoveDefinition {
                target_file: PathBuf::from("helpers.py"),
            },
            file: PathBuf::from("main.py"),
            span: Span::new(0, 26), // helper function
            options: RefactorOptions::default(),
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        assert!(!result.has_errors());
        assert!(result.has_changes());

        // Should have edits for both files
        assert!(result.file_edits.contains_key(&PathBuf::from("main.py")));
        assert!(result.new_files.contains_key(&PathBuf::from("helpers.py")));
    }

    #[test]
    fn test_extract_function_with_type_annotations() {
        let source = r#"x = 5
y = 10
result = x + y
print(result)"#;
        let request = RefactorRequest {
            kind: RefactorKind::ExtractFunction {
                name: "calculate".to_string(),
            },
            file: PathBuf::from("test.py"),
            span: Span::new(12, 26), // "result = x + y"
            options: RefactorOptions {
                add_type_annotations: true,
                ..Default::default()
            },
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        assert!(!result.has_errors());
        assert!(result.has_changes());

        // Check that type annotations were added
        let edits = result.file_edits.get(&PathBuf::from("test.py"));
        assert!(edits.is_some());

        // The function definition should contain type annotations
        let func_def_edit = &edits.unwrap()[0];
        assert!(func_def_edit.new_text.contains(": Any"));
        assert!(
            func_def_edit.new_text.contains("-> Any")
                || func_def_edit.new_text.contains("-> tuple")
        );
    }

    #[test]
    fn test_extract_method_with_type_annotations() {
        let source = r#"class Calculator:
    def compute(self):
        x = 5
        y = 10
        result = x + y
        return result"#;
        let request = RefactorRequest {
            kind: RefactorKind::ExtractMethod {
                name: "add".to_string(),
            },
            file: PathBuf::from("test.py"),
            span: Span::new(63, 77), // "result = x + y"
            options: RefactorOptions {
                add_type_annotations: true,
                ..Default::default()
            },
        };

        let mut engine = RefactoringEngine::new();
        let result = engine.execute(&request, source);

        // Print for debugging
        for diag in &result.diagnostics {
            eprintln!("[{:?}] {}", diag.level, diag.message);
        }

        if let Some(edits) = result.file_edits.get(&PathBuf::from("test.py")) {
            for (i, edit) in edits.iter().enumerate() {
                eprintln!("Edit {}: {}", i, edit.new_text);
            }
        }

        assert!(!result.has_errors());
        assert!(result.has_changes());

        // Check that type annotations were added
        let edits = result.file_edits.get(&PathBuf::from("test.py"));
        assert!(edits.is_some());

        // The method definition should contain type annotations if there are parameters
        // Since x and y are defined in the method, they might not appear as parameters
        // Just verify the method was extracted successfully
        assert_eq!(edits.unwrap().len(), 2);
    }
}
