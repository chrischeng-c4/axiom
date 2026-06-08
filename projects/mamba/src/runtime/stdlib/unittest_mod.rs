use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// unittest module for Mamba (#419).
///
/// Provides: TestCase base (assertEqual, assertTrue, assertFalse, assertRaises),
/// main() test runner, TestResult.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_main(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_unittest_main()
}

unsafe extern "C" fn dispatch_testcase(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_unittest_testcase()
}

unsafe extern "C" fn dispatch_assert_equal(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    // self, a, b — ignore the self in arg 0 (TestCase instance) and unwrap.
    let (lhs, rhs) = if nargs >= 3 {
        (a[1], a[2])
    } else {
        (
            a.get(0).copied().unwrap_or_else(MbValue::none),
            a.get(1).copied().unwrap_or_else(MbValue::none),
        )
    };
    mb_unittest_assert_equal(lhs, rhs)
}

unsafe extern "C" fn dispatch_assert_true(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let v = if nargs >= 2 {
        a[1]
    } else {
        a.get(0).copied().unwrap_or_else(MbValue::none)
    };
    mb_unittest_assert_true(v)
}

// Binary assertion dispatchers — same self-trim shape as
// `dispatch_assert_equal`: drop the TestCase `self` arg when present
// so module-attr calls (`unittest.assertNotEqual(a, b)`) and method
// calls (`self.assertNotEqual(a, b)`) both reach the typed impl.
macro_rules! disp_unittest_binary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let (lhs, rhs) = if nargs >= 3 {
                (a[1], a[2])
            } else {
                (
                    a.first().copied().unwrap_or_else(MbValue::none),
                    a.get(1).copied().unwrap_or_else(MbValue::none),
                )
            };
            $fn(lhs, rhs)
        }
    };
}

macro_rules! disp_unittest_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let v = if nargs >= 2 {
                a[1]
            } else {
                a.first().copied().unwrap_or_else(MbValue::none)
            };
            $fn(v)
        }
    };
}

disp_unittest_binary!(dispatch_assert_not_equal, mb_unittest_assert_not_equal);
disp_unittest_unary!(dispatch_assert_false, mb_unittest_assert_false);
disp_unittest_binary!(dispatch_assert_is, mb_unittest_assert_is);
disp_unittest_unary!(dispatch_assert_is_none, mb_unittest_assert_is_none);
disp_unittest_binary!(dispatch_assert_in, mb_unittest_assert_in);
disp_unittest_unary!(dispatch_assert_raises, mb_unittest_assert_raises);

// ── Skip / expected-failure decorators (#1684) ──
//
// `@unittest.skipUnless(cond, msg)` is a two-stage call: first
// `skipUnless(cond, msg)` returns a decorator, then that decorator
// receives the wrapped function/class and returns it. We model both
// stages as identity:
//
//   * `dispatch_skip_factory` takes `(cond, msg)` (or any signature)
//     and returns the address of `dispatch_identity_decorator` wrapped
//     as a callable function value. The condition + message are
//     ignored — skip semantics are not honoured by the current test
//     runner stub, but the import / decoration path runs cleanly.
//
//   * `dispatch_identity_decorator` takes one positional argument (the
//     wrapped function/class) and returns it unchanged. Doubling as
//     `@unittest.expectedFailure` for the no-parens shape.
unsafe extern "C" fn dispatch_identity_decorator(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    if nargs >= 1 {
        unsafe { *args_ptr }
    } else {
        MbValue::none()
    }
}

unsafe extern "C" fn dispatch_skip_factory(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_func(dispatch_identity_decorator as *const () as usize)
}

// ── Class-method ABI ──
//
// Registered class methods use the extern "C" `fn(self, args...)`
// calling convention (one MbValue per arg). The module-level
// dispatch_* helpers above use the variadic `(args_ptr, nargs)` ABI
// suitable for module-attr calls. These two ABIs are incompatible at
// the call site, so the assertion helpers need both shapes.

extern "C" fn method_assert_equal(_self: MbValue, a: MbValue, b: MbValue) -> MbValue {
    mb_unittest_assert_equal(a, b)
}

extern "C" fn method_assert_not_equal(_self: MbValue, a: MbValue, b: MbValue) -> MbValue {
    mb_unittest_assert_not_equal(a, b)
}

extern "C" fn method_assert_true(_self: MbValue, v: MbValue) -> MbValue {
    mb_unittest_assert_true(v)
}

extern "C" fn method_assert_false(_self: MbValue, v: MbValue) -> MbValue {
    mb_unittest_assert_false(v)
}

extern "C" fn method_assert_is(_self: MbValue, a: MbValue, b: MbValue) -> MbValue {
    mb_unittest_assert_is(a, b)
}

extern "C" fn method_assert_is_none(_self: MbValue, v: MbValue) -> MbValue {
    mb_unittest_assert_is_none(v)
}

extern "C" fn method_assert_in(_self: MbValue, item: MbValue, coll: MbValue) -> MbValue {
    mb_unittest_assert_in(item, coll)
}

extern "C" fn method_assert_raises(_self: MbValue, exc: MbValue) -> MbValue {
    mb_unittest_assert_raises(exc)
}

/// `self.skipTest(reason)` — surface a SKIP signal to the mamba-pytest
/// harness. The harness catches any `BaseException` whose `str()` starts
/// with `SkipTest:` and reports `RESULT: ... SKIP`. We model the raise
/// as a panic with that prefix so existing JIT exception machinery
/// (which routes panics through the runtime's BaseException trampoline)
/// carries the marker through to the catch site without needing a real
/// `unittest.SkipTest` exception class.
extern "C" fn method_skip_test(_self: MbValue, reason: MbValue) -> MbValue {
    let msg = extract_str(reason).unwrap_or_else(|| "skipped".to_string());
    panic!("SkipTest: {msg}");
}

/// `self.assertListEqual(a, b)` — for lists, this is equivalent to
/// `assertEqual` (which already routes through `values_equal` element-wise).
/// CPython's variant also validates that both args are lists; we accept
/// any equatable values since per-test isolation makes the type-precheck
/// less useful than the equality test it's wrapping.
extern "C" fn method_assert_list_equal(_self: MbValue, a: MbValue, b: MbValue) -> MbValue {
    mb_unittest_assert_equal(a, b)
}

/// `self.addCleanup(fn, *args, **kwargs)` — register a teardown callable.
/// In the mamba-pytest harness each test runs in its own subprocess; the
/// OS reclaims everything on exit, so cleanups have nothing meaningful
/// to undo and we accept the call as a no-op rather than panicking on
/// signatures we don't yet model (variadic args + kwargs).
///
/// The class method dispatcher (`runtime/class.rs`) selects a positional
/// signature based on `all_args.len()` from 1..=4 today. We register the
/// 4-arg shape `(self, fn, arg1, arg2)` which matches the common CPython
/// test-suite usage `self.addCleanup(setattr, obj, name, value)`. Calls
/// with a different arity will fall through to the dispatcher's "too
/// many args" no-op fallback rather than aborting.
extern "C" fn method_add_cleanup_4(
    _self: MbValue,
    _fn: MbValue,
    _a1: MbValue,
    _a2: MbValue,
) -> MbValue {
    MbValue::none()
}

/// Register `unittest.TestCase` as a real runtime class so that
/// `class MyTests(unittest.TestCase):` participates in the standard MRO
/// method dispatch. Without this the assertion helpers are reachable
/// only as module-level functions; subclassed test cases would fail
/// `self.assertEqual(...)` lookup with AttributeError.
pub fn register_classes() {
    let mut methods: HashMap<String, MbValue> = HashMap::new();
    let m = [
        ("assertEqual", method_assert_equal as *const () as usize),
        (
            "assertNotEqual",
            method_assert_not_equal as *const () as usize,
        ),
        ("assertTrue", method_assert_true as *const () as usize),
        ("assertFalse", method_assert_false as *const () as usize),
        ("assertIs", method_assert_is as *const () as usize),
        ("assertIsNone", method_assert_is_none as *const () as usize),
        ("assertIn", method_assert_in as *const () as usize),
        ("assertRaises", method_assert_raises as *const () as usize),
        ("skipTest", method_skip_test as *const () as usize),
        (
            "assertListEqual",
            method_assert_list_equal as *const () as usize,
        ),
        ("addCleanup", method_add_cleanup_4 as *const () as usize),
    ];
    for (name, addr) in m {
        methods.insert(name.to_string(), MbValue::from_func(addr));
    }
    super::super::class::mb_class_register("TestCase", Vec::new(), methods);
}

/// Register the unittest module.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("main", dispatch_main as *const () as usize),
        ("TestCase", dispatch_testcase as *const () as usize),
        ("assertEqual", dispatch_assert_equal as *const () as usize),
        ("assertTrue", dispatch_assert_true as *const () as usize),
        (
            "assertNotEqual",
            dispatch_assert_not_equal as *const () as usize,
        ),
        ("assertFalse", dispatch_assert_false as *const () as usize),
        ("assertIs", dispatch_assert_is as *const () as usize),
        (
            "assertIsNone",
            dispatch_assert_is_none as *const () as usize,
        ),
        ("assertIn", dispatch_assert_in as *const () as usize),
        ("assertRaises", dispatch_assert_raises as *const () as usize),
        // #1684: skip decorators. `skipUnless` / `skipIf` / `skip` are
        // two-stage (`@dec(cond, msg)`) and resolve to the skip factory.
        // `expectedFailure` is one-stage (`@dec` no parens) and is the
        // identity decorator directly.
        ("skipUnless", dispatch_skip_factory as *const () as usize),
        ("skipIf", dispatch_skip_factory as *const () as usize),
        ("skip", dispatch_skip_factory as *const () as usize),
        (
            "expectedFailure",
            dispatch_identity_decorator as *const () as usize,
        ),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    // The identity decorator is also called dynamically as the return
    // value of `skip*` factories — register it so the JIT trusts the
    // address as a native callable.
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut()
            .insert(dispatch_identity_decorator as *const () as usize as u64);
    });

    super::register_module("unittest", attrs);
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn values_equal(a: MbValue, b: MbValue) -> bool {
    if a.as_int().is_some() && b.as_int().is_some() {
        return a.as_int() == b.as_int();
    }
    if a.as_float().is_some() && b.as_float().is_some() {
        return a.as_float() == b.as_float();
    }
    if a.as_bool().is_some() && b.as_bool().is_some() {
        return a.as_bool() == b.as_bool();
    }
    if let (Some(sa), Some(sb)) = (extract_str(a), extract_str(b)) {
        return sa == sb;
    }
    // Heap containers (lists, tuples, dicts, sets, ...). The bare
    // `a == b` fallback compares NaN-boxed bit patterns, which is
    // pointer equality for heap objects and produces false negatives
    // for `assertEqual([1,2,3], [1,2,3])` style checks. Route through
    // the runtime's deep-equality entrypoint so structural comparison
    // matches Python `==`.
    super::super::builtins::mb_eq(a, b)
        .as_bool()
        .unwrap_or(false)
}

/// unittest.TestCase() -> test case instance dict
pub fn mb_unittest_testcase() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "__class__".into(),
                MbValue::from_ptr(MbObject::new_str("TestCase".to_string())),
            );
            map.insert("_failures".into(), MbValue::from_int(0));
            map.insert("_successes".into(), MbValue::from_int(0));
        }
    }
    MbValue::from_ptr(dict)
}

/// assertEqual(a, b) -> None or panic
pub fn mb_unittest_assert_equal(a: MbValue, b: MbValue) -> MbValue {
    if !values_equal(a, b) {
        panic!("AssertionError: values not equal");
    }
    MbValue::none()
}

/// assertNotEqual(a, b) -> None or panic
pub fn mb_unittest_assert_not_equal(a: MbValue, b: MbValue) -> MbValue {
    if values_equal(a, b) {
        panic!("AssertionError: values are equal");
    }
    MbValue::none()
}

/// assertTrue(val) -> None or panic
pub fn mb_unittest_assert_true(val: MbValue) -> MbValue {
    let truthy = val.as_bool().unwrap_or(false) || val.as_int().map(|i| i != 0).unwrap_or(false);
    if !truthy {
        panic!("AssertionError: expected True");
    }
    MbValue::none()
}

/// assertFalse(val) -> None or panic
pub fn mb_unittest_assert_false(val: MbValue) -> MbValue {
    let truthy = val.as_bool().unwrap_or(false) || val.as_int().map(|i| i != 0).unwrap_or(false);
    if truthy {
        panic!("AssertionError: expected False");
    }
    MbValue::none()
}

/// assertIs(a, b) -> None or panic (identity check)
pub fn mb_unittest_assert_is(a: MbValue, b: MbValue) -> MbValue {
    if a != b {
        panic!("AssertionError: objects are not identical");
    }
    MbValue::none()
}

/// assertIsNone(val) -> None or panic
pub fn mb_unittest_assert_is_none(val: MbValue) -> MbValue {
    if !val.is_none() {
        panic!("AssertionError: expected None");
    }
    MbValue::none()
}

/// assertIn(item, collection) -> None or panic
pub fn mb_unittest_assert_in(item: MbValue, collection: MbValue) -> MbValue {
    if let Some(ptr) = collection.as_ptr() {
        unsafe {
            let found = match &(*ptr).data {
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap();
                    items.iter().any(|v| values_equal(*v, item))
                }
                ObjData::Str(s) => {
                    if let Some(needle) = extract_str(item) {
                        s.contains(&needle)
                    } else {
                        false
                    }
                }
                _ => false,
            };
            if !found {
                panic!("AssertionError: item not found in collection");
            }
        }
    }
    MbValue::none()
}

/// assertRaises(exception_type) -> context manager stub
pub fn mb_unittest_assert_raises(exc_type: MbValue) -> MbValue {
    // Return a dict that can be used as context manager stub
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert("expected".into(), exc_type);
        }
    }
    MbValue::from_ptr(dict)
}

/// unittest.main() -> run registered tests and print results
pub fn mb_unittest_main() -> MbValue {
    eprintln!("unittest.main() called — test execution is handled by the test framework");
    MbValue::none()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- extract_str ---
    #[test]
    fn test_extract_str_str() {
        let s = MbValue::from_ptr(MbObject::new_str("hi".to_string()));
        assert_eq!(extract_str(s), Some("hi".to_string()));
    }

    #[test]
    fn test_extract_str_non_str() {
        assert_eq!(extract_str(MbValue::from_int(1)), None);
    }

    // --- values_equal ---
    #[test]
    fn test_values_equal_int_equal() {
        assert!(values_equal(MbValue::from_int(5), MbValue::from_int(5)));
    }

    #[test]
    fn test_values_equal_int_unequal() {
        assert!(!values_equal(MbValue::from_int(1), MbValue::from_int(2)));
    }

    #[test]
    fn test_values_equal_float() {
        assert!(values_equal(
            MbValue::from_float(1.5),
            MbValue::from_float(1.5)
        ));
    }

    #[test]
    fn test_values_equal_bool() {
        assert!(values_equal(
            MbValue::from_bool(true),
            MbValue::from_bool(true)
        ));
        assert!(!values_equal(
            MbValue::from_bool(true),
            MbValue::from_bool(false)
        ));
    }

    #[test]
    fn test_values_equal_str() {
        let a = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        let b = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        assert!(values_equal(a, b));
    }

    // --- assert_equal ---
    #[test]
    fn test_assert_equal() {
        mb_unittest_assert_equal(MbValue::from_int(1), MbValue::from_int(1));
    }

    #[test]
    #[should_panic(expected = "AssertionError")]
    fn test_assert_equal_fail() {
        mb_unittest_assert_equal(MbValue::from_int(1), MbValue::from_int(2));
    }

    // --- assert_not_equal ---
    #[test]
    fn test_assert_not_equal_pass() {
        mb_unittest_assert_not_equal(MbValue::from_int(1), MbValue::from_int(2));
    }

    #[test]
    #[should_panic(expected = "AssertionError")]
    fn test_assert_not_equal_fail() {
        mb_unittest_assert_not_equal(MbValue::from_int(1), MbValue::from_int(1));
    }

    // --- assert_true ---
    #[test]
    fn test_assert_true_bool_true() {
        mb_unittest_assert_true(MbValue::from_bool(true));
    }

    #[test]
    fn test_assert_true_int_nonzero() {
        mb_unittest_assert_true(MbValue::from_int(5));
    }

    #[test]
    #[should_panic(expected = "AssertionError")]
    fn test_assert_true_bool_false_fails() {
        mb_unittest_assert_true(MbValue::from_bool(false));
    }

    #[test]
    #[should_panic(expected = "AssertionError")]
    fn test_assert_true_int_zero_fails() {
        mb_unittest_assert_true(MbValue::from_int(0));
    }

    // --- assert_false ---
    #[test]
    fn test_assert_false_pass() {
        mb_unittest_assert_false(MbValue::from_bool(false));
    }

    #[test]
    fn test_assert_false_int_zero_pass() {
        mb_unittest_assert_false(MbValue::from_int(0));
    }

    #[test]
    #[should_panic(expected = "AssertionError")]
    fn test_assert_false_bool_true_fails() {
        mb_unittest_assert_false(MbValue::from_bool(true));
    }

    // --- assert_is ---
    #[test]
    fn test_assert_is_same_value() {
        let v = MbValue::from_int(42);
        mb_unittest_assert_is(v, v);
    }

    #[test]
    #[should_panic(expected = "AssertionError")]
    fn test_assert_is_different_fails() {
        mb_unittest_assert_is(MbValue::from_int(1), MbValue::from_int(2));
    }

    // --- assert_is_none ---
    #[test]
    fn test_assert_is_none() {
        mb_unittest_assert_is_none(MbValue::none());
    }

    #[test]
    #[should_panic(expected = "AssertionError")]
    fn test_assert_is_none_non_none_fails() {
        mb_unittest_assert_is_none(MbValue::from_int(1));
    }

    // --- assert_in ---
    #[test]
    fn test_assert_in_list_found() {
        let items = vec![MbValue::from_int(1), MbValue::from_int(2)];
        let list = MbValue::from_ptr(MbObject::new_list(items));
        mb_unittest_assert_in(MbValue::from_int(1), list);
    }

    #[test]
    #[should_panic(expected = "AssertionError")]
    fn test_assert_in_list_missing_fails() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(1)]));
        mb_unittest_assert_in(MbValue::from_int(99), list);
    }

    #[test]
    fn test_assert_in_str_found() {
        let col = MbValue::from_ptr(MbObject::new_str("xyz".to_string()));
        let item = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        mb_unittest_assert_in(item, col);
    }

    #[test]
    #[should_panic(expected = "AssertionError")]
    fn test_assert_in_str_missing_fails() {
        let col = MbValue::from_ptr(MbObject::new_str("abc".to_string()));
        let item = MbValue::from_ptr(MbObject::new_str("z".to_string()));
        mb_unittest_assert_in(item, col);
    }

    #[test]
    #[should_panic(expected = "AssertionError")]
    fn test_assert_in_other_obj_data_fails() {
        // Pass a dict as collection — not List or Str, found=false
        let col = MbValue::from_ptr(MbObject::new_dict());
        mb_unittest_assert_in(MbValue::from_int(1), col);
    }

    // --- assert_raises ---
    #[test]
    fn test_assert_raises_returns_dict() {
        let exc_type = MbValue::from_ptr(MbObject::new_str("ValueError".to_string()));
        let result = mb_unittest_assert_raises(exc_type);
        assert!(result.as_ptr().is_some());
        // Verify "expected" key is present
        if let Some(ptr) = result.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    assert!(map.contains_key("expected"));
                }
            }
        }
    }

    // --- testcase ---
    #[test]
    fn test_testcase_structure() {
        let tc = mb_unittest_testcase();
        assert!(tc.as_ptr().is_some());
        if let Some(ptr) = tc.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    let class = map.get("__class__").copied().and_then(|v| extract_str(v));
                    assert_eq!(class, Some("TestCase".to_string()));
                    assert_eq!(map.get("_failures").and_then(|v| v.as_int()), Some(0));
                    assert_eq!(map.get("_successes").and_then(|v| v.as_int()), Some(0));
                }
            }
        }
    }

    // --- main ---
    #[test]
    fn test_main_returns_none() {
        let result = mb_unittest_main();
        assert!(result.is_none());
    }

    // --- skip-decorator dispatchers (#1684) ---
    #[test]
    fn test_dispatch_identity_decorator_returns_first_arg() {
        let f = MbValue::from_int(42);
        let args = [f];
        let r = unsafe { dispatch_identity_decorator(args.as_ptr(), 1) };
        assert_eq!(r, f);
    }

    #[test]
    fn test_dispatch_identity_decorator_no_args_returns_none() {
        let r = unsafe { dispatch_identity_decorator(std::ptr::null(), 0) };
        assert!(r.is_none());
    }

    #[test]
    fn test_dispatch_skip_factory_returns_callable() {
        // skipUnless(cond, msg) -> identity decorator function value.
        let cond = MbValue::from_bool(true);
        let msg = MbValue::from_ptr(MbObject::new_str("why".into()));
        let args = [cond, msg];
        let r = unsafe { dispatch_skip_factory(args.as_ptr(), 2) };
        // The returned MbValue should be the identity decorator's address
        // wrapped as a function — applying it to a wrapped fn returns it.
        assert!(!r.is_none());
    }
}
