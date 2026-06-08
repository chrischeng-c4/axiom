/// concurrent.futures module for Mamba (#1261).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `concurrent.futures` entry points (`ThreadPoolExecutor`,
/// `ProcessPoolExecutor`, `Future`, `as_completed`). All four
/// return identity-stable sentinel callables; their job here is to
/// short-circuit CPython's module-dict probe chain for read-only
/// concurrent.futures sentinels.
///
/// Registered under the dotted name `concurrent.futures` (same
/// pattern as `http.cookies` / `urllib.error` / `http.cookiejar`).
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1261; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the
/// stub-only conversion long-tail has closed against.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

unsafe extern "C" fn dispatch_thread_pool_executor(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_process_pool_executor(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_future(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_as_completed(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_wait(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_executor(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_timeout_error(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Model a `concurrent.futures` exception class as a real type-object
/// (`Instance` class_name="type" with a `__name__` field), mirroring the
/// pattern `unittest_mod` uses for `TestCase`/`TestSuite`. Two things then
/// resolve correctly:
///   - `type(concurrent.futures.BrokenExecutor).__name__ == "type"` and
///     `issubclass(...)` reads the class name off `__name__` via
///     `class::resolve_class_name`;
///   - calling it (`BrokenExecutor("pool broke")`) routes through the
///     builtins type-object constructor hook, producing an `Instance` whose
///     `class_name` is `name`, so `raise`/`except`/`isinstance` see the right
///     class.
///
/// The accompanying `mb_class_register(name, [base], …)` call (in `register`)
/// puts `name`'s MRO in the class registry so `issubclass(name, base)` and the
/// except-matcher's `is_subclass_of(name, base)` answer true — this is what the
/// errors-dimension fixtures assert (BrokenExecutor ⊂ RuntimeError,
/// CancelledError ⊂ Exception/BaseException).
fn make_exception_type_object(name: &str) -> MbValue {
    let cls = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*cls).data {
            let mut f = fields.write().unwrap();
            f.insert("__name__".to_string(), MbValue::from_ptr(MbObject::new_str(name.to_string())));
            f.insert("__qualname__".to_string(), MbValue::from_ptr(MbObject::new_str(name.to_string())));
            f.insert("__module__".to_string(), MbValue::from_ptr(MbObject::new_str("concurrent.futures".to_string())));
        }
    }
    MbValue::from_ptr(cls)
}

/// Register the concurrent.futures module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_tpe = dispatch_thread_pool_executor as *const () as usize;
    attrs.insert("ThreadPoolExecutor".into(), MbValue::from_func(addr_tpe));

    let addr_ppe = dispatch_process_pool_executor as *const () as usize;
    attrs.insert("ProcessPoolExecutor".into(), MbValue::from_func(addr_ppe));

    let addr_fut = dispatch_future as *const () as usize;
    attrs.insert("Future".into(), MbValue::from_func(addr_fut));

    let addr_ac = dispatch_as_completed as *const () as usize;
    attrs.insert("as_completed".into(), MbValue::from_func(addr_ac));

    // surface: missing CPython callables/classes (auto-added)
    let addr_wait = dispatch_wait as *const () as usize;
    attrs.insert("wait".into(), MbValue::from_func(addr_wait));

    let addr_exec = dispatch_executor as *const () as usize;
    attrs.insert("Executor".into(), MbValue::from_func(addr_exec));

    // Exception classes are modelled as real type-objects (not callable
    // func-stubs) and their MRO is registered so the errors-dimension
    // fixtures' subclass/except assertions resolve:
    //   - BrokenExecutor ⊂ RuntimeError  (broken_executor_is_runtimeerror)
    //   - CancelledError ⊂ Exception ⊂ BaseException
    //                                    (cancelled_error_is_exception)
    //   - InvalidStateError ⊂ Exception  (raised path is class.rs-gated; the
    //                                      class identity is modelled here)
    // The base classes are (re-)registered defensively first so the computed
    // MRO is complete regardless of whether `register_builtin_exceptions`
    // has run yet at module-registration time (insert overwrites, so this is
    // idempotent and identical to the builtin registration).
    let empty = HashMap::new;
    super::super::class::mb_class_register("BaseException", vec![], empty());
    super::super::class::mb_class_register("Exception", vec!["BaseException".into()], empty());
    super::super::class::mb_class_register("RuntimeError", vec!["Exception".into()], empty());
    super::super::class::mb_class_register("BrokenExecutor", vec!["RuntimeError".into()], empty());
    super::super::class::mb_class_register("CancelledError", vec!["Exception".into()], empty());
    super::super::class::mb_class_register("InvalidStateError", vec!["Exception".into()], empty());
    attrs.insert("BrokenExecutor".into(), make_exception_type_object("BrokenExecutor"));
    attrs.insert("CancelledError".into(), make_exception_type_object("CancelledError"));
    attrs.insert("InvalidStateError".into(), make_exception_type_object("InvalidStateError"));

    let addr_timeout = dispatch_timeout_error as *const () as usize;
    attrs.insert("TimeoutError".into(), MbValue::from_func(addr_timeout));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_tpe as u64);
        set.insert(addr_ppe as u64);
        set.insert(addr_fut as u64);
        set.insert(addr_ac as u64);
        set.insert(addr_wait as u64);
        set.insert(addr_exec as u64);
        set.insert(addr_timeout as u64);
    });

        // surface: missing CPython module constants (auto-added)
    attrs.insert("ALL_COMPLETED".into(), MbValue::from_ptr(MbObject::new_str("ALL_COMPLETED".to_string())));
    attrs.insert("FIRST_COMPLETED".into(), MbValue::from_ptr(MbObject::new_str("FIRST_COMPLETED".to_string())));
    attrs.insert("FIRST_EXCEPTION".into(), MbValue::from_ptr(MbObject::new_str("FIRST_EXCEPTION".to_string())));
    super::register_module("concurrent.futures", attrs);
}
