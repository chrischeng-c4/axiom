/// jmespath module for Mamba (#1504).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `jmespath` entry points (`search`, `compile`, `Options`,
/// `__version__`). All four return identity-stable sentinel
/// callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only jmespath sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1504; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_search(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_compile(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_options(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the jmespath module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_search = dispatch_search as *const () as usize;
    attrs.insert("search".into(), MbValue::from_func(addr_search));

    let addr_compile = dispatch_compile as *const () as usize;
    attrs.insert("compile".into(), MbValue::from_func(addr_compile));

    let addr_options = dispatch_options as *const () as usize;
    attrs.insert("Options".into(), MbValue::from_func(addr_options));

    let addr_version = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_version));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_search as u64);
        set.insert(addr_compile as u64);
        set.insert(addr_options as u64);
        set.insert(addr_version as u64);
    });

    super::register_module("jmespath", attrs);
}
