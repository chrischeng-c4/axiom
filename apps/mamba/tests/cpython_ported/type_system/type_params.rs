//! Ported from Lib/test/test_type_params_ported.py
//! Integration tests: type_system/type_params.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_type_params_ported.py`.
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

/// Ported from `Lib/test/test_type_params_ported.py`.
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

/// Ported from `Lib/test/test_type_params_ported.py`.
#[test]
fn test_bool_is_instance_of_int() {
    let out = jit_capture(
        r#"print(isinstance(True, int))
print(isinstance(False, int))
"#,
    );
    assert_output(&out, "True\nTrue\n");
}

/// Ported from `Lib/test/test_type_params_ported.py`.
#[test]
fn test_function_returning_list() {
    let out = jit_capture(
        r#"def make_list(n):
    return [i for i in range(n)]

print(make_list(0))
print(make_list(4))
print(make_list(1))
"#,
    );
    assert_output(&out, "[]\n[0, 1, 2, 3]\n[0]\n");
}

/// Ported from `Lib/test/test_type_params_ported.py`.
#[test]
fn test_function_returning_tuple() {
    let out = jit_capture(
        r#"def make_pair(a, b):
    return (a, b)

def make_triple(a, b, c):
    return (a, b, c)

print(make_pair(1, 2))
print(make_triple("x", "y", "z"))
"#,
    );
    assert_output(&out, "(1, 2)\n('x', 'y', 'z')\n");
}

/// Ported from `Lib/test/test_type_params_ported.py`.
#[test]
fn test_recursive_factorial() {
    let out = jit_capture(
        r#"def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

print(factorial(0))
print(factorial(1))
print(factorial(5))
print(factorial(10))
"#,
    );
    assert_output(&out, "1\n1\n120\n3628800\n");
}

/// Ported from `Lib/test/test_type_params_ported.py`.
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

/// Ported from `Lib/test/test_type_params_ported.py`.
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

/// Ported from `Lib/test/test_type_params_ported.py`.
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

