use super::super::rc::MbObject;
use super::super::value::MbValue;
/// multiprocessing module for Mamba (#1476).
///
/// Minimal callable-dispatcher shim covering the four most-used
/// top-level entry points on 3.12 (`multiprocessing.Process`,
/// `multiprocessing.Queue`, `multiprocessing.cpu_count`,
/// `multiprocessing.current_process`). All four return an empty
/// dict sentinel today; their job here is to be identity-stable
/// module-attribute reads so the `multiprocessing` module-attribute
/// resolver short-circuits CPython's module-dict probe chain for
/// read-only sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1476; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the stdlib
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_process(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_queue(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_cpu_count(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_int(1)
}

unsafe extern "C" fn dispatch_current_process(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the multiprocessing module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_process = dispatch_process as *const () as usize;
    attrs.insert("Process".into(), MbValue::from_func(addr_process));

    let addr_queue = dispatch_queue as *const () as usize;
    attrs.insert("Queue".into(), MbValue::from_func(addr_queue));

    let addr_cpu_count = dispatch_cpu_count as *const () as usize;
    attrs.insert("cpu_count".into(), MbValue::from_func(addr_cpu_count));

    let addr_current_process = dispatch_current_process as *const () as usize;
    attrs.insert(
        "current_process".into(),
        MbValue::from_func(addr_current_process),
    );

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_process as u64);
        set.insert(addr_queue as u64);
        set.insert(addr_cpu_count as u64);
        set.insert(addr_current_process as u64);
    });

    super::register_module("multiprocessing", attrs);
}
