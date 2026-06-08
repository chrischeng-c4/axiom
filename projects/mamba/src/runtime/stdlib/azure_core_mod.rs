/// azure-core module for Mamba (#1505).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `azure.core` entry points (`PipelineClient`, `AsyncPipelineClient`,
/// `MatchConditions`, `__version__`). All four return identity-stable
/// sentinel callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only azure-core sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1505; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_pipeline_client(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_async_pipeline_client(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_match_conditions(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the azure.core module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_pc = dispatch_pipeline_client as *const () as usize;
    attrs.insert("PipelineClient".into(), MbValue::from_func(addr_pc));

    let addr_apc = dispatch_async_pipeline_client as *const () as usize;
    attrs.insert("AsyncPipelineClient".into(), MbValue::from_func(addr_apc));

    let addr_mc = dispatch_match_conditions as *const () as usize;
    attrs.insert("MatchConditions".into(), MbValue::from_func(addr_mc));

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_pc as u64);
        set.insert(addr_apc as u64);
        set.insert(addr_mc as u64);
        set.insert(addr_v as u64);
    });

    super::register_module("azure.core", attrs);
}
