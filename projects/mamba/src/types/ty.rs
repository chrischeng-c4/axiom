/// Unique identifier for a type in the TypeContext.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

/// Unique identifier for a type variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeVarId(pub u32);

/// Literal value for `Literal[42]` / `Literal["a", "b"]` types (#243).
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Int(i64),
    Str(String),
    Bool(bool),
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
    Union(Vec<TypeId>),
    Fn {
        params: Vec<TypeId>,
        ret: TypeId,
        variadic: bool,
    },
    /// `match_args: None` = no explicit `__match_args__`; callers fall back to field order.
    /// `match_args: Some(names)` = explicit (even `Some(vec![])` means no positional matching).
    Class {
        name: String,
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
        };
        assert!(!f.is_numeric());
        assert_eq!(
            f,
            Ty::Fn {
                params: vec![TypeId(3)],
                ret: TypeId(5),
                variadic: false
            }
        );
        assert_ne!(
            f,
            Ty::Fn {
                params: vec![TypeId(3)],
                ret: TypeId(3),
                variadic: false
            }
        );
    }

    #[test]
    fn test_ty_class() {
        let c = Ty::Class {
            name: "Foo".to_string(),
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
