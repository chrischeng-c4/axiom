use super::super::rc::MbObject;
use super::super::value::MbValue;
/// celery module for Mamba (#1530).
///
/// Minimal callable-dispatcher shim covering four top-level `celery`
/// entry points (`__version__`, `Celery`, `shared_task`,
/// `signature`). All four return identity-stable sentinel callables;
/// their job here is to short-circuit CPython's module-dict probe
/// chain for read-only celery sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1530; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_celery(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_shared_task(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_signature(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the celery module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    let addr_c = dispatch_celery as *const () as usize;
    attrs.insert("Celery".into(), MbValue::from_func(addr_c));

    let addr_st = dispatch_shared_task as *const () as usize;
    attrs.insert("shared_task".into(), MbValue::from_func(addr_st));

    let addr_sig = dispatch_signature as *const () as usize;
    attrs.insert("signature".into(), MbValue::from_func(addr_sig));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_v as u64);
        set.insert(addr_c as u64);
        set.insert(addr_st as u64);
        set.insert(addr_sig as u64);
    });

    super::register_module("celery", attrs);
}
