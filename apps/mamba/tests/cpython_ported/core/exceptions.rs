//! Ported from Lib/test/test_exceptions_ported.py
//! Integration tests: core/exceptions.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_exceptions_ported.py`.
#[test]
fn test_except_catches_zero_division() {
    let out = jit_capture(
        r#"def safe_div(a, b):
    try:
        return a // b
    except ZeroDivisionError:
        return -1
print(safe_div(10, 2))
print(safe_div(10, 0))
"#,
    );
    assert_output(&out, "5\n-1\n");
}

/// Ported from `Lib/test/test_exceptions_ported.py`.
#[test]
fn test_raise_value_error_with_message() {
    let out = jit_capture(
        r#"try:
    raise ValueError("bad input")
except ValueError as e:
    print("caught:", e)
"#,
    );
    assert_output(&out, "caught: bad input\n");
}

/// Ported from `Lib/test/test_exceptions_ported.py`.
#[test]
fn test_multiple_except_clauses_dispatch() {
    let out = jit_capture(
        r#"def classify(x):
    try:
        if x == 0:
            raise ZeroDivisionError("z")
        elif x < 0:
            raise ValueError("neg")
        return "ok"
    except ZeroDivisionError:
        return "zero"
    except ValueError:
        return "value"
print(classify(0))
print(classify(-5))
print(classify(10))
"#,
    );
    assert_output(&out, "zero\nvalue\nok\n");
}

/// Ported from `Lib/test/test_exceptions_ported.py`.
#[test]
fn test_finally_runs_on_normal_and_except_paths() {
    let out = jit_capture(
        r#"def f(x):
    try:
        if x == 0:
            raise ValueError("z")
        print(f"ok {x}")
    except ValueError:
        print(f"caught {x}")
    finally:
        print(f"final {x}")
f(5)
f(0)
"#,
    );
    assert_output(&out, "ok 5\nfinal 5\ncaught 0\nfinal 0\n");
}

/// Ported from `Lib/test/test_exceptions_ported.py`.
#[test]
fn test_safe_int_via_except() {
    let out = jit_capture(
        r#"def safe_int(s):
    try:
        return int(s)
    except ValueError:
        return -1

print(safe_int("42"))
print(safe_int("xyz"))
print(safe_int("100"))
"#,
    );
    assert_output(&out, "42\n-1\n100\n");
}

/// Ported from `Lib/test/test_exceptions_ported.py`.
#[test]
fn test_multi_except_dispatch() {
    let out = jit_capture(
        r#"def classify(v):
    try:
        if v == "zero":
            raise ZeroDivisionError()
        if v == "key":
            raise KeyError()
        return "ok"
    except ZeroDivisionError:
        return "zd"
    except KeyError:
        return "ke"

print(classify("ok"))
print(classify("zero"))
print(classify("key"))
"#,
    );
    assert_output(&out, "ok\nzd\nke\n");
}

/// Ported from `Lib/test/test_exceptions_ported.py`.
#[test]
fn test_finally_runs_with_return_in_try() {
    let out = jit_capture(
        r#"order = []
def with_fin():
    try:
        order.append("try")
        return "result"
    finally:
        order.append("fin")

r = with_fin()
print(r)
print(order)
"#,
    );
    assert_output(&out, "result\n['try', 'fin']\n");
}

/// Ported from `Lib/test/test_exceptions_ported.py`.
#[test]
fn test_assert_passes_when_truthy() {
    let out = jit_capture(
        r#"def check(x):
    assert x > 0, "must be positive"
    return x
print(check(5))
"#,
    );
    assert_output(&out, "5\n");
}

/// Ported from `Lib/test/test_exceptions_ported.py`.
#[test]
fn test_assert_raises_with_message() {
    let out = jit_capture(
        r#"def check(x):
    assert x > 0, "must be positive"
    return x
try:
    check(-1)
except AssertionError as e:
    print("assertion:", e)
"#,
    );
    assert_output(&out, "assertion: must be positive\n");
}

/// Ported from `Lib/test/test_exceptions_ported.py`.
#[test]
fn test_value_zerodivision_runtime() {
    let out = jit_capture(
        r#"try:
    x = int("abc")
except ValueError:
    print("ValueError caught")
try:
    y = 1 / 0
except ZeroDivisionError:
    print("div by zero caught")
try:
    raise RuntimeError("boom")
except RuntimeError as e:
    print("RuntimeError:", e)
"#,
    );
    assert_output(
        &out,
        "ValueError caught\ndiv by zero caught\nRuntimeError: boom\n",
    );
}

/// Ported from `Lib/test/test_exceptions_ported.py`.
#[test]
fn test_keyerror_caught_on_dict_lookup() {
    let out = jit_capture(
        r#"d = {"a": 1, "b": 2}
try:
    print(d["missing"])
except KeyError:
    print("KeyError caught")
print(d["a"])
"#,
    );
    assert_output(&out, "KeyError caught\n1\n");
}

/// Ported from `Lib/test/test_exceptions_ported.py`.
#[test]
fn test_raised_exception_bound_via_as() {
    let out = jit_capture(
        r#"try:
    raise ValueError("bad input")
except ValueError as e:
    print("got:", e)

x = 10
try:
    if x > 5:
        raise Exception("too big")
except Exception as e:
    print("Exception:", e)
"#,
    );
    assert_output(&out, "got: bad input\nException: too big\n");
}

