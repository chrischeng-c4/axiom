//! Type narrowing based on control flow analysis
//!
//! This module handles type narrowing for:
//! - isinstance() checks
//! - None checks (is None, is not None)
//! - Truthiness checks
//! - Type guards

use std::collections::HashMap;

use super::ty::Type;

/// Represents a condition that can narrow types
#[derive(Debug, Clone)]
pub enum NarrowingCondition {
    /// isinstance(x, T) or isinstance(x, (T1, T2))
    IsInstance { var_name: String, types: Vec<Type> },
    /// x is None
    IsNone { var_name: String },
    /// x is not None
    IsNotNone { var_name: String },
    /// x (truthiness check)
    Truthy { var_name: String },
    /// not x (falsiness check)
    Falsy { var_name: String },
    /// x == literal
    Equals { var_name: String, value: Type },
    /// x != literal
    NotEquals { var_name: String, value: Type },
    /// Compound: cond1 and cond2
    And(Box<NarrowingCondition>, Box<NarrowingCondition>),
    /// Compound: cond1 or cond2
    Or(Box<NarrowingCondition>, Box<NarrowingCondition>),
    /// Negation: not cond
    Not(Box<NarrowingCondition>),
    /// hasattr(x, "attr") - checks if object has an attribute
    HasAttr { var_name: String, attr_name: String },
    /// callable(x) - checks if object is callable
    IsCallable { var_name: String },
    /// not callable(x)
    NotCallable { var_name: String },
    /// TypeGuard function call (PEP 647)
    /// Narrows type only in positive branch
    TypeGuard {
        var_name: String,
        narrowed_type: Type,
    },
    /// TypeIs function call (PEP 742)
    /// Narrows type in both positive and negative branches
    TypeIs {
        var_name: String,
        narrowed_type: Type,
    },
    /// type(x) is T
    TypeCheck { var_name: String, target_type: Type },
    /// type(x) is not T
    NotTypeCheck { var_name: String, target_type: Type },
    /// Unknown/unanalyzable condition
    Unknown,
}

/// Type narrower that tracks narrowed types in different branches
#[derive(Debug, Clone, Default)]
pub struct TypeNarrower {
    /// Stack of narrowing scopes (for nested if/else)
    scopes: Vec<NarrowingScope>,
}

/// A narrowing scope tracks type refinements in a specific branch
#[derive(Debug, Clone, Default)]
pub struct NarrowingScope {
    /// Narrowed types for variables in this scope
    pub narrowed: HashMap<String, Type>,
}

impl TypeNarrower {
    pub fn new() -> Self {
        Self { scopes: vec![] }
    }

    /// Push a new narrowing scope (entering an if/else branch)
    pub fn push_scope(&mut self) {
        self.scopes.push(NarrowingScope::default());
    }

    /// Pop the current narrowing scope (leaving a branch)
    pub fn pop_scope(&mut self) -> Option<NarrowingScope> {
        self.scopes.pop()
    }

    /// Apply a narrowing condition to the current scope
    pub fn apply_condition(
        &mut self,
        condition: &NarrowingCondition,
        original_types: &HashMap<String, Type>,
    ) {
        match condition {
            NarrowingCondition::IsInstance { var_name, types } => {
                // Narrow to the union of instance types
                let narrowed = if types.len() == 1 {
                    types[0].clone()
                } else {
                    Type::union(types.clone())
                };
                self.narrow_var(var_name, narrowed);
            }
            NarrowingCondition::IsNone { var_name } => {
                self.narrow_var(var_name, Type::None);
            }
            NarrowingCondition::IsNotNone { var_name } => {
                if let Some(original) = original_types.get(var_name) {
                    let narrowed = original.without_none();
                    self.narrow_var(var_name, narrowed);
                }
            }
            NarrowingCondition::Truthy { var_name } => {
                // Truthy narrows away None and False-like values
                if let Some(original) = original_types.get(var_name) {
                    let narrowed = original.without_none();
                    self.narrow_var(var_name, narrowed);
                }
            }
            NarrowingCondition::Falsy { var_name } => {
                // Falsy could mean None, False, 0, "", etc.
                // For Optional types, narrow to None
                if let Some(original) = original_types.get(var_name) {
                    if original.contains_none() {
                        self.narrow_var(var_name, Type::None);
                    }
                }
            }
            NarrowingCondition::Equals { var_name, value } => {
                self.narrow_var(var_name, value.clone());
            }
            NarrowingCondition::NotEquals { var_name, value } => {
                if let Some(original) = original_types.get(var_name) {
                    // If comparing against None, remove None from type
                    if matches!(value, Type::None) {
                        let narrowed = original.without_none();
                        self.narrow_var(var_name, narrowed);
                    }
                }
            }
            NarrowingCondition::And(left, right) => {
                self.apply_condition(left, original_types);
                self.apply_condition(right, original_types);
            }
            NarrowingCondition::Or(left, right) => {
                // For OR, we need to compute the union of both branches
                let mut left_narrower = self.clone();
                let mut right_narrower = self.clone();
                left_narrower.apply_condition(left, original_types);
                right_narrower.apply_condition(right, original_types);

                // Merge: take union of narrowed types
                // This is conservative - we keep the widest possible type
                // For now, just don't narrow on OR
            }
            NarrowingCondition::Not(inner) => {
                // Apply the negation of the condition
                let negated = Self::negate_condition_internal(inner);
                self.apply_condition(&negated, original_types);
            }
            NarrowingCondition::HasAttr {
                var_name,
                attr_name,
            } => {
                // hasattr() doesn't narrow the type itself, but we record it
                // so that attribute access on the variable is considered valid
                // For now, we just mark that we know it has this attribute
                if let Some(original) = original_types.get(var_name) {
                    // If the type has a structural attribute, narrow to that
                    // For now, keep the original type but mark narrowing happened
                    self.narrow_var(
                        var_name,
                        Type::Instance {
                            name: format!("HasAttr[{}, {}]", original, attr_name),
                            module: None,
                            type_args: vec![],
                        },
                    );
                }
            }
            NarrowingCondition::IsCallable { var_name } => {
                // callable() narrows to Callable type
                if let Some(_original) = original_types.get(var_name) {
                    self.narrow_var(
                        var_name,
                        Type::Callable {
                            params: vec![],
                            ret: Box::new(Type::Any),
                        },
                    );
                }
            }
            NarrowingCondition::NotCallable { var_name } => {
                // not callable() - we know it's not callable, but we can't really
                // narrow the type further without more context
                if let Some(original) = original_types.get(var_name) {
                    self.narrow_var(var_name, original.clone());
                }
            }
            NarrowingCondition::TypeGuard {
                var_name,
                narrowed_type,
            } => {
                // TypeGuard[T] narrows only in positive branch
                self.narrow_var(var_name, narrowed_type.clone());
            }
            NarrowingCondition::TypeIs {
                var_name,
                narrowed_type,
            } => {
                // TypeIs[T] also narrows in positive branch
                // The negative branch narrowing happens in negate_condition
                self.narrow_var(var_name, narrowed_type.clone());
            }
            NarrowingCondition::TypeCheck {
                var_name,
                target_type,
            } => {
                // type(x) is T - narrow to exact type
                self.narrow_var(var_name, target_type.clone());
            }
            NarrowingCondition::NotTypeCheck {
                var_name,
                target_type: _,
            } => {
                // type(x) is not T - can't narrow much without more context
                // Keep original type for now
                if let Some(original) = original_types.get(var_name) {
                    self.narrow_var(var_name, original.clone());
                }
            }
            NarrowingCondition::Unknown => {}
        }
    }

    /// Negate a condition (internal use)
    fn negate_condition_internal(condition: &NarrowingCondition) -> NarrowingCondition {
        match condition {
            NarrowingCondition::IsNone { var_name } => NarrowingCondition::IsNotNone {
                var_name: var_name.clone(),
            },
            NarrowingCondition::IsNotNone { var_name } => NarrowingCondition::IsNone {
                var_name: var_name.clone(),
            },
            NarrowingCondition::Truthy { var_name } => NarrowingCondition::Falsy {
                var_name: var_name.clone(),
            },
            NarrowingCondition::Falsy { var_name } => NarrowingCondition::Truthy {
                var_name: var_name.clone(),
            },
            NarrowingCondition::And(left, right) => {
                // not (A and B) = (not A) or (not B)
                NarrowingCondition::Or(
                    Box::new(Self::negate_condition_internal(left)),
                    Box::new(Self::negate_condition_internal(right)),
                )
            }
            NarrowingCondition::Or(left, right) => {
                // not (A or B) = (not A) and (not B)
                NarrowingCondition::And(
                    Box::new(Self::negate_condition_internal(left)),
                    Box::new(Self::negate_condition_internal(right)),
                )
            }
            NarrowingCondition::Not(inner) => {
                // not (not x) = x
                (**inner).clone()
            }
            NarrowingCondition::IsCallable { var_name } => NarrowingCondition::NotCallable {
                var_name: var_name.clone(),
            },
            NarrowingCondition::NotCallable { var_name } => NarrowingCondition::IsCallable {
                var_name: var_name.clone(),
            },
            // TypeGuard doesn't narrow in negative branch (PEP 647)
            NarrowingCondition::TypeGuard { .. } => NarrowingCondition::Unknown,
            // TypeIs narrows in both branches (PEP 742)
            // In negative branch, we exclude the narrowed type
            NarrowingCondition::TypeIs {
                var_name,
                narrowed_type,
            } => {
                // Negation creates an "exclude this type" condition
                // For now, we represent this as Unknown and handle specially
                NarrowingCondition::Not(Box::new(NarrowingCondition::TypeIs {
                    var_name: var_name.clone(),
                    narrowed_type: narrowed_type.clone(),
                }))
            }
            NarrowingCondition::TypeCheck {
                var_name,
                target_type,
            } => NarrowingCondition::NotTypeCheck {
                var_name: var_name.clone(),
                target_type: target_type.clone(),
            },
            NarrowingCondition::NotTypeCheck {
                var_name,
                target_type,
            } => NarrowingCondition::TypeCheck {
                var_name: var_name.clone(),
                target_type: target_type.clone(),
            },
            other => other.clone(),
        }
    }

    /// Set a narrowed type for a variable
    fn narrow_var(&mut self, name: &str, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.narrowed.insert(name.to_string(), ty);
        }
    }

    /// Get the narrowed type for a variable, if any
    pub fn get_narrowed(&self, name: &str) -> Option<&Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.narrowed.get(name) {
                return Some(ty);
            }
        }
        None
    }

    /// Compute the intersection of a type with the narrowed type
    pub fn narrow_type(&self, name: &str, original: &Type) -> Type {
        if let Some(narrowed) = self.get_narrowed(name) {
            // Return the more specific type
            narrowed.clone()
        } else {
            original.clone()
        }
    }
}

/// Negate a narrowing condition (public helper)
pub fn negate_condition(condition: &NarrowingCondition) -> NarrowingCondition {
    match condition {
        NarrowingCondition::IsNone { var_name } => NarrowingCondition::IsNotNone {
            var_name: var_name.clone(),
        },
        NarrowingCondition::IsNotNone { var_name } => NarrowingCondition::IsNone {
            var_name: var_name.clone(),
        },
        NarrowingCondition::Truthy { var_name } => NarrowingCondition::Falsy {
            var_name: var_name.clone(),
        },
        NarrowingCondition::Falsy { var_name } => NarrowingCondition::Truthy {
            var_name: var_name.clone(),
        },
        NarrowingCondition::And(left, right) => {
            // not (A and B) = (not A) or (not B)
            NarrowingCondition::Or(
                Box::new(negate_condition(left)),
                Box::new(negate_condition(right)),
            )
        }
        NarrowingCondition::Or(left, right) => {
            // not (A or B) = (not A) and (not B)
            NarrowingCondition::And(
                Box::new(negate_condition(left)),
                Box::new(negate_condition(right)),
            )
        }
        NarrowingCondition::Not(inner) => {
            // not (not x) = x
            (**inner).clone()
        }
        NarrowingCondition::IsCallable { var_name } => NarrowingCondition::NotCallable {
            var_name: var_name.clone(),
        },
        NarrowingCondition::NotCallable { var_name } => NarrowingCondition::IsCallable {
            var_name: var_name.clone(),
        },
        // TypeGuard doesn't narrow in negative branch (PEP 647)
        NarrowingCondition::TypeGuard { .. } => NarrowingCondition::Unknown,
        // TypeIs narrows in both branches (PEP 742)
        NarrowingCondition::TypeIs {
            var_name,
            narrowed_type,
        } => NarrowingCondition::Not(Box::new(NarrowingCondition::TypeIs {
            var_name: var_name.clone(),
            narrowed_type: narrowed_type.clone(),
        })),
        NarrowingCondition::TypeCheck {
            var_name,
            target_type,
        } => NarrowingCondition::NotTypeCheck {
            var_name: var_name.clone(),
            target_type: target_type.clone(),
        },
        NarrowingCondition::NotTypeCheck {
            var_name,
            target_type,
        } => NarrowingCondition::TypeCheck {
            var_name: var_name.clone(),
            target_type: target_type.clone(),
        },
        other => other.clone(),
    }
}

/// Represents a match case pattern for type narrowing (PEP 634)
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum MatchPattern {
    /// Literal pattern: case 42, case "hello"
    Literal(Type),
    /// Class pattern: case int(), case str()
    Class(Type),
    /// Capture pattern: case x
    Capture(String),
    /// Wildcard pattern: case _
    Wildcard,
    /// Or pattern: case 1 | 2 | 3
    Or(Vec<MatchPattern>),
    /// Sequence pattern: case [x, y, z]
    Sequence(Vec<MatchPattern>),
    /// Mapping pattern: case {"key": value}
    Mapping(Vec<(String, MatchPattern)>),
    /// As pattern: case int() as n
    As(Box<MatchPattern>, String),
    /// Guard: case x if x > 0
    Guard(Box<MatchPattern>, String), // guard expr as string
}

/// Parse a match case pattern and derive narrowing condition
#[allow(dead_code)]
pub fn parse_match_pattern(
    source: &str,
    subject_var: &str,
    pattern_node: &tree_sitter::Node,
) -> NarrowingCondition {
    let node_text =
        |n: &tree_sitter::Node| -> &str { n.utf8_text(source.as_bytes()).unwrap_or("") };

    match pattern_node.kind() {
        // Class pattern: case int(), case str(), case MyClass()
        "class_pattern" => {
            if let Some(class_node) = pattern_node.child_by_field_name("class") {
                let class_name = node_text(&class_node);
                let target_type = parse_simple_type_name(class_name);
                return NarrowingCondition::IsInstance {
                    var_name: subject_var.to_string(),
                    types: vec![target_type],
                };
            }
        }
        // Literal pattern: case 42, case "hello", case None
        "none" => {
            return NarrowingCondition::IsNone {
                var_name: subject_var.to_string(),
            };
        }
        "integer" | "float" | "string" | "true" | "false" => {
            let _pattern_text = node_text(pattern_node);
            let ty = match pattern_node.kind() {
                "integer" => Type::Int,
                "float" => Type::Float,
                "string" => Type::Str,
                "true" | "false" => Type::Bool,
                _ => Type::Unknown,
            };
            return NarrowingCondition::Equals {
                var_name: subject_var.to_string(),
                value: ty,
            };
        }
        // As pattern: case int() as n
        "as_pattern" => {
            // Get the inner pattern and recurse
            if let Some(pattern) = pattern_node.child_by_field_name("pattern") {
                return parse_match_pattern(source, subject_var, &pattern);
            }
        }
        // Or pattern: case int | str
        "or_pattern" | "union_pattern" => {
            let mut types = Vec::new();
            let mut cursor = pattern_node.walk();
            for child in pattern_node.children(&mut cursor) {
                if child.kind() != "|" {
                    // Try to get type from each alternative
                    if let NarrowingCondition::IsInstance { types: t, .. } =
                        parse_match_pattern(source, subject_var, &child)
                    {
                        types.extend(t);
                    }
                }
            }
            if !types.is_empty() {
                return NarrowingCondition::IsInstance {
                    var_name: subject_var.to_string(),
                    types,
                };
            }
        }
        // Wildcard or capture doesn't narrow
        "wildcard_pattern" | "capture_pattern" => {
            return NarrowingCondition::Unknown;
        }
        _ => {}
    }

    NarrowingCondition::Unknown
}

/// Parse a match statement and get the subject variable
#[allow(dead_code)]
pub fn get_match_subject(source: &str, node: &tree_sitter::Node) -> Option<String> {
    if node.kind() != "match_statement" {
        return None;
    }

    if let Some(subject) = node.child_by_field_name("subject") {
        if subject.kind() == "identifier" {
            let text = subject.utf8_text(source.as_bytes()).ok()?;
            return Some(text.to_string());
        }
    }

    None
}

/// Parse an assert statement and extract the narrowing condition
/// e.g., `assert isinstance(x, int)` -> IsInstance { var_name: "x", types: [int] }
/// e.g., `assert x is not None` -> IsNotNone { var_name: "x" }
#[allow(dead_code)]
pub fn parse_assert(source: &str, node: &tree_sitter::Node) -> NarrowingCondition {
    // assert statements have the condition as the first child
    if node.kind() != "assert_statement" {
        return NarrowingCondition::Unknown;
    }

    // Get the condition (skip "assert" keyword)
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() != "assert" {
            return parse_condition(source, &child);
        }
    }

    NarrowingCondition::Unknown
}

/// Parse a condition expression into a NarrowingCondition
pub fn parse_condition(source: &str, node: &tree_sitter::Node) -> NarrowingCondition {
    let node_text =
        |n: &tree_sitter::Node| -> &str { n.utf8_text(source.as_bytes()).unwrap_or("") };

    match node.kind() {
        "comparison_operator" => parse_comparison(source, node),
        "boolean_operator" => parse_boolean_op(source, node),
        "not_operator" => {
            if let Some(arg) = node.child(1) {
                NarrowingCondition::Not(Box::new(parse_condition(source, &arg)))
            } else {
                NarrowingCondition::Unknown
            }
        }
        "call" => parse_call_condition(source, node),
        "identifier" => {
            // Bare identifier is a truthiness check
            NarrowingCondition::Truthy {
                var_name: node_text(node).to_string(),
            }
        }
        _ => NarrowingCondition::Unknown,
    }
}

#[allow(dead_code)]
fn parse_comparison(source: &str, node: &tree_sitter::Node) -> NarrowingCondition {
    let node_text =
        |n: &tree_sitter::Node| -> &str { n.utf8_text(source.as_bytes()).unwrap_or("") };

    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    if children.len() >= 3 {
        let left = &children[0];
        let op = node_text(&children[1]);
        let right = &children[2];

        let left_text = node_text(left);
        let right_text = node_text(right);

        match op {
            "is" => {
                if right_text == "None" && left.kind() == "identifier" {
                    return NarrowingCondition::IsNone {
                        var_name: left_text.to_string(),
                    };
                }
                // Handle type(x) is T
                if let Some(type_check) = parse_type_check(source, left, right, false) {
                    return type_check;
                }
            }
            "is not" => {
                if right_text == "None" && left.kind() == "identifier" {
                    return NarrowingCondition::IsNotNone {
                        var_name: left_text.to_string(),
                    };
                }
                // Handle type(x) is not T
                if let Some(type_check) = parse_type_check(source, left, right, true) {
                    return type_check;
                }
            }
            "==" => {
                if left.kind() == "identifier" && right_text == "None" {
                    return NarrowingCondition::IsNone {
                        var_name: left_text.to_string(),
                    };
                }
            }
            "!=" => {
                if left.kind() == "identifier" && right_text == "None" {
                    return NarrowingCondition::IsNotNone {
                        var_name: left_text.to_string(),
                    };
                }
            }
            _ => {}
        }
    }

    NarrowingCondition::Unknown
}

/// Parse type(x) is T or type(x) is not T patterns
#[allow(dead_code)]
fn parse_type_check(
    source: &str,
    left: &tree_sitter::Node,
    right: &tree_sitter::Node,
    negated: bool,
) -> Option<NarrowingCondition> {
    let node_text =
        |n: &tree_sitter::Node| -> &str { n.utf8_text(source.as_bytes()).unwrap_or("") };

    // Check if left is type(x)
    if left.kind() == "call" {
        if let Some(func) = left.child_by_field_name("function") {
            if node_text(&func) == "type" {
                if let Some(args) = left.child_by_field_name("arguments") {
                    let mut cursor = args.walk();
                    let arg_nodes: Vec<_> = args
                        .children(&mut cursor)
                        .filter(|n| n.kind() != "(" && n.kind() != ")" && n.kind() != ",")
                        .collect();

                    if !arg_nodes.is_empty() && arg_nodes[0].kind() == "identifier" {
                        let var_name = node_text(&arg_nodes[0]).to_string();
                        let target_type = parse_simple_type_name(node_text(right));

                        if negated {
                            return Some(NarrowingCondition::NotTypeCheck {
                                var_name,
                                target_type,
                            });
                        } else {
                            return Some(NarrowingCondition::TypeCheck {
                                var_name,
                                target_type,
                            });
                        }
                    }
                }
            }
        }
    }

    None
}

#[allow(dead_code)]
fn parse_boolean_op(source: &str, node: &tree_sitter::Node) -> NarrowingCondition {
    let node_text =
        |n: &tree_sitter::Node| -> &str { n.utf8_text(source.as_bytes()).unwrap_or("") };

    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    if children.len() >= 3 {
        let left = &children[0];
        let op = node_text(&children[1]);
        let right = &children[2];

        let left_cond = parse_condition(source, left);
        let right_cond = parse_condition(source, right);

        match op {
            "and" => NarrowingCondition::And(Box::new(left_cond), Box::new(right_cond)),
            "or" => NarrowingCondition::Or(Box::new(left_cond), Box::new(right_cond)),
            _ => NarrowingCondition::Unknown,
        }
    } else {
        NarrowingCondition::Unknown
    }
}

#[allow(dead_code)]
fn parse_call_condition(source: &str, node: &tree_sitter::Node) -> NarrowingCondition {
    let node_text =
        |n: &tree_sitter::Node| -> &str { n.utf8_text(source.as_bytes()).unwrap_or("") };

    let func = match node.child_by_field_name("function") {
        Some(f) => f,
        None => return NarrowingCondition::Unknown,
    };

    let func_name = node_text(&func);

    let args = match node.child_by_field_name("arguments") {
        Some(a) => a,
        None => return NarrowingCondition::Unknown,
    };

    let mut cursor = args.walk();
    let arg_nodes: Vec<_> = args
        .children(&mut cursor)
        .filter(|n| n.kind() != "(" && n.kind() != ")" && n.kind() != ",")
        .collect();

    match func_name {
        "isinstance" => {
            if arg_nodes.len() >= 2 {
                let var_node = &arg_nodes[0];
                let type_node = &arg_nodes[1];

                if var_node.kind() == "identifier" {
                    let var_name = node_text(var_node).to_string();
                    let types = parse_isinstance_types(source, type_node);
                    return NarrowingCondition::IsInstance { var_name, types };
                }
            }
        }
        "hasattr" => {
            // hasattr(x, "attr_name")
            if arg_nodes.len() >= 2 {
                let var_node = &arg_nodes[0];
                let attr_node = &arg_nodes[1];

                if var_node.kind() == "identifier" && attr_node.kind() == "string" {
                    let var_name = node_text(var_node).to_string();
                    // Extract string content without quotes
                    let attr_text = node_text(attr_node);
                    let attr_name = attr_text
                        .trim_start_matches(|c| c == '"' || c == '\'')
                        .trim_end_matches(|c| c == '"' || c == '\'')
                        .to_string();
                    return NarrowingCondition::HasAttr {
                        var_name,
                        attr_name,
                    };
                }
            }
        }
        "callable" => {
            // callable(x)
            if !arg_nodes.is_empty() {
                let var_node = &arg_nodes[0];
                if var_node.kind() == "identifier" {
                    let var_name = node_text(var_node).to_string();
                    return NarrowingCondition::IsCallable { var_name };
                }
            }
        }
        _ => {}
    }

    NarrowingCondition::Unknown
}

#[allow(dead_code)]
fn parse_isinstance_types(source: &str, node: &tree_sitter::Node) -> Vec<Type> {
    let node_text =
        |n: &tree_sitter::Node| -> &str { n.utf8_text(source.as_bytes()).unwrap_or("") };

    match node.kind() {
        "identifier" => {
            let name = node_text(node);
            vec![parse_simple_type_name(name)]
        }
        "tuple" => {
            let mut types = Vec::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "identifier" {
                    types.push(parse_simple_type_name(node_text(&child)));
                }
            }
            types
        }
        _ => vec![Type::Unknown],
    }
}

#[allow(dead_code)]
fn parse_simple_type_name(name: &str) -> Type {
    match name {
        "int" => Type::Int,
        "float" => Type::Float,
        "str" => Type::Str,
        "bool" => Type::Bool,
        "bytes" => Type::Bytes,
        "list" => Type::list(Type::Unknown),
        "dict" => Type::dict(Type::Unknown, Type::Unknown),
        "set" => Type::Set(Box::new(Type::Unknown)),
        "tuple" => Type::Tuple(vec![]),
        _ => Type::Instance {
            name: name.to_string(),
            module: None,
            type_args: vec![],
        },
    }
}

#[cfg(test)]
#[path = "narrow_tests.rs"]
mod tests;
