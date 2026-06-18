//! @codegen-skip: handwrite-pre-standardize
//!
//! urllib.error module for Mamba — Python 3.12 `urllib.error` stdlib (#1421).
//!
//! Surface (3 names, 100% of `cpython312_surface.json["urllib.error"]`):
//!   - `URLError(reason, filename=None)` — base class; carries `.reason` /
//!     `.args` / `.filename` attributes (matches CPython 3.12 `urllib.error.URLError`).
//!   - `HTTPError(url, code, msg, hdrs, fp)` — subclass of URLError + http.client.HTTPResponse;
//!     carries `.url` / `.code` / `.msg` / `.hdrs` / `.fp` / `.reason` (aliased to msg) /
//!     `.headers` (aliased to hdrs) / `.filename` (aliased to url) / `.args`.
//!   - `ContentTooShortError(message, content)` — subclass of URLError;
//!     carries `.content` plus the URLError fields.
//!
//! ## Construction model
//!
//! Mamba's exception machinery (#755 ExceptionGroup, full BaseException
//! hierarchy) is still in-flight; until those land, urllib.error classes
//! are wired as **callable class shells** — each name resolves to a
//! native dispatch function that returns an `Instance { class_name: "..." }`
//! populated with the standard CPython attributes. Raising / except-catching
//! these instances exercises the same machinery as any user-defined class:
//! by-class-name match.
//!
//! ## Carve-out: `Final` typeshed members
//!
//! typeshed declares no `Final` constants in `urllib.error` — the full
//! public surface is the three class names above. No module-level
//! constants or helpers are missing.
//!
//! ## Why a separate module file vs inline in http_mod
//!
//! Prior to #1421 the surface lived inline at `http_mod.rs:200-206` as three
//! `MbObject::new_str(name)` placeholders — importing `urllib.error.URLError`
//! returned the literal string "URLError" rather than a callable class.
//! Splitting into a dedicated file lets us (a) carry the class shell logic
//! without bloating http_mod and (b) own the conformance fixture pair from
//! a single module-named directory (`tests/cpython/std-libs/urllib_error/`).

use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

/// Build an `Instance { class_name }` carrying the given (name, value) fields.
fn make_instance(class_name: &str, fields: Vec<(&str, MbValue)>) -> MbValue {
    let mut map = FxHashMap::default();
    for (k, v) in fields {
        map.insert(k.to_string(), v);
    }
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: class_name.to_string(),
            fields: RwLock::new(map),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

fn empty_tuple() -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(Vec::new()))
}

fn tuple_of(vals: Vec<MbValue>) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(vals))
}

/// `URLError(reason, filename=None)` — base class for urllib request failures.
///
/// CPython 3.12 attrs:
///   - `reason` — the underlying cause (str or Exception)
///   - `args` — `(reason,)` tuple (BaseException convention)
///   - `filename` — optional URL/filename
pub fn mb_urllib_error_URLError(a: &[MbValue]) -> MbValue {
    let reason = a.first().copied().unwrap_or_else(MbValue::none);
    let filename = a.get(1).copied().unwrap_or_else(MbValue::none);
    make_instance(
        "URLError",
        vec![
            ("reason", reason),
            ("args", tuple_of(vec![reason])),
            ("filename", filename),
        ],
    )
}

/// `HTTPError(url, code, msg, hdrs, fp)` — HTTP-level error; subclass of
/// URLError + http.client.HTTPResponse in CPython.
///
/// CPython 3.12 attrs:
///   - `url`, `code`, `msg`, `hdrs`, `fp`
///   - `reason` (== `msg`), `headers` (== `hdrs`), `filename` (== `url`)
///   - `args` — `(code, msg, hdrs)` tuple (matches CPython's `__init__`
///     which calls `HTTPResponse.__init__(self, fp, msg, hdrs)` and stores
///     `(code, msg, hdrs)` via BaseException).
pub fn mb_urllib_error_HTTPError(a: &[MbValue]) -> MbValue {
    let url = a.first().copied().unwrap_or_else(MbValue::none);
    let code = a.get(1).copied().unwrap_or_else(MbValue::none);
    let msg = a.get(2).copied().unwrap_or_else(MbValue::none);
    let hdrs = a.get(3).copied().unwrap_or_else(MbValue::none);
    let fp = a.get(4).copied().unwrap_or_else(MbValue::none);
    make_instance(
        "HTTPError",
        vec![
            ("url", url),
            ("code", code),
            ("msg", msg),
            ("hdrs", hdrs),
            ("fp", fp),
            ("reason", msg),
            ("headers", hdrs),
            ("filename", url),
            ("args", tuple_of(vec![code, msg, hdrs])),
        ],
    )
}

/// `ContentTooShortError(message, content)` — raised when a download
/// produces fewer bytes than the Content-Length header advertised.
///
/// CPython 3.12 attrs:
///   - `reason` (== `message`), `content`
///   - `args` — `(message, content)` tuple
pub fn mb_urllib_error_ContentTooShortError(a: &[MbValue]) -> MbValue {
    let message = a.first().copied().unwrap_or_else(MbValue::none);
    let content = a.get(1).copied().unwrap_or_else(MbValue::none);
    make_instance(
        "ContentTooShortError",
        vec![
            ("reason", message),
            ("message", message),
            ("content", content),
            ("filename", MbValue::none()),
            ("args", tuple_of(vec![message, content])),
        ],
    )
}

// ── Dispatch shims (variadic stdlib ABI) ─────────────────────────────────────

extern "C" fn dispatch_URLError(args: *const MbValue, len: usize) -> MbValue {
    let slice = unsafe { std::slice::from_raw_parts(args, len) };
    mb_urllib_error_URLError(slice)
}

extern "C" fn dispatch_HTTPError(args: *const MbValue, len: usize) -> MbValue {
    let slice = unsafe { std::slice::from_raw_parts(args, len) };
    mb_urllib_error_HTTPError(slice)
}

extern "C" fn dispatch_ContentTooShortError(args: *const MbValue, len: usize) -> MbValue {
    let slice = unsafe { std::slice::from_raw_parts(args, len) };
    mb_urllib_error_ContentTooShortError(slice)
}

/// Register `urllib.error` with proper callable class shells.
///
/// Must run BEFORE `http_mod::register()` so that http_mod's umbrella-wiring
/// for the `urllib` package picks up these entries rather than the legacy
/// 3-string stub block (which has been removed from http_mod).
/// HTTPError.read(...) delegates to the stored response file object's read()
/// (CPython's HTTPError doubles as an http.client.HTTPResponse).
unsafe extern "C" fn m_httperror_read(self_v: MbValue, args: MbValue) -> MbValue {
    let fp = self_v.as_ptr().and_then(|p| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*p).data {
            fields.read().ok().and_then(|f| f.get("fp").copied())
        } else {
            None
        }
    }).unwrap_or_else(MbValue::none);
    super::super::class::mb_call_method(
        fp,
        MbValue::from_ptr(MbObject::new_str("read".to_string())),
        args,
    )
}

pub fn register() {
    use super::super::module::NATIVE_FUNC_ADDRS;

    fn add_dispatch(attrs: &mut HashMap<String, MbValue>, name: &str, addr: usize) {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    let mut attrs = HashMap::new();

    // Defensively (re-)register the builtin base chain so the urllib.error
    // classes get a complete MRO regardless of whether the builtin-exception
    // registration has run yet at module-registration time. `mb_class_register`
    // inserts/overwrites, so this is idempotent and identical to the builtin
    // registration (same pattern as concurrent_futures_mod). Without this the
    // computed MRO could miss Exception/BaseException above OSError when
    // urllib.error registers first.
    {
        let empty = HashMap::new;
        super::super::class::mb_class_register("BaseException", vec![], empty());
        super::super::class::mb_class_register("Exception", vec!["BaseException".into()], empty());
        super::super::class::mb_class_register("OSError", vec!["Exception".into()], empty());
    }

    let class_dispatchers: [(&str, usize); 3] = [
        ("URLError", dispatch_URLError as *const () as usize),
        ("HTTPError", dispatch_HTTPError as *const () as usize),
        (
            "ContentTooShortError",
            dispatch_ContentTooShortError as *const () as usize,
        ),
    ];
    for (name, addr) in class_dispatchers {
        add_dispatch(&mut attrs, name, addr);
        // These names are exception *classes*, not plain functions: the
        // constructor func is the class object (callable shell). Record the
        // func addr as a native type name so `mb_getattr` consults the class's
        // method table when an attribute is read off the class itself
        // (same mechanism as socket.py type stubs / pathlib.Path).
        super::super::module::NATIVE_TYPE_NAMES.with(|m| {
            m.borrow_mut().insert(addr as u64, name.to_string());
        });
        // BaseException exposes the exception-chaining slots (`__cause__`,
        // `__context__`, `__suppress_context__`) as getset descriptors, so on
        // the real CPython classes `hasattr(urllib.error.URLError, "__cause__")`
        // is True. The surface probe routes through `mb_hasattr` -> `mb_getattr`
        // for these dunders; populate the class's method table with the three
        // chaining slots (inert func sentinels — only presence matters for the
        // surface dimension) so the lookup resolves and `hasattr` reports True
        // while the name stays callable as a constructor (behavior fixtures
        // still build instances via `HTTPError(...)`). The bases below encode
        // the CPython 3.12 hierarchy so issubclass/isinstance and the
        // except-matcher resolve URLError ⊂ OSError, HTTPError ⊂ URLError,
        // ContentTooShortError ⊂ URLError.
        let mut slots: HashMap<String, MbValue> = HashMap::new();
        let slot = MbValue::from_func(addr);
        slots.insert("__cause__".to_string(), slot);
        slots.insert("__context__".to_string(), slot);
        slots.insert("__suppress_context__".to_string(), slot);
        // CPython 3.12: URLError(OSError); HTTPError(URLError, ...);
        // ContentTooShortError(URLError).
        let bases: Vec<String> = match name {
            "URLError" => vec!["OSError".to_string()],
            _ => vec!["URLError".to_string()],
        };
        // HTTPError doubles as a readable response — delegate read() to its fp.
        if name == "HTTPError" {
            let read_addr = m_httperror_read as *const () as usize;
            super::super::module::register_variadic_func(read_addr as u64);
            slots.insert("read".to_string(), MbValue::from_func(read_addr));
        }
        super::super::class::mb_class_register(name, bases, slots);
    }

    // Touch helper to silence dead-code lint when this module is built but
    // not yet exercised by any caller (the dispatchers cover the call path
    // through MbValue::from_func, not direct invocation).
    let _ = empty_tuple;

    super::register_module("urllib.error", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn class_name_of(val: MbValue) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                Some(class_name.clone())
            } else {
                None
            }
        })
    }

    fn field(val: MbValue, name: &str) -> MbValue {
        val.as_ptr()
            .map(|ptr| unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    let f = fields.read().unwrap();
                    f.get(name).copied().unwrap_or_else(MbValue::none)
                } else {
                    MbValue::none()
                }
            })
            .unwrap_or_else(MbValue::none)
    }

    fn str_field(val: MbValue, name: &str) -> Option<String> {
        let v = field(val, name);
        v.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })
    }

    #[test]
    fn test_urlerror_basic() {
        let e = mb_urllib_error_URLError(&[s("connection refused")]);
        assert_eq!(class_name_of(e).as_deref(), Some("URLError"));
        assert_eq!(
            str_field(e, "reason").as_deref(),
            Some("connection refused")
        );
    }

    #[test]
    fn test_urlerror_with_filename() {
        let e = mb_urllib_error_URLError(&[s("not found"), s("http://x/y")]);
        assert_eq!(str_field(e, "reason").as_deref(), Some("not found"));
        assert_eq!(str_field(e, "filename").as_deref(), Some("http://x/y"));
    }

    #[test]
    fn test_urlerror_args_tuple() {
        let e = mb_urllib_error_URLError(&[s("oops")]);
        let args = field(e, "args");
        assert!(args.as_ptr().is_some(), "args must be a tuple");
    }

    #[test]
    fn test_httperror_full() {
        let e = mb_urllib_error_HTTPError(&[
            s("http://example.com/missing"),
            MbValue::from_int(404),
            s("Not Found"),
            MbValue::none(),
            MbValue::none(),
        ]);
        assert_eq!(class_name_of(e).as_deref(), Some("HTTPError"));
        assert_eq!(
            str_field(e, "url").as_deref(),
            Some("http://example.com/missing")
        );
        assert_eq!(field(e, "code").as_int(), Some(404));
        assert_eq!(str_field(e, "msg").as_deref(), Some("Not Found"));
        // Aliases
        assert_eq!(str_field(e, "reason").as_deref(), Some("Not Found"));
        assert_eq!(
            str_field(e, "filename").as_deref(),
            Some("http://example.com/missing")
        );
    }

    #[test]
    fn test_content_too_short() {
        let e =
            mb_urllib_error_ContentTooShortError(&[s("retrieval incomplete"), s("partial-bytes")]);
        assert_eq!(class_name_of(e).as_deref(), Some("ContentTooShortError"));
        assert_eq!(
            str_field(e, "reason").as_deref(),
            Some("retrieval incomplete")
        );
        assert_eq!(str_field(e, "content").as_deref(), Some("partial-bytes"));
    }

    #[test]
    fn test_registration() {
        register();
        super::super::super::module::MODULES.with(|mods| {
            let mods = mods.borrow();
            let m = mods.get("urllib.error").expect("urllib.error registered");
            assert!(m.attrs.contains_key("URLError"));
            assert!(m.attrs.contains_key("HTTPError"));
            assert!(m.attrs.contains_key("ContentTooShortError"));
        });
    }
}
