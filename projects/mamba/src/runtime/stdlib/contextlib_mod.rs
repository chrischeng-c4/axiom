use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// contextlib module for Mamba (#413).
///
/// Provides: contextlib.suppress(*exceptions), contextlib.nullcontext(value),
///           contextlib.contextmanager (stub marker).
/// Suppress creates a dict describing which exception types to suppress;
/// actual suppression is handled at runtime by with-statement codegen.
use std::collections::HashMap;

/// Helper: extract a string from an MbValue.
#[allow(dead_code)]
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
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

disp_unary!(d_suppress, mb_contextlib_suppress);
disp_unary!(d_nullcontext, mb_contextlib_nullcontext);
disp_unary!(d_contextmanager, mb_contextlib_contextmanager);

unsafe extern "C" fn d_noop(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
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
    make_exit_stack_dict()
}

/// Register the contextlib module.
pub fn register() {
    let mut attrs = HashMap::new();

    let noop = d_noop as usize;
    let identity = d_identity as usize;
    let dispatchers: Vec<(&str, usize)> = vec![
        ("suppress", d_suppress as *const () as usize),
        ("nullcontext", d_nullcontext as *const () as usize),
        ("contextmanager", d_contextmanager as *const () as usize),
        ("redirect_stdout", noop),
        ("redirect_stderr", noop),
        ("closing", identity),
        ("ExitStack", d_exit_stack_ctor as usize),
        ("AsyncExitStack", d_exit_stack_ctor as usize),
        ("asynccontextmanager", identity),
        ("AbstractContextManager", noop),
        ("AbstractAsyncContextManager", noop),
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

    super::register_module("contextlib", attrs);
}

/// contextlib.suppress(*exceptions) -> dict representing a suppress context manager.
///
/// Stores the exception types list under "_type" = "suppress" and "_exceptions"
/// as a list of exception type name strings. Actual suppression is handled by
/// the with-statement codegen checking exception type against this list.
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
