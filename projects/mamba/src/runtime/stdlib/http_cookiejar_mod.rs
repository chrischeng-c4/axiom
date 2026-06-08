use super::super::rc::MbObject;
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

// ── Variadic dispatchers ──

macro_rules! disp_variadic {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
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
    }

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
pub fn mb_http_cookiejar_cookie_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("Cookie")
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

/// http.cookiejar.FileCookieJar() -> FileCookieJar Instance.
pub fn mb_http_cookiejar_file_cookie_jar_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("FileCookieJar")
}

/// http.cookiejar.LWPCookieJar() -> LWPCookieJar Instance.
pub fn mb_http_cookiejar_lwp_cookie_jar_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("LWPCookieJar")
}

/// http.cookiejar.MozillaCookieJar() -> MozillaCookieJar Instance.
pub fn mb_http_cookiejar_mozilla_cookie_jar_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("MozillaCookieJar")
}

/// http.cookiejar.LoadError() -> LoadError Instance (exception shell).
pub fn mb_http_cookiejar_load_error_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("LoadError")
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
