use super::super::rc::MbObject;
use super::super::value::MbValue;
/// msgpack module for Mamba (#1499).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `msgpack` entry points (`pack`, `unpack`, `Packer`, `Unpacker`).
/// All four return identity-stable sentinel callables; their job
/// here is to short-circuit CPython's module-dict probe chain for
/// read-only msgpack sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1499; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_pack(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_unpack(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_packer(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_unpacker(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the msgpack module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_pack = dispatch_pack as *const () as usize;
    attrs.insert("pack".into(), MbValue::from_func(addr_pack));

    let addr_unpack = dispatch_unpack as *const () as usize;
    attrs.insert("unpack".into(), MbValue::from_func(addr_unpack));

    let addr_packer = dispatch_packer as *const () as usize;
    attrs.insert("Packer".into(), MbValue::from_func(addr_packer));

    let addr_unpacker = dispatch_unpacker as *const () as usize;
    attrs.insert("Unpacker".into(), MbValue::from_func(addr_unpacker));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_pack as u64);
        set.insert(addr_unpack as u64);
        set.insert(addr_packer as u64);
        set.insert(addr_unpacker as u64);
    });

    super::register_module("msgpack", attrs);
}
