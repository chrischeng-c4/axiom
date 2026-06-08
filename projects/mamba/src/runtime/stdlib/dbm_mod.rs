/// dbm module for Mamba (#1261).
///
/// Minimal callable-dispatcher shim covering the top-level `dbm`
/// entry points (`open`, `whichdb`, `error`) plus the `dbm.dumb`
/// pure-Python backend submodule (`open`, `error`).
///
/// `dbm.error` mirrors CPython, which exposes it as a *tuple* of
/// exception classes — `(dbm.error, _gdbm.error, _dbm.error, ...)` —
/// where the lead element `dbm.error` is itself an `OSError`
/// subclass. The shim registers `dbm.error` and `dbm.dumb.error`
/// in the class registry with base `OSError` (so `issubclass(...,
/// OSError)` resolves) and exposes `dbm.error` as a 2-tuple of the
/// str-named class plus `OSError`, so `type(dbm.error).__name__ ==
/// "tuple"` holds while the lead element stays a real exception
/// type.
///
/// `open` / `whichdb` / `dbm.dumb.open` remain identity-stable
/// sentinel callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only dbm sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1261; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the
/// stub-only conversion long-tail has closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_open(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_whichdb(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_dumb_open(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the dbm module (and the `dbm.dumb` submodule).
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_o = dispatch_open as *const () as usize;
    attrs.insert("open".into(), MbValue::from_func(addr_o));

    let addr_w = dispatch_whichdb as *const () as usize;
    attrs.insert("whichdb".into(), MbValue::from_func(addr_w));

    // `dbm.error` is a tuple of exception classes on CPython, with the lead
    // element being a real `OSError` subclass. Register the lead class with
    // base `OSError` so the str-named element resolves to it, then expose the
    // module attribute as a 2-tuple `(dbm.error, OSError)`. This makes
    // `type(dbm.error).__name__ == "tuple"` hold while keeping the lead
    // element an exception type.
    super::super::class::mb_class_register(
        "dbm.error",
        vec!["OSError".to_string()],
        HashMap::new(),
    );
    let error_tuple = MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_str("dbm.error".to_string())),
        MbValue::from_ptr(MbObject::new_str("OSError".to_string())),
    ]));
    attrs.insert("error".into(), error_tuple);

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_o as u64);
        set.insert(addr_w as u64);
    });

    super::register_module("dbm", attrs);

    register_dumb();
}

/// Register the `dbm.dumb` pure-Python backend submodule.
///
/// CPython's `dbm.dumb` exposes `open()` (callable) and `error`, the
/// dumb-backend exception type, declared as `class error(OSError)`.
fn register_dumb() {
    let mut attrs = HashMap::new();

    let addr_d = dispatch_dumb_open as *const () as usize;
    attrs.insert("open".into(), MbValue::from_func(addr_d));

    // `dbm.dumb.error` is `class error(OSError)`. Register it with base
    // `OSError` so `issubclass(dbm.dumb.error, OSError)` resolves; the module
    // attribute is the str-named class that maps straight to the registry
    // entry.
    super::super::class::mb_class_register(
        "dbm.dumb.error",
        vec!["OSError".to_string()],
        HashMap::new(),
    );
    attrs.insert(
        "error".into(),
        MbValue::from_ptr(MbObject::new_str("dbm.dumb.error".to_string())),
    );

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr_d as u64);
    });

    super::register_module("dbm.dumb", attrs);
}
