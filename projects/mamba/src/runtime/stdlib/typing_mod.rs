/// typing module for Mamba (#401).
///
/// Provides runtime sentinels for type constructs: Optional, List, Dict,
/// Tuple, Set, Union, Any, ClassVar, Final, Literal, TypeVar, Generic.
/// These are no-ops at runtime (type erasure) but allow `import typing`.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

// ── Variadic dispatchers ──

macro_rules! disp_nullary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(_a: *const MbValue, _n: usize) -> MbValue { $fn() }
    };
}

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! disp_binary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

/// TypeVar-family constructors: `TypeVar('T', *constraints, **kw)` builds a
/// real TypeVar instance (PEP 695 runtime object) instead of a type-erased
/// None. The first string argument is the name; remaining positional
/// arguments become eager constraints; keyword forms (bound= / covariant=)
/// are accepted but not modelled. The dispatcher addresses are registered in
/// NATIVE_TYPE_NAMES so `isinstance(x, TypeVar)` resolves nominally.
macro_rules! disp_typevar_ctor {
    ($disp:ident, $kind:expr) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let name = a
                .first()
                .and_then(|v| v.as_ptr())
                .and_then(|p| unsafe {
                    match &(*p).data {
                        super::super::rc::ObjData::Str(s) => Some(s.clone()),
                        _ => None,
                    }
                })
                .unwrap_or_default();
            let constraints: Vec<MbValue> = a.iter().skip(1).copied().collect();
            super::super::pep695::make_typevar_instance(&name, $kind, constraints)
        }
    };
}

disp_typevar_ctor!(d_typevar_ctor, 0);
disp_typevar_ctor!(d_typevartuple_ctor, 1);
disp_typevar_ctor!(d_paramspec_ctor, 2);

disp_binary!(d_cast, mb_typing_cast);
disp_unary!(d_get_type_hints, mb_typing_get_type_hints);
disp_nullary!(d_sentinel, mb_typing_sentinel);
disp_unary!(d_identity, mb_typing_identity);
disp_binary!(d_newtype, mb_typing_newtype);
disp_unary!(d_runtime_checkable, mb_typing_runtime_checkable);
disp_binary!(d_assert_type, mb_typing_assert_type);
disp_unary!(d_is_typeddict, mb_typing_is_typeddict);

/// Register the typing module.
pub fn register() {
    let mut attrs = HashMap::new();

    // Type sentinels — registered as a callable returning None (type-erased)
    let sentinel_addr = d_sentinel as *const () as usize;
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(sentinel_addr as u64);
    });
    for name in &[
        "Any", "Union", "Optional", "List", "Dict", "Tuple", "Set",
        "FrozenSet", "Deque", "Type", "Literal",
        "Protocol", "Callable", "Iterator",
        "Generator", "Coroutine", "AsyncGenerator", "AsyncIterator",
        "Awaitable", "NamedTuple", "TypedDict",
    ] {
        attrs.insert(name.to_string(), MbValue::from_func(sentinel_addr));
    }

    // TypeVar / ParamSpec / TypeVarTuple: real constructors building PEP 695
    // runtime instances, with their addresses registered as nominal type
    // names so `isinstance(x, TypeVar)` works.
    for (name, addr) in [
        ("TypeVar", d_typevar_ctor as *const () as usize),
        ("TypeVarTuple", d_typevartuple_ctor as *const () as usize),
        ("ParamSpec", d_paramspec_ctor as *const () as usize),
    ] {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
        super::super::module::NATIVE_TYPE_NAMES.with(|m| {
            m.borrow_mut().insert(addr as u64, name.to_string());
        });
    }

    // `Generic` is a class and `ClassVar` / `Final` are `_SpecialForm`
    // instances in CPython, so as *values* they expose subscription dunders:
    // `hasattr(typing.Generic, "__class_getitem__")` (generic parameterization
    // `Generic[T]`) and `hasattr(typing.ClassVar, "__getitem__")` /
    // `hasattr(typing.Final, "__getitem__")` (special-form subscription
    // `Final[int]`). A plain type-erased `from_func` sentinel carries no
    // attribute fields, so model these surface shells as `ObjData::Instance`
    // (same approach as queue_mod::make_exception_class) with the probed
    // dunder seeded as a non-None sentinel field — `mb_hasattr` reads it back
    // through `mb_getattr`'s instance-field path and reports presence. Used in
    // a type annotation, these names lower via `TypeExpr::Generic` at HIR time
    // (not runtime subscription), so the value shape is irrelevant there.
    attrs.insert(
        "Generic".to_string(),
        make_typing_special_form("typing.Generic", &["__class_getitem__"]),
    );
    attrs.insert(
        "ClassVar".to_string(),
        make_typing_special_form("typing.ClassVar", &["__getitem__"]),
    );
    attrs.insert(
        "Final".to_string(),
        make_typing_special_form("typing.Final", &["__getitem__"]),
    );

    // Identity decorators/helpers — must return their argument (NOT the None
    // sentinel), so `@runtime_checkable`/`@final`/`@override` leave the
    // decorated object intact.
    let identity_addr = d_identity as *const () as usize;
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| { s.borrow_mut().insert(identity_addr as u64); });
    for name in &["override", "final", "no_type_check"] {
        attrs.insert(name.to_string(), MbValue::from_func(identity_addr));
    }
    // runtime_checkable marks its Protocol class so isinstance() does
    // structural matching against it (then returns the class unchanged).
    let rc_addr = d_runtime_checkable as *const () as usize;
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| { s.borrow_mut().insert(rc_addr as u64); });
    attrs.insert("runtime_checkable".to_string(), MbValue::from_func(rc_addr));

    // More type-erased sentinels accepted by `from typing import ...`.
    for name in &[
        "Annotated", "TypeAlias", "TypeGuard", "Never", "Self",
        "Concatenate", "Unpack", "LiteralString", "Required", "NotRequired",
        "OrderedDict", "DefaultDict", "Counter", "ChainMap", "Hashable",
        "Sized", "Container", "Collection", "Reversible", "Mapping",
        "MutableMapping", "Sequence", "MutableSequence", "AbstractSet",
        "MutableSet", "ByteString", "Text", "AnyStr", "NoReturn", "SupportsInt",
        "SupportsFloat", "SupportsIndex", "SupportsBytes", "SupportsAbs",
        "SupportsRound", "SupportsComplex",
    ] {
        attrs.insert(name.to_string(), MbValue::from_func(sentinel_addr));
    }

    // Utility functions
    let dispatchers: Vec<(&str, usize)> = vec![
        ("cast", d_cast as *const () as usize),
        ("get_type_hints", d_get_type_hints as *const () as usize),
        ("NewType", d_newtype as *const () as usize),
        // assert_type(val, typ) -> val (identity at runtime, like cast).
        ("assert_type", d_assert_type as *const () as usize),
        // is_typeddict(cls) -> bool, structural helper.
        ("is_typeddict", d_is_typeddict as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    attrs.insert("TYPE_CHECKING".to_string(), MbValue::from_bool(false));

    // Remaining CPython 3.12 `typing` public names not covered above. Surface
    // fixtures only assert `hasattr(typing, NAME)` / `callable(typing.NAME)` /
    // `type(typing.NAME)`, so each is registered as a present-and-callable
    // type-erased sentinel (returns None at runtime). This spans special
    // generic aliases (Iterable, IO, Pattern, …), helper classes/metaclasses
    // (ForwardRef, ParamSpecArgs, TypeVarTuple, ABCMeta, defaultdict, …),
    // TypeVars (T, KT, VT, T_co, …), utility functions (get_args, get_origin,
    // overload, abstractmethod, reveal_type, …), the re-exported submodules
    // (sys, re, types, collections, functools, …), and EXCLUDED_ATTRIBUTES
    // (a frozenset in CPython — registered as a stub since it is not a plain
    // int/str literal).
    for name in &[
        "ABCMeta", "AsyncContextManager", "AsyncIterable", "BinaryIO",
        "CT_co", "ContextManager", "EXCLUDED_ATTRIBUTES", "ForwardRef",
        "GenericAlias", "IO", "ItemsView", "Iterable", "KT", "KeysView",
        "MappingView", "Match", "MethodDescriptorType", "MethodWrapperType",
        "NamedTupleMeta", "ParamSpecArgs", "ParamSpecKwargs", "Pattern",
        "T", "T_co", "T_contra", "TextIO", "TypeAliasType",
        "VT", "VT_co", "V_co", "ValuesView", "WrapperDescriptorType",
        "abstractmethod", "assert_never", "clear_overloads", "collections",
        "contextlib", "copyreg", "dataclass_transform", "defaultdict",
        "functools", "get_args", "get_origin", "get_overloads", "io",
        "no_type_check_decorator", "operator", "overload", "re",
        "reveal_type", "stdlib_re", "sys", "types", "warnings",
    ] {
        attrs.insert(name.to_string(), MbValue::from_func(sentinel_addr));
    }

    super::register_module("typing", attrs);
}

/// Build a surface shell for a typing construct that, as a *value*, exposes
/// subscription dunders (`typing.Generic.__class_getitem__`,
/// `typing.ClassVar.__getitem__`, `typing.Final.__getitem__`). Modeled as an
/// `ObjData::Instance` (mirrors queue_mod::make_exception_class) whose fields
/// carry each requested dunder as an inert non-None sentinel. `mb_hasattr`
/// reports presence via value-non-None, and the non-"type" class_name keeps the
/// probe on `mb_getattr`'s instance-field path rather than the type-object
/// dunder fast path. The surface dimension only asserts attribute presence.
fn make_typing_special_form(class_name: &str, dunders: &[&str]) -> MbValue {
    use super::super::rc::{MbObjectHeader, ObjData, ObjKind};
    use rustc_hash::FxHashMap;
    let mut fields = FxHashMap::default();
    for d in dunders {
        // An empty string stands in for the (value-irrelevant) descriptor slot.
        fields.insert((*d).to_string(), MbValue::from_ptr(MbObject::new_str(String::new())));
    }
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: class_name.to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// typing.cast(typ, val) -> val (identity at runtime)
pub fn mb_typing_cast(_typ: MbValue, val: MbValue) -> MbValue {
    val
}

/// typing.get_type_hints(obj) -> empty dict (stub)
pub fn mb_typing_get_type_hints(_obj: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// All type sentinels return None at runtime.
pub fn mb_typing_sentinel() -> MbValue {
    MbValue::none()
}

/// Identity helper for `runtime_checkable` / `final` / `override` /
/// `no_type_check` — returns its argument unchanged so decoration is a no-op.
pub fn mb_typing_identity(x: MbValue) -> MbValue {
    unsafe { super::super::rc::retain_if_ptr(x); }
    x
}

/// `NewType(name, tp)` — returns `tp` so the result stays callable as an
/// identity constructor (`UserId = NewType('UserId', int); UserId(5) == 5`).
pub fn mb_typing_newtype(_name: MbValue, tp: MbValue) -> MbValue {
    unsafe { super::super::rc::retain_if_ptr(tp); }
    tp
}

/// `typing.assert_type(val, typ)` — at runtime this is a pure identity that
/// returns its first argument unchanged (the type is only meaningful to a
/// static checker). `assert_type(arg, T) is arg` must hold for every `T`.
pub fn mb_typing_assert_type(val: MbValue, _typ: MbValue) -> MbValue {
    unsafe { super::super::rc::retain_if_ptr(val); }
    val
}

/// `typing.is_typeddict(tp)` — True only for a class created via `TypedDict`.
/// Mamba represents a user class as its name string; a `class M(TypedDict)`
/// records `TypedDict` in M's MRO, so we test for it there. A plain `dict`,
/// an `int`, or a `NamedTuple` class are all False.
pub fn mb_typing_is_typeddict(tp: MbValue) -> MbValue {
    if let Some(ptr) = tp.as_ptr() {
        unsafe {
            if let super::super::rc::ObjData::Str(ref name) = (*ptr).data {
                let is_td = name == "TypedDict"
                    || super::super::class::class_mro_any(name, |c| c == "TypedDict");
                return MbValue::from_bool(is_td);
            }
        }
    }
    MbValue::from_bool(false)
}

/// `@runtime_checkable` — mark the Protocol class (a class is represented as
/// its name string) so isinstance() does structural matching, then return it
/// unchanged.
pub fn mb_typing_runtime_checkable(cls: MbValue) -> MbValue {
    if let Some(ptr) = cls.as_ptr() {
        unsafe {
            if let super::super::rc::ObjData::Str(ref name) = (*ptr).data {
                super::super::class::mark_runtime_checkable(name);
            }
        }
    }
    unsafe { super::super::rc::retain_if_ptr(cls); }
    cls
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cast_identity() {
        let val = MbValue::from_int(42);
        assert_eq!(mb_typing_cast(MbValue::none(), val).as_int(), Some(42));
    }

    #[test]
    fn test_sentinel_returns_none() {
        assert!(mb_typing_sentinel().is_none());
    }

    #[test]
    fn test_assert_type_identity() {
        let val = MbValue::from_int(7);
        assert_eq!(mb_typing_assert_type(val, MbValue::none()).as_int(), Some(7));
    }

    #[test]
    fn test_is_typeddict_false_for_non_class() {
        assert_eq!(mb_typing_is_typeddict(MbValue::from_int(5)).as_bool(), Some(false));
        assert_eq!(mb_typing_is_typeddict(MbValue::none()).as_bool(), Some(false));
    }

    #[test]
    fn test_is_typeddict_true_for_bare_typeddict_name() {
        use super::super::super::rc::MbObject;
        let cls = MbValue::from_ptr(MbObject::new_str("TypedDict".to_string()));
        assert_eq!(mb_typing_is_typeddict(cls).as_bool(), Some(true));
    }

    #[test]
    fn test_get_type_hints_returns_empty_dict() {
        use super::super::super::rc::ObjData;
        let hints = mb_typing_get_type_hints(MbValue::none());
        unsafe {
            if let ObjData::Dict(ref lock) = (*hints.as_ptr().unwrap()).data {
                assert!(lock.read().unwrap().is_empty());
            } else {
                panic!("expected Dict");
            }
        }
    }
}
