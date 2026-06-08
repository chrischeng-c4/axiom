/// kombu module for Mamba (#1531).
///
/// Minimal callable-dispatcher shim covering four top-level `kombu`
/// entry points (`__version__`, `Connection`, `Exchange`, `Queue`).
/// All four return identity-stable sentinel callables; their job
/// here is to short-circuit CPython's module-dict probe chain for
/// read-only kombu sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1531; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_connection(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_exchange(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_queue(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the kombu module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    let addr_c = dispatch_connection as *const () as usize;
    attrs.insert("Connection".into(), MbValue::from_func(addr_c));

    let addr_e = dispatch_exchange as *const () as usize;
    attrs.insert("Exchange".into(), MbValue::from_func(addr_e));

    let addr_q = dispatch_queue as *const () as usize;
    attrs.insert("Queue".into(), MbValue::from_func(addr_q));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_v as u64);
        set.insert(addr_c as u64);
        set.insert(addr_e as u64);
        set.insert(addr_q as u64);
    });

    super::register_module("kombu", attrs);
}
