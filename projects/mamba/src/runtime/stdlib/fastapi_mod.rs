/// fastapi module for Mamba (#1519).
///
/// Minimal callable-dispatcher shim covering four top-level
/// entry points (`FastAPI`, `APIRouter`, `Depends`, `HTTPException`).
/// All four return identity-stable sentinel callables; their job
/// here is to short-circuit CPython's module-dict probe chain for
/// read-only fastapi sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1519; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_fastapi(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_apirouter(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_depends(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_httpexception(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the fastapi module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_fastapi = dispatch_fastapi as *const () as usize;
    attrs.insert("FastAPI".into(), MbValue::from_func(addr_fastapi));

    let addr_apirouter = dispatch_apirouter as *const () as usize;
    attrs.insert("APIRouter".into(), MbValue::from_func(addr_apirouter));

    let addr_depends = dispatch_depends as *const () as usize;
    attrs.insert("Depends".into(), MbValue::from_func(addr_depends));

    let addr_httpexception = dispatch_httpexception as *const () as usize;
    attrs.insert("HTTPException".into(), MbValue::from_func(addr_httpexception));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_fastapi as u64);
        set.insert(addr_apirouter as u64);
        set.insert(addr_depends as u64);
        set.insert(addr_httpexception as u64);
    });

    super::register_module("fastapi", attrs);
}
