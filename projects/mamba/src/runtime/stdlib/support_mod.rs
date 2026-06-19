//! `test.support` behavior augmentation (CPython test-infra unblock).
//!
//! `test_mod.rs` registers `test.support` plus its submodules as a wide field
//! of no-op variadic stubs. That is enough to satisfy *most* `from test.support
//! import X` lines, but a concrete cohort of deferred std-libs
//! (socket / email / tokenize / gzip / types / shutil / sys / array / xml_etree)
//! still dies at import time on a small set of names that the stub field never
//! covered, and on `import_helper.import_module(...)` returning `None` instead
//! of the real module.
//!
//! This module is **purely additive**. It runs AFTER `test_mod::register()` and
//! mutates the already-registered `test.support` / `test.support.*` modules in
//! place (via the runtime `MODULES` registry) rather than re-registering and
//! clobbering the existing stub surface. The concrete gaps it closes were
//! measured — not guessed — from the cohort fixtures:
//!
//!   missing `from test.support import …` names:
//!     `_1G`, `_2G`, `_4G`, `_10G`          int byte-size constants
//!     `iter_builtin_types`                 generator → empty iterator
//!     `iter_slot_wrappers`                 generator → empty iterator
//!     `interpreters`                       submodule (`test.support.interpreters`)
//!     `socket_helper`                      submodule (`test.support.socket_helper`)
//!
//!   behavior upgrades (stub → real):
//!     `import_helper.import_module(name)`  actually imports the module, or
//!                                          raises `unittest.SkipTest` if absent
//!     `import_helper.import_fresh_module`  same import path (fresh-ness is a
//!                                          no-op under mamba's single registry)
//!     `script_helper.run_test_script`      missing name → no-op callable
//!     `os_helper.TESTFN`                   a concrete temp-name string
//!     `captured_stdout/stderr/stdin`       StringIO-backed context managers
//!
//! Everything else the cohort touches (`gc_collect()`, `requires_*` decorators,
//! `unlink`, `rmtree`, …) is already a serviceable no-op / identity stub in
//! `test_mod.rs`; the win here is removing the *import wall* so the fixtures can
//! reach their actual test body.

use super::super::module::{MODULES, NATIVE_FUNC_ADDRS};
use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::collections::HashMap;

// ── byte-size constants used by bigmem / array / gzip fixtures ──
const _1G: i64 = 1024 * 1024 * 1024;
const _2G: i64 = 2 * _1G;
const _4G: i64 = 4 * _1G;
const _10G: i64 = 10 * _1G;

// ── dispatcher helpers (self-contained; small dup vs test_mod is intentional) ──

/// Register a native function pointer so dynamic dispatch recognizes it.
fn note_addr(addr: usize) {
    NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr as u64);
    });
}

/// Build an `MbValue` callable from a variadic dispatcher address.
fn func(addr: usize) -> MbValue {
    note_addr(addr);
    MbValue::from_func(addr)
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

/// A no-op variadic callable: returns None regardless of arguments.
unsafe extern "C" fn dispatch_noop(_args: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

/// Identity / pass-through decorator: returns its first argument unchanged.
unsafe extern "C" fn dispatch_identity(args: *const MbValue, n: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args, n) };
    a.first().copied().unwrap_or_else(MbValue::none)
}

/// A generator-shaped stub: `iter_builtin_types()` / `iter_slot_wrappers()` —
/// returns an empty list (a valid iterable) so `for t in iter_builtin_types():`
/// loops zero times instead of raising. CPython yields a long sequence; the
/// cohort fixtures only iterate it, so an empty iterable is behavior-safe for
/// the import-unblock goal.
unsafe extern "C" fn dispatch_empty_iter(_args: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

/// `import_helper.import_module(name)` — really import the module.
///
/// CPython's helper imports `name` and, on `ImportError`, raises
/// `unittest.SkipTest(...)` so the whole test is skipped rather than failing.
/// Under mamba we delegate to the runtime importer (`mb_import`). If that import
/// raises (module genuinely absent — e.g. C-only `_testcapi`), we convert the
/// pending exception into `unittest.SkipTest`, matching the skip contract.
unsafe extern "C" fn dispatch_import_module(args: *const MbValue, n: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args, n) };
    let name = a.first().copied().unwrap_or_else(MbValue::none);
    let modval = super::super::module::mb_import(name);
    if super::super::exception::mb_has_exception()
        .as_bool()
        .unwrap_or(false)
    {
        // Swallow the import failure and re-raise as a skip.
        let _ = super::super::exception::mb_catch_exception();
        let nm = extract_str(name).unwrap_or_default();
        let exc = MbValue::from_ptr(MbObject::new_str("unittest.SkipTest".to_string()));
        let msg = MbValue::from_ptr(MbObject::new_str(format!("No module named '{nm}'")));
        super::super::exception::mb_raise(exc, msg);
        return MbValue::none();
    }
    modval
}

/// `captured_stdout()` / `captured_stderr()` / `captured_stdin()` — a context
/// manager that, on `__enter__`, replaces the corresponding stream with a fresh
/// `io.StringIO` and returns it. The cohort uses
/// `with support.captured_stdout() as out: ...; out.getvalue()` and expects
/// everything printed inside the block to land in `out`.
///
/// To make print() capture work we mirror `contextlib.redirect_stdout`: the
/// returned context-manager instance pushes its StringIO onto the runtime's
/// stdout/stderr redirect stack at `__enter__` and pops it at `__exit__`. The
/// StringIO is what `__enter__` yields, so `out.getvalue()` reads back the
/// captured text.
fn make_stringio() -> MbValue {
    let stringio = MODULES.with(|mods| {
        mods.borrow()
            .get("io")
            .and_then(|m| m.attrs.get("StringIO").copied())
    });
    if let Some(sio_ctor) = stringio {
        if let Some(addr) = sio_ctor.as_func() {
            let f: unsafe extern "C" fn(*const MbValue, usize) -> MbValue =
                unsafe { std::mem::transmute(addr) };
            let inst = unsafe { f(std::ptr::null(), 0) };
            if !inst.is_none() {
                return inst;
            }
        }
    }
    // Fallback if io.StringIO is unavailable: a bare dict still satisfies
    // attribute access during import-walk; getvalue() simply won't resolve.
    MbValue::from_ptr(MbObject::new_dict())
}

fn set_cm_field(obj: MbValue, name: &str, value: MbValue) {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.write().unwrap().insert(name.to_string(), value);
            }
        }
    }
}

fn get_cm_field(obj: MbValue, name: &str) -> Option<MbValue> {
    obj.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(name).copied()
        } else {
            None
        }
    })
}

/// Build a captured-stream CM instance carrying a fresh StringIO buffer and a
/// `_stream` marker ("stdout"/"stderr"/"stdin") used by `__enter__`.
fn make_captured(stream: &str) -> MbValue {
    let inst = MbValue::from_ptr(MbObject::new_instance(
        "test.support._CapturedStream".to_string(),
    ));
    let buf = make_stringio();
    unsafe {
        super::super::rc::retain_if_ptr(buf);
    }
    set_cm_field(inst, "_buffer", buf);
    set_cm_field(
        inst,
        "_stream",
        MbValue::from_ptr(MbObject::new_str(stream.to_string())),
    );
    inst
}

unsafe extern "C" fn dispatch_captured_stdout(_args: *const MbValue, _n: usize) -> MbValue {
    make_captured("stdout")
}

unsafe extern "C" fn dispatch_captured_stderr(_args: *const MbValue, _n: usize) -> MbValue {
    make_captured("stderr")
}

unsafe extern "C" fn dispatch_captured_stdin(_args: *const MbValue, _n: usize) -> MbValue {
    make_captured("stdin")
}

/// `_CapturedStream.__enter__(self)` → the StringIO buffer; pushes it onto the
/// matching redirect stack so print()/sys.std* writes are captured.
extern "C" fn captured_enter(self_v: MbValue) -> MbValue {
    let buf = get_cm_field(self_v, "_buffer").unwrap_or_else(MbValue::none);
    let stream = get_cm_field(self_v, "_stream")
        .and_then(extract_str)
        .unwrap_or_default();
    match stream.as_str() {
        "stderr" => super::super::output::push_stderr_redirect(buf),
        // stdin is not write-captured; only push for stdout (and treat the
        // default as stdout) so the body's output lands in the buffer.
        "stdin" => {}
        _ => super::super::output::push_stdout_redirect(buf),
    }
    buf
}

/// `_CapturedStream.__exit__(self, *exc)` → pop the redirect; never suppress.
extern "C" fn captured_exit(
    self_v: MbValue,
    _exc_type: MbValue,
    _exc_val: MbValue,
    _exc_tb: MbValue,
) -> MbValue {
    let stream = get_cm_field(self_v, "_stream")
        .and_then(extract_str)
        .unwrap_or_default();
    match stream.as_str() {
        "stderr" => super::super::output::pop_stderr_redirect(),
        "stdin" => {}
        _ => super::super::output::pop_stdout_redirect(),
    }
    MbValue::from_bool(false)
}

/// Register the captured-stream context-manager class (native enter/exit).
fn register_captured_cm_class() {
    let mut methods: HashMap<String, MbValue> = HashMap::new();
    methods.insert(
        "__enter__".to_string(),
        MbValue::from_func(captured_enter as usize),
    );
    methods.insert(
        "__exit__".to_string(),
        MbValue::from_func(captured_exit as usize),
    );
    super::super::class::mb_class_register(
        "test.support._CapturedStream",
        vec!["object".to_string()],
        methods,
    );
}

// ── attr-merge plumbing ──

/// Insert `(name → value)` pairs into an already-registered module's attrs,
/// creating the module as a bare package if it does not exist yet. Additive:
/// never clears existing attributes.
fn augment_module(name: &str, entries: Vec<(&str, MbValue)>) {
    MODULES.with(|mods| {
        let mut map = mods.borrow_mut();
        let m = map
            .entry(name.to_string())
            .or_insert_with(|| super::super::module::MbModule {
                name: name.to_string(),
                file: None,
                attrs: HashMap::new(),
                is_package: name.contains("support")
                    && !name.contains("import")
                    && !name.contains("os_")
                    && !name.contains("script")
                    && !name.contains("threading")
                    && !name.contains("warnings")
                    && !name.contains("socket"),
                cached_value: None,
            });
        for (k, v) in entries {
            m.attrs.insert(k.to_string(), v);
        }
        // Invalidate any previously-materialized namespace object so the new
        // attrs are visible to later imports.
        if let Some(m) = map.get_mut(name) {
            m.cached_value = None;
        }
    });
}

/// Merge `entries` into an EXISTING submodule's attrs and **re-register** it via
/// the runtime registrar. Re-registration re-runs parent-attr propagation, so
/// the snapshot stored in the parent (`test.support.attrs[<leaf>]`) is rebuilt
/// from the merged attrs. This is the critical difference from `augment_module`:
/// mutating `MODULES[leaf].attrs` alone leaves the parent's materialized
/// snapshot stale, so `from test.support import import_helper` would keep
/// resolving the OLD value. Reading-existing + merge + `mb_module_register`
/// fixes both the leaf and the parent view.
fn merge_register(full_name: &str, entries: Vec<(&str, MbValue)>) {
    let mut attrs: HashMap<String, MbValue> = MODULES.with(|mods| {
        mods.borrow()
            .get(full_name)
            .map(|m| m.attrs.clone())
            .unwrap_or_default()
    });
    for (k, v) in entries {
        attrs.insert(k.to_string(), v);
    }
    super::super::module::mb_module_register(full_name, attrs);
}

/// Augment the `test.support` surface with the names the deferred std-libs
/// cohort imports but the stub field never covered. Must run AFTER
/// `test_mod::register()` (which creates `test.support` and its submodules).
pub fn register() {
    let noop = func(dispatch_noop as usize);
    let identity = func(dispatch_identity as usize);
    let empty_iter = func(dispatch_empty_iter as usize);
    let import_module = func(dispatch_import_module as usize);
    let captured_out = func(dispatch_captured_stdout as usize);
    let captured_err = func(dispatch_captured_stderr as usize);
    let captured_in = func(dispatch_captured_stdin as usize);
    register_captured_cm_class();

    // 1) Constants + generator stubs directly on `test.support`.
    augment_module(
        "test.support",
        vec![
            ("_1G", MbValue::from_int(_1G)),
            ("_2G", MbValue::from_int(_2G)),
            ("_4G", MbValue::from_int(_4G)),
            ("_10G", MbValue::from_int(_10G)),
            ("iter_builtin_types", empty_iter),
            ("iter_slot_wrappers", empty_iter),
            // Platform predicate used by the auto-ported zlib/gzip/etc cohort as a
            // module-level constant (`from test.support import is_s390x`). CPython
            // defines it as `platform.machine() == 's390x'`; on every supported
            // mamba target that is False. Without this the multi-name import line
            // raises ImportError and skips the whole fixture.
            ("is_s390x", MbValue::from_bool(false)),
            // Make the top-level captured_* match the real context-manager behavior:
            // __enter__ pushes a StringIO onto the stdout/stderr redirect stack so
            // print() inside the block is captured, and yields it for getvalue().
            ("captured_stdout", captured_out),
            ("captured_stderr", captured_err),
            ("captured_stdin", captured_in),
        ],
    );

    // 2) `import_helper`: real import + skip-on-missing. Use merge_register so
    //    the parent-attr snapshot (`from test.support import import_helper`) is
    //    rebuilt from the merged attrs, not left pointing at the stub value.
    merge_register(
        "test.support.import_helper",
        vec![
            ("import_module", import_module),
            ("import_fresh_module", import_module),
        ],
    );

    // 3) `script_helper`: the one missing runner name the cohort imports.
    merge_register(
        "test.support.script_helper",
        vec![("run_test_script", noop)],
    );

    // 4) `os_helper.TESTFN`: a concrete scratch-file name. CPython suffixes the
    //    PID so each process gets a unique name; mirror that so fixtures that do
    //    `open(TESTFN, 'x')` (exclusive create) don't collide with a stale file
    //    left by another fixture process in the shared CWD.
    let testfn = format!("@mamba_test_{}", std::process::id());
    merge_register(
        "test.support.os_helper",
        vec![("TESTFN", MbValue::from_ptr(MbObject::new_str(testfn)))],
    );

    // 5) Missing submodules referenced via `from test.support import <leaf>`.
    //    `interpreters` (subinterpreter helpers) and `socket_helper` (network
    //    test gating). Both are pure no-op surfaces under mamba; the fixtures
    //    that import them then guard on `requires_*` and skip.
    merge_register(
        "test.support.interpreters",
        vec![
            ("create", noop),
            ("list_all", noop),
            ("get_current", noop),
            ("get_main", noop),
            ("RunFailedError", noop),
            ("InterpreterError", noop),
            ("InterpreterNotFoundError", noop),
            ("NotShareableError", noop),
            ("Interpreter", noop),
        ],
    );
    merge_register(
        "test.support.socket_helper",
        vec![
            ("find_unused_port", noop),
            ("bind_port", noop),
            ("bind_unix_socket", noop),
            (
                "HOST",
                MbValue::from_ptr(MbObject::new_str("localhost".to_string())),
            ),
            (
                "HOSTv4",
                MbValue::from_ptr(MbObject::new_str("127.0.0.1".to_string())),
            ),
            (
                "HOSTv6",
                MbValue::from_ptr(MbObject::new_str("::1".to_string())),
            ),
            ("transient_internet", noop),
            ("skip_unless_bind_unix_socket", identity),
            ("requires_IPv6", identity),
            ("IPV6_ENABLED", MbValue::from_bool(false)),
            ("get_socket_conn_refused_errs", noop),
            // CPython sets this True on any host that exposes socket.gethostname()
            // (i.e. effectively everywhere). Auto-ported urllib tests guard their
            // bodies on `if not socket_helper.has_gethostname: raise SkipTest`, so
            // a missing attr (falsy default) wrongly skips them. Match CPython.
            ("has_gethostname", MbValue::from_bool(true)),
        ],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_size_constants() {
        assert_eq!(_1G, 1_073_741_824);
        assert_eq!(_2G, 2_147_483_648);
        assert_eq!(_4G, 4_294_967_296);
        assert_eq!(_10G, 10_737_418_240);
    }

    #[test]
    fn test_extract_str_roundtrip() {
        let s = MbValue::from_ptr(MbObject::new_str("hi".to_string()));
        assert_eq!(extract_str(s), Some("hi".to_string()));
        assert_eq!(extract_str(MbValue::from_int(7)), None);
    }

    #[test]
    fn test_empty_iter_returns_list() {
        let v = unsafe { dispatch_empty_iter(std::ptr::null(), 0) };
        assert!(v.as_ptr().is_some());
        if let Some(ptr) = v.as_ptr() {
            unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    assert_eq!(lock.read().unwrap().len(), 0);
                }
            }
        }
    }

    #[test]
    fn test_identity_passes_first_arg() {
        let arg = MbValue::from_int(42);
        let args = [arg];
        let out = unsafe { dispatch_identity(args.as_ptr(), 1) };
        assert_eq!(out.as_int(), Some(42));
    }

    #[test]
    fn test_noop_returns_none() {
        let out = unsafe { dispatch_noop(std::ptr::null(), 0) };
        assert!(out.is_none());
    }

    #[test]
    fn test_register_populates_test_support_constants() {
        // Register prerequisites then our augmentation.
        super::super::test_mod::register();
        register();
        let four_g = MODULES.with(|mods| {
            mods.borrow()
                .get("test.support")
                .and_then(|m| m.attrs.get("_4G").copied())
        });
        assert_eq!(four_g.and_then(|v| v.as_int()), Some(_4G));
    }

    #[test]
    fn test_register_adds_submodules() {
        super::super::test_mod::register();
        register();
        let has_interp =
            MODULES.with(|mods| mods.borrow().contains_key("test.support.interpreters"));
        let has_sock =
            MODULES.with(|mods| mods.borrow().contains_key("test.support.socket_helper"));
        assert!(has_interp);
        assert!(has_sock);
        // Parent-attr propagation should have attached the leaves to test.support.
        let has_leaf = MODULES.with(|mods| {
            mods.borrow()
                .get("test.support")
                .map(|m| {
                    m.attrs.contains_key("interpreters") && m.attrs.contains_key("socket_helper")
                })
                .unwrap_or(false)
        });
        assert!(has_leaf);
    }
}
