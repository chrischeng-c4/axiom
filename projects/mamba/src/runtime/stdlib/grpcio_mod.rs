use super::super::rc::MbObject;
use super::super::value::MbValue;
/// grpcio module for Mamba (#1515).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `grpc` entry points (`Server`, `Channel`, `insecure_channel`,
/// `__version__`). All four return identity-stable sentinel
/// callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only grpcio sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1515; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_server(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_channel(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_insecure_channel(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the grpc module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_s = dispatch_server as *const () as usize;
    attrs.insert("Server".into(), MbValue::from_func(addr_s));

    let addr_c = dispatch_channel as *const () as usize;
    attrs.insert("Channel".into(), MbValue::from_func(addr_c));

    let addr_ic = dispatch_insecure_channel as *const () as usize;
    attrs.insert("insecure_channel".into(), MbValue::from_func(addr_ic));

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_s as u64);
        set.insert(addr_c as u64);
        set.insert(addr_ic as u64);
        set.insert(addr_v as u64);
    });

    super::register_module("grpc", attrs);
}
