/// starlette module for Mamba (#1520).
///
/// Minimal callable-dispatcher shim covering four top-level
/// submodule entry points (`starlette.applications`,
/// `starlette.routing`, `starlette.responses`,
/// `starlette.requests`). All four return identity-stable sentinel
/// callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only starlette sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1520; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_applications(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_routing(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_responses(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_requests(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the starlette module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_applications = dispatch_applications as *const () as usize;
    attrs.insert("applications".into(), MbValue::from_func(addr_applications));

    let addr_routing = dispatch_routing as *const () as usize;
    attrs.insert("routing".into(), MbValue::from_func(addr_routing));

    let addr_responses = dispatch_responses as *const () as usize;
    attrs.insert("responses".into(), MbValue::from_func(addr_responses));

    let addr_requests = dispatch_requests as *const () as usize;
    attrs.insert("requests".into(), MbValue::from_func(addr_requests));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_applications as u64);
        set.insert(addr_routing as u64);
        set.insert(addr_responses as u64);
        set.insert(addr_requests as u64);
    });

    super::register_module("starlette", attrs);
}
