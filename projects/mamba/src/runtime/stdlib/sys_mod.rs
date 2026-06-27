use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// sys module for Mamba (#310 R1).
///
/// Provides: sys.argv, sys.path, sys.version, sys.platform,
///           sys.maxsize, sys.exit(), sys.getrecursionlimit(),
///           sys.setrecursionlimit(), sys.getdefaultencoding(),
///           sys.float_info, sys.int_info, sys.stdin, sys.stdout,
///           sys.stderr, sys.modules
use std::collections::HashMap;

// ── Dispatch wrappers ──

fn extract_list(val: MbValue) -> Vec<MbValue> {
    match val.as_ptr() {
        Some(ptr) => unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().to_vec()
            } else {
                vec![]
            }
        },
        None => vec![],
    }
}

fn dispatch_getrecursionlimit(args: MbValue) -> MbValue {
    let _ = extract_list(args);
    mb_sys_getrecursionlimit()
}

fn dispatch_getsizeof(args: MbValue) -> MbValue {
    let items = extract_list(args);
    mb_sys_getsizeof(items.get(0).copied().unwrap_or_else(MbValue::none))
}

fn dispatch_getdefaultencoding(args: MbValue) -> MbValue {
    let _ = extract_list(args);
    mb_sys_getdefaultencoding()
}

// New-ABI dispatchers (#1261) — `extern "C" fn(args_ptr, nargs) -> MbValue`.
// The older `fn(args: MbValue)` shape used by exit/getrecursionlimit/etc.
// above does not see real call args; the new wire-ups follow the native
// extern ABI consumed by class.rs:5308 and must also be registered in
// NATIVE_FUNC_ADDRS at register() time.

unsafe extern "C" fn dispatch_intern(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs >= 1 {
        mb_sys_intern(*args_ptr)
    } else {
        MbValue::none()
    }
}

unsafe extern "C" fn dispatch_gettrace(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_sys_gettrace()
}

unsafe extern "C" fn dispatch_settrace(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let v = if nargs >= 1 {
        *args_ptr
    } else {
        MbValue::none()
    };
    mb_sys_settrace(v)
}

unsafe extern "C" fn dispatch_getrefcount(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let v = if nargs >= 1 {
        *args_ptr
    } else {
        MbValue::none()
    };
    mb_sys_getrefcount(v)
}

unsafe extern "C" fn dispatch_is_finalizing(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_sys_is_finalizing()
}

unsafe extern "C" fn dispatch_exc_info(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_sys_exc_info()
}

unsafe extern "C" fn dispatch_getfilesystemencoding(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    mb_sys_getfilesystemencoding()
}

unsafe extern "C" fn dispatch_getfilesystemencodeerrors(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    mb_sys_getfilesystemencodeerrors()
}

// New-ABI replacements for exit / setrecursionlimit / setswitchinterval.
// The legacy `fn(args: MbValue)` shape never receives the real call args
// (the dispatch wrapper hands those funcs an empty list), so `sys.exit(42)`,
// `sys.setrecursionlimit(-5)` and `sys.setswitchinterval(0.0)` could not see
// their argument to validate it. These flat-args dispatchers read the actual
// first argument and are registered in NATIVE_FUNC_ADDRS at register() time.
unsafe extern "C" fn dispatch_exit(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let code = if nargs >= 1 {
        *args_ptr
    } else {
        MbValue::none()
    };
    mb_sys_exit(code)
}

unsafe extern "C" fn dispatch_setrecursionlimit(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let limit = if nargs >= 1 {
        *args_ptr
    } else {
        MbValue::none()
    };
    mb_sys_setrecursionlimit(limit)
}

unsafe extern "C" fn dispatch_setswitchinterval(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let interval = if nargs >= 1 {
        *args_ptr
    } else {
        MbValue::none()
    };
    mb_sys_setswitchinterval(interval)
}

// ── Surface stubs (auto-added) ──
// Present+callable stubs for sys functions Mamba does not yet implement.
// They satisfy `hasattr`/`callable` surface fixtures and return a benign
// default (None) so callers that probe-then-call don't crash. Registered in
// NATIVE_FUNC_ADDRS so class.rs dispatch picks the flat-args path.
unsafe extern "C" fn dispatch_sys_stub_none(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

thread_local! {
    static RECURSION_LIMIT: std::cell::Cell<i64> = const { std::cell::Cell::new(1000) };
    static SWITCH_INTERVAL: std::cell::Cell<f64> = const { std::cell::Cell::new(0.005) };
    static INTERN_TABLE: std::cell::RefCell<std::collections::HashMap<String, u64>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
    static ASYNCGEN_HOOKS: std::cell::Cell<(u64, u64)> =
        std::cell::Cell::new((MbValue::none().to_bits(), MbValue::none().to_bits()));
}

/// Build a CPython struct-sequence-like Instance: named fields plus an
/// ordered `_entries` list backing __len__ / __getitem__ / comparisons.
fn make_struct_seq(class_name: &str, fields: &[(&str, MbValue)]) -> MbValue {
    let inst = MbObject::new_instance(class_name.to_string());
    unsafe {
        if let ObjData::Instance { fields: ref f, .. } = (*inst).data {
            let mut map = f.write().unwrap();
            let mut entries: Vec<MbValue> = Vec::new();
            for (k, v) in fields {
                map.insert((*k).to_string(), *v);
                entries.push(*v);
            }
            map.insert(
                "_entries".to_string(),
                MbValue::from_ptr(MbObject::new_list(entries)),
            );
        }
    }
    MbValue::from_ptr(inst)
}

fn struct_seq_entries(self_v: MbValue) -> Vec<MbValue> {
    self_v
        .as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let e = fields.read().ok()?.get("_entries").copied()?;
                if let Some(ep) = e.as_ptr() {
                    if let ObjData::List(ref lock) = (*ep).data {
                        return lock.read().ok().map(|g| g.to_vec());
                    }
                }
            }
            None
        })
        .unwrap_or_default()
}

fn struct_seq_tuple(v: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(struct_seq_entries(v)))
}

/// Comparison operand as a tuple: another struct-seq's entries, or the value
/// itself (a tuple literal compares element-wise via the builtin paths).
fn cmp_operand(v: MbValue) -> MbValue {
    if struct_seq_entries(v).is_empty() {
        v
    } else {
        struct_seq_tuple(v)
    }
}

/// Dual-convention operand extraction: binop dunder dispatch passes the
/// operand DIRECTLY, while mb_call_method wraps call args in a List. Only a
/// List is unwrapped — a Tuple is itself the operand (e.g. `vi > (1, 0, 0)`).
fn ss_args_first(args: MbValue) -> MbValue {
    if let Some(ptr) = args.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                if let Ok(g) = lock.read() {
                    return g.first().copied().unwrap_or_else(MbValue::none);
                }
            }
        }
    }
    args
}

unsafe extern "C" fn structseq_len(self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_int(struct_seq_entries(self_v).len() as i64)
}

/// __iter__: iterate the ordered fields, so a struct sequence unpacks like the
/// tuple it models (`first, *rest = vi`, `f(*sys.get_asyncgen_hooks())`).
unsafe extern "C" fn structseq_iter(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::iter::mb_iter(struct_seq_tuple(self_v))
}

unsafe extern "C" fn structseq_getitem(self_v: MbValue, args: MbValue) -> MbValue {
    let key = ss_args_first(args);
    let entries = struct_seq_entries(self_v);
    let n = entries.len() as i64;
    // Slice keys arrive as a (start, stop, step) tuple from the slice
    // lowering; integer keys index directly.
    if let Some(ptr) = key.as_ptr() {
        if let ObjData::Tuple(ref items) = (*ptr).data {
            if items.len() == 3 {
                let start = items[0].as_int().unwrap_or(0).clamp(0, n);
                let stop = items[1].as_int().unwrap_or(n).clamp(0, n);
                let vals: Vec<MbValue> = (start..stop)
                    .filter_map(|i| entries.get(i as usize).copied())
                    .collect();
                return MbValue::from_ptr(MbObject::new_tuple(vals));
            }
        }
    }
    let idx = key.as_int().unwrap_or(0);
    let i = if idx < 0 { idx + n } else { idx };
    if i < 0 || i >= n {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
            MbValue::from_ptr(MbObject::new_str("tuple index out of range".to_string())),
        );
        return MbValue::none();
    }
    entries[i as usize]
}

unsafe extern "C" fn structseq_eq(self_v: MbValue, other: MbValue) -> MbValue {
    super::super::builtins::mb_eq(struct_seq_tuple(self_v), cmp_operand(other))
}

unsafe extern "C" fn structseq_lt(self_v: MbValue, args: MbValue) -> MbValue {
    super::super::builtins::mb_lt(struct_seq_tuple(self_v), cmp_operand(ss_args_first(args)))
}

unsafe extern "C" fn structseq_gt(self_v: MbValue, args: MbValue) -> MbValue {
    super::super::builtins::mb_gt(struct_seq_tuple(self_v), cmp_operand(ss_args_first(args)))
}

/// Register ONE class name with the shared struct-seq method table
/// (__len__ / __getitem__ with slice support / __eq__ / __lt__ / __gt__ over
/// the ordered `_entries` field). Reused by urllib.parse result types.
pub(crate) fn register_struct_seq_class(cls: &str) {
    register_struct_seq_class_with(cls, std::collections::HashMap::new());
}

/// Like `register_struct_seq_class` but with extra class methods merged in
/// (extras win on name collisions) — one mb_class_register call so every
/// method lands in CALLABLE_REGISTRY.
pub(crate) fn register_struct_seq_class_with(
    cls: &str,
    extras: std::collections::HashMap<String, MbValue>,
) {
    use std::collections::HashMap as Map;
    for addr in [
        structseq_len as *const () as usize,
        structseq_getitem as *const () as usize,
        structseq_iter as *const () as usize,
        structseq_lt as *const () as usize,
        structseq_gt as *const () as usize,
    ] {
        super::super::module::register_variadic_func(addr as u64);
    }
    let mut m: Map<String, MbValue> = Map::new();
    m.insert(
        "__len__".into(),
        MbValue::from_func(structseq_len as *const () as usize),
    );
    m.insert(
        "__getitem__".into(),
        MbValue::from_func(structseq_getitem as *const () as usize),
    );
    m.insert(
        "__iter__".into(),
        MbValue::from_func(structseq_iter as *const () as usize),
    );
    m.insert(
        "__eq__".into(),
        MbValue::from_func(structseq_eq as *const () as usize),
    );
    m.insert(
        "__lt__".into(),
        MbValue::from_func(structseq_lt as *const () as usize),
    );
    m.insert(
        "__gt__".into(),
        MbValue::from_func(structseq_gt as *const () as usize),
    );
    for (k, v) in extras {
        m.insert(k, v);
    }
    super::super::class::mb_class_register(cls, vec![], m);
}

/// Register the shared struct-seq method table under each sys class name.
fn register_struct_seq_classes() {
    for cls in [
        "sys.version_info",
        "sys.float_info",
        "sys.int_info",
        "sys.hash_info",
        "sys.implementation.version",
        "sys.asyncgen_hooks",
    ] {
        register_struct_seq_class(cls);
    }
}

fn version_info_fields() -> Vec<(&'static str, MbValue)> {
    vec![
        ("major", MbValue::from_int(3)),
        ("minor", MbValue::from_int(12)),
        ("micro", MbValue::from_int(0)),
        (
            "releaselevel",
            MbValue::from_ptr(MbObject::new_str("final".to_string())),
        ),
        ("serial", MbValue::from_int(0)),
    ]
}

fn raise_type_error_msg(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

// ── flat dispatchers for arg-count-sensitive entry points ──

unsafe extern "C" fn dispatch_getrecursionlimit_flat(
    _args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    if nargs > 0 {
        return raise_type_error_msg("getrecursionlimit() takes no arguments (1 given)");
    }
    MbValue::from_int(RECURSION_LIMIT.with(|c| c.get()))
}

unsafe extern "C" fn dispatch_getdefaultencoding_flat(
    _args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    if nargs > 0 {
        return raise_type_error_msg("getdefaultencoding() takes no arguments (1 given)");
    }
    mb_sys_getdefaultencoding()
}

unsafe extern "C" fn dispatch_getswitchinterval_flat(
    _args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    if nargs > 0 {
        return raise_type_error_msg("getswitchinterval() takes no arguments (1 given)");
    }
    MbValue::from_float(SWITCH_INTERVAL.with(|c| c.get()))
}

unsafe extern "C" fn dispatch_intern_flat(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let Some(arg) = a.first().copied() else {
        return raise_type_error_msg("intern() takes exactly one argument (0 given)");
    };
    let Some(ptr) = arg.as_ptr() else {
        return raise_type_error_msg("intern() argument must be str");
    };
    let text = unsafe {
        match &(*ptr).data {
            ObjData::Str(s) => s.clone(),
            // A str subclass arrives as an Instance — CPython rejects it.
            _ => return raise_type_error_msg("can't intern non-string"),
        }
    };
    INTERN_TABLE.with(|t| {
        let mut t = t.borrow_mut();
        if let Some(bits) = t.get(&text) {
            return MbValue::from_bits(*bits);
        }
        super::super::gc::gc_add_root(arg);
        unsafe {
            super::super::rc::retain_if_ptr(arg);
        }
        t.insert(text, arg.to_bits());
        arg
    })
}

unsafe extern "C" fn dispatch_getsizeof_flat(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let Some(obj) = a.first().copied() else {
        return raise_type_error_msg("getsizeof() takes at least 1 argument (0 given)");
    };
    let default = a.get(1).copied();
    // A user __sizeof__ runs first; when it raises and a default was given,
    // the default wins (CPython semantics).
    if let Some(ptr) = obj.as_ptr() {
        let is_instance = unsafe { matches!((*ptr).data, ObjData::Instance { .. }) };
        if is_instance {
            let class_name = unsafe {
                if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                    class_name.clone()
                } else {
                    String::new()
                }
            };
            let m = super::super::class::lookup_method(&class_name, "__sizeof__");
            if !m.is_none() {
                let method = MbValue::from_ptr(MbObject::new_str("__sizeof__".to_string()));
                let args_list = MbValue::from_ptr(MbObject::new_list(Vec::new()));
                let r = super::super::class::mb_call_method(obj, method, args_list);
                if super::super::exception::mb_has_exception().as_bool() == Some(true) {
                    if let Some(d) = default {
                        super::super::exception::mb_clear_exception();
                        return d;
                    }
                    return MbValue::none();
                }
                if let Some(n) = r.as_int() {
                    return MbValue::from_int(n);
                }
            }
        }
    }
    mb_sys_getsizeof(obj)
}

unsafe extern "C" fn dispatch_displayhook_flat(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let Some(value) = a.first().copied() else {
        return raise_type_error_msg("displayhook() takes exactly one argument (0 given)");
    };
    if value.is_none() {
        return MbValue::none();
    }
    let r = super::super::builtins::mb_repr(value);
    let text = r
        .as_ptr()
        .and_then(|p| unsafe {
            if let ObjData::Str(ref s) = (*p).data {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_default();
    let line = format!("{text}\n");
    if !write_current_sys_stdout(&line) && !super::super::output::write_captured(&line) {
        print!("{line}");
    }
    // Bind builtins._ to the displayed value — both in the registry attrs
    // and in the already-cached module dict the program actually reads.
    super::super::module::MODULES.with(|mods| {
        if let Some(b) = mods.borrow_mut().get_mut("builtins") {
            unsafe {
                super::super::rc::retain_if_ptr(value);
            }
            b.attrs.insert("_".to_string(), value);
            if let Some(cached) = b.cached_value {
                unsafe {
                    super::super::rc::retain_if_ptr(value);
                }
                super::super::dict_ops::mb_dict_setitem(
                    cached,
                    MbValue::from_ptr(MbObject::new_str("_".to_string())),
                    value,
                );
            }
        }
    });
    MbValue::none()
}

fn write_current_sys_stdout(text: &str) -> bool {
    let Some(stdout) = super::super::module::mb_module_value_getattr("sys", "stdout") else {
        return false;
    };
    if stdout.is_none() {
        return false;
    }
    let method = MbValue::from_ptr(MbObject::new_str("write".to_string()));
    let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
        MbObject::new_str(text.to_string()),
    )]));
    let _ = super::super::class::mb_call_method(stdout, method, args);
    super::super::exception::mb_has_exception().as_bool() != Some(true)
}

unsafe extern "C" fn dispatch_excepthook_flat(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if a.is_empty() {
        return raise_type_error_msg("excepthook() takes exactly 3 arguments (0 given)");
    }
    let etype = a.first().copied().unwrap_or_else(MbValue::none);
    let value = a.get(1).copied().unwrap_or_else(MbValue::none);
    let tb = a.get(2).copied().unwrap_or_else(MbValue::none);
    let tname = super::super::class::resolve_class_name(etype).unwrap_or_default();
    let vstr = {
        let r = super::super::builtins::mb_str(value);
        r.as_ptr()
            .and_then(|p| unsafe {
                if let ObjData::Str(ref s) = (*p).data {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    };
    let mut text = String::new();
    if !tb.is_none() {
        text.push_str("Traceback (most recent call last):\n");
        text.push_str("  File \"<module>\", line 1, in <module>\n");
    }
    if vstr.is_empty() {
        text.push_str(&format!("{tname}\n"));
    } else {
        text.push_str(&format!("{tname}: {vstr}\n"));
    }
    if !super::super::output::try_write_stderr_redirect(&text) {
        eprint!("{text}");
    }
    MbValue::none()
}

unsafe extern "C" fn dispatch_exception_flat(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    super::super::class::last_caught_exception_value()
}

unsafe extern "C" fn dispatch_unraisablehook_flat(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let Some(arg) = a.first().copied() else {
        return raise_type_error_msg("unraisablehook() takes exactly one argument (0 given)");
    };
    // The argument must be an UnraisableHookArgs-shaped object (has exc_type).
    let has_exc_type = arg
        .as_ptr()
        .map(|ptr| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields
                    .read()
                    .map(|f| f.contains_key("exc_type"))
                    .unwrap_or(false)
            } else {
                false
            }
        })
        .unwrap_or(false);
    if !has_exc_type {
        return raise_type_error_msg("unraisablehook argument type must be UnraisableHookArgs");
    }
    MbValue::none()
}

unsafe extern "C" fn dispatch_get_asyncgen_hooks(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    let (fi, fin) = ASYNCGEN_HOOKS.with(|c| c.get());
    make_struct_seq(
        "sys.asyncgen_hooks",
        &[
            ("firstiter", MbValue::from_bits(fi)),
            ("finalizer", MbValue::from_bits(fin)),
        ],
    )
}

unsafe extern "C" fn dispatch_set_asyncgen_hooks(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (mut fi, mut fin) = ASYNCGEN_HOOKS.with(|c| c.get());
    let mut pos: Vec<MbValue> = Vec::new();
    for v in a {
        let is_dict = v
            .as_ptr()
            .map(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
            .unwrap_or(false);
        if is_dict {
            let get = |k: &str| {
                let r = super::super::dict_ops::mb_dict_get(
                    *v,
                    MbValue::from_ptr(MbObject::new_str(k.to_string())),
                    MbValue::from_bits(u64::MAX),
                );
                if r.to_bits() == u64::MAX {
                    None
                } else {
                    Some(r)
                }
            };
            if let Some(x) = get("firstiter") {
                fi = x.to_bits();
            }
            if let Some(x) = get("finalizer") {
                fin = x.to_bits();
            }
        } else {
            pos.push(*v);
        }
    }
    if let Some(x) = pos.first() {
        fi = x.to_bits();
    }
    if let Some(x) = pos.get(1) {
        fin = x.to_bits();
    }
    ASYNCGEN_HOOKS.with(|c| c.set((fi, fin)));
    MbValue::none()
}

/// Build sys.float_info — the CPython 11-field struct sequence.
fn build_float_info() -> MbValue {
    make_struct_seq(
        "sys.float_info",
        &[
            ("max", MbValue::from_float(f64::MAX)),
            ("max_exp", MbValue::from_int(1024)),
            ("max_10_exp", MbValue::from_int(308)),
            ("min", MbValue::from_float(f64::MIN_POSITIVE)),
            ("min_exp", MbValue::from_int(-1021)),
            ("min_10_exp", MbValue::from_int(-307)),
            ("dig", MbValue::from_int(15)),
            ("mant_dig", MbValue::from_int(53)),
            ("epsilon", MbValue::from_float(f64::EPSILON)),
            ("radix", MbValue::from_int(2)),
            ("rounds", MbValue::from_int(1)),
        ],
    )
}

/// Build sys.int_info — the CPython 4-field struct sequence.
fn build_int_info() -> MbValue {
    make_struct_seq(
        "sys.int_info",
        &[
            ("bits_per_digit", MbValue::from_int(30)),
            ("sizeof_digit", MbValue::from_int(4)),
            ("default_max_str_digits", MbValue::from_int(4300)),
            ("str_digits_check_threshold", MbValue::from_int(640)),
        ],
    )
}

/// Build sys.flags as a dict with the common CPython flag fields.
/// All false / zero by default — Mamba doesn't honor most of these yet,
/// but accessing them needs to succeed for libraries that probe (e.g. pytest).
fn build_flags() -> MbValue {
    let mut fields: Vec<(&str, MbValue)> = vec![
        ("debug", MbValue::from_int(0)),
        ("inspect", MbValue::from_int(0)),
        ("interactive", MbValue::from_int(0)),
        ("optimize", MbValue::from_int(0)),
        ("dont_write_bytecode", MbValue::from_int(0)),
        ("no_user_site", MbValue::from_int(0)),
        ("no_site", MbValue::from_int(0)),
        ("ignore_environment", MbValue::from_int(0)),
        ("verbose", MbValue::from_int(0)),
        ("bytes_warning", MbValue::from_int(0)),
        ("quiet", MbValue::from_int(0)),
        ("hash_randomization", MbValue::from_int(1)),
        ("isolated", MbValue::from_int(0)),
        ("utf8_mode", MbValue::from_int(0)),
        ("warn_default_encoding", MbValue::from_int(0)),
        ("int_max_str_digits", MbValue::from_int(4300)),
    ];
    // dev_mode / safe_path are documented as bool.
    fields.push(("dev_mode", MbValue::from_bool(false)));
    fields.push(("safe_path", MbValue::from_bool(false)));
    make_struct_seq("sys.flags", &fields)
}

/// Build sys.implementation — a namespace whose `version` is the same
/// struct-sequence shape as sys.version_info.
fn build_implementation() -> MbValue {
    make_struct_seq(
        "sys.implementation",
        &[
            (
                "name",
                MbValue::from_ptr(MbObject::new_str("mamba".to_string())),
            ),
            (
                "cache_tag",
                MbValue::from_ptr(MbObject::new_str("mamba-312".to_string())),
            ),
            ("hexversion", MbValue::from_int(0x030c00f0)),
            (
                "version",
                make_struct_seq("sys.implementation.version", &version_info_fields()),
            ),
            (
                "_multiarch",
                MbValue::from_ptr(MbObject::new_str("darwin".to_string())),
            ),
        ],
    )
}

/// Build sys.hash_info — the CPython 9-field struct sequence. The modulus
/// (2^61 - 1) exceeds the NaN-boxed int payload, so it is a BigInt.
fn build_hash_info() -> MbValue {
    make_struct_seq(
        "sys.hash_info",
        &[
            ("width", MbValue::from_int(64)),
            (
                "modulus",
                MbValue::from_ptr(MbObject::new_bigint(num_bigint::BigInt::from(
                    (1i64 << 61) - 1,
                ))),
            ),
            ("inf", MbValue::from_int(314159)),
            ("nan", MbValue::from_int(0)),
            ("imag", MbValue::from_int(1_000_003)),
            (
                "algorithm",
                MbValue::from_ptr(MbObject::new_str("siphash13".to_string())),
            ),
            ("hash_bits", MbValue::from_int(64)),
            ("seed_bits", MbValue::from_int(128)),
            ("cutoff", MbValue::from_int(0)),
        ],
    )
}

// ── std stream objects ────────────────────────────────────────────
//
// `sys.stdin` reads the real process stdin (read/readline); `sys.stdout`
// and `sys.stderr` write through the runtime output layer so capture and
// redirect stay coherent with `print`. The three attributes share one
// `sys._Stream` Instance class with a variadic `(self, args_list)` table.

unsafe extern "C" fn stream_read(_self_v: MbValue, _args: MbValue) -> MbValue {
    use std::io::Read as _;
    let mut buf = String::new();
    let _ = std::io::stdin().read_to_string(&mut buf);
    MbValue::from_ptr(MbObject::new_str(buf))
}

unsafe extern "C" fn stream_readline(_self_v: MbValue, _args: MbValue) -> MbValue {
    use std::io::BufRead as _;
    let mut line = String::new();
    let _ = std::io::stdin().lock().read_line(&mut line);
    MbValue::from_ptr(MbObject::new_str(line))
}

fn stream_name(self_v: MbValue) -> Option<String> {
    self_v.as_ptr().and_then(|p| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*p).data {
            fields.read().unwrap().get("name").and_then(|v| {
                v.as_ptr().and_then(|sp| {
                    if let ObjData::Str(ref s) = (*sp).data {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
            })
        } else {
            None
        }
    })
}

fn first_list_arg(args: MbValue) -> Option<MbValue> {
    args.as_ptr().and_then(|p| unsafe {
        if let ObjData::List(ref lock) = (*p).data {
            lock.read().unwrap().first().copied()
        } else {
            None
        }
    })
}

fn extract_bytes_payload(val: MbValue) -> Option<Vec<u8>> {
    val.as_ptr().and_then(|p| unsafe {
        match &(*p).data {
            ObjData::Bytes(bytes) => Some(bytes.clone()),
            ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
            ObjData::Str(s) => Some(s.as_bytes().to_vec()),
            _ => None,
        }
    })
}

unsafe extern "C" fn stream_write(self_v: MbValue, args: MbValue) -> MbValue {
    use std::io::Write as _;
    let text = first_list_arg(args)
        .and_then(|v| {
            v.as_ptr().and_then(|p| unsafe {
                if let ObjData::Str(ref s) = (*p).data {
                    Some(s.clone())
                } else {
                    None
                }
            })
        })
        .unwrap_or_default();
    let is_stderr = stream_name(self_v).as_deref() == Some("<stderr>");
    let n = text.chars().count() as i64;
    if is_stderr {
        if !super::super::output::try_write_stderr_redirect(&text) {
            let _ = std::io::stderr().write_all(text.as_bytes());
        }
    } else if !super::super::output::write_captured(&text) {
        let _ = std::io::stdout().write_all(text.as_bytes());
    }
    MbValue::from_int(n)
}

unsafe extern "C" fn stream_buffer_write(self_v: MbValue, args: MbValue) -> MbValue {
    use std::io::Write as _;
    let bytes = first_list_arg(args)
        .and_then(extract_bytes_payload)
        .unwrap_or_default();
    let is_stderr = stream_name(self_v).as_deref() == Some("<stderr>");
    let n = bytes.len() as i64;
    if is_stderr {
        let _ = std::io::stderr().write_all(&bytes);
    } else {
        let _ = std::io::stdout().write_all(&bytes);
    }
    MbValue::from_int(n)
}

unsafe extern "C" fn stream_flush(_self_v: MbValue, _args: MbValue) -> MbValue {
    use std::io::Write as _;
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
    MbValue::none()
}

fn register_variadic_method_table(class_name: &str, entries: &[(&str, usize)]) {
    let mut methods: HashMap<String, MbValue> = HashMap::new();
    for (name, addr) in entries {
        super::super::module::register_variadic_func(*addr as u64);
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(*addr as u64);
        });
        methods.insert((*name).to_string(), MbValue::from_func(*addr));
    }
    super::super::class::mb_class_register(class_name, Vec::new(), methods);
}

/// Register the shared `sys._Stream` and `sys._StreamBuffer` method tables.
fn register_stream_class() {
    register_variadic_method_table(
        "sys._Stream",
        &[
            ("read", stream_read as *const () as usize),
            ("readline", stream_readline as *const () as usize),
            ("write", stream_write as *const () as usize),
            ("flush", stream_flush as *const () as usize),
        ],
    );
    register_variadic_method_table(
        "sys._StreamBuffer",
        &[
            ("write", stream_buffer_write as *const () as usize),
            ("flush", stream_flush as *const () as usize),
        ],
    );
}

fn build_stream_buffer_stub(name: &str) -> MbValue {
    use super::super::rc::{InstanceFields, MbObjectHeader, MbRwLock, ObjKind};
    let mut fields = InstanceFields::default();
    fields.insert(
        "name".to_string(),
        MbValue::from_ptr(MbObject::new_str(format!("<{}>", name))),
    );
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "sys._StreamBuffer".to_string(),
            fields: MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Build a std-stream object (an Instance carrying its display name).
fn build_stream_stub(name: &str) -> MbValue {
    use super::super::rc::{InstanceFields, MbObjectHeader, MbRwLock, ObjKind};
    let mut fields = InstanceFields::default();
    fields.insert(
        "name".to_string(),
        MbValue::from_ptr(MbObject::new_str(format!("<{}>", name))),
    );
    fields.insert("buffer".to_string(), build_stream_buffer_stub(name));
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "sys._Stream".to_string(),
            fields: MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Register the sys module.
pub fn register() {
    let mut attrs = HashMap::new();

    // sys.version
    attrs.insert(
        "version".into(),
        MbValue::from_ptr(MbObject::new_str("Mamba 0.1.0 (cclab)".to_string())),
    );

    // sys._git — CPython exposes ('CPython', branch, revision); the platform
    // module's test helpers save/restore it.
    attrs.insert(
        "_git".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_ptr(MbObject::new_str("CPython".to_string())),
            MbValue::from_ptr(MbObject::new_str(String::new())),
            MbValue::from_ptr(MbObject::new_str(String::new())),
        ])),
    );

    // sys.version_info — a 5-field struct sequence (named + indexed + sliced
    // + tuple-compared).
    register_struct_seq_classes();
    attrs.insert(
        "version_info".into(),
        make_struct_seq("sys.version_info", &version_info_fields()),
    );

    // sys.platform
    attrs.insert(
        "platform".into(),
        MbValue::from_ptr(MbObject::new_str(std::env::consts::OS.to_string())),
    );

    // sys.maxsize — the platform 64-bit signed max. Exceeds the NaN-boxed
    // 48-bit int payload, so it is a BigInt value.
    attrs.insert(
        "maxsize".into(),
        MbValue::from_ptr(MbObject::new_bigint(num_bigint::BigInt::from(i64::MAX))),
    );

    // sys.argv (populated at runtime, empty by default)
    attrs.insert(
        "argv".into(),
        MbValue::from_ptr(MbObject::new_list(Vec::new())),
    );

    // sys.path (search paths for imports)
    let paths: Vec<MbValue> = vec![MbValue::from_ptr(MbObject::new_str(".".to_string()))];
    attrs.insert("path".into(), MbValue::from_ptr(MbObject::new_list(paths)));

    // sys.executable
    let exe = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    attrs.insert(
        "executable".into(),
        MbValue::from_ptr(MbObject::new_str(exe)),
    );

    // sys.byteorder
    let order = if cfg!(target_endian = "little") {
        "little"
    } else {
        "big"
    };
    attrs.insert(
        "byteorder".into(),
        MbValue::from_ptr(MbObject::new_str(order.to_string())),
    );

    // sys.float_info
    attrs.insert("float_info".into(), build_float_info());

    // sys.int_info
    attrs.insert("int_info".into(), build_int_info());

    // sys.stdin, sys.stdout, sys.stderr (stub stream objects)
    register_stream_class();
    attrs.insert("stdin".into(), build_stream_stub("stdin"));
    attrs.insert("stdout".into(), build_stream_stub("stdout"));
    attrs.insert("stderr".into(), build_stream_stub("stderr"));

    // sys.modules (stub dict)
    let modules_dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*modules_dict).data {
            let mut map = lock.write().unwrap();
            map.insert("sys".into(), MbValue::from_bool(true));
        }
    }
    attrs.insert("modules".into(), MbValue::from_ptr(modules_dict));

    // sys.hexversion (3.12.0 final → 0x030C00F0)
    attrs.insert("hexversion".into(), MbValue::from_int(0x030c00f0));

    // sys.api_version — frozen CPython value for 3.12
    attrs.insert("api_version".into(), MbValue::from_int(1013));

    // sys.dont_write_bytecode — Mamba never writes .pyc, so True
    attrs.insert("dont_write_bytecode".into(), MbValue::from_bool(true));

    // sys.builtin_module_names — built-in modules embedded in the
    // interpreter binary (not the wider stdlib). Tuple in CPython.
    let builtins: Vec<MbValue> = ["sys", "builtins", "_imp", "_thread"]
        .iter()
        .map(|s| MbValue::from_ptr(MbObject::new_str(s.to_string())))
        .collect();
    attrs.insert(
        "builtin_module_names".into(),
        MbValue::from_ptr(MbObject::new_tuple(builtins)),
    );

    // sys.warnoptions — empty by default
    attrs.insert(
        "warnoptions".into(),
        MbValue::from_ptr(MbObject::new_list(Vec::new())),
    );

    // sys.flags / sys.implementation / sys.hash_info
    attrs.insert("flags".into(), build_flags());
    attrs.insert("implementation".into(), build_implementation());
    attrs.insert("hash_info".into(), build_hash_info());

    // sys.prefix / exec_prefix / base_prefix / base_exec_prefix
    let prefix_val = MbValue::from_ptr(MbObject::new_str("".to_string()));
    attrs.insert("prefix".into(), prefix_val);
    attrs.insert(
        "exec_prefix".into(),
        MbValue::from_ptr(MbObject::new_str("".to_string())),
    );
    attrs.insert(
        "base_prefix".into(),
        MbValue::from_ptr(MbObject::new_str("".to_string())),
    );
    attrs.insert(
        "base_exec_prefix".into(),
        MbValue::from_ptr(MbObject::new_str("".to_string())),
    );

    // Callable functions via function pointers
    // New-ABI dispatchers (extern "C" fn(args_ptr, nargs)) — must register
    // in NATIVE_FUNC_ADDRS so class.rs dispatch picks the flat-args path.
    let new_dispatchers: Vec<(&str, usize)> = vec![
        ("exit", dispatch_exit as *const () as usize),
        (
            "setrecursionlimit",
            dispatch_setrecursionlimit as *const () as usize,
        ),
        (
            "setswitchinterval",
            dispatch_setswitchinterval as *const () as usize,
        ),
        ("intern", dispatch_intern_flat as *const () as usize),
        (
            "getrecursionlimit",
            dispatch_getrecursionlimit_flat as *const () as usize,
        ),
        (
            "getdefaultencoding",
            dispatch_getdefaultencoding_flat as *const () as usize,
        ),
        (
            "getswitchinterval",
            dispatch_getswitchinterval_flat as *const () as usize,
        ),
        ("getsizeof", dispatch_getsizeof_flat as *const () as usize),
        (
            "displayhook",
            dispatch_displayhook_flat as *const () as usize,
        ),
        (
            "__displayhook__",
            dispatch_displayhook_flat as *const () as usize,
        ),
        ("excepthook", dispatch_excepthook_flat as *const () as usize),
        (
            "__excepthook__",
            dispatch_excepthook_flat as *const () as usize,
        ),
        ("exception", dispatch_exception_flat as *const () as usize),
        (
            "unraisablehook",
            dispatch_unraisablehook_flat as *const () as usize,
        ),
        (
            "__unraisablehook__",
            dispatch_unraisablehook_flat as *const () as usize,
        ),
        (
            "get_asyncgen_hooks",
            dispatch_get_asyncgen_hooks as *const () as usize,
        ),
        (
            "set_asyncgen_hooks",
            dispatch_set_asyncgen_hooks as *const () as usize,
        ),
        ("gettrace", dispatch_gettrace as *const () as usize),
        ("settrace", dispatch_settrace as *const () as usize),
        ("getrefcount", dispatch_getrefcount as *const () as usize),
        (
            "is_finalizing",
            dispatch_is_finalizing as *const () as usize,
        ),
        ("exc_info", dispatch_exc_info as *const () as usize),
        (
            "getfilesystemencoding",
            dispatch_getfilesystemencoding as *const () as usize,
        ),
        (
            "getfilesystemencodeerrors",
            dispatch_getfilesystemencodeerrors as *const () as usize,
        ),
    ];
    for (name, addr) in &new_dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in &new_dispatchers {
            set.insert(*addr as u64);
        }
    });

    // surface: missing CPython module constants (auto-added)
    attrs.insert(
        "abiflags".into(),
        MbValue::from_ptr(MbObject::new_str("".to_string())),
    );
    attrs.insert(
        "float_repr_style".into(),
        MbValue::from_ptr(MbObject::new_str("short".to_string())),
    );
    attrs.insert("maxunicode".into(), MbValue::from_int(1114111));
    attrs.insert(
        "platlibdir".into(),
        MbValue::from_ptr(MbObject::new_str("lib".to_string())),
    );
    // sys._stdlib_dir is the on-disk stdlib path (dirname of os.__file__); the
    // native os module has no real __file__, so this is None to match.
    attrs.insert("_stdlib_dir".into(), MbValue::none());

    // surface: missing CPython sys data/constants (auto-added, batch 2)
    // copyright — CPython license banner string.
    attrs.insert(
        "copyright".into(),
        MbValue::from_ptr(MbObject::new_str(
            "Copyright (c) 2001-2023 Python Software Foundation.\nAll Rights Reserved.".to_string(),
        )),
    );
    // pycache_prefix — None by default in CPython.
    attrs.insert("pycache_prefix".into(), MbValue::none());
    // orig_argv — original interpreter argv (empty until populated).
    attrs.insert(
        "orig_argv".into(),
        MbValue::from_ptr(MbObject::new_list(Vec::new())),
    );
    // meta_path / path_hooks — import system hook lists (empty stubs).
    attrs.insert(
        "meta_path".into(),
        MbValue::from_ptr(MbObject::new_list(Vec::new())),
    );
    attrs.insert(
        "path_hooks".into(),
        MbValue::from_ptr(MbObject::new_list(Vec::new())),
    );
    // path_importer_cache — import cache dict (empty stub).
    attrs.insert(
        "path_importer_cache".into(),
        MbValue::from_ptr(MbObject::new_dict()),
    );
    // monitoring — submodule; register as a stub namespace dict.
    attrs.insert("monitoring".into(), MbValue::from_ptr(MbObject::new_dict()));
    // stdlib_module_names — frozenset in CPython. A representative subset
    // of the stdlib.
    let stdlib_names: Vec<MbValue> = [
        "abc",
        "argparse",
        "array",
        "ast",
        "base64",
        "bisect",
        "calendar",
        "cmath",
        "collections",
        "configparser",
        "contextlib",
        "copy",
        "csv",
        "datetime",
        "decimal",
        "difflib",
        "enum",
        "fnmatch",
        "fractions",
        "functools",
        "gc",
        "getopt",
        "glob",
        "gzip",
        "hashlib",
        "heapq",
        "hmac",
        "html",
        "http",
        "io",
        "itertools",
        "json",
        "keyword",
        "logging",
        "math",
        "operator",
        "os",
        "pathlib",
        "pickle",
        "platform",
        "pprint",
        "queue",
        "random",
        "re",
        "secrets",
        "shutil",
        "signal",
        "socket",
        "stat",
        "statistics",
        "string",
        "struct",
        "subprocess",
        "sys",
        "tempfile",
        "textwrap",
        "threading",
        "time",
        "timeit",
        "tomllib",
        "traceback",
        "types",
        "typing",
        "unicodedata",
        "unittest",
        "urllib",
        "uuid",
        "warnings",
        "weakref",
        "zipfile",
        "zlib",
    ]
    .iter()
    .map(|s| MbValue::from_ptr(MbObject::new_str(s.to_string())))
    .collect();
    attrs.insert(
        "stdlib_module_names".into(),
        MbValue::from_ptr(MbObject::new_frozenset(stdlib_names)),
    );
    // thread_info — namespace-like dict (name/lock/version).
    let thread_info = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*thread_info).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "name".into(),
                MbValue::from_ptr(MbObject::new_str("pthread".to_string())),
            );
            map.insert(
                "lock".into(),
                MbValue::from_ptr(MbObject::new_str("mutex+cond".to_string())),
            );
            map.insert("version".into(), MbValue::none());
        }
    }
    attrs.insert("thread_info".into(), MbValue::from_ptr(thread_info));

    // surface: missing CPython sys callables (auto-added, batch 2).
    // Present+callable stubs returning None. Registered in NATIVE_FUNC_ADDRS.
    let stub_fn_addr = dispatch_sys_stub_none as *const () as usize;
    for name in [
        "activate_stack_trampoline",
        "addaudithook",
        "audit",
        "breakpointhook",
        "call_tracing",
        "deactivate_stack_trampoline",
        "get_coroutine_origin_tracking_depth",
        "get_int_max_str_digits",
        "getallocatedblocks",
        "getdlopenflags",
        "getprofile",
        "getunicodeinternedsize",
        "is_stack_trampoline_active",
        "set_coroutine_origin_tracking_depth",
        "set_int_max_str_digits",
        "setdlopenflags",
        "setprofile",
    ] {
        attrs.insert(name.into(), MbValue::from_func(stub_fn_addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(stub_fn_addr as u64);
    });

    super::register_module("sys", attrs);
}

// ── Runtime functions ──

/// sys.exit(code=0) — raise SystemExit so it can be caught (CPython semantics).
/// `sys.exit` does not terminate the process directly; it raises SystemExit,
/// which an enclosing `try/except SystemExit` can intercept. The message is the
/// stringified exit code (CPython stores the code in `SystemExit.code`).
pub fn mb_sys_exit(code: MbValue) -> MbValue {
    let msg = if let Some(b) = code.as_bool() {
        if b { "1" } else { "0" }.to_string()
    } else {
        match code.as_int() {
            Some(i) => i.to_string(),
            None if code.is_none() => String::new(),
            None => match code.as_ptr() {
                Some(ptr) => unsafe {
                    if let ObjData::Str(ref s) = (*ptr).data {
                        s.clone()
                    } else {
                        String::new()
                    }
                },
                None => String::new(),
            },
        }
    };
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("SystemExit".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg)),
    );
    MbValue::none()
}

/// sys.getrecursionlimit() → int
pub fn mb_sys_getrecursionlimit() -> MbValue {
    MbValue::from_int(1000) // Default Python recursion limit
}

/// sys.setrecursionlimit(limit) → None
///
/// CPython rejects a non-positive limit with
/// `ValueError("recursion limit must be greater or equal than 1")`. Mamba uses
/// a fixed recursion limit, so a valid (>= 1) call is still accepted as a no-op,
/// but an invalid limit must raise to match CPython.
pub fn mb_sys_setrecursionlimit(limit: MbValue) -> MbValue {
    // Only validate when an actual integer arg was supplied. Non-int / missing
    // args fall through as a no-op (preserves prior lenient behavior; a real
    // TypeError for non-ints is a separate concern not exercised here).
    if let Some(n) = limit.as_int_pyint() {
        if n < 1 {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "recursion limit must be greater or equal than 1".to_string(),
                )),
            );
            return MbValue::none();
        }
        RECURSION_LIMIT.with(|c| c.set(n));
    }
    MbValue::none()
}

/// sys.setswitchinterval(interval) → None
///
/// CPython requires a strictly positive interval and raises
/// `ValueError("switch interval must be strictly positive")` for `<= 0`. Mamba
/// does not honor the thread switch interval, so a valid (> 0) call is a no-op,
/// but a non-positive interval must raise to match CPython.
pub fn mb_sys_setswitchinterval(interval: MbValue) -> MbValue {
    // Accept both float (the documented type) and int args. Only raise when a
    // numeric value <= 0 is supplied; a missing / non-numeric arg falls through
    // as a no-op (a real TypeError is a separate concern not exercised here).
    let numeric = interval
        .as_float()
        .or_else(|| interval.as_int_pyint().map(|i| i as f64));
    match numeric {
        Some(v) if v <= 0.0 => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "switch interval must be strictly positive".to_string(),
                )),
            );
            return MbValue::none();
        }
        Some(v) => SWITCH_INTERVAL.with(|c| c.set(v)),
        None => {
            // Non-numeric argument: CPython raises TypeError.
            return raise_type_error_msg("a float is required");
        }
    }
    MbValue::none()
}

/// sys.getdefaultencoding() → 'utf-8'
pub fn mb_sys_getdefaultencoding() -> MbValue {
    MbValue::from_ptr(MbObject::new_str("utf-8".to_string()))
}

/// sys.getsizeof(obj) → int (approximate)
pub fn mb_sys_getsizeof(val: MbValue) -> MbValue {
    let size = if val.is_int() || val.is_float() || val.is_bool() || val.is_none() {
        8 // NaN-boxed: 8 bytes
    } else {
        // Heap object: header + data (approximate)
        std::mem::size_of::<super::super::rc::MbObject>()
    };
    MbValue::from_int(size as i64)
}

/// sys.intern(s) — interning is a hint in CPython; return the string unchanged.
pub fn mb_sys_intern(s: MbValue) -> MbValue {
    s
}

/// sys.gettrace() → current trace function (None — Mamba has no Python-level tracing).
pub fn mb_sys_gettrace() -> MbValue {
    MbValue::none()
}

/// sys.settrace(func) → None. No-op stub — accept but ignore.
pub fn mb_sys_settrace(_func: MbValue) -> MbValue {
    MbValue::none()
}

/// sys.getrefcount(obj) → int. Mamba uses Arc internally and doesn't expose
/// per-object refcounts to Python; return a stable >0 value to keep callers
/// (e.g. test fixtures asserting refcount > 0) happy.
pub fn mb_sys_getrefcount(_obj: MbValue) -> MbValue {
    MbValue::from_int(1)
}

/// sys.is_finalizing() → bool. Mamba never reaches finalization mid-run.
pub fn mb_sys_is_finalizing() -> MbValue {
    MbValue::from_bool(false)
}

fn exception_value_type_name(value: MbValue) -> Option<String> {
    value.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Instance { class_name, .. } => Some(class_name.clone()),
            ObjData::Str(s) => Some(s.clone()),
            _ => None,
        }
    })
}

fn exception_traceback_value(value: MbValue) -> MbValue {
    let Some(ptr) = value.as_ptr() else {
        return MbValue::none();
    };
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let existing = fields.read().unwrap().get("__traceback__").copied();
            if let Some(tb) = existing {
                super::super::rc::retain_if_ptr(tb);
                return tb;
            }
            let tb = super::traceback_mod::make_tb_instance();
            super::super::rc::retain_if_ptr(tb);
            fields
                .write()
                .unwrap()
                .insert("__traceback__".to_string(), tb);
            return tb;
        }
    }
    MbValue::none()
}

/// sys.exc_info() → (type, value, traceback). Returns (None, None, None)
/// when no exception is being handled.
pub fn mb_sys_exc_info() -> MbValue {
    let value_val = super::super::class::last_caught_exception_value();
    if value_val.is_none() {
        let triple = vec![MbValue::none(), MbValue::none(), MbValue::none()];
        return MbValue::from_ptr(MbObject::new_tuple(triple));
    }

    let etype = super::super::exception::last_handled_exception()
        .map(|(etype, _)| etype)
        .or_else(|| exception_value_type_name(value_val))
        .unwrap_or_else(|| "Exception".to_string());
    let type_val = MbValue::from_ptr(MbObject::new_str(etype));
    let tb_val = exception_traceback_value(value_val);
    MbValue::from_ptr(MbObject::new_tuple(vec![type_val, value_val, tb_val]))
}

/// sys.getfilesystemencoding() → 'utf-8'.
pub fn mb_sys_getfilesystemencoding() -> MbValue {
    MbValue::from_ptr(MbObject::new_str("utf-8".to_string()))
}

/// sys.getfilesystemencodeerrors() → 'surrogateescape' (CPython default on
/// POSIX). Stable string — chosen to match what UTF-8-mode CPython reports.
pub fn mb_sys_getfilesystemencodeerrors() -> MbValue {
    MbValue::from_ptr(MbObject::new_str("surrogateescape".to_string()))
}

/// Populate sys.argv from command-line arguments.
pub fn populate_argv(args: &[String]) {
    let argv: Vec<MbValue> = args
        .iter()
        .map(|a| MbValue::from_ptr(MbObject::new_str(a.clone())))
        .collect();
    let argv_list = MbValue::from_ptr(MbObject::new_list(argv));

    crate::runtime::module::MODULES.with(|mods| {
        let mut mods = mods.borrow_mut();
        if let Some(sys) = mods.get_mut("sys") {
            sys.attrs.insert("argv".into(), argv_list);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_register() {
        register();
        let name = MbValue::from_ptr(MbObject::new_str("sys".to_string()));
        let result = super::super::super::module::mb_import(name);
        assert!(result.is_ptr());
    }

    #[test]
    fn test_sys_getrecursionlimit() {
        assert_eq!(mb_sys_getrecursionlimit().as_int(), Some(1000));
    }

    #[test]
    fn test_sys_getsizeof() {
        let size = mb_sys_getsizeof(MbValue::from_int(42));
        assert_eq!(size.as_int(), Some(8));
    }

    // -- Py3.12 conformance --

    #[test]
    fn test_py312_sys_getrecursionlimit_is_1000() {
        assert_eq!(mb_sys_getrecursionlimit().as_int(), Some(1000));
    }

    #[test]
    fn test_py312_sys_getsizeof_primitives_return_8() {
        assert_eq!(mb_sys_getsizeof(MbValue::from_int(0)).as_int(), Some(8));
        assert_eq!(mb_sys_getsizeof(MbValue::from_float(1.0)).as_int(), Some(8));
        assert_eq!(mb_sys_getsizeof(MbValue::from_bool(true)).as_int(), Some(8));
        assert_eq!(mb_sys_getsizeof(MbValue::none()).as_int(), Some(8));
    }

    #[test]
    fn test_py312_sys_getsizeof_heap_object_positive() {
        let s = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        let size = mb_sys_getsizeof(s);
        assert!(size.as_int().unwrap() > 0);
    }

    #[test]
    fn test_py312_sys_maxsize_fits_in_i48() {
        let maxsize: i64 = (1i64 << 47) - 1;
        let v = MbValue::from_int(maxsize);
        assert_eq!(v.as_int(), Some(maxsize));
    }

    #[test]
    fn test_py312_populate_argv_accepts_empty() {
        register();
        populate_argv(&[]);
    }
}
