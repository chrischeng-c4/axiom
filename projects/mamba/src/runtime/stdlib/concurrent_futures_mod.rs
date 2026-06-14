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

use rustc_hash::FxHashMap;
use crate::runtime::rc::MbRwLock as RwLock;
use std::sync::atomic::AtomicU32;

fn make_instance(class_name: &str, fields_kv: Vec<(&str, MbValue)>) -> MbValue {
    let mut fields = FxHashMap::default();
    for (k, v) in fields_kv {
        fields.insert(k.to_string(), v);
    }
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: class_name.to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn get_field(inst: MbValue, key: &str) -> Option<MbValue> {
    inst.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn set_field(inst: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                super::super::rc::retain_if_ptr(val);
                let prev = fields.write().unwrap().insert(key.to_string(), val);
                if let Some(p) = prev {
                    super::super::rc::release_if_ptr(p);
                }
            }
        }
    }
}

fn seq_items(val: MbValue) -> Vec<MbValue> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => lock.read().ok().map(|g| g.to_vec()),
            ObjData::Tuple(items) => Some(items.clone()),
            _ => None,
        }
    }).unwrap_or_default()
}

fn is_dict_value(v: MbValue) -> bool {
    v.as_ptr().map(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) }).unwrap_or(false)
}

fn raise_cf(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(new_str(exc), new_str(msg));
    MbValue::none()
}

unsafe fn arg_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() { &[] } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    }
}

fn instance_class(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            Some(class_name.clone())
        } else { None }
    })
}

// ── Future ──

fn make_pending_future() -> MbValue {
    make_instance("concurrent.futures.Future", vec![
        ("_state", new_str("PENDING")),
        ("_result", MbValue::none()),
        ("_exception", MbValue::none()),
        ("_callbacks", MbValue::from_ptr(MbObject::new_list(Vec::new()))),
    ])
}

fn future_state(fut: MbValue) -> String {
    get_field(fut, "_state").and_then(|v| v.as_ptr().map(|p| unsafe {
        if let ObjData::Str(ref s) = (*p).data { s.clone() } else { String::new() }
    })).unwrap_or_default()
}

fn future_finish_result(fut: MbValue, value: MbValue) {
    set_field(fut, "_result", value);
    set_field(fut, "_state", new_str("FINISHED"));
    run_done_callbacks(fut);
}

fn future_finish_exception(fut: MbValue, exc: MbValue) {
    set_field(fut, "_exception", exc);
    set_field(fut, "_state", new_str("FINISHED"));
    run_done_callbacks(fut);
}

fn run_done_callbacks(fut: MbValue) {
    let cbs = get_field(fut, "_callbacks").map(seq_items).unwrap_or_default();
    for cb in cbs {
        let args = MbValue::from_ptr(MbObject::new_list(vec![fut]));
        let _ = super::super::builtins::mb_call_spread(cb, args);
        if super::super::exception::mb_has_exception().as_bool() == Some(true) {
            super::super::exception::mb_clear_exception();
        }
    }
    set_field(fut, "_callbacks", MbValue::from_ptr(MbObject::new_list(Vec::new())));
}

/// Re-raise the stored exception of a finished future.
fn reraise_future_exception(fut: MbValue) -> MbValue {
    let exc = get_field(fut, "_exception").unwrap_or_else(MbValue::none);
    let cls = instance_class(exc).unwrap_or_else(|| "Exception".to_string());
    let msg = super::super::builtins::mb_str(exc);
    let msg_s = msg.as_ptr().map(|p| unsafe {
        if let ObjData::Str(ref s) = (*p).data { s.clone() } else { String::new() }
    }).unwrap_or_default();
    raise_cf(&cls, &msg_s)
}

unsafe extern "C" fn future_result(self_v: MbValue, _args: MbValue) -> MbValue {
    match future_state(self_v).as_str() {
        "FINISHED" => {
            let exc = get_field(self_v, "_exception").unwrap_or_else(MbValue::none);
            if !exc.is_none() {
                return reraise_future_exception(self_v);
            }
            get_field(self_v, "_result").unwrap_or_else(MbValue::none)
        }
        "CANCELLED" => raise_cf("CancelledError", ""),
        _ => raise_cf("TimeoutError", ""),
    }
}

unsafe extern "C" fn future_exception(self_v: MbValue, _args: MbValue) -> MbValue {
    get_field(self_v, "_exception").unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn future_done(self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_bool(matches!(future_state(self_v).as_str(), "FINISHED" | "CANCELLED"))
}

unsafe extern "C" fn future_cancelled(self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_bool(future_state(self_v) == "CANCELLED")
}

unsafe extern "C" fn future_running(self_v: MbValue, _args: MbValue) -> MbValue {
    let _ = self_v;
    MbValue::from_bool(false)
}

unsafe extern "C" fn future_cancel(self_v: MbValue, _args: MbValue) -> MbValue {
    match future_state(self_v).as_str() {
        "PENDING" => {
            set_field(self_v, "_state", new_str("CANCELLED"));
            run_done_callbacks(self_v);
            MbValue::from_bool(true)
        }
        "CANCELLED" => MbValue::from_bool(true),
        _ => MbValue::from_bool(false),
    }
}

unsafe extern "C" fn future_add_done_callback(self_v: MbValue, args: MbValue) -> MbValue {
    let cb = seq_items(args).first().copied().unwrap_or_else(MbValue::none);
    if matches!(future_state(self_v).as_str(), "FINISHED" | "CANCELLED") {
        let call_args = MbValue::from_ptr(MbObject::new_list(vec![self_v]));
        let _ = super::super::builtins::mb_call_spread(cb, call_args);
        if super::super::exception::mb_has_exception().as_bool() == Some(true) {
            super::super::exception::mb_clear_exception();
        }
        return MbValue::none();
    }
    if let Some(list) = get_field(self_v, "_callbacks") {
        super::super::list_ops::mb_list_append(list, cb);
    }
    MbValue::none()
}

unsafe extern "C" fn future_set_result(self_v: MbValue, args: MbValue) -> MbValue {
    if future_state(self_v) == "FINISHED" {
        let state = future_state(self_v);
        return raise_cf("InvalidStateError", &format!(
            "{}: {state}", "invalid state"
        ));
    }
    let value = seq_items(args).first().copied().unwrap_or_else(MbValue::none);
    future_finish_result(self_v, value);
    MbValue::none()
}

unsafe extern "C" fn future_set_exception(self_v: MbValue, args: MbValue) -> MbValue {
    if future_state(self_v) == "FINISHED" {
        return raise_cf("InvalidStateError", "invalid state");
    }
    let exc = seq_items(args).first().copied().unwrap_or_else(MbValue::none);
    future_finish_exception(self_v, exc);
    MbValue::none()
}

// ── Executor (synchronous model: submit runs the task immediately) ──

unsafe extern "C" fn dispatch_thread_pool_executor(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let _ = unsafe { arg_slice(args_ptr, nargs) };
    make_instance("concurrent.futures.ThreadPoolExecutor", vec![
        ("_shutdown", MbValue::from_bool(false)),
    ])
}

unsafe extern "C" fn dispatch_process_pool_executor(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    unsafe { dispatch_thread_pool_executor(args_ptr, nargs) }
}

unsafe extern "C" fn dispatch_future(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    make_pending_future()
}

unsafe extern "C" fn executor_submit(self_v: MbValue, args: MbValue) -> MbValue {
    if get_field(self_v, "_shutdown").and_then(|v| v.as_bool()) == Some(true) {
        return raise_cf("RuntimeError", "cannot schedule new futures after shutdown");
    }
    let items: Vec<MbValue> = seq_items(args).into_iter()
        .filter(|v| !is_dict_value(*v))
        .collect();
    let func = items.first().copied().unwrap_or_else(MbValue::none);
    let call_args = MbValue::from_ptr(MbObject::new_list(items[1..].to_vec()));
    let fut = make_pending_future();
    let result = super::super::builtins::mb_call_spread(func, call_args);
    if super::super::exception::mb_has_exception().as_bool() == Some(true) {
        let exc = super::super::class::mb_catch_exception_instance();
        future_finish_exception(fut, exc);
    } else {
        future_finish_result(fut, result);
    }
    fut
}

unsafe extern "C" fn executor_map(self_v: MbValue, args: MbValue) -> MbValue {
    if get_field(self_v, "_shutdown").and_then(|v| v.as_bool()) == Some(true) {
        return raise_cf("RuntimeError", "cannot schedule new futures after shutdown");
    }
    let items: Vec<MbValue> = seq_items(args).into_iter()
        .filter(|v| !is_dict_value(*v))
        .collect();
    let func = items.first().copied().unwrap_or_else(MbValue::none);
    let iterable = items.get(1).copied().unwrap_or_else(MbValue::none);
    // Materialize the iterable (list/tuple or iterator handle).
    let elems = {
        let direct = seq_items(iterable);
        if !direct.is_empty() || iterable.as_ptr().is_some() {
            direct
        } else {
            let handle = super::super::iter::mb_iter(iterable);
            let mut out = Vec::new();
            if !handle.is_none() {
                loop {
                    if super::super::iter::mb_has_next(handle).as_bool() != Some(true) {
                        break;
                    }
                    out.push(super::super::iter::mb_next(handle));
                }
            }
            out
        }
    };
    let mut results = Vec::with_capacity(elems.len());
    for e in elems {
        let call_args = MbValue::from_ptr(MbObject::new_list(vec![e]));
        let r = super::super::builtins::mb_call_spread(func, call_args);
        if super::super::exception::mb_has_exception().as_bool() == Some(true) {
            // CPython surfaces the task exception when the result is consumed;
            // the synchronous model surfaces it immediately.
            return MbValue::none();
        }
        results.push(r);
    }
    MbValue::from_ptr(MbObject::new_list(results))
}

unsafe extern "C" fn executor_shutdown(self_v: MbValue, _args: MbValue) -> MbValue {
    set_field(self_v, "_shutdown", MbValue::from_bool(true));
    MbValue::none()
}

unsafe extern "C" fn executor_enter(self_v: MbValue, _args: MbValue) -> MbValue {
    self_v
}

unsafe extern "C" fn executor_exit(self_v: MbValue, _args: MbValue) -> MbValue {
    set_field(self_v, "_shutdown", MbValue::from_bool(true));
    MbValue::from_bool(false)
}

// ── module-level helpers ──

/// Futures from a list/tuple/set or — like CPython iteration — a dict's keys.
fn futures_of(v: MbValue) -> Vec<MbValue> {
    if is_dict_value(v) {
        return seq_items(super::super::dict_ops::mb_dict_keys(v));
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            if let ObjData::Set(ref lock) = (*ptr).data {
                return lock.read().unwrap().iter().copied().collect();
            }
        }
    }
    seq_items(v)
}

unsafe extern "C" fn dispatch_as_completed(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let futs = a.first().copied().map(futures_of).unwrap_or_default();
    MbValue::from_ptr(MbObject::new_list(futs))
}

unsafe extern "C" fn dispatch_wait(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { arg_slice(args_ptr, nargs) };
    let futs = a.first().copied().map(futures_of).unwrap_or_default();
    let mut done = Vec::new();
    let mut not_done = Vec::new();
    for f in futs {
        if matches!(future_state(f).as_str(), "FINISHED" | "CANCELLED") {
            done.push(f);
        } else {
            not_done.push(f);
        }
    }
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_set(done)),
        MbValue::from_ptr(MbObject::new_set(not_done)),
    ]))
}

unsafe extern "C" fn dispatch_executor(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_timeout_error(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

fn register_cf_classes() {
    use std::collections::HashMap as Map;
    let var = |addr: usize| {
        super::super::module::register_variadic_func(addr as u64);
        MbValue::from_func(addr)
    };
    let mut ex: Map<String, MbValue> = Map::new();
    ex.insert("submit".into(), var(executor_submit as *const () as usize));
    ex.insert("map".into(), var(executor_map as *const () as usize));
    ex.insert("shutdown".into(), var(executor_shutdown as *const () as usize));
    ex.insert("__enter__".into(), var(executor_enter as *const () as usize));
    ex.insert("__exit__".into(), var(executor_exit as *const () as usize));
    super::super::class::mb_class_register(
        "concurrent.futures.ThreadPoolExecutor", vec![], ex.clone());

    let mut fut: Map<String, MbValue> = Map::new();
    fut.insert("result".into(), var(future_result as *const () as usize));
    fut.insert("exception".into(), var(future_exception as *const () as usize));
    fut.insert("done".into(), var(future_done as *const () as usize));
    fut.insert("cancelled".into(), var(future_cancelled as *const () as usize));
    fut.insert("running".into(), var(future_running as *const () as usize));
    fut.insert("cancel".into(), var(future_cancel as *const () as usize));
    fut.insert("add_done_callback".into(), var(future_add_done_callback as *const () as usize));
    fut.insert("set_result".into(), var(future_set_result as *const () as usize));
    fut.insert("set_exception".into(), var(future_set_exception as *const () as usize));
    super::super::class::mb_class_register("concurrent.futures.Future", vec![], fut);
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
    register_cf_classes();
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

    // TimeoutError resolves to the builtin exception name for except clauses.
    super::super::class::mb_class_register(
        "TimeoutError", vec!["OSError".to_string()], HashMap::new());
    attrs.insert("TimeoutError".into(), make_exception_type_object("TimeoutError"));
    let addr_timeout = dispatch_timeout_error as *const () as usize;
    let _ = addr_timeout;

    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        map.insert(addr_fut as u64, "concurrent.futures.Future".to_string());
        map.insert(addr_tpe as u64, "concurrent.futures.ThreadPoolExecutor".to_string());
    });
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
