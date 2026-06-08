/// atexit module for Mamba (#652).
///
/// Implements Python-compatible exit handler registration.
/// Handlers are stored as the actual callable plus the positional/keyword
/// arguments captured at `register()` time, and are invoked in LIFO order
/// when `_run_exitfuncs()` runs (at interpreter exit, or explicitly).
use std::cell::RefCell;
use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::ObjData;
#[cfg(test)]
use super::super::rc::MbObject;

macro_rules! dispatch_variadic {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a: &[MbValue] = if nargs == 0 {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
            };
            $fn(a)
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

dispatch_variadic!(dispatch_register, mb_atexit_register);
dispatch_variadic!(dispatch_unregister, mb_atexit_unregister);
dispatch_nullary!(dispatch_run_exitfuncs, mb_atexit_run_exitfuncs);
dispatch_nullary!(dispatch_clear, mb_atexit_clear);
dispatch_nullary!(dispatch_ncallbacks, mb_atexit_ncallbacks);

/// A single registered exit handler: the callable plus the arguments to
/// forward to it. `func`/`pos`/`kwargs` are NaN-boxed `MbValue`s; the heap
/// objects they reference are kept alive for the registry's lifetime by an
/// explicit retain on store (released on removal).
#[derive(Clone)]
struct Handler {
    func: MbValue,
    pos: Vec<MbValue>,
    kwargs: Option<MbValue>,
}

thread_local! {
    /// Registered exit handlers, in registration order (index 0 = first
    /// registered). `_run_exitfuncs` walks this in reverse for LIFO order.
    static ATEXIT_HANDLERS: RefCell<Vec<Handler>> = const { RefCell::new(Vec::new()) };
}

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

/// Split a native-dispatcher arg slice into positional args and a trailing
/// keyword dict. Mamba lowers `f(a, k=v)` to a flat arg slice `[a, {"k": v}]`,
/// so a trailing `dict` value is treated as captured keyword arguments.
fn split_kwargs(args: &[MbValue]) -> (Vec<MbValue>, Option<MbValue>) {
    if let Some(last) = args.last() {
        let is_dict = last
            .as_ptr()
            .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
            .unwrap_or(false);
        if is_dict {
            return (args[..args.len() - 1].to_vec(), Some(*last));
        }
    }
    (args.to_vec(), None)
}

/// True when two callables refer to the same underlying object/function.
/// CPython's `unregister` matches by `==`; for the plain functions and bound
/// methods exercised here, NaN-box bit equality is the correct identity.
fn same_callable(a: MbValue, b: MbValue) -> bool {
    a.to_bits() == b.to_bits()
}

/// atexit.register(func, *args, **kwargs) -> func
/// Records `func` and its captured arguments; returns `func` unchanged so it
/// can be used as a decorator (CPython contract).
pub fn mb_atexit_register(args: &[MbValue]) -> MbValue {
    let func = args.first().copied().unwrap_or_else(MbValue::none);
    let (pos, kwargs) = split_kwargs(&args[1.min(args.len())..]);

    // Keep the callable and its forwarded arguments alive for as long as the
    // registry holds them.
    unsafe {
        super::super::rc::retain_if_ptr(func);
        for v in &pos {
            super::super::rc::retain_if_ptr(*v);
        }
        if let Some(kw) = kwargs {
            super::super::rc::retain_if_ptr(kw);
        }
    }

    ATEXIT_HANDLERS.with(|h| {
        h.borrow_mut().push(Handler { func, pos, kwargs });
    });
    func
}

/// Release the heap references a handler kept alive.
fn release_handler(h: &Handler) {
    unsafe {
        super::super::rc::release_if_ptr(h.func);
        for v in &h.pos {
            super::super::rc::release_if_ptr(*v);
        }
        if let Some(kw) = h.kwargs {
            super::super::rc::release_if_ptr(kw);
        }
    }
}

/// atexit.unregister(func)
/// Removes every registration whose callable matches `func` by identity.
/// Unregistering a callable that was never registered is a silent no-op.
pub fn mb_atexit_unregister(args: &[MbValue]) -> MbValue {
    let func = args.first().copied().unwrap_or_else(MbValue::none);
    ATEXIT_HANDLERS.with(|h| {
        let mut handlers = h.borrow_mut();
        let mut kept = Vec::with_capacity(handlers.len());
        for handler in handlers.drain(..) {
            if same_callable(handler.func, func) {
                release_handler(&handler);
            } else {
                kept.push(handler);
            }
        }
        *handlers = kept;
    });
    MbValue::none()
}

/// Invoke a single handler's callable with its captured args/kwargs.
///
/// When the callee is a user-defined `*args`/`**kwargs` function we dispatch
/// its packed `(args_tuple, kwargs_dict)` ABI directly: this both forwards the
/// captured keyword arguments faithfully and presents `*args` as a tuple
/// (matching CPython) rather than the list `mb_call_spread` would build.
fn call_handler(h: &Handler) {
    if let Some(addr) = super::super::builtins::resolve_callable_pub(h.func) {
        let is_native = super::super::module::is_native_func(addr as u64);
        let has_star = super::super::module::is_variadic_func(addr as u64);
        let has_kwargs = super::super::module::is_kwargs_func(addr as u64);
        if !is_native && (has_star || has_kwargs) {
            let kwargs_dict = h.kwargs.unwrap_or_else(|| {
                MbValue::from_ptr(super::super::rc::MbObject::new_dict())
            });
            unsafe {
                match (has_star, has_kwargs) {
                    (true, true) => {
                        let args_tuple = MbValue::from_ptr(
                            super::super::rc::MbObject::new_tuple(h.pos.clone()),
                        );
                        let f: extern "C" fn(MbValue, MbValue) -> MbValue =
                            std::mem::transmute(addr);
                        f(args_tuple, kwargs_dict);
                    }
                    (true, false) => {
                        let args_tuple = MbValue::from_ptr(
                            super::super::rc::MbObject::new_tuple(h.pos.clone()),
                        );
                        let f: extern "C" fn(MbValue) -> MbValue =
                            std::mem::transmute(addr);
                        f(args_tuple);
                    }
                    (false, true) => {
                        let f: extern "C" fn(MbValue) -> MbValue =
                            std::mem::transmute(addr);
                        f(kwargs_dict);
                    }
                    (false, false) => unreachable!(),
                }
            }
            return;
        }
    }

    // Non-variadic / native callees: spread the positional arguments (with any
    // captured kwargs dict appended) through the general call path.
    let mut items = h.pos.clone();
    if let Some(kw) = h.kwargs {
        items.push(kw);
    }
    let args_list = MbValue::from_ptr(super::super::rc::MbObject::new_list(items));
    super::super::builtins::mb_call_spread(h.func, args_list);
}

/// atexit._run_exitfuncs()
/// Runs all registered exit handlers in LIFO order, draining the queue. A
/// callback that raises does not abort the run: the exception is reported and
/// the remaining callbacks still execute (CPython semantics).
pub fn mb_atexit_run_exitfuncs() -> MbValue {
    // Drain the queue up front so the handlers run exactly once even if one of
    // them re-enters `_run_exitfuncs` (and so a second call is a no-op).
    let handlers: Vec<Handler> = ATEXIT_HANDLERS.with(|h| {
        let mut locked = h.borrow_mut();
        std::mem::take(&mut *locked)
    });

    for handler in handlers.iter().rev() {
        call_handler(handler);
        // Isolate a raising callback: report it, clear the pending exception,
        // and keep running the remaining handlers.
        if super::super::exception::mb_has_exception().as_bool() == Some(true) {
            if let Some(tb) = super::super::exception::mb_take_uncaught_traceback() {
                eprintln!("Error in atexit._run_exitfuncs:");
                eprintln!("{}", tb);
            }
            super::super::exception::mb_clear_exception();
        }
    }

    for handler in &handlers {
        release_handler(handler);
    }
    MbValue::none()
}

/// atexit._clear()
/// Removes all registered exit handlers.
pub fn mb_atexit_clear() -> MbValue {
    ATEXIT_HANDLERS.with(|h| {
        let drained: Vec<Handler> = std::mem::take(&mut *h.borrow_mut());
        for handler in &drained {
            release_handler(handler);
        }
    });
    MbValue::none()
}

/// atexit._ncallbacks() -> int
/// Returns the number of registered exit handlers.
pub fn mb_atexit_ncallbacks() -> MbValue {
    let n = ATEXIT_HANDLERS.with(|h| h.borrow().len());
    MbValue::from_int(n as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_count() {
        mb_atexit_clear();
        let func = MbValue::from_ptr(MbObject::new_str("my_cleanup".to_string()));
        mb_atexit_register(&[func]);
        assert_eq!(mb_atexit_ncallbacks().as_int(), Some(1));
        mb_atexit_clear();
    }

    #[test]
    fn test_unregister() {
        mb_atexit_clear();
        let func = MbValue::from_ptr(MbObject::new_str("cleanup_fn".to_string()));
        mb_atexit_register(&[func]);
        assert_eq!(mb_atexit_ncallbacks().as_int(), Some(1));
        mb_atexit_unregister(&[func]);
        assert_eq!(mb_atexit_ncallbacks().as_int(), Some(0));
        mb_atexit_clear();
    }

    #[test]
    fn test_run_exitfuncs_drains() {
        mb_atexit_clear();
        let func = MbValue::from_ptr(MbObject::new_str("exit_fn".to_string()));
        mb_atexit_register(&[func]);
        let result = mb_atexit_run_exitfuncs();
        assert!(result.is_none());
        // handlers cleared after running
        assert_eq!(mb_atexit_ncallbacks().as_int(), Some(0));
    }
}
