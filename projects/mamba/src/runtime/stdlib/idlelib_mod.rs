use super::super::rc::MbObject;
use super::super::value::MbValue;
/// idlelib module for Mamba (mamba-stdlib).
///
/// Stub implementation of Python's `idlelib` package. Registers the
/// `idlelib` module namespace so `import idlelib` succeeds. All
/// functional APIs raise `NotImplementedError` — no Tkinter GUI
/// functionality.
///
/// All callables are registered as proper dispatcher function pointers
/// (not string placeholders), so consumers that invoke them get a
/// runtime `NotImplementedError` rather than a "string is not callable"
/// type error.
use std::collections::HashMap;

macro_rules! dispatch_variadic_stub {
    ($name:ident, $label:literal) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            raise_not_implemented($label)
        }
    };
}

dispatch_variadic_stub!(dispatch_idle, "idle");
dispatch_variadic_stub!(dispatch_run, "run");
dispatch_variadic_stub!(dispatch_idle_test, "idle_test");
dispatch_variadic_stub!(dispatch_pyshell, "PyShell");
dispatch_variadic_stub!(dispatch_config, "config");
dispatch_variadic_stub!(dispatch_colorizer, "colorizer");
dispatch_variadic_stub!(dispatch_autocomplete, "autocomplete");
dispatch_variadic_stub!(dispatch_calltip, "calltip");
dispatch_variadic_stub!(dispatch_debugger, "debugger");
dispatch_variadic_stub!(dispatch_editor, "editor");
dispatch_variadic_stub!(dispatch_filelist, "filelist");
dispatch_variadic_stub!(dispatch_outwin, "outwin");
dispatch_variadic_stub!(dispatch_rpc, "rpc");

pub fn register() {
    let mut attrs = HashMap::new();

    // Standard module attributes
    attrs.insert(
        "__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str("idlelib".to_string())),
    );
    attrs.insert(
        "__file__".to_string(),
        MbValue::from_ptr(MbObject::new_str("idlelib/__init__.py".to_string())),
    );
    attrs.insert(
        "__package__".to_string(),
        MbValue::from_ptr(MbObject::new_str("idlelib".to_string())),
    );
    attrs.insert(
        "__path__".to_string(),
        MbValue::from_ptr(MbObject::new_list(vec![])),
    );

    // Stub submodule/function attributes — registered as callable dispatchers.
    let dispatchers: Vec<(&str, usize)> = vec![
        ("idle", dispatch_idle as usize),
        ("run", dispatch_run as usize),
        ("idle_test", dispatch_idle_test as usize),
        ("PyShell", dispatch_pyshell as usize),
        ("config", dispatch_config as usize),
        ("colorizer", dispatch_colorizer as usize),
        ("autocomplete", dispatch_autocomplete as usize),
        ("calltip", dispatch_calltip as usize),
        ("debugger", dispatch_debugger as usize),
        ("editor", dispatch_editor as usize),
        ("filelist", dispatch_filelist as usize),
        ("outwin", dispatch_outwin as usize),
        ("rpc", dispatch_rpc as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    super::register_module("idlelib", attrs);
}

// ── Helper ──

fn raise_not_implemented(name: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("NotImplementedError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "idlelib.{name} is not implemented in Mamba"
        ))),
    );
    MbValue::none()
}

// ── Stub functions — each raises NotImplementedError ──
//
// These wrappers keep the historical pub-fn surface (used by direct
// Rust callers/tests) while the module-level attribute is a dispatcher.

pub fn mb_idlelib_idle() -> MbValue {
    raise_not_implemented("idle")
}
pub fn mb_idlelib_run() -> MbValue {
    raise_not_implemented("run")
}
pub fn mb_idlelib_idle_test() -> MbValue {
    raise_not_implemented("idle_test")
}
pub fn mb_idlelib_pyshell() -> MbValue {
    raise_not_implemented("PyShell")
}
pub fn mb_idlelib_config() -> MbValue {
    raise_not_implemented("config")
}
pub fn mb_idlelib_colorizer() -> MbValue {
    raise_not_implemented("colorizer")
}
pub fn mb_idlelib_autocomplete() -> MbValue {
    raise_not_implemented("autocomplete")
}
pub fn mb_idlelib_calltip() -> MbValue {
    raise_not_implemented("calltip")
}
pub fn mb_idlelib_debugger() -> MbValue {
    raise_not_implemented("debugger")
}
pub fn mb_idlelib_editor() -> MbValue {
    raise_not_implemented("editor")
}
pub fn mb_idlelib_filelist() -> MbValue {
    raise_not_implemented("filelist")
}
pub fn mb_idlelib_outwin() -> MbValue {
    raise_not_implemented("outwin")
}
pub fn mb_idlelib_rpc() -> MbValue {
    raise_not_implemented("rpc")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::exception;
    use crate::runtime::rc::ObjData;

    /// Helper: extract a string from an MbValue that should be a Str ptr.
    fn extract_str(v: MbValue) -> Option<String> {
        unsafe {
            if let Some(ptr) = v.as_ptr() {
                if let ObjData::Str(ref s) = (*ptr).data {
                    return Some(s.clone());
                }
            }
        }
        None
    }

    #[test]
    fn test_register() {
        register();
        // After registration, importing "idlelib" should yield a module with __name__ = "idlelib"
        let module_val = crate::runtime::module::mb_module_getattr(
            MbValue::from_ptr(MbObject::new_str("idlelib".to_string())),
            MbValue::from_ptr(MbObject::new_str("__name__".to_string())),
        );
        assert_eq!(extract_str(module_val), Some("idlelib".to_string()));

        // Verify __package__ attribute
        let pkg = crate::runtime::module::mb_module_getattr(
            MbValue::from_ptr(MbObject::new_str("idlelib".to_string())),
            MbValue::from_ptr(MbObject::new_str("__package__".to_string())),
        );
        assert_eq!(extract_str(pkg), Some("idlelib".to_string()));

        // Stub callable attributes are now registered as function pointers,
        // not string placeholders. Verify `idle` is a callable (is_func) —
        // the historical bug stored the function name as a string here.
        let idle_sym = crate::runtime::module::mb_module_getattr(
            MbValue::from_ptr(MbObject::new_str("idlelib".to_string())),
            MbValue::from_ptr(MbObject::new_str("idle".to_string())),
        );
        assert!(
            idle_sym.as_func().is_some(),
            "idle should be a callable dispatcher, not a string"
        );

        let pyshell_sym = crate::runtime::module::mb_module_getattr(
            MbValue::from_ptr(MbObject::new_str("idlelib".to_string())),
            MbValue::from_ptr(MbObject::new_str("PyShell".to_string())),
        );
        assert!(
            pyshell_sym.as_func().is_some(),
            "PyShell should be a callable dispatcher"
        );
    }

    #[test]
    fn test_stub_raises() {
        // Clear any pending exception state first
        exception::mb_clear_exception();

        // Call a stub — should set NotImplementedError, not panic
        let result = mb_idlelib_idle();
        assert!(result.is_none(), "stub should return MbValue::none()");
        assert_eq!(
            exception::mb_has_exception().as_bool(),
            Some(true),
            "stub should set pending exception"
        );

        // Catch and verify the exception type
        let exc = exception::mb_catch_exception();
        assert!(!exc.is_none(), "should have caught an exception");

        // Verify exception cleared
        assert_eq!(exception::mb_has_exception().as_bool(), Some(false));

        // Test another stub to ensure pattern is consistent
        let result2 = mb_idlelib_pyshell();
        assert!(result2.is_none());
        assert_eq!(exception::mb_has_exception().as_bool(), Some(true));

        // Clean up
        let _ = exception::mb_catch_exception();
    }

    #[test]
    fn test_dispatcher_raises_not_implemented() {
        // Calling a dispatcher directly via its extern "C" entry should
        // surface the same NotImplementedError as the pub-fn wrapper.
        exception::mb_clear_exception();
        let result = unsafe { dispatch_idle(std::ptr::null(), 0) };
        assert!(result.is_none());
        assert_eq!(
            exception::mb_has_exception().as_bool(),
            Some(true),
            "dispatcher should set pending NotImplementedError"
        );
        let _ = exception::mb_catch_exception();
    }
}
