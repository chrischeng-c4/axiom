/// jinja2 module for Mamba (#1518).
///
/// Minimal callable-dispatcher shim covering the four most-used
/// top-level entry points (`jinja2.Environment`, `jinja2.Template`,
/// `jinja2.FileSystemLoader`, `jinja2.select_autoescape`). All four
/// return identity-stable sentinel callables; their job here is to
/// short-circuit CPython's module-dict probe chain for read-only
/// jinja2 sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1518; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_environment(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_template(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_file_system_loader(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_select_autoescape(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the jinja2 module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_environment = dispatch_environment as *const () as usize;
    attrs.insert("Environment".into(), MbValue::from_func(addr_environment));

    let addr_template = dispatch_template as *const () as usize;
    attrs.insert("Template".into(), MbValue::from_func(addr_template));

    let addr_file_system_loader = dispatch_file_system_loader as *const () as usize;
    attrs.insert("FileSystemLoader".into(), MbValue::from_func(addr_file_system_loader));

    let addr_select_autoescape = dispatch_select_autoescape as *const () as usize;
    attrs.insert("select_autoescape".into(), MbValue::from_func(addr_select_autoescape));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_environment as u64);
        set.insert(addr_template as u64);
        set.insert(addr_file_system_loader as u64);
        set.insert(addr_select_autoescape as u64);
    });

    super::register_module("jinja2", attrs);
}
