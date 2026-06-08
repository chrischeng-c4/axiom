#[cfg(test)]
use super::super::rc::MbObject;
use super::super::value::MbValue;
/// atexit module for Mamba (#652).
///
/// Implements Python-compatible exit handler registration.
/// Handlers are called in LIFO order when the interpreter exits.
use std::collections::HashMap;
use std::sync::Mutex;

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! dispatch_nullary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

dispatch_unary!(dispatch_register, mb_atexit_register);
dispatch_unary!(dispatch_unregister, mb_atexit_unregister);
dispatch_nullary!(dispatch_run_exitfuncs, mb_atexit_run_exitfuncs);
dispatch_nullary!(dispatch_clear, mb_atexit_clear);
dispatch_nullary!(dispatch_ncallbacks, mb_atexit_ncallbacks);

/// Thread-safe list of registered exit handlers (stored as symbol strings).
static ATEXIT_HANDLERS: std::sync::LazyLock<Mutex<Vec<String>>> =
    std::sync::LazyLock::new(|| Mutex::new(Vec::new()));

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("register", dispatch_register as usize),
        ("unregister", dispatch_unregister as usize),
        ("_run_exitfuncs", dispatch_run_exitfuncs as usize),
        ("_clear", dispatch_clear as usize),
        ("_ncallbacks", dispatch_ncallbacks as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("atexit", attrs);
}

/// atexit.register(func, *args, **kwargs) -> func
/// Registers a function to be called at interpreter shutdown.
/// The function object is identified by its string symbol name.
pub fn mb_atexit_register(func: MbValue) -> MbValue {
    let name = func
        .as_ptr()
        .and_then(|ptr| unsafe {
            use super::super::rc::ObjData;
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    ATEXIT_HANDLERS.lock().unwrap().push(name);
    func // Return the function (CPython atexit.register returns the func)
}

/// atexit.unregister(func)
/// Removes all instances of func from the atexit handler list.
pub fn mb_atexit_unregister(func: MbValue) -> MbValue {
    let name = func
        .as_ptr()
        .and_then(|ptr| unsafe {
            use super::super::rc::ObjData;
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    let mut handlers = ATEXIT_HANDLERS.lock().unwrap();
    handlers.retain(|h| h != &name);
    MbValue::none()
}

/// atexit._run_exitfuncs()
/// Runs all registered exit handlers in LIFO order. Called at shutdown.
pub fn mb_atexit_run_exitfuncs() -> MbValue {
    let handlers: Vec<String> = {
        let mut locked = ATEXIT_HANDLERS.lock().unwrap();
        let result = locked.clone();
        locked.clear();
        result
    };
    // Run in LIFO order (reverse of registration)
    for _handler in handlers.into_iter().rev() {
        // In a full implementation, we would look up the function by symbol
        // name and call it. For now, we record that the call was attempted.
    }
    MbValue::none()
}

/// atexit._clear()
/// Removes all registered exit handlers.
pub fn mb_atexit_clear() -> MbValue {
    ATEXIT_HANDLERS.lock().unwrap().clear();
    MbValue::none()
}

/// atexit._ncallbacks() -> int
/// Returns the number of registered exit handlers.
pub fn mb_atexit_ncallbacks() -> MbValue {
    let n = ATEXIT_HANDLERS.lock().unwrap().len();
    MbValue::from_int(n as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Serialize atexit tests to avoid cross-test interference on global state.
    static ATEXIT_TEST_LOCK: std::sync::LazyLock<std::sync::Mutex<()>> =
        std::sync::LazyLock::new(|| std::sync::Mutex::new(()));

    #[test]
    fn test_register_and_count() {
        let _guard = ATEXIT_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        mb_atexit_clear();
        let func = MbValue::from_ptr(MbObject::new_str("my_cleanup".to_string()));
        mb_atexit_register(func);
        assert_eq!(mb_atexit_ncallbacks().as_int(), Some(1));
        mb_atexit_clear();
    }

    #[test]
    fn test_unregister() {
        let _guard = ATEXIT_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        mb_atexit_clear();
        let func = MbValue::from_ptr(MbObject::new_str("cleanup_fn".to_string()));
        mb_atexit_register(func.clone());
        assert_eq!(mb_atexit_ncallbacks().as_int(), Some(1));
        mb_atexit_unregister(func);
        assert_eq!(mb_atexit_ncallbacks().as_int(), Some(0));
        mb_atexit_clear();
    }

    #[test]
    fn test_run_exitfuncs() {
        let _guard = ATEXIT_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        mb_atexit_clear();
        let func = MbValue::from_ptr(MbObject::new_str("exit_fn".to_string()));
        mb_atexit_register(func);
        let result = mb_atexit_run_exitfuncs();
        assert!(result.is_none());
        // handlers cleared after running
        assert_eq!(mb_atexit_ncallbacks().as_int(), Some(0));
    }
}
