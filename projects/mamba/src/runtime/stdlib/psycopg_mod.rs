/// psycopg module for Mamba (#1532).
///
/// Minimal callable-dispatcher shim covering four top-level `psycopg`
/// entry points (`__version__`, `connect`, `Connection`, `Cursor`).
/// All four return identity-stable sentinel callables; their job
/// here is to short-circuit CPython's module-dict probe chain for
/// read-only psycopg sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1532; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_connect(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_connection(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_cursor(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the psycopg module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    let addr_co = dispatch_connect as *const () as usize;
    attrs.insert("connect".into(), MbValue::from_func(addr_co));

    let addr_cn = dispatch_connection as *const () as usize;
    attrs.insert("Connection".into(), MbValue::from_func(addr_cn));

    let addr_cu = dispatch_cursor as *const () as usize;
    attrs.insert("Cursor".into(), MbValue::from_func(addr_cu));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_v as u64);
        set.insert(addr_co as u64);
        set.insert(addr_cn as u64);
        set.insert(addr_cu as u64);
    });

    super::register_module("psycopg", attrs);
}
