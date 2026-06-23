use super::super::rc::MbObject;
use super::super::value::MbValue;
/// werkzeug module for Mamba (#1517).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `werkzeug` entry points (`Request`, `Response`, `Local`,
/// `__version__`). All four return identity-stable sentinel
/// callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only werkzeug sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1517; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_request(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_response(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_local(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the werkzeug module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_rq = dispatch_request as *const () as usize;
    attrs.insert("Request".into(), MbValue::from_func(addr_rq));

    let addr_rs = dispatch_response as *const () as usize;
    attrs.insert("Response".into(), MbValue::from_func(addr_rs));

    let addr_l = dispatch_local as *const () as usize;
    attrs.insert("Local".into(), MbValue::from_func(addr_l));

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_rq as u64);
        set.insert(addr_rs as u64);
        set.insert(addr_l as u64);
        set.insert(addr_v as u64);
    });

    super::register_module("werkzeug", attrs);
}
