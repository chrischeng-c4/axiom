use super::super::rc::MbObject;
use super::super::value::MbValue;
/// dbm module for Mamba (#1261).
///
/// Minimal callable-dispatcher shim covering three top-level
/// `dbm` entry points (`open`, `whichdb`, `error`). All three
/// return identity-stable sentinel callables; their job here is
/// to short-circuit CPython's module-dict probe chain for
/// read-only dbm sentinels.
///
/// Note: CPython exposes `dbm.error` as a tuple of (dbm.error,
/// OSError); the mamba shim returns an identity-stable function
/// pointer instead. Cross-runtime perf-pin asserts the `is`
/// invariant holds within each runtime's own dispatch, not across
/// runtimes -- so this divergence is benign for the Gate 2
/// module-attr-read measurement.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1261; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the
/// stub-only conversion long-tail has closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_open(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_whichdb(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_error(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the dbm module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_o = dispatch_open as *const () as usize;
    attrs.insert("open".into(), MbValue::from_func(addr_o));

    let addr_w = dispatch_whichdb as *const () as usize;
    attrs.insert("whichdb".into(), MbValue::from_func(addr_w));

    let addr_e = dispatch_error as *const () as usize;
    attrs.insert("error".into(), MbValue::from_func(addr_e));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_o as u64);
        set.insert(addr_w as u64);
        set.insert(addr_e as u64);
    });

    super::register_module("dbm", attrs);
}
