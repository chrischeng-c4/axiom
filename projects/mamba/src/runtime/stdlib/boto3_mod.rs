/// boto3 module for Mamba (#1501).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `boto3` entry points (`client`, `resource`, `Session`,
/// `setup_default_session`). All four return identity-stable
/// sentinel callables; their job here is to short-circuit
/// CPython's module-dict probe chain for read-only boto3
/// sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1501; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_client(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_resource(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_session(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_setup_default_session(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the boto3 module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_client = dispatch_client as *const () as usize;
    attrs.insert("client".into(), MbValue::from_func(addr_client));

    let addr_resource = dispatch_resource as *const () as usize;
    attrs.insert("resource".into(), MbValue::from_func(addr_resource));

    let addr_session = dispatch_session as *const () as usize;
    attrs.insert("Session".into(), MbValue::from_func(addr_session));

    let addr_setup = dispatch_setup_default_session as *const () as usize;
    attrs.insert("setup_default_session".into(), MbValue::from_func(addr_setup));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_client as u64);
        set.insert(addr_resource as u64);
        set.insert(addr_session as u64);
        set.insert(addr_setup as u64);
    });

    super::register_module("boto3", attrs);
}
