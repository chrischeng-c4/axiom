use super::super::rc::MbObject;
use super::super::value::MbValue;
/// s3transfer module for Mamba (#1503).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `s3transfer` entry points (`TransferManager`, `TransferConfig`,
/// `S3Transfer`, `tasks`). All four return identity-stable sentinel
/// callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only s3transfer sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1503; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_transfer_manager(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_transfer_config(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_s3_transfer(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_tasks(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the s3transfer module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_tm = dispatch_transfer_manager as *const () as usize;
    attrs.insert("TransferManager".into(), MbValue::from_func(addr_tm));

    let addr_tc = dispatch_transfer_config as *const () as usize;
    attrs.insert("TransferConfig".into(), MbValue::from_func(addr_tc));

    let addr_st = dispatch_s3_transfer as *const () as usize;
    attrs.insert("S3Transfer".into(), MbValue::from_func(addr_st));

    let addr_tasks = dispatch_tasks as *const () as usize;
    attrs.insert("tasks".into(), MbValue::from_func(addr_tasks));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_tm as u64);
        set.insert(addr_tc as u64);
        set.insert(addr_st as u64);
        set.insert(addr_tasks as u64);
    });

    super::register_module("s3transfer", attrs);
}
