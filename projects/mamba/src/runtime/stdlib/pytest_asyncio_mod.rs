use super::super::rc::MbObject;
use super::super::value::MbValue;
/// pytest_asyncio module for Mamba (#1526).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `pytest_asyncio` entry points (`__version__`, `fixture`,
/// `is_async_test`, `Mode`). All four return identity-stable
/// sentinel callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only pytest_asyncio sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1526; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_fixture(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_is_async_test(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_mode(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the pytest_asyncio module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    let addr_f = dispatch_fixture as *const () as usize;
    attrs.insert("fixture".into(), MbValue::from_func(addr_f));

    let addr_i = dispatch_is_async_test as *const () as usize;
    attrs.insert("is_async_test".into(), MbValue::from_func(addr_i));

    let addr_m = dispatch_mode as *const () as usize;
    attrs.insert("Mode".into(), MbValue::from_func(addr_m));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_v as u64);
        set.insert(addr_f as u64);
        set.insert(addr_i as u64);
        set.insert(addr_m as u64);
    });

    super::register_module("pytest_asyncio", attrs);
}
