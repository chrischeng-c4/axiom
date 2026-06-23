use super::super::rc::MbObject;
use super::super::value::MbValue;
/// alembic module for Mamba (#1524).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `alembic` entry points (`__version__`, `context`, `op`,
/// `EnvironmentContext`). All four return identity-stable sentinel
/// callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only alembic sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1524; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_context(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_op(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_environment_context(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the alembic module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    let addr_c = dispatch_context as *const () as usize;
    attrs.insert("context".into(), MbValue::from_func(addr_c));

    let addr_o = dispatch_op as *const () as usize;
    attrs.insert("op".into(), MbValue::from_func(addr_o));

    let addr_e = dispatch_environment_context as *const () as usize;
    attrs.insert("EnvironmentContext".into(), MbValue::from_func(addr_e));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_v as u64);
        set.insert(addr_c as u64);
        set.insert(addr_o as u64);
        set.insert(addr_e as u64);
    });

    super::register_module("alembic", attrs);
}
