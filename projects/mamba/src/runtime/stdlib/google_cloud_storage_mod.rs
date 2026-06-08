use super::super::rc::MbObject;
use super::super::value::MbValue;
/// google-cloud-storage module for Mamba (#1510).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `google.cloud.storage` entry points (`Client`, `Bucket`, `Blob`,
/// `__version__`). All four return identity-stable sentinel
/// callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only google-cloud-storage
/// sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1510; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_client(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_bucket(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_blob(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the google.cloud.storage module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_c = dispatch_client as *const () as usize;
    attrs.insert("Client".into(), MbValue::from_func(addr_c));

    let addr_bk = dispatch_bucket as *const () as usize;
    attrs.insert("Bucket".into(), MbValue::from_func(addr_bk));

    let addr_b = dispatch_blob as *const () as usize;
    attrs.insert("Blob".into(), MbValue::from_func(addr_b));

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_c as u64);
        set.insert(addr_bk as u64);
        set.insert(addr_b as u64);
        set.insert(addr_v as u64);
    });

    super::register_module("google.cloud.storage", attrs);
}
