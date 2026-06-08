/// copyreg stub module for mamba.
///
/// CPython's copyreg provides functions used by pickle / copy to register
/// constructors and reduce functions. Fixtures only need the symbols to
/// resolve at import time; runtime behaviour is provided as no-op callables
/// so import succeeds and downstream uses degrade rather than fail at the
/// import line.

use std::collections::HashMap;
use super::super::value::MbValue;

unsafe extern "C" fn d_noop(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

pub fn register() {
    let noop = d_noop as usize;
    let mut attrs = HashMap::new();

    let names: &[&str] = &[
        "pickle",
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
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(noop as u64);
        });
    }

    super::register_module("copyreg", attrs);
}
