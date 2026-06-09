//! Rust type inference engine
//!
//! This module provides type inference for Rust code, including:
//! - Generic type resolution
//! - Trait bound checking
//! - Method resolution
//! - Pattern matching type inference

use std::collections::HashMap;
use std::sync::Arc;

use tree_sitter::Node;

use super::rust_types::*;
use super::ty::TypeVarId;

// Re-export from submodules
pub use super::rust_lifetimes::{
    Borrow, BorrowId, BorrowState, LifetimeAnalyzer, LifetimeConstraint, LifetimeError,
    LifetimeErrorKind,
};
pub use super::rust_symbols::{
    RustConstant, RustFunction, RustSymbolCollector, RustSymbols, RustTypeAlias,
};
pub use super::rust_traits::{MethodResolution, TraitResolver};

// ============================================================================
// Type Inference Context
// ============================================================================

/// Context for Rust type inference
#[derive(Debug, Clone)]
pub struct RustTypeContext {
    /// Current scope's type bindings
    pub type_bindings: HashMap<String, RustType>,
    /// Current scope's lifetime bindings
    pub lifetime_bindings: HashMap<String, Lifetime>,
    /// Available trait impls
    pub trait_impls: Vec<Arc<ImplBlock>>,
    /// Known trait definitions
    pub trait_defs: HashMap<TraitId, Arc<TraitDef>>,
    /// Known struct definitions
    pub struct_defs: HashMap<String, Arc<StructDef>>,
    /// Known enum definitions
    pub enum_defs: HashMap<String, Arc<EnumDef>>,
    /// Type variable counter
    type_var_counter: usize,
    /// Lifetime counter
    lifetime_counter: usize,
}

impl RustTypeContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            type_bindings: HashMap::new(),
            lifetime_bindings: HashMap::new(),
            trait_impls: Vec::new(),
            trait_defs: HashMap::new(),
            struct_defs: HashMap::new(),
            enum_defs: HashMap::new(),
            type_var_counter: 0,
            lifetime_counter: 0,
        }
    }

    /// Create a new type variable
    pub fn fresh_type_var(&mut self, name: impl Into<String>) -> RustType {
        let id = TypeVarId(self.type_var_counter);
        self.type_var_counter += 1;
        RustType::TypeParam {
            id,
            name: name.into(),
            bounds: vec![],
        }
    }

    /// Create a new lifetime
    pub fn fresh_lifetime(&mut self) -> Lifetime {
        let id = LifetimeId(self.lifetime_counter);
        self.lifetime_counter += 1;
        Lifetime::Inferred(id)
    }

    /// Look up a type binding
    pub fn lookup_type(&self, name: &str) -> Option<RustType> {
        self.type_bindings.get(name).cloned()
    }

    /// Add a type binding
    pub fn bind_type(&mut self, name: String, ty: RustType) {
        self.type_bindings.insert(name, ty);
    }

    /// Look up a lifetime binding
    pub fn lookup_lifetime(&self, name: &str) -> Option<Lifetime> {
        self.lifetime_bindings.get(name).cloned()
    }

    /// Add a lifetime binding
    pub fn bind_lifetime(&mut self, name: String, lifetime: Lifetime) {
        self.lifetime_bindings.insert(name, lifetime);
    }

    /// Create a child context (for nested scopes)
    pub fn child(&self) -> Self {
        Self {
            type_bindings: self.type_bindings.clone(),
            lifetime_bindings: self.lifetime_bindings.clone(),
            trait_impls: self.trait_impls.clone(),
            trait_defs: self.trait_defs.clone(),
            struct_defs: self.struct_defs.clone(),
            enum_defs: self.enum_defs.clone(),
            type_var_counter: self.type_var_counter,
            lifetime_counter: self.lifetime_counter,
        }
    }
}

impl Default for RustTypeContext {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Rust Type Inferencer
// ============================================================================

/// Rust type inference engine
pub struct RustTypeInferencer {
    /// Type context
    context: RustTypeContext,
    /// Trait resolver for method resolution
    trait_resolver: super::rust_traits::TraitResolver,
    /// Type substitutions (for unification)
    substitutions: HashMap<TypeVarId, RustType>,
    /// Lifetime substitutions (reserved for future lifetime analysis)
    #[allow(dead_code)]
    lifetime_substitutions: HashMap<LifetimeId, Lifetime>,
    /// Inference errors
    errors: Vec<RustTypeError>,
}

/// Type inference error
#[derive(Debug, Clone)]
pub struct RustTypeError {
    /// Error message
    pub message: String,
    /// Location in source
    pub span: Option<(usize, usize)>,
    /// Error kind
    pub kind: RustTypeErrorKind,
}

/// Kind of type error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustTypeErrorKind {
    /// Type mismatch
    TypeMismatch,
    /// Unbound type variable
    UnboundTypeVar,
    /// Trait not implemented
    TraitNotImplemented,
    /// Lifetime error
    LifetimeError,
    /// Borrow checker error
    BorrowError,
    /// Ambiguous type
    AmbiguousType,
}

impl RustTypeInferencer {
    /// Create a new type inferencer
    pub fn new() -> Self {
        Self {
            context: RustTypeContext::new(),
            trait_resolver: super::rust_traits::TraitResolver::new(),
            substitutions: HashMap::new(),
            lifetime_substitutions: HashMap::new(),
            errors: Vec::new(),
        }
    }

    /// Create with existing context
    pub fn with_context(context: RustTypeContext) -> Self {
        // Build trait resolver from context's trait impls and defs
        let mut trait_resolver = super::rust_traits::TraitResolver::new();
        for impl_block in &context.trait_impls {
            trait_resolver.register_impl((**impl_block).clone());
        }
        for (_, trait_def) in &context.trait_defs {
            trait_resolver.register_trait((**trait_def).clone());
        }

        Self {
            context,
            trait_resolver,
            substitutions: HashMap::new(),
            lifetime_substitutions: HashMap::new(),
            errors: Vec::new(),
        }
    }

    /// Get a reference to the trait resolver
    pub fn trait_resolver(&self) -> &super::rust_traits::TraitResolver {
        &self.trait_resolver
    }

    /// Register an impl block for method resolution
    pub fn register_impl(&mut self, impl_block: ImplBlock) {
        self.trait_resolver.register_impl(impl_block);
    }

    /// Register a trait definition
    pub fn register_trait(&mut self, trait_def: TraitDef) {
        self.trait_resolver.register_trait(trait_def);
    }

    /// Get inference errors
    pub fn errors(&self) -> &[RustTypeError] {
        &self.errors
    }

    /// Clear errors
    pub fn clear_errors(&mut self) {
        self.errors.clear();
    }

    /// Infer the type of a Rust expression from AST node
    pub fn infer_expr(&mut self, node: &Node, source: &str) -> RustType {
        let kind = node.kind();

        match kind {
            // Literals
            "integer_literal" => self.infer_integer_literal(node, source),
            "float_literal" => RustType::F64,
            "string_literal" | "raw_string_literal" => RustType::Reference {
                lifetime: Some(Lifetime::Static),
                mutable: false,
                inner: Box::new(RustType::Str),
            },
            "char_literal" => RustType::Char,
            "boolean_literal" => RustType::Bool,

            // Identifiers
            "identifier" => self.infer_identifier(node, source),

            // Compound expressions
            "call_expression" => self.infer_call_expr(node, source),
            "field_expression" => self.infer_field_expr(node, source),
            "index_expression" => self.infer_index_expr(node, source),
            "reference_expression" => self.infer_reference_expr(node, source),
            "dereference_expression" => self.infer_deref_expr(node, source),
            "binary_expression" => self.infer_binary_expr(node, source),
            "unary_expression" => self.infer_unary_expr(node, source),
            "if_expression" => self.infer_if_expr(node, source),
            "match_expression" => self.infer_match_expr(node, source),
            "block" => self.infer_block(node, source),
            "tuple_expression" => self.infer_tuple_expr(node, source),
            "array_expression" => self.infer_array_expr(node, source),
            "struct_expression" => self.infer_struct_expr(node, source),
            "closure_expression" => self.infer_closure_expr(node, source),

            // Unit
            "unit_expression" => RustType::Unit,

            // Unknown
            _ => {
                self.errors.push(RustTypeError {
                    message: format!("Unknown expression kind: {}", kind),
                    span: Some((node.start_byte(), node.end_byte())),
                    kind: RustTypeErrorKind::AmbiguousType,
                });
                RustType::Infer
            }
        }
    }

    /// Infer integer literal type
    fn infer_integer_literal(&mut self, node: &Node, source: &str) -> RustType {
        let text = &source[node.start_byte()..node.end_byte()];

        // Check for type suffix
        if text.ends_with("i8") {
            RustType::I8
        } else if text.ends_with("i16") {
            RustType::I16
        } else if text.ends_with("i32") {
            RustType::I32
        } else if text.ends_with("i64") {
            RustType::I64
        } else if text.ends_with("i128") {
            RustType::I128
        } else if text.ends_with("isize") {
            RustType::Isize
        } else if text.ends_with("u8") {
            RustType::U8
        } else if text.ends_with("u16") {
            RustType::U16
        } else if text.ends_with("u32") {
            RustType::U32
        } else if text.ends_with("u64") {
            RustType::U64
        } else if text.ends_with("u128") {
            RustType::U128
        } else if text.ends_with("usize") {
            RustType::Usize
        } else {
            // Default to i32 (will be refined by context)
            RustType::I32
        }
    }

    /// Infer identifier type from context
    fn infer_identifier(&mut self, node: &Node, source: &str) -> RustType {
        let name = &source[node.start_byte()..node.end_byte()];

        if let Some(ty) = self.context.lookup_type(name) {
            ty
        } else {
            self.errors.push(RustTypeError {
                message: format!("Unbound identifier: {}", name),
                span: Some((node.start_byte(), node.end_byte())),
                kind: RustTypeErrorKind::UnboundTypeVar,
            });
            RustType::Error
        }
    }

    /// Infer call expression type
    fn infer_call_expr(&mut self, node: &Node, source: &str) -> RustType {
        if let Some(func) = node.child_by_field_name("function") {
            // Check if this is a method call (field_expression followed by call)
            if func.kind() == "field_expression" {
                return self.infer_method_call(&func, node, source);
            }

            let func_type = self.infer_expr(&func, source);

            match &func_type {
                RustType::FnPointer { return_type, .. } => *return_type.clone(),
                RustType::Closure { return_type, .. } => *return_type.clone(),
                RustType::Named { .. } => {
                    // Constructor call - return the named type
                    func_type.clone()
                }
                _ => {
                    // Try to resolve as method call
                    self.context.fresh_type_var("call_result")
                }
            }
        } else {
            RustType::Infer
        }
    }

    /// Infer method call type using trait resolver
    fn infer_method_call(
        &mut self,
        field_expr: &Node,
        _call_node: &Node,
        source: &str,
    ) -> RustType {
        if let (Some(value), Some(field)) = (
            field_expr.child_by_field_name("value"),
            field_expr.child_by_field_name("field"),
        ) {
            let receiver_type = self.infer_expr(&value, source);
            let method_name = &source[field.start_byte()..field.end_byte()];

            // Try to resolve the method using the trait resolver
            if let Some(resolution) = self
                .trait_resolver
                .resolve_method(&receiver_type, method_name)
            {
                return resolution.method.signature.return_type.clone();
            }

            // Fallback: check if it's a field that's callable
            let field_type = self.resolve_field_type(&receiver_type, method_name);
            match &field_type {
                RustType::FnPointer { return_type, .. } => return *return_type.clone(),
                RustType::Closure { return_type, .. } => return *return_type.clone(),
                _ => {}
            }

            // Method not found - generate a fresh type var
            self.context.fresh_type_var("method_result")
        } else {
            RustType::Infer
        }
    }

    /// Infer field expression type
    fn infer_field_expr(&mut self, node: &Node, source: &str) -> RustType {
        if let (Some(value), Some(field)) = (
            node.child_by_field_name("value"),
            node.child_by_field_name("field"),
        ) {
            let base_type = self.infer_expr(&value, source);
            let field_name = &source[field.start_byte()..field.end_byte()];

            self.resolve_field_type(&base_type, field_name)
        } else {
            RustType::Infer
        }
    }

    /// Resolve field type from a struct/enum type
    fn resolve_field_type(&self, base_type: &RustType, field_name: &str) -> RustType {
        match base_type {
            RustType::Named { name, .. } => {
                if let Some(struct_def) = self.context.struct_defs.get(name) {
                    if let StructFields::Named(fields) = &struct_def.fields {
                        for field in fields {
                            if field.name == field_name {
                                return field.ty.clone();
                            }
                        }
                    }
                }
                RustType::Infer
            }
            RustType::Reference { inner, .. } => {
                // Auto-deref
                self.resolve_field_type(inner, field_name)
            }
            RustType::Tuple(elements) => {
                // Tuple field access (e.g., tuple.0)
                if let Ok(index) = field_name.parse::<usize>() {
                    elements.get(index).cloned().unwrap_or(RustType::Error)
                } else {
                    RustType::Error
                }
            }
            _ => RustType::Infer,
        }
    }

    /// Infer index expression type
    fn infer_index_expr(&mut self, node: &Node, source: &str) -> RustType {
        if let Some(value) = node.child_by_field_name("value") {
            let base_type = self.infer_expr(&value, source);

            match &base_type {
                RustType::Array { element, .. } => *element.clone(),
                RustType::Slice(element) => *element.clone(),
                RustType::Reference { inner, .. } => {
                    // Auto-deref for slices/arrays
                    match inner.as_ref() {
                        RustType::Array { element, .. } => *element.clone(),
                        RustType::Slice(element) => *element.clone(),
                        _ => RustType::Infer,
                    }
                }
                _ => RustType::Infer,
            }
        } else {
            RustType::Infer
        }
    }

    /// Infer reference expression type
    fn infer_reference_expr(&mut self, node: &Node, source: &str) -> RustType {
        let mutable = node.child_by_field_name("mutable_specifier").is_some();

        if let Some(value) = node.child_by_field_name("value") {
            let inner_type = self.infer_expr(&value, source);
            RustType::Reference {
                lifetime: Some(self.context.fresh_lifetime()),
                mutable,
                inner: Box::new(inner_type),
            }
        } else {
            RustType::Infer
        }
    }

    /// Infer dereference expression type
    fn infer_deref_expr(&mut self, node: &Node, source: &str) -> RustType {
        if let Some(value) = node.child_by_field_name("value") {
            let ptr_type = self.infer_expr(&value, source);

            match ptr_type {
                RustType::Reference { inner, .. } => *inner,
                RustType::RawPointer { inner, .. } => *inner,
                _ => {
                    // Try Deref trait
                    RustType::Infer
                }
            }
        } else {
            RustType::Infer
        }
    }

    /// Infer binary expression type
    fn infer_binary_expr(&mut self, node: &Node, source: &str) -> RustType {
        if let (Some(left), Some(op), Some(right)) = (
            node.child_by_field_name("left"),
            node.child_by_field_name("operator"),
            node.child_by_field_name("right"),
        ) {
            let left_type = self.infer_expr(&left, source);
            let _right_type = self.infer_expr(&right, source);
            let operator = &source[op.start_byte()..op.end_byte()];

            // Comparison operators return bool
            if matches!(
                operator,
                "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&" | "||"
            ) {
                return RustType::Bool;
            }

            // Arithmetic operators return the same type
            left_type
        } else {
            RustType::Infer
        }
    }

    /// Infer unary expression type
    fn infer_unary_expr(&mut self, node: &Node, source: &str) -> RustType {
        if let (Some(op), Some(value)) = (
            node.child_by_field_name("operator"),
            node.child_by_field_name("value"),
        ) {
            let inner_type = self.infer_expr(&value, source);
            let operator = &source[op.start_byte()..op.end_byte()];

            match operator {
                "!" => RustType::Bool,
                "-" => inner_type,
                "*" => match inner_type {
                    RustType::Reference { inner, .. } => *inner,
                    RustType::RawPointer { inner, .. } => *inner,
                    _ => RustType::Infer,
                },
                _ => inner_type,
            }
        } else {
            RustType::Infer
        }
    }

    /// Infer if expression type
    fn infer_if_expr(&mut self, node: &Node, source: &str) -> RustType {
        if let Some(consequence) = node.child_by_field_name("consequence") {
            self.infer_block(&consequence, source)
        } else {
            RustType::Unit
        }
    }

    /// Infer match expression type
    fn infer_match_expr(&mut self, node: &Node, source: &str) -> RustType {
        // Get type from first arm
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "match_arm" {
                if let Some(value) = child.child_by_field_name("value") {
                    return self.infer_expr(&value, source);
                }
            }
        }
        RustType::Never
    }

    /// Infer block type (type of last expression)
    fn infer_block(&mut self, node: &Node, source: &str) -> RustType {
        let mut last_type = RustType::Unit;
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "expression_statement" => {
                    if let Some(expr) = child.child(0) {
                        let _ = self.infer_expr(&expr, source);
                    }
                    last_type = RustType::Unit; // Statement, not expression
                }
                "let_declaration" => {
                    self.process_let_declaration(&child, source);
                    last_type = RustType::Unit;
                }
                _ if !matches!(child.kind(), "{" | "}") => {
                    // Last expression without semicolon
                    last_type = self.infer_expr(&child, source);
                }
                _ => {}
            }
        }

        last_type
    }

    /// Process let declaration and add binding
    fn process_let_declaration(&mut self, node: &Node, source: &str) {
        if let (Some(pattern), Some(value)) = (
            node.child_by_field_name("pattern"),
            node.child_by_field_name("value"),
        ) {
            let value_type = self.infer_expr(&value, source);
            let pattern_name = &source[pattern.start_byte()..pattern.end_byte()];

            // Check for explicit type annotation
            if let Some(type_node) = node.child_by_field_name("type") {
                let annotated_type = self.parse_type(&type_node, source);
                self.context
                    .bind_type(pattern_name.to_string(), annotated_type);
            } else {
                self.context.bind_type(pattern_name.to_string(), value_type);
            }
        }
    }

    /// Infer tuple expression type
    fn infer_tuple_expr(&mut self, node: &Node, source: &str) -> RustType {
        let mut elements = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if !matches!(child.kind(), "(" | ")" | ",") {
                elements.push(self.infer_expr(&child, source));
            }
        }

        RustType::Tuple(elements)
    }

    /// Infer array expression type
    fn infer_array_expr(&mut self, node: &Node, source: &str) -> RustType {
        let mut elements = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if !matches!(child.kind(), "[" | "]" | "," | ";") {
                elements.push(self.infer_expr(&child, source));
            }
        }

        if let Some(first) = elements.first() {
            RustType::Array {
                element: Box::new(first.clone()),
                size: elements.len(),
            }
        } else {
            RustType::Array {
                element: Box::new(RustType::Infer),
                size: 0,
            }
        }
    }

    /// Infer struct expression type
    fn infer_struct_expr(&mut self, node: &Node, source: &str) -> RustType {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = &source[name_node.start_byte()..name_node.end_byte()];
            RustType::Named {
                name: name.to_string(),
                module: None,
                type_args: vec![],
                lifetime_args: vec![],
            }
        } else {
            RustType::Infer
        }
    }

    /// Infer closure expression type
    fn infer_closure_expr(&mut self, node: &Node, source: &str) -> RustType {
        let mut params = Vec::new();

        if let Some(params_node) = node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                if child.kind() == "parameter" {
                    params.push(RustType::Infer);
                }
            }
        }

        let return_type = if let Some(body) = node.child_by_field_name("body") {
            self.infer_expr(&body, source)
        } else {
            RustType::Infer
        };

        RustType::Closure {
            kind: ClosureKind::Fn, // Default, will be refined
            params,
            return_type: Box::new(return_type),
        }
    }

    /// Parse a type annotation
    fn parse_type(&mut self, node: &Node, source: &str) -> RustType {
        let kind = node.kind();

        match kind {
            "primitive_type" => {
                let text = &source[node.start_byte()..node.end_byte()];
                match text {
                    "bool" => RustType::Bool,
                    "char" => RustType::Char,
                    "str" => RustType::Str,
                    "i8" => RustType::I8,
                    "i16" => RustType::I16,
                    "i32" => RustType::I32,
                    "i64" => RustType::I64,
                    "i128" => RustType::I128,
                    "isize" => RustType::Isize,
                    "u8" => RustType::U8,
                    "u16" => RustType::U16,
                    "u32" => RustType::U32,
                    "u64" => RustType::U64,
                    "u128" => RustType::U128,
                    "usize" => RustType::Usize,
                    "f32" => RustType::F32,
                    "f64" => RustType::F64,
                    _ => RustType::Infer,
                }
            }
            "type_identifier" => {
                let name = &source[node.start_byte()..node.end_byte()];
                RustType::Named {
                    name: name.to_string(),
                    module: None,
                    type_args: vec![],
                    lifetime_args: vec![],
                }
            }
            "reference_type" => {
                let mutable = node.child_by_field_name("mutable_specifier").is_some();
                let inner = if let Some(type_node) = node.child_by_field_name("type") {
                    self.parse_type(&type_node, source)
                } else {
                    RustType::Infer
                };

                RustType::Reference {
                    lifetime: None,
                    mutable,
                    inner: Box::new(inner),
                }
            }
            "tuple_type" => {
                let mut elements = Vec::new();
                let mut cursor = node.walk();

                for child in node.children(&mut cursor) {
                    if !matches!(child.kind(), "(" | ")" | ",") {
                        elements.push(self.parse_type(&child, source));
                    }
                }

                RustType::Tuple(elements)
            }
            "array_type" => {
                let element = if let Some(elem) = node.child_by_field_name("element") {
                    self.parse_type(&elem, source)
                } else {
                    RustType::Infer
                };

                // Parse size expression: [T; N]
                // The size is the "length" field in tree-sitter-rust, or
                // the expression after the semicolon.
                let size = node
                    .child_by_field_name("length")
                    .and_then(|len_node| {
                        let text = &source[len_node.start_byte()..len_node.end_byte()];
                        self.parse_const_size_expr(text)
                    })
                    .unwrap_or(0);

                RustType::Array {
                    element: Box::new(element),
                    size,
                }
            }
            "unit_type" => RustType::Unit,
            "never_type" => RustType::Never,
            // Associated type projection: <T as Trait>::Item
            "scoped_type_identifier" | "qualified_type" => self.parse_qualified_path(node, source),
            // Generic type with type arguments: Vec<i32>
            "generic_type" => {
                let name = node
                    .child_by_field_name("type")
                    .map(|n| source[n.start_byte()..n.end_byte()].to_string())
                    .unwrap_or_default();

                let type_arg_nodes: Vec<Node> = node
                    .child_by_field_name("type_arguments")
                    .map(|args| {
                        let mut cursor = args.walk();
                        args.children(&mut cursor)
                            .filter(|c| !matches!(c.kind(), "<" | ">" | "," | "lifetime"))
                            .collect()
                    })
                    .unwrap_or_default();

                let type_args: Vec<RustType> = type_arg_nodes
                    .iter()
                    .map(|c| self.parse_type(c, source))
                    .collect();

                let lifetime_args: Vec<Lifetime> = node
                    .child_by_field_name("type_arguments")
                    .map(|args| {
                        let mut cursor = args.walk();
                        args.children(&mut cursor)
                            .filter(|c| c.kind() == "lifetime")
                            .map(|c| {
                                let lt_text = &source[c.start_byte()..c.end_byte()];
                                let lt_name = lt_text.trim_start_matches('\'');
                                if lt_name == "static" {
                                    Lifetime::Static
                                } else {
                                    self.context
                                        .lookup_lifetime(lt_name)
                                        .unwrap_or(Lifetime::Anonymous)
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                RustType::Named {
                    name,
                    module: None,
                    type_args,
                    lifetime_args,
                }
            }
            // Trait object: dyn Trait
            "dynamic_type" => {
                let mut bounds = Vec::new();
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() != "dyn" {
                        bounds.extend(self.parse_trait_bounds_from_node(&child, source));
                    }
                }
                RustType::TraitObject {
                    bounds,
                    lifetime: None,
                }
            }
            _ => RustType::Infer,
        }
    }

    /// Parse a const size expression for array types `[T; N]`.
    ///
    /// Handles integer literals, simple arithmetic on literals (e.g., `2 + 3`),
    /// and common const expressions. Returns `None` for expressions that
    /// cannot be evaluated statically (e.g., const generic parameters).
    fn parse_const_size_expr(&self, text: &str) -> Option<usize> {
        let text = text.trim();

        // Direct integer literal
        if let Ok(n) = text.parse::<usize>() {
            return Some(n);
        }

        // Handle hex literals: 0x...
        if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
            return usize::from_str_radix(hex, 16).ok();
        }

        // Handle binary literals: 0b...
        if let Some(bin) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B")) {
            return usize::from_str_radix(bin, 2).ok();
        }

        // Handle octal literals: 0o...
        if let Some(oct) = text.strip_prefix("0o").or_else(|| text.strip_prefix("0O")) {
            return usize::from_str_radix(oct, 8).ok();
        }

        // Handle underscored literals: 1_000_000
        let no_underscores = text.replace('_', "");
        if let Ok(n) = no_underscores.parse::<usize>() {
            return Some(n);
        }

        // Handle simple binary arithmetic: A + B, A * B, A - B
        for op in [" + ", " * ", " - "] {
            if let Some(pos) = text.find(op) {
                let left = text[..pos].trim();
                let right = text[pos + op.len()..].trim();
                if let (Some(l), Some(r)) = (
                    self.parse_const_size_expr(left),
                    self.parse_const_size_expr(right),
                ) {
                    return match op.trim() {
                        "+" => Some(l + r),
                        "*" => Some(l * r),
                        "-" => l.checked_sub(r),
                        _ => None,
                    };
                }
            }
        }

        // Cannot evaluate — likely a const generic parameter (N)
        None
    }

    /// Parse a type annotation that may include complex trait bounds.
    ///
    /// Handles `Fn(A) -> B + Send + 'static` style compound bounds:
    /// - `Fn(A, B) -> C`: parsed as a `Closure` type
    /// - `+ Send`: additional auto-trait bound
    /// - `+ 'static`: lifetime bound
    ///
    /// Returns a list of trait bounds parsed from the text.
    pub fn parse_trait_bounds_from_node(&mut self, node: &Node, source: &str) -> Vec<TraitBound> {
        let mut bounds = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                // Each bound separated by '+'
                "trait_bound"
                | "type_identifier"
                | "scoped_type_identifier"
                | "generic_type"
                | "function_type" => {
                    let text = &source[child.start_byte()..child.end_byte()];

                    // Check for Fn/FnMut/FnOnce trait syntax
                    if text.starts_with("Fn(")
                        || text.starts_with("FnMut(")
                        || text.starts_with("FnOnce(")
                    {
                        // Parse as function trait bound — we still create a
                        // TraitBound wrapping the Fn trait family.
                        let trait_name = if text.starts_with("FnOnce") {
                            "FnOnce"
                        } else if text.starts_with("FnMut") {
                            "FnMut"
                        } else {
                            "Fn"
                        };

                        bounds.push(TraitBound {
                            trait_ref: TraitRef {
                                trait_id: TraitId(0),
                                name: trait_name.to_string(),
                                type_args: vec![self.parse_type(&child, source)],
                                lifetime_args: vec![],
                            },
                            is_negative: false,
                            higher_ranked_lifetimes: vec![],
                        });
                    } else {
                        // Regular trait bound
                        let name = text.to_string();
                        bounds.push(TraitBound {
                            trait_ref: TraitRef {
                                trait_id: TraitId(0),
                                name,
                                type_args: vec![],
                                lifetime_args: vec![],
                            },
                            is_negative: false,
                            higher_ranked_lifetimes: vec![],
                        });
                    }
                }
                "lifetime" => {
                    // Lifetime bound like 'static
                    let lt_text = &source[child.start_byte()..child.end_byte()];
                    let lt_name = lt_text.trim_start_matches('\'');
                    let lifetime = if lt_name == "static" {
                        Lifetime::Static
                    } else {
                        self.context
                            .lookup_lifetime(lt_name)
                            .unwrap_or(Lifetime::Anonymous)
                    };

                    // Lifetime bounds are represented as a trait bound with
                    // lifetime args on a synthetic outlives predicate.
                    bounds.push(TraitBound {
                        trait_ref: TraitRef {
                            trait_id: TraitId(0),
                            name: format!("'{}", lt_name),
                            type_args: vec![],
                            lifetime_args: vec![lifetime],
                        },
                        is_negative: false,
                        higher_ranked_lifetimes: vec![],
                    });
                }
                // Skip '+' separators and other punctuation
                _ => {}
            }
        }

        bounds
    }

    /// Parse an associated type projection: `<T as Trait>::Item`
    ///
    /// This is a qualified path where we extract:
    /// - The base type `T`
    /// - The trait `Trait`
    /// - The associated type name `Item`
    pub fn parse_qualified_path(&mut self, node: &Node, source: &str) -> RustType {
        // tree-sitter-rust represents `<T as Trait>::Item` as:
        //   scoped_type_identifier or qualified_type
        //     type: qualified_type
        //       type: T
        //       trait: Trait
        //     name: Item
        let text = &source[node.start_byte()..node.end_byte()];

        // Try to parse from the AST structure first
        if let Some(type_node) = node.child_by_field_name("type") {
            let base = self.parse_type(&type_node, source);

            // Look for the "as" trait
            let trait_ref = node.child_by_field_name("trait").map(|trait_node| {
                let trait_name = source[trait_node.start_byte()..trait_node.end_byte()].to_string();
                TraitRef {
                    trait_id: TraitId(0),
                    name: trait_name,
                    type_args: vec![],
                    lifetime_args: vec![],
                }
            });

            // Look for the associated type name
            let assoc_name = node
                .child_by_field_name("name")
                .map(|n| source[n.start_byte()..n.end_byte()].to_string())
                .unwrap_or_default();

            if !assoc_name.is_empty() {
                return RustType::Projection {
                    base: Box::new(base),
                    trait_ref,
                    name: assoc_name,
                };
            }
        }

        // Fallback: attempt text-based parsing for `<T as Trait>::Name`
        if text.starts_with('<') {
            if let Some(as_pos) = text.find(" as ") {
                let base_text = &text[1..as_pos];
                let rest = &text[as_pos + 4..];
                if let Some(gt_pos) = rest.find(">::") {
                    let trait_name = &rest[..gt_pos];
                    let assoc_name = &rest[gt_pos + 3..];

                    let base = self.parse_type_from_text(base_text);
                    let trait_ref = Some(TraitRef {
                        trait_id: TraitId(0),
                        name: trait_name.to_string(),
                        type_args: vec![],
                        lifetime_args: vec![],
                    });

                    return RustType::Projection {
                        base: Box::new(base),
                        trait_ref,
                        name: assoc_name.to_string(),
                    };
                }
            }
        }

        RustType::Infer
    }

    /// Parse a type from raw text (fallback for text-based parsing)
    fn parse_type_from_text(&mut self, text: &str) -> RustType {
        let text = text.trim();
        match text {
            "bool" => RustType::Bool,
            "char" => RustType::Char,
            "str" => RustType::Str,
            "i8" => RustType::I8,
            "i16" => RustType::I16,
            "i32" => RustType::I32,
            "i64" => RustType::I64,
            "i128" => RustType::I128,
            "isize" => RustType::Isize,
            "u8" => RustType::U8,
            "u16" => RustType::U16,
            "u32" => RustType::U32,
            "u64" => RustType::U64,
            "u128" => RustType::U128,
            "usize" => RustType::Usize,
            "f32" => RustType::F32,
            "f64" => RustType::F64,
            "()" => RustType::Unit,
            "!" => RustType::Never,
            "_" => RustType::Infer,
            _ => RustType::Named {
                name: text.to_string(),
                module: None,
                type_args: vec![],
                lifetime_args: vec![],
            },
        }
    }

    /// Apply Rust lifetime elision rules to a function signature.
    ///
    /// Rust's lifetime elision rules (RFC 141):
    /// 1. Each elided lifetime in input position becomes a distinct lifetime parameter.
    /// 2. If there is exactly one input lifetime (elided or explicit), that lifetime
    ///    is assigned to all elided output lifetimes.
    /// 3. If there are multiple input lifetimes but one of them is `&self` or
    ///    `&mut self`, the lifetime of `self` is assigned to all elided output lifetimes.
    ///
    /// Returns the output type with elided lifetimes filled in.
    pub fn apply_lifetime_elision(
        &mut self,
        self_param: &Option<SelfParam>,
        params: &[RustType],
        return_type: &RustType,
    ) -> RustType {
        // Collect all input lifetimes (explicit ones from params)
        let mut input_lifetimes: Vec<Lifetime> = Vec::new();
        let mut self_lifetime: Option<Lifetime> = None;

        // Check self parameter
        match self_param {
            Some(SelfParam::Ref(lt)) => {
                let lt = lt.clone().unwrap_or_else(|| self.context.fresh_lifetime());
                self_lifetime = Some(lt.clone());
                input_lifetimes.push(lt);
            }
            Some(SelfParam::RefMut(lt)) => {
                let lt = lt.clone().unwrap_or_else(|| self.context.fresh_lifetime());
                self_lifetime = Some(lt.clone());
                input_lifetimes.push(lt);
            }
            _ => {}
        }

        // Collect lifetimes from reference parameters
        for param in params {
            Self::collect_input_lifetimes(param, &mut input_lifetimes, &mut self.context);
        }

        // Determine the output lifetime according to elision rules
        let output_lifetime = if let Some(lt) = self_lifetime {
            // Rule 3: self lifetime wins
            Some(lt)
        } else if input_lifetimes.len() == 1 {
            // Rule 2: single input lifetime
            Some(input_lifetimes[0].clone())
        } else {
            // Multiple inputs without self: cannot elide, leave as-is
            None
        };

        // Apply the output lifetime to all elided positions in return type
        if let Some(lt) = output_lifetime {
            Self::fill_elided_lifetimes(return_type, &lt)
        } else {
            return_type.clone()
        }
    }

    /// Collect input lifetimes from a parameter type
    fn collect_input_lifetimes(
        ty: &RustType,
        lifetimes: &mut Vec<Lifetime>,
        ctx: &mut RustTypeContext,
    ) {
        match ty {
            RustType::Reference {
                lifetime, inner, ..
            } => {
                let lt = lifetime.clone().unwrap_or_else(|| ctx.fresh_lifetime());
                lifetimes.push(lt);
                // Recurse into inner type for nested references
                Self::collect_input_lifetimes(inner, lifetimes, ctx);
            }
            RustType::Named {
                lifetime_args,
                type_args,
                ..
            } => {
                lifetimes.extend(lifetime_args.iter().cloned());
                for arg in type_args {
                    Self::collect_input_lifetimes(arg, lifetimes, ctx);
                }
            }
            RustType::Tuple(elements) => {
                for elem in elements {
                    Self::collect_input_lifetimes(elem, lifetimes, ctx);
                }
            }
            RustType::Slice(inner) | RustType::Array { element: inner, .. } => {
                Self::collect_input_lifetimes(inner, lifetimes, ctx);
            }
            _ => {}
        }
    }

    /// Fill in elided lifetimes in an output type with the given lifetime
    fn fill_elided_lifetimes(ty: &RustType, lt: &Lifetime) -> RustType {
        match ty {
            RustType::Reference {
                lifetime,
                mutable,
                inner,
            } => {
                let filled_lt = if lifetime.is_none() {
                    Some(lt.clone())
                } else {
                    lifetime.clone()
                };
                RustType::Reference {
                    lifetime: filled_lt,
                    mutable: *mutable,
                    inner: Box::new(Self::fill_elided_lifetimes(inner, lt)),
                }
            }
            RustType::Named {
                name,
                module,
                type_args,
                lifetime_args,
            } => RustType::Named {
                name: name.clone(),
                module: module.clone(),
                type_args: type_args
                    .iter()
                    .map(|t| Self::fill_elided_lifetimes(t, lt))
                    .collect(),
                lifetime_args: lifetime_args.clone(),
            },
            RustType::Tuple(elements) => RustType::Tuple(
                elements
                    .iter()
                    .map(|e| Self::fill_elided_lifetimes(e, lt))
                    .collect(),
            ),
            RustType::Slice(inner) => {
                RustType::Slice(Box::new(Self::fill_elided_lifetimes(inner, lt)))
            }
            RustType::Array { element, size } => RustType::Array {
                element: Box::new(Self::fill_elided_lifetimes(element, lt)),
                size: *size,
            },
            other => other.clone(),
        }
    }

    /// Unify two types
    pub fn unify(&mut self, t1: &RustType, t2: &RustType) -> Result<RustType, RustTypeError> {
        match (t1, t2) {
            // Same types
            (a, b) if a == b => Ok(a.clone()),

            // Infer can unify with anything
            (RustType::Infer, other) | (other, RustType::Infer) => Ok(other.clone()),

            // Type parameters
            (RustType::TypeParam { id, .. }, other) | (other, RustType::TypeParam { id, .. }) => {
                if let Some(existing) = self.substitutions.get(id).cloned() {
                    self.unify(&existing, other)
                } else {
                    self.substitutions.insert(*id, other.clone());
                    Ok(other.clone())
                }
            }

            // References
            (
                RustType::Reference {
                    mutable: m1,
                    inner: i1,
                    ..
                },
                RustType::Reference {
                    mutable: m2,
                    inner: i2,
                    ..
                },
            ) => {
                if m1 != m2 {
                    return Err(RustTypeError {
                        message: "Mutability mismatch".to_string(),
                        span: None,
                        kind: RustTypeErrorKind::TypeMismatch,
                    });
                }
                let unified = self.unify(i1, i2)?;
                Ok(RustType::Reference {
                    lifetime: None,
                    mutable: *m1,
                    inner: Box::new(unified),
                })
            }

            // Tuples
            (RustType::Tuple(t1s), RustType::Tuple(t2s)) => {
                if t1s.len() != t2s.len() {
                    return Err(RustTypeError {
                        message: "Tuple length mismatch".to_string(),
                        span: None,
                        kind: RustTypeErrorKind::TypeMismatch,
                    });
                }
                let unified: Result<Vec<_>, _> = t1s
                    .iter()
                    .zip(t2s.iter())
                    .map(|(a, b)| self.unify(a, b))
                    .collect();
                Ok(RustType::Tuple(unified?))
            }

            // Arrays
            (
                RustType::Array {
                    element: e1,
                    size: s1,
                },
                RustType::Array {
                    element: e2,
                    size: s2,
                },
            ) => {
                if s1 != s2 {
                    return Err(RustTypeError {
                        message: "Array size mismatch".to_string(),
                        span: None,
                        kind: RustTypeErrorKind::TypeMismatch,
                    });
                }
                let unified = self.unify(e1, e2)?;
                Ok(RustType::Array {
                    element: Box::new(unified),
                    size: *s1,
                })
            }

            // Type mismatch
            _ => Err(RustTypeError {
                message: format!("Cannot unify {:?} with {:?}", t1, t2),
                span: None,
                kind: RustTypeErrorKind::TypeMismatch,
            }),
        }
    }

    /// Apply substitutions to a type
    pub fn apply_substitutions(&self, ty: &RustType) -> RustType {
        match ty {
            RustType::TypeParam { id, .. } => {
                if let Some(subst) = self.substitutions.get(id) {
                    self.apply_substitutions(subst)
                } else {
                    ty.clone()
                }
            }
            RustType::Reference {
                lifetime,
                mutable,
                inner,
            } => RustType::Reference {
                lifetime: lifetime.clone(),
                mutable: *mutable,
                inner: Box::new(self.apply_substitutions(inner)),
            },
            RustType::Tuple(elements) => RustType::Tuple(
                elements
                    .iter()
                    .map(|e| self.apply_substitutions(e))
                    .collect(),
            ),
            RustType::Array { element, size } => RustType::Array {
                element: Box::new(self.apply_substitutions(element)),
                size: *size,
            },
            RustType::Named {
                name,
                module,
                type_args,
                lifetime_args,
            } => RustType::Named {
                name: name.clone(),
                module: module.clone(),
                type_args: type_args
                    .iter()
                    .map(|t| self.apply_substitutions(t))
                    .collect(),
                lifetime_args: lifetime_args.clone(),
            },
            _ => ty.clone(),
        }
    }
}

impl Default for RustTypeInferencer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresh_type_var() {
        let mut ctx = RustTypeContext::new();
        let t1 = ctx.fresh_type_var("T");
        let t2 = ctx.fresh_type_var("U");

        match (&t1, &t2) {
            (
                RustType::TypeParam {
                    id: id1,
                    name: name1,
                    ..
                },
                RustType::TypeParam {
                    id: id2,
                    name: name2,
                    ..
                },
            ) => {
                assert_ne!(id1, id2);
                assert_eq!(name1, "T");
                assert_eq!(name2, "U");
            }
            _ => panic!("Expected TypeParam"),
        }
    }

    #[test]
    fn test_type_binding() {
        let mut ctx = RustTypeContext::new();
        ctx.bind_type("x".to_string(), RustType::I32);

        assert_eq!(ctx.lookup_type("x"), Some(RustType::I32));
        assert_eq!(ctx.lookup_type("y"), None);
    }

    #[test]
    fn test_unify_same_types() {
        let mut inferencer = RustTypeInferencer::new();

        let result = inferencer.unify(&RustType::I32, &RustType::I32);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RustType::I32);
    }

    #[test]
    fn test_unify_with_infer() {
        let mut inferencer = RustTypeInferencer::new();

        let result = inferencer.unify(&RustType::Infer, &RustType::I32);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RustType::I32);
    }

    #[test]
    fn test_unify_type_mismatch() {
        let mut inferencer = RustTypeInferencer::new();

        let result = inferencer.unify(&RustType::I32, &RustType::Str);
        assert!(result.is_err());
    }
}
