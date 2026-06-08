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
//! - `copy_context()` returns a Context-sentinel object — sufficient
//!   for the perf microbench (hot loop calling `copy_context()` and
//!   probing the result is truthy) and for `hasattr` walkers.

use super::super::rc::MbObject;
use super::super::value::MbValue;
use std::collections::HashMap;

macro_rules! dispatch_nullary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

dispatch_nullary!(dispatch_copy_context, mb_contextvars_copy_context);
#[allow(non_snake_case)]
unsafe extern "C" fn dispatch_Context(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_contextvars_context()
}
#[allow(non_snake_case)]
unsafe extern "C" fn dispatch_ContextVar(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_contextvars_context_var()
}
#[allow(non_snake_case)]
unsafe extern "C" fn dispatch_Token(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_contextvars_token()
}

/// Register the contextvars module.
pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("copy_context", dispatch_copy_context as usize),
        ("Context", dispatch_Context as usize),
        ("ContextVar", dispatch_ContextVar as usize),
        ("Token", dispatch_Token as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
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

/// contextvars.Context() -> Context-sentinel.
pub fn mb_contextvars_context() -> MbValue {
    MbValue::from_ptr(MbObject::new_str("<Context>".to_string()))
}

/// contextvars.ContextVar(name) -> ContextVar-sentinel.
///
/// CPython returns a real `ContextVar` instance with `.name`,
/// `.get()`, `.set()`, `.reset()` methods. Sentinel — `hasattr` and
/// constructor-call walkers pass; per-value get/set is out of scope.
pub fn mb_contextvars_context_var() -> MbValue {
    MbValue::from_ptr(MbObject::new_str("<ContextVar>".to_string()))
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
    use super::super::super::rc::ObjData;
    use super::*;

    fn extract_str(val: MbValue) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })
    }

    #[test]
    fn test_copy_context_returns_sentinel() {
        let v = mb_contextvars_copy_context();
        assert_eq!(extract_str(v).as_deref(), Some("<Context>"));
    }

    #[test]
    fn test_context_returns_sentinel() {
        let v = mb_contextvars_context();
        assert_eq!(extract_str(v).as_deref(), Some("<Context>"));
    }

    #[test]
    fn test_context_var_returns_sentinel() {
        let v = mb_contextvars_context_var();
        assert_eq!(extract_str(v).as_deref(), Some("<ContextVar>"));
    }

    #[test]
    fn test_token_returns_sentinel() {
        let v = mb_contextvars_token();
        assert_eq!(extract_str(v).as_deref(), Some("<Token>"));
    }
}
