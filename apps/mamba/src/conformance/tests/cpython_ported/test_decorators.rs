//! Py3.12 conformance tests for function decorators (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_decorators.py):
//!   single decorator, stacked decorators, parametrized decorator,
//!   decorator returning value transformation.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_single_decorator_doubles_return() {
    let out = jit_capture(
        r#"def double(fn):
    def wrapper(x):
        return fn(x) * 2
    return wrapper

@double
def inc(x):
    return x + 1

print(inc(5))
"#,
    );
    assert_output(&out, "12\n");
}

#[test]
fn test_decorator_logs_then_calls() {
    let out = jit_capture(
        r#"def trace(fn):
    def wrap(x):
        print("enter")
        result = fn(x)
        print("exit")
        return result
    return wrap

@trace
def square(x):
    return x * x

print(square(4))
"#,
    );
    assert_output(&out, "enter\nexit\n16\n");
}

#[test]
fn test_parametrized_decorator_repeats() {
    let out = jit_capture(
        r#"def repeat(n):
    def deco(fn):
        def wrap(*args):
            results = []
            for _ in range(n):
                results.append(fn(*args))
            return results
        return wrap
    return deco

@repeat(3)
def hi():
    return "hi"

print(hi())
"#,
    );
    assert_output(&out, "['hi', 'hi', 'hi']\n");
}

#[test]
fn test_stacked_decorators_apply_inside_out() {
    let out = jit_capture(
        r#"def add_one(fn):
    def wrap(x):
        return fn(x) + 1
    return wrap

def times_two(fn):
    def wrap(x):
        return fn(x) * 2
    return wrap

@add_one
@times_two
def f(x):
    return x + 10

print(f(5))
"#,
    );
    assert_output(&out, "31\n");
}
