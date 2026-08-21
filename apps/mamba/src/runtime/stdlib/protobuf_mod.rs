use super::super::rc::MbObject;
use super::super::value::MbValue;
/// google.protobuf module for Mamba (#1513).
///
/// Minimal callable-dispatcher shim covering four top-level
/// submodule entry points (`google.protobuf.message`,
/// `google.protobuf.descriptor`, `google.protobuf.json_format`,
/// `google.protobuf.text_format`). All four return identity-stable
/// sentinel callables; their job here is to short-circuit
/// CPython's module-dict probe chain for read-only protobuf
/// sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1513; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_message(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_descriptor(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_json_format(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_text_format(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the google.protobuf module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_message = dispatch_message as *const () as usize;
    attrs.insert("message".into(), MbValue::from_func(addr_message));

    let addr_descriptor = dispatch_descriptor as *const () as usize;
    attrs.insert("descriptor".into(), MbValue::from_func(addr_descriptor));

    let addr_json_format = dispatch_json_format as *const () as usize;
    attrs.insert("json_format".into(), MbValue::from_func(addr_json_format));

    let addr_text_format = dispatch_text_format as *const () as usize;
    attrs.insert("text_format".into(), MbValue::from_func(addr_text_format));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_message as u64);
        set.insert(addr_descriptor as u64);
        set.insert(addr_json_format as u64);
        set.insert(addr_text_format as u64);
    });

    super::register_module("google.protobuf", attrs);
}
