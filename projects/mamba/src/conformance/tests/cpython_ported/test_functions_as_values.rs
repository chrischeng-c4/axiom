//! Py3.12 conformance tests for first-class function values
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_funcattrs.py —
//! function-object-as-value sections):
//!   storing functions in a list and iterating them, passing functions
//!   as parameters, and returning a function from a function.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_function_list_iteration() {
    let out = jit_capture(
        r#"def add(a, b):
    return a + b

def mul(a, b):
    return a * b

ops = [add, mul]
for op in ops:
    print(op(3, 4))
"#,
    );
    assert_output(&out, "7\n12\n");
}

#[test]
fn test_function_passed_as_argument() {
    let out = jit_capture(
        r#"def add(a, b):
    return a + b

def mul(a, b):
    return a * b

def apply(f, x, y):
    return f(x, y)

print(apply(add, 10, 20))
print(apply(mul, 5, 6))
"#,
    );
    assert_output(&out, "30\n30\n");
}

#[test]
fn test_function_dispatch_via_dict() {
    let out = jit_capture(
        r#"def add(a, b):
    return a + b

def sub(a, b):
    return a - b

def mul(a, b):
    return a * b

ops = {"+": add, "-": sub, "*": mul}
print(ops["+"](7, 3))
print(ops["-"](7, 3))
print(ops["*"](7, 3))
"#,
    );
    assert_output(&out, "10\n4\n21\n");
}
