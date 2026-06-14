//! Tests for type definitions

use super::*;

#[test]
fn test_type_display() {
    assert_eq!(Type::Int.to_string(), "int");
    assert_eq!(Type::optional(Type::Str).to_string(), "str | None");
    assert_eq!(Type::list(Type::Int).to_string(), "list[int]");
    assert_eq!(
        Type::dict(Type::Str, Type::Int).to_string(),
        "dict[str, int]"
    );
}

#[test]
fn test_union_flattening() {
    let union = Type::union(vec![
        Type::Int,
        Type::Union(vec![Type::Str, Type::Float]),
        Type::Int, // duplicate
    ]);

    match union {
        Type::Union(types) => {
            assert_eq!(types.len(), 3);
            assert!(types.contains(&Type::Int));
            assert!(types.contains(&Type::Str));
            assert!(types.contains(&Type::Float));
        }
        _ => panic!("Expected Union"),
    }
}

#[test]
fn test_without_none() {
    let optional = Type::optional(Type::Str);
    assert_eq!(optional.without_none(), Type::Str);

    let union = Type::Union(vec![Type::Int, Type::None, Type::Str]);
    let without = union.without_none();
    match without {
        Type::Union(types) => {
            assert_eq!(types.len(), 2);
            assert!(!types.contains(&Type::None));
        }
        _ => panic!("Expected Union"),
    }
}

#[test]
fn test_type_var_substitution() {
    use std::collections::HashMap;

    // Create a generic type: List[T]
    let t = Type::type_var(0, "T");
    let list_t = Type::list(t);

    // Substitute T -> Int
    let mut subs = HashMap::new();
    subs.insert(TypeVarId(0), Type::Int);

    let result = list_t.substitute(&subs);
    assert_eq!(result, Type::list(Type::Int));
}

#[test]
fn test_type_vars_collection() {
    // Create Dict[K, V]
    let k = Type::type_var(0, "K");
    let v = Type::type_var(1, "V");
    let dict_kv = Type::dict(k, v);

    let vars = dict_kv.type_vars();
    assert_eq!(vars.len(), 2);
    assert!(vars.contains(&TypeVarId(0)));
    assert!(vars.contains(&TypeVarId(1)));
}

#[test]
fn test_nested_substitution() {
    use std::collections::HashMap;

    // Create Optional[List[T]]
    let t = Type::type_var(0, "T");
    let list_t = Type::list(t);
    let optional_list_t = Type::optional(list_t);

    // Substitute T -> Str
    let mut subs = HashMap::new();
    subs.insert(TypeVarId(0), Type::Str);

    let result = optional_list_t.substitute(&subs);
    assert_eq!(result, Type::optional(Type::list(Type::Str)));
}

#[test]
fn test_callable_substitution() {
    use std::collections::HashMap;

    // Create Callable[[T], T]
    let t = Type::type_var(0, "T");
    let callable = Type::Callable {
        params: vec![Param {
            name: "x".to_string(),
            ty: t.clone(),
            has_default: false,
            kind: ParamKind::Positional,
        }],
        ret: Box::new(t),
    };

    // Substitute T -> Int
    let mut subs = HashMap::new();
    subs.insert(TypeVarId(0), Type::Int);

    let result = callable.substitute(&subs);
    match result {
        Type::Callable { params, ret } => {
            assert_eq!(params[0].ty, Type::Int);
            assert_eq!(*ret, Type::Int);
        }
        _ => panic!("Expected Callable"),
    }
}

#[test]
fn test_unify_type_var() {
    use std::collections::HashMap;

    // Unify T with int -> T=int
    let t = Type::type_var(0, "T");
    let mut subs = HashMap::new();

    assert!(t.unify(&Type::Int, &mut subs));
    assert_eq!(subs.get(&TypeVarId(0)), Some(&Type::Int));
}

#[test]
fn test_unify_list() {
    use std::collections::HashMap;

    // Unify list[T] with list[str] -> T=str
    let t = Type::type_var(0, "T");
    let list_t = Type::list(t);
    let mut subs = HashMap::new();

    assert!(list_t.unify(&Type::list(Type::Str), &mut subs));
    assert_eq!(subs.get(&TypeVarId(0)), Some(&Type::Str));
}

#[test]
fn test_unify_dict() {
    use std::collections::HashMap;

    // Unify dict[K, V] with dict[str, int] -> K=str, V=int
    let k = Type::type_var(0, "K");
    let v = Type::type_var(1, "V");
    let dict_kv = Type::dict(k, v);
    let mut subs = HashMap::new();

    assert!(dict_kv.unify(&Type::dict(Type::Str, Type::Int), &mut subs));
    assert_eq!(subs.get(&TypeVarId(0)), Some(&Type::Str));
    assert_eq!(subs.get(&TypeVarId(1)), Some(&Type::Int));
}

#[test]
fn test_unify_consistency() {
    use std::collections::HashMap;

    // Unify T with int, then T with int again -> ok
    let t = Type::type_var(0, "T");
    let mut subs = HashMap::new();

    assert!(t.unify(&Type::Int, &mut subs));
    assert!(t.unify(&Type::Int, &mut subs)); // Same binding, ok

    // Unify T with str after binding to int -> fail
    assert!(!t.unify(&Type::Str, &mut subs));
}

#[test]
fn test_protocol_display() {
    let protocol = Type::Protocol {
        name: "Sized".to_string(),
        module: Some("typing".to_string()),
        members: vec![("__len__".to_string(), Type::callable(vec![], Type::Int))],
    };
    assert_eq!(protocol.to_string(), "Protocol[Sized]{__len__}");

    // Empty protocol
    let empty_proto = Type::Protocol {
        name: "Empty".to_string(),
        module: None,
        members: vec![],
    };
    assert_eq!(empty_proto.to_string(), "Protocol[Empty]");
}

#[test]
fn test_literal_display() {
    assert_eq!(
        Type::Literal(LiteralValue::Int(42)).to_string(),
        "Literal[42]"
    );
    assert_eq!(
        Type::Literal(LiteralValue::Str("hello".to_string())).to_string(),
        "Literal[\"hello\"]"
    );
    assert_eq!(
        Type::Literal(LiteralValue::Bool(true)).to_string(),
        "Literal[true]"
    );
    assert_eq!(
        Type::Literal(LiteralValue::None).to_string(),
        "Literal[None]"
    );
}

#[test]
fn test_typed_dict_display() {
    let td = Type::TypedDict {
        name: "Person".to_string(),
        fields: vec![
            ("name".to_string(), Type::Str, true),
            ("age".to_string(), Type::Int, true),
            ("email".to_string(), Type::Str, false), // optional
        ],
        total: true,
    };
    assert_eq!(
        td.to_string(),
        "TypedDict[Person]{name: str, age: int, email?: str}"
    );

    // Empty TypedDict
    let empty_td = Type::TypedDict {
        name: "Empty".to_string(),
        fields: vec![],
        total: true,
    };
    assert_eq!(empty_td.to_string(), "TypedDict[Empty]");
}

// === Phase 1 Type System Tests ===

#[test]
fn test_final_display() {
    let final_int = Type::final_type(Type::Int);
    assert_eq!(final_int.to_string(), "Final[int]");

    let final_str = Type::final_type(Type::Str);
    assert_eq!(final_str.to_string(), "Final[str]");

    // Nested
    let final_list = Type::final_type(Type::list(Type::Int));
    assert_eq!(final_list.to_string(), "Final[list[int]]");
}

#[test]
fn test_annotated_display() {
    let annotated = Type::annotated(Type::Int, vec!["Positive".to_string()]);
    assert_eq!(annotated.to_string(), "Annotated[int, Positive]");

    let multi = Type::annotated(
        Type::Str,
        vec!["MaxLen(100)".to_string(), "NotEmpty".to_string()],
    );
    assert_eq!(multi.to_string(), "Annotated[str, MaxLen(100), NotEmpty]");
}

#[test]
fn test_literal_string_display() {
    assert_eq!(Type::LiteralString.to_string(), "LiteralString");
}

#[test]
fn test_self_type_display() {
    let unresolved = Type::self_type(None);
    assert_eq!(unresolved.to_string(), "Self");

    let resolved = Type::self_type(Some("MyClass".to_string()));
    assert_eq!(resolved.to_string(), "Self[MyClass]");
}

#[test]
fn test_overloaded_display() {
    let overloaded = Type::overloaded(vec![
        Type::callable(vec![Type::Int], Type::Int),
        Type::callable(vec![Type::Str], Type::Str),
    ]);
    assert_eq!(
        overloaded.to_string(),
        "Overloaded[(int) -> int, (str) -> str]"
    );
}

#[test]
fn test_param_spec_display() {
    let p = Type::param_spec(0, "P");
    assert_eq!(p.to_string(), "ParamSpec[P]");
}

#[test]
fn test_type_var_tuple_display() {
    let ts = Type::type_var_tuple(0, "Ts");
    assert_eq!(ts.to_string(), "TypeVarTuple[Ts]");

    let unpacked = Type::Unpack(Box::new(ts));
    assert_eq!(unpacked.to_string(), "*TypeVarTuple[Ts]");
}

#[test]
fn test_concatenate_display() {
    let p = Type::param_spec(0, "P");
    let concat = Type::concatenate(vec![Type::Int, Type::Str], p);
    assert_eq!(concat.to_string(), "Concatenate[int, str, ParamSpec[P]]");
}

#[test]
fn test_final_unwrap() {
    let final_int = Type::final_type(Type::Int);
    assert_eq!(final_int.unwrap_final(), &Type::Int);

    // Non-Final returns self
    assert_eq!(Type::Str.unwrap_final(), &Type::Str);
}

#[test]
fn test_annotated_unwrap() {
    let annotated = Type::annotated(Type::Int, vec!["Test".to_string()]);
    assert_eq!(annotated.unwrap_annotated(), &Type::Int);

    // Non-Annotated returns self
    assert_eq!(Type::Str.unwrap_annotated(), &Type::Str);
}

#[test]
fn test_is_final() {
    assert!(Type::final_type(Type::Int).is_final());
    assert!(!Type::Int.is_final());
}

#[test]
fn test_is_literal_string() {
    assert!(Type::LiteralString.is_literal_string());
    assert!(Type::Literal(LiteralValue::Str("hello".to_string())).is_literal_string());
    assert!(!Type::Str.is_literal_string());
}

// === Phase B Type Tests (PEP 647, 742) ===

#[test]
fn test_typeguard_display() {
    let tg = Type::type_guard(Type::Int);
    assert_eq!(tg.to_string(), "TypeGuard[int]");

    let tg_str = Type::type_guard(Type::Str);
    assert_eq!(tg_str.to_string(), "TypeGuard[str]");

    let tg_nested = Type::type_guard(Type::list(Type::Int));
    assert_eq!(tg_nested.to_string(), "TypeGuard[list[int]]");
}

#[test]
fn test_typeis_display() {
    let ti = Type::type_is(Type::Int);
    assert_eq!(ti.to_string(), "TypeIs[int]");

    let ti_str = Type::type_is(Type::Str);
    assert_eq!(ti_str.to_string(), "TypeIs[str]");
}

#[test]
fn test_typeguard_helpers() {
    let tg = Type::type_guard(Type::Int);
    assert!(tg.is_type_guard());
    assert!(!tg.is_type_is());
    assert_eq!(tg.get_guard_type(), Some(&Type::Int));

    // Non-TypeGuard returns false/None
    assert!(!Type::Int.is_type_guard());
    assert_eq!(Type::Int.get_guard_type(), None);
}

#[test]
fn test_typeis_helpers() {
    let ti = Type::type_is(Type::Str);
    assert!(ti.is_type_is());
    assert!(!ti.is_type_guard());
    assert_eq!(ti.get_guard_type(), Some(&Type::Str));

    // Non-TypeIs returns false
    assert!(!Type::Str.is_type_is());
}
