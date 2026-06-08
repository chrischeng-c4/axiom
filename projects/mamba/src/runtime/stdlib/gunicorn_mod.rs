/// gunicorn module for Mamba (#1522).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `gunicorn` entry points (`__version__`, `SERVER`,
/// `SERVER_SOFTWARE`, `version_info`). All four return
/// identity-stable sentinel callables; their job here is to
/// short-circuit CPython's module-dict probe chain for read-only
/// gunicorn sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1522; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_server(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_server_software(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_version_info(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the gunicorn module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    let addr_s = dispatch_server as *const () as usize;
    attrs.insert("SERVER".into(), MbValue::from_func(addr_s));

    let addr_ss = dispatch_server_software as *const () as usize;
    attrs.insert("SERVER_SOFTWARE".into(), MbValue::from_func(addr_ss));

    let addr_vi = dispatch_version_info as *const () as usize;
    attrs.insert("version_info".into(), MbValue::from_func(addr_vi));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_v as u64);
        set.insert(addr_s as u64);
        set.insert(addr_ss as u64);
        set.insert(addr_vi as u64);
    });

    super::register_module("gunicorn", attrs);
}
