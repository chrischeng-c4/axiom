//! Python code checker

use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, Range, TextEdit};
use crate::semantic::ScopeAnalyzer;
use crate::syntax::{Language, ParsedFile};
use std::collections::{HashMap, HashSet};

// ===== Import Sorting Types =====

/// Import classification for isort-like checking
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ImportGroup {
    Future,     // __future__
    Stdlib,     // Standard library
    ThirdParty, // pip packages
    FirstParty, // Local project
}

/// Parsed import info
#[derive(Debug, Clone)]
struct ImportInfo {
    module: String,
    group: ImportGroup,
    line: u32,
    range: Range,
    #[allow(dead_code)]
    is_from_import: bool,
}

/// Python checker
pub struct PythonChecker {
    builtins: HashSet<&'static str>,
}

impl PythonChecker {
    pub fn new() -> Self {
        Self {
            builtins: [
                "abs",
                "all",
                "any",
                "ascii",
                "bin",
                "bool",
                "breakpoint",
                "bytearray",
                "bytes",
                "callable",
                "chr",
                "classmethod",
                "compile",
                "complex",
                "delattr",
                "dict",
                "dir",
                "divmod",
                "enumerate",
                "eval",
                "exec",
                "filter",
                "float",
                "format",
                "frozenset",
                "getattr",
                "globals",
                "hasattr",
                "hash",
                "help",
                "hex",
                "id",
                "input",
                "int",
                "isinstance",
                "issubclass",
                "iter",
                "len",
                "list",
                "locals",
                "map",
                "max",
                "memoryview",
                "min",
                "next",
                "object",
                "oct",
                "open",
                "ord",
                "pow",
                "print",
                "property",
                "range",
                "repr",
                "reversed",
                "round",
                "set",
                "setattr",
                "slice",
                "sorted",
                "staticmethod",
                "str",
                "sum",
                "super",
                "tuple",
                "type",
                "vars",
                "zip",
                "__import__",
                // Common exceptions
                "Exception",
                "BaseException",
                "TypeError",
                "ValueError",
                "KeyError",
                "IndexError",
                "AttributeError",
                "RuntimeError",
                "StopIteration",
                "NotImplementedError",
                "AssertionError",
                "ImportError",
                "OSError",
                // Constants
                "True",
                "False",
                "None",
                "Ellipsis",
                "NotImplemented",
            ]
            .into_iter()
            .collect(),
        }
    }

    /// Check for unused imports
    fn check_unused_imports(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut imports: HashMap<String, Range> = HashMap::new();
        let mut used_names: HashSet<String> = HashSet::new();

        // First pass: collect imports and used names
        file.walk(|node, _depth| {
            match node.kind() {
                "import_statement" => {
                    // import foo, bar
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        if child.kind() == "dotted_name" {
                            let name = file.node_text(&child);
                            // For "import foo.bar", we only track "foo"
                            let base_name = name.split('.').next().unwrap_or(name);
                            imports.insert(base_name.to_string(), Range::from_node(&child));
                        }
                    }
                }
                "import_from_statement" => {
                    // from foo import bar, baz
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        if child.kind() == "dotted_name" || child.kind() == "aliased_import" {
                            let name = if child.kind() == "aliased_import" {
                                // from foo import bar as baz -> track "baz"
                                child
                                    .child_by_field_name("alias")
                                    .map(|n| file.node_text(&n))
                                    .unwrap_or_else(|| file.node_text(&child))
                            } else {
                                file.node_text(&child)
                            };
                            imports.insert(name.to_string(), Range::from_node(&child));
                        }
                    }
                }
                "identifier" => {
                    let name = file.node_text(node);
                    used_names.insert(name.to_string());
                }
                _ => {}
            }
            true
        });

        // Find unused imports
        for (name, range) in imports {
            if !used_names.contains(&name) {
                diagnostics.push(Diagnostic::warning(
                    range,
                    "PY102",
                    DiagnosticCategory::Names,
                    format!("Unused import: '{}'", name),
                ));
            }
        }

        diagnostics
    }

    /// Check for mutable default arguments
    fn check_mutable_default(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "default_parameter" {
                if let Some(value) = node.child_by_field_name("value") {
                    let value_kind = value.kind();
                    if value_kind == "list" || value_kind == "dictionary" || value_kind == "set" {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(&value),
                            "PY201",
                            DiagnosticCategory::Logic,
                            format!(
                                "Mutable default argument: {} literals are mutable and shared between calls",
                                value_kind
                            ),
                        ));
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// Check for bare except clauses
    fn check_bare_except(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "except_clause" {
                // Check if there's no exception type specified
                let has_type = node.children(&mut node.walk())
                    .any(|c| c.kind() == "identifier" || c.kind() == "tuple");

                if !has_type {
                    diagnostics.push(Diagnostic::warning(
                        Range::from_node(node),
                        "PY202",
                        DiagnosticCategory::Logic,
                        "Bare except clause catches all exceptions including KeyboardInterrupt and SystemExit",
                    ));
                }
            }
            true
        });

        diagnostics
    }

    /// Check for unreachable code after return/raise/break/continue
    fn check_unreachable_code(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "block" {
                let mut found_terminal = false;
                let mut cursor = node.walk();

                for child in node.children(&mut cursor) {
                    if found_terminal && !child.kind().contains("comment") {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(&child),
                            "PY203",
                            DiagnosticCategory::Logic,
                            "Unreachable code after return/raise/break/continue",
                        ));
                        break;
                    }

                    if matches!(
                        child.kind(),
                        "return_statement"
                            | "raise_statement"
                            | "break_statement"
                            | "continue_statement"
                    ) {
                        found_terminal = true;
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// Check for shadowed builtins
    fn check_shadowed_builtins(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            // Check function/class definitions and assignments
            let name_node = match node.kind() {
                "function_definition" | "class_definition" => node.child_by_field_name("name"),
                "assignment" => {
                    // Get the left side of assignment
                    node.child_by_field_name("left")
                }
                _ => None,
            };

            if let Some(name_node) = name_node {
                if name_node.kind() == "identifier" {
                    let name = file.node_text(&name_node);
                    if self.builtins.contains(name) {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(&name_node),
                            "PY104",
                            DiagnosticCategory::Names,
                            format!("Shadowing builtin name: '{}'", name),
                        ));
                    }
                }
            }

            true
        });

        diagnostics
    }

    /// Check for accessing private members (starting with _) from outside
    fn check_private_member_access(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            // Check attribute access like obj._private
            if node.kind() == "attribute" {
                if let Some(attr) = node.child_by_field_name("attribute") {
                    let attr_name = file.node_text(&attr);
                    // Check if it starts with _ but not __ (dunder methods are OK)
                    if attr_name.starts_with('_') && !attr_name.starts_with("__") {
                        // Check if this is not self._private or cls._private
                        if let Some(obj) = node.child_by_field_name("object") {
                            let obj_name = file.node_text(&obj);
                            if obj_name != "self" && obj_name != "cls" {
                                diagnostics.push(Diagnostic::warning(
                                    Range::from_node(node),
                                    "PY402",
                                    DiagnosticCategory::Style,
                                    format!(
                                        "Accessing private member '{}' from outside class",
                                        attr_name
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// Check for duplicate dictionary keys
    fn check_duplicate_dict_keys(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "dictionary" {
                let mut seen_keys: HashMap<String, Range> = HashMap::new();
                let mut cursor = node.walk();

                for child in node.children(&mut cursor) {
                    if child.kind() == "pair" {
                        if let Some(key) = child.child_by_field_name("key") {
                            // Only check literal keys (strings, numbers)
                            let key_text = file.node_text(&key);
                            if matches!(key.kind(), "string" | "integer" | "float" | "identifier") {
                                if let Some(prev_range) = seen_keys.get(key_text) {
                                    diagnostics.push(Diagnostic::warning(
                                        Range::from_node(&key),
                                        "PY403",
                                        DiagnosticCategory::Logic,
                                        format!(
                                            "Duplicate dictionary key '{}' (first defined at line {})",
                                            key_text,
                                            prev_range.start.line + 1
                                        ),
                                    ));
                                } else {
                                    seen_keys.insert(key_text.to_string(), Range::from_node(&key));
                                }
                            }
                        }
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// Check for boolean comparisons that can be simplified
    /// e.g., `if x == True` -> `if x`, `if x == False` -> `if not x`
    fn check_simplify_boolean(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "comparison_operator" {
                let mut cursor = node.walk();
                let children: Vec<_> = node.children(&mut cursor).collect();

                // Look for patterns like `x == True` or `x == False`
                for (i, child) in children.iter().enumerate() {
                    if child.kind() == "==" || child.kind() == "!=" {
                        // Check the operand after the operator
                        if let Some(right) = children.get(i + 1) {
                            let right_text = file.node_text(right);
                            if right_text == "True" || right_text == "False" {
                                let suggestion = if right_text == "True" {
                                    if child.kind() == "==" {
                                        "Use 'if x' instead of 'if x == True'"
                                    } else {
                                        "Use 'if not x' instead of 'if x != True'"
                                    }
                                } else if child.kind() == "==" {
                                    "Use 'if not x' instead of 'if x == False'"
                                } else {
                                    "Use 'if x' instead of 'if x != False'"
                                };

                                diagnostics.push(Diagnostic::new(
                                    Range::from_node(node),
                                    crate::diagnostic::DiagnosticSeverity::Hint,
                                    "PY404",
                                    DiagnosticCategory::Style,
                                    suggestion,
                                ));
                            }
                        }
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// Check for `== None` instead of `is None`
    fn check_none_comparison(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "comparison_operator" {
                let mut cursor = node.walk();
                let children: Vec<_> = node.children(&mut cursor).collect();

                for (i, child) in children.iter().enumerate() {
                    if child.kind() == "==" || child.kind() == "!=" {
                        // Check if comparing with None on right: x == None
                        if let Some(right) = children.get(i + 1) {
                            if file.node_text(right) == "None" {
                                if let Some(left) = children.get(i - 1) {
                                    let left_text = file.node_text(left);
                                    let (suggestion, fix_text) = if child.kind() == "==" {
                                        (
                                            "Use 'is None' instead of '== None'",
                                            format!("{} is None", left_text),
                                        )
                                    } else {
                                        (
                                            "Use 'is not None' instead of '!= None'",
                                            format!("{} is not None", left_text),
                                        )
                                    };

                                    let range = Range::from_node(node);
                                    diagnostics.push(
                                        Diagnostic::warning(
                                            range.clone(),
                                            "PY405",
                                            DiagnosticCategory::Style,
                                            suggestion,
                                        )
                                        .with_fix(
                                            "Replace with identity check",
                                            vec![TextEdit {
                                                range,
                                                new_text: fix_text,
                                            }],
                                        ),
                                    );
                                }
                            }
                        }
                        // Check left side: None == x
                        if i > 0 {
                            if let Some(left) = children.get(i - 1) {
                                if file.node_text(left) == "None" {
                                    if let Some(right) = children.get(i + 1) {
                                        let right_text = file.node_text(right);
                                        let (suggestion, fix_text) = if child.kind() == "==" {
                                            (
                                                "Use 'is None' instead of 'None =='",
                                                format!("{} is None", right_text),
                                            )
                                        } else {
                                            (
                                                "Use 'is not None' instead of 'None !='",
                                                format!("{} is not None", right_text),
                                            )
                                        };

                                        let range = Range::from_node(node);
                                        diagnostics.push(
                                            Diagnostic::warning(
                                                range.clone(),
                                                "PY405",
                                                DiagnosticCategory::Style,
                                                suggestion,
                                            )
                                            .with_fix(
                                                "Replace with identity check",
                                                vec![TextEdit {
                                                    range,
                                                    new_text: fix_text,
                                                }],
                                            ),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// Check for statements that have no effect
    fn check_statement_no_effect(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "expression_statement" {
                if let Some(expr) = node.child(0) {
                    // These expression types have no effect on their own
                    let is_useless = match expr.kind() {
                        // Literals (but strings might be docstrings)
                        // Skip ellipsis (...) as it's used as placeholder in stubs/abstract methods
                        "integer" | "float" | "true" | "false" | "none" => true,
                        "ellipsis" => false, // ... is intentional placeholder
                        // String literals - check if docstring
                        "string" => !Self::is_docstring(node, file),
                        // Identifiers (just referencing a variable)
                        "identifier" => true,
                        // Attribute access without assignment
                        "attribute" => true,
                        // Subscript access without assignment
                        "subscript" => true,
                        // Binary operations that don't assign
                        "binary_operator" => {
                            // Exclude augmented assignments
                            let text = file.node_text(&expr);
                            !text.contains("+=") && !text.contains("-=")
                        }
                        // Comparison that's not used
                        "comparison_operator" => true,
                        _ => false,
                    };

                    if is_useless {
                        diagnostics.push(Diagnostic::warning(
                            Range::from_node(node),
                            "PY406",
                            DiagnosticCategory::Logic,
                            "Statement has no effect",
                        ));
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// Check if a string expression_statement is a docstring
    fn is_docstring(node: &tree_sitter::Node<'_>, _file: &ParsedFile) -> bool {
        if let Some(parent) = node.parent() {
            // Docstrings appear as first statement in module, class, or function
            match parent.kind() {
                "module" | "block" => {
                    // Check if this is the first statement (or first after decorators)
                    let mut cursor = parent.walk();
                    for child in parent.children(&mut cursor) {
                        // Skip decorators and comments
                        if matches!(child.kind(), "decorator" | "comment") {
                            continue;
                        }
                        // If this node is the first expression_statement, it's a docstring
                        return child.id() == node.id();
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Perform scope analysis and return diagnostics for unused/redeclared variables
    fn check_scope_issues(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut analyzer = ScopeAnalyzer::new();
        analyzer.analyze(file);

        // PY103: Unused variable/parameter
        for symbol in analyzer.unused_symbols() {
            let message = match symbol.kind {
                crate::semantic::ScopeSymbolKind::Parameter => {
                    format!("Unused parameter: '{}'", symbol.name)
                }
                _ => {
                    format!("Unused variable: '{}'", symbol.name)
                }
            };
            diagnostics.push(Diagnostic::warning(
                symbol.defined_at.clone(),
                "PY103",
                DiagnosticCategory::Names,
                message,
            ));
        }

        // PY106: Variable redeclaration (assigned multiple times without use)
        for symbol in analyzer.redeclared_symbols() {
            diagnostics.push(Diagnostic::new(
                symbol.defined_at.clone(),
                crate::diagnostic::DiagnosticSeverity::Hint,
                "PY106",
                DiagnosticCategory::Names,
                format!("Variable '{}' is assigned multiple times", symbol.name),
            ));
        }

        diagnostics
    }

    // ===== Pydantic Checks =====

    /// Check if a class inherits from BaseModel (Pydantic)
    fn is_pydantic_model(node: &tree_sitter::Node<'_>, file: &ParsedFile) -> bool {
        if node.kind() != "class_definition" {
            return false;
        }

        // Check superclass_list for BaseModel
        if let Some(bases) = node.child_by_field_name("superclasses") {
            let mut cursor = bases.walk();
            for child in bases.children(&mut cursor) {
                let text = file.node_text(&child);
                // Match BaseModel, pydantic.BaseModel, etc.
                if text == "BaseModel"
                    || text.ends_with(".BaseModel")
                    || text == "BaseSettings"
                    || text.ends_with(".BaseSettings")
                {
                    return true;
                }
            }
        }
        false
    }

    /// PY501: Mutable default in Pydantic model field
    fn check_pydantic_mutable_default(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if Self::is_pydantic_model(node, file) {
                // Check class body for field definitions
                if let Some(body) = node.child_by_field_name("body") {
                    let mut cursor = body.walk();
                    for child in body.children(&mut cursor) {
                        // Look for annotated assignments: field: type = value
                        if child.kind() == "expression_statement" {
                            if let Some(expr) = child.child(0) {
                                if expr.kind() == "assignment" {
                                    if let Some(value) = expr.child_by_field_name("right") {
                                        let value_kind = value.kind();
                                        // Mutable defaults: [], {}, set()
                                        if value_kind == "list"
                                            || value_kind == "dictionary"
                                            || value_kind == "set"
                                        {
                                            diagnostics.push(Diagnostic::warning(
                                                Range::from_node(&value),
                                                "PY501",
                                                DiagnosticCategory::Logic,
                                                "Mutable default in Pydantic model. Use Field(default_factory=...) instead",
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// PY502: Deprecated @validator decorator (Pydantic V1 -> V2)
    fn check_pydantic_deprecated_validator(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "decorator" {
                let text = file.node_text(node);
                // @validator(...) is deprecated in V2, use @field_validator
                if text.starts_with("@validator") && !text.starts_with("@validator_") {
                    diagnostics.push(Diagnostic::warning(
                        Range::from_node(node),
                        "PY502",
                        DiagnosticCategory::Style,
                        "Deprecated: @validator is Pydantic V1. Use @field_validator in V2",
                    ));
                }
                // @root_validator is deprecated, use @model_validator
                if text.starts_with("@root_validator") {
                    diagnostics.push(Diagnostic::warning(
                        Range::from_node(node),
                        "PY502",
                        DiagnosticCategory::Style,
                        "Deprecated: @root_validator is Pydantic V1. Use @model_validator in V2",
                    ));
                }
            }
            true
        });

        diagnostics
    }

    /// PY503: Deprecated Config class in Pydantic model (V1 -> V2)
    fn check_pydantic_deprecated_config(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if Self::is_pydantic_model(node, file) {
                // Check class body for nested Config class
                if let Some(body) = node.child_by_field_name("body") {
                    let mut cursor = body.walk();
                    for child in body.children(&mut cursor) {
                        if child.kind() == "class_definition" {
                            if let Some(name) = child.child_by_field_name("name") {
                                if file.node_text(&name) == "Config" {
                                    diagnostics.push(Diagnostic::warning(
                                        Range::from_node(&child),
                                        "PY503",
                                        DiagnosticCategory::Style,
                                        "Deprecated: class Config is Pydantic V1. Use model_config = ConfigDict(...) in V2",
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            true
        });

        diagnostics
    }

    // ===== Import Sorting Checks (isort-like) =====

    /// Check import sorting and grouping
    fn check_import_sorting(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut imports: Vec<ImportInfo> = Vec::new();
        let mut seen_modules: HashMap<String, Range> = HashMap::new();
        let mut first_non_import_line: Option<u32> = None;

        // Collect all imports
        file.walk(|node, _depth| {
            let is_import = node.kind() == "import_statement";
            let is_from_import = node.kind() == "import_from_statement";

            if is_import || is_from_import {
                let line = node.start_position().row as u32;
                let range = Range::from_node(node);

                // Check if import is after non-import code
                if let Some(first_code) = first_non_import_line {
                    if line > first_code {
                        diagnostics.push(Diagnostic::warning(
                            range.clone(),
                            "PY604",
                            DiagnosticCategory::Style,
                            "Import should be at the top of the file",
                        ));
                    }
                }

                // Extract module name
                let module = self.extract_import_module(node, file);
                if module.is_empty() {
                    return true;
                }

                // Check for duplicate imports
                if let Some(prev_range) = seen_modules.get(&module) {
                    diagnostics.push(Diagnostic::warning(
                        range.clone(),
                        "PY603",
                        DiagnosticCategory::Style,
                        format!("Duplicate import: '{}'", module),
                    ));
                    let _ = prev_range; // suppress unused warning
                } else {
                    seen_modules.insert(module.clone(), range.clone());
                }

                // Classify import group
                let group = Self::classify_import(&module);

                imports.push(ImportInfo {
                    module,
                    group,
                    line,
                    range,
                    is_from_import,
                });
            } else if first_non_import_line.is_none() {
                // Track first non-import, non-comment, non-docstring line
                let kind = node.kind();
                if kind != "comment"
                    && kind != "expression_statement"  // might be docstring
                    && kind != "module"
                    && node.parent().map(|p| p.kind()) == Some("module")
                {
                    first_non_import_line = Some(node.start_position().row as u32);
                }
            }

            true
        });

        // Check sorting within groups
        if imports.len() > 1 {
            let mut prev_group: Option<ImportGroup> = None;
            let mut prev_module: Option<String> = None;
            let mut prev_line: Option<u32> = None;

            for import in &imports {
                // Check if groups are properly separated
                if let Some(pg) = prev_group {
                    if import.group != pg {
                        // Different group - should have blank line between
                        if let Some(pl) = prev_line {
                            if import.line == pl + 1 {
                                diagnostics.push(Diagnostic::new(
                                    import.range.clone(),
                                    crate::diagnostic::DiagnosticSeverity::Hint,
                                    "PY602",
                                    DiagnosticCategory::Style,
                                    format!(
                                        "Add blank line before {} imports",
                                        Self::group_name(import.group)
                                    ),
                                ));
                            }
                        }
                        prev_module = None; // Reset for new group
                    }
                }

                // Check alphabetical order within group
                if let Some(ref pm) = prev_module {
                    if prev_group == Some(import.group) {
                        if import.module.to_lowercase() < pm.to_lowercase() {
                            diagnostics.push(Diagnostic::new(
                                import.range.clone(),
                                crate::diagnostic::DiagnosticSeverity::Hint,
                                "PY601",
                                DiagnosticCategory::Style,
                                format!("Import '{}' should come before '{}'", import.module, pm),
                            ));
                        }
                    }
                }

                prev_group = Some(import.group);
                prev_module = Some(import.module.clone());
                prev_line = Some(import.line);
            }
        }

        diagnostics
    }

    /// Extract module name from import statement
    fn extract_import_module(&self, node: &tree_sitter::Node<'_>, file: &ParsedFile) -> String {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "dotted_name" => {
                    return file.node_text(&child).to_string();
                }
                "aliased_import" => {
                    if let Some(name) = child.child_by_field_name("name") {
                        return file.node_text(&name).to_string();
                    }
                }
                _ => {}
            }
        }
        String::new()
    }

    /// Classify an import into a group
    fn classify_import(module: &str) -> ImportGroup {
        let base = module.split('.').next().unwrap_or(module);

        if base == "__future__" {
            return ImportGroup::Future;
        }

        // Python stdlib modules (common ones)
        const STDLIB: &[&str] = &[
            "abc",
            "argparse",
            "ast",
            "asyncio",
            "base64",
            "builtins",
            "collections",
            "concurrent",
            "contextlib",
            "copy",
            "csv",
            "dataclasses",
            "datetime",
            "decimal",
            "enum",
            "functools",
            "glob",
            "gzip",
            "hashlib",
            "heapq",
            "html",
            "http",
            "importlib",
            "inspect",
            "io",
            "itertools",
            "json",
            "logging",
            "math",
            "multiprocessing",
            "operator",
            "os",
            "pathlib",
            "pickle",
            "platform",
            "pprint",
            "queue",
            "random",
            "re",
            "shutil",
            "signal",
            "socket",
            "sqlite3",
            "ssl",
            "string",
            "struct",
            "subprocess",
            "sys",
            "tempfile",
            "textwrap",
            "threading",
            "time",
            "timeit",
            "traceback",
            "types",
            "typing",
            "unittest",
            "urllib",
            "uuid",
            "warnings",
            "weakref",
            "xml",
            "zipfile",
        ];

        if STDLIB.contains(&base) {
            return ImportGroup::Stdlib;
        }

        // Common third-party packages
        const THIRD_PARTY: &[&str] = &[
            "aiohttp",
            "boto3",
            "celery",
            "click",
            "django",
            "fastapi",
            "flask",
            "httpx",
            "jwt",
            "numpy",
            "pandas",
            "pydantic",
            "pytest",
            "redis",
            "requests",
            "rich",
            "scipy",
            "sentry_sdk",
            "sqlalchemy",
            "starlette",
            "tenacity",
            "toml",
            "tqdm",
            "uvicorn",
            "yaml",
        ];

        if THIRD_PARTY.contains(&base) {
            return ImportGroup::ThirdParty;
        }

        // Default: assume first-party (local project)
        ImportGroup::FirstParty
    }

    /// Get display name for import group
    fn group_name(group: ImportGroup) -> &'static str {
        match group {
            ImportGroup::Future => "future",
            ImportGroup::Stdlib => "standard library",
            ImportGroup::ThirdParty => "third-party",
            ImportGroup::FirstParty => "first-party",
        }
    }

    // ===== Complexity Checks (pylint-like) =====

    /// PY701: Too many arguments, PY702: Function too long
    fn check_function_complexity(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        const MAX_ARGS: usize = 7;
        const MAX_LINES: usize = 50;

        file.walk(|node, _depth| {
            if node.kind() == "function_definition" || node.kind() == "async_function_definition" {
                let name = node
                    .child_by_field_name("name")
                    .map(|n| file.node_text(&n).to_string())
                    .unwrap_or_default();

                // Check argument count
                if let Some(params) = node.child_by_field_name("parameters") {
                    let mut count = 0;
                    let mut cursor = params.walk();
                    for child in params.children(&mut cursor) {
                        if matches!(
                            child.kind(),
                            "identifier"
                                | "typed_parameter"
                                | "default_parameter"
                                | "typed_default_parameter"
                                | "list_splat_pattern"
                                | "dictionary_splat_pattern"
                        ) {
                            count += 1;
                        }
                    }
                    if count > MAX_ARGS {
                        diagnostics.push(Diagnostic::new(
                            Range::from_node(&params),
                            crate::diagnostic::DiagnosticSeverity::Hint,
                            "PY701",
                            DiagnosticCategory::Style,
                            format!(
                                "Function '{}' has {} arguments (max {})",
                                name, count, MAX_ARGS
                            ),
                        ));
                    }
                }

                // Check function length
                let start = node.start_position().row;
                let end = node.end_position().row;
                let lines = end - start + 1;
                if lines > MAX_LINES {
                    diagnostics.push(Diagnostic::new(
                        Range::from_node(node),
                        crate::diagnostic::DiagnosticSeverity::Hint,
                        "PY702",
                        DiagnosticCategory::Style,
                        format!("Function '{}' is {} lines (max {})", name, lines, MAX_LINES),
                    ));
                }
            }
            true
        });

        diagnostics
    }
}

impl Default for PythonChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Checker for PythonChecker {
    fn language(&self) -> Language {
        Language::Python
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Check for syntax errors from tree-sitter
        if file.has_errors {
            file.walk(|node, _depth| {
                if node.is_error() || node.is_missing() {
                    diagnostics.push(Diagnostic::error(
                        Range::from_node(node),
                        "PY000",
                        DiagnosticCategory::Syntax,
                        "Syntax error",
                    ));
                }
                true
            });
        }

        // Run all checks
        diagnostics.extend(self.check_unused_imports(file));
        diagnostics.extend(self.check_mutable_default(file));
        diagnostics.extend(self.check_bare_except(file));
        diagnostics.extend(self.check_unreachable_code(file));
        diagnostics.extend(self.check_shadowed_builtins(file));
        // New JetBrains-inspired checks
        diagnostics.extend(self.check_private_member_access(file));
        diagnostics.extend(self.check_duplicate_dict_keys(file));
        diagnostics.extend(self.check_simplify_boolean(file));
        diagnostics.extend(self.check_none_comparison(file));
        diagnostics.extend(self.check_statement_no_effect(file));
        // Scope-based checks
        diagnostics.extend(self.check_scope_issues(file));
        // Pydantic checks
        diagnostics.extend(self.check_pydantic_mutable_default(file));
        diagnostics.extend(self.check_pydantic_deprecated_validator(file));
        diagnostics.extend(self.check_pydantic_deprecated_config(file));
        // Import sorting checks
        diagnostics.extend(self.check_import_sorting(file));
        // Complexity checks
        diagnostics.extend(self.check_function_complexity(file));
        // Security checks (delegated to python_security module)
        diagnostics.extend(super::python_security::check_eval_usage(file));
        diagnostics.extend(super::python_security::check_exec_usage(file));
        diagnostics.extend(super::python_security::check_pickle_usage(file));
        diagnostics.extend(super::python_security::check_subprocess_shell(file));
        let source_lines: Vec<&str> = file.source.lines().collect();
        diagnostics.extend(super::python_security::check_hardcoded_secrets(
            &source_lines,
        ));

        // R2: Run the full type-inference engine (control-flow aware narrowing,
        // TypeVar generic resolution, @overload dispatch, Protocol structural typing,
        // TypeGuard / TypeIs predicates) and surface type errors as diagnostics.
        // The TypeChecker internally uses TypeNarrower to narrow types through
        // isinstance(), is None, TypeGuard, TypeIs branches.
        if file.language == crate::syntax::Language::Python && !file.has_errors {
            let mut type_checker = crate::type_inference::TypeChecker::new(&file.source);
            let type_diags = type_checker.check_file(file);
            diagnostics.extend(type_diags);
        }

        diagnostics
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec![
            "PY000", // Syntax error
            "PY102", // Unused import
            "PY103", // Unused variable/parameter
            "PY104", // Shadowed builtin
            "PY106", // Variable redeclaration
            "PY201", // Mutable default argument
            "PY202", // Bare except
            "PY203", // Unreachable code
            // JetBrains-inspired
            "PY402", // Private member access
            "PY403", // Duplicate dict keys
            "PY404", // Simplify boolean
            "PY405", // == None instead of is None
            "PY406", // Statement no effect
            // Pydantic
            "PY501", // Mutable default in Pydantic model
            "PY502", // Deprecated @validator/@root_validator
            "PY503", // Deprecated Config class
            // Import sorting (isort-like)
            "PY601", // Imports not sorted
            "PY602", // Import groups not separated
            "PY603", // Duplicate import
            "PY604", // Import should be at top of file
            // Complexity (pylint-like)
            "PY701", // Too many arguments
            "PY702", // Function too long
            // Security
            "PY301", // eval() usage
            "PY302", // exec() usage
            "PY303", // pickle.loads()/pickle.load() usage
            "PY304", // subprocess with shell=True
            "PY305", // Hardcoded secrets
        ]
    }
}
