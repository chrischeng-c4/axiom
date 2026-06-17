//! @codegen-skip: handwrite-pre-standardize
//!
//! contextvars module for Mamba — Python 3.12 `contextvars` stdlib (#1469).
//!
//! Surface (CPython 3.12 `contextvars` denominator, 4 names):
//!   Context, ContextVar, Token, copy_context
//!
//! HANDWRITE-BEGIN reason: per-section primitive vocabulary for stdlib
//! shims is not yet emitted by score codegen. Tracked as part of the
//! brute-force Phase-2 sweep; will be replaced when aw standardize
//! lands the stdlib-shim section type. Issue #1469.
//!
//! Implementation model:
//!
//! - The *current context* is a thread-local `var_id → value` map, so every
//!   OS thread starts with its own empty context (matching CPython's
//!   per-thread context stack for the simple non-async case).
//! - `ContextVar(name, default=...)` returns an Instance
//!   (`class_name = "ContextVar"`) carrying `name`, a process-unique
//!   `_var_id`, and the optional default. `get` / `set` / `reset` dispatch
//!   through the Instance method path in `class.rs` (same shape as
//!   `re.Pattern`).
//! - `set` returns a `Token` Instance carrying `var`, `old_value`
//!   (or the `Token.MISSING` sentinel) and a `_used` flag; `reset`
//!   validates ownership / reuse like CPython (ValueError / RuntimeError).
//! - `copy_context()` snapshots the current map into a `Context` Instance
//!   (parallel `_ids` / `_vars` / `_vals` lists). `Context.run(fn, *args)`
//!   installs the snapshot, invokes the callable, captures writes back into
//!   the Context, and restores the caller's context (writes do not leak).
//!
//! Carve-outs (documented gaps from CPython parity):
//!
//! - `Context` does not implement the full Mapping protocol (`dict(ctx)`,
//!   `len(ctx)`, membership) — iteration support is gated on the generic
//!   Instance iterator plumbing.
//! - Async-task context propagation (each task owning a context copy) is
//!   wired in `tokio_exec` only insofar as tasks run on pool threads with
//!   their own thread-local context.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use rustc_hash::FxHashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind, InstanceFields, MbRwLock};

thread_local! {
    /// The thread's current context: var_id → value.
    static CURRENT: RefCell<FxHashMap<u64, MbValue>> = RefCell::new(FxHashMap::default());
    /// Lazily-created per-process `Token.MISSING` sentinel (one identity per
    /// process is enough for `is` checks because the sentinel never crosses
    /// the API except through this module).
    static MISSING: RefCell<Option<MbValue>> = const { RefCell::new(None) };
}

static NEXT_VAR_ID: AtomicU64 = AtomicU64::new(1);

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn new_instance(class_name: &str, fields: InstanceFields) -> MbValue {
    let obj = Box::new(MbObject {
        header: MbObjectHeader { rc: AtomicU32::new(1), kind: ObjKind::Instance },
        data: ObjData::Instance {
            class_name: class_name.to_string(),
            fields: MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

fn inst_field(v: MbValue, key: &str) -> Option<MbValue> {
    let ptr = v.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(key).copied()
        } else {
            None
        }
    }
}

fn inst_set_field(v: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.write().unwrap().insert(key.to_string(), val);
            }
        }
    }
}

fn instance_class(v: MbValue) -> Option<String> {
    let ptr = v.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            Some(class_name.clone())
        } else {
            None
        }
    }
}

/// The `Token.MISSING` sentinel — a unique Instance compared by identity.
pub fn missing_sentinel() -> MbValue {
    MISSING.with(|m| {
        let mut slot = m.borrow_mut();
        if let Some(v) = *slot {
            return v;
        }
        let v = new_instance("Token.MISSING", InstanceFields::default());
        *slot = Some(v);
        v
    })
}

// ── ContextVar ────────────────────────────────────────────────────

/// `ContextVar(name, *, default=...)` constructor. The keyword form lowers
/// to a trailing kwargs dict.
unsafe extern "C" fn dispatch_contextvar_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let name = a.first().copied().unwrap_or_else(MbValue::none);
    let mut default = None;
    for v in a.iter().skip(1) {
        if let Some(ptr) = v.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    use super::super::dict_ops::DictKey;
                    if let Some(d) = lock
                        .read()
                        .unwrap()
                        .get(&DictKey::Str("default".to_string()))
                        .copied()
                    {
                        default = Some(d);
                    }
                    continue;
                }
            }
        }
        // Positional default (CPython requires keyword; tolerate positional).
        default = Some(*v);
    }
    let mut fields = InstanceFields::default();
    fields.insert("name".to_string(), name);
    fields.insert(
        "_var_id".to_string(),
        MbValue::from_int(NEXT_VAR_ID.fetch_add(1, Ordering::Relaxed) as i64),
    );
    if let Some(d) = default {
        unsafe { super::super::rc::retain_if_ptr(d) };
        fields.insert("_default".to_string(), d);
        fields.insert("_has_default".to_string(), MbValue::from_bool(true));
    } else {
        fields.insert("_has_default".to_string(), MbValue::from_bool(false));
    }
    new_instance("ContextVar", fields)
}

fn var_id(var: MbValue) -> u64 {
    inst_field(var, "_var_id").and_then(|v| v.as_int()).unwrap_or(0) as u64
}

/// `var.get()` / `var.get(default)`.
pub fn mb_contextvar_get(var: MbValue, default: Option<MbValue>) -> MbValue {
    let id = var_id(var);
    if let Some(v) = CURRENT.with(|c| c.borrow().get(&id).copied()) {
        return v;
    }
    if let Some(d) = default {
        return d;
    }
    if inst_field(var, "_has_default").and_then(|v| v.as_bool()) == Some(true) {
        return inst_field(var, "_default").unwrap_or(MbValue::none());
    }
    let name = inst_field(var, "name")
        .and_then(extract_str)
        .unwrap_or_default();
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("LookupError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!("<ContextVar name={name:?}>"))),
    );
    MbValue::none()
}

/// `var.set(value)` → Token recording the previous value.
pub fn mb_contextvar_set(var: MbValue, value: MbValue) -> MbValue {
    let id = var_id(var);
    let old = CURRENT.with(|c| {
        let mut map = c.borrow_mut();
        unsafe { super::super::rc::retain_if_ptr(value) };
        map.insert(id, value)
    });
    let mut fields = InstanceFields::default();
    unsafe { super::super::rc::retain_if_ptr(var) };
    fields.insert("var".to_string(), var);
    fields.insert(
        "old_value".to_string(),
        old.unwrap_or_else(missing_sentinel),
    );
    fields.insert("_used".to_string(), MbValue::from_bool(false));
    new_instance("Token", fields)
}

/// `var.reset(token)` — restore the value recorded by the matching `set`.
pub fn mb_contextvar_reset(var: MbValue, token: MbValue) -> MbValue {
    if instance_class(token).as_deref() != Some("Token") {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "expected an instance of Token".to_string(),
            )),
        );
        return MbValue::none();
    }
    let token_var = inst_field(token, "var").unwrap_or(MbValue::none());
    if token_var != var {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "Token was created by a different ContextVar".to_string(),
            )),
        );
        return MbValue::none();
    }
    if inst_field(token, "_used").and_then(|v| v.as_bool()) == Some(true) {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("RuntimeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "Token has already been used once".to_string(),
            )),
        );
        return MbValue::none();
    }
    let id = var_id(var);
    let old = inst_field(token, "old_value").unwrap_or_else(missing_sentinel);
    CURRENT.with(|c| {
        let mut map = c.borrow_mut();
        if old == missing_sentinel() {
            map.remove(&id);
        } else {
            map.insert(id, old);
        }
    });
    inst_set_field(token, "_used", MbValue::from_bool(true));
    MbValue::none()
}

// ── Context ───────────────────────────────────────────────────────

/// Snapshot of the current thread context as a `Context` Instance. The
/// snapshot keeps the var ids and values in parallel `_ids` / `_vals`
/// list fields (MbValue lists keep the GC aware of the held values).
pub fn mb_contextvars_copy_context() -> MbValue {
    let (ids, vals): (Vec<MbValue>, Vec<MbValue>) = CURRENT.with(|c| {
        let map = c.borrow();
        let mut ids = Vec::with_capacity(map.len());
        let mut vals = Vec::with_capacity(map.len());
        for (id, v) in map.iter() {
            ids.push(MbValue::from_int(*id as i64));
            unsafe { super::super::rc::retain_if_ptr(*v) };
            vals.push(*v);
        }
        (ids, vals)
    });
    let mut fields = InstanceFields::default();
    fields.insert("_ids".to_string(), MbValue::from_ptr(MbObject::new_list(ids)));
    fields.insert("_vals".to_string(), MbValue::from_ptr(MbObject::new_list(vals)));
    new_instance("Context", fields)
}

fn list_items(v: MbValue) -> Vec<MbValue> {
    v.as_ptr()
        .and_then(|p| unsafe {
            if let ObjData::List(ref lock) = (*p).data {
                Some(lock.read().unwrap().iter().copied().collect())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

/// `ctx.run(callable, *args)` — execute under the snapshot, capture writes
/// back into the Context, restore the caller's context.
pub fn mb_context_run(ctx: MbValue, func: MbValue, args: Vec<MbValue>) -> MbValue {
    // Install the snapshot.
    let snapshot: FxHashMap<u64, MbValue> = {
        let ids = list_items(inst_field(ctx, "_ids").unwrap_or(MbValue::none()));
        let vals = list_items(inst_field(ctx, "_vals").unwrap_or(MbValue::none()));
        ids.iter()
            .zip(vals.iter())
            .filter_map(|(id, v)| id.as_int().map(|i| (i as u64, *v)))
            .collect()
    };
    let saved = CURRENT.with(|c| std::mem::replace(&mut *c.borrow_mut(), snapshot));

    let args_list = MbValue::from_ptr(MbObject::new_list(args));
    let result = super::super::builtins::mb_call_spread(func, args_list);

    // Capture writes back into the Context, then restore the caller's map.
    let finished = CURRENT.with(|c| std::mem::replace(&mut *c.borrow_mut(), saved));
    let mut ids = Vec::with_capacity(finished.len());
    let mut vals = Vec::with_capacity(finished.len());
    for (id, v) in finished.iter() {
        ids.push(MbValue::from_int(*id as i64));
        vals.push(*v);
    }
    inst_set_field(ctx, "_ids", MbValue::from_ptr(MbObject::new_list(ids)));
    inst_set_field(ctx, "_vals", MbValue::from_ptr(MbObject::new_list(vals)));
    result
}

/// `ctx.copy()` — a new Context with the same snapshot.
pub fn mb_context_copy(ctx: MbValue) -> MbValue {
    let ids = list_items(inst_field(ctx, "_ids").unwrap_or(MbValue::none()));
    let vals = list_items(inst_field(ctx, "_vals").unwrap_or(MbValue::none()));
    for v in &vals {
        unsafe { super::super::rc::retain_if_ptr(*v) };
    }
    let mut fields = InstanceFields::default();
    fields.insert("_ids".to_string(), MbValue::from_ptr(MbObject::new_list(ids)));
    fields.insert("_vals".to_string(), MbValue::from_ptr(MbObject::new_list(vals)));
    new_instance("Context", fields)
}

// ── Module registration ───────────────────────────────────────────

unsafe extern "C" fn dispatch_copy_context(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    mb_contextvars_copy_context()
}

/// Record a native function address so `callable(...)` resolves it as a
/// function. Mirrors the per-name registration in `register`.
fn register_native_addr(addr: usize) {
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr as u64);
    });
}

/// Build a class-like type-singleton object: Instance with
/// `class_name="type"`, a `__name__` field, plus any extra fields.
/// `callable(...)` returns True for a `class_name="type"` Instance and
/// `isinstance` resolves the `__name__`.
fn make_class_object(name: &str, extra: &[(&str, MbValue)]) -> MbValue {
    let mut fields = InstanceFields::default();
    fields.insert(
        "__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );
    for (k, v) in extra {
        fields.insert((*k).to_string(), *v);
    }
    new_instance("type", fields)
}

/// First positional element of a method-call args list (the variadic method
/// ABI delivers args as a list).
fn method_arg0(args: MbValue) -> Option<MbValue> {
    args.as_ptr().and_then(|p| unsafe {
        if let ObjData::List(ref lk) = (*p).data {
            lk.read().unwrap().first().copied()
        } else {
            None
        }
    })
}

// Variadic instance-method wrappers (ABI: `fn(self_v, args_list)`) so that the
// unbound forms `ContextVar.get`/`set`/`reset` resolve to callable methods via
// the func->native-class bridge. They produce the same result as the inline
// `class.rs` ContextVar dispatch, so registering the class is safe regardless
// of which path instance calls (`cv.get()`) take.
unsafe extern "C" fn cv_method_get(self_v: MbValue, args: MbValue) -> MbValue {
    mb_contextvar_get(self_v, method_arg0(args))
}
unsafe extern "C" fn cv_method_set(self_v: MbValue, args: MbValue) -> MbValue {
    mb_contextvar_set(self_v, method_arg0(args).unwrap_or_else(MbValue::none))
}
unsafe extern "C" fn cv_method_reset(self_v: MbValue, args: MbValue) -> MbValue {
    mb_contextvar_reset(self_v, method_arg0(args).unwrap_or_else(MbValue::none))
}

// Native dispatchers exposed as callable attributes on the `Context` type
// object so `Context.run` / `Context.copy` resolve to callables (and work as
// unbound forms `Context.run(ctx, fn)`).
unsafe extern "C" fn dispatch_context_run(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let ctx = a.first().copied().unwrap_or_else(MbValue::none);
    let func = a.get(1).copied().unwrap_or_else(MbValue::none);
    let rest = a.get(2..).map(|s| s.to_vec()).unwrap_or_default();
    mb_context_run(ctx, func, rest)
}
unsafe extern "C" fn dispatch_context_copy(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    mb_context_copy(a.first().copied().unwrap_or_else(MbValue::none))
}

/// Register the contextvars module.
pub fn register() {
    let mut attrs = HashMap::new();

    let plain: Vec<(&str, usize)> = vec![
        ("copy_context", dispatch_copy_context as usize),
        ("ContextVar", dispatch_contextvar_new as usize),
    ];
    for (name, addr) in plain {
        register_native_addr(addr);
        attrs.insert(name.to_string(), MbValue::from_func(addr));
    }
    // isinstance(cv, contextvars.ContextVar) — bind the constructor addr to
    // the instance class name.
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        m.borrow_mut().insert(
            dispatch_contextvar_new as usize as u64,
            "ContextVar".to_string(),
        );
    });
    // Register the ContextVar class with its instance methods so the unbound
    // forms `ContextVar.get`/`set`/`reset` resolve to callable methods (the
    // func->native-class bridge does lookup_method against this table).
    let mut cv_methods: HashMap<String, MbValue> = HashMap::new();
    for (name, addr) in [
        ("get", cv_method_get as usize),
        ("set", cv_method_set as usize),
        ("reset", cv_method_reset as usize),
    ] {
        super::super::module::register_variadic_func(addr as u64);
        cv_methods.insert(name.to_string(), MbValue::from_func(addr));
    }
    super::super::class::mb_class_register("ContextVar", vec![], cv_methods);

    // `Token` is a type-singleton exposing the MISSING sentinel;
    // `Context` is a type-singleton so isinstance(ctx, contextvars.Context)
    // resolves via `__name__`.
    attrs.insert(
        "Token".to_string(),
        make_class_object("Token", &[("MISSING", missing_sentinel())]),
    );
    let ctx_run_addr = dispatch_context_run as usize;
    let ctx_copy_addr = dispatch_context_copy as usize;
    register_native_addr(ctx_run_addr);
    register_native_addr(ctx_copy_addr);
    attrs.insert(
        "Context".to_string(),
        make_class_object(
            "Context",
            &[
                ("run", MbValue::from_func(ctx_run_addr)),
                ("copy", MbValue::from_func(ctx_copy_addr)),
            ],
        ),
    );

    super::register_module("contextvars", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn make_var(name: &str, default: Option<MbValue>) -> MbValue {
        let mut args = vec![s(name)];
        if let Some(d) = default {
            args.push(d);
        }
        unsafe { dispatch_contextvar_new(args.as_ptr(), args.len()) }
    }

    #[test]
    fn test_name_attribute_and_default() {
        let var = make_var("my_var", Some(MbValue::from_int(42)));
        assert_eq!(
            inst_field(var, "name").and_then(extract_str).as_deref(),
            Some("my_var"),
        );
        assert_eq!(mb_contextvar_get(var, None).as_int(), Some(42));
    }

    #[test]
    fn test_set_get_reset_roundtrip() {
        let var = make_var("rt", None);
        let token = mb_contextvar_set(var, MbValue::from_int(7));
        assert_eq!(mb_contextvar_get(var, None).as_int(), Some(7));
        assert_eq!(
            inst_field(token, "old_value"),
            Some(missing_sentinel()),
            "first set records MISSING",
        );
        mb_contextvar_reset(var, token);
        // Unset again: get with explicit default returns it.
        assert_eq!(
            mb_contextvar_get(var, Some(MbValue::from_int(9))).as_int(),
            Some(9),
        );
    }

    #[test]
    fn test_reset_wrong_var_raises_valueerror() {
        let a = make_var("a", None);
        let b = make_var("b", None);
        let token = mb_contextvar_set(a, MbValue::from_int(1));
        mb_contextvar_reset(b, token);
        assert_eq!(
            crate::runtime::exception::current_exception_type().as_deref(),
            Some("ValueError"),
        );
        crate::runtime::exception::mb_clear_exception();
    }

    #[test]
    fn test_copy_context_snapshots() {
        let var = make_var("snap", None);
        mb_contextvar_set(var, s("captured"));
        let ctx = mb_contextvars_copy_context();
        assert_eq!(instance_class(ctx).as_deref(), Some("Context"));
        let ids = list_items(inst_field(ctx, "_ids").unwrap());
        assert!(ids.iter().any(|v| v.as_int() == Some(var_id(var) as i64)));
    }
}
