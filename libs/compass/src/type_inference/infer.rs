//! Type inference engine for Python

use std::collections::HashMap;
use tree_sitter::Node;

use super::annotation::parse_type_annotation;
use super::builtins::add_builtins;
use super::class_info::ClassInfo;
use super::frameworks::FrameworkRegistry;
use super::imports::{parse_import, ImportResolver};
use super::stubs::StubLoader;
use super::ty::{Param, ParamKind, Type, TypeVarId, Variance};
use super::type_env::TypeEnv;

/// Type inferencer for Python code
pub struct TypeInferencer<'a> {
    /// Source code
    source: &'a str,
    /// Type environment
    env: TypeEnv,
    /// Class registry (class name -> class info)
    classes: HashMap<String, ClassInfo>,
    /// TypeVar registry (name -> type)
    type_vars: HashMap<String, Type>,
    /// Counter for generating fresh type variables
    next_type_var: usize,
    /// Type overrides from narrowing (checked before env)
    type_overrides: Option<HashMap<String, Type>>,
    /// Stub loader for builtin/typing/collections stubs
    #[allow(dead_code)]
    stubs: StubLoader,
    /// Import resolver for module resolution
    resolver: ImportResolver,
    /// Overloaded function signatures (name -> list of Callable signatures)
    overload_signatures: HashMap<String, Vec<Type>>,
    /// Current class name (for Self type resolution)
    current_class: Option<String>,
    /// Framework type providers
    framework_registry: FrameworkRegistry,
}

impl<'a> TypeInferencer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut env = TypeEnv::new();
        // Add builtins
        add_builtins(&mut env);

        // Initialize stubs and resolver
        let mut stubs = StubLoader::new();
        stubs.load_builtins();

        let mut resolver = ImportResolver::new();
        for (path, info) in stubs.modules() {
            resolver.register_module(path, info.clone());
        }

        Self {
            source,
            env,
            classes: HashMap::new(),
            type_vars: HashMap::new(),
            next_type_var: 0,
            type_overrides: None,
            stubs,
            resolver,
            overload_signatures: HashMap::new(),
            current_class: None,
            framework_registry: FrameworkRegistry::new(),
        }
    }

    /// Get a mutable reference to the framework registry for configuration.
    pub fn framework_registry_mut(&mut self) -> &mut FrameworkRegistry {
        &mut self.framework_registry
    }

    /// Add a type binding to the environment (useful for testing).
    pub fn bind_type(&mut self, name: String, ty: Type) {
        self.env.bind(name, ty);
    }

    /// Set type overrides from narrowing (used for control flow analysis)
    pub fn set_type_overrides(&mut self, overrides: HashMap<String, Type>) {
        self.type_overrides = Some(overrides);
    }

    /// Clear type overrides
    pub fn clear_type_overrides(&mut self) {
        self.type_overrides = None;
    }

    /// Register a TypeVar definition
    pub fn register_type_var(&mut self, name: &str, bound: Option<Type>, constraints: Vec<Type>) {
        self.register_type_var_with_variance(name, bound, constraints, Variance::Invariant);
    }

    /// Register a TypeVar definition with specific variance
    pub fn register_type_var_with_variance(
        &mut self,
        name: &str,
        bound: Option<Type>,
        constraints: Vec<Type>,
        variance: Variance,
    ) {
        let id = TypeVarId(self.next_type_var);
        self.next_type_var += 1;
        let tv = Type::TypeVar {
            id,
            name: name.to_string(),
            bound: bound.map(Box::new),
            constraints,
            variance,
        };
        self.type_vars.insert(name.to_string(), tv.clone());
        self.env.bind(name.to_string(), tv);
    }

    /// Look up a TypeVar by name
    pub fn get_type_var(&self, name: &str) -> Option<&Type> {
        self.type_vars.get(name)
    }

    /// Instantiate a generic type with concrete type arguments
    pub fn instantiate_generic(&self, generic_type: &Type, type_args: &[Type]) -> Type {
        let type_var_ids = generic_type.type_vars();
        if type_var_ids.len() != type_args.len() {
            return Type::Error; // Wrong number of type args
        }

        let substitutions: HashMap<TypeVarId, Type> = type_var_ids
            .into_iter()
            .zip(type_args.iter().cloned())
            .collect();

        generic_type.substitute(&substitutions)
    }

    /// Get class info by name
    pub fn get_class(&self, name: &str) -> Option<&ClassInfo> {
        self.classes.get(name)
    }

    /// Get attribute type with inheritance support
    /// Walks up the inheritance chain to find the attribute
    pub fn get_attribute_recursive(&self, class_name: &str, attr_name: &str) -> Option<Type> {
        if let Some(class_info) = self.classes.get(class_name) {
            // First check the current class
            if let Some(ty) = class_info.get_attribute(attr_name) {
                return Some(ty.clone());
            }
            // Then check base classes (in order)
            for base_name in &class_info.bases {
                if let Some(ty) = self.get_attribute_recursive(base_name, attr_name) {
                    return Some(ty);
                }
            }
        }
        None
    }

    /// Get class-level attribute (class vars, methods) with inheritance support
    fn get_class_attribute_recursive(&self, class_name: &str, attr_name: &str) -> Option<Type> {
        if let Some(class_info) = self.classes.get(class_name) {
            // First check class variables
            if let Some(ty) = class_info.class_vars.get(attr_name) {
                return Some(ty.clone());
            }
            // Then check methods
            if let Some(ty) = class_info.methods.get(attr_name) {
                return Some(ty.clone());
            }
            // Then check base classes
            for base_name in &class_info.bases {
                if let Some(ty) = self.get_class_attribute_recursive(base_name, attr_name) {
                    return Some(ty);
                }
            }
        }
        None
    }

    /// Check if a class is a subclass of another (including self)
    pub fn is_subclass(&self, child: &str, parent: &str) -> bool {
        if child == parent {
            return true;
        }
        if let Some(class_info) = self.classes.get(child) {
            for base_name in &class_info.bases {
                if self.is_subclass(base_name, parent) {
                    return true;
                }
            }
        }
        false
    }

    /// Generate a fresh type variable (invariant by default)
    #[allow(dead_code)]
    fn fresh_type_var(&mut self, name: &str) -> Type {
        let id = TypeVarId(self.next_type_var);
        self.next_type_var += 1;
        Type::TypeVar {
            id,
            name: name.to_string(),
            bound: None,
            constraints: vec![],
            variance: Variance::Invariant,
        }
    }

    /// Get the text of a node
    fn node_text(&self, node: &Node) -> &str {
        node.utf8_text(self.source.as_bytes()).unwrap_or("")
    }

    /// Infer the type of an expression
    ///
    /// Returns `Type::Error` for error nodes (from parser recovery) to prevent
    /// cascading errors in downstream type checking.
    pub fn infer_expr(&mut self, node: &Node) -> Type {
        // Error recovery: return Error type for ERROR nodes from tree-sitter
        // This prevents cascading type errors when parsing fails
        if node.is_error() || node.is_missing() {
            return Type::Error;
        }

        match node.kind() {
            // Explicit ERROR node kind (fallback check)
            "ERROR" => Type::Error,

            // Literals
            "integer" => Type::Int,
            "float" => Type::Float,
            "string" => Type::Str,
            "true" | "false" => Type::Bool,
            "none" => Type::None,
            "ellipsis" => Type::Any, // ... is typically used as placeholder

            // Identifier lookup
            "identifier" => {
                let name = self.node_text(node);
                // Check type overrides first (from narrowing)
                if let Some(ref overrides) = self.type_overrides {
                    if let Some(ty) = overrides.get(name) {
                        return ty.clone();
                    }
                }
                // Use Error type for unresolved identifiers in error regions
                // This helps prevent cascading errors
                self.env.lookup(name).cloned().unwrap_or(Type::Unknown)
            }

            // Binary operators
            "binary_operator" => self.infer_binary_op(node),

            // Unary operators
            "unary_operator" => self.infer_unary_op(node),

            // Comparison operators
            "comparison_operator" => Type::Bool,
            "boolean_operator" => Type::Bool,
            "not_operator" => Type::Bool,

            // Container literals
            "list" => self.infer_list_literal(node),
            "dictionary" => self.infer_dict_literal(node),
            "set" => self.infer_set_literal(node),
            "tuple" => self.infer_tuple_literal(node),

            // List comprehension
            "list_comprehension" => self.infer_list_comprehension(node),

            // Call expression
            "call" => self.infer_call(node),

            // Attribute access
            "attribute" => self.infer_attribute(node),

            // Subscript
            "subscript" => self.infer_subscript(node),

            // Conditional expression (ternary)
            "conditional_expression" => self.infer_conditional(node),

            // Lambda
            "lambda" => self.infer_lambda(node),

            // Await expression
            "await" => {
                if let Some(arg) = node.child(1) {
                    // Unwrap Awaitable[T] -> T
                    let inner = self.infer_expr(&arg);
                    // For now, just return the inner type
                    inner
                } else {
                    Type::Unknown
                }
            }

            // Parenthesized expression
            "parenthesized_expression" => {
                if let Some(inner) = node.child(1) {
                    self.infer_expr(&inner)
                } else {
                    Type::Unknown
                }
            }

            _ => Type::Unknown,
        }
    }

    /// Infer binary operator result type
    fn infer_binary_op(&mut self, node: &Node) -> Type {
        let left = node.child_by_field_name("left");
        let right = node.child_by_field_name("right");
        let op = node.child_by_field_name("operator");

        let (left_ty, right_ty) = match (left, right) {
            (Some(l), Some(r)) => (self.infer_expr(&l), self.infer_expr(&r)),
            _ => return Type::Unknown,
        };

        let op_text = op.map(|o| self.node_text(&o)).unwrap_or("");

        match op_text {
            // Arithmetic operators
            "+" => match (&left_ty, &right_ty) {
                (Type::Str, Type::Str) => Type::Str,
                (Type::List(a), Type::List(b)) if a == b => Type::list((**a).clone()),
                (Type::Int, Type::Int) => Type::Int,
                (Type::Float, _) | (_, Type::Float) => Type::Float,
                _ => Type::Unknown,
            },
            "-" | "*" => match (&left_ty, &right_ty) {
                (Type::Str, Type::Int) if op_text == "*" => Type::Str, // "a" * 3
                (Type::Int, Type::Str) if op_text == "*" => Type::Str, // 3 * "a"
                (Type::Int, Type::Int) => Type::Int,
                (Type::Float, _) | (_, Type::Float) => Type::Float,
                _ => Type::Unknown,
            },
            "/" => Type::Float, // Python 3 true division
            "//" => Type::Int,  // Floor division
            "%" => match (&left_ty, &right_ty) {
                (Type::Str, _) => Type::Str, // String formatting
                (Type::Int, Type::Int) => Type::Int,
                _ => Type::Unknown,
            },
            "**" => match (&left_ty, &right_ty) {
                (Type::Int, Type::Int) => Type::Int,
                _ => Type::Float,
            },

            // Bitwise operators
            "&" | "|" | "^" | "<<" | ">>" => Type::Int,

            // Membership/identity
            "in" | "not in" | "is" | "is not" => Type::Bool,

            _ => Type::Unknown,
        }
    }

    /// Infer unary operator result type
    fn infer_unary_op(&mut self, node: &Node) -> Type {
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();

        let op = children.first().map(|n| self.node_text(n)).unwrap_or("");
        let operand = children.get(1);

        match op {
            "-" | "+" => {
                if let Some(arg) = operand {
                    let arg_ty = self.infer_expr(arg);
                    match arg_ty {
                        Type::Int => Type::Int,
                        Type::Float => Type::Float,
                        _ => Type::Unknown,
                    }
                } else {
                    Type::Unknown
                }
            }
            "~" => Type::Int, // Bitwise NOT
            "not" => Type::Bool,
            _ => Type::Unknown,
        }
    }

    /// Infer list literal type
    fn infer_list_literal(&mut self, node: &Node) -> Type {
        let mut cursor = node.walk();
        let mut element_types = Vec::new();

        for child in node.children(&mut cursor) {
            if child.kind() != "[" && child.kind() != "]" && child.kind() != "," {
                element_types.push(self.infer_expr(&child));
            }
        }

        if element_types.is_empty() {
            Type::list(Type::Unknown)
        } else {
            // Use the first element's type (simplified)
            // A full implementation would compute LUB (least upper bound)
            Type::list(element_types[0].clone())
        }
    }

    /// Infer dict literal type
    fn infer_dict_literal(&mut self, node: &Node) -> Type {
        let mut cursor = node.walk();
        let mut key_type = Type::Unknown;
        let mut value_type = Type::Unknown;

        for child in node.children(&mut cursor) {
            if child.kind() == "pair" {
                if let Some(key) = child.child_by_field_name("key") {
                    key_type = self.infer_expr(&key);
                }
                if let Some(value) = child.child_by_field_name("value") {
                    value_type = self.infer_expr(&value);
                }
                break; // Just use first pair for type
            }
        }

        Type::dict(key_type, value_type)
    }

    /// Infer set literal type
    fn infer_set_literal(&mut self, node: &Node) -> Type {
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() != "{" && child.kind() != "}" && child.kind() != "," {
                let elem_ty = self.infer_expr(&child);
                return Type::Set(Box::new(elem_ty));
            }
        }

        Type::Set(Box::new(Type::Unknown))
    }

    /// Infer tuple literal type
    fn infer_tuple_literal(&mut self, node: &Node) -> Type {
        let mut cursor = node.walk();
        let mut element_types = Vec::new();

        for child in node.children(&mut cursor) {
            if child.kind() != "(" && child.kind() != ")" && child.kind() != "," {
                element_types.push(self.infer_expr(&child));
            }
        }

        Type::Tuple(element_types)
    }

    /// Infer list comprehension type
    fn infer_list_comprehension(&mut self, node: &Node) -> Type {
        // [expr for x in iter] -> list[type of expr]
        if let Some(body) = node.child(1) {
            // This is simplified - should handle the iteration binding
            let elem_ty = self.infer_expr(&body);
            Type::list(elem_ty)
        } else {
            Type::list(Type::Unknown)
        }
    }

    /// Infer function call result type with generic type inference
    fn infer_call(&mut self, node: &Node) -> Type {
        let func = match node.child_by_field_name("function") {
            Some(f) => f,
            None => return Type::Unknown,
        };

        // Check if this is a method call on an instance (e.g., user.save())
        if func.kind() == "attribute" {
            let object = func.child_by_field_name("object");
            let attribute = func.child_by_field_name("attribute");

            if let (Some(obj), Some(attr)) = (object, attribute) {
                let object_type = self.infer_expr(&obj);
                let method_name = self.node_text(&attr);

                // Try framework-specific method signature
                if let Some(method_sig) = self
                    .framework_registry
                    .get_method_signature(&object_type, method_name)
                {
                    return method_sig.return_type;
                }
            }
        }

        let func_ty = self.infer_expr(&func);

        match &func_ty {
            Type::Callable { params, ret } => {
                // Collect argument types
                let arg_types = self.collect_call_arguments(node);

                // Check if return type has type variables
                let type_vars = ret.type_vars();
                if type_vars.is_empty() {
                    // No generics, just return the declared return type
                    return (**ret).clone();
                }

                // Unify parameter types with argument types to infer TypeVars
                let mut substitutions = HashMap::new();
                for (param, arg_ty) in params.iter().zip(arg_types.iter()) {
                    param.ty.unify(arg_ty, &mut substitutions);
                }

                // Apply substitutions to the return type
                ret.substitute(&substitutions)
            }
            Type::Instance { name, .. } => {
                // Constructor call returns instance
                Type::Instance {
                    name: name.clone(),
                    module: None,
                    type_args: vec![],
                }
            }
            Type::ClassType { name, .. } => {
                // type[T]() returns T
                Type::Instance {
                    name: name.clone(),
                    module: None,
                    type_args: vec![],
                }
            }
            _ => Type::Unknown,
        }
    }

    /// Collect argument types from a call expression
    fn collect_call_arguments(&mut self, node: &Node) -> Vec<Type> {
        let mut arg_types = Vec::new();

        if let Some(args_node) = node.child_by_field_name("arguments") {
            let mut cursor = args_node.walk();
            for child in args_node.children(&mut cursor) {
                match child.kind() {
                    "(" | ")" | "," => continue,
                    "keyword_argument" => {
                        // Skip keyword for now, just get the value
                        if let Some(value) = child.child_by_field_name("value") {
                            arg_types.push(self.infer_expr(&value));
                        }
                    }
                    _ => {
                        arg_types.push(self.infer_expr(&child));
                    }
                }
            }
        }

        arg_types
    }

    /// Infer attribute access type
    fn infer_attribute(&mut self, node: &Node) -> Type {
        let object = match node.child_by_field_name("object") {
            Some(o) => o,
            None => return Type::Unknown,
        };
        let attribute = match node.child_by_field_name("attribute") {
            Some(a) => a,
            None => return Type::Unknown,
        };

        let object_type = self.infer_expr(&object);
        let attr_name = self.node_text(&attribute);

        match &object_type {
            Type::Instance { name, .. } => {
                // 1. Try standard attribute resolution with inheritance
                if let Some(ty) = self.get_attribute_recursive(name, attr_name) {
                    return ty;
                }

                // 2. Try framework-specific attribute resolution
                if let Some(ty) = self
                    .framework_registry
                    .get_attribute_type(&object_type, attr_name)
                {
                    return ty;
                }

                Type::Unknown
            }
            Type::ClassType { name, .. } => {
                // Class attribute access (static methods, class vars) with inheritance
                self.get_class_attribute_recursive(name, attr_name)
                    .unwrap_or(Type::Unknown)
            }
            Type::Optional(inner) => {
                // For Optional[T].attr, return the attribute type from T with inheritance
                if let Type::Instance { name, .. } = inner.as_ref() {
                    if let Some(ty) = self.get_attribute_recursive(name, attr_name) {
                        return ty;
                    }

                    // Try framework-specific attribute resolution
                    if let Some(ty) = self
                        .framework_registry
                        .get_attribute_type(inner.as_ref(), attr_name)
                    {
                        return ty;
                    }
                }
                Type::Unknown
            }
            _ => Type::Unknown,
        }
    }

    /// Infer subscript type
    fn infer_subscript(&mut self, node: &Node) -> Type {
        let value = match node.child_by_field_name("value") {
            Some(v) => v,
            None => return Type::Unknown,
        };

        let value_ty = self.infer_expr(&value);

        match &value_ty {
            Type::List(elem) => (**elem).clone(),
            Type::Dict(_, val) => (**val).clone(),
            Type::Tuple(elems) => {
                // For tuple, try to get specific index
                if let Some(subscript) = node.child_by_field_name("subscript") {
                    if subscript.kind() == "integer" {
                        if let Ok(idx) = self.node_text(&subscript).parse::<usize>() {
                            if let Some(ty) = elems.get(idx) {
                                return ty.clone();
                            }
                        }
                    }
                }
                // Unknown index, return union of all element types
                Type::union(elems.clone())
            }
            Type::Str => Type::Str, // str[n] -> str
            _ => Type::Unknown,
        }
    }

    /// Infer conditional expression type
    fn infer_conditional(&mut self, node: &Node) -> Type {
        // Python: true_val if condition else false_val
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();

        // [true_val, "if", condition, "else", false_val]
        if children.len() >= 5 {
            let true_ty = self.infer_expr(&children[0]);
            let false_ty = self.infer_expr(&children[4]);

            if true_ty == false_ty {
                true_ty
            } else {
                Type::union(vec![true_ty, false_ty])
            }
        } else {
            Type::Unknown
        }
    }

    /// Infer lambda type
    fn infer_lambda(&mut self, node: &Node) -> Type {
        // lambda params: body
        let mut params = Vec::new();

        if let Some(params_node) = node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                if child.kind() == "identifier" {
                    params.push(Param {
                        name: self.node_text(&child).to_string(),
                        ty: Type::Unknown,
                        has_default: false,
                        kind: ParamKind::Positional,
                    });
                }
            }
        }

        let ret = if let Some(body) = node.child_by_field_name("body") {
            self.infer_expr(&body)
        } else {
            Type::Unknown
        };

        Type::Callable {
            params,
            ret: Box::new(ret),
        }
    }

    /// Check if a function has the @overload decorator
    fn has_overload_decorator(&self, node: &Node) -> bool {
        // Look for decorator node
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "decorator" {
                let decorator_text = self.node_text(&child);
                if decorator_text.contains("overload") {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a class has the @dataclass decorator
    /// Looks in both the node itself and its parent (for decorated_definition wrapper)
    fn has_dataclass_decorator(&self, node: &Node) -> bool {
        // First check the node's children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "decorator" {
                let decorator_text = self.node_text(&child);
                if decorator_text.contains("dataclass") {
                    return true;
                }
            }
        }

        // Also check the parent if it's a decorated_definition
        if let Some(parent) = node.parent() {
            if parent.kind() == "decorated_definition" {
                let mut cursor = parent.walk();
                for child in parent.children(&mut cursor) {
                    if child.kind() == "decorator" {
                        let decorator_text = self.node_text(&child);
                        if decorator_text.contains("dataclass") {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Analyze a dataclass and generate __init__ signature from fields
    pub fn analyze_dataclass(&mut self, node: &Node) -> Type {
        let class_name = node
            .child_by_field_name("name")
            .map(|n| self.node_text(&n).to_string())
            .unwrap_or_default();

        let mut fields: Vec<(String, Type, bool)> = Vec::new(); // (name, type, has_default)

        // Find the class body
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                match child.kind() {
                    // Annotated assignment: field_name: Type or field_name: Type = default
                    "expression_statement" => {
                        if let Some(assignment) = child.child(0) {
                            if assignment.kind() == "assignment" {
                                if let Some((name, ty, has_default)) =
                                    self.parse_dataclass_field(&assignment)
                                {
                                    fields.push((name, ty, has_default));
                                }
                            }
                        }
                    }
                    // Type annotated field: field_name: Type
                    "type" => {
                        if let Some(name) = child.child_by_field_name("name") {
                            let field_name = self.node_text(&name).to_string();
                            let field_type = child
                                .child_by_field_name("type")
                                .map(|t| parse_type_annotation(self.source, &t))
                                .unwrap_or(Type::Any);
                            let has_default = child.child_by_field_name("value").is_some();
                            fields.push((field_name, field_type, has_default));
                        }
                    }
                    _ => {}
                }
            }
        }

        // Generate __init__ method from fields
        let init_params: Vec<Param> = fields
            .iter()
            .map(|(name, ty, has_default)| Param {
                name: name.clone(),
                ty: ty.clone(),
                has_default: *has_default,
                kind: ParamKind::Positional,
            })
            .collect();

        let init_type = Type::Callable {
            params: init_params,
            ret: Box::new(Type::None),
        };

        // Register the class with __init__
        let mut class_info = ClassInfo::new(class_name.clone());
        class_info.methods.insert("__init__".to_string(), init_type);

        // Add field types as attributes
        for (name, ty, _) in &fields {
            class_info.attributes.insert(name.clone(), ty.clone());
        }

        self.classes.insert(class_name.clone(), class_info);

        Type::ClassType {
            name: class_name,
            module: None,
        }
    }

    /// Parse a dataclass field from an assignment node
    fn parse_dataclass_field(&self, node: &Node) -> Option<(String, Type, bool)> {
        // Look for type-annotated assignment: name: Type = value or name: Type
        let left = node.child_by_field_name("left")?;

        if left.kind() == "identifier" {
            let name = self.node_text(&left).to_string();

            // Check for type annotation
            let annotation = node.child_by_field_name("type");
            let ty = annotation
                .map(|a| parse_type_annotation(self.source, &a))
                .unwrap_or(Type::Any);

            let has_default = node.child_by_field_name("right").is_some();

            return Some((name, ty, has_default));
        }

        None
    }

    /// Check if this is a NamedTuple class definition
    fn is_namedtuple_class(&self, node: &Node) -> bool {
        // Check for class Foo(NamedTuple):
        // tree-sitter-python uses argument_list for base classes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "argument_list" {
                let bases_text = self.node_text(&child);
                if bases_text.contains("NamedTuple") {
                    return true;
                }
            }
        }
        false
    }

    /// Analyze a NamedTuple class and generate proper types
    pub fn analyze_namedtuple(&mut self, node: &Node) -> Type {
        let class_name = node
            .child_by_field_name("name")
            .map(|n| self.node_text(&n).to_string())
            .unwrap_or_default();

        let mut field_types: Vec<Type> = Vec::new();
        let mut fields: Vec<(String, Type)> = Vec::new();

        // Find the class body and extract field annotations
        // tree-sitter parses `x: int` as an assignment node with a type child
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "expression_statement" {
                    if let Some(assignment) = child.child(0) {
                        if assignment.kind() == "assignment" {
                            // Get the field name from the left (identifier)
                            if let Some(name_node) = assignment.child(0) {
                                if name_node.kind() == "identifier" {
                                    let field_name = self.node_text(&name_node).to_string();

                                    // Get the type from the type annotation
                                    let mut type_cursor = assignment.walk();
                                    let mut field_type = Type::Any;
                                    for assign_child in assignment.children(&mut type_cursor) {
                                        if assign_child.kind() == "type" {
                                            field_type =
                                                parse_type_annotation(self.source, &assign_child);
                                            break;
                                        }
                                    }

                                    fields.push((field_name, field_type.clone()));
                                    field_types.push(field_type);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Create a tuple type for the NamedTuple (unused for now but may be useful)
        let _tuple_type = Type::Tuple(field_types);

        // Register class with field attributes
        let mut class_info = ClassInfo::new(class_name.clone());
        for (name, ty) in &fields {
            class_info.attributes.insert(name.clone(), ty.clone());
        }
        self.classes.insert(class_name.clone(), class_info);

        // Bind the class name to a ClassType
        let class_type = Type::ClassType {
            name: class_name.clone(),
            module: None,
        };
        self.env.bind(class_name, class_type.clone());

        class_type
    }

    /// Check if a class or method has @property decorator
    /// Looks in both the node itself and its parent (for decorated_definition wrapper)
    pub fn has_property_decorator(&self, node: &Node) -> bool {
        // First check the node's children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "decorator" {
                let decorator_text = self.node_text(&child);
                if decorator_text.contains("property") {
                    return true;
                }
            }
        }

        // Also check the parent if it's a decorated_definition
        if let Some(parent) = node.parent() {
            if parent.kind() == "decorated_definition" {
                let mut cursor = parent.walk();
                for child in parent.children(&mut cursor) {
                    if child.kind() == "decorator" {
                        let decorator_text = self.node_text(&child);
                        if decorator_text.contains("property") {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Get property type (return type of getter)
    pub fn get_property_type(&mut self, node: &Node) -> Type {
        // Property type is the return type of the getter method
        node.child_by_field_name("return_type")
            .map(|n| parse_type_annotation(self.source, &n))
            .unwrap_or(Type::Unknown)
    }

    /// Analyze a function definition and add it to the environment
    pub fn analyze_function(&mut self, node: &Node) -> Type {
        let name = node
            .child_by_field_name("name")
            .map(|n| self.node_text(&n).to_string())
            .unwrap_or_default();

        let mut params = Vec::new();
        let mut return_type = Type::Unknown;

        // Parse parameters
        if let Some(params_node) = node.child_by_field_name("parameters") {
            params = self.parse_parameters(&params_node);
        }

        // Parse return type annotation
        if let Some(return_node) = node.child_by_field_name("return_type") {
            return_type = parse_type_annotation(self.source, &return_node);
        }

        // Resolve Self type if we're in a class context
        if let Some(ref class_name) = self.current_class {
            return_type = self.resolve_self_type(return_type, class_name);
        }

        let func_type = Type::Callable {
            params,
            ret: Box::new(return_type),
        };

        // Check for @overload decorator
        if self.has_overload_decorator(node) {
            // This is an overload signature, collect it
            self.overload_signatures
                .entry(name.clone())
                .or_default()
                .push(func_type.clone());
            // Don't bind to env yet, wait for the implementation
            return func_type;
        }

        // Check if we have collected overload signatures for this function
        if let Some(signatures) = self.overload_signatures.remove(&name) {
            // Create an Overloaded type with all signatures
            let mut all_signatures = signatures;
            all_signatures.push(func_type.clone()); // Add the implementation signature
            let overloaded_type = Type::Overloaded {
                signatures: all_signatures,
            };
            self.env.bind(name, overloaded_type.clone());
            return overloaded_type;
        }

        self.env.bind(name, func_type.clone());
        func_type
    }

    /// Resolve Self type to the actual class type
    fn resolve_self_type(&self, ty: Type, class_name: &str) -> Type {
        match ty {
            Type::SelfType { .. } => Type::Instance {
                name: class_name.to_string(),
                module: None,
                type_args: vec![],
            },
            Type::Optional(inner) => {
                Type::Optional(Box::new(self.resolve_self_type(*inner, class_name)))
            }
            Type::Union(types) => Type::union(
                types
                    .into_iter()
                    .map(|t| self.resolve_self_type(t, class_name))
                    .collect(),
            ),
            Type::List(elem) => Type::List(Box::new(self.resolve_self_type(*elem, class_name))),
            other => other,
        }
    }

    /// Analyze a class definition and add it to the registry
    pub fn analyze_class(&mut self, node: &Node) -> ClassInfo {
        // Check for special class types first
        if self.has_dataclass_decorator(node) {
            self.analyze_dataclass(node);
            // Get the class name and return the info
            let name = node
                .child_by_field_name("name")
                .map(|n| self.node_text(&n).to_string())
                .unwrap_or_default();
            return self.classes.get(&name).cloned().unwrap_or_default();
        }

        if self.is_namedtuple_class(node) {
            self.analyze_namedtuple(node);
            let name = node
                .child_by_field_name("name")
                .map(|n| self.node_text(&n).to_string())
                .unwrap_or_default();
            return self.classes.get(&name).cloned().unwrap_or_default();
        }

        let name = node
            .child_by_field_name("name")
            .map(|n| self.node_text(&n).to_string())
            .unwrap_or_default();

        // Set current class for Self type resolution
        let prev_class = self.current_class.take();
        self.current_class = Some(name.clone());

        let mut class_info = ClassInfo::new(name.clone());

        // Parse base classes
        if let Some(bases) = node.child_by_field_name("superclasses") {
            let mut cursor = bases.walk();
            for child in bases.children(&mut cursor) {
                if child.kind() == "identifier" || child.kind() == "attribute" {
                    class_info.bases.push(self.node_text(&child).to_string());
                }
            }
        }

        // Parse class body
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                match child.kind() {
                    "function_definition" | "async_function_definition" => {
                        self.parse_class_method(&child, &mut class_info);
                    }
                    "decorated_definition" => {
                        // Handle decorated methods (e.g., @property, @staticmethod)
                        let mut inner_cursor = child.walk();
                        for inner in child.children(&mut inner_cursor) {
                            if inner.kind() == "function_definition"
                                || inner.kind() == "async_function_definition"
                            {
                                self.parse_class_method(&inner, &mut class_info);
                            }
                        }
                    }
                    "expression_statement" => {
                        self.parse_class_attribute(&child, &mut class_info);
                    }
                    _ => {}
                }
            }
        }

        // Restore previous class context
        self.current_class = prev_class;

        // Add class type to environment
        let class_type = Type::ClassType {
            name: name.clone(),
            module: None,
        };
        self.env.bind(name.clone(), class_type);

        // Store class info
        self.classes.insert(name, class_info.clone());

        class_info
    }

    /// Parse a method in a class definition
    fn parse_class_method(&mut self, node: &Node, class_info: &mut ClassInfo) {
        let method_name = node
            .child_by_field_name("name")
            .map(|n| self.node_text(&n).to_string())
            .unwrap_or_default();

        // Check for @property decorator - properties become attributes, not methods
        if self.has_property_decorator(node) {
            let property_type = self.get_property_type(node);
            class_info.attributes.insert(method_name, property_type);
            return;
        }

        let mut params = Vec::new();
        let mut return_type = Type::Unknown;

        // Parse parameters
        if let Some(params_node) = node.child_by_field_name("parameters") {
            params = self.parse_parameters(&params_node);

            // Check for self parameter and extract attribute assignments
            if let Some(first_param) = params.first() {
                if first_param.name == "self" {
                    // This is an instance method
                    // Parse body for self.attr = ... assignments
                    if method_name == "__init__" {
                        if let Some(body) = node.child_by_field_name("body") {
                            self.parse_init_assignments(&body, class_info);
                        }
                    }
                }
            }
        }

        // Parse return type
        if let Some(return_node) = node.child_by_field_name("return_type") {
            return_type = parse_type_annotation(self.source, &return_node);
        }

        // Handle special return type for __init__
        if method_name == "__init__" {
            return_type = Type::None;
        }

        let method_type = Type::Callable {
            params,
            ret: Box::new(return_type),
        };

        class_info.methods.insert(method_name, method_type);
    }

    /// Parse attribute assignments in __init__
    fn parse_init_assignments(&mut self, body: &Node, class_info: &mut ClassInfo) {
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            if child.kind() == "expression_statement" {
                if let Some(expr) = child.child(0) {
                    if expr.kind() == "assignment" {
                        self.parse_self_assignment(&expr, class_info);
                    }
                }
            }
        }
    }

    /// Parse self.attr = value assignments
    fn parse_self_assignment(&mut self, node: &Node, class_info: &mut ClassInfo) {
        if let Some(left) = node.child_by_field_name("left") {
            if left.kind() == "attribute" {
                if let Some(obj) = left.child_by_field_name("object") {
                    if self.node_text(&obj) == "self" {
                        if let Some(attr) = left.child_by_field_name("attribute") {
                            let attr_name = self.node_text(&attr).to_string();

                            // Get type from annotation or infer from value
                            let attr_type =
                                if let Some(type_node) = node.child_by_field_name("type") {
                                    parse_type_annotation(self.source, &type_node)
                                } else if let Some(value) = node.child_by_field_name("right") {
                                    self.infer_expr(&value)
                                } else {
                                    Type::Unknown
                                };

                            class_info.attributes.insert(attr_name, attr_type);
                        }
                    }
                }
            }
        }
    }

    /// Parse class-level attribute (class variable or annotated attribute)
    fn parse_class_attribute(&mut self, node: &Node, class_info: &mut ClassInfo) {
        if let Some(expr) = node.child(0) {
            match expr.kind() {
                "assignment" => {
                    // name = value or name: type = value
                    if let Some(left) = expr.child_by_field_name("left") {
                        if left.kind() == "identifier" {
                            let attr_name = self.node_text(&left).to_string();
                            let attr_type =
                                if let Some(type_node) = expr.child_by_field_name("type") {
                                    parse_type_annotation(self.source, &type_node)
                                } else if let Some(value) = expr.child_by_field_name("right") {
                                    self.infer_expr(&value)
                                } else {
                                    Type::Unknown
                                };
                            class_info.class_vars.insert(attr_name, attr_type);
                        }
                    }
                }
                // Annotated attribute without assignment: name: type
                _ => {}
            }
        }
    }

    /// Parse function parameters
    fn parse_parameters(&mut self, node: &Node) -> Vec<Param> {
        let mut params = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    params.push(Param {
                        name: self.node_text(&child).to_string(),
                        ty: Type::Unknown,
                        has_default: false,
                        kind: ParamKind::Positional,
                    });
                }
                "typed_parameter" => {
                    let name = child
                        .child_by_field_name("name")
                        .map(|n| self.node_text(&n).to_string())
                        .unwrap_or_default();
                    let ty = child
                        .child_by_field_name("type")
                        .map(|t| parse_type_annotation(self.source, &t))
                        .unwrap_or(Type::Unknown);
                    params.push(Param {
                        name,
                        ty,
                        has_default: false,
                        kind: ParamKind::Positional,
                    });
                }
                "default_parameter" | "typed_default_parameter" => {
                    let name = child
                        .child_by_field_name("name")
                        .map(|n| self.node_text(&n).to_string())
                        .unwrap_or_default();
                    let ty = child
                        .child_by_field_name("type")
                        .map(|t| parse_type_annotation(self.source, &t))
                        .unwrap_or(Type::Unknown);
                    params.push(Param {
                        name,
                        ty,
                        has_default: true,
                        kind: ParamKind::Positional,
                    });
                }
                "list_splat_pattern" => {
                    if let Some(name_node) = child.child(1) {
                        params.push(Param {
                            name: self.node_text(&name_node).to_string(),
                            ty: Type::Tuple(vec![Type::Unknown]),
                            has_default: false,
                            kind: ParamKind::VarPositional,
                        });
                    }
                }
                "dictionary_splat_pattern" => {
                    if let Some(name_node) = child.child(1) {
                        params.push(Param {
                            name: self.node_text(&name_node).to_string(),
                            ty: Type::dict(Type::Str, Type::Unknown),
                            has_default: false,
                            kind: ParamKind::VarKeyword,
                        });
                    }
                }
                _ => {}
            }
        }

        params
    }

    /// Bind a variable to a type based on assignment
    pub fn bind_assignment(&mut self, target: &Node, value_type: Type) {
        if target.kind() == "identifier" {
            let name = self.node_text(target).to_string();
            self.env.bind(name, value_type);
        }
    }

    /// Get the current environment (for testing)
    pub fn env(&self) -> &TypeEnv {
        &self.env
    }

    /// Get all variable types from the current environment
    pub fn get_env_types(&self) -> HashMap<String, Type> {
        self.env.get_all_types()
    }

    /// Analyze an import statement and add imported names to the environment
    pub fn analyze_import(&mut self, node: &Node) {
        if let Some(import) = parse_import(self.source, node) {
            let resolved = self.resolver.resolve_import(&import);
            for (name, ty) in resolved {
                self.env.bind(name, ty);
            }
        }
    }

    /// Parse a TypeVar(...) call and extract variance, bounds, and constraints
    ///
    /// Handles patterns like:
    /// - `TypeVar("T")`
    /// - `TypeVar("T", covariant=True)`
    /// - `TypeVar("T", contravariant=True)`
    /// - `TypeVar("T", bound=SomeType)`
    /// - `TypeVar("T", int, str)` (constraints)
    pub fn parse_typevar_call(&self, node: &Node) -> Option<TypeVarInfo> {
        // Must be a call expression
        if node.kind() != "call" {
            return None;
        }

        // Get function being called
        let func = node.child_by_field_name("function")?;
        let func_name = self.node_text(&func);

        // Must be a TypeVar call
        if func_name != "TypeVar" {
            return None;
        }

        // Get arguments
        let args = node.child_by_field_name("arguments")?;
        let mut cursor = args.walk();

        let mut name: Option<String> = None;
        let mut variance = Variance::Invariant;
        let mut bound: Option<Type> = None;
        let mut constraints: Vec<Type> = Vec::new();
        let mut positional_count = 0;

        for child in args.children(&mut cursor) {
            match child.kind() {
                "string" => {
                    // First string is the TypeVar name
                    if name.is_none() {
                        let text = self.node_text(&child);
                        // Remove quotes
                        let text = text.trim_matches(|c| c == '"' || c == '\'');
                        name = Some(text.to_string());
                    }
                    positional_count += 1;
                }
                "identifier" | "subscript" | "attribute" => {
                    // Positional type constraints (not the first string name)
                    if positional_count > 0 {
                        let ty = parse_type_annotation(self.source, &child);
                        constraints.push(ty);
                    }
                    positional_count += 1;
                }
                "keyword_argument" => {
                    // Parse keyword arguments: covariant, contravariant, bound
                    if let Some(arg_name) = child.child_by_field_name("name") {
                        let key = self.node_text(&arg_name);
                        if let Some(value_node) = child.child_by_field_name("value") {
                            let value = self.node_text(&value_node);
                            match key {
                                "covariant" if value == "True" => {
                                    variance = Variance::Covariant;
                                }
                                "contravariant" if value == "True" => {
                                    variance = Variance::Contravariant;
                                }
                                "bound" => {
                                    bound = Some(parse_type_annotation(self.source, &value_node));
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Some(TypeVarInfo {
            name: name?,
            variance,
            bound,
            constraints,
        })
    }

    /// Handle an assignment that might be a TypeVar declaration
    /// Returns true if a TypeVar was registered
    pub fn try_register_typevar_assignment(&mut self, node: &Node) -> bool {
        // Pattern: T = TypeVar("T", ...)
        if node.kind() != "assignment" {
            return false;
        }

        let left = match node.child_by_field_name("left") {
            Some(l) if l.kind() == "identifier" => l,
            _ => return false,
        };

        let right = match node.child_by_field_name("right") {
            Some(r) if r.kind() == "call" => r,
            _ => return false,
        };

        let var_name = self.node_text(&left).to_string();

        if let Some(info) = self.parse_typevar_call(&right) {
            // Register the TypeVar with extracted variance
            self.register_type_var_with_variance(
                &var_name,
                info.bound,
                info.constraints,
                info.variance,
            );
            return true;
        }

        false
    }
}

/// Information extracted from a TypeVar(...) call
#[derive(Debug, Clone)]
pub struct TypeVarInfo {
    /// The name of the TypeVar (e.g., "T" from TypeVar("T", ...))
    pub name: String,
    /// Variance: covariant, contravariant, or invariant
    pub variance: Variance,
    /// Optional upper bound
    pub bound: Option<Type>,
    /// Type constraints (TypeVar can only be one of these types)
    pub constraints: Vec<Type>,
}

#[cfg(test)]
#[path = "infer_tests.rs"]
mod tests;
