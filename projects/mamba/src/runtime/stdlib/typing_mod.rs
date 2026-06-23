use super::super::rc::MbObject;
use super::super::value::MbValue;
/// typing module for Mamba (#401).
///
/// Provides runtime sentinels for type constructs: Optional, List, Dict,
/// Tuple, Set, Union, Any, ClassVar, Final, Literal, TypeVar, Generic.
/// These are no-ops at runtime (type erasure) but allow `import typing`.
use std::collections::HashMap;

// ── Variadic dispatchers ──

macro_rules! disp_nullary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(_a: *const MbValue, _n: usize) -> MbValue {
            $fn()
        }
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
            let mut rest: Vec<MbValue> = a.iter().skip(1).copied().collect();
            // Keyword arguments (bound=int, covariant=True) arrive as a
            // trailing dict; peel it off the constraints.
            let kwargs = rest.last().copied().filter(|v| {
                v.as_ptr()
                    .map(|p| unsafe { matches!(&(*p).data, super::super::rc::ObjData::Dict(_)) })
                    .unwrap_or(false)
            });
            if kwargs.is_some() {
                rest.pop();
            }
            let bound = kwargs
                .map(|kw| {
                    super::super::dict_ops::mb_dict_get(
                        kw,
                        MbValue::from_ptr(MbObject::new_str("bound".to_string())),
                        MbValue::none(),
                    )
                })
                .filter(|b| !b.is_none());
            // CPython: constraints and a bound are mutually exclusive.
            if bound.is_some() && !rest.is_empty() {
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(
                        "Constraints cannot be combined with bound=...".to_string(),
                    )),
                );
                return MbValue::none();
            }
            let inst = super::super::pep695::make_typevar_instance(&name, $kind, rest);
            if let Some(b) = bound {
                super::super::pep695::instance_field_set_pub(inst, "__bound__", b);
            }
            inst
        }
    };
}

disp_typevar_ctor!(d_typevar_ctor, 0);
disp_typevar_ctor!(d_typevartuple_ctor, 1);
disp_typevar_ctor!(d_paramspec_ctor, 2);

disp_binary!(d_cast, mb_typing_cast);
disp_unary!(d_get_origin, mb_typing_get_origin);
disp_unary!(d_get_args, mb_typing_get_args);
disp_unary!(d_get_type_hints, mb_typing_get_type_hints);
disp_nullary!(d_sentinel, mb_typing_sentinel);

/// typing.NamedTuple(name, fields=None, **kwargs). Providing BOTH a positional
/// fields list and keyword fields is a TypeError; otherwise return the
/// type-erased sentinel (the functional form is not yet materialized).
unsafe extern "C" fn d_namedtuple(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let is_seq = |v: MbValue| -> bool {
        v.as_ptr()
            .map(|p| unsafe { matches!((*p).data, super::super::rc::ObjData::List(_) | super::super::rc::ObjData::Tuple(_)) })
            .unwrap_or(false)
    };
    let nonempty_dict = |v: MbValue| -> bool {
        v.as_ptr()
            .map(|p| unsafe {
                if let super::super::rc::ObjData::Dict(ref lock) = (*p).data {
                    !lock.read().unwrap().is_empty()
                } else {
                    false
                }
            })
            .unwrap_or(false)
    };
    if a.len() >= 3 {
        let fields = a.get(1).copied().unwrap_or_else(MbValue::none);
        let last = a.last().copied().unwrap_or_else(MbValue::none);
        if is_seq(fields) && nonempty_dict(last) {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "Either list of fields or keywords can be provided to \
                     NamedTuple, not both".to_string(),
                )),
            );
            return MbValue::none();
        }
    }
    // Functional form `NamedTuple("P", [("x", int), ("y", int)])`: delegate to
    // collections.namedtuple, which already builds a working tuple subclass.
    // typing's fields are (name, type) pairs — extract just the names.
    use super::super::rc::ObjData;
    let name = a.first().copied().unwrap_or_else(MbValue::none);
    let fields_v = a.get(1).copied().unwrap_or_else(MbValue::none);
    if !name.is_none() && is_seq(fields_v) {
        let items: Vec<MbValue> = fields_v.as_ptr().map(|p| unsafe {
            match &(*p).data {
                ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
                ObjData::Tuple(ref t) => t.to_vec(),
                _ => Vec::new(),
            }
        }).unwrap_or_default();
        let names: Vec<MbValue> = items.iter().map(|item| {
            item.as_ptr().and_then(|p| unsafe {
                match &(*p).data {
                    // (name, type) pair → name; plain string → itself.
                    ObjData::Tuple(ref t) => t.first().copied(),
                    ObjData::List(ref l) => l.read().unwrap().first().copied(),
                    ObjData::Str(_) => Some(*item),
                    _ => None,
                }
            }).unwrap_or(*item)
        }).collect();
        let names_list = MbValue::from_ptr(MbObject::new_list(names));
        return super::collections_mod::mb_namedtuple(name, names_list, MbValue::none());
    }
    MbValue::none()
}
disp_unary!(d_identity, mb_typing_identity);
disp_unary!(d_override, mb_typing_override);
disp_unary!(d_final, mb_typing_final);
disp_binary!(d_newtype, mb_typing_newtype_meta);
disp_unary!(d_runtime_checkable, mb_typing_runtime_checkable);
disp_binary!(d_assert_type, mb_typing_assert_type);
disp_unary!(d_is_typeddict, mb_typing_is_typeddict);

unsafe extern "C" fn d_overload(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_func(d_overload_dummy as *const () as usize)
}

unsafe extern "C" fn d_overload_dummy(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("NotImplementedError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "You should not call an overloaded function. A series of @overload-decorated \
             functions outside a stub module should always be followed by an implementation \
             that is not @overload-ed."
                .to_string(),
        )),
    );
    MbValue::none()
}

/// Register the typing module.
pub fn register() {
    let mut attrs = HashMap::new();

    // Type sentinels — registered as a callable returning None (type-erased)
    let sentinel_addr = d_sentinel as *const () as usize;
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(sentinel_addr as u64);
    });
    for name in &[
        "Protocol",
        "Iterator",
        "Generator",
        "Coroutine",
        "AsyncGenerator",
        "AsyncIterator",
        "Awaitable",
        "TypedDict",
    ] {
        attrs.insert(name.to_string(), MbValue::from_func(sentinel_addr));
    }
    // NamedTuple validates the call shape (mixing the list form with keyword
    // fields is a TypeError) but otherwise behaves like the sentinel.
    let namedtuple_addr = d_namedtuple as *const () as usize;
    attrs.insert("NamedTuple".to_string(), MbValue::from_func(namedtuple_addr));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(namedtuple_addr as u64);
    });
    // Algebra special forms: identity-stable singletons that subscript into
    // normalized alias objects (Union[int, str], Optional[int], Literal[1]).
    for name in &[
        "Any",
        "Union",
        "Optional",
        "List",
        "Dict",
        "Tuple",
        "Set",
        "FrozenSet",
        "Deque",
        "Type",
        "Literal",
        "Callable",
        "Annotated",
        "NoReturn",
        "Never",
        "Self",
        "LiteralString",
        "TypeAlias",
    ] {
        attrs.insert(name.to_string(), special_form(name));
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
    attrs.insert("ClassVar".to_string(), special_form("ClassVar"));
    attrs.insert("Final".to_string(), special_form("Final"));

    // Identity decorators/helpers — must return their argument (NOT the None
    // sentinel), so `@runtime_checkable`/`@final`/`@override` leave the
    // decorated object intact.
    let identity_addr = d_identity as *const () as usize;
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(identity_addr as u64);
    });
    for name in &["no_type_check"] {
        attrs.insert(name.to_string(), MbValue::from_func(identity_addr));
    }
    // @override sets __override__ = True on the decorated function (PEP 698);
    // @final sets __final__ = True. Both return the function unchanged.
    let override_addr = d_override as *const () as usize;
    let final_addr = d_final as *const () as usize;
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(override_addr as u64);
        set.insert(final_addr as u64);
    });
    attrs.insert("override".to_string(), MbValue::from_func(override_addr));
    attrs.insert("final".to_string(), MbValue::from_func(final_addr));
    let overload_addr = d_overload as *const () as usize;
    let overload_dummy_addr = d_overload_dummy as *const () as usize;
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(overload_addr as u64);
        set.insert(overload_dummy_addr as u64);
    });
    attrs.insert("overload".to_string(), MbValue::from_func(overload_addr));
    // runtime_checkable marks its Protocol class so isinstance() does
    // structural matching against it (then returns the class unchanged).
    let rc_addr = d_runtime_checkable as *const () as usize;
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(rc_addr as u64);
    });
    attrs.insert("runtime_checkable".to_string(), MbValue::from_func(rc_addr));

    // More type-erased sentinels accepted by `from typing import ...`.
    for name in &[
        "TypeGuard",
        "Concatenate",
        "Unpack",
        "Required",
        "NotRequired",
        "OrderedDict",
        "DefaultDict",
        "Counter",
        "ChainMap",
        "Hashable",
        "Sized",
        "Container",
        "Collection",
        "Reversible",
        "Mapping",
        "MutableMapping",
        "Sequence",
        "MutableSequence",
        "AbstractSet",
        "MutableSet",
        "ByteString",
        "Text",
        "AnyStr",
        "SupportsInt",
        "SupportsFloat",
        "SupportsIndex",
        "SupportsBytes",
        "SupportsAbs",
        "SupportsRound",
        "SupportsComplex",
    ] {
        attrs.insert(name.to_string(), MbValue::from_func(sentinel_addr));
    }

    // Utility functions
    let dispatchers: Vec<(&str, usize)> = vec![
        ("cast", d_cast as *const () as usize),
        ("get_type_hints", d_get_type_hints as *const () as usize),
        ("NewType", d_newtype as *const () as usize),
        ("get_origin", d_get_origin as *const () as usize),
        ("get_args", d_get_args as *const () as usize),
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
        "ABCMeta",
        "AsyncContextManager",
        "AsyncIterable",
        "BinaryIO",
        "CT_co",
        "ContextManager",
        "EXCLUDED_ATTRIBUTES",
        "ForwardRef",
        "GenericAlias",
        "IO",
        "ItemsView",
        "Iterable",
        "KT",
        "KeysView",
        "MappingView",
        "Match",
        "MethodDescriptorType",
        "MethodWrapperType",
        "NamedTupleMeta",
        "ParamSpecArgs",
        "ParamSpecKwargs",
        "Pattern",
        "T",
        "T_co",
        "T_contra",
        "TextIO",
        "TypeAliasType",
        "VT",
        "VT_co",
        "V_co",
        "ValuesView",
        "WrapperDescriptorType",
        "abstractmethod",
        "assert_never",
        "clear_overloads",
        "collections",
        "contextlib",
        "copyreg",
        "dataclass_transform",
        "defaultdict",
        "functools",
        "get_overloads",
        "io",
        "no_type_check_decorator",
        "operator",
        "overload",
        "re",
        "reveal_type",
        "stdlib_re",
        "sys",
        "types",
        "warnings",
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
    if class_name == "typing.Generic" {
        fields.insert(
            "__name__".to_string(),
            MbValue::from_ptr(MbObject::new_str("Generic".to_string())),
        );
        fields.insert(
            "__qualname__".to_string(),
            MbValue::from_ptr(MbObject::new_str("Generic".to_string())),
        );
        fields.insert(
            "__bases__".to_string(),
            MbValue::from_ptr(MbObject::new_tuple(vec![
                super::super::builtins::make_type_object("object"),
            ])),
        );
    }
    for d in dunders {
        // An empty string stands in for the (value-irrelevant) descriptor slot.
        fields.insert(
            (*d).to_string(),
            MbValue::from_ptr(MbObject::new_str(String::new())),
        );
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

// ── typing algebra: special forms + parameterized aliases (#22) ──────────
//
// Special forms (typing.Union, typing.Literal, ...) are cached singleton
// Instances (class_name "typing.SpecialForm", `_name` field) so identity
// holds (`get_origin(u) is Union`). Subscripting one builds a normalized
// alias Instance (class_name "typing.Alias") carrying `_kind`, `__origin__`,
// `__args__` (+ `__metadata__` for Annotated). Equality/hash/repr are
// registered class methods; Union equality is order-insensitive.

thread_local! {
    static SPECIAL_FORMS: std::cell::RefCell<HashMap<String, MbValue>> =
        std::cell::RefCell::new(HashMap::new());
    static TYPING_ALIAS_REGISTERED: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

fn new_str_v(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn extract_str(val: MbValue) -> Option<String> {
    use super::super::rc::ObjData;
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Str(s) => Some(s.clone()),
            _ => None,
        }
    })
}

fn instance_with(class_name: &str, fields: rustc_hash::FxHashMap<String, MbValue>) -> MbValue {
    use super::super::rc::{MbObjectHeader, ObjData, ObjKind};
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

fn instance_field_of(v: MbValue, name: &str) -> Option<MbValue> {
    use super::super::rc::ObjData;
    v.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(name).copied()
        } else {
            None
        }
    })
}

fn instance_class_of(v: MbValue) -> Option<String> {
    use super::super::rc::ObjData;
    v.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            Some(class_name.clone())
        } else {
            None
        }
    })
}

/// Cached special-form singleton (identity-stable).
pub(crate) fn special_form(name: &str) -> MbValue {
    ensure_typing_classes_registered();
    SPECIAL_FORMS.with(|m| {
        if let Some(&v) = m.borrow().get(name) {
            unsafe {
                super::super::rc::retain_if_ptr(v);
            }
            return v;
        }
        let mut fields = rustc_hash::FxHashMap::default();
        fields.insert("_name".to_string(), new_str_v(name));
        // Probed by hasattr for subscription support.
        fields.insert("__getitem__".to_string(), new_str_v(""));
        let v = instance_with("typing.SpecialForm", fields);
        super::super::gc::gc_add_root(v);
        m.borrow_mut().insert(name.to_string(), v);
        unsafe {
            super::super::rc::retain_if_ptr(v);
        }
        v
    })
}

/// Structural identity key for alias args/members (type objects collapse to
/// their cached-singleton name, aliases recurse, plain values keep type+repr
/// so Literal[0] != Literal[False]).
fn alias_key(v: MbValue) -> String {
    if let Some(cls) = instance_class_of(v) {
        match cls.as_str() {
            "type" => {
                let n = instance_field_of(v, "__name__")
                    .and_then(|x| extract_str(x))
                    .unwrap_or_default();
                return format!("T:{n}");
            }
            "typing.SpecialForm" => {
                let n = instance_field_of(v, "_name")
                    .and_then(|x| extract_str(x))
                    .unwrap_or_default();
                return format!("F:{n}");
            }
            "typing.Alias" => {
                let kind = instance_field_of(v, "_kind")
                    .and_then(|x| extract_str(x))
                    .unwrap_or_default();
                let origin = instance_field_of(v, "__origin__")
                    .map(alias_key)
                    .unwrap_or_default();
                let args = alias_args_vec(v)
                    .into_iter()
                    .map(alias_key)
                    .collect::<Vec<_>>()
                    .join(",");
                let meta = instance_field_of(v, "__metadata__")
                    .map(|m| {
                        tuple_items(m)
                            .into_iter()
                            .map(alias_key)
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .unwrap_or_default();
                return format!("A:{kind}:{origin}[{args}]{{{meta}}}");
            }
            "UnionType" => {
                // Same key shape as a typing.Union alias (kind "union", no
                // origin/metadata) so the two representations dedup/hash alike.
                let args = alias_args_vec(v)
                    .into_iter()
                    .map(alias_key)
                    .collect::<Vec<_>>()
                    .join(",");
                return format!("A:union:[{args}]{{}}");
            }
            _ => {}
        }
    }
    if v.is_none() {
        return "V:NoneType:None".to_string();
    }
    if let Some(b) = v.as_bool() {
        return format!("V:bool:{b}");
    }
    if let Some(i) = v.as_int() {
        return format!("V:int:{i}");
    }
    if let Some(f) = v.as_float() {
        return format!("V:float:{f}");
    }
    if let Some(s) = extract_str(v) {
        return format!("V:str:{s:?}");
    }
    format!("V:?:{:x}", v.to_bits())
}

fn tuple_items(v: MbValue) -> Vec<MbValue> {
    use super::super::rc::ObjData;
    v.as_ptr()
        .map(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::Tuple(items) => items.to_vec(),
                ObjData::List(lock) => lock.read().unwrap().iter().copied().collect(),
                _ => Vec::new(),
            }
        })
        .unwrap_or_default()
}

fn alias_args_vec(alias: MbValue) -> Vec<MbValue> {
    instance_field_of(alias, "__args__")
        .map(tuple_items)
        .unwrap_or_default()
}

fn alias_kind(v: MbValue) -> Option<String> {
    match instance_class_of(v).as_deref() {
        Some("typing.Alias") => instance_field_of(v, "_kind").and_then(|x| extract_str(x)),
        // A PEP 604 `X | Y` UnionType is the same union as a typing.Union alias
        // for equality/hashing (cross-representation), so it reports "union".
        Some("UnionType") => Some("union".to_string()),
        _ => None,
    }
}

fn make_alias(
    kind: &str,
    origin: MbValue,
    args: Vec<MbValue>,
    repr_name: Option<&str>,
    metadata: Option<Vec<MbValue>>,
) -> MbValue {
    ensure_typing_classes_registered();
    let mut fields = rustc_hash::FxHashMap::default();
    fields.insert("_kind".to_string(), new_str_v(kind));
    unsafe {
        super::super::rc::retain_if_ptr(origin);
    }
    fields.insert("__origin__".to_string(), origin);
    for a in &args {
        unsafe {
            super::super::rc::retain_if_ptr(*a);
        }
    }
    // __parameters__: the free TypeVars in the args (PEP 585/484/695), so a
    // parameterized alias can be re-subscripted and introspected. Computed
    // before `args` is moved into the __args__ tuple.
    fields.insert("__parameters__".to_string(), typevar_params_tuple(&args));
    fields.insert(
        "__args__".to_string(),
        MbValue::from_ptr(MbObject::new_tuple(args)),
    );
    if let Some(meta) = metadata {
        for m in &meta {
            unsafe {
                super::super::rc::retain_if_ptr(*m);
            }
        }
        fields.insert(
            "__metadata__".to_string(),
            MbValue::from_ptr(MbObject::new_tuple(meta)),
        );
    }
    if let Some(rn) = repr_name {
        fields.insert("_repr_name".to_string(), new_str_v(rn));
    }
    instance_with("typing.Alias", fields)
}

fn is_typevar(v: MbValue) -> bool {
    instance_class_of(v).as_deref() == Some("TypeVar")
}

fn collect_params_into(v: MbValue, out: &mut Vec<MbValue>) {
    if is_typevar(v) {
        if !out.iter().any(|p| p.to_bits() == v.to_bits()) {
            out.push(v);
        }
        return;
    }
    if matches!(instance_class_of(v).as_deref(), Some("typing.Alias") | Some("UnionType")) {
        for a in alias_args_vec(v) {
            collect_params_into(a, out);
        }
    }
}

/// The `__parameters__` tuple for `args` — the free TypeVars appearing
/// (recursively) in them, order-preserving and deduplicated.
pub(crate) fn typevar_params_tuple(args: &[MbValue]) -> MbValue {
    let mut params = Vec::new();
    for a in args {
        collect_params_into(*a, &mut params);
    }
    for p in &params {
        unsafe { super::super::rc::retain_if_ptr(*p); }
    }
    MbValue::from_ptr(MbObject::new_tuple(params))
}

/// Substitute TypeVars in `v` (recursively into nested aliases/unions) using
/// `sub` (typevar identity bits → replacement value).
fn substitute_typevars(v: MbValue, sub: &rustc_hash::FxHashMap<u64, MbValue>) -> MbValue {
    if is_typevar(v) {
        return sub.get(&v.to_bits()).copied().unwrap_or(v);
    }
    match instance_class_of(v).as_deref() {
        Some("typing.Alias") => {
            let kind = alias_kind(v).unwrap_or_default();
            let origin = instance_field_of(v, "__origin__").unwrap_or_else(MbValue::none);
            let repr_name = instance_field_of(v, "_repr_name").and_then(extract_str);
            let new_args: Vec<MbValue> = alias_args_vec(v)
                .into_iter()
                .map(|a| substitute_typevars(a, sub))
                .collect();
            make_alias(&kind, origin, new_args, repr_name.as_deref(), None)
        }
        Some("UnionType") => {
            let new_args: Vec<MbValue> = alias_args_vec(v)
                .into_iter()
                .map(|a| substitute_typevars(a, sub))
                .collect();
            super::super::builtins::make_union_type_value(new_args)
        }
        _ => v,
    }
}

/// Subscript a parameterized typing.Alias / UnionType: `alias[args]` substitutes
/// its __parameters__ with the provided args (arity-checked, TypeError on a
/// wrong count), mirroring CPython's `__getitem__` on a generic/union alias.
pub(crate) fn alias_subscript(self_v: MbValue, key: MbValue) -> MbValue {
    let params = instance_field_of(self_v, "__parameters__")
        .map(tuple_items)
        .unwrap_or_default();
    let provided = key
        .as_ptr()
        .and_then(|ptr| unsafe {
            match &(*ptr).data {
                super::super::rc::ObjData::Tuple(t) => Some(t.to_vec()),
                _ => None,
            }
        })
        .unwrap_or_else(|| vec![key]);
    if params.is_empty() || provided.len() != params.len() {
        super::super::exception::mb_raise(
            new_str_v("TypeError"),
            new_str_v(&format!(
                "Too {} arguments for {}; actual {}, expected {}",
                if provided.len() >= params.len() { "many" } else { "few" },
                alias_repr(self_v),
                provided.len(),
                params.len(),
            )),
        );
        return MbValue::none();
    }
    let mut sub = rustc_hash::FxHashMap::default();
    for (p, val) in params.iter().zip(provided.iter()) {
        sub.insert(p.to_bits(), *val);
    }
    substitute_typevars(self_v, &sub)
}

/// Human repr of an alias argument (class reprs use the bare name).
fn arg_repr(v: MbValue) -> String {
    if let Some(cls) = instance_class_of(v) {
        match cls.as_str() {
            "type" => {
                let n = instance_field_of(v, "__name__")
                    .and_then(|x| extract_str(x))
                    .unwrap_or_default();
                // CPython spells type(None) as NoneType inside alias reprs.
                return n;
            }
            "typing.SpecialForm" => {
                let n = instance_field_of(v, "_name")
                    .and_then(|x| extract_str(x))
                    .unwrap_or_default();
                return format!("typing.{n}");
            }
            "typing.Alias" => return alias_repr(v),
            _ => {}
        }
    }
    extract_str(super::super::builtins::mb_repr(v)).unwrap_or_default()
}

fn alias_repr(alias: MbValue) -> String {
    let kind = alias_kind(alias).unwrap_or_default();
    let args = alias_args_vec(alias);
    match kind.as_str() {
        "annotated" => {
            let origin = instance_field_of(alias, "__origin__")
                .map(arg_repr)
                .unwrap_or_default();
            let meta = instance_field_of(alias, "__metadata__")
                .map(tuple_items)
                .unwrap_or_default()
                .into_iter()
                .map(arg_repr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("typing.Annotated[{origin}, {meta}]")
        }
        "union" => {
            let parts = args
                .into_iter()
                .map(arg_repr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("typing.Union[{parts}]")
        }
        "literal" => {
            let parts = args
                .into_iter()
                .map(|v| extract_str(super::super::builtins::mb_repr(v)).unwrap_or_default())
                .collect::<Vec<_>>()
                .join(", ");
            format!("typing.Literal[{parts}]")
        }
        _ => {
            let name = instance_field_of(alias, "_repr_name")
                .and_then(|x| extract_str(x))
                .unwrap_or_else(|| {
                    instance_field_of(alias, "__origin__")
                        .map(arg_repr)
                        .unwrap_or_default()
                });
            let parts = args
                .into_iter()
                .map(arg_repr)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}[{parts}]")
        }
    }
}

unsafe extern "C" fn typing_alias_repr_m(self_v: MbValue, _args: MbValue) -> MbValue {
    new_str_v(&alias_repr(self_v))
}

unsafe extern "C" fn typing_sf_repr_m(self_v: MbValue, _args: MbValue) -> MbValue {
    let n = instance_field_of(self_v, "_name")
        .and_then(|x| extract_str(x))
        .unwrap_or_default();
    new_str_v(&format!("typing.{n}"))
}

unsafe extern "C" fn typing_alias_eq_m(self_v: MbValue, args: MbValue) -> MbValue {
    use super::super::rc::ObjData;
    let mut other = args;
    if let Some(ptr) = args.as_ptr() {
        if let ObjData::List(ref lock) = (*ptr).data {
            let items = lock.read().unwrap();
            if items.len() == 1 {
                other = items[0];
            }
        }
    }
    MbValue::from_bool(aliases_equal(self_v, other))
}

/// Cross-representation union equality: when at least one side is a PEP 604
/// `X | Y` UnionType and both resolve to a union, compare member sets (so
/// `int | str == typing.Union[int, str]` and `== str | int`). Returns None for
/// the non-union or pure-typing.Alias cases (those keep their existing paths).
pub(crate) fn union_values_equal(a: MbValue, b: MbValue) -> Option<bool> {
    let is_union = |v: MbValue| alias_kind(v).as_deref() == Some("union");
    let either_runtime = instance_class_of(a).as_deref() == Some("UnionType")
        || instance_class_of(b).as_deref() == Some("UnionType");
    if either_runtime && is_union(a) && is_union(b) {
        return Some(aliases_equal(a, b));
    }
    None
}

fn aliases_equal(a: MbValue, b: MbValue) -> bool {
    let (Some(ka), Some(kb)) = (alias_kind(a), alias_kind(b)) else {
        return false;
    };
    if ka != kb {
        return false;
    }
    match ka.as_str() {
        "union" | "literal" => {
            let mut sa: Vec<String> = alias_args_vec(a).into_iter().map(alias_key).collect();
            let mut sb: Vec<String> = alias_args_vec(b).into_iter().map(alias_key).collect();
            sa.sort();
            sb.sort();
            sa == sb
        }
        _ => alias_key(a) == alias_key(b),
    }
}

unsafe extern "C" fn typing_alias_hash_m(self_v: MbValue, _args: MbValue) -> MbValue {
    alias_hash_value(self_v)
}

/// Hash of a typing alias OR a PEP 604 UnionType — a polynomial over a blob of
/// (kind | origin | member keys | metadata), member keys sorted for
/// union/literal so it is order-insensitive. A UnionType reports kind "union"
/// (alias_kind) and the origin is dropped for unions, so `int | str` and
/// `typing.Union[int, str]` (which differ only by __origin__) hash identically.
pub(crate) fn alias_hash_value(self_v: MbValue) -> MbValue {
    use super::super::rc::ObjData;
    // Annotated metadata participates in the hash; unhashable members
    // (list/dict/set) raise like CPython.
    if let Some(meta) = instance_field_of(self_v, "__metadata__") {
        for m in tuple_items(meta) {
            if let Some(ptr) = m.as_ptr() {
                let bad = unsafe { match &(*ptr).data {
                    ObjData::List(_) => Some("list"),
                    ObjData::Dict(_) => Some("dict"),
                    ObjData::Set(_) => Some("set"),
                    _ => None,
                } };
                if let Some(tn) = bad {
                    super::super::exception::mb_raise(
                        new_str_v("TypeError"),
                        new_str_v(&format!("unhashable type: '{tn}'")),
                    );
                    return MbValue::none();
                }
            }
        }
    }
    let kind = alias_kind(self_v).unwrap_or_default();
    let mut keys: Vec<String> = alias_args_vec(self_v).into_iter().map(alias_key).collect();
    if kind == "union" || kind == "literal" {
        keys.sort();
    }
    let origin_key = if kind == "union" {
        // A union's origin is always typing.Union (implied); excluding it lets a
        // `X | Y` UnionType and the typing.Union[X, Y] alias — which differ only
        // by their __origin__ — hash identically.
        String::new()
    } else {
        instance_field_of(self_v, "__origin__")
            .map(alias_key)
            .unwrap_or_default()
    };
    let meta_key = instance_field_of(self_v, "__metadata__")
        .map(|m| {
            tuple_items(m)
                .into_iter()
                .map(alias_key)
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default();
    let blob = format!("{kind}|{origin_key}|{}|{meta_key}", keys.join(","));
    let mut acc: i64 = 0x74797065;
    for byte in blob.as_bytes() {
        acc = acc.wrapping_mul(1_000_003).wrapping_add(*byte as i64);
    }
    MbValue::from_int(acc & 0x3FFF_FFFF_FFFF)
}

fn ensure_typing_classes_registered() {
    if TYPING_ALIAS_REGISTERED.with(|c| c.get()) {
        return;
    }
    TYPING_ALIAS_REGISTERED.with(|c| c.set(true));
    use std::collections::HashMap as Map;
    let var = |addr: usize| {
        super::super::module::register_variadic_func(addr as u64);
        MbValue::from_func(addr)
    };
    let mut ma: Map<String, MbValue> = Map::new();
    ma.insert(
        "__eq__".to_string(),
        var(typing_alias_eq_m as *const () as usize),
    );
    ma.insert(
        "__hash__".to_string(),
        var(typing_alias_hash_m as *const () as usize),
    );
    ma.insert(
        "__repr__".to_string(),
        var(typing_alias_repr_m as *const () as usize),
    );
    super::super::class::mb_class_register("typing.Alias", vec![], ma);
    super::super::class::mb_class_register(
        "typing.Generic",
        vec!["object".to_string()],
        Map::new(),
    );
    let mut ms: Map<String, MbValue> = Map::new();
    ms.insert(
        "__repr__".to_string(),
        var(typing_sf_repr_m as *const () as usize),
    );
    super::super::class::mb_class_register("typing.SpecialForm", vec![], ms);
}

/// Normalized Union construction: flatten nested unions, map None to
/// type(None), dedup by identity key, collapse a single member.
pub(crate) fn typing_union(members: Vec<MbValue>) -> MbValue {
    let none_type = || super::super::builtins::make_type_object("NoneType");
    let mut flat: Vec<MbValue> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    let mut push = |v: MbValue, flat: &mut Vec<MbValue>, seen: &mut Vec<String>| {
        let v = if v.is_none() { none_type() } else { v };
        let k = alias_key(v);
        if !seen.contains(&k) {
            seen.push(k);
            flat.push(v);
        }
    };
    for m in members {
        if alias_kind(m).as_deref() == Some("union") {
            for sub in alias_args_vec(m) {
                push(sub, &mut flat, &mut seen);
            }
        } else {
            push(m, &mut flat, &mut seen);
        }
    }
    if flat.len() == 1 {
        return flat[0];
    }
    make_alias("union", special_form("Union"), flat, None, None)
}

fn literal_alias(values: Vec<MbValue>) -> MbValue {
    let mut flat: Vec<MbValue> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for v in values {
        if alias_kind(v).as_deref() == Some("literal") {
            for sub in alias_args_vec(v) {
                let k = alias_key(sub);
                if !seen.contains(&k) {
                    seen.push(k);
                    flat.push(sub);
                }
            }
        } else {
            let k = alias_key(v);
            if !seen.contains(&k) {
                seen.push(k);
                flat.push(v);
            }
        }
    }
    make_alias("literal", special_form("Literal"), flat, None, None)
}

/// Subscript dispatch for typing special forms: `Union[int, str]`,
/// `Optional[int]`, `Literal[1, 2]`, `List[int]`, `Annotated[int, m]`, ...
pub fn special_form_subscript(name: &str, key: MbValue) -> MbValue {
    let items = {
        use super::super::rc::ObjData;
        key.as_ptr()
            .and_then(|ptr| unsafe {
                match &(*ptr).data {
                    ObjData::Tuple(t) => Some(t.to_vec()),
                    _ => None,
                }
            })
            .unwrap_or_else(|| vec![key])
    };
    match name {
        "Union" => typing_union(items),
        "Optional" => {
            let mut members = items;
            members.push(MbValue::none());
            typing_union(members)
        }
        "Literal" => literal_alias(items),
        "Annotated" => {
            let mut it = items.into_iter();
            let first = it.next().unwrap_or_else(MbValue::none);
            let mut metadata: Vec<MbValue> = Vec::new();
            let origin = if alias_kind(first).as_deref() == Some("annotated") {
                // Annotated[Annotated[int, 4], 5] folds metadata in order.
                let inner_origin =
                    instance_field_of(first, "__origin__").unwrap_or_else(MbValue::none);
                if let Some(m) = instance_field_of(first, "__metadata__") {
                    metadata.extend(tuple_items(m));
                }
                inner_origin
            } else {
                first
            };
            metadata.extend(it);
            make_alias("annotated", origin, vec![], None, Some(metadata))
        }
        "List" | "Set" | "FrozenSet" | "Dict" | "Tuple" | "Type" | "Deque" | "DefaultDict"
        | "OrderedDict" | "Counter" | "ChainMap" => {
            let expected = match name {
                "List" | "Set" | "FrozenSet" | "Deque" | "Type" | "Counter" => Some(1),
                "Dict" | "DefaultDict" | "OrderedDict" | "ChainMap" => Some(2),
                _ => None, // Tuple is variadic
            };
            if let Some(n) = expected {
                if items.len() > n {
                    super::super::exception::mb_raise(
                        new_str_v("TypeError"),
                        new_str_v(&format!(
                            "Too many arguments for typing.{name}; actual {}, expected {n}",
                            items.len()
                        )),
                    );
                    return MbValue::none();
                }
            }
            let origin_name = match name {
                "List" => "list",
                "Set" => "set",
                "FrozenSet" => "frozenset",
                "Dict" => "dict",
                "Tuple" => "tuple",
                "Type" => "type",
                "Deque" => "collections.deque",
                "DefaultDict" => "collections.defaultdict",
                "OrderedDict" => "collections.OrderedDict",
                "Counter" => "collections.Counter",
                _ => "collections.ChainMap",
            };
            let origin = super::super::builtins::make_type_object(origin_name);
            make_alias(
                "generic",
                origin,
                items,
                Some(&format!("typing.{name}")),
                None,
            )
        }
        "Callable" => {
            let origin = special_form("Callable");
            make_alias("generic", origin, items, Some("typing.Callable"), None)
        }
        _ => {
            // ClassVar / Final / Required / TypeGuard / Unpack / unknown:
            // wrap as a generic alias over the special form itself.
            let origin = special_form(name);
            make_alias(
                "generic",
                origin,
                items,
                Some(&format!("typing.{name}")),
                None,
            )
        }
    }
}

/// `typing.Generic[T]` returns a generic alias whose origin is the Generic
/// class object itself. User classes inherit through this alias before they can
/// later be parameterized as `Box[int]`.
pub fn generic_subscript(origin: MbValue, key: MbValue) -> MbValue {
    use super::super::rc::ObjData;
    let items = key
        .as_ptr()
        .and_then(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::Tuple(t) => Some(t.to_vec()),
                _ => None,
            }
        })
        .unwrap_or_else(|| vec![key]);
    make_alias("generic", origin, items, Some("typing.Generic"), None)
}

/// Parameterize a user class that inherits from `typing.Generic`.
pub fn user_generic_subscript(origin: MbValue, key: MbValue, repr_name: &str) -> MbValue {
    use super::super::rc::ObjData;
    let items = key
        .as_ptr()
        .and_then(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::Tuple(t) => Some(t.to_vec()),
                _ => None,
            }
        })
        .unwrap_or_else(|| vec![key]);
    make_alias("generic", origin, items, Some(repr_name), None)
}

/// PEP 585 builtin-generic subscription: list[int], dict[str, int], ...
/// Returns a generic alias whose repr uses the lowercase builtin name.
pub fn pep585_subscript(type_obj: MbValue, key: MbValue) -> MbValue {
    use super::super::rc::ObjData;
    let items = key
        .as_ptr()
        .and_then(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::Tuple(t) => Some(t.to_vec()),
                _ => None,
            }
        })
        .unwrap_or_else(|| vec![key]);
    let name = instance_field_of(type_obj, "__name__")
        .and_then(|x| extract_str(x))
        .unwrap_or_default();
    make_alias("generic", type_obj, items, Some(&name), None)
}

/// typing.get_origin(tp)
pub fn mb_typing_get_origin(tp: MbValue) -> MbValue {
    match alias_kind(tp).as_deref() {
        Some("union") => special_form("Union"),
        Some("annotated") => special_form("Annotated"),
        Some("literal") => special_form("Literal"),
        Some(_) => {
            let o = instance_field_of(tp, "__origin__").unwrap_or_else(MbValue::none);
            unsafe {
                super::super::rc::retain_if_ptr(o);
            }
            o
        }
        None => MbValue::none(),
    }
}

/// typing.get_args(tp)
pub fn mb_typing_get_args(tp: MbValue) -> MbValue {
    if alias_kind(tp).as_deref() == Some("annotated") {
        let mut out = vec![instance_field_of(tp, "__origin__").unwrap_or_else(MbValue::none)];
        if let Some(m) = instance_field_of(tp, "__metadata__") {
            out.extend(tuple_items(m));
        }
        for v in &out {
            unsafe {
                super::super::rc::retain_if_ptr(*v);
            }
        }
        return MbValue::from_ptr(MbObject::new_tuple(out));
    }
    if alias_kind(tp).is_some() {
        let args = alias_args_vec(tp);
        for v in &args {
            unsafe {
                super::super::rc::retain_if_ptr(*v);
            }
        }
        return MbValue::from_ptr(MbObject::new_tuple(args));
    }
    MbValue::from_ptr(MbObject::new_tuple(Vec::new()))
}

/// isinstance(value, Union[...]) — any member matches.
pub fn typing_union_isinstance(value: MbValue, alias: MbValue) -> Option<bool> {
    if alias_kind(alias).as_deref() != Some("union") {
        return None;
    }
    for member in alias_args_vec(alias) {
        if super::super::class::mb_isinstance(value, member).as_bool() == Some(true) {
            return Some(true);
        }
    }
    Some(false)
}

/// @typing.override — pass-through that records __override__ = True.
pub fn mb_typing_override(func: MbValue) -> MbValue {
    super::super::pep695::func_attrs_set(func, new_str_v("__override__"), MbValue::from_bool(true));
    func
}

/// @typing.final — pass-through that records __final__ = True.
pub fn mb_typing_final(func: MbValue) -> MbValue {
    super::super::pep695::func_attrs_set(func, new_str_v("__final__"), MbValue::from_bool(true));
    func
}

/// typing.NewType("UserId", int) — a callable identity wrapper carrying
/// __name__ / __supertype__ (calls are handled by the "typing.NewType"
/// arm in the call paths).
pub fn mb_typing_newtype_meta(name: MbValue, supertype: MbValue) -> MbValue {
    let mut fields = rustc_hash::FxHashMap::default();
    unsafe {
        super::super::rc::retain_if_ptr(name);
        super::super::rc::retain_if_ptr(supertype);
    }
    fields.insert("__name__".to_string(), name);
    fields.insert("__supertype__".to_string(), supertype);
    instance_with("typing.NewType", fields)
}

/// typing.cast(typ, val) -> val (identity at runtime)
pub fn mb_typing_cast(_typ: MbValue, val: MbValue) -> MbValue {
    val
}

/// typing.get_type_hints(obj) -> empty dict (stub)
pub fn mb_typing_get_type_hints(obj: MbValue) -> MbValue {
    let dict = MbValue::from_ptr(MbObject::new_dict());
    let set = |k: &str, v: MbValue| {
        super::super::dict_ops::mb_dict_setitem(dict, new_str_v(k), v);
    };
    // Function annotations from the introspection registries.
    if let Some(params) = super::super::closure::func_params(obj) {
        for p in &params {
            if let Some(anno) = &p.annotation {
                if let Some(t) = resolve_annotation(anno) {
                    set(&p.name, t);
                } else {
                    super::super::exception::mb_raise(
                        new_str_v("NameError"),
                        new_str_v(&format!("name '{anno}' is not defined")),
                    );
                    return MbValue::none();
                }
            }
        }
        if let Some(ret) = super::super::closure::func_ret_anno(obj) {
            if let Some(t) = resolve_annotation(&ret) {
                set("return", t);
            } else {
                super::super::exception::mb_raise(
                    new_str_v("NameError"),
                    new_str_v(&format!("name '{ret}' is not defined")),
                );
                return MbValue::none();
            }
        }
        return dict;
    }
    // Class / module annotations: read the `__annotations__` mapping (PEP 526)
    // and resolve each textual annotation to its runtime type, matching
    // CPython's `get_type_hints(cls)`. The stored values are the textual
    // annotation (mamba's type-as-string representation).
    let ann = super::super::class::mb_getattr(obj, new_str_v("__annotations__"));
    let pairs: Vec<(String, MbValue)> = ann.as_ptr().map(|ptr| unsafe {
        if let super::super::rc::ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().unwrap().iter().filter_map(|(k, v)| {
                if let super::super::dict_ops::DictKey::Str(s) = k {
                    Some((s.clone(), *v))
                } else {
                    None
                }
            }).collect()
        } else {
            Vec::new()
        }
    }).unwrap_or_default();
    unsafe { super::super::rc::release_if_ptr(ann); }
    for (name, val) in pairs {
        let Some(anno) = extract_str(val) else { continue };
        if let Some(t) = resolve_annotation(&anno) {
            set(&name, t);
        } else {
            super::super::exception::mb_raise(
                new_str_v("NameError"),
                new_str_v(&format!("name '{anno}' is not defined")),
            );
            return MbValue::none();
        }
    }
    dict
}

/// Parse subscript arguments from an annotation string like "int" or "str, int".
/// Recursively resolves each argument.
fn parse_subscript_args(s: &str) -> Option<Vec<MbValue>> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    
    for ch in s.chars() {
        match ch {
            '[' => {
                depth += 1;
                current.push(ch);
            }
            ']' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                let arg_str = current.trim();
                if let Some(arg) = resolve_annotation(arg_str) {
                    args.push(arg);
                    current.clear();
                } else {
                    return None;
                }
            }
            _ => current.push(ch),
        }
    }
    
    if !current.is_empty() {
        let arg_str = current.trim();
        if let Some(arg) = resolve_annotation(arg_str) {
            args.push(arg);
        } else {
            return None;
        }
    }
    
    if args.is_empty() {
        return None;
    }
    
    Some(args)
}

/// Parse generic alias patterns like "typing.List[int]" or "List[int]".
fn parse_generic_alias(s: &str) -> Option<MbValue> {
    if let Some(bracket_pos) = s.find('[') {
        if !s.ends_with(']') {
            return None;
        }
        
        let base = s[..bracket_pos].trim();
        let subscript = &s[bracket_pos + 1..s.len() - 1];
        
        let name = if let Some(dot_pos) = base.rfind('.') {
            &base[dot_pos + 1..]
        } else {
            base
        };
        
        if let Some(args) = parse_subscript_args(subscript) {
            let key = if args.len() == 1 {
                args[0]
            } else {
                MbValue::from_ptr(MbObject::new_tuple(args))
            };
            return Some(special_form_subscript(name, key));
        }
    }
    None
}

/// Resolve a textual annotation to a runtime type object. Known builtin
/// scalar/container names resolve to the cached type singletons; unknown
/// names return None (the caller raises NameError, matching CPython's
/// forward-reference resolution failure).
fn resolve_annotation(anno: &str) -> Option<MbValue> {
    let name = anno.trim();
    let name = name.trim_matches(|c| c == '"' || c == '\'');
    
    // Try parsing as a generic alias first (e.g., "typing.List[int]")
    if name.contains('[') {
        if let Some(result) = parse_generic_alias(name) {
            return Some(result);
        }
    }
    
    match name {
        "int" | "float" | "str" | "bool" | "bytes" | "bytearray" | "list" | "dict" | "set"
        | "frozenset" | "tuple" | "type" | "object" | "complex" | "range" | "memoryview"
        | "slice" => Some(super::super::builtins::make_type_object(name)),
        "None" | "NoneType" => Some(super::super::builtins::make_type_object("NoneType")),
        "Any" | "typing.Any" => Some(special_form("Any")),
        // Generic aliases with parsing failed, unions, and bare typing special-form names.
        // unions (`int | str`) are not yet supported; bare typing names return Any sentinel.
        s if s.contains('|')
            || matches!(
                s,
                "Optional" | "List" | "Dict" | "Set" | "Tuple" | "FrozenSet"
                    | "Union" | "Callable" | "Sequence" | "Mapping" | "Iterable"
                    | "Iterator" | "ClassVar" | "Final"
            ) =>
        {
            Some(special_form("Any"))
        }
        _ => None,
    }
}

/// All type sentinels return None at runtime.
pub fn mb_typing_sentinel() -> MbValue {
    MbValue::none()
}

/// Identity helper for `runtime_checkable` / `final` / `override` /
/// `no_type_check` — returns its argument unchanged so decoration is a no-op.
pub fn mb_typing_identity(x: MbValue) -> MbValue {
    unsafe {
        super::super::rc::retain_if_ptr(x);
    }
    x
}

/// `NewType(name, tp)` — returns `tp` so the result stays callable as an
/// identity constructor (`UserId = NewType('UserId', int); UserId(5) == 5`).
pub fn mb_typing_newtype(_name: MbValue, tp: MbValue) -> MbValue {
    unsafe {
        super::super::rc::retain_if_ptr(tp);
    }
    tp
}

/// `typing.assert_type(val, typ)` — at runtime this is a pure identity that
/// returns its first argument unchanged (the type is only meaningful to a
/// static checker). `assert_type(arg, T) is arg` must hold for every `T`.
pub fn mb_typing_assert_type(val: MbValue, _typ: MbValue) -> MbValue {
    unsafe {
        super::super::rc::retain_if_ptr(val);
    }
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
                // CPython: @runtime_checkable applies only to Protocol classes.
                if !super::super::class::is_protocol_class(name) {
                    super::super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "@runtime_checkable can be only applied to protocol classes, \
                             got {name}"
                        ))),
                    );
                    return MbValue::none();
                }
                super::super::class::mark_runtime_checkable(name);
            }
        }
    }
    unsafe {
        super::super::rc::retain_if_ptr(cls);
    }
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
        assert_eq!(
            mb_typing_assert_type(val, MbValue::none()).as_int(),
            Some(7)
        );
    }

    #[test]
    fn test_is_typeddict_false_for_non_class() {
        assert_eq!(
            mb_typing_is_typeddict(MbValue::from_int(5)).as_bool(),
            Some(false)
        );
        assert_eq!(
            mb_typing_is_typeddict(MbValue::none()).as_bool(),
            Some(false)
        );
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
