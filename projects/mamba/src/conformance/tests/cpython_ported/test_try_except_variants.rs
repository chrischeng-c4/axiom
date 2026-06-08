//! Py3.12 conformance tests for `try`/`except`/`finally` variants
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_exceptions.py —
//! exception-handling variants): single-except value conversion,
//! multi-except dispatch, and `finally` execution order with a
//! `return` inside `try`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

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
