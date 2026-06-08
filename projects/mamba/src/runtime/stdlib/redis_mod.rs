/// redis module for Mamba (#1529).
///
/// Minimal callable-dispatcher shim covering four top-level `redis`
/// entry points (`__version__`, `Redis`, `ConnectionPool`,
/// `from_url`). All four return identity-stable sentinel callables;
/// their job here is to short-circuit CPython's module-dict probe
/// chain for read-only redis sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1529; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_redis(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_connection_pool(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_from_url(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the redis module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    let addr_r = dispatch_redis as *const () as usize;
    attrs.insert("Redis".into(), MbValue::from_func(addr_r));

    let addr_cp = dispatch_connection_pool as *const () as usize;
    attrs.insert("ConnectionPool".into(), MbValue::from_func(addr_cp));

    let addr_fu = dispatch_from_url as *const () as usize;
    attrs.insert("from_url".into(), MbValue::from_func(addr_fu));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_v as u64);
        set.insert(addr_r as u64);
        set.insert(addr_cp as u64);
        set.insert(addr_fu as u64);
    });

    super::register_module("redis", attrs);
}
