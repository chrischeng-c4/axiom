use crate::resolve::SymbolId;

/// Unique identifier for a type in the TypeContext.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

/// Unique identifier for a type variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVarId(pub u32);

/// Stable identity for one specialized PEP 695 type-alias expansion.
///
/// Alias references are indirections used only for recursive back-edges. An
/// ordinary alias use still resolves to its expanded semantic head.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AliasInstanceId(pub u32);

/// Static kind of a PEP 695 type parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeVarKind {
    TypeVar,
    TypeVarTuple,
    ParamSpec,
}

/// Static resolution state for a PEP 696 type-parameter default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeParamDefault {
    None,
    Unresolved,
    Resolved(TypeId),
}

impl TypeParamDefault {
    pub fn is_present(self) -> bool {
        !matches!(self, Self::None)
    }

    pub fn resolved(self) -> Option<TypeId> {
        match self {
            Self::Resolved(ty) => Some(ty),
            Self::None | Self::Unresolved => None,
        }
    }
}

/// Whether a class type denotes the class object or an instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClassRole {
    Object,
    Instance,
}

/// Nominal identity and concrete arguments for a user-defined class.
#[derive(Debug, Clone, PartialEq)]
pub struct UserClass {
    pub symbol: SymbolId,
    pub args: Vec<TypeId>,
}

/// Module-qualified identity and type arguments for an external class.
#[derive(Debug, Clone, PartialEq)]
pub struct ExternalClass {
    pub module: String,
    pub name: String,
    pub args: Vec<TypeId>,
}

/// How a generated stdlib callable is bound at the point of access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalCallableAccess {
    Module,
    ClassMember,
    BoundMember,
}

/// Runtime nominal class of an external callable when independently proven.
/// Typeshed describes call signatures but does not distinguish Python from
/// builtin implementations, so generated callables default to `Unknown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalCallableRuntimeKind {
    Unknown,
    PythonFunction,
    PythonMethod,
    BuiltinFunction,
    BuiltinMethod,
    WrapperDescriptor,
    MethodWrapper,
    MethodDescriptor,
    ClassMethodDescriptor,
}

/// Canonical generated-contract identity for a callable value.
#[derive(Debug, Clone, PartialEq)]
pub struct ExternalCallable {
    /// Module that owns the callable declaration. For inherited methods this
    /// can differ from the receiver's module.
    pub module: String,
    pub qualifier: String,
    pub name: String,
    pub access: ExternalCallableAccess,
    pub runtime_kind: ExternalCallableRuntimeKind,
    /// Canonical class whose attribute produced this callable. This preserves
    /// `Self` and nominal return identity across aliases and method chains.
    pub receiver: Option<ExternalClass>,
}

/// External Python values whose generated identity must survive ordinary
/// expression flow instead of living only in symbol side tables.
#[derive(Debug, Clone, PartialEq)]
pub enum ExternalValue {
    Module {
        path: String,
        /// Modules explicitly loaded by the originating import statement.
        /// `import a.b` binds `a`, but only the loaded `a.b` child may be
        /// traversed as a module without another import.
        loaded: Vec<String>,
    },
    Callable(ExternalCallable),
}

/// Literal value for `Literal[42]` / `Literal["a", "b"]` types (#243).
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Int(i64),
    Str(String),
    Bool(bool),
}

/// Python callable parameter category retained by the semantic type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallableParamKind {
    PosOnly,
    PosOrKw,
    VarPos,
    KwOnly,
    VarKw,
}

/// One parameter in a concrete Python callable signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallableParam {
    pub name: Option<String>,
    pub ty: TypeId,
    pub kind: CallableParamKind,
    pub has_default: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamPackTail {
    Closed,
    Ellipsis,
    ParamSpec(TypeVarId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamPack {
    pub params: Vec<CallableParam>,
    pub tail: ParamPackTail,
}

/// Ordered static members bound to one `TypeVarTuple`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypePack {
    pub types: Vec<TypeId>,
}

/// Core type representation for Mamba.
#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Never,
    None,
    Bool,
    Int,   // i64
    Float, // f64
    Str,
    /// Dynamic type — compatible with all types (#240).
    Any,
    List(TypeId),
    Set(TypeId),
    Dict(TypeId, TypeId),
    Tuple(Vec<TypeId>),
    /// A static `TypeVarTuple` expansion. This is only valid inside a
    /// pack-aware container or callable parameter list.
    Unpack(TypeVarId),
    Union(Vec<TypeId>),
    Fn {
        params: Vec<TypeId>,
        ret: TypeId,
        variadic: bool,
        /// Full semantic call shape when it is known. The compact `params`
        /// vector remains the ABI-facing positional prefix.
        signature: Option<Vec<CallableParam>>,
        /// Correlated PEP 612 parameter-list tail for Callable[P, R] and
        /// Callable[Concatenate[prefix..., P], R].
        param_spec: Option<TypeVarId>,
    },
    External(ExternalValue),
    /// A boxed Python class object whose constructor produces `TypeId`.
    TypeObject(TypeId),
    /// `match_args: None` = no explicit `__match_args__`; callers fall back to field order.
    /// `match_args: Some(names)` = explicit (even `Some(vec![])` means no positional matching).
    Class {
        name: String,
        role: ClassRole,
        user: Option<UserClass>,
        external: Option<ExternalClass>,
        fields: Vec<(String, TypeId)>,
        match_args: Option<Vec<String>>,
    },
    Enum {
        name: String,
        variants: Vec<(String, Vec<TypeId>)>,
    },
    /// Type variable with optional bound and constraints (#242).
    TypeVar(TypeVarId),
    /// Literal type: `Literal[42]`, `Literal["a", "b"]` (#243).
    Literal(Vec<LiteralValue>),
    /// `Self` type in class methods (#243).
    SelfType,
    /// Recursive edge into a lazily materialized PEP 695 type alias.
    AliasRef(AliasInstanceId),
    Infer(u32),
    Error,
}

impl Ty {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Ty::Int | Ty::Float | Ty::Bool)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Ty::Error)
    }

    pub fn is_any(&self) -> bool {
        matches!(self, Ty::Any)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_id_equality() {
        let a = TypeId(0);
        let b = TypeId(0);
        let c = TypeId(1);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_type_id_copy() {
        let a = TypeId(42);
        let b = a; // Copy
        assert_eq!(a, b);
    }

    #[test]
    fn test_type_var_id_equality() {
        let a = TypeVarId(0);
        let b = TypeVarId(0);
        let c = TypeVarId(1);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_literal_value_variants() {
        let int_lit = LiteralValue::Int(42);
        let str_lit = LiteralValue::Str("hello".to_string());
        let bool_lit = LiteralValue::Bool(true);

        assert_eq!(int_lit, LiteralValue::Int(42));
        assert_ne!(int_lit, LiteralValue::Int(0));
        assert_eq!(str_lit, LiteralValue::Str("hello".to_string()));
        assert_ne!(str_lit, LiteralValue::Str("world".to_string()));
        assert_eq!(bool_lit, LiteralValue::Bool(true));
        assert_ne!(bool_lit, LiteralValue::Bool(false));
    }

    #[test]
    fn test_ty_is_numeric() {
        assert!(Ty::Int.is_numeric());
        assert!(Ty::Float.is_numeric());
        assert!(!Ty::Str.is_numeric());
        assert!(Ty::Bool.is_numeric()); // Bool is subtype of Int
        assert!(!Ty::None.is_numeric());
        assert!(!Ty::Any.is_numeric());
        assert!(!Ty::Never.is_numeric());
        assert!(!Ty::Error.is_numeric());
    }

    #[test]
    fn test_ty_is_error() {
        assert!(Ty::Error.is_error());
        assert!(!Ty::Int.is_error());
        assert!(!Ty::Any.is_error());
        assert!(!Ty::None.is_error());
    }

    #[test]
    fn test_ty_is_any() {
        assert!(Ty::Any.is_any());
        assert!(!Ty::Int.is_any());
        assert!(!Ty::Error.is_any());
        assert!(!Ty::None.is_any());
    }

    #[test]
    fn test_ty_equality() {
        assert_eq!(Ty::Int, Ty::Int);
        assert_eq!(Ty::None, Ty::None);
        assert_ne!(Ty::Int, Ty::Float);
        assert_ne!(Ty::Bool, Ty::Int);

        // Compound types
        let list_a = Ty::List(TypeId(1));
        let list_b = Ty::List(TypeId(1));
        let list_c = Ty::List(TypeId(2));
        assert_eq!(list_a, list_b);
        assert_ne!(list_a, list_c);
    }

    #[test]
    fn test_ty_clone() {
        let original = Ty::Tuple(vec![TypeId(1), TypeId(2)]);
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_ty_dict() {
        let d = Ty::Dict(TypeId(3), TypeId(5));
        assert!(!d.is_numeric());
        assert!(!d.is_error());
        assert!(!d.is_any());
    }

    #[test]
    fn test_ty_fn() {
        let f = Ty::Fn {
            params: vec![TypeId(3)],
            ret: TypeId(5),
            variadic: false,
            signature: None,
            param_spec: None,
        };
        assert!(!f.is_numeric());
        assert_eq!(
            f,
            Ty::Fn {
                params: vec![TypeId(3)],
                ret: TypeId(5),
                variadic: false,
                signature: None,
                param_spec: None,
            }
        );
        assert_ne!(
            f,
            Ty::Fn {
                params: vec![TypeId(3)],
                ret: TypeId(3),
                variadic: false,
                signature: None,
                param_spec: None,
            }
        );
    }

    #[test]
    fn test_ty_class() {
        let c = Ty::Class {
            name: "Foo".to_string(),
            role: ClassRole::Instance,
            user: None,
            external: None,
            fields: vec![("x".to_string(), TypeId(3))],
            match_args: None,
        };
        assert!(!c.is_numeric());
        assert!(!c.is_error());
    }

    #[test]
    fn test_ty_enum() {
        let e = Ty::Enum {
            name: "Color".to_string(),
            variants: vec![("Red".to_string(), vec![])],
        };
        assert!(!e.is_numeric());
    }

    #[test]
    fn test_ty_union() {
        let u = Ty::Union(vec![TypeId(3), TypeId(5)]);
        assert!(!u.is_numeric());
        assert_eq!(u, Ty::Union(vec![TypeId(3), TypeId(5)]));
    }

    #[test]
    fn test_ty_literal() {
        let lit = Ty::Literal(vec![LiteralValue::Int(1), LiteralValue::Int(2)]);
        assert!(!lit.is_numeric());
    }

    #[test]
    fn test_ty_self_type() {
        assert!(!Ty::SelfType.is_numeric());
        assert!(!Ty::SelfType.is_error());
        assert!(!Ty::SelfType.is_any());
    }

    #[test]
    fn test_ty_infer() {
        let infer = Ty::Infer(0);
        assert!(!infer.is_numeric());
        assert_eq!(infer, Ty::Infer(0));
        assert_ne!(infer, Ty::Infer(1));
    }

    #[test]
    fn test_type_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(TypeId(0));
        set.insert(TypeId(1));
        set.insert(TypeId(0)); // duplicate
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_type_var_id_hash() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(TypeVarId(0), "T");
        map.insert(TypeVarId(1), "U");
        assert_eq!(map.get(&TypeVarId(0)), Some(&"T"));
        assert_eq!(map.get(&TypeVarId(1)), Some(&"U"));
        assert_eq!(map.get(&TypeVarId(2)), None);
    }
}
