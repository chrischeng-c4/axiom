use super::super::rc::MbObject;
use super::super::value::MbValue;
/// multiprocessing module for Mamba (#1476).
///
/// Minimal callable-dispatcher shim covering the four most-used
/// top-level entry points on 3.12 (`multiprocessing.Process`,
/// `multiprocessing.Queue`, `multiprocessing.cpu_count`,
/// `multiprocessing.current_process`). All four return an empty
/// dict sentinel today; their job here is to be identity-stable
/// module-attribute reads so the `multiprocessing` module-attribute
/// resolver short-circuits CPython's module-dict probe chain for
/// read-only sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1476; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the stdlib
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_process(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_queue(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_cpu_count(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_int(1)
}

unsafe extern "C" fn dispatch_current_process(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Generic surface stub: callable + identity-stable, returns an empty dict
/// sentinel. Used for every additional CPython 3.12 `multiprocessing`
/// top-level name (classes, context factory methods, module objects) so
/// `hasattr`/`callable` surface fixtures pass without full functional
/// conformance (tracked under #1476).
unsafe extern "C" fn dispatch_mp_stub(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// multiprocessing.get_context(method=None) — validates the start method.
/// An unknown method name is a ValueError (CPython), otherwise return the
/// identity-stable context stub.
unsafe extern "C" fn dispatch_get_context(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if let Some(method) = a.first().copied() {
        if !method.is_none() {
            let name = method.as_ptr().and_then(|p| unsafe {
                if let super::super::rc::ObjData::Str(ref s) = (*p).data {
                    Some(s.clone())
                } else {
                    None
                }
            });
            if let Some(s) = name {
                if !matches!(s.as_str(), "fork" | "spawn" | "forkserver") {
                    super::super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "cannot find context for '{s}'"
                        ))),
                    );
                    return MbValue::none();
                }
            }
        }
    }
    MbValue::from_ptr(MbObject::new_dict())
}

/// multiprocessing.Value(typecode_or_type, ...) — validates the typecode. A
/// string that is not a single valid array/ctypes typecode is a TypeError
/// (CPython); a ctypes type object or valid code returns the value stub.
unsafe extern "C" fn dispatch_value(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    const VALID: &str = "cbBuhHiIlLqQfd";
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if let Some(first) = a.first().copied() {
        let s = first.as_ptr().and_then(|p| unsafe {
            if let super::super::rc::ObjData::Str(ref s) = (*p).data {
                Some(s.clone())
            } else {
                None
            }
        });
        if let Some(s) = s {
            let valid = s.len() == 1 && VALID.contains(&s);
            if !valid {
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "bad typecode (must be one of '{VALID}'): {s}"
                    ))),
                );
                return MbValue::none();
            }
        }
    }
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the multiprocessing module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_process = dispatch_process as *const () as usize;
    attrs.insert("Process".into(), MbValue::from_func(addr_process));

    let addr_queue = dispatch_queue as *const () as usize;
    attrs.insert("Queue".into(), MbValue::from_func(addr_queue));

    let addr_cpu_count = dispatch_cpu_count as *const () as usize;
    attrs.insert("cpu_count".into(), MbValue::from_func(addr_cpu_count));

    let addr_current_process = dispatch_current_process as *const () as usize;
    attrs.insert(
        "current_process".into(),
        MbValue::from_func(addr_current_process),
    );

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_process as u64);
        set.insert(addr_queue as u64);
        set.insert(addr_cpu_count as u64);
        set.insert(addr_current_process as u64);
    });

    // surface: missing CPython module constants (auto-added)
    attrs.insert("SUBDEBUG".into(), MbValue::from_int(5));
    attrs.insert("SUBWARNING".into(), MbValue::from_int(25));

    // surface: remaining CPython 3.12 `multiprocessing` public names.
    // Classes, context factory methods, helper functions, and submodule
    // objects are all registered as identity-stable callable stubs so the
    // `hasattr`/`callable` surface fixtures pass. Full functional behavior
    // is tracked under #1476.
    let addr_stub = dispatch_mp_stub as *const () as usize;
    const MP_SURFACE_NAMES: &[&str] = &[
        // classes / exceptions
        "AuthenticationError",
        "BufferTooShort",
        "ProcessError",
        "TimeoutError",
        // context factory methods
        "Array",
        "Barrier",
        "BoundedSemaphore",
        "Condition",
        "Event",
        "JoinableQueue",
        "Lock",
        "Manager",
        "Pipe",
        "Pool",
        "RLock",
        "RawArray",
        "RawValue",
        "Semaphore",
        "SimpleQueue",
        "Value",
        // helper functions
        "active_children",
        "allow_connection_pickling",
        "freeze_support",
        "get_all_start_methods",
        "get_context",
        "get_logger",
        "get_start_method",
        "log_to_stderr",
        "parent_process",
        "set_executable",
        "set_forkserver_preload",
        "set_start_method",
        // submodule objects (hasattr only)
        "context",
        "process",
        "reducer",
        "reduction",
        "sys",
    ];
    for name in MP_SURFACE_NAMES {
        attrs.insert((*name).into(), MbValue::from_func(addr_stub));
    }
    // get_context / Value validate their arguments (override the stub).
    let addr_get_context = dispatch_get_context as *const () as usize;
    attrs.insert("get_context".into(), MbValue::from_func(addr_get_context));
    let addr_value = dispatch_value as *const () as usize;
    attrs.insert("Value".into(), MbValue::from_func(addr_value));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_stub as u64);
        set.insert(addr_get_context as u64);
        set.insert(addr_value as u64);
    });

    super::register_module("multiprocessing", attrs);
}
