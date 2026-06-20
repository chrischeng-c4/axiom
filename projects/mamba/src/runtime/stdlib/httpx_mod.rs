use super::super::rc::MbObject;
use super::super::value::MbValue;
/// httpx module for Mamba (#1488).
///
/// Minimal callable-dispatcher shim covering the four most-used
/// top-level entry points (`httpx.Client`, `httpx.AsyncClient`,
/// `httpx.Response`, `httpx.Request`). All four return identity-stable
/// sentinel callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only httpx sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1488; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_client(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_async_client(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_response(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_request(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the httpx module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_client = dispatch_client as *const () as usize;
    attrs.insert("Client".into(), MbValue::from_func(addr_client));

    let addr_async_client = dispatch_async_client as *const () as usize;
    attrs.insert("AsyncClient".into(), MbValue::from_func(addr_async_client));

    let addr_response = dispatch_response as *const () as usize;
    attrs.insert("Response".into(), MbValue::from_func(addr_response));

    let addr_request = dispatch_request as *const () as usize;
    attrs.insert("Request".into(), MbValue::from_func(addr_request));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_client as u64);
        set.insert(addr_async_client as u64);
        set.insert(addr_response as u64);
        set.insert(addr_request as u64);
    });

    super::register_module("httpx", attrs);
}
