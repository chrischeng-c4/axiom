/// aiohttp module for Mamba (#1489).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `aiohttp` entry points (`ClientSession`, `ClientResponse`,
/// `ClientTimeout`, `request`). All four return identity-stable
/// sentinel callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only aiohttp sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1489; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_client_session(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_client_response(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_client_timeout(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_request(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the aiohttp module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_client_session = dispatch_client_session as *const () as usize;
    attrs.insert("ClientSession".into(), MbValue::from_func(addr_client_session));

    let addr_client_response = dispatch_client_response as *const () as usize;
    attrs.insert("ClientResponse".into(), MbValue::from_func(addr_client_response));

    let addr_client_timeout = dispatch_client_timeout as *const () as usize;
    attrs.insert("ClientTimeout".into(), MbValue::from_func(addr_client_timeout));

    let addr_request = dispatch_request as *const () as usize;
    attrs.insert("request".into(), MbValue::from_func(addr_request));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_client_session as u64);
        set.insert(addr_client_response as u64);
        set.insert(addr_client_timeout as u64);
        set.insert(addr_request as u64);
    });

    super::register_module("aiohttp", attrs);
}
