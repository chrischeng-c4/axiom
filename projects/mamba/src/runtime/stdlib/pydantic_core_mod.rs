use super::super::rc::MbObject;
use super::super::value::MbValue;
/// pydantic_core module for Mamba (#1496).
///
/// Minimal callable-dispatcher shim covering the four most-used
/// top-level entry points (`pydantic_core.ValidationError`,
/// `pydantic_core.SchemaValidator`, `pydantic_core.SchemaSerializer`,
/// `pydantic_core.Url`). All four return identity-stable sentinel
/// callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only pydantic_core sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1496; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_validation_error(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_schema_validator(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_schema_serializer(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_url(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the pydantic_core module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_validation_error = dispatch_validation_error as *const () as usize;
    attrs.insert(
        "ValidationError".into(),
        MbValue::from_func(addr_validation_error),
    );

    let addr_schema_validator = dispatch_schema_validator as *const () as usize;
    attrs.insert(
        "SchemaValidator".into(),
        MbValue::from_func(addr_schema_validator),
    );

    let addr_schema_serializer = dispatch_schema_serializer as *const () as usize;
    attrs.insert(
        "SchemaSerializer".into(),
        MbValue::from_func(addr_schema_serializer),
    );

    let addr_url = dispatch_url as *const () as usize;
    attrs.insert("Url".into(), MbValue::from_func(addr_url));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_validation_error as u64);
        set.insert(addr_schema_validator as u64);
        set.insert(addr_schema_serializer as u64);
        set.insert(addr_url as u64);
    });

    super::register_module("pydantic_core", attrs);
}
