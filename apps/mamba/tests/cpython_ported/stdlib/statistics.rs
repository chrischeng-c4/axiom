//! Ported from Lib/test/test_statistics_ported.py
//! Integration tests: stdlib/statistics.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_statistics_ported.py`.
#[test]
fn test_statistics_median_odd_length_int() {
    let out = jit_capture(
        r#"import statistics
print(statistics.median([1, 2, 3, 4, 5]))
"#,
    );
    assert_output(&out, "3\n");
}

/// Ported from `Lib/test/test_statistics_ported.py`.
#[test]
fn test_statistics_median_even_length_avg() {
    let out = jit_capture(
        r#"import statistics
print(statistics.median([1, 3, 5, 7]))
"#,
    );
    assert_output(&out, "4.0\n");
}

/// Ported from `Lib/test/test_statistics_ported.py`.
#[test]
fn test_statistics_median_single_element() {
    let out = jit_capture(
        r#"import statistics
print(statistics.median([42]))
"#,
    );
    assert_output(&out, "42\n");
}

/// Ported from `Lib/test/test_statistics_ported.py`.
#[test]
fn test_statistics_median_two_elements() {
    let out = jit_capture(
        r#"import statistics
print(statistics.median([10, 20]))
"#,
    );
    assert_output(&out, "15.0\n");
}

