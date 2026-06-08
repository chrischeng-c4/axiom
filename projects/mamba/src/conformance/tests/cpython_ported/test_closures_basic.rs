//! Py3.12 conformance tests for closure-based function values
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_funcattrs.py —
//! closure sections): list-cell counter, function-wrapping decorator
//! pattern, and a closure that captures a function and returns a
//! transformed result.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_closure_counter_via_list_cell() {
    let out = jit_capture(
        r#"def make_counter():
    n = [0]
    def inc():
        n[0] = n[0] + 1
        return n[0]
    return inc

c = make_counter()
print(c())
print(c())
print(c())
"#,
    );
    assert_output(&out, "1\n2\n3\n");
}

#[test]
fn test_function_wrapping_decorator_style() {
    let out = jit_capture(
        r#"def double(f):
    def wrapped(x):
        return f(x) * 2
    return wrapped

def add_one(x):
    return x + 1

d = double(add_one)
print(d(5))
print(d(10))
print(d(0))
"#,
    );
    assert_output(&out, "12\n22\n2\n");
}

#[test]
fn test_decorator_chain_three_deep() {
    let out = jit_capture(
        r#"def add_one(x):
    return x + 1

def double(f):
    def wrapped(x):
        return f(x) * 2
    return wrapped

def negate(f):
    def wrapped(x):
        return -f(x)
    return wrapped

d = double(add_one)
nd = negate(d)
print(d(3))
print(nd(3))
print(nd(7))
"#,
    );
    assert_output(&out, "8\n-8\n-16\n");
}
