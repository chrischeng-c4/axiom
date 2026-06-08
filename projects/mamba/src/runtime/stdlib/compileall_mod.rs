/// compileall module for Mamba (#1261).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `compileall` entry points (`compile_dir`, `compile_file`,
/// `compile_path`, `main`). All four return identity-stable
/// sentinel callables; their job here is to short-circuit
/// CPython's module-dict probe chain for read-only compileall
/// sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1261; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the
/// stub-only conversion long-tail has closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_compile_dir(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_compile_file(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_compile_path(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_main(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the compileall module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_cd = dispatch_compile_dir as *const () as usize;
    attrs.insert("compile_dir".into(), MbValue::from_func(addr_cd));

    let addr_cf = dispatch_compile_file as *const () as usize;
    attrs.insert("compile_file".into(), MbValue::from_func(addr_cf));

    let addr_cp = dispatch_compile_path as *const () as usize;
    attrs.insert("compile_path".into(), MbValue::from_func(addr_cp));

    let addr_m = dispatch_main as *const () as usize;
    attrs.insert("main".into(), MbValue::from_func(addr_m));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_cd as u64);
        set.insert(addr_cf as u64);
        set.insert(addr_cp as u64);
        set.insert(addr_m as u64);
    });

    super::register_module("compileall", attrs);
}
