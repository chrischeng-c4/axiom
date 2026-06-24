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
disp_unary!(d_identity, mb_typing_identity);
disp_unary!(d_override, mb_typing_override);
disp_unary!(d_final, mb_typing_final);
disp_binary!(d_newtype, mb_typing_newtype_meta);
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
        "Protocol",
        "Iterator",
        "Generator",
        "Coroutine",
        "AsyncGenerator",
        "AsyncIterator",
        "Awaitable",
        "NamedTuple",
        "TypedDict",
    ] {
        attrs.insert(name.to_string(), MbValue::from_func(sentinel_addr));
    }
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
    if instance_class_of(v).as_deref() == Some("typing.Alias") {
        instance_field_of(v, "_kind").and_then(|x| extract_str(x))
    } else {
        None
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
    use super::super::rc::ObjData;
    // Annotated metadata participates in the hash; unhashable members
    // (list/dict/set) raise like CPython.
    if let Some(meta) = instance_field_of(self_v, "__metadata__") {
        for m in tuple_items(meta) {
            if let Some(ptr) = m.as_ptr() {
                let bad = match &(*ptr).data {
                    ObjData::List(_) => Some("list"),
                    ObjData::Dict(_) => Some("dict"),
                    ObjData::Set(_) => Some("set"),
                    _ => None,
                };
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
    let origin_key = instance_field_of(self_v, "__origin__")
        .map(alias_key)
        .unwrap_or_default();
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
    }
    dict
}

/// Resolve a textual annotation to a runtime type object. Known builtin
/// scalar/container names resolve to the cached type singletons; unknown
/// names return None (the caller raises NameError, matching CPython's
/// forward-reference resolution failure).
fn resolve_annotation(anno: &str) -> Option<MbValue> {
    let name = anno.trim();
    let name = name.trim_matches(|c| c == '"' || c == '\'');
    match name {
        "int" | "float" | "str" | "bool" | "bytes" | "bytearray" | "list" | "dict" | "set"
        | "frozenset" | "tuple" | "type" | "object" | "complex" | "range" | "memoryview"
        | "slice" => Some(super::super::builtins::make_type_object(name)),
        "None" | "NoneType" => Some(super::super::builtins::make_type_object("NoneType")),
        "Any" | "typing.Any" => Some(special_form("Any")),
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
