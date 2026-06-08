//! Py3.12 conformance tests for built-in type conversions (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_builtin.py — conversion
//! constructor sections):
//!   `int()`, `float()`, `str()`, `list()`, `tuple()`, `bool()`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_int_conversion_from_str_and_float() {
    let out = jit_capture(
        r#"print(int("42"))
print(int(3.7))
print(int("-10"))
"#,
    );
    assert_output(&out, "42\n3\n-10\n");
}

#[test]
fn test_float_conversion_from_str_and_int() {
    let out = jit_capture(
        r#"print(float("3.14"))
print(float(2))
print(float("-0.5"))
"#,
    );
    assert_output(&out, "3.14\n2.0\n-0.5\n");
}

#[test]
fn test_str_constructor_on_numbers() {
    let out = jit_capture(
        r#"print(str(42))
print(str(3.14))
print(str(-7))
"#,
    );
    assert_output(&out, "42\n3.14\n-7\n");
}

#[test]
fn test_list_and_tuple_constructors() {
    let out = jit_capture(
        r#"print(list("abc"))
print(tuple([1, 2, 3]))
print(list((4, 5, 6)))
"#,
    );
    assert_output(&out, "['a', 'b', 'c']\n(1, 2, 3)\n[4, 5, 6]\n");
}

#[test]
fn test_bool_constructor_truthiness() {
    let out = jit_capture(
        r#"print(bool(0))
print(bool(1))
print(bool(""))
print(bool("x"))
print(bool([]))
print(bool([0]))
"#,
    );
    assert_output(&out, "False\nTrue\nFalse\nTrue\nFalse\nTrue\n");
}
