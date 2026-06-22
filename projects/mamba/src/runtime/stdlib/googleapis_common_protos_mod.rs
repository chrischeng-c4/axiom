use super::super::rc::MbObject;
use super::super::value::MbValue;
/// googleapis-common-protos module for Mamba (#1512).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `google.rpc` entry points (`status_pb2`, `code_pb2`,
/// `error_details_pb2`, `context_pb2`). All four return identity-stable
/// sentinel callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only googleapis-common-protos
/// sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1512; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_status_pb2(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_code_pb2(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_error_details_pb2(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_context_pb2(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the google.rpc module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_s = dispatch_status_pb2 as *const () as usize;
    attrs.insert("status_pb2".into(), MbValue::from_func(addr_s));

    let addr_co = dispatch_code_pb2 as *const () as usize;
    attrs.insert("code_pb2".into(), MbValue::from_func(addr_co));

    let addr_e = dispatch_error_details_pb2 as *const () as usize;
    attrs.insert("error_details_pb2".into(), MbValue::from_func(addr_e));

    let addr_ct = dispatch_context_pb2 as *const () as usize;
    attrs.insert("context_pb2".into(), MbValue::from_func(addr_ct));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_s as u64);
        set.insert(addr_co as u64);
        set.insert(addr_e as u64);
        set.insert(addr_ct as u64);
    });

    super::register_module("google.rpc", attrs);
}
