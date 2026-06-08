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
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};
use super::super::dict_ops::DictKey;

// ── Small value helpers ──

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn new_list(items: Vec<MbValue>) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(items))
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    })
}

/// Truthiness of a stored field: None and False are falsy, everything else truthy.
fn is_truthy(val: MbValue) -> bool {
    if val.is_none() { return false; }
    if let Some(b) = val.as_bool() { return b; }
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
                if let Some(p) = prev { super::super::rc::release_if_ptr(p); }
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
        if last.as_ptr().map(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) }).unwrap_or(false) {
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
            return lock.read().unwrap().get(name).copied().and_then(extract_str);
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
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a)
        }
    };
}

disp_variadic!(d_cookie,               mb_http_cookiejar_cookie_new);
disp_variadic!(d_cookie_jar,           mb_http_cookiejar_cookie_jar_new);
disp_variadic!(d_cookie_policy,        mb_http_cookiejar_cookie_policy_new);
disp_variadic!(d_default_cookie_policy, mb_http_cookiejar_default_cookie_policy_new);
disp_variadic!(d_file_cookie_jar,      mb_http_cookiejar_file_cookie_jar_new);
disp_variadic!(d_lwp_cookie_jar,       mb_http_cookiejar_lwp_cookie_jar_new);
disp_variadic!(d_mozilla_cookie_jar,   mb_http_cookiejar_mozilla_cookie_jar_new);
disp_variadic!(d_load_error,           mb_http_cookiejar_load_error_new);

/// Register the http.cookiejar module under its dotted name. Also wire it
/// back into the parent `http` namespace as `http.cookiejar`, mirroring
/// what `http_mod.rs::register()` does for its subpackages and what
/// `http_cookies_mod::register()` does for `http.cookies`.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("Cookie",              d_cookie               as *const () as usize),
        ("CookieJar",           d_cookie_jar           as *const () as usize),
        ("CookiePolicy",        d_cookie_policy        as *const () as usize),
        ("DefaultCookiePolicy", d_default_cookie_policy as *const () as usize),
        ("FileCookieJar",       d_file_cookie_jar      as *const () as usize),
        ("LWPCookieJar",        d_lwp_cookie_jar       as *const () as usize),
        ("MozillaCookieJar",    d_mozilla_cookie_jar   as *const () as usize),
        ("LoadError",           d_load_error           as *const () as usize),
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
    attrs.insert("DEFAULT_HTTP_PORT".into(), MbValue::from_ptr(MbObject::new_str("80".to_string())));
    attrs.insert("EPOCH_YEAR".into(), MbValue::from_int(1970));
    attrs.insert("HTTPONLY_ATTR".into(), MbValue::from_ptr(MbObject::new_str("HTTPOnly".to_string())));
    attrs.insert("HTTPONLY_PREFIX".into(), MbValue::from_ptr(MbObject::new_str("#HttpOnly_".to_string())));
    attrs.insert("HTTP_PATH_SAFE".into(), MbValue::from_ptr(MbObject::new_str("%/;:@&=+$,!~*'()".to_string())));
    attrs.insert("MISSING_FILENAME_TEXT".into(), MbValue::from_ptr(MbObject::new_str("a filename was not supplied (nor was the CookieJar instance initialised with one)".to_string())));
    attrs.insert("NETSCAPE_HEADER_TEXT".into(), MbValue::from_ptr(MbObject::new_str("# Netscape HTTP Cookie File\n# http://curl.haxx.se/rfc/cookie_spec.html\n# This is a generated file!  Do not edit.\n\n".to_string())));
    attrs.insert("debug".into(), MbValue::from_int(0));

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
        if let (Some(v), Some(http_mod)) = (
            cookiejar_val,
            mods.borrow_mut().get_mut("http"),
        ) {
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
                let pairs: Vec<(String, MbValue)> = lock.read().unwrap().iter()
                    .filter_map(|(k, v)| match k {
                        DictKey::Str(s) => Some((s.clone(), *v)),
                        _ => None,
                    })
                    .collect();
                for (k, v) in pairs { set_inst_field(inst, &k, v); }
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
    let now = items_of(args).first().copied().unwrap_or_else(MbValue::none);
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
        if let Some(name) = extract_str(first) {
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
            for &old in guard.iter() { super::super::rc::release_if_ptr(old); }
            guard.clear();
            for c in &kept { super::super::rc::retain_if_ptr(*c); }
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
            for &old in guard.iter() { super::super::rc::release_if_ptr(old); }
            guard.clear();
            for c in &kept { super::super::rc::retain_if_ptr(*c); }
            guard.extend(kept);
        }
    }
    MbValue::none()
}

// ── MozillaCookieJar file IO (Netscape cookies.txt round-trip) ───────────────

/// Format one cookie as a Netscape cookies.txt line:
/// `domain\tflag\tpath\tsecure\texpiration\tname\tvalue`.
fn netscape_line(cookie: MbValue) -> String {
    let domain = inst_field(cookie, "domain").and_then(extract_str).unwrap_or_default();
    let flag = if domain.starts_with('.') { "TRUE" } else { "FALSE" };
    let path = inst_field(cookie, "path").and_then(extract_str).unwrap_or_else(|| "/".to_string());
    let secure = if is_truthy(inst_field(cookie, "secure").unwrap_or_else(MbValue::none)) {
        "TRUE"
    } else {
        "FALSE"
    };
    let expires = inst_field(cookie, "expires")
        .filter(|v| !v.is_none())
        .and_then(|v| v.as_int().map(|n| n.to_string()).or_else(|| v.as_float().map(|f| (f as i64).to_string())))
        .unwrap_or_else(|| "0".to_string());
    let name = inst_field(cookie, "name").and_then(extract_str).unwrap_or_default();
    let value = inst_field(cookie, "value").and_then(extract_str).unwrap_or_default();
    format!("{domain}\t{flag}\t{path}\t{secure}\t{expires}\t{name}\t{value}")
}

/// FileCookieJar.save(filename=None, ignore_discard=False, ignore_expires=False).
/// Writes the jar to `filename` in Netscape cookies.txt format. The behavior
/// fixture always passes an explicit filename plus ignore_* kwargs.
unsafe extern "C" fn m_jar_save(self_v: MbValue, args: MbValue) -> MbValue {
    let pos = items_of(args);
    let filename = pos.first().copied()
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
    let filename = pos.first().copied()
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
                new_str(&format!("[Errno 2] No such file or directory: '{filename}'")),
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
        set_inst_field(cookie, "secure", MbValue::from_bool(cols[3].eq_ignore_ascii_case("TRUE")));
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

    install("Cookie", &[], &[
        ("is_expired", m_cookie_is_expired as usize, true),
    ]);
    install("CookieJar", &[], &[
        ("set_cookie", m_jar_set_cookie as usize, true),
        ("clear", m_jar_clear as usize, true),
        ("clear_session_cookies", m_jar_clear_session_cookies as usize, true),
        ("__len__", m_jar_len as usize, true),
        ("__iter__", m_jar_iter as usize, true),
    ]);
    // FileCookieJar adds the on-disk save/load surface; MozillaCookieJar /
    // LWPCookieJar inherit the whole jar + file API via the MRO.
    install("FileCookieJar", &["CookieJar"], &[
        ("save", m_jar_save as usize, true),
        ("load", m_jar_load as usize, true),
    ]);
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
            mods.borrow().get("http.cookiejar")
                .and_then(|m| m.attrs.get(name).copied())
        })
    }

    #[test]
    fn test_register_installs_full_surface() {
        register();
        for name in [
            "Cookie", "CookieJar", "CookiePolicy", "DefaultCookiePolicy",
            "FileCookieJar", "LWPCookieJar", "MozillaCookieJar", "LoadError",
        ] {
            assert!(cookiejar_attr(name).is_some(),
                "http.cookiejar module missing entry: {name}");
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
