/// azure-identity module for Mamba (#1506).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `azure.identity` entry points (`DefaultAzureCredential`,
/// `ClientSecretCredential`, `ManagedIdentityCredential`,
/// `__version__`). All four return identity-stable sentinel
/// callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only azure-identity sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1506; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_default_credential(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_client_secret(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_managed_identity(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the azure.identity module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_dac = dispatch_default_credential as *const () as usize;
    attrs.insert("DefaultAzureCredential".into(), MbValue::from_func(addr_dac));

    let addr_csc = dispatch_client_secret as *const () as usize;
    attrs.insert("ClientSecretCredential".into(), MbValue::from_func(addr_csc));

    let addr_mic = dispatch_managed_identity as *const () as usize;
    attrs.insert("ManagedIdentityCredential".into(), MbValue::from_func(addr_mic));

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_dac as u64);
        set.insert(addr_csc as u64);
        set.insert(addr_mic as u64);
        set.insert(addr_v as u64);
    });

    super::register_module("azure.identity", attrs);
}
