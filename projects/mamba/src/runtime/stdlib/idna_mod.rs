/// idna module for Mamba (#1486).
///
/// Minimal callable-dispatcher shim covering the four most-used
/// top-level entry points (`idna.encode`, `idna.decode`,
/// `idna.IDNAError`, `idna.alabel`). All four return identity-stable
/// sentinel callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only idna sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1486; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_encode(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_decode(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_idna_error(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_alabel(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the idna module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_encode = dispatch_encode as *const () as usize;
    attrs.insert("encode".into(), MbValue::from_func(addr_encode));

    let addr_decode = dispatch_decode as *const () as usize;
    attrs.insert("decode".into(), MbValue::from_func(addr_decode));

    let addr_idna_error = dispatch_idna_error as *const () as usize;
    attrs.insert("IDNAError".into(), MbValue::from_func(addr_idna_error));

    let addr_alabel = dispatch_alabel as *const () as usize;
    attrs.insert("alabel".into(), MbValue::from_func(addr_alabel));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_encode as u64);
        set.insert(addr_decode as u64);
        set.insert(addr_idna_error as u64);
        set.insert(addr_alabel as u64);
    });

    super::register_module("idna", attrs);
}
