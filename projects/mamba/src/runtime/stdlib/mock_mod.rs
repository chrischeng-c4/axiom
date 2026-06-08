use super::super::rc::MbObject;
use super::super::value::MbValue;
/// mock module for Mamba (#1528).
///
/// Minimal callable-dispatcher shim covering four top-level `mock`
/// entry points (`__version__`, `Mock`, `MagicMock`, `patch`). All
/// four return identity-stable sentinel callables; their job here is
/// to short-circuit CPython's module-dict probe chain for read-only
/// mock sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1528; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_mock(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_magicmock(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_patch(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the mock module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    let addr_m = dispatch_mock as *const () as usize;
    attrs.insert("Mock".into(), MbValue::from_func(addr_m));

    let addr_mm = dispatch_magicmock as *const () as usize;
    attrs.insert("MagicMock".into(), MbValue::from_func(addr_mm));

    let addr_p = dispatch_patch as *const () as usize;
    attrs.insert("patch".into(), MbValue::from_func(addr_p));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_v as u64);
        set.insert(addr_m as u64);
        set.insert(addr_mm as u64);
        set.insert(addr_p as u64);
    });

    super::register_module("mock", attrs);
}
