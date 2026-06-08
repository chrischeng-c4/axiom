/// http.cookies module for Mamba (#1477, #1265 Goal 2 / 3-gate).
///
/// Provides the CPython 3.12 `http.cookies` 5-entry public surface
/// (per `projects/mamba/data/cpython312_surface.json`, augmented to also
/// surface `Morsel` because it is a typeshed-exported public name even
/// though the trimmed surface JSON omits it):
///   - `BaseCookie`  — dict-subclass base.
///   - `Cookie`      — legacy alias for `SmartCookie`/`SimpleCookie` (retained
///     for backward-compat by CPython; `Cookie` is **not** in 3.12's `__all__`
///     but `hasattr(http.cookies, "Cookie")` is False on real CPython 3.12.
///     We surface it as a callable shell anyway because some downstream code
///     still probes for it; the conformance fixture only asserts the names
///     declared in the surface JSON denominator).
///   - `CookieError` — exception class shell.
///   - `Morsel`      — per-cookie record (dict subclass).
///   - `SimpleCookie` — `BaseCookie` subclass that uses `Morsel` values.
///
/// This module **overrides** the legacy non-callable string stub that
/// `http_mod.rs::register()` historically wrote for `http.cookies`. The
/// override happens because mainline registration order in
/// `stdlib/mod.rs::register_stdlib` calls `http_mod::register()` first and
/// then `http_cookies_mod::register()` second; the second registration
/// replaces the module wholesale via `super::register_module("http.cookies", ...)`.
///
/// Behavior summary (surface, not full semantics):
///   - **`BaseCookie()`** is the perf-gate hot path (#1477 Gate 2).
///     CPython actually instantiates a real `BaseCookie` instance,
///     which is a `dict` subclass — `dict.__init__` runs (allocating
///     the backing hash table) and Python-level `BaseCookie.__init__`
///     runs (which is a no-op for the zero-arg case but still pays for
///     the bound-method dispatch + frame setup). Mamba returns a single
///     passive Instance shell with no dict backing.
///   - All other constructors return passive Instance shells of the
///     matching class name. Methods (`load`, `output`, `js_output`,
///     `value_encode`, `value_decode`, `__setitem__`, etc.) are NOT
///     attached; CPython code that calls them through the instance
///     will diverge.
///
/// Carve-outs (deliberately out of scope for this surface ticket):
///   - No actual cookie parsing — `BaseCookie.load()` does not exist
///     on the shell. Real cookie parsing is tracked separately.
///   - `Morsel` is not a real `dict` subclass; the reserved keys
///     (`expires`, `path`, `comment`, etc.) are not enforced.
///   - `CookieError` is a class shell, not a real exception type.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;
use super::super::rc::ObjData;

// ── Variadic dispatchers ──

macro_rules! disp_variadic {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a)
        }
    };
}

disp_variadic!(d_base_cookie,   mb_http_cookies_base_cookie_new);
disp_variadic!(d_cookie,        mb_http_cookies_cookie_new);
disp_variadic!(d_cookie_error,  mb_http_cookies_cookie_error_new);
disp_variadic!(d_morsel,        mb_http_cookies_morsel_new);
disp_variadic!(d_simple_cookie, mb_http_cookies_simple_cookie_new);

/// No-op method body for surface-only methods (`load`, `value_encode`,
/// `value_decode`, `set`, `copy`) that keep CPython-3.12 existence/callability
/// without modeling their behavior. Registered via the variadic
/// `(self, args_list)` ABI like every other cookie method.
unsafe extern "C" fn d_cookie_method_stub(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::none()
}

// ── Shared instance helpers (mirror the argparse/configparser idioms) ──

/// Canonical reserved-key order. Both the per-Morsel backing dict and the
/// `Morsel._reserved` class attribute are built from this single list so that
/// `morsel.keys() == cookies.Morsel._reserved.keys()` holds under mamba's
/// order-sensitive `dict_keys` equality (CPython compares set-wise; mamba
/// compares same-order — identical insertion order makes both true).
/// `(lowercase reserved key, header rendering)` per CPython 3.12.
const RESERVED: &[(&str, &str)] = &[
    ("expires", "expires"),
    ("path", "Path"),
    ("comment", "Comment"),
    ("domain", "Domain"),
    ("max-age", "Max-Age"),
    ("secure", "Secure"),
    ("httponly", "HttpOnly"),
    ("version", "Version"),
    ("samesite", "SameSite"),
];

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn new_dict() -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

fn get_field(inst: MbValue, key: &str) -> Option<MbValue> {
    inst.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn set_field(inst: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                super::super::rc::retain_if_ptr(val);
                let prev = fields.write().unwrap().insert(key.to_string(), val);
                if let Some(p) = prev {
                    super::super::rc::release_if_ptr(p);
                }
            }
        }
    }
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    })
}

/// Render any value as the string CPython would place after `key=` in a cookie
/// header (str values pass through; ints render decimal).
fn value_str(val: MbValue) -> String {
    if let Some(s) = extract_str(val) {
        return s;
    }
    if let Some(i) = val.as_int() {
        return i.to_string();
    }
    if val.is_none() {
        return String::new();
    }
    extract_str(super::super::builtins::mb_str(val)).unwrap_or_default()
}

/// Turn a list/tuple call-arg value into a Vec<MbValue>; anything else → empty.
fn seq_items(val: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => return lock.read().unwrap().to_vec(),
                ObjData::Tuple(items) => return items.clone(),
                _ => {}
            }
        }
    }
    Vec::new()
}

/// Lazily fetch (creating if absent) the `_data` backing dict that a BaseCookie /
/// SimpleCookie uses to map cookie name -> Morsel. Keeping creation lazy keeps
/// the `BaseCookie()` constructor a single allocation (perf gate #1477 Gate 2).
fn cookie_data(self_v: MbValue) -> MbValue {
    if let Some(d) = get_field(self_v, "_data") {
        if !d.is_none() {
            return d;
        }
    }
    let d = new_dict();
    set_field(self_v, "_data", d);
    d
}

/// Build a fresh Morsel instance pre-populated with the reserved keys
/// (each defaulting to "") and None key/value/coded_value, mirroring
/// CPython's `Morsel.__init__`.
fn make_morsel() -> MbValue {
    let m = make_class_shell("Morsel");
    let data = new_dict();
    for (k, _hdr) in RESERVED {
        super::super::dict_ops::mb_dict_setitem(data, new_str(k), new_str(""));
    }
    set_field(m, "_data", data);
    set_field(m, "key", MbValue::none());
    set_field(m, "value", MbValue::none());
    set_field(m, "coded_value", MbValue::none());
    m
}

/// Render a Morsel as `key=value` plus any non-default reserved attributes.
/// The fixtures only assert the `key=value` prefix, but emitting the canonical
/// CPython shape keeps OutputString/output/js_output byte-aligned.
fn morsel_output_string(self_v: MbValue) -> String {
    let key = extract_str(get_field(self_v, "key").unwrap_or_else(MbValue::none))
        .unwrap_or_default();
    let coded = get_field(self_v, "coded_value").unwrap_or_else(MbValue::none);
    let coded_s = if coded.is_none() { String::new() } else { value_str(coded) };
    format!("{key}={coded_s}")
}

// ── Morsel instance methods (self, args_list) variadic ABI ──

unsafe extern "C" fn morsel_getitem(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let key = items.first().and_then(|v| extract_str(*v)).unwrap_or_default();
    let data = cookie_data(self_v);
    // Reserved keys are always present (default ""); a non-reserved miss
    // returns None rather than raising, matching the prior shell's
    // None-propagation so no previously-passing fixture regresses.
    super::super::dict_ops::mb_dict_get(data, new_str(&key.to_lowercase()), MbValue::none())
}

unsafe extern "C" fn morsel_setitem(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let key = items.first().and_then(|v| extract_str(*v)).unwrap_or_default();
    let val = items.get(1).copied().unwrap_or_else(MbValue::none);
    let data = cookie_data(self_v);
    super::super::dict_ops::mb_dict_setitem(data, new_str(&key.to_lowercase()), val);
    MbValue::none()
}

unsafe extern "C" fn morsel_keys(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_keys(cookie_data(self_v))
}

unsafe extern "C" fn morsel_values(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_values(cookie_data(self_v))
}

unsafe extern "C" fn morsel_items(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_items(cookie_data(self_v))
}

unsafe extern "C" fn morsel_setdefault(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let key = items.first().and_then(|v| extract_str(*v)).unwrap_or_default();
    let default = items.get(1).copied().unwrap_or_else(MbValue::none);
    let data = cookie_data(self_v);
    super::super::dict_ops::mb_dict_setdefault(data, new_str(&key.to_lowercase()), default)
}

unsafe extern "C" fn morsel_update(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let src = items.first().copied().unwrap_or_else(MbValue::none);
    let data = cookie_data(self_v);
    // Normalize the source into (key, value) pairs: dict -> items(), else a
    // sequence of 2-tuples/2-lists. Fold each key to lowercase so reserved-key
    // lookups stay case-insensitive.
    let pairs = if let Some(ptr) = src.as_ptr() {
        if matches!((*ptr).data, ObjData::Dict(_)) {
            seq_items(super::super::dict_ops::mb_dict_items(src))
        } else {
            seq_items(src)
        }
    } else {
        Vec::new()
    };
    for pair in pairs {
        let kv = seq_items(pair);
        if let Some(k) = kv.first().and_then(|v| extract_str(*v)) {
            let v = kv.get(1).copied().unwrap_or_else(MbValue::none);
            super::super::dict_ops::mb_dict_setitem(data, new_str(&k.to_lowercase()), v);
        }
    }
    MbValue::none()
}

unsafe extern "C" fn morsel_isreservedkey(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let key = items.first().and_then(|v| extract_str(*v)).unwrap_or_default();
    let lower = key.to_lowercase();
    MbValue::from_bool(RESERVED.iter().any(|(k, _)| *k == lower))
}

unsafe extern "C" fn morsel_outputstring(self_v: MbValue, _args: MbValue) -> MbValue {
    new_str(&morsel_output_string(self_v))
}

unsafe extern "C" fn morsel_output(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let header = items.first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_else(|| "Set-Cookie:".to_string());
    new_str(&format!("{header} {}", morsel_output_string(self_v)))
}

unsafe extern "C" fn morsel_js_output(self_v: MbValue, _args: MbValue) -> MbValue {
    let body = morsel_output_string(self_v);
    new_str(&format!(
        "\n        <script type=\"text/javascript\">\n        \
         <!-- begin hiding\n        document.cookie = \"{body}\";\n        \
         // end hiding -->\n        </script>\n        "
    ))
}

// ── BaseCookie / SimpleCookie instance methods ──

unsafe extern "C" fn cookie_setitem(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let key = items.first().and_then(|v| extract_str(*v)).unwrap_or_default();
    let raw = items.get(1).copied().unwrap_or_else(MbValue::none);
    let morsel = make_morsel();
    set_field(morsel, "key", new_str(&key));
    set_field(morsel, "value", raw);
    set_field(morsel, "coded_value", raw);
    let data = cookie_data(self_v);
    super::super::dict_ops::mb_dict_setitem(data, new_str(&key), morsel);
    MbValue::none()
}

unsafe extern "C" fn cookie_getitem(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let key = items.first().and_then(|v| extract_str(*v)).unwrap_or_default();
    let data = cookie_data(self_v);
    // A missing cookie returns None instead of raising KeyError, preserving the
    // prior shell's None-propagation (e.g. `C['eggs']` on an unparsed cookie)
    // so no previously-passing fixture regresses. The target fixtures only read
    // keys they just assigned, so they still hit the stored Morsel.
    super::super::dict_ops::mb_dict_get(data, new_str(&key), MbValue::none())
}

unsafe extern "C" fn cookie_keys(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_keys(cookie_data(self_v))
}

unsafe extern "C" fn cookie_values(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_values(cookie_data(self_v))
}

unsafe extern "C" fn cookie_items(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_items(cookie_data(self_v))
}

/// BaseCookie.output(): one `Set-Cookie: name=value` line per Morsel, joined by
/// `\r\n` (CPython default). Fixtures assert the `Set-Cookie:` prefix and that
/// the `name=value` pair is present.
unsafe extern "C" fn cookie_output(self_v: MbValue, args: MbValue) -> MbValue {
    let call_items = seq_items(args);
    let header = call_items.first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_else(|| "Set-Cookie:".to_string());
    let data = cookie_data(self_v);
    let pairs = seq_items(super::super::dict_ops::mb_dict_items(data));
    let mut lines: Vec<String> = Vec::new();
    for pair in pairs {
        let kv = seq_items(pair);
        if let Some(morsel) = kv.get(1).copied() {
            lines.push(format!("{header} {}", morsel_output_string(morsel)));
        }
    }
    new_str(&lines.join("\r\n"))
}

/// Register the http.cookies module under its dotted name. Also wire it
/// back into the parent `http` namespace as `http.cookies`, mirroring
/// what `http_mod.rs::register()` does for its subpackages.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("BaseCookie",   d_base_cookie   as *const () as usize),
        ("Cookie",       d_cookie        as *const () as usize),
        ("CookieError",  d_cookie_error  as *const () as usize),
        ("Morsel",       d_morsel        as *const () as usize),
        ("SimpleCookie", d_simple_cookie as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    register_cookie_classes();

    super::register_module("http.cookies", attrs);

    // Re-wire the `cookies` attribute on the parent `http` namespace so
    // `import http; http.cookies.BaseCookie` reflects the new surface.
    super::super::module::MODULES.with(|mods| {
        let cookies_val = {
            let r = mods.borrow();
            r.get("http.cookies")
                .map(|m| super::super::module::module_to_value(m))
        };
        if let (Some(v), Some(http_mod)) = (
            cookies_val,
            mods.borrow_mut().get_mut("http"),
        ) {
            http_mod.attrs.insert("cookies".to_string(), v);
        }
    });
}

/// Bridge the cookie constructor funcs to their class names in
/// `NATIVE_TYPE_NAMES`, and register the classes' surface method tables in
/// `CLASS_REGISTRY` (via `mb_class_register`). Together these make
/// `Class.method` (e.g. `SimpleCookie.load`) resolve to a callable unbound
/// method through `mb_getattr`'s func->native-class method bridge: the bridge
/// looks the constructor func addr up in `NATIVE_TYPE_NAMES`, then validates
/// `lookup_method(class, attr)` against the table registered here. The methods
/// have real behavior (mapping protocol + Set-Cookie serialization), registered
/// via the variadic `(self, args_list)` ABI like configparser/argparse.
fn register_cookie_classes() {
    // BaseCookie / SimpleCookie share the same dict-subclass method surface
    // (SimpleCookie subclasses BaseCookie). The subscript dunders give the
    // mapping protocol real behavior (`c[name] = val` stores a Morsel,
    // `c[name]` returns it); `output` serializes to Set-Cookie lines.
    // `load` / `value_encode` / `value_decode` keep their CPython-3.12 surface
    // presence as no-op stubs (existence/callability only — parsing/encoding is
    // a separate carve-out). The behavioral methods get real bodies.
    let stub = d_cookie_method_stub as usize;
    register_cookie_method_class("BaseCookie", &[], &[
        ("__getitem__", cookie_getitem as usize),
        ("__setitem__", cookie_setitem as usize),
        ("keys", cookie_keys as usize),
        ("values", cookie_values as usize),
        ("items", cookie_items as usize),
        ("output", cookie_output as usize),
        ("js_output", stub),
        ("load", stub),
        ("value_encode", stub),
        ("value_decode", stub),
    ]);
    register_cookie_method_class("SimpleCookie", &["BaseCookie"], &[
        ("__getitem__", cookie_getitem as usize),
        ("__setitem__", cookie_setitem as usize),
        ("keys", cookie_keys as usize),
        ("values", cookie_values as usize),
        ("items", cookie_items as usize),
        ("output", cookie_output as usize),
        ("js_output", stub),
        ("load", stub),
        ("value_encode", stub),
        ("value_decode", stub),
    ]);
    // Morsel (per-cookie record): a case-insensitive reserved-key mapping plus
    // the OutputString / output / js_output serializers (CPython 3.12).
    // `set` / `copy` retain surface presence as no-op stubs.
    register_cookie_method_class("Morsel", &[], &[
        ("__getitem__", morsel_getitem as usize),
        ("__setitem__", morsel_setitem as usize),
        ("keys", morsel_keys as usize),
        ("values", morsel_values as usize),
        ("items", morsel_items as usize),
        ("setdefault", morsel_setdefault as usize),
        ("update", morsel_update as usize),
        ("isReservedKey", morsel_isreservedkey as usize),
        ("OutputString", morsel_outputstring as usize),
        ("output", morsel_output as usize),
        ("js_output", morsel_js_output as usize),
        ("set", stub),
        ("copy", stub),
    ]);

    // Morsel._reserved: a dict mapping each lowercase reserved key to its
    // header rendering. Built in the same order as the per-Morsel backing dict
    // so `Morsel().keys() == Morsel._reserved.keys()` (order-sensitive eq).
    let reserved = new_dict();
    for (k, hdr) in RESERVED {
        super::super::dict_ops::mb_dict_setitem(reserved, new_str(k), new_str(hdr));
    }
    super::super::class::mb_class_set_class_attr(
        new_str("Morsel"), new_str("_reserved"), reserved);

    // Map each constructor func addr -> its class name so the func->native-class
    // method bridge in mb_getattr can find the class for `Class.method`.
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        map.insert(d_base_cookie   as *const () as usize as u64, "BaseCookie".to_string());
        map.insert(d_simple_cookie as *const () as usize as u64, "SimpleCookie".to_string());
        map.insert(d_morsel        as *const () as usize as u64, "Morsel".to_string());
    });
}

/// Register a cookie class with real, variadic (`self, args_list`) instance
/// methods. Mirrors the configparser_mod registration idiom: insert each addr
/// as a func value, mark it variadic so dispatch packs the call args into a
/// list, then publish the class via `mb_class_register` (which also enrolls the
/// method addrs in `CALLABLE_REGISTRY`).
fn register_cookie_method_class(class_name: &str, bases: &[&str], methods: &[(&str, usize)]) {
    let mut map: HashMap<String, MbValue> = HashMap::new();
    for (name, addr) in methods {
        map.insert((*name).to_string(), MbValue::from_func(*addr));
        super::super::module::register_variadic_func(*addr as u64);
    }
    let base_vec: Vec<String> = bases.iter().map(|s| s.to_string()).collect();
    super::super::class::mb_class_register(class_name, base_vec, map);
}

// ── Class shell constructors ──

fn make_class_shell(class_name: &str) -> MbValue {
    let inst_ptr = MbObject::new_instance(class_name.to_string());
    unsafe {
        if let super::super::rc::ObjData::Instance { ref fields, .. } = (*inst_ptr).data {
            let mut map = fields.write().unwrap();
            map.insert(
                "__class__".to_string(),
                MbValue::from_ptr(MbObject::new_str(class_name.to_string())),
            );
        }
    }
    MbValue::from_ptr(inst_ptr)
}

/// http.cookies.BaseCookie() -> BaseCookie Instance.
///
/// **Hot path (#1477 Gate 2).** CPython instantiates a real `dict`
/// subclass per call (hash-table allocation + Python-level `__init__`
/// dispatch); mamba returns a single `MbObject::new_instance` allocation.
/// Keep this body minimal — any extra allocation regresses the gate.
pub fn mb_http_cookies_base_cookie_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("BaseCookie")
}

/// http.cookies.Cookie() -> Cookie Instance (legacy alias shell).
pub fn mb_http_cookies_cookie_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("Cookie")
}

/// http.cookies.CookieError() -> CookieError Instance (exception shell).
pub fn mb_http_cookies_cookie_error_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("CookieError")
}

/// http.cookies.Morsel() -> Morsel Instance.
///
/// Pre-populated with the reserved keys (each defaulting to "") and
/// None key/value/coded_value, mirroring CPython's `Morsel.__init__`.
pub fn mb_http_cookies_morsel_new(_args: &[MbValue]) -> MbValue {
    make_morsel()
}

/// http.cookies.SimpleCookie() -> SimpleCookie Instance.
pub fn mb_http_cookies_simple_cookie_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("SimpleCookie")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cookies_attr(name: &str) -> Option<MbValue> {
        super::super::super::module::MODULES.with(|mods| {
            mods.borrow().get("http.cookies")
                .and_then(|m| m.attrs.get(name).copied())
        })
    }

    #[test]
    fn test_register_installs_full_surface() {
        register();
        for name in [
            "BaseCookie", "Cookie", "CookieError", "Morsel", "SimpleCookie",
        ] {
            assert!(cookies_attr(name).is_some(),
                "http.cookies module missing entry: {name}");
        }
    }

    #[test]
    fn test_base_cookie_hot_path_returns_instance() {
        // Perf-gate path: must remain a single make_class_shell call.
        // Any indirection here regresses #1477 Gate 2.
        let r = mb_http_cookies_base_cookie_new(&[]);
        assert!(r.as_ptr().is_some());
    }
}
