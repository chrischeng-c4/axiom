use super::super::rc::MbObject;
use super::super::value::MbValue;
/// marshmallow module for Mamba (#1498).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `marshmallow` entry points (`Schema`, `fields`, `validate`,
/// `ValidationError`). All four return identity-stable sentinel
/// callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only marshmallow sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1498; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_schema(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_fields(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_validate(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_validation_error(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the marshmallow module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_schema = dispatch_schema as *const () as usize;
    attrs.insert("Schema".into(), MbValue::from_func(addr_schema));

    let addr_fields = dispatch_fields as *const () as usize;
    attrs.insert("fields".into(), MbValue::from_func(addr_fields));

    let addr_validate = dispatch_validate as *const () as usize;
    attrs.insert("validate".into(), MbValue::from_func(addr_validate));

    let addr_ve = dispatch_validation_error as *const () as usize;
    attrs.insert("ValidationError".into(), MbValue::from_func(addr_ve));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_schema as u64);
        set.insert(addr_fields as u64);
        set.insert(addr_validate as u64);
        set.insert(addr_ve as u64);
    });

    super::register_module("marshmallow", attrs);
}
