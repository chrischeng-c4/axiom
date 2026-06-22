use super::super::rc::MbObject;
use super::super::value::MbValue;
/// requests module for Mamba (#1487).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `requests` entry points (`get`, `post`, `request`, `Session`).
/// All four return identity-stable sentinel callables; their job
/// here is to short-circuit CPython's module-dict probe chain for
/// read-only requests sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1487; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_get(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_post(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_request(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_session(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the requests module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_get = dispatch_get as *const () as usize;
    attrs.insert("get".into(), MbValue::from_func(addr_get));

    let addr_post = dispatch_post as *const () as usize;
    attrs.insert("post".into(), MbValue::from_func(addr_post));

    let addr_request = dispatch_request as *const () as usize;
    attrs.insert("request".into(), MbValue::from_func(addr_request));

    let addr_session = dispatch_session as *const () as usize;
    attrs.insert("Session".into(), MbValue::from_func(addr_session));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_get as u64);
        set.insert(addr_post as u64);
        set.insert(addr_request as u64);
        set.insert(addr_session as u64);
    });

    super::register_module("requests", attrs);
}
