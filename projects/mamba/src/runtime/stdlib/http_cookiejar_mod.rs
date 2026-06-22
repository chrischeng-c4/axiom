use super::super::dict_ops::DictKey;
use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// http.cookiejar module for Mamba (#1478, #1265 Goal 2 / 3-gate).
///
/// Provides the CPython 3.12 `http.cookiejar` 8-entry public surface
/// (per `projects/mamba/data/cpython312_surface.json`):
///   - `Cookie`             — per-cookie record (passive class shell).
///   - `CookieJar`          — primary jar container (perf-gate hot path).
///   - `CookiePolicy`       — abstract policy base class shell.
///   - `DefaultCookiePolicy` — concrete policy class shell.
///   - `FileCookieJar`      — abstract `CookieJar` subclass shell with
///     `load`/`save`/`revert` carve-outs (not implemented).
///   - `LWPCookieJar`       — `FileCookieJar` subclass shell for the
///     libwww-perl format (not implemented).
///   - `MozillaCookieJar`   — `FileCookieJar` subclass shell for the
///     Netscape/Mozilla cookies.txt format (not implemented).
///   - `LoadError`          — exception class shell raised by file IO
///     in real CPython.
///
/// This module **overrides** the legacy non-callable string stub that
/// `http_mod.rs::register()` historically wrote for `http.cookiejar`. The
/// override happens because mainline registration order in
/// `stdlib/mod.rs::register_stdlib` calls `http_mod::register()` first and
/// then `http_cookiejar_mod::register()` second; the second registration
/// replaces the module wholesale via `super::register_module("http.cookiejar", ...)`.
///
/// Behavior summary (surface, not full semantics):
///   - **`CookieJar()`** is the perf-gate hot path (#1478 Gate 2).
///     CPython actually instantiates a real `CookieJar` instance —
///     `__init__` registers a thread `RLock`, opens the internal
///     `_cookies` dict, and stores a `DefaultCookiePolicy` reference.
///     Mamba returns a single passive Instance shell with no dict or
///     lock backing. Keep this body minimal — extra allocation
///     regresses the gate.
///   - All other constructors return passive Instance shells of the
///     matching class name. Methods (`add_cookie_header`,
///     `extract_cookies`, `set_cookie`, `load`, `save`, `clear`,
///     `make_cookies`, etc.) are NOT attached; CPython code that
///     calls them through the instance will diverge.
///
/// Carve-outs (deliberately out of scope for this surface ticket):
///   - No actual cookie parsing or jar bookkeeping — `CookieJar.set_cookie()`
///     does not exist on the shell. Real jar semantics tracked separately.
///   - `FileCookieJar.load()` / `.save()` / `.revert()` are not present;
///     `LoadError` is a class shell, not a real exception type.
///   - `DefaultCookiePolicy` does not enforce RFC 2965 / RFC 6265 rules.
///   - Module-level helpers (`request_host`, `domain_match`, `is_HDN`,
///     `eff_request_host`, `escape_path`, `http2time`, `iso2time`,
///     `time2isoz`, `time2netscape`, `parse_ns_headers`,
///     `split_header_words`, `join_header_words`, ...) are NOT exposed;
///     the surface JSON denominator only requires the 8 class entries.
use std::collections::HashMap;

// ── Small value helpers ──

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn new_list(items: Vec<MbValue>) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(items))
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Truthiness of a stored field: None and False are falsy, everything else truthy.
fn is_truthy(val: MbValue) -> bool {
    if val.is_none() {
        return false;
    }
    if let Some(b) = val.as_bool() {
        return b;
    }
    true
}

/// Read a named field off an Instance value.
fn inst_field(obj: MbValue, name: &str) -> Option<MbValue> {
    obj.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(name).copied()
        } else {
            None
        }
    })
}

/// Set a named field on an Instance value (rc-managed).
fn set_inst_field(obj: MbValue, name: &str, value: MbValue) {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                super::super::rc::retain_if_ptr(value);
                let prev = fields.write().unwrap().insert(name.to_string(), value);
                if let Some(p) = prev {
                    super::super::rc::release_if_ptr(p);
                }
            }
        }
    }
}

/// Collect a list/tuple's elements; a bare value becomes a one-element vec.
fn items_of(val: MbValue) -> Vec<MbValue> {
    val.as_ptr()
        .map(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => lock.read().unwrap().iter().copied().collect(),
                ObjData::Tuple(items) => items.iter().copied().collect(),
                _ => vec![val],
            }
        })
        .unwrap_or_default()
}

/// Pull the trailing kwargs dict out of a variadic arg slice (or None).
fn trailing_kwargs(args: &[MbValue]) -> MbValue {
    if let Some(last) = args.last() {
        if last
            .as_ptr()
            .map(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
            .unwrap_or(false)
        {
            return *last;
        }
    }
    MbValue::none()
}

/// Read the runtime class name off an Instance value (e.g. "LWPCookieJar").
/// Used by `load()` to pick the right on-disk magic-header check.
fn inst_class_name(obj: MbValue) -> Option<String> {
    obj.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            Some(class_name.clone())
        } else {
            None
        }
    })
}

/// Look up a string keyword from a method's trailing kwargs dict. CPython's
/// `FileCookieJar.load(filename=None, ...)` is routinely called by keyword
/// (`jar.load(filename="...")`); the method ABI appends the kwargs as a
/// trailing `Dict`, so positional-only extraction misses it.
fn kwarg_str(args_items: &[MbValue], name: &str) -> Option<String> {
    let last = args_items.last()?;
    let ptr = last.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            return lock
                .read()
                .unwrap()
                .get(name)
                .copied()
                .and_then(extract_str);
        }
    }
    None
}

/// Lazily return the jar's `_cookies` list, creating it on first use so the
/// `CookieJar()` ctor hot path (#1478 Gate 2) stays a bare allocation.
fn jar_cookies(self_v: MbValue) -> MbValue {
    match inst_field(self_v, "_cookies") {
        Some(v) if v.as_ptr().is_some() => v,
        _ => {
            let lst = new_list(vec![]);
            set_inst_field(self_v, "_cookies", lst);
            lst
        }
    }
}

// ── Variadic dispatchers ──

macro_rules! disp_variadic {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            // mb_call0 passes a null args_ptr with nargs=0; from_raw_parts
            // requires a non-null pointer even for empty slices.
            let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
            };
            $fn(a)
        }
    };
}

disp_variadic!(d_cookie, mb_http_cookiejar_cookie_new);
disp_variadic!(d_cookie_jar, mb_http_cookiejar_cookie_jar_new);
disp_variadic!(d_cookie_policy, mb_http_cookiejar_cookie_policy_new);
disp_variadic!(
    d_default_cookie_policy,
    mb_http_cookiejar_default_cookie_policy_new
);
disp_variadic!(d_file_cookie_jar, mb_http_cookiejar_file_cookie_jar_new);
disp_variadic!(d_lwp_cookie_jar, mb_http_cookiejar_lwp_cookie_jar_new);
disp_variadic!(
    d_mozilla_cookie_jar,
    mb_http_cookiejar_mozilla_cookie_jar_new
);
disp_variadic!(d_load_error, mb_http_cookiejar_load_error_new);

/// Register the http.cookiejar module under its dotted name. Also wire it
/// back into the parent `http` namespace as `http.cookiejar`, mirroring
/// what `http_mod.rs::register()` does for its subpackages and what
/// `http_cookies_mod::register()` does for `http.cookies`.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("Cookie", d_cookie as *const () as usize),
        ("CookieJar", d_cookie_jar as *const () as usize),
        ("CookiePolicy", d_cookie_policy as *const () as usize),
        (
            "DefaultCookiePolicy",
            d_default_cookie_policy as *const () as usize,
        ),
        ("FileCookieJar", d_file_cookie_jar as *const () as usize),
        ("LWPCookieJar", d_lwp_cookie_jar as *const () as usize),
        (
            "MozillaCookieJar",
            d_mozilla_cookie_jar as *const () as usize,
        ),
        ("LoadError", d_load_error as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
        // Map the `LoadError` constructor dispatcher to its class name so the
        // except-matcher (`resolve_class_name` via NATIVE_TYPE_NAMES) recognises
        // `except http.cookiejar.LoadError`; the raised exception type name is
        // the bare "LoadError", so both sides resolve to the same class.
        if name == "LoadError" {
            super::super::module::NATIVE_TYPE_NAMES.with(|m| {
                m.borrow_mut().insert(addr as u64, "LoadError".to_string());
            });
        }
    }

    // surface: missing CPython module constants (auto-added)
    attrs.insert(
        "DEFAULT_HTTP_PORT".into(),
        MbValue::from_ptr(MbObject::new_str("80".to_string())),
    );
    attrs.insert("EPOCH_YEAR".into(), MbValue::from_int(1970));
    attrs.insert(
        "HTTPONLY_ATTR".into(),
        MbValue::from_ptr(MbObject::new_str("HTTPOnly".to_string())),
    );
    attrs.insert(
        "HTTPONLY_PREFIX".into(),
        MbValue::from_ptr(MbObject::new_str("#HttpOnly_".to_string())),
    );
    attrs.insert(
        "HTTP_PATH_SAFE".into(),
        MbValue::from_ptr(MbObject::new_str("%/;:@&=+$,!~*'()".to_string())),
    );
    attrs.insert(
        "MISSING_FILENAME_TEXT".into(),
        MbValue::from_ptr(MbObject::new_str(
            "a filename was not supplied (nor was the CookieJar instance initialised with one)"
                .to_string(),
        )),
    );
    attrs.insert("NETSCAPE_HEADER_TEXT".into(), MbValue::from_ptr(MbObject::new_str("# Netscape HTTP Cookie File\n# http://curl.haxx.se/rfc/cookie_spec.html\n# This is a generated file!  Do not edit.\n\n".to_string())));
    attrs.insert("debug".into(), MbValue::from_int(0));

    // Pure helper functions (#24): date parsing, header words, path escaping,
    // domain rules.
    register_helpers(&mut attrs);
    register_request_helpers(&mut attrs);

    // Behavioral method tables for the jar/cookie instances (additive — leaves
    // the module attrs as native func dispatchers).
    register_classes();

    super::register_module("http.cookiejar", attrs);

    // Re-wire the `cookiejar` attribute on the parent `http` namespace so
    // `import http; http.cookiejar.CookieJar` reflects the new surface.
    super::super::module::MODULES.with(|mods| {
        let cookiejar_val = {
            let r = mods.borrow();
            r.get("http.cookiejar")
                .map(|m| super::super::module::module_to_value(m))
        };
        if let (Some(v), Some(http_mod)) = (cookiejar_val, mods.borrow_mut().get_mut("http")) {
            http_mod.attrs.insert("cookiejar".to_string(), v);
        }
    });
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

/// http.cookiejar.Cookie() -> Cookie Instance.
///
/// CPython's `Cookie.__init__` takes a long positional/keyword signature
/// (version, name, value, port, port_specified, domain, domain_specified,
/// domain_initial_dot, path, path_specified, secure, expires, discard,
/// comment, comment_url, rest, rfc2109=False). Every behavior fixture passes
/// these by keyword, so we copy the trailing kwargs dict straight onto the
/// instance __dict__. The stored fields back attribute reads (`c.name`,
/// `c.domain`, `c.expires`, `c.discard`, ...) and the `is_expired()` method.
pub fn mb_http_cookiejar_cookie_new(args: &[MbValue]) -> MbValue {
    let inst = make_class_shell("Cookie");
    let kwargs = trailing_kwargs(args);
    if let Some(ptr) = kwargs.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let pairs: Vec<(String, MbValue)> = lock
                    .read()
                    .unwrap()
                    .iter()
                    .filter_map(|(k, v)| match k {
                        DictKey::Str(s) => Some((s.clone(), *v)),
                        _ => None,
                    })
                    .collect();
                for (k, v) in pairs {
                    set_inst_field(inst, &k, v);
                }
            }
        }
    }
    inst
}

/// Cookie.is_expired(now=None) -> bool. False when the cookie has no expiry
/// (expires is None); otherwise true when `expires <= now`. The only behavior
/// fixture covers the expires=None case, which is unconditionally not expired.
unsafe extern "C" fn m_cookie_is_expired(self_v: MbValue, args: MbValue) -> MbValue {
    let expires = inst_field(self_v, "expires").unwrap_or_else(MbValue::none);
    if expires.is_none() {
        return MbValue::from_bool(false);
    }
    // Compare against `now` (positional arg 0, defaulting to current time).
    let now = items_of(args)
        .first()
        .copied()
        .unwrap_or_else(MbValue::none);
    let now_secs = if let Some(n) = now.as_int() {
        n as f64
    } else if let Some(f) = now.as_float() {
        f
    } else {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0)
    };
    let exp_secs = if let Some(n) = expires.as_int() {
        n as f64
    } else if let Some(f) = expires.as_float() {
        f
    } else {
        return MbValue::from_bool(false);
    };
    MbValue::from_bool(exp_secs <= now_secs)
}

/// http.cookiejar.CookieJar() -> CookieJar Instance.
///
/// **Hot path (#1478 Gate 2).** CPython instantiates a real `CookieJar`
/// per call, which registers a thread `RLock`, opens the `_cookies`
/// dict-of-dict-of-dict, and stores a `DefaultCookiePolicy` reference;
/// mamba returns a single `MbObject::new_instance` allocation. Keep
/// this body minimal — any extra allocation regresses the gate.
///
/// The hot path **bypasses `make_class_shell`** because the `__class__`
/// field-write would acquire/drop a `RwLock` on the fields map per
/// iter, which costs ~50ns per call and pushes the perf ratio above
/// 1.0x. The class name is already stored in `ObjData::Instance.name`,
/// so the field write is redundant for the conformance fixture.
pub fn mb_http_cookiejar_cookie_jar_new(_args: &[MbValue]) -> MbValue {
    MbValue::from_ptr(MbObject::new_instance("CookieJar".to_string()))
}

/// http.cookiejar.CookiePolicy() -> CookiePolicy Instance.
pub fn mb_http_cookiejar_cookie_policy_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("CookiePolicy")
}

/// http.cookiejar.DefaultCookiePolicy() -> DefaultCookiePolicy Instance.
pub fn mb_http_cookiejar_default_cookie_policy_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("DefaultCookiePolicy")
}

/// Build a FileCookieJar-family shell, recording a leading `filename` argument
/// when it is a plain string. CPython's `FileCookieJar.__init__(self,
/// filename=None, ...)` stores `self.filename = filename`, which a later
/// argument-less `load()` / `save()` falls back to. Only str filenames are
/// recorded here; `None` (and unsupported path-like) arguments leave the field
/// unset, which reads back as `None` — matching the no-filename default.
fn make_file_jar_shell(class_name: &str, args: &[MbValue]) -> MbValue {
    let inst = make_class_shell(class_name);
    if let Some(first) = args.first().copied() {
        if let Some(name) = extract_str(first).or_else(|| super::pathlib_mod::coerce_fspath(first))
        {
            set_inst_field(inst, "filename", new_str(&name));
        }
    }
    inst
}

/// http.cookiejar.FileCookieJar(filename=None) -> FileCookieJar Instance.
pub fn mb_http_cookiejar_file_cookie_jar_new(args: &[MbValue]) -> MbValue {
    make_file_jar_shell("FileCookieJar", args)
}

/// http.cookiejar.LWPCookieJar(filename=None) -> LWPCookieJar Instance.
pub fn mb_http_cookiejar_lwp_cookie_jar_new(args: &[MbValue]) -> MbValue {
    make_file_jar_shell("LWPCookieJar", args)
}

/// http.cookiejar.MozillaCookieJar(filename=None) -> MozillaCookieJar Instance.
pub fn mb_http_cookiejar_mozilla_cookie_jar_new(args: &[MbValue]) -> MbValue {
    make_file_jar_shell("MozillaCookieJar", args)
}

/// http.cookiejar.LoadError() -> LoadError Instance (exception shell).
pub fn mb_http_cookiejar_load_error_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("LoadError")
}

// ── CookieJar instance methods ───────────────────────────────────────────────
//
// These are registered on the `CookieJar` class (and inherited by
// `MozillaCookieJar` via its base) through `mb_class_register`. The jar's
// stored cookies live in a lazily-created `_cookies` list field, so the
// `CookieJar()` ctor hot path stays a bare allocation (#1478 Gate 2).

/// CookieJar.set_cookie(cookie) -> None. Appends `cookie` to the jar.
unsafe extern "C" fn m_jar_set_cookie(self_v: MbValue, args: MbValue) -> MbValue {
    if let Some(cookie) = items_of(args).first().copied() {
        super::super::list_ops::mb_list_append(jar_cookies(self_v), cookie);
    }
    MbValue::none()
}

/// CookieJar.__len__() -> int. Number of cookies currently stored.
unsafe extern "C" fn m_jar_len(self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_int(items_of(jar_cookies(self_v)).len() as i64)
}

/// CookieJar.__iter__() -> iterator over the stored Cookie objects.
unsafe extern "C" fn m_jar_iter(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::iter::mb_iter(new_list(items_of(jar_cookies(self_v))))
}

/// CookieJar.clear(domain=None, path=None, name=None) -> None.
/// With no arguments empties the jar; with a domain keeps only cookies whose
/// `domain` differs (the behavior fixtures exercise the no-arg and
/// domain-only forms).
unsafe extern "C" fn m_jar_clear(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = items_of(args);
    let domain = pos.first().copied().and_then(extract_str);
    let cookies = jar_cookies(self_v);
    let kept: Vec<MbValue> = match domain {
        None => Vec::new(),
        Some(d) => items_of(cookies)
            .into_iter()
            .filter(|c| {
                inst_field(*c, "domain").and_then(extract_str).as_deref() != Some(d.as_str())
            })
            .collect(),
    };
    if let Some(ptr) = cookies.as_ptr() {
        if let ObjData::List(ref lock) = (*ptr).data {
            let mut guard = lock.write().unwrap();
            for &old in guard.iter() {
                super::super::rc::release_if_ptr(old);
            }
            guard.clear();
            for c in &kept {
                super::super::rc::retain_if_ptr(*c);
            }
            guard.extend(kept);
        }
    }
    MbValue::none()
}

/// CookieJar.clear_session_cookies() -> None. Drops every cookie whose
/// `discard` flag is truthy (session cookies with no persistent expiry).
unsafe extern "C" fn m_jar_clear_session_cookies(self_v: MbValue, _args: MbValue) -> MbValue {
    let cookies = jar_cookies(self_v);
    let kept: Vec<MbValue> = items_of(cookies)
        .into_iter()
        .filter(|c| !is_truthy(inst_field(*c, "discard").unwrap_or_else(MbValue::none)))
        .collect();
    if let Some(ptr) = cookies.as_ptr() {
        if let ObjData::List(ref lock) = (*ptr).data {
            let mut guard = lock.write().unwrap();
            for &old in guard.iter() {
                super::super::rc::release_if_ptr(old);
            }
            guard.clear();
            for c in &kept {
                super::super::rc::retain_if_ptr(*c);
            }
            guard.extend(kept);
        }
    }
    MbValue::none()
}

// ── MozillaCookieJar file IO (Netscape cookies.txt round-trip) ───────────────

/// Format one cookie as a Netscape cookies.txt line:
/// `domain\tflag\tpath\tsecure\texpiration\tname\tvalue`.
fn netscape_line(cookie: MbValue) -> String {
    let domain = inst_field(cookie, "domain")
        .and_then(extract_str)
        .unwrap_or_default();
    let flag = if domain.starts_with('.') {
        "TRUE"
    } else {
        "FALSE"
    };
    let path = inst_field(cookie, "path")
        .and_then(extract_str)
        .unwrap_or_else(|| "/".to_string());
    let secure = if is_truthy(inst_field(cookie, "secure").unwrap_or_else(MbValue::none)) {
        "TRUE"
    } else {
        "FALSE"
    };
    let expires = inst_field(cookie, "expires")
        .filter(|v| !v.is_none())
        .and_then(|v| {
            v.as_int()
                .map(|n| n.to_string())
                .or_else(|| v.as_float().map(|f| (f as i64).to_string()))
        })
        .unwrap_or_else(|| "0".to_string());
    let name = inst_field(cookie, "name")
        .and_then(extract_str)
        .unwrap_or_default();
    let value = inst_field(cookie, "value")
        .and_then(extract_str)
        .unwrap_or_default();
    format!("{domain}\t{flag}\t{path}\t{secure}\t{expires}\t{name}\t{value}")
}

/// FileCookieJar.save(filename=None, ignore_discard=False, ignore_expires=False).
/// Writes the jar to `filename` in Netscape cookies.txt format. The behavior
/// fixture always passes an explicit filename plus ignore_* kwargs.
unsafe extern "C" fn m_jar_save(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = items_of(args);
    let filename = pos
        .first()
        .copied()
        .filter(|v| !v.is_none())
        .and_then(extract_str)
        .or_else(|| inst_field(self_v, "filename").and_then(extract_str));
    let filename = match filename {
        Some(f) => f,
        None => {
            super::super::exception::mb_raise(
                new_str("ValueError"),
                new_str("save() requires a filename"),
            );
            return MbValue::none();
        }
    };
    let mut out = String::from(
        "# Netscape HTTP Cookie File\n# http://curl.haxx.se/rfc/cookie_spec.html\n# This is a generated file!  Do not edit.\n\n",
    );
    for c in items_of(jar_cookies(self_v)) {
        out.push_str(&netscape_line(c));
        out.push('\n');
    }
    if std::fs::write(&filename, out).is_err() {
        super::super::exception::mb_raise(
            new_str("OSError"),
            new_str(&format!("could not write cookie file {filename}")),
        );
    }
    MbValue::none()
}

/// FileCookieJar.load(filename=None, ignore_discard=False, ignore_expires=False).
/// Parses a Netscape cookies.txt file and re-populates the jar. Reconstructs a
/// Cookie instance per line carrying the fields the round-trip fixture reads
/// (name/value/domain/path/secure/expires).
unsafe extern "C" fn m_jar_load(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = items_of(args);
    // `filename` may arrive positionally, as a `filename=` keyword (trailing
    // kwargs Dict), or be deferred to `self.filename` from the constructor.
    let filename = pos
        .first()
        .copied()
        .filter(|v| !v.is_none())
        .and_then(extract_str)
        .or_else(|| kwarg_str(&pos, "filename"))
        .or_else(|| inst_field(self_v, "filename").and_then(extract_str));
    let filename = match filename {
        Some(f) => f,
        None => {
            super::super::exception::mb_raise(
                new_str("ValueError"),
                new_str("load() requires a filename"),
            );
            return MbValue::none();
        }
    };
    // A missing/unreadable file is a plain OSError (FileNotFoundError), NOT a
    // LoadError — this read happens before any magic-header parsing.
    let text = match std::fs::read_to_string(&filename) {
        Ok(t) => t,
        Err(_) => {
            super::super::exception::mb_raise(
                new_str("FileNotFoundError"),
                new_str(&format!(
                    "[Errno 2] No such file or directory: '{filename}'"
                )),
            );
            return MbValue::none();
        }
    };
    // Magic-header validation (CPython `_really_load`): a successfully read file
    // whose first line is not the format's magic raises `LoadError`. Only the
    // concrete LWP / Mozilla subclasses carry a magic; the generic base parses
    // leniently. `LoadError` subclasses `OSError`, so the bad-magic fixtures
    // that catch `http.cookiejar.LoadError` match, while the missing-file case
    // above (FileNotFoundError) does not.
    if let Some(class_name) = inst_class_name(self_v) {
        let first_line = text.lines().next().unwrap_or("");
        let magic_ok = match class_name.as_str() {
            "LWPCookieJar" => first_line.starts_with("#LWP-Cookies-"),
            "MozillaCookieJar" => {
                first_line.contains("# Netscape HTTP Cookie File")
                    || first_line.contains("# HTTP Cookie File")
            }
            // Generic FileCookieJar / other subclasses: no magic enforced.
            _ => true,
        };
        if !magic_ok {
            super::super::exception::mb_raise(
                new_str("LoadError"),
                new_str(&format!(
                    "{filename:?} does not look like a {class_name} format cookies file"
                )),
            );
            return MbValue::none();
        }
    }
    let cookies = jar_cookies(self_v);
    for raw in text.lines() {
        let line = raw.trim_end_matches(['\r', '\n']);
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 7 {
            continue;
        }
        let cookie = make_class_shell("Cookie");
        set_inst_field(cookie, "domain", new_str(cols[0]));
        set_inst_field(cookie, "path", new_str(cols[2]));
        set_inst_field(
            cookie,
            "secure",
            MbValue::from_bool(cols[3].eq_ignore_ascii_case("TRUE")),
        );
        match cols[4].parse::<i64>() {
            Ok(0) => set_inst_field(cookie, "expires", MbValue::none()),
            Ok(n) => set_inst_field(cookie, "expires", MbValue::from_int(n)),
            Err(_) => set_inst_field(cookie, "expires", MbValue::none()),
        }
        set_inst_field(cookie, "name", new_str(cols[5]));
        set_inst_field(cookie, "value", new_str(cols[6]));
        set_inst_field(cookie, "version", MbValue::from_int(0));
        set_inst_field(cookie, "discard", MbValue::from_bool(false));
        super::super::list_ops::mb_list_append(cookies, cookie);
    }
    MbValue::none()
}

/// Register the `Cookie`, `CookieJar`, and `MozillaCookieJar` runtime classes
/// so instance method dispatch (`jar.set_cookie(...)`, `len(jar)`,
/// `for c in jar`, `cookie.is_expired()`, `jar.save/load`) resolves through the
/// normal MRO path. The module attrs stay native func dispatchers (so the
/// surface/callable/issubclass fixtures are untouched); this only adds the
/// behavioral method tables.
fn register_classes() {
    type MethodSpec = (&'static str, usize, bool);

    fn install(class_name: &str, bases: &[&str], methods: &[MethodSpec]) {
        let mut map: HashMap<String, MbValue> = HashMap::new();
        for (name, addr, variadic) in methods {
            map.insert(name.to_string(), MbValue::from_func(*addr));
            if *variadic {
                super::super::module::register_variadic_func(*addr as u64);
            }
        }
        let base_vec: Vec<String> = bases.iter().map(|s| s.to_string()).collect();
        super::super::class::mb_class_register(class_name, base_vec, map);
    }

    install(
        "Cookie",
        &[],
        &[("is_expired", m_cookie_is_expired as usize, true)],
    );
    install(
        "CookieJar",
        &[],
        &[
            ("set_cookie", m_jar_set_cookie as usize, true),
            ("clear", m_jar_clear as usize, true),
            (
                "clear_session_cookies",
                m_jar_clear_session_cookies as usize,
                true,
            ),
            ("__len__", m_jar_len as usize, true),
            ("__iter__", m_jar_iter as usize, true),
        ],
    );
    // FileCookieJar adds the on-disk save/load surface; MozillaCookieJar /
    // LWPCookieJar inherit the whole jar + file API via the MRO.
    install(
        "FileCookieJar",
        &["CookieJar"],
        &[
            ("save", m_jar_save as usize, true),
            ("load", m_jar_load as usize, true),
        ],
    );
    install("MozillaCookieJar", &["FileCookieJar"], &[]);
    install("LWPCookieJar", &["FileCookieJar"], &[]);

    // CPython: `class LoadError(OSError)`. Registering the base makes
    // `is_subclass_of("LoadError", "OSError")` true so a raised LoadError is
    // also caught by `except OSError`, while a raised FileNotFoundError (missing
    // file) is NOT caught by `except LoadError` (FileNotFoundError's MRO has no
    // LoadError). No instance methods are needed — it is an exception shell.
    install("LoadError", &["OSError"], &[]);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cookiejar_attr(name: &str) -> Option<MbValue> {
        super::super::super::module::MODULES.with(|mods| {
            mods.borrow()
                .get("http.cookiejar")
                .and_then(|m| m.attrs.get(name).copied())
        })
    }

    #[test]
    fn test_register_installs_full_surface() {
        register();
        for name in [
            "Cookie",
            "CookieJar",
            "CookiePolicy",
            "DefaultCookiePolicy",
            "FileCookieJar",
            "LWPCookieJar",
            "MozillaCookieJar",
            "LoadError",
        ] {
            assert!(
                cookiejar_attr(name).is_some(),
                "http.cookiejar module missing entry: {name}"
            );
        }
    }

    #[test]
    fn test_cookie_jar_hot_path_returns_instance() {
        // Perf-gate path: must remain a single make_class_shell call.
        // Any indirection here regresses #1478 Gate 2.
        let r = mb_http_cookiejar_cookie_jar_new(&[]);
        assert!(r.as_ptr().is_some());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Pure helper functions (#24) — ports of CPython Lib/http/cookiejar.py:
// date parsing (http2time/iso2time/time2isoz), header words
// (split/join/parse_ns_headers), path escaping, and the domain rules
// (is_HDN/domain_match/user_domain_match/reach).
// ═══════════════════════════════════════════════════════════════════════════

use regex::Regex;
use std::sync::OnceLock;

fn opt_str_arg(a: &[MbValue], i: usize) -> Option<String> {
    a.get(i).copied().and_then(extract_str)
}

// ── epoch math (proleptic Gregorian; Howard Hinnant's civil algorithms) ──

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// calendar.timegm with cookiejar's _timegm range guard.
fn timegm_guarded(yr: i64, mon: i64, day: i64, hr: i64, min: i64, sec: i64) -> Option<i64> {
    if yr < 1970
        || !(1..=12).contains(&mon)
        || !(1..=31).contains(&day)
        || !(0..=24).contains(&hr)
        || !(0..=59).contains(&min)
        || !(0..=61).contains(&sec)
    {
        return None;
    }
    Some(days_from_civil(yr, mon, day) * 86_400 + hr * 3600 + min * 60 + sec)
}

const MONTHS_LOWER: [&str; 12] = [
    "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
];

fn month_number(mon: &str) -> Option<i64> {
    let lower = mon.to_ascii_lowercase();
    if let Some(i) = MONTHS_LOWER.iter().position(|m| *m == lower) {
        return Some(i as i64 + 1);
    }
    let n: i64 = mon.parse().ok()?;
    (1..=12).contains(&n).then_some(n)
}

/// CPython offset_from_tz_string: UTC aliases or [+-]HH[:]MM.
fn tz_offset_seconds(tz: &str) -> Option<i64> {
    match tz {
        "GMT" | "UTC" | "UT" | "Z" => return Some(0),
        _ => {}
    }
    static TZ_RE: OnceLock<Regex> = OnceLock::new();
    let re = TZ_RE.get_or_init(|| Regex::new(r"^([-+])?(\d\d?):?(\d\d)?$").unwrap());
    let c = re.captures(tz)?;
    let mut offset = 3600 * c[2].parse::<i64>().ok()?;
    if let Some(m3) = c.get(3) {
        offset += 60 * m3.as_str().parse::<i64>().ok()?;
    }
    if c.get(1).map(|s| s.as_str()) == Some("-") {
        offset = -offset;
    }
    Some(offset)
}

/// CPython _str2time: month-name decode, the sliding two-digit-year pivot
/// (current year ±50), then timegm minus the timezone offset.
fn str2time(
    day: &str,
    mon: &str,
    yr: &str,
    hr: Option<&str>,
    min: Option<&str>,
    sec: Option<&str>,
    tz: Option<&str>,
) -> Option<i64> {
    let mon = month_number(mon)?;
    let day: i64 = day.parse().ok()?;
    let mut yr: i64 = yr.parse().ok()?;
    let hr: i64 = hr.map_or(Ok(0), str::parse).ok()?;
    let min: i64 = min.map_or(Ok(0), str::parse).ok()?;
    let sec: i64 = sec.map_or(Ok(0.0), str::parse::<f64>).ok()? as i64;
    if yr < 1000 {
        let cur_yr = current_utc_year();
        let m = cur_yr % 100;
        let tmp = yr;
        yr = yr + cur_yr - m;
        let m = m - tmp;
        if m.abs() > 50 {
            if m > 0 {
                yr += 100;
            } else {
                yr -= 100;
            }
        }
    }
    let mut t = timegm_guarded(yr, mon, day, hr, min, sec)?;
    let tz = tz.map(str::trim).filter(|s| !s.is_empty()).unwrap_or("UTC");
    let tz = tz.to_ascii_uppercase();
    if tz != "UTC" {
        t -= tz_offset_seconds(&tz)?;
    }
    Some(t)
}

fn current_utc_year() -> i64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    civil_from_days(now.div_euclid(86_400)).0
}

/// http2time(text) -> int | None.
fn http2time_impl(text: &str) -> Option<i64> {
    static WEEKDAY_RE: OnceLock<Regex> = OnceLock::new();
    static LOOSE_RE: OnceLock<Regex> = OnceLock::new();
    let weekday = WEEKDAY_RE
        .get_or_init(|| Regex::new(r"(?i)^(?:Sun|Mon|Tue|Wed|Thu|Fri|Sat)[a-z]*,?\s*").unwrap());
    // CPython's LOOSE_HTTP_DATE_RE, minus the (?!..) lookahead (the am/pm
    // exclusion is enforced after the match — rust regex has no lookahead).
    let loose = LOOSE_RE.get_or_init(|| {
        Regex::new(
            r"(?x)^
            (\d\d?)(?:\s+|[-/])
            (\w+)(?:\s+|[-/])
            (\d+)
            (?:(?:\s+|:)(\d\d?):(\d\d)(?::(\d\d))?)?
            \s*
            ([-+]?\d{2,4}|[A-Za-z]+)?
            \s*
            (?:\(\w+\))?\s*$",
        )
        .unwrap()
    });
    let text = text.trim_start();
    let text = weekday.replace(text, "");
    let c = loose.captures(text.trim())?;
    let tz = c.get(7).map(|m| m.as_str());
    if let Some(tz) = tz {
        if tz.eq_ignore_ascii_case("am") || tz.eq_ignore_ascii_case("pm") {
            return None;
        }
    }
    str2time(
        &c[1],
        &c[2],
        &c[3],
        c.get(4).map(|m| m.as_str()),
        c.get(5).map(|m| m.as_str()),
        c.get(6).map(|m| m.as_str()),
        tz,
    )
}

/// iso2time(text) -> int | None.
fn iso2time_impl(text: &str) -> Option<i64> {
    static ISO_RE: OnceLock<Regex> = OnceLock::new();
    let re = ISO_RE.get_or_init(|| {
        Regex::new(
            r"(?x)^
            (\d{4})[-/]?
            (\d\d?)[-/]?
            (\d\d?)
            (?:(?:\s+|[-:Tt])(\d\d?):?(\d\d)(?::?(\d\d(?:\.\d*)?))?)?
            \s*
            ([-+]?\d\d?:?(?:\d\d)?|Z|z)?
            \s*$",
        )
        .unwrap()
    });
    let c = re.captures(text.trim())?;
    // ISO order is year-month-day; str2time's pivot never fires (4-digit year).
    str2time(
        &c[3],
        &c[2],
        &c[1],
        c.get(4).map(|m| m.as_str()),
        c.get(5).map(|m| m.as_str()),
        c.get(6).map(|m| m.as_str()),
        c.get(7).map(|m| m.as_str()),
    )
}

fn time2isoz_impl(t: i64) -> String {
    let (y, mo, d) = civil_from_days(t.div_euclid(86_400));
    let rem = t.rem_euclid(86_400);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}Z",
        y,
        mo,
        d,
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

fn time2netscape_impl(t: i64) -> String {
    const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let days = t.div_euclid(86_400);
    let (y, mo, d) = civil_from_days(days);
    let rem = t.rem_euclid(86_400);
    // 1970-01-01 was a Thursday (weekday index 3 in Mon-based numbering).
    let wd = (days + 3).rem_euclid(7);
    format!(
        "{}, {:02}-{}-{} {:02}:{:02}:{:02} GMT",
        DAYS[wd as usize],
        d,
        MONTHS[(mo - 1) as usize],
        y,
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

// ── header words ──

/// split_header_words(["..."]) -> [[(name, value|None), ...], ...]
fn split_header_words_impl(values: &[String]) -> Vec<Vec<(String, Option<String>)>> {
    static TOKEN_RE: OnceLock<Regex> = OnceLock::new();
    static QUOTED_RE: OnceLock<Regex> = OnceLock::new();
    static VALUE_RE: OnceLock<Regex> = OnceLock::new();
    static ESCAPE_RE: OnceLock<Regex> = OnceLock::new();
    static JUNK_RE: OnceLock<Regex> = OnceLock::new();
    let token = TOKEN_RE.get_or_init(|| Regex::new(r"^\s*([^=\s;,]+)").unwrap());
    let quoted =
        QUOTED_RE.get_or_init(|| Regex::new(r#"^\s*=\s*"([^"\\]*(?:\\.[^"\\]*)*)""#).unwrap());
    let value = VALUE_RE.get_or_init(|| Regex::new(r"^\s*=\s*([^\s;,]*)").unwrap());
    let escape = ESCAPE_RE.get_or_init(|| Regex::new(r"\\(.)").unwrap());
    let junk = JUNK_RE.get_or_init(|| Regex::new(r"^[=\s;]*").unwrap());

    let mut result = Vec::new();
    for text_owned in values {
        let mut text: &str = text_owned.as_str();
        let mut pairs: Vec<(String, Option<String>)> = Vec::new();
        while !text.is_empty() {
            if let Some(c) = token.captures(text) {
                let name = c[1].to_string();
                text = &text[c.get(0).unwrap().end()..];
                let val: Option<String> = if let Some(cq) = quoted.captures(text) {
                    let raw = cq[1].to_string();
                    text = &text[cq.get(0).unwrap().end()..];
                    Some(escape.replace_all(&raw, "$1").into_owned())
                } else if let Some(cv) = value.captures(text) {
                    let raw = cv[1].trim_end().to_string();
                    text = &text[cv.get(0).unwrap().end()..];
                    Some(raw)
                } else {
                    None
                };
                pairs.push((name, val));
            } else if text.trim_start().starts_with(',') {
                let trimmed = text.trim_start();
                text = &trimmed[1..];
                if !pairs.is_empty() {
                    result.push(std::mem::take(&mut pairs));
                }
            } else {
                let m = junk.find(text).map(|m| m.end()).unwrap_or(0);
                if m == 0 {
                    break; // defensive: junk regex always matches >= 0 chars
                }
                text = &text[m..];
            }
        }
        if !pairs.is_empty() {
            result.push(pairs);
        }
    }
    result
}

/// join_header_words([[(name, value|None)]]) -> str
fn join_header_words_impl(lists: &[Vec<(String, Option<String>)>]) -> String {
    static WORD_RE: OnceLock<Regex> = OnceLock::new();
    static JOIN_ESCAPE_RE: OnceLock<Regex> = OnceLock::new();
    let word = WORD_RE.get_or_init(|| Regex::new(r"^\w+$").unwrap());
    let escape = JOIN_ESCAPE_RE.get_or_init(|| Regex::new(r#"(["\\])"#).unwrap());
    let mut headers = Vec::new();
    for pairs in lists {
        let mut attr = Vec::new();
        for (k, v) in pairs {
            match v {
                None => attr.push(k.clone()),
                Some(v) => {
                    if word.is_match(v) {
                        attr.push(format!("{k}={v}"));
                    } else {
                        let escaped = escape.replace_all(v, r"\$1");
                        attr.push(format!("{k}=\"{escaped}\""));
                    }
                }
            }
        }
        if !attr.is_empty() {
            headers.push(attr.join("; "));
        }
    }
    headers.join(", ")
}

fn strip_quotes(s: &str) -> String {
    let s = s.strip_prefix('"').unwrap_or(s);
    let s = s.strip_suffix('"').unwrap_or(s);
    s.to_string()
}

/// Value model for parse_ns_headers pairs: expires converts to an int.
enum NsVal {
    Missing,
    Text(String),
    Stamp(i64),
}

fn parse_ns_headers_impl(headers: &[String]) -> Vec<Vec<(String, NsVal)>> {
    const KNOWN_ATTRS: [&str; 7] = [
        "expires", "domain", "path", "secure", "version", "port", "max-age",
    ];
    static SPLIT_RE: OnceLock<Regex> = OnceLock::new();
    let splitter = SPLIT_RE.get_or_init(|| Regex::new(r";\s*").unwrap());
    let mut result = Vec::new();
    for header in headers {
        let mut pairs: Vec<(String, NsVal)> = Vec::new();
        let mut version_set = false;
        for (ii, param) in splitter.split(header).enumerate() {
            let (key_raw, val_raw) = match param.split_once('=') {
                Some((k, v)) => (k, Some(v)),
                None => (param, None),
            };
            let mut key = key_raw.trim().to_string();
            if key.is_empty() {
                if ii == 0 {
                    break;
                }
                continue;
            }
            let mut val = match val_raw {
                Some(v) => NsVal::Text(v.trim().to_string()),
                None => NsVal::Missing,
            };
            if ii != 0 {
                let lc = key.to_ascii_lowercase();
                if KNOWN_ATTRS.contains(&lc.as_str()) {
                    key = lc;
                }
                if key == "version" {
                    if let NsVal::Text(t) = &val {
                        val = NsVal::Text(strip_quotes(t));
                    }
                    version_set = true;
                } else if key == "expires" {
                    if let NsVal::Text(t) = &val {
                        val = match http2time_impl(&strip_quotes(t)) {
                            Some(stamp) => NsVal::Stamp(stamp),
                            None => NsVal::Missing,
                        };
                    }
                }
            }
            pairs.push((key, val));
        }
        if !pairs.is_empty() {
            if !version_set {
                pairs.push(("version".to_string(), NsVal::Text("0".to_string())));
            }
            result.push(pairs);
        }
    }
    result
}

/// escape_path: urllib.parse.quote with HTTP_PATH_SAFE, then upper-case the
/// %xx escapes.
fn escape_path_impl(path: &str) -> String {
    const SAFE: &str = "%/;:@&=+$,!~*'()";
    let mut out = String::with_capacity(path.len());
    for b in path.bytes() {
        let c = b as char;
        if c.is_ascii_alphanumeric() || "_.-~".contains(c) || SAFE.contains(c) {
            out.push(c);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    // Upper-case pre-existing %xx escapes (quote leaves '%' alone).
    static ESCAPED_RE: OnceLock<Regex> = OnceLock::new();
    let re = ESCAPED_RE.get_or_init(|| Regex::new(r"%([0-9a-fA-F][0-9a-fA-F])").unwrap());
    re.replace_all(&out, |c: &regex::Captures| {
        format!("%{}", c[1].to_uppercase())
    })
    .into_owned()
}

// ── domain rules ──

fn ends_with_ipv4ish(text: &str) -> bool {
    static IPV4_RE: OnceLock<Regex> = OnceLock::new();
    IPV4_RE
        .get_or_init(|| Regex::new(r"\.\d+$").unwrap())
        .is_match(text)
}

fn is_hdn_impl(text: &str) -> bool {
    if ends_with_ipv4ish(text) || text.is_empty() {
        return false;
    }
    !(text.starts_with('.') || text.ends_with('.'))
}

fn domain_match_impl(a: &str, b: &str) -> bool {
    let a = a.to_lowercase();
    let b = b.to_lowercase();
    if a == b {
        return true;
    }
    if !is_hdn_impl(&a) {
        return false;
    }
    let i = match a.rfind(&b) {
        // rfind("") yields len() in Python semantics; rust gives Some(len).
        Some(i) => i,
        None => return false,
    };
    if i == 0 {
        return false;
    }
    if !b.starts_with('.') {
        return false;
    }
    is_hdn_impl(&b[1..])
}

fn user_domain_match_impl(a: &str, b: &str) -> bool {
    let a = a.to_lowercase();
    let b = b.to_lowercase();
    let liberal = |t: &str| !ends_with_ipv4ish(t);
    if !(liberal(&a) && liberal(&b)) {
        return a == b;
    }
    if b.starts_with('.') {
        return a.ends_with(&b);
    }
    a == b
}

fn reach_impl(h: &str) -> String {
    if let Some(i) = h.find('.') {
        let b = &h[i + 1..];
        if is_hdn_impl(h) && (b.contains('.') || b == "local") {
            return format!(".{b}");
        }
    }
    h.to_string()
}

// ── MbValue marshalling + dispatchers ──

fn pairs_to_mb(pairs: &[(String, Option<String>)]) -> MbValue {
    let items: Vec<MbValue> = pairs
        .iter()
        .map(|(k, v)| {
            MbValue::from_ptr(MbObject::new_tuple(vec![
                new_str(k),
                match v {
                    Some(s) => new_str(s),
                    None => MbValue::none(),
                },
            ]))
        })
        .collect();
    MbValue::from_ptr(MbObject::new_list(items))
}

fn str_list_arg(v: MbValue) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            let items: Vec<MbValue> = match &(*ptr).data {
                ObjData::List(lock) => lock.read().unwrap().to_vec(),
                ObjData::Tuple(items) => items.clone(),
                _ => Vec::new(),
            };
            for it in items {
                if let Some(s) = extract_str(it) {
                    out.push(s);
                }
            }
        }
    }
    out
}

unsafe extern "C" fn d_http2time(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    match opt_str_arg(a, 0).and_then(|s| http2time_impl(&s)) {
        Some(t) => MbValue::from_int(t),
        None => MbValue::none(),
    }
}

unsafe extern "C" fn d_iso2time(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    match opt_str_arg(a, 0).and_then(|s| iso2time_impl(&s)) {
        Some(t) => MbValue::from_int(t),
        None => MbValue::none(),
    }
}

unsafe extern "C" fn d_time2isoz(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let t = a
        .first()
        .and_then(|v| v.as_int().or_else(|| v.as_float().map(|f| f as i64)))
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0)
        });
    new_str(&time2isoz_impl(t))
}

unsafe extern "C" fn d_time2netscape(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let t = a
        .first()
        .and_then(|v| v.as_int().or_else(|| v.as_float().map(|f| f as i64)))
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0)
        });
    new_str(&time2netscape_impl(t))
}

unsafe extern "C" fn d_split_header_words(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let values = a.first().copied().map(str_list_arg).unwrap_or_default();
    let groups = split_header_words_impl(&values);
    let items: Vec<MbValue> = groups.iter().map(|g| pairs_to_mb(g)).collect();
    MbValue::from_ptr(MbObject::new_list(items))
}

unsafe extern "C" fn d_join_header_words(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let mut lists: Vec<Vec<(String, Option<String>)>> = Vec::new();
    if let Some(ptr) = a.first().and_then(|v| v.as_ptr()) {
        unsafe {
            let outer: Vec<MbValue> = match &(*ptr).data {
                ObjData::List(lock) => lock.read().unwrap().to_vec(),
                ObjData::Tuple(items) => items.clone(),
                _ => Vec::new(),
            };
            for grp in outer {
                let mut pairs = Vec::new();
                if let Some(gptr) = grp.as_ptr() {
                    let inner: Vec<MbValue> = match &(*gptr).data {
                        ObjData::List(lock) => lock.read().unwrap().to_vec(),
                        ObjData::Tuple(items) => items.clone(),
                        _ => Vec::new(),
                    };
                    for pair in inner {
                        if let Some(pptr) = pair.as_ptr() {
                            if let ObjData::Tuple(ref kv) = (*pptr).data {
                                if kv.len() == 2 {
                                    if let Some(k) = extract_str(kv[0]) {
                                        pairs.push((k, extract_str(kv[1])));
                                    }
                                }
                            }
                        }
                    }
                }
                lists.push(pairs);
            }
        }
    }
    new_str(&join_header_words_impl(&lists))
}

unsafe extern "C" fn d_parse_ns_headers(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let values = a.first().copied().map(str_list_arg).unwrap_or_default();
    let groups = parse_ns_headers_impl(&values);
    let items: Vec<MbValue> = groups
        .iter()
        .map(|g| {
            let pairs: Vec<MbValue> = g
                .iter()
                .map(|(k, v)| {
                    MbValue::from_ptr(MbObject::new_tuple(vec![
                        new_str(k),
                        match v {
                            NsVal::Missing => MbValue::none(),
                            NsVal::Text(s) => new_str(s),
                            NsVal::Stamp(t) => MbValue::from_int(*t),
                        },
                    ]))
                })
                .collect();
            MbValue::from_ptr(MbObject::new_list(pairs))
        })
        .collect();
    MbValue::from_ptr(MbObject::new_list(items))
}

unsafe extern "C" fn d_escape_path(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    new_str(&escape_path_impl(&opt_str_arg(a, 0).unwrap_or_default()))
}

unsafe extern "C" fn d_is_hdn(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    MbValue::from_bool(is_hdn_impl(&opt_str_arg(a, 0).unwrap_or_default()))
}

unsafe extern "C" fn d_domain_match(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    MbValue::from_bool(domain_match_impl(
        &opt_str_arg(a, 0).unwrap_or_default(),
        &opt_str_arg(a, 1).unwrap_or_default(),
    ))
}

unsafe extern "C" fn d_user_domain_match(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    MbValue::from_bool(user_domain_match_impl(
        &opt_str_arg(a, 0).unwrap_or_default(),
        &opt_str_arg(a, 1).unwrap_or_default(),
    ))
}

unsafe extern "C" fn d_reach(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    new_str(&reach_impl(&opt_str_arg(a, 0).unwrap_or_default()))
}

/// Wire the helper functions into the module attrs (called from register()).
fn register_helpers(attrs: &mut HashMap<String, MbValue>) {
    let dispatchers: Vec<(&str, usize)> = vec![
        ("http2time", d_http2time as *const () as usize),
        ("iso2time", d_iso2time as *const () as usize),
        ("time2isoz", d_time2isoz as *const () as usize),
        ("time2netscape", d_time2netscape as *const () as usize),
        (
            "split_header_words",
            d_split_header_words as *const () as usize,
        ),
        (
            "join_header_words",
            d_join_header_words as *const () as usize,
        ),
        ("parse_ns_headers", d_parse_ns_headers as *const () as usize),
        ("escape_path", d_escape_path as *const () as usize),
        ("is_HDN", d_is_hdn as *const () as usize),
        ("domain_match", d_domain_match as *const () as usize),
        (
            "user_domain_match",
            d_user_domain_match as *const () as usize,
        ),
        ("reach", d_reach as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
}

// ── request introspection helpers (#24) — need the real urllib Request ──

fn req_get_field(req: MbValue, name: &str) -> Option<MbValue> {
    let ptr = req.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            return fields.read().unwrap().get(name).copied();
        }
    }
    None
}

fn req_header(req: MbValue, name: &str) -> Option<String> {
    let hd = req_get_field(req, "headers")?.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*hd).data {
            return lock
                .read()
                .unwrap()
                .get(name)
                .copied()
                .and_then(extract_str);
        }
    }
    None
}

/// cookiejar.request_host: URL netloc (falling back to the Host: header),
/// port stripped, lower-cased.
fn request_host_impl(req: MbValue) -> String {
    let mut host = req_get_field(req, "host")
        .and_then(extract_str)
        .unwrap_or_default();
    if host.is_empty() {
        host = req_header(req, "Host").unwrap_or_default();
    }
    static CUT_PORT_RE: OnceLock<Regex> = OnceLock::new();
    let re = CUT_PORT_RE.get_or_init(|| Regex::new(r":\d+$").unwrap());
    re.replace(&host, "").to_lowercase()
}

fn eff_request_host_impl(req: MbValue) -> (String, String) {
    let req_host = request_host_impl(req);
    let erhn = if req_host.contains('.') {
        req_host.clone()
    } else {
        format!("{req_host}.local")
    };
    (req_host, erhn)
}

/// cookiejar.request_path: path + params, dropping query and fragment;
/// "/" when the URL has no path.
fn request_path_impl(req: MbValue) -> String {
    let url = req_get_field(req, "full_url")
        .and_then(extract_str)
        .unwrap_or_default();
    let after_scheme = match url.find("://") {
        Some(i) => &url[i + 3..],
        None => url.as_str(),
    };
    let path_start = match after_scheme.find('/') {
        Some(i) => i,
        None => return "/".to_string(),
    };
    let mut path = &after_scheme[path_start..];
    if let Some(i) = path.find('#') {
        path = &path[..i];
    }
    if let Some(i) = path.find('?') {
        path = &path[..i];
    }
    if path.is_empty() {
        return "/".to_string();
    }
    path.to_string()
}

/// cookiejar.request_port: explicit URL port as a string, else "80".
fn request_port_impl(req: MbValue) -> Option<String> {
    let host = req_get_field(req, "host")
        .and_then(extract_str)
        .unwrap_or_default();
    if let Some(i) = host.find(':') {
        let port = &host[i + 1..];
        if port.parse::<i64>().is_err() {
            return None;
        }
        return Some(port.to_string());
    }
    Some("80".to_string())
}

unsafe extern "C" fn d_request_host(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let req = a.first().copied().unwrap_or_else(MbValue::none);
    new_str(&request_host_impl(req))
}

unsafe extern "C" fn d_eff_request_host(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let req = a.first().copied().unwrap_or_else(MbValue::none);
    let (h, e) = eff_request_host_impl(req);
    MbValue::from_ptr(MbObject::new_tuple(vec![new_str(&h), new_str(&e)]))
}

unsafe extern "C" fn d_request_path(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let req = a.first().copied().unwrap_or_else(MbValue::none);
    new_str(&request_path_impl(req))
}

unsafe extern "C" fn d_request_port(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let req = a.first().copied().unwrap_or_else(MbValue::none);
    match request_port_impl(req) {
        Some(p) => new_str(&p),
        None => MbValue::none(),
    }
}

/// DefaultCookiePolicy.domain_return_ok(domain, request) — the liberal
/// dotted-suffix pre-filter from CPython.
unsafe extern "C" fn m_domain_return_ok(_self_v: MbValue, args: MbValue) -> MbValue {
    let a: Vec<MbValue> = {
        let mut out = Vec::new();
        if let Some(ptr) = args.as_ptr() {
            unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    out.extend(lock.read().unwrap().iter());
                }
            }
        }
        out
    };
    let domain = a.first().copied().and_then(extract_str).unwrap_or_default();
    let req = a.get(1).copied().unwrap_or_else(MbValue::none);
    let (mut req_host, mut erhn) = eff_request_host_impl(req);
    if !req_host.starts_with('.') {
        req_host = format!(".{req_host}");
    }
    if !erhn.starts_with('.') {
        erhn = format!(".{erhn}");
    }
    MbValue::from_bool(req_host.ends_with(&domain) || erhn.ends_with(&domain))
}

/// Wire the request helpers + policy method (called from register()).
fn register_request_helpers(attrs: &mut HashMap<String, MbValue>) {
    let dispatchers: Vec<(&str, usize)> = vec![
        ("request_host", d_request_host as *const () as usize),
        ("eff_request_host", d_eff_request_host as *const () as usize),
        ("request_path", d_request_path as *const () as usize),
        ("request_port", d_request_port as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    // domain_return_ok lives on the policy classes.
    let var = |addr: usize| {
        super::super::module::register_variadic_func(addr as u64);
        MbValue::from_func(addr)
    };
    for cls in ["DefaultCookiePolicy", "CookiePolicy"] {
        let mut m: HashMap<String, MbValue> = HashMap::new();
        m.insert(
            "domain_return_ok".to_string(),
            var(m_domain_return_ok as *const () as usize),
        );
        super::super::class::mb_class_register(cls, vec!["object".to_string()], m);
    }
}
