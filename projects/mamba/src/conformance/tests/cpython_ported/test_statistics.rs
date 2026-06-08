//! Py3.12 conformance tests for the `statistics` module (issue #759).
//!
//! Ported from CPython 3.12.0 tag (Lib/test/test_statistics.py):
//!   median (odd and even cardinality).
//!
//! `statistics.mean` is intentionally excluded — mamba returns `3.0`
//! (float) for `mean([1, 2, 3, 4, 5])` whereas CPython preserves the
//! integer result `3`. Deferred as a separate type-coercion gap.
//!
//! @issue #759

use super::{assert_output, jit_capture};

#[test]
fn test_statistics_median_odd_length_int() {
    let out = jit_capture(
        r#"import statistics
print(statistics.median([1, 2, 3, 4, 5]))
"#,
    );
    assert_output(&out, "3\n");
}

#[test]
fn test_statistics_median_even_length_avg() {
    let out = jit_capture(
        r#"import statistics
print(statistics.median([1, 3, 5, 7]))
"#,
    );
    assert_output(&out, "4.0\n");
}

#[test]
fn test_statistics_median_single_element() {
    let out = jit_capture(
        r#"import statistics
print(statistics.median([42]))
"#,
    );
    assert_output(&out, "42\n");
}

#[test]
fn test_statistics_median_two_elements() {
    let out = jit_capture(
        r#"import statistics
print(statistics.median([10, 20]))
"#,
    );
    assert_output(&out, "15.0\n");
}
