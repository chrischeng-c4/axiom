use super::super::rc::MbObject;
use super::super::value::MbValue;
/// hypothesis module for Mamba (#1527).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `hypothesis` entry points (`__version__`, `given`, `strategies`,
/// `settings`). All four return identity-stable sentinel callables;
/// their job here is to short-circuit CPython's module-dict probe
/// chain for read-only hypothesis sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1527; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_given(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_strategies(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_settings(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the hypothesis module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    let addr_g = dispatch_given as *const () as usize;
    attrs.insert("given".into(), MbValue::from_func(addr_g));

    let addr_st = dispatch_strategies as *const () as usize;
    attrs.insert("strategies".into(), MbValue::from_func(addr_st));

    let addr_se = dispatch_settings as *const () as usize;
    attrs.insert("settings".into(), MbValue::from_func(addr_se));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_v as u64);
        set.insert(addr_g as u64);
        set.insert(addr_st as u64);
        set.insert(addr_se as u64);
    });

    super::register_module("hypothesis", attrs);
}
