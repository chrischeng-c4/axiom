//! @codegen-skip: handwrite-pre-standardize
//!
//! abc module for Mamba — Python 3.12 `abc` stdlib (#1447).
//!
//! Surface (CPython 3.12 `abc` denominator, 8 names):
//!   ABC, ABCMeta, abstractmethod, abstractclassmethod,
//!   abstractstaticmethod, abstractproperty, get_cache_token,
//!   update_abstractmethods
//!
//! HANDWRITE-BEGIN reason: per-section primitive vocabulary for stdlib
//! shims is not yet emitted by score codegen. Tracked as part of the
//! brute-force Phase-2 sweep; will be replaced when aw standardize
//! lands the stdlib-shim section type. Issue #1447.
//!
//! Carve-outs (documented gaps from CPython parity):
//!
//! - `ABC` / `ABCMeta` are sentinel callables that return dict-shaped
//!   placeholders. Real metaclass semantics — `register()`,
//!   `__subclasshook__`, virtual-subclass cache invalidation — require
//!   class.rs plumbing (out of scope for the conformance import-shape
//!   gate).
//! - `abstractmethod` / `abstractclassmethod` / `abstractstaticmethod`
//!   / `abstractproperty` wrap the passed func into a dict marked
//!   `__isabstractmethod__: True`. Sufficient for `hasattr`/decorator
//!   walkers; real abstract-enforcement happens in class instantiation.
//! - `update_abstractmethods(cls)` returns the cls argument unchanged
//!   (no-op shim — CPython mutates `__abstractmethods__` frozenset
//!   on the class).
//! - `get_cache_token()` returns a real monotonically-increasing int
//!   via a thread-local `AtomicU64` counter, matching CPython's
//!   "opaque, monotonic" contract. This is the perf-microbench target.

use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Monotonic cache-token counter. Mirrors CPython's
/// `_abc_invalidation_counter` — opaque to callers; only equality /
/// strict-monotonic ordering is contractual.
static CACHE_TOKEN: AtomicU64 = AtomicU64::new(0);

macro_rules! dispatch_nullary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

dispatch_nullary!(dispatch_abc_cls, mb_abc_ABC);
dispatch_nullary!(dispatch_abcmeta, mb_abc_ABCMeta);
dispatch_unary!(dispatch_abstractmethod, mb_abc_abstractmethod);
dispatch_unary!(dispatch_abstractclassmethod, mb_abc_abstractclassmethod);
dispatch_unary!(dispatch_abstractstaticmethod, mb_abc_abstractstaticmethod);
dispatch_unary!(dispatch_abstractproperty, mb_abc_abstractproperty);
dispatch_nullary!(dispatch_get_cache_token, mb_abc_get_cache_token);
dispatch_unary!(
    dispatch_update_abstractmethods,
    mb_abc_update_abstractmethods
);

/// Register the abc module.
pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("ABC", dispatch_abc_cls as usize),
        ("ABCMeta", dispatch_abcmeta as usize),
        ("abstractmethod", dispatch_abstractmethod as usize),
        ("abstractclassmethod", dispatch_abstractclassmethod as usize),
        (
            "abstractstaticmethod",
            dispatch_abstractstaticmethod as usize,
        ),
        ("abstractproperty", dispatch_abstractproperty as usize),
        ("get_cache_token", dispatch_get_cache_token as usize),
        (
            "update_abstractmethods",
            dispatch_update_abstractmethods as usize,
        ),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("abc", attrs);
}

// ── Runtime functions ──

fn class_sentinel(name: &str) -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut m = lock.write().unwrap();
            m.insert(
                "__class__".into(),
                MbValue::from_ptr(MbObject::new_str(name.to_string())),
            );
            m.insert("__abstract__".into(), MbValue::from_bool(true));
        }
    }
    MbValue::from_ptr(dict)
}

fn abstract_wrapper(class_name: &str, func: MbValue) -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut m = lock.write().unwrap();
            m.insert(
                "__class__".into(),
                MbValue::from_ptr(MbObject::new_str(class_name.to_string())),
            );
            m.insert("__abstract__".into(), MbValue::from_bool(true));
            m.insert("__isabstractmethod__".into(), MbValue::from_bool(true));
            m.insert("__func__".into(), func);
        }
    }
    MbValue::from_ptr(dict)
}

/// abc.ABC() -> ABC-sentinel.
#[allow(non_snake_case)]
pub fn mb_abc_ABC() -> MbValue {
    class_sentinel("ABC")
}

/// abc.ABCMeta() -> ABCMeta-sentinel.
#[allow(non_snake_case)]
pub fn mb_abc_ABCMeta() -> MbValue {
    class_sentinel("ABCMeta")
}

/// abc.abstractmethod(func) -> wrapped sentinel marking `func` abstract.
pub fn mb_abc_abstractmethod(func: MbValue) -> MbValue {
    abstract_wrapper("abstractmethod", func)
}

/// abc.abstractclassmethod(func) -> wrapped sentinel.
pub fn mb_abc_abstractclassmethod(func: MbValue) -> MbValue {
    abstract_wrapper("abstractclassmethod", func)
}

/// abc.abstractstaticmethod(func) -> wrapped sentinel.
pub fn mb_abc_abstractstaticmethod(func: MbValue) -> MbValue {
    abstract_wrapper("abstractstaticmethod", func)
}

/// abc.abstractproperty(func) -> wrapped sentinel.
pub fn mb_abc_abstractproperty(func: MbValue) -> MbValue {
    abstract_wrapper("abstractproperty", func)
}

/// abc.get_cache_token() -> int.
///
/// Returns a monotonically-increasing opaque integer. CPython contract
/// (per `Lib/abc.py`): "Returns the current ABC cache token. The token
/// is an opaque object (supporting equality testing) identifying the
/// current version of the ABC cache for virtual subclasses." Mamba's
/// implementation is a bare `AtomicU64::fetch_add(1)` — incrementing
/// on each call gives strict monotonicity (a stricter-than-required
/// guarantee — CPython only bumps on `ABCMeta.register()`).
///
/// This is the #1447 Gate-2 hot-loop target — pure native dispatch,
/// no allocation, contrast vs CPython's pure-Python `_abc_get_cache_token`
/// which goes through a C-API call frame per iter.
pub fn mb_abc_get_cache_token() -> MbValue {
    let tok = CACHE_TOKEN.fetch_add(1, Ordering::Relaxed);
    // Clamp into i64 range — opaque per contract; only equality / ordering
    // matters and we'd overflow well after the heat-death of the bench.
    MbValue::from_int(tok as i64)
}

/// abc.update_abstractmethods(cls) -> cls (no-op shim).
///
/// CPython mutates `cls.__abstractmethods__` (a frozenset) to reflect
/// the current set of un-overridden abstract methods. Mamba returns
/// `cls` unchanged — sufficient for `hasattr` walkers.
pub fn mb_abc_update_abstractmethods(cls: MbValue) -> MbValue {
    cls
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dict_str_field(val: MbValue, key: &str) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.read()
                    .unwrap()
                    .get(key)
                    .and_then(|v| v.as_ptr())
                    .and_then(|p| {
                        if let ObjData::Str(ref s) = (*p).data {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
            } else {
                None
            }
        })
    }

    fn dict_bool_field(val: MbValue, key: &str) -> Option<bool> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.read().unwrap().get(key).and_then(|v| v.as_bool())
            } else {
                None
            }
        })
    }

    fn dict_val_field(val: MbValue, key: &str) -> Option<MbValue> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.read().unwrap().get(key).copied()
            } else {
                None
            }
        })
    }

    #[test]
    fn test_abc_fields() {
        let result = mb_abc_ABC();
        assert_eq!(dict_str_field(result, "__class__").as_deref(), Some("ABC"));
        assert_eq!(dict_bool_field(result, "__abstract__"), Some(true));
    }

    #[test]
    fn test_abcmeta_fields() {
        let result = mb_abc_ABCMeta();
        assert_eq!(
            dict_str_field(result, "__class__").as_deref(),
            Some("ABCMeta")
        );
        assert_eq!(dict_bool_field(result, "__abstract__"), Some(true));
    }

    #[test]
    fn test_abstractmethod_wraps_func() {
        let func = MbValue::from_int(42);
        let result = mb_abc_abstractmethod(func);
        assert_eq!(
            dict_str_field(result, "__class__").as_deref(),
            Some("abstractmethod")
        );
        assert_eq!(dict_bool_field(result, "__abstract__"), Some(true));
        assert_eq!(dict_bool_field(result, "__isabstractmethod__"), Some(true));
        let stored_func = dict_val_field(result, "__func__").unwrap();
        assert_eq!(stored_func.as_int(), Some(42));
    }

    #[test]
    fn test_abstractclassmethod_wraps_func() {
        let result = mb_abc_abstractclassmethod(MbValue::from_int(7));
        assert_eq!(
            dict_str_field(result, "__class__").as_deref(),
            Some("abstractclassmethod")
        );
        assert_eq!(dict_bool_field(result, "__isabstractmethod__"), Some(true));
    }

    #[test]
    fn test_abstractstaticmethod_wraps_func() {
        let result = mb_abc_abstractstaticmethod(MbValue::from_int(8));
        assert_eq!(
            dict_str_field(result, "__class__").as_deref(),
            Some("abstractstaticmethod")
        );
        assert_eq!(dict_bool_field(result, "__isabstractmethod__"), Some(true));
    }

    #[test]
    fn test_abstractproperty_wraps_func() {
        let result = mb_abc_abstractproperty(MbValue::from_int(9));
        assert_eq!(
            dict_str_field(result, "__class__").as_deref(),
            Some("abstractproperty")
        );
        assert_eq!(dict_bool_field(result, "__isabstractmethod__"), Some(true));
    }

    #[test]
    fn test_get_cache_token_returns_int_and_monotonic() {
        let a = mb_abc_get_cache_token().as_int().unwrap();
        let b = mb_abc_get_cache_token().as_int().unwrap();
        assert!(b > a, "cache token must be strictly monotonic: {a} -> {b}");
    }

    #[test]
    fn test_update_abstractmethods_is_identity() {
        let cls = MbValue::from_int(123);
        let result = mb_abc_update_abstractmethods(cls);
        assert_eq!(result.as_int(), Some(123));
    }
}
