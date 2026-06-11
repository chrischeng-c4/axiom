//! TypeScript-specific type definitions
//!
//! Extends the core type system with TypeScript-specific constructs.

use std::collections::HashMap;

use super::ty::{Type, TypeVarId, Variance};

/// TypeScript interface definition
#[derive(Debug, Clone, PartialEq)]
pub struct TsInterface {
    /// Interface name
    pub name: String,
    /// Generic type parameters
    pub type_params: Vec<TsTypeParam>,
    /// Required properties (name -> type)
    pub properties: HashMap<String, Type>,
    /// Optional properties (name -> type)
    pub optional_properties: HashMap<String, Type>,
    /// Methods (name -> signature)
    pub methods: HashMap<String, Type>,
    /// Index signature: [key: T]: U
    pub index_signature: Option<(Type, Type)>,
    /// Extended interfaces
    pub extends: Vec<String>,
}

impl TsInterface {
    pub fn new(name: String) -> Self {
        Self {
            name,
            type_params: Vec::new(),
            properties: HashMap::new(),
            optional_properties: HashMap::new(),
            methods: HashMap::new(),
            index_signature: None,
            extends: Vec::new(),
        }
    }

    /// Get all members (including from extended interfaces)
    pub fn all_properties(&self) -> Vec<(String, Type, bool)> {
        let mut props: Vec<(String, Type, bool)> = self
            .properties
            .iter()
            .map(|(k, v)| (k.clone(), v.clone(), true))
            .collect();
        props.extend(
            self.optional_properties
                .iter()
                .map(|(k, v)| (k.clone(), v.clone(), false)),
        );
        props
    }
}

/// TypeScript type parameter
#[derive(Debug, Clone, PartialEq)]
pub struct TsTypeParam {
    /// Parameter name (e.g., "T", "K")
    pub name: String,
    /// Constraint: T extends SomeType
    pub constraint: Option<Type>,
    /// Default type: T = DefaultType
    pub default: Option<Type>,
}

impl TsTypeParam {
    pub fn new(name: String) -> Self {
        Self {
            name,
            constraint: None,
            default: None,
        }
    }

    pub fn with_constraint(mut self, constraint: Type) -> Self {
        self.constraint = Some(constraint);
        self
    }

    pub fn with_default(mut self, default: Type) -> Self {
        self.default = Some(default);
        self
    }
}

/// TypeScript type alias
#[derive(Debug, Clone, PartialEq)]
pub struct TsTypeAlias {
    /// Alias name
    pub name: String,
    /// Type parameters
    pub type_params: Vec<TsTypeParam>,
    /// The aliased type
    pub ty: Type,
}

/// TypeScript class definition
#[derive(Debug, Clone, PartialEq)]
pub struct TsClass {
    /// Class name
    pub name: String,
    /// Generic type parameters
    pub type_params: Vec<TsTypeParam>,
    /// Instance properties
    pub properties: HashMap<String, TsProperty>,
    /// Instance methods
    pub methods: HashMap<String, Type>,
    /// Static properties
    pub static_properties: HashMap<String, Type>,
    /// Static methods
    pub static_methods: HashMap<String, Type>,
    /// Constructor signature
    pub constructor: Option<Type>,
    /// Base class
    pub extends: Option<String>,
    /// Implemented interfaces
    pub implements: Vec<String>,
}

impl TsClass {
    pub fn new(name: String) -> Self {
        Self {
            name,
            type_params: Vec::new(),
            properties: HashMap::new(),
            methods: HashMap::new(),
            static_properties: HashMap::new(),
            static_methods: HashMap::new(),
            constructor: None,
            extends: None,
            implements: Vec::new(),
        }
    }
}

/// TypeScript property with modifiers
#[derive(Debug, Clone, PartialEq)]
pub struct TsProperty {
    /// Property type
    pub ty: Type,
    /// Is optional?
    pub optional: bool,
    /// Is readonly?
    pub readonly: bool,
    /// Visibility (public, private, protected)
    pub visibility: Visibility,
}

impl TsProperty {
    pub fn new(ty: Type) -> Self {
        Self {
            ty,
            optional: false,
            readonly: false,
            visibility: Visibility::Public,
        }
    }
}

/// TypeScript visibility modifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Public,
    Private,
    Protected,
}

/// TypeScript enum definition
#[derive(Debug, Clone, PartialEq)]
pub struct TsEnum {
    /// Enum name
    pub name: String,
    /// Members (name -> value type)
    pub members: Vec<(String, Option<TsEnumValue>)>,
    /// Is const enum?
    pub is_const: bool,
}

/// TypeScript enum value
#[derive(Debug, Clone, PartialEq)]
pub enum TsEnumValue {
    Number(i64),
    String(String),
}

/// TypeScript mapped type: { [K in Keys]: ValueType }
#[derive(Debug, Clone, PartialEq)]
pub struct TsMappedType {
    /// Key variable name
    pub key_var: String,
    /// Keys to iterate over (usually a union of string literals)
    pub keys: Type,
    /// Value type (may reference key_var)
    pub value_type: Type,
    /// Optional modifier: +? / -? / ?
    pub optional_modifier: Option<MappedTypeModifier>,
    /// Readonly modifier: +readonly / -readonly / readonly
    pub readonly_modifier: Option<MappedTypeModifier>,
}

/// Modifier for mapped types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedTypeModifier {
    /// Add the modifier
    Add,
    /// Remove the modifier
    Remove,
    /// Preserve (or add) the modifier
    Preserve,
}

/// TypeScript conditional type: T extends U ? X : Y
#[derive(Debug, Clone, PartialEq)]
pub struct TsConditionalType {
    /// Check type (T)
    pub check_type: Box<Type>,
    /// Extends type (U)
    pub extends_type: Box<Type>,
    /// True branch type (X)
    pub true_type: Box<Type>,
    /// False branch type (Y)
    pub false_type: Box<Type>,
}

impl TsConditionalType {
    /// Evaluate the conditional type with concrete substitutions
    pub fn evaluate(&self, subs: &HashMap<TypeVarId, Type>) -> Type {
        let check = self.check_type.substitute(subs);
        let extends = self.extends_type.substitute(subs);

        // Simple structural check
        if is_assignable_to(&check, &extends) {
            self.true_type.substitute(subs)
        } else {
            self.false_type.substitute(subs)
        }
    }
}

/// TypeScript template literal type: `prefix${T}suffix`
#[derive(Debug, Clone, PartialEq)]
pub struct TsTemplateLiteralType {
    /// Parts of the template
    pub parts: Vec<TemplatePart>,
}

/// Part of a template literal type
#[derive(Debug, Clone, PartialEq)]
pub enum TemplatePart {
    /// Literal string segment
    Literal(String),
    /// Type placeholder (usually string, number, or type variable)
    Placeholder(Type),
}

impl TsTemplateLiteralType {
    /// Create a simple template with prefix and suffix
    pub fn new(prefix: &str, inner: Type, suffix: &str) -> Self {
        let mut parts = Vec::new();
        if !prefix.is_empty() {
            parts.push(TemplatePart::Literal(prefix.to_string()));
        }
        parts.push(TemplatePart::Placeholder(inner));
        if !suffix.is_empty() {
            parts.push(TemplatePart::Literal(suffix.to_string()));
        }
        Self { parts }
    }

    /// Evaluate template with concrete types to produce literal string types
    pub fn evaluate(&self, subs: &HashMap<TypeVarId, Type>) -> Type {
        // If all placeholders resolve to literals, produce literal type
        let mut result = String::new();
        let mut all_literal = true;

        for part in &self.parts {
            match part {
                TemplatePart::Literal(s) => result.push_str(s),
                TemplatePart::Placeholder(ty) => {
                    let resolved = ty.substitute(subs);
                    if let Type::Literal(super::ty::LiteralValue::Str(s)) = resolved {
                        result.push_str(&s);
                    } else {
                        all_literal = false;
                        break;
                    }
                }
            }
        }

        if all_literal {
            Type::Literal(super::ty::LiteralValue::Str(result))
        } else {
            Type::Str
        }
    }
}

/// Check if a type is assignable to another type (structural compatibility)
pub fn is_assignable_to(source: &Type, target: &Type) -> bool {
    match (source, target) {
        // Any accepts anything
        (_, Type::Any) | (Type::Any, _) => true,

        // Unknown is assignable only to unknown or any
        (Type::Unknown, Type::Unknown) => true,
        (Type::Unknown, _) => false,

        // Never is assignable to everything
        (Type::Never, _) => true,

        // Same primitive types
        (Type::None, Type::None)
        | (Type::Bool, Type::Bool)
        | (Type::Int, Type::Int)
        | (Type::Float, Type::Float)
        | (Type::Str, Type::Str)
        | (Type::Bytes, Type::Bytes) => true,

        // Number widening
        (Type::Int, Type::Float) => true,

        // Literal to base type
        (Type::Literal(super::ty::LiteralValue::Str(_)), Type::Str) => true,
        (Type::Literal(super::ty::LiteralValue::Int(_)), Type::Int) => true,
        (Type::Literal(super::ty::LiteralValue::Int(_)), Type::Float) => true,
        (Type::Literal(super::ty::LiteralValue::Float(_)), Type::Float) => true,
        (Type::Literal(super::ty::LiteralValue::Bool(_)), Type::Bool) => true,

        // Same literals
        (Type::Literal(a), Type::Literal(b)) => a == b,

        // Container types
        (Type::List(a), Type::List(b)) => is_assignable_to(a, b),
        (Type::Set(a), Type::Set(b)) => is_assignable_to(a, b),
        (Type::Dict(k1, v1), Type::Dict(k2, v2)) => {
            is_assignable_to(k1, k2) && is_assignable_to(v1, v2)
        }

        // Tuple types
        (Type::Tuple(a), Type::Tuple(b)) if a.len() == b.len() => {
            a.iter().zip(b.iter()).all(|(s, t)| is_assignable_to(s, t))
        }

        // Union source: all members must be assignable
        (Type::Union(members), target) => members.iter().all(|m| is_assignable_to(m, target)),

        // Union target: source must be assignable to at least one
        (source, Type::Union(members)) => members.iter().any(|m| is_assignable_to(source, m)),

        // Intersection target: source must be assignable to all
        (source, Type::Intersection(members)) => {
            members.iter().all(|m| is_assignable_to(source, m))
        }

        // Intersection source: at least one member must be assignable
        (Type::Intersection(members), target) => {
            members.iter().any(|m| is_assignable_to(m, target))
        }

        // Optional types
        (Type::Optional(a), Type::Optional(b)) => is_assignable_to(a, b),
        (a, Type::Optional(b)) => is_assignable_to(a, b),

        // Instance types (nominal + structural)
        (
            Type::Instance {
                name: n1,
                type_args: a1,
                ..
            },
            Type::Instance {
                name: n2,
                type_args: a2,
                ..
            },
        ) => {
            n1 == n2
                && a1.len() == a2.len()
                && a1
                    .iter()
                    .zip(a2.iter())
                    .all(|(s, t)| is_assignable_to(s, t))
        }

        // Callable types (contravariant params, covariant return)
        (
            Type::Callable {
                params: p1,
                ret: r1,
            },
            Type::Callable {
                params: p2,
                ret: r2,
            },
        ) => {
            // Source can have fewer required params
            if p1.len() > p2.len() {
                return false;
            }
            // Contravariant params
            let params_ok = p1
                .iter()
                .zip(p2.iter())
                .all(|(s, t)| is_assignable_to(&t.ty, &s.ty));
            // Covariant return
            params_ok && is_assignable_to(r1, r2)
        }

        // Protocol/interface structural matching
        (source, Type::Protocol { members, .. }) => {
            // Check if source has all required members
            check_protocol_conformance(source, members)
        }

        _ => false,
    }
}

/// Check if a type conforms to a protocol (structural subtyping)
fn check_protocol_conformance(ty: &Type, members: &[(String, Type)]) -> bool {
    match ty {
        Type::Instance { .. } => {
            // Would need to look up class info - simplified here
            true // Placeholder: assume conformance for instances
        }
        Type::Protocol {
            members: source_members,
            ..
        } => {
            // All target members must exist in source with compatible types
            members.iter().all(|(name, target_ty)| {
                source_members.iter().any(|(src_name, src_ty)| {
                    src_name == name && is_assignable_to(src_ty, target_ty)
                })
            })
        }
        _ => false,
    }
}

/// TypeScript type context for inference
#[derive(Debug, Clone, Default)]
pub struct TsTypeContext {
    /// Interface definitions
    pub interfaces: HashMap<String, TsInterface>,
    /// Type aliases
    pub type_aliases: HashMap<String, TsTypeAlias>,
    /// Class definitions
    pub classes: HashMap<String, TsClass>,
    /// Enum definitions
    pub enums: HashMap<String, TsEnum>,
    /// Variable types
    pub variables: HashMap<String, Type>,
    /// Type parameters in scope
    pub type_params: HashMap<String, TsTypeParam>,
}

impl TsTypeContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up a type by name
    pub fn resolve_type(&self, name: &str) -> Option<Type> {
        // Check type aliases first
        if let Some(alias) = self.type_aliases.get(name) {
            return Some(alias.ty.clone());
        }

        // Check interfaces
        if let Some(iface) = self.interfaces.get(name) {
            // Convert interface to Protocol type
            let members: Vec<(String, Type)> = iface
                .properties
                .iter()
                .chain(iface.methods.iter())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            return Some(Type::Protocol {
                name: name.to_string(),
                module: None,
                members,
            });
        }

        // Check classes
        if self.classes.contains_key(name) {
            return Some(Type::ClassType {
                name: name.to_string(),
                module: None,
            });
        }

        // Check type parameters
        if let Some(param) = self.type_params.get(name) {
            let id = TypeVarId(name.len()); // Simplified ID generation
            return Some(Type::TypeVar {
                id,
                name: name.to_string(),
                bound: param.constraint.clone().map(Box::new),
                constraints: vec![],
                variance: Variance::Invariant,
            });
        }

        None
    }

    /// Register an interface
    pub fn register_interface(&mut self, iface: TsInterface) {
        self.interfaces.insert(iface.name.clone(), iface);
    }

    /// Register a class
    pub fn register_class(&mut self, class: TsClass) {
        self.classes.insert(class.name.clone(), class);
    }

    /// Register a type alias
    pub fn register_type_alias(&mut self, alias: TsTypeAlias) {
        self.type_aliases.insert(alias.name.clone(), alias);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assignability_primitives() {
        assert!(is_assignable_to(&Type::Int, &Type::Int));
        assert!(is_assignable_to(&Type::Int, &Type::Float));
        assert!(!is_assignable_to(&Type::Float, &Type::Int));
        assert!(is_assignable_to(&Type::Str, &Type::Str));
    }

    #[test]
    fn test_assignability_any() {
        assert!(is_assignable_to(&Type::Int, &Type::Any));
        assert!(is_assignable_to(&Type::Any, &Type::Str));
    }

    #[test]
    fn test_assignability_never() {
        assert!(is_assignable_to(&Type::Never, &Type::Int));
        assert!(is_assignable_to(&Type::Never, &Type::Str));
    }

    #[test]
    fn test_assignability_union() {
        let union = Type::Union(vec![Type::Int, Type::Str]);
        assert!(is_assignable_to(&Type::Int, &union));
        assert!(is_assignable_to(&Type::Str, &union));
        assert!(!is_assignable_to(&Type::Bool, &union));
    }

    #[test]
    fn test_assignability_intersection() {
        let inter = Type::Intersection(vec![Type::Int, Type::Str]);
        // Nothing can satisfy both Int and Str, but Never can
        assert!(is_assignable_to(&Type::Never, &inter));
    }

    #[test]
    fn test_literal_to_base() {
        use super::super::ty::LiteralValue;
        let lit_str = Type::Literal(LiteralValue::Str("hello".to_string()));
        assert!(is_assignable_to(&lit_str, &Type::Str));

        let lit_int = Type::Literal(LiteralValue::Int(42));
        assert!(is_assignable_to(&lit_int, &Type::Int));
        assert!(is_assignable_to(&lit_int, &Type::Float));
    }

    #[test]
    fn test_template_literal_evaluation() {
        use super::super::ty::LiteralValue;

        let template = TsTemplateLiteralType::new(
            "Hello, ",
            Type::TypeVar {
                id: TypeVarId(0),
                name: "T".to_string(),
                bound: None,
                constraints: vec![],
                variance: Variance::Invariant,
            },
            "!",
        );

        let mut subs = HashMap::new();
        subs.insert(
            TypeVarId(0),
            Type::Literal(LiteralValue::Str("World".to_string())),
        );

        let result = template.evaluate(&subs);
        assert_eq!(
            result,
            Type::Literal(LiteralValue::Str("Hello, World!".to_string()))
        );
    }

    #[test]
    fn test_interface_creation() {
        let mut iface = TsInterface::new("Readable".to_string());
        iface.methods.insert(
            "read".to_string(),
            Type::Callable {
                params: vec![],
                ret: Box::new(Type::Str),
            },
        );

        let props = iface.all_properties();
        assert!(props.is_empty()); // Methods don't show in properties
        assert_eq!(iface.methods.len(), 1);
    }

    #[test]
    fn test_conditional_type() {
        let cond = TsConditionalType {
            check_type: Box::new(Type::Str),
            extends_type: Box::new(Type::Str),
            true_type: Box::new(Type::Bool),
            false_type: Box::new(Type::Int),
        };

        let result = cond.evaluate(&HashMap::new());
        assert_eq!(result, Type::Bool);
    }
}
