/// sqlalchemy module for Mamba (#1523).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `sqlalchemy` entry points (`create_engine`, `Column`, `Table`,
/// `__version__`). All four return identity-stable sentinel
/// callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only sqlalchemy sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1523; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_create_engine(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_column(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_table(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the sqlalchemy module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_ce = dispatch_create_engine as *const () as usize;
    attrs.insert("create_engine".into(), MbValue::from_func(addr_ce));

    let addr_c = dispatch_column as *const () as usize;
    attrs.insert("Column".into(), MbValue::from_func(addr_c));

    let addr_t = dispatch_table as *const () as usize;
    attrs.insert("Table".into(), MbValue::from_func(addr_t));

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_ce as u64);
        set.insert(addr_c as u64);
        set.insert(addr_t as u64);
        set.insert(addr_v as u64);
    });

    super::register_module("sqlalchemy", attrs);
}
