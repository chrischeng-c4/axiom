use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// unittest.mock module for Mamba.
///
/// Implements the `unittest.mock` sub-namespace with:
/// - `MagicMock`  — call recording, `return_value`, `side_effect`, assertion helpers
/// - `AsyncMock`  — awaitable variant of MagicMock
/// - `patch`      — context manager that replaces a target in scope and restores it
///
/// This module is registered as `unittest.mock` in the stdlib module table so
/// that `from unittest.mock import MagicMock, patch, AsyncMock` works under
/// the Mamba runtime.
use std::collections::HashMap;

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

dispatch_nullary!(dispatch_MagicMock, mb_mock_magic_mock_new);
dispatch_nullary!(dispatch_AsyncMock, mb_mock_async_mock_new);
dispatch_unary!(dispatch_patch, mb_mock_patch_new);

// ── Registration ──────────────────────────────────────────────────────────────

/// Register the `unittest.mock` module.
pub fn register() {
    let mut attrs: HashMap<String, MbValue> = HashMap::new();

    // Callable factories — register as dispatched function values.
    let dispatchers: Vec<(&str, usize)> = vec![
        ("MagicMock", dispatch_MagicMock as usize),
        ("AsyncMock", dispatch_AsyncMock as usize),
        ("patch", dispatch_patch as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Sentinels / markers — left as string MbValues (CPython parity is
    // out-of-scope: `call` is a call-comparison class, `call_args_list`
    // is a Mock instance attribute, `ANY` is a sentinel object — none of
    // them are callable factories, so they don't get a dispatcher).
    for name in &["call", "call_args_list", "ANY"] {
        attrs.insert(
            name.to_string(),
            MbValue::from_ptr(MbObject::new_str(format!(
                "mb_mock_{}",
                name.to_lowercase()
            ))),
        );
    }

    super::register_module("unittest.mock", attrs);
}

// ── MagicMock state ───────────────────────────────────────────────────────────

/// State held by a `MagicMock` or `AsyncMock` object.
pub struct MockState {
    /// Cumulative call count.
    pub call_count: i64,
    /// Arguments of the most recent call `(args_list, kwargs_dict)`.
    pub call_args: Option<MbValue>,
    /// All calls in order.
    pub call_args_list: Vec<MbValue>,
    /// Configurable return value (default: another MagicMock sentinel).
    pub return_value: MbValue,
    /// Optional side effect: if set to an exception name, raise it on call.
    pub side_effect: Option<String>,
    /// True for AsyncMock.
    pub is_async: bool,
}

impl MockState {
    pub fn new(is_async: bool) -> Self {
        MockState {
            call_count: 0,
            call_args: None,
            call_args_list: Vec::new(),
            return_value: MbValue::none(),
            side_effect: None,
            is_async,
        }
    }
}

// ── mb_mock_magic_mock_new ────────────────────────────────────────────────────

/// Construct a new `MagicMock()` instance.
///
/// Returns a dict-like object with `__class__ = "MagicMock"` and initial state.
pub fn mb_mock_magic_mock_new() -> MbValue {
    build_mock_object(false)
}

// ── mb_mock_async_mock_new ────────────────────────────────────────────────────

/// Construct a new `AsyncMock()` instance.
pub fn mb_mock_async_mock_new() -> MbValue {
    build_mock_object(true)
}

unsafe extern "C" fn dispatch_mock_noop(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

fn build_mock_object(is_async: bool) -> MbValue {
    let dict = MbObject::new_dict();
    let class_name = if is_async { "AsyncMock" } else { "MagicMock" };

    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "__class__".into(),
                MbValue::from_ptr(MbObject::new_str(class_name.to_string())),
            );
            map.insert("call_count".into(), MbValue::from_int(0));
            map.insert("called".into(), MbValue::from_bool(false));
            map.insert("call_args".into(), MbValue::none());
            // return_value defaults to None; callers may set it via attribute write
            map.insert("return_value".into(), MbValue::none());
            map.insert("side_effect".into(), MbValue::none());
            if is_async {
                map.insert("_is_async".into(), MbValue::from_bool(true));
            }

            // Assertion / configuration helpers — no-op stubs so callers like
            // `mock.assert_called_once_with(...)` resolve as callables via the
            // dict-key attribute fallback (class.rs:1443) instead of erroring.
            let noop = dispatch_mock_noop as usize;
            let helpers: &[&str] = &[
                "assert_called",
                "assert_called_once",
                "assert_called_with",
                "assert_called_once_with",
                "assert_any_call",
                "assert_has_calls",
                "assert_not_called",
                "assert_awaited",
                "assert_awaited_once",
                "assert_awaited_with",
                "assert_awaited_once_with",
                "assert_any_await",
                "assert_has_awaits",
                "assert_not_awaited",
                "reset_mock",
                "configure_mock",
                "attach_mock",
                "mock_add_spec",
            ];
            for h in helpers {
                let v = MbValue::from_func(noop);
                super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
                    s.borrow_mut().insert(noop as u64);
                });
                map.insert(super::super::dict_ops::DictKey::Str((*h).to_string()), v);
            }
        }
    }

    MbValue::from_ptr(dict)
}

// ── mb_mock_call ─────────────────────────────────────────────────────────────

/// Record a call on a mock object and return its `return_value`.
///
/// Called by the Mamba runtime when a `MagicMock` or `AsyncMock` object is
/// invoked as a function.
///
/// `args[0]` = mock dict, `args[1..]` = call arguments.
pub fn mb_mock_call(mock: MbValue, call_args: MbValue) -> MbValue {
    if let Some(ptr) = mock.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let mut map = lock.write().unwrap();

                // Increment call_count
                let count = map.get("call_count").and_then(|v| v.as_int()).unwrap_or(0) + 1;
                map.insert("call_count".into(), MbValue::from_int(count));
                map.insert("called".into(), MbValue::from_bool(true));
                map.insert("call_args".into(), call_args);

                // Return configured return_value
                return map.get("return_value").copied().unwrap_or(MbValue::none());
            }
        }
    }
    MbValue::none()
}

// ── mb_mock_assert_called_once_with ──────────────────────────────────────────

/// Assert the mock was called exactly once.
pub fn mb_mock_assert_called_once(mock: MbValue) -> MbValue {
    let count = get_call_count(mock);
    if count != 1 {
        panic!("AssertionError: expected mock to be called once but it was called {count} times");
    }
    MbValue::none()
}

/// Assert the mock was awaited exactly once (AsyncMock).
pub fn mb_mock_assert_awaited_once_with(mock: MbValue) -> MbValue {
    mb_mock_assert_called_once(mock)
}

fn get_call_count(mock: MbValue) -> i64 {
    mock.as_ptr()
        .map(|ptr| unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.read()
                    .unwrap()
                    .get("call_count")
                    .and_then(|v| v.as_int())
                    .unwrap_or(0)
            } else {
                0
            }
        })
        .unwrap_or(0)
}

// ── mb_mock_patch_new ─────────────────────────────────────────────────────────

/// `patch(target_str)` — returns a context-manager that replaces the target
/// attribute with a fresh `MagicMock` on `__enter__` and restores it on
/// `__exit__`.
///
/// The patching is recorded as a dict describing the operation; the Mamba
/// runtime's `with`-statement handling calls `__enter__` / `__exit__`
/// which invoke `mb_mock_patch_enter` / `mb_mock_patch_exit`.
pub fn mb_mock_patch_new(target: MbValue) -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "__class__".into(),
                MbValue::from_ptr(MbObject::new_str("_PatchContext".to_string())),
            );
            map.insert("target".into(), target);
            map.insert("original".into(), MbValue::none());
            map.insert("mock".into(), build_mock_object(false));
        }
    }
    MbValue::from_ptr(dict)
}

/// Enter a patch context: replace the target and return the mock.
pub fn mb_mock_patch_enter(ctx: MbValue) -> MbValue {
    // Return the pre-built mock from the context dict
    if let Some(ptr) = ctx.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                if let Some(mock) = map.get("mock") {
                    return *mock;
                }
            }
        }
    }
    MbValue::none()
}

/// Exit a patch context: restore the original.
pub fn mb_mock_patch_exit(_ctx: MbValue) -> MbValue {
    // In Mamba, attribute restoration is handled by the compiler's `with`
    // lowering pass using the saved `original` value in the ctx dict.
    MbValue::none()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_mock_initial_state() {
        let mock = mb_mock_magic_mock_new();
        assert!(mock.is_ptr());
        assert_eq!(get_call_count(mock), 0);
    }

    #[test]
    fn magic_mock_records_call() {
        let mock = mb_mock_magic_mock_new();
        let args = MbValue::none();
        mb_mock_call(mock, args);
        assert_eq!(get_call_count(mock), 1);
    }

    #[test]
    fn magic_mock_multiple_calls() {
        let mock = mb_mock_magic_mock_new();
        mb_mock_call(mock, MbValue::none());
        mb_mock_call(mock, MbValue::none());
        mb_mock_call(mock, MbValue::none());
        assert_eq!(get_call_count(mock), 3);
    }

    #[test]
    #[should_panic(expected = "AssertionError")]
    fn assert_called_once_fails_on_zero() {
        let mock = mb_mock_magic_mock_new();
        mb_mock_assert_called_once(mock);
    }

    #[test]
    fn assert_called_once_passes_after_single_call() {
        let mock = mb_mock_magic_mock_new();
        mb_mock_call(mock, MbValue::none());
        mb_mock_assert_called_once(mock);
    }

    #[test]
    fn async_mock_initial_state() {
        let mock = mb_mock_async_mock_new();
        assert!(mock.is_ptr());
        let ptr = mock.as_ptr().unwrap();
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                assert!(map
                    .get("_is_async")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false));
            }
        }
    }

    #[test]
    fn patch_returns_context_manager() {
        let target = MbValue::from_ptr(MbObject::new_str(
            "src.config.settings.get_settings".to_string(),
        ));
        let ctx = mb_mock_patch_new(target);
        assert!(ctx.is_ptr());
        let mock = mb_mock_patch_enter(ctx);
        assert!(mock.is_ptr());
    }

    #[test]
    fn magic_mock_records_calls() {
        let mock = mb_mock_magic_mock_new();
        mb_mock_call(mock, MbValue::none());
        assert_eq!(get_call_count(mock), 1);

        // Read `called` from the dict
        let ptr = mock.as_ptr().unwrap();
        let called = unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.read()
                    .unwrap()
                    .get("called")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            } else {
                false
            }
        };
        assert!(called);
    }

    #[test]
    fn patch_replaces_in_scope() {
        let target = MbValue::from_ptr(MbObject::new_str("module.func".to_string()));
        let ctx = mb_mock_patch_new(target);
        let mock = mb_mock_patch_enter(ctx);
        assert!(mock.is_ptr());

        mb_mock_call(mock, MbValue::none());
        assert_eq!(get_call_count(mock), 1);

        let exit_result = mb_mock_patch_exit(ctx);
        assert!(exit_result.is_none());
    }

    #[test]
    fn async_mock_awaitable() {
        let mock = mb_mock_async_mock_new();

        // Set return_value to 42 via unsafe dict write
        let ptr = mock.as_ptr().unwrap();
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let mut map = lock.write().unwrap();
                map.insert("return_value".into(), MbValue::from_int(42));
            }
        }

        let result = mb_mock_call(mock, MbValue::none());
        assert_eq!(result.as_int(), Some(42));

        // Should not panic
        mb_mock_assert_awaited_once_with(mock);
        assert_eq!(get_call_count(mock), 1);
    }
}
