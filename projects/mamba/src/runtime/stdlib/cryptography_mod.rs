use super::super::rc::MbObject;
use super::super::value::MbValue;
/// cryptography module for Mamba (#1491).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `cryptography` entry points (`Fernet`, `x509`, `hazmat`,
/// `exceptions`). All four return identity-stable sentinel
/// callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only cryptography sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1491; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_fernet(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_x509(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_hazmat(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_exceptions(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the cryptography module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_fernet = dispatch_fernet as *const () as usize;
    attrs.insert("Fernet".into(), MbValue::from_func(addr_fernet));

    let addr_x509 = dispatch_x509 as *const () as usize;
    attrs.insert("x509".into(), MbValue::from_func(addr_x509));

    let addr_hazmat = dispatch_hazmat as *const () as usize;
    attrs.insert("hazmat".into(), MbValue::from_func(addr_hazmat));

    let addr_exceptions = dispatch_exceptions as *const () as usize;
    attrs.insert("exceptions".into(), MbValue::from_func(addr_exceptions));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_fernet as u64);
        set.insert(addr_x509 as u64);
        set.insert(addr_hazmat as u64);
        set.insert(addr_exceptions as u64);
    });

    super::register_module("cryptography", attrs);
}
