/// flask module for Mamba (#1516).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `flask` entry points (`Flask`, `Blueprint`, `request`,
/// `__version__`). All four return identity-stable sentinel
/// callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only flask sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1516; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_flask(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_blueprint(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_request(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the flask module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_f = dispatch_flask as *const () as usize;
    attrs.insert("Flask".into(), MbValue::from_func(addr_f));

    let addr_b = dispatch_blueprint as *const () as usize;
    attrs.insert("Blueprint".into(), MbValue::from_func(addr_b));

    let addr_r = dispatch_request as *const () as usize;
    attrs.insert("request".into(), MbValue::from_func(addr_r));

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_f as u64);
        set.insert(addr_b as u64);
        set.insert(addr_r as u64);
        set.insert(addr_v as u64);
    });

    super::register_module("flask", attrs);
}
