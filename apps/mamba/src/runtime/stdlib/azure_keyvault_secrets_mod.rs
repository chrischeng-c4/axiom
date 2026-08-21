use super::super::rc::MbObject;
use super::super::value::MbValue;
/// azure-keyvault-secrets module for Mamba (#1508).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `azure.keyvault.secrets` entry points (`SecretClient`,
/// `KeyVaultSecret`, `SecretProperties`, `__version__`). All four
/// return identity-stable sentinel callables; their job here is to
/// short-circuit CPython's module-dict probe chain for read-only
/// azure-keyvault-secrets sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1508; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_secret_client(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_keyvault_secret(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_secret_properties(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the azure.keyvault.secrets module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_sc = dispatch_secret_client as *const () as usize;
    attrs.insert("SecretClient".into(), MbValue::from_func(addr_sc));

    let addr_ks = dispatch_keyvault_secret as *const () as usize;
    attrs.insert("KeyVaultSecret".into(), MbValue::from_func(addr_ks));

    let addr_sp = dispatch_secret_properties as *const () as usize;
    attrs.insert("SecretProperties".into(), MbValue::from_func(addr_sp));

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_sc as u64);
        set.insert(addr_ks as u64);
        set.insert(addr_sp as u64);
        set.insert(addr_v as u64);
    });

    super::register_module("azure.keyvault.secrets", attrs);
}
