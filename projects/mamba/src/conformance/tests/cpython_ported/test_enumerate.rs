//! Py3.12 conformance tests for the `enumerate` built-in (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_enumerate.py):
//!   default-start iteration, `start=` kwarg, list constructor produces
//!   pair tuples.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_enumerate_default_start_zero() {
    let out = jit_capture(
        r#"for i, c in enumerate(['a', 'b', 'c']):
    print(i, c)
"#,
    );
    assert_output(&out, "0 a\n1 b\n2 c\n");
}

#[test]
fn test_enumerate_with_start_kwarg() {
    let out = jit_capture(
        r#"for i, c in enumerate(['x', 'y'], start=10):
    print(i, c)
"#,
    );
    assert_output(&out, "10 x\n11 y\n");
}

#[test]
fn test_enumerate_to_list_of_pairs() {
    let out = jit_capture(
        r#"print(list(enumerate(['p', 'q'])))
"#,
    );
    assert_output(&out, "[(0, 'p'), (1, 'q')]\n");
}
