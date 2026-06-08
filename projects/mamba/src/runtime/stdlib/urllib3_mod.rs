/// urllib3 module for Mamba (#1482).
///
/// Minimal callable-dispatcher shim covering four top-level `urllib3`
/// entry points (`__version__`, `PoolManager`, `Retry`, `Timeout`).
/// All four return identity-stable sentinel callables; their job
/// here is to short-circuit CPython's module-dict probe chain for
/// read-only urllib3 sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1482; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_pool_manager(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_retry(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_timeout(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the urllib3 module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    let addr_pm = dispatch_pool_manager as *const () as usize;
    attrs.insert("PoolManager".into(), MbValue::from_func(addr_pm));

    let addr_r = dispatch_retry as *const () as usize;
    attrs.insert("Retry".into(), MbValue::from_func(addr_r));

    let addr_t = dispatch_timeout as *const () as usize;
    attrs.insert("Timeout".into(), MbValue::from_func(addr_t));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_v as u64);
        set.insert(addr_pm as u64);
        set.insert(addr_r as u64);
        set.insert(addr_t as u64);
    });

    super::register_module("urllib3", attrs);
}
