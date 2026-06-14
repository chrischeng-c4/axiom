//! Core type definitions for the Argus type system

use std::collections::HashMap;
use std::fmt;

/// Unique identifier for type variables
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVarId(pub usize);

/// Unique identifier for ParamSpec (PEP 612)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParamSpecId(pub usize);

/// Unique identifier for TypeVarTuple (PEP 646)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVarTupleId(pub usize);

/// Variance of a TypeVar for generic types
///
/// Variance determines how subtyping works for generic types:
/// - Invariant: T[A] is only a subtype of T[B] if A == B (e.g., list in Python is invariant)
/// - Covariant: T[A] is a subtype of T[B] if A is a subtype of B (e.g., Sequence, return types)
/// - Contravariant: T[A] is a subtype of T[B] if B is a subtype of A (e.g., function params)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Variance {
    /// Default: A and B must be exactly equal
    #[default]
    Invariant,
    /// T[A] <: T[B] if A <: B (e.g., Iterator[Cat] <: Iterator[Animal])
    Covariant,
    /// T[A] <: T[B] if B <: A (e.g., Callable[[Animal], None] <: Callable[[Cat], None])
    Contravariant,
}

impl Variance {
    /// Check if this variance allows subtyping in the given direction
    ///
    /// - Covariant: subtype of parameter means subtype of whole type
    /// - Contravariant: supertype of parameter means subtype of whole type
    /// - Invariant: exact match required
    pub fn allows_subtype(&self, is_subtype: bool, is_supertype: bool) -> bool {
        match self {
            Variance::Invariant => is_subtype && is_supertype, // Must be equal
            Variance::Covariant => is_subtype,
            Variance::Contravariant => is_supertype,
        }
    }
}

/// Parameter kind in function signatures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    /// Regular positional parameter
    Positional,
    /// Positional-only parameter (before /)
    PositionalOnly,
    /// Keyword-only parameter (after *)
    KeywordOnly,
    /// *args parameter
    VarPositional,
    /// **kwargs parameter
    VarKeyword,
}

/// Function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub has_default: bool,
    pub kind: ParamKind,
}

/// Literal value for Literal types
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    None,
}

/// The core Type enum representing all possible types
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // === Primitive types ===
    /// The bottom type (NoReturn in Python, never in TypeScript)
    Never,
    /// None / null / unit type
    None,
    /// Boolean
    Bool,
    /// Integer (Python int, TypeScript number)
    Int,
    /// Floating point (Python float, TypeScript number)
    Float,
    /// String
    Str,
    /// Bytes (Python only)
    Bytes,

    // === Container types ===
    /// List / Array
    List(Box<Type>),
    /// Dictionary / Map / Object
    Dict(Box<Type>, Box<Type>),
    /// Set
    Set(Box<Type>),
    /// Tuple with fixed element types
    Tuple(Vec<Type>),

    // === Composite types ===
    /// Optional type (T | None)
    Optional(Box<Type>),
    /// Union type (T | U | V)
    Union(Vec<Type>),
    /// Intersection type (T & U) - TypeScript only
    Intersection(Vec<Type>),

    // === Callable types ===
    /// Function / Callable type
    Callable { params: Vec<Param>, ret: Box<Type> },

    // === Class types ===
    /// Class / Interface / Struct instance
    Instance {
        name: String,
        module: Option<String>,
        type_args: Vec<Type>,
    },
    /// Class type itself (for type[T])
    ClassType {
        name: String,
        module: Option<String>,
    },

    // === Generic types ===
    /// Type variable (T, K, V, etc.)
    TypeVar {
        id: TypeVarId,
        name: String,
        bound: Option<Box<Type>>,
        constraints: Vec<Type>,
        /// Variance of this TypeVar (covariant, contravariant, or invariant)
        variance: Variance,
    },

    // === Protocol types ===
    /// Protocol type (structural subtyping)
    /// A type conforms to a Protocol if it has all required members
    Protocol {
        name: String,
        module: Option<String>,
        /// Required members (method/attribute name -> type)
        members: Vec<(String, Type)>,
    },

    // === TypedDict ===
    /// TypedDict type - dictionary with specific keys and types
    TypedDict {
        name: String,
        /// Fields with their types and whether they are required
        fields: Vec<(String, Type, bool)>, // (name, type, required)
        /// Whether all fields are required by default
        total: bool,
    },

    // === Special types ===
    /// Any type - disables type checking
    Any,
    /// Unknown type - not yet inferred
    Unknown,
    /// Literal type (Literal["foo"], Literal[42])
    Literal(LiteralValue),
    /// Self type (PEP 673) - references the enclosing class
    SelfType {
        /// The class this Self refers to (resolved during checking)
        class_name: Option<String>,
    },
    /// LiteralString type (PEP 675) - string known at compile time
    LiteralString,
    /// Final type (PEP 591) - value cannot be reassigned
    Final(Box<Type>),
    /// Annotated type (PEP 593) - type with runtime metadata
    Annotated {
        inner: Box<Type>,
        metadata: Vec<String>, // Simplified: just store as strings
    },

    // === Overloaded functions ===
    /// Overloaded function with multiple signatures
    Overloaded {
        signatures: Vec<Type>, // Each should be a Callable
    },

    // === ParamSpec (PEP 612) ===
    /// ParamSpec - captures function parameter types
    ParamSpec { id: ParamSpecId, name: String },
    /// Concatenate - prepend params to a ParamSpec
    Concatenate {
        params: Vec<Type>,
        param_spec: Box<Type>, // Should be a ParamSpec
    },

    // === TypeVarTuple (PEP 646) ===
    /// TypeVarTuple - variadic type variable
    TypeVarTuple { id: TypeVarTupleId, name: String },
    /// Unpacked TypeVarTuple (*Ts)
    Unpack(Box<Type>),

    // === Type Guards (PEP 647, 742) ===
    /// TypeGuard[T] - narrows type only in positive branch
    TypeGuard(Box<Type>),
    /// TypeIs[T] - narrows type in both positive and negative branches
    TypeIs(Box<Type>),

    // === Error type ===
    /// Type error placeholder (allows continued analysis)
    Error,
}

impl Type {
    /// Create an Optional type
    pub fn optional(inner: Type) -> Self {
        Type::Optional(Box::new(inner))
    }

    /// Create a List type
    pub fn list(element: Type) -> Self {
        Type::List(Box::new(element))
    }

    /// Create a Dict type
    pub fn dict(key: Type, value: Type) -> Self {
        Type::Dict(Box::new(key), Box::new(value))
    }

    /// Create a Union type, flattening nested unions
    pub fn union(types: Vec<Type>) -> Self {
        let mut flattened = Vec::new();
        for ty in types {
            match ty {
                Type::Union(inner) => flattened.extend(inner),
                Type::Never => {} // Never is identity for union
                other => {
                    if !flattened.contains(&other) {
                        flattened.push(other);
                    }
                }
            }
        }
        match flattened.len() {
            0 => Type::Never,
            // SAFETY: We checked that len() == 1, so pop() will always return Some
            1 => flattened
                .pop()
                .expect("flattened vec has exactly one element"),
            _ => Type::Union(flattened),
        }
    }

    /// Create a simple Callable type
    pub fn callable(params: Vec<Type>, ret: Type) -> Self {
        let params = params
            .into_iter()
            .enumerate()
            .map(|(i, ty)| Param {
                name: format!("_{}", i),
                ty,
                has_default: false,
                kind: ParamKind::Positional,
            })
            .collect();
        Type::Callable {
            params,
            ret: Box::new(ret),
        }
    }

    /// Check if this type is a subtype of Any
    pub fn is_any(&self) -> bool {
        matches!(self, Type::Any)
    }

    /// Check if this type is Unknown
    pub fn is_unknown(&self) -> bool {
        matches!(self, Type::Unknown)
    }

    /// Check if this type is an error
    pub fn is_error(&self) -> bool {
        matches!(self, Type::Error)
    }

    /// Check if this type contains None
    pub fn contains_none(&self) -> bool {
        match self {
            Type::None => true,
            Type::Optional(_) => true,
            Type::Union(types) => types.iter().any(|t| t.contains_none()),
            _ => false,
        }
    }

    /// Remove None from this type
    pub fn without_none(&self) -> Type {
        match self {
            Type::None => Type::Never,
            Type::Optional(inner) => (**inner).clone(),
            Type::Union(types) => {
                let filtered: Vec<_> = types
                    .iter()
                    .filter(|t| !matches!(t, Type::None))
                    .cloned()
                    .collect();
                Type::union(filtered)
            }
            other => other.clone(),
        }
    }

    /// Substitute type variables with concrete types
    pub fn substitute(&self, substitutions: &std::collections::HashMap<TypeVarId, Type>) -> Type {
        match self {
            Type::TypeVar { id, .. } => substitutions
                .get(id)
                .cloned()
                .unwrap_or_else(|| self.clone()),
            Type::List(elem) => Type::List(Box::new(elem.substitute(substitutions))),
            Type::Dict(k, v) => Type::Dict(
                Box::new(k.substitute(substitutions)),
                Box::new(v.substitute(substitutions)),
            ),
            Type::Set(elem) => Type::Set(Box::new(elem.substitute(substitutions))),
            Type::Tuple(elems) => {
                Type::Tuple(elems.iter().map(|t| t.substitute(substitutions)).collect())
            }
            Type::Optional(inner) => Type::Optional(Box::new(inner.substitute(substitutions))),
            Type::Union(types) => {
                Type::union(types.iter().map(|t| t.substitute(substitutions)).collect())
            }
            Type::Intersection(types) => {
                Type::Intersection(types.iter().map(|t| t.substitute(substitutions)).collect())
            }
            Type::Callable { params, ret } => Type::Callable {
                params: params
                    .iter()
                    .map(|p| Param {
                        name: p.name.clone(),
                        ty: p.ty.substitute(substitutions),
                        has_default: p.has_default,
                        kind: p.kind,
                    })
                    .collect(),
                ret: Box::new(ret.substitute(substitutions)),
            },
            Type::Instance {
                name,
                module,
                type_args,
            } => Type::Instance {
                name: name.clone(),
                module: module.clone(),
                type_args: type_args
                    .iter()
                    .map(|t| t.substitute(substitutions))
                    .collect(),
            },
            // Other types are unchanged
            other => other.clone(),
        }
    }

    /// Collect all type variables in this type
    pub fn type_vars(&self) -> Vec<TypeVarId> {
        let mut vars = Vec::new();
        self.collect_type_vars(&mut vars);
        vars
    }

    fn collect_type_vars(&self, vars: &mut Vec<TypeVarId>) {
        match self {
            Type::TypeVar { id, .. } => {
                if !vars.contains(id) {
                    vars.push(*id);
                }
            }
            Type::List(elem) | Type::Set(elem) | Type::Optional(elem) => {
                elem.collect_type_vars(vars);
            }
            Type::Dict(k, v) => {
                k.collect_type_vars(vars);
                v.collect_type_vars(vars);
            }
            Type::Tuple(elems) | Type::Union(elems) | Type::Intersection(elems) => {
                for elem in elems {
                    elem.collect_type_vars(vars);
                }
            }
            Type::Callable { params, ret } => {
                for param in params {
                    param.ty.collect_type_vars(vars);
                }
                ret.collect_type_vars(vars);
            }
            Type::Instance { type_args, .. } => {
                for arg in type_args {
                    arg.collect_type_vars(vars);
                }
            }
            _ => {}
        }
    }

    /// Unify this type (pattern) with a concrete type, collecting TypeVar bindings
    /// Returns Some(substitutions) on success, None on failure
    pub fn unify(&self, concrete: &Type, subs: &mut HashMap<TypeVarId, Type>) -> bool {
        match (self, concrete) {
            // TypeVar: bind it to the concrete type
            (Type::TypeVar { id, .. }, _) => {
                if let Some(existing) = subs.get(id) {
                    // Already bound - check consistency
                    existing == concrete
                } else {
                    subs.insert(*id, concrete.clone());
                    true
                }
            }

            // Same primitive types
            (Type::Never, Type::Never)
            | (Type::None, Type::None)
            | (Type::Bool, Type::Bool)
            | (Type::Int, Type::Int)
            | (Type::Float, Type::Float)
            | (Type::Str, Type::Str)
            | (Type::Bytes, Type::Bytes)
            | (Type::Any, _)
            | (_, Type::Any)
            | (Type::Unknown, _)
            | (_, Type::Unknown) => true,

            // Numeric widening: int -> float
            (Type::Float, Type::Int) => true,

            // Container types: unify element types
            (Type::List(a), Type::List(b)) => a.unify(b, subs),
            (Type::Set(a), Type::Set(b)) => a.unify(b, subs),
            (Type::Optional(a), Type::Optional(b)) => a.unify(b, subs),
            (Type::Optional(a), b) if !matches!(b, Type::None) => a.unify(b, subs),

            (Type::Dict(k1, v1), Type::Dict(k2, v2)) => k1.unify(k2, subs) && v1.unify(v2, subs),

            (Type::Tuple(a), Type::Tuple(b)) if a.len() == b.len() => {
                a.iter().zip(b.iter()).all(|(t1, t2)| t1.unify(t2, subs))
            }

            // Callable: unify params and return
            (
                Type::Callable {
                    params: p1,
                    ret: r1,
                },
                Type::Callable {
                    params: p2,
                    ret: r2,
                },
            ) if p1.len() == p2.len() => {
                let params_ok = p1
                    .iter()
                    .zip(p2.iter())
                    .all(|(a, b)| a.ty.unify(&b.ty, subs));
                params_ok && r1.unify(r2, subs)
            }

            // Instance types: same name, unify type args
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
            ) if n1 == n2 && a1.len() == a2.len() => {
                a1.iter().zip(a2.iter()).all(|(t1, t2)| t1.unify(t2, subs))
            }

            // Union: concrete must unify with at least one member
            (Type::Union(types), concrete) => types.iter().any(|t| {
                let mut temp_subs = subs.clone();
                if t.unify(concrete, &mut temp_subs) {
                    *subs = temp_subs;
                    true
                } else {
                    false
                }
            }),

            // Same type exactly
            (a, b) if a == b => true,

            _ => false,
        }
    }

    /// Create a TypeVar (invariant by default)
    pub fn type_var(id: usize, name: &str) -> Type {
        Type::TypeVar {
            id: TypeVarId(id),
            name: name.to_string(),
            bound: None,
            constraints: vec![],
            variance: Variance::Invariant,
        }
    }

    /// Create a TypeVar with a bound (invariant by default)
    pub fn type_var_bounded(id: usize, name: &str, bound: Type) -> Type {
        Type::TypeVar {
            id: TypeVarId(id),
            name: name.to_string(),
            bound: Some(Box::new(bound)),
            constraints: vec![],
            variance: Variance::Invariant,
        }
    }

    /// Create a TypeVar with specific variance
    pub fn type_var_with_variance(id: usize, name: &str, variance: Variance) -> Type {
        Type::TypeVar {
            id: TypeVarId(id),
            name: name.to_string(),
            bound: None,
            constraints: vec![],
            variance,
        }
    }

    /// Create a covariant TypeVar
    pub fn type_var_covariant(id: usize, name: &str) -> Type {
        Type::type_var_with_variance(id, name, Variance::Covariant)
    }

    /// Create a contravariant TypeVar
    pub fn type_var_contravariant(id: usize, name: &str) -> Type {
        Type::type_var_with_variance(id, name, Variance::Contravariant)
    }

    /// Create a ParamSpec (PEP 612)
    pub fn param_spec(id: usize, name: &str) -> Type {
        Type::ParamSpec {
            id: ParamSpecId(id),
            name: name.to_string(),
        }
    }

    /// Create a TypeVarTuple (PEP 646)
    pub fn type_var_tuple(id: usize, name: &str) -> Type {
        Type::TypeVarTuple {
            id: TypeVarTupleId(id),
            name: name.to_string(),
        }
    }

    /// Create a Final type (PEP 591)
    pub fn final_type(inner: Type) -> Type {
        Type::Final(Box::new(inner))
    }

    /// Create an Annotated type (PEP 593)
    pub fn annotated(inner: Type, metadata: Vec<String>) -> Type {
        Type::Annotated {
            inner: Box::new(inner),
            metadata,
        }
    }

    /// Create a Self type (PEP 673)
    pub fn self_type(class_name: Option<String>) -> Type {
        Type::SelfType { class_name }
    }

    /// Create an Overloaded function type
    pub fn overloaded(signatures: Vec<Type>) -> Type {
        Type::Overloaded { signatures }
    }

    /// Create a Concatenate type (PEP 612)
    pub fn concatenate(params: Vec<Type>, param_spec: Type) -> Type {
        Type::Concatenate {
            params,
            param_spec: Box::new(param_spec),
        }
    }

    /// Unwrap Final to get inner type
    pub fn unwrap_final(&self) -> &Type {
        match self {
            Type::Final(inner) => inner,
            other => other,
        }
    }

    /// Unwrap Annotated to get inner type
    pub fn unwrap_annotated(&self) -> &Type {
        match self {
            Type::Annotated { inner, .. } => inner,
            other => other,
        }
    }

    /// Check if this is a Final type
    pub fn is_final(&self) -> bool {
        matches!(self, Type::Final(_))
    }

    /// Check if this is a LiteralString
    pub fn is_literal_string(&self) -> bool {
        matches!(
            self,
            Type::LiteralString | Type::Literal(LiteralValue::Str(_))
        )
    }

    /// Create a TypeGuard type (PEP 647)
    pub fn type_guard(inner: Type) -> Type {
        Type::TypeGuard(Box::new(inner))
    }

    /// Create a TypeIs type (PEP 742)
    pub fn type_is(inner: Type) -> Type {
        Type::TypeIs(Box::new(inner))
    }

    /// Check if this is a TypeGuard
    pub fn is_type_guard(&self) -> bool {
        matches!(self, Type::TypeGuard(_))
    }

    /// Check if this is a TypeIs
    pub fn is_type_is(&self) -> bool {
        matches!(self, Type::TypeIs(_))
    }

    /// Get the narrowed type from TypeGuard or TypeIs
    pub fn get_guard_type(&self) -> Option<&Type> {
        match self {
            Type::TypeGuard(inner) | Type::TypeIs(inner) => Some(inner),
            _ => None,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Never => write!(f, "Never"),
            Type::None => write!(f, "None"),
            Type::Bool => write!(f, "bool"),
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Str => write!(f, "str"),
            Type::Bytes => write!(f, "bytes"),

            Type::List(elem) => write!(f, "list[{}]", elem),
            Type::Dict(k, v) => write!(f, "dict[{}, {}]", k, v),
            Type::Set(elem) => write!(f, "set[{}]", elem),
            Type::Tuple(elems) => {
                write!(f, "tuple[")?;
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }

            Type::Optional(inner) => write!(f, "{} | None", inner),
            Type::Union(types) => {
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                Ok(())
            }
            Type::Intersection(types) => {
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, " & ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                Ok(())
            }

            Type::Callable { params, ret } => {
                write!(f, "(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param.ty)?;
                }
                write!(f, ") -> {}", ret)
            }

            Type::Instance {
                name, type_args, ..
            } => {
                write!(f, "{}", name)?;
                if !type_args.is_empty() {
                    write!(f, "[")?;
                    for (i, arg) in type_args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, "]")?;
                }
                Ok(())
            }
            Type::ClassType { name, .. } => write!(f, "type[{}]", name),

            Type::Protocol { name, members, .. } => {
                write!(f, "Protocol[{}]", name)?;
                if !members.is_empty() {
                    write!(f, "{{")?;
                    for (i, (member_name, _)) in members.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", member_name)?;
                    }
                    write!(f, "}}")?;
                }
                Ok(())
            }

            Type::TypedDict { name, fields, .. } => {
                write!(f, "TypedDict[{}]", name)?;
                if !fields.is_empty() {
                    write!(f, "{{")?;
                    for (i, (field_name, field_ty, required)) in fields.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        if *required {
                            write!(f, "{}: {}", field_name, field_ty)?;
                        } else {
                            write!(f, "{}?: {}", field_name, field_ty)?;
                        }
                    }
                    write!(f, "}}")?;
                }
                Ok(())
            }

            Type::TypeVar { name, .. } => write!(f, "{}", name),

            Type::Any => write!(f, "Any"),
            Type::Unknown => write!(f, "Unknown"),
            Type::Literal(lit) => match lit {
                LiteralValue::Int(n) => write!(f, "Literal[{}]", n),
                LiteralValue::Float(n) => write!(f, "Literal[{}]", n),
                LiteralValue::Str(s) => write!(f, "Literal[\"{}\"]", s),
                LiteralValue::Bool(b) => write!(f, "Literal[{}]", b),
                LiteralValue::None => write!(f, "Literal[None]"),
            },
            Type::SelfType { class_name } => {
                if let Some(name) = class_name {
                    write!(f, "Self[{}]", name)
                } else {
                    write!(f, "Self")
                }
            }
            Type::LiteralString => write!(f, "LiteralString"),
            Type::Final(inner) => write!(f, "Final[{}]", inner),
            Type::Annotated { inner, metadata } => {
                write!(f, "Annotated[{}", inner)?;
                for m in metadata {
                    write!(f, ", {}", m)?;
                }
                write!(f, "]")
            }
            Type::Overloaded { signatures } => {
                write!(f, "Overloaded[")?;
                for (i, sig) in signatures.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", sig)?;
                }
                write!(f, "]")
            }
            Type::ParamSpec { name, .. } => write!(f, "ParamSpec[{}]", name),
            Type::Concatenate { params, param_spec } => {
                write!(f, "Concatenate[")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                if !params.is_empty() {
                    write!(f, ", ")?;
                }
                write!(f, "{}]", param_spec)
            }
            Type::TypeVarTuple { name, .. } => write!(f, "TypeVarTuple[{}]", name),
            Type::Unpack(inner) => write!(f, "*{}", inner),
            Type::TypeGuard(inner) => write!(f, "TypeGuard[{}]", inner),
            Type::TypeIs(inner) => write!(f, "TypeIs[{}]", inner),

            Type::Error => write!(f, "<error>"),
        }
    }
}

#[cfg(test)]
#[path = "ty_tests.rs"]
mod tests;
