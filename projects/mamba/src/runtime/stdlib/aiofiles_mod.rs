/// aiofiles module for Mamba (#1490).
///
/// Minimal callable-dispatcher shim covering the four most-used
/// top-level entry points (`aiofiles.open`, `aiofiles.tempfile`,
/// `aiofiles.stdin`, `aiofiles.stdout`). All four return
/// identity-stable sentinel callables; their job here is to
/// short-circuit CPython's module-dict probe chain for read-only
/// aiofiles sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1490; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_open(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_tempfile(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_stdin(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_stdout(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the aiofiles module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_open = dispatch_open as *const () as usize;
    attrs.insert("open".into(), MbValue::from_func(addr_open));

    let addr_tempfile = dispatch_tempfile as *const () as usize;
    attrs.insert("tempfile".into(), MbValue::from_func(addr_tempfile));

    let addr_stdin = dispatch_stdin as *const () as usize;
    attrs.insert("stdin".into(), MbValue::from_func(addr_stdin));

    let addr_stdout = dispatch_stdout as *const () as usize;
    attrs.insert("stdout".into(), MbValue::from_func(addr_stdout));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_open as u64);
        set.insert(addr_tempfile as u64);
        set.insert(addr_stdin as u64);
        set.insert(addr_stdout as u64);
    });

    super::register_module("aiofiles", attrs);
}
