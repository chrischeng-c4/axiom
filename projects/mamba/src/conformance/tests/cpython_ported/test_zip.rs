//! Py3.12 conformance tests for the `zip` built-in (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_zip.py):
//!   zip iterates in lockstep, builds list of tuple pairs, and truncates
//!   to the shortest input.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_zip_lockstep_iteration_prints_pairs() {
    let out = jit_capture(
        r#"a = [1, 2, 3]
b = ['x', 'y', 'z']
for i, c in zip(a, b):
    print(i, c)
"#,
    );
    assert_output(&out, "1 x\n2 y\n3 z\n");
}

#[test]
fn test_zip_to_list_of_tuples() {
    let out = jit_capture(
        r#"print(list(zip([1, 2, 3], [4, 5, 6])))
"#,
    );
    assert_output(&out, "[(1, 4), (2, 5), (3, 6)]\n");
}

#[test]
fn test_zip_truncates_to_shortest() {
    let out = jit_capture(
        r#"print(list(zip([1, 2, 3], [4, 5])))
"#,
    );
    assert_output(&out, "[(1, 4), (2, 5)]\n");
}
