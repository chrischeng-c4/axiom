use super::super::value::MbValue;
/// copyreg stub module for mamba.
///
/// CPython's copyreg provides functions used by pickle / copy to register
/// constructors and reduce functions. Fixtures only need the symbols to
/// resolve at import time; runtime behaviour is provided as no-op callables
/// so import succeeds and downstream uses degrade rather than fail at the
/// import line.

use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static REDUCE_REGISTRY: RefCell<HashMap<String, MbValue>> =
        RefCell::new(HashMap::new());
}

unsafe extern "C" fn d_noop(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn d_pickle(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs < 2 {
        return MbValue::none();
    }
    let cls = unsafe { *args_ptr };
    let reducer = unsafe { *args_ptr.add(1) };
    let Some(class_name) = super::super::class::resolve_class_name(cls) else {
        return MbValue::none();
    };
    unsafe { super::super::rc::retain_if_ptr(reducer); }
    REDUCE_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();
        if let Some(prev) = registry.insert(class_name, reducer) {
            unsafe { super::super::rc::release_if_ptr(prev); }
        }
    });
    MbValue::none()
}

pub(crate) fn reduce_func_for_class(class_name: &str) -> Option<MbValue> {
    REDUCE_REGISTRY.with(|registry| registry.borrow().get(class_name).copied())
}

pub fn register() {
    let noop = d_noop as usize;
    let pickle = d_pickle as usize;
    let mut attrs = HashMap::new();

    attrs.insert("pickle".to_string(), MbValue::from_func(pickle));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(pickle as u64);
    });

    let names: &[&str] = &[
        "constructor",
        "add_extension",
        "remove_extension",
        "clear_extension_cache",
        "_reconstructor",
        "__newobj__",
        "__newobj_ex__",
        "_slotnames",
        "dispatch_table",
        "_extension_registry",
        "_inverted_registry",
        "_extension_cache",
    ];
    for name in names {
        attrs.insert((*name).to_string(), MbValue::from_func(noop));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(noop as u64);
    });

    super::register_module("copyreg", attrs);
}
