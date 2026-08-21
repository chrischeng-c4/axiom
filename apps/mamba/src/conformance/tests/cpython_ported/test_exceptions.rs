//! Py3.12 conformance tests for exception handling (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_exceptions.py):
//!   single except, multiple except clauses, raise with message,
//!   try/finally print order.
//!
//! Avoids combining `return` inside an except block with a `finally`
//! clause; that pattern surfaces a separate finally-elision gap that is
//! tracked outside this module.
//!
//! @issue #759

use super::{assert_output, jit_capture};

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
