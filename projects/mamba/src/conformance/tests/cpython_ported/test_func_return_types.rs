//! Py3.12 conformance tests for functions returning composite values
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_funcattrs.py /
//! test_grammar.py — function return sections):
//!   functions that return `list`/`tuple` values and a recursive
//!   `factorial` returning `int`. Mamba's dict-comprehension over an
//!   integer-literal zero is currently a separate bug — this module
//!   does not exercise that path.
//!
//! @issue #759

use super::{assert_output, jit_capture};

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
