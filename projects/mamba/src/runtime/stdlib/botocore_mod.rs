use super::super::rc::MbObject;
use super::super::value::MbValue;
/// botocore module for Mamba (#1502).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `botocore` entry points (`session`, `client`, `exceptions`,
/// `errorfactory`). All four return identity-stable sentinel
/// callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only botocore sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1502; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_session(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_client(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_exceptions(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_errorfactory(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the botocore module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_session = dispatch_session as *const () as usize;
    attrs.insert("session".into(), MbValue::from_func(addr_session));

    let addr_client = dispatch_client as *const () as usize;
    attrs.insert("client".into(), MbValue::from_func(addr_client));

    let addr_exceptions = dispatch_exceptions as *const () as usize;
    attrs.insert("exceptions".into(), MbValue::from_func(addr_exceptions));

    let addr_errorfactory = dispatch_errorfactory as *const () as usize;
    attrs.insert("errorfactory".into(), MbValue::from_func(addr_errorfactory));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_session as u64);
        set.insert(addr_client as u64);
        set.insert(addr_exceptions as u64);
        set.insert(addr_errorfactory as u64);
    });

    super::register_module("botocore", attrs);
}
