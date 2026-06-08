//! Py3.12 conformance tests for `isinstance` variants (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_isinstance.py —
//! single-type, tuple-of-types, and `bool`-as-`int` sections):
//! single-arg type checks, the tuple-of-types form, and the
//! Python rule that `bool` is a subclass of `int`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_isinstance_single_type() {
    let out = jit_capture(
        r#"print(isinstance(1, int))
print(isinstance(1, float))
print(isinstance(1.5, float))
print(isinstance("x", str))
print(isinstance([], list))
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nTrue\nTrue\n");
}

#[test]
fn test_isinstance_tuple_of_types() {
    let out = jit_capture(
        r#"print(isinstance(1, (int, float)))
print(isinstance(1.5, (int, float)))
print(isinstance("x", (int, float)))
print(isinstance(1, (str, bytes)))
print(isinstance([], (list, tuple)))
print(isinstance((1,), (list, tuple)))
"#,
    );
    assert_output(&out, "True\nTrue\nFalse\nFalse\nTrue\nTrue\n");
}

#[test]
fn test_bool_is_instance_of_int() {
    let out = jit_capture(
        r#"print(isinstance(True, int))
print(isinstance(False, int))
"#,
    );
    assert_output(&out, "True\nTrue\n");
}
