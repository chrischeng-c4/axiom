//! Type checking - verifies type compatibility and generates diagnostics

use std::collections::HashMap;
use tree_sitter::Node;

use super::annotation::parse_type_annotation;
use super::infer::TypeInferencer;
use super::narrow::{self, TypeNarrower};
use super::ty::{LiteralValue, ParamKind, Type, Variance};
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Range};
use crate::syntax::ParsedFile;

/// Type error information
#[derive(Debug, Clone)]
pub struct TypeError {
    pub range: Range,
    pub expected: Type,
    pub got: Type,
    pub message: String,
}

/// Function context for tracking return types
#[derive(Debug, Clone)]
struct FunctionContext {
    name: String,
    return_type: Type,
    has_return: bool,
}

/// Position in a function signature for variance checking
#[derive(Debug, Clone, Copy)]
enum VariancePosition {
    /// Input position (parameters) - contravariant
    Input,
    /// Output position (return type) - covariant
    Output,
}

/// Type checker that combines inference with compatibility checking
pub struct TypeChecker<'a> {
    /// Type inferencer
    inferencer: TypeInferencer<'a>,
    /// Collected diagnostics
    diagnostics: Vec<Diagnostic>,
    /// Source code
    source: &'a str,
    /// Stack of function contexts (for nested functions)
    function_stack: Vec<FunctionContext>,
    /// Type narrower for control flow analysis
    narrower: TypeNarrower,
}

impl<'a> TypeChecker<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            inferencer: TypeInferencer::new(source),
            diagnostics: Vec::new(),
            source,
            function_stack: Vec::new(),
            narrower: TypeNarrower::new(),
        }
    }

    /// Get current function context
    fn current_function(&self) -> Option<&FunctionContext> {
        self.function_stack.last()
    }

    /// Mark current function as having a return statement
    fn mark_has_return(&mut self) {
        if let Some(ctx) = self.function_stack.last_mut() {
            ctx.has_return = true;
        }
    }

    /// Check a file and return diagnostics
    pub fn check_file(&mut self, file: &ParsedFile) -> Vec<Diagnostic> {
        let root = file.tree.root_node();
        self.check_node(&root);
        std::mem::take(&mut self.diagnostics)
    }

    /// Recursively check a node
    fn check_node(&mut self, node: &Node) {
        match node.kind() {
            "class_definition" => {
                self.check_class(node);
                return; // check_class handles its own recursion
            }
            "function_definition" | "async_function_definition" => {
                self.check_function(node);
                return; // check_function handles its own recursion
            }
            "if_statement" => {
                self.check_if_statement(node);
                return; // check_if_statement handles its own recursion
            }
            "while_statement" => {
                self.check_while_statement(node);
                return; // handles its own recursion
            }
            "for_statement" => {
                self.check_for_statement(node);
                return; // handles its own recursion
            }
            "try_statement" => {
                self.check_try_statement(node);
                return; // handles its own recursion
            }
            "import_statement" | "import_from_statement" => {
                self.inferencer.analyze_import(node);
            }
            "assignment" => {
                self.check_assignment(node);
            }
            "return_statement" => {
                self.check_return(node);
            }
            "call" => {
                self.check_call(node);
            }
            "attribute" => {
                self.check_attribute(node);
            }
            _ => {}
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(&child);
        }
    }

    /// Check function definition
    fn check_function(&mut self, node: &Node) {
        let name = node
            .child_by_field_name("name")
            .map(|n| self.node_text(&n).to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Get return type annotation
        let return_type = node
            .child_by_field_name("return_type")
            .map(|rt| parse_type_annotation(self.source, &rt))
            .unwrap_or(Type::Unknown);

        // Check for missing return type annotation (only for public functions)
        if node.child_by_field_name("return_type").is_none() && !name.starts_with('_') {
            self.diagnostics.push(Diagnostic::new(
                Range::from_node(node),
                DiagnosticSeverity::Hint,
                "TC002",
                DiagnosticCategory::Type,
                format!("Function '{}' is missing return type annotation", name),
            ));
        }

        // Analyze function and add to environment
        self.inferencer.analyze_function(node);

        // Push function context
        self.function_stack.push(FunctionContext {
            name: name.clone(),
            return_type: return_type.clone(),
            has_return: false,
        });

        // Check function body
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                self.check_node(&child);
            }
        }

        // Pop function context and check for missing return
        if let Some(ctx) = self.function_stack.pop() {
            // If function has declared return type (not None/Unknown) but no return
            if !ctx.has_return && !matches!(ctx.return_type, Type::None | Type::Unknown | Type::Any)
            {
                self.diagnostics.push(Diagnostic::warning(
                    Range::from_node(node),
                    "TC003",
                    DiagnosticCategory::Type,
                    format!(
                        "Function '{}' declares return type '{}' but may not return a value",
                        ctx.name, ctx.return_type
                    ),
                ));
            }
        }
    }

    /// Check class definition
    fn check_class(&mut self, node: &Node) {
        let class_name = node
            .child_by_field_name("name")
            .map(|n| self.node_text(&n).to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Analyze class and register in inferencer
        self.inferencer.analyze_class(node);

        // Validate variance usage in class methods
        if let Some(class_info) = self.inferencer.get_class(&class_name) {
            if class_info.is_generic() {
                self.validate_variance_in_class(node, &class_name, class_info.clone());
            }
        }

        // Check class body
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                self.check_node(&child);
            }
        }
    }

    /// Validate variance usage in a generic class
    ///
    /// Covariant TypeVars should only appear in return positions (outputs)
    /// Contravariant TypeVars should only appear in parameter positions (inputs)
    fn validate_variance_in_class(
        &mut self,
        node: &Node,
        class_name: &str,
        class_info: super::class_info::ClassInfo,
    ) {
        // Get the class body
        let body = match node.child_by_field_name("body") {
            Some(b) => b,
            None => return,
        };

        // Build a map of TypeVar names to their variance
        let typevar_variance: std::collections::HashMap<String, Variance> = class_info
            .generic_params
            .iter()
            .map(|p| (p.name.clone(), p.variance))
            .collect();

        // Check each method in the class
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            if child.kind() == "function_definition" || child.kind() == "async_function_definition"
            {
                self.validate_method_variance(&child, class_name, &typevar_variance);
            }
        }
    }

    /// Validate variance usage in a method
    fn validate_method_variance(
        &mut self,
        node: &Node,
        _class_name: &str,
        typevar_variance: &std::collections::HashMap<String, Variance>,
    ) {
        let method_name = node
            .child_by_field_name("name")
            .map(|n| self.node_text(&n).to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Check parameter types (input positions - contravariant allowed)
        if let Some(params) = node.child_by_field_name("parameters") {
            let mut cursor = params.walk();
            for param in params.children(&mut cursor) {
                if param.kind() == "typed_parameter" || param.kind() == "typed_default_parameter" {
                    if let Some(type_node) = param.child_by_field_name("type") {
                        let type_text = self.node_text(&type_node).to_string();
                        self.check_variance_position(
                            &type_node,
                            &type_text,
                            typevar_variance,
                            VariancePosition::Input,
                            &method_name,
                        );
                    }
                }
            }
        }

        // Check return type (output position - covariant allowed)
        if let Some(return_type) = node.child_by_field_name("return_type") {
            let type_text = self.node_text(&return_type).to_string();
            self.check_variance_position(
                &return_type,
                &type_text,
                typevar_variance,
                VariancePosition::Output,
                &method_name,
            );
        }
    }

    /// Check if a TypeVar is used in a valid position according to its variance
    fn check_variance_position(
        &mut self,
        node: &Node,
        type_text: &str,
        typevar_variance: &std::collections::HashMap<String, Variance>,
        position: VariancePosition,
        method_name: &str,
    ) {
        // Simple check: look for TypeVar names in the type annotation
        for (name, variance) in typevar_variance {
            // Skip if this TypeVar doesn't appear in the type
            if !type_text.contains(name) {
                continue;
            }

            let invalid = match (variance, position) {
                // Covariant TypeVar in input position is invalid
                (Variance::Covariant, VariancePosition::Input) => true,
                // Contravariant TypeVar in output position is invalid
                (Variance::Contravariant, VariancePosition::Output) => true,
                // All other combinations are valid
                _ => false,
            };

            if invalid {
                let pos_name = match position {
                    VariancePosition::Input => "parameter",
                    VariancePosition::Output => "return",
                };
                let var_name = match variance {
                    Variance::Covariant => "covariant",
                    Variance::Contravariant => "contravariant",
                    Variance::Invariant => "invariant",
                };

                self.diagnostics.push(Diagnostic::error(
                    Range::from_node(node),
                    "TC010",
                    DiagnosticCategory::Type,
                    format!(
                        "{} TypeVar '{}' cannot appear in {} type of method '{}'",
                        var_name, name, pos_name, method_name
                    ),
                ));
            }
        }
    }

    /// Check assignment for type consistency
    fn check_assignment(&mut self, node: &Node) {
        let left = node.child_by_field_name("left");
        let right = node.child_by_field_name("right");

        if let (Some(target), Some(value)) = (left, right) {
            let value_type = self.inferencer.infer_expr(&value);

            // Check for annotated assignment
            if let Some(type_node) = node.child_by_field_name("type") {
                let expected = parse_type_annotation(self.source, &type_node);

                if !self.is_assignable(&expected, &value_type) {
                    self.diagnostics.push(Diagnostic::error(
                        Range::from_node(&value),
                        "TC001",
                        DiagnosticCategory::Type,
                        format!(
                            "Type mismatch: expected '{}', got '{}'",
                            expected, value_type
                        ),
                    ));
                }
            }

            // Bind the variable
            self.inferencer.bind_assignment(&target, value_type);
        }
    }

    /// Check return statement
    fn check_return(&mut self, node: &Node) {
        // Mark that we have a return in this function
        self.mark_has_return();

        // Get expected return type from function context
        let expected_return = self
            .current_function()
            .map(|ctx| ctx.return_type.clone())
            .unwrap_or(Type::Unknown);

        // Get actual return value type
        let actual_return = if let Some(value) = node.child(1) {
            self.inferencer.infer_expr(&value)
        } else {
            Type::None // bare "return" returns None
        };

        // Check compatibility
        if !expected_return.is_unknown() && !expected_return.is_any() {
            if !self.is_assignable(&expected_return, &actual_return) {
                self.diagnostics.push(Diagnostic::error(
                    Range::from_node(node),
                    "TC003",
                    DiagnosticCategory::Type,
                    format!(
                        "Incompatible return type: expected '{}', got '{}'",
                        expected_return, actual_return
                    ),
                ));
            }
        }
    }

    /// Check function call
    fn check_call(&mut self, node: &Node) {
        let func = match node.child_by_field_name("function") {
            Some(f) => f,
            None => return,
        };

        let func_type = self.inferencer.infer_expr(&func);

        match &func_type {
            Type::Callable { params, .. } => {
                self.check_call_arguments(node, params);
            }
            Type::Unknown | Type::Any => {
                // Can't check unknown/any types
            }
            _ => {
                // Not callable
                self.diagnostics.push(Diagnostic::error(
                    Range::from_node(&func),
                    "TC004",
                    DiagnosticCategory::Type,
                    format!("Type '{}' is not callable", func_type),
                ));
            }
        }
    }

    /// Check call arguments
    fn check_call_arguments(&mut self, call: &Node, params: &[super::ty::Param]) {
        let args = match call.child_by_field_name("arguments") {
            Some(a) => a,
            None => return,
        };

        let mut positional_args: Vec<(Range, Type)> = Vec::new();
        let mut keyword_args: HashMap<String, (Range, Type)> = HashMap::new();
        let mut cursor = args.walk();

        for child in args.children(&mut cursor) {
            match child.kind() {
                "(" | ")" | "," => continue,
                "keyword_argument" => {
                    // Parse keyword argument: name=value
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(&name_node).to_string();
                        if let Some(value_node) = child.child_by_field_name("value") {
                            let value_type = self.inferencer.infer_expr(&value_node);
                            keyword_args.insert(name, (Range::from_node(&child), value_type));
                        }
                    }
                }
                _ => {
                    positional_args
                        .push((Range::from_node(&child), self.inferencer.infer_expr(&child)));
                }
            }
        }

        // Build param lookup by name
        let param_by_name: HashMap<&str, &super::ty::Param> =
            params.iter().map(|p| (p.name.as_str(), p)).collect();

        // Check for too many positional arguments
        let max_positional = params
            .iter()
            .filter(|p| !matches!(p.kind, ParamKind::VarPositional | ParamKind::VarKeyword))
            .count();

        if positional_args.len() > max_positional
            && !params
                .iter()
                .any(|p| matches!(p.kind, ParamKind::VarPositional))
        {
            self.diagnostics.push(Diagnostic::error(
                Range::from_node(&args),
                "TC006",
                DiagnosticCategory::Type,
                format!(
                    "Too many arguments: expected at most {}, got {}",
                    max_positional,
                    positional_args.len()
                ),
            ));
        }

        // Check positional argument types
        for (i, (range, arg_ty)) in positional_args.iter().enumerate() {
            if let Some(param) = params.get(i) {
                if !param.ty.is_unknown() && !param.ty.is_any() {
                    if !self.is_assignable(&param.ty, arg_ty) {
                        self.diagnostics.push(Diagnostic::error(
                            range.clone(),
                            "TC005",
                            DiagnosticCategory::Type,
                            format!(
                                "Argument '{}' type mismatch: expected '{}', got '{}'",
                                param.name, param.ty, arg_ty
                            ),
                        ));
                    }
                }
            }
        }

        // Check keyword argument types
        for (name, (range, arg_ty)) in &keyword_args {
            if let Some(param) = param_by_name.get(name.as_str()) {
                if !param.ty.is_unknown() && !param.ty.is_any() {
                    if !self.is_assignable(&param.ty, arg_ty) {
                        self.diagnostics.push(Diagnostic::error(
                            range.clone(),
                            "TC005",
                            DiagnosticCategory::Type,
                            format!(
                                "Argument '{}' type mismatch: expected '{}', got '{}'",
                                name, param.ty, arg_ty
                            ),
                        ));
                    }
                }
            } else if !params
                .iter()
                .any(|p| matches!(p.kind, ParamKind::VarKeyword))
            {
                // Unknown keyword argument
                self.diagnostics.push(Diagnostic::error(
                    range.clone(),
                    "TC008",
                    DiagnosticCategory::Type,
                    format!("Unknown keyword argument: '{}'", name),
                ));
            }
        }

        // Check for missing required arguments
        let provided_count = positional_args.len();
        for (i, param) in params.iter().enumerate() {
            if !param.has_default
                && !matches!(param.kind, ParamKind::VarPositional | ParamKind::VarKeyword)
            {
                let provided_positionally = i < provided_count;
                let provided_by_keyword = keyword_args.contains_key(&param.name);

                if !provided_positionally && !provided_by_keyword {
                    self.diagnostics.push(Diagnostic::error(
                        Range::from_node(&args),
                        "TC007",
                        DiagnosticCategory::Type,
                        format!("Missing required argument: '{}'", param.name),
                    ));
                }
            }
        }
    }

    /// Check if statement with type narrowing
    fn check_if_statement(&mut self, node: &Node) {
        // Get the condition
        let condition = match node.child_by_field_name("condition") {
            Some(c) => c,
            None => return,
        };

        // Parse the condition into a narrowing condition
        let narrowing_cond = narrow::parse_condition(self.source, &condition);

        // Collect original types from environment for narrowing
        let original_types = self.inferencer.get_env_types();

        // Handle the consequence (if branch)
        if let Some(consequence) = node.child_by_field_name("consequence") {
            self.narrower.push_scope();
            self.narrower
                .apply_condition(&narrowing_cond, &original_types);

            // Set narrowed types as overrides in the inferencer
            let narrowed_types = self.collect_narrowed_types();
            self.inferencer.set_type_overrides(narrowed_types);

            // Check the body
            let mut cursor = consequence.walk();
            for child in consequence.children(&mut cursor) {
                self.check_node(&child);
            }

            self.inferencer.clear_type_overrides();
            self.narrower.pop_scope();
        }

        // Handle else/elif branches
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "else_clause" => {
                    self.narrower.push_scope();
                    // Apply negated condition for else branch
                    let negated = narrow::negate_condition(&narrowing_cond);
                    self.narrower.apply_condition(&negated, &original_types);

                    // Set narrowed types as overrides in the inferencer
                    let narrowed_types = self.collect_narrowed_types();
                    self.inferencer.set_type_overrides(narrowed_types);

                    // Check the else body
                    if let Some(body) = child.child_by_field_name("body") {
                        let mut body_cursor = body.walk();
                        for body_child in body.children(&mut body_cursor) {
                            self.check_node(&body_child);
                        }
                    }

                    self.inferencer.clear_type_overrides();
                    self.narrower.pop_scope();
                }
                "elif_clause" => {
                    // Recursively handle elif as another if
                    self.check_if_statement(&child);
                }
                _ => {}
            }
        }
    }

    /// Collect all narrowed types from the narrower
    fn collect_narrowed_types(&self) -> HashMap<String, Type> {
        let mut types = HashMap::new();
        // Get all narrowed types from the narrower scopes
        for name in self.inferencer.get_env_types().keys() {
            if let Some(ty) = self.narrower.get_narrowed(name) {
                types.insert(name.clone(), ty.clone());
            }
        }
        types
    }

    /// Check while statement with type narrowing
    fn check_while_statement(&mut self, node: &Node) {
        // Get the condition
        let condition = match node.child_by_field_name("condition") {
            Some(c) => c,
            None => return,
        };

        // Parse the condition for narrowing
        let narrowing_cond = narrow::parse_condition(self.source, &condition);
        let original_types = self.inferencer.get_env_types();

        // Handle the body (condition is true inside loop)
        if let Some(body) = node.child_by_field_name("body") {
            self.narrower.push_scope();
            self.narrower
                .apply_condition(&narrowing_cond, &original_types);

            let narrowed_types = self.collect_narrowed_types();
            self.inferencer.set_type_overrides(narrowed_types);

            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                self.check_node(&child);
            }

            self.inferencer.clear_type_overrides();
            self.narrower.pop_scope();
        }

        // Handle else clause (executed when condition becomes false)
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "else_clause" {
                if let Some(body) = child.child_by_field_name("body") {
                    let mut body_cursor = body.walk();
                    for body_child in body.children(&mut body_cursor) {
                        self.check_node(&body_child);
                    }
                }
            }
        }
    }

    /// Check for statement - binds loop variable
    fn check_for_statement(&mut self, node: &Node) {
        // Get the iterable and infer its element type
        let iterable = node.child_by_field_name("right");
        let element_type = if let Some(iter_node) = iterable {
            let iter_type = self.inferencer.infer_expr(&iter_node);
            match iter_type {
                Type::List(elem) => (*elem).clone(),
                Type::Set(elem) => (*elem).clone(),
                Type::Dict(key, _) => (*key).clone(), // iterating dict gives keys
                Type::Tuple(elems) => {
                    if elems.is_empty() {
                        Type::Unknown
                    } else {
                        Type::union(elems)
                    }
                }
                Type::Str => Type::Str, // iterating str gives chars (single char strings)
                _ => Type::Unknown,
            }
        } else {
            Type::Unknown
        };

        // Bind the loop variable
        if let Some(target) = node.child_by_field_name("left") {
            self.inferencer.bind_assignment(&target, element_type);
        }

        // Check the body
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                self.check_node(&child);
            }
        }

        // Handle else clause
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "else_clause" {
                if let Some(body) = child.child_by_field_name("body") {
                    let mut body_cursor = body.walk();
                    for body_child in body.children(&mut body_cursor) {
                        self.check_node(&body_child);
                    }
                }
            }
        }
    }

    /// Check try statement
    fn check_try_statement(&mut self, node: &Node) {
        // Check the try body
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                self.check_node(&child);
            }
        }

        // Check exception handlers
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "except_clause" => {
                    // Bind exception variable if present
                    if let Some(name) = child.child_by_field_name("name") {
                        // Exception type - default to BaseException if not specified
                        let exc_type = if let Some(type_node) = child.child_by_field_name("type") {
                            parse_type_annotation(self.source, &type_node)
                        } else {
                            Type::Instance {
                                name: "BaseException".to_string(),
                                module: Some("builtins".to_string()),
                                type_args: vec![],
                            }
                        };
                        self.inferencer.bind_assignment(&name, exc_type);
                    }

                    // Check except body
                    if let Some(body) = child.child_by_field_name("body") {
                        let mut body_cursor = body.walk();
                        for body_child in body.children(&mut body_cursor) {
                            self.check_node(&body_child);
                        }
                    }
                }
                "finally_clause" => {
                    if let Some(body) = child.child_by_field_name("body") {
                        let mut body_cursor = body.walk();
                        for body_child in body.children(&mut body_cursor) {
                            self.check_node(&body_child);
                        }
                    }
                }
                "else_clause" => {
                    if let Some(body) = child.child_by_field_name("body") {
                        let mut body_cursor = body.walk();
                        for body_child in body.children(&mut body_cursor) {
                            self.check_node(&body_child);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Check attribute access
    fn check_attribute(&mut self, node: &Node) {
        let object = match node.child_by_field_name("object") {
            Some(o) => o,
            None => return,
        };

        let object_type = self.inferencer.infer_expr(&object);

        // Check for None attribute access
        if object_type.contains_none() {
            let attr = node
                .child_by_field_name("attribute")
                .map(|a| self.node_text(&a))
                .unwrap_or("?");

            self.diagnostics.push(Diagnostic::warning(
                Range::from_node(node),
                "TC009",
                DiagnosticCategory::Type,
                format!(
                    "Accessing '{}' on potentially None value (type: {})",
                    attr, object_type
                ),
            ));
        }
    }

    /// Check if source type is assignable to target type
    fn is_assignable(&self, target: &Type, source: &Type) -> bool {
        // Any accepts anything
        if target.is_any() || source.is_any() {
            return true;
        }

        // Unknown is compatible with anything (not yet inferred)
        if target.is_unknown() || source.is_unknown() {
            return true;
        }

        // Error type is compatible with anything (avoid cascading errors)
        if target.is_error() || source.is_error() {
            return true;
        }

        // Same type
        if target == source {
            return true;
        }

        match (target, source) {
            // Optional accepts None
            (Type::Optional(_), Type::None) => true,
            (Type::Optional(inner), other) => self.is_assignable(inner, other),

            // Union accepts any member
            (Type::Union(types), src) => types.iter().any(|t| self.is_assignable(t, src)),
            (target, Type::Union(types)) => types.iter().all(|t| self.is_assignable(target, t)),

            // Float accepts Int
            (Type::Float, Type::Int) => true,

            // List covariance
            (Type::List(a), Type::List(b)) => self.is_assignable(a, b),

            // Dict invariance (simplified - should be covariant in value)
            (Type::Dict(k1, v1), Type::Dict(k2, v2)) => {
                self.is_assignable(k1, k2) && self.is_assignable(v1, v2)
            }

            // Tuple structural compatibility
            (Type::Tuple(a), Type::Tuple(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(t1, t2)| self.is_assignable(t1, t2))
            }

            // Class subtyping with inheritance and variance checking
            (
                Type::Instance {
                    name: n1,
                    type_args: args1,
                    ..
                },
                Type::Instance {
                    name: n2,
                    type_args: args2,
                    ..
                },
            ) => {
                // Different class names - check subtyping
                if n1 != n2 {
                    return self.inferencer.is_subclass(n2, n1);
                }

                // Same class name - check type arguments with variance
                if args1.is_empty() && args2.is_empty() {
                    return true; // Non-generic or unparameterized
                }

                // If one is generic and one is not, allow (gradual typing)
                if args1.is_empty() || args2.is_empty() {
                    return true;
                }

                // Check arity matches
                if args1.len() != args2.len() {
                    return false;
                }

                // Check each type argument with variance
                self.check_type_args_with_variance(n1, args1, args2)
            }

            // Protocol structural subtyping
            // A type is assignable to a Protocol if it has all required members
            (Type::Protocol { members, .. }, Type::Instance { name, .. }) => {
                // Check that the source type has all required protocol members
                members.iter().all(|(member_name, member_ty)| {
                    if let Some(attr_ty) =
                        self.inferencer.get_attribute_recursive(name, member_name)
                    {
                        self.is_assignable(member_ty, &attr_ty)
                    } else {
                        false
                    }
                })
            }

            // Callable to Protocol with __call__
            (Type::Protocol { members, .. }, Type::Callable { .. }) => {
                // A Callable can match a Protocol if the Protocol only requires __call__
                members.len() == 1 && members.iter().any(|(name, _)| name == "__call__")
            }

            // Literal types are assignable to their base types
            (Type::Int, Type::Literal(LiteralValue::Int(_))) => true,
            (Type::Float, Type::Literal(LiteralValue::Float(_))) => true,
            (Type::Float, Type::Literal(LiteralValue::Int(_))) => true, // int literal -> float
            (Type::Str, Type::Literal(LiteralValue::Str(_))) => true,
            (Type::Bool, Type::Literal(LiteralValue::Bool(_))) => true,
            (Type::None, Type::Literal(LiteralValue::None)) => true,

            // Literal to Literal (same value)
            (Type::Literal(a), Type::Literal(b)) => a == b,

            // TypedDict structural subtyping
            // A dict is assignable to TypedDict if it has all required keys
            (Type::TypedDict { fields, .. }, Type::Dict(key_ty, val_ty)) => {
                // For a generic dict to be assignable to TypedDict,
                // key must be str and value must be compatible with all field types
                if !self.is_assignable(&Type::Str, key_ty) {
                    return false;
                }
                fields
                    .iter()
                    .all(|(_, field_ty, _)| self.is_assignable(field_ty, val_ty))
            }

            // TypedDict to TypedDict: all required fields must match
            (
                Type::TypedDict {
                    fields: target_fields,
                    ..
                },
                Type::TypedDict {
                    fields: source_fields,
                    ..
                },
            ) => {
                // Check all required target fields exist in source with compatible types
                target_fields.iter().all(|(name, ty, required)| {
                    if let Some((_, src_ty, _)) = source_fields.iter().find(|(n, _, _)| n == name) {
                        self.is_assignable(ty, src_ty)
                    } else {
                        !required // Missing field is ok only if not required
                    }
                })
            }

            // TypedDict is a subtype of Dict[str, Union[field_types]]
            (Type::Dict(key_ty, _), Type::TypedDict { .. }) => {
                self.is_assignable(key_ty, &Type::Str)
            }

            // PEP 591: Final[T] is assignable to T
            (target, Type::Final(inner)) => self.is_assignable(target, inner),
            (Type::Final(inner), source) => self.is_assignable(inner, source),

            // PEP 593: Annotated[T, ...] is assignable to T
            (target, Type::Annotated { inner, .. }) => self.is_assignable(target, inner),
            (Type::Annotated { inner, .. }, source) => self.is_assignable(inner, source),

            // PEP 675: LiteralString is assignable to str
            (Type::Str, Type::LiteralString) => true,
            // String literals are assignable to LiteralString
            (Type::LiteralString, Type::Literal(LiteralValue::Str(_))) => true,
            // LiteralString is assignable to LiteralString
            (Type::LiteralString, Type::LiteralString) => true,

            // PEP 673: Self type - compare class names
            (
                Type::SelfType {
                    class_name: Some(n1),
                },
                Type::SelfType {
                    class_name: Some(n2),
                },
            ) => n1 == n2,
            (
                Type::SelfType {
                    class_name: Some(name),
                },
                Type::Instance {
                    name: inst_name, ..
                },
            ) => name == inst_name,
            (
                Type::Instance {
                    name: inst_name, ..
                },
                Type::SelfType {
                    class_name: Some(name),
                },
            ) => name == inst_name,
            // Unresolved Self is compatible with any Self
            (Type::SelfType { class_name: None }, Type::SelfType { .. }) => true,
            (Type::SelfType { .. }, Type::SelfType { class_name: None }) => true,

            // Overloaded functions - source can be any signature
            (Type::Callable { .. }, Type::Overloaded { signatures }) => {
                signatures.iter().any(|sig| self.is_assignable(target, sig))
            }
            // Overloaded to Callable - at least one signature must match
            (Type::Overloaded { signatures }, source) => {
                signatures.iter().any(|sig| self.is_assignable(sig, source))
            }

            _ => false,
        }
    }

    /// Check type arguments with variance rules
    ///
    /// For generic types, variance determines how type arguments relate:
    /// - Covariant: List[Dog] is assignable to List[Animal] (Dog <: Animal)
    /// - Contravariant: Callable[[Animal], None] is assignable to Callable[[Dog], None]
    /// - Invariant: Both types must be equal
    fn check_type_args_with_variance(
        &self,
        class_name: &str,
        target_args: &[Type],
        source_args: &[Type],
    ) -> bool {
        // Get class info to determine variance of each type parameter
        let class_info = self.inferencer.get_class(class_name);

        for (i, (target_arg, source_arg)) in target_args.iter().zip(source_args.iter()).enumerate()
        {
            let variance = class_info
                .map(|info| info.variance_at(i))
                .unwrap_or(Variance::Invariant);

            let compatible = match variance {
                Variance::Covariant => {
                    // Source must be a subtype of target
                    // e.g., List[Dog] assignable to List[Animal] if Dog <: Animal
                    self.is_assignable(target_arg, source_arg)
                }
                Variance::Contravariant => {
                    // Target must be a subtype of source (reversed)
                    // e.g., Callable[[Animal], R] assignable to Callable[[Dog], R]
                    self.is_assignable(source_arg, target_arg)
                }
                Variance::Invariant => {
                    // Types must be equal
                    self.is_assignable(target_arg, source_arg)
                        && self.is_assignable(source_arg, target_arg)
                }
            };

            if !compatible {
                return false;
            }
        }

        true
    }

    /// Get text of a node
    fn node_text(&self, node: &Node) -> &str {
        node.utf8_text(self.source.as_bytes()).unwrap_or("")
    }

    /// Get collected diagnostics
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

// ============================================================================
// SemanticModel builder - produces an owned SemanticModel from type checking
// ============================================================================

use super::model::{ParamInfo, ScopeId, SemanticModel, SemanticSymbolKind, SymbolData, TypeInfo};
use std::path::PathBuf;

/// Builder for creating SemanticModel from parsed code
///
/// This struct traverses the AST and collects type information,
/// producing an owned SemanticModel that can be cached and queried.
pub struct SemanticModelBuilder<'a> {
    source: &'a str,
    file_path: PathBuf,
    model: SemanticModel,
    current_scope: ScopeId,
    scope_stack: Vec<ScopeId>,
}

impl<'a> SemanticModelBuilder<'a> {
    /// Create a new builder for the given source
    pub fn new(source: &'a str, file_path: PathBuf) -> Self {
        let mut model = SemanticModel::new();
        let root_scope = model.add_scope(None, Range::default());

        Self {
            source,
            file_path,
            model,
            current_scope: root_scope,
            scope_stack: vec![root_scope],
        }
    }

    /// Build a SemanticModel from a parsed file
    pub fn build(mut self, file: &ParsedFile) -> SemanticModel {
        let root = file.tree.root_node();
        self.visit_node(&root);
        self.model.finalize();
        self.model
    }

    /// Push a new scope
    fn push_scope(&mut self, range: Range) -> ScopeId {
        let new_scope = self.model.add_scope(Some(self.current_scope), range);
        self.scope_stack.push(self.current_scope);
        self.current_scope = new_scope;
        new_scope
    }

    /// Pop the current scope
    fn pop_scope(&mut self) {
        if let Some(parent) = self.scope_stack.pop() {
            self.current_scope = parent;
        }
    }

    /// Parse a type from a node using the annotation parser
    fn parse_type_from_node(&self, node: &Node) -> TypeInfo {
        let ty = parse_type_annotation(self.source, node);
        TypeInfo::from_type(&ty)
    }

    /// Visit a node and its children
    fn visit_node(&mut self, node: &Node) {
        // Skip error nodes
        if node.is_error() || node.is_missing() {
            return;
        }

        match node.kind() {
            "function_definition" | "async_function_definition" => {
                self.visit_function(node);
                return;
            }
            "class_definition" => {
                self.visit_class(node);
                return;
            }
            "assignment" => {
                self.visit_assignment(node);
            }
            _ => {}
        }

        // Visit children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(&child);
        }
    }

    /// Visit a function definition
    fn visit_function(&mut self, node: &Node) {
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .map(|n| self.node_text(&n).to_string())
            .unwrap_or_default();
        let def_range = name_node
            .as_ref()
            .map(|n| Range::from_node(n))
            .unwrap_or_default();

        // Get return type
        let return_type = node
            .child_by_field_name("return_type")
            .map(|n| self.parse_type_from_node(&n))
            .unwrap_or(TypeInfo::Unknown);

        // Get parameters
        let params = self.collect_parameters(node);

        // Build callable type
        let type_info = TypeInfo::Callable {
            params,
            return_type: Box::new(return_type),
        };

        // Extract docstring
        let documentation = self.extract_docstring(node);

        // Add function symbol
        let symbol_id = self.model.add_symbol(SymbolData {
            name,
            kind: SemanticSymbolKind::Function,
            def_range: def_range.clone(),
            file_path: self.file_path.clone(),
            type_info: type_info.clone(),
            documentation,
            scope_id: self.current_scope,
            parent_id: None,
        });

        // Add typed range for the function name
        self.model
            .add_typed_range(def_range, type_info, Some(symbol_id));

        // Enter function scope
        let func_range = Range::from_node(node);
        self.push_scope(func_range);

        // Process parameters
        if let Some(ref params) = node.child_by_field_name("parameters") {
            self.visit_parameters(params);
        }

        // Process body
        if let Some(ref body) = node.child_by_field_name("body") {
            self.visit_node(body);
        }

        self.pop_scope();
    }

    /// Visit a class definition
    fn visit_class(&mut self, node: &Node) {
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .map(|n| self.node_text(&n).to_string())
            .unwrap_or_default();
        let def_range = name_node
            .as_ref()
            .map(|n| Range::from_node(n))
            .unwrap_or_default();

        let type_info = TypeInfo::Instance {
            name: name.clone(),
            module: None,
            type_args: vec![],
        };

        let documentation = self.extract_docstring(node);

        let class_id = self.model.add_symbol(SymbolData {
            name,
            kind: SemanticSymbolKind::Class,
            def_range: def_range.clone(),
            file_path: self.file_path.clone(),
            type_info: type_info.clone(),
            documentation,
            scope_id: self.current_scope,
            parent_id: None,
        });

        self.model
            .add_typed_range(def_range, type_info, Some(class_id));

        // Enter class scope
        let class_range = Range::from_node(node);
        self.push_scope(class_range);

        // Process body
        if let Some(ref body) = node.child_by_field_name("body") {
            self.visit_class_body(body, class_id);
        }

        self.pop_scope();
    }

    /// Visit class body and add methods/attributes
    fn visit_class_body(&mut self, body: &Node, class_id: super::model::SymbolId) {
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            match child.kind() {
                "function_definition" | "async_function_definition" => {
                    self.visit_method(&child, class_id);
                }
                "expression_statement" => {
                    // Check for class attributes (type annotations)
                    if let Some(expr) = child.child(0) {
                        if expr.kind() == "assignment" || expr.kind() == "type" {
                            self.visit_class_attribute(&expr, class_id);
                        }
                    }
                }
                _ => self.visit_node(&child),
            }
        }
    }

    /// Visit a method (function inside class)
    fn visit_method(&mut self, node: &Node, class_id: super::model::SymbolId) {
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .map(|n| self.node_text(&n).to_string())
            .unwrap_or_default();
        let def_range = name_node
            .as_ref()
            .map(|n| Range::from_node(n))
            .unwrap_or_default();

        let return_type = node
            .child_by_field_name("return_type")
            .map(|n| self.parse_type_from_node(&n))
            .unwrap_or(TypeInfo::Unknown);

        let params = self.collect_parameters(node);

        let type_info = TypeInfo::Callable {
            params,
            return_type: Box::new(return_type),
        };

        let documentation = self.extract_docstring(node);

        let method_id = self.model.add_symbol(SymbolData {
            name,
            kind: SemanticSymbolKind::Method,
            def_range: def_range.clone(),
            file_path: self.file_path.clone(),
            type_info: type_info.clone(),
            documentation,
            scope_id: self.current_scope,
            parent_id: Some(class_id),
        });

        self.model
            .add_typed_range(def_range, type_info, Some(method_id));

        // Process method body in its own scope
        let method_range = Range::from_node(node);
        self.push_scope(method_range);

        if let Some(ref params) = node.child_by_field_name("parameters") {
            self.visit_parameters(params);
        }

        if let Some(ref body) = node.child_by_field_name("body") {
            self.visit_node(body);
        }

        self.pop_scope();
    }

    /// Visit a class attribute
    fn visit_class_attribute(&mut self, node: &Node, class_id: super::model::SymbolId) {
        if let Some(left) = node.child_by_field_name("left") {
            if left.kind() == "identifier" {
                let name = self.node_text(&left).to_string();
                let def_range = Range::from_node(&left);

                let type_info = node
                    .child_by_field_name("type")
                    .map(|n| self.parse_type_from_node(&n))
                    .unwrap_or(TypeInfo::Unknown);

                let symbol_id = self.model.add_symbol(SymbolData {
                    name,
                    kind: SemanticSymbolKind::Attribute,
                    def_range: def_range.clone(),
                    file_path: self.file_path.clone(),
                    type_info: type_info.clone(),
                    documentation: None,
                    scope_id: self.current_scope,
                    parent_id: Some(class_id),
                });

                self.model
                    .add_typed_range(def_range, type_info, Some(symbol_id));
            }
        }
    }

    /// Visit an assignment
    fn visit_assignment(&mut self, node: &Node) {
        if let Some(left) = node.child_by_field_name("left") {
            if left.kind() == "identifier" {
                let name = self.node_text(&left).to_string();
                let def_range = Range::from_node(&left);

                // Try to get type from annotation, or use Unknown
                let type_info = node
                    .child_by_field_name("type")
                    .map(|n| self.parse_type_from_node(&n))
                    .unwrap_or(TypeInfo::Unknown);

                let symbol_id = self.model.add_symbol(SymbolData {
                    name,
                    kind: SemanticSymbolKind::Variable,
                    def_range: def_range.clone(),
                    file_path: self.file_path.clone(),
                    type_info: type_info.clone(),
                    documentation: None,
                    scope_id: self.current_scope,
                    parent_id: None,
                });

                self.model
                    .add_typed_range(def_range, type_info, Some(symbol_id));
            }
        }

        // Visit right-hand side
        if let Some(ref right) = node.child_by_field_name("right") {
            self.visit_node(right);
        }
    }

    /// Visit function parameters
    fn visit_parameters(&mut self, params: &Node) {
        let mut cursor = params.walk();
        for child in params.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    let name = self.node_text(&child).to_string();
                    let def_range = Range::from_node(&child);

                    let symbol_id = self.model.add_symbol(SymbolData {
                        name,
                        kind: SemanticSymbolKind::Parameter,
                        def_range: def_range.clone(),
                        file_path: self.file_path.clone(),
                        type_info: TypeInfo::Unknown,
                        documentation: None,
                        scope_id: self.current_scope,
                        parent_id: None,
                    });

                    self.model
                        .add_typed_range(def_range, TypeInfo::Unknown, Some(symbol_id));
                }
                "typed_parameter" | "typed_default_parameter" | "default_parameter" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = self.node_text(&name_node).to_string();
                        let def_range = Range::from_node(&name_node);

                        let type_info = child
                            .child_by_field_name("type")
                            .map(|n| self.parse_type_from_node(&n))
                            .unwrap_or(TypeInfo::Unknown);

                        let symbol_id = self.model.add_symbol(SymbolData {
                            name,
                            kind: SemanticSymbolKind::Parameter,
                            def_range: def_range.clone(),
                            file_path: self.file_path.clone(),
                            type_info: type_info.clone(),
                            documentation: None,
                            scope_id: self.current_scope,
                            parent_id: None,
                        });

                        self.model
                            .add_typed_range(def_range, type_info, Some(symbol_id));
                    }
                }
                _ => {}
            }
        }
    }

    /// Collect parameters as ParamInfo for type signatures
    fn collect_parameters(&self, node: &Node) -> Vec<ParamInfo> {
        let mut params = Vec::new();

        if let Some(params_node) = node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                let (name, type_info, has_default, is_variadic, is_keyword) = match child.kind() {
                    "identifier" => {
                        let name = self.node_text(&child).to_string();
                        (name, TypeInfo::Unknown, false, false, false)
                    }
                    "typed_parameter" => {
                        let name = child
                            .child_by_field_name("name")
                            .map(|n| self.node_text(&n).to_string())
                            .unwrap_or_default();
                        let ty = child
                            .child_by_field_name("type")
                            .map(|n| self.parse_type_from_node(&n))
                            .unwrap_or(TypeInfo::Unknown);
                        (name, ty, false, false, false)
                    }
                    "default_parameter" => {
                        let name = child
                            .child_by_field_name("name")
                            .map(|n| self.node_text(&n).to_string())
                            .unwrap_or_default();
                        (name, TypeInfo::Unknown, true, false, false)
                    }
                    "typed_default_parameter" => {
                        let name = child
                            .child_by_field_name("name")
                            .map(|n| self.node_text(&n).to_string())
                            .unwrap_or_default();
                        let ty = child
                            .child_by_field_name("type")
                            .map(|n| self.parse_type_from_node(&n))
                            .unwrap_or(TypeInfo::Unknown);
                        (name, ty, true, false, false)
                    }
                    "list_splat_pattern" => {
                        let name = child
                            .child(1)
                            .map(|n| self.node_text(&n).to_string())
                            .unwrap_or_else(|| "*args".to_string());
                        (name, TypeInfo::Unknown, false, true, false)
                    }
                    "dictionary_splat_pattern" => {
                        let name = child
                            .child(1)
                            .map(|n| self.node_text(&n).to_string())
                            .unwrap_or_else(|| "**kwargs".to_string());
                        (name, TypeInfo::Unknown, false, false, true)
                    }
                    _ => continue,
                };

                params.push(ParamInfo {
                    name,
                    type_info,
                    has_default,
                    is_variadic,
                    is_keyword,
                });
            }
        }

        params
    }

    /// Extract docstring from a function or class
    fn extract_docstring(&self, node: &Node) -> Option<String> {
        let body = node.child_by_field_name("body")?;
        let mut cursor = body.walk();
        let first_child = body.children(&mut cursor).next()?;

        if first_child.kind() == "expression_statement" {
            if let Some(expr) = first_child.child(0) {
                if expr.kind() == "string" {
                    let text = self.node_text(&expr);
                    let doc = text
                        .trim_start_matches("\"\"\"")
                        .trim_start_matches("'''")
                        .trim_end_matches("\"\"\"")
                        .trim_end_matches("'''")
                        .trim();
                    return Some(doc.to_string());
                }
            }
        }
        None
    }

    /// Get text of a node
    fn node_text(&self, node: &Node) -> &str {
        node.utf8_text(self.source.as_bytes()).unwrap_or("")
    }
}

/// Create a SemanticModel from a parsed file
pub fn build_semantic_model(file: &ParsedFile, source: &str, file_path: PathBuf) -> SemanticModel {
    SemanticModelBuilder::new(source, file_path).build(file)
}

#[cfg(test)]
#[path = "check_tests.rs"]
mod tests;
