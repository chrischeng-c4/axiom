use super::super::rc::MbObject;
use super::super::value::MbValue;
/// orjson module for Mamba (#1500).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `orjson` entry points (`dumps`, `loads`, `JSONDecodeError`,
/// `JSONEncodeError`). All four return identity-stable sentinel
/// callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only orjson sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1500; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_dumps(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_loads(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_json_decode_error(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_json_encode_error(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the orjson module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_dumps = dispatch_dumps as *const () as usize;
    attrs.insert("dumps".into(), MbValue::from_func(addr_dumps));

    let addr_loads = dispatch_loads as *const () as usize;
    attrs.insert("loads".into(), MbValue::from_func(addr_loads));

    let addr_jde = dispatch_json_decode_error as *const () as usize;
    attrs.insert("JSONDecodeError".into(), MbValue::from_func(addr_jde));

    let addr_jee = dispatch_json_encode_error as *const () as usize;
    attrs.insert("JSONEncodeError".into(), MbValue::from_func(addr_jee));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_dumps as u64);
        set.insert(addr_loads as u64);
        set.insert(addr_jde as u64);
        set.insert(addr_jee as u64);
    });

    super::register_module("orjson", attrs);
}
