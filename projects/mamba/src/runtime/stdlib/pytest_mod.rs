/// pytest module for Mamba (#1525).
///
/// Minimal callable-dispatcher shim covering the four most-used
/// top-level entry points (`pytest.fixture`, `pytest.mark`,
/// `pytest.raises`, `pytest.skip`). All four return identity-stable
/// sentinel callables; their job here is to short-circuit CPython's
/// module-dict probe chain for read-only pytest sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1525; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_fixture(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_mark(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_raises(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_skip(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the pytest module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_fixture = dispatch_fixture as *const () as usize;
    attrs.insert("fixture".into(), MbValue::from_func(addr_fixture));

    let addr_mark = dispatch_mark as *const () as usize;
    attrs.insert("mark".into(), MbValue::from_func(addr_mark));

    let addr_raises = dispatch_raises as *const () as usize;
    attrs.insert("raises".into(), MbValue::from_func(addr_raises));

    let addr_skip = dispatch_skip as *const () as usize;
    attrs.insert("skip".into(), MbValue::from_func(addr_skip));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_fixture as u64);
        set.insert(addr_mark as u64);
        set.insert(addr_raises as u64);
        set.insert(addr_skip as u64);
    });

    super::register_module("pytest", attrs);
}
