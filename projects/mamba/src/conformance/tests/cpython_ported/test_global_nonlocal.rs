//! Py3.12 conformance tests for `global` and `nonlocal` declarations
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_scope.py — global /
//! nonlocal sections):
//!   `global` lets a function rebind a module-level name; `nonlocal`
//!   lets an inner function rebind a name in the enclosing scope.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_global_rebinds_module_variable() {
    let out = jit_capture(
        r#"counter = 0
def inc():
    global counter
    counter += 1
inc()
inc()
inc()
print(counter)
"#,
    );
    assert_output(&out, "3\n");
}

#[test]
fn test_nonlocal_rebinds_enclosing_local() {
    let out = jit_capture(
        r#"def outer():
    x = 10
    def inner():
        nonlocal x
        x = 20
    inner()
    return x
print(outer())
"#,
    );
    assert_output(&out, "20\n");
}

#[test]
fn test_global_seen_across_multiple_functions() {
    let out = jit_capture(
        r#"total = 0
def add(n):
    global total
    total += n
def show():
    print(total)
add(5)
add(7)
show()
"#,
    );
    assert_output(&out, "12\n");
}
