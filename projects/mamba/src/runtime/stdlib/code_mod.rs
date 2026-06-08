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
use rustc_hash::FxHashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

unsafe extern "C" fn dispatch_interactive_interpreter(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_interactive_console(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_interact(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_compile_command(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Shared no-op stub for the bound-method surface of the two `code` class
/// shells (`InteractiveInterpreter.runcode`, `InteractiveConsole.push`, ...).
/// Each method name is a distinct callable field on the type-object shell, so
/// `callable(code.InteractiveConsole.push)` resolves through type-object
/// getattr to this stub.
unsafe extern "C" fn dispatch_code_method_stub(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

/// Register `addr` as a known native function pointer and return it as a
/// callable `MbValue`.
fn surface_func(addr: usize) -> MbValue {
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr as u64);
    });
    MbValue::from_func(addr)
}

/// Build a `code.<Name>` class shell as a type object (`class_name == "type"`,
/// `__name__ = name`) carrying one callable method-stub field per entry in
/// `methods`. Representing the shell as a type object keeps
/// `callable(code.InteractiveInterpreter)` true (the `callable` builtin treats
/// `class_name == "type"` instances as callable) while the method names fall
/// through type-object getattr to ordinary instance-field lookup so
/// `code.InteractiveInterpreter.runcode` etc. return the stub. Mirrors
/// `re_mod::make_re_class_shell`.
fn make_code_class_shell(name: &str, methods: &[&str]) -> MbValue {
    let stub_addr = dispatch_code_method_stub as *const () as usize;
    let mut fields: FxHashMap<String, MbValue> = FxHashMap::default();
    fields.insert("__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str(name.to_string())));
    for m in methods {
        fields.insert((*m).to_string(), surface_func(stub_addr));
    }
    let obj = Box::new(MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "type".to_string(),
            fields: super::super::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Register the code module.
pub fn register() {
    let mut attrs = HashMap::new();

    // `InteractiveInterpreter` / `InteractiveConsole` are exposed as type-object
    // class shells carrying one callable stub per CPython method name, so both
    // `callable(code.InteractiveInterpreter)` and
    // `callable(code.InteractiveInterpreter.runcode)` resolve. CPython's
    // InteractiveConsole subclasses InteractiveInterpreter, so its shell carries
    // the inherited interpreter methods plus the console-only additions.
    let _ = dispatch_interactive_interpreter; // legacy bare-callable stubs retired
    let _ = dispatch_interactive_console;
    attrs.insert("InteractiveInterpreter".into(),
        make_code_class_shell("code.InteractiveInterpreter", &[
            "runsource", "runcode", "showsyntaxerror", "showtraceback", "write",
        ]));
    attrs.insert("InteractiveConsole".into(),
        make_code_class_shell("code.InteractiveConsole", &[
            "runsource", "runcode", "showsyntaxerror", "showtraceback", "write",
            "interact", "push", "resetbuffer", "raw_input",
        ]));

    let addr_i = dispatch_interact as *const () as usize;
    attrs.insert("interact".into(), MbValue::from_func(addr_i));

    let addr_cc = dispatch_compile_command as *const () as usize;
    attrs.insert("compile_command".into(), MbValue::from_func(addr_cc));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_i as u64);
        set.insert(addr_cc as u64);
    });

    super::register_module("code", attrs);
}
