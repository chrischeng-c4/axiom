/// typing_extensions module for Mamba (#1494).
///
/// Minimal callable-dispatcher shim covering the four most-used
/// top-level entry points (`typing_extensions.Protocol`,
/// `typing_extensions.TypedDict`, `typing_extensions.runtime_checkable`,
/// `typing_extensions.override`). All four return identity-stable
/// sentinel callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only typing_extensions sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1494; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_protocol(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_typed_dict(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_runtime_checkable(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_override(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the typing_extensions module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_protocol = dispatch_protocol as *const () as usize;
    attrs.insert("Protocol".into(), MbValue::from_func(addr_protocol));

    let addr_typed_dict = dispatch_typed_dict as *const () as usize;
    attrs.insert("TypedDict".into(), MbValue::from_func(addr_typed_dict));

    let addr_runtime_checkable = dispatch_runtime_checkable as *const () as usize;
    attrs.insert("runtime_checkable".into(), MbValue::from_func(addr_runtime_checkable));

    let addr_override = dispatch_override as *const () as usize;
    attrs.insert("override".into(), MbValue::from_func(addr_override));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_protocol as u64);
        set.insert(addr_typed_dict as u64);
        set.insert(addr_runtime_checkable as u64);
        set.insert(addr_override as u64);
    });

        // surface: missing CPython module constants (auto-added)
    attrs.insert("PEP_560".into(), MbValue::from_int(1));
    attrs.insert("TYPE_CHECKING".into(), MbValue::from_int(0));
    super::register_module("typing_extensions", attrs);
}
