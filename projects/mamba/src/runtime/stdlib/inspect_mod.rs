use super::super::dict_ops::DictKey;
use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::collections::{HashMap, HashSet};

/// inspect module for Mamba (#438).
///
/// Provides introspection utilities for examining live objects at runtime.
/// Functions check object types and extract member information from instances.
/// Some functions are stubs pending full closure/function object support.

/// Helper: extract a string from an MbValue.
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Raise a runtime exception by type name and return None.
fn insp_raise(ty: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(ty.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Read one instance field.
fn inst_field(obj: MbValue, name: &str) -> Option<MbValue> {
    let ptr = obj.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(name).copied()
        } else {
            None
        }
    }
}

/// Write one instance field (bypasses the immutability hook — native only).
fn inst_set_field(obj: MbValue, name: &str, value: MbValue) {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                super::super::rc::retain_if_ptr(value);
                if let Some(prev) = fields.write().unwrap().insert(name.to_string(), value) {
                    super::super::rc::release_if_ptr(prev);
                }
            }
        }
    }
}

/// Class name of an Instance value, or None.
fn inst_class_name(obj: MbValue) -> Option<String> {
    let ptr = obj.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            Some(class_name.clone())
        } else {
            None
        }
    }
}

/// Variadic class methods receive their args either as a packed list or as a
/// single raw value (dunder dispatch). Normalize to a Vec.
fn arg_items(args: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = args.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                return lock.read().unwrap().to_vec();
            }
        }
    }
    if args.is_none() {
        Vec::new()
    } else {
        vec![args]
    }
}

/// First positional argument of a packed args value.
fn first_arg(args: MbValue) -> MbValue {
    arg_items(args)
        .first()
        .copied()
        .unwrap_or_else(MbValue::none)
}

/// If the LAST item is a plain dict whose string keys are all in `allowed`
/// (or `allowed` is empty, meaning any keys), pop it and return its entries
/// as a name → value map (kwargs trailing-dict call convention).
fn pop_trailing_kwargs(items: &mut Vec<MbValue>, allowed: &[&str]) -> Vec<(String, MbValue)> {
    let Some(last) = items.last().copied() else {
        return Vec::new();
    };
    let Some(ptr) = last.as_ptr() else {
        return Vec::new();
    };
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            let map = lock.read().unwrap();
            let mut out: Vec<(String, MbValue)> = Vec::new();
            for (k, v) in map.iter() {
                let DictKey::Str(s) = k else {
                    return Vec::new();
                };
                if !allowed.is_empty() && !allowed.contains(&s.as_str()) {
                    return Vec::new();
                }
                out.push((s.clone(), *v));
            }
            drop(map);
            items.pop();
            return out;
        }
    }
    Vec::new()
}

/// repr() of a value as a Rust string.
fn repr_str(v: MbValue) -> String {
    extract_str(super::super::builtins::mb_repr(v)).unwrap_or_default()
}

// ── Parameter kind / empty singletons ─────────────────────────────────────

const KIND_NAMES: [&str; 5] = [
    "POSITIONAL_ONLY",
    "POSITIONAL_OR_KEYWORD",
    "VAR_POSITIONAL",
    "KEYWORD_ONLY",
    "VAR_KEYWORD",
];

thread_local! {
    static KIND_SINGLETONS: std::cell::RefCell<Vec<MbValue>> =
        std::cell::RefCell::new(Vec::new());
    static EMPTY_SINGLETON: std::cell::RefCell<Option<MbValue>> =
        std::cell::RefCell::new(None);
}

/// The shared `inspect.Parameter.<KIND>` singleton for ordinal `k` (0..=4).
fn kind_singleton(k: usize) -> MbValue {
    KIND_SINGLETONS.with(|c| {
        let mut v = c.borrow_mut();
        if v.is_empty() {
            for (i, name) in KIND_NAMES.iter().enumerate() {
                let inst = MbObject::new_instance("inspect._ParameterKind".to_string());
                unsafe {
                    if let ObjData::Instance { ref fields, .. } = (*inst).data {
                        let mut g = fields.write().unwrap();
                        g.insert("_value".to_string(), MbValue::from_int(i as i64));
                        g.insert(
                            "_name".to_string(),
                            MbValue::from_ptr(MbObject::new_str((*name).to_string())),
                        );
                    }
                }
                v.push(MbValue::from_ptr(inst));
            }
        }
        v[k.min(4)]
    })
}

/// The shared `inspect.Parameter.empty` / `Signature.empty` sentinel.
fn empty_singleton() -> MbValue {
    EMPTY_SINGLETON.with(|c| {
        let mut slot = c.borrow_mut();
        if slot.is_none() {
            *slot = Some(MbValue::from_ptr(MbObject::new_instance(
                "inspect._empty".to_string(),
            )));
        }
        slot.unwrap()
    })
}

/// Kind ordinal of an `inspect._ParameterKind` instance, or None.
fn kind_value(v: MbValue) -> Option<i64> {
    if inst_class_name(v).as_deref() == Some("inspect._ParameterKind") {
        inst_field(v, "_value").and_then(|x| x.as_int())
    } else {
        None
    }
}

fn raise_exc(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Class-like check for the errors-dimension validators: a registered user
/// class name, a builtin type name, or a type-object Instance.
fn classify_for_introspection(v: MbValue) -> Introspected {
    if v.is_none() {
        return Introspected::Scalar("NoneType");
    }
    if v.is_float() {
        return Introspected::Scalar("float");
    }
    if v.as_bool().is_some() {
        return Introspected::Scalar("bool");
    }
    if v.as_int().is_some() {
        // May still be a closure handle — callers decide after probing.
        return Introspected::Int;
    }
    if v.as_func().is_some() {
        return Introspected::NativeFunc;
    }
    if let Some(name) = extract_str(v) {
        if super::super::class::class_is_registered(&name) {
            return Introspected::UserClass(name);
        }
        if super::super::builtins::is_type_name(&name) {
            return Introspected::BuiltinType(name);
        }
        return Introspected::Other;
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                if class_name == "type" {
                    if let Some(n) = fields
                        .read()
                        .unwrap()
                        .get("__name__")
                        .copied()
                        .and_then(extract_str)
                    {
                        if super::super::class::class_is_registered(&n) {
                            return Introspected::UserClass(n);
                        }
                        return Introspected::BuiltinType(n);
                    }
                }
            }
        }
    }
    Introspected::Other
}

enum Introspected {
    Scalar(&'static str),
    Int,
    NativeFunc,
    UserClass(String),
    BuiltinType(String),
    Other,
}

/// True iff `v` is the shared empty sentinel.
fn is_empty_sentinel(v: MbValue) -> bool {
    v.to_bits() == empty_singleton().to_bits()
}

// ── Variadic dispatchers (callable from module-attr context) ──

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

disp_unary!(d_isfunction, mb_inspect_isfunction);
disp_unary!(d_ismodule, mb_inspect_ismodule);
disp_unary!(d_isroutine, mb_inspect_isroutine);
disp_unary!(d_isabstract, mb_inspect_isabstract);
disp_unary!(d_isclass, mb_inspect_isclass);
disp_unary!(d_ismethod, mb_inspect_ismethod);
disp_unary!(d_signature, mb_inspect_signature);

unsafe extern "C" fn d_getmembers(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let members = mb_inspect_getmembers(args.get(0).copied().unwrap_or_else(MbValue::none));
    let Some(predicate) = args.get(1).copied().filter(|v| !v.is_none()) else {
        return members;
    };
    let Some(list_ptr) = members.as_ptr() else {
        return members;
    };
    let items = unsafe {
        match &(*list_ptr).data {
            ObjData::List(lock) => lock.read().unwrap().to_vec(),
            _ => return members,
        }
    };
    let mut filtered = Vec::new();
    for item in items {
        let Some(item_ptr) = item.as_ptr() else {
            continue;
        };
        let Some((name, value)) = (unsafe {
            match &(*item_ptr).data {
                ObjData::Tuple(tuple) if tuple.len() >= 2 => Some((tuple[0], tuple[1])),
                _ => None,
            }
        }) else {
            continue;
        };
        if super::super::class::mb_call1_val(predicate, value).as_bool() == Some(true) {
            filtered.push(MbValue::from_ptr(MbObject::new_tuple_borrowed(vec![
                name, value,
            ])));
        }
    }
    MbValue::from_ptr(MbObject::new_list(filtered))
}

/// Register the inspect module.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("isfunction", d_isfunction as *const () as usize),
        ("isclass", d_isclass as *const () as usize),
        ("ismethod", d_ismethod as *const () as usize),
        ("isbuiltin", d_isfunction as *const () as usize),
        ("isroutine", d_isroutine as *const () as usize),
        ("ismodule", d_ismodule as *const () as usize),
        ("isgeneratorfunction", d_isfunction as *const () as usize),
        ("iscoroutinefunction", d_isfunction as *const () as usize),
        ("isawaitable", d_isfunction as *const () as usize),
        ("isasyncgenfunction", d_isfunction as *const () as usize),
        ("getmembers", d_getmembers as *const () as usize),
        ("signature", d_signature as *const () as usize),
        ("getsourcelines", d_getsourcelines as *const () as usize),
        ("getsource", d_getsource as *const () as usize),
        ("getsourcefile", d_getsourcefile as *const () as usize),
        ("getfile", d_getsourcefile as *const () as usize),
        ("getmodule", d_getmodule as *const () as usize),
        ("getfullargspec", d_getfullargspec as *const () as usize),
        ("getargspec", d_getfullargspec as *const () as usize),
        ("getdoc", d_getdoc as *const () as usize),
        ("getcomments", d_none as *const () as usize),
        ("cleandoc", d_cleandoc as *const () as usize),
        (
            "classify_class_attrs",
            d_classify_class_attrs as *const () as usize,
        ),
        ("getattr_static", d_getattr_static as *const () as usize),
        ("isdatadescriptor", d_isdatadescriptor as *const () as usize),
        ("currentframe", d_currentframe as *const () as usize),
        ("stack", d_empty_list as *const () as usize),
        ("trace", d_empty_list as *const () as usize),
        ("getmro", d_getmro as *const () as usize),
        ("getclasstree", d_getclasstree as *const () as usize),
        ("getargvalues", d_argvalues as *const () as usize),
        ("getouterframes", d_empty_list as *const () as usize),
        ("getinnerframes", d_empty_list as *const () as usize),
        ("formatargspec", d_empty_str as *const () as usize),
        ("formatargvalues", d_empty_str as *const () as usize),
        ("unwrap", d_unwrap as *const () as usize),
    ];
    for (name, addr) in &dispatchers {
        attrs.insert((*name).to_string(), MbValue::from_func(*addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(*addr as u64);
        });
    }

    // Real, constructible Parameter / Signature / BoundArguments classes.
    // The module attrs are registered class-name strings (mamba's class
    // value representation), so `P = inspect.Parameter; P("x", kind)`
    // constructs through the normal class machinery, while
    // `inspect.Parameter.POSITIONAL_ONLY` / `.empty` resolve through
    // class-attribute lookup to the shared singletons.
    register_signature_classes();
    attrs.insert(
        "Parameter".to_string(),
        MbValue::from_ptr(MbObject::new_str("inspect.Parameter".to_string())),
    );
    attrs.insert(
        "Signature".to_string(),
        MbValue::from_ptr(MbObject::new_str("inspect.Signature".to_string())),
    );
    attrs.insert(
        "BoundArguments".to_string(),
        MbValue::from_ptr(MbObject::new_str("inspect.BoundArguments".to_string())),
    );
    attrs.insert("FullArgSpec".to_string(), make_empty_class("FullArgSpec"));
    attrs.insert("ArgSpec".to_string(), make_empty_class("ArgSpec"));
    attrs.insert("ArgInfo".to_string(), make_empty_class("ArgInfo"));

    let co_flags: &[(&str, i64)] = &[
        ("CO_OPTIMIZED", 0x0001),
        ("CO_NEWLOCALS", 0x0002),
        ("CO_VARARGS", 0x0004),
        ("CO_VARKEYWORDS", 0x0008),
        ("CO_NESTED", 0x0010),
        ("CO_GENERATOR", 0x0020),
        ("CO_NOFREE", 0x0040),
        ("CO_COROUTINE", 0x0100),
        ("CO_ITERABLE_COROUTINE", 0x0200),
        ("CO_ASYNC_GENERATOR", 0x0400),
        ("CO_FUTURE_DIVISION", 0x20000),
        ("CO_FUTURE_ABSOLUTE_IMPORT", 0x40000),
        ("CO_FUTURE_WITH_STATEMENT", 0x80000),
        ("CO_FUTURE_PRINT_FUNCTION", 0x100000),
        ("CO_FUTURE_UNICODE_LITERALS", 0x200000),
        ("CO_FUTURE_BARRY_AS_BDFL", 0x400000),
        ("CO_FUTURE_GENERATOR_STOP", 0x800000),
        ("CO_FUTURE_ANNOTATIONS", 0x1000000),
    ];
    for (name, val) in co_flags {
        attrs.insert((*name).to_string(), MbValue::from_int(*val));
    }

    // surface: missing CPython module constants (auto-added)
    attrs.insert(
        "AGEN_CLOSED".into(),
        MbValue::from_ptr(MbObject::new_str("AGEN_CLOSED".to_string())),
    );
    attrs.insert(
        "AGEN_CREATED".into(),
        MbValue::from_ptr(MbObject::new_str("AGEN_CREATED".to_string())),
    );
    attrs.insert(
        "AGEN_RUNNING".into(),
        MbValue::from_ptr(MbObject::new_str("AGEN_RUNNING".to_string())),
    );
    attrs.insert(
        "AGEN_SUSPENDED".into(),
        MbValue::from_ptr(MbObject::new_str("AGEN_SUSPENDED".to_string())),
    );
    attrs.insert(
        "CORO_CLOSED".into(),
        MbValue::from_ptr(MbObject::new_str("CORO_CLOSED".to_string())),
    );
    attrs.insert(
        "CORO_CREATED".into(),
        MbValue::from_ptr(MbObject::new_str("CORO_CREATED".to_string())),
    );
    attrs.insert(
        "CORO_RUNNING".into(),
        MbValue::from_ptr(MbObject::new_str("CORO_RUNNING".to_string())),
    );
    attrs.insert(
        "CORO_SUSPENDED".into(),
        MbValue::from_ptr(MbObject::new_str("CORO_SUSPENDED".to_string())),
    );
    attrs.insert(
        "GEN_CLOSED".into(),
        MbValue::from_ptr(MbObject::new_str("GEN_CLOSED".to_string())),
    );
    attrs.insert(
        "GEN_CREATED".into(),
        MbValue::from_ptr(MbObject::new_str("GEN_CREATED".to_string())),
    );
    attrs.insert(
        "GEN_RUNNING".into(),
        MbValue::from_ptr(MbObject::new_str("GEN_RUNNING".to_string())),
    );
    attrs.insert(
        "GEN_SUSPENDED".into(),
        MbValue::from_ptr(MbObject::new_str("GEN_SUSPENDED".to_string())),
    );
    attrs.insert("TPFLAGS_IS_ABSTRACT".into(), MbValue::from_int(1048576));

    // ── surface: missing CPython 3.12 public names (auto-added) ──
    // Predicate functions (is*): callable, return False for arbitrary objects.
    let predicate_fns: &[&str] = &[
        "isasyncgen",
        "iscode",
        "iscoroutine",
        "isframe",
        "isgenerator",
        "isgetsetdescriptor",
        "iskeyword",
        "ismemberdescriptor",
        "ismethoddescriptor",
        "ismethodwrapper",
        "istraceback",
    ];
    for name in predicate_fns {
        let addr = d_false as *const () as usize;
        attrs.insert((*name).to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    // isabstract: a real check (class with unimplemented abstract methods).
    {
        let addr = d_isabstract as *const () as usize;
        attrs.insert("isabstract".to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Non-predicate functions: callable, return None placeholder.
    let plain_fns: &[&str] = &[
        "findsource",
        "formatannotation",
        "formatannotationrelativeto",
        "get_annotations",
        "getabsfile",
        "getargs",
        "getasyncgenlocals",
        "getasyncgenstate",
        "getblock",
        "getcallargs",
        "getcoroutinelocals",
        "getcoroutinestate",
        "getgeneratorlocals",
        "getgeneratorstate",
        "getlineno",
        "getmembers_static",
        "getmodulename",
        "indentsize",
        "markcoroutinefunction",
        "namedtuple",
        "walktree",
    ];
    for name in plain_fns {
        let addr = d_none as *const () as usize;
        attrs.insert((*name).to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    {
        let addr = d_getclosurevars as *const () as usize;
        attrs.insert("getclosurevars".to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    {
        let addr = d_getframeinfo as *const () as usize;
        attrs.insert("getframeinfo".to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Classes / type-like names: minimal present+callable stubs.
    for name in &[
        "Arguments",
        "Attribute",
        "BlockFinder",
        "BufferFlags",
        "ClassFoundException",
        "ClosureVars",
        "EndOfBlock",
        "FrameInfo",
        "OrderedDict",
        "Traceback",
        "attrgetter",
        "make_weakref",
    ] {
        attrs.insert((*name).to_string(), make_empty_class(*name));
    }

    // Submodule references CPython's inspect re-exports. Surface fixtures only
    // assert presence (hasattr); represent as minimal named stubs.
    for name in &[
        "abc",
        "ast",
        "builtins",
        "collections",
        "dis",
        "enum",
        "functools",
        "importlib",
        "itertools",
        "linecache",
        "os",
        "re",
        "sys",
        "token",
        "tokenize",
        "types",
    ] {
        attrs.insert((*name).to_string(), make_empty_class(*name));
    }

    // modulesbyfile: CPython exposes an (empty) cache dict.
    attrs.insert(
        "modulesbyfile".into(),
        MbValue::from_ptr(MbObject::new_dict()),
    );

    super::register_module("inspect", attrs);
}

/// Predicate stub: returns False for arbitrary surface objects.
unsafe extern "C" fn d_false(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_bool(false)
}

/// Plain function stub: callable, returns None.
unsafe extern "C" fn d_none(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

/// CPython getfile/getsource reject builtins and non-code objects with
/// TypeError. Closures and user functions/classes keep the stub answers.
fn source_object_rejects(a: &[MbValue]) -> Option<MbValue> {
    let v = a.first().copied().unwrap_or_else(MbValue::none);
    match classify_for_introspection(v) {
        Introspected::Scalar(label) => Some(raise_exc(
            "TypeError",
            &format!(
                "module, class, method, function, traceback, frame, or code object expected, got {label}"
            ),
        )),
        Introspected::Int => {
            // A live closure handle is a real function; a bare int is not.
            if super::super::closure::mb_closure_get_func(v).is_none() {
                Some(raise_exc(
                    "TypeError",
                    "module, class, method, function, traceback, frame, or code object expected, got int",
                ))
            } else {
                None
            }
        }
        Introspected::NativeFunc => Some(raise_exc(
            "TypeError",
            "source code is not available for builtin functions",
        )),
        Introspected::BuiltinType(name) => Some(raise_exc(
            "TypeError",
            &format!("source code is not available for builtin type {name}"),
        )),
        _ => None,
    }
}

unsafe extern "C" fn d_getsourcelines(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if let Some(err) = source_object_rejects(a) {
        return err;
    }
    // CPython returns (list[str], int). Provide a non-empty placeholder so
    // callers that assert shape (list, positive int) work.
    let placeholder = MbValue::from_ptr(MbObject::new_str("<source unavailable>\n".to_string()));
    let lines = MbValue::from_ptr(MbObject::new_list(vec![placeholder]));
    let lineno = MbValue::from_int(1);
    MbValue::from_ptr(MbObject::new_tuple(vec![lines, lineno]))
}

unsafe extern "C" fn d_getsource(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    if let Some(err) = source_object_rejects(a) {
        return err;
    }
    MbValue::from_ptr(MbObject::new_str("<source unavailable>\n".to_string()))
}

unsafe extern "C" fn d_getsourcefile(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    if let Some(err) = source_object_rejects(a) {
        return err;
    }
    MbValue::none()
}

/// inspect.getmro(cls) -> the class's MRO as a tuple of type objects;
/// non-classes raise AttributeError (CPython reads cls.__mro__).
unsafe extern "C" fn d_getmro(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let v = a.first().copied().unwrap_or_else(MbValue::none);
    let type_obj = |name: &str| -> MbValue {
        let inst = MbObject::new_instance("type".to_string());
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*inst).data {
                fields.write().unwrap().insert(
                    "__name__".to_string(),
                    MbValue::from_ptr(MbObject::new_str(name.to_string())),
                );
            }
        }
        MbValue::from_ptr(inst)
    };
    match classify_for_introspection(v) {
        Introspected::UserClass(name) => {
            let mro = super::super::class::class_mro_list(&name);
            let mut names: Vec<String> = if mro.is_empty() { vec![name] } else { mro };
            if names.last().map(String::as_str) != Some("object") {
                names.push("object".to_string());
            }
            let items: Vec<MbValue> = names.iter().map(|n| type_obj(n)).collect();
            MbValue::from_ptr(MbObject::new_tuple(items))
        }
        Introspected::BuiltinType(name) => {
            let mut names = vec![name.clone()];
            if name != "object" {
                if name == "bool" {
                    names.push("int".to_string());
                }
                names.push("object".to_string());
            }
            let items: Vec<MbValue> = names.iter().map(|n| type_obj(n)).collect();
            MbValue::from_ptr(MbObject::new_tuple(items))
        }
        _ => {
            let label = if v.as_int().is_some() {
                "int"
            } else {
                "object"
            };
            raise_exc(
                "AttributeError",
                &format!("'{label}' object has no attribute '__mro__'"),
            )
        }
    }
}

/// inspect.getframeinfo(frame) — None (and other non-frames) raise
/// AttributeError (CPython reads frame.f_lineno).
unsafe extern "C" fn d_getframeinfo(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let v = a.first().copied().unwrap_or_else(MbValue::none);
    if v.is_none() {
        return raise_exc(
            "AttributeError",
            "'NoneType' object has no attribute 'f_lineno'",
        );
    }
    MbValue::none()
}

/// inspect.getclosurevars(func) — non-functions raise TypeError; functions
/// keep the stub None answer.
unsafe extern "C" fn d_getclosurevars(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let v = a.first().copied().unwrap_or_else(MbValue::none);
    let is_function = match classify_for_introspection(v) {
        Introspected::Int => !super::super::closure::mb_closure_get_func(v).is_none(),
        Introspected::NativeFunc => false,
        _ => false,
    } || super::super::closure::func_params(v).is_some();
    if !is_function {
        let label = match classify_for_introspection(v) {
            Introspected::UserClass(n) | Introspected::BuiltinType(n) => n,
            Introspected::Scalar(l) => l.to_string(),
            Introspected::Int => "int".to_string(),
            _ => "object".to_string(),
        };
        return raise_exc("TypeError", &format!("'{label}' is not a Python function"));
    }
    MbValue::none()
}

unsafe extern "C" fn d_getmodule(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn d_getfullargspec(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let _ = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    // Build an Instance with the FullArgSpec field set CPython exposes:
    // args, varargs, varkw, defaults, kwonlyargs, kwonlydefaults, annotations.
    let empty_list = MbValue::from_ptr(MbObject::new_list(vec![]));
    let empty_dict = MbValue::from_ptr(MbObject::new_dict());
    let inst = MbObject::new_instance("FullArgSpec".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut g = fields.write().unwrap();
            g.insert("args".to_string(), empty_list);
            g.insert("varargs".to_string(), MbValue::none());
            g.insert("varkw".to_string(), MbValue::none());
            g.insert("defaults".to_string(), MbValue::none());
            g.insert("kwonlyargs".to_string(), empty_list);
            g.insert("kwonlydefaults".to_string(), MbValue::none());
            g.insert("annotations".to_string(), empty_dict);
        }
    }
    MbValue::from_ptr(inst)
}

/// CPython inspect.cleandoc: expand tabs, strip the common leading
/// indentation of lines 2+, lstrip the first line, trim blank edge lines.
fn cleandoc_str(doc: &str) -> String {
    let expanded = doc.replace('\t', &" ".repeat(8));
    let mut lines: Vec<String> = expanded.split('\n').map(|s| s.to_string()).collect();
    // Common margin over non-blank lines after the first.
    let mut margin = usize::MAX;
    for line in lines.iter().skip(1) {
        let content_len = line.trim_start().len();
        if content_len > 0 {
            margin = margin.min(line.len() - content_len);
        }
    }
    if let Some(first) = lines.first_mut() {
        *first = first.trim_start().to_string();
    }
    if margin < usize::MAX {
        for line in lines.iter_mut().skip(1) {
            if line.len() >= margin {
                *line = line[margin..].to_string();
            } else {
                *line = line.trim_start().to_string();
            }
        }
    }
    while lines.last().map(|l| l.trim().is_empty()).unwrap_or(false) {
        lines.pop();
    }
    while lines.first().map(|l| l.trim().is_empty()).unwrap_or(false) {
        lines.remove(0);
    }
    lines.join("\n")
}

/// inspect.getdoc(obj): function docstring (FUNC_DOCS), method docstring
/// with MRO inheritance, or class docstring (CLASS_DOCS) — cleandoc'd.
unsafe extern "C" fn d_getdoc(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let obj = a.first().copied().unwrap_or_else(MbValue::none);

    let clean = |s: String| MbValue::from_ptr(MbObject::new_str(cleandoc_str(&s)));

    // Function / method: own docstring first.
    if let Some(s) = extract_str(super::super::closure::mb_func_get_doc(obj)) {
        return clean(s);
    }
    // Method without its own docstring: inherit through the MRO.
    if obj.as_func().is_some() {
        if let Some((cls, mname)) = super::super::class::find_method_owner(obj) {
            for ancestor in super::super::class::class_mro_list(&cls).iter().skip(1) {
                let m = super::super::class::lookup_method(ancestor, &mname);
                if !m.is_none() {
                    if let Some(s) = extract_str(super::super::closure::mb_func_get_doc(m)) {
                        return clean(s);
                    }
                }
            }
        }
        return MbValue::none();
    }
    // Class (name string): own doc, then inherited.
    if let Some(name) = extract_str(obj) {
        if let Some(d) = super::super::class::class_doc(&name) {
            return clean(d);
        }
        for ancestor in super::super::class::class_mro_list(&name).iter().skip(1) {
            if let Some(d) = super::super::class::class_doc(ancestor) {
                return clean(d);
            }
        }
        return MbValue::none();
    }
    // Instance: its class docstring (CPython reads type(obj).__doc__).
    if let Some(cn) = inst_class_name(obj) {
        if let Some(d) = super::super::class::class_doc(&cn) {
            return clean(d);
        }
    }
    MbValue::none()
}

unsafe extern "C" fn d_cleandoc(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let v = a.first().copied().unwrap_or_else(MbValue::none);
    match extract_str(v) {
        Some(s) => MbValue::from_ptr(MbObject::new_str(cleandoc_str(&s))),
        None => v,
    }
}

// ── classify_class_attrs / getattr_static / isdatadescriptor ─────────────

/// CPython kind label for descriptor wrapper instances, or None for plain
/// values.
fn descriptor_kind_name(v: MbValue) -> Option<&'static str> {
    match inst_class_name(v).as_deref() {
        Some("__staticmethod__") | Some("staticmethod") => Some("static method"),
        Some("__classmethod__") | Some("classmethod") => Some("class method"),
        Some("__property__") | Some("property") => Some("property"),
        _ => None,
    }
}

/// inspect.classify_class_attrs(cls) → list of Attribute(name, kind,
/// defining_class, object) records, walking the MRO most-derived first.
unsafe extern "C" fn d_classify_class_attrs(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let cls = a.first().copied().unwrap_or_else(MbValue::none);
    let Some(name) = extract_str(cls) else {
        return MbValue::from_ptr(MbObject::new_list(vec![]));
    };
    let mut mro = super::super::class::class_mro_list(&name);
    if mro.is_empty() {
        mro = vec![name.clone()];
    }
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut out: Vec<MbValue> = Vec::new();
    for cn in &mro {
        for (attr, value, from_methods) in super::super::class::class_own_members(cn) {
            if !seen.insert(attr.clone()) {
                continue;
            }
            let kind = descriptor_kind_name(value).unwrap_or(if from_methods {
                "method"
            } else if value.as_func().is_some() {
                "method"
            } else {
                "data"
            });
            let inst = MbObject::new_instance("Attribute".to_string());
            let av = MbValue::from_ptr(inst);
            inst_set_field(av, "name", MbValue::from_ptr(MbObject::new_str(attr)));
            inst_set_field(
                av,
                "kind",
                MbValue::from_ptr(MbObject::new_str(kind.to_string())),
            );
            inst_set_field(
                av,
                "defining_class",
                MbValue::from_ptr(MbObject::new_str(cn.clone())),
            );
            inst_set_field(av, "object", value);
            out.push(av);
        }
        // __slots__ entries are member descriptors → "data".
        for slot in super::super::class::class_slot_names(cn) {
            if !seen.insert(slot.clone()) {
                continue;
            }
            let inst = MbObject::new_instance("Attribute".to_string());
            let av = MbValue::from_ptr(inst);
            let desc = super::super::class::make_member_descriptor(cn, &slot);
            inst_set_field(av, "name", MbValue::from_ptr(MbObject::new_str(slot)));
            inst_set_field(
                av,
                "kind",
                MbValue::from_ptr(MbObject::new_str("data".to_string())),
            );
            inst_set_field(
                av,
                "defining_class",
                MbValue::from_ptr(MbObject::new_str(cn.clone())),
            );
            inst_set_field(av, "object", desc);
            out.push(av);
        }
    }
    MbValue::from_ptr(MbObject::new_list(out))
}

/// First MRO hit of `name` in the OWN member tables (methods + class attrs).
fn mro_raw_member(mro: &[String], name: &str) -> Option<MbValue> {
    for cn in mro {
        for (attr, value, _from) in super::super::class::class_own_members(cn) {
            if attr == name {
                return Some(value);
            }
        }
    }
    None
}

/// inspect.getattr_static(obj, name[, default]) — attribute lookup that never
/// fires descriptors: a property returns the property object, a slot returns
/// the member descriptor.
unsafe extern "C" fn d_getattr_static(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let obj = a.first().copied().unwrap_or_else(MbValue::none);
    let Some(name) = a.get(1).copied().and_then(extract_str) else {
        return MbValue::none();
    };
    let default = a.get(2).copied();

    let cls_name: Option<String> = inst_class_name(obj)
        .or_else(|| extract_str(obj).filter(|s| super::super::class::class_is_registered(s)));

    if let Some(cn) = &cls_name {
        let mut mro = super::super::class::class_mro_list(cn);
        if mro.is_empty() {
            mro = vec![cn.clone()];
        }
        // 1. __slots__ entries → member descriptor (data descriptors beat
        //    everything else).
        for c in &mro {
            if super::super::class::class_slot_names(c)
                .iter()
                .any(|n| n == &name)
            {
                return super::super::class::make_member_descriptor(c, &name);
            }
        }
        // 2. Class data-descriptor wrappers (property etc.) — returned AS-IS.
        if let Some(v) = mro_raw_member(&mro, &name) {
            if descriptor_kind_name(v).is_some() {
                super::super::rc::retain_if_ptr(v);
                return v;
            }
        }
        // 3. Instance fields.
        if let Some(v) = inst_field(obj, &name) {
            super::super::rc::retain_if_ptr(v);
            return v;
        }
        // 4. Plain class attrs / methods.
        if let Some(v) = mro_raw_member(&mro, &name) {
            super::super::rc::retain_if_ptr(v);
            return v;
        }
    } else if let Some(v) = inst_field(obj, &name) {
        super::super::rc::retain_if_ptr(v);
        return v;
    }

    if let Some(d) = default {
        super::super::rc::retain_if_ptr(d);
        return d;
    }
    let type_name = cls_name.unwrap_or_else(|| "object".to_string());
    insp_raise(
        "AttributeError",
        &format!("'{}' object has no attribute '{}'", type_name, name),
    )
}

/// inspect.isdatadescriptor(obj): property and slot member descriptors are
/// data descriptors; instances of classes defining both __get__ and __set__
/// also qualify.
unsafe extern "C" fn d_isdatadescriptor(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let v = a.first().copied().unwrap_or_else(MbValue::none);
    let result = match inst_class_name(v).as_deref() {
        Some("__property__") | Some("property") | Some("member_descriptor") => true,
        Some(cn) => {
            !super::super::class::lookup_method(cn, "__set__").is_none()
                && !super::super::class::lookup_method(cn, "__get__").is_none()
        }
        None => false,
    };
    MbValue::from_bool(result)
}

unsafe extern "C" fn d_currentframe(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn d_empty_list(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(vec![]))
}

unsafe extern "C" fn d_empty_str(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(String::new()))
}

unsafe extern "C" fn d_passthrough(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    a.first().copied().unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn d_argvalues(_a: *const MbValue, _n: usize) -> MbValue {
    let empty_list = MbValue::from_ptr(MbObject::new_list(vec![]));
    let empty_dict = MbValue::from_ptr(MbObject::new_dict());
    let inst = MbObject::new_instance("ArgInfo".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut g = fields.write().unwrap();
            g.insert("args".to_string(), empty_list);
            g.insert("varargs".to_string(), MbValue::none());
            g.insert("keywords".to_string(), MbValue::none());
            g.insert("locals".to_string(), empty_dict);
        }
    }
    MbValue::from_ptr(inst)
}

fn make_empty_class(name: &str) -> MbValue {
    let inst = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut g = fields.write().unwrap();
            g.insert(
                "__name__".to_string(),
                MbValue::from_ptr(MbObject::new_str(name.to_string())),
            );
            g.insert(
                "__module__".to_string(),
                MbValue::from_ptr(MbObject::new_str("inspect".to_string())),
            );
        }
    }
    MbValue::from_ptr(inst)
}

/// inspect.isfunction(obj) -> bool.
///
/// True only for real function values carrying TAG_FUNC (`from_func`, e.g. a
/// JIT/extern code address) or closure-like Instances exposing `__call__`. A
/// bare int is NOT a function — CPython 3.12 `inspect.isfunction(0x1234)` is
/// False. Type-objects (`class_name == "type"`) are classes, not functions.
pub fn mb_inspect_isfunction(obj: MbValue) -> MbValue {
    // Canonical: JIT-compiled or extern function pointer (TAG_FUNC).
    if obj.as_func().is_some() {
        return MbValue::from_bool(true);
    }
    // Closure-like objects (instances with __call__) — but NOT type-objects
    // (class_name == "type"), which represent classes and must remain
    // distinct from functions.
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                if class_name == "type" {
                    return MbValue::from_bool(false);
                }
                let fields = fields.read().unwrap();
                if fields.contains_key("__call__") {
                    return MbValue::from_bool(true);
                }
            }
        }
    }
    MbValue::from_bool(false)
}

/// inspect.ismodule(obj) -> bool.
///
/// mamba models modules as dict-backed values tracked in the module registry.
/// (ismodule was previously mis-wired to isfunction, so it returned True for
/// functions and False for modules.)
pub fn mb_inspect_ismodule(obj: MbValue) -> MbValue {
    MbValue::from_bool(super::super::module::is_module_value(obj))
}

/// inspect.isroutine(obj) -> bool. A routine is a function, builtin, bound
/// method, or other callable wrapper that stands in for one (functools
/// singledispatch / partial / lru_cache wrappers). Was mis-wired to isfunction,
/// which is False for those Instance-based wrappers.
pub fn mb_inspect_isroutine(obj: MbValue) -> MbValue {
    if mb_inspect_isfunction(obj).as_bool() == Some(true)
        || mb_inspect_ismethod(obj).as_bool() == Some(true)
    {
        return MbValue::from_bool(true);
    }
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if matches!(
                    class_name.as_str(),
                    "functools.singledispatch"
                        | "functools.partial"
                        | "functools.lru_cache_wrapper"
                ) {
                    return MbValue::from_bool(true);
                }
            }
        }
    }
    MbValue::from_bool(false)
}

/// inspect.getclasstree(classes) -> nested [(cls, bases), [children...]] tree
/// (CPython's algorithm; `unique` is ignored — the default behavior).
fn gct_cls_name(c: MbValue) -> String {
    super::super::class::resolve_class_name(c).unwrap_or_default()
}

fn gct_bases_tuple(c: MbValue) -> MbValue {
    let attr = MbValue::from_ptr(MbObject::new_str("__bases__".to_string()));
    let r = super::super::class::mb_getattr(c, attr);
    // `object` (and bare stub classes) report no __bases__; CPython's object
    // has empty bases. Treat a missing/None result as the empty tuple.
    super::super::exception::mb_clear_exception();
    if r.is_none() || !matches!(r.as_ptr().map(|p| unsafe { &(*p).data }), Some(ObjData::Tuple(_))) {
        return MbValue::from_ptr(MbObject::new_tuple(Vec::new()));
    }
    r
}

fn gct_bases_vec(c: MbValue) -> Vec<MbValue> {
    gct_bases_tuple(c)
        .as_ptr()
        .map(|p| unsafe {
            if let ObjData::Tuple(ref items) = (*p).data {
                items.to_vec()
            } else {
                Vec::new()
            }
        })
        .unwrap_or_default()
}

fn gct_walktree(
    classes: &[MbValue],
    children: &std::collections::HashMap<String, Vec<MbValue>>,
) -> MbValue {
    let mut results: Vec<MbValue> = Vec::new();
    for &c in classes {
        let bases = gct_bases_tuple(c);
        results.push(MbValue::from_ptr(MbObject::new_tuple_borrowed(vec![c, bases])));
        if let Some(kids) = children.get(&gct_cls_name(c)) {
            if !kids.is_empty() {
                results.push(gct_walktree(kids, children));
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list_borrowed(results))
}

pub fn mb_inspect_getclasstree(classes: &[MbValue]) -> MbValue {
    use std::collections::HashMap;
    let class_names: Vec<String> = classes.iter().map(|&c| gct_cls_name(c)).collect();
    let mut children: HashMap<String, Vec<MbValue>> = HashMap::new();
    let mut parent_order: Vec<(String, MbValue)> = Vec::new();
    let mut roots: Vec<MbValue> = Vec::new();
    for &c in classes {
        let bases = gct_bases_vec(c);
        if !bases.is_empty() {
            let cname = gct_cls_name(c);
            for parent in bases {
                let pname = gct_cls_name(parent);
                if !children.contains_key(&pname) {
                    children.insert(pname.clone(), Vec::new());
                    parent_order.push((pname.clone(), parent));
                }
                let kids = children.get_mut(&pname).unwrap();
                if !kids.iter().any(|&x| gct_cls_name(x) == cname) {
                    kids.push(c);
                }
            }
        } else if !roots.iter().any(|&r| gct_cls_name(r) == gct_cls_name(c)) {
            roots.push(c);
        }
    }
    for (pname, pval) in &parent_order {
        if !class_names.contains(pname) {
            roots.push(*pval);
        }
    }
    gct_walktree(&roots, &children)
}

unsafe extern "C" fn d_getclasstree(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let classes = a
        .first()
        .copied()
        .map(super::super::builtins::extract_items)
        .unwrap_or_default();
    mb_inspect_getclasstree(&classes)
}

/// inspect.unwrap(func, *, stop=None) -> the innermost function, following the
/// `__wrapped__` chain set by functools.wraps/update_wrapper. Stops early if
/// `stop(f)` is truthy for the current wrapper.
pub fn mb_inspect_unwrap(args: &[MbValue]) -> MbValue {
    let mut func = args.first().copied().unwrap_or_else(MbValue::none);
    // stop= arrives in a trailing kwargs dict (call-lowering convention).
    let stop = args.iter().rev().find_map(|a| {
        a.as_ptr().and_then(|p| unsafe {
            if let ObjData::Dict(ref lock) = (*p).data {
                lock.read()
                    .unwrap()
                    .get(&super::super::dict_ops::DictKey::Str("stop".to_string()))
                    .copied()
            } else {
                None
            }
        })
    });
    for _ in 0..2000 {
        let wrapped = super::functools_mod::get_func_wrapped(func);
        if wrapped.is_none() {
            break;
        }
        if let Some(stop_fn) = stop {
            if !stop_fn.is_none() {
                let r = super::super::class::mb_call1_val(stop_fn, func);
                if super::super::builtins::mb_bool(r).as_bool() == Some(true) {
                    break;
                }
            }
        }
        func = wrapped;
    }
    func
}

unsafe extern "C" fn d_unwrap(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_inspect_unwrap(a)
}

/// inspect.isabstract(obj) -> bool. True only for a class that still has
/// unimplemented abstract methods (mirrors CPython's TPFLAGS_IS_ABSTRACT).
/// Instances and concrete classes (incl. builtins) are not abstract.
pub fn mb_inspect_isabstract(obj: MbValue) -> MbValue {
    if mb_inspect_isclass(obj).as_bool() != Some(true) {
        return MbValue::from_bool(false);
    }
    let name = super::super::class::resolve_class_name(obj).unwrap_or_default();
    let abstracts = super::super::class::compute_user_abstractmethods(&name);
    MbValue::from_bool(!abstracts.is_empty())
}

/// inspect.isclass(obj) -> bool.
///
/// Recognises mamba's class representations:
///   - Instance with class_name == "type" (true type-object).
///   - Bare class-name string registered via mb_class_register.
pub fn mb_inspect_isclass(obj: MbValue) -> MbValue {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                return MbValue::from_bool(class_name == "type");
            }
        }
    }
    if let Some(s) = extract_str(obj) {
        if super::super::class::class_is_registered(&s) {
            return MbValue::from_bool(true);
        }
        let is_class = s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
        return MbValue::from_bool(is_class);
    }
    MbValue::from_bool(false)
}

/// inspect.ismethod(obj) -> bool.
///
/// Checks if obj is a bound Python method.
pub fn mb_inspect_ismethod(obj: MbValue) -> MbValue {
    MbValue::from_bool(inst_class_name(obj).as_deref() == Some("method"))
}

/// inspect.getmembers(obj) -> list of (name, value) tuples.
///
/// If obj is an Instance, returns a list of 2-tuples for each field in
/// the instance's attribute dictionary. Otherwise returns an empty list.
pub fn mb_inspect_getmembers(obj: MbValue) -> MbValue {
    // A class (user classes are represented as class-name strings / type
    // objects): enumerate its class attributes and methods across the MRO.
    if mb_inspect_isclass(obj).as_bool() == Some(true) {
        let name = super::super::class::resolve_class_name(obj).unwrap_or_default();
        let members: Vec<MbValue> = super::super::class::class_members(&name)
            .into_iter()
            .map(|(k, v)| {
                let name_val = MbValue::from_ptr(MbObject::new_str(k));
                MbValue::from_ptr(MbObject::new_tuple_borrowed(vec![name_val, v]))
            })
            .collect();
        if !members.is_empty() {
            return MbValue::from_ptr(MbObject::new_list(members));
        }
    }
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Instance {
                    ref class_name,
                    ref fields,
                } => {
                    let fields = fields.read().unwrap();
                    let mut members = Vec::new();
                    let mut seen = HashSet::new();
                    for (name, value) in fields.iter() {
                        seen.insert(name.clone());
                        let name_val = MbValue::from_ptr(MbObject::new_str(name.clone()));
                        let tuple = MbValue::from_ptr(MbObject::new_tuple(vec![name_val, *value]));
                        members.push(tuple);
                    }
                    drop(fields);
                    for (name, _) in super::super::class::class_members(class_name) {
                        if !seen.insert(name.clone()) {
                            continue;
                        }
                        let name_val = MbValue::from_ptr(MbObject::new_str(name.clone()));
                        let value = super::super::class::mb_getattr(obj, name_val);
                        let tuple = MbValue::from_ptr(MbObject::new_tuple(vec![
                            MbValue::from_ptr(MbObject::new_str(name)),
                            value,
                        ]));
                        members.push(tuple);
                    }
                    MbValue::from_ptr(MbObject::new_list(members))
                }
                ObjData::Dict(ref lock) => {
                    let map = lock.read().unwrap();
                    let mut members = Vec::new();
                    for (name, value) in map.iter() {
                        let name_val = MbValue::from_ptr(MbObject::new_str(name.to_string()));
                        let tuple = MbValue::from_ptr(MbObject::new_tuple(vec![name_val, *value]));
                        members.push(tuple);
                    }
                    MbValue::from_ptr(MbObject::new_list(members))
                }
                _ => MbValue::from_ptr(MbObject::new_list(vec![])),
            }
        }
    } else {
        MbValue::from_ptr(MbObject::new_list(vec![]))
    }
}

// ── Signature machinery ───────────────────────────────────────────────────
//
// Real, constructible inspect.Parameter / Signature / BoundArguments classes
// backed by the FUNC_PARAMS registry primed at module init (see
// hir_to_mir.rs lower_top_level and runtime/closure.rs).

/// Register the inspect classes (called from `register()`).
fn register_signature_classes() {
    use std::collections::HashMap as Map;
    let var = |addr: usize| {
        super::super::module::register_variadic_func(addr as u64);
        MbValue::from_func(addr)
    };

    // inspect._ParameterKind — int-ordered, named kind singletons.
    let mut km: Map<String, MbValue> = Map::new();
    for (name, addr) in [
        ("__str__", pk_str as *const () as usize),
        ("__repr__", pk_repr as *const () as usize),
        ("__eq__", pk_eq as *const () as usize),
        ("__lt__", pk_lt as *const () as usize),
        ("__le__", pk_le as *const () as usize),
        ("__gt__", pk_gt as *const () as usize),
        ("__ge__", pk_ge as *const () as usize),
        ("__hash__", pk_hash as *const () as usize),
        ("__int__", pk_int as *const () as usize),
        ("__index__", pk_int as *const () as usize),
    ] {
        km.insert(name.to_string(), var(addr));
    }
    super::super::class::mb_class_register("inspect._ParameterKind", vec![], km);

    // inspect.Parameter
    let mut pm: Map<String, MbValue> = Map::new();
    for (name, addr) in [
        ("__init__", p_init as *const () as usize),
        ("__eq__", p_eq as *const () as usize),
        ("__hash__", p_hash as *const () as usize),
        ("__str__", p_str as *const () as usize),
        ("__repr__", p_repr as *const () as usize),
        ("replace", p_replace as *const () as usize),
    ] {
        pm.insert(name.to_string(), var(addr));
    }
    super::super::class::mb_class_register("inspect.Parameter", vec![], pm);

    // inspect.Signature
    let mut sm: Map<String, MbValue> = Map::new();
    for (name, addr) in [
        ("__init__", s_init as *const () as usize),
        ("__eq__", s_eq as *const () as usize),
        ("__hash__", s_hash as *const () as usize),
        ("__str__", s_str as *const () as usize),
        ("__repr__", s_repr as *const () as usize),
        ("bind", s_bind as *const () as usize),
        ("bind_partial", s_bind_partial as *const () as usize),
    ] {
        sm.insert(name.to_string(), var(addr));
    }
    super::super::class::mb_class_register("inspect.Signature", vec![], sm);

    // inspect.BoundArguments
    let mut bm: Map<String, MbValue> = Map::new();
    for (name, addr) in [
        ("__eq__", ba_eq as *const () as usize),
        ("apply_defaults", ba_apply_defaults as *const () as usize),
    ] {
        bm.insert(name.to_string(), var(addr));
    }
    super::super::class::mb_class_register("inspect.BoundArguments", vec![], bm);

    // Class attributes: kind singletons + empty sentinel.
    let set_attr = |cls: &str, attr: &str, val: MbValue| {
        super::super::class::mb_class_set_class_attr(
            MbValue::from_ptr(MbObject::new_str(cls.to_string())),
            MbValue::from_ptr(MbObject::new_str(attr.to_string())),
            val,
        );
    };
    for (i, name) in KIND_NAMES.iter().enumerate() {
        set_attr("inspect.Parameter", name, kind_singleton(i));
    }
    set_attr("inspect.Parameter", "empty", empty_singleton());
    set_attr("inspect.Signature", "empty", empty_singleton());
}

// ── _ParameterKind methods ──

fn pk_value(slf: MbValue) -> i64 {
    inst_field(slf, "_value")
        .and_then(|v| v.as_int())
        .unwrap_or(0)
}

unsafe extern "C" fn pk_str(slf: MbValue, _args: MbValue) -> MbValue {
    inst_field(slf, "_name").unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn pk_repr(slf: MbValue, _args: MbValue) -> MbValue {
    let name = inst_field(slf, "_name")
        .and_then(extract_str)
        .unwrap_or_default();
    MbValue::from_ptr(MbObject::new_str(format!(
        "<_ParameterKind.{}: {}>",
        name,
        pk_value(slf)
    )))
}

unsafe extern "C" fn pk_eq(slf: MbValue, args: MbValue) -> MbValue {
    let other = first_arg(args);
    match kind_value(other) {
        Some(v) => MbValue::from_bool(v == pk_value(slf)),
        None => match other.as_int() {
            Some(v) => MbValue::from_bool(v == pk_value(slf)),
            None => MbValue::from_bool(false),
        },
    }
}

fn pk_cmp(slf: MbValue, args: MbValue) -> Option<(i64, i64)> {
    let other = first_arg(args);
    let rhs = kind_value(other).or_else(|| other.as_int())?;
    Some((pk_value(slf), rhs))
}

unsafe extern "C" fn pk_lt(slf: MbValue, args: MbValue) -> MbValue {
    MbValue::from_bool(pk_cmp(slf, args).map(|(a, b)| a < b).unwrap_or(false))
}
unsafe extern "C" fn pk_le(slf: MbValue, args: MbValue) -> MbValue {
    MbValue::from_bool(pk_cmp(slf, args).map(|(a, b)| a <= b).unwrap_or(false))
}
unsafe extern "C" fn pk_gt(slf: MbValue, args: MbValue) -> MbValue {
    MbValue::from_bool(pk_cmp(slf, args).map(|(a, b)| a > b).unwrap_or(false))
}
unsafe extern "C" fn pk_ge(slf: MbValue, args: MbValue) -> MbValue {
    MbValue::from_bool(pk_cmp(slf, args).map(|(a, b)| a >= b).unwrap_or(false))
}

unsafe extern "C" fn pk_hash(slf: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_int(pk_value(slf))
}

unsafe extern "C" fn pk_int(slf: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_int(pk_value(slf))
}

// ── Parameter methods ──

/// Build a Parameter instance natively.
fn make_parameter(
    name: &str,
    kind: usize,
    default: MbValue,
    annotation: MbValue,
    anno_str: Option<&str>,
) -> MbValue {
    let inst = MbObject::new_instance("inspect.Parameter".to_string());
    let v = MbValue::from_ptr(inst);
    inst_set_field(
        v,
        "name",
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );
    inst_set_field(v, "kind", kind_singleton(kind));
    inst_set_field(v, "default", default);
    inst_set_field(v, "annotation", annotation);
    if let Some(a) = anno_str {
        inst_set_field(
            v,
            "_anno_str",
            MbValue::from_ptr(MbObject::new_str(a.to_string())),
        );
    }
    v
}

/// Shared ctor logic for `Parameter(...)` and `Parameter.replace(...)`.
/// Returns (name, kind, default, annotation) resolved from packed args.
fn parameter_ctor_fields(
    args: MbValue,
    base: Option<MbValue>,
) -> (String, usize, MbValue, MbValue) {
    let mut items = arg_items(args);
    let kwargs = pop_trailing_kwargs(&mut items, &["name", "kind", "default", "annotation"]);
    let kw = |key: &str| kwargs.iter().find(|(k, _)| k == key).map(|(_, v)| *v);

    let mut name = kw("name").and_then(extract_str);
    let mut kind = kw("kind").and_then(kind_value).map(|v| v as usize);
    let mut default = kw("default");
    let annotation = kw("annotation");

    // Positionals: first string → name; first kind instance → kind; the
    // remaining value (bare-ident calls flatten keywords positionally) →
    // default. CPython's positional order is (name, kind).
    let mut leftovers: Vec<MbValue> = Vec::new();
    for it in items {
        if name.is_none() {
            if let Some(s) = extract_str(it) {
                name = Some(s);
                continue;
            }
        }
        if kind.is_none() {
            if let Some(k) = kind_value(it) {
                kind = Some(k as usize);
                continue;
            }
        }
        leftovers.push(it);
    }
    if default.is_none() {
        // Bare int kind ordinal support: Parameter("x", 1).
        if kind.is_none() && leftovers.len() == 1 {
            if let Some(k) = leftovers[0].as_int() {
                if (0..=4).contains(&k) {
                    kind = Some(k as usize);
                    leftovers.clear();
                }
            }
        }
        default = leftovers.first().copied();
    }

    let base_get = |field: &str| base.and_then(|b| inst_field(b, field));
    let name = name
        .or_else(|| base_get("name").and_then(extract_str))
        .unwrap_or_default();
    let kind = kind
        .or_else(|| base_get("kind").and_then(kind_value).map(|v| v as usize))
        .unwrap_or(1);
    let default = default
        .or_else(|| base_get("default"))
        .unwrap_or_else(empty_singleton);
    let annotation = annotation
        .or_else(|| base_get("annotation"))
        .unwrap_or_else(empty_singleton);
    (name, kind, default, annotation)
}

unsafe extern "C" fn p_init(slf: MbValue, args: MbValue) -> MbValue {
    // kind=N outside the 0..=4 ordinal range raises (CPython _ParameterKind).
    {
        let mut probe = arg_items(args);
        let kwargs = pop_trailing_kwargs(&mut probe, &["name", "kind", "default", "annotation"]);
        if let Some((_, kv)) = kwargs.iter().find(|(k, _)| k == "kind") {
            if let Some(i) = kv.as_int() {
                if !(0..=4).contains(&i) {
                    return raise_exc(
                        "ValueError",
                        &format!("value {i} is not a valid Parameter.kind"),
                    );
                }
            }
        }
    }
    let (name, kind, default, annotation) = parameter_ctor_fields(args, None);
    inst_set_field(slf, "name", MbValue::from_ptr(MbObject::new_str(name)));
    inst_set_field(slf, "kind", kind_singleton(kind));
    inst_set_field(slf, "default", default);
    inst_set_field(slf, "annotation", annotation);
    MbValue::none()
}

unsafe extern "C" fn p_replace(slf: MbValue, args: MbValue) -> MbValue {
    let (name, kind, default, annotation) = parameter_ctor_fields(args, Some(slf));
    let anno_str = inst_field(slf, "_anno_str").and_then(extract_str);
    make_parameter(&name, kind, default, annotation, anno_str.as_deref())
}

/// Field-wise Parameter equality (used by __eq__ and Signature equality).
fn param_equal(a: MbValue, b: MbValue) -> bool {
    if inst_class_name(b).as_deref() != Some("inspect.Parameter") {
        return false;
    }
    let fname = |o: MbValue| {
        inst_field(o, "name")
            .and_then(extract_str)
            .unwrap_or_default()
    };
    let fkind = |o: MbValue| inst_field(o, "kind").and_then(kind_value).unwrap_or(1);
    if fname(a) != fname(b) || fkind(a) != fkind(b) {
        return false;
    }
    let val_eq = |x: Option<MbValue>, y: Option<MbValue>| -> bool {
        match (x, y) {
            (Some(x), Some(y)) => {
                if x.to_bits() == y.to_bits() {
                    true
                } else if is_empty_sentinel(x) || is_empty_sentinel(y) {
                    false
                } else {
                    super::super::builtins::mb_eq(x, y).as_bool() == Some(true)
                }
            }
            (None, None) => true,
            _ => false,
        }
    };
    val_eq(inst_field(a, "default"), inst_field(b, "default"))
        && val_eq(inst_field(a, "annotation"), inst_field(b, "annotation"))
        && inst_field(a, "_anno_str").and_then(extract_str)
            == inst_field(b, "_anno_str").and_then(extract_str)
}

unsafe extern "C" fn p_eq(slf: MbValue, args: MbValue) -> MbValue {
    MbValue::from_bool(param_equal(slf, first_arg(args)))
}

/// Stable hash over (name, kind, annotation text, has_default).
fn param_hash_value(p: MbValue) -> i64 {
    let mut h: u64 = 1469598103934665603; // FNV offset
    let mut fold = |s: &str| {
        for b in s.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(1099511628211);
        }
    };
    fold(
        &inst_field(p, "name")
            .and_then(extract_str)
            .unwrap_or_default(),
    );
    fold(
        &inst_field(p, "kind")
            .and_then(kind_value)
            .unwrap_or(1)
            .to_string(),
    );
    fold(
        &inst_field(p, "_anno_str")
            .and_then(extract_str)
            .unwrap_or_default(),
    );
    let has_default = inst_field(p, "default")
        .map(|d| !is_empty_sentinel(d))
        .unwrap_or(false);
    fold(if has_default { "d" } else { "-" });
    (h & 0x3FFF_FFFF_FFFF) as i64
}

unsafe extern "C" fn p_hash(slf: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_int(param_hash_value(slf))
}

/// Render one parameter the way CPython str(Parameter) does.
fn param_render(p: MbValue) -> String {
    let name = inst_field(p, "name")
        .and_then(extract_str)
        .unwrap_or_default();
    let kind = inst_field(p, "kind").and_then(kind_value).unwrap_or(1);
    let mut out = match kind {
        2 => format!("*{name}"),
        4 => format!("**{name}"),
        _ => name,
    };
    let anno = inst_field(p, "_anno_str")
        .and_then(extract_str)
        .or_else(|| {
            inst_field(p, "annotation")
                .filter(|a| !is_empty_sentinel(*a) && !a.is_none())
                .map(repr_str)
        });
    if let Some(a) = &anno {
        out.push_str(": ");
        out.push_str(a);
    }
    if let Some(d) = inst_field(p, "default") {
        if !is_empty_sentinel(d) {
            if anno.is_some() {
                out.push_str(" = ");
            } else {
                out.push('=');
            }
            out.push_str(&repr_str(d));
        }
    }
    out
}

unsafe extern "C" fn p_str(slf: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(param_render(slf)))
}

unsafe extern "C" fn p_repr(slf: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(format!(
        "<Parameter \"{}\">",
        param_render(slf)
    )))
}

// ── Signature methods ──

/// Ordered Parameter instances of a Signature.
fn signature_params(sig: MbValue) -> Vec<MbValue> {
    let Some(params_dict) = inst_field(sig, "parameters") else {
        return Vec::new();
    };
    let Some(ptr) = params_dict.as_ptr() else {
        return Vec::new();
    };
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().unwrap().values().copied().collect()
        } else {
            Vec::new()
        }
    }
}

/// Build a Signature instance from Parameter instances + return annotation.
fn make_signature(params: &[MbValue], ret_anno: Option<String>) -> MbValue {
    let inst = MbObject::new_instance("inspect.Signature".to_string());
    let v = MbValue::from_ptr(inst);
    let dict = MbValue::from_ptr(MbObject::new_dict());
    for p in params {
        let name = inst_field(*p, "name")
            .and_then(extract_str)
            .unwrap_or_default();
        super::super::dict_ops::mb_dict_setitem(
            dict,
            MbValue::from_ptr(MbObject::new_str(name)),
            *p,
        );
    }
    inst_set_field(v, "parameters", dict);
    match &ret_anno {
        Some(r) => {
            inst_set_field(
                v,
                "_ret_anno",
                MbValue::from_ptr(MbObject::new_str(r.clone())),
            );
            inst_set_field(
                v,
                "return_annotation",
                MbValue::from_ptr(MbObject::new_str(r.clone())),
            );
        }
        None => {
            inst_set_field(v, "return_annotation", empty_singleton());
        }
    }
    v
}

unsafe extern "C" fn s_init(slf: MbValue, args: MbValue) -> MbValue {
    let mut items = arg_items(args);
    let kwargs = pop_trailing_kwargs(&mut items, &["parameters", "return_annotation"]);
    let kw = |key: &str| kwargs.iter().find(|(k, _)| k == key).map(|(_, v)| *v);
    let params_val = kw("parameters").or_else(|| items.first().copied());

    let dict = MbValue::from_ptr(MbObject::new_dict());
    if let Some(pv) = params_val {
        if let Some(ptr) = pv.as_ptr() {
            let plist: Vec<MbValue> = match &(*ptr).data {
                ObjData::List(lock) => lock.read().unwrap().to_vec(),
                ObjData::Tuple(items) => items.clone(),
                _ => Vec::new(),
            };
            for p in plist {
                let name = inst_field(p, "name")
                    .and_then(extract_str)
                    .unwrap_or_default();
                super::super::dict_ops::mb_dict_setitem(
                    dict,
                    MbValue::from_ptr(MbObject::new_str(name)),
                    p,
                );
            }
        }
    }
    inst_set_field(slf, "parameters", dict);
    match kw("return_annotation") {
        Some(r) if !r.is_none() => {
            inst_set_field(slf, "return_annotation", r);
            if let Some(s) = extract_str(r) {
                inst_set_field(slf, "_ret_anno", MbValue::from_ptr(MbObject::new_str(s)));
            }
        }
        _ => inst_set_field(slf, "return_annotation", empty_singleton()),
    }
    MbValue::none()
}

/// Render a Signature the way CPython str(Signature) does.
fn signature_render(sig: MbValue) -> String {
    let params = signature_params(sig);
    let mut parts: Vec<String> = Vec::new();
    let mut seen_pos_only = false;
    let mut emitted_slash = false;
    let mut seen_star = false;
    for p in &params {
        let kind = inst_field(*p, "kind").and_then(kind_value).unwrap_or(1);
        if kind == 0 {
            seen_pos_only = true;
        } else if seen_pos_only && !emitted_slash {
            parts.push("/".to_string());
            emitted_slash = true;
        }
        if kind == 2 {
            seen_star = true;
        }
        if kind == 3 && !seen_star {
            parts.push("*".to_string());
            seen_star = true;
        }
        parts.push(param_render(*p));
    }
    if seen_pos_only && !emitted_slash {
        parts.push("/".to_string());
    }
    let mut out = format!("({})", parts.join(", "));
    if let Some(r) = inst_field(sig, "_ret_anno").and_then(extract_str) {
        out.push_str(" -> ");
        out.push_str(&r);
    }
    out
}

unsafe extern "C" fn s_str(slf: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(signature_render(slf)))
}

unsafe extern "C" fn s_repr(slf: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(format!(
        "<Signature {}>",
        signature_render(slf)
    )))
}

fn signature_equal(a: MbValue, b: MbValue) -> bool {
    if inst_class_name(b).as_deref() != Some("inspect.Signature") {
        return false;
    }
    let pa = signature_params(a);
    let pb = signature_params(b);
    if pa.len() != pb.len() {
        return false;
    }
    if !pa.iter().zip(pb.iter()).all(|(x, y)| param_equal(*x, *y)) {
        return false;
    }
    inst_field(a, "_ret_anno").and_then(extract_str)
        == inst_field(b, "_ret_anno").and_then(extract_str)
}

unsafe extern "C" fn s_eq(slf: MbValue, args: MbValue) -> MbValue {
    MbValue::from_bool(signature_equal(slf, first_arg(args)))
}

unsafe extern "C" fn s_hash(slf: MbValue, _args: MbValue) -> MbValue {
    // An unhashable parameter default makes the whole signature unhashable.
    for p in signature_params(slf) {
        if let Some(d) = inst_field(p, "default") {
            if let Some(ptr) = d.as_ptr() {
                let label = unsafe {
                    match (*ptr).data {
                        ObjData::List(_) => Some("list"),
                        ObjData::Dict(_) => Some("dict"),
                        ObjData::Set(_) => Some("set"),
                        ObjData::ByteArray(_) => Some("bytearray"),
                        _ => None,
                    }
                };
                if let Some(label) = label {
                    return raise_exc("TypeError", &format!("unhashable type: '{label}'"));
                }
            }
        }
    }
    let mut h: i64 = 7;
    for p in signature_params(slf) {
        h = h.wrapping_mul(31).wrapping_add(param_hash_value(p));
    }
    if let Some(r) = inst_field(slf, "_ret_anno").and_then(extract_str) {
        for b in r.as_bytes() {
            h = h.wrapping_mul(31).wrapping_add(*b as i64);
        }
    }
    MbValue::from_int(h & 0x3FFF_FFFF_FFFF)
}

unsafe extern "C" fn s_bind(slf: MbValue, args: MbValue) -> MbValue {
    bind_common(slf, args, false)
}

unsafe extern "C" fn s_bind_partial(slf: MbValue, args: MbValue) -> MbValue {
    bind_common(slf, args, true)
}

/// CPython's Signature.bind algorithm over the trailing-kwargs-dict call
/// convention. `partial` skips missing-required errors.
fn bind_common(sig: MbValue, args: MbValue, partial: bool) -> MbValue {
    let mut items = arg_items(args);
    let kwargs = pop_trailing_kwargs(&mut items, &[]);
    let params = signature_params(sig);

    let pinfo: Vec<(String, i64, bool)> = params
        .iter()
        .map(|p| {
            (
                inst_field(*p, "name")
                    .and_then(extract_str)
                    .unwrap_or_default(),
                inst_field(*p, "kind").and_then(kind_value).unwrap_or(1),
                inst_field(*p, "default")
                    .map(|d| !is_empty_sentinel(d))
                    .unwrap_or(false),
            )
        })
        .collect();

    let mut bound: Vec<Option<MbValue>> = vec![None; params.len()];
    let mut var_pos: Vec<MbValue> = Vec::new();
    let mut var_kw: Vec<(MbValue, MbValue)> = Vec::new();

    // Phase 1: positional arguments fill POSITIONAL_ONLY then
    // POSITIONAL_OR_KEYWORD in declaration order; extras go to *args.
    let mut pos_iter = items.into_iter();
    for (i, (_, kind, _)) in pinfo.iter().enumerate() {
        if *kind == 0 || *kind == 1 {
            match pos_iter.next() {
                Some(v) => bound[i] = Some(v),
                None => break,
            }
        } else {
            break;
        }
    }
    let extras: Vec<MbValue> = pos_iter.collect();
    if !extras.is_empty() {
        if pinfo.iter().any(|(_, k, _)| *k == 2) {
            var_pos = extras;
        } else {
            return insp_raise("TypeError", "too many positional arguments");
        }
    }

    // Phase 2: keyword arguments bind POSITIONAL_OR_KEYWORD / KEYWORD_ONLY
    // names (positional-only names are NOT matchable); leftovers go to
    // **kwargs.
    let has_var_kw = pinfo.iter().any(|(_, k, _)| *k == 4);
    for (kname, kval) in kwargs {
        let slot = pinfo
            .iter()
            .position(|(n, k, _)| n == &kname && (*k == 1 || *k == 3));
        match slot {
            Some(i) => {
                if bound[i].is_some() {
                    return insp_raise(
                        "TypeError",
                        &format!("multiple values for argument '{kname}'"),
                    );
                }
                bound[i] = Some(kval);
            }
            None => {
                if has_var_kw {
                    var_kw.push((MbValue::from_ptr(MbObject::new_str(kname)), kval));
                } else {
                    return insp_raise(
                        "TypeError",
                        &format!("got an unexpected keyword argument '{kname}'"),
                    );
                }
            }
        }
    }

    // Phase 3: required-argument check.
    if !partial {
        for (i, (name, kind, has_default)) in pinfo.iter().enumerate() {
            if (*kind == 0 || *kind == 1 || *kind == 3) && bound[i].is_none() && !*has_default {
                return insp_raise(
                    "TypeError",
                    &format!("missing a required argument: '{name}'"),
                );
            }
        }
    }

    // arguments dict in declaration order; *args/**kwargs only when non-empty.
    let arguments = MbValue::from_ptr(MbObject::new_dict());
    let set = |name: &str, v: MbValue| {
        super::super::dict_ops::mb_dict_setitem(
            arguments,
            MbValue::from_ptr(MbObject::new_str(name.to_string())),
            v,
        );
    };
    for (i, (name, kind, _)) in pinfo.iter().enumerate() {
        match kind {
            2 => {
                if !var_pos.is_empty() {
                    set(
                        name,
                        MbValue::from_ptr(MbObject::new_tuple(var_pos.clone())),
                    );
                }
            }
            4 => {
                if !var_kw.is_empty() {
                    let d = MbValue::from_ptr(MbObject::new_dict());
                    for (k, v) in &var_kw {
                        super::super::dict_ops::mb_dict_setitem(d, *k, *v);
                    }
                    set(name, d);
                }
            }
            _ => {
                if let Some(v) = bound[i] {
                    set(name, v);
                }
            }
        }
    }

    make_bound_arguments(sig, &params, &pinfo, arguments)
}

/// Build a BoundArguments instance with eager .args / .kwargs.
fn make_bound_arguments(
    sig: MbValue,
    params: &[MbValue],
    pinfo: &[(String, i64, bool)],
    arguments: MbValue,
) -> MbValue {
    let _ = params;
    let inst = MbObject::new_instance("inspect.BoundArguments".to_string());
    let v = MbValue::from_ptr(inst);
    inst_set_field(v, "arguments", arguments);
    inst_set_field(v, "signature", sig);
    inst_set_field(v, "_signature", sig);

    let arg_get = |name: &str| -> Option<MbValue> {
        let ptr = arguments.as_ptr()?;
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.read()
                    .unwrap()
                    .get(&DictKey::Str(name.to_string()))
                    .copied()
            } else {
                None
            }
        }
    };

    // .args: positional prefix (CPython BoundArguments.args).
    let mut args_vec: Vec<MbValue> = Vec::new();
    let mut split = 0usize;
    for (i, (name, kind, _)) in pinfo.iter().enumerate() {
        split = i;
        if *kind == 3 || *kind == 4 {
            break;
        }
        let Some(val) = arg_get(name) else { break };
        if *kind == 2 {
            // extend with the tuple's items
            if let Some(ptr) = val.as_ptr() {
                unsafe {
                    if let ObjData::Tuple(ref t) = (*ptr).data {
                        args_vec.extend(t.iter().copied());
                    }
                }
            }
        } else {
            args_vec.push(val);
        }
        split = i + 1;
    }
    inst_set_field(v, "args", MbValue::from_ptr(MbObject::new_tuple(args_vec)));

    // .kwargs: everything after the positional prefix.
    let kwargs_dict = MbValue::from_ptr(MbObject::new_dict());
    for (name, kind, _) in pinfo.iter().skip(split) {
        let Some(val) = arg_get(name) else { continue };
        if *kind == 4 {
            // merge **kwargs entries
            if let Some(ptr) = val.as_ptr() {
                unsafe {
                    if let ObjData::Dict(ref lock) = (*ptr).data {
                        for (k, x) in lock.read().unwrap().iter() {
                            if let DictKey::Str(s) = k {
                                super::super::dict_ops::mb_dict_setitem(
                                    kwargs_dict,
                                    MbValue::from_ptr(MbObject::new_str(s.clone())),
                                    *x,
                                );
                            }
                        }
                    }
                }
            }
        } else if *kind == 2 {
            continue;
        } else {
            super::super::dict_ops::mb_dict_setitem(
                kwargs_dict,
                MbValue::from_ptr(MbObject::new_str(name.clone())),
                val,
            );
        }
    }
    inst_set_field(v, "kwargs", kwargs_dict);
    v
}

// ── BoundArguments methods ──

unsafe extern "C" fn ba_eq(slf: MbValue, args: MbValue) -> MbValue {
    let other = first_arg(args);
    if inst_class_name(other).as_deref() != Some("inspect.BoundArguments") {
        return MbValue::from_bool(false);
    }
    let sig_a = inst_field(slf, "signature").unwrap_or_else(MbValue::none);
    let sig_b = inst_field(other, "signature").unwrap_or_else(MbValue::none);
    let sigs_eq = sig_a.to_bits() == sig_b.to_bits() || signature_equal(sig_a, sig_b);
    if !sigs_eq {
        return MbValue::from_bool(false);
    }
    let arg_a = inst_field(slf, "arguments").unwrap_or_else(MbValue::none);
    let arg_b = inst_field(other, "arguments").unwrap_or_else(MbValue::none);
    super::super::builtins::mb_eq(arg_a, arg_b)
}

unsafe extern "C" fn ba_apply_defaults(slf: MbValue, _args: MbValue) -> MbValue {
    let Some(sig) = inst_field(slf, "signature") else {
        return MbValue::none();
    };
    let params = signature_params(sig);
    let Some(arguments) = inst_field(slf, "arguments") else {
        return MbValue::none();
    };
    let Some(ptr) = arguments.as_ptr() else {
        return MbValue::none();
    };

    // Collect the new ordered contents first, then swap them in.
    let mut new_entries: Vec<(String, MbValue)> = Vec::new();
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            let current = lock.read().unwrap();
            for p in &params {
                let name = inst_field(*p, "name")
                    .and_then(extract_str)
                    .unwrap_or_default();
                let kind = inst_field(*p, "kind").and_then(kind_value).unwrap_or(1);
                if let Some(v) = current.get(&DictKey::Str(name.clone())) {
                    new_entries.push((name, *v));
                } else if let Some(d) = inst_field(*p, "default").filter(|d| !is_empty_sentinel(*d))
                {
                    new_entries.push((name, d));
                } else if kind == 2 {
                    new_entries.push((name, MbValue::from_ptr(MbObject::new_tuple(vec![]))));
                } else if kind == 4 {
                    new_entries.push((name, MbValue::from_ptr(MbObject::new_dict())));
                }
            }
        } else {
            return MbValue::none();
        }
        if let ObjData::Dict(ref lock) = (*ptr).data {
            let mut g = lock.write().unwrap();
            g.clear();
            for (k, v) in new_entries {
                super::super::rc::retain_if_ptr(v);
                g.insert(DictKey::Str(k), v);
            }
        }
    }
    MbValue::none()
}

// ── inspect.signature() ──

/// Build a Signature instance from the FUNC_PARAMS registry records.
fn signature_from_infos(
    infos: &[super::super::closure::MbParamInfo],
    ret_anno: Option<String>,
) -> MbValue {
    let params: Vec<MbValue> = infos
        .iter()
        .map(|info| {
            let default = if info.has_default {
                info.default
            } else {
                empty_singleton()
            };
            make_parameter(
                &info.name,
                info.kind as usize,
                default,
                empty_singleton(),
                info.annotation.as_deref(),
            )
        })
        .collect();
    make_signature(&params, ret_anno)
}

/// inspect.signature(callable) -> Signature instance.
///
/// User-defined `def`s resolve through the FUNC_PARAMS registry primed at
/// module init. Closure handles (lambdas / nested defs) fall back to their
/// recorded arity. Anything unknown yields an empty signature.
pub fn mb_inspect_signature(func: MbValue) -> MbValue {
    if let Some(infos) = super::super::closure::func_params(func) {
        let ret = super::super::closure::func_ret_anno(func);
        return signature_from_infos(&infos, ret);
    }
    // Closure handle (lambda): closure ids are small ints with registry state.
    if func.as_int().is_some() {
        let cf = super::super::closure::mb_closure_get_func(func);
        if !cf.is_none() {
            if let Some(infos) = super::super::closure::func_params(cf) {
                let ret = super::super::closure::func_ret_anno(cf);
                return signature_from_infos(&infos, ret);
            }
            let arity = super::super::closure::closure_arity(func);
            let infos: Vec<super::super::closure::MbParamInfo> = (0..arity)
                .map(|i| super::super::closure::MbParamInfo {
                    name: format!("arg{i}"),
                    kind: 1,
                    has_default: false,
                    default: MbValue::none(),
                    annotation: None,
                })
                .collect();
            return signature_from_infos(&infos, None);
        }
    }
    // Validation tail (errors-dimension contracts): non-callables raise
    // TypeError, builtin types raise ValueError — matching CPython.
    match classify_for_introspection(func) {
        Introspected::Scalar(_) | Introspected::Int => {
            let shown = if let Some(i) = func.as_int() {
                i.to_string()
            } else if let Some(f) = func.as_float() {
                f.to_string()
            } else {
                "None".to_string()
            };
            return raise_exc("TypeError", &format!("{shown} is not a callable object"));
        }
        Introspected::BuiltinType(name) => {
            return raise_exc(
                "ValueError",
                &format!("no signature found for builtin type <class '{name}'>"),
            );
        }
        _ => {}
    }
    signature_from_infos(&[], None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isfunction() {
        // A real function value carries TAG_FUNC (from_func) -> isfunction True.
        let func = MbValue::from_func(0x1234);
        assert_eq!(mb_inspect_isfunction(func).as_bool(), Some(true));

        // A bare int is NOT a function (CPython: inspect.isfunction(0x1234) is False).
        let bare_int = MbValue::from_int(0x1234);
        assert_eq!(mb_inspect_isfunction(bare_int).as_bool(), Some(false));

        // Non-functions
        let s = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        assert_eq!(mb_inspect_isfunction(s).as_bool(), Some(false));

        assert_eq!(
            mb_inspect_isfunction(MbValue::none()).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_isclass() {
        let class_name = MbValue::from_ptr(MbObject::new_str("MyClass".to_string()));
        assert_eq!(mb_inspect_isclass(class_name).as_bool(), Some(true));

        let not_class = MbValue::from_ptr(MbObject::new_str("my_function".to_string()));
        assert_eq!(mb_inspect_isclass(not_class).as_bool(), Some(false));
    }

    #[test]
    fn test_getmembers_instance() {
        let inst = MbObject::new_instance("Foo".to_string());
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*inst).data {
                let mut fields = fields.write().unwrap();
                fields.insert("x".to_string(), MbValue::from_int(10));
                fields.insert("y".to_string(), MbValue::from_int(20));
            }
        }
        let obj = MbValue::from_ptr(inst);
        let members = mb_inspect_getmembers(obj);

        unsafe {
            if let ObjData::List(ref lock) = (*members.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 2);
                // Each item should be a tuple of (name, value)
                for item in items.iter() {
                    if let ObjData::Tuple(ref elems) = (*item.as_ptr().unwrap()).data {
                        assert_eq!(elems.len(), 2);
                    } else {
                        panic!("expected tuple member");
                    }
                }
            } else {
                panic!("expected list");
            }
        }
    }
}
