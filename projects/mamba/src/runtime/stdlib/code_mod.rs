use super::super::rc::MbObject;
use super::super::value::MbValue;
/// code module for Mamba (#1261).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `code` entry points (`InteractiveInterpreter`,
/// `InteractiveConsole`, `interact`, `compile_command`). All four
/// return identity-stable sentinel callables; their job here is to
/// short-circuit CPython's module-dict probe chain for read-only
/// code-module sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1261; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the
/// stub-only conversion long-tail has closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_interactive_interpreter(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_interactive_console(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_interact(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_compile_command(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the code module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_ii = dispatch_interactive_interpreter as *const () as usize;
    attrs.insert("InteractiveInterpreter".into(), MbValue::from_func(addr_ii));

    let addr_ic = dispatch_interactive_console as *const () as usize;
    attrs.insert("InteractiveConsole".into(), MbValue::from_func(addr_ic));

    let addr_i = dispatch_interact as *const () as usize;
    attrs.insert("interact".into(), MbValue::from_func(addr_i));

    let addr_cc = dispatch_compile_command as *const () as usize;
    attrs.insert("compile_command".into(), MbValue::from_func(addr_cc));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_ii as u64);
        set.insert(addr_ic as u64);
        set.insert(addr_i as u64);
        set.insert(addr_cc as u64);
    });

    super::register_module("code", attrs);
}
