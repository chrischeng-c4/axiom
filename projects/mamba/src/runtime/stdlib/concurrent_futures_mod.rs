use super::super::rc::MbObject;
use super::super::value::MbValue;
/// concurrent.futures module for Mamba (#1261).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `concurrent.futures` entry points (`ThreadPoolExecutor`,
/// `ProcessPoolExecutor`, `Future`, `as_completed`). All four
/// return identity-stable sentinel callables; their job here is to
/// short-circuit CPython's module-dict probe chain for read-only
/// concurrent.futures sentinels.
///
/// Registered under the dotted name `concurrent.futures` (same
/// pattern as `http.cookies` / `urllib.error` / `http.cookiejar`).
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1261; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the
/// stub-only conversion long-tail has closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_thread_pool_executor(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_process_pool_executor(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_future(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_as_completed(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the concurrent.futures module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_tpe = dispatch_thread_pool_executor as *const () as usize;
    attrs.insert("ThreadPoolExecutor".into(), MbValue::from_func(addr_tpe));

    let addr_ppe = dispatch_process_pool_executor as *const () as usize;
    attrs.insert("ProcessPoolExecutor".into(), MbValue::from_func(addr_ppe));

    let addr_fut = dispatch_future as *const () as usize;
    attrs.insert("Future".into(), MbValue::from_func(addr_fut));

    let addr_ac = dispatch_as_completed as *const () as usize;
    attrs.insert("as_completed".into(), MbValue::from_func(addr_ac));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_tpe as u64);
        set.insert(addr_ppe as u64);
        set.insert(addr_fut as u64);
        set.insert(addr_ac as u64);
    });

    super::register_module("concurrent.futures", attrs);
}
