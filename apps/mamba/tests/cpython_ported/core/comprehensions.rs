//! Ported from Lib/test/test_comprehensions_ported.py
//! Integration tests: core/comprehensions.rs

use super::super::harness::*;

/// Ported from `Lib/test/test_comprehensions_ported.py`.
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

/// Ported from `Lib/test/test_comprehensions_ported.py`.
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

/// Ported from `Lib/test/test_comprehensions_ported.py`.
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

/// Ported from `Lib/test/test_comprehensions_ported.py`.
#[test]
fn test_list_comprehension_squares() {
    let out = jit_capture(
        r#"print([x * x for x in range(5)])
"#,
    );
    assert_output(&out, "[0, 1, 4, 9, 16]\n");
}

/// Ported from `Lib/test/test_comprehensions_ported.py`.
#[test]
fn test_list_comprehension_with_filter() {
    let out = jit_capture(
        r#"print([x for x in range(10) if x % 2 == 0])
"#,
    );
    assert_output(&out, "[0, 2, 4, 6, 8]\n");
}

/// Ported from `Lib/test/test_comprehensions_ported.py`.
#[test]
fn test_dict_comprehension_squares() {
    let out = jit_capture(
        r#"print({x: x * x for x in range(4)})
"#,
    );
    assert_output(&out, "{0: 0, 1: 1, 2: 4, 3: 9}\n");
}

/// Ported from `Lib/test/test_comprehensions_ported.py`.
#[test]
fn test_set_comprehension_mod() {
    let out = jit_capture(
        r#"print({x % 3 for x in range(10)})
"#,
    );
    assert_output(&out, "{0, 1, 2}\n");
}

/// Ported from `Lib/test/test_comprehensions_ported.py`.
#[test]
fn test_generator_expression_with_sum() {
    let out = jit_capture(
        r#"print(sum(x * x for x in range(5)))
"#,
    );
    assert_output(&out, "30\n");
}

