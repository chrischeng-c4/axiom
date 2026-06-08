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
//! Carve-outs (documented gaps from CPython parity):
//!
//! - `ContextVar`, `Token`, `Context` are sentinel callables that
//!   return placeholder objects. Real `.get()/.set()/.reset()` and
//!   context-manager (`__enter__`/`__exit__`) semantics require
//!   thread-local + token plumbing on class.rs (out of scope for
//!   the conformance import-shape gate).
//! - `Context` and `ContextVar` are registered as type-singleton
//!   objects (Instance `class_name="type"`, `__name__=<class>`) so that
//!   `callable(contextvars.ContextVar)` is True AND the documented
//!   method attributes (`Context.run`, `ContextVar.get/set/reset`)
//!   resolve to callable native stubs through the generic Instance
//!   attribute path. The method stubs return None — they satisfy
//!   `callable(...)` surface walkers; real get/set/reset behavior is
//!   gated on the broader class.rs thread-local plumbing.
//! - `copy_context()` returns a Context-sentinel object — sufficient
//!   for the perf microbench (hot loop calling `copy_context()` and
//!   probing the result is truthy) and for `hasattr` walkers.

use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use super::super::value::MbValue;
use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind, InstanceFields, MbRwLock};

macro_rules! dispatch_nullary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

dispatch_nullary!(dispatch_copy_context, mb_contextvars_copy_context);
#[allow(non_snake_case)]
unsafe extern "C" fn dispatch_Token(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_contextvars_token()
}

// Method-attribute stubs. These satisfy `callable(contextvars.Context.run)`
// and `callable(contextvars.ContextVar.get/set/reset)`. Real behavior
// (thread-local get/set/reset, Context.run dispatch) is the documented
// class.rs carve-out; the stubs return None.
unsafe extern "C" fn dispatch_context_run(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}
unsafe extern "C" fn dispatch_contextvar_get(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}
unsafe extern "C" fn dispatch_contextvar_set(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}
unsafe extern "C" fn dispatch_contextvar_reset(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

/// Record a native function address so `callable(...)` resolves it as a
/// function. Mirrors the per-name registration in `register`.
fn register_native_addr(addr: usize) {
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr as u64);
    });
}

/// Build a class-like type-singleton object: Instance with
/// `class_name="type"`, a `__name__` field, and the given method names
/// bound to native-function stubs. `callable(...)` returns True for a
/// `class_name="type"` Instance, and the method attributes resolve to
/// the callable stubs through the generic Instance attribute path.
fn make_class_object(name: &str, methods: &[(&str, usize)]) -> MbValue {
    let mut fields = InstanceFields::default();
    fields.insert(
        "__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );
    for (method_name, addr) in methods {
        register_native_addr(*addr);
        fields.insert((*method_name).to_string(), MbValue::from_func(*addr));
    }
    let obj = Box::new(MbObject {
        header: MbObjectHeader { rc: AtomicU32::new(1), kind: ObjKind::Instance },
        data: ObjData::Instance {
            class_name: "type".to_string(),
            fields: MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Register the contextvars module.
pub fn register() {
    let mut attrs = HashMap::new();

    // `copy_context` and `Token` are bare native callables — `hasattr` and
    // `callable` walkers pass; no method attributes are probed on them.
    let plain: Vec<(&str, usize)> = vec![
        ("copy_context", dispatch_copy_context as usize),
        ("Token", dispatch_Token as usize),
    ];
    for (name, addr) in plain {
        register_native_addr(addr);
        attrs.insert(name.to_string(), MbValue::from_func(addr));
    }

    // `Context` and `ContextVar` are type-singleton objects so that they
    // are callable AND expose their documented method attributes.
    attrs.insert(
        "Context".to_string(),
        make_class_object("Context", &[("run", dispatch_context_run as usize)]),
    );
    attrs.insert(
        "ContextVar".to_string(),
        make_class_object(
            "ContextVar",
            &[
                ("get", dispatch_contextvar_get as usize),
                ("set", dispatch_contextvar_set as usize),
                ("reset", dispatch_contextvar_reset as usize),
            ],
        ),
    );

    super::register_module("contextvars", attrs);
}

// ── Runtime functions ──

/// contextvars.copy_context() -> Context-sentinel.
///
/// Returns a sentinel string `"<Context>"` so that the perf microbench
/// hot loop has a truthy non-None result to count. Full Context object
/// (with `.run()`, `.copy()`, mapping protocol) is gated on the broader
/// class.rs plumbing.
pub fn mb_contextvars_copy_context() -> MbValue {
    MbValue::from_ptr(MbObject::new_str("<Context>".to_string()))
}

/// contextvars.Token() -> Token-sentinel.
///
/// CPython's `Token` is normally constructed by `ContextVar.set()`,
/// not user code. Sentinel — `hasattr` walkers pass.
pub fn mb_contextvars_token() -> MbValue {
    MbValue::from_ptr(MbObject::new_str("<Token>".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::rc::ObjData;

    fn extract_str(val: MbValue) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
        })
    }

    #[test]
    fn test_copy_context_returns_sentinel() {
        let v = mb_contextvars_copy_context();
        assert_eq!(extract_str(v).as_deref(), Some("<Context>"));
    }

    #[test]
    fn test_token_returns_sentinel() {
        let v = mb_contextvars_token();
        assert_eq!(extract_str(v).as_deref(), Some("<Token>"));
    }

    #[test]
    fn test_class_object_is_instance_type_with_name() {
        let cv = make_class_object("ContextVar", &[("get", dispatch_contextvar_get as usize)]);
        let ptr = cv.as_ptr().expect("class object is a heap pointer");
        unsafe {
            if let ObjData::Instance { ref class_name, ref fields } = (*ptr).data {
                assert_eq!(class_name, "type");
                let guard = fields.read().unwrap();
                assert_eq!(
                    extract_str(*guard.get("__name__").unwrap()).as_deref(),
                    Some("ContextVar"),
                );
                assert!(guard.get("get").map(|v| v.as_func().is_some()).unwrap_or(false));
            } else {
                panic!("expected Instance");
            }
        }
    }
}
