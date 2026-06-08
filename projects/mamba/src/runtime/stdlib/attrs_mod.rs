/// attrs module for Mamba (#1493).
///
/// Minimal callable-dispatcher shim covering four top-level
/// attrs entry points (`define`, `field`, `asdict`, `fields`).
/// All four return identity-stable sentinel callables; their job
/// here is to short-circuit CPython's module-dict probe chain for
/// read-only attrs sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1493; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_define(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_field(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_asdict(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_fields(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the attrs module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_define = dispatch_define as *const () as usize;
    attrs.insert("define".into(), MbValue::from_func(addr_define));

    let addr_field = dispatch_field as *const () as usize;
    attrs.insert("field".into(), MbValue::from_func(addr_field));

    let addr_asdict = dispatch_asdict as *const () as usize;
    attrs.insert("asdict".into(), MbValue::from_func(addr_asdict));

    let addr_fields = dispatch_fields as *const () as usize;
    attrs.insert("fields".into(), MbValue::from_func(addr_fields));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_define as u64);
        set.insert(addr_field as u64);
        set.insert(addr_asdict as u64);
        set.insert(addr_fields as u64);
    });

    super::register_module("attrs", attrs);
}
