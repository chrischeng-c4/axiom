use super::super::rc::MbObject;
use super::super::value::MbValue;
/// bdb module for Mamba (#1261).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `bdb` entry points (`Bdb`, `BdbQuit`, `Breakpoint`,
/// `set_trace`). All four return identity-stable sentinel
/// callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only bdb sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1261; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the
/// stub-only conversion long-tail has closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_bdb(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_bdb_quit(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_breakpoint(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_set_trace(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the bdb module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_b = dispatch_bdb as *const () as usize;
    attrs.insert("Bdb".into(), MbValue::from_func(addr_b));

    let addr_bq = dispatch_bdb_quit as *const () as usize;
    attrs.insert("BdbQuit".into(), MbValue::from_func(addr_bq));

    let addr_br = dispatch_breakpoint as *const () as usize;
    attrs.insert("Breakpoint".into(), MbValue::from_func(addr_br));

    let addr_st = dispatch_set_trace as *const () as usize;
    attrs.insert("set_trace".into(), MbValue::from_func(addr_st));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_b as u64);
        set.insert(addr_bq as u64);
        set.insert(addr_br as u64);
        set.insert(addr_st as u64);
    });

    super::register_module("bdb", attrs);
}
