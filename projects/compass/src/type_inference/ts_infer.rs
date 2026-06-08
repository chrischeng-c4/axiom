//! TypeScript type inference engine
//!
//! Implements TypeScript-specific type inference including:
//! - Generic type inference with constraints
//! - Union and intersection type handling
//! - Structural subtyping for interfaces
//! - Control flow type narrowing
//! - Literal types and template literals

use std::collections::HashMap;

use tree_sitter::Node;

#[allow(unused_imports)]
use super::ts_types::{is_assignable_to, TsInterface, TsTypeContext};
use super::ty::{LiteralValue, Param, ParamKind, Type, TypeVarId, Variance};

/// TypeScript type inferencer
pub struct TsTypeInferencer<'a> {
    /// Source code
    source: &'a str,
    /// Type context (interfaces, classes, aliases)
    context: TsTypeContext,
    /// Type variable substitutions
    #[allow(dead_code)]
    substitutions: HashMap<TypeVarId, Type>,
    /// Control flow type narrowing
    narrowed_types: HashMap<String, Type>,
    /// Counter for fresh type variables
    next_type_var_id: usize,
    /// Current scope depth
    scope_depth: usize,
}

impl<'a> TsTypeInferencer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            context: TsTypeContext::new(),
            substitutions: HashMap::new(),
            narrowed_types: HashMap::new(),
            next_type_var_id: 0,
            scope_depth: 0,
        }
    }

    /// Get the type context
    pub fn context(&self) -> &TsTypeContext {
        &self.context
    }

    /// Get mutable access to the type context
    pub fn context_mut(&mut self) -> &mut TsTypeContext {
        &mut self.context
    }

    /// Get node text
    fn node_text(&self, node: &Node) -> &str {
        node.utf8_text(self.source.as_bytes()).unwrap_or("")
    }

    /// Generate a fresh type variable
    pub fn fresh_type_var(&mut self, name: &str) -> Type {
        let id = TypeVarId(self.next_type_var_id);
        self.next_type_var_id += 1;
        Type::TypeVar {
            id,
            name: name.to_string(),
            bound: None,
            constraints: vec![],
            variance: Variance::Invariant,
        }
    }

    /// Generate a fresh type variable with constraint
    pub fn fresh_type_var_bounded(&mut self, name: &str, bound: Type) -> Type {
        let id = TypeVarId(self.next_type_var_id);
        self.next_type_var_id += 1;
        Type::TypeVar {
            id,
            name: name.to_string(),
            bound: Some(Box::new(bound)),
            constraints: vec![],
            variance: Variance::Invariant,
        }
    }

    /// Infer the type of an expression
    pub fn infer_expr(&mut self, node: &Node) -> Type {
        if node.is_error() || node.is_missing() {
            return Type::Error;
        }

        match node.kind() {
            // Literals
            "number" => self.infer_number_literal(node),
            "string" | "template_string" => self.infer_string_literal(node),
            "true" => Type::Literal(LiteralValue::Bool(true)),
            "false" => Type::Literal(LiteralValue::Bool(false)),
            "null" => Type::None,
            "undefined" => Type::None,

            // Identifier lookup
            "identifier" => self.infer_identifier(node),

            // Binary expressions
            "binary_expression" => self.infer_binary_expr(node),

            // Unary expressions
            "unary_expression" => self.infer_unary_expr(node),

            // Call expression
            "call_expression" => self.infer_call_expr(node),

            // Member expression (property access)
            "member_expression" => self.infer_member_expr(node),

            // Object literal
            "object" => self.infer_object_literal(node),

            // Array literal
            "array" => self.infer_array_literal(node),

            // Arrow function
            "arrow_function" => self.infer_arrow_function(node),

            // Function expression
            "function_expression" | "function" => self.infer_function_expr(node),

            // Conditional (ternary)
            "ternary_expression" => self.infer_ternary_expr(node),

            // As expression (type assertion)
            "as_expression" => self.infer_as_expr(node),

            // Type assertion (<Type>expr)
            "type_assertion" => self.infer_type_assertion(node),

            // Parenthesized expression
            "parenthesized_expression" => {
                if let Some(inner) = node.child(1) {
                    self.infer_expr(&inner)
                } else {
                    Type::Unknown
                }
            }

            // Await expression
            "await_expression" => {
                if let Some(arg) = node.child(1) {
                    // Unwrap Promise<T> -> T
                    let inner_ty = self.infer_expr(&arg);
                    self.unwrap_promise(inner_ty)
                } else {
                    Type::Unknown
                }
            }

            // New expression
            "new_expression" => self.infer_new_expr(node),

            _ => Type::Unknown,
        }
    }

    /// Infer number literal (could be int or float, with literal type)
    fn infer_number_literal(&self, node: &Node) -> Type {
        let text = self.node_text(node);
        if text.contains('.') || text.contains('e') || text.contains('E') {
            if let Ok(n) = text.parse::<f64>() {
                return Type::Literal(LiteralValue::Float(n));
            }
        } else if let Ok(n) = text.parse::<i64>() {
            return Type::Literal(LiteralValue::Int(n));
        }
        Type::Float // TypeScript uses 'number' for both
    }

    /// Infer string literal
    fn infer_string_literal(&self, node: &Node) -> Type {
        let text = self.node_text(node);
        // Remove quotes
        let content = text
            .trim_start_matches(|c| c == '"' || c == '\'' || c == '`')
            .trim_end_matches(|c| c == '"' || c == '\'' || c == '`');

        if node.kind() == "template_string" {
            // Check if template has interpolations
            let mut cursor = node.walk();
            let has_interpolation = node
                .children(&mut cursor)
                .any(|c| c.kind() == "template_substitution");

            if has_interpolation {
                return Type::Str; // Complex template, return string
            }
        }

        Type::Literal(LiteralValue::Str(content.to_string()))
    }

    /// Infer identifier type
    fn infer_identifier(&mut self, node: &Node) -> Type {
        let name = self.node_text(node);

        // Check narrowed types first (control flow analysis)
        if let Some(ty) = self.narrowed_types.get(name) {
            return ty.clone();
        }

        // Check variable bindings
        if let Some(ty) = self.context.variables.get(name) {
            return ty.clone();
        }

        // Check for type references
        if let Some(ty) = self.context.resolve_type(name) {
            return ty;
        }

        Type::Unknown
    }

    /// Infer binary expression type
    fn infer_binary_expr(&mut self, node: &Node) -> Type {
        let left = node.child_by_field_name("left");
        let right = node.child_by_field_name("right");
        let op = node.child_by_field_name("operator");

        // Extract op_text first before mutable borrows
        let op_text = op
            .map(|o| o.utf8_text(self.source.as_bytes()).unwrap_or(""))
            .unwrap_or("");

        let (left_ty, right_ty) = match (left, right) {
            (Some(l), Some(r)) => (self.infer_expr(&l), self.infer_expr(&r)),
            _ => return Type::Unknown,
        };

        match op_text {
            // Arithmetic
            "+" => match (&left_ty, &right_ty) {
                (Type::Str, _) | (_, Type::Str) => Type::Str,
                (Type::Literal(LiteralValue::Str(_)), _)
                | (_, Type::Literal(LiteralValue::Str(_))) => Type::Str,
                _ => Type::Float, // TypeScript number
            },
            "-" | "*" | "/" | "%" | "**" => Type::Float,

            // Comparison
            "==" | "===" | "!=" | "!==" | "<" | ">" | "<=" | ">=" => Type::Bool,

            // Logical
            "&&" => {
                // Returns right if left is truthy
                if is_falsy(&left_ty) {
                    left_ty
                } else {
                    right_ty
                }
            }
            "||" => {
                // Returns left if truthy, else right
                if is_truthy(&left_ty) {
                    left_ty
                } else {
                    Type::union(vec![left_ty, right_ty])
                }
            }
            "??" => {
                // Nullish coalescing: left if not null/undefined
                match &left_ty {
                    Type::None | Type::Optional(_) => right_ty,
                    _ => left_ty,
                }
            }

            // Bitwise
            "&" | "|" | "^" | "<<" | ">>" | ">>>" => Type::Int,

            // in / instanceof
            "in" | "instanceof" => Type::Bool,

            _ => Type::Unknown,
        }
    }

    /// Infer unary expression type
    fn infer_unary_expr(&mut self, node: &Node) -> Type {
        let op = node.child_by_field_name("operator");
        let arg = node.child_by_field_name("argument");

        let op_text = op.map(|o| self.node_text(&o)).unwrap_or("");

        match op_text {
            "!" => Type::Bool,
            "+" | "-" => Type::Float,
            "~" => Type::Int,
            "typeof" => Type::Str,
            "void" => Type::None,
            "delete" => Type::Bool,
            _ => {
                if let Some(a) = arg {
                    self.infer_expr(&a)
                } else {
                    Type::Unknown
                }
            }
        }
    }

    /// Infer call expression with generic type inference
    pub fn infer_call_expr(&mut self, node: &Node) -> Type {
        let function = match node.child_by_field_name("function") {
            Some(f) => f,
            None => return Type::Unknown,
        };

        let func_ty = self.infer_expr(&function);

        match func_ty {
            Type::Callable { params, ret } => {
                // Collect argument types
                let arg_types = self.collect_arguments(node);

                // Check for type variables that need inference
                let type_vars = ret.type_vars();
                if type_vars.is_empty() {
                    return (*ret).clone();
                }

                // Unify parameters with arguments to infer type variables
                let mut subs = HashMap::new();
                for (param, arg_ty) in params.iter().zip(arg_types.iter()) {
                    param.ty.unify(arg_ty, &mut subs);
                }

                // Apply substitutions to return type
                ret.substitute(&subs)
            }
            Type::ClassType { name, .. } => {
                // Constructor call returns instance
                Type::Instance {
                    name,
                    module: None,
                    type_args: vec![],
                }
            }
            _ => Type::Unknown,
        }
    }

    /// Collect argument types from a call expression
    fn collect_arguments(&mut self, node: &Node) -> Vec<Type> {
        let mut args = Vec::new();

        if let Some(args_node) = node.child_by_field_name("arguments") {
            let mut cursor = args_node.walk();
            for child in args_node.children(&mut cursor) {
                if child.kind() != "(" && child.kind() != ")" && child.kind() != "," {
                    args.push(self.infer_expr(&child));
                }
            }
        }

        args
    }

    /// Infer member expression (property access)
    fn infer_member_expr(&mut self, node: &Node) -> Type {
        let object = match node.child_by_field_name("object") {
            Some(o) => o,
            None => return Type::Unknown,
        };
        let property = match node.child_by_field_name("property") {
            Some(p) => p,
            None => return Type::Unknown,
        };

        let object_ty = self.infer_expr(&object);
        let prop_name = self.node_text(&property);

        self.get_property_type(&object_ty, prop_name)
    }

    /// Get property type from a type
    fn get_property_type(&self, ty: &Type, prop: &str) -> Type {
        match ty {
            Type::Instance { name, .. } => {
                // Look up class
                if let Some(class) = self.context.classes.get(name) {
                    if let Some(prop_info) = class.properties.get(prop) {
                        return prop_info.ty.clone();
                    }
                    if let Some(method_ty) = class.methods.get(prop) {
                        return method_ty.clone();
                    }
                }
                // Look up interface
                if let Some(iface) = self.context.interfaces.get(name) {
                    if let Some(prop_ty) = iface.properties.get(prop) {
                        return prop_ty.clone();
                    }
                    if let Some(prop_ty) = iface.optional_properties.get(prop) {
                        return Type::optional(prop_ty.clone());
                    }
                    if let Some(method_ty) = iface.methods.get(prop) {
                        return method_ty.clone();
                    }
                }
                Type::Unknown
            }
            Type::Protocol { members, .. } => {
                for (name, member_ty) in members {
                    if name == prop {
                        return member_ty.clone();
                    }
                }
                Type::Unknown
            }
            Type::List(_) => {
                // Array methods
                match prop {
                    "length" => Type::Int,
                    "push" | "pop" | "shift" | "unshift" | "splice" | "slice" | "concat"
                    | "join" | "map" | "filter" | "reduce" | "forEach" => {
                        Type::Any // Simplified
                    }
                    _ => Type::Unknown,
                }
            }
            Type::Str => match prop {
                "length" => Type::Int,
                "charAt" | "substring" | "slice" | "trim" | "toLowerCase" | "toUpperCase"
                | "split" | "replace" => Type::Any,
                _ => Type::Unknown,
            },
            Type::Union(members) => {
                // Get property from all union members
                let prop_types: Vec<Type> = members
                    .iter()
                    .map(|m| self.get_property_type(m, prop))
                    .filter(|t| !matches!(t, Type::Unknown))
                    .collect();

                if prop_types.is_empty() {
                    Type::Unknown
                } else if prop_types.len() == 1 {
                    prop_types.into_iter().next().unwrap()
                } else {
                    Type::union(prop_types)
                }
            }
            Type::Optional(inner) => Type::optional(self.get_property_type(inner, prop)),
            _ => Type::Unknown,
        }
    }

    /// Infer object literal type
    fn infer_object_literal(&mut self, node: &Node) -> Type {
        let mut members: Vec<(String, Type)> = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "pair" | "property" => {
                    let key = child.child_by_field_name("key");
                    let value = child.child_by_field_name("value");

                    if let (Some(k), Some(v)) = (key, value) {
                        let key_name = k
                            .utf8_text(self.source.as_bytes())
                            .unwrap_or("")
                            .trim_matches(|c| c == '"' || c == '\'')
                            .to_string();
                        let value_ty = self.infer_expr(&v);
                        members.push((key_name, value_ty));
                    }
                }
                "shorthand_property_identifier" => {
                    let name = child.utf8_text(self.source.as_bytes()).unwrap_or("");
                    let ty = self
                        .context
                        .variables
                        .get(name)
                        .cloned()
                        .unwrap_or(Type::Unknown);
                    members.push((name.to_string(), ty));
                }
                "spread_element" => {
                    // Spread adds all properties from the spread target
                    if let Some(arg) = child.child(1) {
                        let spread_ty = self.infer_expr(&arg);
                        if let Type::Protocol {
                            members: spread_members,
                            ..
                        } = spread_ty
                        {
                            members.extend(spread_members);
                        }
                    }
                }
                _ => {}
            }
        }

        Type::Protocol {
            name: "".to_string(),
            module: None,
            members,
        }
    }

    /// Infer array literal type
    fn infer_array_literal(&mut self, node: &Node) -> Type {
        let mut element_types = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "[" && child.kind() != "]" && child.kind() != "," {
                element_types.push(self.infer_expr(&child));
            }
        }

        if element_types.is_empty() {
            Type::list(Type::Unknown)
        } else {
            // Compute union of all element types
            let unified = if element_types.iter().all(|t| t == &element_types[0]) {
                element_types[0].clone()
            } else {
                Type::union(element_types)
            };
            Type::list(unified)
        }
    }

    /// Infer arrow function type
    fn infer_arrow_function(&mut self, node: &Node) -> Type {
        let params = self.parse_function_params(node);

        // Infer return type from body
        let ret = if let Some(body) = node.child_by_field_name("body") {
            if body.kind() == "statement_block" {
                // Block body - would need to analyze return statements
                Type::Unknown
            } else {
                // Expression body
                self.infer_expr(&body)
            }
        } else {
            Type::Unknown
        };

        Type::Callable {
            params,
            ret: Box::new(ret),
        }
    }

    /// Infer function expression type
    fn infer_function_expr(&mut self, node: &Node) -> Type {
        let params = self.parse_function_params(node);

        // Check for explicit return type annotation
        let ret = if let Some(return_type) = node.child_by_field_name("return_type") {
            self.parse_type_annotation(&return_type)
        } else {
            Type::Unknown
        };

        Type::Callable {
            params,
            ret: Box::new(ret),
        }
    }

    /// Parse function parameters
    fn parse_function_params(&mut self, node: &Node) -> Vec<Param> {
        let mut params = Vec::new();

        if let Some(params_node) = node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                match child.kind() {
                    "required_parameter" | "optional_parameter" => {
                        let name = child
                            .child_by_field_name("pattern")
                            .or_else(|| child.child_by_field_name("name"))
                            .map(|n| self.node_text(&n).to_string())
                            .unwrap_or_default();

                        let ty = child
                            .child_by_field_name("type")
                            .map(|t| self.parse_type_annotation(&t))
                            .unwrap_or(Type::Unknown);

                        let has_default = child.child_by_field_name("value").is_some();
                        let optional = child.kind() == "optional_parameter";

                        params.push(Param {
                            name,
                            ty: if optional { Type::optional(ty) } else { ty },
                            has_default: has_default || optional,
                            kind: ParamKind::Positional,
                        });
                    }
                    "rest_parameter" => {
                        let name = child
                            .child_by_field_name("pattern")
                            .map(|n| self.node_text(&n).to_string())
                            .unwrap_or_default();

                        let ty = child
                            .child_by_field_name("type")
                            .map(|t| self.parse_type_annotation(&t))
                            .unwrap_or(Type::list(Type::Unknown));

                        params.push(Param {
                            name,
                            ty,
                            has_default: false,
                            kind: ParamKind::VarPositional,
                        });
                    }
                    _ => {}
                }
            }
        }

        params
    }

    /// Infer ternary expression type
    fn infer_ternary_expr(&mut self, node: &Node) -> Type {
        let condition = node.child_by_field_name("condition");
        let consequence = node.child_by_field_name("consequence");
        let alternative = node.child_by_field_name("alternative");

        match (consequence, alternative) {
            (Some(c), Some(a)) => {
                // Apply narrowing based on condition
                if let Some(cond) = condition {
                    self.apply_type_guard(&cond, true);
                }
                let conseq_ty = self.infer_expr(&c);

                // Restore and apply opposite narrowing
                self.narrowed_types.clear();
                if let Some(cond) = condition {
                    self.apply_type_guard(&cond, false);
                }
                let alt_ty = self.infer_expr(&a);

                self.narrowed_types.clear();

                if conseq_ty == alt_ty {
                    conseq_ty
                } else {
                    Type::union(vec![conseq_ty, alt_ty])
                }
            }
            _ => Type::Unknown,
        }
    }

    /// Infer 'as' expression (type assertion)
    fn infer_as_expr(&mut self, node: &Node) -> Type {
        if let Some(type_node) = node.child_by_field_name("type") {
            self.parse_type_annotation(&type_node)
        } else {
            Type::Unknown
        }
    }

    /// Infer type assertion (<Type>expr)
    fn infer_type_assertion(&mut self, node: &Node) -> Type {
        if let Some(type_node) = node.child_by_field_name("type") {
            self.parse_type_annotation(&type_node)
        } else {
            Type::Unknown
        }
    }

    /// Infer new expression
    fn infer_new_expr(&mut self, node: &Node) -> Type {
        let constructor = match node.child_by_field_name("constructor") {
            Some(c) => c,
            None => return Type::Unknown,
        };

        let constructor_name = constructor
            .utf8_text(self.source.as_bytes())
            .unwrap_or("")
            .to_string();

        // Parse type arguments if present (e.g., new Map<string, number>())
        let type_args = if let Some(type_args_node) = node.child_by_field_name("type_arguments") {
            let arg_nodes: Vec<Node> = {
                let mut cursor = type_args_node.walk();
                type_args_node
                    .children(&mut cursor)
                    .filter(|c| c.kind() != "<" && c.kind() != ">" && c.kind() != ",")
                    .collect()
            };
            arg_nodes
                .iter()
                .map(|c| self.parse_type_annotation(c))
                .collect()
        } else {
            vec![]
        };

        // Look up class
        if self.context.classes.contains_key(constructor_name.as_str()) {
            return Type::Instance {
                name: constructor_name,
                module: None,
                type_args,
            };
        }

        Type::Unknown
    }

    /// Parse type annotation from AST node
    pub fn parse_type_annotation(&mut self, node: &Node) -> Type {
        match node.kind() {
            // Predefined types
            "predefined_type" => {
                let text = self.node_text(node);
                match text {
                    "string" => Type::Str,
                    "number" => Type::Float,
                    "boolean" => Type::Bool,
                    "void" | "undefined" => Type::None,
                    "null" => Type::None,
                    "never" => Type::Never,
                    "any" => Type::Any,
                    "unknown" => Type::Unknown,
                    "object" => Type::Any, // Simplified
                    "symbol" => Type::Any, // Simplified
                    "bigint" => Type::Int,
                    _ => Type::Unknown,
                }
            }

            // Type identifier
            "type_identifier" | "identifier" => {
                let name = self.node_text(node);
                self.context.resolve_type(name).unwrap_or(Type::Unknown)
            }

            // Generic type: Array<T>, Promise<T>, etc.
            "generic_type" => self.parse_generic_type(node),

            // Union type: A | B
            "union_type" => {
                let mut cursor = node.walk();
                let types: Vec<Type> = node
                    .children(&mut cursor)
                    .filter(|c| c.kind() != "|")
                    .map(|c| self.parse_type_annotation(&c))
                    .collect();
                Type::union(types)
            }

            // Intersection type: A & B
            "intersection_type" => {
                let mut cursor = node.walk();
                let types: Vec<Type> = node
                    .children(&mut cursor)
                    .filter(|c| c.kind() != "&")
                    .map(|c| self.parse_type_annotation(&c))
                    .collect();
                Type::Intersection(types)
            }

            // Array type: T[]
            "array_type" => {
                if let Some(elem) = node.child(0) {
                    Type::list(self.parse_type_annotation(&elem))
                } else {
                    Type::list(Type::Unknown)
                }
            }

            // Tuple type: [A, B, C]
            "tuple_type" => {
                let mut cursor = node.walk();
                let types: Vec<Type> = node
                    .children(&mut cursor)
                    .filter(|c| c.kind() != "[" && c.kind() != "]" && c.kind() != ",")
                    .map(|c| self.parse_type_annotation(&c))
                    .collect();
                Type::Tuple(types)
            }

            // Function type: (a: A) => B
            "function_type" => self.parse_function_type(node),

            // Object type: { x: T, y: U }
            "object_type" => self.parse_object_type(node),

            // Literal type
            "literal_type" => {
                if let Some(child) = node.child(0) {
                    match child.kind() {
                        "string" => {
                            let text = self.node_text(&child);
                            let content = text.trim_matches(|c| c == '"' || c == '\'');
                            Type::Literal(LiteralValue::Str(content.to_string()))
                        }
                        "number" => {
                            let text = self.node_text(&child);
                            if let Ok(n) = text.parse::<i64>() {
                                Type::Literal(LiteralValue::Int(n))
                            } else if let Ok(n) = text.parse::<f64>() {
                                Type::Literal(LiteralValue::Float(n))
                            } else {
                                Type::Float
                            }
                        }
                        "true" => Type::Literal(LiteralValue::Bool(true)),
                        "false" => Type::Literal(LiteralValue::Bool(false)),
                        _ => Type::Unknown,
                    }
                } else {
                    Type::Unknown
                }
            }

            // Parenthesized type
            "parenthesized_type" => {
                if let Some(inner) = node.child(1) {
                    self.parse_type_annotation(&inner)
                } else {
                    Type::Unknown
                }
            }

            // Readonly type
            "readonly_type" => {
                if let Some(inner) = node.child(1) {
                    self.parse_type_annotation(&inner) // Just ignore readonly
                } else {
                    Type::Unknown
                }
            }

            // Conditional type: T extends U ? X : Y
            "conditional_type" => self.parse_conditional_type(node),

            // Indexed access type: T[K]
            "indexed_access_type" => self.parse_indexed_access_type(node),

            // Mapped type: { [K in keyof T]: V }
            "mapped_type_clause" | "mapped_type" => self.parse_mapped_type(node),

            // Template literal type: `hello ${string}`
            "template_literal_type" => self.parse_template_literal_type(node),

            // keyof type
            "keyof_type" | "type_query" => {
                // Simplified: return string | number | symbol
                Type::Union(vec![Type::Str, Type::Int])
            }

            _ => Type::Unknown,
        }
    }

    /// Parse generic type like Array<T> or Map<K, V>
    fn parse_generic_type(&mut self, node: &Node) -> Type {
        let name_node = node.child_by_field_name("name");
        let args_node = node.child_by_field_name("type_arguments");

        // Extract name as owned string first
        let name = name_node
            .and_then(|n| n.utf8_text(self.source.as_bytes()).ok())
            .unwrap_or("")
            .to_string();

        // Collect child nodes first, then parse
        let type_arg_nodes: Vec<Node> = args_node
            .map(|args| {
                let mut cursor = args.walk();
                args.children(&mut cursor)
                    .filter(|c| c.kind() != "<" && c.kind() != ">" && c.kind() != ",")
                    .collect()
            })
            .unwrap_or_default();

        let type_args: Vec<Type> = type_arg_nodes
            .iter()
            .map(|c| self.parse_type_annotation(c))
            .collect();

        // Handle built-in generics
        match name.as_str() {
            "Array" | "ReadonlyArray" => {
                if let Some(elem) = type_args.into_iter().next() {
                    Type::list(elem)
                } else {
                    Type::list(Type::Unknown)
                }
            }
            "Promise" | "PromiseLike" => {
                if let Some(elem) = type_args.into_iter().next() {
                    // Return Promise<T> as-is (could create a wrapper type)
                    Type::Instance {
                        name: "Promise".to_string(),
                        module: None,
                        type_args: vec![elem],
                    }
                } else {
                    Type::Instance {
                        name: "Promise".to_string(),
                        module: None,
                        type_args: vec![Type::Unknown],
                    }
                }
            }
            "Map" | "ReadonlyMap" => {
                if type_args.len() >= 2 {
                    Type::dict(type_args[0].clone(), type_args[1].clone())
                } else {
                    Type::dict(Type::Unknown, Type::Unknown)
                }
            }
            "Set" | "ReadonlySet" => {
                if let Some(elem) = type_args.into_iter().next() {
                    Type::Set(Box::new(elem))
                } else {
                    Type::Set(Box::new(Type::Unknown))
                }
            }
            "Partial" => {
                // Make all properties optional
                if let Some(inner) = type_args.into_iter().next() {
                    self.make_partial(inner)
                } else {
                    Type::Unknown
                }
            }
            "Required" => {
                // Make all properties required
                if let Some(inner) = type_args.into_iter().next() {
                    self.make_required(inner)
                } else {
                    Type::Unknown
                }
            }
            "Readonly" => {
                // Just return inner type (readonly not tracked)
                type_args.into_iter().next().unwrap_or(Type::Unknown)
            }
            "Record" => {
                if type_args.len() >= 2 {
                    Type::dict(type_args[0].clone(), type_args[1].clone())
                } else {
                    Type::dict(Type::Str, Type::Unknown)
                }
            }
            _ => {
                // User-defined generic type
                Type::Instance {
                    name: name.to_string(),
                    module: None,
                    type_args,
                }
            }
        }
    }

    /// Parse function type: (params) => ReturnType
    fn parse_function_type(&mut self, node: &Node) -> Type {
        let params_node = node.child_by_field_name("parameters");
        let return_node = node.child_by_field_name("return_type");

        let params: Vec<Param> = params_node
            .map(|pn| {
                let mut cursor = pn.walk();
                pn.children(&mut cursor)
                    .filter_map(|c| {
                        if c.kind() == "parameter" || c.kind() == "required_parameter" {
                            let name = c
                                .child_by_field_name("name")
                                .map(|n| self.node_text(&n).to_string())
                                .unwrap_or_default();
                            let ty = c
                                .child_by_field_name("type")
                                .map(|t| self.parse_type_annotation(&t))
                                .unwrap_or(Type::Unknown);
                            Some(Param {
                                name,
                                ty,
                                has_default: false,
                                kind: ParamKind::Positional,
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let ret = return_node
            .map(|r| self.parse_type_annotation(&r))
            .unwrap_or(Type::Unknown);

        Type::Callable {
            params,
            ret: Box::new(ret),
        }
    }

    /// Parse object type: { x: T, y: U }
    fn parse_object_type(&mut self, node: &Node) -> Type {
        let mut members: Vec<(String, Type)> = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "property_signature" {
                let name = child
                    .child_by_field_name("name")
                    .map(|n| self.node_text(&n).to_string())
                    .unwrap_or_default();
                let ty = child
                    .child_by_field_name("type")
                    .map(|t| self.parse_type_annotation(&t))
                    .unwrap_or(Type::Unknown);
                members.push((name, ty));
            } else if child.kind() == "method_signature" {
                let name = child
                    .child_by_field_name("name")
                    .map(|n| self.node_text(&n).to_string())
                    .unwrap_or_default();
                let ty = self.parse_function_type(&child);
                members.push((name, ty));
            }
        }

        Type::Protocol {
            name: "".to_string(),
            module: None,
            members,
        }
    }

    /// Parse conditional type: T extends U ? X : Y
    ///
    /// When T is known, we can evaluate the condition immediately.
    /// When T is a type variable or unknown, we store the full conditional
    /// as a union of both branches (conservative approximation).
    fn parse_conditional_type(&mut self, node: &Node) -> Type {
        let check = node.child_by_field_name("left");
        let extends = node.child_by_field_name("right");
        let consequence = node.child_by_field_name("consequence");
        let alternative = node.child_by_field_name("alternative");

        let check_ty = check
            .map(|c| self.parse_type_annotation(&c))
            .unwrap_or(Type::Unknown);
        let extends_ty = extends
            .map(|e| self.parse_type_annotation(&e))
            .unwrap_or(Type::Unknown);

        // If check type is a type variable or unknown, we cannot resolve the
        // conditional at this point. Return a union of both branches as a
        // conservative approximation.
        let check_is_unresolved = matches!(&check_ty, Type::TypeVar { .. } | Type::Unknown);

        if check_is_unresolved {
            let true_ty = consequence
                .map(|c| self.parse_type_annotation(&c))
                .unwrap_or(Type::Unknown);
            let false_ty = alternative
                .map(|a| self.parse_type_annotation(&a))
                .unwrap_or(Type::Unknown);
            Type::union(vec![true_ty, false_ty])
        } else if is_assignable_to(&check_ty, &extends_ty) {
            consequence
                .map(|c| self.parse_type_annotation(&c))
                .unwrap_or(Type::Unknown)
        } else {
            alternative
                .map(|a| self.parse_type_annotation(&a))
                .unwrap_or(Type::Unknown)
        }
    }

    /// Parse indexed access type: T[K]
    fn parse_indexed_access_type(&mut self, node: &Node) -> Type {
        let object = node.child_by_field_name("object");
        let index = node.child_by_field_name("index");

        let object_ty = object
            .map(|o| self.parse_type_annotation(&o))
            .unwrap_or(Type::Unknown);
        let index_ty = index
            .map(|i| self.parse_type_annotation(&i))
            .unwrap_or(Type::Unknown);

        // If index is a literal string, look up property
        if let Type::Literal(LiteralValue::Str(key)) = &index_ty {
            return self.get_property_type(&object_ty, key);
        }

        // Otherwise return union of all values (simplified)
        Type::Unknown
    }

    /// Parse mapped type: { [K in keyof T]: V }
    ///
    /// Mapped types iterate over keys from a source type and produce
    /// a new object type. When the source keys are known (e.g., a union
    /// of literal strings), we can expand the mapped type into a concrete
    /// Protocol. Otherwise, we represent it as a Dict.
    fn parse_mapped_type(&mut self, node: &Node) -> Type {
        // Look for the type parameter, constraint, and value type.
        // tree-sitter-typescript represents mapped types with children:
        //   { [ name "in" constraint ] : value_type }
        let mut key_name = String::new();
        let mut constraint_ty = Type::Unknown;
        let mut value_ty = Type::Unknown;

        let mut cursor = node.walk();
        let children: Vec<Node> = node.children(&mut cursor).collect();

        // Walk children to find the mapping clause and value type.
        // Pattern: "{" "[" identifier "in" type "]" ":" type "}"
        // Or the mapped_type_clause child contains (name, constraint).
        for child in &children {
            match child.kind() {
                "mapped_type_clause" => {
                    // The clause has name and constraint children
                    if let Some(name_node) = child.child_by_field_name("name") {
                        key_name = name_node
                            .utf8_text(self.source.as_bytes())
                            .unwrap_or("")
                            .to_string();
                    }
                    if let Some(type_node) = child.child_by_field_name("type") {
                        constraint_ty = self.parse_type_annotation(&type_node);
                    }
                }
                "type_annotation" | _
                    if child.kind().contains("type")
                        && child.kind() != "mapped_type_clause"
                        && !matches!(child.kind(), "{" | "}" | "[" | "]" | ":" | "in") =>
                {
                    // This is the value type
                    value_ty = self.parse_type_annotation(child);
                }
                _ => {}
            }
        }

        // If constraint is a union of literal strings, expand to Protocol
        if let Type::Union(members) = &constraint_ty {
            let all_literal_strings = members
                .iter()
                .all(|m| matches!(m, Type::Literal(LiteralValue::Str(_))));

            if all_literal_strings {
                let protocol_members: Vec<(String, Type)> = members
                    .iter()
                    .filter_map(|m| {
                        if let Type::Literal(LiteralValue::Str(s)) = m {
                            Some((s.clone(), value_ty.clone()))
                        } else {
                            None
                        }
                    })
                    .collect();

                return Type::Protocol {
                    name: String::new(),
                    module: None,
                    members: protocol_members,
                };
            }
        }

        // Fallback: represent as a dictionary from the key constraint to the value
        let _ = key_name; // Consumed for expansion above, not needed for Dict
        Type::dict(constraint_ty, value_ty)
    }

    /// Parse template literal type: `hello ${string}`
    ///
    /// Template literal types are string types built from static segments
    /// and interpolated type positions. When all interpolations resolve to
    /// concrete literal strings, we can produce a literal string type.
    /// Otherwise we return Type::Str.
    fn parse_template_literal_type(&mut self, node: &Node) -> Type {
        let mut all_literal = true;
        let mut result = String::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                // Static text segments
                "template_type_string" | "string_fragment" | "template_chars" => {
                    let text = child.utf8_text(self.source.as_bytes()).unwrap_or("");
                    result.push_str(text);
                }
                // Interpolation: ${Type}
                "template_type" => {
                    // The interpolated type is the first child that is a type node
                    if let Some(type_child) = child.child(0) {
                        let inner_ty = self.parse_type_annotation(&type_child);
                        match &inner_ty {
                            Type::Literal(LiteralValue::Str(s)) => {
                                result.push_str(s);
                            }
                            _ => {
                                all_literal = false;
                            }
                        }
                    } else {
                        all_literal = false;
                    }
                }
                // Skip delimiters
                "`" | "${" | "}" => {}
                _ => {
                    // Any child that is a type node inside an interpolation
                    let text = child.utf8_text(self.source.as_bytes()).unwrap_or("");
                    if !text.is_empty() && !matches!(child.kind(), "{" | "}" | "`" | "${") {
                        // Try to parse as type for interpolation
                        let inner_ty = self.parse_type_annotation(&child);
                        match &inner_ty {
                            Type::Literal(LiteralValue::Str(s)) => {
                                result.push_str(s);
                            }
                            _ => {
                                all_literal = false;
                            }
                        }
                    }
                }
            }
        }

        if all_literal && !result.is_empty() {
            Type::Literal(LiteralValue::Str(result))
        } else {
            Type::Str
        }
    }

    /// Unwrap Promise<T> to T
    fn unwrap_promise(&self, ty: Type) -> Type {
        match ty {
            Type::Instance {
                name, type_args, ..
            } if name == "Promise" => type_args.into_iter().next().unwrap_or(Type::Unknown),
            _ => ty,
        }
    }

    /// Make a type partial (all properties optional)
    fn make_partial(&self, ty: Type) -> Type {
        match ty {
            Type::Protocol {
                name,
                module,
                members,
            } => {
                let new_members: Vec<(String, Type)> = members
                    .into_iter()
                    .map(|(k, v)| (k, Type::optional(v)))
                    .collect();
                Type::Protocol {
                    name,
                    module,
                    members: new_members,
                }
            }
            other => Type::optional(other),
        }
    }

    /// Make a type required (all properties required)
    fn make_required(&self, ty: Type) -> Type {
        match ty {
            Type::Protocol {
                name,
                module,
                members,
            } => {
                let new_members: Vec<(String, Type)> = members
                    .into_iter()
                    .map(|(k, v)| {
                        let unwrapped = match v {
                            Type::Optional(inner) => (*inner).clone(),
                            other => other,
                        };
                        (k, unwrapped)
                    })
                    .collect();
                Type::Protocol {
                    name,
                    module,
                    members: new_members,
                }
            }
            Type::Optional(inner) => (*inner).clone(),
            other => other,
        }
    }

    /// Apply type guard narrowing
    pub fn apply_type_guard(&mut self, condition: &Node, is_true_branch: bool) {
        match condition.kind() {
            "binary_expression" => {
                let left = condition.child_by_field_name("left");
                let op = condition.child_by_field_name("operator");
                let right = condition.child_by_field_name("right");

                let op_text = op.map(|o| self.node_text(&o)).unwrap_or("");

                match op_text {
                    // typeof narrowing
                    "===" | "==" if is_true_branch => {
                        self.apply_typeof_guard(&left, &right);
                    }
                    "!==" | "!=" if !is_true_branch => {
                        self.apply_typeof_guard(&left, &right);
                    }
                    // instanceof narrowing
                    "instanceof" => {
                        if let Some(l) = left {
                            if l.kind() == "identifier" {
                                let var_name = self.node_text(&l);
                                if let Some(r) = right {
                                    let class_name = self.node_text(&r);
                                    if is_true_branch {
                                        let ty = Type::Instance {
                                            name: class_name.to_string(),
                                            module: None,
                                            type_args: vec![],
                                        };
                                        self.narrowed_types.insert(var_name.to_string(), ty);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            "unary_expression" => {
                let op = condition.child_by_field_name("operator");
                if op.map(|o| self.node_text(&o)) == Some("!") {
                    if let Some(arg) = condition.child_by_field_name("argument") {
                        // Invert the guard
                        self.apply_type_guard(&arg, !is_true_branch);
                    }
                }
            }
            _ => {}
        }
    }

    /// Apply typeof guard
    fn apply_typeof_guard(&mut self, typeof_side: &Option<Node>, literal_side: &Option<Node>) {
        // typeof x === "string"
        if let (Some(typeof_node), Some(literal)) = (typeof_side, literal_side) {
            if typeof_node.kind() == "unary_expression" {
                let op = typeof_node.child_by_field_name("operator");
                if op.map(|o| self.node_text(&o)) == Some("typeof") {
                    if let Some(arg) = typeof_node.child_by_field_name("argument") {
                        if arg.kind() == "identifier" {
                            let var_name = self.node_text(&arg);
                            let type_str = self
                                .node_text(literal)
                                .trim_matches(|c| c == '"' || c == '\'');

                            let narrowed = match type_str {
                                "string" => Type::Str,
                                "number" => Type::Float,
                                "boolean" => Type::Bool,
                                "undefined" => Type::None,
                                "object" => Type::Any,
                                "function" => Type::Any,
                                _ => return,
                            };

                            self.narrowed_types.insert(var_name.to_string(), narrowed);
                        }
                    }
                }
            }
        }
    }

    /// Check structural compatibility between a value and an interface
    pub fn check_structural_compatibility(&self, value_ty: &Type, interface_name: &str) -> bool {
        let interface = match self.context.interfaces.get(interface_name) {
            Some(i) => i,
            None => return false,
        };

        // Get value members
        let value_members: HashMap<String, Type> = match value_ty {
            Type::Protocol { members, .. } => members.iter().cloned().collect(),
            Type::Instance { name, .. } => {
                if let Some(class) = self.context.classes.get(name) {
                    class
                        .properties
                        .iter()
                        .map(|(k, v)| (k.clone(), v.ty.clone()))
                        .chain(class.methods.iter().map(|(k, v)| (k.clone(), v.clone())))
                        .collect()
                } else {
                    return false;
                }
            }
            _ => return false,
        };

        // Check required properties
        for (prop_name, prop_ty) in &interface.properties {
            match value_members.get(prop_name) {
                Some(value_prop_ty) => {
                    if !is_assignable_to(value_prop_ty, prop_ty) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Check methods
        for (method_name, method_ty) in &interface.methods {
            match value_members.get(method_name) {
                Some(value_method_ty) => {
                    if !is_assignable_to(value_method_ty, method_ty) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        true
    }

    /// Bind a variable type
    pub fn bind_variable(&mut self, name: String, ty: Type) {
        self.context.variables.insert(name, ty);
    }

    /// Enter a new scope
    pub fn enter_scope(&mut self) {
        self.scope_depth += 1;
    }

    /// Exit current scope
    pub fn exit_scope(&mut self) {
        self.scope_depth = self.scope_depth.saturating_sub(1);
        self.narrowed_types.clear();
    }
}

/// Check if a type is definitely falsy
fn is_falsy(ty: &Type) -> bool {
    match ty {
        Type::None | Type::Never => true,
        Type::Literal(LiteralValue::Bool(false)) => true,
        Type::Literal(LiteralValue::Int(0)) => true,
        Type::Literal(LiteralValue::Str(s)) if s.is_empty() => true,
        _ => false,
    }
}

/// Check if a type is definitely truthy
fn is_truthy(ty: &Type) -> bool {
    match ty {
        Type::Literal(LiteralValue::Bool(true)) => true,
        Type::Literal(LiteralValue::Int(n)) if *n != 0 => true,
        Type::Literal(LiteralValue::Str(s)) if !s.is_empty() => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_inferencer() -> TsTypeInferencer<'static> {
        TsTypeInferencer::new("")
    }

    #[test]
    fn test_fresh_type_var() {
        let mut inf = make_inferencer();
        let tv1 = inf.fresh_type_var("T");
        let tv2 = inf.fresh_type_var("U");

        match (&tv1, &tv2) {
            (Type::TypeVar { id: id1, .. }, Type::TypeVar { id: id2, .. }) => {
                assert_ne!(id1, id2);
            }
            _ => panic!("Expected TypeVar"),
        }
    }

    #[test]
    fn test_type_context_interface() {
        let mut ctx = TsTypeContext::new();

        let mut iface = TsInterface::new("Readable".to_string());
        iface.methods.insert(
            "read".to_string(),
            Type::Callable {
                params: vec![],
                ret: Box::new(Type::Str),
            },
        );
        ctx.register_interface(iface);

        let resolved = ctx.resolve_type("Readable");
        assert!(matches!(resolved, Some(Type::Protocol { .. })));
    }

    #[test]
    fn test_structural_compatibility() {
        let mut inf = make_inferencer();

        // Register interface
        let mut iface = TsInterface::new("Named".to_string());
        iface.properties.insert("name".to_string(), Type::Str);
        inf.context_mut().register_interface(iface);

        // Check object literal
        let obj = Type::Protocol {
            name: "".to_string(),
            module: None,
            members: vec![("name".to_string(), Type::Str)],
        };

        assert!(inf.check_structural_compatibility(&obj, "Named"));

        // Missing property
        let obj2 = Type::Protocol {
            name: "".to_string(),
            module: None,
            members: vec![],
        };
        assert!(!inf.check_structural_compatibility(&obj2, "Named"));
    }

    #[test]
    fn test_union_type_handling() {
        let union = Type::Union(vec![Type::Str, Type::Int]);

        // Should be assignable to wider union
        let _wider = Type::Union(vec![Type::Str, Type::Int, Type::Bool]);
        assert!(is_assignable_to(&Type::Str, &union));
        assert!(!is_assignable_to(&Type::Bool, &union));
    }

    #[test]
    fn test_partial_type() {
        let inf = make_inferencer();

        let proto = Type::Protocol {
            name: "Person".to_string(),
            module: None,
            members: vec![
                ("name".to_string(), Type::Str),
                ("age".to_string(), Type::Int),
            ],
        };

        let partial = inf.make_partial(proto);

        match partial {
            Type::Protocol { members, .. } => {
                assert!(members
                    .iter()
                    .all(|(_, ty)| matches!(ty, Type::Optional(_))));
            }
            _ => panic!("Expected Protocol"),
        }
    }

    #[test]
    fn test_bind_and_lookup() {
        let mut inf = make_inferencer();
        inf.bind_variable("x".to_string(), Type::Int);

        let result = inf.context.variables.get("x");
        assert_eq!(result, Some(&Type::Int));
    }
}
