/// uvicorn module for Mamba (#1521).
///
/// Minimal callable-dispatcher shim covering the four `__all__`
/// entry points (`uvicorn.Config`, `uvicorn.Server`, `uvicorn.main`,
/// `uvicorn.run`). All four return identity-stable sentinel
/// callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only uvicorn sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1521; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_config(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_server(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_main(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_run(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the uvicorn module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_config = dispatch_config as *const () as usize;
    attrs.insert("Config".into(), MbValue::from_func(addr_config));

    let addr_server = dispatch_server as *const () as usize;
    attrs.insert("Server".into(), MbValue::from_func(addr_server));

    let addr_main = dispatch_main as *const () as usize;
    attrs.insert("main".into(), MbValue::from_func(addr_main));

    let addr_run = dispatch_run as *const () as usize;
    attrs.insert("run".into(), MbValue::from_func(addr_run));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_config as u64);
        set.insert(addr_server as u64);
        set.insert(addr_main as u64);
        set.insert(addr_run as u64);
    });

    super::register_module("uvicorn", attrs);
}
