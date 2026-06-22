//! Py3.12 conformance tests for common exception classes and `as`
//! binding (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_exceptions.py —
//! built-in exception sections):
//!   catching `ValueError`, `ZeroDivisionError`, `KeyError`,
//!   `RuntimeError`, and binding the exception value via `as e`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

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
