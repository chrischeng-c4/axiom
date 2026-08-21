//! Py3.12 conformance tests for `isinstance` and `issubclass`
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_isinstance.py — basic
//! and tuple-arg sections):
//!   built-in type checks, tuple-of-types syntax, `bool` is `int`,
//!   and user-class `isinstance` plus `issubclass`.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_isinstance_on_builtin_types() {
    let out = jit_capture(
        r#"print(isinstance(1, int))
print(isinstance(1.5, float))
print(isinstance("hi", str))
print(isinstance([1, 2], list))
print(isinstance({1: 2}, dict))
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\nTrue\nTrue\n");
}

#[test]
fn test_isinstance_with_tuple_of_types_and_bool() {
    let out = jit_capture(
        r#"print(isinstance(1, (int, float)))
print(isinstance("hi", (int, float)))
print(isinstance(True, bool))
print(isinstance(True, int))
"#,
    );
    assert_output(&out, "True\nFalse\nTrue\nTrue\n");
}

#[test]
fn test_isinstance_and_issubclass_with_user_classes() {
    let out = jit_capture(
        r#"class Animal:
    pass
class Dog(Animal):
    pass
d = Dog()
print(isinstance(d, Dog))
print(isinstance(d, Animal))
print(issubclass(Dog, Animal))
print(issubclass(Animal, Dog))
"#,
    );
    assert_output(&out, "True\nTrue\nTrue\nFalse\n");
}
