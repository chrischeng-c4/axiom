//! Py3.12 conformance tests for variable scopes (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_scope.py):
//!   `global` rebinds module-level name, `nonlocal` rebinds enclosing
//!   function name, closure captures by reference across multiple calls.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_global_rebinds_module_name() {
    let out = jit_capture(
        r#"count = 0
def inc():
    global count
    count += 1
inc()
inc()
inc()
print(count)
"#,
    );
    assert_output(&out, "3\n");
}

#[test]
fn test_nonlocal_rebinds_enclosing_function_name() {
    let out = jit_capture(
        r#"def outer():
    x = 10
    def inner():
        nonlocal x
        x += 5
    inner()
    inner()
    return x
print(outer())
"#,
    );
    assert_output(&out, "20\n");
}

#[test]
fn test_closure_captures_state_across_calls() {
    let out = jit_capture(
        r#"def make_counter():
    n = 0
    def step():
        nonlocal n
        n += 1
        return n
    return step

c = make_counter()
print(c())
print(c())
print(c())
"#,
    );
    assert_output(&out, "1\n2\n3\n");
}
