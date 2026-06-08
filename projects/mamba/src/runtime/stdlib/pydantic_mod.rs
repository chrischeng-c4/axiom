use super::super::rc::MbObject;
use super::super::value::MbValue;
/// pydantic module for Mamba (#1495).
///
/// Minimal callable-dispatcher shim covering the four most-used
/// top-level entry points (`pydantic.BaseModel`, `pydantic.Field`,
/// `pydantic.ValidationError`, `pydantic.TypeAdapter`). All four
/// return identity-stable sentinel callables; their job here is to
/// short-circuit CPython's module-dict probe chain for read-only
/// pydantic sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1495; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_base_model(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_field(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_validation_error(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_type_adapter(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the pydantic module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_base_model = dispatch_base_model as *const () as usize;
    attrs.insert("BaseModel".into(), MbValue::from_func(addr_base_model));

    let addr_field = dispatch_field as *const () as usize;
    attrs.insert("Field".into(), MbValue::from_func(addr_field));

    let addr_validation_error = dispatch_validation_error as *const () as usize;
    attrs.insert(
        "ValidationError".into(),
        MbValue::from_func(addr_validation_error),
    );

    let addr_type_adapter = dispatch_type_adapter as *const () as usize;
    attrs.insert("TypeAdapter".into(), MbValue::from_func(addr_type_adapter));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_base_model as u64);
        set.insert(addr_field as u64);
        set.insert(addr_validation_error as u64);
        set.insert(addr_type_adapter as u64);
    });

    super::register_module("pydantic", attrs);
}
