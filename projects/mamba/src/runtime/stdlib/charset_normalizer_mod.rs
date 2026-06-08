/// charset_normalizer module for Mamba (#1484).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `charset_normalizer` entry points (`from_bytes`, `from_path`,
/// `detect`, `CharsetMatch`). All four return identity-stable
/// sentinel callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only charset_normalizer sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1484; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_from_bytes(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_from_path(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_detect(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_charset_match(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the charset_normalizer module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_from_bytes = dispatch_from_bytes as *const () as usize;
    attrs.insert("from_bytes".into(), MbValue::from_func(addr_from_bytes));

    let addr_from_path = dispatch_from_path as *const () as usize;
    attrs.insert("from_path".into(), MbValue::from_func(addr_from_path));

    let addr_detect = dispatch_detect as *const () as usize;
    attrs.insert("detect".into(), MbValue::from_func(addr_detect));

    let addr_charset_match = dispatch_charset_match as *const () as usize;
    attrs.insert("CharsetMatch".into(), MbValue::from_func(addr_charset_match));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_from_bytes as u64);
        set.insert(addr_from_path as u64);
        set.insert(addr_detect as u64);
        set.insert(addr_charset_match as u64);
    });

    super::register_module("charset_normalizer", attrs);
}
