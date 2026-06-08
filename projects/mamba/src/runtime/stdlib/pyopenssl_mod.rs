use super::super::rc::MbObject;
use super::super::value::MbValue;
/// pyOpenSSL module for Mamba (#1492).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `OpenSSL` entry points (`SSL`, `crypto`, `version`, `rand`).
/// All four return identity-stable sentinel callables; their job
/// here is to short-circuit CPython's module-dict probe chain for
/// read-only OpenSSL sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1492; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_ssl(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_crypto(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_rand(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the OpenSSL module (pyOpenSSL).
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_ssl = dispatch_ssl as *const () as usize;
    attrs.insert("SSL".into(), MbValue::from_func(addr_ssl));

    let addr_crypto = dispatch_crypto as *const () as usize;
    attrs.insert("crypto".into(), MbValue::from_func(addr_crypto));

    let addr_version = dispatch_version as *const () as usize;
    attrs.insert("version".into(), MbValue::from_func(addr_version));

    let addr_rand = dispatch_rand as *const () as usize;
    attrs.insert("rand".into(), MbValue::from_func(addr_rand));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_ssl as u64);
        set.insert(addr_crypto as u64);
        set.insert(addr_version as u64);
        set.insert(addr_rand as u64);
    });

    super::register_module("OpenSSL", attrs);
}
