use super::super::rc::MbObject;
use super::super::value::MbValue;
/// jsonschema module for Mamba (#1497).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `jsonschema` entry points (`validate`, `Draft7Validator`,
/// `SchemaError`, `ValidationError`). All four return identity-stable
/// sentinel callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only jsonschema sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1497; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_validate(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_draft7_validator(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_schema_error(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_validation_error(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the jsonschema module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_validate = dispatch_validate as *const () as usize;
    attrs.insert("validate".into(), MbValue::from_func(addr_validate));

    let addr_draft7 = dispatch_draft7_validator as *const () as usize;
    attrs.insert("Draft7Validator".into(), MbValue::from_func(addr_draft7));

    let addr_schema_error = dispatch_schema_error as *const () as usize;
    attrs.insert("SchemaError".into(), MbValue::from_func(addr_schema_error));

    let addr_ve = dispatch_validation_error as *const () as usize;
    attrs.insert("ValidationError".into(), MbValue::from_func(addr_ve));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_validate as u64);
        set.insert(addr_draft7 as u64);
        set.insert(addr_schema_error as u64);
        set.insert(addr_ve as u64);
    });

    super::register_module("jsonschema", attrs);
}
