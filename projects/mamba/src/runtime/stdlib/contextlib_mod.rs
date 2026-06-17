/// contextlib module for Mamba (#413).
///
/// Provides: contextlib.suppress(*exceptions), contextlib.nullcontext(value),
///           contextlib.contextmanager (stub marker).
/// Suppress creates a dict describing which exception types to suppress;
/// actual suppression is handled at runtime by with-statement codegen.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

/// Helper: extract a string from an MbValue.
#[allow(dead_code)]
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    })
}

/// Read a named field off an Instance value.
fn inst_field(obj: MbValue, name: &str) -> Option<MbValue> {
    obj.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(name).copied()
        } else {
            None
        }
    })
}

/// Set a named field on an Instance value.
fn set_inst_field(obj: MbValue, name: &str, value: MbValue) {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.write().unwrap().insert(name.to_string(), value);
            }
        }
    }
}

/// Collect the elements of a list/tuple MbValue into a Vec.
fn items_of(val: MbValue) -> Vec<MbValue> {
    val.as_ptr()
        .map(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => lock.read().unwrap().iter().copied().collect(),
                ObjData::Tuple(items) => items.iter().copied().collect(),
                _ => vec![val],
            }
        })
        .unwrap_or_default()
}

// ── Variadic dispatchers ──

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

disp_unary!(d_nullcontext, mb_contextlib_nullcontext);
disp_unary!(d_contextmanager, mb_contextlib_contextmanager);

unsafe extern "C" fn d_noop(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

/// `AbstractContextManager` used as a type object. Registered in
/// NATIVE_TYPE_NAMES so `resolve_class_name` maps the pointer back to the
/// class name, enabling the structural `__subclasshook__` in `mb_issubclass`.
unsafe extern "C" fn d_abstract_cm(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn d_abstract_async_cm(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn d_identity(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    a.get(0).copied().unwrap_or_else(MbValue::none)
}

/// Build a dict carrying the no-op methods commonly used on ExitStack-like
/// context managers. `mb_getattr` on a dict resolves `.method` as a dict key
/// (class.rs:1443), so `stack.enter_context(...)` finds the stored callable
/// here instead of NoneType-erroring.
fn make_exit_stack_dict() -> MbValue {
    let noop = d_noop as usize;
    let identity = d_identity as usize;
    let methods: &[(&str, usize)] = &[
        ("enter_context", identity),
        ("push", identity),
        ("callback", identity),
        ("push_async_exit", identity),
        ("push_async_callback", identity),
        ("enter_async_context", identity),
        ("pop_all", noop),
        ("close", noop),
        ("aclose", noop),
        ("__enter__", d_self_returner as usize),
        ("__exit__", noop),
        ("__aenter__", d_self_returner as usize),
        ("__aexit__", noop),
    ];
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            for (name, addr) in methods {
                let v = MbValue::from_func(*addr);
                super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
                    s.borrow_mut().insert(*addr as u64);
                });
                map.insert(super::super::dict_ops::DictKey::Str((*name).to_string()), v);
            }
        }
    }
    MbValue::from_ptr(dict)
}

unsafe extern "C" fn d_self_returner(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    a.get(0).copied().unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn d_exit_stack_ctor(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    let _ = make_exit_stack_dict; // retained for reference; superseded below.
    new_exit_stack("contextlib.ExitStack")
}

unsafe extern "C" fn d_async_exit_stack_ctor(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    new_exit_stack("contextlib.AsyncExitStack")
}

// ── contextlib.ExitStack — native, stateful context manager ───────────────
//
// An ExitStack instance keeps a `_callbacks` list of entries; each entry is a
// 3-tuple `(kind, callable, args)`:
//   kind = "cm"   → callable is a context manager; exit calls its __exit__.
//   kind = "cb"   → callable(*args) on exit (exception state ignored).
//   kind = "exit" → callable(exc_type, exc_val, exc_tb) on exit; honors its
//                   truthy return to suppress.
// Callbacks unwind LIFO. Method dispatch is intercepted in class.rs
// (`mb_call_method`) by class_name and routed to `mb_exitstack_method`.

fn new_exit_stack(class_name: &str) -> MbValue {
    let inst = MbValue::from_ptr(MbObject::new_instance(class_name.to_string()));
    let cbs = MbValue::from_ptr(MbObject::new_list(vec![]));
    set_inst_field(inst, "_callbacks", cbs);
    inst
}

fn exit_stack_callbacks(self_v: MbValue) -> MbValue {
    match inst_field(self_v, "_callbacks") {
        Some(v) if v.as_ptr().is_some() => v,
        _ => {
            let cbs = MbValue::from_ptr(MbObject::new_list(vec![]));
            set_inst_field(self_v, "_callbacks", cbs);
            cbs
        }
    }
}

fn make_entry(kind: &str, callable: MbValue, args: Vec<MbValue>) -> MbValue {
    let kind_v = MbValue::from_ptr(MbObject::new_str(kind.to_string()));
    let args_v = MbValue::from_ptr(MbObject::new_list(args));
    unsafe {
        super::super::rc::retain_if_ptr(callable);
    }
    MbValue::from_ptr(MbObject::new_tuple(vec![kind_v, callable, args_v]))
}

/// Run every registered callback in LIFO order. `pending` is the in-flight
/// exception value (or None). Returns true if the exception was suppressed by
/// one of the exit callbacks. New exceptions raised by callbacks become the
/// in-flight exception for the remaining (earlier-registered) callbacks.
fn exit_stack_unwind(self_v: MbValue, mut pending: MbValue) -> bool {
    let cbs = exit_stack_callbacks(self_v);
    let entries = items_of(cbs);
    // Drain the stack so a re-close is a no-op (pop_all/close semantics).
    if let Some(ptr) = cbs.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.write().unwrap().clear();
            }
        }
    }
    let mut suppressed_all = false;
    for entry in entries.into_iter().rev() {
        let parts = items_of(entry);
        if parts.len() < 3 {
            continue;
        }
        let kind = extract_str(parts[0]).unwrap_or_default();
        let callable = parts[1];
        let args = items_of(parts[2]);

        let has_exc = !pending.is_none();
        match kind.as_str() {
            "cb" => {
                // Plain callback — invoked regardless of exception state.
                let args_list = MbValue::from_ptr(MbObject::new_list(args));
                let _ = super::super::builtins::mb_call_spread(callable, args_list);
                // A callback that itself raises becomes the new pending exc.
                if super::super::exception::mb_has_exception().as_bool() == Some(true) {
                    pending = super::super::exception::mb_get_exception();
                    super::super::exception::mb_clear_exception();
                }
            }
            _ => {
                // "cm" and "exit": call __exit__-style with (type, val, tb).
                let none = MbValue::none();
                let (et, ev, tb) = if has_exc {
                    (pending, pending, none)
                } else {
                    (none, none, none)
                };
                let result = if kind == "cm" {
                    // Drive the wrapped context manager's exit. For generator
                    // CMs the runtime exception slot must reflect `pending`.
                    if has_exc {
                        super::super::exception::mb_reraise(pending);
                    }
                    super::super::class::mb_context_exit(callable, MbValue::none())
                } else {
                    // Raw exit callable: exitfn(exc_type, exc_val, exc_tb).
                    let args_list = MbValue::from_ptr(MbObject::new_list(vec![et, ev, tb]));
                    super::super::builtins::mb_call_spread(callable, args_list)
                };
                // After a "cm" exit, mb_context_exit already re-raised the
                // exception if not suppressed; capture the resulting state.
                if kind == "cm" {
                    let still = super::super::exception::mb_has_exception().as_bool()
                        == Some(true);
                    if has_exc && !still {
                        // suppressed
                        pending = MbValue::none();
                        suppressed_all = true;
                    } else if still {
                        pending = super::super::exception::mb_get_exception();
                        super::super::exception::mb_clear_exception();
                    }
                } else {
                    // Raw exit callable: truthy return suppresses.
                    let suppress = super::super::builtins::mb_is_truthy(result) != 0;
                    if has_exc && suppress {
                        pending = MbValue::none();
                        suppressed_all = true;
                    }
                    // A raw exit callable that itself raised replaces pending.
                    if super::super::exception::mb_has_exception().as_bool() == Some(true) {
                        pending = super::super::exception::mb_get_exception();
                        super::super::exception::mb_clear_exception();
                    }
                }
            }
        }
    }
    // Re-raise whatever exception survived the unwind.
    if !pending.is_none() {
        super::super::exception::mb_reraise(pending);
        return false;
    }
    suppressed_all
}

/// `ExitStack.__enter__(self)` → self.
extern "C" fn exit_stack_enter(self_v: MbValue) -> MbValue {
    unsafe { super::super::rc::retain_if_ptr(self_v); }
    self_v
}

/// `ExitStack.__exit__(self, exc_type, exc_val, exc_tb)` → unwind LIFO,
/// returning truthy when the in-flight exception is suppressed.
extern "C" fn exit_stack_exit(
    self_v: MbValue,
    exc_type: MbValue,
    _exc_val: MbValue,
    _exc_tb: MbValue,
) -> MbValue {
    let suppressed = exit_stack_unwind(self_v, exc_type);
    MbValue::from_bool(suppressed)
}

/// Dispatch an ExitStack method. `self_v` is the instance, `name` the method,
/// `args` the positional-arg list (mamba packs trailing kwargs as the final
/// list element, which the no-kwargs fixtures never exercise).
pub fn mb_exitstack_method(self_v: MbValue, name: &str, args: MbValue) -> MbValue {
    let items = items_of(args);
    match name {
        "enter_context" => {
            let cm = items.first().copied().unwrap_or_else(MbValue::none);
            let entered = super::super::class::mb_context_enter(cm);
            let entry = make_entry("cm", cm, vec![]);
            super::super::list_ops::mb_list_append(exit_stack_callbacks(self_v), entry);
            entered
        }
        "callback" => {
            let cb = items.first().copied().unwrap_or_else(MbValue::none);
            let rest = items.get(1..).map(|s| s.to_vec()).unwrap_or_default();
            let entry = make_entry("cb", cb, rest);
            super::super::list_ops::mb_list_append(exit_stack_callbacks(self_v), entry);
            unsafe { super::super::rc::retain_if_ptr(cb); }
            cb // callback() returns the function unchanged
        }
        "push" => {
            let exitfn = items.first().copied().unwrap_or_else(MbValue::none);
            let entry = make_entry("exit", exitfn, vec![]);
            super::super::list_ops::mb_list_append(exit_stack_callbacks(self_v), entry);
            unsafe { super::super::rc::retain_if_ptr(exitfn); }
            exitfn // push() returns the callable unchanged
        }
        "pop_all" => {
            // Transfer all callbacks to a fresh stack; clear ours.
            let class_name = self_v
                .as_ptr()
                .and_then(|p| unsafe {
                    if let ObjData::Instance { ref class_name, .. } = (*p).data {
                        Some(class_name.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "contextlib.ExitStack".to_string());
            let new_stack = new_exit_stack(&class_name);
            let src = exit_stack_callbacks(self_v);
            let moved = items_of(src);
            if let Some(ptr) = src.as_ptr() {
                unsafe {
                    if let ObjData::List(ref lock) = (*ptr).data {
                        lock.write().unwrap().clear();
                    }
                }
            }
            let dst = exit_stack_callbacks(new_stack);
            for e in moved {
                super::super::list_ops::mb_list_append(dst, e);
            }
            new_stack
        }
        "close" | "aclose" => {
            exit_stack_unwind(self_v, MbValue::none());
            MbValue::none()
        }
        "__enter__" | "__aenter__" => {
            unsafe { super::super::rc::retain_if_ptr(self_v); }
            self_v
        }
        "__exit__" | "__aexit__" => {
            let exc = items.first().copied().unwrap_or_else(MbValue::none);
            let suppressed = exit_stack_unwind(self_v, exc);
            MbValue::from_bool(suppressed)
        }
        _ => MbValue::none(),
    }
}

// ── contextlib.closing — native context-manager class ─────────────────────
//
// `closing(obj)` returns an instance whose `__enter__` yields the wrapped
// object and whose `__exit__` calls `obj.close()` (re-raising any body
// exception, i.e. never suppressing). Method ABIs are chosen to match how the
// `with` lowering dispatches them: `mb_context_enter` calls `__enter__(self)`
// via `mb_call_method1` (1-arg SystemV), and `mb_context_exit` calls
// `__exit__(self, exc_type, exc_val, exc_tb)` via a registered 4-arg SystemV
// call. `mb_class_register` registers both addresses in CALLABLE_REGISTRY so
// those direct-dispatch paths fire.

/// `closing.__enter__(self)` → the wrapped object (so `with closing(r) as y`
/// binds `y is r`).
extern "C" fn closing_enter(self_v: MbValue) -> MbValue {
    let wrapped = inst_field(self_v, "_thing").unwrap_or_else(MbValue::none);
    unsafe { super::super::rc::retain_if_ptr(wrapped); }
    wrapped
}

/// `closing.__exit__(self, *exc_info)` → call `wrapped.close()`, never
/// suppress. Returning a falsy value means a pending exception is re-raised
/// by `mb_context_exit`.
extern "C" fn closing_exit(
    self_v: MbValue,
    _exc_type: MbValue,
    _exc_val: MbValue,
    _exc_tb: MbValue,
) -> MbValue {
    if let Some(wrapped) = inst_field(self_v, "_thing") {
        let name = MbValue::from_ptr(MbObject::new_str("close".to_string()));
        let empty = MbValue::from_ptr(MbObject::new_list(vec![]));
        let _ = super::super::class::mb_call_method(wrapped, name, empty);
    }
    MbValue::from_bool(false)
}

/// `contextlib.closing(thing)` constructor.
unsafe extern "C" fn d_closing_ctor(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let thing = a.first().copied().unwrap_or_else(MbValue::none);
    let inst = MbValue::from_ptr(MbObject::new_instance("contextlib.closing".to_string()));
    unsafe { super::super::rc::retain_if_ptr(thing); }
    set_inst_field(inst, "_thing", thing);
    inst
}

// ── contextlib.suppress — native context-manager class ────────────────────
//
// `suppress(*exceptions)` returns an instance storing the suppressed exception
// type names. `__enter__` returns None; `__exit__` inspects the pending
// exception (passed as `exc_type` by `mb_context_exit`) and returns truthy iff
// it is an instance of one of the suppressed types — matching CPython's
// subclass-aware behavior. Stateless reads of `_exceptions` make the same
// instance reusable and reentrant for free.

/// `suppress.__enter__(self)` → None.
extern "C" fn suppress_enter(_self_v: MbValue) -> MbValue {
    MbValue::none()
}

/// `suppress.__exit__(self, exc_type, exc_val, exc_tb)` → True to suppress when
/// the in-flight exception matches a stored type; False otherwise.
extern "C" fn suppress_exit(
    self_v: MbValue,
    exc_type: MbValue,
    _exc_val: MbValue,
    _exc_tb: MbValue,
) -> MbValue {
    // No exception in flight → nothing to suppress.
    if exc_type.is_none() {
        return MbValue::from_bool(false);
    }
    let Some(stored) = inst_field(self_v, "_exceptions") else {
        return MbValue::from_bool(false);
    };
    for exc_class in items_of(stored) {
        // `exc_type` is the exception's CLASS object (CPython __exit__ contract),
        // so check it with issubclass; also accept the value form via isinstance
        // for robustness. Both walk the exception hierarchy, so
        // suppress(ArithmeticError) catches ZeroDivisionError.
        if super::super::class::mb_issubclass(exc_type, exc_class).as_bool() == Some(true)
            || super::super::class::mb_isinstance(exc_type, exc_class).as_bool() == Some(true)
        {
            return MbValue::from_bool(true);
        }
    }
    MbValue::from_bool(false)
}

/// `contextlib.suppress(*exceptions)` constructor.
unsafe extern "C" fn d_suppress_ctor(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let inst = MbValue::from_ptr(MbObject::new_instance("contextlib.suppress".to_string()));
    let mut excs: Vec<MbValue> = Vec::with_capacity(nargs);
    for &e in a {
        unsafe { super::super::rc::retain_if_ptr(e); }
        excs.push(e);
    }
    let list = MbValue::from_ptr(MbObject::new_list(excs));
    set_inst_field(inst, "_exceptions", list);
    inst
}

// ── contextlib.contextmanager — generator-driven context manager ──────────
//
// `@contextmanager` decorates a generator function. The decorator returns the
// generator function unchanged, so the call-site arg/kwarg lowering (which
// already knows the wrapped function's parameter names) forwards arguments
// correctly and `cm(...)` produces a *generator handle*. The `with` lowering
// then routes generator handles to `cm_gen_enter` / `cm_gen_exit` (hooked in
// `mb_context_enter` / `mb_context_exit`), which drive the generator:
//   __enter__: run the body to the first `yield`, return the yielded value.
//   __exit__:  resume (or `throw` into) the generator, enforcing the CPython
//              "generator didn't stop" / suppression / re-raise contract.

/// Extract `(type_name, message)` from a pending-exception MbValue (an
/// Instance whose `class_name` is the exception type and whose `message`
/// field holds the str).
fn exc_type_and_msg(exc: MbValue) -> (String, String) {
    let ty = exc
        .as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                Some(class_name.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "Exception".to_string());
    let msg = inst_field(exc, "message")
        .and_then(extract_str)
        .unwrap_or_default();
    (ty, msg)
}

/// `@contextmanager` __enter__ for a generator handle: advance to the first
/// `yield` and return the yielded value. If the generator never yields (body
/// returns immediately) CPython raises `RuntimeError("generator didn't
/// yield")`.
pub fn cm_gen_enter(gen_handle: MbValue) -> MbValue {
    let yielded = super::super::generator::mb_generator_next(gen_handle);
    // If advancing completed the generator, mb_generator_next leaves a pending
    // StopIteration (or a PEP-479 RuntimeError). A contextmanager body that
    // does not yield must surface RuntimeError("generator didn't yield").
    let exhausted = super::super::generator::mb_generator_is_exhausted(gen_handle)
        .as_bool()
        == Some(true);
    if exhausted {
        // Clear whatever the completion raised and substitute the canonical
        // "didn't yield" RuntimeError.
        super::super::exception::clear_current_exception();
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("RuntimeError".to_string())),
            MbValue::from_ptr(MbObject::new_str("generator didn't yield".to_string())),
        );
        return MbValue::none();
    }
    yielded
}

/// `@contextmanager` __exit__ for a generator handle. Returns truthy to
/// suppress a pending exception. The pending exception (if any) is read from
/// the runtime by `mb_context_exit` and passed in as `exc_type`; that helper
/// has already cleared the runtime exception slot before calling us.
pub fn cm_gen_exit(
    gen_handle: MbValue,
    exc_type: MbValue,
    _exc_val: MbValue,
    _exc_tb: MbValue,
) -> MbValue {
    let has_exc = !exc_type.is_none();

    if !has_exc {
        // Clean exit: resume the generator; it must run to completion.
        let _ = super::super::generator::mb_generator_next(gen_handle);
        let exhausted = super::super::generator::mb_generator_is_exhausted(gen_handle)
            .as_bool()
            == Some(true);
        if !exhausted {
            // Yielded a second time → RuntimeError("generator didn't stop").
            super::super::generator::mb_generator_close(gen_handle);
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("RuntimeError".to_string())),
                MbValue::from_ptr(MbObject::new_str("generator didn't stop".to_string())),
            );
            return MbValue::from_bool(false);
        }
        // Completion raised StopIteration; consume it so it does not leak.
        if super::super::exception::current_exception_type().as_deref() == Some("StopIteration") {
            super::super::exception::clear_current_exception();
        }
        return MbValue::from_bool(false);
    }

    // Exception in flight: throw it into the generator at the yield point.
    let (ty, msg) = exc_type_and_msg(exc_type);
    let type_v = MbValue::from_ptr(MbObject::new_str(ty.clone()));
    let msg_v = MbValue::from_ptr(MbObject::new_str(msg.clone()));
    let yielded = super::super::generator::mb_generator_throw(gen_handle, type_v, msg_v);

    let exhausted = super::super::generator::mb_generator_is_exhausted(gen_handle)
        .as_bool()
        == Some(true);

    if !exhausted {
        // The generator caught the exception and yielded again → RuntimeError
        // "generator didn't stop after throw()".
        let _ = yielded;
        super::super::generator::mb_generator_close(gen_handle);
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("RuntimeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "generator didn't stop after throw()".to_string(),
            )),
        );
        return MbValue::from_bool(false);
    }

    // Generator completed. Inspect the resulting exception state:
    //   - StopIteration  → the body swallowed the exception (suppress).
    //   - same exception → the body re-raised it (do NOT suppress; the caller
    //                       re-raises the original `pending`).
    //   - different exc  → the body raised a new exception; let it propagate
    //                       (return falsy but the new exception is already
    //                       pending, so `mb_context_exit` must not re-raise
    //                       the original — handled by returning a sentinel that
    //                       leaves the new exception in place).
    let cur = super::super::exception::current_exception_type();
    match cur.as_deref() {
        Some("StopIteration") => {
            // Body caught the exception → suppress it.
            super::super::exception::clear_current_exception();
            MbValue::from_bool(true)
        }
        Some(other) if other == ty => {
            // Re-raised the same exception type. CPython compares identity;
            // here we compare type+message to decide whether it's the same
            // logical exception. If so, do not suppress — but we have already
            // consumed the runtime slot conceptually. Return falsy so the
            // caller re-raises the original `pending` (same effect).
            super::super::exception::clear_current_exception();
            MbValue::from_bool(false)
        }
        Some(_) => {
            // A *different* exception escaped the generator. Leave it pending
            // and signal "suppress the original" so `mb_context_exit` does not
            // clobber it by re-raising the original — the new exception is the
            // one that must propagate.
            MbValue::from_bool(true)
        }
        None => {
            // No exception pending: body returned without re-raising → suppress.
            MbValue::from_bool(true)
        }
    }
}

// ── contextlib.redirect_stdout — native context manager ───────────────────
//
// `redirect_stdout(target)` stores `target` (an io.StringIO-like writable
// stream). `__enter__` pushes the target onto the output module's stdout
// redirect stack and returns it (so `with redirect_stdout(buf) as x: x is
// buf`). `__exit__` pops the target, restoring the previous stdout. Reusable:
// re-entering pushes again, so a single instance can wrap multiple blocks.

/// `redirect_stdout.__enter__(self)` → the redirect target (pushed onto the
/// stdout redirect stack).
extern "C" fn redirect_stdout_enter(self_v: MbValue) -> MbValue {
    let target = inst_field(self_v, "_new_target").unwrap_or_else(MbValue::none);
    super::super::output::push_stdout_redirect(target);
    unsafe { super::super::rc::retain_if_ptr(target); }
    target
}

/// `redirect_stdout.__exit__(self, *exc)` → pop the redirect, never suppress.
extern "C" fn redirect_stdout_exit(
    _self_v: MbValue,
    _exc_type: MbValue,
    _exc_val: MbValue,
    _exc_tb: MbValue,
) -> MbValue {
    super::super::output::pop_stdout_redirect();
    MbValue::from_bool(false)
}

/// `contextlib.redirect_stdout(target)` constructor.
unsafe extern "C" fn d_redirect_stdout_ctor(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let target = a.first().copied().unwrap_or_else(MbValue::none);
    let inst = MbValue::from_ptr(MbObject::new_instance("contextlib.redirect_stdout".to_string()));
    unsafe { super::super::rc::retain_if_ptr(target); }
    set_inst_field(inst, "_new_target", target);
    inst
}

// ── contextlib.redirect_stderr — native context manager ───────────────────

/// `redirect_stderr.__enter__(self)` → the redirect target (pushed onto the
/// stderr redirect stack).
extern "C" fn redirect_stderr_enter(self_v: MbValue) -> MbValue {
    let target = inst_field(self_v, "_new_target").unwrap_or_else(MbValue::none);
    super::super::output::push_stderr_redirect(target);
    unsafe { super::super::rc::retain_if_ptr(target); }
    target
}

/// `redirect_stderr.__exit__(self, *exc)` → pop the redirect, never suppress.
extern "C" fn redirect_stderr_exit(
    _self_v: MbValue,
    _exc_type: MbValue,
    _exc_val: MbValue,
    _exc_tb: MbValue,
) -> MbValue {
    super::super::output::pop_stderr_redirect();
    MbValue::from_bool(false)
}

/// `contextlib.redirect_stderr(target)` constructor.
unsafe extern "C" fn d_redirect_stderr_ctor(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let target = a.first().copied().unwrap_or_else(MbValue::none);
    let inst = MbValue::from_ptr(MbObject::new_instance("contextlib.redirect_stderr".to_string()));
    unsafe { super::super::rc::retain_if_ptr(target); }
    set_inst_field(inst, "_new_target", target);
    inst
}

// ── contextlib.ContextDecorator — base class usable as a decorator ────────
//
// A `ContextDecorator` subclass can be used as a function decorator: applying
// an instance to a function wraps every call in `with self: func(*args)`.
// `__call__(self, func)` returns a `_cd_wrapper` instance capturing the
// manager instance and the wrapped function; the wrapper's own (variadic)
// `__call__` performs the enter / call / exit dance, re-entering the manager
// on every invocation.

/// `ContextDecorator.__call__(self, func)` → a callable wrapper instance.
extern "C" fn context_decorator_call(self_v: MbValue, func: MbValue) -> MbValue {
    let wrapper =
        MbValue::from_ptr(MbObject::new_instance("contextlib._cd_wrapper".to_string()));
    unsafe {
        super::super::rc::retain_if_ptr(self_v);
        super::super::rc::retain_if_ptr(func);
    }
    set_inst_field(wrapper, "_cm", self_v);
    set_inst_field(wrapper, "_func", func);
    wrapper
}

/// `_cd_wrapper.__call__(self, *args)` → `with cm: return func(*args)`.
/// Registered as a variadic method, so the second argument is the positional
/// arg list.
extern "C" fn cd_wrapper_call(self_v: MbValue, args_list: MbValue) -> MbValue {
    let cm = inst_field(self_v, "_cm").unwrap_or_else(MbValue::none);
    let func = inst_field(self_v, "_func").unwrap_or_else(MbValue::none);

    let _ = super::super::class::mb_context_enter(cm);
    let result = super::super::builtins::mb_call_spread(func, args_list);

    // Forward any exception state into __exit__ (mb_context_exit reads the
    // runtime exception slot itself).
    let exited = super::super::class::mb_context_exit(cm, MbValue::none());
    let _ = exited;
    result
}

/// Register a native context-manager class with `__enter__`/`__exit__`.
/// `mb_class_register` records both method addresses in CALLABLE_REGISTRY so
/// the `with` lowering's direct-dispatch paths (`mb_call_method1` for enter,
/// the 4-arg registered call for exit) invoke them with the correct ABI.
fn register_cm_class(name: &str, enter: usize, exit: usize) {
    let mut methods: HashMap<String, MbValue> = HashMap::new();
    methods.insert("__enter__".to_string(), MbValue::from_func(enter));
    methods.insert("__exit__".to_string(), MbValue::from_func(exit));
    super::super::class::mb_class_register(name, vec!["object".to_string()], methods);
}

/// Register the contextlib module.
pub fn register() {
    let mut attrs = HashMap::new();

    // Native context-manager classes whose __enter__/__exit__ drive the
    // `with` lowering directly.
    register_cm_class("contextlib.closing", closing_enter as usize, closing_exit as usize);
    register_cm_class("contextlib.suppress", suppress_enter as usize, suppress_exit as usize);
    register_cm_class(
        "contextlib.ExitStack",
        exit_stack_enter as usize,
        exit_stack_exit as usize,
    );
    register_cm_class(
        "contextlib.AsyncExitStack",
        exit_stack_enter as usize,
        exit_stack_exit as usize,
    );
    register_cm_class(
        "contextlib.redirect_stdout",
        redirect_stdout_enter as usize,
        redirect_stdout_exit as usize,
    );
    register_cm_class(
        "contextlib.redirect_stderr",
        redirect_stderr_enter as usize,
        redirect_stderr_exit as usize,
    );

    // ContextDecorator base class: subclasses inherit `__call__`, making an
    // instance usable as a function decorator.
    {
        let mut m: HashMap<String, MbValue> = HashMap::new();
        m.insert(
            "__call__".to_string(),
            MbValue::from_func(context_decorator_call as usize),
        );
        super::super::class::mb_class_register(
            "ContextDecorator",
            vec!["object".to_string()],
            m,
        );
        super::super::class::mb_class_register(
            "AsyncContextDecorator",
            vec!["object".to_string()],
            HashMap::new(),
        );
    }
    // The decorator wrapper instance: its variadic `__call__` runs the wrapped
    // function inside the manager's with-block on each invocation.
    {
        let addr = cd_wrapper_call as usize;
        super::super::module::register_variadic_func(addr as u64);
        let mut m: HashMap<String, MbValue> = HashMap::new();
        m.insert("__call__".to_string(), MbValue::from_func(addr));
        super::super::class::mb_class_register(
            "contextlib._cd_wrapper",
            vec!["object".to_string()],
            m,
        );
    }

    let noop = d_noop as usize;
    let identity = d_identity as usize;
    let _ = identity;
    let dispatchers: Vec<(&str, usize)> = vec![
        ("suppress", d_suppress_ctor as usize),
        ("nullcontext", d_nullcontext as *const () as usize),
        ("contextmanager", d_contextmanager as *const () as usize),
        ("redirect_stdout", d_redirect_stdout_ctor as usize),
        ("redirect_stderr", d_redirect_stderr_ctor as usize),
        ("closing", d_closing_ctor as usize),
        ("ExitStack", d_exit_stack_ctor as usize),
        ("AsyncExitStack", d_async_exit_stack_ctor as usize),
        ("asynccontextmanager", identity),
        ("AbstractContextManager", d_abstract_cm as usize),
        ("AbstractAsyncContextManager", d_abstract_async_cm as usize),
        ("aclosing", identity),
        ("chdir", noop),
        ("ContextDecorator", noop),
        ("AsyncContextDecorator", noop),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Map the AbstractContextManager type-object pointers to their class names
    // so `resolve_class_name` (and thus `issubclass`) recognizes them.
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        map.insert(d_abstract_cm as usize as u64, "AbstractContextManager".into());
        map.insert(
            d_abstract_async_cm as usize as u64,
            "AbstractAsyncContextManager".into(),
        );
    });

    super::register_module("contextlib", attrs);
}

/// contextlib.suppress(*exceptions) -> dict representing a suppress context manager.
///
/// Stores the exception types list under "_type" = "suppress" and "_exceptions"
/// as a list of exception type name strings. Actual suppression is handled by
/// the with-statement codegen checking exception type against this list.
///
/// Superseded by the native `contextlib.suppress` class (`d_suppress_ctor` +
/// `suppress_enter`/`suppress_exit`); retained for the unit test below.
#[allow(dead_code)]
pub fn mb_contextlib_suppress(exceptions: MbValue) -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "_type".into(),
                MbValue::from_ptr(MbObject::new_str("suppress".to_string())),
            );
            // Store the exceptions list as-is if it's already a list,
            // otherwise wrap it in a single-element list.
            let exc_list = if let Some(ptr) = exceptions.as_ptr() {
                match &(*ptr).data {
                    ObjData::List(_) | ObjData::Tuple(_) => exceptions,
                    _ => MbValue::from_ptr(MbObject::new_list(vec![exceptions])),
                }
            } else if exceptions.is_none() {
                MbValue::from_ptr(MbObject::new_list(vec![]))
            } else {
                MbValue::from_ptr(MbObject::new_list(vec![exceptions]))
            };
            map.insert("_exceptions".into(), exc_list);
        }
    }
    MbValue::from_ptr(dict)
}

/// contextlib.nullcontext(value) -> value as-is (no-op context manager).
///
/// Returns the enter value directly. When used in a `with` statement,
/// the value is simply passed through without any setup or teardown.
pub fn mb_contextlib_nullcontext(value: MbValue) -> MbValue {
    if value.is_none() {
        // Default: return None if no value provided
        MbValue::none()
    } else {
        value
    }
}

/// contextlib.contextmanager — stub marker.
///
/// In CPython this is a decorator that turns a generator into a context manager.
/// Here we return the function as-is since full generator support is pending.
pub fn mb_contextlib_contextmanager(func: MbValue) -> MbValue {
    // Stub: return the function unchanged
    func
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suppress_creates_dict() {
        let exc = MbValue::from_ptr(MbObject::new_str("ValueError".to_string()));
        let result = mb_contextlib_suppress(exc);
        unsafe {
            let ptr = result.as_ptr().expect("expected pointer");
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                let type_val = map.get("_type").expect("missing _type");
                if let ObjData::Str(ref s) = (*type_val.as_ptr().unwrap()).data {
                    assert_eq!(s, "suppress");
                } else {
                    panic!("expected string for _type");
                }
                assert!(map.contains_key("_exceptions"));
            } else {
                panic!("expected dict");
            }
        }
    }

    #[test]
    fn test_nullcontext_returns_value() {
        let val = MbValue::from_int(42);
        let result = mb_contextlib_nullcontext(val);
        assert_eq!(result.as_int(), Some(42));
    }

    #[test]
    fn test_nullcontext_default_none() {
        let result = mb_contextlib_nullcontext(MbValue::none());
        assert!(result.is_none());
    }
}
