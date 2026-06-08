//! Py3.12 conformance tests for nested list comprehensions
//! (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_listcomps.py —
//! nested-comprehension sections): flattening a matrix, transposing
//! it, summing the flattened values, and filtering inside a
//! nested comprehension.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_flatten_matrix_with_nested_comp() {
    let out = jit_capture(
        r#"matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
flat = [x for row in matrix for x in row]
print(flat)
"#,
    );
    assert_output(&out, "[1, 2, 3, 4, 5, 6, 7, 8, 9]\n");
}

#[test]
fn test_transpose_matrix_with_nested_comp() {
    let out = jit_capture(
        r#"matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
trans = [[row[i] for row in matrix] for i in range(3)]
print(trans)
"#,
    );
    assert_output(&out, "[[1, 4, 7], [2, 5, 8], [3, 6, 9]]\n");
}

#[test]
fn test_nested_comp_with_filter_and_sum() {
    let out = jit_capture(
        r#"matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
total = sum(x for row in matrix for x in row)
print(total)
evens = [x for row in matrix for x in row if x % 2 == 0]
print(evens)
"#,
    );
    assert_output(&out, "45\n[2, 4, 6, 8]\n");
}
